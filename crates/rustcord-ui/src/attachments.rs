//! The received-image viewer resolves current entities instead of retaining URLs.

use crate::{editor_id, Intent, Ui, ViewContext};
use limn::{Color, Component, Element, ImageViewer, Key, Size, UiAction, UiEvent, ViewerLayout};
use rustcord_components::{
    message::{attachment_ids, Copied},
    Look,
};
use rustcord_i18n::{t, tr};
use rustcord_model::{
    Attachment, AttachmentId, ChannelId, ImageKind, Message, MessageId, RemoteFile, RemoteImage,
    Timestamp, UserId,
};
use rustcord_state::Store;
use std::time::Instant;

/// The viewer's controls, and its IDs' prefix, which the video viewer's «Más» shares.
const PREFIX: &str = "attachment-viewer";
pub(crate) const CLOSE: &str = "attachment-viewer:close";
const ZOOM_IN: &str = "attachment-viewer:zoom-in";
const ZOOM_OUT: &str = "attachment-viewer:zoom-out";
const FIT: &str = "attachment-viewer:fit";
const PREVIOUS: &str = "attachment-viewer:previous";
const NEXT: &str = "attachment-viewer:next";
/// `attachment-viewer:show:{n}`: a thumbnail of the strip, the `n`th picture.
const SHOW: &str = "attachment-viewer:show:";
/// Close the viewer and forward the message the picture is in.
const FORWARD: &str = "attachment-viewer:forward";
/// The «Más» menu's button, and the report of its dismissal. Its choices are
/// `attachment-viewer:more:…`; its «Ver detalles» is a submenu's title, which
/// closes nothing.
const MORE: &str = "attachment-viewer:more";
const MORE_DISMISS: &str = "attachment-viewer:more:dismiss";
const DETAILS: &str = "attachment-viewer:more:details";
/// A preview is decoded to fit this box; a larger picture has more to show.
const PREVIEW_BOX: (u32, u32) = (768, 512);

/// What the viewer shows: a file of a message (or of the copy it forwards),
/// or a picture of its cards.
#[derive(Clone, Debug, PartialEq)]
pub enum Viewed {
    File(AttachmentId),
    /// The `picture`th picture of the cards that start at `first`, of the
    /// message's own cards or of its forward's.
    Card {
        forwarded: bool,
        first: usize,
        picture: usize,
    },
}

/// Account-local presentation state; payloads stay in the bounded image cache.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentViewer {
    pub account: Option<UserId>,
    pub channel: ChannelId,
    pub message: MessageId,
    pub viewed: Viewed,
    /// What the viewer last reported of its zoom: the scale (screen pixels
    /// for each of the picture's) and whether it is fitted. The tree keeps
    /// the zoom; this only shows it in the header.
    pub zoom: Option<(f32, bool)>,
    /// The account asked for the picture at its own size, not its preview's,
    /// or zoomed in past what its preview has.
    pub original: bool,
    /// The «Más» menu is open.
    pub more: bool,
}

/// What the open viewer shows, resolved from the current message.
pub(crate) struct Shown<'a> {
    pub title: String,
    pub preview: &'a RemoteImage,
    /// The picture at its own size, when it is larger than its preview.
    pub original: Option<RemoteImage>,
    /// The file to save, for a file.
    pub file: Option<(&'a Attachment, &'a RemoteFile)>,
    /// Its place among the pictures of its message or its cards.
    pub index: usize,
    pub total: usize,
    /// Its own size in pixels, when the service said it.
    pub dimensions: Option<(u32, u32)>,
    /// A blurred stand-in for it while its preview loads.
    pub placeholder: Option<&'a RemoteImage>,
    /// Every picture of the gallery: its preview and its name.
    pub pictures: Vec<(&'a RemoteImage, String)>,
}

fn message(store: &Store, channel: ChannelId, message: MessageId) -> Option<&Message> {
    store
        .channel(channel)
        .filter(|channel| Store::visible(channel))?;
    store.timeline(channel)?.get(message).map(|found| &**found)
}

pub(crate) fn attachment(
    store: &Store,
    channel: ChannelId,
    message_id: MessageId,
    id: AttachmentId,
) -> Option<&Attachment> {
    message(store, channel, message_id)?
        .files()
        .find(|attachment| attachment.id == id)
}

/// A spoiler the account has not revealed: nothing of it is requested.
pub(crate) fn hidden(ui: &Ui, attachment: &Attachment) -> bool {
    attachment.spoiler && !ui.messages.revealed_files.contains(&attachment.id)
}

/// An explicit picture the account has not revealed: blurred where the
/// account's filter hides it.
pub(crate) fn unrevealed(ui: &Ui, attachment: &Attachment) -> bool {
    attachment.explicit && !ui.messages.revealed_files.contains(&attachment.id)
}

/// Whether the account's filter hides `attachment` of `found` now.
fn obscured(ui: &Ui, store: &Store, found: &Message, attachment: &Attachment) -> bool {
    unrevealed(ui, attachment) && store.obscures_explicit(found.channel, found.author)
}

/// The same picture at its own size, when it is larger than its preview.
fn original(source: &RemoteImage, dimensions: Option<(u32, u32)>) -> Option<RemoteImage> {
    dimensions
        .filter(|(w, h)| *w > PREVIEW_BOX.0 || *h > PREVIEW_BOX.1)
        .map(|_| RemoteImage {
            url: source.url.clone(),
            kind: ImageKind::Original,
        })
}

/// The files of `found` the viewer shows, in order: pictures with a preview
/// that are not hidden behind a spoiler or the account's filter.
fn viewable_files<'a>(ui: &Ui, store: &Store, found: &'a Message) -> Vec<&'a Attachment> {
    found
        .files()
        .filter(|file| {
            file.preview.is_some() && !hidden(ui, file) && !obscured(ui, store, found, file)
        })
        .collect()
}

/// The cards of `found` the viewer may show the pictures of: the message's
/// own, or the copy's it forwards, when their pictures are not hidden.
fn card_pictures<'a>(
    ui: &Ui,
    store: &Store,
    found: &'a Message,
    forwarded: bool,
    first: usize,
) -> Option<Vec<&'a rustcord_model::EmbedImage>> {
    let (embeds, content) = if forwarded {
        let copy = found.forwarded.as_ref()?;
        (&copy.embeds, &copy.content)
    } else {
        (&found.embeds, &found.content)
    };
    let (_, count) = rustcord_model::card_groups(embeds)
        .into_iter()
        .find(|(start, _)| *start == first)?;
    let revealed = ui.messages.revealed.get(&found.id).is_some_and(|shown| {
        shown.contains(&rustcord_components::message::card_spoiler(
            forwarded, first,
        ))
    });
    // Explicit pictures the account's filter hides open once revealed.
    let explicit = rustcord_components::message::card_explicit(embeds, first, count)
        && store.obscures_explicit(found.channel, found.author);
    (revealed || !embeds[first].hides_pictures(content) && !explicit)
        .then(|| rustcord_model::card_pictures(embeds, first, count))
}

pub(crate) fn shown<'a>(ui: &Ui, store: &'a Store) -> Option<Shown<'a>> {
    let viewer = ui.attachment_viewer.as_ref()?;
    (ui.channel() == Some(viewer.channel) && viewer.account == store.me()).then_some(())?;
    match viewer.viewed {
        Viewed::File(id) => {
            let message = message(store, viewer.channel, viewer.message)?;
            let files = viewable_files(ui, store, message);
            let index = files.iter().position(|file| file.id == id)?;
            let found = files[index];
            let preview = found.preview.as_ref()?;
            Some(Shown {
                title: found.filename.clone(),
                preview,
                original: original(preview, found.dimensions),
                file: found.download.as_ref().map(|file| (found, file)),
                index,
                total: files.len(),
                dimensions: found.dimensions,
                placeholder: found.placeholder.as_ref(),
                pictures: files
                    .iter()
                    .filter_map(|file| Some((file.preview.as_ref()?, file.filename.clone())))
                    .collect(),
            })
        }
        Viewed::Card {
            forwarded,
            first,
            picture,
        } => {
            let found = message(store, viewer.channel, viewer.message)?;
            let pictures = card_pictures(ui, store, found, forwarded, first)?;
            let image = pictures.get(picture)?;
            let embeds = if forwarded {
                &found.forwarded.as_ref()?.embeds
            } else {
                &found.embeds
            };
            Some(Shown {
                title: embeds[first]
                    .title
                    .clone()
                    .unwrap_or_else(|| t("Imagen de la tarjeta").into()),
                preview: &image.source,
                original: original(&image.source, Some(image.dimensions)),
                file: None,
                index: picture,
                total: pictures.len(),
                dimensions: Some(image.dimensions),
                placeholder: None,
                pictures: pictures
                    .iter()
                    .enumerate()
                    .map(|(at, picture)| (&picture.source, tr!("Imagen {n}", n = at + 1)))
                    .collect(),
            })
        }
    }
}

/// The source the viewer wants loaded: its preview, and the picture at its
/// own size once the account asked for it; with `around`, also its stand-in
/// while the preview loads and the strip's thumbnails.
pub(crate) fn viewer_sources(ui: &Ui, store: &Store, around: bool) -> Vec<RemoteImage> {
    let Some(shown) = shown(ui, store) else {
        return Vec::new();
    };
    let mut wanted = vec![shown.preview.clone()];
    if around {
        if !ui.images.contains_key(shown.preview) {
            wanted.extend(shown.placeholder.cloned());
        }
        wanted.extend(shown.pictures.iter().map(|(preview, _)| (*preview).clone()));
    }
    if ui
        .attachment_viewer
        .as_ref()
        .is_some_and(|viewer| viewer.original)
    {
        wanted.extend(shown.original);
    }
    wanted
}

/// Deletion, a permission update or navigation cannot leave stale account media open.
pub(crate) fn refresh(ui: &mut Ui, store: &Store) {
    if ui.attachment_viewer.is_some() && shown(ui, store).is_none() {
        ui.attachment_viewer = None;
    }
}

/// The control that opened what the viewer shows: focus goes back to it.
fn opener(viewer: &AttachmentViewer) -> String {
    match viewer.viewed {
        Viewed::File(id) => attachment_ids::action(viewer.message, id, "preview"),
        Viewed::Card {
            forwarded,
            first,
            picture,
        } => rustcord_components::message::card_picture(
            &format!("msg:{}", viewer.message),
            forwarded,
            first,
            picture,
        ),
    }
}

fn close(ui: &mut Ui, store: &Store) -> Vec<Intent> {
    let still = shown(ui, store).is_some();
    let Some(viewer) = ui.attachment_viewer.take() else {
        return Vec::new();
    };
    if ui.channel() == Some(viewer.channel) && ui.prefs.image_previews && still {
        vec![Intent::Focus(opener(&viewer))]
    } else {
        ui.channel()
            .map(|channel| vec![Intent::Focus(editor_id(channel))])
            .unwrap_or_default()
    }
}

/// Open the viewer on `viewed`, closing a document's; returns what that asks.
fn open(
    ui: &mut Ui,
    store: &Store,
    channel: ChannelId,
    message: MessageId,
    viewed: Viewed,
) -> Vec<Intent> {
    let mut intents = crate::documents::close(ui);
    ui.attachment_viewer = Some(AttachmentViewer {
        account: store.me(),
        channel,
        message,
        viewed,
        zoom: None,
        original: false,
        more: false,
    });
    intents.push(Intent::Focus(CLOSE.into()));
    intents
}

/// `card:{forwarded}:{first}:{picture}` of a message: open that picture of
/// its cards, unless it is hidden or previews are off.
pub(crate) fn open_card(
    ui: &mut Ui,
    store: &Store,
    channel: ChannelId,
    id: MessageId,
    what: &str,
) -> Vec<Intent> {
    let mut parts = what.split(':');
    let mut number = || parts.next()?.parse::<usize>().ok();
    let (Some(forwarded), Some(first), Some(picture)) = (number(), number(), number()) else {
        return Vec::new();
    };
    let opened = ui.prefs.image_previews
        && message(store, channel, id)
            .and_then(|found| card_pictures(ui, store, found, forwarded == 1, first))
            .is_some_and(|pictures| picture < pictures.len());
    if !opened {
        return Vec::new();
    }
    open(
        ui,
        store,
        channel,
        id,
        Viewed::Card {
            forwarded: forwarded == 1,
            first,
            picture,
        },
    )
}

/// Show the picture before or after, among its message's or its cards'
/// pictures. The gallery wraps around: past its last picture is its first,
/// and before its first is its last.
fn step(ui: &mut Ui, store: &Store, by: isize) {
    let Some(shown) = shown(ui, store) else {
        return;
    };
    if shown.total < 2 {
        return;
    }
    let next = (shown.index as isize + by).rem_euclid(shown.total as isize) as usize;
    go(ui, store, next);
}

/// Show the `index`th picture of the gallery, fitted and from its preview.
fn go(ui: &mut Ui, store: &Store, index: usize) {
    if shown(ui, store).is_none_or(|shown| index >= shown.total) {
        return;
    }
    let Some(viewer) = ui.attachment_viewer.as_ref() else {
        return;
    };
    let file = match viewer.viewed {
        Viewed::File(_) => message(store, viewer.channel, viewer.message).and_then(|found| {
            viewable_files(ui, store, found)
                .get(index)
                .map(|file| file.id)
        }),
        Viewed::Card { .. } => None,
    };
    let Some(viewer) = ui.attachment_viewer.as_mut() else {
        return;
    };
    match &mut viewer.viewed {
        Viewed::File(id) => match file {
            Some(file) => *id = file,
            None => return,
        },
        Viewed::Card { picture, .. } => *picture = index,
    }
    viewer.zoom = None;
    viewer.original = false;
    viewer.more = false;
}

/// The viewer opens fitted to its box, and the tree reports no zoom until
/// one: when that fit already shows more of the picture than its preview
/// has, its own pixels load at once, as a zoom past the preview asks. The
/// host calls this after layout. The fit it records keeps it from asking
/// again, so «Ver la vista previa» still goes back to the preview.
pub fn follow_viewer(ui: &mut Ui, store: &Store, tree: &limn::UiTree) {
    if ui
        .attachment_viewer
        .as_ref()
        .is_none_or(|viewer| viewer.zoom.is_some() || viewer.original)
    {
        return;
    }
    let Some(shown) = shown(ui, store) else {
        return;
    };
    // An animation plays its preview; its own size is not offered.
    let animated = ui
        .images
        .get(shown.preview)
        .is_some_and(|ready| ui.animated.contains(ready.sha256()));
    let Some((width, height)) = shown
        .dimensions
        .filter(|_| shown.original.is_some() && !animated)
    else {
        return;
    };
    let view = format!("{CLOSE}:view:");
    let Some(room) = tree
        .frame()
        .regions
        .iter()
        .find(|region| region.kind == limn::HitKind::Zoom && region.id.starts_with(&view))
        .map(|region| region.rect)
    else {
        return;
    };
    // The viewer never enlarges a picture past its own size.
    let fit = (room.width / width.max(1) as f32)
        .min(room.height / height.max(1) as f32)
        .min(1.0);
    if fit > preview_share((width, height)) {
        if let Some(viewer) = ui.attachment_viewer.as_mut() {
            viewer.zoom = Some((fit, true));
            viewer.original = true;
        }
    }
}

/// The share of its own size a picture's preview has: what the viewer may
/// enlarge it to before its own pixels are wanted.
fn preview_share((width, height): (u32, u32)) -> f32 {
    let share = |box_side: u32, side: u32| box_side as f32 / side.max(1) as f32;
    share(PREVIEW_BOX.0, width)
        .min(share(PREVIEW_BOX.1, height))
        .min(1.0)
}

pub(crate) fn update(
    ui: &mut Ui,
    store: &mut Store,
    action: &UiAction,
    now: Instant,
) -> Option<Vec<Intent>> {
    if let UiAction::Zoomed { id, scale, fitted } = action {
        if !id.starts_with(CLOSE) {
            return None;
        }
        // An animation plays its preview; its own size is not offered.
        let wanted = shown(ui, store)
            .filter(|shown| shown.original.is_some())
            .filter(|shown| {
                ui.images
                    .get(shown.preview)
                    .is_none_or(|ready| !ui.animated.contains(ready.sha256()))
            })
            .and_then(|shown| shown.dimensions)
            .is_some_and(|dimensions| *scale > preview_share(dimensions));
        if let Some(viewer) = ui.attachment_viewer.as_mut() {
            viewer.zoom = Some((*scale, *fitted));
            // Zoomed past what its preview has: its own pixels load.
            viewer.original |= wanted;
        }
        return Some(Vec::new());
    }
    let UiAction::Clicked { id } = action else {
        return None;
    };
    // The «Más» button toggles its menu; any other choice closes it, except the
    // submenu's title, which only the pointer or the keyboard opens.
    if id == MORE {
        if let Some(viewer) = ui.attachment_viewer.as_mut() {
            viewer.more = !viewer.more;
        }
        return Some(Vec::new());
    }
    if id != DETAILS {
        if let Some(viewer) = ui.attachment_viewer.as_mut() {
            viewer.more = false;
        }
    }
    match id.as_str() {
        CLOSE => return Some(close(ui, store)),
        // The viewer zooms itself; it reports the result as `Zoomed`.
        ZOOM_IN | ZOOM_OUT | FIT | MORE_DISMISS | DETAILS => return Some(Vec::new()),
        PREVIOUS | NEXT => {
            step(ui, store, if id == PREVIOUS { -1 } else { 1 });
            return Some(Vec::new());
        }
        FORWARD => {
            let viewer = ui.attachment_viewer.as_ref()?;
            let (channel, message) = (viewer.channel, viewer.message);
            ui.attachment_viewer = None;
            return crate::message_actions::message_action(
                ui, store, channel, message, "forward", now,
            );
        }
        _ => {}
    }
    if let Some(index) = id.strip_prefix(SHOW) {
        if let Ok(index) = index.parse() {
            go(ui, store, index);
        }
        return Some(Vec::new());
    }
    if id.starts_with(MORE) {
        let showing = shown(ui, store);
        let file = showing
            .as_ref()
            .and_then(|shown| shown.file)
            .map(|(file, _)| file);
        let link = showing
            .as_ref()
            .filter(|shown| shown.file.is_none())
            .map(|shown| shown.preview.url.clone());
        return Some(copy_choice(ui, now, id, file, link).unwrap_or_default());
    }
    let (message, id, verb) = attachment_ids::parse(id)?;
    Some(file_verb(ui, store, message, id, verb))
}

/// What a «Más» choice copies, for either viewer: a card's picture link, or a
/// file's detail, which the viewer then says it copied. Such a choice's ID is
/// `{viewer}:more:{choice}`; any other choice gives `None`.
pub(crate) fn copy_choice(
    ui: &mut Ui,
    now: Instant,
    id: &str,
    file: Option<&Attachment>,
    link: Option<String>,
) -> Option<Vec<Intent>> {
    let (_, choice) = id.split_once(":more:")?;
    if choice == "link" {
        return link.map(|link| vec![Intent::Copy(link)]);
    }
    let key = choice.strip_prefix("detail:")?;
    let (_, _, value) = rustcord_components::message::viewer_details(file?)
        .into_iter()
        .find(|(name, _, _)| *name == key)?;
    ui.notify(t("Copiado al portapapeles."), None, false, now);
    Some(vec![Intent::Copy(value)])
}

/// What a file's action does: `reveal`, `preview`, `save-as`, `copy-image`,
/// `copy-video`, `open`, `copy` and `download`. It resolves the file again.
fn file_verb(
    ui: &mut Ui,
    store: &Store,
    message: MessageId,
    id: AttachmentId,
    verb: &str,
) -> Vec<Intent> {
    let Some(channel) = ui.channel() else {
        return Vec::new();
    };
    let Some(found) = attachment(store, channel, message, id) else {
        return Vec::new();
    };
    // The developer's «Copiar ID del adjunto», as the message's copy-id copies its ID.
    if verb == "copy-id" {
        return vec![Intent::Copy(id.0.to_string())];
    }
    if verb == "reveal" {
        // A spoiler, or an explicit picture.
        if found.spoiler || found.explicit {
            ui.messages.revealed_files.insert(id);
        }
        return Vec::new();
    }
    if verb == "preview" {
        let shown = self::message(store, channel, message)
            .is_some_and(|row| !hidden(ui, found) && !obscured(ui, store, row, found));
        if found.preview.is_some() && ui.prefs.image_previews && shown {
            return open(ui, store, channel, message, Viewed::File(id));
        }
        return Vec::new();
    }
    let Some(source) = &found.download else {
        return Vec::new();
    };
    if verb == "save-as" {
        return crate::save_as::start(ui, store, channel, message, found);
    }
    // The picture or the video itself, not while it is hidden.
    let copied = match verb {
        "copy-image" => Some(Copied::Image),
        "copy-video" => Some(Copied::Video),
        _ => None,
    };
    if let Some(copied) = copied {
        let allowed = rustcord_components::message::copyable(found) == Some(copied)
            && !hidden(ui, found)
            && self::message(store, channel, message)
                .is_some_and(|row| !obscured(ui, store, row, found));
        return allowed
            .then(|| Intent::CopyFile {
                source: source.clone(),
                filename: found.filename.clone(),
                size_bytes: found.size_bytes,
                image: copied == Copied::Image,
            })
            .into_iter()
            .collect();
    }
    match verb {
        "open" => vec![Intent::OpenUrl(source.url.clone())],
        "copy" => vec![Intent::Copy(source.url.clone())],
        "download" => vec![Intent::DownloadFile {
            source: source.clone(),
            filename: found.filename.clone(),
            size_bytes: found.size_bytes,
        }],
        _ => Vec::new(),
    }
}

pub(crate) fn intercept(ui: &mut Ui, store: &Store, event: &UiEvent) -> Option<Vec<Intent>> {
    let viewer = ui.attachment_viewer.as_ref()?;
    let UiEvent::Key { key, ctrl, .. } = event else {
        return None;
    };
    let (message, ctrl, more) = (viewer.message, *ctrl, viewer.more);
    // While «Más» is open, its arrows, Space, Home and End are the menu's: limn moves through it.
    if more
        && matches!(
            key,
            Key::ArrowLeft
                | Key::ArrowUp
                | Key::ArrowRight
                | Key::ArrowDown
                | Key::Space
                | Key::Home
                | Key::End
        )
    {
        return None;
    }
    match key {
        // Esc closes the «Más» menu first, then the viewer.
        Key::Escape if more => {
            if let Some(viewer) = ui.attachment_viewer.as_mut() {
                viewer.more = false;
            }
            return Some(Vec::new());
        }
        Key::Escape => return Some(close(ui, store)),
        // The gallery wraps around: Right, Down and Space go on, Left and Up back.
        Key::ArrowLeft | Key::ArrowUp | Key::ArrowRight | Key::ArrowDown if !ctrl => {
            step(
                ui,
                store,
                if matches!(key, Key::ArrowLeft | Key::ArrowUp) {
                    -1
                } else {
                    1
                },
            );
            return Some(Vec::new());
        }
        Key::Space => {
            step(ui, store, 1);
            return Some(Vec::new());
        }
        Key::Home | Key::End => {
            let last = shown(ui, store).map_or(0, |shown| shown.total.saturating_sub(1));
            go(ui, store, if *key == Key::Home { 0 } else { last });
            return Some(Vec::new());
        }
        // The same as the header's: copy the picture, save it as.
        Key::C | Key::S if ctrl => {
            let file = shown(ui, store)
                .and_then(|shown| shown.file)
                .map(|(file, _)| file.id);
            let verb = if *key == Key::C {
                "copy-image"
            } else {
                "save-as"
            };
            return Some(
                file.map_or_else(Vec::new, |file| file_verb(ui, store, message, file, verb)),
            );
        }
        _ => {}
    }
    // The viewer's own keys zoom, move and turn the picture (limn's zoom
    // view); the modal's focus trap and activation work too. Conversation
    // shortcuts must not edit or switch the workspace behind the picture.
    let zoom = matches!(
        key,
        Key::Plus | Key::Equals | Key::Minus | Key::Num0 | Key::Num1 | Key::Num2 | Key::Num3
    ) || ctrl
        && matches!(
            key,
            Key::R | Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown
        );
    if zoom || matches!(key, Key::Tab | Key::Enter) {
        None
    } else {
        Some(Vec::new())
    }
}

/// The lead of a viewer whose message is `row`: its author's picture, name and
/// when it was written, as the timeline shows them.
pub(crate) fn lead(ui: &Ui, store: &Store, row: &Message, look: &Look, wall: Timestamp) -> Element {
    let server = store.channel(row.channel).and_then(|found| found.server);
    let author = crate::timeline::author(ui, store, server, row.author, look);
    let face = rustcord_components::identity::decorated(
        |side| {
            rustcord_components::avatar_with_image(
                &author.name,
                author.accent,
                side,
                author.image.clone(),
            )
        },
        40.0,
        author.marks.decoration.clone(),
    );
    let when = rustcord_components::message::written_when(row.timestamp, wall);
    rustcord_components::message::viewer_lead(face, &author.name, author.ink, &when, look)
}

pub(crate) fn overlay(
    ui: &Ui,
    store: &Store,
    root: Element,
    look: &Look,
    context: &ViewContext,
) -> Element {
    let Some(viewer) = &ui.attachment_viewer else {
        return root;
    };
    let Some(found) = shown(ui, store) else {
        return root;
    };
    let mut image = ImageViewer::new(root, true, &found.title, CLOSE, look.menu_palette())
        .viewport(context.window)
        .return_focus(opener(viewer))
        .layout(ViewerLayout::Lightbox)
        .scrim(Color::rgba(0, 0, 0, 235))
        .zoom_actions(ZOOM_OUT, FIT, ZOOM_IN)
        .gallery(found.index, found.total)
        .previous_action(PREVIOUS)
        .next_action(NEXT);
    if let Some(row) = message(store, viewer.channel, viewer.message) {
        image = image.lead(lead(ui, store, row, look, context.wall));
    }
    if let Some((width, height)) = found.dimensions {
        image = image.picture_size(Size::new(width as f32, height as f32));
    }
    if let Some((scale, fitted)) = viewer.zoom {
        image = image.zoom_level(scale, fitted);
    }
    let preview = ui.images.get(found.preview).cloned();
    // An animation plays as the timeline plays it; its own size is its
    // preview's, so it is neither offered nor copied.
    let playing = preview
        .as_ref()
        .filter(|ready| ui.animated.contains(ready.sha256()));
    // The same actions as the attachment's menu: they resolve the entity again.
    let file = found.file.map(|(file, _)| file);
    let forward = store
        .abilities(viewer.channel)
        .is_some_and(|abilities| abilities.forward.allowed());
    if forward {
        image = image.action(FORWARD, limn::Icon::Forward, t("Reenviar"));
    }
    if let Some(file) = file {
        let action = |verb| attachment_ids::action(viewer.message, file.id, verb);
        image = image
            .action(
                action("save-as"),
                limn::Icon::ArrowDown,
                t("Guardar imagen"),
            )
            .action(
                action("open"),
                limn::Icon::ExternalLink,
                t("Abrir en el navegador"),
            );
    }
    if found.pictures.len() > 1 {
        for (at, (preview, label)) in found.pictures.iter().enumerate() {
            image = image.thumbnail(
                format!("{SHOW}{at}"),
                ui.images.get(*preview).cloned(),
                label,
            );
        }
    }
    let entries = rustcord_components::message::viewer_entries(
        PREFIX,
        viewer.message,
        file,
        playing.is_none(),
        ui.prefs.developer,
    );
    image = image.tool(rustcord_components::message::viewer_more(
        MORE,
        entries,
        viewer.more,
        MORE_DISMISS,
        look,
    ));
    // The picture at its own size replaces its preview once it is here;
    // the preview shows at that size meanwhile.
    let large = found
        .original
        .as_ref()
        .filter(|_| viewer.original)
        .and_then(|source| ui.images.get(source).cloned());
    if let Some(ready) = large.or(preview.clone()) {
        // Played by the host as it plays the preview in the timeline.
        if let Some(ready) = playing {
            image = image.surface(
                rustcord_components::message::animation_surface(ready.sha256()),
                ready.alt(),
            );
        }
        image = image.image(ready);
    } else if ui.image_failures.contains(found.preview) {
        image = image.error(t("Se produjo un error al cargar la imagen."));
    } else if let Some(stand_in) = found.placeholder.and_then(|source| ui.images.get(source)) {
        image = image.placeholder(stand_in.clone());
    }
    image.render()
}
