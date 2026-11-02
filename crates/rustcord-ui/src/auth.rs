//! Provider-independent authorization before entering the account's workspace.

use limn::{
    Access, Button, ButtonSizing, ButtonVariant, Component, Element, Key, Role, Spinner, UiAction,
};
use rustcord_components::{auth, text, Look};
use rustcord_i18n::t;
use rustcord_model::UserId;
use rustcord_provider::{AuthenticationState, Command, ConnectionState, ProviderKind};
use rustcord_state::{Applied, Store};

use crate::{accounts, Intent, Ui, ViewContext};

/// Authentication ends at the workspace, including accounts with no conversations.
pub fn visible(store: &Store) -> bool {
    store
        .provider
        .as_ref()
        .is_some_and(|info| info.capabilities.authentication)
        && !matches!(store.authentication, AuthenticationState::SignedIn { .. })
}

pub fn on_applied(ui: &mut Ui, store: &Store, applied: &Applied) -> Option<Vec<Intent>> {
    if !matches!(applied, Applied::Authentication) {
        return None;
    }
    let focus = match &store.authentication {
        AuthenticationState::SignedOut | AuthenticationState::Failed { .. } => {
            let mut intents = crate::search::dismiss(ui);
            intents.extend(crate::context::cancel(ui));
            intents.extend(crate::call_sounds::stop_choosing(ui));
            // An unofficial provider's sign-in is its own login page: the
            // first time, open it directly instead of asking for a click,
            // unless there are remembered accounts to choose from.
            let open = !ui.sign_in_opened
                && store.accounts.is_empty()
                && matches!(store.authentication, AuthenticationState::SignedOut)
                && store
                    .provider
                    .as_ref()
                    .is_some_and(|info| info.kind == ProviderKind::Unofficial);
            *ui = fresh(ui, true);
            let first = match (&store.authentication, store.accounts.first()) {
                (AuthenticationState::SignedOut, Some(account)) => account_id(account.id),
                _ => "auth:sign-in".into(),
            };
            intents.extend([Intent::StopMeter, Intent::Focus(first)]);
            if open {
                intents.push(Intent::Command(Command::BeginSignIn));
            }
            return Some(intents);
        }
        AuthenticationState::AwaitingBrowser { .. } => "auth:browser",
        AuthenticationState::AwaitingExternal | AuthenticationState::Exchanging => "auth:cancel",
        AuthenticationState::SignedIn { .. } => "home",
        _ => return Some(Vec::new()),
    };
    Some(vec![Intent::Focus(focus.into())])
}

/// A new interface for another account or provider: what the person set for
/// the window stays, what the host found of the desktop and the window, and
/// the providers the host lists.
pub(crate) fn fresh(ui: &mut Ui, sign_in_opened: bool) -> Ui {
    Ui {
        theme: ui.theme,
        reduced_motion: ui.reduced_motion,
        show_metrics: ui.show_metrics,
        // Request IDs must not be reused if the same account signs in again:
        // an old invitation completion may still be draining through the host.
        requests_made: ui.requests_made,
        prefs: std::mem::take(&mut ui.prefs),
        providers: std::mem::take(&mut ui.providers),
        // The host reads the window's state once, before it opens.
        desktop: ui.desktop.kept(),
        // The themes are kept on this device.
        themes: ui.themes.kept(),
        sign_in_opened,
        ..Ui::new()
    }
}

/// The control that enters a remembered account.
fn account_id(account: UserId) -> String {
    format!("auth:account={}", account.0)
}

/// The remembered account `id` names, `prefix` stripped.
pub(crate) fn remembered(store: &Store, id: &str, prefix: &str) -> Option<UserId> {
    let account = UserId(id.strip_prefix(prefix)?.parse().ok()?);
    store
        .accounts
        .iter()
        .any(|known| known.id == account)
        .then_some(account)
}

pub fn update(store: &Store, action: &UiAction) -> Option<Vec<Intent>> {
    if !visible(store) {
        return None;
    }
    let UiAction::Clicked { id } = action else {
        return Some(Vec::new());
    };
    let choosing = matches!(
        store.authentication,
        AuthenticationState::SignedOut | AuthenticationState::Failed { .. }
    );
    if choosing {
        if let Some(account) = remembered(store, id, "auth:account=") {
            return Some(vec![Intent::Command(Command::UseAccount(Some(account)))]);
        }
        if id == "auth:other" {
            return Some(vec![Intent::Command(Command::UseAccount(None))]);
        }
    }
    let intent = match (id.as_str(), &store.authentication) {
        ("auth:sign-in", AuthenticationState::SignedOut | AuthenticationState::Failed { .. }) => {
            Intent::Command(Command::BeginSignIn)
        }
        ("auth:browser", AuthenticationState::AwaitingBrowser { url }) => {
            Intent::OpenUrl(url.clone())
        }
        (
            "auth:cancel",
            AuthenticationState::AwaitingBrowser { .. }
            | AuthenticationState::AwaitingExternal
            | AuthenticationState::Exchanging,
        ) => Intent::Command(Command::CancelSignIn),
        _ => return Some(Vec::new()),
    };
    Some(vec![intent])
}

pub fn key(store: &Store, key: Key, ctrl: bool) -> Option<Vec<Intent>> {
    if !visible(store) {
        return None;
    }
    if key == Key::Escape
        && matches!(
            store.authentication,
            AuthenticationState::AwaitingBrowser { .. }
                | AuthenticationState::AwaitingExternal
                | AuthenticationState::Exchanging
        )
    {
        return Some(vec![Intent::Command(Command::CancelSignIn)]);
    }
    // Leave Tab and activation keys to limn's accessible buttons.
    ctrl.then(Vec::new)
}

pub fn view(ui: &Ui, store: &Store, look: &Look, context: &ViewContext) -> Element {
    let inner = auth::content_width(context.window);
    let provider = store
        .provider
        .as_ref()
        .expect("the authorization page has a provider");
    let choosing = matches!(
        store.authentication,
        AuthenticationState::SignedOut | AuthenticationState::Failed { .. }
    );
    let (title, subtitle) = heading(
        &store.authentication,
        provider.kind,
        choosing && !store.accounts.is_empty(),
    );
    let mut content = body(ui, store, inner, look);
    // Where this comes from, always; an unofficial one also says what it risks.
    content
        .push(text(crate::providers::origin(provider), 13.0, look.muted).size(Some(inner), None));
    if provider.kind == ProviderKind::Unofficial {
        content
            .push(text(t(crate::providers::UNOFFICIAL), 13.0, look.idle).size(Some(inner), None));
    }
    if choosing && crate::providers::choosable(ui) {
        content.push(
            action(
                crate::providers::LIST,
                if ui.providers.open {
                    t("Ocultar los proveedores")
                } else {
                    t("Usar otro proveedor")
                },
                false,
                inner,
                look,
            )
            .variant(ButtonVariant::Ghost)
            .render(),
        );
        if ui.providers.open {
            content.extend(crate::providers::choices(ui, inner, look));
        }
    }
    auth::page(title, subtitle, content, context.window, look)
}

fn heading(
    state: &AuthenticationState,
    kind: ProviderKind,
    chooser: bool,
) -> (&'static str, &'static str) {
    if chooser {
        return (
            t("Elige una cuenta"),
            t("Selecciona una cuenta para iniciar sesión o agrega una nueva."),
        );
    }
    match state {
        AuthenticationState::Unavailable => (
            t("Preparando el acceso"),
            t("Enseguida puedes conectar tu cuenta."),
        ),
        AuthenticationState::SignedOut if kind == ProviderKind::Unofficial => (
            t("¡Te damos la bienvenida de nuevo!"),
            t("¡Nos alegra verte de nuevo!"),
        ),
        AuthenticationState::SignedOut => (
            t("¡Qué bueno verte!"),
            t("Conecta tu cuenta desde el navegador para continuar."),
        ),
        AuthenticationState::Failed { .. } => (
            t("No pudimos iniciar sesión"),
            t("Puedes volver a intentarlo cuando quieras."),
        ),
        AuthenticationState::AwaitingBrowser { .. } => (
            t("Continúa en tu navegador"),
            t("Autoriza el acceso y vuelve a Rustcord para continuar."),
        ),
        AuthenticationState::AwaitingExternal => (
            t("Continúa el inicio de sesión"),
            t("Completa el acceso en la ventana de inicio de sesión."),
        ),
        AuthenticationState::Exchanging => (t("Un momento…"), t("Estamos verificando tu cuenta.")),
        AuthenticationState::SignedIn { .. } => (t("Tu cuenta"), ""),
    }
}

fn action(id: &str, label: &str, primary: bool, width: f32, look: &Look) -> Button {
    Button::new(
        id,
        label,
        if primary {
            look.accent_palette()
        } else {
            rustcord_components::accounts::button_palette(look)
        },
    )
    .variant(if primary {
        ButtonVariant::Default
    } else {
        ButtonVariant::Secondary
    })
    .width(width)
    .sizing(ButtonSizing {
        height: 44.0,
        font_size: 15.0,
        icon_size: 16.0,
        horizontal_padding: 16.0,
        inner_spacing: 8.0,
    })
}

/// The login chooser: the remembered accounts, each entered with «Iniciar
/// sesión» or signed out from its «⋯», and «Agregar una cuenta».
fn chooser(ui: &Ui, store: &Store, width: f32, look: &Look) -> Vec<Element> {
    vec![
        rustcord_components::accounts::list(&accounts::login_rows(ui, store), width, look),
        action("auth:other", t("Agregar una cuenta"), false, width, look).render(),
    ]
}

fn body(ui: &Ui, store: &Store, width: f32, look: &Look) -> Vec<Element> {
    let cancel = || action("auth:cancel", t("Cancelar"), false, width, look).render();
    match &store.authentication {
        AuthenticationState::Unavailable => {
            vec![Spinner::new("auth:progress", look.accent)
                .render()
                .centered_x()]
        }
        AuthenticationState::AwaitingExternal => vec![
            Spinner::new("auth:progress", look.accent)
                .render()
                .centered_x(),
            cancel(),
        ],
        AuthenticationState::SignedOut | AuthenticationState::Failed { .. } => {
            let mut content = Vec::new();
            if let (AuthenticationState::SignedOut, ConnectionState::Disconnected { reason }) =
                (&store.authentication, &store.connection)
            {
                if reason != "Sesión cerrada" {
                    content.push(text(t(reason), 13.0, look.muted).size(Some(width), None));
                }
            }
            if let AuthenticationState::Failed { reason } = &store.authentication {
                content.push(
                    text(t(reason), 13.0, look.danger)
                        .size(Some((width - 24.0).max(0.0)), None)
                        .padding(12.0)
                        .background(look.raised)
                        .radius(6.0)
                        .accessible(Access::new(Role::Alert).id("auth:error").name(t(reason))),
                );
            }
            let failed = matches!(store.authentication, AuthenticationState::Failed { .. });
            if failed || store.accounts.is_empty() {
                let label = if failed {
                    t("Volver a intentar")
                } else {
                    t("Iniciar sesión")
                };
                content.push(action("auth:sign-in", label, true, width, look).render());
            }
            if !store.accounts.is_empty() {
                content.extend(chooser(ui, store, width, look));
            }
            content
        }
        AuthenticationState::AwaitingBrowser { .. } => vec![Element::column(vec![
            action("auth:browser", t("Abrir navegador"), true, width, look).render(),
            cancel(),
        ])
        .gap(8.0)],
        AuthenticationState::Exchanging => vec![
            Spinner::new("auth:progress", look.accent)
                .render()
                .centered_x(),
            cancel(),
        ],
        AuthenticationState::SignedIn { .. } => Vec::new(),
    }
}
