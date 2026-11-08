//! The illustrations of Discord's empty states, as its web client serves
//! them (`assets/illustrations/discord`, drawn from its SVGs at twice their
//! size), each shown at Discord's size; and the two layouts Discord sets
//! them in. `docs/discord-reference.md` has where each comes from.

use std::sync::OnceLock;

use limn::{Color, Element, FontFamily, RasterImage};

use crate::{gap, Look};

/// An empty state's illustration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Art {
    /// Threads popout: three bubbles tied with threads.
    Threads,
    /// Pinned messages: a sad face with a pin in it.
    Pins,
    /// A stream that is over: a couch with two controllers.
    StreamEnded,
    /// Message search that found nothing: a magnifying glass.
    SearchEmpty,
    /// Message search that failed: a broken magnifying glass.
    SearchFailed,
    /// A server without bans: a hammer and lightning.
    Bans,
    /// No invites: a paper plane among clouds.
    Invites,
    /// A server without roles: two member cards.
    Roles,
}

macro_rules! png {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../../assets/illustrations/discord/",
            $name,
            ".png"
        ))
        .as_slice()
    };
}

/// An art's name, its size where Discord shows it (the stream's at its
/// largest), and its file for a dark look and for a light one (the same
/// where Discord has one version).
type Entry = (&'static str, (f32, f32), &'static [u8], &'static [u8]);

/// Each art, in `Art`'s order.
const ARTS: [Entry; 8] = [
    ("threads", (256.0, 144.0), png!("threads"), png!("threads")),
    ("pins", (94.0, 120.0), png!("pins-dark"), png!("pins-light")),
    (
        "stream-ended",
        (381.5, 185.0),
        png!("stream-ended"),
        png!("stream-ended"),
    ),
    (
        "search-no-results",
        (160.0, 160.0),
        png!("search-no-results-dark"),
        png!("search-no-results-light"),
    ),
    (
        "search-error",
        (160.0, 160.0),
        png!("search-error-dark"),
        png!("search-error-light"),
    ),
    (
        "bans",
        (256.0, 212.0),
        png!("bans-dark"),
        png!("bans-light"),
    ),
    (
        "invites",
        (256.0, 130.0),
        png!("invites-dark"),
        png!("invites-light"),
    ),
    ("roles", (288.0, 192.0), png!("roles"), png!("roles")),
];

impl Art {
    /// Where Discord shows it, in logical pixels: width and height.
    pub fn size(self) -> (f32, f32) {
        ARTS[self as usize].1
    }
}

/// `art` as `look` shows it: Discord's light version on a light look. Each
/// picture is made once; the window decodes it once for each size drawn.
pub fn picture(art: Art, look: &Look) -> RasterImage {
    static MADE: OnceLock<Vec<RasterImage>> = OnceLock::new();
    let made = MADE.get_or_init(|| {
        ARTS.iter()
            .flat_map(|(name, _, dark, light)| [made(name, dark), made(name, light)])
            .collect()
    });
    made[art as usize * 2 + usize::from(look.canvas.r > 128)].clone()
}

fn made(name: &str, png: &[u8]) -> RasterImage {
    use base64::Engine as _;
    use sha2::Digest as _;
    RasterImage::new(
        format!("{:x}", sha2::Sha256::digest(png)),
        limn::RasterImageFormat::Png,
        u32::try_from(png.len()).expect("a bundled illustration is small"),
        std::sync::Arc::from(base64::engine::general_purpose::STANDARD.encode(png)),
        name,
    )
    .expect("a bundled illustration is a valid picture")
}

/// `art` at Discord's size, `scale` times it.
pub fn art(art: Art, scale: f32, look: &Look) -> Element {
    let (width, height) = art.size();
    Element::raster_image(picture(art, look)).size(Some(width * scale), Some(height * scale))
}

/// Centered lines that wrap within `width`.
fn copy(value: &str, size: f32, family: FontFamily, color: Color, width: f32) -> Element {
    Element::text(value)
        .font_family(family)
        .font_size(size)
        .foreground(color)
        .center_content()
        .max_width(width)
        .shrink_to_fit()
        .centered_x()
}

/// Discord's empty state of a settings page: the art, `space` under it its
/// title in 20 px muted and its note in 16 px under that, 300 wide at most.
pub fn slate(which: Art, space: f32, title: &str, note: &str, look: &Look) -> Element {
    let mut parts = vec![art(which, 1.0, look).centered_x(), gap(0.0, space)];
    parts.push(copy(
        title,
        20.0,
        FontFamily::SansSemibold,
        look.muted,
        300.0,
    ));
    if !note.is_empty() {
        parts.push(gap(0.0, 8.0));
        parts.push(copy(note, 16.0, FontFamily::Sans, look.muted, 300.0));
    }
    Element::column(parts)
}

/// Discord's larger empty state, of a popout or a page: the art, its title
/// in 24 px, its text in 16 px and the way to begin (`action`). `space`:
/// under the art and over the action.
pub fn intro(
    which: Art,
    title: &str,
    text: &str,
    action: Option<Element>,
    space: (f32, f32),
    look: &Look,
) -> Element {
    let mut parts = vec![
        art(which, 1.0, look).centered_x(),
        gap(0.0, space.0),
        copy(title, 24.0, FontFamily::SansSemibold, look.text, 560.0),
        gap(0.0, 8.0),
        copy(text, 16.0, FontFamily::Sans, look.text, 560.0),
    ];
    if let Some(action) = action {
        parts.push(gap(0.0, space.1));
        parts.push(action.centered_x());
    }
    Element::column(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::frame;
    use limn::{DrawCommand, Size};

    const ALL: [Art; 8] = [
        Art::Threads,
        Art::Pins,
        Art::StreamEnded,
        Art::SearchEmpty,
        Art::SearchFailed,
        Art::Bans,
        Art::Invites,
        Art::Roles,
    ];

    #[test]
    fn every_file_is_a_png_twice_discords_size() {
        for (name, (width, height), dark, light) in ARTS {
            for png in [dark, light] {
                assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "{name}");
                let side = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
                let (w, h) = (side(16) as f32, side(20) as f32);
                assert!((w - width * 2.0).abs() <= 1.0, "{name}: {w} wide");
                assert!((h - height * 2.0).abs() <= 1.0, "{name}: {h} high");
            }
        }
    }

    #[test]
    fn a_light_look_shows_discords_light_version() {
        for art in ALL {
            let (dark, light) = (picture(art, &Look::ONYX), picture(art, &Look::LIGHT));
            assert_eq!(dark.sha256(), picture(art, &Look::DARK).sha256());
            let one_version = matches!(art, Art::Threads | Art::StreamEnded | Art::Roles);
            assert_eq!(dark.sha256() == light.sha256(), one_version, "{art:?}");
        }
    }

    #[test]
    fn a_slate_draws_its_art_at_discords_size_over_its_words() {
        let look = Look::ONYX;
        let (_, laid) = frame(
            slate(Art::Bans, 40.0, "No hay personas vetadas", "Nadie.", &look),
            Size::new(600.0, 600.0),
        );
        let drawn = laid
            .commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::RasterImage { rect, image, .. } => {
                    Some((*rect, image.alt().to_owned()))
                }
                _ => None,
            })
            .expect("the art");
        assert_eq!(
            (drawn.0.width, drawn.0.height, drawn.1.as_str()),
            (256.0, 212.0, "bans")
        );
        let title = laid
            .text_layouts()
            .into_iter()
            .find(|block| block.source == "No hay personas vetadas")
            .expect("the title");
        assert!(title.runs[0].rect.y >= drawn.0.y + 212.0 + 40.0);
    }
}
