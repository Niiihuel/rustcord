//! «Modo de transmisión en vivo», at the end of «Apariencia» as Discord's:
//! the mode, its turn by OBS or XSplit, and what it hides and silences.

use limn::Element;

use rustcord_components::user_settings as parts;
use rustcord_i18n::{es, t};

use super::{Ctx, Section};
use crate::settings::SettingsPage::Appearance;

pub(super) const SECTIONS: &[Section] = &[Section {
    page: Appearance,
    key: "streamer",
    title: es("Modo de transmisión en vivo"),
    rows: &[
        (
            "streamer",
            es("activar modo streamer transmisión en vivo ocultar información personal"),
        ),
        (
            "streamer-auto",
            es("activar automáticamente obs xsplit programas transmitir"),
        ),
        (
            "streamer-when",
            es("si el modo de transmisión en vivo está activado"),
        ),
        (
            "streamer-personal",
            es("ocultar información personal correo electrónico cuentas vinculadas notas"),
        ),
        (
            "streamer-invites",
            es("ocultar enlaces de invitación servidores código"),
        ),
        (
            "streamer-sounds",
            es("desactivar efectos de sonido silenciar"),
        ),
        (
            "streamer-notifications",
            es("desactivar notificaciones avisos escritorio"),
        ),
    ],
}];

/// The setting `key` of the section; `None` for the rest.
pub(super) fn setting(ctx: &Ctx, key: &str) -> Option<Element> {
    let look = ctx.look;
    let streamer = ctx.ui.prefs.streamer;
    let toggle = |id: &str, label: &str, note: Option<&str>, on: bool, denied: Option<&str>| {
        parts::toggle(id, label, note, on, denied, look)
    };
    Some(match key {
        "streamer" => toggle(
            "us:streamer=on",
            t("Activar modo Streamer"),
            Some(t("El modo de transmisión en vivo oculta tu información personal para que otros que ven tu transmisión no puedan verla.")),
            streamer.on,
            None,
        ),
        // The programs' names are what is read, as for the games; nowhere else.
        "streamer-auto" => toggle(
            "us:streamer=auto",
            t("Activa automáticamente el modo de transmisión en vivo si OBS o XSplit están en funcionamiento"),
            Some(t("Rustcord mira los nombres de los programas abiertos cada 10 segundos.")),
            streamer.auto,
            (!cfg!(any(target_os = "linux", windows)))
                .then(|| t("No disponible en este sistema.")),
        ),
        "streamer-when" => parts::setting(
            t("Si el modo de transmisión en vivo está activado..."),
            None,
            None,
            true,
            look,
        ),
        "streamer-personal" => toggle(
            "us:streamer=personal",
            t("Ocultar mi información personal, como el correo electrónico, las cuentas vinculadas y las notas"),
            None,
            streamer.hide_personal,
            None,
        ),
        "streamer-invites" => toggle(
            "us:streamer=invites",
            t("Ocultar los enlaces de invitación a mis servidores"),
            None,
            streamer.hide_invites,
            None,
        ),
        "streamer-sounds" => toggle(
            "us:streamer=sounds",
            t("Desactiva todos los efectos de sonido"),
            None,
            streamer.mute_sounds,
            None,
        ),
        "streamer-notifications" => toggle(
            "us:streamer=notifications",
            t("Desactivar notificaciones"),
            None,
            streamer.mute_notifications,
            None,
        ),
        _ => return None,
    })
}
