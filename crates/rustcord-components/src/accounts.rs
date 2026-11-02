//! The accounts a provider remembers, as one list: the account manager, the
//! login chooser, the loading screen and the settings share it, as Discord's
//! account list does: a card with a row per account, the open one
//! marked, and a «⋯» whose only item signs that account out.

use limn::{
    Access, Button, ButtonVariant, Component, ComponentPalette, ControlSize, DropdownMenu,
    DropdownMenuEntry, DropdownMenuItem, Element, FontFamily, Icon, IconButton, Modal,
    PopoverPlacement, RasterImage, Role,
};
use rustcord_i18n::t;

use crate::{avatar_with_image, gap, line, Look};

/// One remembered account, as the list shows it.
#[derive(Clone)]
pub struct AccountRow {
    pub name: String,
    /// Its picture, when it has one; its initial otherwise.
    pub picture: Option<RasterImage>,
    /// The open account: marked «Cuenta activa», and it has no action.
    pub active: bool,
    /// The button beside the «⋯»: its control ID and its label.
    pub action: Option<(String, String)>,
    /// The «⋯» that opens the menu, and the menu's state.
    pub more: String,
    pub menu: bool,
    /// The menu's only item, «Cerrar sesión», and the ID that closes the menu.
    pub sign_out: String,
    pub more_dismiss: String,
}

/// The list `width` wide: one row after another, a hairline between each two.
pub fn list(rows: &[AccountRow], width: f32, look: &Look) -> Element {
    let mut children = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            children.push(separator(look));
        }
        children.push(account(row, look));
    }
    Element::panel(Element::column(children))
        .size(Some(width), None)
        .background(look.raised)
        .radius(8.0)
}

/// A secondary button's fill: the list's ground is the raised tone, so the
/// border tone is the one that still shows on it.
pub fn button_palette(look: &Look) -> ComponentPalette {
    ComponentPalette {
        raised: look.border,
        ..look.menu_palette()
    }
}

/// A hairline inset 16 px from the left, as Discord's.
fn separator(look: &Look) -> Element {
    Element::row(vec![
        gap(16.0, 0.0),
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border)
            .flex(1.0),
    ])
}

fn account(row: &AccountRow, look: &Look) -> Element {
    let mut copy =
        vec![line(row.name.clone(), 16.0, look.text).font_family(FontFamily::SansSemibold)];
    if row.active {
        copy.push(
            line(t("Cuenta activa"), 14.0, look.online).font_family(FontFamily::SansSemibold),
        );
    }
    let mut trailing = Vec::new();
    if let Some((id, label)) = &row.action {
        trailing.push(
            Button::new(id.clone(), label.clone(), button_palette(look))
                .variant(ButtonVariant::Secondary)
                .size(ControlSize::Compact)
                .render()
                .centered_y(),
        );
    }
    trailing.push(more(row, look).centered_y());
    // The row's padding is one value: the 4 px spacers make the 16 px sides.
    Element::row(vec![
        gap(4.0, 0.0),
        avatar_with_image(&row.name, look.accent, 40.0, row.picture.clone()).centered_y(),
        Element::column(copy).gap(2.0).flex(1.0).centered_y(),
        Element::row(trailing).gap(8.0).centered_y(),
        gap(4.0, 0.0),
    ])
    .gap(12.0)
    .padding(12.0)
    .accessible(Access::new(Role::Group).name(row.name.clone()))
}

pub mod ids {
    /// Closes «Gestionar cuentas» (its close button and Escape).
    pub const CLOSE: &str = "acc:close";
    /// «Agregar una cuenta».
    pub const ADD: &str = "acc:add";
}

/// What the dialog says under its title: a limit that was reached, or an
/// account that signed out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub text: String,
    /// A refusal (danger) rather than a success.
    pub critical: bool,
}

/// The width of «Gestionar cuentas», as Discord's medium modal (480 px).
const MANAGE_WIDTH: f32 = 480.0;

/// «Gestionar cuentas» over `root`: the notice, if any, over the list and
/// «Agregar una cuenta» under it.
pub fn manage(root: Element, rows: &[AccountRow], notice: Option<&Notice>, look: &Look) -> Element {
    let mut body = Vec::new();
    if let Some(notice) = notice {
        body.push(notice_panel(notice, look));
        body.push(gap(0.0, 16.0));
    }
    body.push(list(rows, MANAGE_WIDTH - 2.0 * 16.0, look));
    let footer = Element::row(vec![
        Element::spacer().flex(1.0),
        Button::new(ids::ADD, t("Agregar una cuenta"), button_palette(look))
            .variant(ButtonVariant::Secondary)
            .render(),
    ]);
    Modal::new(
        root,
        true,
        t("Gestionar cuentas"),
        t("Cambia de cuenta, inicia sesión, cierra sesión, lo que tú quieras."),
        ids::CLOSE,
        look.menu_palette(),
    )
    .width(MANAGE_WIDTH)
    .body(Element::column(body))
    .footer(footer)
    .render()
}

/// A notice under a title: a refusal in the danger tone, a success in the online tone.
pub fn notice_panel(notice: &Notice, look: &Look) -> Element {
    let ink = if notice.critical {
        look.danger
    } else {
        look.online
    };
    let mark = if notice.critical {
        Icon::WarningTriangle
    } else {
        Icon::Check
    };
    Element::panel(
        Element::row(vec![
            crate::icon(mark, ink).centered_y(),
            crate::text(notice.text.clone(), 14.0, look.text)
                .flex(1.0)
                .centered_y(),
        ])
        .gap(12.0)
        .padding(12.0),
    )
    .background(Look::alpha(ink, 28))
    .border(Look::alpha(ink, 150), 1.0)
    .radius(8.0)
    .accessible(Access::new(Role::Alert).name(notice.text.clone()))
}

/// The «⋯» of a row, with its one item in the danger color.
fn more(row: &AccountRow, look: &Look) -> Element {
    let label = t("Más opciones");
    let button = IconButton::new(row.more.clone(), Icon::More, look.menu_palette())
        .variant(ButtonVariant::Ghost)
        .tooltip(label)
        .accessibility_label(label)
        .render();
    let sign_out =
        DropdownMenuItem::new(row.sign_out.clone(), t("Cerrar sesión")).destructive(look.danger);
    DropdownMenu::new(
        button,
        vec![DropdownMenuEntry::Item(sign_out)],
        row.menu,
        row.more_dismiss.clone(),
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::MENU_METRICS)
    .width(MENU_WIDTH)
    .placement(PopoverPlacement::BottomEnd)
    .render()
}

const MENU_WIDTH: f32 = 200.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};
    use limn::Size;

    fn row(name: &str, active: bool, menu: bool) -> AccountRow {
        AccountRow {
            name: name.into(),
            picture: None,
            active,
            action: (!active).then(|| (format!("acc:switch={name}"), "Cambiar".into())),
            more: format!("acc:more={name}"),
            menu,
            sign_out: format!("acc:sign-out={name}"),
            more_dismiss: "acc:more-dismiss".into(),
        }
    }

    #[test]
    fn the_open_account_is_marked_and_the_others_can_be_switched_to() {
        let rows = [row("Ada", true, false), row("Bruno", false, false)];
        let (_, laid) = frame(list(&rows, 440.0, &Look::DARK), Size::new(440.0, 300.0));
        let painted = texts(&laid);
        assert_eq!(
            painted
                .iter()
                .filter(|text| *text == "Cuenta activa")
                .count(),
            1,
            "{painted:?}"
        );
        assert!(painted.contains(&"Bruno".to_owned()));
        let shown = controls(&laid);
        assert!(shown.contains(&"acc:switch=Bruno".to_owned()));
        assert!(!shown.contains(&"acc:switch=Ada".to_owned()));
        assert!(shown.contains(&"acc:more=Ada".to_owned()));
        // Closed, the menu's item is not laid out.
        assert!(!shown.contains(&"acc:sign-out=Ada".to_owned()));
    }

    #[test]
    fn the_more_menu_offers_signing_out_while_it_is_open() {
        let rows = [row("Bruno", false, true)];
        let (_, laid) = frame(list(&rows, 440.0, &Look::DARK), Size::new(440.0, 300.0));
        assert!(controls(&laid).contains(&"acc:sign-out=Bruno".to_owned()));
    }
}
