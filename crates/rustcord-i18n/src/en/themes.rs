//! Own themes, the main color and the window's transparency:
//! `rustcord-components/src/{look,theme}.rs`, `rustcord-ui/src/themes.rs`, its page
//! and `rustcord-core/src/theme_files.rs`.

pub(super) const ENTRIES: super::Entries = &[
    // The colors a theme sets, in the editor's groups.
    ("Superficies", "Surfaces"),
    ("Riel de servidores", "Server rail"),
    ("Columnas", "Columns"),
    ("Campos y botones", "Fields and buttons"),
    ("Menús", "Menus"),
    ("Bajo el puntero", "Under the pointer"),
    ("Elegido", "Selected"),
    ("Bordes", "Borders"),
    ("Sombras", "Shadows"),
    ("Tarjetas de un mensaje", "Cards in a message"),
    ("Texto secundario", "Secondary text"),
    ("Texto atenuado", "Dimmed text"),
    ("Pistas y horas", "Hints and times"),
    ("Acciones y estados", "Actions and states"),
    ("Color principal", "Main color"),
    ("Texto sobre el color principal", "Text on the main color"),
    ("Fondo de una mención", "Mention background"),
    ("Peligro", "Danger"),
    ("Relleno de peligro", "Danger fill"),
    // The preview.
    ("Tu servidor", "Your server"),
    ("música", "music"),
    ("avisos", "announcements"),
    ("Así se ve tu tema.", "This is how your theme looks."),
    ("Y así, lo secundario.", "And this, what is secondary."),
    ("tú, ¿lo probaste?", "@you, did you try it?"),
    ("Escribe un mensaje", "Write a message"),
    ("Menú", "Menu"),
    ("Cambiar {name}", "Change {name}"),
    // Theme files.
    (
        "El archivo pesa más de 64 KB: no es un tema de Rustcord.",
        "The file is over 64 KB: it is not a Rustcord theme.",
    ),
    (
        "El archivo no es un tema de Rustcord o está dañado.",
        "The file is not a Rustcord theme, or it is damaged.",
    ),
    ("El tema es de otra versión de Rustcord.", "The theme is from another version of Rustcord."),
    (
        "El nombre del tema está vacío, es muy largo o tiene caracteres de control.",
        "The theme's name is empty, too long or has control characters.",
    ),
    (
        "La base del tema tiene que ser onyx, dark o light.",
        "The theme's base must be onyx, dark or light.",
    ),
    (
        "El tema nombra un color que Rustcord no conoce: {key}.",
        "The theme names a color Rustcord does not know: {key}.",
    ),
    (
        "El color {key} del tema no es #RRGGBB ni #RRGGBBAA.",
        "The theme's color {key} is neither #RRGGBB nor #RRGGBBAA.",
    ),
    ("tema", "theme"),
    ("No se eligió ningún archivo.", "No file was chosen."),
    ("No se pudo leer el archivo.", "The file could not be read."),
    (
        "Ya hay un archivo con ese nombre. Elige otro.",
        "There is a file with that name already. Choose another.",
    ),
    // The settings.
    (
        "editor de temas colores vista previa base guardar",
        "theme editor colors preview base save",
    ),
    (
        "color principal acento preajustes personalizado",
        "main color accent presets custom",
    ),
    (
        "transparencia ventana desenfoque escritorio fondo compositor kwin hyprland",
        "transparency window blur desktop background compositor kwin hyprland",
    ),
    ("Crear tema", "Create theme"),
    ("Editar tema", "Edit theme"),
    ("Importar…", "Import…"),
    ("Exportar…", "Export…"),
    ("¿Borrar «{name}»? No se puede deshacer.", "Delete “{name}”? It cannot be undone."),
    (
        "Elige una base y cambia los colores que quieras: la vista previa los muestra al instante y la ventana, al guardar. Se guarda en este equipo.",
        "Choose a base and change the colors you want: the preview shows them at once and the window when you save. It is kept on this computer.",
    ),
    ("Nombre del tema", "Theme name"),
    ("Base", "Base"),
    ("Los colores que no cambies salen de ella.", "The colors you do not change come from it."),
    (
        "Tiñe botones, menciones e indicadores sobre cualquier tema. El primero es el del tema.",
        "Tints buttons, mentions and indicators over any theme. The first one is the theme's own.",
    ),
    ("Transparencia de la ventana", "Window transparency"),
    (
        "Opaca: el escritorio no se ve a través de la ventana.",
        "Opaque: the desktop does not show through the window.",
    ),
    (
        "{percent} %: el escritorio se ve a través del riel, las columnas y la conversación. Los menús y los diálogos siguen opacos.",
        "{percent} %: the desktop shows through the rail, the columns and the conversation. Menus and dialogs stay opaque.",
    ),
    ("Se aplica al volver a abrir Rustcord.", "It applies when Rustcord opens again."),
    (
        "El desenfoque lo pone el compositor (KWin o Hyprland), si lo tiene activado.",
        "Blur comes from the compositor (KWin or Hyprland), if it has it on.",
    ),
    // What is done with them.
    ("Mi tema", "My theme"),
    ("{name} (copia)", "{name} (copy)"),
    ("Escribe un nombre de hasta {max} caracteres.", "Write a name of up to {max} characters."),
    ("Ya tienes un tema con ese nombre.", "You have a theme with that name already."),
    (
        "Puedes guardar hasta {max} temas: borra uno antes.",
        "You can keep up to {max} themes: delete one first.",
    ),
    ("Tema guardado", "Theme saved"),
    ("Se borró «{name}»", "“{name}” was deleted"),
    ("No se pudo importar el tema", "The theme could not be imported"),
    ("Se importó «{name}»", "“{name}” was imported"),
    ("Tema exportado", "Theme exported"),
    ("Se guardó como {filename}.", "It was saved as {filename}."),
    ("No se pudo exportar el tema", "The theme could not be exported"),
    ("Adaptar al modo claro u oscuro de tu dispositivo.", "Match your device's light or dark mode."),
    ("Igual que el tema del dispositivo", "Same as Device Theme"),
    ("Modo claro", "Light mode"),
    ("Modo oscuro", "Dark mode"),
    ("Temas predeterminados", "Default Themes"),
    ("accesibilidad saturación colores alto contraste", "accessibility saturation colors high contrast"),
    ("apariencia cambia el tema de la aplicación tema colores", "appearance changes the app's theme theme colors"),
    ("igual que el tema del dispositivo sistema claro oscuro escritorio automático adaptar", "same as device theme system light dark desktop automatic adapt"),
    ("modo claro tema claro día luz", "light mode light theme day light"),
    ("modo oscuro tema oscuro ónix negro noche propio", "dark mode dark theme onyx black night own"),
    ("temas predeterminados claro oscuro ónix negro colores modo propio crear importar exportar", "default themes light dark onyx black colors mode own create import export"),
];
