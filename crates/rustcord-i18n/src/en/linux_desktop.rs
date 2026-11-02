//! Linux desktop: replacing a file in «Guardar como»,
//! the folder of the call's sounds chosen in the system's dialog, and the
//! window's decorations and GPU (`rustcord-ui`, `rustcord-core`, `rustcord-platform`).

pub(super) const ENTRIES: super::Entries = &[
    // «Guardar como» over a file already there.
    (
        "Ya hay un archivo con ese nombre",
        "There's already a file with that name",
    ),
    (
        "Reemplazarlo borra el que está ahí. Cancela para elegir otro nombre.",
        "Replacing it deletes the one that's there. Cancel to choose another name.",
    ),
    ("Reemplazar", "Replace"),
    (
        "No se pudo reemplazar el archivo en esa carpeta.",
        "Couldn't replace the file in that folder.",
    ),
    (
        "El archivo cambió mientras se descargaba. Vuelve a guardarlo.",
        "The file changed while downloading. Save it again.",
    ),
    // The folder of sounds.
    ("Elegir una carpeta", "Choose a folder"),
    ("Elegir…", "Choose…"),
    ("Eligiendo…", "Choosing…"),
    (
        "Rustcord no puede guardar el nombre de esa carpeta. Elige otra.",
        "Rustcord can't keep that folder's name. Choose another one.",
    ),
    // The window and its GPU.
    // Who draws the title bar.
    (
        "barra de título decoraciones de la ventana bordes del sistema sin marco minimizar maximizar cerrar",
        "title bar window decorations system borders frameless minimize maximize close",
    ),
    ("Barra de título", "Title bar"),
    ("Automática", "Automatic"),
    ("La de Rustcord", "Rustcord's"),
    ("La del sistema", "The system's"),
    (
        "Automática: la de Rustcord, salvo en Wayland, donde usa la del sistema.",
        "Automatic: Rustcord's, except on Wayland, where it uses the system's.",
    ),
    (
        "dibujar con gpu placa de video gráficos integrada dedicada aceleración por hardware",
        "draw with render gpu graphics card integrated discrete dedicated hardware acceleration",
    ),
    ("Dibujar con", "Draw with"),
    ("Automático", "Automatic"),
    (
        "Ahora dibuja con {gpu}. Se aplica la próxima vez que abras Rustcord.",
        "Drawing with {gpu} now. Takes effect the next time you open Rustcord.",
    ),
    (
        "Se aplica la próxima vez que abras Rustcord.",
        "Takes effect the next time you open Rustcord.",
    ),
    (
        "La GPU elegida no pudo abrir la ventana: esta vez dibuja cualquiera.",
        "The chosen GPU couldn't open the window: any GPU draws it this time.",
    ),
    (
        "La GPU que eligió Rustcord no pudo abrir la ventana: esta vez dibuja cualquiera.",
        "The GPU Rustcord chose couldn't open the window: any GPU draws it this time.",
    ),
    (
        "La variable {name} elige la GPU mientras esté.",
        "The {name} variable chooses the GPU while it's set.",
    ),
];
