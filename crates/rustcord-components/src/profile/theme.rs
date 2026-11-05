//! A profile's own colors, as Discord's web client paints them (read
//! 2026-11-05, `docs/discord-reference.md`'s «Profile theme colors»): a
//! gradient frame around the card, veils over it mixed toward the colors,
//! text leaning toward them, and buttons in them.

use limn::{Color, ComponentPalette, CornerRadii, Element};
use rustcord_model::Rgb;

use crate::Look;

/// The colors a themed card draws with, worked out from its two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileTheme {
    pub primary: Color,
    pub secondary: Color,
    /// Light text on dark veils; else dark text on light ones.
    pub dark: bool,
    /// The card's veil, its boxes' and the note's.
    pub(super) veil: Color,
    pub(super) boxed: Color,
    pub(super) note: Color,
    /// The mark's ring: the veil over the primary, opaque, as the hole cut in
    /// the banner shows it.
    pub(super) ring: Color,
    /// What the text leans toward.
    ink: Color,
    /// The primary button at rest, under the pointer and pressed.
    pub(super) button: [Color; 3],
    /// The other buttons, the same three.
    pub(super) secondary_button: [Color; 3],
    /// A role's pill and its outline.
    pub(super) role: (Color, Color),
}

const BLACK: Color = Color::rgb(0, 0, 0);

impl ProfileTheme {
    /// The theme of `primary` (under the banner) and `secondary` (at the foot).
    pub fn new(primary: Rgb, secondary: Rgb) -> Self {
        let rgb = |color: Rgb| {
            let (r, g, b) = color.channels();
            Color::rgb(r, g, b)
        };
        let (primary, secondary) = (rgb(primary), rgb(secondary));
        let dark = darkness(primary) > 0.5;
        let (darker, lighter) = if darkness(primary) >= darkness(secondary) {
            (primary, secondary)
        } else {
            (secondary, primary)
        };
        // Black veils lean a third toward the darker color, white ones toward
        // the lighter; the text toward the other.
        let (veil, toward, ink) = if dark {
            (BLACK, with(darker, 0.05), {
                let (hue, ..) = hsl(lighter);
                with(lighter, if blue(hue) { 0.8 } else { 0.95 })
            })
        } else {
            (Color::WHITE, with(lighter, 0.9), {
                let (hue, ..) = hsl(darker);
                with(darker, if blue(hue) { 0.25 } else { 0.15 })
            })
        };
        let veil = mix(veil, toward, 1.0 / 3.0);
        let at = |share: f32| Look::alpha(veil, (share * 255.0).round() as u8);
        let button = legible(primary);
        let shade = if dark { BLACK } else { Color::WHITE };
        let white = |alphas: [u8; 3]| alphas.map(|alpha| Look::alpha(Color::WHITE, alpha));
        Self {
            primary,
            secondary,
            dark,
            veil: at(0.6),
            boxed: at(0.45),
            note: at(0.3),
            ring: mix(primary, veil, 0.6),
            ink,
            button: [button, mix(button, shade, 0.1), mix(button, shade, 0.2)],
            // White at 8, 16 and 12 % on a dark card; 72, 92 and 84 % on a light one.
            secondary_button: white(if dark { [20, 41, 31] } else { [184, 235, 214] }),
            role: (Look::alpha(primary, 128), Look::alpha(Color::WHITE, 51)),
        }
    }

    /// `look` as the card draws over its veil: the text of a look as dark
    /// as the card, a quarter of the way to the theme's ink; the card's
    /// grounds its veils.
    pub(super) fn look(&self, look: &Look) -> Look {
        let base = match (self.dark, look.canvas.r > 128) {
            (true, false) | (false, true) => *look,
            (true, true) => Look::ONYX,
            (false, false) => Look::LIGHT,
        };
        let ink = |color: Color| mix(color, self.ink, 0.25);
        Look {
            card: self.veil,
            menu: self.veil,
            raised: self.boxed,
            text: ink(base.text),
            muted: ink(base.muted),
            dim: ink(base.dim),
            faint: ink(base.faint),
            accent: look.accent,
            on_accent: look.on_accent,
            ..base
        }
    }

    /// The primary button's colors: white on the primary made legible.
    pub(super) fn button_palette(&self, look: &Look) -> ComponentPalette {
        let [raised, hover, selected] = self.button;
        ComponentPalette {
            raised,
            hover,
            selected,
            text: Color::WHITE,
            ..look.palette()
        }
    }

    /// `content` on the card: Discord's gradient (the primary down to the
    /// foot of a `banner` px banner, the secondary from `footer` px over the
    /// bottom) with `edge` of it around the content's veil, rounded by
    /// `outer` outside and as much less the edge inside.
    pub(super) fn frame(
        &self,
        content: Element,
        banner: f32,
        footer: f32,
        edge: f32,
        outer: CornerRadii,
    ) -> Element {
        let band = |color: Color, height: f32| {
            Element::panel(Element::spacer())
                .size(None, Some(height))
                .background(color)
        };
        let less = |radius: f32| (radius - edge).max(0.0);
        let inner = CornerRadii {
            top_left: less(outer.top_left),
            top_right: less(outer.top_right),
            bottom_right: less(outer.bottom_right),
            bottom_left: less(outer.bottom_left),
        };
        Element::stack(vec![
            Element::column(vec![
                band(self.primary, edge + banner),
                Element::panel(Element::spacer())
                    .vertical_gradient([self.primary, self.secondary])
                    .flex(1.0),
                band(self.secondary, footer + edge),
            ]),
            Element::panel(
                Element::panel(content)
                    .background(self.veil)
                    .corner_radii(inner)
                    .clip_children(),
            )
            .padding(edge),
        ])
        .corner_radii(outer)
        .clip_children()
    }
}

/// How dark `color` reads: 1 less its weighted brightness, from 0 to 1.
fn darkness(color: Color) -> f32 {
    1.0 - (0.299 * f32::from(color.r) + 0.587 * f32::from(color.g) + 0.114 * f32::from(color.b))
        / 255.0
}

/// A hue Discord lifts or deepens less: the blues and violets, 200 to 300.
fn blue(hue: f32) -> bool {
    (200.0..=300.0).contains(&hue)
}

/// `color`'s hue fully saturated at `lightness`.
fn with(color: Color, lightness: f32) -> Color {
    let (hue, ..) = hsl(color);
    from_hsl(hue, 1.0, lightness)
}

/// `from` moved `share` of the way to `to`, its alpha too.
fn mix(from: Color, to: Color, share: f32) -> Color {
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * share).round() as u8;
    Color::rgba(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
        channel(from.a, to.a),
    )
}

/// Hue in degrees, saturation and lightness from 0 to 1.
pub(crate) fn hsl(color: Color) -> (f32, f32, f32) {
    let [r, g, b] = [color.r, color.g, color.b].map(|channel| f32::from(channel) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let (lightness, span) = ((max + min) / 2.0, max - min);
    if span == 0.0 {
        return (0.0, 0.0, lightness);
    }
    let saturation = span / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if max == r {
        ((g - b) / span).rem_euclid(6.0)
    } else if max == g {
        (b - r) / span + 2.0
    } else {
        (r - g) / span + 4.0
    };
    (sector * 60.0, saturation, lightness)
}

pub(crate) fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let second = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match (hue / 60.0).rem_euclid(6.0) as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let base = lightness - chroma / 2.0;
    let channel = |value: f32| ((value + base) * 255.0).round().clamp(0.0, 255.0) as u8;
    Color::rgb(channel(r), channel(g), channel(b))
}

/// WCAG's contrast of `color` against white.
fn contrast_with_white(color: Color) -> f32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b);
    1.05 / (luminance + 0.05)
}

/// `color` darkened or lightened until white on it reads 7 to 10 to 1.
/// shortcut: steps its HSL lightness by 1 %, where Discord moves it in
/// chroma.js; switch to its exact steps if the buttons look off.
fn legible(color: Color) -> Color {
    let (hue, saturation, mut lightness) = hsl(color);
    for _ in 0..100 {
        let shown = from_hsl(hue, saturation, lightness);
        match contrast_with_white(shown) {
            ratio if ratio < 7.0 => lightness = (lightness - 0.01).max(0.0),
            ratio if ratio > 10.0 => lightness = (lightness + 0.01).min(1.0),
            _ => return shown,
        }
    }
    from_hsl(hue, saturation, lightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dark_primary_makes_a_dark_card_with_veils_and_a_legible_button() {
        let theme = ProfileTheme::new(Rgb(0x2B_6C_B0), Rgb(0x9F_7A_EA));
        assert!(theme.dark);
        assert_eq!(
            (theme.veil.a, theme.boxed.a, theme.note.a),
            (153, 115, 77),
            "60, 45 and 30 %"
        );
        assert!(theme.veil.r < 20 && theme.veil.b < 30, "nearly black");
        assert_eq!(theme.ring.a, 255, "the ring is opaque");
        let ratio = contrast_with_white(theme.button[0]);
        assert!((7.0..=10.0).contains(&ratio), "{ratio}");
        assert!(
            theme.button[1].r <= theme.button[0].r,
            "darker under the pointer"
        );
        assert_eq!(theme.secondary_button.map(|color| color.a), [20, 41, 31]);
        // The text leans toward a light ink.
        let look = theme.look(&Look::ONYX);
        assert_ne!(look.text, Look::ONYX.text);
        assert_eq!((look.card, look.raised), (theme.veil, theme.boxed));
    }

    #[test]
    fn a_light_primary_makes_a_light_card_with_dark_text_even_on_a_dark_look() {
        let theme = ProfileTheme::new(Rgb(0xF5_E6_C8), Rgb(0xFF_C0_CB));
        assert!(!theme.dark);
        assert!(theme.veil.r > 200, "white veils");
        let look = theme.look(&Look::ONYX);
        assert!(look.text.r < 80, "dark text: {:?}", look.text);
        assert_eq!(theme.secondary_button.map(|color| color.a), [184, 235, 214]);
    }

    #[test]
    fn colors_go_to_hsl_and_back() {
        for color in [
            Color::rgb(43, 108, 176),
            Color::rgb(255, 0, 0),
            Color::rgb(10, 200, 90),
            Color::rgb(128, 128, 128),
        ] {
            let (hue, saturation, lightness) = hsl(color);
            assert_eq!(from_hsl(hue, saturation, lightness), color);
        }
        assert!((contrast_with_white(BLACK) - 21.0).abs() < 0.01);
    }
}
