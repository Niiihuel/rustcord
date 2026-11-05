//! The desktop host (`apps/desktop`).

pub(super) const ENTRIES: super::Entries = &[
    // The providers it can start.
    ("Simulación local", "Local simulation"),
    (
        "Datos inventados en este equipo, sin conexión con ningún servicio.",
        "Made-up data on this computer, with no connection to any service.",
    ),
    ("Discord con OAuth2", "Discord with OAuth2"),
    (
        "Autorización oficial en el navegador: tu identidad y la lista de tus servidores, sin mensajes ni voz.",
        "Official authorization in the browser: your identity and your list of servers, without messages or voice.",
    ),
    ("Discord con sesión personal", "Discord with a personal session"),
    (
        "Tu sesión en una ventana de inicio propia de Rustcord: conversaciones, mensajes y voz, recordada en el llavero del sistema.",
        "Your session, in Rustcord's own sign-in window: conversations, messages and voice, remembered in the system keyring.",
    ),
    (
        "Falta el ID público de tu aplicación: abre Rustcord con --discord-client-id <id> o RUSTCORD_DISCORD_CLIENT_ID.",
        "Your app's public ID is missing: open Rustcord with --discord-client-id <id> or RUSTCORD_DISCORD_CLIENT_ID.",
    ),
    (
        "Esta compilación no incluye el inicio de sesión no oficial: hace falta Linux o Windows y la feature session-login.",
        "This build doesn't include the unofficial sign-in: it needs Linux or Windows and the session-login feature.",
    ),
    ("Falta el ID público de la aplicación", "The app's public ID is missing"),
    (
        "El login no oficial requiere Linux o Windows y compilar con --features session-login. En NixOS, usa nix-shell --arg withLogin true; la interfaz principal sigue siendo limn.",
        "The unofficial sign-in requires Linux or Windows and building with --features session-login. On NixOS, use nix-shell --arg withLogin true; the main interface is still limn.",
    ),
    ("No se pudo cambiar de proveedor", "Couldn't switch providers"),
    // The command line.
    (
        "--provider requiere mock, discord o discord-session",
        "--provider requires mock, discord or discord-session",
    ),
    (
        "--discord-client-id requiere el ID público de la aplicación",
        "--discord-client-id requires the app's public ID",
    ),
    ("--capture-draft requiere texto", "--capture-draft requires text"),
    ("--capture-hover-id requiere un id", "--capture-hover-id requires an id"),
    ("--capture-search requiere texto", "--capture-search requires text"),
    ("--capture-splash requiere milisegundos", "--capture-splash requires milliseconds"),
    (
        "--capture-connection requiere connecting, reconnecting, offline o back",
        "--capture-connection requires connecting, reconnecting, offline or back",
    ),
    ("--language requiere es o en", "--language requires es or en"),
    (
        "--title-bar requiere auto, rustcord o system",
        "--title-bar requires auto, rustcord or system",
    ),
    (
        "argumento desconocido: {other}; opciones: --provider mock|discord|discord-session, --discord-client-id <id>, --bench, --screenshot <archivo>, --channel <nombre>, --action <id>, --capture-accounts, --data, --latency <ms>, --language es|en, --title-bar auto|rustcord|system, o un enlace https://",
        "unknown argument: {other}; options: --provider mock|discord|discord-session, --discord-client-id <id>, --bench, --screenshot <file>, --channel <name>, --action <id>, --data, --latency <ms>, --language es|en, --title-bar auto|rustcord|system, or an https:// link",
    ),
    (
        "--bench y --latency sólo se usan con --provider mock",
        "--bench and --latency are only used with --provider mock",
    ),
    (
        "Las opciones --capture-* requieren --provider mock y --screenshot",
        "The --capture-* options require --provider mock and --screenshot",
    ),
    (
        "Captura búsqueda y contexto en ejecuciones separadas",
        "Capture search and context in separate runs",
    ),
    (
        "Discord requiere el ID público de tu aplicación: --discord-client-id <id> o RUSTCORD_DISCORD_CLIENT_ID. Registra el redirect http://127.0.0.1:48731/oauth/callback en el Developer Portal. No ingreses contraseñas ni tokens.",
        "Discord requires your app's public ID: --discord-client-id <id> or RUSTCORD_DISCORD_CLIENT_ID. Register the redirect http://127.0.0.1:48731/oauth/callback in the Developer Portal. Don't enter passwords or tokens.",
    ),
    (
        "Rustcord ya estaba abierto: se trajo al frente.",
        "Rustcord was already open: it was brought to the front.",
    ),
    // Notices.
    ("No se pudieron restaurar las lecturas", "Couldn't restore read marks"),
    (
        "{error} Las nuevas marcas quedarán sólo en memoria hasta reiniciar o borrar los datos locales.",
        "{error} New read marks will stay in memory only until you restart or delete local data.",
    ),
    ("Sin almacenamiento local", "No local storage"),
    (
        "La conversación seleccionada cambió.",
        "The selected conversation changed.",
    ),
    ("No se pudo revisar la invitación", "Couldn't check the invite"),
    (
        "El enlace ya no es válido para esta conexión.",
        "The link is no longer valid for this connection.",
    ),
    (
        "Este adjunto no tiene un archivo para reproducir.",
        "This attachment has no file to play.",
    ),
    ("Guardando en Descargas…", "Saving to Downloads…"),
    ("No se pudo descargar", "Couldn't download"),
    ("Archivo guardado", "File saved"),
    ("El archivo ya no está disponible.", "The file is no longer available."),
    (
        "No se pudieron consultar los dispositivos.",
        "Couldn't look up the devices.",
    ),
    ("Imagen", "Image"),
    ("Imagen elegida", "Chosen image"),
    ("Imagen simulada", "Simulated image"),
    ("No se pudo mostrar la imagen.", "Couldn't show the image."),
    ("Página {page} del documento", "Page {page} of the document"),
    ("No se pudo mostrar la vista previa.", "Couldn't show the preview."),
    ("Guardando antes de cerrar…", "Saving before closing…"),
    (
        "Esperando la confirmación local. Esc cancela el cierre.",
        "Waiting for local confirmation. Esc cancels closing.",
    ),
    (
        "El almacenamiento local dejó de responder. Tus cambios siguen pendientes.",
        "Local storage stopped responding. Your changes are still pending.",
    ),
    (
        "El cambio supera el límite del almacenamiento local. Esc permite volver a editarlo.",
        "The change exceeds the local storage limit. Press Esc to edit it again.",
    ),
    ("No se pudo guardar en este equipo", "Couldn't save on this computer"),
    ("Datos locales borrados", "Local data deleted"),
    (
        "Se confirmó el borrado de la caché y los borradores de esta cuenta.",
        "Deleting this account's cache and drafts was confirmed.",
    ),
    ("Cierre cancelado", "Closing canceled"),
    // Audio, video and calls.
    (
        "No se pudo usar el audio de la llamada",
        "Couldn't use the call's audio",
    ),
    ("Se cortó el audio de la llamada", "The call's audio cut out"),
    (
        "No se pudo usar la salida de audio",
        "Couldn't use the audio output",
    ),
    ("No se pudo usar el micrófono", "Couldn't use the microphone"),
    ("Entrada cambiada a {name}", "Input switched to {name}"),
    ("Salida cambiada a {name}", "Output switched to {name}"),
    (
        "El sonido dejó de estar disponible.",
        "The sound is no longer available.",
    ),
    // The desktop: start with the session, local socket, fonts, shortcuts.
    (
        "No se encontró la carpeta de configuración.",
        "The configuration folder wasn't found.",
    ),
    (
        "No se pudo cambiar el inicio con la sesión.",
        "Couldn't change starting with the session.",
    ),
    (
        "No se pudo abrir el socket local: puede que otro cliente lo esté usando.",
        "Couldn't open the local socket: another client may be using it.",
    ),
    (
        "No se encontró «{font}» en el sistema: se usa la fuente de Rustcord.",
        "“{font}” wasn't found on the system: Rustcord's font is used.",
    ),
    ("Pulsar para hablar en Rustcord", "Push to talk in Rustcord"),
    ("{shortcut} en Rustcord", "{shortcut} in Rustcord"),
    ("Atajo", "Shortcut"),
    (
        "el escritorio no da los botones del ratón fuera de la ventana.",
        "the desktop doesn't give mouse buttons outside the window.",
    ),
    // Uploads.
    (
        "Todavía no puedes adjuntar archivos arrastrándolos. Usa Adjuntar.",
        "You can't attach files by dragging them yet. Use Attach files.",
    ),
    ("No se pudo reintentar", "Couldn't try again"),
];
