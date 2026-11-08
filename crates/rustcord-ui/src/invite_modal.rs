//! Discord's «Invita a tus amigos»: the modal the header's person with a
//! plus, a channel's row and menu, the server's menus, the call's bar and
//! the invitations of the settings open. As Discord's web client does, it
//! asks for the channel's link once as it opens, handing back the code made
//! for that channel before in this run so the same link comes back while it
//! works; «Generar un nuevo enlace» asks for a new one. A friend's
//! «Invitar» opens their direct conversation and sends the link there, one
//! friend at a time: the next waits for the last one's answer, as Discord's
//! queue does.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use limn::{Element, UiAction};
use rustcord_components::invite::{self as parts, ids, Friend, InviteView, Invited, Settings};
use rustcord_components::Look;
use rustcord_i18n::{t, tr};
use rustcord_model::{ChannelId, ChannelKind, Delivery, ServerId, UserId};
use rustcord_provider::{
    Command, Created, Failure, InviteFrom, InvitePlace, ProviderKind, RequestId,
};
use rustcord_state::{Applied, Permit, Store};

use crate::server_settings::advanced::{invite_channels, invite_link, INVITE_AGES, INVITE_USES};
use crate::{Intent, Ui, ViewContext};

/// The link's options as the owner's Discord starts them: 30 days (an index
/// of [`INVITE_AGES`]; Discord's own default is 7, and 30 for the accounts
/// it tries it with), no limit of uses, and members who stay.
const DEFAULT_AGE: usize = 6;
/// How long «Copiado» stays on the button.
const COPIED: Duration = Duration::from_secs(1);

/// How the link is made: indexes of [`INVITE_AGES`] and [`INVITE_USES`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Options {
    pub age: usize,
    pub uses: usize,
    pub temporary: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            age: DEFAULT_AGE,
            uses: 0,
            temporary: false,
        }
    }
}

/// The link made last for a channel, kept for this run: the code handed
/// back the next time the modal opens on it, with the options it was made with.
#[derive(Clone, Debug, PartialEq)]
pub struct Made {
    pub code: String,
    pub options: Options,
}

/// Where the link to a friend is.
#[derive(Clone, Debug, PartialEq)]
enum Sending {
    /// Their conversation is being opened with this request.
    Opening(RequestId),
    /// The link is on its way to their conversation, until its message
    /// is confirmed or fails.
    Sent(ChannelId),
    /// The provider confirmed it.
    Done,
    Failed(String),
    /// It may have arrived: sending it again could repeat it.
    Unsure(String),
}

/// The modal while open.
#[derive(Debug)]
pub struct InviteModal {
    server: ServerId,
    channel: ChannelId,
    /// Where it was opened, told with each link asked for.
    place: InvitePlace,
    query: String,
    options: Options,
    /// The link being made, by its request.
    making: Option<RequestId>,
    code: Option<String>,
    /// Why the link could not be made.
    failed: Option<String>,
    copied: Option<Instant>,
    /// The settings view, open with what it would make.
    settings: Option<Options>,
    select: Option<&'static str>,
    sends: HashMap<UserId, Sending>,
}

/// Whether the modal is open.
pub(crate) fn visible(ui: &Ui) -> bool {
    ui.invite.is_some()
}

/// Open the modal for `channel` of `server` and ask for its link; a denial
/// says why in the corner instead.
pub(crate) fn open(
    ui: &mut Ui,
    store: &Store,
    (server, channel): (ServerId, ChannelId),
    place: InvitePlace,
    now: Instant,
) -> Vec<Intent> {
    let Some(abilities) = store.server_abilities(server) else {
        return Vec::new();
    };
    let reason = abilities
        .create_invites
        .reason()
        .map(str::to_owned)
        .or_else(|| {
            (!invite_channels(store, server).contains(&channel))
                .then(|| t("No puedes invitar a este canal.").to_owned())
        });
    if let Some(reason) = reason {
        ui.notify(t("No se creó la invitación"), Some(reason), true, now);
        return Vec::new();
    }
    let options = ui
        .invites_made
        .get(&channel)
        .map_or_else(Options::default, |made| made.options);
    ui.invite = Some(InviteModal {
        server,
        channel,
        place,
        query: String::new(),
        options,
        making: None,
        code: None,
        failed: None,
        copied: None,
        settings: None,
        select: None,
        sends: HashMap::new(),
    });
    let mut intents = vec![Intent::SetInput {
        id: ids::QUERY.into(),
        value: String::new(),
    }];
    intents.extend(make(ui, true));
    intents
}

/// The channel a server's own entries invite to, as Discord's: the one open
/// in it (a thread's parent) when an invitation can lead there, else the
/// first that can.
pub(crate) fn channel_for(ui: &Ui, store: &Store, server: ServerId) -> Option<ChannelId> {
    let channels = invite_channels(store, server);
    ui.channels
        .get(&Some(server))
        .and_then(|open| store.channel(*open))
        .map(|open| match open.kind {
            ChannelKind::Thread => open.parent.unwrap_or(open.id),
            _ => open.id,
        })
        .filter(|open| channels.contains(open))
        .or_else(|| channels.first().copied())
}

/// Open the modal from a server's own entry at `place`.
pub(crate) fn open_server(
    ui: &mut Ui,
    store: &Store,
    server: ServerId,
    place: InvitePlace,
    now: Instant,
) -> Vec<Intent> {
    match channel_for(ui, store, server) {
        Some(channel) => open(ui, store, (server, channel), place, now),
        None => Vec::new(),
    }
}

/// Ask for the link with the modal's options. Opening (`reuse`) hands back
/// the code made before for this channel with the same options, so the
/// provider may answer with it instead of a new one.
fn make(ui: &mut Ui, reuse: bool) -> Vec<Intent> {
    let request = ui.fresh_request();
    let Some(modal) = ui.invite.as_mut() else {
        return Vec::new();
    };
    let from = if reuse {
        InviteFrom::Opening {
            place: modal.place,
            previous: ui
                .invites_made
                .get(&modal.channel)
                .filter(|made| made.options == modal.options)
                .map(|made| made.code.clone()),
        }
    } else {
        InviteFrom::Generating(modal.place)
    };
    modal.making = Some(request);
    modal.code = None;
    modal.failed = None;
    modal.copied = None;
    modal.sends.clear();
    vec![Intent::Command(Command::CreateInvite {
        request,
        server: modal.server,
        channel: modal.channel,
        max_age_secs: INVITE_AGES[modal.options.age.min(INVITE_AGES.len() - 1)].0,
        max_uses: INVITE_USES[modal.options.uses.min(INVITE_USES.len() - 1)].0,
        temporary: modal.options.temporary,
        from,
    })]
}

/// Answer what the modal reports. `None` when the action is not its own.
pub(crate) fn update(
    ui: &mut Ui,
    store: &mut Store,
    action: &UiAction,
    now: Instant,
) -> Option<Vec<Intent>> {
    let modal = ui.invite.as_mut()?;
    match action {
        UiAction::InputChanged { id, value } if id == ids::QUERY => {
            modal.query = value.clone();
            Some(Vec::new())
        }
        UiAction::Clicked { id } => clicked(ui, store, id, now),
        _ => None,
    }
}

fn clicked(ui: &mut Ui, store: &mut Store, id: &str, now: Instant) -> Option<Vec<Intent>> {
    let modal = ui.invite.as_mut()?;
    if let Some(user) = id.strip_prefix(ids::SEND) {
        let user = UserId(user.parse().ok()?);
        return Some(send(ui, store, user));
    }
    for (select, choices) in [
        (ids::AGE, INVITE_AGES.len()),
        (ids::USES, INVITE_USES.len()),
    ] {
        if let Some(index) = id
            .strip_prefix(select)
            .and_then(|rest| rest.strip_prefix(':'))
        {
            let index: usize = index.parse().ok().filter(|index| *index < choices)?;
            if let Some(settings) = modal.settings.as_mut() {
                if select == ids::AGE {
                    settings.age = index;
                } else {
                    settings.uses = index;
                }
            }
            modal.select = None;
            return Some(Vec::new());
        }
    }
    Some(match id {
        ids::CLOSE => {
            ui.invite = None;
            Vec::new()
        }
        ids::COPY => match &modal.code {
            Some(code) => {
                modal.copied = Some(now);
                // The whole link, even when streamer mode dots it.
                vec![Intent::Copy(invite_link(store, code))]
            }
            None => Vec::new(),
        },
        ids::EDIT => {
            modal.settings = Some(modal.options);
            modal.select = None;
            Vec::new()
        }
        ids::AGE | ids::USES => {
            let select = if id == ids::AGE { ids::AGE } else { ids::USES };
            modal.select = (modal.select != Some(select)).then_some(select);
            Vec::new()
        }
        ids::SELECT_DISMISS => {
            modal.select = None;
            Vec::new()
        }
        ids::TEMPORARY => {
            if let Some(settings) = modal.settings.as_mut() {
                settings.temporary = !settings.temporary;
            }
            Vec::new()
        }
        ids::BACK => {
            modal.settings = None;
            modal.select = None;
            Vec::new()
        }
        // A new link, never the one handed out before.
        ids::GENERATE => {
            let denied = store
                .server_abilities(modal.server)
                .and_then(|abilities| abilities.create_invites.reason().map(str::to_owned));
            if let Some(reason) = denied {
                ui.notify(t("No se creó la invitación"), Some(reason), true, now);
                return Some(Vec::new());
            }
            match modal.settings.take() {
                Some(options) => {
                    modal.options = options;
                    modal.select = None;
                    make(ui, false)
                }
                None => Vec::new(),
            }
        }
        _ => return None,
    })
}

/// Send the link to `user`, in their direct conversation, opened for it
/// when there is none. One at a time: the next waits for this one's answer.
fn send(ui: &mut Ui, store: &mut Store, user: UserId) -> Vec<Intent> {
    let Some(modal) = ui.invite.as_ref() else {
        return Vec::new();
    };
    let ready = matches!(
        invited(modal, &link_of(modal, store), user),
        Invited::Ready | Invited::Failed(_)
    );
    // Only a friend outside the server, as the list offers them.
    let outside = store.member_of(modal.server, user).is_none();
    if !ready || !outside || busy(modal, store) || !store.friends().contains(&user) {
        return Vec::new();
    }
    let Some(link) = link_of(modal, store) else {
        return Vec::new();
    };
    if let Some(channel) = store.direct_with(user) {
        return deliver(ui, user, channel, link);
    }
    if let Permit::Denied(reason) = store.open_direct(user) {
        if let Some(modal) = ui.invite.as_mut() {
            modal.sends.insert(user, Sending::Failed(reason));
        }
        return Vec::new();
    }
    let request = ui.fresh_request();
    if let Some(modal) = ui.invite.as_mut() {
        modal.sends.insert(user, Sending::Opening(request));
    }
    vec![Intent::Command(Command::OpenDirect { request, user })]
}

fn deliver(ui: &mut Ui, user: UserId, channel: ChannelId, content: String) -> Vec<Intent> {
    if let Some(modal) = ui.invite.as_mut() {
        modal.sends.insert(user, Sending::Sent(channel));
    }
    vec![Intent::Send {
        channel,
        content,
        reply_to: None,
        mention_reply: false,
    }]
}

/// The link the modal sends and copies, once made.
fn link_of(modal: &InviteModal, store: &Store) -> Option<String> {
    modal.code.as_ref().map(|code| invite_link(store, code))
}

/// The link the open modal shows and sends, once made.
#[cfg(test)]
pub(crate) fn shown_link(ui: &Ui, store: &Store) -> Option<String> {
    link_of(ui.invite.as_ref()?, store)
}

/// Whether a link to a friend is on its way: the next one waits.
fn busy(modal: &InviteModal, store: &Store) -> bool {
    let link = link_of(modal, store);
    modal
        .sends
        .keys()
        .any(|user| invited(modal, &link, *user) == Invited::Sending)
}

/// Where the link to `user` is, as [`follow`] read it from the message
/// in their conversation: never sent before the provider says so.
fn invited(modal: &InviteModal, link: &Option<String>, user: UserId) -> Invited {
    if link.is_none() {
        return Invited::Waiting;
    }
    match modal.sends.get(&user) {
        None => Invited::Ready,
        Some(Sending::Opening(_) | Sending::Sent(_)) => Invited::Sending,
        Some(Sending::Done) => Invited::Sent,
        Some(Sending::Failed(reason)) => Invited::Failed(reason.clone()),
        Some(Sending::Unsure(reason)) => Invited::Unsure(reason.clone()),
    }
}

/// Follow the links on their way: the message sent says whether it got
/// there. A confirmed message replaces the one shown while it was sent,
/// and a conversation not loaded keeps neither, so one gone was confirmed:
/// it is there from the moment it is sent until then.
fn follow(modal: &mut InviteModal, store: &Store) {
    let Some(link) = link_of(modal, store) else {
        return;
    };
    let me = store.me();
    for sending in modal.sends.values_mut() {
        let Sending::Sent(channel) = sending else {
            continue;
        };
        let delivery = store.timeline(*channel).and_then(|timeline| {
            timeline
                .messages()
                .iter()
                .rev()
                .find(|message| Some(message.author) == me && message.content == link)
                .map(|message| message.delivery.clone())
        });
        match delivery {
            Some(Delivery::Sending) => {}
            Some(Delivery::Sent) | None => *sending = Sending::Done,
            Some(Delivery::Failed { reason }) => *sending = Sending::Failed(reason),
            Some(Delivery::Unknown { reason }) => *sending = Sending::Unsure(reason),
        }
    }
}

/// What the provider answered: the link, or why not, and the conversations
/// opened for it.
pub(crate) fn on_applied(ui: &mut Ui, store: &Store, applied: &Applied) -> Vec<Intent> {
    // A new session forgets the links made, as Discord's client does when
    // it connects; what was on its way may or may not have arrived.
    if matches!(applied, Applied::Ready) {
        ui.invites_made.clear();
        for sending in ui
            .invite
            .iter_mut()
            .flat_map(|modal| modal.sends.values_mut())
        {
            if matches!(sending, Sending::Opening(_) | Sending::Sent(_)) {
                *sending = Sending::Unsure(t("Se reconectó antes de saber si llegó.").to_owned());
            }
        }
        return Vec::new();
    }
    let Some(modal) = ui.invite.as_mut() else {
        return Vec::new();
    };
    follow(modal, store);
    match applied {
        Applied::InviteCreated { request, code, .. } if modal.making == Some(*request) => {
            modal.making = None;
            modal.code = Some(code.clone());
            ui.invites_made.insert(
                modal.channel,
                Made {
                    code: code.clone(),
                    options: modal.options,
                },
            );
            Vec::new()
        }
        Applied::Failed {
            failure: Failure::NewInvite { request, .. },
            reason,
            ..
        } if modal.making == Some(*request) => {
            modal.making = None;
            modal.failed = Some(reason.clone());
            Vec::new()
        }
        Applied::Created {
            request,
            created: Created::Channel(channel),
        } => {
            let Some(user) = opening(modal, *request) else {
                return Vec::new();
            };
            // Only their own conversation takes the link.
            match (
                store.direct_with(user) == Some(*channel),
                link_of(modal, store),
            ) {
                (true, Some(link)) => deliver(ui, user, *channel, link),
                _ => {
                    if let Some(modal) = ui.invite.as_mut() {
                        modal.sends.remove(&user);
                    }
                    Vec::new()
                }
            }
        }
        Applied::Failed {
            failure: Failure::Direct { request, .. },
            reason,
            ..
        } => {
            if let Some(user) = opening(modal, *request) {
                modal.sends.insert(user, Sending::Failed(reason.clone()));
            }
            Vec::new()
        }
        // The server or the channel is gone: so is the modal.
        Applied::ServerRemoved { server, .. } if *server == modal.server => {
            ui.invite = None;
            Vec::new()
        }
        Applied::ChannelRemoved(channel) if *channel == modal.channel => {
            ui.invite = None;
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// The friend whose conversation `request` opens.
fn opening(modal: &InviteModal, request: RequestId) -> Option<UserId> {
    modal
        .sends
        .iter()
        .find(|(_, sending)| **sending == Sending::Opening(request))
        .map(|(user, _)| *user)
}

/// «Copiado» goes back to «Copiar». True when the view changed.
pub(crate) fn expire(ui: &mut Ui, now: Instant) -> bool {
    let Some(modal) = ui.invite.as_mut() else {
        return false;
    };
    let over = modal.copied.is_some_and(|at| now >= at + COPIED);
    if over {
        modal.copied = None;
    }
    over
}

pub(crate) fn next_expiry(ui: &Ui, now: Instant) -> Option<Duration> {
    let at = ui.invite.as_ref()?.copied?;
    Some((at + COPIED).saturating_duration_since(now))
}

/// The friends the link can go to, as Discord's list has them: those with
/// a conversation first, the latest first, then the rest by name; never a
/// bot or someone ignored, nor, unless found by a search, who is in the
/// server already (a search names them «Miembro»).
fn friends(ui: &Ui, store: &Store, modal: &InviteModal, look: &Look) -> Vec<Friend> {
    let query = modal.query.trim().to_lowercase();
    let link = link_of(modal, store);
    let busy = busy(modal, store);
    let friends = store.friends();
    let mut ordered: Vec<UserId> = store
        .directs()
        .into_iter()
        .filter(|channel| channel.kind == ChannelKind::Direct)
        .filter_map(|channel| channel.recipients.first().copied())
        .filter(|user| friends.contains(user))
        .collect();
    let talked: std::collections::HashSet<UserId> = ordered.iter().copied().collect();
    ordered.extend(friends.into_iter().filter(|user| !talked.contains(user)));
    ordered
        .into_iter()
        .filter_map(|user| {
            let account = store.user(user);
            if account.is_some_and(|account| account.bot) || store.is_ignored(user) {
                return None;
            }
            let member = store.member_of(modal.server, user).is_some();
            if member && query.is_empty() {
                return None;
            }
            let name = store.display_name(user, None);
            let handle = account.map_or_else(String::new, |account| account.name.clone());
            let nickname = store
                .relationship(user)
                .and_then(|relationship| relationship.nickname.clone())
                .unwrap_or_default();
            let matches = query.is_empty()
                || [&name, &handle, &nickname]
                    .iter()
                    .any(|said| said.to_lowercase().contains(&query));
            let invited = match invited(modal, &link, user) {
                _ if member => Invited::Member,
                Invited::Ready | Invited::Failed(_) if busy => Invited::Waiting,
                invited => invited,
            };
            matches.then(|| Friend {
                user: user.0,
                name,
                handle,
                image: account.and_then(|account| ui.picture(account.picture())),
                accent: account.map_or(look.faint, |account| look.ink(account.accent)),
                invited,
            })
        })
        .collect()
}

/// How long the link lasts and how many times it works, in Discord's words.
fn expiry(options: Options) -> String {
    let age = INVITE_AGES
        .get(options.age)
        .filter(|(seconds, _)| *seconds > 0)
        .map(|(_, label)| t(label));
    let uses = INVITE_USES
        .get(options.uses)
        .filter(|(uses, _)| *uses > 0)
        .map(|(_, label)| t(label));
    match (age, uses) {
        (None, None) => t("Tu enlace de invitación nunca caducará.").to_owned(),
        (None, Some(uses)) => tr!("Tu enlace de invitación caduca después de {uses}.", uses),
        (Some(time), None) => tr!("Tu enlace de invitación caduca en {time}.", time),
        (Some(time), Some(uses)) => tr!(
            "Tu enlace de invitación caduca en {time}, o después de {uses}.",
            time,
            uses
        ),
    }
}

/// The modal over `root` while open.
pub(crate) fn overlay(
    ui: &Ui,
    store: &Store,
    root: Element,
    look: &Look,
    context: &ViewContext,
) -> Element {
    let Some(modal) = ui.invite.as_ref() else {
        return root;
    };
    let (Some(server), Some(channel)) = (store.server(modal.server), store.channel(modal.channel))
    else {
        return root;
    };
    let friends = friends(ui, store, modal, look);
    let note = match store.friends_list() {
        Permit::Denied(reason) => Some(reason),
        Permit::Allowed if friends.is_empty() && !modal.query.trim().is_empty() => {
            Some(t("Ningún amigo coincide.").into())
        }
        Permit::Allowed if friends.is_empty() => {
            Some(t("No tienes amigos a quienes invitar a este servidor.").into())
        }
        Permit::Allowed => None,
    };
    let pending = match &modal.failed {
        Some(reason) => reason.clone(),
        None => t("Creando el enlace…").to_owned(),
    };
    let simulated = store
        .provider
        .as_ref()
        .is_some_and(|info| info.kind == ProviderKind::Simulated)
        .then(|| t("Simulada: este enlace no lleva a ningún lado.").to_owned());
    let view = InviteView {
        server: server.shown_name().to_owned(),
        channel: channel.name.clone(),
        glyph: match channel.kind {
            ChannelKind::Voice => limn::Icon::Volume,
            kind => crate::channel_list::glyph(kind).unwrap_or(limn::Icon::Hash),
        },
        query: modal.query.clone(),
        friends,
        note,
        // Dotted in streamer mode; copied whole.
        link: link_of(modal, store).map(|link| crate::streamer::link(ui, &link)),
        pending,
        copied: modal.copied.is_some(),
        expiry: expiry(modal.options),
        simulated,
        settings: modal.settings.map(|options| Settings {
            ages: INVITE_AGES
                .iter()
                .map(|(_, label)| t(label).to_owned())
                .collect(),
            age: options.age,
            uses: INVITE_USES
                .iter()
                .map(|(_, label)| t(label).to_owned())
                .collect(),
            use_limit: options.uses,
            temporary: options.temporary,
            open: modal.select,
            generating: modal.making.is_some(),
        }),
    };
    parts::modal(root, &view, context.window, look)
}
