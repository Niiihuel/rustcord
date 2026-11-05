//! Profile card and cosmetics: the popout's note icon
//! (`rustcord-components/src/profile.rs`), a badge's hint, the voice card and the switches of the cosmetics
//! (`rustcord-ui/src/settings/pages/cosmetics.rs`).

pub(super) const ENTRIES: super::Entries = &[
    // The popout.
    ("Agregar una nota", "Add a note"),
    ("Nota: {note}", "Note: {note}"),
    // A badge's hint, when its catalog says only when it was earned.
    ("Obtenida el {fecha}", "Earned on {fecha}"),
    // The voice card: where someone is talking.
    ("En una llamada", "In a call"),
    ("En un escenario", "In a stage"),
    ("Abrir voz", "Open voice"),
    ("Abrir llamada", "Open call"),
    ("Abrir escenario", "Open stage"),
    ("Unirse al escenario", "Join stage"),
    ("en {server}", "in {server}"),
    (
        "Solo visible para quienes tienen acceso a este canal",
        "Only visible to people with access to this channel",
    ),
    (
        "Solo visible para quienes tienen acceso a este chat",
        "Only visible to people with access to this chat",
    ),
    // Profile cosmetics.
    ("Cosméticos de los perfiles", "Profile cosmetics"),
    (
        "decoraciones de avatar marco cosméticos ocultar descargar",
        "avatar decorations frame cosmetics hide download",
    ),
    (
        "placas de nombre nameplates lista de miembros cosméticos ocultar",
        "nameplates member list cosmetics hide",
    ),
    (
        "efectos de perfil animaciones tarjeta cosméticos ocultar",
        "profile effects animations card cosmetics hide",
    ),
    ("Decoraciones de avatar", "Avatar decorations"),
    (
        "Los marcos que se dibujan alrededor de las caras.",
        "The frames drawn around faces.",
    ),
    ("Placas de nombre", "Nameplates"),
    (
        "El arte detrás de los nombres en la lista de miembros.",
        "The artwork behind names in the member list.",
    ),
    ("Efectos de perfil", "Profile effects"),
    (
        "Las animaciones sobre las tarjetas de perfil.",
        "The animations over profile cards.",
    ),
    (
        "estilos de nombre fuente tipografía efecto colores nitro cosméticos ocultar",
        "display name styles font typeface effect colors nitro cosmetics hide",
    ),
    ("Estilos de nombre", "Display name styles"),
    (
        "La fuente, el efecto y los colores que cada persona elige para su nombre.",
        "The font, effect and colors each person chooses for their name.",
    ),
    (
        "Lo oculto no se descarga. Sólo cambia lo que ves en este equipo.",
        "What is hidden is not downloaded. It only changes what you see on this computer.",
    ),
];
