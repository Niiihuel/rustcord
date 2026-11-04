//! The profile beside a direct conversation, as Discord's: a card 8 px in
//! from the column's edges, its head as a popout's, then who they are and
//! who they share with the account, their bio, «Miembro desde», what they do,
//! «Colección de juegos» and «Lista de deseados», and «Ver perfil completo»
//! fixed under the scroll. Measures are `docs/discord-reference.md`'s
//! «DM profile panel».

use limn::{Color, Component, Element, FontFamily, ScrollArea};
use rustcord_i18n::t;

use super::{
    activity_parts, bio, bubble, card_mutuals, card_names, card_notice, custom_status, game_card,
    hairline, ids, raised, restriction_badge, sized_card_head, HeadSize, Profile, AVATAR, BANNER,
    DOT, INSET, RING, WIDTH,
};
use crate::{centered, gap, line, Look};

/// The column the panel takes beside the conversation: Discord's 320.
pub const SIDE_WIDTH: f32 = 320.0;
/// The card: 8 in from the column, a 2 px edge, its inside a popout's width.
const MARGIN: f32 = 8.0;
const EDGE: f32 = 2.0;
/// Its head: the mark 57 down (a popout's is 61), 144 tall.
const SIDE_HEAD: HeadSize = HeadSize {
    width: WIDTH,
    banner: BANNER,
    avatar: AVATAR,
    ring: RING,
    left: INSET,
    top: 57.0,
    dot: DOT,
    height: 144.0,
};
/// Items of the wishlist shown, 78 px squares.
const WISHES_SHOWN: usize = 3;
const TILE: f32 = 78.0;

/// `profile` as the panel beside a direct conversation shows it, as tall as
/// it is given.
pub fn side_panel(profile: &Profile, look: &Look) -> Element {
    // Its cards keep the popout's fill; the panel's own ground is a step
    // lighter (Discord's #101013), and its mark's ring takes it.
    let card_fill = raised(look, 16);
    let fill = raised(look, 7);
    let look = &Look {
        card: fill,
        menu: if look.canvas.r > 128 { fill } else { look.menu },
        ..*look
    };
    let width = WIDTH - 2.0 * INSET;
    // The custom status in the popout's bubble, beside the mark.
    let status =
        custom_status(profile).map(|(custom, emoji)| bubble(custom, emoji, false, None, look));
    let note = profile
        .full_action
        .as_ref()
        .map(|action| action.replacen(ids::FULL, ids::FULL_NOTE, 1));
    let mut info = vec![card_names(profile, note.as_deref(), false, look)];
    if let Some(notice) = &profile.notice {
        info.push(card_notice(notice, look));
    }
    if let Some(restriction) = &profile.restriction {
        info.push(restriction_badge(restriction, look));
    }
    info.extend(card_mutuals(profile, look));
    let mut sections = vec![Element::column(info).gap(8.0)];
    sections.extend(bio(profile, false, width, look));
    // The account's date only: a conversation is in no server.
    if let Some(since) = profile.member_since.first() {
        sections.push(
            Element::column(vec![
                line(t("Miembro desde"), 12.0, look.text).font_family(FontFamily::SansSemibold),
                line(since.date.clone(), 14.0, look.text),
            ])
            .gap(8.0),
        );
    }
    let mut cards = activity_parts(profile, width, look);
    cards.extend(game_card(profile, 14.0, card_fill, look));
    cards.extend(wish_card(profile, card_fill, look));
    if !cards.is_empty() {
        sections.push(Element::column(cards).gap(8.0));
    }
    let body = ScrollArea::new(
        format!("{}{}", ids::SIDE_BODY, profile.handle),
        Element::column(vec![
            sized_card_head(profile, SIDE_HEAD, 8.0, status, look),
            Element::row(vec![
                gap(INSET, 0.0),
                Element::column(sections).gap(20.0).flex(1.0),
                gap(INSET, 0.0),
            ]),
            gap(0.0, 16.0),
        ]),
    )
    .appearance(look.scrollbar(false))
    .render()
    .flex(1.0);
    let mut inside = vec![body];
    if let Some(action) = &profile.full_action {
        inside.push(full_button(action, look));
    }
    let card = Element::panel(Element::column(inside))
        .padding(EDGE)
        .radius(8.0)
        .background(fill)
        .border(Color::rgba(255, 255, 255, 20), 1.0)
        .clip_children()
        .flex(1.0);
    Element::panel(Element::row(vec![card]))
        .padding(MARGIN)
        .size(Some(SIDE_WIDTH), None)
        .background(look.sidebar)
}

/// «Ver perfil completo» under the panel's scroll: 40 tall in 16 of room,
/// a faint fill and hairline, 16 px medium text.
fn full_button(action: &str, look: &Look) -> Element {
    let label = t("Ver perfil completo");
    let button = Element::clickable(
        action,
        Element::panel(centered(
            line(label, 16.0, look.text).font_family(FontFamily::SansMedium),
        ))
        .size(None, Some(40.0))
        .radius(8.0)
        .background(hairline(look, 31))
        .border(hairline(look, 41), 1.0),
    )
    .radius(8.0)
    .hover_background(look.hover)
    .accessible(limn::Access::new(limn::Role::Button).name(label.to_owned()));
    Element::panel(button).padding(16.0)
}

/// «Lista de deseados»: its title and «Ver todo» (the full profile), then
/// the first three items as 78 px squares spread across, each its picture
/// on a faint ground. Nothing without items.
fn wish_card(profile: &Profile, fill: Color, look: &Look) -> Option<Element> {
    if profile.wishlist.is_empty() {
        return None;
    }
    let mut header = vec![line(t("Lista de deseados"), 14.0, look.text)
        .font_family(FontFamily::SansMedium)
        .centered_y()];
    header.push(Element::spacer().flex(1.0));
    if let Some(action) = &profile.full_action {
        let all = t("Ver todo");
        header.push(
            Element::clickable(
                action.replacen(ids::FULL, ids::FULL_WISHLIST, 1),
                line(all, 12.0, look.text),
            )
            .hover_foreground(look.muted)
            .accessible(limn::Access::new(limn::Role::Button).name(all.to_owned()))
            .centered_y(),
        );
    }
    let mut tiles = Vec::new();
    for (index, wish) in profile.wishlist.iter().take(WISHES_SHOWN).enumerate() {
        if index > 0 {
            tiles.push(Element::spacer().flex(1.0));
        }
        let mut layers = vec![Element::panel(Element::spacer())
            .size(Some(TILE), Some(TILE))
            .radius(8.0)
            .background(hairline(look, 31))];
        if let Some(picture) = &wish.picture {
            layers.push(
                Element::raster_image(picture.clone().cover())
                    .size(Some(TILE), Some(TILE))
                    .radius(8.0),
            );
        }
        tiles.push(
            Element::stack(layers)
                .size(Some(TILE), Some(TILE))
                .accessible(limn::Access::default().description(wish.name.clone())),
        );
    }
    Some(
        Element::panel(Element::column(vec![Element::row(header), Element::row(tiles)]).gap(8.0))
            .padding(8.0)
            .radius(8.0)
            .background(fill),
    )
}
