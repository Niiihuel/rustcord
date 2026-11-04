//! Voice, from plain props: the panel of the account's call above its own
//! panel, the popover of the connection's details, the people in a voice
//! channel under it in the column, and the call's stage where the
//! conversation goes: its tiles and the bar of controls under them, or the
//! invitation to join while the account is not in it.
//!
//! What a provider cannot do is shown disabled with its reason, never left to
//! fail: with no video, the camera and the screen say so.

use limn::{
    Access, AlertDialog, Button, ButtonSize, ButtonSizing, ButtonVariant, ChartSeries, Checkbox,
    Color, Component, ComponentPalette, ControlSize, CornerRadii, DropdownMenu, DropdownMenuEntry,
    DropdownMenuItem, Element, FontFamily, Icon, IconButton, LineChart, Modal, Point,
    PopoverPlacement, RadiusRole, Role, ScrollArea, ScrollbarAppearance, Size, Slider, Switch,
    Tooltip, TooltipPlacement, TooltipSpec,
};
use rustcord_i18n::{es, t, tr};
use rustcord_model::{CallPhase, CallQuality, Rgb, ShareKind, StreamQuality};

use crate::identity::{activity_section, ActivityCard};
use crate::{centered, gap, icon, line, strong_line, text, Look};

/// Signal bars, for the connection.
pub const SIGNAL: &str = crate::glyphs::glyph!("signal");

/// The actions these pieces report. Those that name a person are built with
/// the functions here.
pub mod ids {
    /// The microphone of the account in its call.
    pub const MUTE: &str = "vz:mute";
    /// Hang up, from the panel or from the stage.
    pub const LEAVE: &str = "vz:leave";
    /// Join the channel the stage shows, or try again after a failure there.
    pub const JOIN: &str = "vz:join";
    /// The panel's way to try again once the call failed.
    pub const RETRY: &str = "vz:retry";
    /// The connection's details: the signal of the panel, and its title.
    pub const DETAILS: &str = "vz:details";
    pub const DETAILS_TITLE: &str = "vz:details:title";
    pub const DETAILS_DISMISS: &str = "vz:details:dismiss";
    /// The bar's «Más opciones» and its menu.
    pub const MORE: &str = "vz:more";
    pub const MORE_DISMISS: &str = "vz:more:dismiss";
    pub const VOICE_SETTINGS: &str = "vz:voice-settings";
    /// The camera and the screen of the account, on and off.
    pub const CAMERA: &str = "vz:camera";
    pub const SCREEN: &str = "vz:screen";
    /// «Dejar de compartir» and «Apagar la cámara» on their previews.
    pub const STOP_SCREEN: &str = "vz:stop-screen";
    /// The account's own stream as a card in the grid (Discord's «¡Estás en
    /// directo!»): a click focuses it on the stage, its button stops the share.
    pub const OWN_STREAM: &str = "vz:own-stream";
    pub const OWN_STREAM_STOP: &str = "vz:own-stream:stop";
    /// The picture-in-picture outside the call's stage: a click on it or its
    /// channel's name returns to the call.
    pub const PIP: &str = "pip:return";
    /// The surface the still of the account's own stream goes on, on its card.
    pub const OWN_THUMB_SURFACE: &str = "video:own-thumb";
    /// The quality picker that opens before sharing the screen: its
    /// sizes (`vz:quality:720`), rates (`vz:fps:30`), and going live.
    pub const QUALITY: &str = "vz:quality:";
    pub const FPS: &str = "vz:fps:";
    pub const SHARE: &str = "vz:share";
    /// «Con sonido» in the picker: the applications' sound goes along.
    pub const SOUND: &str = "vz:sound";
    /// «Elegir otra pantalla» in the picker: the system's asks again.
    pub const PICK_SCREEN: &str = "vz:pick-screen";
    /// A screen or a window in Rustcord's own picker, where the system has
    /// none (Windows): `vz:source:{id}`, by the host's id of it.
    pub const SOURCE: &str = "vz:source:";
    pub const SHARE_DISMISS: &str = "vz:share:dismiss";
    /// The Go Live modal's tabs: applications, and devices.
    pub const SHARE_APPS: &str = "vz:share:apps";
    pub const SHARE_DEVICES: &str = "vz:share:devices";
    /// The native picker's two cards: the whole screen, or a window.
    pub const SHARE_SCREEN: &str = "vz:share:kind:screen";
    pub const SHARE_WINDOW: &str = "vz:share:kind:window";
    /// The footer's presets: 720p at 30, and 1080p at 60.
    pub const SHARE_SD: &str = "vz:share:sd";
    pub const SHARE_HD: &str = "vz:share:hd";
    /// The gear beside the button, and its popout's dismissal.
    pub const SHARE_OPTIONS: &str = "vz:share:options";
    pub const SHARE_OPTIONS_DISMISS: &str = "vz:share:options:dismiss";
    pub const STOP_CAMERA: &str = "vz:stop-camera";
    /// «Dejar de transmitir» on the panel's streaming row.
    pub const STREAM_STOP: &str = "vz:stream-stop";
    /// The wave beside the hang-up: the noise suppression popout, its switch
    /// (the settings' own noise action), its dismissal and the voice settings.
    pub const NOISE: &str = "vz:noise";
    pub const NOISE_DISMISS: &str = "vz:noise:dismiss";
    pub const NOISE_SWITCH: &str = "us:noise";
    pub const NOISE_SETTINGS: &str = "us:page=voice";
    /// The action row's two buttons Rustcord cannot act on yet.
    /// The surfaces the host presents the previews' frames on.
    pub const SCREEN_SURFACE: &str = "video:screen";
    pub const CAMERA_SURFACE: &str = "video:camera";
    /// «Dejar de ver» on a stream watched: `vz:stop-watching:{persona}`.
    pub const STOP_WATCHING: &str = "vz:stop-watching:";
    /// «Pausar» and «Reanudar» on the preview of the shared screen.
    pub const PAUSE: &str = "vz:pause";
    pub const RESUME: &str = "vz:resume";
    /// «Salir de pantalla completa» on a stream shown alone.
    pub const FULLSCREEN_EXIT: &str = "vz:fullscreen-exit";
    /// The stage's header: the channel's chat.
    pub const CHAT: &str = "vz:chat";
    /// The header's warning over a reduced stream, and its quality segment:
    /// the anchors of their hints.
    pub const WARNING: &str = "vz:stream-warning";
    pub const STREAM_QUALITY: &str = "vz:stream-quality";
    /// The spectators of `owner`'s stream on the header, and the dismissal of
    /// the list their hover opens.
    pub fn viewers(owner: u64) -> String {
        format!("vz:viewers:{owner}")
    }

    pub fn viewers_dismiss(owner: u64) -> String {
        format!("vz:viewers:dismiss:{owner}")
    }

    /// One spectator's picture in that stack, named for them.
    pub fn viewer(owner: u64, user: u64) -> String {
        format!("vz:viewer:{owner}:{user}")
    }
    /// The pill over the bar that hides the tiles under a stream focused.
    pub const TILES: &str = "call:tiles:toggle";
    /// The bar's invitation to the call's channel.
    pub const INVITE: &str = "vz:invite";
    /// A person's menu, and their card.
    pub const MENU_DISMISS: &str = "vz:menu:dismiss";
    pub const CARD_DISMISS: &str = "vz:card:dismiss";
    /// Items of a person's menu, followed by their ID.
    pub const MENTION: &str = "vz:menu:mention:";
    pub const COPY_ID: &str = "vz:menu:copy-id:";
    /// The account's own microphone and sound, from its menu.
    pub const MENU_MUTE: &str = "vz:menu:mute";
    pub const MENU_DEAFEN: &str = "vz:menu:deafen";
    /// The question before a call from a person's menu: its «Iniciar llamada»,
    /// its «Cancelar», and the check box «No volver a preguntar».
    pub const PERSON_CALL: &str = "vz:person-call";
    pub const PERSON_CALL_CANCEL: &str = "vz:person-call:cancel";
    pub const PERSON_CALL_FORGET: &str = "vz:person-call:forget";
    /// The arrow beside the bar's microphone: its quick voice panel,
    /// [`crate::quick_voice`].
    pub const INPUT_OPTIONS: &str = "vz:input-options";
    /// The arrow beside the bar's camera: the camera's panel, [`crate::quick_voice::camera`].
    pub const CAMERA_OPTIONS: &str = "vz:camera-options";

    /// Someone's row under a voice channel: a click opens their card.
    pub fn row(user: u64) -> String {
        format!("vz:user:{user}")
    }

    /// Someone's tile on the stage.
    pub fn tile(user: u64) -> String {
        format!("vz:tile:{user}")
    }

    /// The «⋯» of someone's tile, shown while the pointer is on it.
    pub fn tile_more(user: u64) -> String {
        format!("vz:tile-more:{user}")
    }

    /// Someone's avatar in a private call's VOICE mode: their menu on a
    /// secondary click, nothing on a plain one.
    pub fn avatar(user: u64) -> String {
        format!("vz:avatar:{user}")
    }

    /// How loud someone plays for the account, from their menu.
    pub fn volume(user: u64) -> String {
        format!("vz:volume:{user}")
    }

    /// Silence someone for the account only, from their menu.
    pub fn local_mute(user: u64) -> String {
        format!("vz:mute:{user}")
    }

    /// «Mirar transmisión» on the tile of a stream nobody watches: it is
    /// watched alone.
    pub fn watch(user: u64) -> String {
        format!("vz:watch:{user}")
    }

    /// The icon beside it: the stream is added to those watched.
    pub fn watch_add(user: u64) -> String {
        format!("vz:watch-add:{user}")
    }

    /// The tile of that stream as a whole: a click watches it, as `watch` does.
    pub fn watch_tile(user: u64) -> String {
        format!("vz:stream-tile:{user}")
    }

    /// The card of someone live beside their row, shown on hover. Each its
    /// own: limn holds a hover card open while the pointer is on the surface
    /// named by it, so a shared one opened every card at once.
    pub fn live_dismiss(user: u64) -> String {
        format!("vz:live:dismiss:{user}")
    }

    /// The account's own stream's button on its own card (disabled).
    pub fn live_own(user: u64) -> String {
        format!("vz:live:own:{user}")
    }

    /// «Mirar transmisión» on the card of someone live beside their row.
    pub fn live_watch(user: u64) -> String {
        format!("vz:live:watch:{user}")
    }

    /// «Dejar de ver» on `user`'s stream.
    pub fn stop_watching(user: u64) -> String {
        format!("{STOP_WATCHING}{user}")
    }

    /// «Cerrar transmisión» on a stream that is over: it lets the stream go
    /// as [`stop_watching`] does, under its own ID, since the call bar's red
    /// button keeps that one and limn styles a control by its ID.
    pub const CLOSE_STREAM: &str = "vz:close-stream:";

    pub fn close_stream(user: u64) -> String {
        format!("{CLOSE_STREAM}{user}")
    }

    /// «Déjanos ayudarte.» under a stream that failed to start: the help article.
    pub const STREAM_HELP: &str = "vz:stream-help";

    /// «Pantalla completa» on `user`'s stream: alone, the window full screen.
    pub fn fullscreen(user: u64) -> String {
        format!("vz:fullscreen:{user}")
    }

    /// «Abrir en otra ventana» on `user`'s stream: in a window of its own too.
    pub fn window(user: u64) -> String {
        format!("vz:window:{user}")
    }

    /// The surface the host presents `user`'s stream on in its own window.
    pub fn window_surface(user: u64) -> String {
        format!("video:window:{user}")
    }

    /// The surface the host presents someone's stream on.
    pub fn stream_surface(user: u64) -> String {
        format!("video:stream:{user}")
    }

    /// The surface the host presents someone's camera on, in their tile.
    pub fn camera_surface(user: u64) -> String {
        format!("video:camera:{user}")
    }

    /// `user`'s stream watched on the stage: a click focuses it, or lets go
    /// of the focus, a double click shows it full screen, a secondary click
    /// opens its menu.
    pub fn stream_tile(user: u64) -> String {
        format!("vz:stream:{user}")
    }

    /// The «⋯» of `user`'s stream, shown while the pointer is on it: its
    /// menu, anchored there.
    pub fn stream_more(user: u64) -> String {
        format!("vz:stream-more:{user}")
    }

    /// «Acercar» and «Alejar» on the focused `user`'s stream.
    pub fn zoom_in(user: u64) -> String {
        format!("vz:zoom-in:{user}")
    }

    pub fn zoom_out(user: u64) -> String {
        format!("vz:zoom-out:{user}")
    }

    /// «Enfocar» `user`'s stream, or «Volver a la grilla» while it is.
    pub fn spotlight(user: u64) -> String {
        format!("vz:spotlight:{user}")
    }

    /// The stage's bar and its header, hover regions: while the pointer is
    /// on one, what hides once the pointer rests stays.
    pub const BAR: &str = "vz:bar";
    pub const HEADER: &str = "vz:header";

    /// The controls over `user`'s stream, a hover region as [`BAR`] is.
    pub const STREAM_CONTROLS: &str = "vz:stream-controls:";
    pub fn stream_controls(user: u64) -> String {
        format!("{STREAM_CONTROLS}{user}")
    }
}

/// Why the camera and the screen cannot be used on this system.
pub const NO_VIDEO: &str =
    es("La vista previa de pantalla y cámara solo funciona en Linux por ahora");

/// The account's camera and screen, as the call's controls show them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VideoControls {
    pub screen: bool,
    pub camera: bool,
    /// Why they cannot be used here, if they cannot.
    pub denied: Option<String>,
    /// What their hints add: that what they show stays on this machine.
    pub note: String,
    /// What the screen's hint adds, when it is not `note`.
    pub screen_note: Option<String>,
    /// What the camera's hint adds, when it is not `note`.
    pub camera_note: Option<String>,
    /// The quality picker, open on the bar's screen button, and what is
    /// chosen in it.
    pub picker: Option<StreamQuality>,
    /// The applications' sound goes along with the screen.
    pub sound: bool,
    /// The system remembers the screen chosen: it can be chosen anew.
    pub pick_again: bool,
    /// What the picker lets the account share, on the native picker's cards.
    pub kind: ShareKind,
    /// The picker's devices tab is the one shown.
    pub devices: bool,
    /// The gear's popout is open, over the picker.
    pub options: bool,
    /// What Rustcord's own picker offers to share, where the system has no
    /// dialog of its own (Windows); empty elsewhere.
    pub sources: Vec<ScreenChoice>,
}

/// A screen or a window Rustcord's own picker offers, the one chosen checked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScreenChoice {
    pub id: String,
    pub name: String,
    pub window: bool,
    pub chosen: bool,
}

/// The camera's and the screen's buttons, `height` high. Off, they fill
/// `idle` (and `idle_hover` under the pointer); on, they fill `lit` and
/// `lit_hover`, and with `edge` they are outlined in it too (the panel's).
fn video_buttons(
    controls: &VideoControls,
    height: f32,
    (idle, idle_hover): (Color, Color),
    (lit, lit_hover): (Color, Color),
    edge: Option<Color>,
    look: &Look,
) -> [Element; 2] {
    let button = |id: &str, on: bool, glyph: Icon, titles: (&str, &str)| {
        let (fill, hover, ink) = if on {
            (lit, lit_hover, look.online)
        } else {
            (idle, idle_hover, look.text)
        };
        let button = IconButton::new(id, glyph, filled(fill, hover, look))
            .variant(ButtonVariant::Raised)
            .sizing(ButtonSizing {
                height,
                font_size: 14.0,
                icon_size: 20.0,
                horizontal_padding: 8.0,
                inner_spacing: 6.0,
            })
            .radius(RadiusRole::Card)
            .active(on);
        let rendered = match &controls.denied {
            Some(reason) => button
                .ink(look.faint)
                .tooltip_with(denied_hint(titles.1, t(reason)))
                .disabled()
                .render(),
            None => button
                .ink(ink)
                .hover_ink(ink)
                .tooltip_with(
                    TooltipSpec::new(if on { titles.0 } else { titles.1 }).description(
                        match (id, &controls.screen_note, &controls.camera_note) {
                            (ids::SCREEN, Some(note), _) | (ids::CAMERA, _, Some(note)) => {
                                note.clone()
                            }
                            _ => controls.note.clone(),
                        },
                    ),
                )
                .render(),
        };
        // The panel's buttons are outlined in both states: the lit ones in `edge`.
        // The outline follows the wrapper's own radius, so it matches the button's.
        match edge {
            Some(lit) => rendered
                .radius(RadiusRole::Card.px())
                .border(if on { lit } else { look.border }, 1.0),
            None => rendered,
        }
    };
    [
        button(
            ids::CAMERA,
            controls.camera,
            if controls.camera {
                Icon::Video
            } else {
                Icon::VideoOff
            },
            (t("Apagar la cámara"), t("Encender la cámara")),
        ),
        button(
            ids::SCREEN,
            controls.screen,
            // Discord's: a screen with an arrow, crossed out while shared.
            Icon::Custom(if controls.screen {
                SCREEN_STOP
            } else {
                crate::glyphs::SCREEN_SHARE
            }),
            (t("Dejar de compartir"), t("Compartir pantalla")),
        ),
    ]
}

/// The account's screen or camera on the stage, shown only here.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    /// The surface the host presents its frames on.
    pub surface: String,
    /// «Tu pantalla», «Tu cámara», «Pantalla de Ana».
    pub title: String,
    /// The size of the frame on the surface; `None` while the system asks
    /// or the first frame is on its way.
    pub frame: Option<(u32, u32)>,
    /// The frames a second it comes at, once a second of them was counted;
    /// for the account's own share, what it sends.
    pub fps: Option<u32>,
    /// What shows until then: «Esperando al sistema…».
    pub waiting: String,
    /// What stops it, and how it reads.
    pub stop: (String, &'static str),
    /// That it stays on this machine.
    pub note: String,
    /// What else it offers beside the way to stop it, and how each reads:
    /// «Pausar».
    pub actions: Vec<(String, &'static str)>,
    /// A stream watched: what its tile offers in place of `stop` and
    /// `actions`.
    pub stream: Option<StreamTile>,
    /// It takes the stage, the rest in a strip under it.
    pub focused: bool,
    /// The account's own stream as a card in the grid: its still, and a
    /// click focuses it ([`ids::OWN_STREAM`]).
    pub card: bool,
}

/// The zoom of a focused stream's picture, from 1× (the fit) to 5×, by
/// quarters with the buttons, as the official client's.
pub const ZOOM_MIN: f32 = 1.0;
pub const ZOOM_MAX: f32 = 5.0;
pub const ZOOM_STEP: f32 = 0.25;

/// A zoomed picture: its level, and how far its middle is from the middle of
/// its tile, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    pub level: f32,
    pub pan: (f32, f32),
}

impl Zoom {
    /// The picture as it fits its tile.
    pub const FIT: Zoom = Zoom {
        level: ZOOM_MIN,
        pan: (0.0, 0.0),
    };
}

/// A frame as it fits a tile, at 1×.
pub fn fitted(frame: (u32, u32), room: Size) -> Size {
    let scale = (room.width / frame.0 as f32).min(room.height / frame.1 as f32);
    Size::new(frame.0 as f32 * scale, frame.1 as f32 * scale)
}

/// `zoom` at `level` about `pivot` (from the tile's middle, so the point of
/// the picture under it stays there), then moved by `by`. A picture larger
/// than its tile never leaves the tile's edges bare.
pub fn zoom_to(
    zoom: Zoom,
    level: f32,
    pivot: (f32, f32),
    by: (f32, f32),
    picture: Size,
    room: Size,
) -> Zoom {
    let level = level.clamp(ZOOM_MIN, ZOOM_MAX);
    let ratio = level / zoom.level;
    let moved = |at: f32, pan: f32, by: f32| at - (at - pan) * ratio + by;
    let reach = |picture: f32, room: f32| ((picture * level - room) / 2.0).max(0.0);
    let (x, y) = (
        reach(picture.width, room.width),
        reach(picture.height, room.height),
    );
    Zoom {
        level,
        pan: (
            moved(pivot.0, zoom.pan.0, by.0).clamp(-x, x),
            moved(pivot.1, zoom.pan.1, by.1).clamp(-y, y),
        ),
    }
}

/// The zoomed picture of a frame on a surface, moved by its pan from the
/// middle of the tile; the tile clips what goes past it.
fn zoomed_surface(view: &Preview, frame: (u32, u32), size: Size, zoom: Zoom) -> Element {
    let fit = fitted(frame, size);
    let (width, height) = (fit.width * zoom.level, fit.height * zoom.level);
    Element::column(vec![Element::row(vec![Element::surface(
        view.surface.clone(),
        view.title.clone(),
    )
    .size(Some(width), Some(height))
    .translate(Point::new(
        (size.width - width) / 2.0 + zoom.pan.0,
        (size.height - height) / 2.0 + zoom.pan.1,
    ))])])
}

/// A stream watched, as its tile offers it, as in the
/// official client: a click focuses it, a double click shows it full
/// screen. Its name goes in a pill at the bottom left and its controls,
/// icons in round chips, at the bottom right: its sound and «⋯», and while
/// it takes the stage its own window and full screen too. Then hanging up
/// gives way to «Dejar de ver».
#[derive(Clone, Debug, PartialEq)]
pub struct StreamTile {
    pub owner: u64,
    /// The streamer's name alone: the pill at the bottom left says it, where
    /// the title of the picture says «Pantalla de» before it.
    pub name: String,
    /// The key of its sound for the account's mixer, apart from the voice's.
    pub sound: u64,
    /// Its sound is silenced for the account.
    pub silenced: bool,
    /// It shows in a window of its own already.
    pub windowed: bool,
    /// Its menu, while open from its «⋯».
    pub menu: Option<Vec<MenuEntry>>,
    /// It fills the window, full screen: its way back in place of full
    /// screen.
    pub fullscreen: bool,
    /// It is over for now (stopped, ended or failed): the stage says so with
    /// a picture at rest in place of the stream, and no note.
    pub over: bool,
    /// It failed to start, or ended while its owner still streams: the title
    /// says so, and `waiting` is the reason. Else `waiting` is the title.
    pub failed: bool,
    /// The pointer rested on it a while: what is drawn over it fades out,
    /// the bar under it too, and the pointer hides over it, as the official
    /// client's (3 s). Only while it takes the stage or the window.
    pub idle: bool,
    /// The game its streamer plays, named in the stage's header.
    pub activity: Option<String>,
    /// Its picture is reduced (the network is slow): the header warns.
    pub degraded: bool,
    /// Who watches it: the header's viewers, none drawn yet.
    pub viewers: Vec<Participant>,
    /// Its picture's zoom, while it takes the stage or the window: its
    /// buttons are shown then.
    pub zoom: Option<Zoom>,
}

/// Rustcord's own glyphs for «Dejar de ver» and «Abrir en otra ventana».
const SCREEN_STOP: &str = crate::glyphs::glyph!("screen-stop");
const POP_OUT: &str = crate::glyphs::glyph!("pop-out");
const ZOOM_IN: &str = crate::glyphs::glyph!("add");
const ADD_STREAM: &str = crate::glyphs::glyph!("add");
const ZOOM_OUT: &str = crate::glyphs::glyph!("minus");

/// The level of a zoom as a percentage, «150 %», in a pill as the name's.
fn zoom_level(level: f32) -> Element {
    let percent = (level * 100.0).round() as u32;
    Element::panel(Element::row(vec![
        gap(10.0, 0.0),
        strong_line(tr!("{percent} %", percent), 14.0, OVER_INK)
            .centered_y()
            .shrink_to_fit(),
        gap(10.0, 0.0),
    ]))
    .size(None, Some(OVER_BUTTON))
    .radius(OVER_BUTTON / 2.0)
    .background(OVER)
    .shrink_to_fit()
}

/// What is drawn over a picture, as the official client's chips: black at
/// 60 %, 80 % under the pointer, with white on it whatever the theme.
const OVER: Color = Color::rgba(0, 0, 0, 153);
const OVER_HOVER: Color = Color::rgba(0, 0, 0, 204);
const OVER_INK: Color = Color::rgb(255, 255, 255);
/// The name of a tile, and the labels over its picture: black at 56 %, as
/// Discord's name label.
const PILL: Color = Color::rgba(0, 0, 0, 143);
/// The ground of a stream nobody watches: Discord's stream preview, #1E1F22.
const STREAM_GROUND: Color = Color::rgb(30, 31, 34);
/// The buttons over a picture, 32 across, rounded 8 and outlined in
/// `#97979f` at 16 % as Discord's «Más» on a tile.
const OVER_BUTTON: f32 = 32.0;
const OVER_EDGE: Color = Color::rgba(0x97, 0x97, 0x9f, 41);

/// An icon button over a picture, named and hinted by `hint`.
fn over_button(
    id: impl Into<String>,
    glyph: Icon,
    hint: &str,
    fill: Color,
    look: &Look,
) -> Element {
    over_button_at(id, glyph, hint, fill, true, look)
}

/// [`over_button`], or off when `enabled` is false: what it does is not
/// possible now, as «Acercar» at the largest zoom.
fn over_button_at(
    id: impl Into<String>,
    glyph: Icon,
    hint: &str,
    fill: Color,
    enabled: bool,
    look: &Look,
) -> Element {
    let hover = if fill == OVER {
        OVER_HOVER
    } else {
        lighter(fill, 18)
    };
    let button = IconButton::new(id, glyph, filled(fill, hover, look))
        .variant(ButtonVariant::Raised)
        .sizing(ButtonSizing {
            height: OVER_BUTTON,
            font_size: 14.0,
            icon_size: 18.0,
            horizontal_padding: 7.0,
            inner_spacing: 6.0,
        })
        .width(OVER_BUTTON)
        .radius(RadiusRole::Card)
        .ink(OVER_INK)
        .hover_ink(OVER_INK)
        .tooltip(hint)
        .accessibility_label(hint);
    let button = if enabled {
        button.render()
    } else {
        button.disabled().render()
    };
    // The outline follows the wrapper's own radius, as the button's.
    button.radius(RadiusRole::Card.px()).border(OVER_EDGE, 1.0)
}

/// A preview's title in a tile's name tag at its bottom left (see
/// [`name_tag`]); a stream's is its streamer's name behind a screen, as the
/// official client's. Nothing on a tile too small for a name.
fn name_pill(view: &Preview, size: Size) -> Element {
    let (said, marks) = match &view.stream {
        Some(stream) => (
            stream.name.as_str(),
            vec![icon(Icon::Screen, OVER_INK).size(Some(16.0), Some(16.0))],
        ),
        None => (view.title.as_str(), Vec::new()),
    };
    name_tag(said, marks, size).unwrap_or_else(Element::spacer)
}

/// A small note over a picture, such as that it is simulated: it says, it
/// is no button.
fn badge(note: &str) -> Element {
    Element::panel(Element::row(vec![
        gap(8.0, 0.0),
        line(note, 12.0, OVER_INK).centered_y().shrink_to_fit(),
        gap(8.0, 0.0),
    ]))
    .size(None, Some(22.0))
    .radius(4.0)
    .background(OVER)
    .shrink_to_fit()
}

/// A preview's tile: the frame as large as fits, its note and the way to
/// stop it on top, its title in a chip at the bottom left. The camera's is
/// mirrored, as a webcam shows the person in front of it; what goes to the
/// call is not.
pub fn preview(view: &Preview, size: Size, look: &Look) -> Element {
    preview_tile(view, size, false, look)
}

/// [`preview`], or `compact` in the strip under a stream focused: its
/// picture and its title only.
fn preview_tile(view: &Preview, size: Size, compact: bool, look: &Look) -> Element {
    let picture = match view.frame {
        Some((width, height)) => match view.stream.as_ref().and_then(|stream| stream.zoom) {
            Some(zoom) => zoomed_surface(view, (width, height), size, zoom),
            None => {
                let scale = (size.width / width as f32).min(size.height / height as f32);
                centered(
                    Element::surface(view.surface.clone(), view.title.clone())
                        .surface_mirrored(view.surface == ids::CAMERA_SURFACE)
                        .size(Some(width as f32 * scale), Some(height as f32 * scale)),
                )
            }
        },
        // A stream over says so, and offers to be watched again or closed.
        None => match &view.stream {
            Some(stream) if stream.over => over_picture(view, look),
            // The account's own stream before its first frame: the screen's glyph.
            None if view.card => {
                centered(icon(Icon::Screen, look.faint).size(Some(48.0), Some(48.0)))
            }
            _ => centered(line(&view.waiting, 14.0, look.muted)),
        },
    };
    let title = name_pill(view, size);
    let over = match &view.stream {
        // In the strip, a stream watched says it is live at its top right.
        Some(_) if compact => Element::column(vec![
            Element::row(vec![Element::spacer().flex(1.0), live(22.0, 10.0, look)]),
            Element::spacer().flex(1.0),
            Element::row(vec![title, Element::spacer().flex(1.0)]),
        ])
        .padding(8.0),
        _ if compact => Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![title, Element::spacer().flex(1.0)]),
        ])
        .padding(8.0),
        None if view.card => own_over(view, title, look),
        // A stream watched: its ways out are in its menu and the bar.
        Some(stream) => stream_over(view, stream, title, look),
        // The account's own: pausing it and stopping it, as icons.
        None => {
            let mut buttons: Vec<Element> = view
                .actions
                .iter()
                .map(|(id, hint)| {
                    let glyph = match id.as_str() {
                        ids::RESUME => Icon::Play,
                        _ => Icon::Pause,
                    };
                    over_button(id.clone(), glyph, t(hint), OVER, look)
                })
                .collect();
            let glyph = match view.stop.0.as_str() {
                ids::STOP_SCREEN => Icon::Custom(SCREEN_STOP),
                ids::STOP_CAMERA => Icon::VideoOff,
                _ => Icon::StopFill,
            };
            buttons.push(over_button(
                view.stop.0.clone(),
                glyph,
                t(view.stop.1),
                look.danger,
                look,
            ));
            Element::column(vec![
                top_left(view, view.focused, look),
                Element::spacer().flex(1.0),
                Element::row(vec![
                    title,
                    Element::spacer().flex(1.0),
                    Element::row(buttons).gap(8.0),
                ])
                .gap(8.0),
            ])
            .padding(12.5)
        }
    };
    let backdrop = Element::panel(picture)
        .size(Some(size.width), Some(size.height))
        .radius(TILE_RADIUS)
        .background(Color::rgb(0, 0, 0));
    let shown = Element::stack(vec![backdrop.clip_children(), over]);
    match &view.stream {
        Some(stream) => {
            let tile = Element::clickable(ids::stream_tile(stream.owner), shown)
                .radius(TILE_RADIUS)
                .hover_region();
            if stream.idle && (view.focused || stream.fullscreen) {
                tile.cursor(limn::Cursor::Hidden)
            } else {
                tile
            }
        }
        None if view.card => Element::clickable(ids::OWN_STREAM, shown)
            .radius(TILE_RADIUS)
            .hover_region(),
        None => shown,
    }
    .size(Some(size.width), Some(size.height))
    .accessibility_label(view.title.clone())
}

/// The account's own stream's card over its still: «EN VIVO», its title, who
/// sees it and a red button to stop (Discord's card of the same).
fn own_over(view: &Preview, title: Element, look: &Look) -> Element {
    let stop = Button::new(
        view.stop.0.as_str(),
        view.stop.1,
        filled(look.danger, lighter(look.danger, 18), look),
    )
    .variant(ButtonVariant::Raised)
    .sizing(ButtonSizing {
        height: OVER_BUTTON,
        font_size: 14.0,
        icon_size: 18.0,
        horizontal_padding: 12.0,
        inner_spacing: 6.0,
    })
    .radius(RadiusRole::Card)
    .ink(OVER_INK)
    .hover_ink(OVER_INK)
    .render();
    Element::column(vec![
        Element::row(vec![live(22.0, 10.0, look), Element::spacer().flex(1.0)]),
        Element::spacer().flex(1.0),
        Element::row(vec![title, Element::spacer().flex(1.0)]),
        gap(0.0, 6.0),
        Element::row(vec![
            line(&view.note, 12.0, OVER_INK)
                .centered_y()
                .shrink_to_fit(),
            Element::spacer().flex(1.0),
            stop,
        ])
        .gap(8.0),
    ])
    .padding(12.5)
}

/// What a stream over shows, centred on its tile, with no picture: a failure's
/// title, its reason under it and the line to the help; an ended stream's
/// wording as its title. «Cerrar transmisión» comes 16 px under.
fn over_picture(view: &Preview, look: &Look) -> Element {
    let failed = view.stream.as_ref().is_some_and(|stream| stream.failed);
    let mut text = Vec::new();
    if failed {
        text.push(strong_line(
            t("No se pudo iniciar la transmisión :("),
            16.0,
            OVER_INK,
        ));
        text.push(strong_line(view.waiting.clone(), 14.0, look.muted));
        text.push(help_line(look));
    } else {
        text.push(strong_line(view.waiting.clone(), 16.0, OVER_INK));
    }
    let close = stream_text_button(&view.stop.0, t(view.stop.1), ButtonVariant::Secondary, look);
    centered(Element::column(vec![Element::column(text).gap(4.0), close]).gap(16.0))
}

/// «¿Tienes problemas con tu transmisión? Déjanos ayudarte.», where the last
/// words open the help article.
fn help_line(look: &Look) -> Element {
    Element::row(vec![
        line(t("¿Tienes problemas con tu transmisión?"), 14.0, look.muted),
        Element::clickable(
            ids::STREAM_HELP,
            line(t("Déjanos ayudarte."), 14.0, look.accent),
        ),
    ])
    .gap(4.0)
}

/// A text button on a stream that is over: «Cerrar transmisión», secondary.
fn stream_text_button(id: &str, label: &str, variant: ButtonVariant, look: &Look) -> Element {
    Button::new(id, label, look.sidebar_palette())
        .variant(variant)
        .sizing(ButtonSizing {
            height: PANEL_BUTTON,
            font_size: 14.0,
            icon_size: 20.0,
            horizontal_padding: 8.0,
            inner_spacing: 6.0,
        })
        .radius(RadiusRole::Card)
        .render()
}

/// What goes over a stream watched: its note as a badge at the top left,
/// its name at the bottom left and its controls at the bottom right. In
/// the grid the controls show under the pointer; while it takes the stage
/// or the window all of it shows until the pointer rests, then fades.
fn stream_over(view: &Preview, stream: &StreamTile, title: Element, look: &Look) -> Element {
    // Over: no note and no controls, only its name.
    if stream.over {
        return Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![title, Element::spacer().flex(1.0)]),
        ])
        .padding(12.5);
    }
    let big = view.focused || stream.fullscreen;
    if big && stream.idle {
        return Element::spacer();
    }
    let over = Element::column(vec![
        top_left(view, big, look),
        Element::spacer().flex(1.0),
        Element::row(vec![
            title,
            Element::spacer().flex(1.0),
            stream_buttons(stream, big, look),
        ])
        .gap(8.0),
    ])
    .padding(12.5);
    if big {
        over.animate_presence(format!("{}:over", ids::stream_tile(stream.owner)))
    } else {
        over
    }
}

/// The note's badge at the top left. Without a note a stream is live: «EN
/// VIVO» in the grid, as the official client's, and nothing on the stage,
/// whose header says it.
fn top_left(view: &Preview, big: bool, look: &Look) -> Element {
    let shown = match view.note.as_str() {
        "" if big => return Element::spacer().size(None, Some(0.0)),
        "" => live(22.0, 10.0, look),
        note => badge(note),
    };
    Element::row(vec![shown, Element::spacer().flex(1.0)])
}

/// A stream's controls at its tile's bottom right, as the official
/// client's: «⋯» and, in the grid, its sound under the pointer (kept while
/// silenced). On the stage the sound, its own window and full screen are on
/// the bar instead; a stream shown alone in full screen has no bar, so they
/// stay here.
fn stream_buttons(stream: &StreamTile, big: bool, look: &Look) -> Element {
    let reveal = |shown: Element, kept: bool, what: &str| {
        if kept || big {
            shown
        } else {
            shown.reveal_on_hover(format!("{}:{what}", ids::stream_tile(stream.owner)))
        }
    };
    let mut buttons = Vec::new();
    if !big || stream.fullscreen {
        let (glyph, hint) = if stream.silenced {
            (Icon::VolumeMuted, t("Activar el sonido"))
        } else {
            (Icon::Volume, t("Silenciar"))
        };
        buttons.push(reveal(
            over_button(ids::local_mute(stream.sound), glyph, hint, OVER, look),
            stream.silenced,
            "sound",
        ));
    }
    if stream.fullscreen {
        if !stream.windowed {
            buttons.push(over_button(
                ids::window(stream.owner),
                Icon::Custom(POP_OUT),
                t("Abrir en otra ventana"),
                OVER,
                look,
            ));
        }
        buttons.push(over_button(
            ids::FULLSCREEN_EXIT,
            Icon::Collapse,
            t("Salir de pantalla completa"),
            OVER,
            look,
        ));
    }
    // The focused stream's zoom: «Alejar» and the level once zoomed, then
    // «Acercar», off at the largest zoom.
    if let Some(zoom) = stream.zoom {
        if zoom.level > ZOOM_MIN {
            buttons.push(over_button(
                ids::zoom_out(stream.owner),
                Icon::Custom(ZOOM_OUT),
                t("Alejar"),
                OVER,
                look,
            ));
            buttons.push(zoom_level(zoom.level));
        }
        buttons.push(over_button_at(
            ids::zoom_in(stream.owner),
            Icon::Custom(ZOOM_IN),
            t("Acercar"),
            OVER,
            zoom.level < ZOOM_MAX,
            look,
        ));
    }
    let open = stream.menu.is_some();
    let more = DropdownMenu::new(
        over_button(
            ids::stream_more(stream.owner),
            Icon::More,
            t("Más opciones"),
            OVER,
            look,
        ),
        menu_entries(stream.menu.clone().unwrap_or_default(), look),
        open,
        ids::MENU_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::MENU_METRICS)
    .width(menu_width(stream.menu.as_deref().unwrap_or_default()))
    .placement(PopoverPlacement::TopEnd)
    .render();
    buttons.push(reveal(more, open, "more"));
    // The pointer on them keeps them, and the bar, from hiding.
    Element::row(buttons)
        .gap(8.0)
        .hover_region_id(ids::stream_controls(stream.owner))
}

/// A stream in a window of its own: the frame as large as fits, its note
/// and its title, nothing to click (the window's own close lets it go).
/// Once the pointer rests on it (`idle`) they fade out, as on the stage.
pub fn stream_window(view: &Preview, size: Size, idle: bool, look: &Look) -> Element {
    let picture = match view.frame {
        Some((width, height)) => {
            let scale = (size.width / width as f32).min(size.height / height as f32);
            centered(
                Element::surface(view.surface.clone(), view.title.clone())
                    .size(Some(width as f32 * scale), Some(height as f32 * scale)),
            )
        }
        None => centered(line(&view.waiting, 14.0, look.muted)),
    };
    let over = if idle {
        Element::spacer()
    } else {
        Element::column(vec![
            Element::row(vec![badge(&view.note), Element::spacer().flex(1.0)]),
            Element::spacer().flex(1.0),
            Element::row(vec![name_pill(view, size), Element::spacer().flex(1.0)]),
        ])
        .padding(12.5)
        .animate_presence(format!("{}:over", view.surface))
    };
    Element::stack(vec![
        Element::panel(picture)
            .size(Some(size.width), Some(size.height))
            .background(Color::rgb(0, 0, 0))
            .clip_children(),
        over,
    ])
    .size(Some(size.width), Some(size.height))
    .accessibility_label(view.title.clone())
}

/// The picture-in-picture: a stream, or the account's own share, floating
/// over the window while the call's stage is not on screen (Discord's:
/// 320 across, 16:9).
#[derive(Clone, Debug, PartialEq)]
pub struct PipView {
    /// The surface the host presents its frames on.
    pub surface: String,
    /// «Pantalla de Ana», «Tu pantalla»: its accessible name.
    pub title: String,
    /// The voice channel's name, top left; a click on it returns to the call.
    pub channel: String,
    /// The size of the frame on the surface, once the first one is there.
    pub frame: Option<(u32, u32)>,
    /// What shows until then.
    pub waiting: String,
    /// «Simulada» over the picture, when the provider simulates it.
    pub note: Option<String>,
    /// Its controls at the bottom right, then «Desconectar» last.
    pub buttons: Vec<PipButton>,
    pub corner: PipCorner,
    /// How far a drag in progress has carried it from its corner.
    pub offset: limn::Point,
}

/// One of the picture-in-picture's round buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct PipButton {
    pub id: String,
    pub hint: &'static str,
    pub glyph: PipGlyph,
}

/// The glyph of a picture-in-picture button, the same as its stage's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PipGlyph {
    PopOut,
    StopShare,
    HangUp,
}

/// The corner the picture-in-picture sits in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PipCorner {
    pub bottom: bool,
    pub right: bool,
}

/// The picture-in-picture's box, 16:9 at 320 across, and its gap to the
/// window's edges; from the top, the conversation's header comes first.
pub const PIP_WIDTH: f32 = 320.0;
pub const PIP_HEIGHT: f32 = 180.0;
pub const PIP_MARGIN: f32 = 16.0;
const PIP_RADIUS: f32 = 8.0;

/// The picture-in-picture over the window, in its corner. Its channel's name
/// is a pill at the top left, «EN VIVO» at the top right, and its buttons at
/// the bottom right; a click on the picture returns to the call.
pub fn pip(view: &PipView, look: &Look) -> Element {
    let picture = match view.frame {
        Some((width, height)) => {
            let scale = (PIP_WIDTH / width as f32).min(PIP_HEIGHT / height as f32);
            centered(
                Element::surface(view.surface.clone(), view.title.clone())
                    .size(Some(width as f32 * scale), Some(height as f32 * scale)),
            )
        }
        None => centered(line(&view.waiting, 12.0, look.muted)),
    };
    let name = Element::panel(Element::row(vec![
        gap(10.0, 0.0),
        strong_line(&view.channel, 14.0, OVER_INK)
            .centered_y()
            .shrink_to_fit(),
        gap(12.0, 0.0),
    ]))
    .size(None, Some(OVER_BUTTON))
    .radius(OVER_BUTTON / 2.0)
    .background(OVER)
    .shrink_to_fit();
    let mut bottom = Vec::new();
    if let Some(note) = &view.note {
        bottom.push(caption(note));
    }
    bottom.push(Element::spacer().flex(1.0));
    for button in &view.buttons {
        let (glyph, fill) = match button.glyph {
            PipGlyph::PopOut => (Icon::Custom(POP_OUT), OVER),
            PipGlyph::StopShare => (Icon::Custom(SCREEN_STOP), OVER),
            PipGlyph::HangUp => (Icon::HangUp, look.danger),
        };
        bottom.push(over_button(
            button.id.clone(),
            glyph,
            button.hint,
            fill,
            look,
        ));
        bottom.push(gap(8.0, 0.0));
    }
    let over = Element::column(vec![
        Element::row(vec![
            name,
            Element::spacer().flex(1.0),
            live(22.0, 10.0, look),
        ]),
        Element::spacer().flex(1.0),
        Element::row(bottom),
    ])
    .padding(8.0);
    let shown = Element::stack(vec![
        Element::panel(picture)
            .size(Some(PIP_WIDTH), Some(PIP_HEIGHT))
            .radius(PIP_RADIUS)
            .background(STREAM_GROUND)
            .clip_children(),
        over,
    ])
    .size(Some(PIP_WIDTH), Some(PIP_HEIGHT));
    let tile = Element::clickable(ids::PIP, shown)
        .radius(PIP_RADIUS)
        .hover_region()
        .draggable_in_place()
        .shadow(look.shadow, limn::Point::new(0.0, 8.0), 24.0, 0.0)
        .translate(view.offset)
        .accessibility_label(view.title.clone());
    // Its corner: the host puts it where the pointer let go, nearest corner.
    let (above, below) = if view.corner.bottom {
        (
            Element::spacer().flex(1.0),
            Element::spacer().size(None, Some(PIP_MARGIN)),
        )
    } else {
        (
            Element::spacer().size(None, Some(PIP_MARGIN + crate::sidebar::HEADER)),
            Element::spacer().flex(1.0),
        )
    };
    let (beside_left, beside_right) = if view.corner.right {
        (
            Element::spacer().flex(1.0),
            Element::spacer().size(Some(PIP_MARGIN), None),
        )
    } else {
        (
            Element::spacer().size(Some(PIP_MARGIN), None),
            Element::spacer().flex(1.0),
        )
    };
    Element::column(vec![
        above,
        Element::row(vec![beside_left, tile, beside_right]),
        below,
    ])
}

/// The voice section of the account's box, one card with it: 100 high, its
/// two rows of 32 around 12 in and 12 apart (Discord's card).
pub const PANEL_HEIGHT: f32 = 104.0;
const PANEL_INSET: f32 = 14.0;
/// Between the panel's two rows, as Discord's.
const PANEL_ROWS_GAP: f32 = 12.0;
/// The signal and the panel's buttons are 32 square, 8 apart.
const SIGNAL_SIZE: f32 = 32.0;
const PANEL_BUTTON: f32 = 32.0;
const PANEL_GAP: f32 = 10.0;
/// The connection's details: 319 wide.
pub const DETAILS_WIDTH: f32 = 319.0;
/// A person's row under a voice channel: 32 high (4, 24 of picture, 4, as
/// Discord's), its picture 24 across, centered in a ring box 30 across that
/// keeps the same size while they speak. The picture starts under the
/// channel's name (`sidebar::INSET` + 43). Its highlight is 1 short above and
/// below, so two rows' never touch, and ends where a channel's does
/// (`sidebar::INSET` from the column's edge), rounded the same 6.
pub const ROW_HEIGHT: f32 = 32.0;
const ROW_INDENT: f32 = 38.0;
const ROW_AVATAR: f32 = 24.0;
const ROW_RING: f32 = 30.0;
const ROW_RADIUS: f32 = 6.0;
/// The voice card: 280 wide with 16 of padding, so its content is 248 wide.
pub const LIVE_CARD_WIDTH: f32 = 280.0;
/// The width of its content, and of the buttons and pictures in it.
const LIVE_CONTENT: f32 = 248.0;
/// Discord's 17 px between a row and its card: transparent, so the pointer crosses.
const LIVE_GAP: f32 = 17.0;
/// A stream's picture: 16:9 of the content's width.
const LIVE_PICTURE: (f32, f32) = (248.0, 139.5);
/// Tiles keep 16:9, 8 apart as Discord's, rounded 8. The tiles' area is 8 in
/// from the sides and 16 from the top, where Discord's overlays its header.
const TILE_GAP: f32 = 8.0;
const TILE_RADIUS: f32 = 8.0;
const AREA_SIDE: f32 = 8.0;
const AREA_TOP: f32 = 16.0;
/// Up to this many tiles all show; past it the rows scroll, with this many
/// (partly) in view at most.
const GRID_VISIBLE: usize = 25;
/// The strip under a stream focused: its tiles 195×112 as Discord's, at most
/// `STRIP_MAX` of them.
const STRIP: f32 = 112.0;
const STRIP_WIDTH: f32 = 195.0;
const STRIP_MAX: usize = 12;
/// The least a stream focused keeps; a shorter stage has no strip.
const MIN_FOCUS: f32 = 180.0;
/// The bar of the stage, as Discord's: groups of 40 px buttons 8 apart,
/// 4 inside a 1 px edge, rounded 12, so 50 high; groups and hanging up (66 wide)
/// 12 apart, and 12 under the tiles; 20 over the bottom.
pub const BAR_HEIGHT: f32 = 50.0;
const BAR_BOTTOM: f32 = 20.0;
const BAR_BUTTON: f32 = 40.0;
const BAR_GAP: f32 = 12.0;
const HANG_UP_WIDTH: f32 = 66.0;
/// A private call's bar overlaps its tiles' bottom by this much (Discord's).
const BAR_OVERLAP: f32 = 6.0;
/// The bar's colors, Discord's whatever the theme: its groups' fill and
/// edge; a plain button's ink and hover; the red glow of a muted
/// microphone; the green one of a camera or a screen on; the trays' ink.
const GROUP_FILL: Color = Color::rgb(0x0a, 0x0a, 0x0c);
const GROUP_EDGE: Color = Color::rgba(0x95, 0x95, 0xa2, 41);
const PLAIN_INK: Color = Color::rgb(255, 255, 255);
const PLAIN_HOVER: Color = Color::rgba(0x99, 0x99, 0x99, 20);
const RED_GLOW: (Color, Color, Color) = (
    Color::rgba(0xd6, 0x29, 0x3a, 31),
    Color::rgba(0xd3, 0x2c, 0x38, 41),
    Color::rgb(0xda, 0x3e, 0x44),
);
const GREEN_GLOW: (Color, Color, Color) = (
    Color::rgba(0x00, 0x84, 0x42, 31),
    Color::rgba(0x00, 0x86, 0x47, 61),
    Color::rgb(0x73, 0xc4, 0x8b),
);
const TRAY_INK: Color = Color::rgb(0x96, 0x97, 0x9e);
/// Hanging up: its fill, under the pointer, and pressed.
const HANG_UP: [Color; 3] = [
    Color::rgb(0xd2, 0x2d, 0x39),
    Color::rgb(0xa9, 0x23, 0x2e),
    Color::rgb(0x97, 0x1d, 0x28),
];
/// Hover fades on the bar, as Discord's 150 ms.
const BAR_FADE_MS: u16 = 150;
/// «Más opciones», 330 wide.
const MORE_WIDTH: f32 = 330.0;

/// "03:27", or "1:02:09" past the hour.
pub fn clock(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    let (hours, minutes, seconds) = (seconds / 3_600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// "0,4 %": packets lost, per thousand, as a person reads it.
pub fn loss(permille: u16) -> String {
    tr!(
        "{whole},{tenth} %",
        whole = permille / 10,
        tenth = permille % 10
    )
}

/// What the call's title says in `phase`, and its color.
fn phase_title(phase: &CallPhase, simulated: bool, look: &Look) -> (String, Color) {
    match phase {
        CallPhase::Joining => (t("Conectando a la voz…").into(), look.idle),
        CallPhase::Securing => (t("Asegurando el audio…").into(), look.idle),
        CallPhase::Connected if simulated => (t("Voz conectada (simulada)").into(), look.online),
        CallPhase::Connected => (t("Voz conectada").into(), look.online),
        CallPhase::Waiting if simulated => {
            (t("Esperando a los demás (simulada)").into(), look.online)
        }
        CallPhase::Waiting => (t("Esperando a los demás").into(), look.online),
        CallPhase::Reconnecting { .. } => (t("Reconectando la voz…").into(), look.idle),
        CallPhase::Failed { .. } => (t("No se pudo conectar la voz").into(), look.danger),
    }
}

/// A palette whose raised fill is `fill`, for a button drawn on it.
fn filled(fill: Color, hover: Color, look: &Look) -> ComponentPalette {
    ComponentPalette {
        raised: fill,
        hover,
        selected: hover,
        ..look.palette()
    }
}

/// `color` a little lighter, for the hover of a filled button.
fn lighter(color: Color, by: u8) -> Color {
    Color::rgba(
        color.r.saturating_add(by),
        color.g.saturating_add(by),
        color.b.saturating_add(by),
        color.a.max(by),
    )
}

/// The hint of a control that cannot be used: what it would do, and why not.
pub(crate) fn denied_hint(title: &str, reason: &str) -> TooltipSpec {
    TooltipSpec::new(title).description(reason)
}

/// The account's call, as its panel shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    pub channel: String,
    /// The server the channel is in, after it: «General / Rustcord prueba».
    pub server: Option<String>,
    pub phase: CallPhase,
    pub quality: Option<CallQuality>,
    /// Joining and quality are simulated by the provider: no audio is carried.
    pub simulated: bool,
    pub video: VideoControls,
    /// The account shares its screen while the stage is not open: the
    /// subtitle says «Estás en directo».
    pub live: bool,
    /// The source the account streams, while it does: a card above the panel
    /// with it and a button to stop.
    pub stream_source: Option<String>,
    /// The stream's size and rate, as the card words them: «720p 30 FPS».
    pub stream_quality: Option<String>,
    /// The microphone's noise suppression is on.
    pub noise_suppression: bool,
    /// The noise suppression popout, open over its button.
    pub noise_open: bool,
}

/// The voice section above the account's panel, in the same card: the signal
/// (its hint the latency, a click its details, tinted as the title's color
/// while connected), how far the call got and where, the noise suppression
/// and the way to hang up; then the camera and the screen. While it failed, the reason and a way to try
/// again. The clock is in the details. `details` floats over the signal while
/// open. While the account streams, its card goes above the panel.
pub fn panel(view: &Panel, details: Option<Element>, look: &Look) -> Element {
    let (title, ink) = phase_title(&view.phase, view.simulated, look);
    let connected = view.phase.is_connected();
    let latency = match view.quality.filter(|_| connected) {
        Some(quality) => format!("{} ms", quality.rtt_ms),
        None => t("Sin medidas").into(),
    };
    // No quality grade reaches the panel: the title's color (green, amber,
    // red) carries it.
    let (fill, hover) = if connected {
        (Look::alpha(ink, 20), Look::alpha(ink, 36))
    } else {
        (look.raised, lighter(look.raised, 14))
    };
    let signal = IconButton::new(
        ids::DETAILS,
        Icon::Custom(SIGNAL),
        filled(fill, hover, look),
    )
    .variant(ButtonVariant::Raised)
    .sizing(ButtonSizing {
        height: SIGNAL_SIZE,
        font_size: 14.0,
        icon_size: 20.0,
        horizontal_padding: 8.0,
        inner_spacing: 6.0,
    })
    .width(SIGNAL_SIZE)
    .radius(RadiusRole::Card)
    .ink(ink)
    .hover_ink(ink)
    .tooltip(latency)
    .render();
    let signal = match details {
        Some(details) => Element::popover(
            signal,
            Element::panel(details)
                .size(Some(DETAILS_WIDTH), None)
                .background(look.menu)
                .border(look.border, 1.0)
                .radius(8.0)
                .shadow(look.shadow, limn::Point::new(0.0, 8.0), 20.0, 0.0)
                .clip_children(),
            ids::DETAILS_DISMISS,
            PopoverPlacement::TopStart,
        )
        .popover_gap(10.0),
        None => signal,
    };
    let mut place = view.channel.clone();
    if let Some(server) = &view.server {
        place = format!("{place} / {server}");
    }
    let subtitle = match &view.phase {
        CallPhase::Failed { reason } => reason.clone(),
        CallPhase::Reconnecting { attempt } => tr!("Intento {attempt} · {place}", attempt, place),
        _ if view.live => tr!("Estás en directo · {place}", place),
        _ => place,
    };
    // The title reads «Detalles de voz» under the pointer: it opens them too.
    let heading = Element::stack(vec![
        strong_line(title, 16.0, ink).conceal_on_hover("vz:details:title:phase"),
        strong_line(t("Detalles de voz"), 16.0, ink).reveal_on_hover("vz:details:title:hover"),
    ]);
    let summary = Element::clickable(
        ids::DETAILS_TITLE,
        Element::column(vec![heading, strong_line(subtitle, 12.0, look.muted)]),
    )
    .hover_region()
    .accessibility_label(t("Detalles de voz"));
    let wave = IconButton::new(
        ids::NOISE,
        Icon::Custom(crate::glyphs::NOISE),
        look.sidebar_palette(),
    )
    .variant(ButtonVariant::Ghost)
    .sizing(ButtonSizing {
        height: PANEL_BUTTON,
        font_size: 14.0,
        icon_size: 20.0,
        horizontal_padding: 9.0,
        inner_spacing: 6.0,
    })
    .radius(RadiusRole::Card)
    .ink(if view.noise_suppression {
        look.text
    } else {
        look.muted
    })
    .hover_ink(look.text)
    .tooltip(t("Supresión de ruido"))
    .render()
    .centered_y();
    let wave = if view.noise_open {
        Element::popover(
            wave,
            noise_popout(view.noise_suppression, look),
            ids::NOISE_DISMISS,
            PopoverPlacement::TopEnd,
        )
        .popover_gap(10.0)
    } else {
        wave
    };
    let hang_up = IconButton::new(ids::LEAVE, Icon::HangUp, look.sidebar_palette())
        .variant(ButtonVariant::Ghost)
        .sizing(ButtonSizing {
            height: PANEL_BUTTON,
            font_size: 14.0,
            icon_size: 20.0,
            horizontal_padding: 9.0,
            inner_spacing: 6.0,
        })
        .radius(RadiusRole::Card)
        .ink(look.muted)
        .hover_ink(look.text)
        .tooltip(t("Desconectar de voz"))
        .render()
        .centered_y();
    // The wave and the hang-up 4 apart, as Discord's.
    let first = vec![
        signal.centered_y(),
        summary.flex(1.0).centered_y(),
        Element::row(vec![wave, hang_up]).gap(4.0).centered_y(),
    ];
    let second = if let CallPhase::Failed { .. } = view.phase {
        Element::row(vec![Button::new(
            ids::RETRY,
            t("Reintentar"),
            look.sidebar_palette(),
        )
        .variant(ButtonVariant::Secondary)
        .sizing(ButtonSizing {
            height: PANEL_BUTTON,
            font_size: 14.0,
            icon_size: 20.0,
            horizontal_padding: 8.0,
            inner_spacing: 6.0,
        })
        .radius(RadiusRole::Card)
        .render()
        .flex(1.0)])
    } else {
        let actions = video_buttons(
            &view.video,
            PANEL_BUTTON,
            (look.hover, look.selected),
            (Look::alpha(look.online, 31), Look::alpha(look.online, 61)),
            Some(Look::alpha(look.online, 31)),
            look,
        )
        .into_iter()
        .map(|button| button.flex(1.0))
        .collect();
        Element::row(actions).gap(PANEL_GAP)
    };
    let rows = vec![
        Element::row(first).gap(8.0).size(None, Some(PANEL_BUTTON)),
        second.size(None, Some(PANEL_BUTTON)),
    ];
    let card = Element::panel(
        Element::column(rows)
            .gap(PANEL_ROWS_GAP)
            .padding(PANEL_INSET),
    )
    .size(None, Some(PANEL_HEIGHT));
    match &view.stream_source {
        Some(source) => Element::column(vec![
            stream_card(source, view.stream_quality.as_deref(), look),
            Element::panel(Element::spacer())
                .size(None, Some(1.0))
                .background(look.border),
            card,
        ])
        .size(None, Some(STREAM_CARD + 1.0 + PANEL_HEIGHT)),
        None => card,
    }
}

/// The stream card above the panel, 56 high: its tile, the source's name and
/// quality, and the button to stop.
const STREAM_CARD: f32 = PANEL_BUTTON + 2.0 * PANEL_INSET;

/// The account's stream, as Discord's card above the voice panel: a green
/// tile with the screen's glyph, the source's name over its quality, and a
/// button to stop sharing.
fn stream_card(source: &str, quality: Option<&str>, look: &Look) -> Element {
    let tile = Element::panel(centered(
        icon(Icon::Screen, Color::rgb(255, 255, 255)).size(Some(20.0), Some(20.0)),
    ))
    .size(Some(PANEL_BUTTON), Some(PANEL_BUTTON))
    .radius(8.0)
    .background(look.online);
    let mut names = vec![strong_line(source, 14.0, look.text).shrink_to_fit()];
    if let Some(quality) = quality {
        names.push(strong_line(quality, 11.0, look.muted));
    }
    let stop = IconButton::new(
        ids::STREAM_STOP,
        Icon::Custom(SCREEN_STOP),
        look.sidebar_palette(),
    )
    .variant(ButtonVariant::Ghost)
    .sizing(ButtonSizing {
        height: PANEL_BUTTON,
        font_size: 14.0,
        icon_size: 20.0,
        horizontal_padding: 9.0,
        inner_spacing: 6.0,
    })
    .radius(RadiusRole::Card)
    .ink(look.muted)
    .hover_ink(look.text)
    .tooltip(t("Dejar de transmitir"))
    .render()
    .centered_y();
    Element::panel(
        Element::row(vec![
            tile,
            Element::column(names).flex(1.0).centered_y(),
            stop,
        ])
        .gap(PANEL_GAP)
        .padding(PANEL_INSET),
    )
    .size(None, Some(STREAM_CARD))
    .background(look.raised)
    .corner_radii(limn::CornerRadii {
        top_left: 8.0,
        top_right: 8.0,
        bottom_left: 0.0,
        bottom_right: 0.0,
    })
}

/// The noise suppression popout over its button: its switch, what it does,
/// and a way to the voice settings.
fn noise_popout(on: bool, look: &Look) -> Element {
    let link = Element::clickable(
        ids::NOISE_SETTINGS,
        line(t("Ajustes de voz"), 12.0, look.accent),
    )
    .hover_region();
    Element::panel(
        Element::column(vec![
            Element::row(vec![
                strong_line(t("Supresión de ruido"), 14.0, look.text).flex(1.0),
                Switch::new(ids::NOISE_SWITCH, on, look.palette()).render(),
            ])
            .gap(8.0)
            .centered_y(),
            text(
                t("Filtra el ruido de fondo de tu micrófono."),
                12.0,
                look.muted,
            ),
            link,
        ])
        .gap(8.0)
        .padding(16.0),
    )
    .size(Some(244.0), None)
    .background(look.menu)
    .border(look.border, 1.0)
    .radius(4.0)
    .shadow(look.shadow, limn::Point::new(0.0, 8.0), 20.0, 0.0)
    .clip_children()
}

/// The connection's details, as its popover shows them.
#[derive(Clone, Debug, PartialEq)]
pub struct Details {
    /// The round trips measured in this call, oldest first, each with when
    /// it was taken, as a person reads the time.
    pub samples: Vec<(String, u32)>,
    /// Packets lost, per thousand, at the last measure.
    pub loss_permille: Option<u16>,
    /// How long the connected call has run, shown as «Duración».
    pub duration: Option<std::time::Duration>,
    pub simulated: bool,
}

/// The tabs of the voice details beside «Conexión».
pub mod details_tabs {
    pub const CONNECTION: &str = "vz:details:tab=connection";
    pub const PRIVACY: &str = "vz:details:tab=privacy";
}

/// The popover of the connection: its latency over the call, its average
/// and last, and the packets lost. Values that are made up say so. With
/// `privacy`, a «Privacidad» tab beside «Conexión» shows instead of
/// the connection's while `open`.
pub fn details_tabbed(view: &Details, privacy: Option<(Element, bool)>, look: &Look) -> Element {
    let tab = |id: &str, label: &str, width: f32, chosen: bool| {
        let ink = if chosen { look.accent } else { look.muted };
        let underline = Element::panel(Element::spacer())
            .size(Some(width), Some(2.0))
            .radius(1.0)
            .background(if chosen {
                look.accent
            } else {
                Color::rgba(0, 0, 0, 0)
            });
        let column = Element::column(vec![strong_line(label, 14.0, ink), underline]).gap(8.0);
        match privacy {
            Some(_) => Element::clickable(id.to_owned(), column).hover_foreground(look.text),
            None => column,
        }
    };
    let showing_privacy = privacy.as_ref().is_some_and(|(_, open)| *open);
    let mut tabs = vec![tab(
        details_tabs::CONNECTION,
        t("Conexión"),
        62.0,
        !showing_privacy,
    )];
    if privacy.is_some() {
        tabs.push(tab(
            details_tabs::PRIVACY,
            t("Privacidad"),
            70.0,
            showing_privacy,
        ));
    }
    let mut body = vec![
        Element::row(tabs).gap(16.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
    ];
    if let Some((privacy, true)) = privacy {
        body.push(privacy);
        return Element::column(body).gap(12.0).padding(16.0);
    }
    let rtts: Vec<u32> = view.samples.iter().map(|(_, rtt)| *rtt).collect();
    if rtts.len() >= 2 {
        body.push(
            LineChart::new(
                "vz:details:chart",
                view.samples.iter().map(|(when, _)| when.clone()),
                vec![ChartSeries::new(
                    t("Latencia (ms)"),
                    rtts.iter().map(|rtt| f64::from(*rtt)),
                )],
                look.menu_palette(),
            )
            .height(90.0)
            .color(look.accent)
            .smooth(false)
            .render(),
        );
    }
    let stat = |label: &str, value: String| {
        Element::row(vec![
            line(format!("{label}:"), 13.0, look.muted).shrink_to_fit(),
            strong_line(value, 13.0, look.text).shrink_to_fit(),
        ])
        .gap(4.0)
    };
    if let Some(duration) = view.duration {
        body.push(
            Element::row(vec![
                line(format!("{}:", t("Duración")), 13.0, look.muted).shrink_to_fit(),
                Element::text(clock(duration))
                    .font_family(FontFamily::Monospace)
                    .font_size(13.0)
                    .foreground(look.text)
                    .shrink_to_fit(),
            ])
            .gap(4.0),
        );
    }
    match (rtts.last(), rtts.is_empty()) {
        (Some(last), false) => {
            let average = rtts.iter().map(|rtt| f64::from(*rtt)).sum::<f64>() / rtts.len() as f64;
            body.push(
                Element::column(vec![
                    stat(
                        t("Latencia promedio"),
                        format!("{} ms", average.round() as u32),
                    ),
                    stat(t("Última latencia"), format!("{last} ms")),
                    stat(
                        t("Pérdida de paquetes"),
                        view.loss_permille.map_or_else(|| "—".into(), loss),
                    ),
                ])
                .gap(4.0),
            );
        }
        _ => body.push(text(
            t("Todavía no hay medidas de esta conexión."),
            13.0,
            look.muted,
        )),
    }
    body.push(text(
        t(
            "Una latencia alta retrasa la voz y una pérdida alta la entrecorta. Si \
         sigue así, desconéctate y vuelve a entrar.",
        ),
        12.0,
        look.faint,
    ));
    let note = if view.simulated {
        t("Valores simulados: esta llamada no transmite audio.")
    } else {
        t("Medido por el proveedor.")
    };
    body.push(
        Element::panel(
            Element::row(vec![
                icon(Icon::InfoCircle, look.muted).centered_y(),
                text(note, 12.0, look.muted).flex(1.0).centered_y(),
            ])
            .gap(6.0)
            .padding(8.0),
        )
        .radius(6.0)
        .background(look.raised),
    );
    Element::column(body).gap(12.0).padding(16.0)
}

/// Someone in a voice channel.
#[derive(Clone, Debug, PartialEq)]
pub struct Participant {
    pub user: u64,
    pub name: String,
    pub accent: Color,
    pub muted: bool,
    pub deafened: bool,
    pub speaking: bool,
    /// The server keeps their microphone off, whatever they chose.
    pub server_muted: bool,
    /// The server keeps their sound off.
    pub server_deafened: bool,
    /// The account silenced them for itself.
    pub silenced: bool,
    /// Sharing their screen in the channel.
    pub streaming: bool,
    /// The account itself: live like anyone, but nothing to watch.
    pub own: bool,
    /// The size of their camera's frame on their surface, while it shows:
    /// their tile shows it instead of their mark.
    pub camera: Option<(u32, u32)>,
    /// Their camera is on in the channel: a mark in their row, shown or not.
    pub camera_on: bool,
    /// The latest picture of their stream, on their tile.
    pub thumbnail: Option<Thumbnail>,
    /// Their picture where they are seen, once loaded; their initial until then.
    pub image: Option<limn::RasterImage>,
    /// The frame over their face on their tile, once loaded, as the
    /// official client's call tiles draw it.
    pub decoration: Option<limn::RasterImage>,
    /// Their row can be dragged to another voice channel: the account may
    /// move them.
    pub movable: bool,
    /// They are being carried to another voice channel: their row is lifted
    /// on a card's surface, which the copy following the pointer keeps.
    pub carried: bool,
    /// The game they play, with its picture: the mark at the end of their row.
    pub game: Option<GameMark>,
}

/// The latest picture of someone's stream, as the column shows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Thumbnail {
    Image(limn::RasterImage),
    /// The simulation's: no picture leaves a simulated stream.
    Simulated,
    /// The account's own stream: the host's still of it, on its surface.
    Own,
}

/// The stream of someone live, as the first section of their card shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveStream {
    /// What they stream: the game they play, when known.
    pub activity: Option<String>,
    /// The latest picture of their stream; what is said instead until one
    /// arrives, or when none can.
    pub picture: Result<Thumbnail, String>,
    /// Whether «Mirar transmisión» can be used, or why not.
    pub watch: Result<(), String>,
    /// The account already watches it.
    pub watching: bool,
    /// The account's own stream: its header, its picture, and a button that
    /// says it is live, in place of watching (`watch`, `watching` unused).
    pub own: bool,
}

/// The card of someone beside their row under a voice channel, while the
/// pointer is on it: their stream while they are live, then what they do.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveCard {
    pub user: u64,
    pub name: String,
    /// Their stream, while they are live. Never the account's own: there is
    /// nothing of it to watch.
    pub stream: Option<LiveStream>,
    /// One section per activity they list, in order.
    pub activities: Vec<ActivityCard>,
}

/// The game someone plays and its picture, drawn at the end of their row.
#[derive(Clone, Debug, PartialEq)]
pub struct GameMark {
    pub name: String,
    pub image: limn::RasterImage,
}

/// The card of someone beside their row, as Discord's voice popout: their
/// stream while they are live, then a section for each activity they list.
/// Sections after the first have a line above them. Transparent 17 on its
/// left, toward the row, so the pointer crosses into it.
pub fn live_card(card: &LiveCard, look: &Look) -> Element {
    let mut sections = Vec::new();
    if let Some(stream) = &card.stream {
        sections.push(stream_section(card.user, stream, look));
    }
    sections.extend(
        card.activities
            .iter()
            .map(|activity| activity_section(activity, LIVE_CONTENT, look)),
    );
    let sections = sections
        .into_iter()
        .enumerate()
        .map(|(index, section)| {
            if index == 0 {
                section
            } else {
                Element::column(vec![rule(look), gap(0.0, 12.0), section])
            }
        })
        .collect();
    let surface = Element::panel(Element::column(sections).gap(12.0).padding(16.0))
        .size(Some(LIVE_CARD_WIDTH), None)
        .background(look.menu)
        .border(look.border, 1.0)
        .radius(12.0)
        .shadow(look.shadow, limn::Point::new(0.0, 9.0), 22.0, 0.0)
        .accessibility_label(card.name.clone());
    Element::row(vec![gap(LIVE_GAP, 0.0), surface]).size(Some(LIVE_CARD_WIDTH + LIVE_GAP), None)
}

/// The line above a section after the first.
fn rule(look: &Look) -> Element {
    Element::panel(Element::spacer())
        .size(None, Some(1.0))
        .background(look.border)
}

/// The stream of someone live: its title, its picture, and «Mirar transmisión».
fn stream_section(user: u64, stream: &LiveStream, look: &Look) -> Element {
    if stream.own {
        return own_stream_section(user, stream, look);
    }
    let (width, height) = LIVE_PICTURE;
    let title = match &stream.activity {
        Some(game) => tr!("Transmitiendo {name}", name = game),
        None => t("Transmitiendo").to_owned(),
    };
    let picture = match &stream.picture {
        Ok(Thumbnail::Image(image)) => {
            Element::raster_image(image.clone().cover()).size(Some(width), Some(height))
        }
        Ok(Thumbnail::Simulated) => centered(text(t("Vista previa simulada"), 13.0, look.muted)),
        Ok(Thumbnail::Own) => Element::surface(ids::OWN_THUMB_SURFACE, title.clone()),
        Err(note) => centered(text(note.clone(), 13.0, look.muted)),
    };
    let picture = Element::panel(picture)
        .size(Some(width), Some(height))
        .radius(8.0)
        .background(look.canvas)
        .clip_children();
    // Discord's green button, or once watched, a disabled grey one that says so.
    let button = if stream.watching {
        Button::new(
            ids::live_watch(user),
            t("Viendo una transmisión"),
            veil_palette(look),
        )
        .variant(ButtonVariant::Raised)
        .button_size(ButtonSize::Small)
        .width(width)
        .disabled()
        .render()
    } else {
        let button = Button::new(
            ids::live_watch(user),
            t("Mirar transmisión"),
            look.palette(),
        )
        .variant(ButtonVariant::Success)
        .button_size(ButtonSize::Small)
        .icon(Icon::Screen)
        .width(width);
        match &stream.watch {
            Ok(()) => button.render(),
            Err(reason) => button
                .tooltip_with(denied_hint(t("Mirar transmisión"), reason))
                .disabled()
                .render(),
        }
    };
    Element::column(vec![
        strong_line(title, 12.0, look.muted),
        Element::column(vec![picture, button]).gap(8.0),
    ])
    .gap(8.0)
}

/// The account's own stream, as Discord's own card: «Transmitiendo ahora»
/// with «EN VIVO», its picture or the words about it, and a grey button that
/// says it is live.
fn own_stream_section(user: u64, stream: &LiveStream, look: &Look) -> Element {
    let (width, height) = LIVE_PICTURE;
    let picture = match &stream.picture {
        Ok(Thumbnail::Own) => {
            Element::surface(ids::OWN_THUMB_SURFACE, t("Tu transmisión").to_owned())
        }
        Ok(_) => centered(text(
            t("La transmisión acaba de empezar. ¡Únete!"),
            13.0,
            look.muted,
        )),
        Err(note) => centered(text(note.clone(), 13.0, look.muted)),
    };
    let picture = Element::panel(picture)
        .size(Some(width), Some(height))
        .radius(8.0)
        .background(look.canvas)
        .clip_children();
    let button = Button::new(
        ids::live_own(user),
        t("¡Estás transmitiendo en vivo!"),
        veil_palette(look),
    )
    .variant(ButtonVariant::Raised)
    .button_size(ButtonSize::Small)
    .icon(Icon::Screen)
    .width(width)
    .disabled()
    .render();
    Element::column(vec![
        Element::row(vec![
            strong_line(t("Transmitiendo ahora"), 12.0, look.muted),
            Element::spacer().flex(1.0),
            live(18.0, 9.0, look),
        ]),
        Element::column(vec![picture, button]).gap(8.0),
    ])
    .gap(8.0)
}

/// The picture of the game someone plays, at the end of their row, with its
/// name in a tooltip.
fn game_mark(id: &str, game: &GameMark, look: &Look) -> Element {
    let picture = Element::raster_image(game.image.clone().cover())
        .size(Some(16.0), Some(16.0))
        .radius(4.0);
    Tooltip::new(
        picture,
        format!("{id}:game"),
        tr!("Jugando {game}", game = game.name.as_str()),
        look.menu_palette(),
    )
    .render()
}

/// The «En vivo» mark of someone streaming, `size` high, its text `font` high.
fn live(size: f32, font: f32, look: &Look) -> Element {
    Element::panel(Element::row(vec![
        gap(6.0, 0.0),
        Element::text(t("EN VIVO"))
            .font_family(FontFamily::SansBold)
            .font_size(font)
            .foreground(Color::rgb(255, 255, 255))
            .centered_y(),
        gap(6.0, 0.0),
    ]))
    .size(None, Some(size))
    .radius(size / 2.0)
    .background(look.danger)
    .shrink_to_fit()
}

/// A row's picture centered in a ring box, ringed in green by 1 px while its
/// person speaks: the box is the same size either way, so speaking moves nothing.
fn row_ring(face: Element, speaking: bool, look: &Look) -> Element {
    let ring = if speaking {
        look.online
    } else {
        Color::rgba(0, 0, 0, 0)
    };
    Element::stack(vec![
        centered(face),
        Element::panel(Element::spacer())
            .size(Some(ROW_RING), Some(ROW_RING))
            .radius(ROW_RING / 2.0)
            .border(ring, 1.0),
    ])
    .size(Some(ROW_RING), Some(ROW_RING))
}

/// The marks of a microphone off and of sound off, each with its hint.
fn marks(person: &Participant, prefix: &str, ink: Color, look: &Look) -> Vec<Element> {
    let mut marks = Vec::new();
    let mut mark = |glyph: Icon, suffix: &str, hint: &str, ink: Color| {
        let id = format!("{prefix}:{suffix}");
        marks.push(
            Tooltip::new(
                icon(glyph, ink).size(Some(16.0), Some(16.0)),
                id,
                hint,
                look.menu_palette(),
            )
            .render(),
        );
    };
    // The camera first, as the official client's row: then the mute and the sound.
    if person.camera_on {
        mark(Icon::Video, "camera", t("Cámara encendida"), ink);
    }
    // The server's marks are red: their own unmute does not lift them.
    if person.server_muted {
        mark(
            Icon::MicrophoneMuted,
            "muted",
            t("Silenciado por el servidor"),
            look.danger,
        );
    } else if person.muted {
        mark(Icon::MicrophoneMuted, "muted", t("Silenciado"), ink);
    }
    if person.server_deafened {
        mark(
            Icon::Custom(crate::HEADPHONES_OFF),
            "deafened",
            t("Ensordecido por el servidor"),
            look.danger,
        );
    } else if person.deafened {
        mark(
            Icon::Custom(crate::HEADPHONES_OFF),
            "deafened",
            t("Ensordecido"),
            ink,
        );
    }
    if person.silenced {
        mark(Icon::VolumeMuted, "silenced", t("Silenciado para ti"), ink);
    }
    marks
}

/// Someone under the voice channel they are in: their mark (ringed while
/// they speak), their name, and whether they hear and can be heard. A click
/// opens their card (`card`, floating beside the row while open); a
/// secondary click, their menu. Someone live shows `streamed` beside the
/// row while the pointer is on it.
pub fn participant(
    person: &Participant,
    card: Option<Element>,
    streamed: Option<&LiveCard>,
    look: &Look,
) -> Element {
    let id = ids::row(person.user);
    let face = row_ring(
        crate::avatar_with_image(
            &person.name,
            person.accent,
            ROW_AVATAR,
            person.image.clone(),
        ),
        person.speaking,
        look,
    );
    let name_ink = if person.speaking {
        look.text
    } else {
        look.muted
    };
    // The row's 1 px and the ring's 3 px put the picture under the channel's
    // name: `sidebar::INSET` + 43 from the column's edge.
    let mut parts = vec![
        gap(10.5, 0.0),
        face.centered_y(),
        gap(8.0, 0.0),
        line(&person.name, 14.0, name_ink).flex(1.0).centered_y(),
    ];
    if person.streaming {
        parts.push(gap(4.0, 0.0));
        parts.push(live(18.0, 12.0, look).centered_y());
    }
    if let Some(game) = &person.game {
        parts.push(gap(4.0, 0.0));
        parts.push(game_mark(&id, game, look).centered_y());
    }
    for mark in marks(person, &id, look.muted, look) {
        parts.push(gap(4.0, 0.0));
        parts.push(mark.centered_y());
    }
    // Ends 4 in from the highlight's edge, as the marks did.
    parts.push(gap(3.0, 0.0));
    let mut state = Vec::new();
    if person.streaming {
        state.push(t("en vivo"));
    }
    if person.camera_on {
        state.push(t("cámara encendida"));
    }
    if person.speaking {
        state.push(t("hablando"));
    }
    if person.server_muted {
        state.push(t("silenciado por el servidor"));
    } else if person.muted {
        state.push(t("silenciado"));
    }
    if person.server_deafened {
        state.push(t("ensordecido por el servidor"));
    } else if person.deafened {
        state.push(t("ensordecido"));
    }
    if person.silenced {
        state.push(t("silenciado para ti"));
    }
    let label = if state.is_empty() {
        person.name.clone()
    } else {
        format!("{}, {}", person.name, state.join(", "))
    };
    // The highlight is drawn inside the row, short of its edges; the row
    // itself reaches the column's edge, so the pointer goes from it to the
    // card of someone live without leaving either.
    let fill = Element::panel(Element::spacer()).radius(ROW_RADIUS);
    let fill = if person.carried {
        fill.background(look.menu).border(look.border, 1.0)
    } else {
        fill.background(look.hover)
            .reveal_on_hover(format!("{id}:hover"))
    };
    let highlight = Element::column(vec![gap(0.0, 1.0), fill.flex(1.0), gap(0.0, 1.0)]);
    let body = Element::stack(vec![highlight, Element::row(parts).padding(1.0)])
        .size(None, Some(ROW_HEIGHT));
    let row = Element::clickable(
        id,
        Element::row(vec![body.flex(1.0), gap(crate::sidebar::INSET, 0.0)]),
    )
    .hover_region()
    .hover_foreground(look.text)
    .accessibility_label(label);
    let row = if person.movable { row.draggable() } else { row };
    let row = match (card, streamed) {
        (Some(card), _) => crate::profile::card_popout(
            row,
            Some(card),
            ids::CARD_DISMISS,
            PopoverPlacement::RightStart,
            look,
        ),
        // Shown while the pointer is on the row or on the card, as the
        // official client's preview of a stream.
        (None, Some(live)) => Element::popover(
            row,
            live_card(live, look),
            ids::live_dismiss(person.user),
            PopoverPlacement::RightStart,
        )
        .popover_on_hover(),
        (None, None) => row,
    };
    Element::row(vec![gap(ROW_INDENT, 0.0), row.flex(1.0)])
}

/// How tiles share an area: how many across, and each one's size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub columns: usize,
    pub tile: Size,
}

/// The arrangement of `count` tiles of 16:9 in `area` that makes them
/// largest, `TILE_GAP` apart, in whole pixels: the width floored, the height
/// rounded, as Discord's 520×293 and 412×232. Past `GRID_VISIBLE` the rows
/// scroll, and the tiles are as large as keeps that many partly in view
/// (Discord's rule).
pub fn grid(count: usize, area: Size) -> Grid {
    let count = count.max(1);
    let width = area.width.max(0.0);
    if count > GRID_VISIBLE {
        let columns_at = |tile: f32| ((width + TILE_GAP) / (tile + TILE_GAP)).floor().max(1.0);
        let shown = |tile: f32| {
            let high = tile * 9.0 / 16.0;
            let rows = (((area.height - high) / (high + TILE_GAP)).ceil() + 1.0).max(1.0);
            columns_at(tile) * rows
        };
        let tile = (1..=width.floor() as usize)
            .map(|tile| tile as f32)
            .find(|tile| shown(*tile) <= GRID_VISIBLE as f32)
            .unwrap_or(width.floor());
        return Grid {
            columns: columns_at(tile) as usize,
            tile: Size::new(tile, (tile * 9.0 / 16.0).round()),
        };
    }
    let mut best = (1, 0.0_f32);
    for columns in 1..=count {
        let rows = count.div_ceil(columns);
        let across = (width - TILE_GAP * (columns - 1) as f32) / columns as f32;
        let down = (area.height - TILE_GAP * (rows - 1) as f32) / rows as f32;
        let tile = across.min(down * 16.0 / 9.0).max(0.0);
        if tile > best.1 {
            best = (columns, tile);
        }
    }
    let tile = best.1.floor();
    Grid {
        columns: best.0,
        tile: Size::new(tile, (tile * 9.0 / 16.0).round()),
    }
}

/// A tile's face, the person's picture on it: 80 across, or 40 on a tile
/// under 124 wide. It does not grow with the tile.
pub(crate) fn face_size(size: Size) -> f32 {
    if size.width < 124.0 {
        40.0
    } else {
        80.0
    }
}

/// How far a tile's overlays stand from its edges: 12, or 4 on a tile 220
/// wide or less.
pub(crate) fn margin(size: Size) -> f32 {
    if size.width > 220.0 {
        12.0
    } else {
        4.0
    }
}

/// What sits over a tile's picture in a pill: black at 56 %, rounded 8.
fn pill(content: Element) -> Element {
    Element::panel(content)
        .radius(8.0)
        .background(PILL)
        .shrink_to_fit()
}

/// A name in a pill at the bottom left of a tile, `marks` before it: 16 px,
/// or 14 on a tile 220 wide or less. `None` on a tile 124 wide or less, where
/// it is hidden.
fn name_tag(name: &str, marks: Vec<Element>, size: Size) -> Option<Element> {
    if size.width <= 124.0 {
        return None;
    }
    let font = if size.width > 220.0 { 16.0 } else { 14.0 };
    // Padding 6 all round; the 2 at each end make the pill 8 across.
    let mut parts = vec![gap(2.0, 0.0)];
    for mark in marks {
        parts.push(mark.centered_y());
        parts.push(gap(4.0, 0.0));
    }
    parts.push(line(name, font, OVER_INK).centered_y().shrink_to_fit());
    parts.push(gap(2.0, 0.0));
    Some(pill(Element::row(parts).padding(6.0)))
}

/// A person's name on their tile, their marks before it (see [`name_tag`]).
pub(crate) fn tile_name(person: &Participant, size: Size, look: &Look) -> Option<Element> {
    name_tag(
        &person.name,
        marks(person, &ids::tile(person.user), OVER_INK, look),
        size,
    )
}

/// A note over a tile's picture at its top left, such as «Cámara simulada»:
/// 12 px, 6 across and 2 high inside its pill.
fn caption(text: &str) -> Element {
    pill(
        Element::row(vec![
            gap(4.0, 0.0),
            line(text, 12.0, OVER_INK).centered_y(),
            gap(4.0, 0.0),
        ])
        .padding(2.0),
    )
}

/// The speaking mark: 2 px of green on the tile's edge and 1 px of black just
/// inside it, Discord's inset ring.
fn speaking_edge(size: Size, look: &Look) -> Element {
    let inside = Element::panel(Element::spacer())
        .radius(TILE_RADIUS - 2.0)
        .border(Color::rgb(0, 0, 0), 1.0)
        .flex(1.0);
    Element::panel(Element::column(vec![
        gap(0.0, 2.0),
        Element::row(vec![gap(2.0, 0.0), inside, gap(2.0, 0.0)]).flex(1.0),
        gap(0.0, 2.0),
    ]))
    .size(Some(size.width), Some(size.height))
    .radius(TILE_RADIUS)
    .border(look.online, 2.0)
}

/// Discord's secondary button: a grey veil that shows on any stream.
fn veil_palette(look: &Look) -> ComponentPalette {
    ComponentPalette {
        raised: Color::rgba(152, 152, 160, 31),
        hover: Color::rgba(152, 152, 160, 41),
        selected: Color::rgba(152, 152, 160, 51),
        text: Color::rgb(255, 255, 255),
        ..look.palette()
    }
}

/// «Mirar transmisión» in the middle of a stream's tile, «Ver» on a tile
/// under 175 wide. Disabled with its reason when it cannot be used.
/// «Mirar transmisión» («Ver» under 175 px) on a stream nobody watches; with
/// `another`, beside it an icon that adds the stream to those watched, as the
/// official client's, 8 px after it.
fn watch_button(
    person: &Participant,
    size: Size,
    permit: &Result<(), String>,
    another: bool,
    look: &Look,
) -> Element {
    let label = if size.width < 175.0 {
        t("Ver")
    } else {
        t("Mirar transmisión")
    };
    let small = size.width < 195.0;
    let button = Button::new(ids::watch(person.user), label, veil_palette(look))
        .variant(ButtonVariant::Secondary)
        .button_size(if small {
            ButtonSize::Small
        } else {
            ButtonSize::Default
        });
    let watch = match permit {
        Ok(()) => button.render(),
        Err(reason) => button
            .tooltip_with(denied_hint(label, reason))
            .disabled()
            .render(),
    };
    if !another {
        return watch;
    }
    let height = if small { 30.0 } else { 38.0 };
    let hint = t("Comienza a ver múltiples transmisiones");
    let add = IconButton::new(
        ids::watch_add(person.user),
        Icon::Custom(ADD_STREAM),
        veil_palette(look),
    )
    .variant(ButtonVariant::Secondary)
    .sizing(ButtonSizing {
        height,
        font_size: 14.0,
        icon_size: 20.0,
        horizontal_padding: 8.0,
        inner_spacing: 6.0,
    })
    .width(height)
    .round()
    .tooltip(hint)
    .accessibility_label(hint);
    let add = match permit {
        Ok(()) => add.render(),
        Err(_) => add.disabled().render(),
    };
    Element::row(vec![watch, gap(8.0, 0.0), add])
}

/// The tile of a live stream nobody watches, after its streamer's:
/// the picture when known, dimmed as a watched stream's, on Discord's
/// stream ground; «EN VIVO» top right, «Vista previa simulada» top left in the
/// simulation, the watch button in the middle, and the streamer's name in the
/// pill. No face, so nothing sits under the button. A click watches the
/// stream, as the button does, when the account may.
fn watch_tile(
    person: &Participant,
    size: Size,
    watch: &Result<(), String>,
    another: bool,
    look: &Look,
) -> Element {
    let picture = match &person.thumbnail {
        Some(Thumbnail::Image(image)) => Element::stack(vec![
            Element::raster_image(image.clone().cover()).size(Some(size.width), Some(size.height)),
            Element::panel(Element::spacer())
                .size(Some(size.width), Some(size.height))
                .background(Look::alpha(Color::rgb(0, 0, 0), 96)),
        ]),
        _ => Element::spacer().size(Some(size.width), Some(size.height)),
    };
    let ground = Element::panel(picture)
        .size(Some(size.width), Some(size.height))
        .radius(TILE_RADIUS)
        .background(STREAM_GROUND)
        .clip_children();
    let mut left = Vec::new();
    if person.thumbnail == Some(Thumbnail::Simulated) {
        left.push(caption(t("Vista previa simulada")));
    }
    let top = Element::row(vec![
        Element::row(left).gap(6.0),
        Element::spacer().flex(1.0),
        live(16.0, 12.0, look),
    ])
    .padding(8.0);
    let middle = centered(watch_button(person, size, watch, another, look)).flex(1.0);
    let mut bottom: Vec<Element> = name_tag(&person.name, Vec::new(), size)
        .into_iter()
        .map(Element::shrink_to_fit)
        .collect();
    bottom.push(Element::spacer().flex(1.0));
    let bottom = Element::row(bottom).padding(margin(size));
    let shown = Element::stack(vec![ground, Element::column(vec![top, middle, bottom])])
        .size(Some(size.width), Some(size.height));
    let shown = match watch {
        Ok(()) => Element::clickable(ids::watch_tile(person.user), shown),
        Err(_) => shown,
    };
    shown.accessibility_label(tr!("Transmisión de {name}", name = person.name))
}

/// A person on the stage: their round picture on their color, or their
/// camera letterboxed on Discord's #1E1F22 («Cámara simulada» over it when
/// `simulated`), and their name in a pill at the bottom left with their
/// marks. Under the pointer a «⋯» opens their menu (`menu`, while open).
/// Their card floats beside the name while open, or beside the tile when it
/// has no name. Outlined in green while they speak. `idle`, the name and the
/// «⋯» fade out. Their stream is a tile of its own, see [`watch_tile`].
#[allow(clippy::too_many_arguments)]
fn tile(
    person: &Participant,
    size: Size,
    room: f32,
    card: Option<Element>,
    menu: Option<Vec<MenuEntry>>,
    simulated: bool,
    idle: bool,
    look: &Look,
) -> Element {
    let face = face_size(size);
    let open = menu.is_some();
    let more = over_button(
        ids::tile_more(person.user),
        Icon::More,
        t("Más"),
        PILL,
        look,
    );
    let width = menu_width(menu.as_deref().unwrap_or_default());
    let more = DropdownMenu::new(
        more,
        menu_entries(menu.unwrap_or_default(), look),
        open,
        ids::MENU_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::menu_metrics_within(room))
    .width(width)
    .placement(PopoverPlacement::RightStart)
    .render();
    // Kept in sight while its menu is open.
    let more = if open {
        more
    } else {
        more.reveal_on_hover(format!("{}:more", ids::tile(person.user)))
    };
    let (name, card) = match tile_name(person, size, look).filter(|_| !idle) {
        Some(name) => (
            Some(crate::profile::card_popout(
                name.animate_presence(format!("{}:name", ids::tile(person.user))),
                card,
                ids::CARD_DISMISS,
                PopoverPlacement::RightStart,
                look,
            )),
            None,
        ),
        None => (None, card),
    };
    let mut bottom: Vec<Element> = name.into_iter().map(Element::shrink_to_fit).collect();
    bottom.push(Element::spacer().flex(1.0));
    if open || !idle {
        bottom.push(more);
    }
    let bottom = Element::row(bottom).padding(margin(size));
    // Simulated things say so at the top left.
    let mut left = Vec::new();
    if person.camera.is_some() && simulated {
        left.push(caption(t("Cámara simulada")));
    }
    let top = Element::row(vec![
        Element::row(left).gap(6.0),
        Element::spacer().flex(1.0),
    ])
    .padding(8.0);
    let over = Element::column(vec![top, Element::spacer().flex(1.0), bottom]);
    let picture = match person.camera {
        Some((width, height)) => {
            let scale = (size.width / width as f32).min(size.height / height as f32);
            Element::panel(centered(
                Element::surface(ids::camera_surface(person.user), person.name.clone())
                    .size(Some(width as f32 * scale), Some(height as f32 * scale)),
            ))
            .size(Some(size.width), Some(size.height))
            .radius(TILE_RADIUS)
            .background(STREAM_GROUND)
            .clip_children()
        }
        None => Element::panel(centered(crate::identity::decorated(
            |side| {
                crate::avatar_with_image(&person.name, person.accent, side, person.image.clone())
            },
            face,
            person.decoration.clone(),
        )))
        .size(Some(size.width), Some(size.height))
        .radius(TILE_RADIUS)
        .background(ground(person)),
    };
    let mut layers = vec![picture, over];
    if person.speaking {
        layers.push(speaking_edge(size, look));
    }
    let tile = Element::clickable(ids::tile(person.user), Element::stack(layers))
        .size(Some(size.width), Some(size.height))
        .radius(TILE_RADIUS)
        .hover_region()
        .accessibility_label(person.name.clone());
    crate::profile::card_popout(
        tile,
        card,
        ids::CARD_DISMISS,
        PopoverPlacement::RightStart,
        look,
    )
}

/// The ground of a tile without video: the color of the person's picture,
/// as Discord's. Their initial is drawn on their own color, so its ground
/// goes a quarter darker for the round avatar to read on it.
fn ground(person: &Participant) -> Color {
    if person.image.is_some() {
        return person.accent;
    }
    let darker = |channel: u8| (f32::from(channel) * 0.75) as u8;
    let Color { r, g, b, a } = person.accent;
    Color::rgba(darker(r), darker(g), darker(b), a)
}

/// A conversation's call under its header, its own header over its top:
/// Discord's 240 high in VOICE mode and about 360 with tiles (one is
/// 412×232), both with its 48 px header, which ours may outgrow.
pub const VOICE_HEIGHT: f32 = 192.0 + crate::sidebar::HEADER;
pub const TILES_HEIGHT: f32 = 312.0 + crate::sidebar::HEADER;
/// VOICE mode's avatars: 80 across, or 40 when the row would not fit them,
/// with 8 px of room each side, so 16 between two.
const AVATAR_LARGE: f32 = 80.0;
const AVATAR_SMALL: f32 = 40.0;
const AVATAR_ROOM: f32 = 8.0;
/// The ring of someone being rung stands 3 px outside their picture.
const RING_OUTSIDE: f32 = 3.0;

/// VOICE mode: everyone in the call and everyone it rings, as one
/// row of round avatars centred both ways, in name order and without names
/// under them. A row too wide for `width` overflows both ends and is clipped,
/// never wrapped. `people` pairs each with whether they are being rung.
pub(crate) fn avatar_row(people: &[(&Participant, bool)], width: f32, look: &Look) -> Element {
    let mut people = people.to_vec();
    people.sort_by_key(|(person, _)| person.name.to_lowercase());
    let face = if (AVATAR_LARGE + 2.0 * AVATAR_ROOM) * people.len() as f32 <= width {
        AVATAR_LARGE
    } else {
        AVATAR_SMALL
    };
    let pitch = face + 2.0 * AVATAR_ROOM;
    let height = face + 2.0 * RING_OUTSIDE;
    let total = pitch * people.len() as f32;
    let cells: Vec<Element> = people
        .iter()
        .map(|(person, ringing)| avatar_cell(person, *ringing, face, look))
        .collect();
    // Centred even when it overflows, as flex centring overflows both ends.
    let row = Element::row(cells)
        .size(Some(total), Some(height))
        .translate(limn::Point::new((width - total) / 2.0, 0.0));
    Element::panel(row)
        .size(Some(width), Some(height))
        .clip_children()
}

/// One avatar of [`avatar_row`] in its cell: the picture, with a speaker's
/// ring or a mute's mark over it; someone being rung is dimmed to 30 % and
/// ringed outside.
fn avatar_cell(person: &Participant, ringing: bool, face: f32, look: &Look) -> Element {
    let pitch = face + 2.0 * AVATAR_ROOM;
    let height = face + 2.0 * RING_OUTSIDE;
    let picture = crate::avatar_with_image(&person.name, person.accent, face, person.image.clone());
    let mut layers = Vec::new();
    // shortcut: no 2.5 s pulse; limn's animate helpers play once, so the ring stands still.
    if ringing {
        layers.push(centered(
            Element::panel(Element::spacer())
                .size(Some(height), Some(height))
                .radius(height / 2.0)
                .border(Look::alpha(look.text, 217), RING_OUTSIDE),
        ));
        // limn has no static opacity on an element (`dim_on_hover` dims only
        // under the pointer); the ground at 70 % over the picture stands in.
        let dim = Element::panel(Element::spacer())
            .size(Some(face), Some(face))
            .radius(face / 2.0)
            .background(Look::alpha(look.raised, 178));
        layers.push(centered(
            Element::stack(vec![picture, dim]).size(Some(face), Some(face)),
        ));
    } else {
        let mut over = vec![picture];
        if ringed_speaker(person) {
            over.push(voice_ring(face, look));
        }
        over.extend(mute_mark(person, face, look));
        layers.push(centered(Element::stack(over).size(Some(face), Some(face))));
    }
    let label = match state_of(person) {
        Some(state) => format!("{}, {state}", person.name),
        None => person.name.clone(),
    };
    Element::clickable(
        ids::avatar(person.user),
        Element::stack(layers).size(Some(pitch), Some(height)),
    )
    .size(Some(pitch), Some(height))
    .accessibility_label(label)
}

/// Whether someone's picture is ringed in green: they speak, and neither they
/// nor the account keeps them silent.
fn ringed_speaker(person: &Participant) -> bool {
    person.speaking
        && !(person.muted
            || person.deafened
            || person.server_muted
            || person.server_deafened
            || person.silenced)
}

/// Discord's fallback ring of a speaker: 2 px of green just inside the
/// picture's edge, and 1 px of black inside that.
fn voice_ring(face: f32, look: &Look) -> Element {
    let inside = face - 4.0;
    Element::stack(vec![
        Element::panel(Element::spacer())
            .size(Some(face), Some(face))
            .radius(face / 2.0)
            .border(look.online, 2.0),
        centered(
            Element::panel(Element::spacer())
                .size(Some(inside), Some(inside))
                .radius(inside / 2.0)
                .border(Color::rgb(0, 0, 0), 1.0),
        ),
    ])
    .size(Some(face), Some(face))
}

/// The mark of a mute, deafening or local silence, over the picture's bottom
/// right in the danger color: 24 px with a 16 px glyph on an 80 px picture,
/// 12 with 12 on a 40 px one. Local silence comes first, then deafening.
fn mute_mark(person: &Participant, face: f32, look: &Look) -> Option<Element> {
    let glyph = if person.silenced {
        Icon::VolumeMuted
    } else if person.deafened || person.server_deafened {
        Icon::Custom(crate::HEADPHONES_OFF)
    } else if person.muted || person.server_muted {
        Icon::MicrophoneMuted
    } else {
        return None;
    };
    let (badge, mark) = if face > AVATAR_SMALL {
        (24.0, 16.0)
    } else {
        (12.0, 12.0)
    };
    // shortcut: no cut-out ring around the mark; add it with the design pass.
    let disc = Element::panel(centered(icon(glyph, OVER_INK).size(Some(mark), Some(mark))))
        .size(Some(badge), Some(badge))
        .radius(badge / 2.0)
        .background(look.danger_fill);
    Some(
        Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![Element::spacer().flex(1.0), disc]),
        ])
        .size(Some(face), Some(face)),
    )
}

/// What the account sees of someone's voice, as their label says it: the
/// same words as their marks, with the local silence first.
fn state_of(person: &Participant) -> Option<&'static str> {
    if person.silenced {
        Some(t("Silenciado para ti"))
    } else if person.deafened || person.server_deafened {
        Some(t("Ensordecido"))
    } else if person.muted || person.server_muted {
        Some(t("Silenciado"))
    } else {
        None
    }
}

/// The call of a voice channel where the conversation goes.
#[derive(Clone, Debug)]
pub struct Stage {
    pub channel: String,
    /// How far the account's call here got; `None` while it is not here.
    pub phase: Option<CallPhase>,
    /// Who is in the channel, the account first.
    pub people: Vec<Participant>,
    /// The account's microphone is muted.
    pub muted: bool,
    pub simulated: bool,
    /// Why the account cannot join, if it cannot.
    pub join_denied: Option<String>,
    /// Whether the channel's chat shows beside the stage; or why it cannot.
    pub chat: Result<bool, String>,
    /// The room it has: the width of the conversation, the window's height.
    pub size: Size,
    /// A person's card, open from their tile.
    pub card: Option<(u64, Element)>,
    /// A person's menu, open from the «⋯» of their tile.
    pub tile_menu: Option<(u64, Vec<MenuEntry>)>,
    /// «Más opciones» of the bar is open.
    pub more_open: bool,
    pub video: VideoControls,
    /// The account's screen and camera, before the people.
    pub previews: Vec<Preview>,
    /// Whether the account can watch the streams here, or why not.
    pub watch: Result<(), String>,
    /// The streams watched: they take the stage, side by side.
    pub watching: Vec<Preview>,
    /// The tiles under a stream focused are hidden: the stage takes their
    /// room, and the pill over the bar shows them again.
    pub tiles_hidden: bool,
    /// A direct or group conversation's call: what it adds. `None` for a
    /// voice channel.
    pub private: Option<crate::private_call::PrivateStage>,
    /// The quick voice panel of the sound (`true`) or the microphone, while
    /// open: [`crate::quick_voice::panel`]. Only the microphone's has an
    /// arrow on the bar.
    pub quick: Option<(bool, Element)>,
    /// The camera's panel over the bar's camera arrow, while open.
    pub camera_panel: Option<Element>,
    /// Room kept free at the header's end for the window's own buttons.
    pub end: f32,
    /// The account may invite people to the channel: the bar offers it.
    pub invite: bool,
    /// The pointer rested on the stage a while: its header, the bar and the
    /// names on the tiles fade out, as Discord's (3 s).
    pub idle: bool,
}

/// The stage of a voice channel: its header, and its tiles over the bar of
/// controls while the account is in the call; else who is there and a way
/// to join.
pub fn stage(view: Stage, look: &Look) -> Element {
    let note = match &view.phase {
        Some(CallPhase::Joining) => Some((t("Conectando…").to_owned(), look.idle)),
        Some(CallPhase::Securing) => Some((t("Asegurando el audio…").to_owned(), look.idle)),
        Some(CallPhase::Reconnecting { attempt }) => {
            Some((tr!("Reconectando (intento {attempt})…", attempt), look.idle))
        }
        Some(CallPhase::Connected) if view.simulated => Some((
            t("Llamada simulada: no se transmite audio").to_owned(),
            look.faint,
        )),
        Some(CallPhase::Waiting) if view.simulated => {
            Some((t("Nadie más en la llamada simulada").to_owned(), look.faint))
        }
        Some(CallPhase::Waiting) => Some((t("Nadie más en la llamada").to_owned(), look.faint)),
        _ => None,
    };
    // `centered_y` wraps its child, so the row's shrinking goes on the wrapper.
    // Both give up width down to a floor: the note never shrinks below what
    // still reads «Llamada simulada…», so a simulated call always says so.
    let private = view.private.as_ref();
    // A conversation's call says how it goes where a channel says its name:
    // the conversation's header is right over it.
    let title = private
        .and_then(|private| private.status.clone())
        .unwrap_or_else(|| view.channel.clone());
    let mut header = vec![
        icon(
            if private.is_some() {
                Icon::Phone
            } else {
                Icon::Volume
            },
            look.faint,
        )
        .size(Some(18.0), Some(18.0))
        .centered_y(),
        strong_line(title, 16.0, look.text)
            .centered_y()
            .min_size(80.0, 0.0)
            .shrink_to_fit(),
    ];
    // The stream focused, after the channel, as the official client's header:
    // «General · (avatar) Ana está jugando a Juego», then at the right the
    // warning over a reduced picture and the quality and «EN VIVO» pill.
    let mut right = Vec::new();
    if let Some(shown) = view.watching.iter().find(|shown| shown.focused) {
        header.push(dot(look));
        let streamer = shown.stream.as_ref().and_then(|stream| {
            let person = view
                .people
                .iter()
                .find(|person| person.user == stream.owner)?;
            Some((person, stream))
        });
        match streamer {
            Some((person, stream)) => {
                header.push(
                    crate::avatar_with_image(
                        &person.name,
                        person.accent,
                        24.0,
                        person.image.clone(),
                    )
                    .centered_y(),
                );
                let label = match &stream.activity {
                    Some(game) => {
                        tr!(
                            "{name} está jugando a {game}",
                            name = &person.name,
                            game = game
                        )
                    }
                    None => tr!("Pantalla de {name}", name = &person.name),
                };
                header.push(
                    strong_line(label, 16.0, look.text)
                        .centered_y()
                        .shrink_to_fit(),
                );
            }
            None => header.push(
                strong_line(&shown.title, 16.0, look.text)
                    .centered_y()
                    .shrink_to_fit(),
            ),
        }
        header.push(Element::spacer().flex(1.0));
        if let Some(stream) = &shown.stream {
            if !stream.viewers.is_empty() {
                right.push(viewers_stack(stream.owner, &stream.viewers, look));
            }
            if stream.degraded {
                right.push(warning(look));
            }
        }
        // «720p 30 FPS»; «720p» until the rate is known.
        let quality = shown.frame.map(|(_, height)| match shown.fps {
            Some(fps) => format!("{height}p {fps} FPS"),
            None => format!("{height}p"),
        });
        // A stream that is over is no longer live.
        if !shown.stream.as_ref().is_some_and(|stream| stream.over) {
            right.push(live_pill(quality, look));
        }
    } else {
        header.push(Element::spacer().flex(1.0));
    }
    if !right.is_empty() {
        header.push(Element::row(right).gap(16.0).centered_y());
    }
    if let Some((note, ink)) = note {
        header.push(
            line(note, 13.0, ink)
                .centered_y()
                .min_size(124.0, 0.0)
                .shrink_to_fit(),
        );
    }
    if let Some(permit) = private.and_then(|private| private.ring_again.as_ref()) {
        header.push(crate::private_call::ring_again(permit, look).centered_y());
    }
    let chat =
        IconButton::new(ids::CHAT, Icon::ChatFill, look.palette()).variant(ButtonVariant::Ghost);
    let chat = match &view.chat {
        Ok(open) => {
            let hint = if *open {
                t("Ocultar chat")
            } else {
                t("Mostrar chat")
            };
            chat.active(*open)
                .tooltip_with(TooltipSpec::new(hint).placement(TooltipPlacement::Below))
        }
        Err(reason) => chat
            .tooltip_with(denied_hint(t("Mostrar chat"), reason).placement(TooltipPlacement::Below))
            .disabled(),
    };
    // A conversation's call has its chat under it already, or beside it
    // once expanded; it switches between the two instead.
    match private {
        None => header.push(chat.render().centered_y()),
        Some(private) => {
            let (glyph, hint) = if private.expanded {
                (Icon::Collapse, t("Reducir la llamada"))
            } else {
                (Icon::Fullscreen, t("Expandir la llamada"))
            };
            header.push(
                IconButton::new(crate::private_call::ids::EXPAND, glyph, look.palette())
                    .variant(ButtonVariant::Ghost)
                    .tooltip_with(TooltipSpec::new(hint).placement(TooltipPlacement::Below))
                    .render()
                    .centered_y(),
            );
        }
    }
    // The window's own buttons end where the row's padding does.
    if view.end > 0.0 {
        header.push(Element::spacer().size(Some(view.end - 12.0), None));
    }
    // The pointer on it keeps it, as on the bar.
    let header = Element::panel(Element::row(header).gap(10.0).padding(12.0))
        .size(None, Some(crate::sidebar::HEADER))
        .background(look.canvas)
        .hover_region_id(ids::HEADER);
    let in_call = matches!(
        view.phase,
        Some(
            CallPhase::Joining
                | CallPhase::Securing
                | CallPhase::Connected
                | CallPhase::Waiting
                | CallPhase::Reconnecting { .. }
        )
    );
    if !in_call {
        let rule = Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border);
        return Element::column(vec![header, rule, invitation(&view, look).flex(1.0)])
            .background(look.canvas);
    }
    // In the call, as Discord's: no rule under the header, which fades in
    // place once the pointer rests, the tiles staying where they are.
    let header = if view.idle {
        Element::spacer().size(None, Some(crate::sidebar::HEADER))
    } else {
        header.animate_presence(ids::HEADER)
    };
    let body = call(&view, look);
    if overlaid(&view) {
        let header = Element::column(vec![header, Element::spacer().flex(1.0)]);
        return Element::stack(vec![body, header]).background(look.canvas);
    }
    Element::column(vec![header, body.flex(1.0)]).background(look.canvas)
}

/// The dot between the channel and the streamer in the header.
fn dot(look: &Look) -> Element {
    Element::row(vec![
        gap(4.0, 0.0),
        Element::panel(Element::spacer())
            .size(Some(4.0), Some(4.0))
            .round()
            .background(look.faint),
        gap(4.0, 0.0),
    ])
    .centered_y()
}

/// Pictures of the spectators in the header: 24 across, 22 apart (so two
/// under each other), each in a ring of the header's ground.
const VIEWER_FACE: f32 = 24.0;
const VIEWER_STEP: f32 = 22.0;
const VIEWER_RING: f32 = 28.0;
/// Past this many, the stack shows this many pictures and a «+N» for the rest.
const VIEWERS_SHOWN: usize = 5;
const VIEWERS_CARD_WIDTH: f32 = 200.0;

/// Who watches a stream, in the header: their pictures overlapping, each
/// named for its accessibility. Hovered, the list opens under it.
fn viewers_stack(owner: u64, viewers: &[Participant], look: &Look) -> Element {
    let heading = tr!("Espectadores: {n}", n = viewers.len());
    let past = viewers.len() > VIEWERS_SHOWN;
    let shown = if past {
        VIEWERS_SHOWN - 1
    } else {
        viewers.len()
    };
    let slot = |index: usize, content: Element| {
        Element::row(vec![gap(index as f32 * VIEWER_STEP, 0.0), content])
            .size(None, Some(VIEWER_RING))
    };
    let mut items: Vec<Element> = viewers
        .iter()
        .take(shown)
        .enumerate()
        .map(|(index, person)| {
            let picture = crate::avatar_with_image(
                &person.name,
                person.accent,
                VIEWER_FACE,
                person.image.clone(),
            );
            let ring = Element::panel(centered(picture))
                .size(Some(VIEWER_RING), Some(VIEWER_RING))
                .round()
                .background(look.canvas);
            slot(index, ring).accessible(
                Access::new(Role::Image)
                    .id(ids::viewer(owner, person.user))
                    .name(person.name.clone()),
            )
        })
        .collect();
    if past {
        let rest = format!("+{}", viewers.len() - shown);
        let chip = Element::panel(Element::row(vec![
            gap(8.0, 0.0),
            Element::text(rest)
                .font_family(FontFamily::SansBold)
                .font_size(12.0)
                .foreground(look.muted)
                .centered_y(),
            gap(8.0, 0.0),
        ]))
        .size(None, Some(VIEWER_FACE))
        .round()
        .background(look.raised)
        .shrink_to_fit()
        .centered_y();
        items.push(slot(shown, chip));
    }
    let stack = Element::clickable(ids::viewers(owner), Element::stack(items))
        .size(None, Some(VIEWER_RING))
        .accessibility_label(heading);
    Element::popover(
        stack,
        viewers_card(viewers, look),
        ids::viewers_dismiss(owner),
        PopoverPlacement::BottomEnd,
    )
    .popover_on_hover()
}

/// The list of a stream's spectators, under their stack: its heading, then
/// each one's picture and name.
fn viewers_card(viewers: &[Participant], look: &Look) -> Element {
    let heading = tr!("Espectadores: {n}", n = viewers.len());
    let mut rows = vec![strong_line(heading.clone(), 12.0, look.muted)];
    for person in viewers {
        rows.push(
            Element::row(vec![
                crate::avatar_with_image(
                    &person.name,
                    person.accent,
                    VIEWER_FACE,
                    person.image.clone(),
                ),
                gap(8.0, 0.0),
                strong_line(person.name.clone(), 14.0, look.text)
                    .centered_y()
                    .shrink_to_fit(),
            ])
            .centered_y(),
        );
    }
    Element::panel(Element::column(rows).gap(8.0).padding(12.0))
        .size(Some(VIEWERS_CARD_WIDTH), None)
        .background(look.menu)
        .border(look.border, 1.0)
        .radius(8.0)
        .shadow(look.shadow, limn::Point::new(0.0, 9.0), 22.0, 0.0)
        .accessibility_label(heading)
}

/// The warning over a stream whose picture is reduced: what it means, on hover.
fn warning(look: &Look) -> Element {
    let glyph = icon(Icon::WarningTriangle, look.idle)
        .size(Some(20.0), Some(20.0))
        .centered_y();
    Tooltip::new(
        glyph,
        ids::WARNING,
        t("La transmisión que estás viendo en este momento tiene una calidad de video o audio reducida. Esto puede deberse a las condiciones de la red."),
        look.menu_palette(),
    )
    .max_width(200.0)
    .render()
}

/// The stream's quality and «EN VIVO» as one pill: two segments that touch,
/// round at their outer ends only; alone, «EN VIVO» is round at both.
fn live_pill(quality: Option<String>, look: &Look) -> Element {
    const LIVE_RED: Color = Color::rgb(0xD2, 0x2D, 0x39);
    const QUALITY_GREY: Color = Color::rgb(0x4E, 0x50, 0x58);
    const RADIUS: f32 = 8.0;
    let segment = |text: String, background: Color| {
        Element::panel(Element::row(vec![
            gap(6.0, 0.0),
            Element::text(text)
                .font_family(FontFamily::SansBold)
                .font_size(12.0)
                .foreground(OVER_INK)
                .centered_y(),
            gap(6.0, 0.0),
        ]))
        .size(None, Some(16.0))
        .background(background)
        .shrink_to_fit()
    };
    let Some(quality) = quality else {
        return segment(t("EN VIVO").to_owned(), LIVE_RED).radius(RADIUS);
    };
    let quality = segment(quality, QUALITY_GREY).corner_radii(CornerRadii {
        top_left: RADIUS,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: RADIUS,
    });
    let quality = Tooltip::new(
        quality,
        ids::STREAM_QUALITY,
        t("Calidad de transmisión"),
        look.menu_palette(),
    )
    .render();
    let live = segment(t("EN VIVO").to_owned(), LIVE_RED).corner_radii(CornerRadii {
        top_left: 0.0,
        top_right: RADIUS,
        bottom_right: RADIUS,
        bottom_left: 0.0,
    });
    Element::row(vec![quality, live]).centered_y()
}

/// The tiles, as large as the room above the bar lets them be, and the bar.
fn call(view: &Stage, look: &Look) -> Element {
    // The streams watched first, then the account's own, then everyone, as
    // the official client: a streamer keeps their tile while watched.
    let previews: Vec<&Preview> = view.watching.iter().chain(&view.previews).collect();
    let people: Vec<&Participant> = view.people.iter().collect();
    let watched = |person: &Participant| {
        let surface = ids::stream_surface(person.user);
        view.watching.iter().any(|shown| shown.surface == surface)
    };
    // A live stream nobody watches is a tile of its own, after its
    // streamer's; the account's own is a card among the previews.
    let offered = |person: &Participant| person.streaming && !person.own && !watched(person);
    let streams = people.iter().filter(|person| offered(person)).count();
    // Private calls: whom it rings, after who is in it.
    let ringing = view
        .private
        .as_ref()
        .map_or(&[][..], |private| &private.ringing[..]);
    // VOICE mode: a private call with nobody on camera and nobody streaming.
    let voice = view
        .private
        .as_ref()
        .is_some_and(|private| private.voice_mode);
    // A stream focused takes the stage, the rest
    // in a strip under it, as in the official client.
    let focus = previews.iter().position(|shown| shown.focused);
    // The pill over the bar offers to hide the tiles, when there are some.
    let hideable = focus.is_some() && previews.len() + people.len() + streams + ringing.len() > 1;
    // The bar's room under the tiles; the pill over the strip or the bar
    // takes none of its own. A conversation's call under its header has its
    // own header over its top, and its bar overlaps the tiles' bottom.
    let overlaid = overlaid(view);
    let (above, below) = if overlaid {
        (
            crate::sidebar::HEADER + AREA_TOP,
            BAR_BOTTOM + BAR_HEIGHT - BAR_OVERLAP,
        )
    } else {
        (AREA_TOP, BAR_BOTTOM + BAR_HEIGHT + BAR_GAP)
    };
    let header = if overlaid {
        0.0
    } else {
        crate::sidebar::HEADER
    };
    let area = Size::new(
        view.size.width - 2.0 * AREA_SIDE,
        view.size.height - header - above - below,
    );
    let person = |person: &Participant, size: Size| {
        let card = view
            .card
            .as_ref()
            .filter(|(user, _)| *user == person.user)
            .map(|(_, card)| card.clone());
        let menu = view
            .tile_menu
            .as_ref()
            .filter(|(user, _)| *user == person.user)
            .map(|(_, menu)| menu.clone());
        tile(
            person,
            size,
            view.size.height,
            card,
            menu,
            view.simulated,
            view.idle,
            look,
        )
    };
    // With a stream watched already, the streams nobody watches offer to be
    // added to those.
    let another = !view.watching.is_empty();
    // Each person's tile, and after it the tile of their stream when offered.
    let people_tiles = |size: Size| -> Vec<Element> {
        people
            .iter()
            .flat_map(|shown| {
                let mut tiles = vec![person(shown, size)];
                if offered(shown) {
                    tiles.push(watch_tile(shown, size, &view.watch, another, look));
                }
                tiles
            })
            .collect()
    };
    // The strip's height, when shown: the pill over the bar lifts onto its top edge.
    let mut strip_height = 0.0;
    let tiles = match (voice, focus) {
        (true, _) => {
            let row: Vec<(&Participant, bool)> = people
                .iter()
                .map(|person| (*person, false))
                .chain(ringing.iter().map(|person| (person, true)))
                .collect();
            // On the stage's black, centred in all of it when the header
            // lies over it (the bar 20 over its bottom), else over the bar.
            let mut column = vec![
                Element::spacer().flex(1.0),
                avatar_row(&row, view.size.width, look),
                Element::spacer().flex(1.0),
            ];
            if !overlaid {
                column.push(gap(0.0, below));
            }
            Element::column(column)
        }
        (false, Some(index)) => {
            let count = previews.len() - 1 + people.len() + streams + ringing.len();
            let strip = if count > 0 && area.height >= STRIP + TILE_GAP + MIN_FOCUS {
                STRIP
            } else {
                0.0
            };
            let strip = if view.tiles_hidden { 0.0 } else { strip };
            strip_height = strip;
            // 16:9 as the official client's tiles, so what is over the
            // stream sits on its picture rather than on black bars.
            let room = Size::new(area.width, area.height - strip - TILE_GAP.min(strip));
            let width = room.width.min(room.height * 16.0 / 9.0);
            let main = Size::new(width, width * 9.0 / 16.0);
            let fits = ((area.width + TILE_GAP) / (STRIP_WIDTH + TILE_GAP)).floor() as usize;
            let fits = fits.min(STRIP_MAX);
            let small = Size::new(STRIP_WIDTH, strip);
            let mut rest: Vec<Element> = previews
                .iter()
                .enumerate()
                .filter(|(at, _)| *at != index)
                .map(|(_, shown)| preview_tile(shown, small, true, look))
                .collect();
            rest.extend(people_tiles(small));
            rest.extend(
                ringing
                    .iter()
                    .map(|shown| crate::private_call::ringing_tile(shown, small, look)),
            );
            rest.truncate(fits.max(1));
            let mut column = vec![
                gap(0.0, above),
                Element::spacer().flex(1.0),
                Element::row(vec![
                    Element::spacer().flex(1.0),
                    preview(previews[index], main, look),
                    Element::spacer().flex(1.0),
                ]),
            ];
            // The strip sits at the bottom, under the spacer that centres the
            // stream above it.
            if strip > 0.0 {
                column.push(gap(0.0, TILE_GAP));
                column.push(Element::spacer().flex(1.0));
                column.push(Element::row(vec![
                    Element::spacer().flex(1.0),
                    Element::row(rest).gap(TILE_GAP),
                    Element::spacer().flex(1.0),
                ]));
            } else {
                column.push(Element::spacer().flex(1.0));
            }
            column.push(gap(0.0, below));
            Element::column(column)
        }
        (false, None) => {
            let count = previews.len() + people.len() + streams + ringing.len();
            let layout = grid(count, area);
            let mut tiles: Vec<Element> = previews
                .iter()
                .map(|shown| preview(shown, layout.tile, look))
                .collect();
            tiles.extend(people_tiles(layout.tile));
            tiles.extend(
                ringing
                    .iter()
                    .map(|shown| crate::private_call::ringing_tile(shown, layout.tile, look)),
            );
            let mut rows = Vec::new();
            let mut tiles = tiles.into_iter().peekable();
            while tiles.peek().is_some() {
                let row: Vec<Element> = tiles.by_ref().take(layout.columns).collect();
                rows.push(Element::row(vec![
                    Element::spacer().flex(1.0),
                    Element::row(row).gap(TILE_GAP),
                    Element::spacer().flex(1.0),
                ]));
            }
            // Past `GRID_VISIBLE` the rows scroll, as Discord's; fewer sit centred.
            let body = if count > GRID_VISIBLE {
                ScrollArea::new(
                    format!("vz:grid:{}", view.channel),
                    Element::column(rows).gap(TILE_GAP),
                )
                .appearance(ScrollbarAppearance::overlay().buttons(false))
                .render()
                .flex(1.0)
            } else {
                Element::column(vec![
                    Element::spacer().flex(1.0),
                    Element::column(rows).gap(TILE_GAP),
                    Element::spacer().flex(1.0),
                ])
                .flex(1.0)
            };
            Element::column(vec![gap(0.0, above), body, gap(0.0, below)])
        }
    };
    // The bar fades (and drops) once the pointer rests on the stage, as the
    // official client's; with a stream focused, with what is over it.
    let idle = view.idle
        || view
            .watching
            .iter()
            .filter(|shown| shown.focused)
            .find_map(|shown| shown.stream.as_ref())
            .is_some_and(|stream| stream.idle);
    let (bar, pill) = if idle {
        (Element::spacer(), None)
    } else {
        (
            bar(view, look).animate_presence(ids::BAR),
            hideable.then(|| tiles_pill(view.tiles_hidden, look)),
        )
    };
    let mut layers = vec![
        tiles,
        Element::column(vec![
            Element::spacer().flex(1.0),
            bar.size(None, Some(BAR_HEIGHT)),
            gap(0.0, BAR_BOTTOM),
        ]),
    ];
    // Centred, 8 under the strip's top edge while the strip shows, as
    // measured on Discord's; else just above the bar.
    if let Some(pill) = pill {
        let lift = if strip_height > 0.0 {
            strip_height - 8.0 - TILES_PILL_HEIGHT
        } else {
            0.0
        };
        layers.push(Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![
                Element::spacer().flex(1.0),
                pill,
                Element::spacer().flex(1.0),
            ])
            .size(None, Some(TILES_PILL_HEIGHT)),
            gap(0.0, below + lift),
        ]));
    }
    Element::stack(layers)
}

/// The toggle over the strip, Discord's: 60×32, round, #2C2D32, a step
/// lighter under the pointer.
const TILES_PILL_WIDTH: f32 = 60.0;
const TILES_PILL_HEIGHT: f32 = 32.0;
const TILES_PILL: Color = Color::rgb(0x2c, 0x2d, 0x32);

/// The toggle over the strip's top or the bar, as the official client's: a
/// chevron and the people. Up shows the strip, down hides it; its label says
/// what it does.
fn tiles_pill(hidden: bool, look: &Look) -> Element {
    let (chevron, hint) = if hidden {
        (Icon::ChevronUp, t("Mostrar miembros"))
    } else {
        (Icon::ChevronDown, t("Ocultar miembros"))
    };
    let pill = Element::clickable(
        ids::TILES,
        centered(Element::row(vec![
            icon(chevron, OVER_INK)
                .size(Some(16.0), Some(16.0))
                .centered_y(),
            icon(Icon::Users, OVER_INK)
                .large_icon(24.0)
                .size(Some(24.0), Some(24.0))
                .centered_y(),
        ])),
    )
    .size(Some(TILES_PILL_WIDTH), Some(TILES_PILL_HEIGHT))
    .round()
    .background(TILES_PILL)
    .hover_background(lighter(TILES_PILL, 12))
    .accessibility_label(hint);
    Tooltip::new(pill, ids::TILES, hint, look.palette()).render()
}

/// A conversation's call under its header: the stage's header lies over its
/// top, as Discord's conversation header over its call.
fn overlaid(view: &Stage) -> bool {
    view.private
        .as_ref()
        .is_some_and(|private| !private.expanded)
}

/// A button of the bar, `width` by 40 with its glyph `glyph` px across,
/// rounded at `corners`: its fill, the fill under the pointer and its ink
/// are `tone`'s (Discord's plain button, or its red or green glow). Drawn
/// off in the faint ink, its hint still saying why, when not `enabled`.
#[allow(clippy::too_many_arguments)]
fn bar_button(
    id: &str,
    glyph: Icon,
    (width, size): (f32, f32),
    corners: CornerRadii,
    (fill, hover, ink): (Color, Color, Color),
    (title, description): (&str, Option<String>),
    enabled: bool,
    look: &Look,
) -> Element {
    let face = |ink: Color| {
        centered(
            icon(glyph, ink)
                .large_icon(size)
                .size(Some(size), Some(size)),
        )
    };
    let button = if enabled {
        Element::clickable(id, face(ink))
            .hover_background(hover)
            .pressed_background(hover)
            .transition_ms(BAR_FADE_MS)
            .accessible(Access::new(Role::Button).name(title))
    } else {
        Element::panel(face(look.faint))
            .accessible(Access::new(Role::Button).id(id).name(title).disabled(true))
    }
    .background(fill)
    .size(Some(width), Some(BAR_BUTTON))
    .corner_radii(corners);
    let hint = Tooltip::new(button, id, title, look.menu_palette());
    match description {
        Some(text) => hint.description(text),
        None => hint,
    }
    .render()
}

/// Rounded 8 at the corners `left` and `right` (a button and its arrow
/// meet square).
fn corners(left: bool, right: bool) -> CornerRadii {
    let at = |round: bool| if round { 8.0 } else { 0.0 };
    CornerRadii {
        top_left: at(left),
        top_right: at(right),
        bottom_right: at(right),
        bottom_left: at(left),
    }
}

/// The bar under the tiles, as Discord's: the microphone and the camera,
/// each with the arrow of its panel; the screen and «Más opciones»; hanging
/// up. The invitation at the left end, a stream focused's controls at the
/// right. Deafening is the account panel's, not the bar's.
fn bar(view: &Stage, look: &Look) -> Element {
    let plain = (Color::TRANSPARENT, PLAIN_HOVER, PLAIN_INK);
    let glow = |on: bool, lit| if on { lit } else { plain };
    // The ghost icons at the bar's ends: the invitation, the stream's controls.
    let tray = |id: String, glyph: Icon, hint: &str| {
        bar_button(
            &id,
            glyph,
            (BAR_BUTTON, 24.0),
            corners(true, true),
            (Color::TRANSPARENT, PLAIN_HOVER, TRAY_INK),
            (hint, None),
            true,
            look,
        )
        .centered_y()
    };
    // Each with the arrow of its quick panel, 1 px after it, which opens
    // over the bar.
    let options = |button: Element, id: &str, hint: &str, tone, panel: Option<&Element>| {
        let arrow = bar_button(
            id,
            if panel.is_some() {
                Icon::ChevronUp
            } else {
                Icon::ChevronDown
            },
            (crate::quick_voice::ARROW_WIDTH, 16.0),
            corners(false, true),
            tone,
            (hint, None),
            true,
            look,
        );
        let arrow = match panel {
            Some(panel) => Element::popover(
                arrow,
                panel.clone(),
                crate::quick_voice::ids::DISMISS,
                PopoverPlacement::TopStart,
            )
            .popover_gap(12.0),
            None => arrow,
        };
        Element::row(vec![button, arrow]).gap(1.0)
    };
    let quick = view
        .quick
        .as_ref()
        .filter(|(output, _)| !output)
        .map(|(_, panel)| panel);
    let tone = glow(view.muted, RED_GLOW);
    let microphone = bar_button(
        ids::MUTE,
        if view.muted {
            Icon::MicrophoneMuted
        } else {
            Icon::Microphone
        },
        (BAR_BUTTON, 20.0),
        corners(true, false),
        tone,
        (
            if view.muted {
                t("Desactivar silencio")
            } else {
                t("Silenciar")
            },
            None,
        ),
        true,
        look,
    );
    // Muted, it rocks under the pointer as the account panel's does.
    let microphone = if view.muted {
        microphone.hover_motion("vz:mute:motion", crate::motion::muted_microphone())
    } else {
        microphone
    };
    let microphone = options(
        microphone,
        ids::INPUT_OPTIONS,
        t("Opciones de entrada"),
        tone,
        quick,
    );
    // The camera and the screen: green while on, off with why when the
    // system has none; their hints say where what they show goes.
    let video = &view.video;
    let video_hint = |id: &str, on: bool, titles: (&'static str, &'static str)| match &video.denied
    {
        Some(reason) => (titles.1, Some(t(reason).to_owned())),
        None => {
            let note = match (id, &video.screen_note, &video.camera_note) {
                (ids::SCREEN, Some(note), _) | (ids::CAMERA, _, Some(note)) => note,
                _ => &video.note,
            };
            (if on { titles.0 } else { titles.1 }, Some(note.clone()))
        }
    };
    let tone = glow(video.camera, GREEN_GLOW);
    let camera = bar_button(
        ids::CAMERA,
        if video.camera {
            Icon::Video
        } else {
            Icon::VideoOff
        },
        (BAR_BUTTON, 20.0),
        corners(true, false),
        tone,
        video_hint(
            ids::CAMERA,
            video.camera,
            (t("Apagar la cámara"), t("Encender la cámara")),
        ),
        video.denied.is_none(),
        look,
    );
    let camera = options(
        camera,
        ids::CAMERA_OPTIONS,
        t("Opciones de cámara"),
        tone,
        view.camera_panel.as_ref(),
    );
    let screen = bar_button(
        ids::SCREEN,
        Icon::Custom(if video.screen {
            SCREEN_STOP
        } else {
            crate::glyphs::SCREEN_SHARE
        }),
        (BAR_BUTTON, 20.0),
        corners(true, true),
        glow(video.screen, GREEN_GLOW),
        video_hint(
            ids::SCREEN,
            video.screen,
            (t("Dejar de compartir"), t("Compartir pantalla")),
        ),
        video.denied.is_none(),
        look,
    );
    let screen = match video.picker {
        Some(chosen) => go_live(screen, chosen, video, view.size, look),
        None => screen,
    };
    // Discord's 4 of padding inside its 1 px edge, which limn draws inside
    // the box.
    let group = |buttons: Vec<Element>| {
        Element::panel(Element::row(buttons).gap(8.0).padding(5.0))
            .size(None, Some(BAR_HEIGHT))
            .radius(12.0)
            .background(GROUP_FILL)
            .border(GROUP_EDGE, 1.0)
    };
    let more = bar_button(
        ids::MORE,
        Icon::More,
        (BAR_BUTTON, 20.0),
        corners(true, true),
        if view.more_open {
            (PLAIN_HOVER, PLAIN_HOVER, PLAIN_INK)
        } else {
            plain
        },
        (t("Más opciones"), None),
        true,
        look,
    );
    let more = DropdownMenu::new(
        more,
        vec![DropdownMenuEntry::Item(
            DropdownMenuItem::new(ids::VOICE_SETTINGS, t("Ajustes de voz y video"))
                .icon(Icon::Settings),
        )],
        view.more_open,
        ids::MORE_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::MENU_METRICS)
    .width(MORE_WIDTH)
    .placement(PopoverPlacement::TopEnd)
    .render();
    // With one focused, as in the official client,
    // hanging up gives way to letting it go.
    let focused = view
        .watching
        .iter()
        .filter(|shown| shown.focused)
        .find_map(|shown| shown.stream.as_ref());
    let (id, glyph, hint) = match focused {
        Some(stream) => (
            ids::stop_watching(stream.owner),
            Icon::Custom(SCREEN_STOP),
            t("Dejar de ver"),
        ),
        None => (ids::LEAVE.to_owned(), Icon::HangUp, t("Desconectar")),
    };
    let [fill, hover, pressed] = HANG_UP;
    let hang_up = Element::clickable(
        id.clone(),
        centered(
            // Discord's phone is about 20 px across: this glyph at 18.
            icon(glyph, Color::rgb(255, 255, 255))
                .large_icon(18.0)
                .size(Some(18.0), Some(18.0)),
        ),
    )
    .size(Some(HANG_UP_WIDTH), Some(BAR_HEIGHT))
    .radius(12.0)
    .background(fill)
    .hover_background(hover)
    .pressed_background(pressed)
    .transition_ms(BAR_FADE_MS)
    .border(GROUP_EDGE, 1.0)
    .accessible(Access::new(Role::Button).name(hint));
    let hang_up = Tooltip::new(hang_up, id, hint, look.menu_palette()).render();
    // The invitation at the left, as the official client's call bar; the
    // stream's controls take the right end. Both ends are the same width, so
    // the groups stay centered whatever is at them. The pointer on the
    // invitation keeps the bar from hiding too.
    let invite = (view.invite && view.size.width > INVITE_MIN_WIDTH).then(|| {
        let hint = if focused.is_some() {
            t("Invitar a transmisión")
        } else {
            t("Invitar")
        };
        tray(ids::INVITE.to_owned(), Icon::Invite, hint).hover_region_id(ids::BAR)
    });
    let left = Element::row(invite.into_iter().collect()).size(Some(BAR_SIDE), Some(BAR_HEIGHT));
    // The stream focused: its sound, its own window and full screen, at the
    // right end as the official client's bar has them.
    let right = match focused {
        Some(stream) => {
            let (glyph, hint) = if stream.silenced {
                (Icon::VolumeMuted, t("Activar el sonido"))
            } else {
                (Icon::Volume, t("Silenciar"))
            };
            let mut buttons = vec![tray(ids::local_mute(stream.sound), glyph, hint)];
            if !stream.windowed {
                buttons.push(tray(
                    ids::window(stream.owner),
                    Icon::Custom(POP_OUT),
                    t("Abrir en otra ventana"),
                ));
            }
            buttons.push(if stream.fullscreen {
                tray(
                    ids::FULLSCREEN_EXIT.to_owned(),
                    Icon::Collapse,
                    t("Salir de pantalla completa"),
                )
            } else {
                tray(
                    ids::fullscreen(stream.owner),
                    Icon::Fullscreen,
                    t("Pantalla completa"),
                )
            });
            Element::row(vec![
                Element::spacer().flex(1.0),
                Element::row(buttons)
                    .gap(8.0)
                    .centered_y()
                    .hover_region_id(ids::BAR),
            ])
            .size(Some(BAR_SIDE), Some(BAR_HEIGHT))
        }
        None => Element::spacer().size(Some(BAR_SIDE), Some(BAR_HEIGHT)),
    };
    // Centered in the stage: the stream's controls at its right end keep the
    // pointer on the bar, as its overlay's did.
    Element::row(vec![
        left,
        Element::spacer().flex(1.0),
        Element::row(vec![
            group(vec![microphone, camera]),
            group(vec![screen, more]),
            hang_up,
        ])
        .gap(BAR_GAP)
        .hover_region_id(ids::BAR),
        Element::spacer().flex(1.0),
        right,
    ])
    .size(None, Some(BAR_HEIGHT))
}

/// Each end of the bar, as wide as the other: the centered groups stay put.
const BAR_SIDE: f32 = 136.0;
/// Narrower than this the stage's bar has no room for the invitation.
const INVITE_MIN_WIDTH: f32 = 650.0;

/// The Go Live modal over `button`, as Discord's: what to
/// share, the stream's quality and its options in the footer, and going
/// live. `chosen` is the quality, `window` the window's size.
fn go_live(
    button: Element,
    chosen: StreamQuality,
    video: &VideoControls,
    window: Size,
    look: &Look,
) -> Element {
    let body = Element::column(vec![
        share_tabs(video.devices, look),
        gap(0.0, 24.0),
        if video.devices {
            no_devices(look)
        } else if video.sources.is_empty() {
            screen_cards(video.kind, look)
        } else {
            source_list(video, look)
        },
    ]);
    let footer = go_live_footer(gear_popout(chosen, video, look), chosen, video, look);
    Modal::new(
        button,
        true,
        "",
        "",
        ids::SHARE_DISMISS,
        look.menu_palette(),
    )
    .width(Modal::fit_width(window.width, GO_LIVE_WIDTH))
    .height((window.height - 90.0).min(GO_LIVE_HEIGHT))
    .backdrop_dismiss(true)
    .body(body)
    .footer(footer)
    .render()
}

/// The modal's width, and its height where the window leaves room.
const GO_LIVE_WIDTH: f32 = 720.0;
const GO_LIVE_HEIGHT: f32 = 550.0;
/// The two presets of the footer: SD, 720p at 30, and HD, 1080p at 60.
const SD: StreamQuality = StreamQuality {
    height: 720,
    fps: 30,
};
const HD: StreamQuality = StreamQuality {
    height: 1080,
    fps: 60,
};

/// One segment of a segmented control: `content`, centred, filled when
/// `selected`.
fn segment(id: &str, content: Element, selected: bool, look: &Look) -> Element {
    let segment = Element::clickable(id, centered(content))
        .size(None, Some(32.0))
        .radius(6.0)
        .hover_background(if selected { look.raised } else { look.hover });
    if selected {
        segment
            .background(lighter(look.raised, 12))
            .border(look.border, 1.0)
    } else {
        segment
    }
}

/// Segments in a bordered pill of `height` high.
fn segmented(segments: Vec<Element>, height: f32, look: &Look) -> Element {
    Element::panel(Element::row(segments).gap(4.0).padding(4.0))
        .size(None, Some(height))
        .radius(8.0)
        .border(look.border, 1.0)
}

/// The modal's tabs: applications, and the devices a camera is.
fn share_tabs(devices: bool, look: &Look) -> Element {
    let tab = |id: &str, glyph: Icon, label: &str, selected: bool| {
        let ink = if selected { look.text } else { look.muted };
        segment(
            id,
            Element::row(vec![
                icon(glyph, ink).centered_y(),
                strong_line(label, 14.0, ink).centered_y(),
            ])
            .gap(8.0),
            selected,
            look,
        )
        .flex(1.0)
    };
    segmented(
        vec![
            tab(ids::SHARE_APPS, Icon::Grid, t("Aplicaciones"), !devices),
            tab(ids::SHARE_DEVICES, Icon::Video, t("Dispositivos"), devices),
        ],
        40.0,
        look,
    )
}

/// The screen and the window to share, where the system chooses (Linux).
fn screen_cards(kind: ShareKind, look: &Look) -> Element {
    let heading = centered(
        Element::text(t("Comparte una pantalla, ventana o aplicación"))
            .font_family(FontFamily::SansBold)
            .font_size(20.0)
            .foreground(look.text),
    )
    .size(None, Some(28.0));
    let card = |id: &str, glyph: Element, label: &str, selected: bool| {
        let fill = if selected {
            lighter(look.raised, 30)
        } else {
            lighter(look.raised, 12)
        };
        let mut card = Element::clickable(
            id,
            centered(Element::column(vec![
                centered(glyph).size(None, Some(32.0)),
                gap(0.0, 16.0),
                centered(strong_line(label, 14.0, look.text)).size(None, Some(20.0)),
            ])),
        )
        .flex(1.0)
        .padding(32.0)
        .radius(8.0)
        .background(fill)
        .hover_background(lighter(look.raised, 22));
        if selected {
            card = card.border(look.accent, 1.0);
        }
        card
    };
    Element::column(vec![
        heading,
        gap(0.0, 32.0),
        Element::row(vec![
            card(
                ids::SHARE_SCREEN,
                Element::icon(Icon::Screen)
                    .large_icon(32.0)
                    .foreground(look.text),
                t("Comparte toda la pantalla"),
                kind == ShareKind::Screen,
            ),
            card(
                ids::SHARE_WINDOW,
                window_glyph(look),
                t("Comparte una ventana"),
                kind == ShareKind::Window,
            ),
        ])
        .gap(16.0),
    ])
}

/// A window, as Discord's card draws it: a rounded box with its title bar's
/// three dots (no glyph of the set says "window").
fn window_glyph(look: &Look) -> Element {
    let dot = || {
        Element::panel(Element::spacer())
            .size(Some(3.0), Some(3.0))
            .radius(1.5)
            .background(look.raised)
    };
    Element::panel(Element::column(vec![
        Element::row(vec![dot(), dot(), dot()]).gap(2.0),
        Element::spacer().flex(1.0),
    ]))
    .padding(4.0)
    .size(Some(32.0), Some(24.0))
    .radius(4.0)
    .background(look.text)
}

/// Rustcord's own list of what it can share (Windows): screens, then windows,
/// the one chosen outlined.
fn source_list(video: &VideoControls, look: &Look) -> Element {
    let mut rows = Vec::new();
    for (windows, heading) in [(false, t("Pantallas")), (true, t("Ventanas"))] {
        let offered: Vec<&ScreenChoice> = video
            .sources
            .iter()
            .filter(|source| source.window == windows)
            .collect();
        if offered.is_empty() {
            continue;
        }
        rows.push(line(heading, 12.0, look.muted));
        for source in offered {
            let row = Element::clickable(
                format!("{}{}", ids::SOURCE, source.id),
                Element::row(vec![
                    icon(if windows { Icon::Grid } else { Icon::Screen }, look.text).centered_y(),
                    line(source.name.clone(), 14.0, look.text)
                        .flex(1.0)
                        .centered_y(),
                ])
                .gap(12.0)
                .padding(12.0),
            )
            .radius(8.0)
            .background(lighter(look.raised, 12))
            .hover_background(lighter(look.raised, 22));
            rows.push(if source.chosen {
                row.border(look.accent, 1.0)
            } else {
                row
            });
        }
    }
    Element::column(rows).gap(8.0)
}

/// The devices tab while the system offers none to share.
fn no_devices(look: &Look) -> Element {
    let centered_x = |child: Element| {
        Element::row(vec![
            Element::spacer().flex(1.0),
            child,
            Element::spacer().flex(1.0),
        ])
    };
    centered(Element::column(vec![
        centered_x(icon(Icon::Video, look.faint).size(Some(48.0), Some(48.0))),
        gap(0.0, 16.0),
        centered_x(
            Element::text(t("No tienes ningún dispositivo de captura disponible."))
                .font_family(FontFamily::SansBold)
                .font_size(16.0)
                .foreground(look.text),
        ),
        gap(0.0, 8.0),
        centered_x(
            text(
                t("En el futuro, puedes usar esta pestaña para compartir desde dispositivos específicos como tu cámara."),
                14.0,
                look.muted,
            )
            .size(Some(440.0), None),
        ),
    ]))
}

/// The gear's popout: the resolution and the rate, the sound, and the
/// screen to choose again. It opens above the gear.
fn gear_popout(chosen: StreamQuality, video: &VideoControls, look: &Look) -> Element {
    let item = |id: String, label: String, on: bool| {
        DropdownMenuEntry::Item(DropdownMenuItem::new(id, label).selected(on))
    };
    let mut entries = vec![DropdownMenuEntry::Heading(
        t("Resolución de la pantalla").into(),
    )];
    entries.extend(StreamQuality::HEIGHTS.iter().map(|height| {
        item(
            format!("{}{height}", ids::QUALITY),
            format!("{height}p"),
            chosen.height == *height,
        )
    }));
    entries.push(DropdownMenuEntry::Heading(
        t("Velocidad de fotogramas").into(),
    ));
    entries.extend(StreamQuality::RATES.iter().map(|fps| {
        item(
            format!("{}{fps}", ids::FPS),
            format!("{fps}fps"),
            chosen.fps == *fps,
        )
    }));
    entries.push(DropdownMenuEntry::Separator);
    entries.push(DropdownMenuEntry::Item(
        DropdownMenuItem::new(ids::SOUND, t("Silenciar audio de transmisión"))
            .checkbox(!video.sound),
    ));
    if video.pick_again {
        entries.push(item(
            ids::PICK_SCREEN.into(),
            t("Elegir otra pantalla").into(),
            false,
        ));
    }
    let gear = IconButton::new(ids::SHARE_OPTIONS, Icon::Settings, look.menu_palette())
        .variant(ButtonVariant::Secondary)
        .sizing(ButtonSizing {
            height: 40.0,
            font_size: 14.0,
            icon_size: 20.0,
            horizontal_padding: 10.0,
            inner_spacing: 0.0,
        })
        .tooltip(t("Opciones"))
        .render();
    DropdownMenu::new(
        gear,
        entries,
        video.options,
        ids::SHARE_OPTIONS_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .width(MORE_WIDTH)
    .placement(PopoverPlacement::TopEnd)
    .render()
}

/// The footer: what is chosen in words, the presets, the gear and the
/// button that goes live. Windows needs a source chosen; the devices tab
/// has nothing to go live with.
fn go_live_footer(
    gear: Element,
    chosen: StreamQuality,
    video: &VideoControls,
    look: &Look,
) -> Element {
    let audio = if video.sound {
        t("Audio del sistema")
    } else {
        t("Transmisión silenciada")
    };
    let (height, fps) = (chosen.height, chosen.fps);
    let summary = Element::column(vec![
        strong_line(t("Personalizados"), 16.0, look.text),
        line(
            tr!("{height}p • {fps}fps • {audio}", height, fps, audio),
            12.0,
            look.muted,
        ),
    ])
    .flex(1.0);
    let presets = segmented(
        [(SD, "SD"), (HD, "HD")]
            .into_iter()
            .map(|(preset, label)| {
                segment(
                    if preset == SD {
                        ids::SHARE_SD
                    } else {
                        ids::SHARE_HD
                    },
                    text(label, 14.0, look.text),
                    chosen == preset,
                    look,
                )
                .size(Some(56.0), Some(32.0))
            })
            .collect(),
        40.0,
        look,
    );
    let linux = video.sources.is_empty();
    let chosen_source = video.sources.iter().any(|source| source.chosen);
    let (label, enabled) = if linux {
        (t("Realiza una selección"), !video.devices)
    } else {
        (t("Transmitir"), chosen_source && !video.devices)
    };
    let go = Button::new(ids::SHARE, label, look.accent_palette())
        .size(ControlSize::Regular)
        .enabled(enabled)
        .render();
    Element::panel(
        Element::row(vec![
            summary,
            Element::row(vec![presets, gear, go]).gap(8.0).centered_y(),
        ])
        .gap(16.0)
        .padding(16.0),
    )
    .radius(8.0)
    .background(lighter(look.menu, 8))
}

/// The people over a call the account is not in, at most, as Discord's preview.
const INVITATION_TILES: usize = 5;
/// Discord's preview tiles: 160 by 90, 8 apart, three to a row (two when
/// there are four).
const INVITATION_TILE: Size = Size {
    width: 160.0,
    height: 90.0,
};

/// One person's tile of the invitation: their colour as in the grid, their
/// picture at 40 px, and the speaking edge while they speak.
fn invitation_tile(person: &Participant, look: &Look) -> Element {
    let picture = centered(crate::avatar_with_image(
        &person.name,
        person.accent,
        40.0,
        person.image.clone(),
    ));
    let mut layers = vec![Element::panel(picture)
        .size(Some(INVITATION_TILE.width), Some(INVITATION_TILE.height))
        .radius(TILE_RADIUS)
        .background(person.accent)];
    if person.speaking {
        layers.push(speaking_edge(INVITATION_TILE, look));
    }
    Element::stack(layers)
        .size(Some(INVITATION_TILE.width), Some(INVITATION_TILE.height))
        .accessibility_label(person.name.clone())
}

/// While the account is not in the call: who is in it, the channel, why it
/// could not connect if it tried, and the way in.
fn invitation(view: &Stage, look: &Look) -> Element {
    // Names up to three, as for who is typing; past that, how many.
    let who = match view.people.as_slice() {
        [] => t("No hay nadie en el chat de voz en este momento").to_owned(),
        [one] => tr!("{one} está en el chat de voz", one = one.name),
        [one, two] => tr!(
            "{one} y {two} están en el chat de voz",
            one = one.name,
            two = two.name
        ),
        [one, two, three] => tr!(
            "{one}, {two} y {three} están en el chat de voz",
            one = one.name,
            two = two.name,
            three = three.name
        ),
        people => tr!(
            "{count} personas están en el chat de voz",
            count = people.len()
        ),
    };
    // Private calls: their own line, and what joining says.
    let invited = view
        .private
        .as_ref()
        .and_then(|private| private.invitation.clone());
    let who = invited.as_ref().map_or(who, |(line, _)| line.clone());
    let mut parts = Vec::new();
    // Who is in it, as tiles of their colour, the speaker's edged green, as
    // Discord's preview shows them. Cameras need Rustcord's own connection: pictures
    // only, until the account joins from here.
    if !view.people.is_empty() {
        let people: Vec<&Participant> = view.people.iter().take(INVITATION_TILES).collect();
        let per_row = if people.len() == 4 { 2 } else { 3 };
        let rows: Vec<Element> = people
            .chunks(per_row)
            .map(|row| {
                let tiles = row
                    .iter()
                    .map(|person| invitation_tile(person, look))
                    .collect();
                centered(Element::row(tiles).gap(8.0)).size(None, Some(INVITATION_TILE.height))
            })
            .collect();
        parts.push(Element::column(rows).gap(8.0));
        parts.push(gap(0.0, 4.0));
    }
    parts.extend([
        centered(
            Element::text(view.channel.clone())
                .font_family(FontFamily::SansBold)
                .font_size(32.0)
                .foreground(look.text),
        )
        .size(None, Some(40.0)),
        // Long names are cut short within the column.
        centered(line(who, 14.0, look.muted).shrink_to_fit()).size(None, Some(20.0)),
    ]);
    if let Some(CallPhase::Failed { reason }) = &view.phase {
        parts.push(centered(text(reason.clone(), 13.0, look.danger)).size(None, Some(20.0)));
    }
    let palette = ComponentPalette {
        text: Color::rgb(255, 255, 255),
        surface: Color::rgb(0, 0, 0),
        ..look.palette()
    };
    let label = if matches!(view.phase, Some(CallPhase::Failed { .. })) {
        t("Reintentar")
    } else if let Some((_, label)) = &invited {
        label.as_str()
    } else {
        t("Unirse al chat de voz")
    };
    // Answering a call coming in is green, as its notice's.
    let incoming = view
        .private
        .as_ref()
        .is_some_and(|private| private.incoming);
    let join = Button::new(ids::JOIN, label, palette)
        .variant(if incoming {
            ButtonVariant::Success
        } else {
            ButtonVariant::Default
        })
        .width(184.0);
    let join = match &view.join_denied {
        Some(reason) => join
            .tooltip_with(TooltipSpec::new(reason.clone()))
            .disabled(),
        None => join,
    };
    parts.push(gap(0.0, 12.0));
    let join = if incoming {
        Element::row(vec![crate::private_call::decline(look), join.render()]).gap(10.0)
    } else {
        join.render()
    };
    parts.push(centered(join).size(None, Some(41.0)));
    Element::panel(Element::column(vec![
        Element::spacer().flex(1.0),
        Element::column(parts).gap(8.0),
        Element::spacer().flex(1.0),
    ]))
    .background(look.canvas)
    // Discord's indigo glow, rising from the bottom over the whole view.
    .bottom_background_fade(Color::rgba(88, 64, 200, 150), view.size.height)
}

/// An entry of a person's menu.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuEntry {
    /// The action it reports, its label, and whether it is checked.
    Item(String, String, Option<bool>),
    /// How loud `user` plays for the account, from 0 to `MAX_VOLUME`, and a
    /// note under it.
    Volume {
        user: u64,
        /// «Volumen de usuario», «Volumen de transmisión».
        label: &'static str,
        volume: f32,
        note: Option<String>,
    },
    Separator,
    /// An item that opens a list of entries beside it: its label, and those
    /// entries.
    Submenu(String, Vec<MenuEntry>),
    // A person's menu as the official client draws it.
    /// An action that removes or punishes, such as kicking: its action and
    /// label, drawn in the theme's danger color.
    Danger(String, String),
    /// A check box in the danger color, such as the server mute: its action,
    /// its label, and whether it is checked.
    DangerChecked(String, String, bool),
    /// An action with a second line under its label that says what it does,
    /// such as «Añadir nota»: its action, its label, and that line.
    Described {
        id: String,
        label: String,
        description: String,
    },
    /// An item that is off with no reason to give, such as a friend request
    /// already sent: its action and label. It does not react to a click.
    Inert(String, String),
    /// An action the account may take but cannot now, such as one the rank
    /// blocks: its action, its label, and why, as a muted second line. It
    /// does not react to a click.
    Disabled(String, String, String),
    /// A role in the menu's roles: its action, its name, its color (`None`
    /// for one without), and whether the person has it, a check box at the
    /// end of the row.
    Role {
        id: String,
        name: String,
        color: Option<Rgb>,
        has: bool,
    },
}

/// The most a person's volume goes: 200 %.
pub const MAX_VOLUME: f32 = 2.0;

/// The top of the volume slider, in Discord's positions (see [`slider_position`]).
pub const SLIDER_TOP: f32 = 200.0;

/// The position of `volume` (1.0 is 100 %) on the volume slider, from 0 to
/// [`SLIDER_TOP`]. Discord's curve reads loudness, not a straight line: a power
/// below 100 % and a logarithm above, so 100 % sits in the middle.
pub fn slider_position(volume: f32) -> f32 {
    100.0
        * if volume < 1.0 {
            volume.powf(0.357)
        } else {
            20.0 * volume.log10() / 6.0 + 1.0
        }
}

/// The volume at `position` on the slider: the inverse of [`slider_position`].
pub fn slider_volume(position: f32) -> f32 {
    let fraction = position / 100.0;
    if fraction < 1.0 {
        fraction.powf(1.0 / 0.357)
    } else {
        10f32.powf((fraction - 1.0) * 6.0 / 20.0)
    }
}

/// A person's menu, 220 wide: Discord's is sized by its content, about 210 px.
pub const MENU_WIDTH: f32 = 220.0;

/// A person's menu as wide as its longest row: Discord's menus fit their
/// content, from 188 to 320 (its submenus' widest), never under `MENU_WIDTH`'s
/// 220 so short menus keep one width.
// limn has no way to give a menu its rows' measured width; it is estimated
// from a 14 px label's average glyph.
pub fn menu_width(entries: &[MenuEntry]) -> f32 {
    const GLYPH: f32 = 7.6;
    const SMALL_GLYPH: f32 = 6.6;
    let text = |label: &str| label.chars().count() as f32 * GLYPH;
    let small = |line: &str| line.chars().count() as f32 * SMALL_GLYPH;
    // A leading mark (a role's dot) or a trailing one (a check box, a
    // submenu's chevron) in one row gives every row its slot.
    let leading = entries
        .iter()
        .any(|entry| matches!(entry, MenuEntry::Role { .. }));
    let trailing = entries.iter().any(|entry| {
        matches!(
            entry,
            MenuEntry::Item(_, _, Some(_))
                | MenuEntry::DangerChecked(..)
                | MenuEntry::Submenu(..)
                | MenuEntry::Role { .. }
        )
    });
    let widest = entries
        .iter()
        .map(|entry| match entry {
            MenuEntry::Item(_, label, _)
            | MenuEntry::DangerChecked(_, label, _)
            | MenuEntry::Submenu(label, _)
            | MenuEntry::Danger(_, label)
            | MenuEntry::Inert(_, label) => text(label),
            MenuEntry::Role { name, .. } => text(name),
            MenuEntry::Described {
                label, description, ..
            } => text(label).max(small(description)),
            MenuEntry::Disabled(_, label, reason) => text(label).max(small(reason)),
            MenuEntry::Volume { .. } | MenuEntry::Separator => 0.0,
        })
        .fold(0.0, f32::max);
    // The menu's inset and the row's padding on both sides, and the slots.
    let slots = if leading { 24.0 } else { 0.0 } + if trailing { 28.0 } else { 0.0 };
    (widest + slots + 32.0).clamp(MENU_WIDTH, 320.0)
}

/// `entries` as limn's menu takes them.
pub fn menu_entries(entries: Vec<MenuEntry>, look: &Look) -> Vec<DropdownMenuEntry> {
    entries
        .into_iter()
        .map(|entry| match entry {
            MenuEntry::Separator => DropdownMenuEntry::Separator,
            MenuEntry::Volume {
                user,
                label,
                volume,
                note,
            } => {
                let position = slider_position(volume);
                let mut parts = vec![
                    line(t(label), 14.0, look.text),
                    // limn's Slider has no value tooltip (Discord shows it while hovering or dragging).
                    Slider::new(
                        ids::volume(user),
                        (position / SLIDER_TOP).clamp(0.0, 1.0),
                        look.palette(),
                    )
                    .ink(look.accent)
                    .accessibility_label(t(label))
                    .render(),
                    line(
                        tr!("{pct} %", pct = format!("{position:.0}")),
                        12.0,
                        look.muted,
                    ),
                ];
                // It wraps: the 220 px menu would cut the sentence that says it is simulated.
                if let Some(note) = note {
                    parts.push(text(note, 12.0, look.faint));
                }
                DropdownMenuEntry::Custom(Box::new(Element::column(parts).gap(6.0).padding(8.0)))
            }
            MenuEntry::Item(id, label, checked) => {
                let item = DropdownMenuItem::new(id, label);
                DropdownMenuEntry::Item(match checked {
                    Some(checked) => item.checkbox(checked),
                    None => item,
                })
            }
            MenuEntry::Submenu(label, choices) => {
                let id = format!("vz:submenu:{label}");
                DropdownMenuEntry::Item(
                    DropdownMenuItem::new(id, label).submenu(menu_entries(choices, look)),
                )
            }
            MenuEntry::Danger(id, label) => {
                DropdownMenuEntry::Item(DropdownMenuItem::new(id, label).destructive(look.danger))
            }
            MenuEntry::DangerChecked(id, label, checked) => DropdownMenuEntry::Item(
                DropdownMenuItem::new(id, label)
                    .checkbox(checked)
                    .destructive(look.danger),
            ),
            MenuEntry::Described {
                id,
                label,
                description,
            } => DropdownMenuEntry::Item(DropdownMenuItem::new(id, label).description(description)),
            MenuEntry::Inert(id, label) => {
                DropdownMenuEntry::Item(DropdownMenuItem::new(id, label).disabled())
            }
            MenuEntry::Disabled(id, label, reason) => DropdownMenuEntry::Item(
                DropdownMenuItem::new(id, label)
                    .description(reason)
                    .disabled(),
            ),
            MenuEntry::Role {
                id,
                name,
                color,
                has,
            } => {
                // A 12 px dot in the role's color, centered in the 16 px icon slot.
                let dot = Element::spacer()
                    .background(color.map_or(look.muted, |color| look.ink(color)))
                    .radius(6.0);
                DropdownMenuEntry::Item(
                    DropdownMenuItem::new(id, name)
                        .leading(Element::panel(dot).padding(2.0))
                        .checkbox(has),
                )
            }
        })
        .collect()
}

/// A person's menu opened by a secondary click, its corner at `at` in the
/// window. Laid over everything: put it last in a stack the window's size.
pub fn context_menu(at: (f32, f32), entries: Vec<MenuEntry>, room: f32, look: &Look) -> Element {
    let width = menu_width(&entries);
    DropdownMenu::new(
        gap(1.0, 1.0),
        menu_entries(entries, look),
        true,
        ids::MENU_DISMISS,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::menu_metrics_within(room))
    .width(width)
    .at(limn::Point::new(at.0.max(0.0), at.1.max(0.0)))
    .render()
}

/// The question before a call from a person's menu, over `root`: whether
/// «No volver a preguntar» is ticked, which the account keeps.
pub fn call_question(root: Element, dont_ask_again: bool, look: &Look) -> Element {
    let palette = look.menu_palette();
    let check = Checkbox::new(ids::PERSON_CALL_FORGET, dont_ask_again, palette)
        .accessibility_label(t("No volver a preguntar"))
        .render()
        .centered_y();
    AlertDialog::new(
        root,
        true,
        t("¿Quieres empezar una llamada?"),
        t("Disfruta de este momento. Avisa cuando lo tengas todo listo."),
        ids::PERSON_CALL,
        ids::PERSON_CALL_CANCEL,
        palette,
    )
    .confirm_label(t("Iniciar llamada"))
    .body(
        Element::row(vec![
            check,
            line(t("No volver a preguntar"), 14.0, look.text),
        ])
        .gap(10.0),
    )
    .render()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::identity::{ActivityButton, ActivityKind, ActivityTime, SpotifyCard};

    use super::*;
    use crate::testing::{controls, frame, texts};

    fn person(user: u64, muted: bool, deafened: bool, speaking: bool) -> Participant {
        Participant {
            user,
            name: format!("Persona {user}"),
            accent: Color::rgb(90, 120, 200),
            muted,
            deafened,
            speaking,
            server_muted: false,
            server_deafened: false,
            silenced: false,
            streaming: false,
            own: false,
            camera: None,
            camera_on: false,
            thumbnail: None,
            image: None,
            decoration: None,
            movable: false,
            carried: false,
            game: None,
        }
    }

    #[test]
    fn a_participant_shows_their_picture_once_it_loaded() {
        let picture = limn::RasterImage::new(
            "2".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "x",
        )
        .unwrap();
        let drawn = |person: &Participant| {
            let (_, laid) = frame(
                participant(person, None, None, &Look::DARK),
                Size::new(240.0, 40.0),
            );
            laid.commands
                .iter()
                .any(|command| matches!(command, limn::DrawCommand::RasterImage { .. }))
        };
        let initial = person(1, false, false, false);
        assert!(!drawn(&initial), "their initial while it loads");
        assert!(drawn(&Participant {
            image: Some(picture),
            ..initial
        }));
    }

    #[test]
    fn a_tile_draws_the_frame_over_their_face() {
        let frame_over = limn::RasterImage::new(
            "3".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "marco",
        )
        .unwrap();
        let drawn = |person: &Participant| {
            let (_, laid) = frame(
                tile(
                    person,
                    Size::new(320.0, 180.0),
                    800.0,
                    None,
                    None,
                    false,
                    false,
                    &Look::DARK,
                ),
                Size::new(320.0, 180.0),
            );
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::RasterImage { image, .. } => Some(image.alt().to_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let plain = person(1, false, false, false);
        assert!(drawn(&plain).is_empty(), "their initial alone");
        assert_eq!(
            drawn(&Participant {
                decoration: Some(frame_over),
                ..plain
            }),
            ["marco"]
        );
    }

    #[test]
    fn a_server_mute_is_marked_red_and_said_apart_from_their_own() {
        let look = Look::DARK;
        let red_mic = |person: &Participant| {
            let (_, laid) = frame(
                participant(person, None, None, &look),
                Size::new(240.0, 40.0),
            );
            laid.commands.iter().any(|command| {
                matches!(command, limn::DrawCommand::Icon { icon: Icon::MicrophoneMuted, tint, .. } if *tint == look.danger)
            })
        };
        let own = person(1, true, false, false);
        assert!(!red_mic(&own));
        let held = Participant {
            server_muted: true,
            ..person(1, false, false, false)
        };
        assert!(
            red_mic(&held),
            "unmuted by choice, still held by the server"
        );
    }

    fn connected() -> Panel {
        Panel {
            channel: "General".into(),
            server: Some("Rustcord prueba".into()),
            phase: CallPhase::Connected,
            quality: Some(CallQuality {
                rtt_ms: 47,
                loss_permille: 4,
            }),
            simulated: true,
            live: false,
            stream_source: None,
            stream_quality: None,
            noise_suppression: false,
            noise_open: false,
            video: VideoControls {
                denied: Some(NO_VIDEO.into()),
                ..Default::default()
            },
        }
    }

    fn rect_of(frame: &limn::Frame, id: &str) -> limn::Rect {
        frame
            .regions
            .iter()
            .find(|region| region.id == id)
            .map(|region| region.rect)
            .unwrap()
    }

    #[test]
    fn call_clocks_read_as_minutes_until_the_hour() {
        assert_eq!(clock(Duration::from_secs(207)), "03:27");
        assert_eq!(clock(Duration::from_secs(3_729)), "1:02:09");
        assert_eq!(loss(4), "0,4 %");
    }

    #[test]
    fn the_card_is_100_high_and_says_where_and_how_the_call_is() {
        let look = Look::ONYX;
        let alone = Element::column(vec![
            panel(&connected(), None, &look),
            Element::spacer().flex(1.0),
        ]);
        let (_, laid) = frame(alone, Size::new(357.5, 300.0));
        let shown = texts(&laid);
        assert!(shown.contains(&"Voz conectada (simulada)".to_owned()));
        assert!(shown.contains(&"General / Rustcord prueba".to_owned()));
        let ids = controls(&laid);
        assert!(ids.contains(&ids::DETAILS.to_owned()));
        assert!(ids.contains(&ids::LEAVE.to_owned()));
        // No video on this system: the camera and the screen are drawn, not usable.
        assert!(!ids.contains(&ids::CAMERA.to_owned()));
        assert!(!ids.contains(&ids::SCREEN.to_owned()));
        let local = Panel {
            video: VideoControls::default(),
            ..connected()
        };
        let (_, usable) = frame(panel(&local, None, &look), Size::new(357.5, 300.0));
        let usable = controls(&usable);
        assert!(
            usable.contains(&ids::CAMERA.to_owned()) && usable.contains(&ids::SCREEN.to_owned())
        );
        // Its signal starts 12 down, its buttons end 12 up: 100 high.
        let (_, placed) = frame(panel(&local, None, &look), Size::new(357.5, 300.0));
        let rect = |id: &str| {
            placed
                .regions
                .iter()
                .find(|region| region.id == id)
                .map(|region| region.rect)
                .unwrap()
        };
        let (signal, camera) = (rect(ids::DETAILS), rect(ids::CAMERA));
        assert_eq!((signal.y, signal.height), (PANEL_INSET, 32.0));
        assert_eq!(camera.y + camera.height, PANEL_HEIGHT - PANEL_INSET);
        assert_eq!(camera.height, 32.0);
    }

    #[test]
    fn a_streaming_account_shows_its_card_above_the_panel_with_a_stop() {
        let look = Look::DARK;
        let streaming = Panel {
            stream_source: Some("Una ventana, pantalla o aplicación".into()),
            stream_quality: Some("720p 30 FPS".into()),
            ..connected()
        };
        let (_, laid) = frame(panel(&streaming, None, &look), Size::new(357.5, 300.0));
        let shown = texts(&laid);
        assert!(shown
            .iter()
            .any(|text| text == "Una ventana, pantalla o aplicación"));
        assert!(shown.iter().any(|text| text == "720p 30 FPS"));
        assert!(controls(&laid).contains(&ids::STREAM_STOP.to_owned()));
        // The card is 56 high, its rule 1 more, and the panel's title comes under both.
        let stop = rect_of(&laid, ids::STREAM_STOP);
        assert!(stop.y < rect_of(&laid, ids::DETAILS).y);
        assert_eq!((stop.y, stop.height), (PANEL_INSET, PANEL_BUTTON));
        assert_eq!(
            rect_of(&laid, ids::DETAILS).y,
            STREAM_CARD + 1.0 + PANEL_INSET
        );
        // Not streaming: no card, no stop, the panel as before.
        let (_, idle) = frame(panel(&connected(), None, &look), Size::new(357.5, 300.0));
        assert!(!controls(&idle).contains(&ids::STREAM_STOP.to_owned()));
        assert_eq!(rect_of(&idle, ids::DETAILS).y, PANEL_INSET);
    }

    #[test]
    fn the_action_row_has_two_equal_buttons_eight_apart() {
        let look = Look::DARK;
        let local = Panel {
            video: VideoControls::default(),
            ..connected()
        };
        let (_, laid) = frame(panel(&local, None, &look), Size::new(357.5, 300.0));
        let (camera, screen) = (rect_of(&laid, ids::CAMERA), rect_of(&laid, ids::SCREEN));
        // 357.5 less the card's 12 on each side, two buttons and a gap of 8.
        let width = (357.5 - 2.0 * PANEL_INSET - PANEL_GAP) / 2.0;
        assert!((camera.width - width).abs() < 0.01, "{}", camera.width);
        assert!((screen.width - width).abs() < 0.01, "{}", screen.width);
        assert!((screen.x - (camera.x + camera.width + PANEL_GAP)).abs() < 0.01);
    }

    #[test]
    fn the_wave_sits_beside_the_hang_up_and_opens_the_noise_popout() {
        let look = Look::DARK;
        let (_, laid) = frame(panel(&connected(), None, &look), Size::new(357.5, 300.0));
        let (wave, leave) = (rect_of(&laid, ids::NOISE), rect_of(&laid, ids::LEAVE));
        assert_eq!(wave.y, leave.y);
        assert!((wave.x + wave.width + 4.0 - leave.x).abs() < 0.01);
        assert!(!controls(&laid).contains(&ids::NOISE_SWITCH.to_owned()));
        // The popout lays out over the panel, so its content is checked alone.
        let (_, popped) = frame(noise_popout(true, &look), Size::new(244.0, 300.0));
        let ids_now = controls(&popped);
        assert!(ids_now.contains(&ids::NOISE_SWITCH.to_owned()));
        assert!(ids_now.contains(&ids::NOISE_SETTINGS.to_owned()));
        // The sentence wraps in 212 px at 7 px a character.
        assert!(texts(&popped)
            .iter()
            .any(|text| text.starts_with("Filtra el ruido")));
    }

    #[test]
    fn a_call_alone_in_its_channel_says_it_waits() {
        let waiting = Panel {
            phase: CallPhase::Waiting,
            simulated: false,
            ..connected()
        };
        let (_, laid) = frame(panel(&waiting, None, &Look::DARK), Size::new(357.5, 300.0));
        let shown = texts(&laid);
        assert!(
            shown.contains(&"Esperando a los demás".to_owned()),
            "{shown:?}"
        );
        // The simulation waits too, and says it is one.
        let simulated = Panel {
            simulated: true,
            ..waiting
        };
        let (_, laid) = frame(
            panel(&simulated, None, &Look::DARK),
            Size::new(357.5, 300.0),
        );
        // The wave takes its width from the title: at this width it is cut short.
        let shown = texts(&laid);
        assert!(
            shown
                .iter()
                .any(|text| text.starts_with("Esperando a los demás (simul")),
            "{shown:?}"
        );
        // So does the stage.
        let view = |simulated| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Waiting),
            people: vec![person(1, false, false, false)],
            muted: false,
            simulated,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        for (simulated, said) in [
            (false, "Nadie más en la llamada"),
            (true, "Nadie más en la llamada simulada"),
        ] {
            let (_, laid) = frame(stage(view(simulated), &Look::ONYX), Size::new(980.0, 762.0));
            let shown = texts(&laid);
            assert!(shown.contains(&said.to_owned()), "{shown:?}");
        }
    }

    #[test]
    fn a_failed_call_says_why_and_offers_to_try_again() {
        let failed = Panel {
            phase: CallPhase::Failed {
                reason: "Se perdió la conexión".into(),
            },
            ..connected()
        };
        let (_, laid) = frame(panel(&failed, None, &Look::DARK), Size::new(357.5, 300.0));
        let shown = texts(&laid);
        assert!(shown.contains(&"No se pudo conectar la voz".to_owned()));
        assert!(shown.contains(&"Se perdió la conexión".to_owned()));
        assert!(controls(&laid).contains(&ids::RETRY.to_owned()));
    }

    #[test]
    fn the_details_average_the_latency_and_say_the_values_are_made_up() {
        let view = Details {
            samples: vec![
                ("10:00".into(), 40),
                ("10:01".into(), 47),
                ("10:02".into(), 46),
            ],
            loss_permille: Some(0),
            duration: Some(Duration::from_secs(207)),
            simulated: true,
        };
        let (_, laid) = frame(
            details_tabbed(&view, None, &Look::DARK),
            Size::new(DETAILS_WIDTH, 600.0),
        );
        let shown = texts(&laid);
        for expected in ["Conexión", "44 ms", "46 ms", "0,0 %", "Duración:", "03:27"] {
            assert!(
                shown.iter().any(|text| text == expected),
                "{expected} in {shown:?}"
            );
        }
        assert!(shown.iter().any(|text| text.contains("Valores simulados")));
    }

    #[test]
    fn tiles_take_the_arrangement_that_makes_them_largest() {
        // Two in a tall room go one over the other, as captured: 546 wide,
        // whole pixels, 307 high.
        let two = grid(2, Size::new(946.0, 622.5));
        assert_eq!(two.columns, 1);
        assert_eq!(two.tile, Size::new(546.0, 307.0), "{two:?}");
        let four = grid(4, Size::new(946.0, 622.5));
        assert_eq!(four.columns, 2);
        assert_eq!(four.tile, Size::new(469.0, 264.0), "{four:?}");
        // Discord's two side by side in its 1064 wide stage: 520×293.
        let pair = grid(2, Size::new(1048.0, 578.0));
        assert_eq!((pair.columns, pair.tile), (2, Size::new(520.0, 293.0)));
        assert_eq!(grid(0, Size::new(100.0, 100.0)).columns, 1);
    }

    /// The area of a stage 1000 by 500 for its tiles: the insets of the call
    /// (8 at the sides, 16 at the top) and the bar's room under them.
    fn stage_area() -> Size {
        Size::new(
            1000.0 - 2.0 * AREA_SIDE,
            500.0 - AREA_TOP - (BAR_BOTTOM + BAR_HEIGHT + BAR_GAP),
        )
    }

    #[test]
    fn seven_tiles_take_240_by_135_in_rows_of_four_and_three() {
        let layout = grid(7, stage_area());
        assert_eq!(layout.columns, 4);
        assert_eq!(layout.tile, Size::new(240.0, 135.0));
        let rows: Vec<usize> = [(); 7].chunks(layout.columns).map(<[()]>::len).collect();
        assert_eq!(rows, [4, 3]);
    }

    #[test]
    fn thirty_tiles_are_as_large_as_keeps_25_in_view_and_their_rows_scroll() {
        let area = stage_area();
        let layout = grid(30, area);
        // 158 wide: five across, and five rows partly in view at most.
        assert_eq!(layout.columns, 5);
        assert_eq!(layout.tile, Size::new(158.0, 89.0));
        let rows = 30usize.div_ceil(layout.columns) as f32;
        let content = rows * layout.tile.height + (rows - 1.0) * TILE_GAP;
        assert!(content > area.height, "{content} in {area:?}");
    }

    #[test]
    fn a_stage_of_thirty_scrolls_its_tiles_below_its_header() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: (1..=30)
                .map(|user| person(user, false, false, false))
                .collect(),
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let (_, laid) = frame(stage(view, &look), size);
        let scroll = laid
            .regions
            .iter()
            .find(|region| region.id == "vz:grid:General")
            .expect("the rows scroll");
        assert!(matches!(scroll.kind, limn::HitKind::Scroll));
        let below = BAR_BOTTOM + BAR_HEIGHT + BAR_GAP;
        // No rule under the header in a call, as Discord's.
        let high = size.height - crate::sidebar::HEADER - AREA_TOP - below;
        assert!((scroll.rect.height - high).abs() < 0.5, "{:?}", scroll.rect);
        // Only the tiles in view are laid out as controls, at most 25 of them.
        let tiles = controls(&laid)
            .iter()
            .filter(|id| id.starts_with("vz:tile:"))
            .count();
        assert!(tiles > 0 && tiles <= GRID_VISIBLE, "{tiles} in view");
    }

    #[test]
    fn a_tile_names_its_person_from_125_wide_and_its_watch_says_watch_by_width() {
        let look = Look::ONYX;
        let streamer = Participant {
            streaming: true,
            own: false,
            ..person(2, false, false, false)
        };
        let watch = Ok(());
        let face = |width: f32| {
            let size = Size::new(width, width * 9.0 / 16.0);
            let element = tile(&streamer, size, 800.0, None, None, false, false, &look);
            texts(&frame(element, size).1)
        };
        let offer = |width: f32| {
            let size = Size::new(width, width * 9.0 / 16.0);
            let element = watch_tile(&streamer, size, &watch, false, &look);
            texts(&frame(element, size).1)
        };
        assert_eq!(face_size(Size::new(124.0, 70.0)), 80.0);
        assert_eq!(face_size(Size::new(123.0, 70.0)), 40.0);
        assert!(!face(124.0).contains(&"Persona 2".to_owned()));
        assert!(face(125.0).contains(&"Persona 2".to_owned()));
        assert!(!face(200.0).contains(&"EN VIVO".to_owned()));
        let narrow = offer(124.0);
        assert!(!narrow.contains(&"Persona 2".to_owned()), "{narrow:?}");
        assert!(narrow.contains(&"Ver".to_owned()), "{narrow:?}");
        let wide = offer(200.0);
        assert!(wide.contains(&"Persona 2".to_owned()), "{wide:?}");
        assert!(wide.contains(&"Mirar transmisión".to_owned()), "{wide:?}");
        assert!(wide.contains(&"EN VIVO".to_owned()));
    }

    #[test]
    fn a_live_stream_nobody_watches_is_a_tile_after_its_streamers() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let streaming = |user| Participant {
            streaming: true,
            own: false,
            ..person(user, false, false, false)
        };
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false), streaming(2)],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let (_, laid) = frame(stage(view, &look), size);
        // Three tiles: the person, their stream after them, and the first one.
        let tiles: Vec<String> = controls(&laid)
            .into_iter()
            .filter(|id| id.starts_with("vz:tile:") || id.starts_with("vz:stream-tile:"))
            .collect();
        assert_eq!(tiles.len(), 3, "{tiles:?}");
        let at = |id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap_or_else(|| panic!("{id}"))
                .rect
        };
        let (face, stream) = (at(&ids::tile(2)), at(&ids::watch_tile(2)));
        assert!(
            (stream.y, stream.x) > (face.y, face.x),
            "{stream:?} after {face:?}"
        );
        let said = texts(&laid);
        assert_eq!(said.iter().filter(|text| *text == "EN VIVO").count(), 1);
        assert_eq!(
            said.iter()
                .filter(|text| *text == "Mirar transmisión")
                .count(),
            1
        );
        assert!(controls(&laid).contains(&ids::watch(2)));
    }

    #[test]
    fn the_stage_shows_each_person_and_a_bar_with_the_video_controls() {
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, true, true, false), person(2, false, false, true)],
            muted: true,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        // The camera's arrow opens its panel over the bar.
        let open = Stage {
            camera_panel: Some(Element::text("Panel de la cámara")),
            ..view.clone()
        };
        let (_, opened) = frame(stage(open, &Look::ONYX), Size::new(980.0, 762.0));
        assert!(texts(&opened).contains(&"Panel de la cámara".to_owned()));
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let ids = controls(&laid);
        for id in [ids::MUTE, ids::MORE, ids::LEAVE] {
            assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
        }
        // Deafening is the account panel's, as Discord's bar has it.
        assert!(!ids.iter().any(|id| id == "vz:deafen"), "{ids:?}");
        for id in [ids::CAMERA, ids::SCREEN, ids::CAMERA_OPTIONS] {
            assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
        }
        assert!(!texts(&laid).contains(&"Panel de la cámara".to_owned()));
        assert!(
            !ids.contains(&ids::CHAT.to_owned()),
            "the chat is not usable"
        );
        assert!(ids.contains(&ids::tile(1)) && ids.contains(&ids::tile(2)));
        let shown = texts(&laid);
        assert!(shown.contains(&"Persona 1".to_owned()));
        assert!(shown.iter().any(|text| text.contains("simulada")));
        // The one speaking is outlined in green.
        let outlined = laid.commands.iter().any(|command| {
            matches!(command, limn::DrawCommand::StrokeRect { color, .. } if *color == Look::ONYX.online)
        });
        assert!(outlined);
    }

    /// The call's stage with `video` as its controls, in a window of 980×762.
    fn picker_stage(video: VideoControls) -> Stage {
        Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false)],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video,
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            previews: Vec::new(),
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        }
    }

    #[test]
    fn go_live_offers_the_tabs_the_cards_and_says_what_goes_out_in_the_footer() {
        let view = picker_stage(VideoControls {
            picker: Some(StreamQuality::default()),
            sound: true,
            ..Default::default()
        });
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let ids = controls(&laid);
        for id in [
            ids::SHARE_APPS,
            ids::SHARE_DEVICES,
            ids::SHARE_SCREEN,
            ids::SHARE_WINDOW,
            ids::SHARE_SD,
            ids::SHARE_HD,
            ids::SHARE_OPTIONS,
            ids::SHARE,
        ] {
            assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
        }
        let shown = texts(&laid);
        for expected in [
            "Aplicaciones",
            "Dispositivos",
            "Comparte una pantalla, ventana o aplicación",
            "Comparte toda la pantalla",
            "Comparte una ventana",
            "Realiza una selección",
            "Personalizados",
            "720p • 30fps • Audio del sistema",
        ] {
            assert!(
                shown.iter().any(|text| text == expected),
                "{expected} in {shown:?}"
            );
        }
    }

    #[test]
    fn go_live_says_when_the_stream_is_muted_and_its_gear_lists_sizes_rates_and_sound() {
        let view = picker_stage(VideoControls {
            picker: Some(StreamQuality {
                height: 1080,
                fps: 60,
            }),
            options: true,
            ..Default::default()
        });
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let ids = controls(&laid);
        for id in [
            "vz:quality:480",
            "vz:quality:1080",
            "vz:fps:15",
            "vz:fps:60",
            ids::SOUND,
        ] {
            assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
        }
        let shown = texts(&laid);
        for expected in [
            "1080p • 60fps • Transmisión silenciada",
            "Resolución de la pantalla",
            "Velocidad de fotogramas",
            "Silenciar audio de transmisión",
        ] {
            assert!(
                shown.iter().any(|text| text == expected),
                "{expected} in {shown:?}"
            );
        }
    }

    #[test]
    fn the_devices_tab_says_there_is_none_and_offers_no_source_to_go_live_with() {
        let view = picker_stage(VideoControls {
            picker: Some(StreamQuality::default()),
            devices: true,
            ..Default::default()
        });
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let ids = controls(&laid);
        assert!(!ids.contains(&ids::SHARE_SCREEN.to_owned()), "{ids:?}");
        assert!(!ids.contains(&ids::SHARE.to_owned()), "disabled: {ids:?}");
        let shown = texts(&laid);
        assert!(shown
            .iter()
            .any(|text| { text == "No tienes ningún dispositivo de captura disponible." }));
        // The sentence is cut across lines, as separate texts.
        assert!(shown.concat().contains(
            "En el futuro, puedes usar esta pestaña para compartir desde dispositivos específicos como tu cámara."
        ), "{shown:?}");
    }

    #[test]
    fn rustcord_s_own_picker_lists_screens_then_windows_and_goes_live_with_one_chosen() {
        let source = |id: &str, name: &str, window, chosen| ScreenChoice {
            id: id.into(),
            name: name.into(),
            window,
            chosen,
        };
        let sources = |chosen: bool| {
            vec![
                source(
                    r"screen:\\.\DISPLAY1",
                    "Pantalla 1 (principal) · 1920×1080",
                    false,
                    chosen,
                ),
                source("window:42", "Un juego", true, false),
            ]
        };
        let open = |sources| {
            let view = picker_stage(VideoControls {
                picker: Some(StreamQuality::default()),
                sources,
                ..Default::default()
            });
            let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
            (controls(&laid), texts(&laid))
        };
        let (ids, shown) = open(sources(false));
        assert!(
            ids.contains(&r"vz:source:screen:\\.\DISPLAY1".to_owned()),
            "{ids:?}"
        );
        assert!(ids.contains(&"vz:source:window:42".to_owned()), "{ids:?}");
        assert!(
            !ids.contains(&ids::SHARE.to_owned()),
            "nothing chosen: {ids:?}"
        );
        let at = |text: &str| {
            shown
                .iter()
                .position(|shown| shown == text)
                .unwrap_or_else(|| panic!("{text} in {shown:?}"))
        };
        assert!(at("Pantallas") < at("Pantalla 1 (principal) · 1920×1080"));
        assert!(at("Ventanas") < at("Un juego"));
        let (ids, shown) = open(sources(true));
        assert!(ids.contains(&ids::SHARE.to_owned()), "{ids:?}");
        assert!(shown.iter().any(|text| text == "Transmitir"), "{shown:?}");
    }

    #[test]
    fn previews_come_first_say_they_stay_here_and_can_be_stopped() {
        let note = "Solo tú lo ves. La transmisión llega con las llamadas.";
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false)],
            muted: false,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls {
                screen: true,
                camera: true,
                denied: None,
                note: note.into(),
                ..Default::default()
            },
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
            previews: vec![
                Preview {
                    surface: ids::SCREEN_SURFACE.into(),
                    title: "Tu pantalla".into(),
                    waiting: "Esperando al sistema…".into(),
                    frame: Some((1920, 1080)),
                    fps: None,
                    stop: (ids::STOP_SCREEN.into(), "Dejar de compartir"),
                    note: note.into(),
                    actions: vec![(ids::PAUSE.into(), "Pausar")],
                    stream: None,
                    focused: false,
                    card: false,
                },
                Preview {
                    surface: ids::CAMERA_SURFACE.into(),
                    title: "Tu cámara".into(),
                    waiting: "Esperando al sistema…".into(),
                    frame: None,
                    fps: None,
                    stop: (ids::STOP_CAMERA.into(), "Apagar la cámara"),
                    note: note.into(),
                    actions: Vec::new(),
                    stream: None,
                    focused: false,
                    card: false,
                },
            ],
        };
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let ids = controls(&laid);
        assert!(
            ids.contains(&ids::STOP_SCREEN.to_owned())
                && ids.contains(&ids::STOP_CAMERA.to_owned())
                && ids.contains(&ids::PAUSE.to_owned())
        );
        let shown = texts(&laid);
        for expected in ["Tu pantalla", "Tu cámara", "Esperando al sistema…", note] {
            assert!(
                shown.iter().any(|text| text == expected),
                "{expected} in {shown:?}"
            );
        }
        // Only the frame that arrived is drawn, at 16:9 in its tile.
        let surfaces: Vec<_> = laid
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::Surface { id, rect, .. } => Some((id.clone(), *rect)),
                _ => None,
            })
            .collect();
        assert_eq!(surfaces.len(), 1, "{surfaces:?}");
        assert_eq!(surfaces[0].0, ids::SCREEN_SURFACE);
        let rect = surfaces[0].1;
        assert!((rect.width / rect.height - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn only_the_cameras_preview_is_mirrored() {
        let mirrored = |surface: &str| {
            let tile = Preview {
                surface: surface.into(),
                title: "Vista previa".into(),
                waiting: "Esperando al sistema…".into(),
                frame: Some((1280, 720)),
                fps: None,
                stop: (ids::STOP_CAMERA.into(), "Apagar la cámara"),
                note: "Sólo en esta computadora".into(),
                actions: Vec::new(),
                stream: None,
                focused: false,
                card: false,
            };
            let (_, laid) = frame(
                preview(&tile, Size::new(480.0, 270.0), &Look::ONYX),
                Size::new(480.0, 270.0),
            );
            laid.commands.iter().find_map(|command| match command {
                limn::DrawCommand::Surface { mirrored, .. } => Some(*mirrored),
                _ => None,
            })
        };
        assert_eq!(
            mirrored(ids::CAMERA_SURFACE),
            Some(true),
            "as a webcam shows the person"
        );
        assert_eq!(mirrored(ids::SCREEN_SURFACE), Some(false));
    }

    #[test]
    fn out_of_the_call_the_stage_invites_to_join() {
        let view = Stage {
            channel: "General".into(),
            phase: None,
            people: Vec::new(),
            muted: false,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        let shown = texts(&laid);
        assert!(shown.contains(&"No hay nadie en el chat de voz en este momento".to_owned()));
        assert!(controls(&laid).contains(&ids::JOIN.to_owned()));
        assert!(!controls(&laid).contains(&ids::LEAVE.to_owned()));
    }

    #[test]
    fn the_invitation_shows_five_tiles_at_most_three_to_a_row_or_two_for_four() {
        let invite = |people: Vec<Participant>| Stage {
            channel: "General".into(),
            phase: None,
            people,
            muted: false,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        // The tiles' rectangles, grouped by row: how many sit in each.
        let rows = |count: u64| -> Vec<usize> {
            let people = (1..=count).map(|user| person(user, false, false, false));
            let (_, laid) = frame(
                stage(invite(people.collect()), &Look::ONYX),
                Size::new(980.0, 762.0),
            );
            let mut rows: Vec<(f32, usize)> = Vec::new();
            for command in &laid.commands {
                let limn::DrawCommand::FillRect { rect, radius, .. } = command else {
                    continue;
                };
                if (rect.width, rect.height, *radius) != (160.0, 90.0, TILE_RADIUS) {
                    continue;
                }
                match rows.iter_mut().find(|(y, _)| *y == rect.y) {
                    Some((_, n)) => *n += 1,
                    None => rows.push((rect.y, 1)),
                }
            }
            rows.into_iter().map(|(_, n)| n).collect()
        };
        assert_eq!(rows(3), vec![3]);
        assert_eq!(rows(4), vec![2, 2]);
        assert_eq!(rows(5), vec![3, 2]);
        assert_eq!(rows(6), vec![3, 2], "no more than five");
        let (_, laid) = frame(
            stage(
                invite(
                    (1..=5)
                        .map(|user| person(user, false, false, false))
                        .collect(),
                ),
                &Look::ONYX,
            ),
            Size::new(980.0, 762.0),
        );
        assert!(texts(&laid).contains(&"Unirse al chat de voz".to_owned()));
    }

    #[test]
    fn a_row_under_the_channel_marks_who_is_muted_or_deafened() {
        let look = Look::DARK;
        let (_, laid) = frame(
            participant(&person(7, true, true, false), None, None, &look),
            Size::new(280.0, ROW_HEIGHT),
        );
        let ids: Vec<String> = laid
            .regions
            .iter()
            .map(|region| region.id.clone())
            .collect();
        assert!(ids.contains(&ids::row(7)));
        assert!(ids.contains(&format!("{}:muted", ids::row(7))));
        assert!(ids.contains(&format!("{}:deafened", ids::row(7))));
        assert!(!ids.contains(&format!("{}:silenced", ids::row(7))));
        let silenced = Participant {
            silenced: true,
            ..person(7, false, false, false)
        };
        let (_, laid) = frame(
            participant(&silenced, None, None, &look),
            Size::new(280.0, ROW_HEIGHT),
        );
        assert!(laid
            .regions
            .iter()
            .any(|region| region.id == format!("{}:silenced", ids::row(7))));
    }

    #[test]
    fn the_account_streaming_is_live_too_with_nothing_to_watch() {
        let me = Participant {
            streaming: true,
            own: true,
            ..person(1, false, false, false)
        };
        // Their stream is its card among the previews, which says it is live.
        let card = Preview {
            surface: ids::OWN_THUMB_SURFACE.into(),
            title: "¡Estás en directo!".into(),
            frame: None,
            fps: None,
            waiting: "Esperando al sistema…".into(),
            stop: (ids::OWN_STREAM_STOP.into(), "Dejar de transmitir"),
            note: "Los demás ven tu pantalla.".into(),
            actions: Vec::new(),
            stream: None,
            focused: false,
            card: true,
        };
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![me.clone(), person(2, false, false, false)],
            muted: false,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: vec![card],
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let look = Look::ONYX;
        let (_, laid) = frame(stage(view, &look), Size::new(980.0, 762.0));
        // Their tile is plain, with no stream tile of their own; the card says
        // it is live, and no «Mirar transmisión» is offered for their own.
        assert!(texts(&laid).iter().any(|text| text == "EN VIVO"));
        assert!(!controls(&laid).contains(&ids::watch(1)));
        assert!(!controls(&laid).contains(&ids::watch_tile(1)));
        // Their row in the voice list says it too.
        let (_, row) = frame(participant(&me, None, None, &look), Size::new(300.0, 40.0));
        assert!(texts(&row).iter().any(|text| text == "EN VIVO"));
    }

    #[test]
    fn someone_streaming_is_live_can_be_watched_and_their_stream_takes_the_stage() {
        let streamer = Participant {
            streaming: true,
            own: false,
            ..person(2, false, false, false)
        };
        let view = |watch: Result<(), String>, watching| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false), streamer.clone()],
            muted: false,
            simulated: true,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch,
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let look = Look::ONYX;
        let (_, laid) = frame(
            stage(view(Ok(()), Vec::new()), &look),
            Size::new(980.0, 762.0),
        );
        assert!(controls(&laid).contains(&ids::watch(2)));
        assert!(
            !controls(&laid).contains(&ids::watch(1)),
            "only who streams"
        );
        assert!(texts(&laid).iter().any(|text| text == "EN VIVO"));
        // Not allowed: shown, disabled, with why.
        let (_, laid) = frame(
            stage(view(Err("Entra a la llamada".into()), Vec::new()), &look),
            Size::new(980.0, 762.0),
        );
        assert!(!controls(&laid).contains(&ids::watch(2)));
        // Watched: the stream alone, with the way to stop.
        let stream = Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: "En vivo · simulado".into(),
            actions: Vec::new(),
            stream: None,
            focused: false,
            card: false,
        };
        let (_, laid) = frame(
            stage(view(Ok(()), vec![stream]), &look),
            Size::new(980.0, 762.0),
        );
        let ids = controls(&laid);
        assert!(ids.contains(&ids::stop_watching(2)));
        // Everyone stays, as in the official client: the streamer's tile
        // no longer offers to watch.
        assert!(ids.contains(&ids::tile(1)) && ids.contains(&ids::tile(2)));
        assert!(!ids.contains(&ids::watch(2)));
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::Surface { id, .. } if *id == ids::stream_surface(2)
        )));
        // Their row in the column says it too.
        let (_, laid) = frame(
            participant(&streamer, None, None, &look),
            Size::new(280.0, ROW_HEIGHT),
        );
        assert!(texts(&laid).iter().any(|text| text == "EN VIVO"));
        // In a window of its own: the picture, nothing to click.
        let stream = Preview {
            surface: ids::window_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: "En vivo".into(),
            actions: Vec::new(),
            stream: None,
            focused: false,
            card: false,
        };
        let (_, laid) = frame(
            stream_window(&stream, Size::new(960.0, 540.0), false, &look),
            Size::new(960.0, 540.0),
        );
        assert!(controls(&laid).is_empty());
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::Surface { id, rect, .. }
                if *id == ids::window_surface(2) && rect.width == 960.0
        )));
        // With a picture of their stream under it, to watch it from there.
        // The row is its own height: no picture of the stream under it.
        let previewed = Participant {
            thumbnail: Some(Thumbnail::Simulated),
            ..streamer.clone()
        };
        let (_, laid) = frame(
            participant(&previewed, None, None, &look),
            Size::new(280.0, ROW_HEIGHT + 100.0),
        );
        assert!(!texts(&laid)
            .iter()
            .any(|text| text == "Vista previa simulada"));
    }

    #[test]
    fn the_picture_in_picture_is_a_16_9_box_in_its_corner_with_its_channel_and_buttons() {
        let button = |id: String, hint, glyph| PipButton { id, hint, glyph };
        let mut view = PipView {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            channel: "Sala común".into(),
            frame: Some((1280, 720)),
            waiting: "Esperando video…".into(),
            note: None,
            buttons: vec![
                button(ids::window(2), "Ventana aparte", PipGlyph::PopOut),
                button(ids::stop_watching(2), "Dejar de ver", PipGlyph::StopShare),
                button(ids::LEAVE.into(), "Desconectar", PipGlyph::HangUp),
            ],
            corner: PipCorner {
                bottom: false,
                right: true,
            },
            offset: limn::Point::new(0.0, 0.0),
        };
        let window = Size::new(1280.0, 800.0);
        let (_, laid) = frame(pip(&view, &Look::DARK), window);
        let shown = controls(&laid);
        assert!(shown.contains(&ids::PIP.to_owned()), "a click returns");
        for id in [ids::window(2), ids::stop_watching(2), ids::LEAVE.into()] {
            assert!(shown.contains(&id), "{id} is on the box");
        }
        assert!(texts(&laid).iter().any(|text| text == "Sala común"));
        assert!(texts(&laid).iter().any(|text| text == "EN VIVO"));
        // 1280x720 fits its 320 across, and the box is 180 high.
        assert!(laid.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::Surface { id, rect, .. }
                if *id == ids::stream_surface(2) && rect.width == PIP_WIDTH && rect.height == 180.0
        )));
        let rect = |laid: &limn::Frame| {
            laid.regions
                .iter()
                .rev()
                .find(|region| region.id == ids::PIP)
                .unwrap()
                .rect
        };
        // Top right: 16 in from the edges, the header's height and a gap under it.
        let at = rect(&laid);
        assert_eq!(at.x + at.width, window.width - PIP_MARGIN);
        assert_eq!(at.y, PIP_MARGIN + crate::sidebar::HEADER);
        // Bottom left: the same gap from the other edges, nothing under the header.
        view.corner = PipCorner {
            bottom: true,
            right: false,
        };
        let (_, laid) = frame(pip(&view, &Look::DARK), window);
        let at = rect(&laid);
        assert_eq!(at.x, PIP_MARGIN);
        assert_eq!(at.y + at.height, window.height - PIP_MARGIN);
        // Before the first picture: what it waits for, and no surface yet.
        view.frame = None;
        let (_, laid) = frame(pip(&view, &Look::DARK), window);
        assert!(texts(&laid).iter().any(|text| text == "Esperando video…"));
        assert!(!laid
            .commands
            .iter()
            .any(|command| matches!(command, limn::DrawCommand::Surface { .. })));
    }

    #[test]
    fn a_stream_focused_takes_the_stage_and_the_rest_go_in_a_strip_under_it() {
        let streaming = |user| Participant {
            streaming: true,
            own: false,
            ..person(user, false, false, false)
        };
        let stream = |user: u64, focused| Preview {
            surface: ids::stream_surface(user),
            title: format!("Pantalla de Persona {user}"),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(user), "Dejar de ver"),
            note: "En vivo".into(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: user,
                name: format!("Persona {user}"),
                sound: user + 1_000,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                zoom: None,
                failed: false,
            }),
            focused,
            card: false,
        };
        let view = |watching| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                streaming(2),
                streaming(3),
                streaming(4),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let surfaces = |laid: &limn::Frame| -> Vec<(String, limn::Rect)> {
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::Surface { id, rect, .. } => Some((id.clone(), *rect)),
                    _ => None,
                })
                .collect()
        };
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        // Side by side, each tile clickable to focus it.
        let (_, grid) = frame(
            stage(view(vec![stream(2, false), stream(3, false)]), &look),
            size,
        );
        let shown = surfaces(&grid);
        assert_eq!(shown.len(), 2);
        let ids = controls(&grid);
        assert!(ids.contains(&ids::stream_tile(2)));
        assert!(ids.contains(&ids::watch(4)));
        // Under the pointer, as the official client's: its sound and «⋯»;
        // the bar hangs up and has nothing at its end.
        assert!(ids.contains(&ids::local_mute(1_002)) && ids.contains(&ids::stream_more(2)));
        assert!(ids.contains(&ids::LEAVE.to_owned()));
        assert!(!ids.contains(&ids::stop_watching(2)) && !ids.contains(&ids::fullscreen(2)));
        // Focused: it takes the stage, the other and who else streams go
        // small under it, without their buttons.
        let (_, focused) = frame(
            stage(view(vec![stream(2, true), stream(3, false)]), &look),
            size,
        );
        let shown = surfaces(&focused);
        let big = shown
            .iter()
            .find(|(id, _)| *id == ids::stream_surface(2))
            .unwrap()
            .1;
        let small = shown
            .iter()
            .find(|(id, _)| *id == ids::stream_surface(3))
            .unwrap()
            .1;
        assert!(big.width > 2.0 * small.width);
        assert!(small.height <= STRIP + 0.01 && small.y > big.y + big.height);
        let ids = controls(&focused);
        assert!(
            ids.contains(&ids::stream_tile(3)),
            "the strip's focus it instead"
        );
        // The header names the stream focused and how it comes; the one in
        // the strip says it is live, as the official client's.
        let said = texts(&focused);
        let count = |text: &str| said.iter().filter(|shown| *shown == text).count();
        assert_eq!(count("Pantalla de Persona 2"), 1, "the header");
        assert_eq!(
            count("Persona 2"),
            2,
            "the pill and the streamer's own tile"
        );
        assert_eq!(count("720p"), 1);
        assert!(count("EN VIVO") >= 2, "header and the strip's stream");
        // The streamer focused keeps their tile; the strip's offer no watching.
        assert!(ids.contains(&ids::tile(2)) && !ids.contains(&ids::watch(3)));
        // The bar lets go of the one focused in place of hanging up; its
        // sound, its own window and full screen are icons on the bar at its
        // right end, as the official client's.
        assert!(ids.contains(&ids::stop_watching(2)) && !ids.contains(&ids::stop_watching(3)));
        assert!(!ids.contains(&ids::LEAVE.to_owned()));
        assert!(ids.contains(&ids::fullscreen(2)) && !ids.contains(&ids::fullscreen(3)));
        assert!(ids.contains(&ids::window(2)));
        let at = |laid: &limn::Frame, id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap_or_else(|| panic!("{id}"))
                .rect
        };
        let (sound, window, full, more) = (
            at(&focused, &ids::local_mute(1_002)),
            at(&focused, &ids::window(2)),
            at(&focused, &ids::fullscreen(2)),
            at(&focused, &ids::stream_more(2)),
        );
        assert!(sound.x < window.x && window.x < full.x);
        assert!(sound.x > big.x + big.width / 2.0 && sound.y > big.y + big.height);
        assert!(more.x + more.width <= big.x + big.width + 0.5);
        assert!(more.y > big.y + big.height / 2.0, "at the bottom");
        assert_eq!((full.width, full.height), (BAR_BUTTON, BAR_BUTTON));
        // Already in a window of its own: only full screen.
        let mut windowed = stream(2, true);
        windowed.stream.as_mut().unwrap().windowed = true;
        let (_, laid) = frame(stage(view(vec![windowed, stream(3, false)]), &look), size);
        assert!(!controls(&laid).contains(&ids::window(2)));
        // Its «⋯» opens its menu there.
        let mut open = stream(2, false);
        open.stream.as_mut().unwrap().menu = Some(vec![MenuEntry::Item(
            ids::stop_watching(2),
            "Dejar de ver".into(),
            None,
        )]);
        let (_, laid) = frame(stage(view(vec![open, stream(3, false)]), &look), size);
        assert!(texts(&laid).contains(&"Dejar de ver".to_owned()));
    }

    #[test]
    fn the_bar_is_spaced_as_the_official_clients_tray() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let view = Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let (_, laid) = frame(stage(view, &look), size);
        let at = |id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .unwrap_or_else(|| panic!("{id}"))
                .rect
        };
        let (mute, camera, leave) = (at(ids::MUTE), at(ids::CAMERA), at(ids::LEAVE));
        assert_eq!((mute.width, mute.height), (BAR_BUTTON, BAR_BUTTON));
        // Each arrow 20 wide, 1 after its button; 8 between it and the next
        // button, 5 (edge and padding) inside a group, 12 between groups.
        let arrow = at(ids::INPUT_OPTIONS);
        assert_eq!((arrow.x, arrow.width), (mute.x + mute.width + 1.0, 20.0));
        assert_eq!(camera.x, arrow.x + arrow.width + 8.0);
        let (camera_arrow, screen) = (at(ids::CAMERA_OPTIONS), at(ids::SCREEN));
        assert_eq!(
            screen.x,
            camera_arrow.x + camera_arrow.width + 5.0 + BAR_GAP + 5.0
        );
        assert_eq!(mute.y, leave.y + 5.0, "in the middle of the 50 high bar");
        assert_eq!((leave.width, leave.height), (HANG_UP_WIDTH, BAR_HEIGHT));
        assert_eq!(leave.y + leave.height, size.height - BAR_BOTTOM);
        // The tiles end 12 over the bar.
        let tile = at(&ids::tile(1));
        assert!(tile.y + tile.height <= leave.y - BAR_GAP + 0.5);
        // Centered.
        let left = at(ids::MUTE).x - 5.0;
        let right = leave.x + leave.width;
        assert!(((left + right) / 2.0 - size.width / 2.0).abs() < 1.0);
    }

    #[test]
    fn a_stream_nobody_watches_has_its_own_tile_with_the_picture_and_no_face() {
        let image = limn::RasterImage::new(
            "1".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "vista previa",
        )
        .unwrap();
        let shown = |thumbnail| Participant {
            streaming: true,
            own: false,
            thumbnail,
            ..person(2, false, false, false)
        };
        let look = Look::ONYX;
        let size = Size::new(480.0, 270.0);
        let offer =
            |person: &Participant| frame(watch_tile(person, size, &Ok(()), false, &look), size).1;
        let pictured = offer(&shown(Some(Thumbnail::Image(image.clone()))));
        assert!(pictured
            .commands
            .iter()
            .any(|command| matches!(command, limn::DrawCommand::RasterImage { .. })));
        let texts_shown = texts(&pictured);
        for expected in ["EN VIVO", "Mirar transmisión", "Persona 2"] {
            assert!(
                texts_shown.iter().any(|text| text == expected),
                "{expected}"
            );
        }
        // None yet: the ground alone; the simulation's says it is made up.
        let plain = offer(&shown(None));
        assert!(!plain
            .commands
            .iter()
            .any(|command| matches!(command, limn::DrawCommand::RasterImage { .. })));
        let simulated = offer(&shown(Some(Thumbnail::Simulated)));
        assert!(texts(&simulated)
            .iter()
            .any(|text| text == "Vista previa simulada"));
        // Their own tile is a plain person's: no picture of the stream, no
        // «EN VIVO», no watch button; the stream has its tile of its own.
        let face = frame(
            tile(
                &shown(Some(Thumbnail::Image(image))),
                size,
                800.0,
                None,
                None,
                false,
                false,
                &look,
            ),
            size,
        )
        .1;
        assert!(!face
            .commands
            .iter()
            .any(|command| matches!(command, limn::DrawCommand::RasterImage { .. })));
        let words = texts(&face);
        assert!(words.iter().any(|text| text == "Persona 2"));
        assert!(!words.iter().any(|text| text == "EN VIVO"), "{words:?}");
        assert!(!controls(&face).contains(&ids::watch(2)));
    }

    #[test]
    fn a_stream_nobody_watches_has_a_grey_veil_under_its_watch_button_on_any_stream() {
        let look = Look::ONYX;
        let size = Size::new(480.0, 270.0);
        let streamer = Participant {
            streaming: true,
            own: false,
            ..person(2, false, false, false)
        };
        let (_, laid) = frame(watch_tile(&streamer, size, &Ok(()), false, &look), size);
        let at = laid
            .regions
            .iter()
            .find(|region| region.id == ids::watch(2))
            .unwrap()
            .rect;
        // Raised at rest: 12 % of a grey, where the dark ground reads it.
        assert!(
            fills(&laid, Color::rgba(152, 152, 160, 31)).contains(&at),
            "{at:?}"
        );
    }

    #[test]
    fn a_camera_that_shows_takes_the_place_of_the_mark_and_says_when_simulated() {
        let filmed = Participant {
            camera: Some((640, 360)),
            ..person(2, false, false, false)
        };
        let view = |simulated| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false), filmed.clone()],
            muted: false,
            simulated,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let look = Look::ONYX;
        let surfaces = |laid: &limn::Frame| -> Vec<(String, limn::Rect)> {
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::Surface { id, rect, .. } => Some((id.clone(), *rect)),
                    _ => None,
                })
                .collect()
        };
        let (_, laid) = frame(stage(view(true), &look), Size::new(980.0, 762.0));
        let shown = surfaces(&laid);
        assert_eq!(shown.len(), 1, "{shown:?}");
        assert_eq!(shown[0].0, ids::camera_surface(2));
        // 16:9 in a 16:9 tile: it fills it.
        assert!((shown[0].1.width / shown[0].1.height - 16.0 / 9.0).abs() < 0.01);
        assert!(texts(&laid).iter().any(|text| text == "Cámara simulada"));
        let (_, laid) = frame(stage(view(false), &look), Size::new(980.0, 762.0));
        assert!(!texts(&laid).iter().any(|text| text == "Cámara simulada"));
        assert!(controls(&laid).contains(&ids::tile(2)));
    }

    #[test]
    fn the_chat_toggles_beside_the_stage_when_the_channel_has_one() {
        let view = |chat| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false)],
            muted: false,
            simulated: true,
            join_denied: None,
            chat,
            size: Size::new(600.0, 600.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching: Vec::new(),
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        for open in [false, true] {
            let (_, laid) = frame(stage(view(Ok(open)), &Look::ONYX), Size::new(600.0, 600.0));
            assert!(controls(&laid).contains(&ids::CHAT.to_owned()));
        }
    }

    // A person's menu, laid out alone as `context_menu` opens it.
    fn person_menu(entries: Vec<MenuEntry>, look: &Look) -> limn::Frame {
        frame(
            context_menu((0.0, 0.0), entries, 800.0, look),
            Size::new(600.0, 600.0),
        )
        .1
    }

    fn icon_centers(laid: &limn::Frame, wanted: Icon) -> Vec<f32> {
        laid.commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::Icon { rect, icon, .. } if *icon == wanted => {
                    Some(rect.x + rect.width / 2.0)
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_person_s_check_boxes_end_their_rows_where_a_submenu_s_chevron_does() {
        let look = Look::ONYX;
        let laid = person_menu(
            vec![
                MenuEntry::Item(
                    "vz:mod:mute:7".into(),
                    "Silenciar servidor".into(),
                    Some(true),
                ),
                MenuEntry::Submenu(
                    "Mover a".into(),
                    vec![MenuEntry::Item(
                        "vz:mod:move:7:9".into(),
                        "Sala".into(),
                        None,
                    )],
                ),
                MenuEntry::Separator,
                MenuEntry::Item(ids::local_mute(7), "Silenciar".into(), Some(true)),
            ],
            &look,
        );
        let (checks, chevrons) = (
            icon_centers(&laid, Icon::Check),
            icon_centers(&laid, Icon::ChevronRight),
        );
        assert_eq!((checks.len(), chevrons.len()), (2, 1));
        for check in checks {
            assert_eq!(check, chevrons[0], "one column at the row's end");
        }
    }

    #[test]
    fn a_role_has_a_dot_of_its_color_and_a_check_box_and_punishing_is_in_danger_red() {
        let look = Look::ONYX;
        let red = Rgb(0xE7_4C_3C);
        let laid = person_menu(
            vec![
                MenuEntry::Role {
                    id: "role:1".into(),
                    name: "Moderación".into(),
                    color: Some(red),
                    has: true,
                },
                MenuEntry::Role {
                    id: "role:2".into(),
                    name: "Bots".into(),
                    color: None,
                    has: false,
                },
                MenuEntry::Separator,
                MenuEntry::Item("timeout-off".into(), "Quitar el aislamiento".into(), None),
                MenuEntry::Danger("kick".into(), "Expulsar a Ada".into()),
            ],
            &look,
        );
        let dot = |color: Color| {
            laid.commands.iter().any(|command| {
                matches!(command, limn::DrawCommand::FillRect { rect, color: fill, .. }
                    if *fill == color && rect.width == 12.0 && rect.height == 12.0)
            })
        };
        assert!(dot(look.ink(red)), "the role's own color");
        assert!(dot(look.muted), "a role without one is grey");
        // The role it has is ticked at the end of its row, past its name.
        let name = laid
            .commands
            .iter()
            .find_map(|command| match command {
                // Seven pixels a character, as `frame` measures.
                limn::DrawCommand::Text { text, position, .. } if text == "Moderación" => {
                    Some(position.x + text.chars().count() as f32 * 7.0)
                }
                _ => None,
            })
            .unwrap();
        assert!(icon_centers(&laid, Icon::Check)[0] > name);
        let ink = |wanted: &str| {
            laid.commands
                .iter()
                .find_map(|command| match command {
                    limn::DrawCommand::Text { text, color, .. } if text == wanted => Some(*color),
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(ink("Expulsar a Ada"), look.danger);
        assert_eq!(ink("Quitar el aislamiento"), look.text);
    }

    #[test]
    fn an_action_that_is_off_says_why_and_does_not_react() {
        let look = Look::ONYX;
        let entries = menu_entries(
            vec![MenuEntry::Disabled(
                "ss:kick".into(),
                "Expulsar a Ada".into(),
                "Su rol está a la altura del tuyo o por encima.".into(),
            )],
            &look,
        );
        let (_, laid) = frame(
            DropdownMenu::new(
                gap(1.0, 1.0),
                entries,
                true,
                ids::MENU_DISMISS,
                look.menu_palette(),
            )
            .width(MENU_WIDTH)
            .render(),
            Size::new(400.0, 400.0),
        );
        let shown = texts(&laid);
        assert!(shown.contains(&"Expulsar a Ada".to_owned()), "{shown:?}");
        // The reason is one line: limn cuts it with an ellipsis.
        assert!(
            shown
                .iter()
                .any(|text| text.starts_with("Su rol está a la altura")),
            "{shown:?}"
        );
        assert!(!controls(&laid).contains(&"ss:kick".to_owned()));
    }

    #[test]
    fn a_person_s_volume_is_a_slider_in_their_menu() {
        let look = Look::ONYX;
        let entries = menu_entries(
            vec![
                MenuEntry::Volume {
                    user: 7,
                    label: "Volumen de usuario",
                    volume: 1.5,
                    note: Some("Simulado".into()),
                },
                MenuEntry::Item(ids::local_mute(7), "Silenciar".into(), Some(true)),
            ],
            &look,
        );
        let (_, laid) = frame(
            DropdownMenu::new(
                gap(1.0, 1.0),
                entries,
                true,
                ids::MENU_DISMISS,
                look.menu_palette(),
            )
            .width(MENU_WIDTH)
            .render(),
            Size::new(400.0, 400.0),
        );
        let ids = controls(&laid);
        assert!(ids.contains(&ids::volume(7)), "{ids:?}");
        assert!(ids.contains(&ids::local_mute(7)), "{ids:?}");
        let shown = texts(&laid);
        // 150 % of volume is at 159 on Discord's slider.
        for expected in ["Volumen de usuario", "159 %", "Simulado", "Silenciar"] {
            assert!(
                shown.iter().any(|text| text == expected),
                "{expected} in {shown:?}"
            );
        }
    }

    #[test]
    fn the_volume_slider_reads_loudness_as_discord_does_and_comes_back() {
        // Silence at the bottom, 100 % in the middle, 200 % at the top.
        assert!(slider_position(0.0).abs() < 1e-4);
        assert!((slider_position(1.0) - 100.0).abs() < 1e-4);
        assert!((slider_position(2.0) - 200.0).abs() < 0.5);
        for volume in [0.1_f32, 0.5, 0.75, 1.0, 1.5, 1.99] {
            let back = slider_volume(slider_position(volume));
            assert!((back - volume).abs() < 1e-4, "{volume} came back as {back}");
        }
        assert!(slider_volume(SLIDER_TOP) <= MAX_VOLUME);
    }

    #[test]
    fn an_item_with_a_line_under_it_says_what_it_does_and_one_off_does_not_react() {
        let look = Look::ONYX;
        let laid = person_menu(
            vec![
                MenuEntry::Described {
                    id: "pp:full-note:7".into(),
                    label: "Añadir nota".into(),
                    description: "Solo tú puedes ver esto".into(),
                },
                MenuEntry::Inert(
                    "friend:7:sent".into(),
                    "Solicitud de amistad enviada".into(),
                ),
            ],
            &look,
        );
        let shown = texts(&laid);
        assert!(shown.contains(&"Añadir nota".to_owned()), "{shown:?}");
        assert!(
            shown.contains(&"Solo tú puedes ver esto".to_owned()),
            "{shown:?}"
        );
        let clickable = controls(&laid);
        assert!(clickable.contains(&"pp:full-note:7".to_owned()));
        assert!(!clickable.contains(&"friend:7:sent".to_owned()));
    }

    #[test]
    fn the_server_mute_is_a_check_box_in_the_danger_color() {
        let look = Look::ONYX;
        let laid = person_menu(
            vec![MenuEntry::DangerChecked(
                "vz:mod:mute:7".into(),
                "Silenciar servidor".into(),
                true,
            )],
            &look,
        );
        assert!(controls(&laid).contains(&"vz:mod:mute:7".to_owned()));
        assert_eq!(icon_centers(&laid, Icon::Check).len(), 1, "ticked");
        let ink = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::Text { text, color, .. } if text == "Silenciar servidor" => {
                    Some(*color)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(ink, look.danger);
    }

    #[test]
    fn the_call_question_says_what_it_asks_and_has_the_check_box_not_to_ask_again() {
        let look = Look::ONYX;
        let (_, laid) = frame(
            call_question(Element::spacer(), false, &look),
            Size::new(800.0, 600.0),
        );
        let shown = texts(&laid);
        // The body wraps over two lines in the test's measure: its first is enough.
        for expected in [
            "¿Quieres empezar una llamada?",
            "Disfruta de este momento.",
            "No volver a preguntar",
        ] {
            assert!(
                shown.iter().any(|text| text.starts_with(expected)),
                "{expected} in {shown:?}"
            );
        }
        let clickable = controls(&laid);
        for id in [
            ids::PERSON_CALL,
            ids::PERSON_CALL_CANCEL,
            ids::PERSON_CALL_FORGET,
        ] {
            assert!(clickable.contains(&id.to_owned()), "{id} in {clickable:?}");
        }
    }

    /// The hover's fills of `look.hover` painted in `frame`.
    fn hover_fills(frame: &limn::Frame, look: &Look) -> Vec<limn::Rect> {
        frame
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect {
                    rect,
                    color,
                    radius,
                } if *color == look.hover => {
                    assert_eq!(*radius, ROW_RADIUS);
                    Some(*rect)
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_row_under_the_pointer_lights_up_inside_it_as_a_channel_does() {
        use limn::{Point, UiEvent};
        let look = Look::DARK;
        let width = crate::sidebar::WIDTH - 1.0;
        let channel = crate::sidebar::channel(
            crate::sidebar::ChannelRow {
                id: "voice:1".into(),
                name: "Sala".into(),
                voice: true,
                ..Default::default()
            },
            &look,
        );
        let muted = person(7, true, true, false);
        let column = Element::column(vec![
            channel,
            participant(&muted, None, None, &look),
            participant(&person(8, false, false, false), None, None, &look),
        ]);
        let size = Size::new(width, 200.0);
        let (mut tree, laid) = frame(column, size);
        assert!(hover_fills(&laid, &look).is_empty(), "nothing lit at rest");
        let region = |laid: &limn::Frame, id: &str| {
            laid.regions
                .iter()
                .find(|region| region.id == id)
                .map(|region| region.rect)
                .unwrap()
        };
        let row = region(&laid, &ids::row(7));
        let next = region(&laid, &ids::row(8));
        // The row reaches the column's edge, for the card beside it.
        assert_eq!(row.x + row.width, width);
        tree.handle_event(UiEvent::PointerMove(Point::new(row.x + 20.0, row.y + 10.0)));
        let lit = hover_fills(&tree.layout(size), &look);
        assert_eq!(lit.len(), 1, "{lit:?}");
        let lit = lit[0];
        // Its right edge where a channel's is, short of the scroll bar.
        assert_eq!(lit.x + lit.width, width - crate::sidebar::INSET);
        assert_eq!(lit.x, ROW_INDENT);
        // Inside its row: it does not reach the next one.
        assert_eq!(lit.y, row.y + 1.0);
        assert_eq!(lit.height, ROW_HEIGHT - 2.0);
        assert!(lit.y + lit.height < next.y);
        // The marks stay inside it.
        let deafened = region(&tree.layout(size), &format!("{}:deafened", ids::row(7)));
        assert!(deafened.x + deafened.width <= lit.x + lit.width - 4.0);
        // The channel lights the same way, rounded alike.
        let voice = region(&tree.layout(size), "voice:1");
        tree.handle_event(UiEvent::PointerMove(Point::new(
            voice.x + 20.0,
            voice.y + 10.0,
        )));
        let lit = hover_fills(&tree.layout(size), &look);
        assert_eq!(lit.len(), 1, "{lit:?}");
        assert_eq!(lit[0].x + lit[0].width, width - crate::sidebar::INSET);
    }

    #[test]
    fn a_voice_row_is_32_high_with_its_picture_24_under_the_channels_name() {
        let look = Look::DARK;
        let width = crate::sidebar::WIDTH - 1.0;
        let column = Element::column(vec![
            crate::sidebar::channel(
                crate::sidebar::ChannelRow {
                    id: "voice:3".into(),
                    name: "Sala".into(),
                    voice: true,
                    ..Default::default()
                },
                &look,
            ),
            participant(&person(9, false, false, false), None, None, &look),
        ]);
        let (_, laid) = frame(column, Size::new(width, 200.0));
        let row = laid
            .regions
            .iter()
            .find(|region| region.id == ids::row(9))
            .unwrap()
            .rect;
        assert_eq!(row.height, ROW_HEIGHT);
        let picture = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::FillRect { rect, color, .. }
                    if *color == Color::rgb(90, 120, 200) =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!((picture.width, picture.height), (24.0, 24.0));
        let name = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::Text { text, position, .. } if text == "Sala" => Some(*position),
                _ => None,
            })
            .unwrap();
        assert_eq!(picture.x, name.x, "the picture sits under the name");
    }

    #[test]
    fn speaking_rings_the_picture_and_moves_nothing() {
        let look = Look::DARK;
        let width = crate::sidebar::WIDTH - 1.0;
        let draw = |speaking: bool| {
            let (_, laid) = frame(
                participant(&person(9, false, false, speaking), None, None, &look),
                Size::new(width, 200.0),
            );
            let rings: Vec<limn::Rect> = laid
                .commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::StrokeRect { rect, color, .. } if *color == look.online => {
                        Some(*rect)
                    }
                    _ => None,
                })
                .collect();
            (laid, rings)
        };
        let (quiet, none) = draw(false);
        let (loud, rings) = draw(true);
        assert!(none.is_empty(), "no green ring at rest: {none:?}");
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert_eq!((rings[0].width, rings[0].height), (ROW_RING, ROW_RING));
        let regions = |laid: &limn::Frame| -> Vec<limn::Rect> {
            laid.regions.iter().map(|region| region.rect).collect()
        };
        assert_eq!(regions(&quiet), regions(&loud), "the same rects either way");
    }

    #[test]
    fn the_pointer_on_someone_live_shows_their_stream_s_card_beside_the_row() {
        use limn::{Point, UiEvent};
        let look = Look::DARK;
        let streamer = Participant {
            streaming: true,
            own: false,
            ..person(2, true, false, false)
        };
        let card = LiveCard {
            user: 2,
            name: "Persona 2".into(),
            stream: Some(LiveStream {
                own: false,
                activity: Some("VALORANT".into()),
                picture: Ok(Thumbnail::Simulated),
                watch: Ok(()),
                watching: false,
            }),
            activities: Vec::new(),
        };
        let size = Size::new(900.0, 400.0);
        let column = |card: Option<&LiveCard>| {
            Element::row(vec![
                // Away from the window's top, which the card keeps clear of.
                Element::column(vec![
                    gap(0.0, 40.0),
                    participant(&streamer, None, card, &look),
                ])
                .size(Some(crate::sidebar::WIDTH - 1.0), None),
                Element::spacer().flex(1.0),
            ])
        };
        let (mut tree, laid) = frame(column(Some(&card)), size);
        assert!(
            !controls(&laid).contains(&ids::live_watch(2)),
            "not until the pointer is on the row"
        );
        let row = laid
            .regions
            .iter()
            .find(|region| region.id == ids::row(2))
            .unwrap()
            .rect;
        tree.handle_event(UiEvent::PointerMove(Point::new(row.x + 20.0, row.y + 10.0)));
        let shown = tree.layout(size);
        assert!(controls(&shown).contains(&ids::live_watch(2)));
        let words = texts(&shown);
        for expected in [
            "Persona 2",
            "Transmitiendo VALORANT",
            "Vista previa simulada",
            "Mirar transmisión",
        ] {
            assert!(
                words.iter().any(|text| text == expected),
                "{expected} in {words:?}"
            );
        }
        // Beside the row, its top with the row's.
        let surface = shown
            .regions
            .iter()
            .find(|region| region.id == format!("popover_surface:{}", ids::live_dismiss(2)))
            .unwrap()
            .rect;
        assert_eq!(surface.x, row.x + row.width);
        assert_eq!(surface.y, row.y);
        assert_eq!(surface.width, LIVE_CARD_WIDTH + 17.0);
        // On their marks too.
        let muted = shown
            .regions
            .iter()
            .find(|region| region.id == format!("{}:muted", ids::row(2)))
            .unwrap()
            .rect;
        tree.handle_event(UiEvent::PointerMove(Point::new(
            muted.x + 4.0,
            muted.y + 4.0,
        )));
        assert!(controls(&tree.layout(size)).contains(&ids::live_watch(2)));
        // The pointer goes over to it and it stays, to be clicked.
        let watch = shown
            .regions
            .iter()
            .find(|region| region.id == ids::live_watch(2))
            .unwrap()
            .rect;
        for point in [
            Point::new(surface.x + 2.0, row.y + 10.0),
            Point::new(watch.x + 4.0, watch.y + 4.0),
        ] {
            tree.handle_event(UiEvent::PointerMove(point));
            assert!(controls(&tree.layout(size)).contains(&ids::live_watch(2)));
        }
        // Away from both, it goes.
        tree.handle_event(UiEvent::PointerMove(Point::new(5.0, 390.0)));
        assert!(!controls(&tree.layout(size)).contains(&ids::live_watch(2)));
        // A click on «Mirar transmisión» reports it.
        tree.handle_event(UiEvent::PointerMove(Point::new(row.x + 20.0, row.y + 10.0)));
        tree.layout(size);
        tree.handle_event(UiEvent::PointerMove(Point::new(
            watch.x + 4.0,
            watch.y + 4.0,
        )));
        tree.layout(size);
        tree.handle_event(UiEvent::PointerDown(Point::new(
            watch.x + 4.0,
            watch.y + 4.0,
        )));
        assert_eq!(
            tree.handle_event(UiEvent::PointerUp(Point::new(watch.x + 4.0, watch.y + 4.0))),
            vec![limn::UiAction::Clicked {
                id: ids::live_watch(2)
            }]
        );
        // Not allowed: disabled, with why; no picture yet: said so.
        let denied = LiveCard {
            stream: Some(LiveStream {
                own: false,
                picture: Err("Cargando la vista previa…".into()),
                watch: Err("Entra al canal de voz para ver la transmisión".into()),
                activity: None,
                watching: false,
            }),
            ..card
        };
        let (_, laid) = frame(live_card(&denied, &look), size);
        assert!(!controls(&laid).contains(&ids::live_watch(2)));
        let words = texts(&laid);
        assert!(words.iter().any(|text| text == "Cargando la vista previa…"));
        // Without a known game, the title says only that it streams.
        assert!(words.iter().any(|text| text == "Transmitiendo"));
    }

    #[test]
    fn a_live_stream_offers_a_green_watch_and_once_watched_a_grey_disabled_one_says_so() {
        let look = Look::ONYX;
        let size = Size::new(480.0, 400.0);
        let card = |watching| LiveCard {
            user: 2,
            name: "Persona 2".into(),
            stream: Some(LiveStream {
                own: false,
                activity: None,
                picture: Ok(Thumbnail::Simulated),
                watch: Ok(()),
                watching,
            }),
            activities: Vec::new(),
        };
        let (_, offered) = frame(live_card(&card(false), &look), size);
        assert!(texts(&offered).contains(&"Mirar transmisión".to_owned()));
        assert!(controls(&offered).contains(&ids::live_watch(2)));
        // Painted in limn's Success fill, Rustcord's green as answering a call.
        let at = offered
            .regions
            .iter()
            .find(|region| region.id == ids::live_watch(2))
            .unwrap()
            .rect;
        assert!(
            fills(&offered, Color::rgb(47, 140, 94)).contains(&at),
            "{at:?}"
        );
        let (_, watched) = frame(live_card(&card(true), &look), size);
        let said = texts(&watched);
        assert!(
            said.contains(&"Viendo una transmisión".to_owned()),
            "{said:?}"
        );
        assert!(!said.contains(&"Mirar transmisión".to_owned()), "{said:?}");
        assert!(!controls(&watched).contains(&ids::live_watch(2)));
        // Under it the grey veil, as the tile's button has.
        assert!(!fills(&watched, Color::rgba(152, 152, 160, 31)).is_empty());
    }

    #[test]
    fn a_voice_card_is_280_wide_with_its_stream_then_each_activity_set_off_by_a_line() {
        let look = Look::DARK;
        let game = ActivityCard {
            kind: ActivityKind::Playing,
            name: "Factorio".into(),
            detail: Some("Fábrica".into()),
            state: None,
            party: Some((2, 4)),
            spotify: None,
            large_text: None,
            image: None,
            small_image: None,
            time: Some(ActivityTime::Elapsed(443)),
            buttons: vec![ActivityButton {
                label: "Ver la fábrica".into(),
                id: "vz:live:activity:2:0:button:0".into(),
                enabled: Ok(()),
            }],
        };
        let track = ActivityCard {
            kind: ActivityKind::Listening,
            name: "Spotify".into(),
            detail: Some("Radio".into()),
            state: Some("Trío Ferroso".into()),
            party: None,
            spotify: Some(SpotifyCard {
                id: "vz:live:activity:2:1:spotify".into(),
                open: Err("Simulado: no abre Spotify".into()),
            }),
            large_text: Some("Noches".into()),
            image: None,
            small_image: None,
            time: Some(ActivityTime::Track {
                at: 253,
                length: 253,
                progress: 1.0,
            }),
            buttons: Vec::new(),
        };
        let card = LiveCard {
            user: 2,
            name: "Persona 2".into(),
            stream: Some(LiveStream {
                own: false,
                activity: Some("VALORANT".into()),
                picture: Ok(Thumbnail::Simulated),
                watch: Ok(()),
                watching: false,
            }),
            activities: vec![game, track],
        };
        let size = Size::new(400.0, 800.0);
        let (_, laid) = frame(live_card(&card, &look), size);
        let shown = texts(&laid);
        for expected in [
            "Transmitiendo VALORANT",
            "Mirar transmisión",
            "Jugando ahora",
            "Ver la fábrica",
            "Escuchando ahora",
            "Escuchar en Spotify",
        ] {
            assert!(shown.iter().any(|text| text == expected), "{expected}");
        }
        // The card is 280 wide, and a line sits above each section after the first.
        let width_of = |color: Color| {
            laid.commands.iter().find_map(|command| match command {
                limn::DrawCommand::FillRect {
                    rect, color: fill, ..
                } if *fill == color => Some(rect.width),
                _ => None,
            })
        };
        assert_eq!(width_of(look.menu), Some(LIVE_CARD_WIDTH));
        let lines = laid
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    limn::DrawCommand::FillRect { rect, color, .. }
                        if *color == look.border && rect.height == 1.0
                )
            })
            .count();
        assert_eq!(lines, 2, "after the stream and the game");
    }

    // Icons over a stream, hidden once the pointer
    // rests, as the official client's.
    #[test]
    fn an_ended_stream_says_so_and_the_tiles_hide_under_a_stream_focused() {
        let over = |user: u64, focused| Preview {
            surface: ids::stream_surface(user),
            title: format!("Pantalla de Persona {user}"),
            frame: None,
            fps: None,
            waiting: "Esta transmisión ha finalizado.".into(),
            stop: (ids::close_stream(user), "Cerrar transmisión"),
            note: "Terminó".into(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: user,
                name: format!("Persona {user}"),
                sound: user + 1_000,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: true,
                idle: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                zoom: None,
                failed: false,
            }),
            focused,
            card: false,
        };
        let live = |user: u64, focused| Preview {
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stream: Some(StreamTile {
                over: false,
                ..over(user, focused).stream.unwrap()
            }),
            ..over(user, focused)
        };
        let view = |watching, tiles_hidden| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        // Over: no note, no badge, and the way out is «Cerrar transmisión».
        let (_, ended) = frame(stage(view(vec![over(2, false)], false), &look), size);
        let said = texts(&ended);
        assert!(said.contains(&"Esta transmisión ha finalizado.".to_owned()));
        assert!(!said.contains(&"Terminó".to_owned()));
        // Its own ID: the call bar's red button keeps `stop_watching`.
        assert!(controls(&ended).contains(&ids::close_stream(2)));
        assert!(said.contains(&"Cerrar transmisión".to_owned()));
        // No picture, and «Cerrar transmisión» is not danger (the bar's red
        // button, under it, is).
        let bar_top = size.height - BAR_BOTTOM - BAR_HEIGHT;
        assert!(!ended.commands.iter().any(|command| matches!(
            command,
            limn::DrawCommand::FillRect { color, rect, .. } | limn::DrawCommand::FillCornerRect { color, rect, .. }
                if *color == look.danger_fill && rect.y < bar_top
        )));
        // Focused, the tiles of the others sit under it, and the pill hides
        // them: the stage takes their room.
        let stage_height = |laid: &limn::Frame| {
            laid.commands
                .iter()
                .find_map(|command| match command {
                    limn::DrawCommand::Surface { id, rect, .. }
                        if *id == ids::stream_surface(2) =>
                    {
                        Some(rect.height)
                    }
                    _ => None,
                })
                .unwrap()
        };
        let (_, shown) = frame(stage(view(vec![live(2, true)], false), &look), size);
        assert!(controls(&shown).contains(&ids::tile(1)));
        assert!(controls(&shown).contains(&ids::TILES.to_owned()));
        let (_, hidden) = frame(stage(view(vec![live(2, true)], true), &look), size);
        assert!(!controls(&hidden).contains(&ids::tile(1)));
        assert!(controls(&hidden).contains(&ids::TILES.to_owned()));
        assert!(stage_height(&hidden) > stage_height(&shown));
        // Nothing to hide with a single stream: no pill.
        let (_, alone) = frame(
            stage(
                Stage {
                    people: Vec::new(),
                    ..view(vec![live(2, true)], false)
                },
                &look,
            ),
            size,
        );
        assert!(!controls(&alone).contains(&ids::TILES.to_owned()));
    }

    #[test]
    fn a_person_with_camera_on_shows_the_camera_mark_and_one_off_does_not() {
        let look = Look::ONYX;
        let shown = |camera_on| {
            let row = Participant {
                camera_on,
                ..person(1, false, false, false)
            };
            let (_, laid) = frame(participant(&row, None, None, &look), Size::new(300.0, 40.0));
            laid.commands.iter().any(|command| {
                matches!(
                    command,
                    limn::DrawCommand::Icon {
                        icon: Icon::Video,
                        ..
                    }
                )
            })
        };
        assert!(shown(true));
        assert!(!shown(false));
    }

    #[test]
    fn the_own_stream_card_says_it_is_live_and_its_click_and_stop_are_named() {
        let card = Preview {
            surface: ids::OWN_THUMB_SURFACE.into(),
            title: "¡Estás en directo!".into(),
            frame: Some((1920, 1080)),
            fps: None,
            waiting: "Esperando al sistema…".into(),
            stop: (ids::OWN_STREAM_STOP.into(), "Dejar de transmitir"),
            note: "Los demás ven tu pantalla.".into(),
            actions: Vec::new(),
            stream: None,
            focused: false,
            card: true,
        };
        let look = Look::ONYX;
        let size = Size::new(480.0, 270.0);
        let (_, laid) = frame(preview(&card, size, &look), size);
        let shown = controls(&laid);
        assert!(shown.contains(&ids::OWN_STREAM.to_owned()), "{shown:?}");
        assert!(
            shown.contains(&ids::OWN_STREAM_STOP.to_owned()),
            "{shown:?}"
        );
        let painted = texts(&laid);
        assert!(
            painted.contains(&"¡Estás en directo!".to_owned()),
            "{painted:?}"
        );
        assert!(painted.contains(&"Los demás ven tu pantalla.".to_owned()));
        // Without a frame yet, the card still stands with its button.
        let waiting = Preview {
            frame: None,
            ..card
        };
        let (_, laid) = frame(preview(&waiting, size, &look), size);
        assert!(controls(&laid).contains(&ids::OWN_STREAM_STOP.to_owned()));
    }

    #[test]
    fn over_a_stream_focused_its_controls_are_named_icons_that_rest_hides() {
        let stream = |idle: bool, fullscreen: bool| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: "En vivo · simulada".into(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen,
                over: false,
                idle,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                zoom: None,
                failed: false,
            }),
            focused: !fullscreen,
            card: false,
        };
        let look = Look::ONYX;
        let size = Size::new(960.0, 540.0);
        // Over the stage its sound, window and full screen are on the bar: here
        // only «⋯», named and an icon.
        let (_, laid) = frame(preview(&stream(false, false), size, &look), size);
        let shown = controls(&laid);
        assert!(!shown.contains(&ids::fullscreen(2)) && !shown.contains(&ids::window(2)));
        assert!(!shown.contains(&ids::local_mute(1_002)));
        let id = ids::stream_more(2);
        assert!(shown.contains(&id), "{id}");
        let node = laid
            .access
            .iter()
            .find(|node| node.id == id)
            .unwrap_or_else(|| panic!("{id} has no node"));
        assert_eq!(node.role, limn::Role::Button);
        assert!(!node.name.is_empty(), "{id} is named");
        // No words to press: what is written is its streamer's name and the
        // badge that says it is simulated.
        let mut said = texts(&laid);
        said.sort();
        assert_eq!(said, ["En vivo · simulada", "Persona 2"]);
        // Full screen: the way back in place of full screen, an icon too.
        let (_, laid) = frame(preview(&stream(false, true), size, &look), size);
        let shown = controls(&laid);
        assert!(shown.contains(&ids::FULLSCREEN_EXIT.to_owned()));
        assert!(!shown.contains(&ids::fullscreen(2)));
        assert!(!texts(&laid).contains(&"Salir de pantalla completa".to_owned()));
        // At rest: the picture alone, and no pointer over it.
        let (mut tree, laid) = frame(preview(&stream(true, false), size, &look), size);
        assert_eq!(controls(&laid), [ids::stream_tile(2)]);
        assert!(texts(&laid).is_empty());
        tree.handle_event(limn::UiEvent::PointerMove(limn::Point::new(480.0, 270.0)));
        assert_eq!(tree.cursor(), Some(limn::Cursor::Hidden));
        // The account's own screen: pausing and stopping are icons too.
        let own = Preview {
            surface: ids::SCREEN_SURFACE.into(),
            title: "Tu pantalla".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando al sistema…".into(),
            stop: (ids::STOP_SCREEN.into(), "Dejar de compartir"),
            note: "Solo tú lo ves".into(),
            actions: vec![(ids::PAUSE.into(), "Pausar")],
            stream: None,
            focused: false,
            card: false,
        };
        let (_, laid) = frame(preview(&own, size, &look), size);
        let shown = controls(&laid);
        assert!(
            shown.contains(&ids::STOP_SCREEN.to_owned()) && shown.contains(&ids::PAUSE.to_owned())
        );
        let said = texts(&laid);
        assert!(
            !said.contains(&"Dejar de compartir".to_owned())
                && !said.contains(&"Pausar".to_owned())
        );
    }

    #[test]
    fn a_stream_focused_at_rest_takes_the_bar_with_it_and_the_grid_never_rests() {
        let stream = |focused: bool, idle: bool| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: "En vivo".into(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                zoom: None,
                failed: false,
            }),
            focused,
            card: false,
        };
        let view = |watching| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size: Size::new(980.0, 762.0),
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let (_, moving) = frame(stage(view(vec![stream(true, false)]), &look), size);
        assert!(controls(&moving).contains(&ids::MUTE.to_owned()));
        let (_, rest) = frame(stage(view(vec![stream(true, true)]), &look), size);
        let shown = controls(&rest);
        assert!(!shown.contains(&ids::MUTE.to_owned()) && !shown.contains(&ids::stop_watching(2)));
        assert!(shown.contains(&ids::tile(1)), "the strip stays");
        // In the grid nothing rests: the bar is there however long it stays.
        let (_, grid) = frame(stage(view(vec![stream(false, true)]), &look), size);
        assert!(controls(&grid).contains(&ids::MUTE.to_owned()));
    }

    #[test]
    fn the_rate_shows_in_the_header_badge_and_nothing_over_the_picture() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let stream = |fps: Option<u32>, focused: bool| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                zoom: None,
                failed: false,
            }),
            focused,
            card: false,
        };
        let stage_with = |watching: Vec<Preview>| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![person(1, false, false, false)],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        // Focused with its rate: the header says «720p 30 FPS», and «720p»
        // until the rate is known; a live stream writes nothing over its picture.
        let (_, laid) = frame(stage(stage_with(vec![stream(Some(30), true)]), &look), size);
        let said = texts(&laid);
        assert!(said.iter().any(|text| text == "720p 30 FPS"), "{said:?}");
        assert!(!said.iter().any(|text| text.starts_with("Resolución")));
        assert!(!said.iter().any(|text| text == "En vivo"), "{said:?}");
        let (_, laid) = frame(stage(stage_with(vec![stream(None, true)]), &look), size);
        let said = texts(&laid);
        assert!(said.iter().any(|text| text == "720p"), "{said:?}");
        // The account's own share: the rate it sends.
        let own = Preview {
            surface: ids::SCREEN_SURFACE.into(),
            title: "Tu pantalla".into(),
            frame: Some((1280, 720)),
            fps: Some(30),
            waiting: "Esperando al sistema…".into(),
            stop: (ids::STOP_SCREEN.into(), "Dejar de compartir"),
            note: "En vivo · 720p · 30 fps · 2500 kbit/s".into(),
            actions: Vec::new(),
            stream: None,
            focused: true,
            card: false,
        };
        let (_, laid) = frame(stage(stage_with(vec![own]), &look), size);
        let said = texts(&laid);
        assert!(said.iter().any(|text| text == "720p 30 FPS"), "{said:?}");
        assert!(!said.iter().any(|text| text.starts_with("Bitrate")));
    }

    #[test]
    fn the_header_names_the_streamer_with_their_game_or_their_screen() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let focused = |activity: Option<&str>, degraded: bool| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: Some(30),
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: activity.map(str::to_owned),
                degraded,
                viewers: Vec::new(),
                name: "Persona 2".into(),
                zoom: None,
                failed: false,
            }),
            focused: true,
            card: false,
        };
        let stage_with = |watching: Vec<Preview>| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        // With a game: the header names it; the picture's pill names the streamer.
        let (_, laid) = frame(
            stage(stage_with(vec![focused(Some("Juego"), false)]), &look),
            size,
        );
        let said = texts(&laid);
        assert!(
            said.iter()
                .any(|text| text == "Persona 2 está jugando a Juego"),
            "{said:?}"
        );
        assert!(!said.iter().any(|text| text == "Pantalla de Persona 2"));
        assert!(said.iter().any(|text| text == "Persona 2"), "{said:?}");
        // Without one: the header says «Pantalla de» them; the pill, their name.
        let (_, laid) = frame(stage(stage_with(vec![focused(None, false)]), &look), size);
        let said = texts(&laid);
        assert_eq!(
            said.iter()
                .filter(|text| *text == "Pantalla de Persona 2")
                .count(),
            1
        );
        assert!(said.iter().any(|text| text == "Persona 2"), "{said:?}");
        // The warning is there only while the picture is reduced.
        let warned =
            |laid: &limn::Frame| laid.regions.iter().any(|region| region.id == ids::WARNING);
        assert!(!warned(&laid));
        let (_, laid) = frame(stage(stage_with(vec![focused(None, true)]), &look), size);
        assert!(warned(&laid));
    }

    #[test]
    fn the_quality_and_live_make_one_pill_and_the_quality_waits_for_its_frame() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let quality_grey = Color::rgb(0x4E, 0x50, 0x58);
        let live_red = Color::rgb(0xD2, 0x2D, 0x39);
        // The segments have their own corners, so they are drawn with them;
        // in the header, over the bar's red button.
        let painted = |laid: &limn::Frame, color: Color| -> Vec<limn::Rect> {
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::FillRect {
                        rect, color: ink, ..
                    }
                    | limn::DrawCommand::FillCornerRect {
                        rect, color: ink, ..
                    } if *ink == color && rect.y < crate::sidebar::HEADER => Some(*rect),
                    _ => None,
                })
                .collect()
        };
        let stream = |frame: Option<(u32, u32)>, fps: Option<u32>| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame,
            fps,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                name: "Persona 2".into(),
                zoom: None,
                failed: false,
            }),
            focused: true,
            card: false,
        };
        let stage_with = |watching: Vec<Preview>| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        // With the quality known, two touching segments: the quality, then «EN VIVO».
        let (_, laid) = frame(
            stage(stage_with(vec![stream(Some((1280, 720)), Some(30))]), &look),
            size,
        );
        let said = texts(&laid);
        assert!(said.iter().any(|text| text == "720p 30 FPS"), "{said:?}");
        assert!(said.iter().any(|text| text == "EN VIVO"), "{said:?}");
        let quality = painted(&laid, quality_grey);
        let live = painted(&laid, live_red);
        assert_eq!((quality.len(), live.len()), (1, 1));
        assert!(
            (quality[0].x + quality[0].width - live[0].x).abs() < 0.5,
            "they touch"
        );
        // Without the rate, only the resolution; without the frame, no quality at all.
        let (_, laid) = frame(
            stage(stage_with(vec![stream(Some((1280, 720)), None)]), &look),
            size,
        );
        assert!(texts(&laid).iter().any(|text| text == "720p"));
        let (_, laid) = frame(stage(stage_with(vec![stream(None, None)]), &look), size);
        let said = texts(&laid);
        assert!(!said.iter().any(|text| text.contains("720p")), "{said:?}");
        assert!(said.iter().any(|text| text == "EN VIVO"), "{said:?}");
        assert!(painted(&laid, quality_grey).is_empty());
    }

    #[test]
    fn the_bar_invites_only_where_the_stage_is_wide_enough_and_the_account_may() {
        let look = Look::ONYX;
        let laid = |invite: bool, width: f32| {
            let view = Stage {
                channel: "General".into(),
                phase: Some(CallPhase::Connected),
                people: vec![person(1, false, false, false)],
                muted: false,
                simulated: false,
                join_denied: None,
                chat: Err("Sin chat".into()),
                size: Size::new(width, 762.0),
                card: None,
                tile_menu: None,
                more_open: false,
                video: VideoControls::default(),
                previews: Vec::new(),
                watch: Ok(()),
                watching: Vec::new(),
                tiles_hidden: false,
                private: None,
                quick: None,
                camera_panel: None,
                end: 0.0,
                invite,
                idle: false,
            };
            frame(stage(view, &look), Size::new(width, 762.0)).1
        };
        let wide = laid(true, 980.0);
        assert!(controls(&wide).contains(&ids::INVITE.to_owned()));
        assert!(!controls(&laid(true, 640.0)).contains(&ids::INVITE.to_owned()));
        assert!(!controls(&laid(false, 980.0)).contains(&ids::INVITE.to_owned()));
        // The groups stay where they are, with the invitation or without it.
        let mute = |laid: &limn::Frame| {
            laid.regions
                .iter()
                .find(|region| region.id == ids::MUTE)
                .unwrap()
                .rect
                .x
        };
        assert_eq!(mute(&wide), mute(&laid(false, 980.0)));
    }

    #[test]
    fn the_stream_focused_keeps_its_sound_window_and_full_screen_on_the_bar() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let stream = |focused: bool| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
                name: "Persona 2".into(),
                zoom: None,
                failed: false,
            }),
            focused,
            card: false,
        };
        let stage_with = |watching: Vec<Preview>| Stage {
            channel: "General".into(),
            phase: Some(CallPhase::Connected),
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            muted: false,
            simulated: false,
            join_denied: None,
            chat: Err("Sin chat".into()),
            size,
            card: None,
            tile_menu: None,
            more_open: false,
            video: VideoControls::default(),
            previews: Vec::new(),
            watch: Ok(()),
            watching,
            tiles_hidden: false,
            private: None,
            quick: None,
            camera_panel: None,
            end: 0.0,
            invite: false,
            idle: false,
        };
        let mute = |laid: &limn::Frame| {
            laid.regions
                .iter()
                .find(|region| region.id == ids::MUTE)
                .unwrap()
                .rect
                .x
        };
        // Focused: the bar holds the sound, the window and full screen, once
        // each; the stream's overlay keeps only its «⋯».
        let (_, focused) = frame(stage(stage_with(vec![stream(true)]), &look), size);
        let shown = controls(&focused);
        let count = |id: String| shown.iter().filter(|shown| **shown == id).count();
        assert_eq!(count(ids::local_mute(1_002)), 1);
        assert_eq!(count(ids::window(2)), 1);
        assert_eq!(count(ids::fullscreen(2)), 1);
        assert_eq!(count(ids::stream_more(2)), 1);
        // Not focused, nothing of that is on the bar, and the groups did not move.
        let (_, grid) = frame(stage(stage_with(vec![stream(false)]), &look), size);
        let shown = controls(&grid);
        assert!(!shown.contains(&ids::fullscreen(2)) && !shown.contains(&ids::window(2)));
        assert_eq!(mute(&focused), mute(&grid));
        // Shown alone in full screen there is no bar: the way out stays over it.
        let alone = Preview {
            stream: Some(StreamTile {
                fullscreen: true,
                ..stream(false).stream.unwrap()
            }),
            ..stream(false)
        };
        let (_, laid) = frame(preview(&alone, size, &look), size);
        let shown = controls(&laid);
        assert!(shown.contains(&ids::FULLSCREEN_EXIT.to_owned()));
        assert!(shown.contains(&ids::window(2)));
    }

    #[test]
    fn the_header_shows_who_watches_as_pictures_and_past_five_a_count() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let stream = |viewers: Vec<Participant>| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: Some(30),
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                activity: None,
                degraded: false,
                viewers,
                name: "Persona 2".into(),
                zoom: None,
                failed: false,
            }),
            focused: true,
            card: false,
        };
        let laid = |viewers: Vec<Participant>| {
            let view = Stage {
                channel: "General".into(),
                phase: Some(CallPhase::Connected),
                people: vec![
                    person(1, false, false, false),
                    person(2, false, false, false),
                ],
                muted: false,
                simulated: false,
                join_denied: None,
                chat: Err("Sin chat".into()),
                size,
                card: None,
                tile_menu: None,
                more_open: false,
                video: VideoControls::default(),
                previews: Vec::new(),
                watch: Ok(()),
                watching: vec![stream(viewers)],
                tiles_hidden: false,
                private: None,
                quick: None,
                camera_panel: None,
                end: 0.0,
                invite: false,
                idle: false,
            };
            frame(stage(view, &look), size).1
        };
        let names = ["Ana", "Beto", "Cami", "Dani", "Eva", "Fede", "Gus"];
        let people = |count: usize| -> Vec<Participant> {
            names[..count]
                .iter()
                .enumerate()
                .map(|(index, name)| Participant {
                    name: (*name).into(),
                    ..person(10 + index as u64, false, false, false)
                })
                .collect()
        };
        // The pictures of the stack, each a node named for its viewer.
        let pictured = |laid: &limn::Frame| {
            laid.access
                .iter()
                .filter(|node| node.id.starts_with("vz:viewer:2:"))
                .count()
        };
        let stack = ids::viewers(2);
        let heading = |laid: &limn::Frame| {
            laid.access
                .iter()
                .find(|node| node.id == stack)
                .map(|node| node.name.clone())
        };
        // Three: three pictures, and the heading names how many.
        let three = laid(people(3));
        assert_eq!(pictured(&three), 3);
        assert_eq!(heading(&three).as_deref(), Some("Espectadores: 3"));
        assert!(!texts(&three).iter().any(|text| text.starts_with('+')));
        // Seven: four pictures and «+3» for the rest.
        let seven = laid(people(7));
        assert_eq!(pictured(&seven), 4);
        assert!(
            texts(&seven).contains(&"+3".to_owned()),
            "{:?}",
            texts(&seven)
        );
        assert_eq!(heading(&seven).as_deref(), Some("Espectadores: 7"));
        // None: nothing, not even the stack.
        let none = laid(Vec::new());
        assert_eq!(pictured(&none), 0);
        assert!(heading(&none).is_none());
        // The list their hover opens: its heading, then each one by name.
        let card = frame(viewers_card(&people(3), &look), Size::new(200.0, 400.0)).1;
        let said = texts(&card);
        assert!(said.contains(&"Espectadores: 3".to_owned()), "{said:?}");
        assert!(said.contains(&"Beto".to_owned()));
    }

    /// Someone called `name`, for the VOICE row.
    #[test]
    fn a_stream_focused_names_its_streamer_in_the_pill_and_not_the_screen() {
        let watched = Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                zoom: None,
                failed: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
            }),
            focused: true,
            card: false,
        };
        let view = Stage {
            people: vec![person(1, false, false, false)],
            watching: vec![watched],
            ..picker_stage(VideoControls::default())
        };
        let (_, laid) = frame(stage(view, &Look::ONYX), Size::new(980.0, 762.0));
        // Only the pill says the name alone: the strip shows the other person.
        assert!(texts(&laid).contains(&"Persona 2".to_owned()));
    }

    #[test]
    fn a_zoom_keeps_the_point_under_the_pointer_and_never_bares_the_tile() {
        let room = Size::new(400.0, 225.0);
        let picture = fitted((1600, 900), room);
        assert_eq!((picture.width, picture.height), (400.0, 225.0));
        // Twice as large, about a point right of the middle: the picture point
        // under it stays under it, so the middle moves left by as much.
        let twice = zoom_to(Zoom::FIT, 2.0, (100.0, 0.0), (0.0, 0.0), picture, room);
        assert_eq!((twice.level, twice.pan), (2.0, (-100.0, 0.0)));
        // Panned past an edge, it stops where the picture meets the tile's.
        let edge = zoom_to(twice, 2.0, (0.0, 0.0), (1_000.0, 0.0), picture, room);
        assert_eq!(edge.pan, (200.0, 0.0));
        // The levels stop at 1× and 5×, and at the fit there is nothing to pan.
        assert_eq!(
            zoom_to(Zoom::FIT, 9.0, (0.0, 0.0), (0.0, 0.0), picture, room).level,
            ZOOM_MAX
        );
        assert_eq!(
            zoom_to(twice, 0.5, (0.0, 0.0), (0.0, 0.0), picture, room).level,
            ZOOM_MIN
        );
        let fit = zoom_to(Zoom::FIT, 1.0, (50.0, 0.0), (30.0, 0.0), picture, room);
        assert_eq!(fit.pan, (0.0, 0.0));
    }

    #[test]
    fn a_focused_stream_zooms_by_its_buttons_and_says_the_level() {
        let look = Look::ONYX;
        let size = Size::new(640.0, 360.0);
        let stream = |zoom: Option<Zoom>| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1600, 900)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                zoom,
                failed: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
            }),
            focused: true,
            card: false,
        };
        let shown = |zoom| {
            let (_, laid) = frame(preview(&stream(zoom), size, &look), size);
            (controls(&laid), texts(&laid), laid)
        };
        // At the fit, only «Acercar»; the picture fills the tile.
        let (fit, _, _) = shown(Some(Zoom::FIT));
        assert!(fit.contains(&ids::zoom_in(2)) && !fit.contains(&ids::zoom_out(2)));
        // Zoomed, «Alejar» and the level too.
        let (zoomed, said, _) = shown(Some(Zoom {
            level: 2.0,
            pan: (0.0, 0.0),
        }));
        assert!(zoomed.contains(&ids::zoom_out(2)));
        assert!(said.contains(&"200 %".to_owned()), "{said:?}");
        // Not a stream that does not take the stage: no zoom.
        let (none, _, _) = shown(None);
        assert!(!none.contains(&ids::zoom_in(2)));
    }

    #[test]
    fn the_strip_pill_sits_on_the_strips_top_edge_and_says_what_it_does() {
        let watched = Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: Some((1280, 720)),
            fps: None,
            waiting: "Esperando video…".into(),
            stop: (ids::stop_watching(2), "Dejar de ver"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: false,
                idle: false,
                zoom: Some(Zoom::FIT),
                failed: false,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
            }),
            focused: true,
            card: false,
        };
        let size = Size::new(980.0, 762.0);
        let at = |tiles_hidden: bool| {
            let view = Stage {
                people: vec![person(1, false, false, false)],
                watching: vec![watched.clone()],
                tiles_hidden,
                ..picker_stage(VideoControls::default())
            };
            let (_, laid) = frame(stage(view, &Look::ONYX), size);
            laid
        };
        // Shown: the pill sits 8 under the strip's top edge, as Discord's.
        let shown = at(false);
        let region = |laid: &limn::Frame, id: &str| {
            laid.regions
                .iter()
                .rev()
                .find(|region| region.id == id)
                .map(|region| region.rect)
        };
        let pill = region(&shown, ids::TILES).expect("the pill is laid out");
        let strip = region(&shown, &ids::tile(1)).expect("the strip shows the other person");
        assert!((pill.y - strip.y - 8.0).abs() <= 1.0, "{pill:?} {strip:?}");
        assert_eq!((pill.width, pill.height), (60.0, 32.0));
        let named = |laid: &limn::Frame| {
            laid.access
                .iter()
                .find(|node| node.id == ids::TILES)
                .map(|node| node.name.clone())
        };
        assert_eq!(named(&shown).as_deref(), Some("Ocultar miembros"));
        // Hidden: no strip, and the pill says it shows them again.
        let hidden = at(true);
        assert!(region(&hidden, &ids::tile(1)).is_none());
        assert_eq!(named(&hidden).as_deref(), Some("Mostrar miembros"));
    }

    #[test]
    fn a_stream_nobody_watches_adds_an_icon_to_add_it_only_with_another_watched() {
        let look = Look::ONYX;
        let streamer = Participant {
            streaming: true,
            own: false,
            ..person(2, false, false, false)
        };
        let size = Size::new(200.0, 112.5);
        let alone = frame(watch_tile(&streamer, size, &Ok(()), false, &look), size).1;
        assert!(!controls(&alone).contains(&ids::watch_add(2)));
        let beside = frame(watch_tile(&streamer, size, &Ok(()), true, &look), size).1;
        assert!(controls(&beside).contains(&ids::watch(2)));
        assert!(controls(&beside).contains(&ids::watch_add(2)));
        // Under 175 px the primary says «Ver».
        let narrow = Size::new(120.0, 67.5);
        let said = texts(&frame(watch_tile(&streamer, narrow, &Ok(()), true, &look), narrow).1);
        assert!(said.contains(&"Ver".to_owned()), "{said:?}");
    }

    #[test]
    fn a_stream_that_failed_says_why_and_offers_the_help_and_closing_only() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let over = |failed: bool, waiting: &str| Preview {
            surface: ids::stream_surface(2),
            title: "Pantalla de Persona 2".into(),
            frame: None,
            fps: None,
            waiting: waiting.into(),
            stop: (ids::close_stream(2), "Cerrar transmisión"),
            note: String::new(),
            actions: Vec::new(),
            stream: Some(StreamTile {
                owner: 2,
                name: "Persona 2".into(),
                sound: 1_002,
                silenced: false,
                windowed: false,
                menu: None,
                fullscreen: false,
                over: true,
                failed,
                idle: false,
                zoom: None,
                activity: None,
                degraded: false,
                viewers: Vec::new(),
            }),
            focused: true,
            card: false,
        };
        let shown = |view: Preview| {
            let (_, laid) = frame(preview(&view, size, &look), size);
            (texts(&laid), controls(&laid))
        };
        // A failure: its title, the reason, the line to the help and one way out.
        let (said, ids_shown) = shown(over(true, "Discord no respondió a tiempo."));
        assert!(said.contains(&"No se pudo iniciar la transmisión :(".to_owned()));
        assert!(said.contains(&"Discord no respondió a tiempo.".to_owned()));
        assert!(said.contains(&"¿Tienes problemas con tu transmisión?".to_owned()));
        assert!(said.contains(&"Déjanos ayudarte.".to_owned()));
        assert!(ids_shown.contains(&ids::STREAM_HELP.to_owned()));
        assert!(ids_shown.contains(&ids::close_stream(2)));
        assert!(!ids_shown.iter().any(|id| id.contains("retry")));
        // An ended stream: its wording is the title, and no help is offered.
        let (ended, ended_ids) = shown(over(false, "Esta transmisión ha finalizado."));
        assert!(ended.contains(&"Esta transmisión ha finalizado.".to_owned()));
        assert!(!ended.contains(&"Déjanos ayudarte.".to_owned()));
        assert!(!ended_ids.contains(&ids::STREAM_HELP.to_owned()));
        assert!(ended_ids.contains(&ids::close_stream(2)));
    }

    fn named(name: &str, user: u64) -> Participant {
        Participant {
            name: name.into(),
            ..person(user, false, false, false)
        }
    }

    /// The VOICE row of `people` (each with whether they are being rung), `width` wide.
    fn voice_row(people: &[(&Participant, bool)], width: f32, look: &Look) -> limn::Frame {
        frame(avatar_row(people, width, look), Size::new(width, 200.0)).1
    }

    fn avatar_box(laid: &limn::Frame, user: u64) -> limn::Rect {
        laid.regions
            .iter()
            .find(|region| region.id == ids::avatar(user))
            .unwrap()
            .rect
    }

    /// The rectangles filled in `color`.
    fn fills(laid: &limn::Frame, color: Color) -> Vec<limn::Rect> {
        laid.commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect {
                    rect, color: ink, ..
                } if *ink == color => Some(*rect),
                _ => None,
            })
            .collect()
    }

    /// The rectangles stroked in `color`, `width` thick.
    fn strokes(laid: &limn::Frame, color: Color, width: f32) -> Vec<limn::Rect> {
        laid.commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::StrokeRect {
                    rect,
                    color: ink,
                    width: thick,
                    ..
                } if *ink == color && *thick == width => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn icon_boxes(laid: &limn::Frame, wanted: Icon) -> Vec<limn::Rect> {
        laid.commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::Icon { rect, icon, .. } if *icon == wanted => Some(*rect),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn voice_avatars_are_80_across_while_the_row_fits_and_40_once_not() {
        let look = Look::DARK;
        let (ana, bruno, carla) = (named("Ana", 1), named("Bruno", 2), named("Carla", 3));
        let people = [(&ana, false), (&bruno, false), (&carla, false)];
        // Three 96 px cells fit in 400; in 200 they take 40 px faces, 56 apart.
        let wide = voice_row(&people, 400.0, &look);
        assert_eq!(avatar_box(&wide, 1).width, 96.0);
        let faces = fills(&wide, Color::rgb(90, 120, 200));
        assert_eq!(faces.len(), 3);
        assert!(faces.iter().all(|face| face.width == 80.0), "{faces:?}");
        let narrow = voice_row(&people, 200.0, &look);
        assert_eq!(avatar_box(&narrow, 1).width, 56.0);
        let faces = fills(&narrow, Color::rgb(90, 120, 200));
        assert!(faces.iter().all(|face| face.width == 40.0), "{faces:?}");
    }

    #[test]
    fn voice_avatars_run_in_name_order_and_centre_as_one_row() {
        let look = Look::DARK;
        let (zoe, ana) = (named("Zoe", 1), named("Ana", 2));
        let laid = voice_row(&[(&zoe, false), (&ana, false)], 400.0, &look);
        // Two 96 px cells centred in 400 start at 104; Ana comes first by name.
        assert_eq!(avatar_box(&laid, 2).x, 104.0);
        assert_eq!(avatar_box(&laid, 1).x, 200.0);
        assert_eq!(avatar_box(&laid, 2).y, avatar_box(&laid, 1).y, "one row");
    }

    #[test]
    fn a_row_too_wide_for_its_stage_overflows_both_ends_and_never_wraps() {
        let look = Look::DARK;
        let names: Vec<Participant> = (1..=9)
            .map(|user| named(&format!("Persona {user:02}"), user))
            .collect();
        let people: Vec<(&Participant, bool)> =
            names.iter().map(|person| (person, false)).collect();
        let laid = voice_row(&people, 200.0, &look);
        // Nine 56 px cells are 504 wide, centred in 200: the faces start at
        // -144 and step 56. Only those showing are drawn, and the two at the
        // ends run out of the stage, one at each side (-32 and 192).
        let faces = fills(&laid, Color::rgb(90, 120, 200));
        let xs: Vec<f32> = faces.iter().map(|face| face.x).collect();
        assert!(xs.contains(&-32.0) && xs.contains(&192.0), "{xs:?}");
        assert!(
            faces.iter().all(|face| face.y == faces[0].y),
            "one row, no wrap"
        );
    }

    #[test]
    fn a_mute_is_a_red_badge_of_24_on_80_and_12_on_40_and_local_silence_comes_first() {
        let look = Look::DARK;
        let all = Participant {
            muted: true,
            deafened: true,
            silenced: true,
            ..named("Ana", 1)
        };
        let laid = voice_row(&[(&all, false)], 400.0, &look);
        let badges = fills(&laid, look.danger_fill);
        assert_eq!(badges.len(), 1);
        assert_eq!((badges[0].width, badges[0].height), (24.0, 24.0));
        // Local silence is the one mark drawn: not the deafened or the muted.
        let silenced = icon_boxes(&laid, Icon::VolumeMuted);
        assert_eq!(silenced.len(), 1);
        assert_eq!(silenced[0].width, 16.0);
        assert!(icon_boxes(&laid, Icon::Custom(crate::HEADPHONES_OFF)).is_empty());
        assert!(icon_boxes(&laid, Icon::MicrophoneMuted).is_empty());
        // At 40 px: a 12 px badge with a 12 px glyph, deafened before muted.
        let deaf = Participant {
            muted: true,
            deafened: true,
            ..named("Ana", 1)
        };
        let narrow = voice_row(
            &[
                (&deaf, false),
                (&named("Bruno", 2), false),
                (&named("Carla", 3), false),
            ],
            200.0,
            &look,
        );
        let badges = fills(&narrow, look.danger_fill);
        assert_eq!(badges.len(), 1);
        assert_eq!((badges[0].width, badges[0].height), (12.0, 12.0));
        let headphones = icon_boxes(&narrow, Icon::Custom(crate::HEADPHONES_OFF));
        assert_eq!(headphones.len(), 1);
        assert_eq!(headphones[0].width, 12.0);
        assert!(icon_boxes(&narrow, Icon::MicrophoneMuted).is_empty());
    }

    #[test]
    fn a_speaker_is_ringed_in_green_unless_muted_deafened_or_silenced() {
        let look = Look::DARK;
        let ringed = |person: &Participant| {
            strokes(
                &voice_row(&[(person, false)], 400.0, &look),
                look.online,
                2.0,
            )
            .len()
        };
        let speaking = Participant {
            speaking: true,
            ..named("Ana", 1)
        };
        assert_eq!(ringed(&speaking), 1);
        assert_eq!(ringed(&named("Ana", 1)), 0, "nobody speaks at rest");
        let quiet = [
            Participant {
                muted: true,
                ..speaking.clone()
            },
            Participant {
                deafened: true,
                ..speaking.clone()
            },
            Participant {
                silenced: true,
                ..speaking.clone()
            },
            Participant {
                server_muted: true,
                ..speaking.clone()
            },
        ];
        for person in &quiet {
            assert_eq!(ringed(person), 0, "{person:?}");
        }
    }

    #[test]
    fn someone_being_rung_is_dimmed_to_30_percent_and_ringed_outside() {
        let look = Look::DARK;
        let rung = voice_row(&[(&named("Bruno", 2), true)], 400.0, &look);
        // The ground shows through 70 % of the picture, and a ring at 85 % of
        // the text colour stands 3 px outside it: 86 across for an 80 px face.
        assert_eq!(fills(&rung, Look::alpha(look.raised, 178)).len(), 1);
        let ring = strokes(&rung, Look::alpha(look.text, 217), 3.0);
        assert_eq!(ring.len(), 1);
        assert_eq!((ring[0].width, ring[0].height), (86.0, 86.0));
        // In the call, nobody is dimmed.
        let here = voice_row(&[(&named("Bruno", 2), false)], 400.0, &look);
        assert!(fills(&here, Look::alpha(look.raised, 178)).is_empty());
    }

    #[test]
    fn voice_avatars_are_labelled_with_their_name_and_state_and_nothing_else() {
        let look = Look::DARK;
        let label = |person: &Participant, ringing: bool| {
            voice_row(&[(person, ringing)], 400.0, &look)
                .access
                .into_iter()
                .find(|node| node.id == ids::avatar(person.user))
                .unwrap()
                .name
        };
        assert_eq!(label(&named("Ana", 1), false), "Ana");
        let muted = Participant {
            muted: true,
            ..named("Ana", 1)
        };
        assert_eq!(label(&muted, false), "Ana, Silenciado");
        let deaf = Participant {
            deafened: true,
            ..muted.clone()
        };
        assert_eq!(label(&deaf, false), "Ana, Ensordecido");
        let silenced = Participant {
            silenced: true,
            ..deaf
        };
        assert_eq!(label(&silenced, false), "Ana, Silenciado para ti");
        assert_eq!(
            label(&named("Bruno", 2), true),
            "Bruno",
            "rung, not in the call"
        );
    }

    /// Where `id`'s control is in `laid`, if it is.
    fn found(laid: &limn::Frame, id: &str) -> Option<limn::Rect> {
        laid.regions
            .iter()
            .rev()
            .find(|region| region.id == id)
            .map(|region| region.rect)
    }

    /// Whether a fill of `color` covers the top left of `rect`.
    fn filled_at(laid: &limn::Frame, rect: limn::Rect, color: Color) -> bool {
        laid.commands.iter().any(|command| match command {
            limn::DrawCommand::FillRect {
                rect: at,
                color: ink,
                ..
            }
            | limn::DrawCommand::FillCornerRect {
                rect: at,
                color: ink,
                ..
            } => *ink == color && at.x == rect.x && at.y == rect.y,
            _ => false,
        })
    }

    #[test]
    fn at_rest_the_stage_keeps_its_tiles_and_fades_its_header_bar_and_names() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let view = |idle| Stage {
            people: vec![
                person(1, false, false, false),
                person(2, false, false, false),
            ],
            idle,
            ..picker_stage(VideoControls::default())
        };
        let (_, awake) = frame(stage(view(false), &look), size);
        let (_, rest) = frame(stage(view(true), &look), size);
        for id in [ids::MUTE, ids::LEAVE, ids::CAMERA] {
            assert!(
                found(&awake, id).is_some() && found(&rest, id).is_none(),
                "{id}"
            );
        }
        assert!(texts(&awake).contains(&"General".to_owned()));
        let said = texts(&rest);
        assert!(!said.contains(&"General".to_owned()) && !said.contains(&"Persona 1".to_owned()));
        assert_eq!(found(&awake, &ids::tile(1)), found(&rest, &ids::tile(1)));
    }

    #[test]
    fn the_bar_glows_red_for_a_muted_microphone_and_green_for_a_camera_on() {
        let look = Look::ONYX;
        let size = Size::new(980.0, 762.0);
        let view = |muted, camera| Stage {
            muted,
            ..picker_stage(VideoControls {
                camera,
                ..VideoControls::default()
            })
        };
        let (_, plain) = frame(stage(view(false, false), &look), size);
        let (_, lit) = frame(stage(view(true, true), &look), size);
        for (id, glow) in [(ids::MUTE, RED_GLOW.0), (ids::CAMERA, GREEN_GLOW.0)] {
            assert!(!filled_at(&plain, rect_of(&plain, id), glow), "{id}");
            assert!(filled_at(&lit, rect_of(&lit, id), glow), "{id}");
        }
        // Their arrows glow with them.
        let arrow = rect_of(&lit, ids::INPUT_OPTIONS);
        assert!(filled_at(&lit, arrow, RED_GLOW.0));
        let leave = rect_of(&lit, ids::LEAVE);
        assert!(filled_at(&lit, leave, HANG_UP[0]));
    }

    #[test]
    fn a_tile_without_a_picture_sits_on_its_color_a_quarter_darker() {
        let plain = person(1, false, false, false);
        assert_eq!(ground(&plain), Color::rgb(67, 90, 150));
        let picture = limn::RasterImage::new(
            "4".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "x",
        )
        .unwrap();
        let pictured = Participant {
            image: Some(picture),
            ..plain
        };
        assert_eq!(ground(&pictured), pictured.accent);
    }

    #[test]
    fn a_conversation_s_call_fits_faces_or_a_412_by_232_tile_under_its_header() {
        let look = Look::ONYX;
        let call = |voice_mode: bool, height: f32| {
            let size = Size::new(980.0, height);
            let view = Stage {
                people: vec![Participant {
                    camera: (!voice_mode).then_some((1280, 720)),
                    ..person(1, false, false, false)
                }],
                size,
                private: Some(crate::private_call::PrivateStage {
                    voice_mode,
                    ..Default::default()
                }),
                ..picker_stage(VideoControls::default())
            };
            frame(stage(view, &look), size).1
        };
        // Faces: centred in the whole 240, the bar 20 over its bottom.
        let faces = call(true, VOICE_HEIGHT);
        let face = rect_of(&faces, &ids::avatar(1));
        assert_eq!(face.y + face.height / 2.0, VOICE_HEIGHT / 2.0);
        let leave = rect_of(&faces, ids::LEAVE);
        assert_eq!(leave.y + leave.height, VOICE_HEIGHT - BAR_BOTTOM);
        assert!(face.y + face.height <= leave.y, "{face:?} {leave:?}");
        // Tiles: one is 412×232 under the header, the bar over its bottom.
        let tiles = call(false, TILES_HEIGHT);
        let tile = rect_of(&tiles, &ids::tile(1));
        assert_eq!((tile.width, tile.height), (412.0, 232.0));
        assert!(tile.y >= crate::sidebar::HEADER);
        let leave = rect_of(&tiles, ids::LEAVE);
        assert_eq!(leave.y + leave.height, TILES_HEIGHT - BAR_BOTTOM);
        assert_eq!(tile.y + tile.height - leave.y, BAR_OVERLAP);
    }
}
