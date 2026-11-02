//! The image and video viewer: its header, its strip and
//! its keys (`rustcord-ui`).

pub(super) const ENTRIES: super::Entries = &[
    ("Imagen {n}", "Image {n}"),
    ("Guardar imagen", "Save Image"),
    ("Guardar video", "Save Video"),
    ("Copiar enlace multimedia", "Copy Media Link"),
    ("Ver detalles", "View Details"),
    ("Nombre de archivo", "Filename"),
    ("Tamaño", "Size"),
    ("Copiado al portapapeles.", "Copied to clipboard."),
    ("Copiar ID del adjunto", "Copy Attachment ID"),
    (
        "Se produjo un error al cargar la imagen.",
        "Image failed to load.",
    ),
    ("Descripción (texto alternativo)", "Description (Alt Text)"),
];
