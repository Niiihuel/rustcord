//! «Lectura»: GIFs that
//! play on their own or under the pointer, the links of the pictures
//! shown and the speed of the wheel; and Rustcord's own, the member list in
//! narrow windows.

use limn::{Component, Element};

use rustcord_components::{line, user_settings as parts};
use rustcord_i18n::{es, t, tr};

use super::{Ctx, Section};
use crate::reading::speed_fraction;
use crate::settings::SettingsPage::Appearance;

pub(super) const SECTIONS: &[Section] = &[Section {
    page: Appearance,
    key: "reading",
    title: es("Lectura"),
    // «Reproducir automáticamente los GIF» is in Accesibilidad, as Discord's,
    // and the media links' switch in «Mensajes», under the pictures'.
    rows: &[
        (
            "scroll-speed",
            es("velocidad de desplazamiento rueda del mouse ratón trackpad panel táctil scroll"),
        ),
        (
            "members-narrow",
            es("mantener la lista de miembros en ventanas angostas personas panel chica"),
        ),
    ],
}];

/// The setting `key` of the section; `None` for the rest.
pub(super) fn setting(ctx: &Ctx, key: &str) -> Option<Element> {
    let (ui, look) = (ctx.ui, ctx.look);
    let prefs = &ui.prefs;
    let toggle = |id: &str, label: &str, note: &str, on: bool| {
        parts::toggle(id, label, Some(note), on, None, look)
    };
    Some(match key {
        "animate-gifs" => toggle(
            "us:animate-gifs",
            t("Reproducir automáticamente los GIF cuando Rustcord está en uso"),
            if ui.reduced_motion {
                t("Con el movimiento reducido no se anima nada, ni al pasar el puntero.")
            } else {
                t("Los GIF, stickers y emoji animados se mueven solos. Apagado, muestran su primer cuadro y se animan al pasar el puntero.")
            },
            prefs.animate_gifs,
        ),
        "hide-media-links" => toggle(
            "us:hide-media-links",
            t("Ocultar los enlaces de imagen y GIF"),
            t("Si un mensaje es sólo el enlace de una imagen o un GIF que se muestra, se ve la imagen sin el enlace."),
            prefs.hide_media_links,
        ),
        "scroll-speed" => Element::column(vec![
            parts::setting(
                t("Velocidad de desplazamiento"),
                Some(t("Rueda del mouse y trackpad. 100 % es lo normal.")),
                Some(parts::secondary_button(
                    "us:scroll-speed-reset",
                    t("Restablecer"),
                    prefs.scroll_speed != crate::Prefs::default().scroll_speed,
                    look,
                )),
                true,
                look,
            ),
            parts::slider(
                "us:scroll-speed",
                speed_fraction(prefs.scroll_speed),
                look,
            )
            .width(ctx.geometry.column)
            .accessibility_label(t("Velocidad de desplazamiento"))
            .render(),
            line(
                tr!("{percent} %", percent = prefs.scroll_speed),
                13.0,
                look.muted,
            ),
            Element::spacer().size(None, Some(8.0)),
        ])
        .gap(8.0),
        "members-narrow" => toggle(
            "us:members-narrow",
            t("Mantener la lista de miembros en ventanas angostas"),
            t("La lista de miembros de un servidor queda abierta aunque la ventana sea angosta: la conversación se angosta en su lugar."),
            prefs.members_narrow,
        ),
        _ => return None,
    })
}
