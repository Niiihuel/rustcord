//! Streamer mode's parts: the notice over the window while it is on, and
//! what a page hiding the account's information shows in its place.

use limn::{Button, ButtonVariant, Color, Component, ComponentPalette, Element, Icon};
use rustcord_i18n::t;

use crate::{line, Look};

/// The notice's height, as Discord's.
pub const NOTICE_HEIGHT: f32 = 40.0;
/// Discord's streamer purple, measured on its notice.
pub const PURPLE: Color = Color::rgba(145, 70, 255, 255);

macro_rules! layer {
    ($name:literal) => {
        include_str!(concat!(
            "../../../assets/illustrations/streamer/",
            $name,
            ".svg"
        ))
    };
}
/// The layers of Rustcord's own drawing, bottom first, each in its color.
const DRAWING: [(&str, Color); 10] = [
    (layer!("body"), Color::rgba(88, 101, 242, 255)),
    (layer!("camera"), Color::rgba(60, 69, 165, 255)),
    (layer!("screen"), Color::rgba(27, 29, 69, 255)),
    (layer!("eye"), Color::rgba(165, 166, 255, 255)),
    (layer!("rec"), Color::rgba(255, 92, 168, 255)),
    (layer!("headset"), Color::rgba(156, 124, 255, 255)),
    (layer!("mic"), Color::rgba(123, 92, 246, 255)),
    (layer!("keyboard"), Color::rgba(140, 213, 255, 255)),
    (layer!("keys"), Color::rgba(88, 101, 242, 255)),
    (layer!("mouse"), Color::rgba(197, 179, 255, 255)),
];

/// What the account's pages show while they hide it: the drawing over
/// the button that says why (`id` opens the mode's settings).
pub fn hidden(id: &str, look: &Look) -> Element {
    // The drawing is 220 by 150; the button as Discord's, in the accent.
    let (width, height) = (220.0, 150.0);
    let drawing = Element::stack(
        DRAWING
            .iter()
            .map(|(svg, color)| {
                Element::icon(Icon::Custom(svg))
                    .large_icon(width)
                    .foreground(*color)
                    .size(Some(width), Some(height))
            })
            .collect(),
    )
    .size(Some(width), Some(height));
    let button = Button::new(id, t("Modo Streamer activado"), look.accent_palette()).render();
    Element::column(vec![
        Element::spacer().flex(1.0),
        drawing.centered_x(),
        button.centered_x(),
        Element::spacer().flex(1.0),
    ])
    .gap(16.0)
}

/// The notice across the window's top: what is on, «Desactivar» (`off`),
/// and the × that hides it (`dismiss`), clear of the window's buttons
/// (`room` at the right end).
pub fn notice(off: &str, dismiss: &str, room: f32, look: &Look) -> Element {
    let white = Color::rgba(255, 255, 255, 255);
    // Ink and line in white on the purple, as Discord's.
    let on_purple = ComponentPalette {
        surface: PURPLE,
        raised: Color::rgba(255, 255, 255, 36),
        hover: Color::rgba(255, 255, 255, 36),
        selected: Color::rgba(255, 255, 255, 56),
        border: white,
        text: white,
        muted: white,
        ..look.palette()
    };
    let deactivate = Button::new(off, t("Desactivar"), on_purple)
        .variant(ButtonVariant::Outline)
        .render()
        .size(None, Some(24.0))
        .centered_y();
    let close = Button::new(dismiss, "", on_purple)
        .icon(Icon::Close)
        .variant(ButtonVariant::Ghost)
        .accessibility_label(t("Ocultar el aviso"))
        .render()
        .centered_y();
    Element::row(vec![
        Element::spacer().flex(1.0),
        line(t("El modo Streamer está activado."), 14.0, white)
            .font_family(limn::FontFamily::SansMedium)
            .centered_y(),
        deactivate,
        Element::spacer().flex(1.0),
        close,
        Element::spacer().size(Some(8.0 + room), None),
    ])
    .gap(10.0)
    .size(None, Some(NOTICE_HEIGHT))
    .background(PURPLE)
}

/// `link` with its last part dotted, as an invitation shows while hidden:
/// `discord.gg/abc` is `discord.gg/•••`.
pub fn dotted(link: &str) -> String {
    let (base, code) = link
        .rsplit_once('/')
        .map_or(("", link), |(base, code)| (base, code));
    let dots = "•".repeat(code.chars().count());
    if base.is_empty() && !link.contains('/') {
        dots
    } else {
        format!("{base}/{dots}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hidden_link_keeps_its_site_and_dots_its_code() {
        assert_eq!(
            dotted("https://discord.gg/abc12"),
            "https://discord.gg/•••••"
        );
        assert_eq!(dotted("abc"), "•••");
    }

    #[test]
    fn every_layer_of_the_drawing_is_one_color_of_the_same_size() {
        for (svg, _) in DRAWING {
            assert!(svg.contains(r#"viewBox="0 0 220 150""#) && svg.contains("currentColor"));
        }
    }
}
