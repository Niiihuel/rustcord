//! Windows' media and desktop: the screen picker of Rustcord,
//! the camera, the screen's sound, games, discord-ipc and the system's
//! fonts on Windows (`rustcord-ui`, `rustcord-media`).

pub(super) const ENTRIES: super::Entries = &[
    (
        "La del texto de toda la ventana. Si desinstalas una fuente, Rustcord vuelve a la suya.",
        "The text font for the whole window. If you uninstall a font, Rustcord goes back to its own.",
    ),
    (
        "Otros programas de este equipo, como juegos o reproductores, pueden mostrar lo que haces. Solo ven tu usuario, sin acceso a la red.",
        "Other programs on this computer, like games or players, can show what you're doing. They only see your user, with no network access.",
    ),
    // Rustcord's own screen picker and the screen (rustcord-media, rustcord-components).
    ("Pantallas", "Screens"),
    ("Ventanas", "Windows"),
    (
        "Pantalla {number} (principal) · {width}×{height}",
        "Screen {number} (main) · {width}×{height}",
    ),
    ("Pantalla {number} · {width}×{height}", "Screen {number} · {width}×{height}"),
    (
        "Esta versión de Windows no permite capturar la pantalla: hace falta Windows 10 (1903) o posterior.",
        "This version of Windows can't capture the screen: it takes Windows 10 (1903) or later.",
    ),
    ("La ventana elegida ya no está abierta.", "The chosen window isn't open anymore."),
    (
        "El sonido de la pantalla necesita Windows 10 (versión 2004) o posterior.",
        "The screen's sound needs Windows 10 (version 2004) or later.",
    ),
    // The camera (rustcord-media).
    (
        "Windows no deja que las aplicaciones usen la cámara: permítelo en Configuración › Privacidad y seguridad › Cámara.",
        "Windows doesn't let apps use the camera: allow it in Settings › Privacy & security › Camera.",
    ),
    ("La cámara está en uso por otro programa.", "The camera is in use by another program."),
    (
        "La cámara no ofrece imágenes que Rustcord pueda leer.",
        "The camera doesn't offer pictures Rustcord can read.",
    ),
    (
        "Este Windows no tiene Media Foundation (en las ediciones N, instala el Media Feature Pack).",
        "This Windows doesn't have Media Foundation (on N editions, install the Media Feature Pack).",
    ),
    // Video settings (rustcord-ui).
    (
        "En Windows, el video todavía se procesa en el procesador.",
        "On Windows, video is still processed by the processor.",
    ),
];
