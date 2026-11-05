//! The inbox, as Discord's popout of what waits for the account.
//!
//! «No leídos» lists the channels with something unread that the account
//! has not silenced, a card each with its unread messages as Rustcord has
//! them: the newest page of each is asked for as its card comes into view.
//! «Menciones» lists the loaded messages that mention the account: it is not
//! a complete mention history, and says so. «Recordatorios» lists the
//! messages marked unread by hand («Marcar como no leído»). «Marcadores» has
//! nothing yet: Rustcord keeps no bookmarks.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use limn::{Element, Icon, Key, Point, PopoverPlacement, UiAction, UiTree};
use rustcord_components::inbox::{
    self as parts, ids, Bell, Body, EmptyState, Face, Head, Header, Tab, Tools,
};
use rustcord_components::message::{group_starts_at, live_row, Author, Live, MessageView};
use rustcord_components::text::Names;
use rustcord_components::Look;
use rustcord_i18n::{t, tr};
use rustcord_model::{Channel, ChannelId, ChannelKind, Message, MessageId, ServerId, UserId};
use rustcord_state::Store;

use crate::{editor_id, Intent, Ui, ViewContext};

/// The cards of a tab, at most; a note says when there are more.
const MAX_CARDS: usize = 100;
/// The unread messages a card shows, from the first, as Discord's.
const MAX_MESSAGES: usize = 25;
/// The newest pages on their way at once, for the cards in view.
const LOADING: usize = 3;

/// A card: a channel, and the message it is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    channel: ChannelId,
    message: Option<MessageId>,
}

#[derive(Debug)]
pub(crate) struct Inbox {
    account: Option<UserId>,
    tab: Tab,
    revision: u64,
    entries: Arc<Vec<Entry>>,
    /// The tab has more than it shows.
    limited: bool,
    /// Cards folded in this session.
    folded: HashSet<ChannelId>,
    /// Channels whose newest page was asked for since it opened.
    requested: HashSet<ChannelId>,
    /// Of those, the ones whose page did not come: not asked again while
    /// open, and no longer holding one of the `LOADING` slots.
    failed: HashSet<ChannelId>,
    /// The card whose bell's menu is open.
    bell: Option<ChannelId>,
    /// «¿Marcar todo como leído?» waits for an answer.
    confirm: bool,
}

impl Inbox {
    #[cfg(test)]
    pub(crate) fn listed(&self) -> Vec<(ChannelId, Option<MessageId>)> {
        self.entries
            .iter()
            .map(|entry| (entry.channel, entry.message))
            .collect()
    }

    fn new(account: Option<UserId>, tab: Tab) -> Self {
        Self {
            account,
            tab,
            revision: u64::MAX,
            entries: Arc::default(),
            limited: false,
            folded: HashSet::new(),
            requested: HashSet::new(),
            failed: HashSet::new(),
            bell: None,
            confirm: false,
        }
    }
}

/// A live same-account bootstrap updates the background conversation without
/// dismissing the inbox opened from cached startup data.
pub(crate) fn on_ready(ui: &mut Ui, store: &mut Store) -> Vec<Intent> {
    let preserved = ui
        .inbox
        .take()
        .filter(|inbox| inbox.account.is_some() && inbox.account == store.me());
    let mut intents = crate::on_ready(ui, store);
    if let Some(mut inbox) = preserved {
        let focus = inbox.tab.id();
        // What was asked of the old session is asked again of the new.
        inbox.requested.clear();
        inbox.failed.clear();
        inbox.revision = u64::MAX;
        ui.inbox = Some(inbox);
        refresh(ui, store);
        intents.retain(|intent| !matches!(intent, Intent::Focus(_) | Intent::FocusInput { .. }));
        intents.push(Intent::Focus(focus.into()));
    }
    intents
}

pub(crate) fn refresh(ui: &mut Ui, store: &Store) {
    let Some(inbox) = &ui.inbox else { return };
    if inbox.account != store.me() {
        ui.inbox = None;
        return;
    }
    if inbox.revision == store.revision() {
        return;
    }
    let tab = inbox.tab;
    let (mut entries, total) = match tab {
        Tab::Unread => unread(ui, store),
        Tab::Mentions => mentions(store),
        Tab::Reminders => reminders(store),
        Tab::Bookmarks => (Vec::new(), 0),
    };
    entries.truncate(MAX_CARDS);
    let Some(inbox) = &mut ui.inbox else { return };
    inbox.limited = total > entries.len();
    inbox.entries = Arc::new(entries);
    inbox.revision = store.revision();
    if inbox
        .bell
        .is_some_and(|bell| !inbox.entries.iter().any(|entry| entry.channel == bell))
    {
        inbox.bell = None;
    }
}

/// Write the cards again, as in another language.
pub(crate) fn rewrite(ui: &mut Ui, store: &Store) {
    if let Some(inbox) = &mut ui.inbox {
        inbox.revision = u64::MAX;
    }
    refresh(ui, store);
}

/// The channels with something unread the account has not silenced, as
/// Discord orders them: direct conversations, those that mention the
/// account, then the rest, each in the rail's and the list's order.
fn unread(ui: &Ui, store: &Store) -> (Vec<Entry>, usize) {
    let now = store.wall();
    let rail: HashMap<ServerId, usize> = crate::account::layout(store)
        .iter()
        .flat_map(|folder| folder.servers.iter().copied())
        .enumerate()
        .map(|(index, server)| (server, index))
        .collect();
    let mut found = Vec::new();
    for channel in store.read_channels() {
        if !channel.kind.has_messages() {
            continue;
        }
        let read = store.read(channel.id);
        let key = match channel.server {
            None if read.unread() => (1, 0, Reverse(read.last_message), 0, 0),
            None => continue,
            Some(server) => {
                let (unread, mentions, muted) = crate::channel_list::marks(ui, store, channel, now);
                if muted
                    || crate::account::server_notify(ui, store, server).muted(now)
                    || !(unread || mentions > 0)
                {
                    continue;
                }
                let category = channel
                    .parent
                    .and_then(|parent| store.channel(parent))
                    .map_or(-1, |parent| parent.position);
                (
                    if mentions > 0 { 2 } else { 4 },
                    rail.get(&server).copied().unwrap_or(usize::MAX),
                    Reverse(None),
                    category,
                    channel.position,
                )
            }
        };
        found.push((key, channel.id));
    }
    found.sort_unstable();
    let total = found.len();
    (
        found
            .into_iter()
            .map(|(_, channel)| Entry {
                channel,
                message: None,
            })
            .collect(),
        total,
    )
}

/// The loaded messages that mention the account, newest first.
fn mentions(store: &Store) -> (Vec<Entry>, usize) {
    let Some(me) = store.me() else {
        return (Vec::new(), 0);
    };
    let mut found = Vec::new();
    for channel in store.read_channels() {
        let Some(timeline) = store.timeline(channel.id) else {
            continue;
        };
        for message in timeline.messages().iter() {
            if message.author != me
                && !message.delivery.is_local()
                && !store.blocked(message.author)
                && !store.is_ignored(message.author)
                && store.counts_as_mention(message, me)
            {
                found.push((Reverse((message.timestamp, message.id)), channel.id));
            }
        }
    }
    found.sort_unstable();
    let total = found.len();
    (
        found
            .into_iter()
            .map(|(Reverse((_, message)), channel)| Entry {
                channel,
                message: Some(message),
            })
            .collect(),
        total,
    )
}

/// The messages marked unread by hand, the last marked first.
fn reminders(store: &Store) -> (Vec<Entry>, usize) {
    let mut found: Vec<_> = store
        .read_channels()
        .filter_map(|channel| {
            let message = store.read(channel.id).unread_from?;
            Some((Reverse(message), channel.id))
        })
        .collect();
    found.sort_unstable();
    let total = found.len();
    (
        found
            .into_iter()
            .map(|(Reverse(message), channel)| Entry {
                channel,
                message: Some(message),
            })
            .collect(),
        total,
    )
}

fn allowed(store: &Store, channel: ChannelId) -> bool {
    store.channel(channel).is_some_and(Store::visible)
        && store
            .abilities(channel)
            .is_some_and(|a| a.read_history.allowed())
}

pub(crate) fn close(ui: &mut Ui) -> Vec<Intent> {
    ui.inbox = None;
    ui.channel()
        .map(|channel| vec![Intent::Focus(editor_id(channel))])
        .unwrap_or_default()
}

/// Escape closes what is open in it, its menu or its question first.
pub(crate) fn key(ui: &mut Ui, key: Key) -> Option<Vec<Intent>> {
    let inbox = ui.inbox.as_mut()?;
    if key != Key::Escape {
        return None;
    }
    if inbox.bell.take().is_some() || std::mem::take(&mut inbox.confirm) {
        return Some(Vec::new());
    }
    Some(close(ui))
}

/// Any click but its own closes a card's bell menu, as the header's.
pub(crate) fn clicked_elsewhere(ui: &mut Ui, id: &str) {
    if id.starts_with("inbox:bell:")
        || id.starts_with(rustcord_components::channel_menu::ids::SUBMENU)
    {
        return;
    }
    if let Some(inbox) = &mut ui.inbox {
        inbox.bell = None;
    }
}

pub(crate) fn update(ui: &mut Ui, store: &mut Store, action: &UiAction) -> Option<Vec<Intent>> {
    let UiAction::Clicked { id } = action else {
        return None;
    };
    if !id.starts_with("inbox:") && !id.starts_with("reads:retry:") {
        return None;
    }
    if id == ids::OPEN {
        if ui.inbox.take().is_some() {
            return Some(close(ui));
        }
        if crate::invitations::visible(ui)
            || ui.settings.is_some()
            || ui.server_settings.is_some()
            || ui.channel_settings.is_some()
        {
            return Some(Vec::new());
        }
        let mut intents = crate::search::dismiss(ui);
        intents.extend(crate::context::cancel(ui));
        intents.extend(crate::channel_list::close_pins(ui));
        ui.channel_list.menu = None;
        ui.channel_list.bell = None;
        ui.server_menu.open = false;
        ui.profile = None;
        ui.inbox = Some(Inbox::new(store.me(), Tab::Unread));
        refresh(ui, store);
        return Some(intents);
    }
    if id == ids::CLOSE {
        return Some(close(ui));
    }
    let channel_of = |rest: &str| rest.parse::<u64>().ok().map(ChannelId);
    if let Some(channel) = id.strip_prefix("reads:retry:").and_then(channel_of) {
        return Some(
            if ui.channel() == Some(channel) && allowed(store, channel) {
                store
                    .retry_read_command(channel)
                    .map(Intent::Command)
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            },
        );
    }
    ui.inbox.as_ref()?;
    if let Some(tab) = Tab::from_id(id) {
        let inbox = ui.inbox.as_mut()?;
        if inbox.tab != tab {
            inbox.tab = tab;
            inbox.revision = u64::MAX;
            inbox.bell = None;
            refresh(ui, store);
        }
        return Some(Vec::new());
    }
    match id.as_str() {
        ids::FRIENDS => {
            if store.relationships().is_none() {
                return Some(Vec::new());
            }
            ui.inbox = None;
            let intents = crate::relationships::show(ui);
            ui.friends.tab = rustcord_components::friends::FriendsTab::Pending;
            return Some(intents);
        }
        ids::READ_ALL => {
            // Shift skips the question, as in Discord.
            if ui.messages.shift_held {
                return Some(read_all(ui, store));
            }
            ui.inbox.as_mut()?.confirm = true;
            return Some(Vec::new());
        }
        ids::READ_ALL_CONFIRM => return Some(read_all(ui, store)),
        ids::READ_ALL_CANCEL => {
            ui.inbox.as_mut()?.confirm = false;
            return Some(Vec::new());
        }
        ids::BELL_DISMISS => {
            ui.inbox.as_mut()?.bell = None;
            return Some(Vec::new());
        }
        _ => {}
    }
    refresh(ui, store);
    let listed = |ui: &Ui, channel: ChannelId| {
        allowed(store, channel)
            && ui
                .inbox
                .as_ref()
                .is_some_and(|inbox| inbox.entries.iter().any(|entry| entry.channel == channel))
    };
    if let Some(channel) = id.strip_prefix("inbox:fold:").and_then(channel_of) {
        let inbox = ui.inbox.as_mut()?;
        if !inbox.folded.remove(&channel) {
            inbox.folded.insert(channel);
        }
        return Some(Vec::new());
    }
    if let Some(channel) = id.strip_prefix("inbox:bell:").and_then(channel_of) {
        if listed(ui, channel) {
            let inbox = ui.inbox.as_mut()?;
            inbox.bell = (inbox.bell != Some(channel)).then_some(channel);
        }
        return Some(Vec::new());
    }
    if let Some(channel) = id.strip_prefix("inbox:read:").and_then(channel_of) {
        if !listed(ui, channel) {
            return Some(Vec::new());
        }
        let result = store
            .mark_read_command(channel)
            .map(Intent::Command)
            .into_iter()
            .collect();
        refresh(ui, store);
        return Some(result);
    }
    let (channel, message) = if let Some(rest) = id.strip_prefix("inbox:jump:") {
        let (channel, message) = rest.split_once(':')?;
        (
            channel_of(channel)?,
            message.parse::<u64>().ok().map(MessageId),
        )
    } else {
        (id.strip_prefix("inbox:go:").and_then(channel_of)?, None)
    };
    if !listed(ui, channel) {
        return Some(Vec::new());
    }
    let anchor = message.or_else(|| crate::reads::anchor(store, channel));
    let server = store.channel(channel)?.server;
    ui.inbox = None;
    Some(crate::open_at(ui, store, server, channel, anchor))
}

/// Mark every channel of the unread tab read, as Discord's «Marcar todo
/// como leído» does with the unsilenced ones.
fn read_all(ui: &mut Ui, store: &mut Store) -> Vec<Intent> {
    let Some(inbox) = &mut ui.inbox else {
        return Vec::new();
    };
    inbox.confirm = false;
    let (entries, _) = unread(ui, store);
    let mut intents = Vec::new();
    for entry in entries {
        if allowed(store, entry.channel) {
            intents.extend(store.mark_read_command(entry.channel).map(Intent::Command));
        }
    }
    refresh(ui, store);
    intents
}

/// Ask for the newest page of the cards in view whose unread messages are
/// not loaded, a few at a time, once each while the inbox is open.
pub(crate) fn after_layout(ui: &mut Ui, store: &Store, tree: &UiTree) -> Vec<Intent> {
    let Some(inbox) = &mut ui.inbox else {
        return Vec::new();
    };
    if inbox.tab != Tab::Unread {
        return Vec::new();
    }
    let Some(rows) = tree.visible_rows(ids::LIST) else {
        return Vec::new();
    };
    let waiting = inbox
        .requested
        .iter()
        .filter(|channel| !loaded(store, **channel) && !inbox.failed.contains(*channel))
        .count();
    let mut intents = Vec::new();
    let visible: Vec<Entry> = shown(store, inbox)
        .skip(rows.start)
        .take(rows.len())
        .collect();
    for entry in visible {
        if waiting + intents.len() >= LOADING {
            break;
        }
        let channel = entry.channel;
        if loaded(store, channel)
            || inbox.folded.contains(&channel)
            || inbox.requested.contains(&channel)
            || inbox.failed.contains(&channel)
            || !allowed(store, channel)
        {
            continue;
        }
        inbox.requested.insert(channel);
        intents.push(Intent::LoadLatest(channel));
    }
    intents
}

/// A card's newest page did not come: its slot is free, and it is not
/// asked again while the inbox stays open.
pub(crate) fn history_failed(ui: &mut Ui, channel: ChannelId) {
    if let Some(inbox) = &mut ui.inbox {
        if inbox.requested.contains(&channel) {
            inbox.failed.insert(channel);
        }
    }
}

/// The entries the list draws, one row each, in its order: what
/// `surface` lays out, so a row's index is its entry's here.
fn shown<'a>(store: &'a Store, inbox: &'a Inbox) -> impl Iterator<Item = Entry> + 'a {
    inbox.entries.iter().copied().filter(move |entry| {
        allowed(store, entry.channel)
            && store.channel(entry.channel).is_some()
            // A mention whose message is gone, or folded, draws no card.
            && !(inbox.tab == Tab::Mentions
                && entry.message.is_some()
                && (inbox.folded.contains(&entry.channel)
                    || card_messages(store, entry).0.is_empty()))
    })
}

/// The newest messages of `channel` are here.
fn loaded(store: &Store, channel: ChannelId) -> bool {
    store
        .timeline(channel)
        .is_some_and(|timeline| timeline.latest)
}

/// The authors of the messages of the cards in view, for their faces.
pub(crate) fn wanted(
    ui: &Ui,
    store: &Store,
    tree: &UiTree,
    add: &mut impl FnMut(Option<&rustcord_model::RemoteImage>),
) {
    let Some(inbox) = &ui.inbox else { return };
    let Some(rows) = tree.visible_rows(ids::LIST) else {
        return;
    };
    for entry in shown(store, inbox).skip(rows.start).take(rows.len()) {
        let Some(channel) = store.channel(entry.channel) else {
            continue;
        };
        if channel.server.is_none() {
            add(channel.icon.as_ref());
            for user in &channel.recipients {
                add(store.avatar_in(None, *user));
            }
        }
        for message in card_messages(store, &entry).0 {
            add(store.avatar_in(channel.server, message.author));
            for emoji in crate::images::emoji_pictures(&message) {
                add(Some(emoji));
            }
        }
    }
}

/// The messages a card shows: a channel's first unread ones loaded, and
/// whether there are more; or the one it is about.
fn card_messages(store: &Store, entry: &Entry) -> (Vec<Arc<Message>>, bool) {
    let Some(timeline) = store.timeline(entry.channel) else {
        return (Vec::new(), false);
    };
    if let Some(message) = entry.message {
        return (timeline.get(message).cloned().into_iter().collect(), false);
    }
    let read = store.read(entry.channel);
    let messages = timeline.messages();
    let first = messages.iter().position(|message| {
        !message.delivery.is_local()
            && match read.unread_from {
                Some(from) => message.id >= from,
                None => Some(message.id) > read.last_read,
            }
    });
    let Some(first) = first else {
        return (Vec::new(), false);
    };
    // The first loaded is unread and the unread do not start there: some
    // come before it.
    let before = first == 0
        && !timeline.reached_start
        && read.unread_from.or(read.first_unread) != Some(messages[0].id);
    let unread: Vec<_> = messages[first..]
        .iter()
        .filter(|message| !message.delivery.is_local())
        .cloned()
        .collect();
    let more = before || unread.len() > MAX_MESSAGES;
    (unread.into_iter().take(MAX_MESSAGES).collect(), more)
}

/// Whether the conversation's header shows the tray, so the popout opens
/// under it; otherwise it opens at the header's end.
fn in_header(ui: &Ui, store: &Store, width: f32) -> bool {
    !crate::view::narrow_header(ui, width)
        && !crate::relationships::visible(ui)
        && ui
            .channel()
            .and_then(|channel| store.channel(channel))
            .is_some_and(|channel| channel.kind.has_messages())
}

/// The popout, for the header's tray while it shows it.
pub(crate) fn popout(
    ui: &Ui,
    store: &Store,
    look: &Look,
    context: &ViewContext,
) -> Option<Element> {
    let inbox = ui.inbox.as_ref()?;
    if inbox.account != store.me() || !in_header(ui, store, context.window.width) {
        return None;
    }
    Some(surface(ui, store, inbox, look, context))
}

/// Over `root`: the popout at the header's end when the header has no tray
/// to open it under (a narrow window has it in its «más»), and the question
/// before marking everything read.
pub(crate) fn view(
    ui: &Ui,
    store: &Store,
    mut root: Element,
    look: &Look,
    context: &ViewContext,
) -> Element {
    let Some(inbox) = &ui.inbox else { return root };
    if inbox.account != store.me() {
        return root;
    }
    if !in_header(ui, store, context.window.width) {
        let end = context.window.width - crate::title_bar::room(ui) - 8.0;
        root = Element::popover(
            root,
            surface(ui, store, inbox, look, context),
            ids::CLOSE,
            PopoverPlacement::BottomEnd,
        )
        .popover_at(Point::new(end, rustcord_components::sidebar::HEADER))
        .popover_gap(0.0);
    }
    if inbox.confirm {
        root = parts::confirm(root, look);
    }
    root
}

/// What every card of the list reads, shared by the closure that builds
/// them as they come into view.
struct Cards {
    cards: Vec<Card>,
    note: Option<String>,
    authors: HashMap<(Option<ServerId>, UserId), Author>,
    names: HashMap<ChannelId, Names>,
    /// The emoji its messages name, once their pictures arrived.
    pictures: HashMap<rustcord_model::RemoteImage, limn::RasterImage>,
    look: Look,
    width: f32,
    now: rustcord_model::Timestamp,
    style: rustcord_components::message::MessageStyle,
}

struct Card {
    entry: Entry,
    server: Option<ServerId>,
    head: Head,
    tools: Tools,
    content: Content,
}

enum Content {
    Messages(Vec<Arc<Message>>, bool),
    Loading,
    Folded,
    Single(Arc<Message>),
    Note(String),
}

impl Cards {
    fn key(&self, index: usize) -> u64 {
        match self.cards.get(index) {
            Some(card) => card.entry.message.map_or(card.entry.channel.0, |m| m.0),
            None => u64::MAX,
        }
    }

    fn row(&self, index: usize) -> Element {
        let Some(card) = self.cards.get(index) else {
            return parts::note(self.note.as_deref().unwrap_or_default(), &self.look);
        };
        let body = match &card.content {
            Content::Messages(messages, more) => {
                // Days are told apart unless all of them are today's.
                let today = messages.first().is_none_or(|first| {
                    rustcord_components::time::day(first.timestamp)
                        == rustcord_components::time::day(self.now)
                });
                let rows = messages
                    .iter()
                    .enumerate()
                    .map(|(index, message)| {
                        let day = (!today
                            && index.checked_sub(1).is_none_or(|previous| {
                                rustcord_components::time::day(messages[previous].timestamp)
                                    != rustcord_components::time::day(message.timestamp)
                            }))
                        .then(|| rustcord_components::time::day_label(message.timestamp, self.now));
                        let continued = !group_starts_at(messages, index, MAX_MESSAGES);
                        let continued = continued && day.is_none();
                        self.message(card, message, continued, day)
                    })
                    .collect();
                Body::Messages { rows, more: *more }
            }
            Content::Loading => Body::Loading,
            Content::Folded => Body::Folded,
            Content::Single(message) => {
                Body::Single(Box::new(self.message(card, message, false, None)))
            }
            Content::Note(note) => Body::Note(note.clone()),
        };
        parts::card(&card.head, card.tools.clone(), body, &self.look)
    }

    fn message(
        &self,
        card: &Card,
        message: &Message,
        continued: bool,
        day: Option<String>,
    ) -> Element {
        let author = self
            .authors
            .get(&(card.server, message.author))
            .cloned()
            .unwrap_or_else(|| Author {
                marks: Default::default(),
                image: None,
                name: t("desconocido").into(),
                ink: self.look.muted,
                accent: self.look.faint,
                bot: false,
            });
        let row = live_row(
            MessageView {
                message,
                author,
                continued,
                day,
                first_new: false,
                start: None,
                reply: None,
                mentions_me: false,
                abilities: Default::default(),
                reacting: false,
                author_card: None,
                width: parts::message_width(self.width),
                now: self.now,
                style: self.style,
            },
            Live {
                names: self.names.get(&card.entry.channel),
                attachment_images: Some(&self.pictures),
                ..Live::default()
            },
            &self.look,
        );
        parts::jumpable(row, card.entry.channel.0, message.id.0, &self.look)
    }
}

/// The popout itself: its header over the tab's cards, or what the tab
/// says when it has none.
fn surface(ui: &Ui, store: &Store, inbox: &Inbox, look: &Look, context: &ViewContext) -> Element {
    let size = parts::size(context.window);
    // Paint revalidates visibility as well: an old projection never exposes a
    // revoked card while the host is applying that frame's events.
    let entries: Vec<Entry> = inbox
        .entries
        .iter()
        .copied()
        .filter(|entry| allowed(store, entry.channel))
        .collect();
    let local = !store.read_sync();
    let header = parts::header(
        &Header {
            tab: inbox.tab,
            read_all: inbox.tab == Tab::Unread && !entries.is_empty(),
            // Once the list of friends is known.
            friends: store
                .relationships()
                .is_some()
                .then(|| store.pending_requests()),
            local,
        },
        look,
    );
    if entries.is_empty() {
        return parts::popout(
            header,
            parts::empty(&empty_state(inbox.tab), look),
            false,
            size,
            look,
        );
    }
    let note = match inbox.tab {
        Tab::Mentions => Some(t(
            "Sólo las menciones de los mensajes cargados en este equipo; no es un historial completo.",
        )
        .to_owned()),
        Tab::Reminders if !store.remote_unread() => {
            Some(t("Los mensajes marcados como no leídos solo en Rustcord.").to_owned())
        }
        _ => None,
    };
    let note = match (inbox.limited, note) {
        (true, Some(note)) => Some(format!(
            "{} {note}",
            tr!("Se muestran los primeros {count}.", count = MAX_CARDS)
        )),
        (true, None) => Some(tr!("Se muestran los primeros {count}.", count = MAX_CARDS)),
        (false, note) => note,
    };
    let mut authors = HashMap::new();
    let mut names = HashMap::new();
    let mut pictures = HashMap::new();
    let mut cards = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(channel) = store.channel(entry.channel) else {
            continue;
        };
        let mut head = head(ui, store, channel, look);
        // A message's card names its channel only, as Discord's.
        if entry.message.is_some() {
            head.mentions = 0;
        }
        let folded = inbox.folded.contains(&entry.channel);
        let (messages, more) = if folded {
            (Vec::new(), false)
        } else {
            card_messages(store, &entry)
        };
        if !messages.is_empty() {
            // What its text names, and the conversations its links lead to.
            let known = names
                .remove(&entry.channel)
                .unwrap_or_else(|| crate::timeline::names_for(store, entry.channel, look));
            names.insert(
                entry.channel,
                crate::timeline::places(known, store, channel.server, &messages, context.wall),
            );
            for message in &messages {
                for source in crate::images::emoji_pictures(message) {
                    if let Some(image) = ui.images.get(source) {
                        pictures.insert(source.clone(), image.clone());
                    }
                }
                authors
                    .entry((channel.server, message.author))
                    .or_insert_with(|| {
                        crate::timeline::author(ui, store, channel.server, message.author, look)
                    });
            }
        }
        let (tools, content) = match (inbox.tab, entry.message) {
            (Tab::Mentions, Some(message)) => (
                Tools::Mention { message: message.0 },
                match messages.into_iter().next() {
                    Some(found) => Content::Single(found),
                    None => continue,
                },
            ),
            (_, Some(message)) => (
                Tools::Reminder {
                    message: message.0,
                    local,
                },
                match messages.into_iter().next() {
                    Some(found) => Content::Single(found),
                    None => Content::Note(t("El mensaje marcado no está cargado.").into()),
                },
            ),
            (_, None) => (
                Tools::Unread {
                    bell: channel.server.is_some().then(|| {
                        let (muted, entries) =
                            crate::channel_list::bell_entries(ui, store, channel, context.wall);
                        Bell {
                            open: inbox.bell == Some(entry.channel),
                            muted,
                            entries,
                        }
                    }),
                    folded,
                    local,
                },
                if folded {
                    Content::Folded
                } else if !loaded(store, entry.channel) {
                    Content::Loading
                } else if messages.is_empty() {
                    Content::Note(t("Sus mensajes no leídos no están cargados.").into())
                } else {
                    Content::Messages(messages, more)
                },
            ),
        };
        cards.push(Card {
            entry,
            server: channel.server,
            head,
            tools,
            content,
        });
    }
    let count = cards.len() + usize::from(note.is_some());
    let shared = Arc::new(Cards {
        cards,
        note,
        authors,
        names,
        pictures,
        look: *look,
        width: size.width,
        now: context.wall,
        style: rustcord_components::message::MessageStyle {
            sample: true,
            ..ui.prefs.message_style()
        },
    });
    let keys = shared.clone();
    let list = parts::list(
        count,
        move |index| keys.key(index),
        move |index| shared.row(index),
        look,
    );
    parts::popout(header, list, true, size, look)
}

/// A card's channel: its server's mark or the person's, its name, its
/// mentions, and where it is.
fn head(ui: &Ui, store: &Store, channel: &Channel, look: &Look) -> Head {
    let read = store.read(channel.id);
    let name = store.channel_title(channel.id);
    let (face, glyph, path) = match channel.server.and_then(|server| store.server(server)) {
        Some(server) => {
            let mut path = server.shown_name().to_owned();
            if let Some(parent) = channel.parent.and_then(|parent| store.channel(parent)) {
                path.push_str(" \u{203A} ");
                path.push_str(&store.channel_title(parent.id));
            }
            (
                Face {
                    name: server.shown_name().to_owned(),
                    initials: server.initials(),
                    image: ui.image(&server.icon),
                    accent: look.ink(server.accent),
                    person: false,
                },
                Some(crate::channel_list::glyph(channel.kind).unwrap_or(Icon::Hash)),
                path,
            )
        }
        None => {
            let other = channel
                .recipients
                .iter()
                .copied()
                .find(|user| Some(*user) != store.me());
            let (image, accent) = match (channel.kind, other.and_then(|user| store.user(user))) {
                (ChannelKind::Group, _) => (ui.image(&channel.icon), look.accent),
                (_, Some(user)) => (
                    ui.picture(store.avatar_in(None, user.id)),
                    look.ink(user.accent),
                ),
                _ => (None, look.accent),
            };
            (
                Face {
                    name: name.clone(),
                    initials: String::new(),
                    image,
                    accent,
                    person: true,
                },
                None,
                if channel.kind == ChannelKind::Group {
                    tr!("{count} miembros", count = channel.recipients.len() + 1)
                } else {
                    String::new()
                },
            )
        }
    };
    Head {
        channel: channel.id.0,
        face,
        glyph,
        name,
        mentions: read.mentions,
        path,
    }
}

/// What a tab with nothing says, as Discord's, honest about what Rustcord
/// keeps.
fn empty_state(tab: Tab) -> EmptyState {
    let (glyph, title, tip) = match tab {
        Tab::Unread => (
            Icon::Custom(rustcord_components::glyphs::INBOX),
            t("¡Ya estás al día!"),
            t("Los mensajes no leídos de todos tus canales sin silenciar aparecerán aquí. ¡Es hora de poner el buzón en cero!"),
        ),
        Tab::Mentions => (
            Icon::At,
            t("¡Lo lograste, llegaste al final!"),
            t("Aquí aparecen las menciones de los mensajes cargados en este equipo; no es un historial completo."),
        ),
        Tab::Bookmarks => (
            Icon::Custom(rustcord_components::glyphs::BOOKMARK),
            t("Guarda tus recuerdos favoritos"),
            t("Rustcord todavía no guarda marcadores."),
        ),
        Tab::Reminders => (
            Icon::Clock,
            t("Organízate y no te olvides de hacer un seguimiento"),
            t("Haz clic derecho en cualquier mensaje y elige «Marcar como no leído». Aquí lo tendrás hasta que lo leas."),
        ),
    };
    EmptyState {
        glyph,
        title: title.into(),
        tip: tip.into(),
    }
}
