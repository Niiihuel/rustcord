//! Discord's «Invita a tus amigos» modal, from plain props: the friends
//! that can be sent the link, each with its own «Invitar», the link to copy
//! and how long it lasts, and the settings that make a new one. Measured on
//! the owner's capture of the desktop client (see `docs/discord-reference.md`).

use limn::{
    Access, Button, ButtonSizing, ButtonVariant, Color, Component, ComponentPalette, DismissButton,
    DropdownOption, Element, FontFamily, Icon, PopoverPlacement, Role, ScrollArea, SearchField,
    Select, Size, Switch, TextSpan,
};
use rustcord_i18n::{t, tr};

use crate::{avatar_with_image, gap, line, strong_line, text, Look};

/// The actions the modal reports.
pub mod ids {
    pub const CLOSE: &str = "invite:close";
    pub const QUERY: &str = "invite:query";
    pub const LIST: &str = "invite:list";
    pub const COPY: &str = "invite:copy";
    pub const EDIT: &str = "invite:edit";
    /// Followed by a friend's ID: send them the link.
    pub const SEND: &str = "invite:send:";
    /// The settings' selects; an option is `{select}:{index}`.
    pub const AGE: &str = "invite:age";
    pub const USES: &str = "invite:uses";
    pub const SELECT_DISMISS: &str = "invite:select:dismiss";
    pub const TEMPORARY: &str = "invite:temporary";
    pub const BACK: &str = "invite:back";
    pub const GENERATE: &str = "invite:generate";
}

/// The card: 480 wide, 24 in, rounded 12, and at most the window less 74
/// above and below.
pub const WIDTH: f32 = 480.0;
const PAD: f32 = 24.0;
const RADIUS: f32 = 12.0;
const MARGIN: f32 = 74.0;
/// A friend's row: a 32 avatar, 48 apart.
const ROW: f32 = 48.0;
const AVATAR: f32 = 32.0;
/// The title and subtitle, the search and the room under it, over the list.
const HEADER: f32 = 24.0 + 24.0 + 8.0 + 20.0 + 24.0 + FIELD + 12.0;
/// The hairline, the sentence over the link, the link and the expiry under it.
const FOOTER: f32 = 1.0 + 24.0 + 20.0 + 8.0 + FIELD + 16.0 + 16.0 + 16.0;
const FIELD: f32 = 40.0;
/// The list's room under its last row.
const LIST_END: f32 = 8.0;
/// Black at 12 %: the search's and the link's fill.
const FIELD_FILL: Color = Color::rgba(0, 0, 0, 31);
/// `#9595a2` at 16 %: their edge.
const FIELD_EDGE: Color = Color::rgba(149, 149, 162, 41);
/// A row's «Invitar» and the link's «Copiar»: 32 high, 14 px medium.
const SMALL: ButtonSizing = ButtonSizing {
    height: 32.0,
    font_size: 14.0,
    icon_size: 16.0,
    horizontal_padding: 12.0,
    inner_spacing: 4.0,
};

/// Where an invitation to a friend is.
#[derive(Clone, Debug, PartialEq)]
pub enum Invited {
    /// It can be sent.
    Ready,
    /// Not now: the link is not there yet, or another is on its way.
    Waiting,
    Sending,
    Sent,
    /// It did not get there, and why; it can be sent again.
    Failed(String),
    /// It may have arrived, and why it is not known: sending it again
    /// could repeat it.
    Unsure(String),
    /// They are in the server already: found by a search, they are named
    /// so instead of invited, as Discord does.
    Member,
}

/// A friend the link can be sent to.
#[derive(Clone, Debug, PartialEq)]
pub struct Friend {
    pub user: u64,
    pub name: String,
    /// Their username, under the name.
    pub handle: String,
    pub image: Option<limn::RasterImage>,
    pub accent: Color,
    pub invited: Invited,
}

/// The settings view: how long the link lasts and how many times it works,
/// as indexes of `ages` and `uses`, and whether who enters with it stays.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub ages: Vec<String>,
    pub age: usize,
    pub uses: Vec<String>,
    pub use_limit: usize,
    pub temporary: bool,
    /// The select whose list is open: [`ids::AGE`] or [`ids::USES`].
    pub open: Option<&'static str>,
    /// A new link is being made.
    pub generating: bool,
}

/// What the modal shows.
#[derive(Clone, Debug, PartialEq)]
pub struct InviteView {
    pub server: String,
    /// Where whoever enters lands, and its glyph.
    pub channel: String,
    pub glyph: Icon,
    pub query: String,
    pub friends: Vec<Friend>,
    /// Said instead of the friends when none are shown.
    pub note: Option<String>,
    /// The link, once made; shown as given (dotted in streamer mode).
    pub link: Option<String>,
    /// Said in the field while there is no link: it is being made, or why not.
    pub pending: String,
    /// The link was just copied.
    pub copied: bool,
    /// How long it lasts, before «Editar el enlace de invitación».
    pub expiry: String,
    /// A simulated provider says its link leads nowhere.
    pub simulated: Option<String>,
    pub settings: Option<Settings>,
}

/// The modal over `root`, centered in a `window` this large.
pub fn modal(root: Element, view: &InviteView, window: Size, look: &Look) -> Element {
    let width = WIDTH.min((window.width - 32.0).max(280.0));
    let tall = (window.height - 2.0 * MARGIN).max(320.0);
    let (title, body) = match &view.settings {
        Some(settings) => (
            t("Ajustes del enlace de invitación al servidor").to_owned(),
            settings_body(settings, width - 2.0 * PAD, look),
        ),
        None => (
            tr!("Invita a tus amigos a {server}", server = view.server),
            invite_body(view, tall, look),
        ),
    };
    let close = DismissButton::new(format!("{}:x", ids::CLOSE), look.menu_palette())
        .target_size(24.0)
        .dispatch_as(ids::CLOSE)
        .render();
    let surface = Element::stack(vec![
        body,
        Element::column(vec![
            gap(0.0, 16.0),
            Element::row(vec![Element::spacer().flex(1.0), close, gap(16.0, 0.0)]),
        ]),
    ]);
    let card = Element::panel(surface)
        .size(Some(width), None)
        .radius(RADIUS)
        .background(Look::alpha(look.card, 255))
        .border(look.border, 1.0)
        .clip_children()
        .accessible(
            Access::new(Role::Dialog)
                .id(format!("{}:dialog", ids::CLOSE))
                .name(title),
        );
    // Escape and a click beside it close it, as Discord's; the search
    // takes the keys as it opens.
    let modal = Element::popover(root, card, ids::CLOSE, PopoverPlacement::Center)
        .popover_scrim(look.scrim())
        .popover_backdrop_dismiss(true);
    match view.settings {
        Some(_) => modal,
        None => modal.popover_initial_focus(ids::QUERY),
    }
}

/// The title, 20 px semibold, 24 from the top and clear of the close button.
fn title(title: &str, look: &Look) -> Element {
    Element::row(vec![
        text(title, 20.0, look.text)
            .font_family(FontFamily::SansSemibold)
            .flex(1.0),
        gap(32.0, 0.0),
    ])
}

fn invite_body(view: &InviteView, tall: f32, look: &Look) -> Element {
    // The channel's glyph leads its name, inline with the sentence.
    let subtitle = Element::formatted_text(vec![
        TextSpan::new(
            format!("{} ", t("Los destinatarios llegarán a")),
            look.muted,
            16.0,
            FontFamily::Sans,
        ),
        TextSpan::new(&view.channel, look.muted, 16.0, FontFamily::Sans).icon(view.glyph),
    ])
    .max_lines(1);
    let search = SearchField::new(
        ids::QUERY,
        &view.query,
        t("Buscar amigos"),
        ComponentPalette {
            raised: FIELD_FILL,
            border: FIELD_EDGE,
            ..look.menu_palette()
        },
    )
    .height(FIELD)
    // Discord's keeps saying what it finds while it has the focus.
    .persistent_placeholder(true)
    .render()
    .radius(8.0)
    .font_size(16.0)
    .hover_background(FIELD_FILL);
    let header = Element::column(vec![
        gap(0.0, PAD),
        title(
            &tr!("Invita a tus amigos a {server}", server = view.server),
            look,
        ),
        gap(0.0, 8.0),
        subtitle,
        gap(0.0, PAD),
        search,
        gap(0.0, 12.0),
    ]);
    let header = inset(header);

    let mut rows: Vec<Element> = view
        .friends
        .iter()
        .map(|friend| friend_row(friend, look))
        .collect();
    if let Some(note) = &view.note {
        rows.push(
            inset(Element::column(vec![
                gap(0.0, 14.0),
                text(note, 14.0, look.muted),
            ]))
            .size(None, Some(ROW)),
        );
    }
    let shown = rows.len();
    rows.push(gap(0.0, LIST_END));
    // As tall as its rows, up to what the window leaves the card.
    let natural = shown as f32 * ROW + LIST_END;
    let room = (tall - HEADER - FOOTER - 2.0).max(ROW);
    let list = ScrollArea::new(ids::LIST, Element::column(rows))
        .appearance(look.scrollbar(true))
        .render()
        .size(None, Some(natural.min(room)));

    Element::column(vec![header, list, footer(view, look)])
}

/// A friend: their picture, name and username, and «Invitar» at the end.
fn friend_row(friend: &Friend, look: &Look) -> Element {
    // Discord's labels: «Invitar», and once sent its es-419 «Invitaciones
    // enviadas»; a failure offers «Invitar» again.
    let enabled = matches!(friend.invited, Invited::Ready | Invited::Failed(_));
    let label = match friend.invited {
        Invited::Sent => t("Invitaciones enviadas"),
        _ => t("Invitar"),
    };
    let handle = match &friend.invited {
        Invited::Failed(reason) | Invited::Unsure(reason) => line(reason, 12.0, look.danger),
        _ => line(&friend.handle, 12.0, look.dim),
    };
    let button = match friend.invited {
        Invited::Member => text(t("Miembro"), 14.0, look.muted).font_family(FontFamily::SansMedium),
        _ => Button::new(
            format!("{}{}", ids::SEND, friend.user),
            label,
            crate::user_settings::veil(look),
        )
        .variant(ButtonVariant::Secondary)
        .sizing(SMALL)
        .loading(friend.invited == Invited::Sending)
        .enabled(enabled)
        .accessibility_label(tr!("Invitar a {name}", name = friend.name))
        .render()
        .min_size(60.0, 0.0),
    };
    inset(Element::row(vec![
        avatar_with_image(&friend.name, friend.accent, AVATAR, friend.image.clone()).centered_y(),
        gap(10.0, 0.0),
        Element::column(vec![strong_line(&friend.name, 16.0, look.text), handle])
            .gap(2.0)
            .flex(1.0)
            .centered_y(),
        gap(12.0, 0.0),
        button.centered_y(),
    ]))
    .size(None, Some(ROW))
}

/// Under a hairline: the link to copy and how long it lasts.
fn footer(view: &InviteView, look: &Look) -> Element {
    let (shown, ink) = match &view.link {
        Some(link) => (link.as_str(), look.text),
        None => (view.pending.as_str(), look.muted),
    };
    let (label, fill) = if view.copied {
        (t("Copiado"), look.online)
    } else {
        (t("Copiar"), look.accent)
    };
    let copy = Button::new(
        ids::COPY,
        label,
        ComponentPalette {
            raised: fill,
            ..look.accent_palette()
        },
    )
    .sizing(ButtonSizing {
        horizontal_padding: 16.0,
        ..SMALL
    })
    .enabled(view.link.is_some())
    .render()
    .min_size(82.0, 0.0);
    let field = Element::panel(Element::row(vec![
        gap(12.0, 0.0),
        line(shown, 16.0, ink).flex(1.0).centered_y(),
        gap(8.0, 0.0),
        copy.centered_y(),
        gap(4.0, 0.0),
    ]))
    .size(None, Some(FIELD))
    .radius(8.0)
    .background(FIELD_FILL)
    .border(FIELD_EDGE, 1.0);
    // The link to the settings follows the sentence, as Discord's.
    let expiry = Element::formatted_text(vec![
        TextSpan::new(
            format!("{} ", view.expiry),
            look.dim,
            12.0,
            FontFamily::Sans,
        ),
        TextSpan::new(
            t("Editar el enlace de invitación."),
            look.palette().link(),
            12.0,
            FontFamily::Sans,
        )
        .action(ids::EDIT),
    ]);
    let mut parts = vec![
        gap(0.0, PAD),
        line(
            t("O envía un enlace de invitación al servidor a un amigo"),
            16.0,
            look.text,
        )
        .size(None, Some(20.0)),
        gap(0.0, 8.0),
        field,
        gap(0.0, 16.0),
        expiry,
    ];
    if let Some(note) = &view.simulated {
        parts.push(gap(0.0, 4.0));
        parts.push(text(note, 12.0, look.dim));
    }
    parts.push(gap(0.0, 16.0));
    Element::column(vec![
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
        inset(Element::column(parts)),
    ])
}

/// How long the link lasts, how many times it works and whether who enters
/// with it stays, then «Cancelar» and «Generar un nuevo enlace».
fn settings_body(settings: &Settings, width: f32, look: &Look) -> Element {
    let select = |id: &'static str, labels: &[String], chosen: usize, name: &str| {
        let options = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                DropdownOption::new(format!("{id}:{index}"), label.clone())
                    .selected(index == chosen)
            })
            .collect();
        look.select(Select::new(
            id,
            labels.get(chosen).cloned().unwrap_or_default(),
            options,
            settings.open == Some(id),
            ids::SELECT_DISMISS,
            look.menu_palette(),
        ))
        .accessibility_label(name)
        .width(width)
        .menu_width(width)
        .render()
    };
    let field = |label: &str, control: Element| {
        Element::column(vec![
            strong_line(label, 16.0, look.text).size(None, Some(20.0)),
            gap(0.0, 8.0),
            control,
        ])
    };
    let temporary = Element::row(vec![
        Element::column(vec![
            strong_line(t("Otorgar membresía temporal"), 16.0, look.text),
            text(
                t("Los miembros temporales son expulsados automáticamente cuando se desconectan, a menos que se les haya asignado un rol"),
                12.0,
                look.muted,
            ),
        ])
        .gap(4.0)
        .flex(1.0),
        gap(16.0, 0.0),
        Switch::new(ids::TEMPORARY, settings.temporary, look.palette())
            .accessibility_label(t("Otorgar membresía temporal"))
            .render(),
    ]);
    let buttons = Element::row(vec![
        Element::spacer().flex(1.0),
        Button::new(ids::BACK, t("Cancelar"), look.menu_palette())
            .variant(ButtonVariant::Ghost)
            .sizing(ButtonSizing {
                height: 38.0,
                horizontal_padding: 16.0,
                ..SMALL
            })
            .render(),
        gap(8.0, 0.0),
        Button::new(
            ids::GENERATE,
            t("Generar un nuevo enlace"),
            look.accent_palette(),
        )
        .sizing(ButtonSizing {
            height: 38.0,
            horizontal_padding: 16.0,
            ..SMALL
        })
        .loading(settings.generating)
        .enabled(!settings.generating)
        .render(),
    ]);
    let body = Element::column(vec![
        gap(0.0, PAD),
        title(t("Ajustes del enlace de invitación al servidor"), look),
        gap(0.0, PAD),
        field(
            t("Caduca después de"),
            select(
                ids::AGE,
                &settings.ages,
                settings.age,
                t("Caduca después de"),
            ),
        ),
        gap(0.0, 16.0),
        field(
            t("Cantidad máxima de usos"),
            select(
                ids::USES,
                &settings.uses,
                settings.use_limit,
                t("Cantidad máxima de usos"),
            ),
        ),
        gap(0.0, PAD),
        temporary,
        gap(0.0, PAD),
        buttons,
        gap(0.0, PAD),
    ]);
    inset(body)
}

/// `content` 24 in from the card's sides.
fn inset(content: Element) -> Element {
    Element::row(vec![gap(PAD, 0.0), content.flex(1.0), gap(PAD, 0.0)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};

    const LOOK: Look = Look::ONYX;

    fn friend(user: u64, name: &str, invited: Invited) -> Friend {
        Friend {
            user,
            name: name.into(),
            handle: name.to_lowercase(),
            image: None,
            accent: Color::rgb(90, 120, 200),
            invited,
        }
    }

    fn view(friends: Vec<Friend>) -> InviteView {
        InviteView {
            server: "RUNE prueba".into(),
            channel: "general".into(),
            glyph: Icon::Hash,
            query: String::new(),
            friends,
            note: None,
            link: Some("https://discord.gg/kkWKkYTZr".into()),
            pending: String::new(),
            copied: false,
            expiry: "Tu enlace de invitación caduca en 30 días.".into(),
            simulated: None,
            settings: None,
        }
    }

    fn laid(view: &InviteView, window: Size) -> limn::Frame {
        frame(modal(Element::spacer(), view, window, &LOOK), window).1
    }

    fn region<'a>(frame: &'a limn::Frame, id: &str) -> &'a limn::HitRegion {
        frame
            .regions
            .iter()
            .find(|region| region.id == id)
            .unwrap_or_else(|| panic!("{id} is laid out"))
    }

    #[test]
    fn the_modal_has_discords_measures() {
        let shown = view(vec![
            friend(1, "Danonino", Invited::Ready),
            friend(2, "Juli", Invited::Ready),
        ]);
        let window = Size::new(1600.0, 950.0);
        let frame = laid(&shown, window);
        let search = region(&frame, ids::QUERY).rect;
        // 480 wide and centered: the search 24 in from both sides, 40 high.
        assert_eq!(search.width, WIDTH - 2.0 * PAD);
        assert_eq!(search.height, FIELD);
        assert!((search.x - (window.width - WIDTH) / 2.0 - PAD).abs() < 1.5);
        // Rows 48 apart, their «Invitar» 32 high at the content's end.
        let first = region(&frame, "invite:send:1").rect;
        let second = region(&frame, "invite:send:2").rect;
        assert_eq!(second.y - first.y, ROW);
        assert_eq!(first.height, 32.0);
        assert!((first.x + first.width - (search.x + search.width)).abs() < 0.5);
        // The list starts 12 under the search; the link 40 high, «Copiar»
        // 4 in from its end.
        assert!((first.y - 8.0 - (search.y + search.height + 12.0)).abs() < 0.5);
        let copy = region(&frame, ids::COPY).rect;
        assert_eq!(copy.height, 32.0);
        assert!((copy.x + copy.width + 4.0 - (search.x + search.width)).abs() < 0.5);
        let painted = texts(&frame);
        for said in [
            "Invita a tus amigos a RUNE prueba",
            "Los destinatarios llegarán a ",
            "general",
            "O envía un enlace de invitación al servidor a un amigo",
            "https://discord.gg/kkWKkYTZr",
        ] {
            assert!(
                painted.iter().any(|text| text == said),
                "{said}: {painted:?}"
            );
        }
        // The settings' link follows the sentence, and wraps with it.
        assert!(painted
            .iter()
            .any(|text| text.starts_with("Editar el enlace")));
        assert!(controls(&frame).contains(&ids::EDIT.to_owned()));
    }

    #[test]
    fn a_long_list_scrolls_inside_a_card_the_window_holds() {
        let many = (0..40)
            .map(|user| friend(user, &format!("Amigo {user}"), Invited::Ready))
            .collect();
        let window = Size::new(1200.0, 700.0);
        let frame = laid(&view(many), window);
        let list = region(&frame, ids::LIST).rect;
        let copy = region(&frame, ids::COPY).rect;
        // The footer stays in view, under the list, inside the window's margin.
        assert!(copy.y > list.y + list.height);
        assert!(copy.y + copy.height < window.height - MARGIN);
    }

    #[test]
    fn each_friend_says_where_their_invitation_is() {
        let shown = view(vec![
            friend(1, "Ana", Invited::Ready),
            friend(2, "Bruno", Invited::Sent),
            friend(3, "Carla", Invited::Waiting),
            friend(4, "Dani", Invited::Failed("Sin conexión".into())),
            friend(5, "Eli", Invited::Member),
        ]);
        let frame = laid(&shown, Size::new(1400.0, 900.0));
        let usable = controls(&frame);
        assert!(usable.contains(&"invite:send:1".to_owned()));
        assert!(!usable.contains(&"invite:send:2".to_owned()), "sent once");
        assert!(!usable.contains(&"invite:send:3".to_owned()), "waits");
        assert!(usable.contains(&"invite:send:4".to_owned()), "tried again");
        assert!(!usable.contains(&"invite:send:5".to_owned()), "already in");
        let painted = texts(&frame);
        assert!(painted.iter().any(|text| text == "Invitaciones enviadas"));
        assert!(painted.iter().any(|text| text == "Sin conexión"));
        assert!(painted.iter().any(|text| text == "Miembro"));
    }

    #[test]
    fn without_a_link_nothing_is_copied_and_the_field_says_why() {
        let mut shown = view(Vec::new());
        shown.link = None;
        shown.pending = "Creando el enlace…".into();
        shown.note = Some("No hay amigos a quienes invitar.".into());
        let frame = laid(&shown, Size::new(1400.0, 900.0));
        assert!(!controls(&frame).contains(&ids::COPY.to_owned()));
        let painted = texts(&frame);
        assert!(painted.iter().any(|text| text == "Creando el enlace…"));
        assert!(painted
            .iter()
            .any(|text| text == "No hay amigos a quienes invitar."));
    }

    #[test]
    fn the_settings_choose_how_the_next_link_is_made() {
        let mut shown = view(Vec::new());
        shown.settings = Some(Settings {
            ages: vec!["7 días".into(), "30 días".into()],
            age: 1,
            uses: vec!["Sin límite".into(), "1 uso".into()],
            use_limit: 0,
            temporary: false,
            open: None,
            generating: false,
        });
        let frame = laid(&shown, Size::new(1400.0, 900.0));
        let usable = controls(&frame);
        for id in [
            ids::AGE,
            ids::USES,
            ids::TEMPORARY,
            ids::BACK,
            ids::GENERATE,
        ] {
            assert!(usable.contains(&id.to_owned()), "{id}: {usable:?}");
        }
        assert!(!usable.contains(&ids::QUERY.to_owned()));
        let painted = texts(&frame);
        assert!(painted.iter().any(|text| text == "30 días"));
    }
}
