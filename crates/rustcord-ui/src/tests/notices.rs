//! Notices must leave the live draft visible, including its dynamic height.

use super::*;
use limn::{DrawCommand, HitKind, Rect, Role};
use rustcord_model::{PendingAttachment, UploadToken};

const UNSUPPORTED: &str = "Todavía no puedes adjuntar archivos arrastrándolos. Usa Adjuntar.";
const DRAFT: &str = "Primera línea del borrador\nSegunda línea conservada\nTercera línea sin enviar\nÚltima línea de la respuesta";

fn draft(files: u64) -> (Host, ChannelId, MessageId) {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    host.type_text(DRAFT);
    let reply = host.store.timeline(channel).unwrap().newest().unwrap().id;
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: format!("msg:{reply}:reply"),
        },
        host.now,
    );
    host.run(intents);
    host.ui.uploads.account = host.store.me();
    host.ui.uploads.drafts.insert(
        channel,
        (1..=files)
            .map(|token| PendingAttachment {
                token: UploadToken(token),
                filename: format!("fixture-{token}.txt"),
                size_bytes: 12,
            })
            .collect(),
    );
    host.frame();
    assert_eq!(host.ui.reply_to, Some(reply));
    (host, channel, reply)
}

fn layout(host: &mut Host, window: Size) -> Frame {
    host.tree.set_root(view(
        &host.ui,
        &host.store,
        &ViewContext {
            window,
            ..host.context()
        },
    ));
    host.tree.layout(window)
}

fn region(frame: &Frame, id: &str, kind: HitKind) -> Rect {
    frame
        .regions
        .iter()
        .find(|region| region.id == id && region.kind == kind)
        .unwrap_or_else(|| panic!("{id} ({kind:?}) must remain visible"))
        .rect
}

fn alert(frame: &Frame, title: &str) -> Rect {
    let matches: Vec<_> = frame
        .access
        .iter()
        .filter(|node| node.role == Role::Alert && node.name == title)
        .collect();
    assert_eq!(matches.len(), 1, "one visible projection of {title}");
    matches[0].rect
}

#[track_caller]
fn visible(rect: Rect, window: Size) {
    assert!(rect.width > 0.0 && rect.height > 0.0, "{rect:?}");
    assert!(
        rect.x >= 0.0
            && rect.y >= 0.0
            && rect.x + rect.width <= window.width + 0.5
            && rect.y + rect.height <= window.height + 0.5,
        "{rect:?} escapes {window:?}"
    );
}

fn center(rect: Rect) -> Point {
    Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

/// Scroll the row of chosen files to its start or its end, as a wheel over
/// it does.
fn scroll_files(host: &mut Host, channel: ChannelId, window: Size, end: bool) {
    let files = format!("composer:{channel}:uploads");
    let row = region(&layout(host, window), &files, HitKind::Scroll);
    for _ in 0..20 {
        host.tree.handle_event(UiEvent::ScrollDirect {
            position: center(row),
            delta_y: if end { -400.0 } else { 400.0 },
        });
    }
}

fn draft_controls(frame: &Frame, channel: ChannelId) -> Vec<Rect> {
    vec![
        region(frame, &editor_id(channel), HitKind::TextArea),
        region(frame, "composer:cancel", HitKind::Button),
        region(
            frame,
            &format!("composer:{channel}:uploads"),
            HitKind::Scroll,
        ),
    ]
}

#[test]
fn an_unsupported_drop_notice_leaves_multiline_reply_and_eight_files_visible() {
    for window in [Size::new(980.0, 800.0), WINDOW] {
        let (mut host, channel, reply) = draft(8);
        host.ui.notify(UNSUPPORTED, None, true, host.now);
        let frame = layout(&mut host, window);
        let notice = alert(&frame, UNSUPPORTED);
        visible(notice, window);
        for control in draft_controls(&frame, channel) {
            visible(control, window);
            assert!(
                notice.intersection(control).is_none(),
                "the notice {notice:?} covers a live draft control {control:?}"
            );
        }
        let editor = region(&frame, &editor_id(channel), HitKind::TextArea);
        assert_eq!(
            frame.hit_test(center(editor)),
            Some(editor_id(channel).as_str())
        );
        host.tree.handle_event(UiEvent::PointerDown(center(editor)));
        host.tree.handle_event(UiEvent::PointerUp(center(editor)));
        assert_eq!(host.tree.focused_id(), Some(editor_id(channel).as_str()));
        assert_eq!(host.tree.input_value(&editor_id(channel)), Some(DRAFT));
        assert_eq!(host.ui.drafts[&channel], DRAFT);
        assert_eq!(host.ui.reply_to, Some(reply));
        assert_eq!(host.ui.uploads.drafts[&channel].len(), 8);
        for (token, end) in [(1, false), (8, true)] {
            if end {
                scroll_files(&mut host, channel, window, true);
            }
            let frame = layout(&mut host, window);
            let id = format!("composer:{channel}:upload:{token}:remove");
            let remove = region(&frame, &id, HitKind::Button);
            visible(remove, window);
            assert_eq!(frame.hit_test(center(remove)), Some(id.as_str()));
            assert!(alert(&frame, UNSUPPORTED).intersection(remove).is_none());
        }
    }
}

#[test]
fn a_notice_floats_bottom_left_and_takes_no_room_from_the_conversation() {
    let (mut host, channel, _) = draft(8);
    let before = layout(&mut host, WINDOW);
    host.ui.notify("Adjunto preparado", None, false, host.now);
    let after = layout(&mut host, WINDOW);
    let notice = alert(&after, "Adjunto preparado");
    for id in [timeline_id(channel), editor_id(channel)] {
        let kind = if id == editor_id(channel) {
            HitKind::TextArea
        } else {
            HitKind::Scroll
        };
        assert_eq!(
            region(&before, &id, kind),
            region(&after, &id, kind),
            "{id}"
        );
    }
    // Over the column of channels, as wide as the account's card.
    let (left, right, _) = rustcord_components::chrome::USER_PANEL_MARGIN;
    let column = rustcord_components::rail::WIDTH + host.ui.prefs.sidebar_width;
    assert_eq!(notice.x, left);
    assert!(
        notice.x + notice.width <= column - right + 0.5,
        "{notice:?}"
    );
}

#[test]
fn notices_stack_bottom_left_newest_lowest_above_the_account_panel() {
    let (mut host, channel, _) = draft(2);
    let titles = ["Primer aviso", "Segundo aviso", "Tercer aviso"];
    for title in titles {
        host.ui
            .notify(title, Some("Detalle".into()), true, host.now);
    }
    for window in [WINDOW, Size::new(980.0, 800.0), Size::new(720.0, 480.0)] {
        let frame = layout(&mut host, window);
        let cards: Vec<Rect> = titles.iter().map(|title| alert(&frame, title)).collect();
        for card in &cards {
            visible(*card, window);
            assert_eq!(card.x, rustcord_components::chrome::USER_PANEL_MARGIN.0);
            assert_eq!(card.width, cards[0].width, "one width for the stack");
        }
        // Oldest on top, the same gap between each, none over another.
        for pair in cards.windows(2) {
            let gap = pair[1].y - (pair[0].y + pair[0].height);
            assert!((gap - 8.0).abs() < 0.5, "{gap}: {cards:?}");
        }
        // The account's controls stay reachable, under the lowest.
        let lowest = cards[2];
        for id in ["me:status", "me:mute", "me:deafen", "me:settings"] {
            let control = region(&frame, id, HitKind::Button);
            assert!(lowest.y + lowest.height + 8.0 <= control.y + 0.5, "{id}");
            assert_eq!(frame.hit_test(center(control)), Some(id));
        }
        for control in draft_controls(&frame, channel) {
            for card in &cards {
                assert!(card.intersection(control).is_none());
            }
        }
    }
    // The fourth pushes the oldest out.
    host.ui.notify("Cuarto aviso", None, false, host.now);
    let frame = layout(&mut host, WINDOW);
    assert!(!frame
        .access
        .iter()
        .any(|node| node.role == Role::Alert && node.name == titles[0]));
    let third = alert(&frame, titles[2]);
    assert!(alert(&frame, "Cuarto aviso").y > third.y);
}

#[test]
fn three_wrapped_notices_scroll_completely_without_overflowing_into_the_draft() {
    let (mut host, channel, reply) = draft(8);
    let window = Size::new(980.0, 800.0);
    let titles: Vec<_> = (1..=3)
        .map(|index| format!("Aviso {index}: conserva el borrador y revisa el detalle completo antes de volver a intentar la preparación de estos archivos"))
        .collect();
    let details: Vec<_> = (1..=3)
        .map(|index| {
            format!(
                "{} FIN-{index}",
                "Este detalle de prueba debe poder leerse completo al desplazar los avisos. "
                    .repeat(30)
            )
        })
        .collect();
    for (title, detail) in titles.iter().zip(&details) {
        host.ui.notify(title, Some(detail.clone()), true, host.now);
    }
    let frame = layout(&mut host, window);
    let editor = region(&frame, &editor_id(channel), HitKind::TextArea);
    visible(editor, window);
    let controls = draft_controls(&frame, channel);
    // Their own list, over the column of channels' (which scrolls too).
    let scroll = frame
        .regions
        .iter()
        .rev()
        .find(|region| {
            region.kind == HitKind::Scroll
                && region.id == "notices"
                && frame.access.iter().any(|node| {
                    node.role == Role::Alert
                        && titles.contains(&node.name)
                        && node.rect.intersection(region.rect).is_some()
                })
        })
        .expect("oversized notices must have a scrollable viewport")
        .clone();
    assert_eq!(
        frame.hit_test(center(scroll.rect)),
        Some("notices"),
        "a wheel over them scrolls them, not the channels under them"
    );
    visible(scroll.rect, window);
    for control in &controls {
        assert!(scroll.rect.intersection(*control).is_none());
    }
    host.tree.scroll_to_start(&scroll.id);
    let mut reached = [false; 3];
    for _ in 0..96 {
        let frame = layout(&mut host, window);
        for block in frame.text_layouts() {
            let Some(index) = details.iter().position(|detail| detail == &block.source) else {
                continue;
            };
            for run in block.runs {
                if let Some(rect) = run.visible {
                    assert!(scroll.rect.intersection(rect).is_some());
                    for control in &controls {
                        assert!(rect.intersection(*control).is_none());
                    }
                    if run.text.contains(&format!("FIN-{}", index + 1)) {
                        reached[index] = true;
                    }
                }
            }
        }
        if host.tree.is_scroll_at_end(&scroll.id, 0.5) {
            break;
        }
        host.tree.handle_event(UiEvent::ScrollDirect {
            position: center(scroll.rect),
            delta_y: -scroll.rect.height * 0.75,
        });
    }
    assert_eq!(reached, [true; 3], "all descriptions must be reachable");
    assert_eq!(host.tree.focused_id(), Some(editor_id(channel).as_str()));
    assert_eq!(host.tree.input_value(&editor_id(channel)), Some(DRAFT));
    assert_eq!(host.ui.reply_to, Some(reply));
    assert_eq!(host.ui.uploads.drafts[&channel].len(), 8);
}

#[test]
fn a_notice_is_projected_once_above_a_modal_then_returns_to_the_conversation() {
    let (mut host, channel, _) = draft(8);
    let title = "Estado del adjunto";
    host.ui.notify(title, None, true, host.now);
    host.key(Key::F, false, true);
    assert!(host.ui.search.is_some());
    let frame = layout(&mut host, WINDOW);
    let projected = alert(&frame, title);
    visible(projected, WINDOW);
    // In the window's corner, as high as over the column.
    assert_eq!(
        projected.x,
        rustcord_components::chrome::USER_PANEL_MARGIN.0
    );
    let account = region(&frame, "me:settings", HitKind::Button);
    assert!(projected.y + projected.height <= account.y, "{projected:?}");
    let index = |needle: &str| {
        frame
            .commands
            .iter()
            .position(|command| matches!(command, DrawCommand::Text { text, .. } if text == needle))
            .unwrap_or_else(|| panic!("{needle} must be painted"))
    };
    assert!(
        index(title) > index("Buscar mensajes"),
        "the notice must be above the modal"
    );
    host.key(Key::Escape, false, false);
    assert!(host.ui.search.is_none());
    let frame = layout(&mut host, WINDOW);
    let notice = alert(&frame, title);
    for control in draft_controls(&frame, channel) {
        assert!(notice.intersection(control).is_none());
    }
    host.ui.notices.clear();
    let frame = layout(&mut host, WINDOW);
    assert!(!frame
        .access
        .iter()
        .any(|node| node.role == Role::Alert && node.name == title));
    assert_eq!(host.tree.focused_id(), Some(editor_id(channel).as_str()));
    assert_eq!(host.tree.input_value(&editor_id(channel)), Some(DRAFT));
}

fn offer(host: &mut Host, stage: rustcord_components::updates::Stage) {
    host.ui.updates.installation = Some("C:/Rustcord".into());
    host.ui.updates.offered = Some(rustcord_components::updates::UpdateView {
        version: "1.2.0".into(),
        installed: "1.1.0".into(),
        notes: "Notas.".into(),
        stage,
    });
}

#[test]
fn an_offered_update_ends_the_header_and_its_corner_card_is_gone() {
    use rustcord_components::updates::{ids, Stage};
    let (mut host, _, _) = draft(2);
    let frame = layout(&mut host, WINDOW);
    assert!(!frame.regions.iter().any(|region| region.id == ids::BUTTON));
    offer(&mut host, Stage::Offered);
    host.ui.notify("Aviso", None, true, host.now);
    let frame = layout(&mut host, WINDOW);
    let button = region(&frame, ids::BUTTON, HitKind::Button);
    let search = region(&frame, "message-search:open", HitKind::Button);
    assert!(
        button.x > search.x + search.width,
        "last of the header's tools"
    );
    assert!(
        button.x + button.width <= WINDOW.width - crate::title_bar::room(&host.ui) + 0.5,
        "clear of the window's buttons"
    );
    assert!(button.y + button.height <= rustcord_components::sidebar::HEADER);
    assert_eq!(frame.hit_test(center(button)), Some(ids::BUTTON));
    // No card, nor its buttons, anywhere.
    for gone in [ids::INSTALL, "update:later", "update:notes"] {
        assert!(
            !frame.regions.iter().any(|region| region.id == gone),
            "{gone}"
        );
    }
    let names: Vec<&str> = frame.access.iter().map(|node| node.name.as_str()).collect();
    assert!(!names.iter().any(|name| name.contains("está disponible")));
    // A notice still sits bottom-left, nowhere near it.
    assert!(alert(&frame, "Aviso").intersection(button).is_none());
}

#[test]
fn updating_from_the_header_in_a_call_asks_first() {
    use rustcord_components::updates::{ids, Stage};
    let mut host = Host::new();
    let click = |host: &mut Host, id: &str| {
        update(
            &mut host.ui,
            &mut host.store,
            &UiAction::Clicked { id: id.into() },
            host.now,
        )
    };
    offer(&mut host, Stage::Offered);
    host.open_channel("general");
    // Out of a call it installs at once.
    assert_eq!(click(&mut host, ids::BUTTON), vec![Intent::InstallUpdate]);
    assert!(!host.ui.updates.voice_confirm);
    host.vz_join("Sala común");
    assert!(host.store.call().is_some());
    host.open_channel("general");
    host.click(ids::BUTTON);
    assert!(host.ui.updates.voice_confirm, "the question shows");
    let painted = host.painted();
    assert!(painted
        .iter()
        .any(|text| text == "¿Salir del canal de voz?"));
    let frame = host.frame();
    for id in [ids::VOICE_CONFIRM, ids::VOICE_CANCEL] {
        region(&frame, id, HitKind::Button);
    }
    host.click(ids::VOICE_CANCEL);
    assert!(!host.ui.updates.voice_confirm);
    assert!(host.store.call().is_some(), "cancelled, the call goes on");
    // Confirmed, it installs; installed, it starts the new version.
    host.click(ids::BUTTON);
    assert_eq!(
        click(&mut host, ids::VOICE_CONFIRM),
        vec![Intent::InstallUpdate]
    );
    assert!(!host.ui.updates.voice_confirm);
    offer(&mut host, Stage::Installed);
    assert_eq!(click(&mut host, ids::BUTTON), Vec::new());
    assert!(host.ui.updates.voice_confirm);
    assert_eq!(
        click(&mut host, ids::VOICE_CONFIRM),
        vec![Intent::RestartForUpdate]
    );
    // While it downloads the button waits, and asks nothing.
    offer(&mut host, Stage::Downloading { have: 1, whole: 2 });
    assert_eq!(click(&mut host, ids::BUTTON), Vec::new());
    assert!(!host.ui.updates.voice_confirm);
    let frame = host.frame();
    let node = frame
        .access
        .iter()
        .find(|node| node.id == ids::BUTTON)
        .expect("the button shows while it downloads");
    assert!(node.disabled);
    assert_eq!(node.name, "Descargando la actualización");
}

#[test]
fn authentication_keeps_one_visible_notice_without_creating_a_composer() {
    let mut host = Host::new();
    host.store
        .provider
        .as_mut()
        .unwrap()
        .capabilities
        .authentication = true;
    host.event(Event::Authentication(
        rustcord_provider::AuthenticationState::SignedOut,
    ));
    let title = "No se pudo completar la selección";
    host.ui.notify(title, None, true, host.now);
    for window in [Size::new(980.0, 800.0), WINDOW] {
        let frame = layout(&mut host, window);
        visible(alert(&frame, title), window);
        assert!(frame
            .regions
            .iter()
            .any(|region| region.id == "auth:sign-in"));
        assert!(!frame
            .regions
            .iter()
            .any(|region| region.id.starts_with("composer:")));
    }
}

#[test]
fn native_selection_does_not_move_notices_back_over_the_live_composer() {
    let (mut host, channel, _) = draft(7);
    let request = begin_attachment_import(
        &mut host.ui,
        &host.store,
        channel,
        AttachmentImportSource::Picker,
    )
    .expect("one file slot remains available");
    assert!(attachment_pick_pending(&host.ui));
    host.ui.notify(UNSUPPORTED, None, true, host.now);
    let frame = layout(&mut host, WINDOW);
    let notice = alert(&frame, UNSUPPORTED);
    for control in draft_controls(&frame, channel) {
        assert!(notice.intersection(control).is_none());
    }
    let cancel = region(
        &frame,
        &format!("composer:{channel}:pick:{request}:cancel"),
        HitKind::Button,
    );
    assert!(notice.intersection(cancel).is_none());
    assert_eq!(
        frame.hit_test(center(cancel)),
        Some(format!("composer:{channel}:pick:{request}:cancel").as_str())
    );
}

#[test]
fn notices_leave_the_live_draft_reachable_at_the_minimum_desktop_size() {
    let (mut host, channel, reply) = draft(8);
    let window = Size::new(720.0, 480.0);
    for index in 1..=3 {
        host.ui.notify(
            format!("Aviso {index}: revisa la preparación de los archivos antes de continuar"),
            Some("Este detalle completo debe seguir accesible mediante scroll. ".repeat(30)),
            true,
            host.now,
        );
    }
    let frame = layout(&mut host, window);
    for control in draft_controls(&frame, channel) {
        visible(control, window);
        for notice in frame.access.iter().filter(|node| node.role == Role::Alert) {
            assert!(notice.rect.intersection(control).is_none());
        }
    }
    let layouts = frame.text_layouts();
    let title = layouts
        .iter()
        .find(|block| block.source.starts_with("Aviso 1:"))
        .expect("the initial notice title must be laid out");
    let first_line = title.runs.first().expect("a title has a line of text");
    let shown = first_line
        .visible
        .expect("the initial title line must be visible");
    let whole = first_line.rect;
    let epsilon = 0.001;
    assert!(
        shown.x <= whole.x + epsilon
            && shown.y <= whole.y + epsilon
            && shown.x + shown.width >= whole.x + whole.width - epsilon
            && shown.y + shown.height >= whole.y + whole.height - epsilon,
        "fitting the editor must leave a whole readable line of the notice, not a sliver: {shown:?} clips {whole:?}"
    );
    let editor = region(&frame, &editor_id(channel), HitKind::TextArea);
    assert_eq!(
        frame.hit_test(center(editor)),
        Some(editor_id(channel).as_str())
    );
    for (token, end) in [(1, false), (8, true)] {
        scroll_files(&mut host, channel, window, end);
        let frame = layout(&mut host, window);
        let id = format!("composer:{channel}:upload:{token}:remove");
        let remove = region(&frame, &id, HitKind::Button);
        visible(remove, window);
        assert_eq!(frame.hit_test(center(remove)), Some(id.as_str()));
    }
    assert_eq!(host.tree.focused_id(), Some(editor_id(channel).as_str()));
    assert_eq!(host.tree.input_value(&editor_id(channel)), Some(DRAFT));
    assert_eq!(host.ui.reply_to, Some(reply));
    assert_eq!(host.ui.uploads.drafts[&channel].len(), 8);
}

#[test]
fn the_update_button_never_crowds_the_header_into_the_window_buttons() {
    use rustcord_components::updates::{ids, Stage};
    let mut host = Host::new();
    host.open_channel("general");
    // The right edge of the header's last tool at `width`, and the search's
    // width (a box in wide windows).
    let last = |host: &mut Host, width: f32| {
        let frame = layout(host, Size::new(width, 800.0));
        let find = |id: &str| {
            frame
                .regions
                .iter()
                .find(|region| region.id == id && region.kind == HitKind::Button)
                .map(|found| found.rect)
        };
        let search = find("message-search:open").expect("the search");
        let last = find(ids::BUTTON).unwrap_or(search);
        (last.x + last.width, search.width)
    };
    // From the narrowest the host lets the window get, with the system's
    // frame and with Rustcord's buttons and edges (`frame::MIN_SIZE`): the
    // button costs the header one tool's room (45), so the header fits with
    // it wherever it fits in 45 less without it; and a narrow header has it
    // in its «más» instead, as it was without it.
    for (buttons, narrowest) in [(false, 720_u16), (true, 868)] {
        host.ui.desktop.window_buttons = buttons;
        let room = crate::title_bar::room(&host.ui);
        for width in (narrowest..=1600).step_by(8) {
            let width = f32::from(width);
            host.ui.updates.offered = None;
            let (plain, search) = last(&mut host, width);
            let (less, less_search) = last(&mut host, width - 45.0);
            offer(&mut host, Stage::Offered);
            let (with, _) = last(&mut host, width);
            if crate::view::narrow_header(&host.ui, width) {
                // Folded, it ends no further than it did, or it fits.
                assert!(with <= plain.max(width - room + 0.5), "{width} ({buttons})");
            } else if search == less_search && less <= width - 45.0 - room + 0.5 {
                assert!(with <= width - room + 0.5, "{width} ({buttons}): {with}");
            }
        }
        let frame = layout(&mut host, Size::new(f32::from(narrowest), 800.0));
        assert!(!frame.regions.iter().any(|region| region.id == ids::BUTTON));
    }
}
