//! Local outgoing-file metadata, with no filesystem or transport dependency.
//!
//! The files chosen for a message are Discord's cards: 218 square, 24
//! apart in a row that scrolls sideways, each its picture (or its glyph)
//! over its name, with its remove button in the top corner.

use crate::Look;
use limn::{
    Access, ButtonVariant, Component, ComponentPalette, Element, Icon, IconButton, RasterImage,
    Role, Spinner,
};
use rustcord_i18n::{t, tr};
use rustcord_model::{ChannelId, PendingAttachment};

/// A card's side when the room allows it, and the least it shrinks to.
const CARD: f32 = 218.0;
const SMALLEST: f32 = 64.0;
const CARD_GAP: f32 = 24.0;
/// The name under the media: 8 above a 18 px line.
const NAME: f32 = 26.0;
/// Under the cards, where the row's bar shows once they pass its width.
const BAR: f32 = 6.0;

/// What a file's card shows over its name.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Thumbnail {
    /// Not a picture, or one that could not be decoded: its glyph.
    #[default]
    File,
    /// A picture being decoded: its box's fill alone, as Discord's.
    Loading,
    Picture(RasterImage),
}

pub struct DraftUploads<'a> {
    pub channel: ChannelId,
    pub files: &'a [PendingAttachment],
    /// What each of `files` shows, in its order; one missing shows its glyph.
    pub thumbnails: Vec<Thumbnail>,
    pub picking: Option<u64>,
    pub selecting: bool,
    pub cancelling: bool,
    pub submitting: bool,
    /// The width of the cards' row.
    pub width: f32,
    /// The most the row of cards may be tall: Discord's cards when it
    /// fits, smaller (down to 64) in a short window.
    pub max_list_height: f32,
}

pub fn draft_uploads(view: DraftUploads<'_>, look: &Look) -> Element {
    let side = (view.max_list_height - BAR).clamp(SMALLEST, CARD);
    let mut cards: Vec<Element> = view
        .files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let thumbnail = view.thumbnails.get(index).unwrap_or(&Thumbnail::File);
            card(view.channel, file, thumbnail, side, view.submitting, look)
        })
        .collect();
    // A choice being read: a card of the fill alone until it is here.
    if let Some(request) = view.picking.filter(|_| !view.selecting) {
        let inner = side - 18.0;
        cards.push(
            frame(fill(inner, inner, look), side, look).accessible(
                Access::new(Role::Group)
                    .id(format!("composer:{}:pick:{request}:loading", view.channel))
                    .name(t("Preparando archivos…")),
            ),
        );
    }
    let mut rows = Vec::new();
    if !cards.is_empty() {
        rows.push(
            Element::horizontal_scroll(
                format!("composer:{}:uploads", view.channel),
                Element::row(cards).gap(CARD_GAP),
            )
            .drag_to_pan()
            .size(Some(view.width), Some(side + BAR)),
        );
    }
    if let Some(request) = view.picking {
        rows.push(
            Element::row(vec![
                Spinner::new(
                    format!("composer:{}:pick:{request}:progress", view.channel),
                    look.accent,
                )
                .render()
                .centered_y(),
                crate::line(
                    if view.cancelling && !view.selecting {
                        t("Cancelando preparación…")
                    } else if view.cancelling {
                        t("Cancelando selección…")
                    } else if !view.selecting {
                        t("Preparando archivos…")
                    } else {
                        t("Seleccionando y preparando archivos…")
                    },
                    12.0,
                    look.muted,
                )
                .flex(1.0)
                .centered_y(),
                IconButton::new(
                    format!("composer:{}:pick:{request}:cancel", view.channel),
                    Icon::Close,
                    look.palette(),
                )
                .variant(ButtonVariant::Ghost)
                .tooltip(if view.selecting {
                    t("Cancelar selección y preparación")
                } else {
                    t("Cancelar preparación")
                })
                .enabled(!view.cancelling)
                .render()
                .centered_y(),
            ])
            .gap(8.0),
        );
    }
    if view.submitting {
        rows.push(crate::line(t("Preparando el envío…"), 12.0, look.muted));
    }
    Element::column(rows).gap(8.0).size(Some(view.width), None)
}

/// One file's card: its media centred over its name, its remove button in
/// the top corner, where Discord's bar of actions sits.
fn card(
    channel: ChannelId,
    file: &PendingAttachment,
    thumbnail: &Thumbnail,
    side: f32,
    submitting: bool,
    look: &Look,
) -> Element {
    // 8 of padding and 1 of border on each side.
    let inner = side - 18.0;
    let room = (inner - NAME).max(0.0);
    let media = match thumbnail {
        Thumbnail::Picture(image) => picture(image, inner, room, look),
        Thumbnail::Loading => fill(inner, room, look),
        Thumbnail::File => Element::icon(Icon::File)
            .large_icon(48.0)
            .foreground(look.muted)
            .centered(),
    };
    let body = Element::column(vec![
        Element::column(vec![
            Element::spacer().flex(1.0),
            media,
            Element::spacer().flex(1.0),
        ])
        .size(Some(inner), Some(room)),
        crate::gap(0.0, 8.0),
        crate::line(&file.filename, 14.0, look.text).size(Some(inner), Some(18.0)),
    ]);
    let remove = IconButton::new(
        format!("composer:{}:upload:{}:remove", channel, file.token.0),
        Icon::Trash,
        ComponentPalette {
            text: look.danger,
            ..look.menu_palette()
        },
    )
    .variant(ButtonVariant::Ghost)
    .tooltip(tr!("Quitar {name}", name = file.filename))
    .enabled(!submitting)
    .render();
    let actions = Element::panel(remove)
        .background(look.menu)
        .border(look.border, 1.0)
        .radius(4.0);
    Element::stack(vec![
        frame(body, side, look),
        Element::column(vec![
            Element::row(vec![Element::spacer().flex(1.0), actions]),
            Element::spacer().flex(1.0),
        ])
        .padding(1.0),
    ])
    .size(Some(side), Some(side))
    .accessible(Access::new(Role::Group).name(format!(
        "{}, {}",
        file.filename,
        crate::file_size(file.size_bytes)
    )))
}

/// A card's square: 8 inside a 1 px border, rounded 8, on the cards' fill.
fn frame(body: Element, side: f32, look: &Look) -> Element {
    // limn draws the border inside the padding.
    Element::panel(body)
        .padding(9.0)
        .size(Some(side), Some(side))
        .background(look.card)
        .border(look.border, 1.0)
        .radius(8.0)
}

/// The box a picture sits in: Discord's subtle fill, rounded 8. Alone, it
/// is the picture's place while it is decoded; it does not move.
fn fill(width: f32, height: f32, look: &Look) -> Element {
    Element::panel(Element::spacer())
        .size(Some(width), Some(height))
        .background(look.hover)
        .radius(8.0)
}

/// `image` whole in a box `width` wide and at most `room` tall, as
/// Discord's: the box takes the picture's height, never enlarged, and its
/// fill shows beside a narrow one.
fn picture(image: &RasterImage, width: f32, room: f32, look: &Look) -> Element {
    let (across, down) = crate::identity::png_size(image)
        .map_or((width, room), |(across, down)| (across as f32, down as f32));
    let scale = (width / across).min(room / down).min(1.0);
    let (across, down) = (across * scale, down * scale);
    Element::panel(
        Element::panel(Element::raster_image(image.clone()).size(Some(across), Some(down)))
            .size(Some(across), Some(down))
            .radius(8.0)
            .clip_children()
            .centered_x(),
    )
    .size(Some(width), Some(down))
    .background(look.hover)
    .radius(8.0)
}

pub(crate) fn file_info(file: &PendingAttachment, width: f32, look: &Look) -> Element {
    Element::row(vec![
        crate::icon(Icon::Paperclip, look.muted).centered_y(),
        Element::column(vec![
            crate::line(&file.filename, 13.0, look.text).size(Some((width - 28.0).max(0.0)), None),
            crate::line(crate::file_size(file.size_bytes), 11.0, look.muted),
        ])
        .gap(2.0)
        .flex(1.0),
    ])
    .gap(8.0)
    .size(Some(width), None)
    .accessible(Access::new(Role::Group).name(format!(
        "{}, {}",
        file.filename,
        crate::file_size(file.size_bytes)
    )))
}

pub fn pending_uploads(
    files: &[PendingAttachment],
    progress: Option<(u64, u64)>,
    width: f32,
    look: &Look,
) -> Element {
    let mut rows: Vec<_> = files
        .iter()
        .map(|file| file_info(file, width, look))
        .collect();
    if let Some((sent, total)) = progress {
        let said = if sent == total {
            t("Esperando confirmación…").to_owned()
        } else {
            tr!(
                "Enviando adjuntos: {sent} de {total}",
                sent = crate::file_size(sent),
                total = crate::file_size(total)
            )
        };
        rows.push(crate::line(said, 12.0, look.muted).size(Some(width), None));
    }
    Element::column(rows).gap(6.0).size(Some(width), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, png, texts};
    use limn::{DrawCommand, Frame, Point, Rect, Size, UiEvent};
    use rustcord_model::UploadToken;

    fn files(count: u64) -> Vec<PendingAttachment> {
        (1..=count)
            .map(|token| PendingAttachment {
                token: UploadToken(token),
                filename: format!("fixture-{token}.txt"),
                size_bytes: 12,
            })
            .collect()
    }

    fn view(files: &[PendingAttachment], width: f32, max_list_height: f32) -> DraftUploads<'_> {
        DraftUploads {
            channel: ChannelId(10),
            files,
            thumbnails: Vec::new(),
            picking: Some(90),
            selecting: true,
            cancelling: false,
            submitting: false,
            width,
            max_list_height,
        }
    }

    fn region(frame: &Frame, id: &str) -> Rect {
        frame
            .regions
            .iter()
            .find(|region| region.id == id)
            .unwrap_or_else(|| panic!("{id} is laid out"))
            .rect
    }

    fn center(rect: Rect) -> Point {
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
    }

    #[test]
    fn a_prepared_file_and_selection_have_scoped_remove_and_cancel_controls() {
        let files = files(1);
        let (_, frame) = frame(
            draft_uploads(view(&files, 320.0, 152.0), &Look::DARK),
            Size::new(320.0, 300.0),
        );
        assert!(texts(&frame).iter().any(|text| text == "fixture-1.txt"));
        for id in ["composer:10:upload:1:remove", "composer:10:pick:90:cancel"] {
            assert!(controls(&frame).contains(&id.to_string()), "missing {id}");
        }
    }

    #[test]
    fn complete_body_progress_still_waits_for_confirmation() {
        let (_, frame) = frame(
            pending_uploads(&[], Some((12, 12)), 320.0, &Look::DARK),
            limn::Size::new(320.0, 300.0),
        );
        assert!(texts(&frame)
            .iter()
            .any(|text| text.contains("Esperando confirmación")));
    }

    /// Discord's card: a picture fitted to the card's width and never
    /// enlarged, the fill alone while it decodes, the glyph otherwise.
    #[test]
    fn a_picture_fits_its_card_and_waits_on_the_fill_alone() {
        let files = files(3);
        let fill = Look::DARK.hover;
        let draw = |thumbnails: Vec<Thumbnail>| {
            let (_, frame) = frame(
                draft_uploads(
                    DraftUploads {
                        thumbnails,
                        picking: None,
                        ..view(&files, 800.0, 400.0)
                    },
                    &Look::DARK,
                ),
                Size::new(800.0, 400.0),
            );
            let pictures: Vec<_> = frame
                .commands
                .iter()
                .filter_map(|command| match command {
                    DrawCommand::RasterImage { rect, .. } => Some((rect.width, rect.height)),
                    _ => None,
                })
                .collect();
            let fills = frame
                .commands
                .iter()
                .filter(|command| matches!(command, DrawCommand::FillRect { color, .. } if *color == fill))
                .count();
            (pictures, fills)
        };
        let (pictures, fills) = draw(vec![
            Thumbnail::Picture(png(400, 300)),
            Thumbnail::Picture(png(300, 800)),
            Thumbnail::Picture(png(40, 30)),
        ]);
        assert_eq!(pictures, [(200.0, 150.0), (65.25, 174.0), (40.0, 30.0)]);
        assert_eq!(fills, 3, "each picture's box keeps the fill behind it");
        let (pictures, fills) = draw(vec![Thumbnail::Loading, Thumbnail::File]);
        assert!(pictures.is_empty());
        assert_eq!(fills, 1, "only the picture decoding shows its box");
    }

    #[test]
    fn eight_prepared_files_scroll_sideways_and_keep_the_editor_and_cancel_visible() {
        let files = files(8);
        let window = Size::new(320.0, 400.0);
        let (mut tree, frame) = frame(
            Element::column(vec![
                Element::clickable(
                    "fixture:uploads",
                    draft_uploads(view(&files, 320.0, 152.0), &Look::DARK),
                ),
                Element::clickable("fixture:editor", Element::text("Nuevo mensaje"))
                    .size(Some(320.0), Some(48.0)),
            ])
            .gap(8.0),
            window,
        );
        assert!(region(&frame, "composer:10:uploads").height <= 152.0);
        for id in ["fixture:editor", "composer:10:pick:90:cancel"] {
            let rect = region(&frame, id);
            assert!(rect.y + rect.height <= window.height, "{id}: {rect:?}");
            assert_eq!(frame.hit_test(center(rect)), Some(id));
        }
        let row = region(&frame, "composer:10:uploads");
        for _ in 0..20 {
            tree.handle_event(UiEvent::ScrollDirect {
                position: center(row),
                delta_y: -200.0,
            });
        }
        let frame = tree.layout(window);
        let last = region(&frame, "composer:10:upload:8:remove");
        assert!(
            last.x >= row.x && last.x + last.width <= row.x + row.width,
            "the final file is reachable by scrolling: {last:?} in {row:?}"
        );
        assert_eq!(
            frame.hit_test(center(last)),
            Some("composer:10:upload:8:remove")
        );
    }

    #[test]
    fn a_short_budget_shrinks_the_cards_and_keeps_picker_and_editor() {
        let files = files(8);
        let window = Size::new(320.0, 240.0);
        for count in [1, 8] {
            let (_, frame) = frame(
                Element::column(vec![
                    draft_uploads(view(&files[..count], window.width, 96.0), &Look::DARK),
                    Element::clickable("fixture:editor", Element::text("Borrador conservado"))
                        .size(Some(window.width), Some(48.0)),
                ])
                .gap(8.0),
                window,
            );
            assert!(region(&frame, "composer:10:uploads").height <= 96.0);
            for id in [
                "fixture:editor",
                "composer:10:pick:90:cancel",
                "composer:10:upload:1:remove",
            ] {
                let rect = region(&frame, id);
                assert!(rect.y >= 0.0 && rect.y + rect.height <= window.height);
                assert_eq!(frame.hit_test(center(rect)), Some(id));
            }
        }
    }
}
