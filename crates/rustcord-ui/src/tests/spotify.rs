//! Spotify links: looked up while in view, drawn as Spotify's player in
//! place of their card, their previews played one at a time.

use super::*;
use rustcord_model::spotify::{SpotifyEntity, SpotifyKind, SpotifyLink, SpotifyTrack};
use rustcord_model::{Embed, RemoteFile};

const LINK: &str = "https://open.spotify.com/playlist/3U5Re9YvytJ2PUXf8mTwgY?si=x";

fn entity() -> SpotifyEntity {
    let track = |title: &str, preview: Option<&str>| SpotifyTrack {
        title: title.into(),
        subtitle: "Artista".into(),
        duration_ms: 200_000,
        explicit: false,
        preview: preview.map(str::to_owned),
    };
    SpotifyEntity {
        title: "Trap Hood".into(),
        subtitle: "Drex".into(),
        cover: None,
        color: None,
        tracks: vec![
            track("Uno", Some("https://p.scdn.co/mp3-preview/1")),
            track("Sin muestra", None),
            track("Tres", Some("https://p.scdn.co/mp3-preview/3")),
        ],
    }
}

/// #general with a message whose second card is a Spotify link's, in view.
fn shared() -> (Host, ChannelId, MessageId) {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    let mut message = host
        .store
        .timeline(channel)
        .unwrap()
        .messages()
        .last()
        .unwrap()
        .as_ref()
        .clone();
    message.id = MessageId(message.id.0 + 1);
    message.content = format!("Escuchen {LINK}");
    message.embeds = vec![
        Embed {
            title: Some("Otra tarjeta".into()),
            ..Embed::default()
        },
        Embed {
            provider: Some("Spotify".into()),
            title: Some("Tarjeta del proveedor".into()),
            url: Some(LINK.into()),
            ..Embed::default()
        },
    ];
    let id = message.id;
    host.event(Event::MessageCreated {
        message,
        nonce: None,
    });
    host.tree.scroll_to_end(&timeline_id(channel));
    host.frame();
    (host, channel, id)
}

fn click(host: &mut Host, id: MessageId, verb: &str) -> Vec<Intent> {
    update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: rustcord_components::message::spotify_action(id, 1, verb),
        },
        host.now,
    )
}

fn played(intents: &[Intent]) -> Option<(u64, String)> {
    intents.iter().find_map(|intent| match intent {
        Intent::PlayVoice {
            request,
            source: Some(RemoteFile { url }),
            kind: Played::Audio,
            ..
        } => Some((*request, url.clone())),
        _ => None,
    })
}

#[test]
fn a_spotify_link_in_view_is_looked_up_once_and_drawn_as_its_player() {
    let (mut host, _, _) = shared();
    let link = SpotifyLink::parse(LINK).unwrap();
    assert_eq!(link.kind, SpotifyKind::Playlist);
    let now = host.now;
    let wanted = wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, now);
    assert_eq!(
        wanted.iter().map(|(_, link)| link).collect::<Vec<_>>(),
        [&link]
    );
    assert!(
        wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, now).is_empty(),
        "a lookup under way is not repeated"
    );
    // Until it answers, the provider's card shows.
    assert!(host.painted().contains(&"Tarjeta del proveedor".to_owned()));
    // A stale answer is ignored.
    let request = wanted[0].0;
    spotify_card_result(&mut host.ui, request + 99, &link, Some(entity()), now);
    assert!(!host.painted().contains(&"Trap Hood".to_owned()));
    spotify_card_result(&mut host.ui, request, &link, Some(entity()), now);
    let painted = host.painted();
    for text in ["Trap Hood", "Drex", "Guardar en Spotify", "Muestra", "Uno"] {
        assert!(painted.contains(&text.to_owned()), "{text}: {painted:?}");
    }
    assert!(!painted.contains(&"Tarjeta del proveedor".to_owned()));
    assert!(
        painted.contains(&"Otra tarjeta".to_owned()),
        "other cards stay"
    );
    // What was found stands; nothing is asked again.
    let later = now + Duration::from_secs(3600);
    assert!(wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, later).is_empty());
}

#[test]
fn a_lookup_that_found_nothing_keeps_the_card_and_is_retried_later() {
    let (mut host, _, _) = shared();
    let now = host.now;
    let (request, link) = wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, now)
        .pop()
        .unwrap();
    spotify_card_result(&mut host.ui, request, &link, None, now);
    assert!(host.painted().contains(&"Tarjeta del proveedor".to_owned()));
    assert!(wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, now).is_empty());
    let later = now + Duration::from_secs(6 * 60);
    assert_eq!(
        wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, later).len(),
        1
    );
}

#[test]
fn previews_play_pause_skip_those_without_one_and_stop_with_the_conversation() {
    let (mut host, _, id) = shared();
    let now = host.now;
    let (request, link) = wanted_spotify_cards(&mut host.ui, &host.store, &host.tree, now)
        .pop()
        .unwrap();
    spotify_card_result(&mut host.ui, request, &link, Some(entity()), now);
    host.frame();

    let intents = click(&mut host, id, "play");
    let (first, url) = played(&intents).expect("the first preview is fetched");
    assert_eq!(url, "https://p.scdn.co/mp3-preview/1");
    let player = host.ui.voice_player.as_ref().unwrap();
    assert_eq!(player.spotify, Some(SpotifyPlaying { embed: 1, track: 0 }));
    assert_eq!(player.state, VoiceState::Loading);
    voice_loaded(&mut host.ui, first, Ok(()));
    assert_eq!(
        host.ui.voice_player.as_ref().unwrap().state,
        VoiceState::Playing
    );
    assert!(host.painted().contains(&"Uno".to_owned()));
    // The same button pauses it and goes on, without fetching it again.
    assert!(click(&mut host, id, "play").is_empty());
    assert_eq!(
        host.ui.voice_player.as_ref().unwrap().state,
        VoiceState::Paused
    );
    assert!(click(&mut host, id, "play").is_empty());
    assert_eq!(
        host.ui.voice_player.as_ref().unwrap().state,
        VoiceState::Playing
    );

    // «Siguiente» skips the track without a preview and ends the first.
    let intents = click(&mut host, id, "next");
    assert!(intents
        .iter()
        .any(|intent| matches!(intent, Intent::StopVoice { request } if *request == first)));
    let (_, url) = played(&intents).unwrap();
    assert_eq!(url, "https://p.scdn.co/mp3-preview/3");
    // Past the last, the first again; «Anterior» goes back.
    assert_eq!(
        played(&click(&mut host, id, "next")).unwrap().1,
        "https://p.scdn.co/mp3-preview/1"
    );
    assert_eq!(
        played(&click(&mut host, id, "track:2")).unwrap().1,
        "https://p.scdn.co/mp3-preview/3"
    );
    assert_eq!(
        played(&click(&mut host, id, "previous")).unwrap().1,
        "https://p.scdn.co/mp3-preview/1"
    );
    assert!(
        click(&mut host, id, "track:1").is_empty(),
        "a track without a preview plays nothing"
    );

    // Its title opens the link, after asking.
    let page = "https://open.spotify.com/playlist/3U5Re9YvytJ2PUXf8mTwgY";
    let opened = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("markdown_link:embed:0:{page}"),
        },
        host.now,
    );
    assert!(opened.is_empty(), "{opened:?}");
    assert_eq!(
        host.ui
            .message_actions
            .external
            .as_ref()
            .map(|link| link.url.as_str()),
        Some(page)
    );

    host.ui.message_actions.external = None;
    // Leaving the conversation stops it.
    host.open_channel("anuncios");
    assert!(host.ui.voice_player.is_none());
}
