//! Discord's settings Rustcord does not do yet, drawn where Discord has them
//! so each page reads as it does there: muted, their controls disabled and
//! showing no value (Rustcord does not know the account's), with a line at
//! the page's foot that says so. Their rows are listed in their sections in
//! [`super::SECTIONS`]; [`ROWS`] says how each one looks.
//!
//! Most are the provider account's own (its data's use, its profile's
//! privacy, its emails) and no provider offers them; the rest are features
//! Rustcord has not built (the soundboard, text to speech, the app's icon).

use limn::{Button, ButtonVariant, Component, DropdownOption, Element, Icon, Select};

use rustcord_components::user_settings as parts;
use rustcord_i18n::{es, t};

use super::Ctx;

/// How a row's control looks.
enum Control {
    Switch,
    Button(&'static str),
    Danger(&'static str),
    /// A field showing Discord's usual choice.
    Select(&'static str),
    Choices(&'static [(&'static str, Option<&'static str>)]),
    Slider,
    /// The button of a fold that opens more.
    Fold,
    /// A label over the rows that follow, and a note maybe.
    Heading,
}

struct Pending {
    key: &'static str,
    label: &'static str,
    note: Option<&'static str>,
    control: Control,
}

const fn row(
    key: &'static str,
    label: &'static str,
    note: Option<&'static str>,
    control: Control,
) -> Pending {
    Pending {
        key,
        label,
        note,
        control,
    }
}

use Control::*;

const ROWS: &[Pending] = &[
    // «Datos y privacidad» › how the account's data is used.
    row(
        "data-basic",
        es("Utilizar mis datos para mejorar el funcionamiento del servicio"),
        Some(es("El servicio necesita almacenar y procesar ciertos datos para brindarte su funcionamiento básico, como tus mensajes, los servidores en los que estás y tus mensajes directos.")),
        Heading,
    ),
    row(
        "data-improve",
        es("Utilizar mis datos para optimizar el servicio"),
        Some(es("Permite que el servicio use y procese tu información para comprender y mejorar sus funciones.")),
        Switch,
    ),
    row(
        "data-personalize",
        es("Utilizar mis datos para personalizar mi experiencia"),
        Some(es("Permite utilizar información, como con quién hablas y qué juegos juegas, para personalizar el servicio para ti.")),
        Switch,
    ),
    row(
        "data-sponsored",
        es("Usar mi actividad para personalizar el contenido patrocinado"),
        Some(es("Permite personalizar el contenido patrocinado, como las misiones, usando tu actividad, como los juegos que juegas.")),
        Switch,
    ),
    row(
        "data-third-party",
        es("Utiliza datos de terceros para personalizar el contenido patrocinado"),
        Some(es("Permite personalizar el contenido patrocinado con datos de anunciantes y proveedores de datos de terceros.")),
        Switch,
    ),
    row(
        "data-discovery",
        es("Muestra el contenido de mis mensajes de los canales del descubrimiento en la web"),
        Some(es("El contenido de los mensajes de estos canales puede mostrarse en la web para que los visitantes puedan conocer mejor tu comunidad.")),
        Switch,
    ),
    row(
        "data-clips",
        es("Permite que mi voz se grabe en los clips"),
        Some(es("Permite que tu voz se incluya cuando alguien en el mismo canal de voz use Clips.")),
        Switch,
    ),
    row(
        "data-request",
        es("Solicitar mis datos"),
        Some(es("Si necesitas una copia de tus datos personales de todo el historial de tu cuenta, puedes obtenerla aquí.")),
        Button(es("Solicitar datos")),
    ),
    // «Privacidad del perfil».
    row(
        "profile-share",
        es("Compartir mi perfil completo con"),
        None,
        Choices(&[
            (
                es("Amigos y todos los servidores"),
                Some(es("Tu perfil completo es visible para tus amigos y cualquier servidor al que te unas.")),
            ),
            (
                es("Solo amigos y servidores pequeños"),
                Some(es("Tu perfil completo es visible para tus amigos y cualquier servidor al que te unas con 200 o menos miembros. Los demás verán una versión limitada.")),
            ),
            (
                es("Solo amigos"),
                Some(es("Tu perfil completo es visible para tus amigos. Todos los demás verán una versión limitada.")),
            ),
        ]),
    ),
    row(
        "profile-updates",
        es("Compartir cuando actualice mi perfil"),
        Some(es("Permite que tus amigos reciban una notificación push cuando actualices tu perfil.")),
        Switch,
    ),
    row(
        "e2ee-persistent",
        es("Activa los códigos de verificación persistentes"),
        Some(es("Otorga a tu dispositivo actual códigos de verificación persistentes: tus amigos solo necesitarán verificarlo una vez, en lugar de cada vez que entres a una llamada de voz.")),
        Switch,
    ),
    // «Permisos de mensajes».
    row(
        "age-commands",
        es("Permite el acceso a los comandos limitados por edad desde las aplicaciones en los mensajes directos"),
        Some(es("Permite que las personas de más de 18 años puedan acceder a los comandos marcados como restringidos por edad en los mensajes directos. Se aplica a todas las aplicaciones.")),
        Switch,
    ),
    row(
        "age-servers",
        es("Permite el acceso a servidores con restricción de edad en iOS"),
        Some(es("Accede a los servidores con restricción de edad (18+) en dispositivos iOS, después de unirte a ellos desde el escritorio.")),
        Switch,
    ),
    row(
        "friend-messages",
        es("Mostrar mensajes personalizados"),
        Some(es("Muestra mensajes personalizados en solicitudes de amistad entrantes. Si aceptas, el mensaje seguirá apareciendo en tus MD.")),
        Switch,
    ),
    row(
        "game-dms",
        es("Permite que tus amigos de los juegos envíen mensajes directos e invitaciones"),
        Some(es("Permite que tus amigos de los juegos conectados te envíen MD y te inviten a jugar, incluso cuando el juego al que juegan no esté abierto.")),
        Switch,
    ),
    row(
        "game-dm-choice",
        es("Mostrar los mensajes directos en las partidas"),
        Some(es("Lee y responde a los MD directamente desde los chats del juego.")),
        Choices(&[
            (es("Mostrar todos los MD"), None),
            (es("Muestra solo los MD de personas que también juegan"), None),
            (es("No mostrar ningún MD"), None),
        ]),
    ),
    // «Notificaciones».
    row("notify-when", es("Notificarme cuando..."), None, Heading),
    row(
        "notify-streams",
        es("Personas que conozco empiezan a transmitir en servidores pequeños"),
        None,
        Switch,
    ),
    row(
        "notify-anniversary",
        es("Un amigo y yo celebramos un aniversario de amistad"),
        None,
        Switch,
    ),
    row("notify-online", es("Los amigos se conectan en línea"), None, Switch),
    row("notify-events", es("Un servidor tiene un evento próximo"), None, Switch),
    row("notify-profiles", es("Los amigos actualizan su perfil"), None, Switch),
    row(
        "notify-reactions",
        es("Alguien reacciona a mis mensajes"),
        None,
        Select(es("Todos los mensajes")),
    ),
    row(
        "reading-sound",
        es("Nuevo mensaje en el canal que estoy leyendo actualmente"),
        None,
        Switch,
    ),
    row(
        "mute-notification-sounds",
        es("Desactivar todos los sonidos de las notificaciones"),
        Some(es("Desactiva todos los sonidos de las notificaciones, incluidos los efectos de sonido de voz y video. Tus preferencias de cada sonido se guardan y vuelven al desactivarlo.")),
        Switch,
    ),
    row(
        "email-communication",
        es("Correos de comunicaciones"),
        Some(es("Recibe correos electrónicos sobre llamadas perdidas, mensajes y resúmenes de mensajes.")),
        Switch,
    ),
    row(
        "email-social",
        es("Correos de interacción"),
        Some(es("Recibe correos electrónicos sobre solicitudes de amistad, nuevas sugerencias de amigos y eventos en tu servidor.")),
        Switch,
    ),
    row(
        "email-updates",
        es("Correos de anuncios y actualizaciones"),
        Some(es("Recibe correos electrónicos sobre actualizaciones del producto, sus últimas funciones, mejoras y corrección de errores.")),
        Switch,
    ),
    row(
        "email-tips",
        es("Correos de consejos"),
        Some(es("Recibe correos electrónicos con consejos útiles e información acerca de funciones menos conocidas.")),
        Switch,
    ),
    row(
        "email-recommendations",
        es("Correos de recomendaciones"),
        Some(es("Recibe correos electrónicos con servidores recomendados y eventos sugeridos.")),
        Switch,
    ),
    row(
        "email-unsubscribe",
        es("Cancelar suscripción de todos los correos electrónicos de marketing"),
        None,
        Danger(es("Darse de baja")),
    ),
    row(
        "notifications-advanced",
        es("Mostrar ajustes de notificaciones avanzados"),
        Some(es("Retraso de notificaciones en dispositivos móviles, permitir la reproducción y el uso del comando /tts y mucho más")),
        Fold,
    ),
    // «Voz y video».
    row(
        "video-preview",
        es("Siempre mostrar vista previa del video"),
        Some(es("Aparece la ventana modal de vista previa cada vez que activas un video")),
        Switch,
    ),
    row(
        "stream-previews",
        es("Mostrar vistas previas de transmisiones"),
        Some(es("Permite que otros previsualicen tu transmisión antes de unirse.")),
        Switch,
    ),
    row(
        "stream-advanced",
        es("Mostrar ajustes avanzados de la transmisión"),
        Some(es("Atenuación de transmisión y Fuerza de la atenuación de transmisión")),
        Fold,
    ),
    row(
        "soundboard-volume",
        es("Volumen del panel de sonidos"),
        Some(es("Controla qué tan alto se escuchan los sonidos.")),
        Slider,
    ),
    row(
        "entrance-sounds",
        es("Sonidos de entrada"),
        Some(es("Elige un sonido del panel de sonidos para que se reproduzca automáticamente cada vez que te unas a un canal de voz.")),
        Heading,
    ),
    row(
        "entrance-server",
        es("Elige un servidor"),
        None,
        Select(es("Todos los servidores")),
    ),
    row("entrance-sound", es("Elige un sonido"), None, Select(es("Ninguno"))),
    // «Apariencia».
    row(
        "theme-sync",
        es("Sincronizar tema en todos mis dispositivos"),
        None,
        Switch,
    ),
    row(
        "theme-profiles",
        es("Aplicar tema a los perfiles de otros usuarios"),
        None,
        Switch,
    ),
    row(
        "theme-servers",
        es("Tema predeterminado en los servidores"),
        None,
        Select(es("Usar el tema del servidor")),
    ),
    row(
        "spoilers",
        es("Mostrar contenido del spoiler"),
        None,
        Select(es("Al hacer clic")),
    ),
    row(
        "split-threads",
        es("Abre los hilos en vista dividida"),
        None,
        Switch,
    ),
    row(
        "favorites-server",
        es("Mostrar servidor de favoritos"),
        Some(es("Tus favoritos permanecen guardados, incluso si desactivas esta opción.")),
        Switch,
    ),
    row(
        "markdown-preview",
        es("Vista previa de emojis, menciones y la sintaxis Markdown mientras escribes"),
        None,
        Switch,
    ),
    row(
        "sticker-suggestions",
        es("Mostrar stickers en los resultados de autocompletar"),
        None,
        Switch,
    ),
    row(
        "game-suggestions",
        es("Mostrar juegos en los resultados de autocompletar"),
        None,
        Switch,
    ),
    row("send-button", es("Mostrar botón de enviar mensaje"), None, Switch),
    row(
        "dm-search",
        es("De forma predeterminada, buscando en los MD..."),
        None,
        Choices(&[
            (es("Busca solo en el MD seleccionado"), None),
            (es("Busca en todos mis MD"), None),
        ]),
    ),
    // «Accesibilidad».
    row(
        "underline-links",
        es("Siempre subrayar enlaces"),
        Some(es("Haz que los enlaces resalten más.")),
        Switch,
    ),
    row(
        "high-contrast",
        es("Activar modo de contraste alto"),
        Some(es("Mejora la visibilidad con colores llamativos y un contraste nítido.")),
        Switch,
    ),
    row(
        "official-messages",
        es("Mensajes oficiales"),
        Some(es("Elige cómo se ven los mensajes oficiales del desarrollador en los servidores verificados que lo admiten.")),
        Select(es("Predeterminado")),
    ),
    row(
        "switch-icons",
        es("Mostrar indicadores activar/desactivar"),
        Some(es("En formularios, menús y ajustes, los interruptores de activar/desactivar tendrán íconos.")),
        Switch,
    ),
    row(
        "tts-rate",
        es("Velocidad del texto a voz"),
        Some(es("Controla la velocidad a la que se lee el texto al usar la función de texto a voz.")),
        Slider,
    ),
    row(
        "image-descriptions",
        es("Mostrar descripciones de imágenes"),
        Some(es("Las descripciones de imagen se usan para describir una imagen para los lectores de pantalla.")),
        Switch,
    ),
    row(
        "legacy-input",
        es("Usar la antigua entrada del chat"),
        Some(es("Desactiva la mayoría de las funciones de chat, pero puede funcionar mejor con lectores de pantalla.")),
        Switch,
    ),
    // «Privacidad de la actividad».
    row(
        "share-online",
        es("Compartir cuando me conecte"),
        Some(es("Permite que tus amigos reciban una notificación push cuando te conectes.")),
        Switch,
    ),
    row(
        "share-streaming",
        es("Compartir cuando empiece a transmitir"),
        Some(es("Permite que los miembros con los que más interactúas reciban una notificación cuando empieces a transmitir en servidores pequeños.")),
        Switch,
    ),
    row(
        "activity-servers",
        es("Comparte mi actividad de forma automática cuando me uno a un servidor"),
        Some(es("Tu actividad siempre se comparte con tus amigos.")),
        Choices(&[
            (es("Comparte en todos los servidores"), None),
            (es("Compartir en servidores con 200 miembros o menos"), None),
            (es("No compartir en ningún servidor"), None),
        ]),
    ),
    row(
        "join-friends",
        es("Permite que amigos se unan a mi partida"),
        Some(es("Permite que tus amigos se unan a tu partida sin tener que enviar una solicitud.")),
        Switch,
    ),
    row(
        "join-voice",
        es("Permite que los participantes del canal de voz se unan a mi partida"),
        Some(es("Permite que cualquier participante en un canal de voz compartido se una a tu partida sin necesidad de enviar una solicitud.")),
        Switch,
    ),
    // «Desarrollador».
    row(
        "app-test-mode",
        es("Modo de prueba de la aplicación"),
        Some(es("Ingresa el ID de una aplicación para activar su modo de prueba.")),
        Switch,
    ),
];

/// Whether `key` is one of these rows.
pub(super) fn is_pending(key: &str) -> bool {
    ROWS.iter().any(|row| row.key == key)
}

/// What a page with any of them shown says at its foot.
pub(super) const FOOTNOTE: &str =
    es("Los ajustes atenuados todavía no están en Rustcord: se muestran, pero no cambian nada.");

/// The row `key`, muted and disabled; `None` for the rest.
pub(super) fn setting(ctx: &Ctx, key: &str) -> Option<Element> {
    let row = ROWS.iter().find(|row| row.key == key)?;
    let look = ctx.look;
    let id = format!("us:pending={key}");
    let (label, note) = (t(row.label), row.note.map(t));
    let control = match row.control {
        Switch => Some(
            parts::switch(id.clone(), false, look)
                .accessibility_label(label)
                .enabled(false)
                .render(),
        ),
        Button(name) => Some(parts::secondary_button(&id, t(name), false, look)),
        Danger(name) => Some(
            Button::new(id.clone(), t(name), look.palette())
                .variant(ButtonVariant::Secondary)
                .enabled(false)
                .render(),
        ),
        Select(current) => Some(
            look.select(Select::new(
                id.clone(),
                t(current),
                vec![DropdownOption::new("us:pending", t(current))],
                false,
                "us:select:dismiss",
                look.palette(),
            ))
            .accessibility_label(label)
            .enabled(false)
            .width(220.0)
            .height(parts::FIELD_HEIGHT)
            .render(),
        ),
        Fold => Some(
            Button::new(id.clone(), "", look.palette())
                .icon(Icon::ChevronDown)
                .variant(ButtonVariant::Secondary)
                .accessibility_label(label)
                .enabled(false)
                .render(),
        ),
        Choices(..) | Slider | Heading => None,
    };
    Some(match row.control {
        Choices(options) => {
            let options: Vec<(&str, Option<&str>)> = options
                .iter()
                .map(|(name, note)| (t(name), note.map(t)))
                .collect();
            Element::column(vec![
                parts::setting(label, note, None, false, look),
                parts::passive_choices(&id, label, &options, look),
            ])
        }
        Slider => Element::column(vec![
            parts::setting(label, note, None, false, look),
            parts::slider(id.clone(), 0.5, look)
                .width(ctx.geometry.column)
                .accessibility_label(label)
                .enabled(false)
                .render(),
        ]),
        Heading => parts::setting(label, note, None, true, look),
        _ => parts::setting(label, note, control, false, look),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_row_is_listed_once_and_in_a_section() {
        let listed: Vec<&str> = super::super::SECTIONS
            .iter()
            .chain(super::super::desktop::SECTIONS)
            .flat_map(|section| section.rows.iter().map(|(key, _)| *key))
            .collect();
        for (index, row) in ROWS.iter().enumerate() {
            assert!(
                ROWS[..index].iter().all(|before| before.key != row.key),
                "{} twice",
                row.key
            );
            assert!(listed.contains(&row.key), "{} has no section", row.key);
        }
    }
}
