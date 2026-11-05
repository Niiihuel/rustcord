//! The cards of Spotify links: what Spotify's public embed player shows of
//! a link shown in a conversation (its title, cover, color and tracks with
//! their 30-second previews), read from the data its page carries. No
//! Spotify account, token or cookie: the anonymous token that page carries
//! is never read. Its pictures and previews load only from Spotify's hosts.

use std::{sync::Arc, time::Duration};

use rustcord_model::spotify::{SpotifyEntity, SpotifyLink, SpotifyTrack};
use rustcord_model::{ImageKind, RemoteImage, Rgb};
use serde_json::Value;
use tokio::{runtime::Handle, sync::mpsc, task::JoinHandle};

/// The most lookups in flight; more answer at once with nothing.
pub(crate) const IN_FLIGHT: usize = 4;
/// A playlist's page with 100 tracks is about 170 KiB.
const PAGE_BYTES: usize = 1024 * 1024;
/// Spotify's embed lists up to 100.
const TRACKS: usize = 100;
/// The longest title or subtitle kept, in characters.
const TEXT: usize = 200;
/// The script that carries the page's data.
const DATA: &str = r#"<script id="__NEXT_DATA__" type="application/json">"#;

/// What a lookup found for `link`: `None` when Spotify gave nothing that
/// can be drawn, and the message keeps the card its provider sent.
#[derive(Debug)]
pub struct SpotifyResult {
    pub request: u64,
    pub link: SpotifyLink,
    pub entity: Option<SpotifyEntity>,
}

pub(crate) struct Spotify {
    runtime: Handle,
    client: Option<reqwest::Client>,
    wake: Arc<dyn Fn() + Send + Sync>,
    sender: mpsc::Sender<SpotifyResult>,
    receiver: mpsc::Receiver<SpotifyResult>,
    tasks: Vec<JoinHandle<()>>,
}

impl Spotify {
    pub fn new(runtime: Handle, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (sender, receiver) = mpsc::channel(IN_FLIGHT);
        Self {
            runtime,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .user_agent(concat!(
                    "Rustcord/",
                    env!("CARGO_PKG_VERSION"),
                    " link preview"
                ))
                .build()
                .ok(),
            wake,
            sender,
            receiver,
            tasks: Vec::new(),
        }
    }

    pub fn start(&mut self, request: u64, link: SpotifyLink) {
        self.tasks.retain(|task| !task.is_finished());
        let client = self.client.clone().filter(|_| self.tasks.len() < IN_FLIGHT);
        let (sender, wake) = (self.sender.clone(), self.wake.clone());
        self.tasks.push(self.runtime.spawn(async move {
            let entity = match client {
                Some(client) => lookup(&client, &link).await,
                None => None,
            };
            let result = SpotifyResult {
                request,
                link,
                entity,
            };
            if sender.send(result).await.is_ok() {
                wake();
            }
        }));
    }

    pub fn poll(&mut self) -> Option<SpotifyResult> {
        self.receiver.try_recv().ok()
    }
}

impl Drop for Spotify {
    fn drop(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

async fn lookup(client: &reqwest::Client, link: &SpotifyLink) -> Option<SpotifyEntity> {
    let url = url::Url::parse(&link.embed()).ok()?;
    // No account headers or cookies: the page is public.
    let mut response = client.get(url).send().await.ok()?;
    if response.status() != reqwest::StatusCode::OK
        || response
            .content_length()
            .is_some_and(|length| length > PAGE_BYTES as u64)
    {
        return None;
    }
    let mut page = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if page.len() + chunk.len() > PAGE_BYTES {
            return None;
        }
        page.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || parse(&page))
        .await
        .ok()
        .flatten()
}

/// A URL on a host `allowed` names, checked as every fetched resource is.
fn on_host(raw: &str, allowed: impl Fn(&str) -> bool) -> Option<url::Url> {
    let host = url::Url::parse(raw)
        .ok()?
        .host_str()
        .filter(|host| allowed(host))?
        .to_owned();
    crate::validated_resource_url(raw, &[&format!("https://{host}")], false)
}

/// A cover on Spotify's picture hosts, as its embed names them.
pub(crate) fn image_url(raw: &str) -> Option<url::Url> {
    on_host(raw, |host| {
        let edge = host
            .strip_prefix("image-cdn-")
            .and_then(|rest| rest.strip_suffix(".spotifycdn.com"));
        matches!(host, "i.scdn.co" | "mosaic.scdn.co")
            || edge.is_some_and(|edge| {
                (1..=4).contains(&edge.len()) && edge.bytes().all(|b| b.is_ascii_lowercase())
            })
    })
}

/// A 30-second preview on Spotify's audio host.
pub(crate) fn preview_url(raw: &str) -> Option<url::Url> {
    on_host(raw, |host| host == "p.scdn.co")
}

/// A title or subtitle, without control characters, at most [`TEXT`] long.
fn text(value: &Value) -> Option<String> {
    let text: String = value
        .as_str()?
        .chars()
        .filter(|c| !c.is_control())
        .take(TEXT)
        .collect();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn preview(item: &Value) -> Option<String> {
    let raw = item["audioPreview"]["url"].as_str()?;
    preview_url(raw).map(|_| raw.to_owned())
}

fn track(item: &Value) -> Option<SpotifyTrack> {
    let artists = || {
        let names: Vec<&str> = item["artists"]
            .as_array()?
            .iter()
            .take(8)
            .filter_map(|artist| artist["name"].as_str())
            .collect();
        text(&Value::from(names.join(", ")))
    };
    Some(SpotifyTrack {
        title: text(&item["title"]).or_else(|| text(&item["name"]))?,
        subtitle: text(&item["subtitle"]).or_else(artists).unwrap_or_default(),
        duration_ms: item["duration"]
            .as_u64()
            .unwrap_or(0)
            .min(u64::from(u32::MAX)),
        explicit: item["isExplicit"].as_bool().unwrap_or(false),
        preview: preview(item),
    })
}

/// The cover closest to 300 px wide among those the entity names.
fn cover(entity: &Value) -> Option<RemoteImage> {
    let identity = entity["visualIdentity"]["image"].as_array();
    let sources = entity["coverArt"]["sources"].as_array();
    let show = entity["relatedEntityCoverArt"].as_array();
    [identity, sources, show]
        .into_iter()
        .flatten()
        .flatten()
        .take(16)
        .filter_map(|image| {
            let url = image["url"]
                .as_str()
                .filter(|url| image_url(url).is_some())?;
            let width = image["maxWidth"].as_u64().or(image["width"].as_u64());
            Some((width.unwrap_or(300).abs_diff(300), url))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, url)| RemoteImage {
            url: url.into(),
            kind: ImageKind::AttachmentPreview,
        })
}

/// What the embed page's data says of its entity.
pub(crate) fn parse(page: &[u8]) -> Option<SpotifyEntity> {
    let page = std::str::from_utf8(page).ok()?;
    let start = page.find(DATA)? + DATA.len();
    let data = &page[start..];
    let data = &data[..data.find("</script>")?];
    let value: Value = serde_json::from_str(data).ok()?;
    let entity = &value["props"]["pageProps"]["state"]["data"]["entity"];
    let first = track(entity)?;
    let tracks = match entity["trackList"].as_array() {
        Some(list) => list.iter().take(TRACKS).filter_map(track).collect(),
        None => vec![first.clone()],
    };
    let channel = |name: &str| {
        entity["visualIdentity"]["backgroundBase"][name]
            .as_u64()
            .filter(|value| *value <= 255)
            .map(|value| value as u32)
    };
    let color = (|| {
        Some(Rgb(channel("red")? << 16
            | channel("green")? << 8
            | channel("blue")?))
    })();
    Some(SpotifyEntity {
        title: first.title,
        subtitle: first.subtitle,
        cover: cover(entity),
        color,
        tracks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(entity: &str) -> Vec<u8> {
        format!(
            "<html><body><script id=\"__NEXT_DATA__\" type=\"application/json\">\
             {{\"props\":{{\"pageProps\":{{\"state\":{{\"data\":{{\"entity\":{entity}}},\
             \"settings\":{{\"session\":{{\"accessToken\":\"never-read\"}}}}}}}}}}}}\
             </script></body></html>"
        )
        .into_bytes()
    }

    #[test]
    fn a_playlist_names_its_cover_color_and_tracks_with_their_previews() {
        let entity = parse(&page(include_str!("../fixtures/spotify-playlist.json"))).unwrap();
        assert_eq!(entity.title, "Trap Hood");
        assert_eq!(entity.subtitle, "Drex");
        assert_eq!(entity.color, Some(Rgb(0x66_50_00)));
        assert_eq!(
            entity.cover.unwrap().url,
            "https://image-cdn-fa.spotifycdn.com/image/ab67706c0000da8437f5d24c615290ccb9b6bd79"
        );
        assert_eq!(
            entity.tracks.len(),
            3,
            "the one without a title is left out"
        );
        let first = &entity.tracks[0];
        assert_eq!(
            (first.title.as_str(), first.subtitle.as_str()),
            ("Straightenin", "Migos")
        );
        assert_eq!((first.duration_ms, first.explicit), (255_023, true));
        assert_eq!(
            first.preview.as_deref(),
            Some("https://p.scdn.co/mp3-preview/36e4d4c60bc8fa8654a068fa5dad4434c373fc40")
        );
        // A preview elsewhere, or none, plays nothing.
        assert_eq!(entity.tracks[1].preview, None);
        assert_eq!(entity.tracks[2].preview, None);
        assert_eq!(entity.tracks[2].title, "Línea sin controles");
    }

    #[test]
    fn a_track_is_its_own_list_and_names_its_artists() {
        let entity = parse(&page(include_str!("../fixtures/spotify-track.json"))).unwrap();
        assert_eq!(entity.title, "Straightenin");
        assert_eq!(entity.subtitle, "Migos, Quavo");
        assert_eq!(entity.tracks.len(), 1);
        assert_eq!(entity.tracks[0].subtitle, "Migos, Quavo");
        assert!(entity.tracks[0].preview.is_some());
        // The 300 px picture, of three.
        assert!(entity
            .cover
            .unwrap()
            .url
            .ends_with("ab67616d00001e02f89d00155b6a066650e5460d"));
    }

    #[test]
    fn pages_without_their_data_or_a_title_give_nothing() {
        assert!(parse(b"<html>nothing here</html>").is_none());
        assert!(parse(&page("{\"title\": \"\"}")).is_none());
        assert!(parse(&page("not json")).is_none());
        let mut long = page(include_str!("../fixtures/spotify-track.json"));
        long.truncate(long.len() - 20);
        assert!(parse(&long).is_none(), "an unterminated script");
    }

    #[test]
    fn only_spotify_hosts_serve_pictures_and_previews() {
        for ok in [
            "https://i.scdn.co/image/ab67616d0000b273",
            "https://mosaic.scdn.co/640/ab67616d",
            "https://image-cdn-ak.spotifycdn.com/image/ab67616d",
            "https://image-cdn-fa.spotifycdn.com/image/ab67616d",
        ] {
            assert!(image_url(ok).is_some(), "{ok}");
        }
        for bad in [
            "http://i.scdn.co/image/a",
            "https://i.scdn.co:444/image/a",
            "https://user@i.scdn.co/image/a",
            "https://image-cdn-toolong.spotifycdn.com/image/a",
            "https://image-cdn-.spotifycdn.com/image/a",
            "https://evil.test/i.scdn.co",
            "https://p.scdn.co/mp3-preview/a",
        ] {
            assert!(image_url(bad).is_none(), "{bad}");
        }
        assert!(preview_url("https://p.scdn.co/mp3-preview/a").is_some());
        assert!(preview_url("https://i.scdn.co/mp3-preview/a").is_none());
        assert!(preview_url("https://p.scdn.co/mp3-preview/a#x").is_none());
    }

    #[test]
    fn titles_lose_control_characters_and_are_bounded() {
        let long = "a".repeat(5 * TEXT);
        let entity = parse(&page(&format!(
            "{{\"title\": \"{long}\", \"subtitle\": \"un\\u0007o\"}}"
        )))
        .unwrap();
        assert_eq!(entity.title.chars().count(), TEXT);
        assert_eq!(entity.subtitle, "uno");
        assert_eq!(entity.tracks.len(), 1);
        assert_eq!(entity.color, None);
    }
}
