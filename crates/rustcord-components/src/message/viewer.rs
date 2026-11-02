//! The media viewer's own pieces, drawn as Discord draws them over a
//! message: who sent what it shows, and its «Más» menu.

use limn::{
    Access, ButtonVariant, Color, Component, ComponentPalette, DropdownMenu, DropdownMenuEntry,
    DropdownMenuItem, Element, FontFamily, Icon, IconButton, PopoverPlacement, Role, Tooltip,
    TooltipPlacement,
};
use rustcord_i18n::t;
use rustcord_model::{Attachment, MessageId};

use super::attachments::{copyable, ids, Copied};
use crate::{file_size, Look};

/// The lead of a lightbox: the sender's picture `face`, beside their name in
/// `ink` (their role's color, or the text's) and when it was written. Nothing in
/// it is a target, and it is at most 184 pixels wide.
pub fn lead(face: Element, name: &str, ink: Color, when: &str, look: &Look) -> Element {
    let text = Element::column(vec![
        Element::truncated_text(name)
            .font_family(FontFamily::SansMedium)
            .font_size(14.0)
            .foreground(ink)
            .size(None, Some(18.0)),
        Element::truncated_text(when)
            .font_family(FontFamily::Sans)
            .font_size(12.0)
            .foreground(look.muted)
            .size(None, Some(16.0)),
    ])
    .gap(2.0)
    .flex(1.0)
    .centered_y();
    Element::row(vec![face, text])
        .gap(12.0)
        .max_width(184.0)
        .centered_y()
}

/// The rows of «Ver detalles» for `file`, each its key, its label and its value:
/// its name, its size with its dimensions when known, and its text alternative
/// when the sender wrote one.
pub fn details(file: &Attachment) -> Vec<(&'static str, &'static str, String)> {
    let size = file_size(file.size_bytes);
    let size = match file.dimensions {
        Some((width, height)) => format!("{width}x{height} ({size})"),
        None => format!("({size})"),
    };
    let mut rows = vec![
        ("name", t("Nombre de archivo"), file.filename.clone()),
        ("size", t("Tamaño"), size),
    ];
    if let Some(description) = &file.description {
        rows.push((
            "alt",
            t("Descripción (texto alternativo)"),
            description.clone(),
        ));
    }
    rows
}

/// The entries of a viewer's «Más» for what it shows: `file` is the message's
/// file, or `None` for a card's picture; `prefix` is the viewer's ID prefix,
/// whose choices are `{prefix}:more:…`. A picture is copied only with
/// `copy_picture`, which is false for an animation.
pub fn entries(
    prefix: &str,
    message: MessageId,
    file: Option<&Attachment>,
    copy_picture: bool,
    developer: bool,
) -> Vec<DropdownMenuEntry> {
    let item = |id: String, label: &str, icon: Icon| {
        DropdownMenuEntry::Item(DropdownMenuItem::new(id, label).icon(icon))
    };
    let mut entries = Vec::new();
    match file {
        Some(file) => {
            let action = |verb| ids::action(message, file.id, verb);
            if copy_picture && copyable(file) == Some(Copied::Image) {
                entries.push(item(action("copy-image"), t("Copiar imagen"), Icon::Copy));
            }
            entries.push(item(
                action("copy"),
                t("Copiar enlace multimedia"),
                Icon::Copy,
            ));
            let rows = details(file)
                .into_iter()
                .map(|(key, label, value)| {
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new(format!("{prefix}:more:detail:{key}"), label)
                            .description(value),
                    )
                })
                .collect();
            entries.push(DropdownMenuEntry::Item(
                DropdownMenuItem::new(format!("{prefix}:more:details"), t("Ver detalles"))
                    .icon(Icon::InfoCircle)
                    .submenu(rows),
            ));
            if developer {
                entries.push(item(
                    action("copy-id"),
                    t("Copiar ID del adjunto"),
                    Icon::Copy,
                ));
            }
        }
        None => entries.push(item(
            format!("{prefix}:more:link"),
            t("Copiar enlace multimedia"),
            Icon::Copy,
        )),
    }
    entries
}

/// The «Más» of a viewer: a 32 by 32 button, `id`, that opens `entries` under
/// it while `open`. The menu reports `dismiss` when it is dismissed.
pub fn more(
    id: &str,
    entries: Vec<DropdownMenuEntry>,
    open: bool,
    dismiss: &str,
    look: &Look,
) -> Element {
    // The viewer's controls are drawn dark whatever the theme, as the lightbox's own are: limn's
    // rule, which its `is_dark` keeps private (the border's brightness).
    let palette = look.menu_palette();
    let dark =
        u16::from(palette.border.r) + u16::from(palette.border.g) + u16::from(palette.border.b)
            < 3 * 128;
    let chrome = if dark {
        palette
    } else {
        ComponentPalette::neutral_dark()
    };
    let button = IconButton::new(id, Icon::More, chrome)
        .variant(ButtonVariant::Ghost)
        .ink(chrome.muted)
        .hover_ink(chrome.text)
        .render()
        .radius(limn::tokens().radius.card)
        .size(Some(32.0), Some(32.0))
        .accessible(Access::new(Role::Button).name(t("Más")));
    let trigger = Tooltip::new(button, id, t("Más"), chrome)
        .placement(TooltipPlacement::Below)
        .render()
        .size(Some(32.0), Some(32.0));
    DropdownMenu::new(trigger, entries, open, dismiss, palette)
        .context_menu_style()
        .context_metrics(crate::look::MENU_METRICS)
        .placement(PopoverPlacement::BottomEnd)
        .render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcord_model::{AttachmentId, RemoteFile};

    fn file(dimensions: Option<(u32, u32)>) -> Attachment {
        Attachment {
            id: AttachmentId(7),
            filename: "dibujo.png".into(),
            size_bytes: 48_213,
            content_type: Some("image/png".into()),
            dimensions,
            preview: None,
            download: Some(RemoteFile {
                url: "https://cdn.example.test/dibujo.png".into(),
            }),
            spoiler: false,
            voice: None,
            explicit: false,
            placeholder: None,
            description: None,
        }
    }

    #[test]
    fn details_give_the_size_with_its_dimensions_when_known() {
        let with = details(&file(Some((300, 340))));
        assert_eq!(
            (with[1].0, with[1].2.as_str()),
            ("size", "300x340 (47.1 KB)")
        );
        let without = details(&file(None));
        assert_eq!((without[1].0, without[1].2.as_str()), ("size", "(47.1 KB)"));
        assert_eq!(
            (without[0].0, without[0].2.as_str()),
            ("name", "dibujo.png")
        );
    }

    #[test]
    fn a_text_alternative_is_a_detail_and_a_submenu_choice_only_when_the_file_has_one() {
        let text = "Un camino entre colinas verdes bajo un sol amarillo.";
        let mut described = file(Some((300, 340)));
        described.description = Some(text.into());
        let rows = details(&described);
        assert_eq!(
            (rows.len(), rows[2].0, rows[2].2.as_str()),
            (3, "alt", text)
        );
        assert_eq!(details(&file(None)).len(), 2);
        // The «Ver detalles» submenu lists its choices by their IDs.
        let choices = |file: &Attachment| {
            let entries = entries("attachment-viewer", MessageId(1), Some(file), true, false);
            let submenu = entries
                .into_iter()
                .find_map(|entry| match entry {
                    DropdownMenuEntry::Item(item)
                        if item.action_id == "attachment-viewer:more:details" =>
                    {
                        item.submenu
                    }
                    _ => None,
                })
                .expect("the submenu");
            submenu
                .into_iter()
                .filter_map(|entry| match entry {
                    DropdownMenuEntry::Item(item) => Some(item.action_id),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let alt = "attachment-viewer:more:detail:alt".to_owned();
        assert!(choices(&described).contains(&alt));
        assert!(!choices(&file(None)).contains(&alt));
    }
}
