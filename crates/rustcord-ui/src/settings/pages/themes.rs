//! «Tema» in Apariencia: the built-in looks and the ones
//! made here, the editor of a theme with its preview, the main color and
//! the window's transparency. What they do is in [`crate::themes`].

use limn::{Color, Component, Element, TextField};
use rustcord_components::server_settings::{swatches, swatches_width, Swatches};
use rustcord_components::theme::{preview, token_row, TokenRow};
use rustcord_components::user_settings::{self as parts, Swatch};
use rustcord_components::{look::TOKENS, text, Look};
use rustcord_i18n::{es, t, tr};
use rustcord_model::Rgb;

use super::{select, Ctx};
use crate::themes::{self as logic, *};
use crate::Theme;

/// The section's settings, each with the words it is found by.
pub(super) const ROWS: &[(&str, &str)] = &[
    (
        "theme-device",
        es("igual que el tema del dispositivo sistema claro oscuro escritorio automático adaptar"),
    ),
    (
        "theme-light",
        es("modo claro tema claro día luz"),
    ),
    (
        "theme-dark",
        es("modo oscuro tema oscuro ónix negro noche propio"),
    ),
    (
        "themes",
        es("temas predeterminados claro oscuro ónix negro colores modo propio crear importar exportar"),
    ),
    (
        "theme-editor",
        es("editor de temas colores vista previa base guardar"),
    ),
    (
        "accent-color",
        es("color principal acento preajustes personalizado"),
    ),
    (
        "transparency",
        es("transparencia ventana desenfoque escritorio fondo compositor kwin hyprland"),
    ),
    (
        "theme-related",
        es("accesibilidad saturación colores alto contraste"),
    ),
];

/// The setting `key` of the section; `None` for the rest, `Some(None)` for
/// one with nothing to show.
pub(super) fn setting(ctx: &Ctx, key: &str) -> Option<Option<Element>> {
    let following = ctx.ui.prefs.system_theme;
    Some(match key {
        "theme-device" => Some(device_theme(ctx)),
        "theme-light" => following.then(|| light_theme(ctx)),
        "theme-dark" => following.then(|| dark_theme(ctx)),
        "themes" => (!following).then(|| chooser(ctx)),
        "theme-editor" => editor(ctx),
        "accent-color" => Some(accent(ctx)),
        "transparency" => Some(transparency(ctx)),
        "theme-related" => Some(parts::related(
            "us:page=accessibility",
            limn::Icon::Sliders,
            t("Accesibilidad"),
            t("Cambia la saturación de los colores, activa el modo de alto contraste y mucho más"),
            ctx.look,
        )),
        _ => return None,
    })
}

/// «Igual que el tema del dispositivo»: on, the light and dark modes below
/// are the look; off, the look is one of the default themes.
fn device_theme(ctx: &Ctx) -> Element {
    parts::toggle(
        SAME,
        t("Igual que el tema del dispositivo"),
        Some(t("Adaptar al modo claro u oscuro de tu dispositivo.")),
        ctx.ui.prefs.system_theme,
        None,
        ctx.look,
    )
}

/// «Modo claro»: the look the desktop's light mode shows, which is Claro.
fn light_theme(ctx: &Ctx) -> Element {
    parts::setting(
        t("Modo claro"),
        None,
        Some(select(
            ctx,
            "theme-light",
            t("Modo claro"),
            &[("us:theme-light=light", t("Claro"), true, true)],
            theme_width(ctx),
        )),
        true,
        ctx.look,
    )
}

/// «Modo oscuro»: the look the desktop's dark mode shows, among Oscuro,
/// Ónix and the own themes with a dark base.
fn dark_theme(ctx: &Ctx) -> Element {
    let themes = &ctx.ui.themes;
    let mut options: Vec<(String, String, bool, bool)> = vec![
        (
            format!("{DARK}dark"),
            t("Oscuro").into(),
            themes.dark == DarkLook::Dark,
            true,
        ),
        (
            format!("{DARK}onyx"),
            t("Ónix").into(),
            themes.dark == DarkLook::Onyx,
            true,
        ),
    ];
    for (at, theme) in themes.saved.iter().enumerate() {
        if theme.base == "light" {
            continue;
        }
        let chosen = themes.dark == DarkLook::Own(theme.name.clone());
        options.push((format!("{DARK}own:{at}"), theme.name.clone(), chosen, true));
    }
    let borrowed: Vec<(&str, &str, bool, bool)> = options
        .iter()
        .map(|(id, name, chosen, enabled)| (id.as_str(), name.as_str(), *chosen, *enabled))
        .collect();
    parts::setting(
        t("Modo oscuro"),
        None,
        Some(select(
            ctx,
            "theme-dark",
            t("Modo oscuro"),
            &borrowed,
            theme_width(ctx),
        )),
        true,
        ctx.look,
    )
}

/// The width of the two pickers, as the other selects of the settings.
fn theme_width(ctx: &Ctx) -> f32 {
    (ctx.geometry.column * 0.4).clamp(180.0, 280.0)
}

fn rgb(color: Color) -> Rgb {
    Rgb(u32::from(color.r) << 16 | u32::from(color.g) << 8 | u32::from(color.b))
}

/// The looks to choose from and what can be done with the one in use.
fn chooser(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let themes = &ui.themes;
    let custom = themes.active.is_some();
    let built_in: Vec<Swatch> = [
        (Theme::Light, "light", t("Claro")),
        (Theme::Dark, "dark", t("Oscuro")),
        (Theme::Onyx, "onyx", t("Ónix")),
    ]
    .map(|(theme, key, name)| Swatch {
        id: format!("us:theme={key}"),
        name: name.into(),
        fill: theme.look().canvas,
        selected: !custom && ui.theme == theme,
    })
    .into();
    let own: Vec<Swatch> = themes
        .saved
        .iter()
        .enumerate()
        .map(|(at, theme)| Swatch {
            id: format!("{USE}{at}"),
            name: theme.name.clone(),
            fill: theme.look().canvas,
            selected: themes.active == Some(at),
        })
        .collect();
    let mut column = vec![
        parts::subtitle(t("Temas predeterminados"), look),
        parts::theme_swatches(&built_in, look),
    ];
    if !own.is_empty() {
        column.push(Element::spacer().size(None, Some(12.0)));
        column.push(parts::theme_swatches(&own, look));
    }
    let waiting = themes.waiting.is_some();
    let mut buttons = vec![
        parts::secondary_button(CREATE, t("Crear tema"), themes.editor.is_none(), look),
        parts::secondary_button(IMPORT, t("Importar…"), !waiting, look),
    ];
    if let Some(theme) = ui.active_theme() {
        if themes.deleting {
            column.push(Element::spacer().size(None, Some(12.0)));
            column.push(parts::notice(
                &tr!("¿Borrar «{name}»? No se puede deshacer.", name = theme.name),
                look,
            ));
            buttons = vec![
                parts::danger_button(DELETE_CONFIRM, t("Borrar"), look),
                parts::secondary_button(DELETE_CANCEL, t("Cancelar"), true, look),
            ];
        } else {
            buttons.extend([
                parts::secondary_button(EDIT, t("Editar"), themes.editor.is_none(), look),
                parts::secondary_button(EXPORT, t("Exportar…"), !waiting, look),
                parts::secondary_button(DELETE, t("Borrar"), true, look),
            ]);
        }
    }
    column.push(Element::spacer().size(None, Some(12.0)));
    column.push(Element::row(buttons).gap(8.0).wrap());
    column.push(Element::spacer().size(None, Some(12.0)));
    Element::column(column)
}

/// The theme being made: its name and base, the preview, and every color.
fn editor(ctx: &Ctx) -> Option<Element> {
    let (ui, look) = (ctx.ui, ctx.look);
    let (border, focus) = parts::field_outline(look);
    let editor = ui.themes.editor.as_ref()?;
    let width = ctx.geometry.column;
    let draft = &editor.draft;
    let shown = draft.look();
    let mut column = vec![
        parts::subtitle(
            if editor.replaces.is_some() {
                t("Editar tema")
            } else {
                t("Crear tema")
            },
            look,
        ),
        text(
            t("Elige una base y cambia los colores que quieras: la vista previa los muestra al instante y la ventana, al guardar. Se guarda en este equipo."),
            14.0,
            look.muted,
        ),
        Element::spacer().size(None, Some(12.0)),
        TextField::new(NAME, editor.name.clone(), t("Nombre del tema"), look.palette())
            .accessibility_label(t("Nombre del tema"))
            .height(parts::FIELD_HEIGHT)
            .outline(border, focus)
            .radius(parts::FIELD_RADIUS)
            .render(),
    ];
    if let Some(error) = &editor.error {
        column.push(text(error.as_str(), 14.0, look.danger).padding(4.0));
    }
    column.extend([
        Element::spacer().size(None, Some(12.0)),
        parts::setting(
            t("Base"),
            Some(t("Los colores que no cambies salen de ella.")),
            None,
            true,
            look,
        ),
        parts::theme_swatches(
            &[
                ("onyx", t("Ónix")),
                ("dark", t("Oscuro")),
                ("light", t("Claro")),
            ]
            .map(|(key, name)| Swatch {
                id: format!("{BASE}{key}"),
                name: name.into(),
                fill: Look::named(key).unwrap_or(Look::ONYX).canvas,
                selected: draft.base == key,
            }),
            look,
        ),
        Element::spacer().size(None, Some(16.0)),
        preview(&shown, width),
    ]);
    for (group, tokens) in TOKENS {
        column.push(parts::subtitle(t(group), look));
        for (key, name) in tokens {
            column.push(token_row(
                &TokenRow {
                    prefix: format!("{TOKEN}{key}"),
                    name: t(name),
                    color: logic::draft_color(draft, key),
                    changed: draft.colors.contains_key(key),
                    open: editor.picking == Some(*key),
                    picked: crate::color_picker::picked(ui, &format!("{TOKEN}{key}")),
                },
                look,
            ));
        }
    }
    column.extend([
        Element::spacer().size(None, Some(16.0)),
        Element::row(vec![
            parts::primary_button(SAVE, t("Guardar"), true, look),
            parts::secondary_button(CANCEL, t("Cancelar"), true, look),
            Element::spacer().flex(1.0),
            parts::secondary_button(EXPORT, t("Exportar…"), ui.themes.waiting.is_none(), look),
        ])
        .gap(8.0),
        Element::spacer().size(None, Some(16.0)),
    ]);
    Some(Element::column(column))
}

/// The main color, over any look: the look's own, a preset or any.
fn accent(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let own = match ui.active_theme() {
        Some(theme) => theme.look().accent,
        None => ui.system_accent().unwrap_or_else(|| ui.theme.look().accent),
    };
    Element::column(vec![
        parts::subtitle(t("Color principal"), look),
        text(
            t("Tiñe botones, menciones e indicadores sobre cualquier tema. El primero es el del tema."),
            14.0,
            look.muted,
        ),
        Element::spacer().size(None, Some(12.0)),
        swatches(
            &Swatches {
                prefix: ACCENT.into(),
                chosen: ui.themes.accent.map(rgb),
                default: Some(rgb(own)),
                custom: ui.themes.accent_hex.clone(),
                enabled: true,
                narrow: ctx.geometry.column < swatches_width(true),
                picked: crate::color_picker::picked(ui, ACCENT),
            },
            look,
        ),
        Element::spacer().size(None, Some(12.0)),
    ])
}

/// How much of the desktop shows through the window.
fn transparency(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let percent = ui.themes.transparency;
    let mut note = if percent == 0 {
        t("Opaca: el escritorio no se ve a través de la ventana.").to_owned()
    } else {
        tr!(
            "{percent} %: el escritorio se ve a través del riel, las columnas y la conversación. Los menús y los diálogos siguen opacos.",
            percent
        )
    };
    if percent > 0 && !ui.themes.see_through {
        note.push(' ');
        note.push_str(t("Se aplica al volver a abrir Rustcord."));
    }
    note.push(' ');
    note.push_str(t(
        "El desenfoque lo pone el compositor (KWin o Hyprland), si lo tiene activado.",
    ));
    Element::column(vec![
        parts::subtitle(t("Transparencia de la ventana"), look),
        text(note, 14.0, look.muted),
        Element::spacer().size(None, Some(12.0)),
        parts::slider(
            TRANSPARENCY,
            f32::from(percent) / f32::from(MAX_TRANSPARENCY),
            look,
        )
        .steps(usize::from(MAX_TRANSPARENCY / TRANSPARENCY_STEP) + 1)
        .width(ctx.geometry.column)
        .accessibility_label(t("Transparencia de la ventana"))
        .render(),
        Element::spacer().size(None, Some(12.0)),
    ])
}
