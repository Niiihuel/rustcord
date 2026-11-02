//! The picture-in-picture: a stream watched, or the account's own share,
//! floats over the conversation while the call's stage is away; a click
//! returns to the call, a drag moves it to another corner, and it is gone
//! outside the call.

use limn::{Point, UiAction};
use rustcord_model::StreamPhase;

use super::*;

/// The picture-in-picture's box, in the window: its corner, over the grid.
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
    // Reading a conversation: the stream floats in the top right corner.
    host.open_channel("general");
    assert_eq!(host.ui.voice.stage, None);
    assert!(host.vz_shows(ids::PIP));
    let corner = at_corner(&mut host);
    assert!(
        corner.x > 1280.0 / 2.0 && corner.y < 800.0 / 2.0,
        "top right"
    );
    assert!(host.frame().commands.iter().any(|command| matches!(
        command,
        limn::DrawCommand::Surface { id, .. } if *id == ids::stream_surface(owner.0)
    )));
    // Its controls: a window of its own, letting go, and hanging up.
    assert!(host.vz_shows(&ids::window(owner.0)));
    assert!(host.vz_shows(&ids::stop_watching(owner.0)));
    assert!(host.vz_shows(ids::LEAVE));
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
    // Hanging up from it leaves the call and its picture.
    host.click(ids::LEAVE);
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

#[test]
fn let_go_in_another_corner_the_picture_in_picture_stays_there() {
    let mut host = Host::new();
    let room = host.vz_join("Sala común");
    host.click(&format!("voice:{room}"));
    let owner = host.tr_streamers(room, 4, 1)[0];
    host.tr_watch(room, owner);
    host.open_channel("general");
    // Let go in the bottom left: the nearest corner, and it stays there.
    crate::pip::update(
        &mut host.ui,
        &host.store,
        &UiAction::Dropped {
            id: ids::PIP.into(),
            position: Point::new(100.0, 700.0),
            target: None,
        },
    );
    let corner = at_corner(&mut host);
    assert!(
        corner.x < 1280.0 / 2.0 && corner.y > 800.0 / 2.0,
        "bottom left"
    );
    // Let go in the top right again: back there.
    crate::pip::update(
        &mut host.ui,
        &host.store,
        &UiAction::Dropped {
            id: ids::PIP.into(),
            position: Point::new(1200.0, 80.0),
            target: None,
        },
    );
    let corner = at_corner(&mut host);
    assert!(
        corner.x > 1280.0 / 2.0 && corner.y < 800.0 / 2.0,
        "top right"
    );
}
