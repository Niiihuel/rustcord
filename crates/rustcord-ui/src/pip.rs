//! The picture-in-picture: the stream the account watches, or its own
//! share, floating over the window while the call's stage is not on screen
//! (the person is reading a conversation). As Discord's, it opens at the
//! bottom right, shows only its picture until the pointer comes over it, and
//! goes wherever it is dragged, inside the window, where it stays until the
//! next drag (memory only). A click on it returns to the call. Cameras are
//! not shown here.

use limn::{Element, Point, Size, UiAction, UiTree};
use rustcord_components::voice::{self as parts, ids, PipButton, PipGlyph, PipView};
use rustcord_components::Look;
use rustcord_i18n::{t, tr};
use rustcord_model::StreamPhase;
use rustcord_state::Store;

use crate::{video, Intent, Ui};

/// Where the picture-in-picture was let go, where it was last laid out, and
/// the drag in progress: where it began and where the pointer is.
#[derive(Debug, Default)]
pub struct Pip {
    at: Option<Point>,
    shown: Option<Point>,
    drag: Option<(Point, Point)>,
}

/// How far the drag in progress has carried it.
fn carried(ui: &Ui) -> Point {
    ui.pip.drag.map_or(Point::new(0.0, 0.0), |(from, to)| {
        Point::new(to.x - from.x, to.y - from.y)
    })
}

/// Its top left corner in a window of `window`: where it was let go, else
/// Discord's bottom right, carried by a drag in progress, inside the window.
pub(crate) fn place(ui: &Ui, window: Size) -> Point {
    let at = ui.pip.at.unwrap_or_else(|| parts::pip_home(window));
    let by = carried(ui);
    parts::pip_inside(Point::new(at.x + by.x, at.y + by.y), window)
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
    // The simulation says so after the name: Discord's has no badge on it.
    let simulated = crate::voice::simulated(store).then(|| t("Simulada").to_owned());
    let at = place(ui, window);
    let watched = store.watching().iter().find(|watching| {
        !ui.video.windows.contains(&watching.owner)
            && !matches!(
                watching.phase,
                StreamPhase::Ended(_) | StreamPhase::Waiting | StreamPhase::Failed(_)
            )
    });
    let channel = store.channel_title(call.channel);
    let server = store.channel(call.channel).and_then(|found| found.server);
    if let Some(watching) = watched {
        let frame = ui
            .video
            .streams
            .iter()
            .find(|(owner, _)| *owner == watching.owner)
            .and_then(|(_, live)| live.frame());
        let name = store.display_name(watching.owner, server);
        return Some(PipView {
            surface: ids::stream_surface(watching.owner.0),
            title: tr!("Pantalla de {name}", name = &name),
            channel,
            name,
            frame,
            waiting: t("Esperando video…").into(),
            note: simulated,
            viewers: store.stream_viewers(watching.owner).len(),
            buttons: vec![
                PipButton {
                    id: ids::window(watching.owner.0),
                    hint: t("Abrir en otra ventana"),
                    glyph: PipGlyph::PopOut,
                },
                PipButton {
                    id: ids::stop_watching(watching.owner.0),
                    hint: t("Dejar de ver"),
                    glyph: PipGlyph::StopShare,
                },
            ],
            at,
        });
    }
    video::sending(ui, store)?;
    let me = store.me()?;
    Some(PipView {
        surface: ids::SCREEN_SURFACE.into(),
        title: t("Tu pantalla").into(),
        channel,
        name: store.display_name(me, server),
        frame: ui.video.screen.live.frame(),
        waiting: t("Esperando al sistema…").into(),
        note: simulated,
        viewers: store.stream_viewers(me).len(),
        buttons: vec![PipButton {
            id: ids::OWN_STREAM_STOP.into(),
            hint: t("Dejar de transmitir"),
            glyph: PipGlyph::StopShare,
        }],
        at,
    })
}

/// The picture-in-picture over `root` at its place, if it shows.
pub(crate) fn overlay(ui: &Ui, store: &Store, root: Element, window: Size, look: &Look) -> Element {
    let Some(view) = view(ui, store, window) else {
        return root;
    };
    Element::stack(vec![root, parts::pip(&view, look)])
}

/// After the layout: where it is, so a drag let go leaves it there.
pub(crate) fn hold(ui: &mut Ui, tree: &UiTree) {
    ui.pip.shown = tree
        .frame()
        .regions
        .iter()
        .rev()
        .find(|region| region.id == ids::PIP)
        .map(|region| Point::new(region.rect.x, region.rect.y));
}

/// Answer the picture-in-picture's clicks and drags; `None` for other parts.
pub(crate) fn update(ui: &mut Ui, store: &Store, action: &UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::Clicked { id } if id == ids::PIP => {
            ui.voice.stage = store.call().map(|call| call.channel);
            ui.voice.card = None;
            Some(Vec::new())
        }
        UiAction::DragStarted { id, position } if id == ids::PIP => {
            ui.pip.drag = Some((*position, *position));
            Some(Vec::new())
        }
        UiAction::DragMoved { id, position, .. } if id == ids::PIP => {
            if let Some((_, to)) = &mut ui.pip.drag {
                *to = *position;
            }
            Some(Vec::new())
        }
        // Where it was last drawn (inside the window), and any last step of
        // the pointer since.
        UiAction::Dropped { id, position, .. } if id == ids::PIP => {
            if let (Some(shown), Some((_, to))) = (ui.pip.shown, ui.pip.drag) {
                ui.pip.at = Some(Point::new(
                    shown.x + position.x - to.x,
                    shown.y + position.y - to.y,
                ));
            }
            ui.pip.drag = None;
            Some(Vec::new())
        }
        UiAction::DragCancelled { id } if id == ids::PIP => {
            ui.pip.drag = None;
            Some(Vec::new())
        }
        _ => None,
    }
}
