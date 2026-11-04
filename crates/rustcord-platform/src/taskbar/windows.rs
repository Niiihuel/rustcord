//! The taskbar on Windows, through `ITaskbarList3`. What it shows is
//! Discord's: the overlay is its tray in a call (deafened over muted over
//! speaking over only connected; red with the glyph for the first two, a
//! green dot for the others, faint inside a ring while silent), and the
//! preview's buttons are its thumbnail toolbar (camera, microphone, sound,
//! hang up), hidden out of a call. The glyphs are Rustcord's own
//! (`assets/glyphs`), tinted. The pictures and buttons are worked out without
//! Windows, so they are tested everywhere.

use resvg::tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use rustcord_i18n::t;

use super::{TaskbarEvent, TaskbarState};

const MICROPHONE: &str = include_str!("../../../../assets/glyphs/microphone.svg");
const MICROPHONE_MUTED: &str = include_str!("../../../../assets/glyphs/microphone-muted.svg");
const HEADPHONES: &str = include_str!("../../../../assets/glyphs/headphones.svg");
const HEADPHONES_OFF: &str = include_str!("../../../../assets/glyphs/headphones-off.svg");
const VIDEO: &str = include_str!("../../../../assets/glyphs/video.svg");
const VIDEO_OFF: &str = include_str!("../../../../assets/glyphs/video-off.svg");
const HANG_UP: &str = include_str!("../../../../assets/glyphs/hang-up.svg");

/// The preview's buttons, in Discord's order.
const CAMERA: u32 = 1;
const MUTE: u32 = 2;
const DEAFEN: u32 = 3;
const DISCONNECT: u32 = 4;

/// The overlay over Rustcord's icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Badge {
    /// White on the dot; none on the green one.
    glyph: Option<&'static str>,
    color: [u8; 3],
    /// Connected and silent: the dot faint inside a ring.
    faint: bool,
    /// What the shell reads out.
    description: &'static str,
}

/// The overlay for `state`; `None` out of a call.
fn badge(state: &TaskbarState) -> Option<Badge> {
    let call = state.call?;
    let _language = rustcord_i18n::scoped(state.language);
    let (glyph, description) = if call.deafened {
        (Some(HEADPHONES_OFF), t("Ensordecido"))
    } else if call.muted {
        (Some(MICROPHONE_MUTED), t("Silenciado"))
    } else {
        (None, t("Voz conectada"))
    };
    Some(Badge {
        glyph,
        color: if glyph.is_some() {
            state.danger
        } else {
            state.positive
        },
        faint: glyph.is_none() && !call.speaking,
        description,
    })
}

/// A button of the preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Button {
    id: u32,
    glyph: &'static str,
    color: [u8; 3],
    tip: &'static str,
    hidden: bool,
}

/// The preview's buttons for `state`, in its language: red what is off.
fn buttons(state: &TaskbarState) -> [Button; 4] {
    let _language = rustcord_i18n::scoped(state.language);
    let call = state.call.unwrap_or_default();
    let button = |id, (glyph, tip), off: bool| Button {
        id,
        glyph,
        color: if off { state.danger } else { state.ink },
        tip,
        hidden: state.call.is_none(),
    };
    [
        // As Discord's, the camera shows what a click does: crossed out
        // while it is on.
        button(
            CAMERA,
            if call.camera {
                (VIDEO_OFF, t("Apagar la cámara"))
            } else {
                (VIDEO, t("Encender la cámara"))
            },
            call.camera,
        ),
        button(
            MUTE,
            if call.muted {
                (MICROPHONE_MUTED, t("Desactivar silencio"))
            } else {
                (MICROPHONE, t("Silenciar"))
            },
            call.muted,
        ),
        button(
            DEAFEN,
            if call.deafened {
                (HEADPHONES_OFF, t("Desactivar ensordecimiento"))
            } else {
                (HEADPHONES, t("Ensordecer"))
            },
            call.deafened,
        ),
        button(DISCONNECT, (HANG_UP, t("Desconectar")), false),
    ]
}

/// What the button `id` asks for.
fn event(id: u32) -> Option<TaskbarEvent> {
    Some(match id {
        CAMERA => TaskbarEvent::ToggleCamera,
        MUTE => TaskbarEvent::ToggleMute,
        DEAFEN => TaskbarEvent::ToggleDeafen,
        DISCONNECT => TaskbarEvent::Disconnect,
        _ => return None,
    })
}

/// `glyph` in `color`, `size` pixels a side.
fn glyph_pixels(glyph: &str, color: [u8; 3], size: u32) -> Vec<u8> {
    let Some(mut pixmap) = Pixmap::new(size, size) else {
        return Vec::new();
    };
    let scale = size as f32 / 24.0;
    draw(
        &mut pixmap,
        glyph,
        color,
        Transform::from_scale(scale, scale),
    );
    bgra(&pixmap)
}

/// The overlay, `size` pixels a side: the dot, and its glyph in white in
/// the middle five eighths.
fn badge_pixels(badge: &Badge, size: u32) -> Vec<u8> {
    let Some(mut pixmap) = Pixmap::new(size, size) else {
        return Vec::new();
    };
    let side = size as f32;
    let [red, green, blue] = badge.color;
    let mut paint = Paint::default();
    // Discord's tray while connected: its green at 40 % in a darker ring.
    paint.set_color_rgba8(red, green, blue, if badge.faint { 102 } else { 255 });
    if let Some(dot) = PathBuilder::from_circle(side / 2.0, side / 2.0, side / 2.0) {
        pixmap.fill_path(&dot, &paint, FillRule::Winding, Transform::identity(), None);
    }
    if badge.faint {
        let width = side / 8.0;
        let darker = |channel: u8| (u16::from(channel) * 68 / 100) as u8;
        paint.set_color_rgba8(darker(red), darker(green), darker(blue), 255);
        if let Some(ring) = PathBuilder::from_circle(side / 2.0, side / 2.0, (side - width) / 2.0) {
            let stroke = Stroke {
                width,
                ..Stroke::default()
            };
            pixmap.stroke_path(&ring, &paint, &stroke, Transform::identity(), None);
        }
    }
    if let Some(glyph) = badge.glyph {
        let inner = side * 0.625;
        let (scale, offset) = (inner / 24.0, (side - inner) / 2.0);
        let transform = Transform::from_row(scale, 0.0, 0.0, scale, offset, offset);
        draw(&mut pixmap, glyph, [255; 3], transform);
    }
    bgra(&pixmap)
}

/// Draw `glyph`, which paints in `currentColor`, in `color`.
fn draw(pixmap: &mut Pixmap, glyph: &str, [red, green, blue]: [u8; 3], transform: Transform) {
    let source = glyph.replace("currentColor", &format!("#{red:02x}{green:02x}{blue:02x}"));
    if let Ok(tree) = resvg::usvg::Tree::from_str(&source, &resvg::usvg::Options::default()) {
        resvg::render(&tree, transform, &mut pixmap.as_mut());
    }
}

/// As a 32-bit Windows icon reads it: blue, green, red and straight alpha,
/// from the top row down.
fn bgra(pixmap: &Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.blue(), color.green(), color.red(), color.alpha()]
        })
        .collect()
}

#[cfg(windows)]
pub(super) use shell::start;

#[cfg(windows)]
mod shell {
    use std::sync::mpsc;

    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{
        DefSubclassProc, ITaskbarList3, RemoveWindowSubclass, SetWindowSubclass, TaskbarList,
        THBF_ENABLED, THBF_HIDDEN, THBN_CLICKED, THB_FLAGS, THB_ICON, THB_TOOLTIP, THUMBBUTTON,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, RegisterWindowMessageW, HICON, WM_COMMAND, WM_NCDESTROY,
    };

    use super::{badge, badge_pixels, buttons, event, glyph_pixels, Badge, Button};
    use crate::taskbar::{Job, TaskbarEvent, TaskbarState};
    use crate::win::{icon, small_icon_size, wide};

    /// This subclass among the window's others.
    const SUBCLASS: usize = 0x5243_5442;

    /// What the window's subclass works with, on the window's thread.
    struct Subclass {
        /// `TaskbarButtonCreated`: the window's button is there, or back.
        created: u32,
        jobs: mpsc::Sender<Job>,
        report: Box<dyn Fn(TaskbarEvent)>,
    }

    pub(in crate::taskbar) fn start(
        window: isize,
        report: impl Fn(TaskbarEvent) + 'static,
    ) -> Option<mpsc::Sender<Job>> {
        let (jobs, received) = mpsc::channel();
        std::thread::Builder::new()
            .name("rustcord-taskbar".into())
            // SAFETY: COM is this thread's own; the window is only named to
            // the shell, which ignores a window that went.
            .spawn(move || unsafe { run(HWND(window as *mut _), received) })
            .ok()?;
        let subclass = Box::into_raw(Box::new(Subclass {
            // SAFETY: a plain call; the name ends in NUL.
            created: unsafe { RegisterWindowMessageW(w!("TaskbarButtonCreated")) },
            jobs: jobs.clone(),
            report: Box::new(report),
        }));
        // SAFETY: on the window's own thread, as the host calls it; the box
        // is the subclass's until the window's last message frees it.
        let watched = unsafe {
            SetWindowSubclass(
                HWND(window as *mut _),
                Some(procedure),
                SUBCLASS,
                subclass as usize,
            )
        };
        if !watched.as_bool() {
            // SAFETY: never given to the window.
            drop(unsafe { Box::from_raw(subclass) });
            return None;
        }
        // The button may be there already: the window showed before this.
        let _ = jobs.send(Job::Created);
        Some(jobs)
    }

    unsafe extern "system" fn procedure(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        data: usize,
    ) -> LRESULT {
        let subclass = data as *mut Subclass;
        if message == WM_NCDESTROY {
            // SAFETY: the window's last message: the subclass and its box go,
            // as the subclassing documentation asks.
            unsafe {
                let _ = RemoveWindowSubclass(window, Some(procedure), SUBCLASS);
                drop(Box::from_raw(subclass));
            }
        } else {
            // SAFETY: the box given at `start`, alive until WM_NCDESTROY.
            let subclass = unsafe { &*subclass };
            if message == WM_COMMAND && (wparam.0 >> 16) & 0xffff == THBN_CLICKED as usize {
                if let Some(clicked) = event((wparam.0 & 0xffff) as u32) {
                    (subclass.report)(clicked);
                    return LRESULT(0);
                }
            } else if message == subclass.created {
                let _ = subclass.jobs.send(Job::Created);
            }
        }
        // SAFETY: the window's own procedure, with what came.
        unsafe { DefSubclassProc(window, message, wparam, lparam) }
    }

    /// The drawing thread: the shell's taskbar list, told what changes.
    ///
    /// # Safety
    /// Called once, on a thread of its own.
    unsafe fn run(window: HWND, jobs: mpsc::Receiver<Job>) {
        unsafe {
            let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if let Ok(list) =
                CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
            {
                if list.HrInit().is_ok() {
                    let mut shown = Shown::default();
                    let mut state = TaskbarState::default();
                    for job in jobs {
                        match job {
                            Job::State(new) => state = new,
                            // A new button has neither buttons nor overlay.
                            Job::Created => shown = Shown::default(),
                        }
                        shown.show(&list, window, &state);
                    }
                }
            }
            if com.is_ok() {
                CoUninitialize();
            }
        }
    }

    /// What the shell was given last, so it is told only what changed.
    #[derive(Default)]
    struct Shown {
        /// On the window's button; `None` until they are added to it.
        buttons: Option<[Button; 4]>,
        badge: Option<Option<Badge>>,
    }

    impl Shown {
        /// # Safety
        /// `list` is this thread's.
        unsafe fn show(&mut self, list: &ITaskbarList3, window: HWND, state: &TaskbarState) {
            let size = small_icon_size();
            let wanted = buttons(state);
            if self.buttons != Some(wanted) {
                let pictures = wanted.map(|button| {
                    icon(size, &glyph_pixels(button.glyph, button.color, size as u32))
                        .unwrap_or_default()
                });
                let thumbs: Vec<THUMBBUTTON> = wanted
                    .iter()
                    .zip(pictures)
                    .map(|(button, picture)| thumb(button, picture))
                    .collect();
                // SAFETY: filled buttons; the shell copies their icons, which
                // go after the call.
                let given = unsafe {
                    match self.buttons {
                        Some(_) => list.ThumbBarUpdateButtons(window, &thumbs),
                        // Once per window's button: when they are on it
                        // already, they are updated instead.
                        None => list
                            .ThumbBarAddButtons(window, &thumbs)
                            .or_else(|_| list.ThumbBarUpdateButtons(window, &thumbs)),
                    }
                };
                destroy(pictures);
                if given.is_err() {
                    // No button yet: the window is hidden, or still opening.
                    return;
                }
                self.buttons = Some(wanted);
            }
            let badge = badge(state);
            if self.badge != Some(badge) {
                let picture =
                    badge.and_then(|badge| icon(size, &badge_pixels(&badge, size as u32)));
                let description = wide(badge.map_or("", |badge| badge.description));
                // SAFETY: as above; no picture takes the overlay away.
                let given = unsafe {
                    list.SetOverlayIcon(
                        window,
                        picture.unwrap_or_default(),
                        PCWSTR(description.as_ptr()),
                    )
                };
                destroy(picture);
                if given.is_ok() {
                    self.badge = Some(badge);
                }
            }
        }
    }

    fn thumb(button: &Button, picture: HICON) -> THUMBBUTTON {
        let mut tip = [0u16; 260];
        for (slot, unit) in tip.iter_mut().zip(button.tip.encode_utf16().take(259)) {
            *slot = unit;
        }
        THUMBBUTTON {
            dwMask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
            iId: button.id,
            hIcon: picture,
            szTip: tip,
            dwFlags: if button.hidden {
                THBF_HIDDEN
            } else {
                THBF_ENABLED
            },
            ..Default::default()
        }
    }

    fn destroy(pictures: impl IntoIterator<Item = HICON>) {
        for picture in pictures {
            if !picture.is_invalid() {
                // SAFETY: an icon made here, which the shell copied.
                let _ = unsafe { DestroyIcon(picture) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taskbar::TaskbarCall;

    const INK: [u8; 3] = [154, 161, 178];
    const RED: [u8; 3] = [210, 45, 57];
    const GREEN: [u8; 3] = [61, 158, 96];

    fn state(call: Option<TaskbarCall>) -> TaskbarState {
        TaskbarState {
            call,
            ink: INK,
            danger: RED,
            positive: GREEN,
            language: Default::default(),
        }
    }

    fn call() -> TaskbarCall {
        TaskbarCall::default()
    }

    #[test]
    fn the_overlay_is_discord_s_tray_in_a_call_and_nothing_out_of_one() {
        assert_eq!(badge(&state(None)), None);
        let everything = TaskbarCall {
            muted: true,
            deafened: true,
            speaking: true,
            camera: true,
        };
        let deafened = badge(&state(Some(everything))).unwrap();
        assert_eq!(
            (deafened.glyph, deafened.color, deafened.description),
            (Some(HEADPHONES_OFF), RED, "Ensordecido")
        );
        let muted = badge(&state(Some(TaskbarCall {
            deafened: false,
            ..everything
        })))
        .unwrap();
        assert_eq!((muted.glyph, muted.color), (Some(MICROPHONE_MUTED), RED));
        let speaking = badge(&state(Some(TaskbarCall {
            speaking: true,
            ..call()
        })))
        .unwrap();
        assert_eq!(
            (speaking.glyph, speaking.color, speaking.faint),
            (None, GREEN, false)
        );
        assert!(badge(&state(Some(call()))).unwrap().faint, "silent: faint");
    }

    #[test]
    fn the_buttons_are_discord_s_red_when_off_and_hidden_out_of_a_call() {
        assert!(buttons(&state(None)).iter().all(|button| button.hidden));
        let muted = buttons(&state(Some(TaskbarCall {
            muted: true,
            ..call()
        })));
        assert_eq!(
            muted.map(|button| button.id),
            [CAMERA, MUTE, DEAFEN, DISCONNECT]
        );
        assert!(muted.iter().all(|button| !button.hidden));
        assert_eq!(
            (muted[1].glyph, muted[1].color, muted[1].tip),
            (MICROPHONE_MUTED, RED, "Desactivar silencio")
        );
        assert_eq!(
            (muted[2].glyph, muted[2].color, muted[2].tip),
            (HEADPHONES, INK, "Ensordecer")
        );
        assert_eq!((muted[0].glyph, muted[0].color), (VIDEO, INK));
        let camera = buttons(&state(Some(TaskbarCall {
            camera: true,
            ..call()
        })));
        assert_eq!(
            (camera[0].glyph, camera[0].color, camera[0].tip),
            (VIDEO_OFF, RED, "Apagar la cámara")
        );
        let english = buttons(&TaskbarState {
            language: rustcord_i18n::Language::English,
            ..state(Some(call()))
        });
        assert_eq!(english[3].tip, "Disconnect");
        for button in muted {
            assert!(event(button.id).is_some());
        }
        assert_eq!(event(0), None);
        assert_eq!(event(5), None);
    }

    #[test]
    fn the_pictures_are_the_glyphs_tinted_as_windows_reads_them() {
        let pixels = glyph_pixels(MICROPHONE, RED, 16);
        assert_eq!(pixels.len(), 16 * 16 * 4);
        assert_eq!(&pixels[..4], &[0, 0, 0, 0], "a corner, clear");
        let [red, green, blue] = RED;
        let pixels = pixels.as_chunks::<4>().0;
        assert!(pixels.contains(&[blue, green, red, 255]), "blue first");
        let muted = badge(&state(Some(TaskbarCall {
            muted: true,
            ..call()
        })))
        .unwrap();
        let dot = badge_pixels(&muted, 16);
        assert_eq!(&dot[..4], &[0, 0, 0, 0], "a corner, outside the dot");
        let dot = dot.as_chunks::<4>().0;
        assert!(dot.contains(&[blue, green, red, 255]), "the red dot");
        assert!(
            dot.iter()
                .any(|pixel| pixel[..3].iter().all(|channel| *channel > 230)),
            "its white glyph"
        );
        let silent = badge_pixels(&badge(&state(Some(call()))).unwrap(), 16);
        let silent = silent.as_chunks::<4>().0;
        // The middle is the faint green, the edge its darker ring.
        assert_eq!(silent[8 * 16 + 8][3], 102);
        assert!(silent.iter().any(|pixel| pixel[3] == 255));
    }
}
