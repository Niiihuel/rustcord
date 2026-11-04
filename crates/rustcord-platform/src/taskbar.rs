//! The window's button on the Windows taskbar during a call, as Discord's:
//! Rustcord's icon stays as it is (the tray shows the call's state) and the
//! preview over the button has Discord's buttons: the camera, the
//! microphone, the sound and hanging up. Out of a call there are none.
//! The pictures are drawn and the shell is
//! told on a thread of their own; only the clicks come through the window's
//! own thread. Elsewhere it is not started.

use std::num::NonZeroIsize;
use std::sync::{mpsc, Arc};

use rustcord_i18n::Language;

#[cfg(any(windows, test))]
mod windows;

/// The account in its call, as the button shows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskbarCall {
    /// The microphone off, by its choice or the server's.
    pub muted: bool,
    /// The sound off, by its choice or the server's.
    pub deafened: bool,
    pub camera: bool,
}

/// What the button shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskbarState {
    /// `None` out of a call: no buttons.
    pub call: Option<TaskbarCall>,
    /// The preview's glyphs, as RGB.
    pub ink: [u8; 3],
    /// What is off (the microphone, the sound) is red.
    pub danger: [u8; 3],
    /// What the tooltips are written in.
    pub language: Language,
}

/// A button of the preview, clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskbarEvent {
    ToggleCamera,
    ToggleMute,
    ToggleDeafen,
    Disconnect,
}

/// What the drawing thread is told.
#[cfg_attr(not(windows), allow(dead_code))]
enum Job {
    State(TaskbarState),
    /// The window's button is there (again): its buttons go on it anew.
    Created,
}

/// The window's button. Dropping it stops the drawing; what was shown
/// stays until the window goes.
pub struct Taskbar {
    jobs: mpsc::Sender<Job>,
    shown: Option<TaskbarState>,
    events: mpsc::Receiver<TaskbarEvent>,
}

impl Taskbar {
    /// Take the button of `window`, a Win32 `HWND`; called on the window's
    /// own thread. `wake` runs after each click. `None` off Windows, or when
    /// the window cannot be watched.
    pub fn start(window: NonZeroIsize, wake: Arc<dyn Fn() + Send + Sync>) -> Option<Self> {
        let (sender, events) = mpsc::channel();
        let report = move |event: TaskbarEvent| {
            if sender.send(event).is_ok() {
                wake();
            }
        };
        #[cfg(windows)]
        let jobs = windows::start(window.get(), report);
        #[cfg(not(windows))]
        let jobs: Option<mpsc::Sender<Job>> = {
            let _ = (window, report);
            None
        };
        Some(Self {
            jobs: jobs?,
            shown: None,
            events,
        })
    }

    /// Show `state`; nothing is sent when it did not change. Never blocks.
    pub fn update(&mut self, state: TaskbarState) {
        if self.shown.as_ref() != Some(&state) {
            let _ = self.jobs.send(Job::State(state.clone()));
            self.shown = Some(state);
        }
    }

    pub fn poll(&self) -> Option<TaskbarEvent> {
        self.events.try_recv().ok()
    }
}
