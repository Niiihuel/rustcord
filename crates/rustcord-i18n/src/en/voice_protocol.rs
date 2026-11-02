//! A call alone in its channel
//! (`rustcord-components/src/voice.rs`) and how long a voice room has had
//! someone in it (`rustcord-components/src/sidebar.rs`).

pub(super) const ENTRIES: super::Entries = &[
    // A call alone in its channel.
    ("Esperando a los demás", "Waiting for others"),
    ("Nadie más en la llamada", "Nobody else in the call"),
    (
        "Esperando a los demás (simulada)",
        "Waiting for others (simulated)",
    ),
    (
        "Nadie más en la llamada simulada",
        "Nobody else in the simulated call",
    ),
    // How long a voice room has been on.
    ("En voz desde hace {since}", "In voice for {since}"),
];
