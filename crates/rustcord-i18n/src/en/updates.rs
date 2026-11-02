//! Rustcord's own updates: what `rustcord-update` reports, the
//! mandatory update screen, the notice of an optional one and Ajustes ›
//! Actualizaciones.

pub(super) const ENTRIES: super::Entries = &[
    // rustcord-update.
    (
        "Esta compilación no tiene la clave de las actualizaciones.",
        "This build has no update key.",
    ),
    (
        "No se pudo conectar con el servidor de actualizaciones.",
        "Could not reach the update server.",
    ),
    (
        "El servidor de actualizaciones respondió {status}.",
        "The update server answered {status}.",
    ),
    (
        "La respuesta del servidor de actualizaciones es demasiado grande.",
        "The update server's answer is too large.",
    ),
    (
        "La firma de la actualización no es válida: no se instaló nada.",
        "The update's signature is not valid: nothing was installed.",
    ),
    (
        "No se pudo leer la descripción de la actualización.",
        "Could not read the update's description.",
    ),
    (
        "Esta versión no tiene descarga para este sistema.",
        "This version has no download for this system.",
    ),
    (
        "La descarga quedó incompleta: se retoma al reintentar.",
        "The download is incomplete: retrying resumes it.",
    ),
    (
        "Lo descargado no coincide con lo firmado: se borró.",
        "What was downloaded does not match what was signed: it was deleted.",
    ),
    (
        "No se pudo escribir la actualización en el disco.",
        "Could not write the update to disk.",
    ),
    (
        "Esta copia de Rustcord no se instaló con su instalador: actualízala desde donde la compilaste.",
        "This copy of Rustcord was not installed by its installer: update it where you built it.",
    ),
    (
        "La actualización se interrumpió. Reintenta para completarla.",
        "The update was interrupted. Try again to finish it.",
    ),
    // rustcord-components: the mandatory screen and the optional notice.
    ("Actualización obligatoria", "Mandatory update"),
    (
        "Rustcord {version} reemplaza a la {installed} que tienes instalada, que ya no se puede usar. Rustcord se actualiza antes de abrir.",
        "Rustcord {version} replaces the {installed} you have installed, which can no longer be used. Rustcord updates before it opens.",
    ),
    ("Rustcord {version} está disponible", "Rustcord {version} is available"),
    (
        "Tienes la {installed}. Se instala y Rustcord vuelve a abrir.",
        "You have {installed}. It installs and Rustcord opens again.",
    ),
    ("Actualizar y reiniciar", "Update and restart"),
    ("Más tarde", "Later"),
    ("Notas", "Notes"),
    ("Descargando… {have} de {whole}", "Downloading… {have} of {whole}"),
    ("Instalando…", "Installing…"),
    ("Reiniciando Rustcord…", "Restarting Rustcord…"),
    (
        "Instalada; reinicia Rustcord para usarla.",
        "Installed; restart Rustcord to use it.",
    ),
    ("Reiniciar ahora", "Restart now"),
    ("Reintentar", "Try again"),
    ("Salir", "Quit"),
    // rustcord-ui: Ajustes › Actualizaciones.
    ("Actualizaciones", "Updates"),
    (
        "versión instalada actual número dónde instalación",
        "installed version current number where installation",
    ),
    (
        "buscar actualizaciones ahora nueva versión descargar instalar reiniciar última verificación",
        "check for updates now new version download install restart last check",
    ),
    (
        "buscar al iniciar abrir comprobar automáticamente obligatoria",
        "check at start open look automatically mandatory",
    ),
    (
        "Instalada en {folder}. Una actualización la reemplaza entera, o no la toca.",
        "Installed in {folder}. An update replaces it whole, or leaves it alone.",
    ),
    (
        "Compilación propia: avisa si hay una versión nueva, pero se actualiza desde tu código.",
        "Custom build: it tells you when there is a new version, but you update from your source.",
    ),
    ("Versión instalada", "Installed version"),
    ("Instalando Rustcord {version}…", "Installing Rustcord {version}…"),
    ("Novedades de Rustcord {version}", "What's new in Rustcord {version}"),
    ("Ver en GitHub", "View on GitHub"),
    (
        "novedades notas cambios versión github imágenes videos",
        "whats new notes changes version github pictures videos",
    ),
    ("Buscar ahora", "Check now"),
    ("Buscar al iniciar", "Check at start"),
    (
        "Al abrir, Rustcord pregunta si hay una versión nueva, con unos segundos de límite. Una obligatoria se instala antes de abrir; las demás se ofrecen sin interrumpir.",
        "When it opens, Rustcord asks whether there is a new version, for a few seconds at most. A mandatory one installs before it opens; the rest are offered without interrupting.",
    ),
    // The settings card (rustcord-components::updates::check_card).
    ("Buscando actualizaciones…", "Checking for updates…"),
    ("Disponible", "Available"),
    ("Descargando Rustcord {version}", "Downloading Rustcord {version}"),
    ("Descargando", "Downloading"),
    ("Instalando", "Installing"),
    ("Rustcord {version} está instalada", "Rustcord {version} is installed"),
    ("Instalada", "Installed"),
    ("No se pudo actualizar", "Could not update"),
    ("Error", "Error"),
    (
        "No se pudo buscar actualizaciones",
        "Could not check for updates",
    ),
    ("Tienes la última versión", "You have the latest version"),
    ("Al día", "Up to date"),
    ("Todavía no se buscó", "Not checked yet"),
    (
        "Instalada: Rustcord {installed}. Última verificación: {when}.",
        "Installed: Rustcord {installed}. Last checked: {when}.",
    ),
    ("Instalada: Rustcord {installed}.", "Installed: Rustcord {installed}."),
    ("{have} de {whole}", "{have} of {whole}"),
];
