//! A Spotify link in a conversation, and what Spotify's public embed player
//! says of it: Rustcord draws that player itself, with its 30-second
//! previews, in place of a web page.

use crate::{RemoteImage, Rgb};

/// What a Spotify link names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpotifyKind {
    Track,
    Album,
    Playlist,
    Episode,
    Show,
    Artist,
}

impl SpotifyKind {
    const ALL: [Self; 6] = [
        Self::Track,
        Self::Album,
        Self::Playlist,
        Self::Episode,
        Self::Show,
        Self::Artist,
    ];

    /// Its word in Spotify's addresses.
    pub fn path(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Album => "album",
            Self::Playlist => "playlist",
            Self::Episode => "episode",
            Self::Show => "show",
            Self::Artist => "artist",
        }
    }

    /// Its player lists tracks under the cover.
    pub fn listed(self) -> bool {
        matches!(self, Self::Album | Self::Playlist | Self::Artist)
    }
}

/// A link to something on `open.spotify.com`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpotifyLink {
    pub kind: SpotifyKind,
    /// Its 22-character base-62 ID.
    pub id: String,
}

impl SpotifyLink {
    /// `https://open.spotify.com/[intl-xx/][embed/]{kind}/{id}`, with any
    /// query (Spotify's `?si=` share mark) left out.
    pub fn parse(url: &str) -> Option<Self> {
        let path = url
            .strip_prefix("https://open.spotify.com/")?
            .split(['?', '#'])
            .next()?;
        let mut parts = path.split('/').peekable();
        parts.next_if(|part| part.starts_with("intl-"));
        parts.next_if_eq(&"embed");
        let word = parts.next()?;
        let kind = SpotifyKind::ALL
            .into_iter()
            .find(|kind| kind.path() == word)?;
        let id = parts.next()?;
        let valid = id.len() == 22 && id.bytes().all(|byte| byte.is_ascii_alphanumeric());
        (valid && parts.all(str::is_empty)).then(|| Self {
            kind,
            id: id.into(),
        })
    }

    /// Its page on Spotify, which «Guardar en Spotify» and full playback open.
    pub fn page(&self) -> String {
        format!("https://open.spotify.com/{}/{}", self.kind.path(), self.id)
    }

    /// The page of Spotify's embed player, whose data the card is drawn from.
    pub fn embed(&self) -> String {
        format!(
            "https://open.spotify.com/embed/{}/{}",
            self.kind.path(),
            self.id
        )
    }
}

/// What Spotify's embed player shows of a link.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpotifyEntity {
    pub title: String,
    /// Its artists, its owner or its show.
    pub subtitle: String,
    pub cover: Option<RemoteImage>,
    /// The card's color, as Spotify takes it from the cover.
    pub color: Option<Rgb>,
    /// What plays: the track or episode itself, or the list's tracks.
    pub tracks: Vec<SpotifyTrack>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpotifyTrack {
    pub title: String,
    pub subtitle: String,
    pub duration_ms: u64,
    pub explicit: bool,
    /// Its 30-second preview on Spotify's own host, when it has one.
    pub preview: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_recognized_with_or_without_their_language_and_share_mark() {
        let id = "3U5Re9YvytJ2PUXf8mTwgY";
        for url in [
            format!("https://open.spotify.com/playlist/{id}"),
            format!("https://open.spotify.com/playlist/{id}?si=abc123"),
            format!("https://open.spotify.com/intl-es/playlist/{id}"),
            format!("https://open.spotify.com/embed/playlist/{id}"),
            format!("https://open.spotify.com/playlist/{id}/"),
        ] {
            let link = SpotifyLink::parse(&url).unwrap_or_else(|| panic!("{url}"));
            assert_eq!(link.kind, SpotifyKind::Playlist);
            assert_eq!(
                link.page(),
                format!("https://open.spotify.com/playlist/{id}")
            );
            assert_eq!(
                link.embed(),
                format!("https://open.spotify.com/embed/playlist/{id}")
            );
        }
        for url in [
            format!("http://open.spotify.com/track/{id}"),
            format!("https://open.spotify.com.evil.test/track/{id}"),
            format!("https://open.spotify.com/user/{id}"),
            format!("https://open.spotify.com/track/{id}/extra"),
            "https://open.spotify.com/track/short".into(),
            "https://open.spotify.com/track/3U5Re9YvytJ2PUXf8mTw-Y".into(),
            "https://open.spotify.com/".into(),
        ] {
            assert_eq!(SpotifyLink::parse(&url), None, "{url}");
        }
        assert!(SpotifyKind::Album.listed() && !SpotifyKind::Track.listed());
    }
}
