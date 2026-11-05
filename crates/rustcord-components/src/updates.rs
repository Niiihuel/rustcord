//! Rustcord's own updates: the screen of a
//! mandatory one, shown before the main window, and the header's button of
//! an optional one, as Discord's title bar has it. Actions start with
//! `update:`.

use limn::{
    Access, AlertDialog, Button, ButtonVariant, Component, ComponentPalette, ControlSize, Element,
    FontFamily, Icon, IconButton, Progress as Bar, Role, ScrollArea, Size, Spinner,
    TooltipPlacement, TooltipSpec,
};
use rustcord_i18n::{t, tr};

use crate::{centered, file_size, line, text, Look};

/// The actions of the screen, the header's button and their settings.
pub mod ids {
    /// Download, install and start again.
    pub const INSTALL: &str = "update:install";
    /// The header's button: install, start the installed version or try
    /// again, as the update stands.
    pub const BUTTON: &str = "update:button";
    /// Update anyway, leaving the voice channel (the question over a call).
    pub const VOICE_CONFIRM: &str = "update:voice-confirm";
    pub const VOICE_CANCEL: &str = "update:voice-cancel";
    /// Try a failed download or installation again.
    pub const RETRY: &str = "update:retry";
    /// Leave Rustcord from the mandatory screen.
    pub const QUIT: &str = "update:quit";
    /// Look for a newer Rustcord now (settings).
    pub const CHECK: &str = "update:check";
    /// Look when Rustcord starts, or no longer (settings).
    pub const AT_START: &str = "update:at-start";
    /// The release's page on GitHub, with its pictures and videos.
    pub const ON_GITHUB: &str = "update:on-github";
    /// Start the installed version now, after a close was cancelled (settings).
    pub const RESTART: &str = "update:restart";
    /// The icon tile at the top of the settings card; pressing it does nothing.
    pub const CARD: &str = "update:card";
}

/// Where a release's page is, as GitHub shows it: its notes with their
/// pictures and videos, and its files.
pub fn release_page(version: &str) -> String {
    format!("https://github.com/Niiihuel/rustcord/releases/tag/v{version}")
}

/// A release's notes as its page reads them, `width` wide: Markdown, where
/// a short line right over a list (the tag's message drops `#` headings)
/// reads as a heading, and a first line that only names the release is
/// left out, the title says it already.
pub fn release_notes(version: &str, notes: &str, width: f32, look: &Look) -> Element {
    let lines: Vec<&str> = notes.trim().lines().collect();
    let name = format!("Rustcord {version}");
    let mut source = String::with_capacity(notes.len() + 64);
    for (at, current) in lines.iter().enumerate() {
        let current = current.trim_end();
        if at == 0 && current.trim() == name {
            continue;
        }
        let heading = !current.trim().is_empty()
            && !current.starts_with(['-', '*', '#', '>', ' '])
            && current.chars().count() <= 60
            && lines
                .get(at + 1)
                .is_some_and(|next| next.trim_start().starts_with("- "));
        if heading {
            source.push_str("### ");
        }
        source.push_str(current);
        source.push('\n');
    }
    limn::Markdown::new(&source, look.palette())
        .width(width)
        .font_size(14.0)
        .render()
        .element
}

/// Where the update offered stands.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Stage {
    /// Offered, not started.
    #[default]
    Offered,
    Downloading {
        have: u64,
        whole: u64,
    },
    Installing,
    /// Installed; Rustcord closes to start the new version.
    Restarting,
    /// Installed, but the close that starts it was cancelled: start it now.
    Installed,
    /// Why it stopped; Rustcord is as it was.
    Failed(String),
}

/// An update as the screen and the notice show it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UpdateView {
    pub version: String,
    pub installed: String,
    pub notes: String,
    pub stage: Stage,
}

/// The window of a mandatory update, before anything else opens: the new
/// version and its notes, the download's progress, and why it failed with
/// «Reintentar» and «Salir».
pub fn screen(view: &UpdateView, window: Size, look: &Look) -> Element {
    let title = t("Actualización obligatoria");
    let mut parts = vec![
        centered(
            Element::icon(Icon::Custom(crate::MARK))
                .large_icon(72.0)
                .foreground(look.text),
        )
        .size(None, Some(80.0)),
        centered(
            line(title, 20.0, look.text)
                .font_family(FontFamily::SansMedium)
                .accessible(Access::new(Role::Heading).name(title)),
        )
        .size(None, Some(28.0)),
        text(
            tr!(
                "Rustcord {version} reemplaza a la {installed} que tienes instalada, que ya no se puede usar. Rustcord se actualiza antes de abrir.",
                version = view.version,
                installed = view.installed
            ),
            14.0,
            look.muted,
        ),
    ];
    if !view.notes.trim().is_empty() {
        parts.push(notes(&view.notes, 180.0, look));
    }
    parts.push(stage(view, look));
    let middle = Element::column(parts).gap(14.0).size(Some(460.0), None);
    Element::panel(centered(middle))
        .background(look.canvas)
        .size(Some(window.width), Some(window.height))
}

/// The release's notes, scrolled when long.
fn notes(notes: &str, height: f32, look: &Look) -> Element {
    ScrollArea::new("update:notes-text", text(notes.trim(), 13.0, look.text))
        .appearance(look.scrollbar(true))
        .render()
        .max_height(height)
}

/// The bar and bytes of a download, the install, the restart or the reason
/// it failed: the same words on the screens and in Ajustes. `None` before
/// the download starts.
pub fn progress(stage: &Stage, look: &Look) -> Option<Element> {
    Some(match stage {
        Stage::Offered => return None,
        Stage::Downloading { have, whole } => {
            let fraction = if *whole == 0 {
                0.0
            } else {
                *have as f32 / *whole as f32
            };
            let said = tr!(
                "Descargando… {have} de {whole}",
                have = file_size(*have),
                whole = file_size(*whole)
            );
            let percent = (fraction * 100.0).round() as u32;
            Element::column(vec![
                Bar::new(fraction, look.accent_palette())
                    .id("update:progress")
                    .render()
                    .accessibility_label(said.clone()),
                line(format!("{said} · {percent}%"), 12.0, look.muted),
            ])
            .gap(6.0)
        }
        Stage::Installing => Element::row(vec![
            Spinner::new("update:installing", look.accent).render(),
            line(t("Instalando…"), 12.0, look.muted),
        ])
        .gap(8.0),
        Stage::Restarting => Element::row(vec![
            Spinner::new("update:restarting", look.accent).render(),
            line(t("Reiniciando Rustcord…"), 12.0, look.muted),
        ])
        .gap(8.0),
        Stage::Installed => line(
            t("Instalada; reinicia Rustcord para usarla."),
            12.0,
            look.muted,
        ),
        Stage::Failed(reason) => text(reason.clone(), 13.0, look.danger).accessible(
            Access::new(Role::Alert)
                .id("update:error")
                .name(reason.clone()),
        ),
    })
}

/// The buttons of the mandatory screen where the update stands, under its
/// progress.
fn stage(view: &UpdateView, look: &Look) -> Element {
    let button = |id: &str, label: &str, primary: bool| {
        Button::new(
            id,
            label,
            if primary {
                look.accent_palette()
            } else {
                look.menu_palette()
            },
        )
        .variant(if primary {
            ButtonVariant::Default
        } else {
            ButtonVariant::Ghost
        })
        .size(ControlSize::Compact)
        .render()
    };
    let buttons = |list: Vec<Element>| Element::row(list).gap(8.0);
    let actions = match &view.stage {
        Stage::Offered => {
            return centered(Spinner::new("update:waiting", look.accent).render())
                .size(None, Some(24.0));
        }
        Stage::Failed(_) => Some(buttons(vec![
            button(ids::RETRY, t("Reintentar"), true),
            button(ids::QUIT, t("Salir"), false),
        ])),
        Stage::Installed => Some(buttons(vec![button(
            ids::RESTART,
            t("Reiniciar ahora"),
            true,
        )])),
        Stage::Downloading { .. } | Stage::Installing | Stage::Restarting => None,
    };
    let mut parts = Vec::new();
    parts.extend(progress(&view.stage, look));
    parts.extend(actions);
    Element::column(parts).gap(8.0)
}

/// The header's button of an update offered, last of its tools, as the one
/// Discord's title bar ends with: its download glyph in the look's green,
/// named by its hint. Offered or failed (in red, the reason under its hint)
/// it installs, installed it starts the new version; while it downloads,
/// installs or restarts it waits at half its ink, as Discord's disabled one.
pub fn header_button(stage: &Stage, look: &Look) -> Element {
    let (hint, enabled) = match stage {
        Stage::Offered => (t("Hay una actualización disponible"), true),
        Stage::Downloading { .. } | Stage::Installing => (t("Descargando la actualización"), false),
        Stage::Restarting => (t("Reiniciando Rustcord…"), false),
        Stage::Installed => (t("¡Actualización lista!"), true),
        Stage::Failed(_) => (t("No se pudo actualizar"), true),
    };
    let mut hint = TooltipSpec::new(hint).placement(TooltipPlacement::Below);
    if let Stage::Failed(reason) = stage {
        // A reason is a sentence: it wraps under the title.
        hint = hint.description(reason.clone()).max_width(280.0);
    }
    let ink = if matches!(stage, Stage::Failed(_)) {
        look.danger
    } else {
        look.online
    };
    // A disabled button paints its palette's muted ink.
    let palette = ComponentPalette {
        muted: Look::alpha(ink, 128),
        ..look.palette()
    };
    IconButton::new(ids::BUTTON, Icon::Custom(crate::glyphs::DOWNLOAD), palette)
        .variant(ButtonVariant::Ghost)
        .tooltip_with(hint)
        .ink(ink)
        .hover_ink(ink)
        .enabled(enabled)
        .render()
}

/// Before an update leaves the call, over `root`: Discord's question, with
/// its red «¡Actualizar de todas formas!». Rustcord does not join the call
/// again once it opens, so it says the person will have to.
pub fn voice_confirm(root: Element, look: &Look) -> Element {
    AlertDialog::new(
        root,
        true,
        t("¿Salir del canal de voz?"),
        t("Si actualizas Rustcord mientras estás en un canal de voz, saldrás del canal y tendrás que volver a entrar. Probablemente actualizarás de todos modos pero, ya sabes, solo te estoy advirtiendo."),
        ids::VOICE_CONFIRM,
        ids::VOICE_CANCEL,
        look.menu_palette(),
    )
    .width(crate::friends::CONFIRM_WIDTH)
    .confirm_label(t("¡Actualizar de todas formas!"))
    .cancel_label(t("Cancelar"))
    .destructive(look.danger)
    .render()
}

/// The color of a status pill: what the state means.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tone {
    /// Something to do or under way.
    Accent,
    /// Up to date.
    Online,
    /// Failed.
    Danger,
}

/// The settings card of the update check: what it says now, the download,
/// install or failure under way, and the buttons that act on it.
pub struct CheckCard {
    pub title: String,
    pub subtitle: String,
    /// The status pill's words and tone, when the state has one.
    pub pill: Option<(String, Tone)>,
    /// Where the download, the install or the restart stands.
    pub stage: Option<Stage>,
    /// A check that failed, when no update is under way.
    pub error: Option<String>,
    /// Working on something: a spinner waits beside the buttons.
    pub busy: bool,
    /// The buttons, right-aligned at the bottom.
    pub actions: Vec<Element>,
}

/// The card's body under its title: the bar and bytes of a download, the
/// install or restart as it runs, or the reason it failed in danger color.
fn card_body(stage: Option<&Stage>, error: Option<&str>, look: &Look) -> Option<Element> {
    let failed = |reason: &str| {
        text(reason.to_owned(), 13.0, look.danger).accessible(
            Access::new(Role::Alert)
                .id("update:error")
                .name(reason.to_owned()),
        )
    };
    match stage {
        Some(Stage::Downloading { have, whole }) => {
            let fraction = if *whole == 0 {
                0.0
            } else {
                *have as f32 / *whole as f32
            };
            let percent = (fraction * 100.0).round() as u32;
            let said = tr!(
                "Descargando… {have} de {whole}",
                have = file_size(*have),
                whole = file_size(*whole)
            );
            Some(
                Element::column(vec![
                    Bar::new(fraction, look.accent_palette())
                        .id("update:progress")
                        .height(8.0)
                        .render()
                        .accessibility_label(said),
                    Element::row(vec![
                        line(
                            tr!(
                                "{have} de {whole}",
                                have = file_size(*have),
                                whole = file_size(*whole)
                            ),
                            12.0,
                            look.muted,
                        )
                        .flex(1.0),
                        line(tr!("{pct} %", pct = percent), 12.0, look.muted),
                    ]),
                ])
                .gap(8.0),
            )
        }
        Some(Stage::Installing) => Some(
            Element::row(vec![
                Spinner::new("update:installing", look.accent).render(),
                line(t("Instalando…"), 12.0, look.muted),
            ])
            .gap(8.0),
        ),
        Some(Stage::Restarting) => Some(
            Element::row(vec![
                Spinner::new("update:restarting", look.accent).render(),
                line(t("Reiniciando Rustcord…"), 12.0, look.muted),
            ])
            .gap(8.0),
        ),
        Some(Stage::Installed) => Some(line(
            t("Instalada; reinicia Rustcord para usarla."),
            12.0,
            look.muted,
        )),
        Some(Stage::Failed(reason)) => Some(failed(reason)),
        Some(Stage::Offered) | None => error.map(failed),
    }
}

/// The settings card of the update check (`update-check`): a tile with
/// Rustcord's mark, the title and subtitle, a status pill, the body and the
/// buttons at the bottom right.
pub fn check_card(card: CheckCard, look: &Look) -> Element {
    let tone = |tone: Tone| match tone {
        Tone::Accent => look.accent,
        Tone::Online => look.online,
        Tone::Danger => look.danger,
    };
    let tile = centered(
        Element::icon(Icon::Custom(crate::MARK))
            .large_icon(22.0)
            .foreground(look.text),
    )
    .size(Some(40.0), Some(40.0))
    .background(look.canvas)
    .radius(10.0);
    // The tile is named, so the page can bring the card's top into view;
    // a whole card as a button would hide its text from accessibility.
    let tile = Element::clickable(ids::CARD, tile);
    let name = card.title.clone();
    let mut head = vec![
        tile,
        // Both wrap: a card's copy is short, and a cut title hides its status.
        Element::column(vec![
            text(card.title, 16.0, look.text).font_family(FontFamily::SansMedium),
            text(card.subtitle, 12.0, look.muted),
        ])
        .gap(2.0)
        .flex(1.0)
        .centered_y(),
    ];
    if let Some((label, pill)) = card.pill {
        let color = tone(pill);
        head.push(
            Element::row(vec![
                Element::spacer().size(Some(8.0), None),
                line(label, 12.0, color),
                Element::spacer().size(Some(8.0), None),
            ])
            .padding(4.0)
            .radius(10.0)
            .background(Look::alpha(color, 31))
            .centered_y(),
        );
    }
    let mut parts = vec![Element::row(head).gap(12.0)];
    parts.extend(card_body(card.stage.as_ref(), card.error.as_deref(), look));
    let mut actions = vec![Element::spacer().flex(1.0)];
    if card.busy {
        actions.push(Spinner::new("update:busy", look.accent).render());
    }
    actions.extend(card.actions);
    parts.push(Element::row(actions).gap(8.0));
    Element::column(parts)
        .gap(12.0)
        .padding(16.0)
        .background(look.raised)
        .border(look.border, 1.0)
        .radius(8.0)
        .accessible(Access::new(Role::Status).name(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::frame;

    fn texts(element: Element) -> Vec<String> {
        crate::testing::texts(&frame(element, Size::new(1280.0, 800.0)).1)
    }

    fn view(stage: Stage) -> UpdateView {
        UpdateView {
            version: "1.2.0".into(),
            installed: "1.1.0".into(),
            notes: "Arregla el inicio.".into(),
            stage,
        }
    }

    #[test]
    fn a_mandatory_update_shows_its_progress_and_on_failure_retry_and_quit() {
        let window = Size::new(1280.0, 800.0);
        let shown = texts(screen(
            &view(Stage::Downloading {
                have: 512 * 1024,
                whole: 2 * 1024 * 1024,
            }),
            window,
            &Look::DARK,
        ));
        assert!(shown
            .iter()
            .any(|text| text.contains("1.2.0") && text.contains("1.1.0")));
        assert!(shown.iter().any(|text| text == "Arregla el inicio."));
        assert!(shown.iter().any(|text| text.starts_with("Descargando…")));
        assert!(!shown.iter().any(|text| text == "Reintentar"));
        let failed = texts(screen(
            &view(Stage::Failed("Sin conexión.".into())),
            window,
            &Look::DARK,
        ));
        for wanted in ["Sin conexión.", "Reintentar", "Salir"] {
            assert!(
                failed.iter().any(|text| text == wanted),
                "{wanted}: {failed:?}"
            );
        }
    }

    /// The header button's name (its hint), the paragraph under it, and
    /// whether it waits.
    fn button(stage: Stage) -> (String, Option<String>, bool) {
        let (_, laid) = frame(header_button(&stage, &Look::ONYX), Size::new(200.0, 60.0));
        let node = laid
            .access
            .iter()
            .find(|node| node.id == ids::BUTTON)
            .expect("the button is there");
        (node.name.clone(), node.description.clone(), node.disabled)
    }

    #[test]
    fn the_header_button_says_where_the_update_stands_and_waits_while_it_downloads() {
        let downloading = Stage::Downloading { have: 1, whole: 2 };
        for (stage, hint, waits) in [
            (Stage::Offered, "Hay una actualización disponible", false),
            (downloading, "Descargando la actualización", true),
            (Stage::Installing, "Descargando la actualización", true),
            (Stage::Restarting, "Reiniciando Rustcord…", true),
            (Stage::Installed, "¡Actualización lista!", false),
        ] {
            assert_eq!(button(stage), (hint.to_owned(), None, waits));
        }
        // A failure says why in its hint, and a press tries again.
        let failed = Stage::Failed("Sin conexión.".into());
        assert_eq!(
            button(failed.clone()),
            ("No se pudo actualizar".to_owned(), None, false)
        );
        let size = Size::new(400.0, 200.0);
        let (mut tree, laid) = frame(header_button(&failed, &Look::ONYX), size);
        let at = laid.regions[0].rect;
        tree.handle_event(limn::UiEvent::PointerMove(limn::Point::new(
            at.x + at.width / 2.0,
            at.y + at.height / 2.0,
        )));
        let later = std::time::Instant::now() + std::time::Duration::from_secs(2);
        tree.layout_at(size, later);
        let shown = crate::testing::texts(&tree.layout_at(size, later));
        assert!(
            shown.iter().any(|text| text == "Sin conexión."),
            "{shown:?}"
        );
    }

    #[test]
    fn the_voice_question_updates_anyway_or_cancels() {
        let (_, laid) = frame(
            voice_confirm(Element::spacer(), &Look::ONYX),
            Size::new(800.0, 600.0),
        );
        let shown = crate::testing::texts(&laid);
        for wanted in [
            "¿Salir del canal de voz?",
            "¡Actualizar de todas formas!",
            "Cancelar",
        ] {
            assert!(
                shown.iter().any(|text| text == wanted),
                "{wanted}: {shown:?}"
            );
        }
        let controls = crate::testing::controls(&laid);
        for id in [ids::VOICE_CONFIRM, ids::VOICE_CANCEL] {
            assert!(controls.iter().any(|control| control == id), "{controls:?}");
        }
    }

    fn check(stage: Option<Stage>, error: Option<&str>) -> CheckCard {
        CheckCard {
            title: "Rustcord 1.2.0 está disponible".into(),
            subtitle: "Instalada: Rustcord 1.1.0.".into(),
            pill: Some(("Disponible".into(), Tone::Accent)),
            stage,
            error: error.map(str::to_owned),
            busy: false,
            actions: Vec::new(),
        }
    }

    #[test]
    fn the_settings_card_shows_the_download_as_bytes_and_percent() {
        let shown = texts(check_card(
            check(
                Some(Stage::Downloading {
                    have: 512 * 1024,
                    whole: 2 * 1024 * 1024,
                }),
                None,
            ),
            &Look::DARK,
        ));
        for wanted in [
            "512.0 KB de 2.0 MB",
            "25 %",
            "Rustcord 1.2.0 está disponible",
        ] {
            assert!(
                shown.iter().any(|text| text == wanted),
                "{wanted}: {shown:?}"
            );
        }
    }

    #[test]
    fn the_settings_card_puts_a_failure_in_its_body_once() {
        let shown = texts(check_card(
            check(Some(Stage::Failed("Sin conexión.".into())), None),
            &Look::DARK,
        ));
        assert_eq!(
            shown.iter().filter(|text| *text == "Sin conexión.").count(),
            1
        );
    }
}
