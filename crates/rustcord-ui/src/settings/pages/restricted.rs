//! «Ignorar y bloquear» (Discord's «Ignore & Block»): the people the account
//! blocks and the ones it ignores, each with the control that undoes it. The
//! section shows only while someone is blocked or ignored.

use limn::Element;

use rustcord_components::friends::ids;
use rustcord_components::user_settings as parts;
use rustcord_i18n::t;
use rustcord_model::{Relationship, RelationshipKind};
use rustcord_state::{Permit, Store};

use super::Ctx;

/// What a list undoes: a block, or the ignoring of someone.
#[derive(Clone, Copy)]
enum Undo {
    Unblock,
    Unignore,
}

impl Undo {
    /// The action the friends page sends, as `friend:{user}:{action}`.
    fn action(self) -> &'static str {
        match self {
            Self::Unblock => "unblock",
            Self::Unignore => "unignore",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Unblock => t("Desbloquear"),
            Self::Unignore => t("Dejar de ignorar"),
        }
    }
}

/// The people the account blocks, then those it ignores, each by name.
fn lists(store: &Store) -> (Vec<&Relationship>, Vec<&Relationship>) {
    let blocked = store
        .relationships()
        .unwrap_or_default()
        .into_iter()
        .filter(|known| known.kind == RelationshipKind::Blocked)
        .collect();
    (blocked, store.ignored())
}

/// Whether the section shows at all: someone is blocked or ignored.
pub(super) fn available(store: &Store) -> bool {
    let (blocked, ignored) = lists(store);
    !blocked.is_empty() || !ignored.is_empty()
}

/// The setting `key` of the section: each list, while it has someone.
pub(super) fn setting(ctx: &Ctx, key: &str) -> Option<Element> {
    let (blocked, ignored) = lists(ctx.store);
    let (title, people, undo) = match key {
        "blocked-users" => (t("Cuentas bloqueadas"), blocked, Undo::Unblock),
        "ignored-users" => (t("Cuentas ignoradas"), ignored, Undo::Unignore),
        _ => return None,
    };
    (!people.is_empty()).then(|| {
        Element::column(vec![
            parts::subtitle(title, ctx.look),
            parts::rows(
                people
                    .into_iter()
                    .map(|known| person(ctx, known, undo))
                    .collect(),
            ),
        ])
    })
}

/// One of them: the picture, the name and the handle, and the control that
/// undoes it, dimmed when the account may not.
fn person(ctx: &Ctx, known: &Relationship, undo: Undo) -> Element {
    let (store, look) = (ctx.store, ctx.look);
    let account = store.user(known.user);
    let name = known
        .nickname
        .clone()
        .or_else(|| account.map(|account| account.shown_name().to_owned()))
        .unwrap_or_else(|| t("Usuario desconocido").to_owned());
    let handle = account.map_or_else(|| known.user.to_string(), |account| account.name.clone());
    let social = store.social(known.user);
    let permit: &Permit = match undo {
        Undo::Unblock => &social.unblock,
        Undo::Unignore => &social.unignore,
    };
    let control = parts::secondary_button(
        &ids::person(known.user, undo.action()),
        undo.label(),
        permit.allowed(),
        look,
    );
    parts::person(
        &name,
        &handle,
        ctx.ui
            .picture(account.and_then(|account| account.picture())),
        account.map_or(look.faint, |account| look.ink(account.accent)),
        control,
        look,
    )
}
