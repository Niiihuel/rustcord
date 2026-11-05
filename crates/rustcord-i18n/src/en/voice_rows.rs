//! People's rows under a voice channel in the column: the card of someone
//! live and carrying someone to another channel (`rustcord-components`,
//! `rustcord-ui`); and the question before a row moves the call to another
//! channel, with its setting.

pub(super) const ENTRIES: super::Entries = &[
    ("Transmisión de {name}", "{name}'s stream"),
    ("Cargando la vista previa…", "Loading the preview…"),
    ("Vista previa no disponible", "Preview unavailable"),
    (
        "Entra al canal de voz para ver la transmisión",
        "Join the voice channel to watch the stream",
    ),
    // Moving the call to another channel.
    ("¿Confirmas?", "Are you sure?"),
    (
        "Parece que estás en otro canal de voz. ¿Realmente quieres cambiar a {channel}?",
        "Looks like you're in another voice channel. Are you sure you want to switch to {channel}?",
    ),
    ("No volver a preguntar", "Don't ask again"),
    ("Cancelar", "Cancel"),
    ("Confirmar", "Confirm"),
    (
        "Cambiar la advertencia del canal de voz",
        "Voice channel switch warning",
    ),
    (
        "Mostrar un aviso de confirmación antes de cambiar a otro canal de voz.",
        "Show a confirmation prompt before switching to another voice channel.",
    ),
    (
        "advertencia aviso confirmación cambiar de canal de voz preguntar antes",
        "warning prompt confirmation switch voice channel ask first",
    ),
];
