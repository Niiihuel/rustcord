//! Local reminders, actual window activity, and the channel-scoped inbox.

use super::*;
use rustcord_model::{HistorySource, Permissions};
use rustcord_provider::{ConnectionState, Failure};

fn clear_everywhere(host: &mut Host) {
    let channels: Vec<_> = host
        .store
        .read_channels()
        .map(|channel| channel.id)
        .collect();
    for channel in channels {
        host.store.mark_read(channel);
    }
}

fn unread_action(host: &mut Host, channel: ChannelId, message: MessageId) {
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("msg:{message}:unread"),
        },
        host.now,
    );
    host.run(intents);
    assert_eq!(host.store.read(channel).unread_from, Some(message));
}

#[test]
fn an_inactive_window_never_marks_visible_messages_read() {
    let mut host = Host::new();
    host.active = false;
    let channel = host.open_channel("general");
    assert!(host.store.read(channel).unread());
    let sent = host.sent.len();
    host.frame();
    host.frame();
    assert!(host.store.read(channel).unread());
    assert!(!host.sent[sent..]
        .iter()
        .any(|command| matches!(command, Command::MarkRead { .. })));
    host.active = true;
    host.tree.scroll_to_end(&timeline_id(channel));
    host.frame();
    assert!(!host.store.read(channel).unread());
}

/// Discord's line: what came while the window was away is marked «NUEVO»,
/// and stays marked once read; read as it comes, nothing is.
#[test]
fn messages_that_come_while_the_window_is_away_get_the_new_line() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    host.tree.scroll_to_end(&timeline_id(channel));
    host.frame();
    // Read, and opened again: nothing left to mark.
    host.open_channel("ayuda");
    host.open_channel("general");
    host.tree.scroll_to_end(&timeline_id(channel));
    host.frame();
    assert!(!host.store.read(channel).unread());
    assert!(!host.ui.new_marker.contains_key(&channel));
    let mut incoming = host.store.timeline(channel).unwrap().messages()[0]
        .as_ref()
        .clone();
    // Someone else the account does not block: a blocked person's messages
    // fold into «N mensajes bloqueados», with no line (and the users' order
    // is a map's, so the first other one is anyone).
    incoming.author = host
        .store
        .users()
        .find(|user| Some(user.id) != host.store.me() && !host.store.blocked(user.id))
        .unwrap()
        .id;
    // Looked at, at its end: read as it comes, no line.
    incoming.id = MessageId(u64::MAX / 4);
    host.event(Event::MessageCreated {
        message: incoming.clone(),
        nonce: None,
    });
    host.frame();
    host.frame();
    assert!(!host.ui.new_marker.contains_key(&channel));
    // The window away: the line at the first that came meanwhile.
    host.active = false;
    let away = MessageId(u64::MAX / 4 + 1);
    incoming.id = away;
    host.event(Event::MessageCreated {
        message: incoming.clone(),
        nonce: None,
    });
    incoming.id = MessageId(u64::MAX / 4 + 2);
    host.event(Event::MessageCreated {
        message: incoming,
        nonce: None,
    });
    host.frame();
    assert_eq!(host.ui.new_marker.get(&channel), Some(&away));
    let painted = host.painted();
    assert!(painted.iter().any(|text| text == "NUEVO"), "{painted:?}");
    // Back, it is read, and the line stays while it is open.
    host.active = true;
    host.tree.scroll_to_end(&timeline_id(channel));
    host.frame();
    host.frame();
    assert!(!host.store.read(channel).unread());
    assert!(host.painted().iter().any(|text| text == "NUEVO"));
}

#[test]
fn a_manual_unread_bar_survives_leaving_and_reopening_without_a_remote_rewind() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    let target = host
        .store
        .timeline(channel)
        .unwrap()
        .messages()
        .iter()
        .rev()
        .find(|message| Some(message.author) != host.store.me() && !message.delivery.is_local())
        .unwrap()
        .id;
    let through = host.store.read(channel).last_read;
    let sent = host.sent.len();
    unread_action(&mut host, channel, target);
    assert_eq!(host.store.read(channel).last_read, through);
    assert!(!host.sent[sent..]
        .iter()
        .any(|command| matches!(command, Command::MarkRead { .. })));
    host.open_channel("ayuda");
    host.open_channel("general");
    assert_eq!(host.store.read(channel).unread_from, Some(target));
    assert!(host
        .painted()
        .iter()
        .any(|text| text.contains("Marcar como leído")));
    host.click(crate::messages::UNREAD_CLEAR);
    assert_eq!(host.store.read(channel).unread_from, None);
}

#[test]
fn inbox_mentions_list_loaded_messages_and_remove_revoked_cards_immediately() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    clear_everywhere(&mut host);
    let mut mention = host.store.timeline(channel).unwrap().messages()[0]
        .as_ref()
        .clone();
    mention.id = MessageId(u64::MAX / 4);
    mention.content = "mira esto, te mencionan".into();
    mention.mentions = vec![host.store.me().unwrap()];
    // Someone else the account does not block (the users' order is a map's).
    mention.author = host
        .store
        .users()
        .find(|user| Some(user.id) != host.store.me() && !host.store.blocked(user.id))
        .unwrap()
        .id;
    host.active = false;
    host.event(Event::MessageCreated {
        message: mention.clone(),
        nonce: None,
    });
    host.click("inbox:open");
    host.click("inbox:mentions");
    let jump = format!("inbox:jump:{channel}:{}", mention.id);
    assert!(host.frame().regions.iter().any(|region| region.id == jump));
    let painted = host.painted().join(" ");
    assert!(painted.contains("te mencionan"), "{painted}");
    // It says what it is not.
    assert!(painted.contains("historial completo"), "{painted}");
    let mut revoked = host.store.channel(channel).unwrap().clone();
    revoked.permissions = Permissions::VIEW;
    host.event(Event::ChannelChanged(revoked));
    assert!(!host.frame().regions.iter().any(|region| region.id == jump));
    let before = host.context_reads.len();
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked { id: jump },
        host.now,
    );
    host.run(intents);
    assert_eq!(host.context_reads.len(), before);
    assert!(host.ui.inbox.is_some());
}

#[test]
fn inbox_opens_an_unloaded_exact_reminder_through_context_and_preserves_the_draft() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    clear_everywhere(&mut host);
    let (history, _) = host.provider.history(channel, None, 1000);
    let target = history[2].id;
    assert!(host.store.timeline(channel).unwrap().get(target).is_none());
    host.type_text("borrador que sigue aquí");
    host.store.mark_unread(channel, target);
    host.click("inbox:open");
    host.click(&format!("inbox:go:{channel}"));
    let (request, found, anchor, source) = host
        .context_reads
        .last()
        .copied()
        .expect("one context read");
    assert_eq!(
        (found, anchor, source),
        (channel, target, HistorySource::Provider)
    );
    host.active = true;
    host.frame();
    assert_eq!(host.store.read(channel).unread_from, Some(target));
    assert_eq!(
        host.ui.drafts.get(&channel).map(String::as_str),
        Some("borrador que sigue aquí")
    );
    host.event(Event::ContextLoaded {
        request,
        channel,
        anchor: target,
        messages: history.into_iter().take(30).collect(),
    });
    host.frame();
    assert_eq!(
        host.store.timeline(channel).unwrap().context_source(),
        Some(HistorySource::Provider)
    );
    assert_eq!(host.store.read(channel).unread_from, Some(target));
}

#[test]
fn local_progress_shows_an_explicit_confirmation_action_without_layout_retries() {
    let mut host = Host::new();
    host.confirm_reads = false;
    host.active = false;
    let channel = host.open_channel("general");
    host.store.connection = ConnectionState::Disconnected {
        reason: "sin conexión".into(),
    };
    assert_eq!(host.store.mark_read_command(channel), None);
    host.store.connection = ConnectionState::Connected;
    let sent = host.sent.len();
    for _ in 0..3 {
        host.frame();
    }
    assert_eq!(host.sent.len(), sent);
    assert!(host
        .painted()
        .iter()
        .any(|text| text.contains("sin confirmar")));
    host.click(&format!("reads:retry:{channel}"));
    let Some(Command::MarkRead { request, .. }) = host.sent.last().cloned() else {
        panic!("explicit retry")
    };
    host.event(Event::Failed {
        failure: Failure::ReadState { request, channel },
        reason: "No se confirmó".into(),
        ambiguous: true,
    });
    assert!(!host.store.read(channel).sync_pending());
    let sent = host.sent.len();
    host.frame();
    host.frame();
    assert_eq!(host.sent.len(), sent);
    assert!(host
        .painted()
        .iter()
        .any(|text| text.contains("Reintentar")));
}

#[test]
fn a_stale_or_deleted_unread_action_cannot_mark_another_channel_or_invent_an_anchor() {
    let mut host = Host::new();
    let previous = host.open_channel("general");
    let old = host.store.timeline(previous).unwrap().oldest().unwrap();
    let current = host.open_channel("ayuda");
    let revision = host.store.revision();
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("msg:{old}:unread"),
        },
        host.now,
    );
    assert!(intents.is_empty());
    assert_eq!(host.store.revision(), revision);
    assert_eq!(host.store.read(current).unread_from, None);
    let deleted = host.store.timeline(current).unwrap().oldest().unwrap();
    host.event(Event::MessageDeleted {
        channel: current,
        message: deleted,
    });
    let revision = host.store.revision();
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("msg:{deleted}:unread"),
        },
        host.now,
    );
    assert!(intents.is_empty());
    assert_eq!(host.store.revision(), revision);
    assert_eq!(host.store.read(current).unread_from, None);
}

#[test]
fn retrying_a_read_does_not_consume_a_new_mention_or_a_manual_reminder() {
    let mut host = Host::new();
    host.active = false;
    host.confirm_reads = false;
    let channel = host.open_channel("general");
    let original = host.store.read(channel).last_message.unwrap();
    let command = host.store.mark_read_command(channel).unwrap();
    let Command::MarkRead { request, .. } = command else {
        unreachable!()
    };
    host.event(Event::Failed {
        failure: Failure::ReadState { request, channel },
        reason: "Sin ACK".into(),
        ambiguous: true,
    });
    let mut mention = host.store.timeline(channel).unwrap().messages()[0]
        .as_ref()
        .clone();
    let reminder = mention.id;
    mention.id = MessageId(u64::MAX / 4);
    mention.mentions = vec![host.store.me().unwrap()];
    // Someone else the account does not block (the users' order is a map's).
    mention.author = host
        .store
        .users()
        .find(|user| Some(user.id) != host.store.me() && !host.store.blocked(user.id))
        .unwrap()
        .id;
    host.event(Event::MessageCreated {
        message: mention,
        nonce: None,
    });
    host.store.mark_unread(channel, reminder);
    host.click(&format!("reads:retry:{channel}"));
    assert!(
        matches!(host.sent.last(), Some(Command::MarkRead { message, .. }) if *message == original)
    );
    assert_eq!(host.store.read(channel).last_read, Some(original));
    assert_eq!(host.store.read(channel).unread_from, Some(reminder));
    assert_eq!(host.store.read(channel).mentions, 1);
}

#[test]
fn an_automatic_unread_anchor_deleted_before_latest_is_recomputed_from_real_rows() {
    let mut host = Host::new();
    host.active = false;
    let channel = host.channel_named("general");
    let first = host.store.read(channel).first_unread.unwrap();
    let server = host.store.channel(channel).unwrap().server;
    let _pending = open(&mut host.ui, &mut host.store, server, channel);
    host.event(Event::MessageDeleted {
        channel,
        message: first,
    });
    let (messages, reached_start) = host.provider.history(channel, None, 50);
    host.event(Event::HistoryLoaded {
        request: 77,
        channel,
        before: None,
        messages,
        reached_start,
    });
    assert!(!host
        .context_reads
        .iter()
        .any(|(_, _, anchor, _)| *anchor == first));
    let read = host.store.read(channel);
    let expected = host
        .store
        .timeline(channel)
        .unwrap()
        .messages()
        .iter()
        .find(|message| Some(message.id) > read.last_read)
        .unwrap()
        .id;
    assert_eq!(host.ui.new_marker.get(&channel), Some(&expected));
}

#[test]
fn a_layout_with_a_pending_scroll_cannot_mark_the_old_end_read() {
    let mut host = Host::new();
    host.active = false;
    let channel = host.open_channel("general");
    host.tree.scroll_to_end(&timeline_id(channel));
    host.layout_only();
    let anchor = host.store.timeline(channel).unwrap().newest().unwrap().id;
    let scroll = open_message(
        &mut host.ui,
        &host.store,
        channel,
        anchor,
        HistorySource::Provider,
    );
    assert!(scroll
        .iter()
        .any(|intent| matches!(intent, Intent::ScrollToIndex { .. })));
    let effects = after_layout(&mut host.ui, &mut host.store, &host.tree, true);
    assert!(!effects
        .iter()
        .any(|intent| matches!(intent, Intent::Command(Command::MarkRead { .. }))));
    assert!(host.store.read(channel).unread());
    host.run(scroll);
    host.active = true;
    host.frame();
    assert!(!host.store.read(channel).unread());
}

#[test]
fn history_arriving_behind_the_inbox_cannot_close_it_or_steal_its_focus() {
    let mut host = Host::new();
    host.active = false;
    let channel = host.channel_named("general");
    let server = host.store.channel(channel).unwrap().server;
    let _pending = open(&mut host.ui, &mut host.store, server, channel);
    host.click("inbox:open");
    let (messages, reached_start) = host.provider.history(channel, None, 50);
    host.event(Event::HistoryLoaded {
        request: 77,
        channel,
        before: None,
        messages,
        reached_start,
    });
    assert!(host.ui.inbox.is_some());
    assert!(host.store.read(channel).unread());
    assert!(host.ui.unread_anchor.is_some());
}

#[test]
fn a_live_ready_for_the_cached_account_preserves_the_inbox_and_focuses_its_selected_tab() {
    let mut host = Host::new();
    host.active = false;
    let channel = host.open_channel("general");
    host.type_text("borrador durante el arranque guardado");
    host.store.from_cache = true;
    host.click("inbox:open");
    host.click("inbox:mentions");
    host.tree.focus("inbox:mentions");
    host.event(Event::Ready(Box::new(host.provider.snapshot())));
    host.frame();
    assert!(
        host.ui.inbox.is_some(),
        "same-account Ready preserves the modal"
    );
    assert_eq!(host.tree.focused_id(), Some("inbox:mentions"));
    assert_eq!(
        host.ui.drafts.get(&channel).map(String::as_str),
        Some("borrador durante el arranque guardado")
    );
    assert!(
        host.ui.unread_anchor.is_some(),
        "background history waits for the modal to close"
    );
    assert!(
        host.ui.context_navigation.is_none(),
        "no previous context survives Ready"
    );
}

#[test]
fn a_ready_for_another_account_closes_the_old_accounts_inbox() {
    let mut host = Host::new();
    host.active = false;
    host.open_channel("general");
    host.click("inbox:open");
    let mut snapshot = host.provider.snapshot();
    let mut other = snapshot.me.take().unwrap();
    other.id = rustcord_model::UserId(900_007_777);
    snapshot.users.push(other.clone());
    snapshot.me = Some(other);
    host.event(Event::Ready(Box::new(snapshot)));
    host.frame();
    assert!(host.ui.inbox.is_none());
    assert!(!host
        .tree
        .focused_id()
        .is_some_and(|id| id.starts_with("inbox:")));
}

#[test]
fn an_empty_inbox_shows_its_message_without_an_empty_scrollable_list() {
    let mut host = Host::new();
    host.active = false;
    host.open_channel("general");
    clear_everywhere(&mut host);
    host.click("inbox:open");
    let frame = host.frame();
    assert!(frame.commands.iter().any(|command| {
        matches!(command, limn::DrawCommand::Text { text, .. } if text == "¡Ya estás al día!")
    }));
    assert!(
        !frame.regions.iter().any(|region| region.id == "inbox:list"),
        "an empty inbox has no scrollable list or phantom card"
    );
    assert!(!frame
        .regions
        .iter()
        .any(|region| region.id.starts_with("inbox:go:")
            || region.id.starts_with("inbox:read:")
            || region.id == "inbox:read-all"));
}

/// Scroll the inbox's list to `channel`'s card, as a person would.
fn show_card(host: &mut Host, channel: ChannelId) {
    let index = unread_cards(host)
        .iter()
        .position(|card| *card == channel)
        .expect("a card of its own");
    host.frame();
    host.tree.scroll_to_index("inbox:list", index);
    host.frame();
}

/// The channels of the unread tab, in the inbox's order.
fn unread_cards(host: &Host) -> Vec<ChannelId> {
    host.ui
        .inbox
        .as_ref()
        .expect("the inbox is open")
        .listed()
        .into_iter()
        .map(|(channel, _)| channel)
        .collect()
}

#[test]
fn the_unread_tab_orders_its_cards_as_discords_and_loads_those_in_view_a_few_at_a_time() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let cards = unread_cards(&host);
    let kind = |channel: ChannelId| host.store.channel(channel).unwrap().kind;
    // The direct conversation first, the channel that mentions the account
    // next, then the rest in the list's order.
    assert_eq!(
        kind(cards[0]),
        rustcord_model::ChannelKind::Direct,
        "{cards:?}"
    );
    assert_eq!(cards[1], host.channel_named("charla"));
    assert!(host.store.read(cards[1]).mentions > 0);
    assert!(cards.contains(&host.channel_named("ayuda")));
    // The frame that showed them asked for the cards in view, three at a
    // time, and the next frames for the rest.
    let loaded = |host: &Host| {
        cards
            .iter()
            .filter(|channel| {
                host.store
                    .timeline(**channel)
                    .is_some_and(|timeline| timeline.latest)
            })
            .count()
    };
    assert!(loaded(&host) <= 3, "{}", loaded(&host));
    host.frame();
    host.frame();
    assert_eq!(loaded(&host), 3);
    // The last comes into view as the list scrolls: then it loads.
    show_card(&mut host, cards[cards.len() - 1]);
    host.frame();
    assert_eq!(loaded(&host), cards.len());
    host.tree.scroll_to_index("inbox:list", 1);
    let charla = cards[1];
    let first = host.store.read(charla).first_unread.unwrap();
    let said = host
        .store
        .timeline(charla)
        .unwrap()
        .get(first)
        .unwrap()
        .content
        .clone();
    let painted = host.painted().join(" ");
    let word = said
        .split_whitespace()
        .rfind(|word| word.chars().all(char::is_alphabetic))
        .unwrap();
    assert!(painted.contains(word), "{word} in {painted}");
    // Each message goes to itself; a card's name to its first unread.
    let frame = host.frame();
    assert!(frame
        .regions
        .iter()
        .any(|region| region.id == format!("inbox:jump:{charla}:{first}")));
    host.click(&format!("inbox:go:{charla}"));
    assert!(host.ui.inbox.is_none());
    assert_eq!(host.ui.channel(), Some(charla));
}

/// Cards whose pages do not come free their slots for the next, and are
/// not asked again while the inbox stays open.
#[test]
fn cards_that_fail_to_load_free_their_slots_and_are_not_asked_again() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let cards = unread_cards(&host);
    let loaded = |host: &Host, channel: ChannelId| {
        host.store
            .timeline(channel)
            .is_some_and(|timeline| timeline.latest)
    };
    // Every card not loaded yet but the last fails: three of them would
    // hold every slot.
    let last = *cards.last().unwrap();
    let failing: Vec<ChannelId> = cards
        .iter()
        .copied()
        .filter(|card| *card != last && !loaded(&host, *card))
        .collect();
    assert!(failing.len() >= 3, "{cards:?}");
    host.failing_history.extend(failing.iter().copied());
    for card in &cards {
        show_card(&mut host, *card);
        host.frame();
        host.frame();
    }
    assert!(loaded(&host, last), "the last loads past the failed ones");
    for card in &failing {
        let asked = host.pages.iter().filter(|page| *page == card).count();
        assert_eq!(asked, 1, "{card:?} asked once: {:?}", host.pages);
    }
}

#[test]
fn a_cards_check_marks_its_channel_read_and_it_leaves() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let ayuda = host.channel_named("ayuda");
    assert!(unread_cards(&host).contains(&ayuda));
    show_card(&mut host, ayuda);
    let sent = host.sent.len();
    host.click(&format!("inbox:read:{ayuda}"));
    assert!(host.sent[sent..]
        .iter()
        .any(|command| matches!(command, Command::MarkRead { channel, .. } if *channel == ayuda)));
    assert!(!host.store.read(ayuda).unread());
    assert!(!unread_cards(&host).contains(&ayuda));
    assert!(host.ui.inbox.is_some());
}

#[test]
fn marking_everything_read_asks_first_and_shift_skips_the_question() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let cards = unread_cards(&host);
    let sent = host.sent.len();
    host.click("inbox:read-all");
    assert!(host
        .painted()
        .iter()
        .any(|text| text == "¿Marcar todo como leído?"));
    host.click("inbox:read-all:cancel");
    assert_eq!(host.sent.len(), sent);
    assert_eq!(unread_cards(&host), cards);
    host.click("inbox:read-all");
    host.click("inbox:read-all:confirm");
    for channel in &cards {
        assert!(!host.store.read(*channel).unread(), "{channel}");
    }
    let marked = host.sent[sent..]
        .iter()
        .filter(|command| matches!(command, Command::MarkRead { .. }))
        .count();
    assert_eq!(marked, cards.len());
    assert!(host
        .painted()
        .iter()
        .any(|text| text == "¡Ya estás al día!"));

    // With Shift held, at once.
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    host.ui.messages.shift_held = true;
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: "inbox:read-all".into(),
        },
        host.now,
    );
    host.run(intents);
    assert!(unread_cards(&host).is_empty());
}

#[test]
fn a_folded_card_keeps_its_header_only_and_is_not_loaded() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let charla = host.channel_named("charla");
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("inbox:fold:{charla}"),
        },
        host.now,
    );
    host.run(intents);
    host.frame();
    host.frame();
    assert!(host
        .store
        .timeline(charla)
        .is_none_or(|timeline| !timeline.latest));
    let frame = host.frame();
    assert!(frame
        .regions
        .iter()
        .any(|region| region.id == format!("inbox:go:{charla}")));
    assert!(!frame
        .regions
        .iter()
        .any(|region| region.id.starts_with(&format!("inbox:jump:{charla}:"))));
    // Unfolded, it loads and shows them.
    host.click(&format!("inbox:fold:{charla}"));
    host.frame();
    host.frame();
    assert!(host
        .store
        .timeline(charla)
        .is_some_and(|timeline| timeline.latest));
}

#[test]
fn a_cards_bell_sets_that_channels_notifications_from_the_inbox() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let ayuda = host.channel_named("ayuda");
    show_card(&mut host, ayuda);
    host.click(&format!("inbox:bell:{ayuda}"));
    let nothing = format!("cl:notify:{ayuda}:nothing");
    assert!(host
        .frame()
        .regions
        .iter()
        .any(|region| region.id == nothing));
    let sent = host.sent.len();
    host.click(&nothing);
    // The provider keeps the account's notifications: it answers.
    for command in host.sent.split_off(sent) {
        for event in host.provider.manage(command) {
            host.event(event);
        }
    }
    // The channel marks nothing now: it leaves the inbox, which stays.
    assert!(host.ui.inbox.is_some());
    assert!(!unread_cards(&host).contains(&ayuda));
    assert!(!host
        .frame()
        .regions
        .iter()
        .any(|region| region.id.starts_with("cl:notify:")));
    // The open conversation's own bell was not touched.
    assert_eq!(host.ui.channel_list.bell, None);
}

#[test]
fn the_friend_requests_button_opens_the_pending_requests() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let waiting = host.store.pending_requests();
    let painted = host.painted();
    if waiting > 0 {
        assert!(painted.iter().any(|text| *text == waiting.to_string()));
    }
    host.click("inbox:friends");
    assert!(host.ui.inbox.is_none());
    assert!(crate::relationships::visible(&host.ui));
    assert_eq!(
        host.ui.friends.tab,
        rustcord_components::friends::FriendsTab::Pending
    );
}

#[test]
fn reminders_list_the_messages_marked_unread_and_go_to_them() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    host.active = false;
    let target = host
        .store
        .timeline(channel)
        .unwrap()
        .messages()
        .iter()
        .rev()
        .find(|message| Some(message.author) != host.store.me() && !message.delivery.is_local())
        .unwrap()
        .clone();
    unread_action(&mut host, channel, target.id);
    host.click("inbox:open");
    host.click("inbox:reminders");
    assert_eq!(
        host.ui.inbox.as_ref().unwrap().listed(),
        vec![(channel, Some(target.id))]
    );
    let jump = format!("inbox:jump:{channel}:{}", target.id);
    let frame = host.frame();
    assert!(frame.regions.iter().any(|region| region.id == jump));
    assert!(frame
        .regions
        .iter()
        .any(|region| region.id == format!("inbox:read:{channel}")));
    host.click(&jump);
    assert!(host.ui.inbox.is_none());
    assert_eq!(host.ui.channel(), Some(channel));
    // Marked read from the card, it goes.
    host.click("inbox:open");
    host.click("inbox:reminders");
    host.click(&format!("inbox:read:{channel}"));
    assert_eq!(host.store.read(channel).unread_from, None);
    assert!(host
        .painted()
        .join(" ")
        .contains("Organízate y no te olvides"));
}

#[test]
fn bookmarks_say_rustcord_keeps_none() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    host.click("inbox:bookmarks");
    let painted = host.painted();
    assert!(painted
        .iter()
        .any(|text| text == "Guarda tus recuerdos favoritos"));
    assert!(painted
        .iter()
        .any(|text| text == "Rustcord todavía no guarda marcadores."));
    let frame = host.frame();
    assert!(!frame
        .regions
        .iter()
        .any(|region| region.id == "inbox:list" || region.id == "inbox:read-all"));
}

#[test]
fn the_popout_opens_under_the_tray_or_at_a_narrow_headers_end() {
    let mut host = Host::new();
    host.open_channel("general");
    host.click("inbox:open");
    let frame = host.frame();
    let find = |frame: &limn::Frame, id: &str| {
        frame
            .regions
            .iter()
            .find(|region| region.id == id)
            .map(|region| region.rect)
            .unwrap_or_else(|| panic!("{id}"))
    };
    let tray = find(&frame, "inbox:open");
    let tab = find(&frame, "inbox:reminders");
    // The popout's end on the tray's (its tabs 4 in), 8 under it.
    assert!(
        (tab.x + tab.width + 4.0 - (tray.x + tray.width)).abs() < 1.0,
        "{tab:?} {tray:?}"
    );
    assert!((find(&frame, "inbox:unread").y - (tray.y + tray.height + 8.0 + 70.0)).abs() < 1.0);
    // A narrow window has no tray in its header: it opens at its end.
    let size = Size::new(720.0, 700.0);
    host.tree.set_root(view(
        &host.ui,
        &host.store,
        &ViewContext {
            window: size,
            ..host.context()
        },
    ));
    let narrow = host.tree.layout(size);
    let tab = find(&narrow, "inbox:reminders");
    assert!(
        tab.x + tab.width <= 720.0 && tab.x > 720.0 - 480.0,
        "{tab:?}"
    );
}
