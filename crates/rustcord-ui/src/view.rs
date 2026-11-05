//! The whole window, from the state.

use limn::{Component, Element, ElementKind, Empty, Icon, ScrollArea, Size};
use rustcord_components::chrome;
use rustcord_components::gap;
use rustcord_components::members::{self, Group, MemberRow, Profile};
use rustcord_components::rail::{self, RailItem, RailVoice};
use rustcord_components::sidebar::{self, DirectRow};
use rustcord_components::{centered, line, Look};
use rustcord_i18n::{t, tr};
use rustcord_model::{ChannelKind, ListGroup, ListRow, ServerId, Timestamp, UserId};
use rustcord_state::Store;
use std::collections::HashMap;
use std::sync::Arc;

use crate::navigation::{region, CHANNELS, COMPOSER, MEMBERS, RAIL};
use crate::{
    channel_list, channel_settings, inbox, messages, profile, search, server_menu, server_settings,
    settings, timeline, voice, Ui, ViewContext,
};

/// The narrowest the conversation gets beside the member list.
const CONVERSATION_MIN: f32 = 480.0;

/// How narrow the conversation gets when the person keeps the member list
/// open in narrow windows: room for a line of text.
const NARROW_CONVERSATION_MIN: f32 = 280.0;
/// Narrower windows fold some of the header's tools into a «más».
const NARROW_HEADER: f32 = 840.0;

/// Whether a window `width` wide folds the header's inbox and threads into
/// a «más», the title needing the room.
pub(crate) fn narrow_header(ui: &Ui, width: f32) -> bool {
    // The update's button takes a tool's room: 32 and 13 before it.
    let update = if crate::updates::header_stage(ui).is_some() {
        45.0
    } else {
        0.0
    };
    width - crate::title_bar::room(ui) - update < NARROW_HEADER
}

/// Below this width the member list gives its room to the conversation;
/// the panels are as wide as the person left them. `narrow`: the
/// width when the person keeps it in narrow windows.
fn members_from(ui: &Ui, narrow: bool) -> f32 {
    let conversation = if narrow {
        NARROW_CONVERSATION_MIN
    } else {
        CONVERSATION_MIN
    };
    rail::WIDTH + ui.prefs.sidebar_width + ui.prefs.members_width + conversation
}
/// From the conversation column's left and right edges to the composer.
const COMPOSER_MARGIN: f32 = 11.5;
/// A quiet strip outside the scroll viewport: message hover stops here.
const COMPOSER_CLEARANCE: f32 = 8.0;
/// Above the first row of the column of channels.
const LIST_TOP: f32 = 2.0;
/// The notices' list over the foot of the column of channels, and the one
/// in the window's corner while something floats over that column.
const NOTICES: &str = "notices";
const CORNER_NOTICES: &str = "notices:corner";
/// The notices take the account card's width, up to this.
const NOTICE_WIDTH: f32 = 360.0;
/// Between two notices, and between the lowest and the account's panel.
const NOTICE_GAP: f32 = 8.0;
/// From the window's foot to the lowest notice in the corner: over the
/// account's row, its edges and its margin, as over the column.
const CORNER_BOTTOM: f32 =
    chrome::USER_PANEL_HEIGHT + 2.0 + chrome::USER_PANEL_MARGIN.2 + NOTICE_GAP;

/// The window for the state.
pub fn view(ui: &Ui, store: &Store, context: &ViewContext) -> Element {
    // Every text written from here on, rows built during layout included.
    rustcord_i18n::set_thread(Some(ui.language()));
    let look = &ui.look();
    // Every hint and floating surface built from here on, in this look.
    limn::set_tokens(look.tokens());
    // Until the session is loaded: Rustcord's mark and the stage it is in.
    if crate::splash::visible(store) {
        let root = with_notices(
            crate::splash::view(ui, store, look, context),
            ui,
            look,
            context.window,
        );
        return crate::title_bar::overlay(ui, root, look);
    }
    if crate::auth::visible(store) {
        let root = crate::auth::view(ui, store, look, context)
            .size(Some(context.window.width), Some(context.window.height));
        return crate::title_bar::overlay(ui, with_notices(root, ui, look, context.window), look);
    }
    // Every time written from here on, rows built during layout included.
    rustcord_components::time::set_twelve_hour(ui.prefs.twelve_hour);
    let window = context.window;
    let mut ground = look.canvas;
    // A stream watched shown alone fills the window, full screen.
    if let Some(focused) = crate::video::focused_view(ui, store, window, context.now) {
        return with_notices(focused, ui, look, window);
    }
    let body = if let Some(settings) = channel_settings::view(ui, store, context) {
        settings
    } else if let Some(settings) = server_settings::view(ui, store, context) {
        settings
    } else {
        // The columns paint nothing while the
        // desktop shows through the window, over its one translucent ground.
        let (panes, backdrop) = ui.panes_look();
        ground = backdrop;
        let look = &panes;
        let main_width = (window.width - rail::WIDTH - ui.prefs.sidebar_width).max(240.0);
        // The account's panel, and the call above it, float over the foot of
        // the rail and the column.
        let panes = Element::row(vec![
            Element::column(vec![
                under_notices(
                    Element::row(vec![
                        region(
                            server_rail(ui, store, look, context.wall),
                            RAIL,
                            t("Servidores"),
                        ),
                        column(ui, store, look, context.wall),
                    ]),
                    ui,
                    look,
                    window,
                )
                .flex(1.0),
                account(ui, store, look, context.now, context.wall),
            ])
            .size(Some(rail::WIDTH + ui.prefs.sidebar_width), None)
            .background(look.sidebar),
            main(ui, store, context, look, main_width).flex(1.0),
        ]);
        // The column's width moves from its edge.
        Element::stack(vec![panes, crate::desktop::column_handle(ui, look)])
    };
    let mut root = Element::panel(body)
        .size(Some(window.width), Some(window.height))
        .background(ground);
    if ui.show_metrics && !context.metrics.is_empty() {
        root = Element::stack(vec![
            root,
            Element::column(vec![
                Element::spacer().flex(1.0),
                Element::row(vec![
                    Element::spacer().flex(1.0),
                    chrome::metrics_panel(&context.metrics, look),
                    Element::spacer().size(Some(16.0), None),
                ]),
                Element::spacer().size(Some(96.0), None),
            ]),
        ]);
    }
    let root = messages::overlays(ui, store, root, look, context);
    let root = crate::message_actions::overlays(ui, store, root, look, context);
    let root = search::view(ui, store, root, look, context);
    let root = inbox::view(ui, store, root, look, context);
    let root = settings::switcher(ui, store, root, look, window);
    let root = settings::modal(ui, store, root, look, context);
    let root = server_menu::dialogs(ui, store, root, look, context);
    let root = channel_list::overlays(ui, store, root, look, context);
    let root = profile::overlays(ui, store, root, look, context);
    let root = crate::accounts::overlay(ui, store, root, look);
    let root = voice::overlays(ui, store, root, window.height, look);
    let root = crate::person_menu::overlays(ui, store, root, window.height, look);
    let root = crate::streams::overlays(ui, store, root, window.height, look);
    // The stream floating over the conversation while the stage is away.
    let root = crate::pip::overlay(ui, store, root, window, look);
    // Private calls: a call coming in, over everything but verifications.
    let root = crate::private_calls::overlay(ui, store, root, look);
    let root = crate::relationships::overlay(ui, store, root, look);
    let root = crate::conversations::overlay(ui, store, root, window.height, look);
    let root = crate::threads::overlay(
        ui,
        store,
        root,
        look,
        context.wall,
        window.height,
        narrow_header(ui, window.width),
    );
    let root = crate::attachments::overlay(ui, store, root, look, context);
    let root = crate::voice_messages::overlay(ui, store, root, look, context);
    let root = crate::documents::overlay(ui, store, root, look, context);
    let root = crate::invitations::overlay(ui, store, root, look, context);
    let root = crate::onboarding::overlay(ui, store, root, look, context);
    let root = crate::events::overlay(ui, store, root, look, context);
    let root = crate::navigation::overlay(ui, root, look);
    let root = crate::account::overlay(ui, store, root, look, context);
    // An app's modal, under the verification it may need.
    let root = crate::interactions::overlay(ui, store, root, look, context);
    let root = crate::challenges::overlay(ui, store, root, look);
    // Before an update leaves the call.
    let root = crate::updates::overlay(ui, root, look);
    // The window's own buttons, over everything; a stream
    // shown alone, above, fills the window without them.
    crate::title_bar::overlay(ui, with_notices(root, ui, look, window), look)
}

/// The notices at the window's bottom left, oldest on top, `NOTICE_GAP`
/// apart: in their slot over the foot of the column of channels
/// ([`under_notices`]) while nothing floats over it, else in the window's
/// corner over everything, as high, so a dialog never hides them. Only the
/// newest three are kept (`Ui::notify`); a long one scrolls in the list.
fn with_notices(mut root: Element, ui: &Ui, look: &Look, window: Size) -> Element {
    if ui.notices.is_empty() && ui.save_as.is_none() {
        return root;
    }
    if let Some(slot) = notice_slot(&mut root) {
        let width = slot.style.width.unwrap_or(NOTICE_WIDTH);
        if let ElementKind::Scroll { child, .. } = &mut slot.kind {
            **child = Element::column(notice_cards(ui, look, width)).gap(NOTICE_GAP);
        }
        return root;
    }
    let cards = Element::column(notice_cards(ui, look, NOTICE_WIDTH)).gap(NOTICE_GAP);
    Element::overlay(
        root,
        Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![
                gap(chrome::USER_PANEL_MARGIN.0, 0.0),
                notice_viewport(CORNER_NOTICES, NOTICE_WIDTH, window.height, look, cards),
                Element::spacer().flex(1.0),
            ])
            .shrink_height_to_fit(),
            gap(0.0, CORNER_BOTTOM),
        ]),
    )
}

/// `channels`, the rail and the column of channels, with the notices' slot
/// over their foot, `NOTICE_GAP` above the account's panel and as wide as
/// its card, up to `NOTICE_WIDTH`. [`with_notices`] fills it.
fn under_notices(channels: Element, ui: &Ui, look: &Look, window: Size) -> Element {
    if ui.notices.is_empty() && ui.save_as.is_none() {
        return channels;
    }
    let (left, right, _) = chrome::USER_PANEL_MARGIN;
    let width = (rail::WIDTH + ui.prefs.sidebar_width - left - right).min(NOTICE_WIDTH);
    Element::stack(vec![
        channels,
        Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![
                gap(left, 0.0),
                notice_viewport(NOTICES, width, window.height, look, Element::spacer()),
                Element::spacer().flex(1.0),
            ])
            .shrink_height_to_fit(),
            gap(0.0, NOTICE_GAP),
        ]),
    ])
}

fn notice_slot(element: &mut Element) -> Option<&mut Element> {
    match &element.kind {
        ElementKind::Scroll { id, .. } if id == NOTICES => return Some(element),
        _ => {}
    }
    match &mut element.kind {
        ElementKind::Column(children)
        | ElementKind::Row(children)
        | ElementKind::Stack(children) => children.iter_mut().find_map(notice_slot),
        ElementKind::Panel(child) => notice_slot(child),
        // Never descend into a hidden base under a rendered modal or overlay.
        _ => None,
    }
}

fn notice_viewport(id: &str, width: f32, height: f32, look: &Look, cards: Element) -> Element {
    ScrollArea::new(id, cards)
        // Over the cards, which keep the whole width.
        .appearance(look.scrollbar(true))
        .render()
        .size(Some(width), None)
        // Short lists keep their natural height; long copy remains reachable.
        .max_height(height / 2.0)
}

/// A save under way, then the notices as Discord's toasts, the newest last.
fn notice_cards(ui: &Ui, look: &Look, width: f32) -> Vec<Element> {
    let saving = crate::save_as::status(ui, look).map(|status| status.size(Some(width), None));
    saving
        .into_iter()
        .chain(ui.notices.iter().map(|notice| {
            chrome::toast(
                &notice.title,
                notice.detail.as_deref(),
                notice.error,
                width,
                look,
            )
        }))
        .collect()
}

/// Before the provider is ready: what it is, and that it is connecting.
fn server_rail(ui: &Ui, store: &Store, look: &Look, wall: Timestamp) -> Element {
    let (home_unread, home_count) = crate::conversations::directs_unread(ui, store, wall);
    let home = RailItem {
        image: None,
        id: "home".into(),
        name: t("Mensajes directos").into(),
        initials: String::new(),
        accent: look.accent,
        glyph: Some(Icon::Custom(rustcord_components::MARK)),
        selected: ui.server.is_none(),
        unread: home_unread && ui.prefs.unread_badges,
        mentions: if ui.prefs.mention_badges {
            home_count
        } else {
            0
        },
        muted: false,
        voice: RailVoice::default(),
    };
    let voice = store.voice_by_server();
    let item = |server: &rustcord_model::Server| {
        // Their faces for the hint (their pictures load while it shows, see
        // `images`), in the rows the official client's has.
        let voice = crate::rail::voice(ui, store, server, voice.get(&server.id), look, wall);
        let waiting = channel_list::server_unread(ui, store, server.id, wall);
        let (unread, mentions, muted) =
            server_menu::rail_marks(ui, store, server.id, waiting, wall);
        let unread = unread && ui.prefs.unread_badges;
        let mentions = if ui.prefs.mention_badges { mentions } else { 0 };
        RailItem {
            image: ui.image(&server.icon),
            id: format!("server:{}", server.id),
            name: server.shown_name().into(),
            initials: server.initials(),
            accent: look.ink(server.accent),
            glyph: None,
            selected: ui.server == Some(server.id),
            unread,
            mentions,
            muted,
            voice,
        }
    };
    // In the account's folders, where the provider keeps them.
    let entries = crate::account::rail_entries(ui, store, look, &item);
    rail::server_rail(home, entries, &crate::rail::drag_marks(ui, store), look)
}

/// The column of channels.
fn column(ui: &Ui, store: &Store, look: &Look, wall: Timestamp) -> Element {
    let (header, rows) = match ui.server.and_then(|server| store.server(server)) {
        Some(server) => (
            server_menu::header(ui, store, server.id, look),
            channel_list::rows(ui, store, server.id, look, wall),
        ),
        None => (
            sidebar::direct_header(look),
            direct_rows(ui, store, look, wall),
        ),
    };
    let list = ScrollArea::new(
        "channels",
        Element::column(vec![
            gap(0.0, LIST_TOP),
            Element::column(rows).gap(sidebar::ROW_GAP),
            gap(0.0, 8.0),
        ]),
    )
    .appearance(look.scrollbar(true))
    .render()
    .flex(1.0);
    let list = region(list, CHANNELS, t("Canales"));
    let mut column = vec![header, separator(look), list];
    // Simulated data says so at the foot of the column.
    if let Some(origin) = crate::providers::origin_of(store) {
        column.push(chrome::provider_note(&origin, look));
    }
    Element::row(vec![Element::column(column).flex(1.0), divider(look)])
        .size(Some(ui.prefs.sidebar_width), None)
        .background(look.sidebar)
}

/// The voice connection and the account, under the rail and the column: one
/// box, its sections divided by hairlines.
fn account(
    ui: &Ui,
    store: &Store,
    look: &Look,
    now: std::time::Instant,
    wall: Timestamp,
) -> Element {
    let mut parts = Vec::new();
    parts.extend(crate::stage::panel(store, look));
    parts.extend(voice::panel(ui, store, look, now, wall));
    parts.push(profile::user_panel(ui, store, look, wall));
    let mut sections = Vec::new();
    for (index, part) in parts.into_iter().enumerate() {
        if index > 0 {
            sections.push(
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(chrome::USER_PANEL_EDGE),
            );
        }
        sections.push(part);
    }
    // As measured on Discord's, the sections sit inside the 1 px edge at
    // the top, bottom and right, but start under it at the left.
    let card = Element::panel(Element::column(vec![
        gap(0.0, 1.0),
        Element::row(vec![Element::column(sections).flex(1.0), gap(1.0, 0.0)]),
        gap(0.0, 1.0),
    ]))
    .radius(8.0)
    .background(look.rail)
    .border(chrome::USER_PANEL_EDGE, 1.0)
    .clip_children();
    let (left, right, bottom) = chrome::USER_PANEL_MARGIN;
    // The dividers of the rail and the column run on behind the panel.
    let dividers = Element::row(vec![
        gap(rail::WIDTH - 1.0, 0.0),
        divider(look),
        Element::spacer().flex(1.0),
        divider(look),
    ]);
    Element::stack(vec![
        dividers,
        Element::column(vec![
            Element::row(vec![gap(left, 0.0), card.flex(1.0), gap(right, 0.0)]),
            gap(0.0, bottom),
        ]),
    ])
}

/// The hairline under a header.
fn separator(look: &Look) -> Element {
    Element::panel(Element::spacer())
        .size(None, Some(1.0))
        .background(look.border)
}

/// The hairline between two columns.
fn divider(look: &Look) -> Element {
    Element::panel(Element::spacer())
        .size(Some(1.0), None)
        .background(look.border)
}

fn direct_rows(ui: &Ui, store: &Store, look: &Look, wall: Timestamp) -> Vec<Element> {
    let friends = crate::relationships::visible(ui);
    let selected = ui.channel().filter(|_| !friends);
    let show_presence = store.friends_presence();
    let pending = store.pending_requests();
    let mut rows = vec![
        rustcord_components::friends::home_entry(friends, pending, look),
        sidebar::section_divider(look),
    ];
    // The pinned ones on top, under their own heading.
    let (pinned, rest) = crate::desktop::split_pinned(ui, store.directs());
    if !pinned.is_empty() {
        rows.push(sidebar::heading(t("Fijados"), look));
    }
    let heading = if store.create_group().allowed() {
        sidebar::heading_with(
            t("Mensajes directos"),
            (rustcord_components::group::ids::NEW, t("Crear grupo")),
            look,
        )
    } else {
        sidebar::heading(t("Mensajes directos"), look)
    };
    let count = pinned.len();
    let mut heading = Some(heading);
    for (at, channel) in pinned.into_iter().chain(rest).enumerate() {
        if at == count {
            rows.extend(heading.take());
        }
        let group = channel.kind == ChannelKind::Group;
        let other = channel.recipients.first().copied();
        let presence = other
            .filter(|_| show_presence && !group)
            .map(|user| store.presence(user));
        let accent = if group {
            look.accent
        } else {
            other
                .and_then(|user| store.user(user))
                .map_or(look.faint, |user| look.ink(user.accent))
        };
        // What they do, as the member list says it, else their status.
        let ignored = other.is_some_and(|user| store.is_ignored(user));
        let (line, emoji) = presence
            .as_ref()
            .map(|presence| crate::identity::status_line(ui, presence, ignored))
            .unwrap_or_default();
        rows.push(sidebar::direct(
            DirectRow {
                image: if group {
                    ui.image(&channel.icon)
                } else {
                    other
                        .and_then(|id| store.user(id))
                        .and_then(|user| ui.picture(user.picture()))
                },
                id: format!("channel:{}", channel.id),
                title: store.channel_title(channel.id),
                subtitle: if group {
                    Some(tr!(
                        "{count} miembros",
                        count = channel.recipients.len() + 1
                    ))
                } else {
                    line
                },
                subtitle_emoji: emoji,
                accent,
                status: presence.map(|presence| look.status(presence.status)),
                group,
                // Discord's ×: a group's leaves it, a conversation's closes it.
                close: if group {
                    store
                        .group_abilities(channel.id)
                        .filter(|can| can.leave.allowed())
                        .map(|_| {
                            (
                                format!("group:{}:x-leave", channel.id),
                                t("Abandonar grupo").to_owned(),
                            )
                        })
                } else {
                    store.close_direct(channel.id).allowed().then(|| {
                        (
                            format!("group:{}:x-close", channel.id),
                            t("Cerrar conversación").to_owned(),
                        )
                    })
                },
                selected: selected == Some(channel.id),
                // Muted, it waits without saying so.
                unread: store.read(channel.id).unread()
                    && !crate::conversations::muted(ui, store, channel.id, wall),
            },
            look,
        ));
    }
    // Nothing but pinned ones: the heading still says where they go.
    rows.extend(heading);
    rows
}

/// The open conversation: its header, the line of the connection, the
/// timeline, the composer, and the member list beside them.
fn main(ui: &Ui, store: &Store, context: &ViewContext, look: &Look, width: f32) -> Element {
    if let Some(stage) = voice::stage_view(ui, store, context, look, width) {
        // The channel's chat is the open conversation, beside the stage; its
        // width moves from its left edge.
        return match voice::chat_width(ui, store, width) {
            Some(chat) => Element::row(vec![
                stage.flex(1.0),
                Element::panel(Element::spacer())
                    .size(Some(1.0), None)
                    .background(look.border),
                Element::stack(vec![
                    conversation(ui, store, context, look, chat - 1.0, false),
                    crate::desktop::call_chat_handle(look),
                ])
                .size(Some(chat - 1.0), None),
            ]),
            None => stage,
        };
    }
    conversation(ui, store, context, look, width, true)
}

/// The open conversation in `width`, with the member list beside it when
/// `members` and the window has room.
fn conversation(
    ui: &Ui,
    store: &Store,
    context: &ViewContext,
    look: &Look,
    width: f32,
    members: bool,
) -> Element {
    if crate::relationships::visible(ui) {
        return over_conversation(
            crate::relationships::view(ui, store, look, context, width).flex(1.0),
            ui,
            store,
            look,
            context,
        );
    }
    let Some(channel) = ui
        .channel()
        .filter(|channel| store.channel(*channel).is_some())
    else {
        let available = match ui.server {
            Some(server) => store.first_text_channel(server).is_some(),
            None => !store.directs().is_empty(),
        };
        let (title, detail) = if available {
            (
                t("Nada abierto"),
                t("Elige un canal o una conversación de la columna."),
            )
        } else if ui.server.is_some() {
            (
                t("Sin canales cargados"),
                t("No hay canales cargados para este servidor."),
            )
        } else {
            (
                t("Tus conversaciones"),
                t("No hay conversaciones cargadas en esta sesión."),
            )
        };
        return over_conversation(
            centered(
                Empty::new(title, detail, look.palette())
                    .icon(Icon::Chat)
                    .render(),
            )
            .flex(1.0),
            ui,
            store,
            look,
            context,
        );
    };
    let conversation = store.channel(channel).expect("checked above");
    let server = conversation.server;
    let wide = members && context.window.width >= members_from(ui, false);
    // Kept in narrow windows if the person asks, down to a line's room.
    let room = members && context.window.width >= members_from(ui, ui.prefs.members_narrow);
    let can_show_members = server.is_some() && room && crate::has_member_list(store);
    let show_members = can_show_members && ui.members_open;
    // A direct conversation's profile beside it, in its own width.
    let side_room = members
        && conversation.kind == ChannelKind::Direct
        && context.window.width
            >= members_from(ui, ui.prefs.members_narrow) - ui.prefs.members_width
                + rustcord_components::profile::SIDE_WIDTH;
    let side = side_room
        .then(|| profile::side_panel(ui, store, look))
        .flatten();
    let content_width = width
        - if show_members {
            ui.prefs.members_width
        } else if side.is_some() {
            rustcord_components::profile::SIDE_WIDTH
        } else {
            0.0
        };
    let glyph = if server.is_some() {
        channel_list::glyph(conversation.kind).unwrap_or(if conversation.kind.is_voice() {
            Icon::Volume
        } else {
            Icon::Hash
        })
    } else {
        Icon::At
    };
    // Below 800 px the title needs the room: the inbox and the list of
    // threads fold into a «más».
    // The window's own buttons take the header's end.
    let narrow = narrow_header(ui, context.window.width);
    // In Discord's order: the threads, the bell, the pinned messages.
    let mut tools: Vec<Element> = Vec::new();
    if !narrow {
        tools.extend(crate::threads::header_tool(
            ui,
            store,
            channel,
            look,
            context.wall,
            context.window.width,
        ));
    }
    tools.extend(channel_list::header_tools(
        ui,
        store,
        channel,
        look,
        context.wall,
    ));
    tools.extend(crate::conversations::header_tool(store, channel, look));
    // Private calls: to call, answer or join.
    tools.extend(crate::private_calls::header_tool(store, channel, look));
    if narrow {
        tools.extend(channel_list::header_more(ui, store, channel, look));
    }
    // Last, where a server's member list has its toggle.
    if side_room {
        tools.push(chrome::profile_toggle(!ui.profiles.side_hidden, look));
    }
    // A thread names what it hangs from before its own name.
    let parent = conversation
        .parent
        .filter(|_| conversation.is_thread())
        .and_then(|parent| store.channel(parent))
        .map(|parent| {
            (
                channel_list::glyph(parent.kind).unwrap_or(Icon::Hash),
                parent.name.as_str(),
            )
        });
    let title = store.channel_title(channel);
    // A group shows its own picture, or its mark, where a channel has a glyph.
    let face =
        (conversation.kind == ChannelKind::Group).then(|| match ui.image(&conversation.icon) {
            Some(image) => {
                rustcord_components::avatar_with_image(&title, look.accent, 24.0, Some(image))
            }
            None => rustcord_components::sidebar::group_mark(look.accent, 24.0),
        });
    let topic_pictures = crate::identity::text_pictures(ui, &conversation.topic_pictures);
    let header = chrome::channel_header(
        glyph,
        &title,
        conversation
            .topic
            .as_deref()
            .map(|topic| (topic, topic_pictures.as_slice())),
        tools,
        // Only offer a list the provider can supply and the window can fit.
        can_show_members.then_some(ui.members_open),
        // The search is a box in a window with room for the member list.
        wide,
        parent,
        face,
        &crate::navigation::header(ui, store, narrow),
        inbox::popout(ui, store, look, context),
        look,
    );
    if conversation.kind == ChannelKind::Forum {
        let mut column = vec![header, separator(look)];
        let mut row = vec![crate::threads::forum_page(
            ui,
            store,
            conversation,
            content_width,
            context.wall,
            look,
        )
        .flex(1.0)];
        if let (true, Some(server)) = (show_members, server) {
            row.push(member_list(ui, store, server, look));
        }
        column.push(over_conversation(
            Element::row(row).flex(1.0),
            ui,
            store,
            look,
            context,
        ));
        return Element::column(column);
    }
    let mut stack = vec![region(
        timeline::timeline(
            ui,
            store,
            channel,
            content_width,
            context.window.height,
            context.wall,
        ),
        &crate::timeline_id(channel),
        t("Mensajes"),
    )];
    if let Some(bar) = messages::unread_bar(ui, store, channel, look) {
        stack.push(Element::column(vec![
            Element::row(vec![
                Element::spacer().size(Some(8.0), None),
                bar.flex(1.0),
                Element::spacer().size(Some(16.0), None),
            ]),
            Element::spacer().flex(1.0),
        ]));
    }
    if context.loading_older {
        stack.push(Element::column(vec![
            timeline::loading_older(look),
            Element::spacer().flex(1.0),
        ]));
    }
    if store.timeline(channel).is_some_and(|timeline| {
        timeline.context_source().is_some()
            || !timeline.is_empty() && (!timeline.latest || !context.at_end)
    }) {
        stack.push(Element::column(vec![
            Element::spacer().flex(1.0),
            chrome::jump_to_latest(0, look),
            Element::spacer().size(None, Some(12.0)),
        ]));
    }
    let mut conversation_column = vec![Element::stack(stack).flex(1.0)];
    if let Some(status) = crate::reads::status(store, channel, look) {
        conversation_column.insert(0, status);
    }
    if let Some(timeline) = store.timeline(channel) {
        let scope = match timeline.context_source() {
            Some(rustcord_model::HistorySource::Cached) => {
                Some(t("Contexto guardado en este equipo · puede haber huecos"))
            }
            Some(rustcord_model::HistorySource::Provider) => {
                Some(t("Estás viendo mensajes antiguos"))
            }
            None if !timeline.latest && !timeline.is_empty() => Some(if context.loading_latest {
                t("Actualizando recientes…")
            } else {
                t("Vista provisional de recientes · falta actualizar")
            }),
            None => None,
        };
        if let Some(scope) = scope {
            conversation_column.insert(0, line(scope, 12.0, look.muted).padding(6.0));
        }
    }
    if ui.search.is_none() {
        if let Some(status) = crate::context::status(ui, look) {
            conversation_column.insert(0, status.padding(8.0));
        }
    }
    if let Some(slowmode) = channel_list::slowmode(store, channel, look) {
        conversation_column.push(slowmode);
    }
    conversation_column.push(gap(0.0, COMPOSER_CLEARANCE));
    conversation_column.push(Element::row(vec![
        gap(COMPOSER_MARGIN, 0.0),
        region(
            messages::composer(
                ui,
                store,
                channel,
                content_width - 2.0 * COMPOSER_MARGIN,
                context,
                look,
            ),
            COMPOSER,
            t("Escribir un mensaje"),
        )
        .flex(1.0),
        gap(COMPOSER_MARGIN, 0.0),
    ]));
    let mut row = vec![Element::column(conversation_column).flex(1.0)];
    if let (true, Some(server)) = (show_members, server) {
        row.push(member_list(ui, store, server, look));
    }
    row.extend(side.map(|side| region(side, MEMBERS, t("Perfil de usuario"))));
    let mut column = vec![header, separator(look)];
    // Private calls: the conversation's call, under its header.
    if let Some(call) = crate::private_calls::call_view(ui, store, context, look, width) {
        column.push(call);
        column.push(separator(look));
    }
    column.push(over_conversation(
        Element::row(row).flex(1.0),
        ui,
        store,
        look,
        context,
    ));
    Element::column(column)
}

/// `content` with the connection's pill laid over its top, below the header,
/// while the pill says something. The pill takes no room, so nothing moves.
fn over_conversation(
    content: Element,
    ui: &Ui,
    store: &Store,
    look: &Look,
    context: &ViewContext,
) -> Element {
    let Some(pill) = crate::connection::view(ui, store, look, context.now) else {
        return content;
    };
    let overlay = Element::column(vec![
        gap(0.0, 8.0),
        Element::row(vec![
            Element::spacer().flex(1.0),
            pill,
            Element::spacer().flex(1.0),
        ]),
        Element::spacer().flex(1.0),
    ]);
    Element::stack(vec![content, overlay]).flex(1.0)
}

/// One row of the member list: a heading, a person and their card if open,
/// or someone the provider has not sent yet.
enum MemberListRow {
    Heading(Group),
    Person(Box<MemberRow>, Option<Box<Profile>>),
    Loading,
}

/// Rows of a provider's member list shown, at most. shortcut: limn's
/// measured list keys every row on each layout; lift this when it can
/// place rows of their own heights by what is in view alone.
const LIST_SHOWN: u32 = 50_000;

/// The title of a provider's group: its role's name, or around or not.
pub(crate) fn group_title(store: &Store, server: ServerId, group: ListGroup) -> String {
    match group {
        ListGroup::Online => t("En línea").into(),
        ListGroup::Offline => t("Desconectados").into(),
        ListGroup::Role(role) => store
            .server(server)
            .and_then(|server| server.roles.iter().find(|known| known.id == role))
            .map(|role| role.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| t("Rol").into()),
    }
}

/// The member list, virtual: only the rows in view are built, so a server
/// of thousands costs what a screenful costs. The provider's own order and
/// counts when it keeps one; else the people known here, grouped here.
fn member_list(ui: &Ui, store: &Store, server: ServerId, look: &Look) -> Element {
    // Once its place is known, the card floats over the list there, not
    // from a row the list may move or drop.
    let held = ui
        .profiles
        .member_at
        .filter(|(user, _)| ui.profile == Some(*user));
    let person = |user: UserId| {
        let row = member_row(ui, store, server, user, ui.prefs.role_colors, look)?;
        let card = (ui.profile == Some(user) && held.is_none())
            .then(|| profile::popout_card(ui, store, Some(server), user, look).map(Box::new))
            .flatten();
        Some(MemberListRow::Person(Box::new(row), card))
    };
    let (count, key, row): (usize, KeyOf, RowOf) = match store.member_list_shared(server) {
        Some(list) => {
            let people: HashMap<u32, (u64, MemberListRow)> = list
                .rows
                .iter()
                .filter_map(|(at, row)| match row {
                    ListRow::Member(user) => Some((*at, (user.0, person(*user)?))),
                    ListRow::Heading { .. } => None,
                })
                .collect();
            let titles: Vec<String> = list
                .groups
                .iter()
                .map(|(group, _)| group_title(store, server, *group))
                .collect();
            // Every row is keyed on each layout: people and headings by a
            // lookup, the rest by place. People's IDs are below 2^63; the
            // others are above.
            let mut keys: HashMap<u32, u64> =
                people.iter().map(|(at, (user, _))| (*at, *user)).collect();
            let mut start = 0u32;
            for (place, (_, count)) in list.groups.iter().enumerate() {
                if *count > 0 {
                    keys.insert(start, u64::MAX - place as u64);
                    start = start.saturating_add(count.saturating_add(1));
                }
            }
            let people = Arc::new(people);
            let shown = Arc::clone(&list);
            (
                list.len().min(LIST_SHOWN) as usize,
                Box::new(move |index| {
                    keys.get(&(index as u32))
                        .copied()
                        .unwrap_or((1 << 63) | index as u64)
                }),
                Box::new(move |index| match shown.row(index as u32) {
                    Some(ListRow::Heading { group, count }) => MemberListRow::Heading(Group {
                        title: shown
                            .groups
                            .iter()
                            .position(|(g, _)| *g == group)
                            .map_or_else(|| t("Rol").into(), |at| titles[at].clone()),
                        count: count as usize,
                    }),
                    _ => match people.get(&(index as u32)) {
                        Some((_, MemberListRow::Person(row, card))) => {
                            MemberListRow::Person(row.clone(), card.clone())
                        }
                        _ => MemberListRow::Loading,
                    },
                }),
            )
        }
        None => {
            let mut rows = Vec::new();
            for group in store.member_groups(server) {
                rows.push((
                    u64::MAX - rows.len() as u64,
                    MemberListRow::Heading(Group {
                        title: group.title,
                        count: group.members.len(),
                    }),
                ));
                for user in group.members {
                    if let Some(row) = person(user) {
                        rows.push((user.0, row));
                    }
                }
            }
            let rows = Arc::new(rows);
            let keys = rows.clone();
            (
                rows.len(),
                Box::new(move |index| keys[index].0),
                Box::new(move |index| match &rows[index].1 {
                    MemberListRow::Heading(group) => MemberListRow::Heading(group.clone()),
                    MemberListRow::Person(row, card) => {
                        MemberListRow::Person(row.clone(), card.clone())
                    }
                    MemberListRow::Loading => MemberListRow::Loading,
                }),
            )
        }
    };
    let look_copy = *look;
    let (left, right) = members::INSET;
    let list = ScrollArea::measured_list("members", count, members::ROW, key, move |index| {
        let look = &look_copy;
        let row = match row(index) {
            MemberListRow::Heading(group) => members::heading(&group, look),
            MemberListRow::Person(row, card) => members::member(&row, card.as_deref(), look),
            MemberListRow::Loading => members::placeholder(index, look),
        };
        Element::row(vec![gap(left, 0.0), row.flex(1.0), gap(right, 0.0)])
    })
    .appearance(look.scrollbar(true))
    .render()
    .flex(1.0);
    let list = Element::row(vec![
        divider(look),
        Element::panel(list).flex(1.0).background(look.sidebar),
    ]);
    // Its width moves from its left edge.
    let list = Element::stack(vec![list, crate::desktop::members_handle(look)])
        .size(Some(ui.prefs.members_width), None);
    let list = region(list, MEMBERS, t("Miembros"));
    match held {
        Some((user, at)) => rustcord_components::profile::card_popout(
            list,
            profile::popout_card(ui, store, Some(server), user, look)
                .map(|found| rustcord_components::profile::card(&found, look)),
            rustcord_components::profile::ids::MEMBER_DISMISS,
            limn::PopoverPlacement::LeftStart,
            look,
        )
        .popover_at(at)
        .popover_gap(rustcord_components::profile::LIST_GAP),
        None => list,
    }
}

type KeyOf = Box<dyn Fn(usize) -> u64 + Send + Sync>;
type RowOf = Box<dyn Fn(usize) -> MemberListRow + Send + Sync>;

fn member_row(
    ui: &Ui,
    store: &Store,
    server: ServerId,
    user: UserId,
    role_colors: bool,
    look: &Look,
) -> Option<MemberRow> {
    let account = store.user(user)?;
    let presence = store.presence(user);
    let ignored = store.is_ignored(user);
    let (status_line, status_emoji) = crate::identity::status_line(ui, &presence, ignored);
    let ink = store
        .top_role(user, server)
        .and_then(|role| role.color)
        .filter(|_| role_colors)
        .map_or(look.text, |color| look.ink(color));
    Some(MemberRow {
        id: format!("member:{user}"),
        name: store.display_name(user, Some(server)),
        image: ui.picture(store.avatar_in(Some(server), user)),
        ink,
        accent: look.ink(account.accent),
        status: look.status(presence.status),
        status_line,
        status_emoji,
        offline: !presence.status.is_around(),
        bot: account.bot,
        owner: store
            .server(server)
            .is_some_and(|server| server.owner == Some(user)),
        restricted: store
            .restriction_of(server, user)
            .map(|restriction| profile::restriction_label(restriction, store.wall())),
        marks: crate::identity::marks(ui, store, Some(server), user, look),
    })
}
