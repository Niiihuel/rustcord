//! The picture-in-picture: a stream watched, or the account's own share,
//! floats over the conversation while the call's stage is away, at the
//! bottom right as Discord's; a click returns to the call, a drag puts it
//! anywhere in the window, and it is gone outside the call.

use limn::{Point, UiAction};
use rustcord_model::StreamPhase;

use super::*;

/// The picture-in-picture's box, in the window.
fn at_corner(host: &mut Host) -> limn::Rect {
    host.vz_rect(ids::PIP)
}

#[test]
fn a_stream_watched_floats_over_the_conversation_until_a_click_returns_to_the_call() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    host.click(&format!("voice:{room}"));
    let owners = host.tr_streamers(room, 4, 1);
    let owner = owners[0];
    host.tr_watch(room, owner);
    // On the call's stage the stream is there, and no picture-in-picture.
    assert!(!host.vz_shows(ids::PIP));
    // Reading a conversation: the stream floats at the bottom right,
    // Discord's 80 from the window's right and 132 from its bottom.
    host.open_channel("general");
    assert_eq!(host.ui.voice.stage, None);
    assert!(host.vz_shows(ids::PIP));
    let corner = at_corner(&mut host);
    assert_eq!(
        (corner.x + corner.width, corner.y + corner.height),
        (1280.0 - 80.0, 800.0 - 132.0)
    );
    assert!(host.frame().commands.iter().any(|command| matches!(
        command,
        limn::DrawCommand::Surface { id, .. } if *id == ids::stream_surface(owner.0)
    )));
    // Its controls: a window of its own and letting go; no hanging up.
    assert!(host.vz_shows(&ids::window(owner.0)));
    assert!(host.vz_shows(&ids::stop_watching(owner.0)));
    assert!(!host.vz_shows(ids::LEAVE));
    // A click on it goes back to the call.
    host.click(ids::PIP);
    assert_eq!(host.ui.voice.stage, Some(room));
    assert!(!host.vz_shows(ids::PIP));
    // Its window of its own takes it off the picture-in-picture.
    host.open_channel("general");
    assert!(host.vz_shows(ids::PIP));
    host.click(&ids::window(owner.0));
    assert!(!host.vz_shows(ids::PIP), "the stream is in its own window");
    host.ui.video.windows.clear();
    // Letting go of the stream from it takes it away.
    host.click(&ids::stop_watching(owner.0));
    assert!(!host.vz_shows(ids::PIP));
}

#[test]
fn the_accounts_own_share_floats_when_no_stream_is_watched_and_stops_with_it() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    let me = host.vz_me();
    host.store.begin_share(room);
    host.event(Event::StreamPhase {
        channel: room,
        owner: me,
        phase: StreamPhase::Live,
    });
    host.ui.video.screen.on = true;
    host.ui.video.screen.live = crate::Live::Showing(1920, 1080, None);
    host.open_channel("general");
    assert!(host.vz_shows(ids::PIP));
    assert!(host.vz_shows(ids::OWN_STREAM_STOP), "«Dejar de transmitir»");
    assert!(!host.vz_shows(&ids::window(me.0)));
    assert!(host.frame().commands.iter().any(|command| matches!(
        command,
        limn::DrawCommand::Surface { id, .. } if *id == ids::SCREEN_SURFACE
    )));
    // Done sharing: it is gone.
    host.click(ids::OWN_STREAM_STOP);
    assert!(!host.vz_shows(ids::PIP));
}

// Discord's «Ajustes de la transmisión» on the own share: its quality changes
// live (the provider is told, the picker stays shut), its sound, the report
// goes to Rustcord's issues, and stopping ends the share.
#[test]
fn the_own_shares_gear_changes_its_quality_live_its_sound_and_stops_it() {
    use rustcord_model::StreamQuality;
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    let me = host.vz_me();
    host.store.begin_share(room);
    host.event(Event::StreamPhase {
        channel: room,
        owner: me,
        phase: StreamPhase::Live,
    });
    host.ui.video.screen.on = true;
    host.ui.video.screen.live = crate::Live::Showing(1920, 1080, None);
    host.open_channel("general");
    assert!(!host.vz_shows(ids::PIP_STOP));
    host.click(ids::PIP_SETTINGS);
    assert!(host.vz_shows(ids::PIP_STOP) && host.vz_shows(ids::PIP_REPORT));
    let choose = |host: &mut Host, id: &str| {
        let action = UiAction::Clicked { id: id.into() };
        crate::update(&mut host.ui, &mut host.store, &action, host.now)
    };
    // Its submenu's row keeps the menu open; a rate in it goes out at once.
    assert!(choose(&mut host, ids::PIP_QUALITY).is_empty() && host.ui.pip.menu);
    let sixty = StreamQuality {
        height: 720,
        fps: 60,
    };
    let intents = choose(&mut host, "vz:fps:60");
    assert!(matches!(
        &intents[..],
        [Intent::Command(Command::SetStreamQuality { channel, quality })]
            if *channel == room && *quality == sixty
    ));
    assert_eq!(host.ui.video.quality, sixty);
    assert!(!host.ui.video.picker && !host.ui.pip.menu);
    // The same quality again says nothing.
    assert!(choose(&mut host, "vz:fps:60").is_empty());
    // The applications' sound, from the menu.
    host.click(ids::PIP_SETTINGS);
    host.click(ids::SOUND);
    assert!(host.ui.video.silent && !host.ui.video.picker);
    // The report opens Rustcord's issues, never Discord's.
    host.click(ids::PIP_SETTINGS);
    let intents = choose(&mut host, ids::PIP_REPORT);
    assert!(matches!(
        &intents[..],
        [Intent::OpenUrl(url)] if url.starts_with("https://github.com/Niiihuel/rustcord/issues")
    ));
    // «Dejar de transmitir» in it ends the share.
    host.click(ids::PIP_SETTINGS);
    let from = host.sent.len();
    host.click(ids::PIP_STOP);
    assert!(host.sent[from..]
        .iter()
        .any(|command| matches!(command, Command::StopStream { channel } if *channel == room)));
    assert!(!host.vz_shows(ids::PIP));
}

#[test]
fn the_picture_in_picture_stays_wherever_it_is_let_go_inside_the_window() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    host.click(&format!("voice:{room}"));
    let owner = host.tr_streamers(room, 4, 1)[0];
    host.tr_watch(room, owner);
    host.open_channel("general");
    let drag = |host: &mut Host, action| crate::pip::update(&mut host.ui, &mut host.store, &action);
    let started = |position| UiAction::DragStarted {
        id: ids::PIP.into(),
        position,
    };
    let moved = |position| UiAction::DragMoved {
        id: ids::PIP.into(),
        position,
        over: None,
    };
    let dropped = |position| UiAction::Dropped {
        id: ids::PIP.into(),
        position,
        target: None,
    };
    // While dragged it follows the pointer from where it was picked up.
    let before = at_corner(&mut host);
    let start = Point::new(before.x + 100.0, before.y + 50.0);
    let to = Point::new(start.x - 523.0, start.y - 301.0);
    drag(&mut host, started(start));
    drag(&mut host, moved(to));
    let carried = at_corner(&mut host);
    assert_eq!((carried.x, carried.y), (before.x - 523.0, before.y - 301.0));
    // A cancelled drag puts it back.
    drag(
        &mut host,
        UiAction::DragCancelled {
            id: ids::PIP.into(),
        },
    );
    assert_eq!(at_corner(&mut host), before);
    // Let go at any point: it stays there, not in a corner.
    drag(&mut host, started(start));
    drag(&mut host, moved(to));
    at_corner(&mut host);
    drag(&mut host, dropped(to));
    let put = at_corner(&mut host);
    assert_eq!((put.x, put.y), (before.x - 523.0, before.y - 301.0));
    // A second drag starts from there; past the window's edge it stops at
    // it, and stays inside once let go.
    let start = Point::new(put.x + 10.0, put.y + 10.0);
    let far = Point::new(start.x - 2_000.0, start.y + 2_000.0);
    drag(&mut host, started(start));
    drag(&mut host, moved(far));
    at_corner(&mut host);
    drag(&mut host, dropped(far));
    let kept = at_corner(&mut host);
    assert_eq!((kept.x, kept.y + kept.height), (0.0, 800.0));
    // Back to the call and out again: still there.
    host.click(ids::PIP);
    host.open_channel("general");
    assert_eq!(at_corner(&mut host), kept);
}
