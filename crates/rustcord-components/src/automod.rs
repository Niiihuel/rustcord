//! The pieces of the AutoMod page of a server's settings: a
//! rule as the list shows it, with its switch, and a kind of rule to make.
//! Controls are named `ss:am:…`.

use limn::{Button, ButtonSize, ButtonVariant, Component, Element, Icon, Switch};
use rustcord_i18n::t;
use rustcord_model::AutoModKind;

use crate::{icon, line, text, Look};

/// The mark of a kind of rule; `None` for one Rustcord does not know.
pub fn kind_glyph(kind: Option<AutoModKind>) -> Icon {
    match kind {
        Some(AutoModKind::Keywords) => Icon::Text,
        Some(AutoModKind::Mentions) => Icon::Bell,
        Some(AutoModKind::Spam) => Icon::WarningTriangle,
        Some(AutoModKind::Presets) => Icon::Book,
        Some(AutoModKind::Profile) => Icon::Users,
        None => Icon::Puzzle,
    }
}

/// A rule as the page lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleLine {
    pub id: u64,
    pub name: String,
    /// What it looks for and what it does.
    pub notes: Vec<String>,
    pub glyph: Icon,
    pub enabled: bool,
    /// Rustcord can change it without losing what it does not know.
    pub editable: bool,
    /// The account may change it.
    pub can: bool,
}

fn lead(glyph: Icon, look: &Look) -> Element {
    Element::panel(crate::centered(
        icon(glyph, look.text).size(Some(24.0), Some(24.0)),
    ))
    .size(Some(40.0), Some(40.0))
    .radius(20.0)
    .background(look.hover)
}

/// A rule, or a kind of rule, as Discord's AutoMod draws them: a card
/// rounded 8 on the raised ground, its 40 px mark, its name 16 semibold
/// over what it does, and its buttons at the end.
fn card(lead: Element, copy: Vec<Element>, buttons: Vec<Element>) -> Element {
    let mut row = vec![
        lead.centered_y(),
        Element::column(copy).gap(2.0).flex(1.0).centered_y(),
    ];
    row.extend(buttons.into_iter().map(Element::centered_y));
    Element::row(row).gap(16.0).padding(16.0)
}

fn framed(card: Element, look: &Look) -> Element {
    Element::panel(card)
        .radius(8.0)
        .background(look.raised)
        .border(look.border, 1.0)
}

fn small(id: String, glyph: Icon, label: String, enabled: bool, look: &Look) -> Element {
    Button::new(id, "", look.palette())
        .icon(glyph)
        .variant(ButtonVariant::Outline)
        .button_size(ButtonSize::Small)
        .enabled(enabled)
        .accessibility_label(label.clone())
        .tooltip(label)
        .render()
}

/// A row of the list: its mark, name and notes, its switch, and buttons to
/// edit it (when Rustcord can) and delete it.
pub fn rule_row(row: &RuleLine, look: &Look) -> Element {
    let mut copy =
        vec![line(&row.name, 16.0, look.text).font_family(limn::FontFamily::SansSemibold)];
    if !row.notes.is_empty() {
        copy.push(line(row.notes.join(" · "), 14.0, look.muted));
    }
    let id = row.id;
    let mut buttons = vec![Switch::new(
        format!("ss:am:rule:{id}:enabled"),
        row.enabled,
        look.palette(),
    )
    .accessibility_label(format!("{} {}", t("Activar"), row.name))
    .enabled(row.can)
    .render()];
    if row.editable {
        buttons.push(small(
            format!("ss:am:rule:{id}:edit"),
            Icon::Edit,
            format!("{} {}", t("Editar"), row.name),
            row.can,
            look,
        ));
    }
    buttons.push(small(
        format!("ss:am:rule:{id}:delete"),
        Icon::Trash,
        format!("{} {}", t("Eliminar"), row.name),
        row.can,
        look,
    ));
    framed(card(lead(row.glyph, look), copy, buttons), look)
}

/// A kind of rule to make: what it does, and «Crear» while the server has
/// room for one more and the account may.
pub fn new_rule_row(
    key: &str,
    kind: AutoModKind,
    title: &str,
    description: &str,
    enabled: bool,
    look: &Look,
) -> Element {
    let copy = vec![
        line(title, 16.0, look.text).font_family(limn::FontFamily::SansSemibold),
        text(description, 14.0, look.muted),
    ];
    // Discord's brand button.
    let create = Button::new(
        format!("ss:am:new:{key}"),
        t("Crear"),
        look.accent_palette(),
    )
    .button_size(ButtonSize::Small)
    .enabled(enabled)
    .accessibility_label(format!("{} {title}", t("Crear")))
    .render();
    framed(
        card(lead(kind_glyph(Some(kind)), look), copy, vec![create]),
        look,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};
    use limn::Size;

    const LOOK: Look = Look::DARK;

    fn row(editable: bool) -> RuleLine {
        RuleLine {
            id: 7,
            name: "Sin spoilers".into(),
            notes: vec!["Palabras propias".into(), "Bloquea".into()],
            glyph: kind_glyph(Some(AutoModKind::Keywords)),
            enabled: true,
            editable,
            can: true,
        }
    }

    #[test]
    fn a_rule_shows_its_switch_and_only_the_buttons_rustcord_can_honor() {
        let (_, laid) = frame(rule_row(&row(true), &LOOK), Size::new(700.0, 80.0));
        let ids = controls(&laid);
        for id in [
            "ss:am:rule:7:enabled",
            "ss:am:rule:7:edit",
            "ss:am:rule:7:delete",
        ] {
            assert!(ids.iter().any(|found| found == id), "{id}: {ids:?}");
        }
        assert!(texts(&laid)
            .iter()
            .any(|text| text.contains("Palabras propias · Bloquea")));
        let (_, unknown) = frame(rule_row(&row(false), &LOOK), Size::new(700.0, 80.0));
        assert!(!controls(&unknown).iter().any(|id| id.ends_with(":edit")));
    }

    #[test]
    fn a_kind_offers_to_create_one() {
        let element = new_rule_row(
            "mentions",
            AutoModKind::Mentions,
            "Spam de menciones",
            "Cuando alguien menciona a demasiadas personas.",
            true,
            &LOOK,
        );
        let (_, laid) = frame(element, Size::new(700.0, 90.0));
        assert!(controls(&laid).iter().any(|id| id == "ss:am:new:mentions"));
    }
}
