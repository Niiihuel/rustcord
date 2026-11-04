//! Rustcord's icon (`assets/icons/hicolor`: the white mark on its red tile)
//! as the tray's pixmaps, at whatever size the tray asks for, with a red
//! dot for mentions; in a call, the call's state drawn in its place.

use resvg::tiny_skia::{FillRule, Paint, PathBuilder, Pixmap as Canvas, Stroke, Transform};

use super::TrayVoice;

const MICROPHONE_MUTED: &str = include_str!("../../../../assets/glyphs/microphone-muted.svg");
const HEADPHONES_OFF: &str = include_str!("../../../../assets/glyphs/headphones-off.svg");
/// Discord's tray colors: the slash of what is off, a call's green and the
/// ring of a silent one.
const RED: [u8; 3] = [0xf0, 0x47, 0x47];
const GREEN: [u8; 3] = [0x43, 0xb5, 0x81];
const RING: [u8; 3] = [0x2e, 0x7b, 0x58];

/// What the icon shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Face {
    Mark,
    /// The mark with the dot.
    Unread,
    Voice(TrayVoice),
}

/// The icon as drawn for each size, smallest first: 16 and 24 with the
/// tile filling the box and the mark on whole pixels.
const DRAWN: [(usize, &[u8]); 6] = [
    (
        16,
        include_bytes!("../../../../assets/icons/hicolor/16x16/apps/rustcord.png"),
    ),
    (
        24,
        include_bytes!("../../../../assets/icons/hicolor/24x24/apps/rustcord.png"),
    ),
    (
        32,
        include_bytes!("../../../../assets/icons/hicolor/32x32/apps/rustcord.png"),
    ),
    (
        48,
        include_bytes!("../../../../assets/icons/hicolor/48x48/apps/rustcord.png"),
    ),
    (
        64,
        include_bytes!("../../../../assets/icons/hicolor/64x64/apps/rustcord.png"),
    ),
    (
        128,
        include_bytes!("../../../../assets/icons/hicolor/128x128/apps/rustcord.png"),
    ),
];
const DOT: [u8; 3] = [0xf2, 0x3f, 0x43];

/// One pixmap as StatusNotifierItem carries it: width, height and ARGB32
/// pixels in network byte order.
pub(crate) type Pixmap = (i32, i32, Vec<u8>);

/// The icon at the sizes panels ask for, from 16 to 64: the panel takes
/// the nearest.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn pixmaps(face: Face) -> Vec<Pixmap> {
    [16, 22, 24, 32, 48, 64]
        .into_iter()
        .map(|size| pixmap(size, face))
        .collect()
}

/// The icon at `size` pixels a side (the Windows tray asks for its own).
pub(crate) fn pixmap(size: usize, face: Face) -> Pixmap {
    match face {
        Face::Mark => mark(size, false),
        Face::Unread => mark(size, true),
        Face::Voice(voice) => call(size.max(1), voice),
    }
}

/// A call's state, `size` pixels a side, as Discord's tray draws it: a green
/// ball, faint inside a darker ring while silent; the microphone or the
/// sound white, crossed out in red.
fn call(size: usize, voice: TrayVoice) -> Pixmap {
    let blank = || (size as i32, size as i32, vec![0; size * size * 4]);
    let Some(mut canvas) = Canvas::new(size as u32, size as u32) else {
        return blank();
    };
    let side = size as f32;
    let middle = side / 2.0;
    match voice {
        TrayVoice::Speaking | TrayVoice::Connected => {
            let speaking = voice == TrayVoice::Speaking;
            let mut paint = Paint::default();
            let [red, green, blue] = GREEN;
            paint.set_color_rgba8(red, green, blue, if speaking { 255 } else { 102 });
            if let Some(ball) = PathBuilder::from_circle(middle, middle, middle) {
                canvas.fill_path(
                    &ball,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
            let width = side / 8.0;
            let [red, green, blue] = RING;
            paint.set_color_rgba8(red, green, blue, 255);
            if let Some(ring) =
                PathBuilder::from_circle(middle, middle, (side - width) / 2.0).filter(|_| !speaking)
            {
                let stroke = Stroke {
                    width,
                    ..Stroke::default()
                };
                canvas.stroke_path(&ring, &paint, &stroke, Transform::identity(), None);
            }
        }
        TrayVoice::Muted | TrayVoice::Deafened => {
            let glyph = if voice == TrayVoice::Muted {
                MICROPHONE_MUTED
            } else {
                HEADPHONES_OFF
            };
            let scale = side / 24.0;
            let transform = Transform::from_scale(scale, scale);
            let (slash, rest) = slashed(glyph);
            draw(&mut canvas, &rest, [255; 3], transform);
            draw(&mut canvas, &slash, RED, transform);
        }
    }
    let pixels = canvas
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.alpha(), color.red(), color.green(), color.blue()]
        })
        .collect();
    (size as i32, size as i32, pixels)
}

/// A crossed-out glyph of `assets/glyphs` as two: its slash, the one shape
/// of its path that crosses the whole box, and the rest. Its path is made
/// of M, L, C and Z only, so its numbers are x and y in turn.
fn slashed(glyph: &str) -> (String, String) {
    let Some((start, end)) = glyph.find(" d=\"").and_then(|at| {
        let start = at + 4;
        Some((start, start + glyph[start..].find('"')?))
    }) else {
        return (String::new(), glyph.to_owned());
    };
    let crosses = |shape: &&str| {
        let numbers: Vec<f32> = shape
            .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .filter_map(|number| number.parse().ok())
            .collect();
        let span = |axis: usize| {
            let values = || numbers.iter().skip(axis).step_by(2).copied();
            values().fold(f32::MIN, f32::max) - values().fold(f32::MAX, f32::min)
        };
        span(0) > 20.0 && span(1) > 20.0
    };
    let (slash, rest): (Vec<&str>, Vec<&str>) =
        glyph[start..end].split_inclusive('Z').partition(crosses);
    let with = |path: Vec<&str>| format!("{}{}{}", &glyph[..start], path.concat(), &glyph[end..]);
    (with(slash), with(rest))
}

/// Draw `glyph`, which paints in `currentColor`, in `color`; nothing for
/// one that does not parse.
pub(crate) fn draw(
    canvas: &mut Canvas,
    glyph: &str,
    [red, green, blue]: [u8; 3],
    transform: Transform,
) {
    let source = glyph.replace("currentColor", &format!("#{red:02x}{green:02x}{blue:02x}"));
    if let Ok(tree) = resvg::usvg::Tree::from_str(&source, &resvg::usvg::Options::default()) {
        resvg::render(&tree, transform, &mut canvas.as_mut());
    }
}

/// Rustcord's mark at `size`: the one drawn for it, or the next larger one
/// shrunk; with the dot when `dot`.
fn mark(size: usize, dot: bool) -> Pixmap {
    let size = size.max(1);
    let (drawn, png) = DRAWN
        .iter()
        .find(|(drawn, _)| *drawn >= size)
        .unwrap_or(&DRAWN[DRAWN.len() - 1]);
    let rgba = decode(png).map_or_else(
        || vec![0; size * size * 4],
        |rgba| shrink(&rgba, *drawn, size),
    );
    let mut pixels = Vec::with_capacity(size * size * 4);
    let scale = 24.0 / size as f32;
    for (index, pixel) in rgba.as_chunks::<4>().0.iter().enumerate() {
        let (x, y) = (index % size, index / size);
        let mut color = [pixel[0], pixel[1], pixel[2]];
        let mut alpha = f32::from(pixel[3]) / 255.0;
        if dot {
            // In the icon's 24-unit box, over the tile's top right corner.
            let point = ((x as f32 + 0.5) * scale, (y as f32 + 0.5) * scale);
            let from = ((point.0 - 19.0).powi(2) + (point.1 - 5.0).powi(2)).sqrt();
            // A ring of the canvas around the dot keeps it apart.
            let ring = coverage(from - 6.0, scale);
            let inside = coverage(from - 4.5, scale);
            alpha = (alpha * (1.0 - ring) + inside).min(1.0);
            color = blend(color, DOT, inside);
        }
        pixels.extend([(alpha * 255.0).round() as u8, color[0], color[1], color[2]]);
    }
    (size as i32, size as i32, pixels)
}

/// An embedded PNG's RGBA pixels, 8 bits each (16 and 24 are drawn with
/// 16); `None` if it has no alpha.
fn decode(png: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(png));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut pixels).ok()?;
    (info.color_type == png::ColorType::Rgba && info.bit_depth == png::BitDepth::Eight).then(|| {
        pixels.truncate(info.buffer_size());
        pixels
    })
}

/// `rgba`, `from` pixels a side, at `to`: each pixel the average of what
/// it covers, weighted by its alpha so clear pixels lend no color.
fn shrink(rgba: &[u8], from: usize, to: usize) -> Vec<u8> {
    if from == to {
        return rgba.to_vec();
    }
    let scale = from as f32 / to as f32;
    // The source pixels under [start, end), with how much of each.
    let span = |start: f32, end: f32| {
        let first = start.floor() as usize;
        let last = (end.ceil() as usize).min(from);
        (first..last).map(move |pixel| {
            let covered = end.min(pixel as f32 + 1.0) - start.max(pixel as f32);
            (pixel, covered.max(0.0))
        })
    };
    let mut out = Vec::with_capacity(to * to * 4);
    for y in 0..to {
        for x in 0..to {
            let mut sum = [0.0f32; 4];
            let mut area = 0.0;
            let (left, top) = (x as f32 * scale, y as f32 * scale);
            for (row, height) in span(top, top + scale) {
                for (column, width) in span(left, left + scale) {
                    let at = (row * from + column) * 4;
                    let weight = width * height;
                    let alpha = f32::from(rgba[at + 3]) * weight;
                    for channel in 0..3 {
                        sum[channel] += f32::from(rgba[at + channel]) * alpha;
                    }
                    sum[3] += alpha;
                    area += weight;
                }
            }
            let color = |channel: usize| {
                if sum[3] > 0.0 {
                    (sum[channel] / sum[3]).round() as u8
                } else {
                    0
                }
            };
            out.extend([
                color(0),
                color(1),
                color(2),
                (sum[3] / area.max(f32::EPSILON)).round() as u8,
            ]);
        }
    }
    out
}

/// 1 inside a shape whose signed distance is `distance`, 0 outside, a
/// ramp across one pixel at its edge.
fn coverage(distance: f32, pixel: f32) -> f32 {
    (0.5 - distance / pixel).clamp(0.0, 1.0)
}

fn blend(under: [u8; 3], over: [u8; 3], amount: f32) -> [u8; 3] {
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * amount).round() as u8;
    [
        mix(under[0], over[0]),
        mix(under[1], over[1]),
        mix(under[2], over[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pixmap: &Pixmap, x: usize, y: usize) -> [u8; 4] {
        let start = (y * pixmap.0 as usize + x) * 4;
        pixmap.2[start..start + 4].try_into().unwrap()
    }

    #[test]
    fn every_size_drawn_is_an_rgba_png_of_its_size() {
        for (size, png) in DRAWN {
            let rgba = decode(png).unwrap_or_else(|| panic!("{size}"));
            assert_eq!(rgba.len(), size * size * 4, "{size}");
        }
    }

    #[test]
    fn the_icon_is_red_with_a_white_mark_at_any_size_and_the_dot_red() {
        for size in [16, 20, 22, 24, 32, 40, 48, 64, 128] {
            let plain = pixmap(size, Face::Mark);
            assert_eq!((plain.0, plain.1), (size as i32, size as i32));
            assert_eq!(plain.2.len(), size * size * 4);
            assert!(
                at(&plain, 0, 0)[0] < 16,
                "{size}: a rounded corner is clear"
            );
            // Red at the tile's foot, under the mark and clear of its edge.
            let [alpha, red, green, blue] = at(&plain, size / 2, size * 13 / 16);
            assert!(alpha > 240, "{size}: {alpha}");
            assert!(
                red > 180 && green < 90 && blue < 90,
                "{size}: {red} {green} {blue}"
            );
            // The rustcord, white.
            assert!(
                plain
                    .2
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel.iter().all(|value| *value > 235)),
                "{size}: the rustcord"
            );
        }
        let plain = pixmap(24, Face::Mark);
        let dotted = pixmap(24, Face::Unread);
        assert_eq!(&at(&dotted, 19, 5)[1..], &DOT);
        assert_eq!(at(&dotted, 19, 5)[0], 255);
        assert_eq!(at(&dotted, 8, 18), at(&plain, 8, 18), "the icon stays");
    }

    #[test]
    fn in_a_call_the_icon_is_its_state_alone_in_discord_s_precedence() {
        let state = |voice, mentions| super::super::TrayState {
            voice,
            mentions,
            ..Default::default()
        };
        assert_eq!(state(None, 0).face(), Face::Mark);
        assert_eq!(state(None, 2).face(), Face::Unread);
        let deafened = TrayVoice::of(true, true, true);
        assert_eq!(deafened, TrayVoice::Deafened);
        assert_eq!(state(Some(deafened), 2).face(), Face::Voice(deafened));
        assert_eq!(TrayVoice::of(true, false, true), TrayVoice::Muted);
        assert_eq!(TrayVoice::of(false, false, true), TrayVoice::Speaking);
        assert_eq!(TrayVoice::of(false, false, false), TrayVoice::Connected);

        let pixel = |voice, x: usize, y: usize| at(&pixmap(16, Face::Voice(voice)), x, y);
        // Speaking: a solid green ball; silent: faint green in a darker ring.
        assert_eq!(pixel(TrayVoice::Speaking, 8, 8), [255, 0x43, 0xb5, 0x81]);
        assert_eq!(pixel(TrayVoice::Speaking, 0, 0)[0], 0, "round");
        assert_eq!(pixel(TrayVoice::Connected, 8, 8)[0], 102);
        let ring = pixel(TrayVoice::Connected, 8, 1);
        assert_eq!(ring[0], 255);
        assert!(
            ring[1..].iter().zip(RING).all(|(a, b)| a.abs_diff(b) <= 2),
            "{ring:?}"
        );
        // Off: the glyph white, its slash red; no mark under them.
        for voice in [TrayVoice::Muted, TrayVoice::Deafened] {
            let drawn = pixmap(16, Face::Voice(voice));
            let pixels = drawn.2.as_chunks::<4>().0;
            assert!(pixels.contains(&[255, 255, 255, 255]), "{voice:?}: white");
            assert!(
                pixels.contains(&[255, RED[0], RED[1], RED[2]]),
                "{voice:?}: slash"
            );
            let opaque = pixels.iter().filter(|pixel| pixel[0] > 200).count();
            assert!(opaque < 16 * 16 / 2, "{voice:?}: no tile under it");
        }
        let (slash, rest) = slashed(MICROPHONE_MUTED);
        assert!(slash.contains(" d=\"M2.7 22.7") && slash.trim_end().ends_with("Z\"/></svg>"));
        assert!(rest.contains(" d=\"M10.8") && !rest.contains("M2.7 22.7"));
        // The headphones' slash is not their first shape.
        let (slash, rest) = slashed(HEADPHONES_OFF);
        assert!(slash.contains(" d=\"M22.7 2.7") && !slash.contains("M20.2"));
        assert!(rest.contains(" d=\"M20.2") && !rest.contains("M22.7 2.7"));
    }

    #[test]
    fn shrinking_averages_what_each_pixel_covers_and_clear_lends_no_color() {
        // Two by two: opaque red and clear black on top, opaque blue below.
        let rgba = [
            255, 0, 0, 255, 0, 0, 0, 0, //
            0, 0, 255, 255, 0, 0, 255, 255,
        ];
        assert_eq!(shrink(&rgba, 2, 1), [85, 0, 170, 191]);
        assert_eq!(shrink(&rgba, 2, 2), rgba);
    }
}
