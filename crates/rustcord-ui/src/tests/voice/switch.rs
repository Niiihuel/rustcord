//! Joining and switching voice channels as Discord's: a row joins at once and
//! leaves the view as it is; moving the call from another channel asks first,
//! unless the account said not to; the stage shown follows the call; a
//! preview's «Unirse» never asks.

use super::*;

impl Host {
    /// Whether a `JoinVoice` for `channel` went out from `from` on.
    fn sw_joined(&self, from: usize, channel: ChannelId) -> bool {
        self.sent[from..].iter().any(
            |command| matches!(command, Command::JoinVoice { channel: to, .. } if *to == channel),
        )
    }
}

#[test]
fn another_channel_asks_first_and_only_confirm_moves_the_call() {
    let mut host = Host::new();
    let text = host.ui.channel().unwrap();
    let room = host.vz_join("Sala común");
    let pair = host.channel_named("Pair programming");
    let row = format!("voice:{pair}");
    // Cancel, its ×, and Escape leave everything as it was.
    for close in ["cancel", "close", "escape"] {
        let from = host.sent.len();
        host.click(&row);
        assert!(host.ui.voice.switch.is_some(), "{close}: asked");
        assert!(host.sent[from..].is_empty(), "{close}: nothing sent yet");
        let painted = host.painted();
        assert!(painted.contains(&"¿Confirmas?".to_owned()));
        assert!(
            painted.contains(&"Pair programming".to_owned()),
            "{painted:?}"
        );
        match close {
            "cancel" => host.click(ids::SWITCH_CANCEL),
            "close" => host.click(&format!("{}:close", ids::SWITCH_CANCEL)),
            _ => host.key(Key::Escape, false, false),
        }
        assert_eq!(host.ui.voice.switch, None, "{close}: closed");
        assert!(host.sent[from..].is_empty(), "{close}: nothing sent");
        assert_eq!(host.store.call().unwrap().channel, room);
        assert!(!host.vz_shows(ids::SWITCH));
    }
    // Confirm moves the call; the conversation stays in view.
    let from = host.sent.len();
    host.click(&row);
    host.click(ids::SWITCH);
    assert!(host.sw_joined(from, pair));
    host.vz_answer(from, 47);
    assert_eq!(host.store.call().unwrap().channel, pair);
    assert_eq!((host.ui.voice.stage, host.ui.voice.preview), (None, None));
    assert_eq!(host.ui.channel(), Some(text));
    assert!(host.vz_shows(&editor_id(text)));
    assert!(host.ui.prefs.voice_switch_warning, "asked again next time");
}

#[test]
fn the_stage_shown_follows_the_call_to_its_new_channel() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    host.click(&format!("voice:{room}"));
    assert_eq!(
        host.ui.voice.stage,
        Some(room),
        "its own row opens its stage"
    );
    let pair = host.channel_named("Pair programming");
    let from = host.sent.len();
    host.tree.reveal_at_start(&format!("voice:{pair}"), 0.0);
    host.click(&format!("voice:{pair}"));
    host.click(ids::SWITCH);
    host.vz_answer(from, 47);
    assert_eq!(host.ui.voice.stage, Some(pair));
    assert!(host.vz_shows(&ids::tile(host.vz_me().0)));
}

#[test]
fn dont_ask_again_turns_the_warning_off_and_the_next_switch_moves_at_once() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    let pair = host.channel_named("Pair programming");
    host.click(&format!("voice:{pair}"));
    host.click(ids::SWITCH_FORGET);
    assert!(host
        .ui
        .voice
        .switch
        .is_some_and(|switch| switch.dont_ask_again));
    let from = host.sent.len();
    host.click(ids::SWITCH);
    assert!(host.sw_joined(from, pair));
    assert!(!host.ui.prefs.voice_switch_warning);
    let kept = host.ui.settings_to_keep();
    assert!(kept.contains(&("voice_switch_warning", Some("false".into()))));
    host.vz_answer(from, 47);
    // Off: the next switch asks nothing.
    let from = host.sent.len();
    host.click(&format!("voice:{room}"));
    assert_eq!(host.ui.voice.switch, None);
    assert!(host.sw_joined(from, room));
}

#[test]
fn a_preview_s_join_moves_the_call_without_asking() {
    let mut host = Host::new();
    host.vz_join("Sala común");
    let pair = host.channel_named("Pair programming");
    // The chat bubble only looks at the channel, as Discord's.
    let from = host.sent.len();
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("{}{pair}", crate::voice::OPEN_CHAT),
        },
        host.now,
    );
    host.run(intents);
    assert_eq!(host.ui.voice.preview, Some(pair));
    assert!(!host.sw_joined(from, pair));
    host.click(ids::JOIN);
    assert_eq!(host.ui.voice.switch, None);
    assert!(host.sw_joined(from, pair));
    assert_eq!(host.ui.voice.stage, Some(pair));
}

#[test]
fn a_channel_with_mentions_opens_its_stage_with_its_chat_once_joined() {
    let mut host = Host::new();
    let open = host.ui.channel().unwrap();
    let pair = host.channel_named("Pair programming");
    let me = host.vz_me();
    let mut mention = host.store.timeline(open).unwrap().messages()[0]
        .as_ref()
        .clone();
    mention.id = rustcord_model::MessageId(u64::MAX / 4);
    mention.channel = pair;
    mention.mentions = vec![me];
    mention.author = host.vz_other_than_me();
    host.event(Event::MessageCreated {
        message: mention,
        nonce: None,
    });
    assert!(host.store.read(pair).mentions > 0);
    let from = host.sent.len();
    host.tree.reveal_at_start(&format!("voice:{pair}"), 0.0);
    host.click(&format!("voice:{pair}"));
    assert!(host.sw_joined(from, pair));
    assert_eq!(host.ui.voice.stage, Some(pair));
    assert!(host.ui.voice.chat.is_some());
    assert_eq!(host.ui.channel(), Some(pair));
}

impl Host {
    /// Someone who is not the account.
    fn vz_other_than_me(&self) -> UserId {
        let me = self.vz_me();
        self.store.users().find(|user| user.id != me).unwrap().id
    }
}
