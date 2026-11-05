//! The settings of one channel or category, on the frame of the server's
//! settings ([`crate::server_settings`]: sidebar, close button, bar of
//! unsaved changes, permission rows): what an overwrite can allow or deny for
//! each kind of channel, the control of three states that sets one
//! permission, the cards of the permissions page (in sync with the category,
//! private), and the list of roles and members with overwrites with the
//! popout that adds one. Pure functions of props; `rustcord-ui` keeps the drafts.
//!
//! Measures follow Discord's captures at 1361×797 logical pixels: in
//! the 705 px page column, a 223 px list of roles and members and the
//! permissions 24.5 px right of it, each with a 101×28 control.

use limn::{
    Access, Button, ButtonSize, ButtonVariant, Color, Component, ControlSize, DropdownOption,
    Element, FontFamily, Icon, Popover, PopoverPlacement, RadioGroup, RadioMetrics, RadioOption,
    RadioVariant, Role, ScrollArea, ScrollbarAppearance, Select, Size, Switch, TextField, TextSpan,
};
use rustcord_i18n::{es, t, tr};
use rustcord_model::{ChannelKind, Permissions};

use crate::server_settings::{color_dot, context_menu, MenuAction, PermissionInfo};
use crate::{centered, gap, line, strong_line, text, Look};

/// How long slow mode makes each member wait between two messages, and what
/// the choice says (Spanish: show it through `t`). The provider takes up to
/// six hours.
pub const SLOWMODE: [(u32, &str); 14] = [
    (0, es("Apagar")),
    (5, es("5 segundos")),
    (10, es("10 segundos")),
    (15, es("15 segundos")),
    (30, es("30 segundos")),
    (60, es("1 minuto")),
    (120, es("2 minutos")),
    (300, es("5 minutos")),
    (600, es("10 minutos")),
    (900, es("15 minutos")),
    (1_800, es("30 minutos")),
    (3_600, es("1 hora")),
    (7_200, es("2 horas")),
    (21_600, es("6 horas")),
];

/// What slow mode of `secs` is called, translated: one of [`SLOWMODE`], or
/// the seconds when a provider set a wait the list does not have.
pub fn slowmode_label(secs: u32) -> String {
    SLOWMODE
        .iter()
        .find(|(value, _)| *value == secs)
        .map_or_else(
            || tr!("{seconds} segundos", seconds = secs),
            |(_, label)| t(label).to_owned(),
        )
}

/// How a time to hide or archive threads after reads: one of
/// [`rustcord_model::ARCHIVE_AFTER`], or its minutes.
pub fn archive_label(minutes: u32) -> String {
    match minutes {
        60 => t("1 hora").into(),
        1440 => t("24 horas").into(),
        4320 => t("3 días").into(),
        10080 => t("1 semana").into(),
        other => tr!("{n} minutos", n = other),
    }
}

/// The most a channel's topic holds, in characters.
pub const TOPIC_LIMIT: usize = 1024;

/// The list of roles and members with overwrites, and the space after it.
pub const TARGET_LIST_WIDTH: f32 = 223.0;
pub const TARGET_GAP: f32 = 24.5;
/// The control of three states.
pub const TRI_STATE_WIDTH: f32 = 101.0;
pub const TRI_STATE_HEIGHT: f32 = 28.0;

type Groups = &'static [(&'static str, &'static [PermissionInfo])];

/// What an overwrite of a text channel can allow or deny.
const TEXT: Groups = &[
    (
        es("Permisos generales de los canales"),
        &[
            (
                Permissions::VIEW,
                es("Ver el canal"),
                es("Deja que los miembros vean este canal."),
            ),
            (
                Permissions::MANAGE_CHANNELS,
                es("Gestionar canal"),
                es(
                    "Deja cambiar el nombre, el tema y los ajustes de este canal, y también eliminarlo.",
                ),
            ),
            (
                Permissions::MANAGE_ROLES,
                es("Gestionar permisos"),
                es("Deja cambiar los permisos de este canal."),
            ),
        ],
    ),
    (
        es("Permisos de membresía"),
        &[(
            Permissions::CREATE_INVITE,
            es("Crear invitación"),
            es("Deja invitar gente nueva al servidor con un enlace que lleva a este canal."),
        )],
    ),
    (
        es("Permisos de los canales de texto"),
        &[
            (
                Permissions::SEND,
                es("Enviar mensajes"),
                es("Deja escribir en este canal."),
            ),
            (
                Permissions::SEND_IN_THREADS,
                es("Enviar mensajes en hilos"),
                es("Deja escribir en los hilos de este canal."),
            ),
            (
                Permissions::CREATE_THREADS,
                es("Crear hilos públicos"),
                es("Deja abrir hilos que todos en este canal pueden ver."),
            ),
            (
                Permissions::CREATE_PRIVATE_THREADS,
                es("Crear hilos privados"),
                es("Deja abrir hilos que solo ven quienes son invitados y la moderación."),
            ),
            (
                Permissions::EMBED_LINKS,
                es("Insertar enlaces"),
                es("Los enlaces que se compartan aquí muestran una vista previa."),
            ),
            (
                Permissions::ATTACH,
                es("Adjuntar archivos"),
                es("Deja subir archivos e imágenes a este canal."),
            ),
            (
                Permissions::SEND_POLLS,
                es("Crear encuestas"),
                es("Deja empezar encuestas en este canal."),
            ),
            (
                Permissions::REACT,
                es("Añadir reacciones"),
                es(
                    "Deja poner reacciones nuevas a los mensajes de este canal. Sin este permiso se puede sumar a las que ya están.",
                ),
            ),
            (
                Permissions::EXTERNAL_EMOJI,
                es("Usar emojis externos"),
                es("Deja usar en este canal emojis de otros servidores."),
            ),
            (
                Permissions::EXTERNAL_STICKERS,
                es("Usar stickers externos"),
                es("Deja enviar en este canal stickers de otros servidores."),
            ),
            (
                Permissions::MENTION_EVERYONE,
                es("Mencionar a @everyone, @here y a todos los roles"),
                es(
                    "Deja avisar a todo el canal o a quienes estén en línea, y mencionar cualquier rol aunque no sea mencionable.",
                ),
            ),
            (
                Permissions::MANAGE_MESSAGES,
                es("Gestionar mensajes"),
                es("Deja borrar mensajes de otros miembros en este canal."),
            ),
            (
                Permissions::PIN_MESSAGES,
                es("Fijar mensajes"),
                es("Deja fijar y desfijar mensajes en este canal."),
            ),
            (
                Permissions::MANAGE_THREADS,
                es("Gestionar hilos"),
                es("Deja renombrar, cerrar y borrar los hilos de este canal, y ver los privados."),
            ),
            (
                Permissions::READ_HISTORY,
                es("Leer el historial de mensajes"),
                es(
                    "Deja leer lo que se escribió antes. Sin este permiso solo se ve lo que llega mientras se está en línea.",
                ),
            ),
        ],
    ),
];

/// What an overwrite of a voice channel can allow or deny. Its chat is left
/// out: voice channels hold no messages here.
const VOICE: Groups = &[
    (
        es("Permisos generales de los canales"),
        &[
            (
                Permissions::VIEW,
                es("Ver el canal"),
                es("Deja que los miembros vean este canal de voz."),
            ),
            (
                Permissions::MANAGE_CHANNELS,
                es("Gestionar canal"),
                es(
                    "Deja cambiar el nombre y los ajustes de este canal de voz, y también eliminarlo.",
                ),
            ),
            (
                Permissions::MANAGE_ROLES,
                es("Gestionar permisos"),
                es("Deja cambiar los permisos de este canal."),
            ),
        ],
    ),
    (
        es("Permisos de membresía"),
        &[(
            Permissions::CREATE_INVITE,
            es("Crear invitación"),
            es("Deja invitar gente nueva al servidor con un enlace que lleva a este canal de voz."),
        )],
    ),
    (
        es("Permisos de los canales de voz"),
        &[
            (
                Permissions::CONNECT,
                es("Conectar"),
                es("Deja entrar a este canal de voz y escuchar."),
            ),
            (
                Permissions::SPEAK,
                es("Hablar"),
                es(
                    "Deja hablar en este canal. Sin este permiso se entra silenciado hasta que alguien con «Silenciar miembros» lo permita.",
                ),
            ),
            (
                Permissions::STREAM,
                es("Video"),
                es("Deja compartir la cámara o la pantalla en este canal."),
            ),
            (
                Permissions::USE_VOICE_ACTIVITY,
                es("Usar Actividad de voz"),
                es(
                    "Deja hablar sin pulsar una tecla. Sin este permiso hay que usar pulsar para hablar.",
                ),
            ),
            (
                Permissions::PRIORITY_SPEAKER,
                es("Prioridad de palabra"),
                es("Mientras habla, baja el volumen de quienes no tienen este permiso."),
            ),
            (
                Permissions::MUTE_MEMBERS,
                es("Silenciar miembros"),
                es("Deja silenciar a otros en este canal, para todos."),
            ),
            (
                Permissions::DEAFEN_MEMBERS,
                es("Ensordecer miembros"),
                es("Deja que otros no puedan hablar ni escuchar en este canal."),
            ),
            (
                Permissions::MOVE_MEMBERS,
                es("Mover miembros"),
                es(
                    "Deja desconectar a otros de este canal o llevarlos a otro al que se tenga acceso.",
                ),
            ),
            (
                Permissions::REQUEST_TO_SPEAK,
                es("Pedir la palabra"),
                es("Deja levantar la mano para hablar, si este canal es un escenario."),
            ),
        ],
    ),
];

/// What an overwrite of a category can allow or deny: for every channel of
/// it, of either kind.
const CATEGORY: Groups = &[
    (
        es("Permisos generales de las categorías"),
        &[
            (
                Permissions::VIEW,
                es("Ver canales"),
                es("Deja que los miembros vean los canales de esta categoría."),
            ),
            (
                Permissions::MANAGE_CHANNELS,
                es("Gestionar canales"),
                es("Deja cambiar el nombre y los ajustes de estos canales, y también eliminarlos."),
            ),
            (
                Permissions::MANAGE_ROLES,
                es("Gestionar permisos"),
                es("Deja cambiar los permisos de estos canales."),
            ),
        ],
    ),
    (
        es("Permisos de membresía"),
        &[(
            Permissions::CREATE_INVITE,
            es("Crear invitación"),
            es("Deja invitar gente nueva al servidor con un enlace que lleva a estos canales."),
        )],
    ),
    (
        es("Permisos de los canales de texto"),
        &[
            (
                Permissions::SEND,
                es("Enviar mensajes"),
                es("Deja escribir en estos canales."),
            ),
            (
                Permissions::SEND_IN_THREADS,
                es("Enviar mensajes en hilos"),
                es("Deja escribir en los hilos de estos canales."),
            ),
            (
                Permissions::CREATE_THREADS,
                es("Crear hilos públicos"),
                es("Deja abrir hilos que todos en estos canales pueden ver."),
            ),
            (
                Permissions::CREATE_PRIVATE_THREADS,
                es("Crear hilos privados"),
                es("Deja abrir hilos que solo ven quienes son invitados y la moderación."),
            ),
            (
                Permissions::EMBED_LINKS,
                es("Insertar enlaces"),
                es("Los enlaces que se compartan en estos canales muestran una vista previa."),
            ),
            (
                Permissions::ATTACH,
                es("Adjuntar archivos"),
                es("Deja subir archivos e imágenes a estos canales."),
            ),
            (
                Permissions::SEND_POLLS,
                es("Crear encuestas"),
                es("Deja empezar encuestas en estos canales."),
            ),
            (
                Permissions::REACT,
                es("Añadir reacciones"),
                es(
                    "Deja poner reacciones nuevas a los mensajes de estos canales. Sin este permiso se puede sumar a las que ya están.",
                ),
            ),
            (
                Permissions::EXTERNAL_EMOJI,
                es("Usar emojis externos"),
                es("Deja usar en estos canales emojis de otros servidores."),
            ),
            (
                Permissions::EXTERNAL_STICKERS,
                es("Usar stickers externos"),
                es("Deja enviar en estos canales stickers de otros servidores."),
            ),
            (
                Permissions::MENTION_EVERYONE,
                es("Mencionar a @everyone, @here y a todos los roles"),
                es(
                    "Deja avisar a todos o a quienes estén en línea, y mencionar cualquier rol aunque no sea mencionable, en estos canales.",
                ),
            ),
            (
                Permissions::MANAGE_MESSAGES,
                es("Gestionar mensajes"),
                es("Deja borrar mensajes de otros miembros en estos canales."),
            ),
            (
                Permissions::PIN_MESSAGES,
                es("Fijar mensajes"),
                es("Deja fijar y desfijar mensajes en estos canales."),
            ),
            (
                Permissions::MANAGE_THREADS,
                es("Gestionar hilos"),
                es(
                    "Deja renombrar, cerrar y borrar los hilos de estos canales, y ver los privados.",
                ),
            ),
            (
                Permissions::READ_HISTORY,
                es("Leer el historial de mensajes"),
                es(
                    "Deja leer lo que se escribió antes en estos canales. Sin este permiso solo se ve lo que llega mientras se está en línea.",
                ),
            ),
        ],
    ),
    (
        es("Permisos de los canales de voz"),
        &[
            (
                Permissions::CONNECT,
                es("Conectar"),
                es("Deja entrar a estos canales de voz y escuchar."),
            ),
            (
                Permissions::SPEAK,
                es("Hablar"),
                es(
                    "Deja hablar en estos canales. Sin este permiso se entra silenciado hasta que alguien con «Silenciar miembros» lo permita.",
                ),
            ),
            (
                Permissions::STREAM,
                es("Video"),
                es("Deja compartir la cámara o la pantalla en estos canales."),
            ),
            (
                Permissions::USE_VOICE_ACTIVITY,
                es("Usar Actividad de voz"),
                es(
                    "Deja hablar sin pulsar una tecla. Sin este permiso hay que usar pulsar para hablar.",
                ),
            ),
            (
                Permissions::PRIORITY_SPEAKER,
                es("Prioridad de palabra"),
                es("Mientras habla, baja el volumen de quienes no tienen este permiso."),
            ),
            (
                Permissions::MUTE_MEMBERS,
                es("Silenciar miembros"),
                es("Deja silenciar a otros en estos canales, para todos."),
            ),
            (
                Permissions::DEAFEN_MEMBERS,
                es("Ensordecer miembros"),
                es("Deja que otros no puedan hablar ni escuchar en estos canales."),
            ),
            (
                Permissions::MOVE_MEMBERS,
                es("Mover miembros"),
                es(
                    "Deja desconectar a otros de estos canales o llevarlos a otro al que se tenga acceso.",
                ),
            ),
            (
                Permissions::REQUEST_TO_SPEAK,
                es("Pedir la palabra"),
                es("Deja levantar la mano para hablar en los escenarios."),
            ),
        ],
    ),
];

/// The permissions an overwrite of a `kind` channel sets, grouped and named
/// as the settings show them; their texts are Spanish, shown through `t`.
pub fn permission_groups(kind: ChannelKind) -> Groups {
    match kind {
        ChannelKind::Voice => VOICE,
        ChannelKind::Category => CATEGORY,
        _ => TEXT,
    }
}

/// What «Ver el canal» adds for `@everyone`: taking it away makes it private.
pub fn everyone_view_note(kind: ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Voice => t("Si se lo quitas a @everyone, el canal de voz pasa a ser privado."),
        ChannelKind::Category => {
            t("Si se lo quitas a @everyone, los canales de la categoría pasan a ser privados.")
        }
        _ => t("Si se lo quitas a @everyone, el canal pasa a ser privado."),
    }
}

/// The id of a permission's control: `cs:perm:{bit}`; its parts end in
/// `:deny`, `:inherit` and `:allow`.
pub fn permission_id(permission: Permissions) -> String {
    format!("cs:perm:{}", permission.0.trailing_zeros())
}

/// Deny ✕, leave as it is ⧸, or allow ✓: one is always chosen. `state` is
/// what [`rustcord_model::Overwrite::state`] says.
pub fn tri_state(
    id: &str,
    state: Option<bool>,
    enabled: bool,
    label: &str,
    look: &Look,
) -> Element {
    let white = Color::rgb(255, 255, 255);
    let segment = |which: &str, glyph: Icon, name: &str, fill: Color, ink: Color, chosen: bool| {
        let body = Element::panel(centered(
            Element::icon(glyph)
                .foreground(if chosen && fill != look.selected {
                    white
                } else {
                    ink
                })
                .size(Some(14.0), Some(14.0)),
        ))
        .size(
            Some((TRI_STATE_WIDTH - 2.0) / 3.0),
            Some(TRI_STATE_HEIGHT - 2.0),
        )
        .radius(5.0)
        .background(if chosen { fill } else { Color::TRANSPARENT });
        let access = Access::new(Role::RadioButton)
            .name(format!("{label}: {name}"))
            .selected(chosen)
            .disabled(!enabled);
        // The chosen one too, so all three are told with which is chosen.
        if enabled {
            Element::clickable(format!("{id}:{which}"), body)
                .radius(5.0)
                .hover_background(look.hover)
                .accessible(access)
        } else {
            body.accessible(access)
        }
    };
    // Disabled, the choice still reads, only quieter.
    let dim = |color: Color| {
        if enabled {
            color
        } else {
            Look::alpha(color, 110)
        }
    };
    Element::panel(
        Element::row(vec![
            segment(
                "deny",
                Icon::Close,
                t("denegar"),
                dim(look.danger),
                dim(look.danger),
                state == Some(false),
            ),
            segment(
                "inherit",
                Icon::Neutral,
                t("heredar"),
                look.selected,
                dim(look.text),
                state.is_none(),
            ),
            segment(
                "allow",
                Icon::Check,
                t("permitir"),
                dim(look.online),
                dim(look.online),
                state == Some(true),
            ),
        ])
        .padding(1.0),
    )
    .size(Some(TRI_STATE_WIDTH), Some(TRI_STATE_HEIGHT))
    .radius(6.0)
    .border(look.border, 1.0)
}

/// Whether a channel's overwrites are its category's, as a card on the
/// permissions page; out of sync, with «Sincronizar ahora» (`cs:sync`).
pub fn sync_card(category: &str, synced: bool, can_sync: bool, look: &Look) -> Element {
    let (glyph, lead) = if synced {
        (
            Icon::Refresh,
            t("Permisos sincronizados con la categoría: "),
        )
    } else {
        (
            Icon::WarningTriangle,
            t("Los permisos no están sincronizados con la categoría: "),
        )
    };
    let sentence = Element::formatted_text(vec![
        TextSpan::new(lead, look.text, 16.0, FontFamily::Sans),
        TextSpan::new(category, look.text, 16.0, FontFamily::SansBold),
    ]);
    let mut parts = vec![
        Element::icon(glyph)
            .foreground(look.idle)
            .size(Some(20.0), Some(20.0))
            .centered_y(),
        sentence.flex(1.0).centered_y(),
    ];
    if !synced {
        parts.push(
            Button::new("cs:sync", t("Sincronizar ahora"), look.menu_palette())
                .variant(ButtonVariant::Secondary)
                .button_size(ButtonSize::Small)
                .enabled(can_sync)
                .render()
                .centered_y(),
        );
    }
    Element::panel(Element::row(parts).gap(12.0).padding(16.0))
        .size(None, Some(if synced { 56.0 } else { 68.0 }))
        .radius(8.0)
        .background(look.raised)
        .border(look.border, 1.0)
        .accessible(Access::new(Role::Status).name(format!("{lead}{category}")))
}

/// A role or a person that may see a private channel.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessLine {
    pub name: String,
    /// «Rol», or the person's handle.
    pub detail: String,
    /// The role's color, or the person's accent.
    pub color: Color,
    pub person: bool,
    /// The person's picture, once loaded.
    pub image: Option<limn::RasterImage>,
    /// Said before the button, such as «Propietario del servidor».
    pub note: Option<String>,
    /// The action that takes their access away, when it can be taken.
    pub remove: Option<String>,
}

/// The card that makes a channel private, and who may see it while it is.
#[derive(Clone, Debug, PartialEq)]
pub struct PrivateCard {
    pub category: bool,
    pub on: bool,
    pub enabled: bool,
    pub roles: Vec<AccessLine>,
    pub members: Vec<AccessLine>,
}

/// «Canal privado» or «Categoría privada» with its switch (`cs:private`),
/// and while on, «Añadir miembros o roles» (`cs:access:add`) and the roles
/// and people who may see it.
pub fn private_card(card: &PrivateCard, look: &Look) -> Element {
    let (title, note) = if card.category {
        (
            t("Categoría privada"),
            t(
                "Al convertir una categoría en privada, solo los miembros y roles seleccionados podrán verla. Los canales vinculados a esta categoría pasarán a tener los mismos ajustes de forma automática.",
            ),
        )
    } else {
        (
            t("Canal privado"),
            t("Al convertir un canal en privado, solo los miembros y roles seleccionados podrán verlo."),
        )
    };
    let head = Element::column(vec![
        Element::row(vec![
            Element::icon(Icon::Lock)
                .foreground(look.text)
                .size(Some(20.0), Some(20.0))
                .centered_y(),
            strong_line(title, 16.0, look.text).flex(1.0).centered_y(),
            Switch::new("cs:private", card.on, look.palette())
                .accessibility_label(title)
                .enabled(card.enabled)
                .render()
                .centered_y(),
        ])
        .gap(10.0),
        text(note, 12.0, look.muted),
    ])
    .gap(8.0)
    .padding(18.0);
    let mut parts = vec![head];
    if card.on {
        let rule = || {
            Element::panel(Element::spacer())
                .size(None, Some(1.0))
                .background(look.border)
        };
        let whom = if card.category {
            t("¿Quién puede acceder a esta categoría?")
        } else {
            t("¿Quién puede acceder a este canal?")
        };
        let mut body = vec![
            Element::row(vec![
                strong_line(whom, 14.0, look.text).flex(1.0).centered_y(),
                Button::new(
                    "cs:access:add",
                    t("Añadir miembros o roles"),
                    look.accent_palette(),
                )
                .button_size(ButtonSize::Small)
                .enabled(card.enabled)
                .render()
                .centered_y(),
            ])
            .gap(12.0),
            rule(),
            strong_line(t("Roles"), 14.0, look.text),
        ];
        if card.roles.is_empty() {
            body.push(
                Element::row(vec![
                    Element::icon(Icon::Users)
                        .foreground(look.faint)
                        .size(Some(16.0), Some(16.0))
                        .centered_y(),
                    line(t("No hay roles"), 14.0, look.faint).centered_y(),
                ])
                .gap(8.0),
            );
        }
        body.extend(card.roles.iter().map(|row| access_row(row, look)));
        body.push(rule());
        body.push(strong_line(t("Miembros"), 14.0, look.text));
        body.extend(card.members.iter().map(|row| access_row(row, look)));
        parts.push(
            Element::panel(Element::column(body).gap(10.0).padding(18.0)).background(look.canvas),
        );
    }
    Element::panel(Element::column(parts))
        .radius(8.0)
        .background(look.raised)
        .border(look.border, 1.0)
        .clip_children()
}

fn access_row(row: &AccessLine, look: &Look) -> Element {
    let mark = if row.person {
        crate::avatar_with_image(&row.name, row.color, 20.0, row.image.clone())
    } else {
        color_dot(row.color, 14.0)
    };
    let mut parts = vec![
        mark.centered_y(),
        line(&row.name, 15.0, look.text).centered_y(),
        line(&row.detail, 13.0, look.muted).flex(1.0).centered_y(),
    ];
    if let Some(note) = &row.note {
        parts.push(line(note, 13.0, look.muted).centered_y());
    }
    if let Some(remove) = &row.remove {
        parts.push(
            Button::new(remove.clone(), "", look.palette())
                .icon(Icon::CloseCircle)
                .variant(ButtonVariant::Ghost)
                .button_size(ButtonSize::Small)
                .accessibility_label(tr!("Quitarle el acceso a {name}", name = row.name))
                .tooltip(t("Quitar"))
                .render()
                .centered_y(),
        );
    }
    Element::row(parts).gap(8.0).size(None, Some(32.0))
}

/// «Permisos avanzados» with its chevron, which opens and closes the view
/// of every overwrite (`cs:advanced`).
pub fn advanced_heading(open: bool, look: &Look) -> Element {
    Element::clickable(
        "cs:advanced",
        Element::row(vec![
            line(t("Permisos avanzados"), 20.0, look.text).centered_y(),
            Element::icon(if open {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            })
            .foreground(look.text)
            .size(Some(16.0), Some(16.0))
            .centered_y(),
            Element::spacer().flex(1.0),
        ])
        .gap(12.0),
    )
    .size(None, Some(40.0))
    .accessible(
        Access::new(Role::Button)
            .name(t("Permisos avanzados"))
            .expanded(open),
    )
}

/// How a row of the list of overwrites is marked.
#[derive(Clone, Debug, PartialEq)]
pub enum TargetMark {
    /// A role, by its color.
    Role(Color),
    /// `@everyone`, unmarked.
    Everyone,
    /// A person, by their accent and their picture once loaded.
    Member(Color, Option<limn::RasterImage>),
}

/// A role or a person with an overwrite, as the advanced view lists them.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetItem {
    /// What a click reports: it shows their permissions.
    pub id: String,
    pub name: String,
    pub mark: TargetMark,
    pub selected: bool,
    /// Its context menu; the first action, when there is one, is also a
    /// button on the selected row, for those without a secondary button.
    pub menu: Vec<MenuAction>,
}

/// The left column of the advanced view: «ROLES/MIEMBROS» with `add` (the
/// ⊕ and its popout), then the roles, `@everyone` and the people.
pub fn target_list(items: &[TargetItem], add: Element, look: &Look) -> Element {
    let mut rows = vec![Element::row(vec![
        gap(10.5, 0.0),
        line(t("ROLES/MIEMBROS"), 12.0, look.faint)
            .font_family(FontFamily::SansBold)
            .flex(1.0)
            .centered_y(),
        add.centered_y(),
    ])
    .size(None, Some(32.0))];
    for item in items {
        let mark = match &item.mark {
            TargetMark::Role(color) => Some(color_dot(*color, 13.0)),
            TargetMark::Member(accent, image) => Some(crate::avatar_with_image(
                &item.name,
                *accent,
                20.0,
                image.clone(),
            )),
            TargetMark::Everyone => None,
        };
        let mut parts = Vec::new();
        if let Some(mark) = mark {
            parts.push(mark.centered_y());
        }
        parts.push(
            line(
                &item.name,
                16.0,
                if item.selected { look.text } else { look.muted },
            )
            .flex(1.0)
            .centered_y(),
        );
        let body = Element::clickable(item.id.clone(), Element::row(parts).gap(8.0).padding(10.5))
            .radius(6.0)
            .hover_background(look.hover)
            .hover_foreground(look.text)
            .accessible(
                Access::new(Role::ListItem)
                    .name(item.name.clone())
                    .selected(item.selected),
            );
        let body = if item.selected {
            body.background(look.selected)
        } else {
            body
        };
        let mut row = vec![body.flex(1.0)];
        if let Some(first) = item.menu.first().filter(|_| item.selected) {
            row.push(
                Button::new(first.id.clone(), "", look.palette())
                    .icon(first.icon.unwrap_or(Icon::Trash))
                    .variant(ButtonVariant::Ghost)
                    .button_size(ButtonSize::Small)
                    .accessibility_label(first.label.clone())
                    .tooltip(first.denied.clone().unwrap_or_else(|| first.label.clone()))
                    .enabled(first.denied.is_none())
                    .render()
                    .centered_y(),
            );
        }
        let row = Element::row(row).gap(2.0).size(None, Some(34.0));
        rows.push(if item.menu.is_empty() {
            row
        } else {
            context_menu(&format!("{}:context", item.id), row, &item.menu, look)
        });
    }
    rows.push(gap(0.0, 8.0));
    rows.push(
        Element::row(vec![
            gap(10.5, 0.0),
            Element::panel(Element::spacer())
                .size(None, Some(1.0))
                .background(look.border)
                .flex(1.0),
            gap(10.5, 0.0),
        ])
        .size(None, Some(1.0)),
    );
    Element::column(rows)
        .gap(2.0)
        .size(Some(TARGET_LIST_WIDTH), None)
}

/// A role or a person the popout offers to add to the list.
#[derive(Clone, Debug, PartialEq)]
pub struct AddChoice {
    pub id: String,
    pub name: String,
    /// The role's color, or the person's accent.
    pub color: Color,
    pub person: bool,
    /// The person's picture, once loaded.
    pub image: Option<limn::RasterImage>,
}

/// The tallest the popout's list of roles and people gets: eight of them.
const ADD_LIST_HEIGHT: f32 = 8.0 * 44.0;

/// The ⊕ of the list (`cs:add`) and, while `open`, the popout «Añadir:» with
/// its field (`cs:add-query`) and what matches it; Esc or a click outside
/// reports `cs:add:dismiss`.
pub fn add_popout(
    enabled: bool,
    open: bool,
    query: &str,
    choices: &[AddChoice],
    look: &Look,
) -> Element {
    let button = Button::new("cs:add", "", look.palette())
        .icon(Icon::Add)
        .variant(ButtonVariant::Ghost)
        .button_size(ButtonSize::Small)
        .accessibility_label(t("Añadir un rol o un miembro"))
        .tooltip(t("Añadir un rol o un miembro"))
        .enabled(enabled)
        .render();
    let mut content = vec![
        strong_line(t("Añadir:"), 16.0, look.text),
        TextField::new("cs:add-query", query, t("Rol/Miembro"), look.menu_palette())
            .accessibility_label(t("Buscar un rol o un miembro"))
            .render(),
    ];
    if choices.is_empty() {
        content.push(text(t("Nada para añadir."), 13.0, look.muted));
    }
    let mut list = Vec::with_capacity(choices.len());
    for choice in choices {
        let (mark, name_ink, detail) = if choice.person {
            (
                crate::avatar_with_image(&choice.name, choice.color, 24.0, choice.image.clone()),
                look.text,
                String::new(),
            )
        } else {
            (gap(0.0, 0.0), choice.color, t("Rol").to_owned())
        };
        list.push(
            Element::clickable(
                choice.id.clone(),
                Element::row(vec![
                    mark.centered_y(),
                    line(&choice.name, 15.0, name_ink).flex(1.0).centered_y(),
                    line(detail, 12.0, look.muted).centered_y(),
                ])
                .gap(10.0)
                .padding(6.0),
            )
            .size(None, Some(36.0))
            .radius(4.0)
            .hover_background(look.hover)
            .accessible(Access::new(Role::Button).name(choice.name.clone())),
        );
    }
    // A server with many roles and people scrolls inside the popout instead
    // of running past the window's foot.
    if !list.is_empty() {
        content.push(
            ScrollArea::new("cs:add-list", Element::column(list).gap(8.0))
                .appearance(ScrollbarAppearance::overlay().buttons(false))
                .render()
                .max_height(ADD_LIST_HEIGHT),
        );
    }
    Popover::new(
        button,
        Element::column(content).gap(8.0).padding(6.0),
        open,
        "cs:add:dismiss",
        look.menu_palette(),
    )
    .placement(PopoverPlacement::BottomEnd)
    .width(255.0)
    .render()
}

/// A choice among a list, shown as a text field shows its text.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldSelect {
    /// What the trigger reports; each option reports its own id.
    pub id: String,
    /// What the choice is called, for those who do not see it.
    pub name: String,
    pub value: String,
    /// Each option's id, its label, and whether it is the one chosen.
    pub options: Vec<(String, String, bool)>,
    pub open: bool,
    /// What Esc or a click outside the open list reports.
    pub dismiss: String,
    pub enabled: bool,
    pub width: f32,
}

/// A [`FieldSelect`]: as tall as a text field, the value at its start in 16
/// px and a chevron at its end, the list under it with a check on the
/// choice, five rows in view as Discord's; open, the chevron turns up
/// and the border takes the accent.
pub fn field_select(select: &FieldSelect, look: &Look) -> Element {
    let options = select
        .options
        .iter()
        .map(|(id, label, chosen)| DropdownOption::new(id.clone(), label.clone()).selected(*chosen))
        .collect();
    look.select(Select::new(
        select.id.clone(),
        select.value.clone(),
        options,
        select.open,
        select.dismiss.clone(),
        look.palette(),
    ))
    .height(41.0)
    .max_visible_items(5)
    .ink(look.accent)
    .width(select.width)
    .placement(PopoverPlacement::BottomStart)
    .accessibility_label(select.name.clone())
    .enabled(select.enabled)
    .render()
}

/// Room between two fields of the overview, as Discord's measures: 24 px,
/// 40 after slow mode's note and 48 after the choices of «Visibilidad del
/// contenido».
pub const FIELD_GAP: f32 = 24.0;
pub const AFTER_SLOWMODE: f32 = 40.0;
pub const AFTER_CHOICES: f32 = 48.0;

/// A field of the overview: its title (16 px medium), its control 8 px
/// under it, and a [`note`] 4 px under the control.
pub fn field(title: &str, control: Element, note: Option<Element>, look: &Look) -> Element {
    let mut parts = vec![strong_line(title, 16.0, look.text), gap(0.0, 8.0), control];
    if let Some(note) = note {
        parts.push(gap(0.0, 4.0));
        parts.push(note);
    }
    Element::column(parts)
}

/// A field's note: 12 px in the muted ink, the parts marked strong in
/// semibold.
pub fn note(parts: &[(&str, bool)], look: &Look) -> Element {
    Element::formatted_text(
        parts
            .iter()
            .map(|(text, strong)| {
                let family = if *strong {
                    FontFamily::SansSemibold
                } else {
                    FontFamily::Sans
                };
                TextSpan::new(*text, look.muted, 12.0, family)
            })
            .collect(),
    )
}

/// The line that sets a voice channel's own options apart: 56 px under the
/// choices above it and 48 over the next field.
pub fn group_divider(look: &Look) -> Element {
    Element::column(vec![
        gap(0.0, 56.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
        gap(0.0, 48.0),
    ])
}

/// What a channel's content shows as: the three choices of «Visibilidad
/// del contenido».
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Always visible.
    Shown,
    /// Hidden as spoilers until each person shows them.
    Spoiler,
    /// Only for adults, who confirm their age first.
    Adults,
}

impl Visibility {
    /// What a channel marked so shows as: for adults wins over spoilers.
    pub fn of(spoiler: bool, nsfw: bool) -> Self {
        match (spoiler, nsfw) {
            (_, true) => Self::Adults,
            (true, false) => Self::Spoiler,
            (false, false) => Self::Shown,
        }
    }

    /// The last part of its choice's id, `cs:visibility:{key}`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Shown => "default",
            Self::Spoiler => "spoiler",
            Self::Adults => "nsfw",
        }
    }

    pub const ALL: [Self; 3] = [Self::Shown, Self::Spoiler, Self::Adults];
}

/// One choice among a few, one under the other, each with its note under
/// its name, as the account's settings draw them (`user_settings::choices`);
/// `options` are each its id, its name, its note and whether it can be chosen.
pub fn radio(
    id: &str,
    label: &str,
    options: &[(String, String, Option<String>, bool)],
    selected: &str,
    look: &Look,
) -> Element {
    let options = options
        .iter()
        .map(|(value, name, note, enabled)| {
            let option = RadioOption::new(value.clone(), name.clone()).enabled(*enabled);
            match note {
                Some(note) => option.description(note.clone()),
                None => option,
            }
        })
        .collect();
    let group = RadioGroup::new(id, options, Some(selected), look.palette())
        .variant(RadioVariant::Plain)
        .size(ControlSize::Spacious)
        .metrics(RadioMetrics {
            circle: 20.0,
            option_gap: 16.0,
            description_size: 14.0,
            description_gap: 4.0,
            label_size: 16.0,
        })
        .accessibility_label(label)
        .render();
    // Discord's group keeps 8 px over its first choice and 4 under its last.
    Element::column(vec![gap(0.0, 8.0), group, gap(0.0, 4.0)])
}

/// «Visibilidad del contenido»: shown, spoilers or for adults, each with
/// its note (`cs:visibility:{key}`); `enabled` says which can be chosen,
/// in [`Visibility::ALL`]'s order.
pub fn visibility(chosen: Visibility, enabled: [bool; 3], look: &Look) -> Element {
    let notes = [
        (t("Predeterminado"), t("El contenido del canal siempre es visible.")),
        (
            t("Canal de spoilers"),
            t("Indica que este canal contiene spoilers para que los giros de trama, temas delicados u otras discusiones sensibles permanezcan ocultos hasta que los usuarios decidan verlos."),
        ),
        (
            t("Canal con restricción de edad"),
            t("Los usuarios deberán confirmar que son mayores de edad para ver el contenido de este canal. Los canales con restricción de edad están exentos del filtro de contenido explícito."),
        ),
    ];
    let options: Vec<_> = Visibility::ALL
        .iter()
        .zip(notes)
        .zip(enabled)
        .map(|((which, (name, note)), enabled)| {
            (
                format!("cs:visibility:{}", which.key()),
                name.to_owned(),
                Some(note.to_owned()),
                enabled,
            )
        })
        .collect();
    let title = t("Visibilidad del contenido");
    field(
        title,
        radio(
            "cs:visibility",
            title,
            &options,
            &format!("cs:visibility:{}", chosen.key()),
            look,
        ),
        None,
        look,
    )
}

/// A slider whose marks are named over it at their own places, as
/// Discord's bitrate: `marks` are each a fraction along the track and its
/// label. The first label starts at the track's start, the last ends at its
/// end, the rest center on their mark; a dash crosses the track at each.
pub fn marked_slider(
    id: &str,
    label: &str,
    fraction: f32,
    marks: &[(f32, String)],
    enabled: bool,
    width: f32,
    look: &Look,
) -> Element {
    /// Room a middle label is centered in.
    const LABEL_BOX: f32 = 80.0;
    const DASH: Size = Size::new(2.0, 24.0);
    let last = marks.len().saturating_sub(1);
    let labels = marks
        .iter()
        .enumerate()
        .map(|(index, (at, name))| {
            let name = line(name.clone(), 12.0, look.muted).font_family(FontFamily::SansMedium);
            match index {
                0 => Element::row(vec![name, Element::spacer().flex(1.0)]),
                _ if index == last => Element::row(vec![Element::spacer().flex(1.0), name]),
                _ => Element::row(vec![
                    gap((at * width - LABEL_BOX / 2.0).max(0.0), 0.0),
                    Element::row(vec![
                        Element::spacer().flex(1.0),
                        name,
                        Element::spacer().flex(1.0),
                    ])
                    .size(Some(LABEL_BOX), None),
                ]),
            }
        })
        .collect();
    let dashes = marks
        .iter()
        .map(|(at, _)| {
            Element::row(vec![
                gap((at * (width - DASH.width)).max(0.0), 0.0),
                Element::panel(Element::spacer())
                    .size(Some(DASH.width), Some(DASH.height))
                    .radius(1.0)
                    .background(look.border),
            ])
        })
        .collect();
    let slider = crate::user_settings::slider(id, fraction, look)
        .width(width)
        .enabled(enabled)
        .accessibility_label(label)
        .render();
    Element::column(vec![
        Element::stack(labels).size(Some(width), Some(16.0)),
        gap(0.0, 4.0),
        Element::stack(vec![
            Element::stack(dashes),
            Element::column(vec![
                Element::spacer().flex(1.0),
                slider,
                Element::spacer().flex(1.0),
            ]),
        ])
        .size(Some(width), Some(DASH.height)),
    ])
}

/// An invitation to the channel, as its list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct InviteRow {
    pub code: String,
    /// Who made it: their name, accent and picture once loaded.
    pub inviter: Option<(String, Color, Option<limn::RasterImage>)>,
    pub uses: String,
    /// What is left until it stops working, or «∞».
    pub expires: String,
    /// What revokes it, when the account may.
    pub revoke: Option<String>,
}

/// The widths of the list's columns, as Discord's: who made it, its code,
/// its uses and when it stops working, each with its room after it.
const INVITE_COLUMNS: [f32; 4] = [203.3, 162.5, 70.9, 121.6];

/// The channel's invitations as Discord's table: its headings and a row
/// each, between two lines; a row that may be revoked ends in a ×.
pub fn invite_table(rows: &[InviteRow], look: &Look) -> Element {
    let rule = || {
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border)
    };
    let cells = |cells: Vec<Element>, end: Option<Element>| {
        let mut row: Vec<Element> = cells
            .into_iter()
            .zip(INVITE_COLUMNS)
            .map(|(cell, width)| cell.centered_y().size(Some(width), None))
            .collect();
        row.push(Element::spacer().flex(1.0));
        if let Some(end) = end {
            row.push(end.centered_y());
        }
        Element::row(row)
    };
    let heading = |label: &str| strong_line(label, 16.0, look.text);
    let mut column = vec![
        rule(),
        gap(0.0, 20.0),
        cells(
            vec![
                heading(t("Persona que invita")),
                heading(t("Código de invitación")),
                heading(t("Usos")),
                heading(t("Caduca")),
            ],
            None,
        )
        .size(None, Some(20.0)),
    ];
    for row in rows {
        let inviter = match &row.inviter {
            Some((name, accent, image)) => Element::row(vec![
                crate::avatar_with_image(name, *accent, 24.0, image.clone()).centered_y(),
                line(name.clone(), 16.0, look.text).centered_y(),
            ])
            .gap(12.0),
            None => line("—", 16.0, look.muted),
        };
        let revoke = row.revoke.as_ref().map(|id| {
            Button::new(id.clone(), "", look.palette())
                .icon(Icon::Close)
                .variant(ButtonVariant::Ghost)
                .button_size(ButtonSize::Small)
                .accessibility_label(tr!("Revocar la invitación {code}", code = row.code))
                .tooltip(t("Revocar"))
                .render()
        });
        column.push(gap(0.0, 12.0));
        column.push(
            cells(
                vec![
                    inviter,
                    line(row.code.clone(), 14.0, look.text).font_family(FontFamily::Monospace),
                    line(row.uses.clone(), 14.0, look.text).font_family(FontFamily::Monospace),
                    line(row.expires.clone(), 14.0, look.text).font_family(FontFamily::Monospace),
                ],
                revoke,
            )
            .size(None, Some(32.0)),
        );
    }
    column.push(gap(0.0, 12.0));
    column.push(rule());
    Element::column(column)
}

/// What a channel's integrations card says: its glyph, its title, how many
/// it holds, and its button (`id`, label, enabled) when it has one.
pub fn integration_card(
    glyph: Icon,
    title: &str,
    count: &str,
    action: Option<(&str, &str, bool)>,
    look: &Look,
) -> Element {
    let mut row = vec![
        gap(28.0, 0.0),
        Element::icon(glyph)
            .foreground(look.muted)
            .size(Some(24.0), Some(24.0))
            .centered_y(),
        gap(28.0, 0.0),
        Element::column(vec![
            strong_line(title, 16.0, look.text),
            line(count, 12.0, look.text),
        ])
        .gap(4.0)
        .flex(1.0)
        .centered_y(),
    ];
    if let Some((id, label, enabled)) = action {
        row.push(
            Button::new(id, label, look.accent_palette())
                .button_size(ButtonSize::Small)
                .enabled(enabled)
                .render()
                .centered_y(),
        );
    }
    row.push(gap(16.0, 0.0));
    Element::panel(Element::row(row))
        .size(None, Some(88.0))
        .radius(8.0)
        .background(look.raised)
}

/// A dialog's two buttons side by side, as wide as it: the way back and the
/// action.
pub fn dialog_buttons(back: (&str, &str), action: (&str, &str, bool), look: &Look) -> Element {
    let (id, label, enabled) = action;
    Element::row(vec![
        Button::new(back.0, back.1, look.menu_palette())
            .variant(ButtonVariant::Secondary)
            .size(ControlSize::Spacious)
            .render()
            .flex(1.0),
        Button::new(id, label, look.accent_palette())
            .size(ControlSize::Spacious)
            .enabled(enabled)
            .render()
            .flex(1.0),
    ])
    .gap(9.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};
    use limn::Size;

    const LOOK: Look = Look::DARK;

    #[test]
    fn each_kind_names_its_permissions_once_and_none_of_the_server() {
        let server_only = Permissions::ADMINISTRATOR
            | Permissions::MANAGE_SERVER
            | Permissions::KICK
            | Permissions::BAN
            | Permissions::TIMEOUT_MEMBERS
            | Permissions::MANAGE_NICKS
            | Permissions::CHANGE_NICK
            | Permissions::VIEW_AUDIT_LOG
            | Permissions::MANAGE_EMOJI;
        for kind in [ChannelKind::Text, ChannelKind::Voice, ChannelKind::Category] {
            let mut seen = Permissions::NONE;
            for (_, permissions) in permission_groups(kind) {
                for (permission, label, description) in *permissions {
                    assert!(!seen.intersects(*permission), "{label} twice in {kind:?}");
                    assert!(!label.is_empty() && !description.is_empty());
                    seen = seen | *permission;
                }
            }
            assert!(!seen.intersects(server_only), "{kind:?}");
            assert!(seen.contains(Permissions::VIEW | Permissions::MANAGE_ROLES));
        }
        let text = |kind| {
            permission_groups(kind)
                .iter()
                .flat_map(|(_, permissions)| permissions.iter())
                .fold(Permissions::NONE, |all, (permission, _, _)| {
                    all | *permission
                })
        };
        assert!(text(ChannelKind::Text).contains(Permissions::SEND));
        assert!(!text(ChannelKind::Text).intersects(Permissions::CONNECT));
        assert!(!text(ChannelKind::Voice).intersects(Permissions::SEND));
        assert!(text(ChannelKind::Category).contains(Permissions::SEND | Permissions::CONNECT));
        assert_eq!(permission_id(Permissions::SEND), "cs:perm:1");
    }

    #[test]
    fn slow_mode_names_its_waits() {
        assert_eq!(slowmode_label(0), "Apagar");
        assert_eq!(slowmode_label(21_600), "6 horas");
        assert_eq!(slowmode_label(45), "45 segundos");
        assert!(SLOWMODE.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn the_control_tells_its_three_states_and_which_is_chosen() {
        let control = tri_state("cs:perm:1", Some(false), true, "Enviar mensajes", &LOOK);
        let (_, laid) = frame(control, Size::new(200.0, 40.0));
        assert_eq!(
            controls(&laid),
            vec![
                "cs:perm:1:deny".to_owned(),
                "cs:perm:1:inherit".to_owned(),
                "cs:perm:1:allow".to_owned()
            ]
        );
        let chosen = |id: &str| {
            laid.access
                .iter()
                .find(|node| node.id == id)
                .and_then(|node| node.selected)
        };
        assert_eq!(chosen("cs:perm:1:deny"), Some(true));
        assert_eq!(chosen("cs:perm:1:allow"), Some(false));
        let locked = tri_state("cs:perm:1", None, false, "Enviar mensajes", &LOOK);
        let (_, laid) = frame(locked, Size::new(200.0, 40.0));
        assert!(controls(&laid).is_empty());
    }

    #[test]
    fn the_private_card_lists_who_may_see_it_only_while_on() {
        let card = PrivateCard {
            category: false,
            on: false,
            enabled: true,
            roles: Vec::new(),
            members: Vec::new(),
        };
        let (_, laid) = frame(private_card(&card, &LOOK), Size::new(705.0, 400.0));
        assert_eq!(controls(&laid), vec!["cs:private".to_owned()]);
        let on = PrivateCard {
            on: true,
            members: vec![AccessLine {
                name: "Ana".into(),
                detail: "ana".into(),
                color: LOOK.accent,
                person: true,
                image: None,
                note: Some("Propietario del servidor".into()),
                remove: None,
            }],
            roles: vec![AccessLine {
                name: "Moderación".into(),
                detail: "Rol".into(),
                color: LOOK.danger,
                person: false,
                image: None,
                note: None,
                remove: Some("cs:access:role:21:remove".into()),
            }],
            ..card
        };
        let (_, laid) = frame(private_card(&on, &LOOK), Size::new(705.0, 400.0));
        let ids = controls(&laid);
        assert!(ids.contains(&"cs:access:add".to_owned()), "{ids:?}");
        assert!(ids.contains(&"cs:access:role:21:remove".to_owned()));
        let painted = texts(&laid);
        assert!(painted.contains(&"Propietario del servidor".to_owned()));
    }

    #[test]
    fn a_field_select_opens_its_list_and_checks_the_choice() {
        let select = FieldSelect {
            id: "cs:slowmode".into(),
            name: "Modo lento".into(),
            value: "Apagado".into(),
            options: SLOWMODE
                .iter()
                .map(|(secs, label)| {
                    (
                        format!("cs:slowmode:{secs}"),
                        (*label).to_owned(),
                        *secs == 0,
                    )
                })
                .collect(),
            open: false,
            dismiss: "cs:slowmode:dismiss".into(),
            enabled: true,
            width: 705.0,
        };
        let alone = |select: &FieldSelect| {
            Element::column(vec![
                field_select(select, &LOOK),
                Element::spacer().flex(1.0),
            ])
        };
        let (_, laid) = frame(alone(&select), Size::new(705.0, 400.0));
        assert_eq!(controls(&laid), vec!["cs:slowmode".to_owned()]);
        let trigger = laid
            .regions
            .iter()
            .find(|region| region.id == "cs:slowmode")
            .unwrap();
        assert_eq!((trigger.rect.width, trigger.rect.height), (705.0, 41.0));
        let open = FieldSelect {
            open: true,
            ..select.clone()
        };
        let (_, laid) = frame(alone(&open), Size::new(705.0, 400.0));
        assert!(controls(&laid).contains(&"cs:slowmode:5".to_owned()));
        let disabled = FieldSelect {
            enabled: false,
            open: true,
            ..select
        };
        let (_, laid) = frame(alone(&disabled), Size::new(705.0, 400.0));
        assert!(
            controls(&laid).is_empty(),
            "disabled, it neither opens nor reports"
        );
    }

    #[test]
    fn the_sync_card_offers_to_sync_only_when_out_of_sync() {
        let (_, laid) = frame(
            sync_card("Desarrollo", true, true, &LOOK),
            Size::new(705.0, 80.0),
        );
        assert!(controls(&laid).is_empty());
        let (_, laid) = frame(
            sync_card("Desarrollo", false, true, &LOOK),
            Size::new(705.0, 80.0),
        );
        assert_eq!(controls(&laid), vec!["cs:sync".to_owned()]);
    }

    /// Where a painted text starts, the first time it is painted.
    fn painted_at(laid: &limn::Frame, wanted: &str) -> Option<limn::Point> {
        laid.commands.iter().find_map(|command| match command {
            limn::DrawCommand::Text { text, position, .. } if text == wanted => Some(*position),
            _ => None,
        })
    }

    #[test]
    fn the_visibility_offers_three_choices_and_locks_those_it_cannot_make() {
        let (_, laid) = frame(
            visibility(Visibility::Spoiler, [true, true, false], &LOOK),
            Size::new(660.0, 400.0),
        );
        let ids = controls(&laid);
        assert!(ids.contains(&"cs:visibility:default".to_owned()), "{ids:?}");
        assert!(
            !ids.contains(&"cs:visibility:nsfw".to_owned()),
            "locked: {ids:?}"
        );
        let painted = texts(&laid);
        for wanted in [
            "Visibilidad del contenido",
            "Predeterminado",
            "Canal de spoilers",
            "Canal con restricción de edad",
        ] {
            assert!(painted.iter().any(|text| text == wanted), "{wanted}");
        }
        let chosen = laid
            .access
            .iter()
            .find(|node| node.id == "cs:visibility:spoiler")
            .and_then(|node| node.selected);
        assert_eq!(chosen, Some(true));
        assert_eq!(Visibility::of(true, true), Visibility::Adults);
        assert_eq!(Visibility::of(false, false).key(), "default");
    }

    #[test]
    fn a_marked_slider_names_its_marks_where_they_fall() {
        let marks = [
            (0.0, "8kbps".to_owned()),
            (56.0 / 88.0, "64kbps".to_owned()),
            (1.0, "96kbps".to_owned()),
        ];
        let (_, laid) = frame(
            Element::column(vec![marked_slider(
                "cs:bitrate",
                "Velocidad de transferencia",
                0.5,
                &marks,
                true,
                660.0,
                &LOOK,
            )]),
            Size::new(660.0, 100.0),
        );
        assert!(laid.regions.iter().any(|region| region.id == "cs:bitrate"));
        let first = painted_at(&laid, "8kbps").unwrap();
        let usual = painted_at(&laid, "64kbps").unwrap();
        let last = painted_at(&laid, "96kbps").unwrap();
        assert!(first.x.abs() < 1.0, "the first starts at the start");
        // 7 px characters: «64kbps» is 42 wide, centered on its mark.
        assert!((usual.x + 21.0 - 420.0).abs() < 2.0, "{usual:?}");
        assert!(
            (last.x + 42.0 - 660.0).abs() < 1.0,
            "the last ends at the end"
        );
        let (_, locked) = frame(
            marked_slider("cs:bitrate", "x", 0.5, &marks, false, 660.0, &LOOK),
            Size::new(660.0, 100.0),
        );
        assert!(
            !locked
                .regions
                .iter()
                .any(|region| region.id == "cs:bitrate" && region.kind == limn::HitKind::Button),
            "a locked slider takes no pointer"
        );
    }

    #[test]
    fn the_invitations_list_its_columns_and_revoke_only_where_allowed() {
        let rows = [
            InviteRow {
                code: "abc".into(),
                inviter: Some(("Ana".into(), LOOK.accent, None)),
                uses: "3".into(),
                expires: "08:04:40:49".into(),
                revoke: Some("cs:invite:abc:revoke".into()),
            },
            InviteRow {
                code: "def".into(),
                inviter: None,
                uses: "0".into(),
                expires: "∞".into(),
                revoke: None,
            },
        ];
        let (_, laid) = frame(invite_table(&rows, &LOOK), Size::new(660.0, 300.0));
        assert_eq!(controls(&laid), vec!["cs:invite:abc:revoke".to_owned()]);
        let painted = texts(&laid);
        for wanted in [
            "Persona que invita",
            "Código de invitación",
            "Usos",
            "Caduca",
            "Ana",
            "abc",
            "08:04:40:49",
            "∞",
        ] {
            assert!(painted.iter().any(|text| text == wanted), "{wanted}");
        }
        // The code's column starts where Discord's does: 203 px in.
        assert!((painted_at(&laid, "abc").unwrap().x - 203.3).abs() < 1.0);
    }

    #[test]
    fn an_integrations_card_counts_and_offers_its_action() {
        let card = integration_card(
            Icon::Bolt,
            "Webhooks",
            "0 webhooks",
            Some(("cs:webhook:new", "Crear Webhook", true)),
            &LOOK,
        );
        let (_, laid) = frame(card, Size::new(660.0, 88.0));
        assert_eq!(controls(&laid), vec!["cs:webhook:new".to_owned()]);
        assert!(texts(&laid).contains(&"0 webhooks".to_owned()));
        let locked = integration_card(
            Icon::Bolt,
            "Webhooks",
            "0 webhooks",
            Some(("cs:webhook:new", "Crear Webhook", false)),
            &LOOK,
        );
        let (_, laid) = frame(locked, Size::new(660.0, 88.0));
        assert!(controls(&laid).is_empty());
    }

    #[test]
    fn a_field_puts_its_note_under_its_control() {
        let (_, laid) = frame(
            field(
                "Modo lento",
                gap(0.0, 40.0),
                Some(note(&[("Selecciona ", false), ("Automática", true)], &LOOK)),
                &LOOK,
            ),
            Size::new(660.0, 200.0),
        );
        let title = painted_at(&laid, "Modo lento").unwrap();
        let note = painted_at(&laid, "Automática").unwrap();
        // The title's line, 8 px, the control's 40 and 4 more.
        assert!(note.y - title.y >= 8.0 + 40.0 + 4.0, "{title:?} {note:?}");
        assert_eq!(archive_label(4320), "3 días");
    }
}
