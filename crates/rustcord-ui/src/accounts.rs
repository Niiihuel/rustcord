//! «Gestionar cuentas», the dialog of the accounts this device remembers, and
//! the accounts «Cambiar de cuenta» lists in the popout (as
//! Discord's manage modal and submenu). Both list the same rows, and the
//! settings and the login chooser list them as well.

use limn::{Element, RasterImage, UiAction};
use rustcord_components::accounts::{self as parts, AccountRow, Notice};
use rustcord_components::profile::{ids as profile, SwitchItem};
use rustcord_components::Look;
use rustcord_i18n::{t, tr};
use rustcord_model::UserId;
use rustcord_provider::Command;
use rustcord_state::Store;

use crate::{Intent, Ui};

/// How many accounts this device remembers, as Discord allows.
pub const MAX_ACCOUNTS: usize = 5;

/// The dialog's state.
#[derive(Debug, Default)]
pub struct Dialog {
    pub open: bool,
    /// The account whose «⋯» menu is open.
    pub menu: Option<UserId>,
    /// What the dialog says under its title.
    pub notice: Option<Say>,
}

/// What the dialog says, written when it is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Say {
    /// «Agregar una cuenta» was pressed with the limit reached.
    Limit,
    /// The account with this name signed out here.
    SignedOut(String),
}

/// The provider remembers accounts, so the lists and their actions exist.
pub(crate) fn remembers(store: &Store) -> bool {
    store
        .provider
        .as_ref()
        .is_some_and(|info| info.capabilities.saved_accounts)
}

fn name(store: &Store, id: UserId) -> String {
    store
        .accounts
        .iter()
        .find(|known| known.id == id)
        .map(|known| known.name.clone())
        .filter(|name| !name.is_empty())
        .or_else(|| store.user(id).map(|user| user.shown_name().to_owned()))
        .unwrap_or_else(|| t("Cuenta guardada").to_owned())
}

/// The accounts the manager and the popout list, the open one first; one
/// the provider does not remember yet is listed anyway.
fn listed_ids(store: &Store) -> Vec<UserId> {
    let me = store.me();
    let mut ids: Vec<UserId> = store.accounts.iter().map(|known| known.id).collect();
    if let Some(me) = me.filter(|me| !ids.contains(me)) {
        ids.insert(0, me);
    }
    ids.sort_by_key(|id| Some(*id) != me);
    ids
}

/// One account as a list shows it: its name and picture as they are now.
fn shown(ui: &Ui, store: &Store, id: UserId) -> (String, Option<RasterImage>) {
    let remembered = store.accounts.iter().find(|known| known.id == id);
    let user = store.user(id);
    let avatar = remembered
        .and_then(|known| known.avatar.as_ref())
        .or_else(|| user.and_then(|user| user.avatar.as_ref()));
    (name(store, id), ui.picture(avatar))
}

/// The accounts of the popout's submenu, the open one first and checked.
pub(crate) fn entries(ui: &Ui, store: &Store) -> Vec<SwitchItem> {
    let me = store.me();
    listed_ids(store)
        .into_iter()
        .map(|id| {
            let (name, picture) = shown(ui, store, id);
            SwitchItem {
                id,
                name,
                picture,
                active: me == Some(id),
            }
        })
        .collect()
}

/// The rows of `ids`, each with the button `action` gives it, if any; the
/// open account is marked. Every row's «⋯» signs it out.
pub(crate) fn account_rows(
    ui: &Ui,
    store: &Store,
    ids: &[UserId],
    action: impl Fn(UserId, bool) -> Option<(String, String)>,
) -> Vec<AccountRow> {
    let me = store.me();
    ids.iter()
        .map(|&id| {
            let active = me == Some(id);
            let (name, picture) = shown(ui, store, id);
            AccountRow {
                name,
                picture,
                active,
                action: action(id, active),
                more: format!("acc:more={}", id.0),
                menu: ui.accounts.menu == Some(id),
                sign_out: format!("acc:sign-out={}", id.0),
                more_dismiss: "acc:more-dismiss".into(),
            }
        })
        .collect()
}

/// The dialog's rows: the open account marked, each other one with «Cambiar».
pub(crate) fn manager_rows(ui: &Ui, store: &Store) -> Vec<AccountRow> {
    account_rows(ui, store, &listed_ids(store), |id, active| {
        (!active).then(|| (format!("acc:switch={}", id.0), t("Cambiar").to_owned()))
    })
}

/// The login chooser's rows: «Iniciar sesión» on every remembered account,
/// and none marked open.
pub(crate) fn login_rows(ui: &Ui, store: &Store) -> Vec<AccountRow> {
    let ids: Vec<UserId> = store.accounts.iter().map(|known| known.id).collect();
    let mut rows = account_rows(ui, store, &ids, |id, _| {
        Some((
            format!("auth:account={}", id.0),
            t("Iniciar sesión").to_owned(),
        ))
    });
    for row in &mut rows {
        row.active = false;
    }
    rows
}

/// The clicks of the dialog and of the popout's accounts; `None` for others.
pub(crate) fn update(ui: &mut Ui, store: &Store, action: &UiAction) -> Option<Vec<Intent>> {
    let UiAction::Clicked { id } = action else {
        return None;
    };
    let id = id.as_str();
    Some(match id {
        profile::SWITCH_ACCOUNT | profile::MANAGE_ACCOUNTS => {
            close_popout(ui);
            ui.accounts = Dialog {
                open: true,
                ..Dialog::default()
            };
            Vec::new()
        }
        parts::ids::CLOSE => {
            ui.accounts = Dialog::default();
            Vec::new()
        }
        parts::ids::ADD => add(ui, store),
        "acc:more-dismiss" => {
            ui.accounts.menu = None;
            Vec::new()
        }
        _ => {
            if let Some(account) = listed(store, id, "acc:switch=") {
                switch(ui, store, account)
            } else if let Some(account) = listed(store, id, "acc:more=") {
                ui.accounts.menu = (ui.accounts.menu != Some(account)).then_some(account);
                Vec::new()
            } else if let Some(account) = listed(store, id, "acc:sign-out=") {
                sign_out(ui, store, account)
            } else {
                let account = listed(store, id, profile::ACCOUNT)?;
                close_popout(ui);
                switch(ui, store, account)
            }
        }
    })
}

/// The account `id` names after `prefix`, when this device lists it.
fn listed(store: &Store, id: &str, prefix: &str) -> Option<UserId> {
    let account = UserId(id.strip_prefix(prefix)?.parse().ok()?);
    (store.me() == Some(account) || store.accounts.iter().any(|known| known.id == account))
        .then_some(account)
}

fn close_popout(ui: &mut Ui) {
    ui.status_menu = false;
    ui.profiles.status_open = false;
}

/// Changes to `account` and closes the dialog; the open one only closes it.
fn switch(ui: &mut Ui, store: &Store, account: UserId) -> Vec<Intent> {
    ui.accounts = Dialog::default();
    if store.me() == Some(account) || !remembers(store) || !crate::settings::can_sign_out(store) {
        return Vec::new();
    }
    // The sign-out that comes first must not open the login window too.
    ui.sign_in_opened = true;
    vec![Intent::Command(Command::UseAccount(Some(account)))]
}

/// «Agregar una cuenta»: with the limit reached it says so; below it the
/// login opens for a new account.
fn add(ui: &mut Ui, store: &Store) -> Vec<Intent> {
    if store.accounts.len() >= MAX_ACCOUNTS {
        ui.accounts.notice = Some(Say::Limit);
        return Vec::new();
    }
    ui.accounts = Dialog::default();
    if !remembers(store) || !crate::settings::can_sign_out(store) {
        return Vec::new();
    }
    ui.sign_in_opened = true;
    vec![Intent::Command(Command::UseAccount(None))]
}

/// «Cerrar sesión»: the open account signs out as settings does; another is
/// forgotten at once, with no question, and the dialog says so.
fn sign_out(ui: &mut Ui, store: &Store, account: UserId) -> Vec<Intent> {
    ui.accounts.menu = None;
    if store.me() == Some(account) {
        ui.accounts = Dialog::default();
        return if crate::settings::can_sign_out(store) {
            vec![Intent::Command(Command::SignOut)]
        } else {
            Vec::new()
        };
    }
    if !remembers(store) {
        return Vec::new();
    }
    // The dialog says so when it is open; the login chooser forgets quietly.
    if ui.accounts.open {
        ui.accounts.notice = Some(Say::SignedOut(name(store, account)));
    }
    vec![Intent::Command(Command::ForgetAccount(account))]
}

/// What the dialog says under its title, as the settings say it too.
pub(crate) fn notice(ui: &Ui) -> Option<Notice> {
    ui.accounts.notice.as_ref().map(|say| match say {
        Say::Limit => Notice {
            text: tr!("Solo puedes agregar {max} cuentas.", max = MAX_ACCOUNTS),
            critical: true,
        },
        Say::SignedOut(name) => Notice {
            text: tr!("{name} cerró sesión", name),
            critical: false,
        },
    })
}

/// The dialog over `root`, while it is open.
pub(crate) fn overlay(ui: &Ui, store: &Store, root: Element, look: &Look) -> Element {
    if !ui.accounts.open {
        return root;
    }
    parts::manage(root, &manager_rows(ui, store), notice(ui).as_ref(), look)
}
