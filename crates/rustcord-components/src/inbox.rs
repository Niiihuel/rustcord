//! The inbox, as Discord's popout under its title bar's tray (here under the
//! header's): a header with the tray, «Bandeja de entrada», «Marcar todo como
//! leído» and the friend requests; four tabs underlined in the accent; and a
//! list, measured as it scrolls, of cards: a channel's (its server's mark,
//! its name and where it is, its tools) over what it holds. Measures in
//! `docs/discord-reference.md` («Inbox»).

use limn::{
    AlertDialog, Color, Component, DropdownMenu, Element, FontFamily, Icon, PopoverPlacement,
    ScrollArea, Size, Tooltip,
};
use rustcord_i18n::t;

use crate::channel_menu::{dropdown_entries, Entry, BELL_WIDTH};
use crate::{centered, gap, icon, line, Look};

/// The actions the inbox reports. Those of a card name its channel, and a
/// message's its message too.
pub mod ids {
    /// The header's tray, which opens and closes it.
    pub const OPEN: &str = "inbox:open";
    /// A click outside it or Escape.
    pub const CLOSE: &str = "inbox:close";
    /// The list of cards.
    pub const LIST: &str = "inbox:list";
    /// «Marcar todo como leído», and its question.
    pub const READ_ALL: &str = "inbox:read-all";
    pub const READ_ALL_CONFIRM: &str = "inbox:read-all:confirm";
    pub const READ_ALL_CANCEL: &str = "inbox:read-all:cancel";
    /// The friend requests waiting.
    pub const FRIENDS: &str = "inbox:friends";
    /// A card's bell menu dismissed.
    pub const BELL_DISMISS: &str = "inbox:bell-dismiss";

    /// The card's channel, at its first unread message.
    pub fn go(channel: u64) -> String {
        format!("inbox:go:{channel}")
    }

    /// Mark the card's channel read.
    pub fn read(channel: u64) -> String {
        format!("inbox:read:{channel}")
    }

    /// Fold the card, or unfold it.
    pub fn fold(channel: u64) -> String {
        format!("inbox:fold:{channel}")
    }

    /// The card's bell and its menu.
    pub fn bell(channel: u64) -> String {
        format!("inbox:bell:{channel}")
    }

    /// `message`, in its conversation.
    pub fn jump(channel: u64, message: u64) -> String {
        format!("inbox:jump:{channel}:{message}")
    }
}

/// The inbox's tabs, in Discord's order.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Tab {
    #[default]
    Unread,
    Mentions,
    Bookmarks,
    Reminders,
}

impl Tab {
    pub const ALL: [Self; 4] = [
        Self::Unread,
        Self::Mentions,
        Self::Bookmarks,
        Self::Reminders,
    ];

    /// The action that chooses it.
    pub fn id(self) -> &'static str {
        match self {
            Self::Unread => "inbox:unread",
            Self::Mentions => "inbox:mentions",
            Self::Bookmarks => "inbox:bookmarks",
            Self::Reminders => "inbox:reminders",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.id() == id)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Unread => t("No leídos"),
            Self::Mentions => t("Menciones"),
            Self::Bookmarks => t("Marcadores"),
            Self::Reminders => t("Recordatorios"),
        }
    }
}

/// The height of the header over the list: 20 of padding, the title's 34,
/// 16, and the tabs' 36 with their line.
pub const HEADER: f32 = 106.0;
/// The tabs' line and a card's edge: Discord's `--border-subtle`.
const SUBTLE: Color = Color::rgba(149, 149, 162, 51);
/// A secondary button's fill, its edge and its fill under the pointer.
const FILL: Color = Color::rgba(151, 151, 159, 31);
const EDGE: Color = Color::rgba(149, 149, 162, 41);
const FILL_HOVER: Color = Color::rgba(151, 151, 159, 51);

/// How big the popout is in a window of `window`: 35 % of its width, from
/// 480 to 600, and at most 70 % of its height (80 % below 800, 85 % below
/// 700), as Discord's.
pub fn size(window: Size) -> Size {
    let share = if window.height <= 700.0 {
        0.85
    } else if window.height <= 800.0 {
        0.80
    } else {
        0.70
    };
    Size::new(
        (window.width * 0.35).clamp(480.0, 600.0),
        window.height * share,
    )
}

/// Where a card's message is laid out: the popout's `width` less the list's
/// 16 at each side, the 8 Discord's scrollbar keeps, the card's edges and
/// 16 after the message.
pub fn message_width(width: f32) -> f32 {
    width - 40.0 - 2.0 - 16.0
}

/// What the header shows besides its title.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Header {
    pub tab: Tab,
    /// «Marcar todo como leído»: the unread tab with something unread.
    pub read_all: bool,
    /// The friend requests waiting; `None` without a list of friends.
    pub friends: Option<usize>,
    /// Reading is kept only here: the provider is not told.
    pub local: bool,
}

/// The mark at a card's start: a server's (rounded), a person's or a
/// group's (round).
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub name: String,
    /// A server's initials, while it has no picture.
    pub initials: String,
    pub image: Option<limn::RasterImage>,
    pub accent: Color,
    pub person: bool,
}

/// A card's channel, as its header shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Head {
    pub channel: u64,
    pub face: Face,
    /// Its kind's glyph; a direct conversation has none.
    pub glyph: Option<Icon>,
    pub name: String,
    pub mentions: u32,
    /// «Servidor › Categoría», or how many are in a group.
    pub path: String,
}

/// A card's bell: its notification menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Bell {
    pub open: bool,
    pub muted: bool,
    pub entries: Vec<Entry>,
}

/// What a card offers at its end.
#[derive(Clone, Debug, PartialEq)]
pub enum Tools {
    /// A channel with unread messages: its bell (a server's channel), mark
    /// it read (`local`: only here), fold it.
    Unread {
        bell: Option<Bell>,
        folded: bool,
        local: bool,
    },
    /// A message the account was mentioned in: go to it.
    Mention { message: u64 },
    /// A message marked to read later: go to it, mark its channel read.
    Reminder { message: u64, local: bool },
}

/// What a card holds under its header.
pub enum Body {
    /// Its unread messages, as the conversation shows them, and whether
    /// there are more than these.
    Messages { rows: Vec<Element>, more: bool },
    /// Its messages are on their way.
    Loading,
    /// Folded: nothing.
    Folded,
    /// One message.
    Single(Box<Element>),
    /// Why there is nothing to show.
    Note(String),
}

/// The inbox's header: the tray, its title, its buttons and the tabs.
pub fn header(view: &Header, look: &Look) -> Element {
    let mut title = vec![
        icon(Icon::Custom(crate::glyphs::INBOX), look.text)
            .size(Some(20.0), Some(20.0))
            .large_icon(20.0)
            .centered_y(),
        gap(8.0, 0.0),
        Element::truncated_text(t("Bandeja de entrada"))
            .font_family(FontFamily::SansSemibold)
            .font_size(20.0)
            .foreground(look.text)
            .accessible(limn::Access::new(limn::Role::Heading).name(t("Bandeja de entrada")))
            .flex(1.0)
            .centered_y(),
    ];
    if view.read_all {
        title.push(gap(8.0, 0.0));
        title.push(
            tool(
                ids::READ_ALL.into(),
                Icon::Custom(crate::glyphs::CHECK_DOUBLE),
                if view.local {
                    t("Marcar todo como leído (solo en Rustcord)")
                } else {
                    t("Marcar todo como leído")
                },
                look,
            )
            .centered_y(),
        );
    }
    if let Some(count) = view.friends {
        title.push(gap(8.0, 0.0));
        title.push(friends(count, look).centered_y());
    }
    let tabs = Tab::ALL
        .into_iter()
        .enumerate()
        .flat_map(|(index, tab)| {
            let mut parts = Vec::new();
            if index > 0 {
                parts.push(gap(10.0, 0.0));
            }
            parts.push(tab_item(tab, tab == view.tab, look).flex(1.0));
            parts
        })
        .collect();
    Element::column(vec![
        gap(0.0, 20.0),
        Element::row(vec![
            gap(20.0, 0.0),
            Element::row(title).flex(1.0),
            gap(20.0, 0.0),
        ])
        .size(None, Some(34.0)),
        gap(0.0, 16.0),
        Element::row(vec![
            gap(4.0, 0.0),
            Element::column(vec![
                Element::row(tabs).size(None, Some(35.0)),
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(SUBTLE),
            ])
            .flex(1.0),
            gap(4.0, 0.0),
        ]),
    ])
    .size(None, Some(HEADER))
}

/// A tab: its label 14 semibold, 16 over its line, the chosen one in the
/// accent and underlined.
fn tab_item(tab: Tab, chosen: bool, look: &Look) -> Element {
    let label = Element::truncated_text(tab.label())
        .font_family(FontFamily::SansSemibold)
        .font_size(14.0)
        .foreground(if chosen { look.accent } else { look.muted });
    let underline = Element::panel(Element::spacer())
        .size(None, Some(2.0))
        .radius(1.0)
        .background(if chosen {
            look.accent
        } else {
            Color::TRANSPARENT
        });
    let mut item = Element::clickable(
        tab.id(),
        Element::column(vec![
            Element::row(vec![
                Element::spacer().flex(1.0),
                label.shrink_to_fit(),
                Element::spacer().flex(1.0),
            ])
            .size(None, Some(20.0)),
            Element::spacer().flex(1.0),
            underline,
        ]),
    )
    .size(None, Some(35.0))
    .accessible(
        limn::Access::new(limn::Role::Tab)
            .name(tab.label())
            .selected(chosen),
    );
    if !chosen {
        item = item.hover_foreground(look.text);
    }
    item
}

/// «Ver solicitudes de amistad»: a person waving and how many wait.
fn friends(count: usize, look: &Look) -> Element {
    let said = t("Ver solicitudes de amistad");
    let mut parts = vec![
        gap(8.0, 0.0),
        icon(Icon::Custom(crate::glyphs::FRIENDS), look.text)
            .size(Some(16.0), Some(16.0))
            .large_icon(16.0)
            .centered_y(),
    ];
    if count > 0 {
        parts.push(gap(8.0, 0.0));
        parts.push(
            Element::panel(centered(
                Element::text(count.min(99).to_string())
                    .font_family(FontFamily::SansSemibold)
                    .font_size(12.0)
                    .foreground(look.text),
            ))
            .size(Some(16.0), Some(16.0))
            .radius(8.0)
            .background(Color::rgba(151, 151, 159, 61))
            .centered_y(),
        );
    }
    parts.push(gap(8.0, 0.0));
    let button = Element::clickable(ids::FRIENDS, Element::row(parts))
        .size(None, Some(34.0))
        .radius(8.0)
        .background(FILL)
        .border(EDGE, 1.0)
        .hover_background(FILL_HOVER)
        .accessibility_label(if count > 0 {
            format!("{said}, {count}")
        } else {
            said.to_owned()
        });
    Tooltip::new(button, ids::FRIENDS, said, look.menu_palette()).render()
}

/// A secondary button 32 square with a 16 px glyph, as the card's and the
/// header's, with its hint.
fn tool(id: String, glyph: Icon, hint: &str, look: &Look) -> Element {
    let button = Element::clickable(
        id.clone(),
        centered(
            icon(glyph, look.text)
                .size(Some(16.0), Some(16.0))
                .large_icon(16.0),
        ),
    )
    .size(Some(32.0), Some(32.0))
    .radius(8.0)
    .background(FILL)
    .border(EDGE, 1.0)
    .hover_background(FILL_HOVER)
    .accessibility_label(hint);
    Tooltip::new(button, id, hint, look.menu_palette()).render()
}

/// The popout: `header` over `content`, `size` at most. A `list` fills it;
/// anything else takes only its own height.
pub fn popout(header: Element, content: Element, list: bool, size: Size, look: &Look) -> Element {
    let column = if list {
        Element::column(vec![header, content.flex(1.0)])
    } else {
        Element::column(vec![header, content])
    };
    Element::panel(column)
        .size(Some(size.width), list.then_some(size.height))
        .max_height(size.height)
        .radius(8.0)
        .background(look.card)
        .border(Color::rgba(255, 255, 255, 20), 1.0)
        .shadow(
            Color::rgba(0, 0, 0, 61),
            limn::Point::new(0.0, 12.0),
            24.0,
            0.0,
        )
        .clip_children()
        .accessible(limn::Access::new(limn::Role::Dialog).name(t("Bandeja de entrada")))
}

/// The header's tray, lit while the inbox is open, with its `popout` under
/// it, the ends lined up, 8 below.
pub fn button(popout: Option<Element>, look: &Look) -> Element {
    let open = popout.is_some();
    let button = limn::IconButton::new(
        ids::OPEN,
        Icon::Custom(crate::glyphs::INBOX),
        look.palette(),
    )
    .variant(limn::ButtonVariant::Ghost)
    .sizing(crate::look::HEADER_TOOL)
    .tooltip(t("Bandeja de entrada"))
    .tooltip_placement(limn::TooltipPlacement::Below)
    .ink(if open { look.text } else { look.dim })
    .hover_ink(look.text)
    .active(open)
    .render();
    match popout {
        Some(popout) => Element::popover(button, popout, ids::CLOSE, PopoverPlacement::BottomEnd)
            .popover_gap(8.0),
        None => button,
    }
}

/// The cards, `count` of them, measured as the list scrolls: 16 from the
/// list's sides (24 at its end, as Discord's scrollbar keeps 8) and 16
/// apart. `key` names each for its measure.
pub fn list(
    count: usize,
    key: impl Fn(usize) -> u64 + Send + Sync + 'static,
    row: impl Fn(usize) -> Element + Send + Sync + 'static,
    look: &Look,
) -> Element {
    ScrollArea::measured_list(ids::LIST, count, 320.0, key, move |index| {
        Element::column(vec![
            gap(0.0, if index == 0 { 16.0 } else { 0.0 }),
            Element::row(vec![gap(16.0, 0.0), row(index).flex(1.0), gap(24.0, 0.0)]),
            gap(0.0, 16.0),
        ])
    })
    .appearance(look.scrollbar(true))
    .render()
}

/// A card: `head` with its `tools` over `body`, on Discord's raised
/// surface, rounded 12 and outlined.
pub fn card(head: &Head, tools: Tools, body: Body, look: &Look) -> Element {
    let channel = head.channel;
    let (click, mut end, fold) = match tools {
        Tools::Unread {
            bell,
            folded,
            local,
        } => {
            let mut end = Vec::new();
            if let Some(bell) = bell {
                end.push(bell_menu(channel, bell, look));
            }
            end.push(tool(
                ids::read(channel),
                Icon::Check,
                read_hint(local),
                look,
            ));
            (ids::go(channel), end, Some(folded))
        }
        Tools::Mention { message } => (
            ids::jump(channel, message),
            vec![tool(
                ids::jump(channel, message),
                Icon::Custom(crate::glyphs::JUMP),
                t("Ir al mensaje"),
                look,
            )],
            None,
        ),
        Tools::Reminder { message, local } => (
            ids::jump(channel, message),
            vec![
                tool(
                    ids::jump(channel, message),
                    Icon::Custom(crate::glyphs::JUMP),
                    t("Ir al mensaje"),
                    look,
                ),
                tool(ids::read(channel), Icon::Check, read_hint(local), look),
            ],
            None,
        ),
    };
    let mut row = vec![Element::clickable(click, who(head, look))
        .flex(1.0)
        .accessibility_label(head.name.clone())];
    for tool in end.drain(..) {
        row.push(gap(8.0, 0.0));
        row.push(tool.centered_y());
    }
    if let Some(folded) = fold {
        let said = if folded { t("Ampliar") } else { t("Minimizar") };
        row.push(gap(8.0, 0.0));
        row.push(
            Element::clickable(
                ids::fold(channel),
                centered(
                    icon(
                        if folded {
                            Icon::ChevronRight
                        } else {
                            Icon::ChevronDown
                        },
                        look.dim,
                    )
                    .size(Some(16.0), Some(16.0))
                    .large_icon(16.0),
                ),
            )
            .size(Some(24.0), Some(24.0))
            .hover_foreground(look.text)
            .accessibility_label(said)
            .centered_y(),
        );
    }
    let top = Element::panel(Element::row(row))
        .padding(12.0)
        .size(None, Some(60.0));
    let mut parts = vec![top];
    match body {
        Body::Folded => {}
        body => {
            parts.push(
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(SUBTLE),
            );
            parts.push(match body {
                Body::Messages { rows, more } => {
                    let mut column = vec![gap(0.0, 8.0)];
                    column.extend(rows);
                    if more {
                        column.push(gap(0.0, 4.0));
                        column.push(
                            Element::row(vec![
                                gap(80.0, 0.0),
                                Element::clickable(
                                    ids::go(channel),
                                    Element::text(format!(
                                        "{} \u{203A}",
                                        t("Ver todos los mensajes sin leer")
                                    ))
                                    .font_family(FontFamily::SansMedium)
                                    .font_size(14.0)
                                    .foreground(look.accent),
                                )
                                .hover_foreground(look.text),
                            ])
                            .size(None, Some(20.0)),
                        );
                    }
                    column.push(gap(0.0, 12.0));
                    Element::column(column)
                }
                Body::Loading => centered(
                    limn::Spinner::new(format!("inbox:loading:{channel}"), look.muted).render(),
                )
                .size(None, Some(64.0))
                .accessible(
                    limn::Access::new(limn::Role::ProgressIndicator).name(t("Cargando mensajes")),
                ),
                Body::Single(message) => {
                    Element::column(vec![gap(0.0, 12.0), *message, gap(0.0, 12.0)])
                }
                Body::Note(note) => Element::panel(
                    Element::text(note)
                        .font_family(FontFamily::Sans)
                        .font_size(14.0)
                        .foreground(look.muted),
                )
                .padding(12.0),
                Body::Folded => unreachable!("handled above"),
            });
        }
    }
    Element::panel(Element::column(parts))
        .radius(12.0)
        .background(look.raised)
        .border(SUBTLE, 1.0)
        .clip_children()
        .accessible(limn::Access::new(limn::Role::ListItem).name(head_name(head)))
}

fn read_hint(local: bool) -> &'static str {
    if local {
        t("Marcar como leído (solo en Rustcord)")
    } else {
        t("Marcar como leído")
    }
}

fn head_name(head: &Head) -> String {
    if head.path.is_empty() {
        head.name.clone()
    } else {
        format!("{}, {}", head.name, head.path)
    }
}

/// The left of a card's header: its mark, a chevron, the channel's glyph,
/// name and mentions over where it is.
fn who(head: &Head, look: &Look) -> Element {
    let face = &head.face;
    let mark = if face.person {
        crate::avatar_with_image(&face.name, face.accent, 32.0, face.image.clone())
    } else {
        let initials = centered(
            Element::text(face.initials.clone())
                .font_family(FontFamily::SansMedium)
                .font_size(12.0)
                .foreground(look.text),
        );
        let mut layers = vec![Element::panel(initials)
            .size(Some(32.0), Some(32.0))
            .radius(8.0)
            .background(look.selected)];
        if let Some(image) = &face.image {
            layers.push(
                Element::raster_image(image.clone().cover())
                    .size(Some(32.0), Some(32.0))
                    .radius(8.0),
            );
        }
        Element::stack(layers).size(Some(32.0), Some(32.0))
    };
    let mut name = Vec::new();
    if let Some(glyph) = head.glyph {
        name.push(
            icon(glyph, look.dim)
                .size(Some(12.0), Some(12.0))
                .large_icon(12.0)
                .centered_y(),
        );
        name.push(gap(4.0, 0.0));
    }
    name.push(
        Element::truncated_text(head.name.clone())
            .font_family(FontFamily::SansSemibold)
            .font_size(16.0)
            .foreground(look.text)
            // On the centering wrapper, which is what the row sizes.
            .centered_y()
            .shrink_to_fit(),
    );
    if head.mentions > 0 {
        name.push(gap(8.0, 0.0));
        name.push(crate::sidebar::count(head.mentions, look).centered_y());
    }
    let mut lines = vec![Element::row(name).size(None, Some(20.0))];
    if !head.path.is_empty() {
        lines.push(line(head.path.clone(), 12.0, look.text).size(None, Some(16.0)));
    }
    Element::row(vec![
        mark.centered_y(),
        gap(8.0, 0.0),
        icon(Icon::ChevronRight, look.dim)
            .size(Some(12.0), Some(12.0))
            .large_icon(12.0)
            .centered_y(),
        gap(8.0, 0.0),
        Element::column(lines).flex(1.0).centered_y(),
    ])
}

/// A card's bell, its notification menu under it while open.
fn bell_menu(channel: u64, bell: Bell, look: &Look) -> Element {
    let button = tool(
        ids::bell(channel),
        if bell.muted {
            Icon::BellSlash
        } else {
            Icon::Bell
        },
        t("Ajustes de notificaciones"),
        look,
    );
    DropdownMenu::new(
        button,
        dropdown_entries(bell.entries, look),
        bell.open,
        ids::BELL_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::MENU_METRICS)
    .width(BELL_WIDTH)
    .placement(PopoverPlacement::BottomEnd)
    .render()
}

/// A message in a card, with «Ir» at its top end while the pointer is over
/// it: 24 high, 8 on either side of the word, 8 in from the end and 8 into
/// Discord's message, which starts 2 over the text the row keeps `above`.
pub fn jumpable(row: Element, above: f32, channel: u64, message: u64, look: &Look) -> Element {
    let jump = Element::clickable(
        ids::jump(channel, message),
        Element::row(vec![
            gap(8.0, 0.0),
            Element::text(t("Ir"))
                .font_family(FontFamily::SansMedium)
                .font_size(12.0)
                .foreground(look.text)
                .centered_y(),
            gap(8.0, 0.0),
        ]),
    )
    .size(None, Some(24.0))
    .radius(4.0)
    .background(look.card)
    .border(SUBTLE, 1.0)
    .hover_background(look.raised)
    .accessibility_label(t("Ir al mensaje"))
    .reveal_on_hover(format!("inbox:msg:{message}:jump"));
    // In a row, so the stack keeps the message's width and not the card's.
    Element::row(vec![Element::stack(vec![
        row,
        Element::column(vec![
            gap(0.0, above + 6.0),
            Element::row(vec![Element::spacer().flex(1.0), jump, gap(8.0, 0.0)]),
            Element::spacer().flex(1.0),
        ]),
    ])
    .hover_region()])
}

/// A day's divider in a card, as Discord's: a hairline 16 in from the
/// card's sides and the day in a pill over its middle, 12 under what is
/// above it.
pub fn day(label: String, look: &Look) -> Element {
    let pill = Element::row(vec![
        gap(4.0, 0.0),
        Element::text(label)
            .font_family(FontFamily::SansMedium)
            .font_size(12.0)
            .foreground(look.muted)
            .centered_y(),
        gap(4.0, 0.0),
    ])
    .size(None, Some(16.0))
    .radius(8.0)
    .background(look.canvas);
    Element::column(vec![
        gap(0.0, 12.0),
        Element::row(vec![
            gap(16.0, 0.0),
            Element::stack(vec![
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(SUBTLE)
                    .centered_y(),
                pill.centered_x(),
            ])
            .flex(1.0),
            gap(16.0, 0.0),
        ])
        .size(None, Some(16.0)),
    ])
}

/// What a tab with nothing in it says: its glyph in a circle, a title and
/// a line under it.
pub struct EmptyState {
    pub glyph: Icon,
    pub title: String,
    pub tip: String,
}

/// A tab with nothing in it, at least 360 high, as Discord's.
pub fn empty(state: &EmptyState, look: &Look) -> Element {
    let circle = Element::panel(centered(
        icon(state.glyph, look.muted)
            .size(Some(36.0), Some(36.0))
            .large_icon(36.0),
    ))
    .size(Some(80.0), Some(80.0))
    .radius(40.0)
    .background(look.canvas);
    // limn centers plain text only: Discord's bold action goes in «».
    let tip = Element::text(state.tip.clone())
        .font_family(FontFamily::SansMedium)
        .font_size(14.0)
        .foreground(look.muted)
        .center_content();
    Element::panel(
        Element::column(vec![
            circle.centered_x(),
            gap(0.0, 16.0),
            Element::text(state.title.clone())
                .font_family(FontFamily::SansSemibold)
                .font_size(24.0)
                .foreground(look.text)
                .center_content(),
            gap(0.0, 8.0),
            tip,
        ])
        .centered_y(),
    )
    .padding(16.0)
    .min_size(0.0, 360.0)
}

/// A muted line closing a list: what it leaves out.
pub fn note(said: &str, look: &Look) -> Element {
    Element::text(said)
        .font_family(FontFamily::Sans)
        .font_size(12.0)
        .foreground(look.muted)
        .center_content()
}

/// «¿Marcar todo como leído?» over `root`, as Discord asks before it.
pub fn confirm(root: Element, look: &Look) -> Element {
    AlertDialog::new(
        root,
        true,
        t("¿Marcar todo como leído?"),
        t("Esto marcará todos tus canales no silenciados como leídos. ¿Realmente quieres continuar?"),
        ids::READ_ALL_CONFIRM,
        ids::READ_ALL_CANCEL,
        look.menu_palette(),
    )
    .width(crate::friends::CONFIRM_WIDTH)
    .confirm_label(t("Marcar como leído"))
    .cancel_label(t("Cancelar"))
    // Discord's brand button: the accent under white text.
    .destructive(look.accent)
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};

    fn head(channel: u64) -> Head {
        Head {
            channel,
            face: Face {
                name: "Taller de Rust".into(),
                initials: "TdR".into(),
                image: None,
                accent: Color::rgb(80, 90, 200),
                person: false,
            },
            glyph: Some(Icon::Hash),
            name: "general".into(),
            mentions: 2,
            path: "Taller de Rust \u{203A} Charla".into(),
        }
    }

    fn region(frame: &limn::Frame, id: &str) -> limn::Rect {
        frame
            .regions
            .iter()
            .find(|region| region.id == id)
            .unwrap_or_else(|| panic!("{id} is laid out"))
            .rect
    }

    #[test]
    fn the_popout_is_a_third_of_the_window_within_discords_bounds() {
        assert_eq!(
            size(Size::new(1440.0, 813.0)),
            Size::new(504.0, 813.0 * 0.70)
        );
        assert_eq!(size(Size::new(1000.0, 760.0)).width, 480.0);
        assert_eq!(size(Size::new(1000.0, 760.0)).height, 760.0 * 0.80);
        assert_eq!(
            size(Size::new(2400.0, 680.0)),
            Size::new(600.0, 680.0 * 0.85)
        );
    }

    #[test]
    fn the_header_has_its_title_buttons_and_four_tabs_underlining_the_chosen() {
        let look = Look::ONYX;
        let view = Header {
            tab: Tab::Mentions,
            read_all: true,
            friends: Some(3),
            local: false,
        };
        let (_, laid) = frame(header(&view, &look), Size::new(504.0, HEADER));
        let painted = texts(&laid);
        for said in [
            "Bandeja de entrada",
            "No leídos",
            "Menciones",
            "Marcadores",
            "Recordatorios",
            "3",
        ] {
            assert!(
                painted.iter().any(|text| text == said),
                "{said}: {painted:?}"
            );
        }
        let shown = controls(&laid);
        for id in [
            ids::READ_ALL,
            ids::FRIENDS,
            "inbox:unread",
            "inbox:mentions",
            "inbox:bookmarks",
            "inbox:reminders",
        ] {
            assert!(shown.contains(&id.to_owned()), "{id}: {shown:?}");
        }
        // Discord's places: the buttons 20 from the end, 8 apart, 32 and
        // 34 high; the tabs 4 in from the sides, as wide each, 10 apart.
        let friends = region(&laid, ids::FRIENDS);
        let read = region(&laid, ids::READ_ALL);
        assert!(
            (friends.x + friends.width - 484.0).abs() < 0.5,
            "{friends:?}"
        );
        assert!(
            (read.x + read.width + 8.0 - friends.x).abs() < 0.5,
            "{read:?}"
        );
        assert_eq!(
            (read.width, read.height, friends.height),
            (32.0, 32.0, 34.0)
        );
        let unread = region(&laid, "inbox:unread");
        let reminders = region(&laid, "inbox:reminders");
        assert_eq!(unread.x, 4.0);
        assert!((reminders.x + reminders.width - 500.0).abs() < 0.5);
        assert!((unread.width - (496.0 - 30.0) / 4.0).abs() < 0.5);
        assert_eq!((unread.y, unread.height), (70.0, 35.0));
        // The chosen tab's line is the accent's, 2 high, under it.
        let mentions = region(&laid, "inbox:mentions");
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::FillRect { rect, color, .. }
                if *color == look.accent && rect.height == 2.0 && rect.x == mentions.x
        )));
        // Away from the unread tab, and without friends, they go.
        let quiet = Header {
            tab: Tab::Bookmarks,
            read_all: false,
            friends: None,
            local: true,
        };
        let (_, laid) = frame(header(&quiet, &look), Size::new(504.0, HEADER));
        let shown = controls(&laid);
        assert!(!shown.contains(&ids::READ_ALL.to_owned()));
        assert!(!shown.contains(&ids::FRIENDS.to_owned()));
    }

    #[test]
    fn an_unread_card_lays_out_its_mark_name_place_and_tools_as_discords() {
        let look = Look::ONYX;
        let body = Body::Messages {
            rows: vec![Element::text("hola").size(None, Some(44.0))],
            more: true,
        };
        let card = card(
            &head(7),
            Tools::Unread {
                bell: Some(Bell {
                    open: false,
                    muted: false,
                    entries: Vec::new(),
                }),
                folded: false,
                local: false,
            },
            body,
            &look,
        );
        let (_, laid) = frame(card, Size::new(464.0, 400.0));
        let painted = texts(&laid);
        for said in [
            "general",
            "2",
            "Taller de Rust \u{203A} Charla",
            "TdR",
            "hola",
            "Ver todos los mensajes sin leer \u{203A}",
        ] {
            assert!(
                painted.iter().any(|text| text == said),
                "{said}: {painted:?}"
            );
        }
        // The bell, the check and the fold at the end, 8 apart; the mark
        // 12 in, on a header 60 high and its line.
        let bell = region(&laid, "inbox:bell:7");
        let read = region(&laid, "inbox:read:7");
        let fold = region(&laid, "inbox:fold:7");
        assert_eq!((bell.width, read.width, fold.width), (32.0, 32.0, 24.0));
        assert!((read.x - (bell.x + 40.0)).abs() < 0.5);
        assert!((fold.x - (read.x + 40.0)).abs() < 0.5);
        assert!((fold.x + fold.width - 452.0).abs() < 0.5, "{fold:?}");
        assert_eq!(region(&laid, "inbox:go:7").x, 12.0);
        assert!((bell.y - 14.0).abs() < 0.5, "{bell:?}");
        // Folded, only the header stays, with its chevron turned.
        let folded = super::card(
            &head(7),
            Tools::Unread {
                bell: None,
                folded: true,
                local: true,
            },
            Body::Folded,
            &look,
        );
        let (_, laid) = frame(folded, Size::new(464.0, 400.0));
        assert!(!texts(&laid).iter().any(|text| text.contains("Ver todos")));
        assert!(!controls(&laid).contains(&"inbox:bell:7".to_owned()));
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::Icon {
                icon: Icon::ChevronRight,
                rect,
                ..
            } if rect.width == 16.0
        )));
    }

    #[test]
    fn a_long_name_ends_in_an_ellipsis_before_the_cards_tools() {
        let look = Look::ONYX;
        let name = "Joako, drex, Yuki, Nini~ (Rat), 24nvto, Deity, Juansi, Joako, Ana";
        let group = Head {
            glyph: None,
            name: name.into(),
            mentions: 0,
            path: "9 miembros".into(),
            ..head(7)
        };
        let card = card(
            &group,
            Tools::Unread {
                bell: None,
                folded: true,
                local: false,
            },
            Body::Folded,
            &look,
        );
        let (_, laid) = frame(card, Size::new(464.0, 200.0));
        let read = region(&laid, "inbox:read:7");
        let (text, position) = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::Text { text, position, .. } if text.starts_with("Joako") => {
                    Some((text.clone(), *position))
                }
                _ => None,
            })
            .expect("the name is painted");
        assert!(text.ends_with('…'), "{text}");
        // 7 px a character, as the test measures them.
        let end = position.x + text.chars().count() as f32 * 7.0;
        assert!(end <= read.x - 8.0, "{end} reaches {read:?}");
    }

    #[test]
    fn mention_and_reminder_cards_go_to_their_message() {
        let look = Look::ONYX;
        let message = || Element::text("mira esto").size(None, Some(44.0));
        let (_, laid) = frame(
            card(
                &head(7),
                Tools::Mention { message: 70 },
                Body::Single(Box::new(message())),
                &look,
            ),
            Size::new(464.0, 300.0),
        );
        let shown = controls(&laid);
        assert!(shown.contains(&"inbox:jump:7:70".to_owned()), "{shown:?}");
        assert!(!shown.contains(&"inbox:read:7".to_owned()));
        let (_, laid) = frame(
            card(
                &head(7),
                Tools::Reminder {
                    message: 70,
                    local: true,
                },
                Body::Note("El mensaje marcado no está cargado.".into()),
                &look,
            ),
            Size::new(464.0, 300.0),
        );
        let shown = controls(&laid);
        assert!(shown.contains(&"inbox:jump:7:70".to_owned()));
        assert!(shown.contains(&"inbox:read:7".to_owned()));
        assert!(texts(&laid)
            .iter()
            .any(|text| text == "El mensaje marcado no está cargado."));
    }

    #[test]
    fn a_jumpable_message_offers_ir_in_its_corner() {
        let look = Look::ONYX;
        // A group's first message, 16 under the one above, in a card 16
        // wider: «Ir» 8 in from the message's end, 6 under its name, 24
        // high and 8 on either side of the word.
        let row = jumpable(
            Element::text("hola").size(Some(446.0), Some(64.0)),
            16.0,
            7,
            70,
            &look,
        );
        let (_, laid) = frame(Element::column(vec![row]), Size::new(462.0, 64.0));
        let jump = region(&laid, "inbox:jump:7:70");
        assert!((jump.x + jump.width - 438.0).abs() < 0.5, "{jump:?}");
        assert_eq!((jump.y, jump.height, jump.width), (22.0, 24.0, 30.0));
    }

    #[test]
    fn a_day_is_a_pill_over_a_hairline_16_in_from_the_card() {
        let look = Look::ONYX;
        let (_, laid) = frame(day("3 de agosto".into(), &look), Size::new(462.0, 28.0));
        let fills: Vec<limn::Rect> = laid
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect();
        let line = fills
            .iter()
            .find(|rect| rect.height == 1.0)
            .expect("a line");
        assert_eq!((line.x, line.width, line.y), (16.0, 430.0, 19.5));
        let pill = fills
            .iter()
            .find(|rect| rect.height == 16.0)
            .expect("a pill");
        // 11 characters of 7 px, 4 on either side, in the middle.
        assert_eq!((pill.width, pill.y), (85.0, 12.0));
        assert!((pill.x + pill.width / 2.0 - 231.0).abs() < 0.5, "{pill:?}");
    }

    #[test]
    fn an_empty_tab_says_why_with_its_glyph() {
        let look = Look::ONYX;
        let state = EmptyState {
            glyph: Icon::Custom(crate::glyphs::INBOX),
            title: "¡Ya estás al día!".into(),
            tip: "Elige «Marcar como no leído».".into(),
        };
        let (_, laid) = frame(empty(&state, &look), Size::new(504.0, 600.0));
        let painted = texts(&laid).join(" ");
        assert!(painted.contains("¡Ya estás al día!"), "{painted}");
        assert!(painted.contains("Marcar como no leído"), "{painted}");
        let height = laid
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect { rect, .. } if rect.width == 80.0 => Some(rect.height),
                _ => None,
            })
            .next();
        assert_eq!(height, Some(80.0));
    }

    #[test]
    fn the_tray_opens_the_popout_under_it_and_is_lit_while_open() {
        let look = Look::ONYX;
        let (_, closed) = frame(button(None, &look), Size::new(32.0, 32.0));
        assert!(controls(&closed).contains(&ids::OPEN.to_owned()));
        let popout = popout(
            header(&Header::default(), &look),
            empty(
                &EmptyState {
                    glyph: Icon::Custom(crate::glyphs::INBOX),
                    title: "¡Ya estás al día!".into(),
                    tip: String::new(),
                },
                &look,
            ),
            false,
            Size::new(504.0, 560.0),
            &look,
        );
        let (_, open) = frame(
            Element::row(vec![
                Element::spacer().flex(1.0),
                button(Some(popout), &look),
                gap(48.0, 0.0),
            ]),
            Size::new(1440.0, 800.0),
        );
        let tray = region(&open, ids::OPEN);
        let tab = region(&open, "inbox:unread");
        // The popout's end on the tray's, its top 8 under it.
        assert!(
            (tab.x - 4.0 - (tray.x + tray.width - 504.0)).abs() < 1.0,
            "{tab:?} {tray:?}"
        );
        assert!((tab.y - (tray.y + tray.height + 8.0 + 70.0)).abs() < 1.0);
    }
}
