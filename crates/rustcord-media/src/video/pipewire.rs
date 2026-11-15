//! Reads one video node of a portal's PipeWire remote into the preview
//! queue. Runs on its own thread until the stop flag, an error or the
//! source going away; the stream, core and remote are released on return.

use std::cell::{Cell, RefCell};
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pipewire as pw;
use pw::spa;
use pw::spa::param::format::MediaSubtype;
use pw::spa::param::video::VideoFormat;
use pw::stream::{StreamFlags, StreamState};
use rustcord_i18n::{es, t};

use super::{
    convert_within, jpeg_frame, lock, CameraInfo, Keeper, Pixels, Plane, Shared, VideoKind,
};

/// How often the stop flag and the deadlines are checked.
const CHECK: Duration = Duration::from_millis(50);
/// A source that has not sent a frame by then is not going to.
const FIRST_FRAME: Duration = Duration::from_secs(15);
/// A camera sends frames all the time; a screen only when it changes.
const CAMERA_STALL: Duration = Duration::from_secs(5);

const GONE: &str = es("La fuente dejó de estar disponible.");

/// The raw pictures asked for, the first preferred (SPA choices repeat
/// their default). A camera may also send MJPEG.
const SCREEN_FORMATS: [VideoFormat; 5] = [
    VideoFormat::BGRx,
    VideoFormat::BGRx,
    VideoFormat::BGRA,
    VideoFormat::RGBx,
    VideoFormat::RGBA,
];
const CAMERA_FORMATS: [VideoFormat; 6] = [
    VideoFormat::YUY2,
    VideoFormat::YUY2,
    VideoFormat::NV12,
    VideoFormat::I420,
    VideoFormat::BGRx,
    VideoFormat::RGBx,
];

/// How a raw format is read, and its name for the diagnostic line.
fn pixels(format: VideoFormat) -> Option<(Pixels, &'static str)> {
    Some(match format {
        VideoFormat::BGRx => (Pixels::Bgr4, "BGRx"),
        VideoFormat::BGRA => (Pixels::Bgr4, "BGRA"),
        VideoFormat::RGBx => (Pixels::Rgb4, "RGBx"),
        VideoFormat::RGBA => (Pixels::Rgb4, "RGBA"),
        VideoFormat::YUY2 => (Pixels::Yuy2, "YUY2"),
        VideoFormat::NV12 => (Pixels::Nv12, "NV12"),
        VideoFormat::I420 => (Pixels::I420, "I420"),
        _ => return None,
    })
}

/// The format negotiated for the stream.
#[derive(Clone, Copy)]
struct Format {
    pixels: Pixels,
    width: u32,
    height: u32,
    /// Each buffer is one JPEG picture: a camera that only gives MJPEG.
    jpeg: bool,
    /// A screen's GPU buffers in the linear layout (`LINEAR`): DMA-BUF the
    /// source may leave unmapped, read here once mapped.
    linear: bool,
}

/// DRM's linear layout: rows one after another, readable as they are.
const LINEAR: i64 = 0;

#[derive(Default)]
struct Reader {
    format: Option<Format>,
    /// Which frames to keep, and the last one previewed while sending.
    keeper: Keeper,
    connected: bool,
    copy: Copied,
}

/// What reading a source's GPU buffers keeps from frame to frame: the rows
/// copied out of one, so nothing is allocated; the buffers as they stay
/// mapped; and what the last frame's sync calls and mapping took.
#[derive(Default)]
struct Copied {
    bytes: Vec<u8>,
    maps: Vec<Kept>,
    mapping: Duration,
}

/// One of a source's GPU buffers, mapped: the file it is (two descriptors
/// may be one buffer, and one number two buffers in turn) and the part of
/// it mapped.
struct Kept {
    file: (u64, u64),
    at: usize,
    len: usize,
    mapped: Mapped,
}

/// A source's buffers stay mapped, at most this many: mapping one again
/// for each frame faults every page of it in again (5,000 for a 2880×1800
/// screen), which a busy process pays several times over.
const KEPT: usize = 16;

/// `len` bytes of the buffer `fd` is, from `at`: its mapping among `maps`,
/// made and kept on its first frame.
fn mapping(maps: &mut Vec<Kept>, fd: std::os::fd::RawFd, at: usize, len: usize) -> Option<&Mapped> {
    // SAFETY: `fstat` fills the struct it is given, zeroed before.
    let file = unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        (libc::fstat(fd, &mut stat) == 0).then_some((stat.st_dev as u64, stat.st_ino as u64))
    }?;
    let found = maps
        .iter()
        .position(|kept| (kept.file, kept.at, kept.len) == (file, at, len));
    let index = match found {
        Some(index) => index,
        None => {
            // More than a source keeps: the oldest is one it let go of.
            if maps.len() >= KEPT {
                maps.remove(0);
            }
            maps.push(Kept {
                file,
                at,
                len,
                mapped: Mapped::new(fd, at, len)?,
            });
            maps.len() - 1
        }
    };
    Some(&maps[index].mapped)
}

pub(super) fn read(
    remote: OwnedFd,
    node: Option<u32>,
    kind: VideoKind,
    device: Option<String>,
    shared: &Arc<Shared>,
) -> Result<(), String> {
    let unavailable = |_| t("No se pudo conectar con PipeWire.").to_owned();
    pw::init();
    let mainloop = pw::main_loop::MainLoopRc::new(None).map_err(unavailable)?;
    let context = pw::context::ContextRc::new(&mainloop, None).map_err(unavailable)?;
    let core = context.connect_fd_rc(remote, None).map_err(unavailable)?;
    let failure: Rc<RefCell<Option<String>>> = Rc::default();
    let fail = {
        let (mainloop, failure) = (mainloop.clone(), failure.clone());
        move |reason: String| {
            failure.borrow_mut().get_or_insert(reason);
            mainloop.quit();
        }
    };

    let _core = core
        .add_listener_local()
        .error({
            let fail = fail.clone();
            move |id, _, _, message| {
                if id == pw::core::PW_ID_CORE {
                    fail(explain(message));
                }
            }
        })
        .register();

    // The cameras the remote shows, for the device choice; and the source
    // read going away (unplugged, or a window closed).
    let registry = core.get_registry_rc().map_err(unavailable)?;
    let cameras: Rc<RefCell<Vec<(u32, CameraInfo)>>> = Rc::default();
    let chosen: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(node));
    let _registry = registry
        .add_listener_local()
        .global({
            let (shared, cameras, chosen) = (shared.clone(), cameras.clone(), chosen.clone());
            let device = device.clone();
            move |global| {
                if kind != VideoKind::Camera || global.type_ != pw::types::ObjectType::Node {
                    return;
                }
                let Some(props) = global.props else { return };
                if props.get("media.class") != Some("Video/Source") {
                    return;
                }
                let Some(id) = props.get("node.name").filter(|name| name.len() <= 256) else {
                    return;
                };
                let name = props
                    .get("node.description")
                    .or_else(|| props.get("node.nick"))
                    .unwrap_or(id)
                    .chars()
                    .take(128)
                    .collect();
                if device.as_deref() == Some(id) {
                    chosen.set(Some(global.id));
                }
                let mut cameras = cameras.borrow_mut();
                // A node is announced again when the portal widens access.
                cameras.retain(|(node, _)| *node != global.id);
                cameras.push((
                    global.id,
                    CameraInfo {
                        id: id.to_owned(),
                        name,
                    },
                ));
                *super::lock(&shared.cameras) =
                    cameras.iter().map(|(_, camera)| camera.clone()).collect();
            }
        })
        .global_remove({
            let (shared, cameras, chosen) = (shared.clone(), cameras.clone(), chosen.clone());
            let fail = fail.clone();
            move |id| {
                let mut cameras = cameras.borrow_mut();
                let before = cameras.len();
                cameras.retain(|(node, _)| *node != id);
                if cameras.len() != before {
                    *super::lock(&shared.cameras) =
                        cameras.iter().map(|(_, camera)| camera.clone()).collect();
                }
                let last_camera = kind == VideoKind::Camera && before > 0 && cameras.is_empty();
                if chosen.get() == Some(id) || last_camera {
                    fail(t(GONE).to_owned());
                }
            }
        })
        .register();

    let mut properties = pw::properties::properties! {
        *pw::keys::MEDIA_TYPE => "Video",
        *pw::keys::MEDIA_CATEGORY => "Capture",
        *pw::keys::MEDIA_ROLE => match kind {
            VideoKind::Screen => "Screen",
            VideoKind::Camera => "Camera",
        },
    };
    if let (VideoKind::Camera, Some(device)) = (kind, &device) {
        properties.insert(*pw::keys::TARGET_OBJECT, device.as_str());
    }
    let stream =
        pw::stream::StreamBox::new(&core, "rustcord-preview", properties).map_err(unavailable)?;
    let last_frame = Rc::new(Cell::new(None::<Instant>));
    let _stream = stream
        .add_local_listener_with_user_data(Reader::default())
        .state_changed({
            let fail = fail.clone();
            move |_, reader, _, state| match state {
                StreamState::Error(message) => fail(explain(&message)),
                StreamState::Unconnected if reader.connected => fail(t(GONE).to_owned()),
                StreamState::Paused | StreamState::Streaming => reader.connected = true,
                _ => {}
            }
        })
        .param_changed({
            let shared = shared.clone();
            move |stream, reader, id, param| {
                if id != spa::param::ParamType::Format.as_raw() {
                    return;
                }
                // Other buffers come with another format.
                reader.copy.maps.clear();
                reader.format = param.and_then(|param| {
                    let (_, subtype) = spa::param::format_utils::parse_format(param).ok()?;
                    let jpeg = subtype == MediaSubtype::Mjpg;
                    // Its size and rate are read the same way for MJPEG.
                    let mut info = spa::param::video::VideoInfoRaw::new();
                    info.parse(param).ok()?;
                    let (pixels, name) = match jpeg {
                        true => (Pixels::Rgb4, "MJPG"),
                        false => pixels(info.format())?,
                    };
                    let size = info.size();
                    // The flag's constant needs a newer libspa feature: its bit.
                    let linear = info.flags().bits() & spa::sys::SPA_VIDEO_FLAG_MODIFIER != 0
                        && info.modifier() == LINEAR as u64;
                    let fraction = |rate: spa::utils::Fraction| (rate.num, rate.denom);
                    let mut stats = lock(&shared.stats);
                    stats.size = (size.width, size.height);
                    stats.format = name;
                    stats.rate = fraction(info.framerate());
                    stats.max_rate = fraction(info.max_framerate());
                    Some(Format {
                        pixels,
                        width: size.width,
                        height: size.height,
                        jpeg,
                        linear,
                    })
                });
                // A screen's GPU buffers, once their linear layout is agreed:
                // the buffers then take DMA-BUF too.
                if kind == VideoKind::Screen {
                    let dmabuf = reader.format.is_some_and(|format| format.linear);
                    if let Ok(buffers) = buffers_pod(dmabuf) {
                        if let Some(pod) = spa::pod::Pod::from_bytes(&buffers) {
                            let _ = stream.update_params(&mut [pod]);
                        }
                    }
                }
            }
        })
        .process({
            let (shared, last_frame) = (shared.clone(), last_frame.clone());
            move |stream, reader| {
                let Some(mut buffer) = stream.dequeue_buffer() else {
                    return;
                };
                let now = Instant::now();
                last_frame.set(Some(now));
                let memory = buffer.datas_mut().first_mut().map(|data| {
                    let mapped = data.data().is_some_and(|bytes| bytes.len() > 1);
                    (data.type_(), mapped)
                });
                lock(&shared.stats).memory = match memory {
                    Some((spa::buffer::DataType::DmaBuf, true)) => "dmabuf (mapped by PipeWire)",
                    Some((spa::buffer::DataType::DmaBuf, false)) => "dmabuf",
                    Some((spa::buffer::DataType::MemFd, _)) => "memfd",
                    Some((spa::buffer::DataType::MemPtr, _)) => "memptr",
                    _ => "other",
                };
                // Dropping the buffer queues it back: skipped frames cost nothing.
                let Reader {
                    format,
                    keeper,
                    copy,
                    ..
                } = reader;
                shared.deliver(keeper, now, |max| {
                    frame((*format)?, buffer.datas_mut(), max, copy)
                });
                let mapping = std::mem::take(&mut copy.mapping);
                if !mapping.is_zero() {
                    lock(&shared.stats).mapping_micros += mapping.as_micros() as u64;
                }
            }
        })
        .register()
        .map_err(unavailable)?;

    let formats = match kind {
        VideoKind::Screen => SCREEN_FORMATS.to_vec(),
        VideoKind::Camera => CAMERA_FORMATS.to_vec(),
    };
    // Asked for at least at the rate kept (30 as before, 60 for 60 fps): a
    // source that settles lower could only be sent slower.
    let fps = (1.0 / shared.capture().1.as_secs_f64())
        .round()
        .clamp(30.0, 60.0) as u32;
    let raw = format_pod(MediaSubtype::Raw, formats, fps, None)
        .map_err(|_| unavailable(pw::Error::CreationFailed))?;
    // A camera that gives only MJPEG is read too, after what it gives raw.
    let mjpg = format_pod(MediaSubtype::Mjpg, Vec::new(), fps, None)
        .map_err(|_| unavailable(pw::Error::CreationFailed))?;
    // A screen whose compositor shares only GPU buffers (niri 26.04): its
    // linear ones, after shared memory, so a source with both keeps that.
    let gpu = format_pod(
        MediaSubtype::Raw,
        SCREEN_FORMATS.to_vec(),
        fps,
        Some(LINEAR),
    )
    .map_err(|_| unavailable(pw::Error::CreationFailed))?;
    let buffers = buffers_pod(kind == VideoKind::Camera)
        .map_err(|_| unavailable(pw::Error::CreationFailed))?;
    let pods = match kind {
        VideoKind::Screen => vec![raw, gpu, buffers],
        VideoKind::Camera => vec![raw, mjpg, buffers],
    };
    let mut params = pods
        .iter()
        .map(|pod| spa::pod::Pod::from_bytes(pod))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| unavailable(pw::Error::CreationFailed))?;
    stream
        .connect(
            spa::utils::Direction::Input,
            node,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .map_err(unavailable)?;

    let started = Instant::now();
    let timer = mainloop.loop_().add_timer({
        let (mainloop, shared, fail) = (mainloop.clone(), shared.clone(), fail.clone());
        move |_| {
            let now = Instant::now();
            if shared.stopped() {
                mainloop.quit();
            } else if !shared.running() && now.duration_since(started) > FIRST_FRAME {
                fail(t("La fuente no envió imágenes.").to_owned());
            } else if kind == VideoKind::Camera
                && last_frame
                    .get()
                    .is_some_and(|last| now.duration_since(last) > CAMERA_STALL)
            {
                fail(t("La cámara dejó de enviar imágenes.").to_owned());
            }
        }
    });
    let _ = timer.update_timer(Some(CHECK), Some(CHECK));
    mainloop.run();
    let _ = stream.disconnect();
    let failure = failure.borrow_mut().take();
    match failure {
        Some(reason) => Err(reason),
        None => Ok(()),
    }
}

/// What the stream accepts: `subtype` pictures (raw ones in `formats`, the
/// first preferred, as SPA choices repeat their default), any size and rate
/// the source has, `fps` preferred; with a `modifier`, only GPU buffers in
/// that layout.
fn format_pod(
    subtype: MediaSubtype,
    formats: Vec<VideoFormat>,
    fps: u32,
    modifier: Option<i64>,
) -> Result<Vec<u8>, spa::pod::serialize::GenError> {
    use spa::param::format::{FormatProperties, MediaType};
    use spa::pod::{ChoiceValue, Property, PropertyFlags, Value};
    use spa::utils::{Choice, ChoiceEnum, ChoiceFlags, Fraction, Id, Rectangle, SpaTypes};
    let id = |key: FormatProperties, value: u32| Property {
        key: key.as_raw(),
        flags: PropertyFlags::empty(),
        value: Value::Id(Id(value)),
    };
    let mut properties = vec![
        id(FormatProperties::MediaType, MediaType::Video.as_raw()),
        id(FormatProperties::MediaSubtype, subtype.as_raw()),
    ];
    let mut formats = formats.into_iter().map(|format| Id(format.as_raw()));
    if let Some(default) = formats.next() {
        properties.push(Property {
            key: FormatProperties::VideoFormat.as_raw(),
            flags: PropertyFlags::empty(),
            value: Value::Choice(ChoiceValue::Id(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Enum {
                    default,
                    alternatives: formats.collect(),
                },
            ))),
        });
    }
    if let Some(modifier) = modifier {
        properties.push(Property {
            key: FormatProperties::VideoModifier.as_raw(),
            flags: PropertyFlags::MANDATORY,
            value: Value::Choice(ChoiceValue::Long(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Enum {
                    default: modifier,
                    alternatives: vec![modifier],
                },
            ))),
        });
    }
    properties.extend([
        Property {
            key: FormatProperties::VideoSize.as_raw(),
            flags: PropertyFlags::empty(),
            value: Value::Choice(ChoiceValue::Rectangle(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Range {
                    default: Rectangle {
                        width: 1280,
                        height: 720,
                    },
                    min: Rectangle {
                        width: 1,
                        height: 1,
                    },
                    max: Rectangle {
                        width: super::MAX_SOURCE_SIDE,
                        height: super::MAX_SOURCE_SIDE,
                    },
                },
            ))),
        },
        Property {
            key: FormatProperties::VideoFramerate.as_raw(),
            flags: PropertyFlags::empty(),
            value: Value::Choice(ChoiceValue::Fraction(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Range {
                    default: Fraction { num: fps, denom: 1 },
                    min: Fraction { num: 0, denom: 1 },
                    max: Fraction {
                        num: 1000,
                        denom: 1,
                    },
                },
            ))),
        },
    ]);
    serialize(
        SpaTypes::ObjectParamFormat,
        spa::param::ParamType::EnumFormat,
        properties,
    )
}

/// Where the stream takes its buffers: shared memory, and with `dmabuf`
/// also DMA-BUF. Count and size stay the source's. A camera's (`dmabuf`):
/// unsaid, a V4L2 camera keeps them in its own mapping, which a client in
/// another process cannot reach, and PipeWire fails the link with "use
/// input buffers: -22". A screen takes DMA-BUF only once the linear layout
/// is agreed (`Format::linear`): without a modifier it could be tiled,
/// unreadable as rows.
fn buffers_pod(dmabuf: bool) -> Result<Vec<u8>, spa::pod::serialize::GenError> {
    use spa::buffer::DataType;
    use spa::pod::{ChoiceValue, Property, PropertyFlags, Value};
    use spa::utils::{Choice, ChoiceEnum, ChoiceFlags, SpaTypes};
    let mut types = [DataType::MemPtr, DataType::MemFd].to_vec();
    if dmabuf {
        types.push(DataType::DmaBuf);
    }
    let mask = types.iter().fold(0, |mask, kind| mask | 1 << kind.as_raw());
    serialize(
        SpaTypes::ObjectParamBuffers,
        spa::param::ParamType::Buffers,
        vec![Property {
            key: spa::sys::SPA_PARAM_BUFFERS_dataType,
            flags: PropertyFlags::empty(),
            // Flags: what both ends allow, bit by bit.
            value: Value::Choice(ChoiceValue::Int(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Flags {
                    default: mask,
                    flags: Vec::new(),
                },
            ))),
        }],
    )
}

fn serialize(
    type_: spa::utils::SpaTypes,
    id: spa::param::ParamType,
    properties: Vec<spa::pod::Property>,
) -> Result<Vec<u8>, spa::pod::serialize::GenError> {
    let object = spa::pod::Object {
        type_: type_.as_raw(),
        id: id.as_raw(),
        properties,
    };
    Ok(spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &spa::pod::Value::Object(object),
    )?
    .0
    .into_inner())
}

/// A PipeWire error in the person's words, its own text kept for a report.
fn explain(message: &str) -> String {
    let reason = if message.contains("buffers") {
        t("PipeWire no pudo compartir las imágenes de la fuente con Rustcord.")
    } else if message.contains("format") {
        t("La fuente no ofrece imágenes en un formato que Rustcord pueda leer.")
    } else {
        return format!("PipeWire: {message}");
    };
    format!("{reason} (PipeWire: {message})")
}

/// One mapped buffer within `max`, if its planes hold a whole frame. A GPU
/// buffer's pixels go through `copy`.
fn frame(
    format: Format,
    datas: &mut [spa::buffer::Data],
    max: (u32, u32),
    copy: &mut Copied,
) -> Option<super::Frame> {
    if format.jpeg {
        let data = datas.first_mut()?;
        let chunk = data.chunk();
        let (offset, size) = (chunk.offset() as usize, chunk.size() as usize);
        let bytes: &[u8] = data.data()?;
        return jpeg_frame(bytes.get(offset..offset.checked_add(size)?)?, max);
    }
    let bytes_per_row = match format.pixels {
        Pixels::Bgr4 | Pixels::Rgb4 => format.width as usize * 4,
        Pixels::Yuy2 => (format.width as usize).div_ceil(2) * 4,
        Pixels::Nv12 | Pixels::I420 => format.width as usize,
    };
    if format.linear {
        let data = datas.first_mut()?;
        if data.type_() == spa::buffer::DataType::DmaBuf {
            let chunk = data.chunk();
            let offset = chunk.offset() as usize;
            let stride = usize::try_from(chunk.stride())
                .ok()
                .filter(|stride| *stride >= bytes_per_row)
                .unwrap_or(bytes_per_row);
            let len = stride.checked_mul(format.height as usize)?;
            // Between DMA-BUF's sync calls, so what the GPU drew is seen: the
            // first waits for it, and has the driver put the buffer where
            // the CPU reads it (amdgpu moves it out of video memory).
            let started = Instant::now();
            let fd = data.fd();
            sync(fd, DMA_BUF_SYNC_READ);
            let synced = started.elapsed();
            let Copied {
                bytes: rows_copy,
                maps,
                ..
            } = &mut *copy;
            let frame = (|| {
                // The source's own mapping, or Rustcord's when it gives none
                // (niri marks no buffer mappable, and says its size is 1).
                let bytes: &[u8] = match data.data().is_some_and(|bytes| bytes.len() > 1) {
                    true => data.data()?,
                    false => {
                        let whole = len.checked_add(offset)?;
                        mapping(maps, fd, data.as_raw().mapoffset as usize, whole)?.bytes()
                    }
                };
                let plane = Plane {
                    bytes: bytes.get(offset..)?,
                    stride,
                };
                // A GPU buffer is slow to read where it is: its rows go
                // through ordinary memory as they are read.
                super::convert_uncached(
                    format.pixels,
                    format.width,
                    format.height,
                    &plane,
                    max,
                    rows_copy,
                )
            })();
            let read = started.elapsed();
            sync(fd, DMA_BUF_SYNC_READ | DMA_BUF_SYNC_END);
            copy.mapping += (synced + (started.elapsed() - read)).max(Duration::from_micros(1));
            return frame;
        }
    }
    let (first, rest) = datas.split_first_mut()?;
    let (main, after) = plane(first, bytes_per_row)?;
    let mut planes = vec![main];
    if rest.is_empty() {
        let stride = planes[0].stride;
        planes.extend(packed_chroma(format.pixels, format.height, stride, after)?);
    } else {
        // Each chroma plane in its own data.
        let half = (format.width as usize).div_ceil(2);
        let rows = match format.pixels {
            Pixels::Nv12 => &[half * 2][..],
            Pixels::I420 => &[half, half][..],
            _ => &[][..],
        };
        for (data, row) in rest.iter_mut().zip(rows) {
            planes.push(plane(data, *row)?.0);
        }
    }
    convert_within(format.pixels, format.width, format.height, &planes, max)
}

/// The chroma planes after `height` luma rows of `stride` in one data:
/// NV12's one at the luma stride, I420's two at half of it.
fn packed_chroma(
    pixels: Pixels,
    height: u32,
    stride: usize,
    bytes: &[u8],
) -> Option<Vec<Plane<'_>>> {
    let (count, chroma) = match pixels {
        Pixels::Nv12 => (1, stride),
        Pixels::I420 => (2, stride.div_ceil(2)),
        _ => return Some(Vec::new()),
    };
    let start = stride.checked_mul(height as usize)?;
    let size = chroma.checked_mul((height as usize).div_ceil(2))?;
    (0..count)
        .map(|index| {
            let offset = start.checked_add(size.checked_mul(index)?)?;
            Some(Plane {
                bytes: bytes.get(offset..)?,
                stride: chroma,
            })
        })
        .collect()
}

/// A buffer's file mapped for reading; unmapped when dropped. It keeps the
/// buffer alive, whatever becomes of the descriptor it was made from.
struct Mapped {
    address: *mut libc::c_void,
    len: usize,
    /// From the page-aligned start of the mapping to the buffer's `mapoffset`.
    skip: usize,
}

/// Tell the kernel a CPU read of the buffer `fd` is starts or ends; a file
/// that is not a DMA-BUF refuses it, harmlessly.
fn sync(fd: std::os::fd::RawFd, flags: u64) {
    // SAFETY: the ioctl reads one u64 (`struct dma_buf_sync`) from it.
    let _ = unsafe { libc::ioctl(fd, DMA_BUF_IOCTL_SYNC, &flags) };
}

/// `DMA_BUF_IOCTL_SYNC` (`_IOW('b', 0, struct dma_buf_sync)`) and its flags.
const DMA_BUF_IOCTL_SYNC: libc::c_ulong = 0x4008_6200;
const DMA_BUF_SYNC_READ: u64 = 1;
const DMA_BUF_SYNC_END: u64 = 1 << 2;

impl Mapped {
    /// `len` bytes of `fd` from `at`, its buffer's `mapoffset`.
    fn new(fd: std::os::fd::RawFd, at: usize, len: usize) -> Option<Self> {
        if fd < 0 || len == 0 {
            return None;
        }
        // SAFETY: sysconf reads a constant of the system.
        let page = usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).ok()?;
        let start = at - at % page.max(1);
        let skip = at - start;
        let len = len.checked_add(skip)?;
        let offset = libc::off_t::try_from(start).ok()?;
        // SAFETY: a read-only shared mapping of `len` bytes of the buffer's
        // file, which the mapping itself keeps alive; it is only read
        // through `bytes` and unmapped on drop.
        let address = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                offset,
            )
        };
        if address == libc::MAP_FAILED {
            return None;
        }
        Some(Self { address, len, skip })
    }

    fn bytes(&self) -> &[u8] {
        // SAFETY: `len` readable bytes mapped at `address` until drop; `skip`
        // is within them.
        let all = unsafe { std::slice::from_raw_parts(self.address.cast::<u8>(), self.len) };
        &all[self.skip..]
    }
}

impl Drop for Mapped {
    fn drop(&mut self) {
        // SAFETY: the mapping `new` made, unmapped once.
        unsafe { libc::munmap(self.address, self.len) };
    }
}

/// The bytes of a data plane from its chunk's offset, and its stride.
fn plane(data: &mut spa::buffer::Data, row: usize) -> Option<(Plane<'_>, &[u8])> {
    let chunk = data.chunk();
    let offset = chunk.offset() as usize;
    let stride = usize::try_from(chunk.stride())
        .ok()
        .filter(|stride| *stride > 0)
        .unwrap_or(row);
    let bytes: &[u8] = data.data()?;
    let bytes = bytes.get(offset..)?;
    Some((Plane { bytes, stride }, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cameras_are_offered_mjpeg_besides_raw_pictures() {
        use spa::param::format::FormatProperties as Key;
        use spa::pod::{deserialize::PodDeserializer, Value};
        let pod = format_pod(MediaSubtype::Mjpg, Vec::new(), 30, None).unwrap();
        let Ok((_, Value::Object(object))) = PodDeserializer::deserialize_any_from(&pod) else {
            panic!("not an object");
        };
        let value = |key: Key| {
            object
                .properties
                .iter()
                .find(|property| property.key == key.as_raw())
                .map(|property| property.value.clone())
        };
        assert_eq!(
            value(Key::MediaSubtype),
            Some(Value::Id(spa::utils::Id(MediaSubtype::Mjpg.as_raw())))
        );
        assert_eq!(value(Key::VideoFormat), None, "MJPEG names no pixel layout");
        assert!(value(Key::VideoSize).is_some() && value(Key::VideoFramerate).is_some());
        let raw = format_pod(MediaSubtype::Raw, vec![VideoFormat::YUY2], 30, None).unwrap();
        let Ok((_, Value::Object(object))) = PodDeserializer::deserialize_any_from(&raw) else {
            panic!("not an object");
        };
        assert!(object
            .properties
            .iter()
            .any(|property| property.key == Key::VideoFormat.as_raw()));
    }

    #[test]
    fn every_format_offered_is_one_read() {
        for format in SCREEN_FORMATS.iter().chain(&CAMERA_FORMATS) {
            assert!(pixels(*format).is_some(), "{format:?}");
        }
    }

    /// The bug a V4L2 camera showed: with no data type said, it kept its
    /// buffers in its own mapping and the link failed with -22.
    #[test]
    fn buffers_are_asked_in_memory_another_process_can_map() {
        use spa::buffer::DataType;
        use spa::pod::{deserialize::PodDeserializer, ChoiceValue, Value};
        use spa::utils::{Choice, ChoiceEnum};
        let mask = |kind| {
            let pod = buffers_pod(kind).unwrap();
            let Ok((_, Value::Object(object))) = PodDeserializer::deserialize_any_from(&pod) else {
                panic!("not an object");
            };
            assert_eq!(object.id, spa::param::ParamType::Buffers.as_raw());
            assert_eq!(
                object.properties.len(),
                1,
                "count and size stay the source's"
            );
            let property = &object.properties[0];
            assert_eq!(property.key, spa::sys::SPA_PARAM_BUFFERS_dataType);
            match &property.value {
                Value::Choice(ChoiceValue::Int(Choice(_, ChoiceEnum::Flags { default, .. }))) => {
                    *default
                }
                other => panic!("not flags: {other:?}"),
            }
        };
        let has = |mask: i32, kind: DataType| mask & 1 << kind.as_raw() != 0;
        let camera = mask(true);
        assert!(has(camera, DataType::MemFd) && has(camera, DataType::MemPtr));
        assert!(has(camera, DataType::DmaBuf));
        // A screen's, until the linear layout is agreed.
        let screen = mask(false);
        assert!(has(screen, DataType::MemFd) && !has(screen, DataType::DmaBuf));
    }

    /// niri 26.04 shares a screen only as GPU buffers with a mandatory
    /// modifier: the linear one is offered, after shared memory.
    #[test]
    fn a_screen_also_takes_linear_gpu_buffers() {
        use spa::param::format::FormatProperties as Key;
        use spa::pod::{deserialize::PodDeserializer, ChoiceValue, PropertyFlags, Value};
        use spa::utils::{Choice, ChoiceEnum};
        let pod = format_pod(MediaSubtype::Raw, SCREEN_FORMATS.to_vec(), 60, Some(LINEAR)).unwrap();
        let Ok((_, Value::Object(object))) = PodDeserializer::deserialize_any_from(&pod) else {
            panic!("not an object");
        };
        let modifier = object
            .properties
            .iter()
            .find(|property| property.key == Key::VideoModifier.as_raw())
            .expect("a modifier");
        assert!(modifier.flags.contains(PropertyFlags::MANDATORY));
        assert!(matches!(
            &modifier.value,
            Value::Choice(ChoiceValue::Long(Choice(_, ChoiceEnum::Enum { default: 0, alternatives })))
                if alternatives == &vec![0]
        ));
        let plain = format_pod(MediaSubtype::Raw, SCREEN_FORMATS.to_vec(), 60, None).unwrap();
        let Ok((_, Value::Object(plain))) = PodDeserializer::deserialize_any_from(&plain) else {
            panic!("not an object");
        };
        assert!(plain
            .properties
            .iter()
            .all(|property| property.key != Key::VideoModifier.as_raw()));
    }

    /// A buffer's file maps for reading; a memfd stands in for a DMA-BUF,
    /// whose sync calls it refuses harmlessly.
    #[test]
    fn a_buffer_file_maps_for_reading() {
        use std::io::Write;
        use std::os::fd::{AsRawFd, FromRawFd};
        // SAFETY: a new anonymous file, owned below.
        let fd = unsafe { libc::memfd_create(c"frame".as_ptr(), 0) };
        assert!(fd >= 0);
        // SAFETY: `fd` is a valid descriptor this test owns.
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        file.write_all(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let mapped = Mapped::new(file.as_raw_fd(), 0, 8).expect("mapped");
        assert_eq!(mapped.bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        // From a buffer's `mapoffset`, aligned down to its page.
        let mapped = Mapped::new(file.as_raw_fd(), 3, 5).expect("mapped");
        assert_eq!(mapped.bytes(), &[4, 5, 6, 7, 8]);
        assert!(Mapped::new(-1, 0, 8).is_none());
    }

    /// A real screen through the whole reader, at the quality a share
    /// starts with, timed. `RUSTCORD_TEST_PW_NODE` names a screen cast's node
    /// on the session's PipeWire: niri's and GNOME's
    /// `org.gnome.Mutter.ScreenCast` give one with no dialog. In release, as
    /// it is sent. A 2880×1800 screen in GPU buffers took 370 ms a frame, so
    /// 2.7 frames a second went out; it takes 14 ms now.
    #[test]
    #[ignore = "needs a running screen cast: RUSTCORD_TEST_PW_NODE"]
    fn a_screen_is_read_fast_enough_for_the_rate_sent() {
        let node = std::env::var("RUSTCORD_TEST_PW_NODE")
            .ok()
            .and_then(|node| node.parse().ok())
            .expect("RUSTCORD_TEST_PW_NODE");
        let socket = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR") + "/pipewire-0";
        let remote = std::os::unix::net::UnixStream::connect(socket).expect("PipeWire");
        // `RUSTCORD_TEST_FPS=1` keeps a frame a second, as a screen that
        // barely changes gives: each one finds the processor asleep.
        let fps = std::env::var("RUSTCORD_TEST_FPS")
            .ok()
            .and_then(|fps| fps.parse().ok())
            .unwrap_or(30);
        let every = Duration::from_secs(1) / fps;
        let shared = Arc::new(Shared {
            sending: Some(((1280, 720), every)),
            ..Shared::new()
        });
        let reader = shared.clone();
        let reading = std::thread::spawn(move || {
            read(remote.into(), Some(node), VideoKind::Screen, None, &reader)
        });
        std::thread::sleep(Duration::from_secs(5));
        let stats = *lock(&shared.stats);
        let failed = matches!(
            &*lock(&shared.state),
            crate::video::VideoState::Ended(Some(_))
        );
        shared.end(None);
        // Stopped from here, its last word is that the source went away.
        let _ = reading.join().expect("the reader");
        assert!(!failed, "the source failed");
        let each = Duration::from_micros(stats.converting_micros / stats.kept.max(1));
        eprintln!("{each:?} a frame: {stats:?}");
        // Half of a frame's time at most: the rest is the source's.
        assert!(stats.kept > 0 && each * 2 < every, "{stats:?}");
    }

    /// A source's buffers stay mapped: one mapping a buffer, whatever
    /// descriptor names it; memfds stand in for DMA-BUFs.
    #[test]
    fn a_buffer_stays_mapped_from_frame_to_frame() {
        use std::io::Write;
        use std::os::fd::{AsRawFd, FromRawFd};
        let file = |bytes: &[u8]| {
            // SAFETY: a new anonymous file, owned by the `File` made of it.
            let mut file = unsafe {
                let fd = libc::memfd_create(c"frame".as_ptr(), 0);
                assert!(fd >= 0);
                std::fs::File::from_raw_fd(fd)
            };
            file.write_all(bytes).unwrap();
            file
        };
        let (one, other) = (file(&[1; 8]), file(&[2; 8]));
        let mut maps = Vec::new();
        let at = |maps: &mut Vec<Kept>, fd| mapping(maps, fd, 0, 8).map(|map| map.bytes().as_ptr());
        let first = at(&mut maps, one.as_raw_fd()).expect("mapped");
        assert_eq!(at(&mut maps, one.as_raw_fd()), Some(first), "kept");
        // The same buffer under another descriptor is the same mapping.
        let again = one.try_clone().unwrap();
        assert_eq!(at(&mut maps, again.as_raw_fd()), Some(first));
        assert_eq!(maps.len(), 1);
        // Another buffer is another mapping, and so is another part of one.
        assert_ne!(at(&mut maps, other.as_raw_fd()), Some(first));
        assert_eq!(
            mapping(&mut maps, other.as_raw_fd(), 0, 8).unwrap().bytes(),
            &[2; 8]
        );
        assert!(mapping(&mut maps, one.as_raw_fd(), 0, 4).is_some());
        assert_eq!(maps.len(), 3);
        assert!(mapping(&mut maps, -1, 0, 8).is_none());
        // No more than a source keeps: the oldest goes.
        for _ in 0..KEPT + 4 {
            assert!(mapping(&mut maps, file(&[3; 8]).as_raw_fd(), 0, 8).is_some());
        }
        assert_eq!(maps.len(), KEPT);
    }

    #[test]
    fn packed_chroma_follows_the_luma_rows() {
        // 4x2 I420 with a padded stride of 6: Y rows, then U, then V.
        let mut bytes = vec![0u8; 6 * 2];
        bytes.extend([1, 1, 1]);
        bytes.extend([2, 2, 2]);
        let planes = packed_chroma(Pixels::I420, 2, 6, &bytes).unwrap();
        assert_eq!(planes.len(), 2);
        assert_eq!((planes[0].bytes[0], planes[0].stride), (1, 3));
        assert_eq!((planes[1].bytes[0], planes[1].stride), (2, 3));
        let nv12 = packed_chroma(Pixels::Nv12, 2, 6, &bytes).unwrap();
        assert_eq!((nv12.len(), nv12[0].bytes[0], nv12[0].stride), (1, 1, 6));
        assert!(
            packed_chroma(Pixels::I420, 4, 6, &bytes).is_none(),
            "too short"
        );
    }

    #[test]
    fn link_failures_say_what_went_wrong() {
        let buffers = explain("error use input buffers: -22 (Invalid argument)");
        assert!(buffers.starts_with(t(
            "PipeWire no pudo compartir las imágenes de la fuente con Rustcord."
        )));
        assert!(buffers.ends_with("(PipeWire: error use input buffers: -22 (Invalid argument))"));
        assert!(explain("no more input formats").starts_with(t(
            "La fuente no ofrece imágenes en un formato que Rustcord pueda leer."
        )));
        assert_eq!(explain("node gone"), "PipeWire: node gone");
    }
}
