//! «Gestionar cuentas» and the popout's «Cambiar de cuenta», driven as a
//! person would: the popout, the dialog, its menus, the limit and the submenu.

use rustcord_model::UserId;
use rustcord_provider::{
    AuthenticationState, Capabilities, ProviderInfo, ProviderKind, RememberedAccount, Snapshot,
};

use super::*;

/// Signed in to a provider that remembers accounts `names` besides the open one.
fn host_remembering(names: &[(u64, &str)]) -> Host {
    let mut host = Host::new();
    host.store = Store::new();
    host.store.provider = Some(ProviderInfo {
        name: "Discord".into(),
        kind: ProviderKind::Unofficial,
        capabilities: Capabilities {
            authentication: true,
            saved_accounts: true,
            ..Capabilities::default()
        },
        notice: "Sesión personal de prueba.".into(),
    });
    host.event(Event::RememberedAccounts(
        names
            .iter()
            .map(|(id, name)| RememberedAccount {
                id: UserId(*id),
                name: (*name).into(),
                avatar: None,
            })
            .collect(),
    ));
    let user = host.provider.snapshot().me.unwrap();
    host.event(Event::Ready(Box::new(Snapshot {
        me: Some(user.clone()),
        users: vec![user],
        ..Snapshot::default()
    })));
    host.event(Event::Authentication(AuthenticationState::SignedIn {
        scopes: Vec::new(),
    }));
    host.sent.clear();
    host
}

/// Open «Gestionar cuentas» from the user panel's popout.
fn open_dialog(host: &mut Host) {
    host.click("me:status");
    host.click("pp:switch-account");
    assert!(host.ui.accounts.open && !host.ui.status_menu);
}

/// Put the pointer over `id`, as a person does to open a submenu.
fn hover(host: &mut Host, id: &str) {
    let frame = host.frame();
    let region = frame
        .regions
        .iter()
        .rev()
        .find(|region| region.id == id)
        .unwrap_or_else(|| panic!("{id} is not on screen"));
    let center = Point::new(
        region.rect.x + region.rect.width / 2.0,
        region.rect.y + region.rect.height / 2.0,
    );
    host.send(UiEvent::PointerMove(center));
}

#[test]
fn the_switch_row_opens_the_dialog_with_the_open_account_marked() {
    let mut host = host_remembering(&[(7, "Otra")]);
    open_dialog(&mut host);
    let text = host.painted().join(" ");
    assert!(text.contains("Gestionar cuentas") && text.contains("Agregar una cuenta"));
    assert!(text.contains("Cuenta activa") && text.contains("Otra"));
    assert!(host.sent.is_empty());
}

#[test]
fn switching_from_the_dialog_closes_it_and_asks_for_that_account() {
    let mut host = host_remembering(&[(7, "Otra")]);
    open_dialog(&mut host);
    host.click("acc:switch=7");
    assert_eq!(host.sent, vec![Command::UseAccount(Some(UserId(7)))]);
    assert!(!host.ui.accounts.open);
}

#[test]
fn signing_out_of_another_account_needs_no_question_and_says_so() {
    let mut host = host_remembering(&[(7, "Otra")]);
    open_dialog(&mut host);
    host.click("acc:more=7");
    host.click("acc:sign-out=7");
    assert_eq!(host.sent, vec![Command::ForgetAccount(UserId(7))]);
    assert!(host.ui.accounts.open);
    assert!(host.painted().join(" ").contains("Otra cerró sesión"));
}

#[test]
fn signing_out_of_the_open_account_goes_the_settings_way_and_closes() {
    let mut host = host_remembering(&[(7, "Otra")]);
    let me = host.provider.snapshot().me.unwrap().id.0;
    open_dialog(&mut host);
    host.click(&format!("acc:more={me}"));
    host.click(&format!("acc:sign-out={me}"));
    assert_eq!(host.sent, vec![Command::SignOut]);
    assert!(!host.ui.accounts.open);
}

#[test]
fn adding_an_account_below_the_limit_asks_for_a_login() {
    let mut host = host_remembering(&[(1, "A"), (2, "B"), (3, "C"), (4, "D")]);
    open_dialog(&mut host);
    host.click("acc:add");
    assert_eq!(host.sent, vec![Command::UseAccount(None)]);
}

#[test]
fn adding_an_account_at_the_limit_says_so_and_sends_nothing() {
    let mut host = host_remembering(&[(1, "A"), (2, "B"), (3, "C"), (4, "D"), (5, "E")]);
    open_dialog(&mut host);
    host.click("acc:add");
    assert!(host.sent.is_empty());
    assert!(host
        .painted()
        .join(" ")
        .contains("Solo puedes agregar 5 cuentas."));
}

#[test]
fn the_switch_row_submenu_lists_the_accounts_and_switches_from_it() {
    let mut host = host_remembering(&[(7, "Otra")]);
    host.click("me:status");
    hover(&mut host, "pp:switch-account");
    assert!(host.painted().join(" ").contains("Otra"));
    assert!(host
        .frame()
        .regions
        .iter()
        .any(|region| { region.id == "pp:manage-accounts" }));
    host.click("pp:account=7");
    assert_eq!(host.sent, vec![Command::UseAccount(Some(UserId(7)))]);
    assert!(!host.ui.status_menu);
}

#[test]
fn a_plain_click_on_more_opens_its_menu() {
    // The capture's clicks are plain `Clicked` actions, not pointer events.
    let mut host = host_remembering(&[(7, "Otra")]);
    open_dialog(&mut host);
    let action = UiAction::Clicked {
        id: "acc:more=7".into(),
    };
    let intents = update(&mut host.ui, &mut host.store, &action, Instant::now());
    host.run(intents);
    assert_eq!(host.ui.accounts.menu, Some(UserId(7)));
    assert!(host.painted().join(" ").contains("Cerrar sesión"));
}
