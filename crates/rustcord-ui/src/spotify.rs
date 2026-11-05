//! Spotify links in the timeline show Spotify's player, drawn by Rustcord,
//! in place of their card, as Discord's client shows them. The host looks
//! up what Spotify's embed shows of each link in view, a few at a time,
//! cached; its 30-second previews play through the voice message's player,
//! one at a time. The title, the logo and «Guardar en Spotify» open the
//! link through the timeline's link handler, which asks first.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use limn::UiTree;
use rustcord_components::message::{SpotifyNow, SpotifyPlayer};
use rustcord_model::spotify::{SpotifyEntity, SpotifyLink};
use rustcord_model::{ChannelId, Embed, Message, MessageId, RemoteImage};
use rustcord_provider::RequestId;
use rustcord_state::Store;

use crate::{timeline_id, Intent, Ui, VoiceState};

/// How long a lookup that found nothing waits before another.
const RETRY: Duration = Duration::from_secs(5 * 60);
/// The most lookups in flight at once, as the core runs them.
const IN_FLIGHT: usize = 4;
/// A lookup without an answer by then frees its place.
const LOST: Duration = Duration::from_secs(30);
/// The most links remembered; past it the answers start over.
const LIMIT: usize = 256;

#[derive(Debug, Default)]
pub(crate) struct SpotifyCards {
    entries: HashMap<SpotifyLink, Entry>,
}

#[derive(Debug, Default)]
struct Entry {
    pending: Option<(RequestId, Instant)>,
    /// When it came, and what Spotify showed, if anything.
    answer: Option<(Instant, Option<Arc<SpotifyEntity>>)>,
}

impl Entry {
    fn in_flight(&self, now: Instant) -> bool {
        self.pending
            .is_some_and(|(_, since)| now.saturating_duration_since(since) < LOST)
    }

    /// One under way, or an answer that stands: what was found stays.
    fn settled(&self, now: Instant) -> bool {
        self.in_flight(now)
            || self.answer.as_ref().is_some_and(|(at, found)| {
                found.is_some() || now.saturating_duration_since(*at) < RETRY
            })
    }
}

/// The preview playing: its track among those of the `embed`th card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpotifyPlaying {
    pub embed: usize,
    pub track: usize,
}

fn link(embed: &Embed) -> Option<SpotifyLink> {
    SpotifyLink::parse(embed.url.as_deref()?)
}

fn entity<'a>(ui: &'a Ui, embed: &Embed) -> Option<&'a Arc<SpotifyEntity>> {
    let entry = ui.spotify_cards.entries.get(&link(embed)?)?;
    entry.answer.as_ref()?.1.as_ref()
}

/// The lookups the Spotify links in view need now, at most four in flight.
/// The host starts each and hands its answer to [`spotify_card_result`].
pub fn wanted_spotify_cards(
    ui: &mut Ui,
    store: &Store,
    tree: &UiTree,
    now: Instant,
) -> Vec<(RequestId, SpotifyLink)> {
    let Some(channel) = ui.channel() else {
        return Vec::new();
    };
    let (Some(timeline), Some(rows)) = (
        store.timeline(channel),
        tree.visible_rows(&timeline_id(channel)),
    ) else {
        return Vec::new();
    };
    let mut wanted = Vec::new();
    for message in timeline.messages().iter().take(rows.end).skip(rows.start) {
        for link in message.embeds.iter().filter_map(link) {
            let cards = &mut ui.spotify_cards;
            if cards
                .entries
                .get(&link)
                .is_some_and(|entry| entry.settled(now))
            {
                continue;
            }
            if cards.entries.values().filter(|e| e.in_flight(now)).count() >= IN_FLIGHT {
                return wanted;
            }
            if cards.entries.len() >= LIMIT && !cards.entries.contains_key(&link) {
                cards.entries.retain(|_, entry| entry.in_flight(now));
            }
            let request = ui.fresh_request();
            ui.spotify_cards
                .entries
                .entry(link.clone())
                .or_default()
                .pending = Some((request, now));
            wanted.push((request, link));
        }
    }
    wanted
}

/// Keep the answer of the latest lookup of `link`; any other is stale.
pub fn spotify_card_result(
    ui: &mut Ui,
    request: RequestId,
    link: &SpotifyLink,
    entity: Option<SpotifyEntity>,
    now: Instant,
) {
    if let Some(entry) = ui
        .spotify_cards
        .entries
        .get_mut(link)
        .filter(|entry| entry.pending.is_some_and(|(asked, _)| asked == request))
    {
        entry.pending = None;
        entry.answer = Some((now, entity.map(Arc::new)));
    }
}

/// The covers of the Spotify players of `message`.
pub(crate) fn images<'a>(
    ui: &'a Ui,
    message: &'a Message,
) -> impl Iterator<Item = &'a RemoteImage> {
    message
        .embeds
        .iter()
        .filter_map(|embed| entity(ui, embed)?.cover.as_ref())
}

/// The players of `message`'s Spotify links whose data arrived.
pub(crate) fn players(ui: &Ui, message: &Message) -> Vec<SpotifyPlayer> {
    message
        .embeds
        .iter()
        .enumerate()
        .filter_map(|(index, embed)| {
            let entity = entity(ui, embed)?.clone();
            let player = ui
                .voice_player
                .as_ref()
                .filter(|player| player.message == message.id);
            let now = player.and_then(|player| {
                let playing = player.spotify.filter(|playing| playing.embed == index)?;
                Some(SpotifyNow {
                    track: playing.track,
                    loading: player.state == VoiceState::Loading,
                    playing: player.state == VoiceState::Playing,
                    position: player.position,
                    duration: player.duration,
                    error: match &player.state {
                        VoiceState::Failed(error) => Some(error.clone()),
                        _ => None,
                    },
                })
            });
            Some(SpotifyPlayer {
                embed: index,
                link: link(embed)?,
                cover: ui.picture(entity.cover.as_ref()),
                entity,
                now,
            })
        })
        .collect()
}

/// The preview of the `track`th track of the `embed`th card of `message`,
/// while the conversation shows it.
pub(crate) fn preview(
    ui: &Ui,
    store: &Store,
    channel: ChannelId,
    message: MessageId,
    playing: SpotifyPlaying,
) -> Option<String> {
    store
        .channel(channel)
        .filter(|channel| Store::visible(channel))?;
    let found = store.timeline(channel)?.get(message)?;
    let track = entity(ui, found.embeds.get(playing.embed)?)?
        .tracks
        .get(playing.track)?;
    track.preview.clone()
}

/// The player's `what` (`{embed}:{verb}`) of `message`: play or pause,
/// the previous or next track with a preview, or one track of the list.
pub(crate) fn act(
    ui: &mut Ui,
    store: &Store,
    channel: ChannelId,
    message: MessageId,
    what: &str,
) -> Vec<Intent> {
    let Some((Ok(embed), verb)) = what
        .split_once(':')
        .map(|(embed, verb)| (embed.parse::<usize>(), verb))
    else {
        return Vec::new();
    };
    let Some(entity) = store
        .timeline(channel)
        .and_then(|timeline| timeline.get(message))
        .and_then(|found| entity(ui, found.embeds.get(embed)?).cloned())
    else {
        return Vec::new();
    };
    let current = ui
        .voice_player
        .as_ref()
        .filter(|player| player.message == message)
        .and_then(|player| player.spotify)
        .filter(|playing| playing.embed == embed)
        .map(|playing| playing.track);
    let count = entity.tracks.len();
    let has = |index: &usize| entity.tracks[*index].preview.is_some();
    let from = current.unwrap_or(0);
    let track = match verb {
        "play" if current.is_some() => return toggle(ui, store, channel, message),
        "play" => (0..count).find(has),
        "previous" => (0..from).rev().find(has).or(current),
        // Past the last, the first again, as Spotify's player does.
        "next" => (from + 1..count).chain(0..from).find(has),
        _ => match verb.strip_prefix("track:").and_then(|n| n.parse().ok()) {
            Some(track) if Some(track) == current => return toggle(ui, store, channel, message),
            Some(track) => Some(track).filter(|track| *track < count && has(track)),
            None => None,
        },
    };
    let Some(track) = track else {
        return Vec::new();
    };
    let playing = SpotifyPlaying { embed, track };
    match preview(ui, store, channel, message, playing) {
        Some(url) => crate::voice_messages::play_preview(ui, store, channel, message, playing, url),
        None => Vec::new(),
    }
}

/// Pause or go on with the preview playing; one that failed is fetched again.
fn toggle(ui: &mut Ui, store: &Store, channel: ChannelId, message: MessageId) -> Vec<Intent> {
    let Some(player) = ui.voice_player.as_mut() else {
        return Vec::new();
    };
    player.state = match player.state {
        VoiceState::Playing => VoiceState::Paused,
        VoiceState::Paused => VoiceState::Playing,
        VoiceState::Loading => VoiceState::Loading,
        VoiceState::Failed(_) => {
            let Some(playing) = player.spotify else {
                return Vec::new();
            };
            return match preview(ui, store, channel, message, playing) {
                Some(url) => {
                    crate::voice_messages::play_preview(ui, store, channel, message, playing, url)
                }
                None => Vec::new(),
            };
        }
    };
    Vec::new()
}
