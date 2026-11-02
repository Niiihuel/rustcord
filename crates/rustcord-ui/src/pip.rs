//! The picture-in-picture: the stream the account watches, or its own
//! share, floating over the window while the call's stage is not on screen
//! (the person is reading a conversation). A click on it, or its channel's
//! name, returns to the call; it can be dragged to another corner, where
//! it stays until the next drag (memory only). Cameras are not shown here.

use limn::{Element, Point, Size, UiAction};
use rustcord_components::voice::{self as parts, ids, PipButton, PipCorner, PipGlyph, PipView};
use rustcord_components::Look;
use rustcord_i18n::{t, tr};
use rustcord_model::StreamPhase;
use rustcord_state::Store;

use crate::{video, Intent, Ui};

/// Where the picture-in-picture was last let go, in the window.
#[derive(Debug, Default)]
pub struct Pip {
    dropped: Option<Point>,
}

/// The corner nearest where it was let go: top right until it is moved.
pub(crate) fn corner(ui: &Ui, window: Size) -> PipCorner {
    let at = ui.pip.dropped.unwrap_or(Point::new(window.width, 0.0));
    PipCorner {
        bottom: at.y > window.height / 2.0,
        right: at.x > window.width / 2.0,
    }
}

/// What the picture-in-picture shows in a window of `window`, if anything:
/// the first watched stream not in a window of its own and still live, else
/// the account's own share while it goes out. Nothing while the stage shows
/// the call, while a menu of the call or a settings page is open, or outside
/// a call.
pub(crate) fn view(ui: &Ui, store: &Store, window: Size) -> Option<PipView> {
    let call = store.call().filter(|call| call.phase.is_connected())?;
    // The call's own view is on screen: its stage, or its conversation with none.
    let blocked = crate::voice::selected(ui) == Some(call.channel)
        || ui.voice.menu.is_some()
        || ui.voice.card.is_some()
        || ui.video.picker
        || ui.settings.is_some()
        || ui.server_settings.is_some()
        || ui.channel_settings.is_some();
    if blocked {
        return None;
    }
    let live = t("En vivo");
    let simulated = crate::voice::simulated(store).then(|| tr!("{note} · simulada", note = live));
    let corner = corner(ui, window);
    let watched = store.watching().iter().find(|watching| {
        !ui.video.windows.contains(&watching.owner)
            && !matches!(
                watching.phase,
                StreamPhase::Ended(_) | StreamPhase::Waiting | StreamPhase::Failed(_)
            )
    });
    let leave = PipButton {
        id: ids::LEAVE.into(),
        hint: t("Desconectar"),
        glyph: PipGlyph::HangUp,
    };
    let channel = store.channel_title(call.channel);
    if let Some(watching) = watched {
        let server = store
            .channel(watching.channel)
            .and_then(|found| found.server);
        let frame = ui
            .video
            .streams
            .iter()
            .find(|(owner, _)| *owner == watching.owner)
            .and_then(|(_, live)| live.frame());
        return Some(PipView {
            surface: ids::stream_surface(watching.owner.0),
            title: tr!(
                "Pantalla de {name}",
                name = store.display_name(watching.owner, server)
            ),
            channel,
            frame,
            waiting: t("Esperando video…").into(),
            note: simulated.clone(),
            buttons: vec![
                PipButton {
                    id: ids::window(watching.owner.0),
                    hint: t("Ventana aparte"),
                    glyph: PipGlyph::PopOut,
                },
                PipButton {
                    id: ids::stop_watching(watching.owner.0),
                    hint: t("Dejar de ver"),
                    glyph: PipGlyph::StopShare,
                },
                leave,
            ],
            corner,
        });
    }
    video::sending(ui, store)?;
    Some(PipView {
        surface: ids::SCREEN_SURFACE.into(),
        title: t("Tu pantalla").into(),
        channel,
        frame: ui.video.screen.live.frame(),
        waiting: t("Esperando al sistema…").into(),
        note: simulated,
        buttons: vec![
            PipButton {
                id: ids::OWN_STREAM_STOP.into(),
                hint: t("Dejar de transmitir"),
                glyph: PipGlyph::StopShare,
            },
            leave,
        ],
        corner,
    })
}

/// The picture-in-picture over `root` in the window's corner, if it shows.
pub(crate) fn overlay(ui: &Ui, store: &Store, root: Element, window: Size, look: &Look) -> Element {
    let Some(view) = view(ui, store, window) else {
        return root;
    };
    Element::stack(vec![root, parts::pip(&view, look)])
}

/// Answer the picture-in-picture's clicks and drags; `None` for other parts.
pub(crate) fn update(ui: &mut Ui, store: &Store, action: &UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::Clicked { id } if id == ids::PIP => {
            ui.voice.stage = store.call().map(|call| call.channel);
            ui.voice.card = None;
            Some(Vec::new())
        }
        UiAction::Dropped { id, position, .. } if id == ids::PIP => {
            ui.pip.dropped = Some(*position);
            Some(Vec::new())
        }
        UiAction::DragStarted { id, .. }
        | UiAction::DragMoved { id, .. }
        | UiAction::DragCancelled { id }
            if id == ids::PIP =>
        {
            Some(Vec::new())
        }
        _ => None,
    }
}
