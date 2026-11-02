//! Video and documents: WebM and MKV videos in conversations
//! (`rustcord-media/src/clip`), highlights in a PDF's search
//! (`rustcord-ui/src/documents.rs`) and Lottie stickers.

pub(super) const ENTRIES: super::Entries = &[
    // WebM and MKV videos.
    (
        "Rustcord no puede leer este archivo WebM o MKV.",
        "Rustcord can't read this WebM or MKV file.",
    ),
    (
        "Este video no dice cuánto dura y no tiene sonido.",
        "This video doesn't say how long it lasts and has no sound.",
    ),
    (
        "Este equipo no tiene el decodificador de video que hace falta.",
        "This computer doesn't have the video decoder it needs.",
    ),
    (
        "Las imágenes de este video son más grandes de lo que Rustcord muestra.",
        "This video's pictures are larger than Rustcord shows.",
    ),
];
