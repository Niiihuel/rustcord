//! The account's settings: a large panel floating over the dimmed window,
//! with a sidebar of pages (and, under the open one, links to its sections
//! that follow the scroll), a search box, and the page's column of settings.
//! Pure functions of props like the rest of the crate; `rustcord-ui` keeps the
//! preferences and turns the actions into intents.
//!
//! Measures follow Discord's 2025 settings as its captures show them: a
//! panel 40 px in from every side of a window up to 1600 wide, a 253 px
//! sidebar of 220 px items 44 px apart, a 48 px header over the page, and a
//! 696 px column of settings centered in what is left.

use limn::{
    Button, ButtonSizing, ButtonVariant, Color, Component, ComponentPalette, ControlSize, Element,
    FontFamily, Icon, LevelBars, MarkStyle, Point, RadioGroup, RadioMetrics, RadioOption,
    RadioVariant, ScrollArea, ScrollbarAppearance, SearchField, Size, Slider, Switch,
    SwitchVariant, Tooltip,
};
use rustcord_model::{
    ChannelId, Delivery, Message, MessageId, MessageKind, Reaction, Timestamp, UserId,
};

use rustcord_i18n::t;

use crate::message::{message_row, Abilities, Author, MessageStyle, MessageView};
use crate::{avatar, gap, icon, line, strong_line, text, with_status, Look};

/// A person, for the account's page.
pub const PERSON: &str = crate::glyphs::glyph!("person");
/// A shield with a check, for the page of data and privacy.
pub const SHIELD: &str = crate::glyphs::glyph!("shield");
/// A circle crossed by a slash: the blocked accounts.
pub const BLOCK: &str = crate::glyphs::glyph!("block");

/// Room between the panel and the window, on every side, up to [`WIDE`].
pub const MARGIN: f32 = 40.0;
/// The room a window wider than [`WIDE`] keeps.
const MARGIN_WIDE: f32 = 72.0;
const WIDE: f32 = 1600.0;
/// A window this narrow is the panel: no margin, radius or border.
const FULL: f32 = 1080.0;
/// The panel is at most this wide, centered in a wider window.
const PANEL_MAX: f32 = 1400.0;
const PANEL_RADIUS: f32 = 12.0;
/// The sidebar: 12 px of ground, 240 px of content and a 1 px line at its end.
pub const SIDEBAR_WIDTH: f32 = 253.0;
/// Where the sidebar's content starts: its 12 px of ground and 4 more.
const ITEM_INSET: f32 = 16.0;
/// The width of the items and of the search field.
pub const ITEM_WIDTH: f32 = 220.0;
pub const ITEM_HEIGHT: f32 = 36.0;
/// Room between two items.
const ITEM_GAP: f32 = 8.0;
/// Room at each side of an item's icon and of its label.
const ITEM_PAD: f32 = 12.0;
/// The account header's avatar, and the header's height with the room around it.
const AVATAR: f32 = 48.0;
const ACCOUNT_HEIGHT: f32 = 64.0;
/// The height of a select or a text field in a settings page.
pub const FIELD_HEIGHT: f32 = 40.0;
/// A settings field's corners, Discord's 8.
pub const FIELD_RADIUS: f32 = 8.0;
/// A settings button's height, and a small one's.
const BUTTON_HEIGHT: f32 = 38.0;
/// A small button of a long list (the sounds): 30 px tall, at least 60 wide.
pub const SMALL_BUTTON_HEIGHT: f32 = 30.0;
/// Under a group's last item: 8 px of ground and its line, then 16 px to the next
/// group (Discord's CSS adds 24; its rendered sidebar measures 16).
const GROUP_PAD: f32 = 8.0;
const GROUP_GAP: f32 = 16.0;
/// The track down the open page's links: 2 px wide, under the middle of its icons.
const TRACK: f32 = 2.0;
const TRACK_INDENT: f32 = ITEM_PAD + 10.0 - TRACK / 2.0;
/// From the track to a link's label, and the room over and under the links.
const LINK_INDENT: f32 = 17.0;
const LINKS_PAD: f32 = 8.0;
const LINK_GAP: f32 = 16.0;
/// The bar over a page, with its title and the close button.
pub const HEADER_HEIGHT: f32 = 48.0;
/// The column of settings, without its ground: 696 px at most.
pub const COLUMN_WIDTH: f32 = 696.0;
/// The ground at each side of the column, and the room above and below it.
const COLUMN_EDGE: f32 = 16.0;
const COLUMN_SPACE: f32 = 64.0;
const COLUMN_SPACE_FULL: f32 = 80.0;
/// Room over and under the line between two sections, Discord's.
const SECTION_SPACE: f32 = 50.0;
/// Room between two settings of a section.
const ROW_GAP: f32 = 8.0;
/// How tall the preview over the accessibility settings is.
pub const PREVIEW_HEIGHT: f32 = 199.0;

/// Where the panel and its column go in a window this big.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub panel: Size,
    /// The column of settings, without its ground.
    pub column: f32,
    /// The panel fills a window this narrow: no margin, radius or border.
    pub full: bool,
}

impl Geometry {
    /// The panel keeps [`MARGIN`] from the window's edges (more from a wider
    /// one, and none in a narrow one), and is at most [`PANEL_MAX`] wide; the
    /// column takes what the sidebar leaves, up to [`COLUMN_WIDTH`].
    pub fn new(window: Size) -> Self {
        let full = window.width <= FULL;
        let margin = if full {
            0.0
        } else if window.width <= WIDE {
            MARGIN
        } else {
            MARGIN_WIDE
        };
        let panel = Size::new(
            (window.width - 2.0 * margin).clamp(320.0, PANEL_MAX),
            (window.height - 2.0 * margin).max(320.0),
        );
        let room = panel.width - SIDEBAR_WIDTH - 2.0 * COLUMN_EDGE;
        Self {
            panel,
            column: room.clamp(200.0, COLUMN_WIDTH),
            full,
        }
    }
}

/// Who the settings are of, over the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub image: Option<limn::RasterImage>,
    pub accent: Color,
    /// The link that opens the profile editor; `None` when there is none.
    pub edit: Option<String>,
    /// The profiles are open: the card is their line of the sidebar.
    pub selected: bool,
}

/// One line of the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub enum Nav {
    /// A group's title, as written.
    Heading(String),
    /// A group's title that is not shown: a screen reader still reads it.
    HiddenHeading(String),
    /// The message under the search when nothing matches it.
    Empty,
    /// The line that closes a group, before the next one.
    Separator,
    Page(NavPage),
    /// A line that acts rather than opens a page: signing out.
    Action(NavAction),
}

/// A line of the sidebar that acts, as «Cerrar sesión».
#[derive(Clone, Debug, PartialEq)]
pub struct NavAction {
    pub id: String,
    pub label: String,
    pub icon: Icon,
}

/// A page of the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct NavPage {
    pub id: String,
    pub label: String,
    pub icon: Icon,
    pub selected: bool,
    /// The links to its sections, shown while it is open.
    pub anchors: Vec<NavAnchor>,
}

/// A link to a section of the open page.
#[derive(Clone, Debug, PartialEq)]
pub struct NavAnchor {
    pub id: String,
    pub label: String,
    /// The section being read.
    pub active: bool,
}

/// A settings field's outline: the look's hairline at rest, its accent while focused.
pub fn field_outline(look: &Look) -> (Color, Color) {
    (look.border, look.accent)
}

/// The search box over the sidebar: a field and, while it holds something,
/// a button that empties it (`{id}:clear`).
pub fn search_box(id: &str, value: &str, look: &Look) -> Element {
    SearchField::new(id, value, t("Buscar"), look.sidebar_palette())
        .accessibility_label(t("Buscar en los ajustes"))
        .size(ControlSize::Spacious)
        .width(ITEM_WIDTH)
        .clearable(true)
        .height(FIELD_HEIGHT)
        .persistent_placeholder(true)
        .render()
}

/// The sidebar: who the settings are of and the search box, both fixed,
/// and under them the pages, scrolling, with the footnote at their end.
pub fn sidebar(
    account: &Account,
    search: Element,
    nav: &[Nav],
    footnote: &str,
    look: &Look,
) -> Element {
    let mut list = vec![gap(0.0, 12.0)];
    let mut entries = nav.iter().peekable();
    while let Some(entry) = entries.next() {
        match entry {
            Nav::Heading(title) => list.push(heading(title, look)),
            // Nothing to see: the title is only read out.
            Nav::HiddenHeading(title) => list.push(
                Element::row(Vec::new())
                    .size(Some(0.0), Some(0.0))
                    .accessible(limn::Access::new(limn::Role::Heading).name(title.clone())),
            ),
            Nav::Empty => list.push(empty_search(look)),
            Nav::Separator => list.push(Element::column(vec![
                gap(0.0, GROUP_PAD),
                Element::row(vec![
                    gap(ITEM_INSET, 0.0),
                    Element::panel(Element::spacer())
                        .size(Some(ITEM_WIDTH), Some(1.0))
                        .background(look.border),
                ]),
                gap(0.0, GROUP_GAP),
            ])),
            Nav::Page(page) => {
                list.push(nav_item(page, look));
                if page.selected && !page.anchors.is_empty() {
                    list.push(links(&page.anchors, look));
                }
                if matches!(entries.peek(), Some(Nav::Page(_) | Nav::Action(_))) {
                    list.push(gap(0.0, ITEM_GAP));
                }
            }
            Nav::Action(action) => list.push(action_item(action, look)),
        }
    }
    list.push(gap(0.0, 20.0));
    list.push(Element::row(vec![
        gap(ITEM_INSET, 0.0),
        text(footnote, 10.0, look.faint).size(Some(ITEM_WIDTH), None),
    ]));
    list.push(gap(0.0, 24.0));
    let fixed = Element::column(vec![
        gap(0.0, 16.0),
        Element::row(vec![gap(ITEM_INSET, 0.0), account_row(account, look)]),
        gap(0.0, 16.0),
        Element::row(vec![gap(ITEM_INSET, 0.0), search]),
    ]);
    let scroll = ScrollArea::new("us:nav", Element::column(list))
        .appearance(ScrollbarAppearance::overlay().buttons(false))
        .render()
        .flex(1.0);
    Element::row(vec![
        Element::column(vec![fixed, scroll]).size(Some(SIDEBAR_WIDTH - 1.0), None),
        Element::panel(Element::spacer())
            .size(Some(1.0), None)
            .background(look.border),
    ])
    .size(Some(SIDEBAR_WIDTH), None)
    .background(look.sidebar)
}

/// Who the settings are of, as a row of the menu: the avatar and the name
/// over «Editar perfiles», as wide as the items, with their radius and hover.
/// The whole row opens the profile editor when there is one.
fn account_row(account: &Account, look: &Look) -> Element {
    let mut who = vec![strong_line(&account.name, 16.0, look.text)];
    if account.edit.is_some() {
        who.push(Element::row(vec![
            line(t("Editar perfiles"), 14.0, look.muted)
                .shrink_to_fit()
                .centered_y(),
            gap(4.0, 0.0),
            icon(Icon::Edit, look.muted)
                .size(Some(12.0), Some(12.0))
                .centered_y(),
        ]));
    }
    let content = Element::row(vec![
        gap(ITEM_PAD, 0.0),
        crate::avatar_with_image(&account.name, account.accent, AVATAR, account.image.clone())
            .centered_y(),
        gap(12.0, 0.0),
        Element::column(who).gap(2.0).flex(1.0).centered_y(),
        gap(ITEM_PAD, 0.0),
    ]);
    let size = (Some(ITEM_WIDTH), Some(ACCOUNT_HEIGHT));
    match &account.edit {
        Some(edit) => {
            let card = Element::clickable(edit, content)
                .size(size.0, size.1)
                .radius(8.0)
                .hover_background(look.hover)
                .hover_foreground(look.text)
                .accessible(
                    limn::Access::new(limn::Role::Button)
                        .name(t("Editar perfiles"))
                        .selected(account.selected),
                );
            if account.selected {
                card.background(look.selected)
            } else {
                card
            }
        }
        None => content.size(size.0, size.1),
    }
}

/// A group's title, in sentence case and quiet: 14 px medium, 8 px above it
/// and 8 px in from the sidebar's items.
fn heading(title: &str, look: &Look) -> Element {
    Element::column(vec![
        gap(0.0, 8.0),
        Element::row(vec![
            gap(ITEM_INSET + 8.0, 0.0),
            line(title, 14.0, look.muted)
                .font_family(FontFamily::SansMedium)
                .flex(1.0)
                .centered_y(),
        ])
        .size(None, Some(18.0)),
        gap(0.0, 4.0),
    ])
}

/// What a search that finds nothing shows: two centered lines of the
/// sidebar's width, with 24 px above and below them.
fn empty_search(look: &Look) -> Element {
    Element::row(vec![
        gap(ITEM_INSET, 0.0),
        Element::column(vec![
            gap(0.0, 24.0),
            text(t("No se encontraron resultados"), 14.0, look.text)
                .font_family(FontFamily::SansSemibold)
                .center_content(),
            gap(0.0, 4.0),
            text(
                t("Lo intentamos, pero no encontramos nada."),
                14.0,
                look.muted,
            )
            .center_content(),
            gap(0.0, 24.0),
        ])
        .size(Some(ITEM_WIDTH), None),
    ])
}

/// A page's line: its icon and its name, which wraps as Discord's does
/// («Privacidad de la actividad»): 36 px tall for one row of text, about 56
/// for two.
fn nav_item(page: &NavPage, look: &Look) -> Element {
    let ink = if page.selected { look.text } else { look.muted };
    let item = Element::clickable(
        &page.id,
        Element::row(vec![
            gap(ITEM_PAD, 0.0),
            icon(page.icon, ink)
                .size(Some(20.0), Some(20.0))
                .centered_y(),
            gap(8.0, 0.0),
            // A row of 16 px text is about 20.6 tall: 7 above and below
            // keep one row within the 36 and give two Discord's 56.
            Element::column(vec![
                gap(0.0, 7.0),
                text(page.label.clone(), 16.0, ink).font_family(FontFamily::SansMedium),
                gap(0.0, 7.0),
            ])
            .flex(1.0)
            .centered_y(),
            gap(ITEM_PAD, 0.0),
        ]),
    )
    .size(Some(ITEM_WIDTH), None)
    .min_size(ITEM_WIDTH, ITEM_HEIGHT)
    .radius(8.0)
    .hover_background(look.hover)
    .hover_foreground(look.text)
    .accessible(
        limn::Access::new(limn::Role::Tab)
            .name(page.label.clone())
            .selected(page.selected),
    );
    let item = if page.selected {
        item.background(look.selected)
    } else {
        item
    };
    Element::row(vec![gap(ITEM_INSET, 0.0), item])
}

/// A line that acts: in the danger ink, with a faint danger fill under the
/// pointer, as Discord's destructive item.
fn action_item(action: &NavAction, look: &Look) -> Element {
    let item = Element::clickable(
        &action.id,
        Element::row(vec![
            gap(ITEM_PAD, 0.0),
            icon(action.icon, look.danger)
                .size(Some(20.0), Some(20.0))
                .centered_y(),
            gap(8.0, 0.0),
            line(action.label.clone(), 16.0, look.danger)
                .font_family(FontFamily::SansMedium)
                .flex(1.0)
                .centered_y(),
            gap(ITEM_PAD, 0.0),
        ]),
    )
    .size(Some(ITEM_WIDTH), Some(ITEM_HEIGHT))
    .radius(8.0)
    .hover_background(Look::alpha(look.danger, 20))
    .hover_foreground(look.danger)
    .accessible(limn::Access::new(limn::Role::Button).name(action.label.clone()));
    Element::row(vec![gap(ITEM_INSET, 0.0), item])
}

/// The links to the sections of the open page, down a track that runs its
/// whole height. The current link's stretch of it is in the text's color,
/// the thumb; the labels wrap beside it.
fn links(anchors: &[NavAnchor], look: &Look) -> Element {
    // A piece of the track between two links, or above and below them.
    let segment = |height: f32| {
        Element::row(vec![
            gap(TRACK_INDENT, 0.0),
            Element::panel(Element::spacer())
                .size(Some(TRACK), Some(height))
                .background(look.selected),
        ])
    };
    let last = anchors.len() - 1;
    let mut rows = vec![segment(LINKS_PAD)];
    for (index, anchor) in anchors.iter().enumerate() {
        let (ink, mark) = if anchor.active {
            (look.text, look.text)
        } else {
            (look.muted, look.selected)
        };
        // The thumb moves by jumping: limn has no transition of a position, nor a
        // tween of an element's translation.
        let link = Element::row(vec![
            gap(TRACK_INDENT, 0.0),
            Element::panel(Element::spacer())
                .size(Some(TRACK), None)
                .background(mark),
            gap(LINK_INDENT, 0.0),
            Element::clickable(&anchor.id, text(anchor.label.clone(), 16.0, ink))
                .size(Some(ITEM_WIDTH - TRACK_INDENT - TRACK - LINK_INDENT), None)
                .hover_foreground(look.text)
                .accessible(
                    limn::Access::new(limn::Role::Link)
                        .name(anchor.label.clone())
                        .selected(anchor.active),
                ),
        ]);
        rows.push(if index < last {
            Element::column(vec![link, segment(LINK_GAP)])
        } else {
            link
        });
    }
    rows.push(segment(LINKS_PAD));
    Element::row(vec![gap(ITEM_INSET, 0.0), Element::column(rows)])
}

/// The panel: the sidebar, and the page under its header with the title and
/// the close button. A window that is no bigger than [`FULL`] is the panel
/// itself, with no radius, border or shadow.
pub fn panel(
    geometry: Geometry,
    sidebar: Element,
    title: &str,
    close: &str,
    page: Element,
    look: &Look,
) -> Element {
    let close = Element::clickable(
        close,
        crate::centered(icon(Icon::Close, look.muted).size(Some(16.0), Some(16.0))),
    )
    .size(Some(30.0), Some(30.0))
    .radius(8.0)
    .hover_background(look.selected)
    .hover_foreground(look.text)
    .accessible(limn::Access::new(limn::Role::Button).name(t("Cerrar")));
    let header = Element::row(vec![
        gap(16.0, 0.0),
        line(title, 16.0, look.text)
            .font_family(FontFamily::SansMedium)
            .flex(1.0)
            .centered_y(),
        close.centered_y(),
        gap(8.0, 0.0),
    ])
    .size(None, Some(HEADER_HEIGHT));
    let rule = Element::panel(Element::spacer())
        .size(None, Some(1.0))
        .background(look.border);
    let frame = Element::row(vec![
        sidebar,
        Element::column(vec![header, rule, page.flex(1.0)]).flex(1.0),
    ])
    .size(Some(geometry.panel.width), Some(geometry.panel.height))
    .background(look.canvas)
    .clip_children()
    .accessible(limn::Access::new(limn::Role::Dialog).name(t("Ajustes de usuario")));
    if geometry.full {
        frame
    } else {
        frame.border(look.border, 1.0).radius(PANEL_RADIUS).shadow(
            look.shadow,
            Point::new(0.0, 12.0),
            24.0,
            0.0,
        )
    }
}

/// A page's column: `sticky` pinned over it while it scrolls, then its
/// sections with a line between two, centered in the page with the ground
/// of [`COLUMN_EDGE`] at each side. `top` is an anchor at its very top.
pub fn page(
    scroll_id: &str,
    top: &str,
    geometry: Geometry,
    sticky: Option<Element>,
    sections: Vec<Element>,
    look: &Look,
) -> Element {
    let space = if geometry.full {
        COLUMN_SPACE_FULL
    } else {
        COLUMN_SPACE
    };
    let mut column = vec![gap(0.0, space).anchor(top)];
    if let Some(sticky) = sticky {
        column.push(sticky.sticky_top().background(look.canvas));
    }
    for (index, section) in sections.into_iter().enumerate() {
        if index > 0 {
            column.push(divider(look));
        }
        column.push(section);
    }
    column.push(gap(0.0, space));
    ScrollArea::new(
        scroll_id,
        Element::row(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![
                gap(COLUMN_EDGE, 0.0),
                Element::column(column).size(Some(geometry.column), None),
                gap(COLUMN_EDGE, 0.0),
            ]),
            Element::spacer().flex(1.0),
        ]),
    )
    .appearance(ScrollbarAppearance::overlay().buttons(false))
    .render()
}

/// The line between two sections, or two groups of one.
pub fn divider(look: &Look) -> Element {
    Element::column(vec![
        gap(0.0, SECTION_SPACE),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
        gap(0.0, SECTION_SPACE),
    ])
}

/// A section: its title, 24 px medium on one line, with an action beside it
/// when there is one, and under it an optional note and its settings, 8 px
/// apart. `anchor` is where its link in the sidebar jumps.
pub fn section(
    anchor: &str,
    title: &str,
    subtitle: Option<&str>,
    trailing: Option<Element>,
    rows: Vec<Element>,
    look: &Look,
) -> Element {
    let mut head = vec![line(title, 24.0, look.text)
        .font_family(FontFamily::SansMedium)
        .flex(1.0)
        .centered_y()];
    if let Some(trailing) = trailing {
        head.push(trailing.centered_y());
    }
    let mut top = vec![Element::row(head).gap(16.0).min_size(0.0, 32.0)];
    if let Some(note) = subtitle {
        top.push(gap(0.0, 4.0));
        top.push(text(note, 14.0, look.muted));
    }
    let mut column = vec![Element::column(top), gap(0.0, 20.0)];
    column.extend(spaced(rows));
    Element::column(column).anchor(anchor)
}

/// Rows one under the other, [`ROW_GAP`] apart.
fn spaced(rows: Vec<Element>) -> Vec<Element> {
    let mut out = Vec::with_capacity(rows.len() * 2);
    for (index, row) in rows.into_iter().enumerate() {
        if index > 0 {
            out.push(gap(0.0, ROW_GAP));
        }
        out.push(row);
    }
    out
}

/// Rows that stand one under the other, as a group of their own.
pub fn rows(rows: Vec<Element>) -> Element {
    Element::column(spaced(rows))
}

/// The square that opens a [`fold_row`].
pub const FOLD_BUTTON: f32 = 48.0;

/// A row that opens more settings under it, as Discord's: its label and
/// what it holds at the left, and at the right a 48 px square with a
/// chevron that reports `id`.
pub fn fold_row(id: &str, label: &str, note: &str, open: bool, look: &Look) -> Element {
    let veil = veil(look);
    let chevron = if open {
        Icon::ChevronUp
    } else {
        Icon::ChevronDown
    };
    let button = Element::clickable(
        id,
        crate::centered(icon(chevron, look.text).large_icon(24.0)),
    )
    .size(Some(FOLD_BUTTON), Some(FOLD_BUTTON))
    .radius(8.0)
    .background(veil.raised)
    .hover_background(veil.hover)
    .accessible(
        limn::Access::new(limn::Role::Button)
            .name(label.to_owned())
            .expanded(open),
    );
    Element::column(vec![
        gap(0.0, 8.0),
        Element::row(vec![
            Element::column(vec![
                text(label, 16.0, look.text).font_family(FontFamily::SansMedium),
                text(note, 14.0, look.muted),
            ])
            .gap(4.0)
            .flex(1.0)
            .centered_y(),
            button.centered_y(),
        ])
        .gap(24.0),
        gap(0.0, 8.0),
    ])
}

/// A row of a list whose rows are parted by lines, such as the call's
/// sounds: the row and a line under it, about 18 px from each.
pub fn listed(row: Element, look: &Look) -> Element {
    Element::column(vec![
        gap(0.0, 10.0),
        row,
        gap(0.0, 19.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
    ])
}

/// Text that does something when clicked, in the links' blue: 14 px, as
/// Discord's «Vista previa del sonido». `name` is what it is called to the
/// platform where the text alone says too little.
pub fn link(id: &str, label: &str, name: &str, look: &Look) -> Element {
    Button::new(id, label, look.palette())
        .variant(ButtonVariant::Link)
        .sizing(ButtonSizing {
            height: 18.0,
            font_size: 14.0,
            icon_size: 14.0,
            horizontal_padding: 0.0,
            inner_spacing: 4.0,
        })
        .ink(look.palette().link())
        .accessibility_label(name)
        .render()
}

/// A title inside a section, over the settings it groups: 16 px semibold,
/// 8 px above it and 16 px to the settings under it.
pub fn subtitle(title: &str, look: &Look) -> Element {
    Element::column(vec![
        gap(0.0, 8.0),
        line(title, 16.0, look.text).font_family(FontFamily::SansSemibold),
        gap(0.0, 16.0),
    ])
}

/// A setting: its name, a note under it, and its control at the right. The
/// control is centered with the name, or starts at its top when a note hangs
/// under the name. The text takes every width the control leaves, up to 560
/// px for the note. A setting that cannot be changed is dimmed.
pub fn setting(
    label: &str,
    note: Option<&str>,
    control: Option<Element>,
    enabled: bool,
    look: &Look,
) -> Element {
    let (ink, quiet) = if enabled {
        (look.text, look.muted)
    } else {
        (look.muted, look.faint)
    };
    let mut copy = vec![text(label, 16.0, ink).font_family(FontFamily::SansMedium)];
    if let Some(note) = note {
        copy.push(text(note, 14.0, quiet).max_width(560.0));
    }
    let mut row = vec![Element::column(copy).gap(4.0).flex(1.0).centered_y()];
    if let Some(control) = control {
        row.push(if note.is_some() {
            control
        } else {
            control.centered_y()
        });
    }
    Element::column(vec![
        gap(0.0, 8.0),
        Element::row(row).gap(24.0).min_size(0.0, 32.0),
        gap(0.0, 8.0),
    ])
}

/// A settings switch, Discord's: a 48 by 24 track with a 16 px thumb, in the
/// accent when on; off, a translucent track with a hairline.
pub fn switch(id: impl Into<String>, on: bool, look: &Look) -> Switch {
    // The off track's fill and hairline follow the look's lightness, as `Look::hint` does.
    let (fill, border) = if look.canvas.r > 128 {
        (Color::rgba(0, 0, 0, 5), Color::rgba(0, 0, 0, 41))
    } else {
        (Color::rgba(0, 0, 0, 31), Color::rgba(152, 152, 160, 51))
    };
    Switch::new(id, on, look.palette())
        .track_size(48.0, 24.0)
        .thumb_inset(4.0)
        .variant(SwitchVariant::Accent)
        .ink(look.accent)
        .off_track(fill, border)
}

/// A setting that is on or off: a switch of 48 by 24 px, in the accent when
/// it is on. One that cannot be changed says why.
pub fn toggle(
    id: &str,
    label: &str,
    note: Option<&str>,
    on: bool,
    denied: Option<&str>,
    look: &Look,
) -> Element {
    let switch = switch(id, on, look)
        .accessibility_label(label)
        .enabled(denied.is_none())
        .render();
    let note = denied.or(note);
    setting(label, note, Some(switch), denied.is_none(), look)
}

/// One choice among a few, one under the other as a plain list, each with
/// its note under its name.
pub fn choices(
    id: &str,
    label: &str,
    options: &[(String, &str, Option<&str>)],
    selected: &str,
    look: &Look,
) -> Element {
    let options = options
        .iter()
        .map(|(value, name, note)| {
            let option = RadioOption::new(value.clone(), *name);
            match note {
                // A note that only repeats the name is left out.
                Some(note) if note != name => option.description(*note),
                _ => option,
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
    Element::column(vec![gap(0.0, 8.0), group, gap(0.0, 4.0)])
}

/// A settings slider, Discord's: a 16 px thumb, 20 under the pointer, on a 4 px
/// bar, 6 under the pointer, filled in the accent.
pub fn slider(id: impl Into<String>, fraction: f32, look: &Look) -> Slider {
    Slider::new(id, fraction, look.palette())
        .ink(look.accent)
        .thumb(16.0, 20.0)
        .bar(4.0, 6.0)
}

/// A slider that stops at marks, each named over it, the usual one's mark in
/// green, as Discord's: `labels[index]` is chosen and `labels[usual]` is the default.
pub fn ticked_slider(
    id: &str,
    label: &str,
    labels: &[&str],
    index: usize,
    usual: usize,
    width: f32,
    look: &Look,
) -> Element {
    let last = labels.len().saturating_sub(1).max(1) as f32;
    slider(id, index as f32 / last, look)
        .steps(labels.len())
        .width(width)
        .mark_style(MarkStyle {
            dash: Size::new(2.0, 24.0),
            label_size: 12.0,
            label_weight_medium: true,
            default_mark_ink: Some(look.online),
        })
        .mark_labels(labels.iter().copied())
        .default_stop(usual)
        .accessibility_label(label)
        .render()
}

/// Which mark of a [`ticked_slider`] of `marks` a fraction falls on.
pub fn mark_at(fraction: f32, marks: usize) -> usize {
    let last = marks.saturating_sub(1);
    ((fraction.clamp(0.0, 1.0) * last as f32).round() as usize).min(last)
}

/// A look the account can choose, as a square of its ground.
#[derive(Clone, Debug, PartialEq)]
pub struct Swatch {
    pub id: String,
    pub name: String,
    pub fill: Color,
    pub selected: bool,
}

/// The looks to choose from: 48 px squares 56 px apart, the chosen one ringed
/// in the accent with a check six px out of its corner, each named on hover.
pub fn theme_swatches(swatches: &[Swatch], look: &Look) -> Element {
    let squares = swatches
        .iter()
        .map(|swatch| {
            let (edge, width) = if swatch.selected {
                (look.accent, 4.0)
            } else {
                (look.muted, 1.0)
            };
            let square = Element::clickable(&swatch.id, Element::spacer())
                .size(Some(48.0), Some(48.0))
                .radius(8.0)
                .background(swatch.fill)
                .border(edge, width)
                .accessible(
                    limn::Access::new(limn::Role::RadioButton)
                        .name(swatch.name.clone())
                        .toggled(swatch.selected),
                );
            let square = if swatch.selected {
                let check = Element::panel(crate::centered(
                    icon(Icon::Check, look.on_accent).size(Some(12.0), Some(12.0)),
                ))
                .size(Some(18.0), Some(18.0))
                .radius(9.0)
                .background(look.accent)
                .translate(Point::new(6.0, -6.0));
                Element::stack(vec![
                    square,
                    Element::column(vec![
                        Element::row(vec![Element::spacer().flex(1.0), check]),
                        Element::spacer().flex(1.0),
                    ]),
                ])
                .size(Some(48.0), Some(48.0))
            } else {
                square
            };
            Tooltip::new(
                square,
                swatch.id.clone(),
                swatch.name.clone(),
                look.menu_palette(),
            )
            .render()
        })
        .collect();
    Element::row(squares).gap(8.0)
}

/// Keys of a shortcut, each alternative a combination of keys pressed together.
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcut {
    pub action: String,
    pub keys: Vec<Vec<String>>,
}

/// A list of shortcuts in a card: one row each, what it does at the left and
/// its keys at the right, alternatives one under the other.
pub fn shortcuts(rows: &[Shortcut], look: &Look) -> Element {
    let mut lines = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            lines.push(
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(look.border),
            );
        }
        let keys = row
            .keys
            .iter()
            .map(|combination| {
                Element::row(
                    std::iter::once(Element::spacer().flex(1.0))
                        .chain(combination.iter().map(|key| keycap(key, look)))
                        .collect(),
                )
                .gap(4.0)
            })
            .collect();
        lines.push(Element::column(vec![
            gap(0.0, 9.0),
            Element::row(vec![
                gap(18.0, 0.0),
                text(row.action.clone(), 16.0, look.text)
                    .flex(1.0)
                    .centered_y(),
                gap(16.0, 0.0),
                Element::column(keys).gap(4.0).centered_y(),
                gap(18.0, 0.0),
            ])
            .min_size(0.0, 41.5),
            gap(0.0, 9.0),
        ]));
    }
    card(Element::column(lines), look)
}

/// One key, as printed on it.
pub(crate) fn keycap(key: &str, look: &Look) -> Element {
    Element::panel(Element::row(vec![
        gap(6.0, 0.0),
        Element::text(key.to_uppercase())
            .font_family(FontFamily::SansBold)
            .font_size(12.0)
            .foreground(look.text)
            .centered_y(),
        gap(6.0, 0.0),
    ]))
    .size(None, Some(22.5))
    .min_size(22.5, 22.5)
    .background(look.selected)
    .border(look.border, 1.0)
    .radius(4.0)
}

/// A card: its content on the raised ground, with a hairline around it and
/// rounded corners.
pub fn card(content: Element, look: &Look) -> Element {
    Element::panel(content)
        .background(look.raised)
        .border(look.border, 1.0)
        .radius(12.0)
}

/// A card that leads to settings on another page, under the heading «Ajustes
/// relacionados»: a row with the page's glyph on a rounded square, its
/// title, its note and a chevron.
pub fn related(id: &str, glyph: Icon, title: &str, note: &str, look: &Look) -> Element {
    let square = Element::panel(crate::centered(icon(glyph, look.text).large_icon(20.0)))
        .size(Some(48.0), Some(48.0))
        .radius(8.0)
        .background(veil(look).raised);
    Element::column(vec![
        gap(0.0, 8.0),
        line(t("Ajustes relacionados"), 16.0, look.muted).font_family(FontFamily::SansSemibold),
        gap(0.0, 8.0),
        Element::clickable(
            id,
            Element::row(vec![
                square.centered_y(),
                Element::column(vec![
                    line(title, 16.0, look.text).font_family(FontFamily::SansMedium),
                    text(note, 14.0, look.muted),
                ])
                .gap(4.0)
                .flex(1.0)
                .centered_y(),
                icon(Icon::ChevronRight, look.muted).centered_y(),
            ])
            .gap(12.0)
            .padding(12.0),
        )
        .radius(16.0)
        .hover_background(look.hover)
        .accessible(limn::Access::new(limn::Role::Link).name(title.to_owned())),
    ])
}

/// Someone of a list in the settings: their 40 px picture, their name over
/// their handle, and at the right the control that undoes what the list is
/// for.
pub fn person(
    name: &str,
    handle: &str,
    image: Option<limn::RasterImage>,
    accent: Color,
    control: Element,
    look: &Look,
) -> Element {
    Element::row(vec![
        crate::avatar_with_image(name, accent, 40.0, image).centered_y(),
        Element::column(vec![
            line(name, 16.0, look.text).font_family(FontFamily::SansSemibold),
            line(handle, 14.0, look.muted),
        ])
        .gap(2.0)
        .flex(1.0)
        .centered_y(),
        control.centered_y(),
    ])
    .gap(8.0)
}

/// A list of people in a card, as Discord's blocked accounts: a header on
/// a lighter ground with `glyph` in a 40 px circle, the list's name over
/// how many it holds, then a [`person`] each, 16 px in from the sides and
/// parted by lines.
pub fn people_card(
    glyph: Icon,
    title: &str,
    count: &str,
    people: Vec<Element>,
    look: &Look,
) -> Element {
    let veil = veil(look);
    let head = Element::row(vec![
        Element::panel(crate::centered(icon(glyph, look.muted).large_icon(24.0)))
            .size(Some(40.0), Some(40.0))
            .radius(20.0)
            .background(veil.hover)
            .centered_y(),
        Element::column(vec![
            line(title, 16.0, look.text).font_family(FontFamily::SansSemibold),
            line(count, 16.0, look.text),
        ])
        .gap(2.0)
        .flex(1.0)
        .centered_y(),
    ])
    .gap(8.0)
    .padding(16.0)
    .background(veil.raised);
    let mut list = Vec::new();
    for (index, person) in people.into_iter().enumerate() {
        if index > 0 {
            list.push(
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(look.border),
            );
        }
        list.push(Element::column(vec![
            gap(0.0, 16.0),
            person,
            gap(0.0, 16.0),
        ]));
    }
    Element::column(vec![
        head,
        Element::row(vec![
            gap(16.0, 0.0),
            Element::column(list).flex(1.0),
            gap(16.0, 0.0),
        ]),
    ])
    .background(look.raised)
    .radius(8.0)
    .clip_children()
    .accessible(limn::Access::new(limn::Role::Group).name(title.to_owned()))
}

/// A note that informs, framed in the accent.
pub fn notice(message: &str, look: &Look) -> Element {
    Element::panel(
        Element::row(vec![
            icon(Icon::InfoCircle, look.accent).centered_y(),
            text(message, 14.0, look.text).flex(1.0).centered_y(),
        ])
        .gap(16.0)
        .padding(12.0),
    )
    .background(Look::alpha(look.accent, 14))
    .border(Look::alpha(look.accent, 150), 1.0)
    .radius(8.0)
    .accessible(limn::Access::new(limn::Role::Status).name(message.to_owned()))
}

/// The microphone's level as recent bars, the newest at the right.
pub fn level_bars(id: &str, levels: &[f32], width: f32, look: &Look) -> Element {
    Element::level_bars(LevelBars {
        id: id.to_owned(),
        levels: levels.to_vec(),
        bar: 4.0,
        gap: 4.0,
        color: look.online,
        rest: look.selected,
    })
    .size(Some(width), Some(32.0))
}

/// The level that counts as voice, over the microphone's level: a bar as
/// wide as the slider under it, filled to the level heard (green once it
/// passes the threshold), with a mark at the threshold. With nothing heard
/// it is Discord's track: yellow under the threshold, green over it.
pub fn sensitivity(level: Option<f32>, threshold: f32, width: f32, look: &Look) -> Element {
    const INSET: f32 = 6.0;
    let share = |value: f32| value.clamp(0.0005, 0.9995);
    let heard = level.is_some();
    let level = level.map_or(threshold, |level| level.clamp(0.0, 1.0));
    let fill = if heard && level >= threshold {
        look.online
    } else {
        look.idle
    };
    let rest = if heard { look.selected } else { look.online };
    let bar = |fraction: f32, color: Color| {
        Element::row(vec![
            Element::panel(Element::spacer())
                .background(color)
                .radius(4.0)
                .flex(share(fraction)),
            Element::spacer().flex(share(1.0 - fraction)),
        ])
    };
    let mark = Element::row(vec![
        Element::spacer().flex(share(threshold)),
        Element::panel(Element::spacer())
            .size(Some(2.0), Some(16.0))
            .background(look.text),
        Element::spacer().flex(share(1.0 - threshold)),
    ]);
    Element::row(vec![
        gap(INSET, 0.0),
        Element::stack(vec![
            Element::column(vec![
                gap(0.0, 4.0),
                Element::stack(vec![
                    Element::panel(Element::spacer())
                        .background(rest)
                        .radius(4.0),
                    bar(level, fill),
                ])
                .size(None, Some(8.0)),
            ]),
            mark,
        ])
        .size(None, Some(16.0))
        .flex(1.0),
        gap(INSET, 0.0),
    ])
    .size(Some(width), Some(16.0))
    .accessible(
        limn::Access::new(limn::Role::ProgressIndicator)
            .name(t("Nivel del micrófono"))
            .value(if heard { level } else { 0.0 }),
    )
}

/// What the preview of the accessibility settings shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    /// The account, who writes the sample messages.
    pub name: String,
    pub ink: Color,
    pub accent: Color,
    pub style: MessageStyle,
    pub now: Timestamp,
}

/// Two sample messages as the timeline would show them with the chosen
/// settings, three people with their status, and a sample button.
pub fn preview(preview: &Preview, width: f32, look: &Look) -> Element {
    let sample = |id: u64, content: &str, reactions: Vec<Reaction>| Message {
        id: MessageId(u64::MAX - id),
        channel: ChannelId(0),
        author: UserId(0),
        content: content.to_owned(),
        timestamp: preview.now,
        edited: None,
        kind: MessageKind::Default,
        reply_to: None,
        mentions: Vec::new(),
        reactions,
        attachments: Vec::new(),
        stickers: Vec::new(),
        text_references: Vec::new(),
        delivery: Delivery::Sent,
        pinned: false,
        embeds: Vec::new(),
        forwarded: None,
        poll: None,
        components: Default::default(),
        call: None,
        mention_roles: Vec::new(),
        mention_everyone: false,
    };
    let first = sample(
        1,
        t("¿qué pasó con todos los porotos?"),
        vec![
            Reaction {
                emoji: "🦀".into(),
                count: 3,
                me: true,
                ..Default::default()
            },
            Reaction {
                emoji: "👀".into(),
                count: 1,
                me: false,
                ..Default::default()
            },
        ],
    );
    let second = sample(
        2,
        t("aquí tienes un enlace: https://example.org/rustcord"),
        Vec::new(),
    );
    let messages_width = (width - 120.0).max(160.0);
    let row = |message: &Message| {
        message_row(
            MessageView {
                message,
                author: Author {
                    marks: Default::default(),
                    image: None,
                    name: preview.name.clone(),
                    ink: preview.ink,
                    accent: preview.accent,
                    bot: false,
                },
                continued: false,
                day: None,
                first_new: false,
                start: None,
                reply: None,
                mentions_me: false,
                abilities: Abilities::default(),
                reacting: false,
                author_card: None,
                width: messages_width,
                now: preview.now,
                style: MessageStyle {
                    sample: true,
                    ..preview.style
                },
            },
            look,
        )
    };
    let people = [
        ("Ada", look.online, look.online),
        ("Brisa", look.idle, look.idle),
        ("Cata", look.busy, look.busy),
    ]
    .into_iter()
    .map(|(name, accent, status)| {
        with_status(avatar(name, accent, 32.0), 32.0, status, look.canvas)
    })
    .collect();
    let button = Element::panel(
        Element::text(t("Botón de ejemplo"))
            .font_family(FontFamily::SansMedium)
            .font_size(14.0)
            .foreground(look.on_accent)
            .center_content(),
    )
    .padding(8.0)
    .radius(6.0)
    .background(look.accent);
    let side = Element::column(vec![
        Element::row(vec![
            Element::spacer().flex(1.0),
            Element::column(people).gap(8.0),
        ]),
        Element::spacer().flex(1.0),
        button,
    ]);
    Element::column(vec![
        line(t("Previsualizar"), 16.0, look.muted),
        gap(0.0, 10.0),
        Element::panel(
            Element::row(vec![
                Element::column(vec![row(&first), row(&second)]).size(Some(messages_width), None),
                side.flex(1.0),
            ])
            .padding(12.0),
        )
        .size(Some(width), Some(PREVIEW_HEIGHT))
        .border(look.border, 1.0)
        .radius(8.0)
        .clip_children(),
        gap(0.0, 16.0),
    ])
    .accessible(limn::Access::new(limn::Role::Group).name(t("Previsualizar")))
}

/// Discord's secondary control: a light veil over the page, 12 % and 16 %
/// under the pointer. Its palette is for a secondary button of a settings page.
pub fn veil(look: &Look) -> ComponentPalette {
    ComponentPalette {
        raised: Look::alpha(look.text, 31),
        hover: Look::alpha(look.text, 41),
        selected: Look::alpha(look.text, 41),
        ..look.palette()
    }
}

/// A button of the settings, quiet on the page's ground: 38 px tall and
/// never narrower than 100.
pub fn secondary_button(id: &str, label: &str, enabled: bool, look: &Look) -> Element {
    Button::new(id, label, veil(look))
        .variant(ButtonVariant::Secondary)
        .size(ControlSize::Spacious)
        .enabled(enabled)
        .render()
        .size(None, Some(BUTTON_HEIGHT))
        .min_size(100.0, 0.0)
}

/// The button that does what a section is for, in the accent.
pub fn primary_button(id: &str, label: &str, enabled: bool, look: &Look) -> Element {
    Button::new(id, label, look.accent_palette())
        .size(ControlSize::Spacious)
        .enabled(enabled)
        .render()
        .size(None, Some(BUTTON_HEIGHT))
        .min_size(100.0, 0.0)
}

/// A settings action that undoes or resets, as Discord's «Reiniciar»: the
/// quiet button of the page, its label in the danger color.
pub fn destructive_button(id: &str, label: &str, enabled: bool, look: &Look) -> Element {
    Button::new(id, label, veil(look))
        .variant(ButtonVariant::Secondary)
        .size(ControlSize::Spacious)
        .ink(look.danger)
        .enabled(enabled)
        .render()
        .size(None, Some(FIELD_HEIGHT))
        .min_size(100.0, 0.0)
}

/// A button that destroys something, in the danger color.
pub fn danger_button(id: &str, label: &str, look: &Look) -> Element {
    Button::new(id, label, look.palette())
        .variant(ButtonVariant::Destructive)
        .size(ControlSize::Spacious)
        .render()
        .size(None, Some(BUTTON_HEIGHT))
        .min_size(100.0, 0.0)
}

/// An account elsewhere, as Discord's connections list them: a card with
/// the service's 32 px logo and the name there over the service's.
pub fn connection(service: &str, name: &str, label: &str, look: &Look) -> Element {
    Element::panel(
        Element::row(vec![
            crate::identity::service_glyph(service, 32.0).centered_y(),
            Element::column(vec![
                line(name, 16.0, look.text).font_family(FontFamily::SansSemibold),
                line(label, 12.0, look.muted),
            ])
            .gap(2.0)
            .flex(1.0)
            .centered_y(),
        ])
        .gap(16.0)
        .padding(16.0),
    )
    .background(look.raised)
    .border(look.border, 1.0)
    .radius(8.0)
    .accessible(limn::Access::new(limn::Role::Group).name(format!("{name}, {label}")))
}

/// A [`setting`] whose note is laid out as given, such as text with a link
/// and words in bold, as Discord's «Mostrar los avatares de los usuarios».
pub fn setting_with_note(
    label: &str,
    note: Element,
    control: Option<Element>,
    enabled: bool,
    look: &Look,
) -> Element {
    let ink = if enabled { look.text } else { look.muted };
    let mut row = vec![Element::column(vec![
        text(label, 16.0, ink).font_family(FontFamily::SansMedium),
        note.max_width(560.0),
    ])
    .gap(4.0)
    .flex(1.0)
    .centered_y()];
    row.extend(control);
    Element::column(vec![
        gap(0.0, 8.0),
        Element::row(row).gap(24.0).min_size(0.0, 32.0),
        gap(0.0, 8.0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};

    #[test]
    fn the_column_takes_what_the_sidebar_leaves_up_to_its_width() {
        let wide = Geometry::new(Size::new(1361.0, 797.0));
        assert_eq!(wide.panel, Size::new(1281.0, 717.0));
        assert_eq!(wide.column, COLUMN_WIDTH);
        assert!(!wide.full);
        let narrow = Geometry::new(Size::new(720.0, 600.0));
        assert!(narrow.full);
        assert_eq!(narrow.panel.width, 720.0);
        assert!(narrow.column < COLUMN_WIDTH && narrow.column >= 200.0);
    }

    #[test]
    fn the_margin_is_40_then_72_and_a_window_up_to_1080_is_the_panel() {
        // Up to 1080 wide the panel fills the window: no margin, radius or border.
        let full = Geometry::new(Size::new(1080.0, 700.0));
        assert!(full.full);
        assert_eq!(full.panel, Size::new(1080.0, 700.0));
        // Wider, 40 px at each side up to 1600 wide...
        let roomy = Geometry::new(Size::new(1081.0, 700.0));
        assert!(!roomy.full);
        assert_eq!(roomy.panel, Size::new(1001.0, 620.0));
        assert_eq!(Geometry::new(Size::new(1400.0, 900.0)).panel.width, 1320.0);
        // ...then 72 px, and the panel stops at 1400 wide.
        assert_eq!(
            Geometry::new(Size::new(1601.0, 900.0)).panel,
            Size::new(1400.0, 756.0)
        );
        assert_eq!(Geometry::new(Size::new(1920.0, 1000.0)).panel.width, 1400.0);
    }

    #[test]
    fn a_ticked_slider_names_every_mark_and_snaps_to_them() {
        let look = Look::DARK;
        let labels = ["12px", "14px", "16px"];
        let (_, laid) = frame(
            ticked_slider("s", "Tamaño", &labels, 2, 1, 400.0, &look),
            Size::new(400.0, 60.0),
        );
        let painted = texts(&laid);
        for label in labels {
            assert!(painted.contains(&label.to_owned()), "{painted:?}");
        }
        assert!(controls(&laid).contains(&"s".to_owned()));
        assert_eq!(mark_at(0.0, 3), 0);
        assert_eq!(mark_at(0.74, 3), 1);
        assert_eq!(mark_at(1.0, 3), 2);
        assert_eq!(mark_at(2.0, 3), 2);
    }

    #[test]
    fn a_section_stacks_its_rows_without_hairlines_between_them() {
        let look = Look::DARK;
        let row = || Element::spacer().size(Some(200.0), Some(40.0));
        let (_, laid) = frame(
            section("s", "Título", None, None, vec![row(), row(), row()], &look),
            Size::new(400.0, 300.0),
        );
        let rules = laid
            .commands
            .iter()
            .filter(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, color, .. }
                    if *color == look.border && rect.height == 1.0)
            })
            .count();
        assert_eq!(rules, 0, "the rows have no hairlines between them");
    }

    #[test]
    fn choices_are_plain_and_a_note_repeating_its_name_is_left_out() {
        let look = Look::DARK;
        let options = [
            ("a".to_owned(), "Uno", None),
            ("b".to_owned(), "Dos", Some("Dos")),
            ("c".to_owned(), "Tres", Some("Una nota")),
        ];
        let (_, laid) = frame(
            choices("c", "Elegir", &options, "a", &look),
            Size::new(400.0, 300.0),
        );
        let painted = texts(&laid);
        assert!(painted.iter().any(|text| text == "Una nota"));
        assert_eq!(
            painted.iter().filter(|text| *text == "Dos").count(),
            1,
            "the name alone: {painted:?}"
        );
        // A plain list sits on the page: no option is drawn as a card.
        let raised = laid
            .commands
            .iter()
            .filter(|command| {
                matches!(command, limn::DrawCommand::FillRect { color, .. } if *color == look.raised)
            })
            .count();
        assert_eq!(raised, 0, "no card behind the options");
    }

    #[test]
    fn the_preview_shows_messages_that_do_nothing_when_clicked() {
        let look = Look::DARK;
        let (_, laid) = frame(
            preview(
                &Preview {
                    name: "Invitado".into(),
                    ink: look.accent,
                    accent: look.accent,
                    style: MessageStyle::default(),
                    now: Timestamp(1_790_000_000_000),
                },
                743.0,
                &look,
            ),
            Size::new(743.0, 260.0),
        );
        let painted = texts(&laid);
        assert!(painted.iter().any(|text| text.contains("porotos")));
        assert!(
            controls(&laid).is_empty(),
            "no reactions, links or menus: {:?}",
            controls(&laid)
        );
    }

    fn account(look: &Look) -> Account {
        Account {
            image: None,
            name: "Invitado".into(),
            accent: look.accent,
            edit: Some("profile:edit".into()),
            selected: false,
        }
    }

    #[test]
    fn a_long_page_name_wraps_and_its_line_grows_as_discord_s() {
        let look = Look::DARK;
        let page = |id: &str, label: &str| {
            Nav::Page(NavPage {
                id: id.into(),
                label: label.into(),
                icon: Icon::Gamepad,
                selected: false,
                anchors: Vec::new(),
            })
        };
        let (_, laid) = frame(
            sidebar(
                &account(&look),
                search_box("q", "", &look),
                &[
                    page("short", "Sistema"),
                    page("long", "Privacidad de la actividad"),
                ],
                "Rustcord",
                &look,
            ),
            Size::new(SIDEBAR_WIDTH, 700.0),
        );
        let rect = |id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap()
                .rect
        };
        assert_eq!(rect("short").height, ITEM_HEIGHT);
        let long = rect("long").height;
        assert!((long - 56.0).abs() < 2.0, "two rows of text: {long}");
        assert!(texts(&laid).concat().contains("Privacidad de la actividad"));
    }

    #[test]
    fn a_people_card_names_its_list_and_parts_its_people_by_lines() {
        let look = Look::DARK;
        let row = |name: &str| {
            person(
                name,
                "handle",
                None,
                look.accent,
                secondary_button(&format!("{name}:undo"), "Desbloquear", true, &look),
                &look,
            )
        };
        let (_, laid) = frame(
            people_card(
                Icon::Custom(BLOCK),
                "Cuentas bloqueadas",
                "2 cuentas",
                vec![row("Ana"), row("Beto")],
                &look,
            ),
            Size::new(COLUMN_WIDTH, 400.0),
        );
        let painted = texts(&laid);
        for text in ["Cuentas bloqueadas", "2 cuentas", "Ana", "Beto"] {
            assert!(painted.iter().any(|shown| shown == text), "{text}");
        }
        assert_eq!(controls(&laid), ["Ana:undo", "Beto:undo"]);
        // One line between the two people; none over the first or under the last.
        let lines = laid
            .commands
            .iter()
            .filter(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, .. } if rect.height == 1.0)
            })
            .count();
        assert_eq!(lines, 1);
    }

    #[test]
    fn the_sidebar_shows_the_open_page_s_sections_only() {
        let look = Look::DARK;
        let page = |id: &str, selected: bool| {
            Nav::Page(NavPage {
                id: id.into(),
                label: id.into(),
                icon: Icon::Palette,
                selected,
                anchors: vec![NavAnchor {
                    id: format!("{id}:a"),
                    label: format!("{id} a"),
                    active: true,
                }],
            })
        };
        let element = sidebar(
            &account(&look),
            search_box("q", "", &look),
            &[page("uno", true), Nav::Separator, page("dos", false)],
            "Rustcord",
            &look,
        );
        let (_, laid) = frame(element, Size::new(SIDEBAR_WIDTH, 700.0));
        let ids = controls(&laid);
        assert!(ids.contains(&"uno:a".to_owned()) && !ids.contains(&"dos:a".to_owned()));
        assert!(ids.contains(&"profile:edit".to_owned()));
        assert!(!ids.contains(&"q:clear".to_owned()), "nothing to clear");
        // Who the settings are of is a row of the menu: the same inset,
        // width and radius as «uno».
        let rect = |id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap()
                .rect
        };
        let (edit, uno) = (rect("profile:edit"), rect("uno"));
        assert_eq!((edit.x, edit.width), (uno.x, uno.width));
        let rounded = |at: limn::Rect, corner: f32| {
            laid.commands.iter().any(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, radius, .. }
                    if *rect == at && *radius == corner)
            })
        };
        assert!(rounded(uno, 8.0), "the selected page's fill");
        let mut tree = limn::UiTree::new(sidebar(
            &account(&look),
            search_box("q", "", &look),
            &[page("uno", true)],
            "Rustcord",
            &look,
        ));
        tree.set_reduced_motion(true);
        tree.layout(Size::new(SIDEBAR_WIDTH, 700.0));
        tree.handle_event(limn::UiEvent::PointerMove(limn::Point::new(
            edit.x + edit.width - 2.0,
            edit.y + edit.height / 2.0,
        )));
        let hovered = tree.layout_at(
            Size::new(SIDEBAR_WIDTH, 700.0),
            std::time::Instant::now() + std::time::Duration::from_secs(2),
        );
        assert!(
            hovered.commands.iter().any(|command| matches!(command,
                limn::DrawCommand::FillRect { rect, radius, .. } if *rect == edit && *radius == 8.0)),
            "its hover fills the whole row, rounded"
        );
        // The avatar, 48 px, and the two lines are centered on each other.
        let avatar = hovered
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::FillRect { rect, radius, .. } if *radius == AVATAR / 2.0 => {
                    Some(*rect)
                }
                _ => None,
            })
            .unwrap();
        let middle = |rect: limn::Rect| rect.y + rect.height / 2.0;
        assert_eq!(middle(avatar), middle(edit));
    }

    #[test]
    fn the_links_of_the_open_page_run_a_track_and_its_thumb_is_on_the_current_one() {
        let look = Look::DARK;
        let links = Nav::Page(NavPage {
            id: "uno".into(),
            label: "uno".into(),
            icon: Icon::Palette,
            selected: true,
            anchors: (0..3)
                .map(|index| NavAnchor {
                    id: format!("uno:{index}"),
                    label: format!("Sección {index}"),
                    active: index == 1,
                })
                .collect(),
        });
        let (_, laid) = frame(
            sidebar(
                &account(&look),
                search_box("q", "", &look),
                &[links],
                "Rustcord",
                &look,
            ),
            Size::new(SIDEBAR_WIDTH, 700.0),
        );
        let stripes = |color: Color| -> Vec<limn::Rect> {
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::FillRect {
                        rect, color: fill, ..
                    } if *fill == color && rect.width == TRACK => Some(*rect),
                    _ => None,
                })
                .collect()
        };
        let thumbs = stripes(look.text);
        assert_eq!(
            thumbs.len(),
            1,
            "one thumb, on the current link: {thumbs:?}"
        );
        let track = stripes(look.selected);
        assert!(track.len() >= 3, "a track down the links: {track:?}");
        let thumb = thumbs[0];
        assert!(track.iter().all(|rect| rect.x == thumb.x));
        let current = laid
            .regions
            .iter()
            .find(|region| region.id == "uno:1")
            .unwrap()
            .rect;
        assert!(
            (thumb.y - current.y).abs() < 2.0,
            "the thumb starts at the current link's line: {thumb:?} vs {current:?}"
        );
    }

    #[test]
    fn a_search_that_finds_nothing_says_so_in_the_sidebar() {
        let look = Look::DARK;
        let (_, laid) = frame(
            sidebar(
                &account(&look),
                search_box("q", "zzz", &look),
                &[Nav::Empty],
                "Rustcord",
                &look,
            ),
            Size::new(SIDEBAR_WIDTH, 400.0),
        );
        // The lines are painted in pieces where they wrap: join them back.
        let painted = texts(&laid).concat();
        assert!(
            painted.contains("No se encontraron resultados"),
            "{painted:?}"
        );
        assert!(
            painted.contains("Lo intentamos, pero no encontramos nada."),
            "{painted:?}"
        );
        assert!(!painted.contains("Ningún ajuste"));
    }

    #[test]
    fn a_settings_search_field_is_40_tall() {
        let look = Look::DARK;
        // A root takes the whole window, so the field sits in a column as in the sidebar.
        let (_, laid) = frame(
            Element::column(vec![search_box("q", "", &look)]),
            Size::new(ITEM_WIDTH, 100.0),
        );
        let field = laid
            .regions
            .iter()
            .find(|region| region.id == "q")
            .unwrap()
            .rect;
        assert_eq!(field.height, 40.0, "{field:?}");
    }

    #[test]
    fn a_fold_row_has_its_square_button_at_the_end_that_says_if_it_is_open() {
        let look = Look::DARK;
        let (_, laid) = frame(
            fold_row("f", "Mostrar más", "Esto y aquello", false, &look),
            Size::new(400.0, 100.0),
        );
        assert_eq!(controls(&laid), ["f"]);
        let button = laid.regions.iter().find(|region| region.id == "f").unwrap();
        assert_eq!((button.rect.width, button.rect.height), (48.0, 48.0));
        assert_eq!(button.rect.x + button.rect.width, 400.0, "at the row's end");
        let said = |frame: &limn::Frame| {
            frame
                .access
                .iter()
                .find(|node| node.id == "f")
                .map(|node| node.expanded)
        };
        assert_eq!(said(&laid), Some(Some(false)));
        let (_, open) = frame(
            fold_row("f", "Mostrar más", "Esto y aquello", true, &look),
            Size::new(400.0, 100.0),
        );
        assert_eq!(said(&open), Some(Some(true)));
        assert!(texts(&open).contains(&"Esto y aquello".to_owned()));
    }

    #[test]
    fn a_reset_is_written_in_the_danger_color_while_it_can_be_used() {
        let look = Look::DARK;
        let inks = |enabled: bool| {
            let (_, laid) = frame(
                destructive_button("r", "Reiniciar", enabled, &look),
                Size::new(200.0, 60.0),
            );
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::Text { color, .. } => Some(*color),
                    _ => None,
                })
                .collect::<Vec<Color>>()
        };
        assert_eq!(inks(true), [look.danger]);
        assert!(
            !inks(false).contains(&look.danger),
            "a disabled one is muted"
        );
    }

    #[test]
    fn a_related_card_leads_to_its_page_with_its_glyph() {
        let look = Look::DARK;
        let (_, laid) = frame(
            related("us:page=x", Icon::Bell, "Notificaciones", "Una nota", &look),
            Size::new(400.0, 120.0),
        );
        assert_eq!(controls(&laid), ["us:page=x"]);
        let squares = laid
            .commands
            .iter()
            .filter(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, .. }
                    if rect.width == 48.0 && rect.height == 48.0)
            })
            .count();
        assert_eq!(squares, 1, "the glyph's square");
    }

    #[test]
    fn a_settings_switch_is_48_by_24() {
        let look = Look::DARK;
        let (_, laid) = frame(
            toggle("sw", "Enviar", None, true, None, &look),
            Size::new(400.0, 100.0),
        );
        let switch = laid
            .regions
            .iter()
            .find(|region| region.id == "sw")
            .unwrap()
            .rect;
        assert_eq!((switch.width, switch.height), (48.0, 24.0), "{switch:?}");
    }

    #[test]
    fn a_connection_is_a_card_of_its_logo_its_name_and_its_service() {
        let look = Look::DARK;
        let (_, laid) = frame(
            connection("github", "ada", "GitHub", &look),
            Size::new(696.0, 120.0),
        );
        let painted = texts(&laid);
        assert!(painted.contains(&"ada".to_owned()), "{painted:?}");
        assert!(painted.contains(&"GitHub".to_owned()), "{painted:?}");
        assert!(controls(&laid).is_empty(), "nothing to click");
        let logo = laid.commands.iter().any(|command| {
            matches!(command, limn::DrawCommand::FillRect { rect, .. }
                if rect.width == 32.0 && rect.height == 32.0)
        });
        assert!(logo, "the logo's 32 px square");
    }

    #[test]
    fn a_note_of_its_own_keeps_its_link_and_the_control_at_the_end() {
        let look = Look::DARK;
        let note = Element::formatted_text(vec![
            limn::TextSpan::new("Ve a ", look.muted, 14.0, FontFamily::Sans),
            limn::TextSpan::new("Accesibilidad", look.accent, 14.0, FontFamily::Sans).action("go"),
        ]);
        let (_, laid) = frame(
            setting_with_note(
                "Avatares",
                note,
                Some(switch("sw", true, &look).render()),
                true,
                &look,
            ),
            Size::new(400.0, 100.0),
        );
        assert!(
            laid.regions.iter().any(|region| region.id == "go"),
            "the link can be clicked"
        );
        let switch = laid
            .regions
            .iter()
            .find(|region| region.id == "sw")
            .unwrap()
            .rect;
        assert_eq!(switch.x + switch.width, 400.0, "at the row's end");
    }
}
