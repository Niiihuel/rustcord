//! Desktop session and games: starting in the
//! tray, the account's activity sharing, the registered games and the
//! providers' proxy (`rustcord-ui`, `rustcord-provider`, `rustcord-discord`).

pub(super) const ENTRIES: super::Entries = &[
    // Starting in the tray.
    (
        "Sin bandeja del sistema, Rustcord abre su ventana igual.",
        "Without a system tray, Rustcord opens its window anyway.",
    ),
    (
        "Primero activa «Iniciar con la sesión».",
        "Turn on «Start with the session» first.",
    ),
    (
        "iniciar minimizado en la bandeja segundo plano sin ventana",
        "start minimized in the tray background without window",
    ),
    // The account's activity sharing.
    ("Qué actividad comparto", "What activity I share"),
    ("Compartir mi actividad", "Share my activity"),
    (
        "Comparte información de actividades de juegos y aplicaciones conectadas que utilizo, lo que incluye cuándo y cómo participo.",
        "Share activity information from games and connected apps I use, including when and how I participate.",
    ),
    (
        "Leyendo el ajuste de tu cuenta…",
        "Reading your account's setting…",
    ),
    (
        "Tu cuenta oculta tu actividad: aunque Rustcord detecte un juego, nadie lo ve.",
        "Your account hides your activity: even if Rustcord detects a game, nobody sees it.",
    ),
    ("Activando…", "Turning on…"),
    (
        "Tu cuenta la muestra a los demás.",
        "Your account shows it to others.",
    ),
    (
        "La sesión la recibió; todavía no se sabe si tu cuenta la muestra.",
        "The session received it; it isn't known yet whether your account shows it.",
    ),
    (
        "Tu cuenta la oculta: nadie la ve.",
        "Your account hides it: nobody sees it.",
    ),
    (
        "Tu cuenta no la muestra: revisa tus juegos y la privacidad de cada servidor.",
        "Your account doesn't show it: check your games and each server's privacy.",
    ),
    (
        "Esperando que tu cuenta la confirme.",
        "Waiting for your account to confirm it.",
    ),
    (
        "compartir mi actividad compartir tu actividad mostrar el juego a los demás visible oculta cuenta",
        "share my activity share your activity show the game to others visible hidden account",
    ),
    (
        "cambiar si se comparte tu actividad",
        "change whether your activity is shared",
    ),
    // Registered games.
    ("Juegos registrados", "Registered games"),
    ("Ningún juego detectado", "No game detected"),
    (
        "Cuando abras un juego que Rustcord reconozca, aparece aquí.",
        "When you open a game Rustcord recognizes, it shows up here.",
    ),
    (
        "Activa «Detectar juegos» para que Rustcord reconozca los que abres.",
        "Turn on «Detect games» so Rustcord recognizes the ones you open.",
    ),
    ("Ahora: {name}", "Now: {name}"),
    (
        "Detectado en este equipo. ¿No es este juego? Ocúltalo y no se detecta más.",
        "Detected on this computer. Not this game? Hide it and it won't be detected again.",
    ),
    ("No es este juego", "Not this game"),
    ("Nombre del juego", "Game name"),
    ("Agregar un programa", "Add a program"),
    (
        "¿No ves tu juego? Elige uno de tus programas abiertos: Rustcord lo detecta como juego desde ahora, sin el proveedor.",
        "Not seeing your game? Choose one of your open programs: Rustcord detects it as a game from now on, without the provider.",
    ),
    (
        "Leyendo los programas abiertos…",
        "Reading the open programs…",
    ),
    (
        "No hay otros programas abiertos.",
        "There are no other open programs.",
    ),
    ("Juego actual", "Current game"),
    ("Juegos añadidos", "Added games"),
    (
        "Controlar quién puede ver la información de actividades de juegos y aplicaciones conectadas que utilizo.",
        "Control who sees activity information from games and connected apps I use.",
    ),
    (
        "privacidad de la actividad juegos aplicaciones conectadas quién ve controlar",
        "activity privacy games connected apps who sees control",
    ),
    (
        "Todavía ninguno. Los que Rustcord detecte se suman solos, para renombrarlos u ocultarlos.",
        "None yet. The ones Rustcord detects join by themselves, to rename or hide them.",
    ),
    ("Restaurar", "Restore"),
    ("Dejar de detectar", "Stop detecting"),
    ("Detectado", "Detected"),
    ("Agregado a mano", "Added by hand"),
    ("Oculto: no se detecta.", "Hidden: not detected."),
    ("Última vez: {day}", "Last time: {day}"),
    ("Juego sin nombre", "Unnamed game"),
    (
        "juego actual detectado ahora no es este juego renombrar",
        "current game detected now not this game rename",
    ),
    (
        "agregar un programa como juego no ves tu juego",
        "add a program as a game not seeing your game",
    ),
    (
        "juegos agregados detectados ocultos renombrar quitar restaurar última vez jugado",
        "added games detected hidden rename remove restore last time played",
    ),
    // The providers' proxy.
    ("Proxy", "Proxy"),
    ("Proxy para la API", "Proxy for the API"),
    ("Automático (el del sistema)", "Automatic (the system's)"),
    ("Sin proxy", "No proxy"),
    ("Esta dirección", "This address"),
    ("Dirección del proxy", "Proxy address"),
    ("Ahora: {url}", "Now: {url}"),
    (
        "Solo para las llamadas a la API. La conexión en vivo y la voz no pasan por él. Se aplica la próxima vez que abras Rustcord.",
        "Only for API calls. The live connection and voice don't go through it. It applies the next time you open Rustcord.",
    ),
    (
        "proxy http https api rest red intermediario dirección sin proxy",
        "proxy http https api rest network intermediary address no proxy",
    ),
    (
        "Usa la dirección de un proxy HTTP o HTTPS, sin usuario ni contraseña, ruta, consulta ni fragmento: http://host:puerto.",
        "Use the address of an HTTP or HTTPS proxy, without user or password, path, query or fragment: http://host:port.",
    ),
    (
        "La dirección del proxy no es válida.",
        "The proxy's address isn't valid.",
    ),
    ("Inicia Rustcord en segundo plano para no molestarte.", "Start Rustcord in the background, out of your way."),
    ("Inicia Rustcord minimizado", "Start Rustcord minimized"),
];
