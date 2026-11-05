//! A Spotify link's player, in place of its card: the look of Spotify's own
//! embed player (its cover on the cover's color, title, «Guardar en
//! Spotify», the logo, the preview's mark and its controls, and a list's
//! tracks under it), drawn by Rustcord. Only the 30-second previews play
//! here; the title, the logo and «Guardar en Spotify» open the link.

use std::sync::Arc;

use limn::{
    Access, Color, Component, Element, FontFamily, Icon, Point, RasterImage, Role, ScrollArea,
    ScrollbarVisibility,
};
use rustcord_i18n::{t, tr};
use rustcord_model::spotify::{SpotifyEntity, SpotifyLink};
use rustcord_model::MessageId;

use super::embeds::opens;
use crate::{centered, gap, icon, line, strong_line};

/// Spotify's embed player is 456 px wide, as its oEmbed says.
const WIDTH: f32 = 456.0;
const PAD: f32 = 16.0;
const COVER: f32 = 152.0;
/// The list's height, what Spotify's 352 px player leaves under the cover.
const LIST: f32 = 160.0;
const ROW: f32 = 50.0;
const PLAY: f32 = 40.0;
const CONTROL: f32 = 32.0;
const WHITE: Color = Color::rgb(255, 255, 255);
/// Spotify's own gray, for a card whose color is not known.
const GRAY: Color = Color::rgb(40, 40, 40);

/// What the timeline knows of a Spotify link of a message.
#[derive(Clone, Debug)]
pub struct SpotifyPlayer {
    /// The message's card it takes the place of.
    pub embed: usize,
    pub link: SpotifyLink,
    pub entity: Arc<SpotifyEntity>,
    pub cover: Option<RasterImage>,
    /// Its preview playing or paused, when it is one of this card's.
    pub now: Option<SpotifyNow>,
}

/// The preview being played, as its card shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpotifyNow {
    /// Its index among the entity's tracks.
    pub track: usize,
    pub loading: bool,
    pub playing: bool,
    /// Seconds into it.
    pub position: f32,
    pub duration: Option<f32>,
    pub error: Option<String>,
}

/// The action `verb` of the player of the `embed`th card of `message`:
/// `play`, `previous`, `next` or `track:{index}`.
pub fn spotify_action(message: MessageId, embed: usize, verb: &str) -> String {
    format!("msg:{message}:spotify:{embed}:{verb}")
}

/// `mm:ss`, as Spotify's list shows a track's length.
fn length(ms: u64) -> String {
    let seconds = ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

/// A round control of `side` px on `ground`, reporting `id` unless `passive`.
fn control(
    id: String,
    glyph: Icon,
    color: Color,
    ground: Option<Color>,
    side: f32,
    name: &str,
    passive: bool,
) -> Element {
    let mut button = Element::panel(centered(icon(glyph, color))).size(Some(side), Some(side));
    if let Some(ground) = ground {
        button = button.background(ground).radius(side / 2.0);
    }
    if passive {
        return button;
    }
    Element::clickable(id, button)
        .size(Some(side), Some(side))
        .accessible(Access::new(Role::Button).name(name))
}

/// Opens the link through the timeline's link handler, which asks first.
fn opening(child: Element, card: &SpotifyPlayer, name: &str, passive: bool) -> Element {
    if passive {
        return child;
    }
    Element::clickable(opens(&card.link.page()), child)
        .accessible(Access::new(Role::Link).name(name))
}

/// The «E» Spotify marks explicit tracks with.
fn explicit(ground: Color) -> Element {
    Element::panel(centered(
        Element::text("E")
            .font_family(FontFamily::SansBold)
            .font_size(9.0)
            .foreground(ground),
    ))
    .size(Some(14.0), Some(14.0))
    .background(Color::rgba(255, 255, 255, 190))
    .radius(2.0)
    .accessible(Access::new(Role::Image).name(t("Explícito")))
}

pub(super) fn player(
    card: &SpotifyPlayer,
    message: MessageId,
    width: f32,
    passive: bool,
) -> Element {
    let width = width.min(WIDTH);
    let inner = width - 2.0 * PAD;
    // A narrower card shrinks its cover, then moves its controls under it.
    let side = (inner * 0.36).clamp(64.0, COVER);
    let under = inner - side - PAD < 220.0;
    let ground = card.entity.color.map_or(GRAY, |color| {
        let (r, g, b) = color.channels();
        Color::rgb(r, g, b)
    });
    let soft = Color::rgba(255, 255, 255, 179);
    let action = |verb: &str| spotify_action(message, card.embed, verb);
    let tracks = &card.entity.tracks;
    let playable = tracks.iter().any(|track| track.preview.is_some());
    let now = card.now.as_ref().filter(|now| now.track < tracks.len());
    let cover = match &card.cover {
        Some(image) => Element::raster_image(image.clone().cover()),
        None => Element::panel(centered(icon(Icon::MusicNote, soft)))
            .background(Color::rgba(0, 0, 0, 60)),
    }
    .size(Some(side), Some(side))
    .radius(8.0)
    .shadow(Color::rgba(0, 0, 0, 90), Point::new(0.0, 4.0), 24.0, 0.0);
    let logo = crate::identity::logo("spotify")
        .map_or_else(|| icon(Icon::MusicNote, WHITE), Element::raster_image);
    let top = Element::row(vec![
        Element::column(vec![
            opening(
                Element::truncated_text(card.entity.title.clone())
                    .font_family(FontFamily::SansBold)
                    .font_size(if under { 18.0 } else { 22.0 })
                    .foreground(WHITE),
                card,
                &card.entity.title,
                passive,
            ),
            line(card.entity.subtitle.clone(), 14.0, soft),
        ])
        .gap(4.0)
        .flex(1.0),
        opening(
            logo.size(Some(24.0), Some(24.0)),
            card,
            t("Abrir en Spotify"),
            passive,
        ),
    ])
    .gap(8.0);
    let save = opening(
        Element::row(vec![
            icon(Icon::Custom(crate::server_menu::glyphs::CIRCLE_PLUS), WHITE)
                .size(Some(16.0), Some(16.0))
                .centered_y(),
            strong_line(t("Guardar en Spotify"), 14.0, WHITE).centered_y(),
        ])
        .gap(6.0),
        card,
        t("Guardar en Spotify"),
        passive,
    );
    let chip = Element::panel(centered(strong_line(t("Muestra"), 11.0, WHITE)))
        .padding(4.0)
        .size(None, Some(20.0))
        .background(Color::rgba(0, 0, 0, 80))
        .radius(4.0);
    let (glyph, name) = match now {
        Some(now) if now.loading => (Icon::PlayFill, t("Cargando muestra")),
        Some(now) if now.playing => (Icon::PauseFill, t("Pausar muestra")),
        _ => (Icon::PlayFill, t("Reproducir muestra")),
    };
    let mut controls = vec![chip.centered_y(), Element::spacer().flex(1.0)];
    if tracks.len() > 1 {
        controls.push(control(
            action("previous"),
            Icon::SkipBack,
            WHITE,
            None,
            CONTROL,
            t("Pista anterior"),
            passive || !playable,
        ));
        controls.push(control(
            action("next"),
            Icon::SkipForward,
            WHITE,
            None,
            CONTROL,
            t("Pista siguiente"),
            passive || !playable,
        ));
    }
    // «…» opens the link, as the title does; it goes first where room is short.
    if inner >= 240.0 {
        controls.push(opening(
            Element::panel(centered(icon(Icon::More, WHITE))).size(Some(CONTROL), Some(CONTROL)),
            card,
            t("Abrir en Spotify"),
            passive,
        ));
    }
    controls.push(control(
        action("play"),
        glyph,
        Color::rgb(0, 0, 0),
        Some(if playable { WHITE } else { soft }),
        PLAY,
        name,
        passive || !playable,
    ));
    let controls = Element::row(controls).gap(4.0);
    let (beside, below) = if under {
        (None, Some(controls))
    } else {
        (Some(controls), None)
    };
    let mut column = vec![top, gap(0.0, 12.0), save, Element::spacer().flex(1.0)];
    column.extend(beside);
    let beside = Element::column(column).flex(1.0).size(None, Some(side));
    let mut parts = vec![Element::row(vec![cover, beside]).gap(PAD)];
    parts.extend(below);
    if let Some(now) = now.filter(|now| now.playing || now.position > 0.0) {
        let done = (now.position / now.duration.unwrap_or(30.0).max(1.0)).clamp(0.0, 1.0);
        parts.push(
            Element::row(vec![
                Element::panel(Element::spacer())
                    .size(Some(inner * done), Some(4.0))
                    .background(WHITE),
                Element::panel(Element::spacer())
                    .size(None, Some(4.0))
                    .flex(1.0)
                    .background(Color::rgba(255, 255, 255, 77)),
            ])
            .clip_children()
            .radius(2.0)
            .accessible(
                Access::new(Role::ProgressIndicator)
                    .name(t("Muestra"))
                    .value(done),
            ),
        );
    }
    if let Some(error) = now.and_then(|now| now.error.as_deref()) {
        parts.push(line(
            tr!("No se pudo reproducir: {error}", error),
            12.0,
            WHITE,
        ));
    }
    if card.link.kind.listed() && !tracks.is_empty() {
        parts.push(list(card, message, now, ground, soft, passive));
    }
    Element::column(parts)
        .gap(12.0)
        .padding(PAD)
        .size(Some(width), None)
        .background(ground)
        .radius(12.0)
        .accessible(Access::new(Role::Group).name(tr!(
            "{title} en Spotify",
            title = card.entity.title.as_str()
        )))
}

/// The list's tracks, each playing its preview when clicked.
fn list(
    card: &SpotifyPlayer,
    message: MessageId,
    now: Option<&SpotifyNow>,
    ground: Color,
    soft: Color,
    passive: bool,
) -> Element {
    let rows = card
        .entity
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let current = now.is_some_and(|now| now.track == index);
            let ink = if track.preview.is_some() { WHITE } else { soft };
            let mut under = Vec::new();
            if track.explicit {
                under.push(explicit(ground).centered_y());
            }
            under.push(line(track.subtitle.clone(), 13.0, soft).flex(1.0));
            let number = if current {
                icon(Icon::MusicNote, WHITE).size(Some(14.0), Some(14.0))
            } else {
                line((index + 1).to_string(), 14.0, soft)
            };
            let title = if current {
                strong_line(track.title.clone(), 15.0, WHITE)
            } else {
                line(track.title.clone(), 15.0, ink)
            };
            let row = Element::row(vec![
                Element::panel(centered(number)).size(Some(24.0), Some(ROW)),
                Element::column(vec![title, Element::row(under).gap(6.0)])
                    .gap(2.0)
                    .flex(1.0)
                    .centered_y(),
                line(length(track.duration_ms), 13.0, soft).centered_y(),
            ])
            .gap(10.0)
            .padding(4.0)
            .size(None, Some(ROW))
            .radius(6.0);
            let row = if current {
                row.background(Color::rgba(255, 255, 255, 26))
            } else {
                row
            };
            if passive || track.preview.is_none() {
                return row;
            }
            Element::clickable(
                spotify_action(message, card.embed, &format!("track:{index}")),
                row,
            )
            .hover_background(Color::rgba(255, 255, 255, 20))
            .accessible(Access::new(Role::Button).name(track.title.as_str()))
        })
        .collect();
    ScrollArea::new(
        spotify_action(message, card.embed, "list"),
        Element::column(rows),
    )
    .visibility(ScrollbarVisibility::AutoHideUnlessHovered {
        hold_ms: 600,
        fade_ms: 200,
    })
    .tint(soft)
    .render()
    .size(None, Some(LIST))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};
    use limn::Size;
    use rustcord_model::spotify::{SpotifyKind, SpotifyTrack};
    use rustcord_model::Rgb;

    fn card(kind: SpotifyKind, now: Option<SpotifyNow>) -> SpotifyPlayer {
        let track = |title: &str, preview: bool| SpotifyTrack {
            title: title.into(),
            subtitle: "Artista".into(),
            duration_ms: 255_023,
            explicit: true,
            preview: preview.then(|| "https://p.scdn.co/mp3-preview/a".into()),
        };
        SpotifyPlayer {
            embed: 1,
            link: SpotifyLink {
                kind,
                id: "3U5Re9YvytJ2PUXf8mTwgY".into(),
            },
            entity: Arc::new(SpotifyEntity {
                title: "Trap Hood".into(),
                subtitle: "Drex".into(),
                cover: None,
                color: Some(Rgb(0x66_50_00)),
                tracks: vec![track("Uno", true), track("Dos", false), track("Tres", true)],
            }),
            cover: None,
            now,
        }
    }

    fn laid_at(card: &SpotifyPlayer, width: f32, passive: bool) -> limn::Frame {
        let player = player(card, MessageId(9), width, passive);
        frame(
            Element::row(vec![player, Element::spacer().flex(1.0)]),
            Size::new(600.0, 500.0),
        )
        .1
    }

    fn laid(card: &SpotifyPlayer, passive: bool) -> limn::Frame {
        laid_at(card, 600.0, passive)
    }

    #[test]
    fn its_controls_stay_on_the_card_wide_or_narrow() {
        let card = card(SpotifyKind::Playlist, None);
        let region = |laid: &limn::Frame, verb: &str| {
            let id = spotify_action(MessageId(9), 1, verb);
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap()
                .rect
        };
        // Wide: the controls under the title, beside the 152 px cover.
        let wide = laid(&card, false);
        let play = region(&wide, "play");
        assert!(play.x + play.width <= WIDTH - PAD + 0.5, "{play:?}");
        assert!(play.y + play.height <= PAD + COVER + 0.5, "{play:?}");
        assert!(play.x > PAD + COVER, "{play:?}");
        let page = opens("https://open.spotify.com/playlist/3U5Re9YvytJ2PUXf8mTwgY");
        assert_eq!(controls(&wide).iter().filter(|id| **id == page).count(), 4);
        // Narrow, as a small window's timeline: a smaller cover, the
        // controls under it, and no «…».
        let narrow = laid_at(&card, 240.0, false);
        let play = region(&narrow, "play");
        assert!(play.x + play.width <= 240.0 - PAD + 0.5, "{play:?}");
        assert!(play.y > PAD + 64.0, "{play:?}");
        assert_eq!(
            controls(&narrow).iter().filter(|id| **id == page).count(),
            3
        );
    }

    #[test]
    fn a_list_names_its_tracks_and_plays_their_previews_from_the_card() {
        let laid = laid(&card(SpotifyKind::Playlist, None), false);
        let said = texts(&laid);
        for text in [
            "Trap Hood",
            "Drex",
            "Guardar en Spotify",
            "Muestra",
            "1",
            "Uno",
            "Artista",
            "04:15",
            "E",
        ] {
            assert!(said.contains(&text.to_owned()), "{text}: {said:?}");
        }
        let ids = controls(&laid);
        for verb in ["play", "previous", "next", "track:0", "track:2"] {
            assert!(
                ids.contains(&spotify_action(MessageId(9), 1, verb)),
                "{verb}: {ids:?}"
            );
        }
        assert!(
            !ids.contains(&spotify_action(MessageId(9), 1, "track:1")),
            "a track without a preview plays nothing"
        );
        let page = opens("https://open.spotify.com/playlist/3U5Re9YvytJ2PUXf8mTwgY");
        assert!(
            ids.contains(&page),
            "the title, logo, «…» and «Guardar» open it"
        );
        // As wide as Spotify's player, on the cover's color.
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::FillRect { rect, color, .. }
                if *color == Color::rgb(0x66, 0x50, 0) && rect.width == WIDTH
        )));
    }

    #[test]
    fn a_track_has_no_list_and_its_preview_shows_how_far_it_played() {
        let idle = laid(&card(SpotifyKind::Track, None), false);
        assert!(!texts(&idle).contains(&"Uno".to_owned()));
        let playing = card(
            SpotifyKind::Track,
            Some(SpotifyNow {
                track: 0,
                playing: true,
                position: 15.0,
                duration: Some(30.0),
                ..SpotifyNow::default()
            }),
        );
        let shown = laid(&playing, false);
        let bar = shown.commands.iter().find_map(|command| match command {
            limn::DrawCommand::FillRect { rect, color, .. }
                if *color == WHITE && rect.height == 4.0 =>
            {
                Some(rect.width)
            }
            _ => None,
        });
        assert_eq!(bar, Some((WIDTH - 2.0 * PAD) / 2.0));
        let failed = card(
            SpotifyKind::Track,
            Some(SpotifyNow {
                error: Some("sin red".into()),
                ..SpotifyNow::default()
            }),
        );
        assert!(texts(&laid(&failed, false)).contains(&"No se pudo reproducir: sin red".to_owned()));
    }

    #[test]
    fn a_sample_opens_and_plays_nothing() {
        assert!(controls(&laid(&card(SpotifyKind::Album, None), true)).is_empty());
    }

    #[test]
    fn lengths_read_as_spotify_writes_them() {
        assert_eq!(length(255_023), "04:15");
        assert_eq!(length(0), "00:00");
        assert_eq!(length(3_600_000), "60:00");
    }
}
