//! The friends page of home, from plain props, as the official client draws
//! it (`docs/discord-reference.md`, «Friends and Active Now»): its bar with
//! the tabs, the note that points to the blocked people, the search, a person
//! with their round actions and the menu of the rest, the form that sends a
//! request by handle, and the question before ending something.

use limn::{
    AlertDialog, Button, ButtonSizing, ButtonVariant, Color, Component, ComponentPalette,
    ControlSize, DropdownMenu, DropdownMenuEntry, DropdownMenuItem, Element, Empty, FontFamily,
    Icon, IconButton, PopoverPlacement, SearchField, TextField, TextSpan,
};

use rustcord_i18n::{t, tr};

use crate::glyphs::{CHAT_PLUS, FRIENDS, MORE_VERTICAL};
use crate::{avatar_with_image, gap, icon, line, strong_line, text, with_status, Look};

/// The actions these pieces report. A person's own end with their ID and
/// the action: `friend:{user}:accept`.
/// The bar's «Nuevo grupo de MD»: its own id, as the + of the direct
/// column opens the same and limn keeps hover state by id.
pub const NEW_GROUP: &str = "friends:new-group";

pub mod ids {
    /// The row over the direct conversations that opens this page.
    pub const HOME: &str = "home:friends";
    /// Followed by the key of a [`super::FriendsTab`].
    pub const TAB: &str = "friends:tab:";
    pub const QUERY: &str = "friends:query";
    pub const USERNAME: &str = "friends:add:username";
    pub const SEND: &str = "friends:add:send";
    /// The note's link to the blocked and ignored people, in the settings.
    pub const NOTICE_SETTINGS: &str = "friends:notice:settings";
    /// The note's ×: it does not come back.
    pub const NOTICE_CLOSE: &str = "friends:notice:close";
    pub const MENU_DISMISS: &str = "friends:menu:dismiss";
    pub const CONFIRM: &str = "friends:confirm";
    pub const CONFIRM_CANCEL: &str = "friends:confirm:cancel";
    /// The dialog that gives a friend a private name.
    pub const NICKNAME: &str = "friends:nickname";
    pub const NICKNAME_SAVE: &str = "friends:nickname:save";
    pub const NICKNAME_CLEAR: &str = "friends:nickname:clear";
    pub const NICKNAME_CLOSE: &str = "friends:nickname:close";
    /// The question about ignoring offers blocking instead, as a link.
    pub const IGNORE_BLOCK: &str = "friends:ignore:block";

    /// What a person's control reports: `friend:{user}:{action}`.
    pub fn person(user: impl std::fmt::Display, action: &str) -> String {
        format!("friend:{user}:{action}")
    }
}

/// A person's row is this high: 61 and the hairline over it.
pub const ROW: f32 = 62.0;
/// A section's heading is this high.
pub const SECTION: f32 = 50.0;
/// The question before removing or blocking.
pub const CONFIRM_WIDTH: f32 = 440.0;

/// The edge of the search and of the form: `#96969f` at 24 %; also the dot
/// between the page's name and its tabs.
const EDGE: Color = Color::rgba(150, 150, 159, 61);
/// The fill of the search and of the form: black at 12 %.
const FIELD_FILL: Color = Color::rgba(0, 0, 0, 31);
/// From the list's left edge to a row, a hairline and a heading's words.
const ROW_LEFT: f32 = 14.0;
const LINE_LEFT: f32 = 30.0;
const HEADING_LEFT: f32 = 24.0;
/// From the list's right edge to a row, and to a hairline or a heading:
/// Discord's margins and its 16 px scrollbar gutter.
const ROW_RIGHT: f32 = 26.0;
const LINE_RIGHT: f32 = 36.0;

/// Which people the page shows, or the form that adds one. The blocked ones
/// are in the settings, as Discord has them: the note says where.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FriendsTab {
    #[default]
    Online,
    All,
    Pending,
    Add,
}

impl FriendsTab {
    pub const ALL: [Self; 4] = [Self::Online, Self::All, Self::Pending, Self::Add];

    pub fn key(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::All => "all",
            Self::Pending => "pending",
            Self::Add => "add",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Online => t("En línea"),
            Self::All => t("Todos"),
            Self::Pending => t("Pendiente"),
            Self::Add => t("Añadir amigos"),
        }
    }
}

/// Something that can be done about a person, and why not when it cannot.
#[derive(Clone, Debug, PartialEq)]
pub struct FriendAction {
    pub id: String,
    pub label: String,
    pub icon: Icon,
    pub denied: Option<String>,
    /// It ends something: drawn in the danger color (a round button, under
    /// the pointer).
    pub danger: bool,
}

/// One person of the page.
#[derive(Clone, Debug, PartialEq)]
pub struct FriendRow {
    pub name: String,
    /// Shown beside the name while the pointer is on the row; `None` when
    /// the line under the name already is the handle.
    pub handle: Option<String>,
    pub image: Option<limn::RasterImage>,
    pub accent: Color,
    /// Their status dot, when the provider reports presence.
    pub status: Option<Color>,
    /// Their status, or their handle for a request.
    pub detail: String,
    /// A server's emoji before `detail`, when that is their custom status.
    pub detail_emoji: Option<crate::identity::TextPicture>,
    /// What a click on the row reports, and what that does: their
    /// conversation, or their profile.
    pub open: (String, String),
    /// The round buttons, in order.
    pub actions: Vec<FriendAction>,
    /// What the "more" button offers; empty, there is no such button.
    pub more: Vec<FriendAction>,
    /// The ID of the "more" button, and whether its menu is open.
    pub more_id: String,
    pub menu_open: bool,
}

/// The row over the direct conversations that opens this page, with the
/// count of requests waiting for the account.
pub fn home_entry(selected: bool, pending: usize, look: &Look) -> Element {
    let ink = if selected { look.text } else { look.dim };
    let mut parts = vec![
        gap(10.0, 0.0),
        icon(Icon::Custom(FRIENDS), ink)
            .large_icon(20.0)
            .centered_y(),
        gap(12.0, 0.0),
        strong_line(t("Amigos"), 16.0, ink).flex(1.0).centered_y(),
    ];
    if pending > 0 {
        parts.push(crate::sidebar::count(pending as u32, look).centered_y());
    }
    parts.push(gap(16.0, 0.0));
    let body = Element::clickable(ids::HOME, Element::row(parts))
        .size(None, Some(38.0))
        .radius(8.0)
        .background(if selected {
            look.selected
        } else {
            Color::TRANSPARENT
        })
        .hover_background(if selected { look.selected } else { look.hover })
        .hover_foreground(look.text)
        .accessible(
            limn::Access::new(limn::Role::ListItem)
                .name(t("Amigos"))
                .selected(selected),
        );
    Element::row(vec![
        gap(crate::sidebar::INSET, 0.0),
        body.flex(1.0),
        gap(crate::sidebar::INSET, 0.0),
    ])
}

/// The bar over the page: what it is, a dot, its tabs as pills («Añadir
/// amigos» in the accent) and, while the account may make one, a new group
/// at its end, `end` short of it (the window's own buttons are there).
/// `pending` counts the requests waiting for the account.
pub fn header(tab: FriendsTab, pending: usize, new_group: bool, end: f32, look: &Look) -> Element {
    let mut strip = vec![
        gap(6.0, 0.0),
        icon(Icon::Custom(FRIENDS), look.dim)
            .large_icon(20.0)
            .centered_y(),
        gap(6.0, 0.0),
        strong_line(t("Amigos"), 16.0, look.text)
            .shrink_to_fit()
            .centered_y(),
        gap(12.0, 0.0),
        Element::panel(Element::spacer())
            .size(Some(4.0), Some(4.0))
            .radius(2.0)
            .background(EDGE)
            .centered_y(),
        gap(4.0, 0.0),
    ];
    for candidate in FriendsTab::ALL {
        strip.push(gap(8.0, 0.0));
        strip.push(tab_pill(candidate, candidate == tab, pending, look).centered_y());
        strip.push(gap(8.0, 0.0));
    }
    let mut parts = vec![
        gap(20.0, 0.0),
        // A narrow window scrolls the tabs sideways instead of cutting them.
        Element::horizontal_scroll("friends:tabs-scroll", Element::row(strip)).flex(1.0),
    ];
    if new_group {
        let palette = ComponentPalette {
            hover: Color::TRANSPARENT,
            ..look.palette()
        };
        parts.push(gap(10.0, 0.0));
        parts.push(
            IconButton::new(NEW_GROUP, Icon::Custom(CHAT_PLUS), palette)
                .variant(ButtonVariant::Ghost)
                .sizing(ButtonSizing {
                    height: 32.0,
                    font_size: 14.0,
                    icon_size: 20.0,
                    horizontal_padding: 6.0,
                    inner_spacing: 0.0,
                })
                .tooltip(t("Nuevo grupo de MD"))
                .tooltip_placement(limn::TooltipPlacement::Below)
                .ink(look.dim)
                .hover_ink(look.text)
                .render()
                .centered_y(),
        );
    }
    parts.push(gap(8.0 + end, 0.0));
    Element::panel(Element::row(parts))
        .size(None, Some(crate::sidebar::HEADER))
        .background(look.canvas)
        .accessible(limn::Access::new(limn::Role::Group).name(t("Amigos")))
}

/// One tab: a pill 32 high, its words 16 medium; the chosen one filled, the
/// others filled only under the pointer. «Añadir amigos» is the accent's,
/// faint while it is the one shown.
fn tab_pill(tab: FriendsTab, active: bool, pending: usize, look: &Look) -> Element {
    let label = tab.label();
    let (fill, hover, ink) = match (tab, active) {
        (FriendsTab::Add, false) => (look.accent, shade(look.accent), look.on_accent),
        (FriendsTab::Add, true) => {
            let faint = Look::alpha(look.accent, 41);
            (faint, faint, look.accent)
        }
        (_, true) => (look.selected, look.selected, look.text),
        (_, false) => (Color::TRANSPARENT, look.hover, look.muted),
    };
    let mut parts = vec![
        gap(12.0, 0.0),
        strong_line(label, 16.0, ink).shrink_to_fit().centered_y(),
    ];
    if tab == FriendsTab::Pending && pending > 0 {
        parts.push(gap(8.0, 0.0));
        parts.push(crate::sidebar::count(pending as u32, look).centered_y());
    }
    parts.push(gap(12.0, 0.0));
    let pill = Element::clickable(format!("{}{}", ids::TAB, tab.key()), Element::row(parts))
        .size(None, Some(32.0))
        .radius(8.0)
        .background(fill)
        .hover_background(hover);
    // The accent's words stay white under the pointer; the others brighten.
    let pill = if tab == FriendsTab::Add {
        pill
    } else {
        pill.hover_foreground(look.text)
    };
    let name = match pending {
        _ if tab != FriendsTab::Pending => label.to_owned(),
        0 => label.to_owned(),
        1 => tr!("{label}, {count} nuevo", label, count = pending),
        _ => tr!("{label}, {count} nuevos", label, count = pending),
    };
    pill.accessible(
        limn::Access::new(limn::Role::Tab)
            .name(name)
            .selected(active),
    )
}

/// The accent under the pointer: Discord's `#5865f2` turns `#4452bb`.
fn shade(color: Color) -> Color {
    let dark = |channel: u8| (f32::from(channel) * 0.78).round() as u8;
    Color::rgba(dark(color.r), dark(color.g), dark(color.b), color.a)
}

/// The note over the list: the blocked and ignored people are in the
/// settings. Its link opens them there; its × closes it for good.
pub fn notice(look: &Look) -> Element {
    // One paragraph, so a narrow page wraps it with its link.
    let said = Element::formatted_text(vec![
        TextSpan::new(
            format!(
                "{} ",
                t("¿Estás buscando las cuentas que bloqueaste o ignoraste?")
            ),
            look.text,
            14.0,
            FontFamily::SansMedium,
        ),
        TextSpan::new(
            t("Dirígete a ajustes"),
            look.palette().link(),
            14.0,
            FontFamily::SansMedium,
        )
        .action(ids::NOTICE_SETTINGS),
    ]);
    let close = IconButton::new(
        ids::NOTICE_CLOSE,
        Icon::Close,
        ComponentPalette {
            hover: Color::TRANSPARENT,
            ..look.palette()
        },
    )
    .variant(ButtonVariant::Ghost)
    .sizing(ButtonSizing {
        height: 27.0,
        font_size: 14.0,
        icon_size: 24.0,
        horizontal_padding: 0.0,
        inner_spacing: 0.0,
    })
    .accessibility_label(t("Cerrar"))
    .ink(look.muted)
    .hover_ink(look.text)
    .render();
    let body = Element::row(vec![
        icon(Icon::InfoCircle, look.muted)
            .large_icon(24.0)
            .centered_y(),
        gap(8.0, 0.0),
        said.flex(1.0).centered_y(),
        gap(8.0, 0.0),
        close.centered_y(),
    ])
    .min_size(0.0, 27.0);
    let note = Element::panel(body)
        .padding(16.0)
        .radius(4.0)
        .background(look.canvas);
    Element::column(vec![
        gap(0.0, 16.0),
        Element::row(vec![gap(30.0, 0.0), note.flex(1.0), gap(20.0, 0.0)]),
        gap(0.0, 8.0),
    ])
}

/// The search over the people of a tab: 40 high, rounded 8, its edge in the
/// accent while it has the focus; 24 and 16 from the page's sides.
pub fn search(query: &str, look: &Look) -> Element {
    let palette = ComponentPalette {
        raised: FIELD_FILL,
        border: EDGE,
        ..look.menu_palette()
    };
    let field = SearchField::new(ids::QUERY, query, t("Buscar"), palette)
        .accessibility_label(t("Buscar amigos"))
        .height(40.0)
        .render()
        .radius(8.0)
        .font_size(16.0)
        .leading_icon_tint(look.text)
        .hover_background(FIELD_FILL)
        .focus_border(look.accent, 1.5);
    Element::column(vec![
        gap(0.0, 12.0),
        Element::row(vec![gap(24.0, 0.0), field.flex(1.0), gap(16.0, 0.0)]),
        gap(0.0, 8.0),
    ])
}

/// A heading over a group of people, as given: «2 en línea», «Recibidas — 1».
pub fn section(title: &str, look: &Look) -> Element {
    Element::row(vec![
        gap(HEADING_LEFT, 0.0),
        strong_line(title, 14.0, look.text).flex(1.0).centered_y(),
        gap(LINE_RIGHT, 0.0),
    ])
    .size(None, Some(SECTION))
}

/// A person of the page under a hairline, with their round actions on the
/// right. The whole row opens [`FriendRow::open`]; under the pointer it is
/// filled, its hairline goes and the handle shows beside the name.
pub fn row(row: &FriendRow, look: &Look) -> Element {
    let face = avatar_with_image(&row.name, row.accent, 32.0, row.image.clone());
    let face = match row.status {
        Some(status) => with_status(face, 32.0, status, look.canvas),
        None => face,
    };
    let mut name = vec![strong_line(&row.name, 16.0, look.text)
        .font_family(FontFamily::SansSemibold)
        .shrink_to_fit()];
    if let Some(handle) = &row.handle {
        name.push(gap(5.0, 0.0));
        name.push(
            line(handle, 14.0, look.text)
                .shrink_to_fit()
                .reveal_on_hover(format!("{}:handle", row.more_id)),
        );
    }
    let mut parts = vec![
        face.centered_y(),
        gap(12.0, 0.0),
        Element::column(vec![
            Element::row(name),
            crate::identity::status_line(
                Some(&row.detail),
                row.detail_emoji.as_ref(),
                14.0,
                look.dim,
                look,
            )
            .unwrap_or_else(|| line("", 14.0, look.dim)),
        ])
        .flex(1.0)
        .centered_y(),
    ];
    for (at, action) in row.actions.iter().enumerate() {
        if at > 0 {
            parts.push(gap(10.0, 0.0));
        }
        parts.push(round(action, look).centered_y());
    }
    if !row.more.is_empty() {
        let anchor = round(
            &FriendAction {
                id: row.more_id.clone(),
                label: t("Más").into(),
                icon: Icon::Custom(MORE_VERTICAL),
                denied: None,
                danger: false,
            },
            look,
        );
        let entries = row.more.iter().map(|action| entry(action, look)).collect();
        if !row.actions.is_empty() {
            parts.push(gap(10.0, 0.0));
        }
        parts.push(
            DropdownMenu::new(
                anchor,
                entries,
                row.menu_open,
                ids::MENU_DISMISS,
                look.menu_palette(),
            )
            .placement(PopoverPlacement::BottomEnd)
            .width(220.0)
            .render()
            .centered_y(),
        );
    }
    let (open, said) = &row.open;
    let body = Element::clickable(
        open.clone(),
        Element::row(vec![
            gap(10.0, 0.0),
            Element::row(parts).flex(1.0),
            gap(10.0, 0.0),
        ]),
    )
    .size(None, Some(ROW - 1.0))
    .radius(8.0)
    .hover_background(look.hover)
    .accessibility_label(said.clone());
    let hairline = Element::row(vec![
        gap(LINE_LEFT, 0.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border)
            .flex(1.0),
        gap(LINE_RIGHT, 0.0),
    ])
    .size(None, Some(1.0))
    .conceal_on_hover(format!("{}:line", row.more_id));
    Element::column(vec![
        hairline,
        Element::row(vec![
            gap(ROW_LEFT, 0.0),
            body.flex(1.0),
            gap(ROW_RIGHT, 0.0),
        ]),
    ])
    .size(None, Some(ROW))
    .hover_region()
    .accessible(limn::Access::new(limn::Role::ListItem).name(row.name.clone()))
}

/// A round action of a row: 36 across with a 20 px glyph, on the page's own
/// ground (it shows only on a row under the pointer), its glyph brightening
/// under the pointer, or turning the danger color for one that ends
/// something.
fn round(action: &FriendAction, look: &Look) -> Element {
    let hint = match &action.denied {
        Some(reason) => format!("{}: {reason}", action.label),
        None => action.label.clone(),
    };
    let palette = ComponentPalette {
        hover: Color::TRANSPARENT,
        ..look.menu_palette()
    };
    let button = IconButton::new(action.id.clone(), action.icon, palette)
        .variant(ButtonVariant::Ghost)
        .sizing(ButtonSizing {
            height: 36.0,
            font_size: 14.0,
            icon_size: 20.0,
            horizontal_padding: 8.0,
            inner_spacing: 0.0,
        })
        .round()
        .tooltip(hint.clone())
        .accessibility_label(hint)
        .ink(look.muted)
        .hover_ink(if action.danger {
            look.danger
        } else {
            look.text
        })
        .enabled(action.denied.is_none())
        .render();
    Element::panel(button).radius(18.0).background(look.canvas)
}

fn entry(action: &FriendAction, look: &Look) -> DropdownMenuEntry {
    let item = DropdownMenuItem::new(action.id.clone(), action.label.clone()).icon(action.icon);
    let item = match &action.denied {
        Some(reason) => item.description(reason.clone()).disabled(),
        None if action.danger => item.destructive(look.danger),
        None => item,
    };
    DropdownMenuEntry::Item(item)
}

/// What the page says when a tab has nobody to show, or cannot show them.
pub fn empty(title: &str, detail: &str, look: &Look) -> Element {
    crate::centered(
        Empty::new(title, detail, look.palette())
            .icon(Icon::Custom(FRIENDS))
            .render(),
    )
}

/// The form that asks someone to be friends by their handle.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AddFriend {
    pub username: String,
    /// The answer to the last request: what it says, and whether it failed.
    pub note: Option<(String, bool)>,
    /// Why requests cannot be sent, when they cannot.
    pub denied: Option<String>,
    /// A request left and has not been answered.
    pub sending: bool,
    /// The simulation answers: said so under the form.
    pub simulated: bool,
}

/// The «Añadir amigos» page: its title and what it is for, then the handle
/// in a box rounded 16 with the button inside its end, edged in the
/// positive or the danger color by the last answer, and that answer under it.
pub fn add_friend(form: &AddFriend, look: &Look) -> Element {
    let edge = match (&form.denied, &form.note) {
        (None, Some((_, false))) => look.online,
        (None, Some((_, true))) => look.danger,
        _ => EDGE,
    };
    let field = TextField::new(
        ids::USERNAME,
        form.username.clone(),
        t("Ingresa un nombre de usuario"),
        ComponentPalette {
            raised: FIELD_FILL,
            ..look.menu_palette()
        },
    )
    .accessibility_label(t("Nombre de usuario"))
    .height(56.0)
    .radius(16.0)
    .outline(edge, look.palette().link());
    let field = if form.denied.is_some() {
        field.disabled()
    } else {
        field
    }
    .render()
    .font_size(16.0)
    .hover_background(FIELD_FILL);
    let send = Button::new(
        ids::SEND,
        t("Enviar solicitud de amistad"),
        look.accent_palette(),
    )
    .size(ControlSize::Regular)
    .loading(form.sending)
    .enabled(form.denied.is_none() && !form.username.trim().is_empty() && !form.sending)
    .render();
    // shortcut: the button lies over the field's end, which a handle longer
    // than about 400 px would run under; limn keeps the room a field leaves
    // for such a control (`input_trailing_space`) to itself.
    let boxed = Element::stack(vec![
        field,
        Element::row(vec![
            Element::spacer().flex(1.0),
            send.centered_y(),
            gap(12.0, 0.0),
        ]),
    ])
    .size(None, Some(56.0));
    let mut parts = vec![
        strong_line(t("Añadir amigos"), 20.0, look.text).font_family(FontFamily::SansSemibold),
        gap(0.0, 8.0),
        strong_line(
            t("Puedes añadir amigos con su nombre de usuario."),
            16.0,
            look.text,
        ),
        gap(0.0, 16.0),
        boxed,
    ];
    let said = match (&form.denied, &form.note) {
        (Some(reason), _) => Some((reason.clone(), look.danger)),
        (None, Some((note, failed))) => Some((
            note.clone(),
            if *failed { look.danger } else { look.online },
        )),
        (None, None) => None,
    };
    if let Some((said, ink)) = said {
        parts.push(gap(0.0, 8.0));
        parts.push(text(said, 14.0, ink));
    }
    if form.simulated {
        parts.push(gap(0.0, 8.0));
        parts.push(text(
            t("Simulación local: las solicitudes se confirman en este equipo, sin enviar nada."),
            14.0,
            look.muted,
        ));
    }
    Element::column(vec![
        gap(0.0, 20.0),
        Element::row(vec![
            gap(30.0, 0.0),
            Element::column(parts).flex(1.0),
            gap(30.0, 0.0),
        ]),
        gap(0.0, 20.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
    ])
}

/// The dialog that gives `name`, a friend, a private name, over `root`.
pub fn nickname_dialog(root: Element, name: &str, value: &str, look: &Look) -> Element {
    let palette = look.menu_palette();
    let body = Element::column(vec![
        TextField::new(ids::NICKNAME, value.to_owned(), t("Sin apodo"), palette)
            .accessibility_label(t("Apodo"))
            .size(ControlSize::Spacious)
            .render(),
        text(
            t("Hasta 32 caracteres. Se ve en tu lista de amigos."),
            12.0,
            look.muted,
        ),
    ])
    .gap(8.0);
    let footer = Element::row(vec![
        Button::new(ids::NICKNAME_CLEAR, t("Quitar apodo"), palette)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Spacious)
            .render(),
        Element::spacer().flex(1.0),
        Button::new(ids::NICKNAME_CLOSE, t("Cancelar"), palette)
            .variant(ButtonVariant::Secondary)
            .size(ControlSize::Spacious)
            .render(),
        Button::new(ids::NICKNAME_SAVE, t("Guardar"), look.accent_palette())
            .size(ControlSize::Spacious)
            .render(),
    ])
    .gap(9.0);
    limn::Modal::new(
        root,
        true,
        t("Cambiar apodo"),
        tr!("Un nombre para {name} que solo ves tú.", name),
        ids::NICKNAME_CLOSE,
        palette,
    )
    .width(CONFIRM_WIDTH)
    .body(body)
    .footer(footer)
    .render()
}

/// The question before ending something with someone, over `root`.
pub fn confirm(root: Element, title: &str, body: &str, confirm: &str, look: &Look) -> Element {
    AlertDialog::new(
        root,
        true,
        title.to_owned(),
        body.to_owned(),
        ids::CONFIRM,
        ids::CONFIRM_CANCEL,
        look.menu_palette(),
    )
    .width(CONFIRM_WIDTH)
    .confirm_label(confirm.to_owned())
    .destructive(look.danger)
    .render()
}

/// Ignoring someone, as the official client asks: what it does and what it
/// does not, and blocking as the other way, offered in a link. Confirmed
/// with `ids::CONFIRM`, cancelled with `ids::CONFIRM_CANCEL`.
pub fn ignore(root: Element, name: &str, look: &Look) -> Element {
    let point = |words: &str| {
        Element::row(vec![
            text("•", 14.0, look.muted),
            gap(8.0, 0.0),
            text(words.to_owned(), 14.0, look.muted).flex(1.0),
        ])
    };
    let body = Element::column(vec![
        point(t("Ocultar su perfil y mensajes")),
        point(t("Puedes volver a mostrarlos en cualquier momento")),
        point(t("Silencia sus notificaciones y actividad")),
        point(t("Verás menos de esta persona")),
        point(t("Todavía pueden enviarte mensajes y ver tu actividad")),
        point(t("No sabrán que los ignoraste")),
        Element::column(vec![
            strong_line(t("¿No es suficiente?"), 14.0, look.text),
            Button::new(
                ids::IGNORE_BLOCK,
                t("Optar por bloquear"),
                look.menu_palette(),
            )
            .variant(ButtonVariant::Link)
            .size(ControlSize::Compact)
            .render(),
            text(
                t("Detén el contacto directo y limita lo que ven"),
                12.0,
                look.muted,
            ),
        ])
        .gap(4.0),
    ])
    .gap(8.0);
    AlertDialog::new(
        root,
        true,
        tr!("¿Deseas ignorar a {name}?", name),
        "",
        ids::CONFIRM,
        ids::CONFIRM_CANCEL,
        look.menu_palette(),
    )
    .width(CONFIRM_WIDTH)
    .confirm_label(t("Ignorar"))
    .body(body)
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};
    use limn::Size;

    #[test]
    fn the_ignore_question_names_the_person_and_offers_blocking_instead() {
        let (_, laid) = frame(
            ignore(Element::spacer(), "Ada", &Look::DARK),
            Size::new(600.0, 640.0),
        );
        let painted = texts(&laid);
        assert!(
            painted.contains(&"¿Deseas ignorar a Ada?".to_owned()),
            "{painted:?}"
        );
        assert!(painted.contains(&"Verás menos de esta persona".to_owned()));
        let shown = controls(&laid);
        assert!(shown.contains(&ids::IGNORE_BLOCK.to_owned()));
        assert!(shown.contains(&ids::CONFIRM.to_owned()));
    }

    fn person(actions: Vec<FriendAction>, more: Vec<FriendAction>, open: bool) -> FriendRow {
        FriendRow {
            name: "Ada".into(),
            handle: Some("ada".into()),
            image: None,
            accent: Color::rgb(90, 120, 200),
            status: Some(Look::DARK.online),
            detail: "En línea".into(),
            detail_emoji: None,
            open: (ids::person(10, "open"), "Abrir conversación con Ada".into()),
            actions,
            more,
            more_id: ids::person(10, "more"),
            menu_open: open,
        }
    }

    fn action(id: &str, denied: Option<&str>) -> FriendAction {
        FriendAction {
            id: id.into(),
            label: id.into(),
            icon: Icon::Chat,
            denied: denied.map(str::to_owned),
            danger: false,
        }
    }

    #[test]
    fn a_custom_status_shows_its_servers_emoji_before_its_words() {
        let emoji = crate::identity::TextPicture {
            token: "<:ok:1>".into(),
            name: "ok".into(),
            image: limn::RasterImage::new(
                "a".repeat(64),
                limn::RasterImageFormat::Png,
                3,
                std::sync::Arc::from("AAAA"),
                "ok",
            ),
            playing: false,
        };
        let shown = FriendRow {
            detail: "de vacaciones".into(),
            detail_emoji: Some(emoji),
            ..person(Vec::new(), Vec::new(), false)
        };
        let (_, frame) = frame(row(&shown, &Look::DARK), Size::new(700.0, 62.0));
        assert!(texts(&frame).contains(&"de vacaciones".to_owned()));
        assert!(frame.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::RasterImage { rect, .. } if rect.width == 14.0
        )));
    }

    #[test]
    fn the_header_offers_every_tab_counts_what_waits_and_keeps_clear_of_the_window_buttons() {
        let (_, laid) = frame(
            header(FriendsTab::Pending, 2, true, 138.0, &Look::DARK),
            Size::new(900.0, 52.0),
        );
        let found = controls(&laid);
        for tab in FriendsTab::ALL {
            assert!(
                found.contains(&format!("friends:tab:{}", tab.key())),
                "{found:?}"
            );
        }
        let painted = texts(&laid);
        assert!(painted.contains(&"Pendiente".to_owned()) && painted.contains(&"2".to_owned()));
        let group = laid
            .regions
            .iter()
            .find(|region| region.id == NEW_GROUP)
            .expect("the new group's button")
            .rect;
        assert!(group.x + group.width <= 900.0 - 138.0, "{group:?}");
        // Whoever may not make a group is not offered one.
        let (_, plain) = frame(
            header(FriendsTab::Online, 0, false, 0.0, &Look::DARK),
            Size::new(900.0, 52.0),
        );
        assert!(!controls(&plain).contains(&NEW_GROUP.to_owned()));
    }

    #[test]
    fn under_the_pointer_a_row_shows_the_handle_and_drops_its_hairline() {
        use limn::{Point, UiEvent};
        let size = Size::new(700.0, 62.0);
        let shown = |laid: &limn::Frame, wanted: &str| {
            laid.commands.iter().any(|command| {
                matches!(command, limn::DrawCommand::Text { text, color, .. } if text == wanted && color.a > 0)
            })
        };
        let hairline = |laid: &limn::Frame| {
            laid.commands.iter().any(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, color, .. } if rect.height == 1.0 && color.a > 0)
            })
        };
        let (mut tree, laid) = frame(
            row(&person(Vec::new(), Vec::new(), false), &Look::DARK),
            size,
        );
        assert!(!shown(&laid, "ada") && hairline(&laid));
        assert!(controls(&laid).contains(&ids::person(10, "open")));
        tree.handle_event(UiEvent::PointerMove(Point::new(200.0, 30.0)));
        let hovered = tree.layout(size);
        assert!(shown(&hovered, "ada"), "the handle shows under the pointer");
        assert!(!hairline(&hovered), "the hairline goes");
    }

    #[test]
    fn a_row_shows_its_actions_and_a_denied_one_cannot_be_used() {
        let row = person(
            vec![
                action("friend:10:message", None),
                action(
                    "friend:10:accept",
                    Some("Esta conexión no permite gestionar amistades"),
                ),
            ],
            vec![action("friend:10:block", None)],
            true,
        );
        let (_, frame) = frame(
            super::row(&row, &Look::DARK).size(Some(700.0), None),
            Size::new(800.0, 400.0),
        );
        let found = controls(&frame);
        assert!(found.contains(&"friend:10:message".to_owned()));
        assert!(!found.contains(&"friend:10:accept".to_owned()), "{found:?}");
        assert!(found.contains(&"friend:10:more".to_owned()));
        // The menu is open: its item is there.
        assert!(found.contains(&"friend:10:block".to_owned()), "{found:?}");
        let painted = texts(&frame);
        assert!(painted.contains(&"Ada".to_owned()) && painted.contains(&"En línea".to_owned()));
    }

    #[test]
    fn the_add_form_says_why_it_cannot_send_and_labels_the_simulation() {
        let denied = AddFriend {
            username: "zoe".into(),
            denied: Some("Esta conexión no permite enviar solicitudes de amistad".into()),
            ..AddFriend::default()
        };
        let (_, laid) = frame(add_friend(&denied, &Look::DARK), Size::new(700.0, 300.0));
        assert!(!controls(&laid).contains(&ids::SEND.to_owned()));
        assert!(texts(&laid)
            .iter()
            .any(|text| text.contains("no permite enviar solicitudes")));
        let simulated = AddFriend {
            username: "zoe".into(),
            simulated: true,
            note: Some(("Solicitud enviada a zoe".into(), false)),
            ..AddFriend::default()
        };
        let (_, frame) = frame(add_friend(&simulated, &Look::DARK), Size::new(700.0, 300.0));
        assert!(controls(&frame).contains(&ids::SEND.to_owned()));
        let painted = texts(&frame);
        assert!(painted
            .iter()
            .any(|text| text.starts_with("Simulación local")));
        assert!(painted.contains(&"Solicitud enviada a zoe".to_owned()));
    }

    #[test]
    fn ending_something_asks_first() {
        let (_, frame) = frame(
            confirm(
                Element::spacer(),
                "Bloquear a Ada",
                "No va a poder escribirte.",
                "Bloquear",
                &Look::DARK,
            ),
            Size::new(900.0, 600.0),
        );
        let found = controls(&frame);
        assert!(
            found.contains(&ids::CONFIRM.to_owned())
                && found.contains(&ids::CONFIRM_CANCEL.to_owned())
        );
        assert!(texts(&frame).contains(&"Bloquear".to_owned()));
    }
}
