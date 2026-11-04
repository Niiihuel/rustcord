//! What the settings show: the panel over the window, its sidebar, each
//! page, and the search through them; and the quick switcher.
//!
//! Every page is listed in [`SECTIONS`] with its settings and the words they
//! are found by. A page's column is built from that list: each setting has
//! a key, and [`setting`] draws it. Searching keeps the settings whose words
//! match, the sections and pages that hold them, and their links.

use limn::{
    AlertDialog, Button, ButtonVariant, Color, Component, DropdownOption, Element, Icon, Kbd,
    ListRow, Modal, PopoverPlacement, ScrollArea, SearchField, Select, Size, TextField,
};
use rustcord_components::quick_voice as quick;
use rustcord_components::server_settings::{swatches, swatches_width, Swatches};
use rustcord_components::user_settings::{
    self as parts, Account, Geometry, Nav, NavAction, NavAnchor, NavPage, Preview,
};
use rustcord_components::{line, strong_line, text, Look};
// `Language` is a page here; the interface's languages are `I18n`.
use rustcord_i18n::{es, t, tr, Language as I18n};
use rustcord_model::ChannelKind;
use rustcord_provider::{ProviderKind, VoiceSupport};
use rustcord_state::Store;

use super::{
    anchor, destinations, profile_locked, SettingsPage, ACCENT, GROUP_GAPS, NAME, SEARCH,
    SWITCHER_QUERY, TEXT_SIZES, ZOOMS,
};
use crate::{GlobalKeys, Ui, ViewContext};

use SettingsPage::*;

mod desktop;
// Call sounds (audio).
mod sounds;

mod image_cache;
mod reading;

pub(super) mod cosmetics;

mod games;

mod shortcuts;

mod themes;

mod updates;
// Who is blocked or ignored, and the controls that undo it.
mod restricted;

/// A section of a page: its key, its title, and its settings, each a key
/// and the words it is found by besides its name.
struct Section {
    page: SettingsPage,
    key: &'static str,
    title: &'static str,
    rows: &'static [(&'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    // Discord's account info first, with the username first; Rustcord's rows follow.
    Section {
        page: Account,
        key: "account-info",
        title: es("Información de cuenta"),
        rows: &[
            ("handle", es("nombre de usuario handle arroba")),
            ("display-name", es("nombre para mostrar apodo cómo te ven")),
            ("accent", es("color del avatar inicial foto imagen")),
            ("profile", es("editar perfil pronombres biografía sobre mí")),
        ],
    },
    Section {
        page: Account,
        key: "account-accounts",
        title: es("Cuentas"),
        rows: &[(
            "accounts",
            es("cambiar de cuenta agregar otra cuenta cuentas recordadas varias gestionar cerrar sesión"),
        )],
    },
    // The profile's page: found by what it edits; the page itself is the editor.
    Section {
        page: Profiles,
        key: "profiles",
        title: es("Perfiles"),
        rows: &[(
            "profiles",
            es("perfil nombre para mostrar pronombres biografía sobre mí avatar banner color decoración efecto apodo por servidor"),
        )],
    },
    // The device's session, and what this device keeps of its data (Discord's
    // danger row sits here, under the device it is about).
    Section {
        page: Account,
        key: "account-device",
        title: es("Este dispositivo"),
        rows: &[
            (
                "device",
                es("dispositivo sesión recordada llavero olvidar la sesión restaurar al abrir"),
            ),
            (
                "forget",
                es("borrar lo guardado en este equipo caché borradores instantánea eliminar datos"),
            ),
        ],
    },
    // Messaging permissions, as Discord's order (the account's synced settings).
    Section {
        page: Messaging,
        key: "content-filters",
        title: es("Filtros de contenido"),
        rows: &[(
            "explicit-content",
            es("contenido explícito imágenes difuminar bloquear sensible"),
        )],
    },
    Section {
        page: Messaging,
        key: "spam-filters",
        title: es("Filtros de spam"),
        rows: &[("spam-filter", es("filtro de spam mensajes directos carpeta"))],
    },
    Section {
        page: Messaging,
        key: "messaging",
        title: es("Permisos de mensajes directos (MD)"),
        rows: &[
            (
                "server-dms",
                es("permitir mensajes directos de miembros de servidores privacidad"),
            ),
            (
                "message-requests",
                es("solicitudes de mensajes filtrar desconocidos privacidad"),
            ),
        ],
    },
    Section {
        page: Messaging,
        key: "friend-requests",
        title: es("Permisos de solicitud de amistad"),
        rows: &[(
            "friend-requests",
            es("quién puede mandarte solicitudes de amistad amigos de amigos"),
        )],
    },
    Section {
        page: Messaging,
        key: "ignore-block",
        title: es("Ignorar y bloquear"),
        rows: &[
            ("blocked-users", es("cuentas bloqueadas desbloquear personas bloqueadas")),
            ("ignored-users", es("cuentas ignoradas dejar de ignorar personas ignoradas")),
        ],
    },
    // Discord's voice encryption category comes first; the rest is Rustcord's own.
    Section {
        page: Privacy,
        key: "e2ee",
        title: es("Cifrado de extremo a extremo de voz"),
        rows: &[(
            "privacy-code",
            es("código de privacidad cifrado de extremo a extremo dave verificar llamada seguridad"),
        )],
    },
    Section {
        page: Notifications,
        key: "overview",
        title: es("Vista general"),
        rows: &[
            ("desktop", es("activar las notificaciones de escritorio sistema avisos")),
            (
                "notification-content",
                es("mostrar el contenido del mensaje privacidad ocultar texto"),
            ),
        ],
    },
    // Discord's Sounds category, with the message sound as its first row.
    Section {
        page: Notifications,
        key: "sounds",
        title: es("Sonidos"),
        rows: &[(
            "message-sound",
            es("sonido de mensajes nuevo mensaje menciones privados tono"),
        )],
    },
    Section {
        page: Notifications,
        key: "badges",
        title: es("Insignias"),
        rows: &[
            (
                "unread-badges",
                es("marcar servidores con mensajes sin leer insignia riel no leídos"),
            ),
            ("mention-badges", es("número de menciones insignia globo rojo")),
        ],
    },
    Section {
        page: Voice,
        key: "voice",
        // Discord's «Voz»: the devices and their levels, the test, the input
        // mode and the sensitivity, with push-to-talk's key and delay.
        title: es("Voz"),
        rows: &[
            (
                "devices",
                es("micrófono altavoz dispositivo de entrada salida auriculares parlantes volumen ganancia actualizar restablecer niveles"),
            ),
            (
                "volumes",
                es("probar sonido prueba de salida altavoz auriculares sonido de la llamada"),
            ),
            (
                "meter",
                es("prueba de micrófono nivel medidor probar comprobar entrada procesada"),
            ),
            (
                "loopback",
                es("escucharme oírme retorno monitor auriculares acople"),
            ),
            (
                "input-mode",
                es("modo de entrada detección actividad de voz pulsar para hablar ptt"),
            ),
            (
                "sensitivity",
                es("sensibilidad de entrada automáticamente umbral detección de voz indicador habla"),
            ),
            (
                "ptt-key",
                es("atajo tecla pulsar para hablar grabar global fuera de la ventana ptt"),
            ),
            (
                "ptt-release",
                es("retraso al soltar pulsar para hablar demora ptt"),
            ),
        ],
    },
    Section {
        page: Voice,
        key: "input-profile",
        title: es("Perfil de entrada"),
        rows: &[
            (
                "input-profile",
                es("perfil aislamiento de voz estudio personalizado procesamiento"),
            ),
            (
                "noise-suppression",
                es("supresión reducción de ruido fondo krisp rnnoise deepfilternet webrtc estándar teclado ventilador"),
            ),
            (
                "echo-cancellation",
                es("cancelación de eco acople altavoces parlantes referencia"),
            ),
            (
                "system-echo",
                es("cancelar el sonido de la pc juegos música videos eco referencia del sistema salida"),
            ),
            (
                "automatic-gain",
                es("control automático de ganancia agc volumen nivel parejo"),
            ),
        ],
    },
    Section {
        page: Voice,
        key: "video",
        title: es("Cámara"),
        rows: &[(
            "camera",
            es("cámara web webcam dispositivo de video probar vista previa verse"),
        )],
    },
    Section {
        page: Voice,
        key: "call-sounds",
        title: es("Sonidos"),
        // Call sounds (audio): drawn in `sounds`.
        rows: sounds::ROWS,
    },
    Section {
        page: Voice,
        key: "advanced",
        title: es("Avanzado"),
        rows: &[(
            "video-hardware",
            es("aceleración por hardware gpu h264 codificador decodificador va-api nvenc nvdec openh264 procesador"),
        )],
    },
    Section {
        page: Appearance,
        key: "theme",
        title: es("Tema"),
        rows: themes::ROWS,
    },
    Section {
        page: Appearance,
        key: "messages",
        title: es("Mensajes"),
        rows: &[
            (
                "image-previews",
                es("vista previa de las imágenes subidas archivos adjuntos fotos"),
            ),
            ("reactions", es("mostrar reacciones de emojis")),
            ("avatars", es("mostrar los avatares de los usuarios compacto")),
            (
                "related-accessibility",
                es("ajustes relacionados accesibilidad tamaño densidad zoom"),
            ),
        ],
    },
    // Discord's «Cuadro de chat»; the emoticons were in «Mensajes».
    Section {
        page: Appearance,
        key: "chat-box",
        title: es("Cuadro de chat"),
        rows: &[(
            "emoticons",
            es("convertir emoticonos en emojis automáticamente caritas :) al enviar"),
        )],
    },
    // The profile cosmetics are drawn in `cosmetics`.
    Section {
        page: Appearance,
        key: "cosmetics",
        title: es("Cosméticos de los perfiles"),
        rows: cosmetics::ROWS,
    },
    Section {
        page: Accessibility,
        key: "readability",
        title: es("Legibilidad del texto"),
        rows: &[("text-size", es("tamaño de texto en el chat fuente letra escala"))],
    },
    Section {
        page: Accessibility,
        key: "density",
        title: es("Densidad visual"),
        rows: &[
            (
                "display",
                es("mostrar el mensaje del chat compacto predeterminado avatares"),
            ),
            ("group-gap", es("espacio entre grupos de mensajes separación")),
            (
                "zoom",
                es("nivel de zoom tamaño de la interfaz escala acercar alejar ctrl"),
            ),
        ],
    },
    Section {
        page: Accessibility,
        key: "color",
        title: es("Color y contraste"),
        rows: &[
            ("role-colors", es("colores de rol nombres")),
            (
                "related-appearance",
                es("apariencia cambia el tema de la aplicación tema colores"),
            ),
        ],
    },
    Section {
        page: Accessibility,
        key: "motion",
        title: es("Movimiento reducido"),
        rows: &[
            (
                "motion",
                es("activar el movimiento reducido animaciones desplazamiento suave"),
            ),
            (
                "animate-gifs",
                es("animar gif animaciones reproducir solos automáticamente pasar el puntero stickers emoji"),
            ),
        ],
    },
    Section {
        page: Language,
        key: "language",
        title: es("Idioma"),
        rows: &[("language", es("selecciona un idioma español inglés lengua"))],
    },
    Section {
        page: Language,
        key: "clock",
        title: es("Formato de tiempo"),
        rows: &[(
            "clock",
            es("formato de hora 12 horas 24 horas reloj a. m. p. m. tiempo"),
        )],
    },
    Section {
        page: Connection,
        key: "provider",
        title: es("Proveedor"),
        rows: &[
            ("provider", es("proveedor conexión simulación servicio discord")),
            (
                "choose",
                es("elegir cambiar proveedor simulación oficial oauth2 sesión no oficial experimento"),
            ),
            (
                "capabilities",
                es("capacidades enviar editar borrar reacciones historial adjuntos presencia voz"),
            ),
            (
                "related-developer",
                es("ajustes relacionados pruebas de la simulación desarrollador"),
            ),
        ],
    },
    Section {
        page: Developer,
        key: "developer",
        title: es("Desarrollador"),
        rows: &[
            ("developer", es("modo desarrollador copiar id identificador")),
            (
                "metrics",
                es("mostrar métricas rendimiento cuadros memoria cpu f12"),
            ),
        ],
    },
    Section {
        page: Developer,
        key: "provider-notes",
        title: es("Proveedor"),
        rows: &[(
            "unavailable-providers",
            es("discord oauth2 id público cliente variable de entorno compilación no disponible"),
        )],
    },
    Section {
        page: Developer,
        key: "simulation",
        title: es("Pruebas de la simulación"),
        rows: &[
            ("disconnect", es("simular una desconexión reconexión cortar")),
            (
                "fail-send",
                es("rechazar el próximo envío fallo error reintentar"),
            ),
            (
                "silent-send",
                es("no responder el próximo envío silencio incierto"),
            ),
            ("ring-me", es("recibir una llamada entrante sonar")),
            (
                "answer-elsewhere",
                es("atender en otro dispositivo llamada"),
            ),
        ],
    },
];

/// A line of the sidebar: a page, or the action that signs out.
#[derive(Clone, Copy)]
enum Entry {
    Page(SettingsPage),
    LogOut,
}

/// The pages in the sidebar's order, in groups as Discord's settings have
/// them: a heading (hidden ones are only read out), then its lines.
const GROUPS: &[(&str, bool, &[Entry])] = &[
    (
        es("General"),
        false,
        &[
            Entry::Page(Account),
            Entry::Page(Profiles),
            Entry::Page(Privacy),
            Entry::Page(Messaging),
            Entry::Page(Notifications),
            Entry::Page(Connection),
        ],
    ),
    (
        es("Experiencia"),
        true,
        &[
            Entry::Page(Voice),
            Entry::Page(Appearance),
            Entry::Page(Accessibility),
            Entry::Page(System),
            Entry::Page(Language),
        ],
    ),
    (
        es("Juegos y aplicaciones"),
        true,
        &[Entry::Page(Games), Entry::Page(Activity)],
    ),
    (es("Otro"), false, &[Entry::Page(Developer), Entry::LogOut]),
];

fn icon_of(page: SettingsPage) -> Icon {
    match page {
        Account => Icon::Custom(parts::PERSON),
        Profiles => Icon::Custom(parts::PERSON),
        Privacy => Icon::Custom(parts::SHIELD),
        Messaging => Icon::Chat,
        Notifications => Icon::Bell,
        Connection => Icon::Puzzle,
        Voice => Icon::Microphone,
        Appearance => Icon::Palette,
        Accessibility => Icon::Sliders,
        System => Icon::Laptop,
        Language => Icon::Globe,
        Games => Icon::Gamepad,
        Activity => Icon::Gamepad,
        Developer => Icon::Code,
    }
}

/// The keys of every section of `page`, in order.
pub(crate) fn section_keys(page: SettingsPage) -> Vec<&'static str> {
    sections()
        .filter(|section| section.page == page)
        .map(|section| section.key)
        .collect()
}

/// The section of `page` named `key`.
pub(crate) fn section_key(page: SettingsPage, key: &str) -> Option<&'static str> {
    section_keys(page).into_iter().find(|found| *found == key)
}

/// The anchor at the top of `page`.
pub(crate) fn top(page: SettingsPage) -> String {
    format!("us:top:{}", page.key())
}

/// How far under the top of the page a section jumped to stands: under the
/// preview where there is one.
pub(crate) fn jump_margin(page: SettingsPage) -> f32 {
    if page == Accessibility {
        parts::PREVIEW_HEIGHT + 56.0
    } else {
        24.0
    }
}

/// Lower case and without accents, to compare what is searched.
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// Whether every word of `query` is in one of `texts`.
fn found(query: &str, texts: &[&str]) -> bool {
    let haystack = fold(&texts.join(" "));
    fold(query)
        .split_whitespace()
        .all(|word| haystack.contains(word))
}

/// Whether the provider is the simulation.
fn simulated(store: &Store) -> bool {
    store
        .provider
        .as_ref()
        .is_some_and(|info| info.kind == ProviderKind::Simulated)
}

/// Every section, in the order each page shows them: the pages' own first,
/// then the sections the desktop work and the reading ones
/// add, in the order that keeps Discord's on each page.
fn sections() -> impl Iterator<Item = &'static Section> {
    SECTIONS
        .iter()
        // «General» of «Sistema» first, then its keys and «Avanzados», as
        // Discord's; Rustcord's own updates come after them.
        .chain(desktop::GENERAL)
        .chain(shortcuts::SECTIONS)
        // The cache of Privacy comes before the call's encryption.
        .chain(image_cache::SECTIONS)
        .chain(desktop::SECTIONS)
        .chain(updates::SECTIONS)
        .chain(reading::SECTIONS)
}

/// The line under a section's title, where it has one.
fn note(key: &str) -> Option<&'static str> {
    Some(match key {
        "shortcuts" => es("Los atajos de teclado son combinaciones personalizadas que te permiten realizar acciones específicas en Rustcord sin necesidad de hacer clic en los menús."),
        "friend-requests" => es("Controla quién puede enviarte solicitudes de amistad y cómo aparecen."),
        _ => return None,
    })
}

/// The name a section has in the sidebar's links, where it is shorter than
/// its title.
fn link_label(section: &Section) -> &'static str {
    match section.key {
        "messaging" => es("Mensajes directos"),
        "friend-requests" => es("Solicitudes de amistad"),
        _ => section.title,
    }
}

/// Whether a section can show at all: the simulation's tests only with it.
fn available(section: &Section, ui: &Ui, store: &Store) -> bool {
    match section.key {
        "simulation" => simulated(store),
        "provider-notes" => ui.providers.options.iter().any(|o| o.unavailable.is_some()),
        "account-accounts" => super::can_sign_out(store),
        "ignore-block" => restricted::available(store),
        key => desktop::available(key, store),
    }
}

/// The sections of `page` that show for `query`, each with the keys of its
/// settings that do: all of them for an empty query or a title that matches.
fn shown(
    page: SettingsPage,
    query: &str,
    ui: &Ui,
    store: &Store,
) -> Vec<(&'static Section, Vec<&'static str>)> {
    let query = query.trim();
    sections()
        .filter(|section| section.page == page && available(section, ui, store))
        .filter_map(|section| {
            // The texts as shown, so a search in either language finds them.
            let title = t(section.title);
            let whole = query.is_empty() || found(query, &[page.title(), title]);
            let rows: Vec<&'static str> = section
                .rows
                .iter()
                .filter(|(key, words)| whole || found(query, &[page.title(), title, key, t(words)]))
                .map(|(key, _)| *key)
                .collect();
            (!rows.is_empty()).then_some((section, rows))
        })
        .collect()
}

fn searching(ui: &Ui) -> Option<&str> {
    Some(ui.settings_state.query.trim()).filter(|query| !query.is_empty())
}

/// While searching, the first section of `page` that matches.
pub(crate) fn first_match(ui: &Ui, store: &Store, page: SettingsPage) -> Option<&'static str> {
    let query = searching(ui)?;
    shown(page, query, ui, store)
        .first()
        .map(|(section, _)| section.key)
}

/// While searching, the first page that has something that matches.
pub(crate) fn first_page(ui: &Ui, store: &Store) -> Option<SettingsPage> {
    let query = searching(ui)?;
    SettingsPage::ALL
        .into_iter()
        .find(|page| !shown(*page, query, ui, store).is_empty())
}

/// What every part of the view reads.
struct Ctx<'a> {
    ui: &'a Ui,
    store: &'a Store,
    look: &'a Look,
    geometry: Geometry,
    context: &'a ViewContext,
}

/// The settings over `root` while they are open.
pub fn modal(ui: &Ui, store: &Store, root: Element, look: &Look, context: &ViewContext) -> Element {
    let Some(page) = ui.settings else { return root };
    let ctx = Ctx {
        ui,
        store,
        look,
        geometry: Geometry::new(context.window),
        context,
    };
    let me = store.me_user();
    let account = Account {
        image: me.and_then(|me| ui.picture(me.picture())),
        name: me.map_or_else(
            || t("Sin cuenta").to_owned(),
            |me| me.shown_name().to_owned(),
        ),
        accent: me.map_or(look.faint, |me| look.ink(me.accent)),
        edit: Some("profile:edit".into()),
    };
    let provider = store
        .provider
        .as_ref()
        .map_or(t("sin proveedor"), |info| t(&info.name));
    let footnote = format!("Rustcord {} · {provider}", env!("CARGO_PKG_VERSION"));
    let sidebar = parts::sidebar(
        &account,
        parts::search_box(SEARCH, &ui.settings_state.query, look),
        &navigation(&ctx, page),
        &footnote,
        look,
    );
    let card = parts::panel(
        ctx.geometry,
        sidebar,
        page.title(),
        "us:close",
        content(&ctx, page),
        look,
    );
    let settings = Element::popover(root, card, "us:close", PopoverPlacement::Center)
        // The search, not the first link: a focus ring on «Editar perfil»
        // is the first thing seen otherwise.
        .popover_initial_focus(SEARCH)
        .popover_scrim(Color::rgba(0, 0, 0, 184))
        .popover_backdrop_dismiss(true)
        .popover_animation_ms(150)
        .popover_open_scale(0.9);
    // Clearing what this device keeps asks first; the settings themselves stay.
    if ui.settings_state.forget_asked {
        return AlertDialog::new(
            settings,
            true,
            t("Borrar datos locales"),
            t("Mensajes, borradores e imágenes guardadas de esta cuenta. Tus ajustes se conservan."),
            "us:forget-yes",
            "us:forget-cancel",
            look.menu_palette(),
        )
        .confirm_label(t("Borrar"))
        .cancel_label(t("Cancelar"))
        .destructive(look.danger)
        .render();
    }
    if !(ui.settings_state.log_out_asked && super::can_sign_out(store)) {
        return settings;
    }
    // Signing out asks first, as Discord's question does: what it does, and
    // the way back.
    AlertDialog::new(
        settings,
        true,
        t("Cerrar sesión"),
        t("¿Realmente quieres cerrar sesión?"),
        "us:sign-out",
        "us:log-out:cancel",
        look.menu_palette(),
    )
    .confirm_label(t("Cerrar sesión"))
    .cancel_label(t("Cancelar"))
    .destructive(look.danger)
    .render()
}

/// The sidebar's pages; while searching, only those with something that
/// matches. The open page lists its sections.
fn navigation(ctx: &Ctx, open: SettingsPage) -> Vec<Nav> {
    let query = searching(ctx.ui).unwrap_or("");
    let mut nav = Vec::new();
    for (heading, titled, group) in GROUPS {
        let entries: Vec<Nav> = group
            .iter()
            .filter_map(|entry| {
                let page = match entry {
                    Entry::Page(page) => *page,
                    Entry::LogOut => return log_out(ctx, query),
                };
                let sections = shown(page, query, ctx.ui, ctx.store);
                if !query.is_empty() && sections.is_empty() {
                    return None;
                }
                let selected = page == open;
                let mut anchors = Vec::new();
                if selected && (sections.len() > 1 || !query.is_empty()) {
                    let active = ctx
                        .ui
                        .settings_state
                        .section
                        .or(sections.first().map(|(section, _)| section.key));
                    anchors = sections
                        .iter()
                        .map(|(section, _)| NavAnchor {
                            id: format!("us:jump={}", section.key),
                            label: t(link_label(section)).to_owned(),
                            active: active == Some(section.key),
                        })
                        .collect();
                }
                Some(Nav::Page(NavPage {
                    id: format!("us:page={}", page.key()),
                    label: page.title().to_owned(),
                    icon: icon_of(page),
                    selected,
                    anchors,
                }))
            })
            .collect();
        if entries.is_empty() {
            continue;
        }
        if !nav.is_empty() {
            nav.push(Nav::Separator);
        }
        nav.push(if *titled {
            Nav::Heading(t(heading).to_owned())
        } else {
            Nav::HiddenHeading(t(heading).to_owned())
        });
        nav.extend(entries);
    }
    if nav.is_empty() {
        nav.push(Nav::Empty);
    }
    nav
}

/// The «Cerrar sesión» line, where the account can sign out and the search,
/// if any, finds it.
fn log_out(ctx: &Ctx, query: &str) -> Option<Nav> {
    let label = t("Cerrar sesión");
    let shown = super::can_sign_out(ctx.store) && (query.is_empty() || found(query, &[label]));
    shown.then(|| {
        Nav::Action(NavAction {
            id: "us:log-out".into(),
            label: label.to_owned(),
            icon: Icon::Leave,
        })
    })
}

/// The open page's column. While searching it shows what matches there,
/// unless nothing does: then the page shows whole.
fn content(ctx: &Ctx, page: SettingsPage) -> Element {
    if page == Profiles {
        return profiles(ctx);
    }
    let query = searching(ctx.ui).unwrap_or("");
    let mut sections = shown(page, query, ctx.ui, ctx.store);
    if sections.is_empty() {
        sections = shown(page, "", ctx.ui, ctx.store);
    }
    let sections = sections
        .into_iter()
        .filter_map(|(section, keys)| {
            let rows: Vec<Element> = keys
                .into_iter()
                .filter_map(|key| setting(ctx, key))
                .collect();
            (!rows.is_empty()).then(|| {
                parts::section(
                    &anchor(section.key),
                    t(section.title),
                    note(section.key).map(t),
                    None,
                    rows,
                    ctx.look,
                )
            })
        })
        .collect();
    let sticky = (page == Accessibility).then(|| preview(ctx));
    parts::page(
        &page.scroll_id(),
        &top(page),
        ctx.geometry,
        sticky,
        sections,
        ctx.look,
    )
}

/// The «Perfiles» page: the profile's editor, as wide as the column and
/// as tall as the page, centred under the header.
fn profiles(ctx: &Ctx) -> Element {
    let height = (ctx.geometry.panel.height - parts::HEADER_HEIGHT - 1.0).max(1.0);
    let size = Size::new(ctx.geometry.column, height);
    let page = crate::profile::settings_page(ctx.ui, ctx.store, size, ctx.look)
        .unwrap_or_else(|| Element::spacer().size(Some(size.width), Some(size.height)));
    Element::row(vec![
        Element::spacer().flex(1.0),
        page.anchor(top(Profiles)),
        Element::spacer().flex(1.0),
    ])
}

/// The preview over the accessibility settings: two messages of the
/// account as the timeline would show them now.
fn preview(ctx: &Ctx) -> Element {
    let look = ctx.look;
    let prefs = &ctx.ui.prefs;
    let me = ctx.store.me_user();
    let accent = me.map_or(look.accent, |me| look.ink(me.accent));
    parts::preview(
        &Preview {
            name: me.map_or_else(|| t("Invitado").to_owned(), |me| me.shown_name().to_owned()),
            ink: if prefs.role_colors { accent } else { look.text },
            accent,
            style: prefs.message_style(),
            now: ctx.context.wall,
        },
        ctx.geometry.column,
        look,
    )
}

/// The setting `key`, as its page shows it; `None` when there is nothing to
/// show for it.
fn setting(ctx: &Ctx, key: &str) -> Option<Element> {
    let (ui, store, look) = (ctx.ui, ctx.store, ctx.look);
    let prefs = &ui.prefs;
    let width = ctx.geometry.column;
    let toggle = |id: &str, label: &str, note: Option<&str>, on: bool| {
        parts::toggle(id, label, note, on, None, look)
    };
    let stacked = |label: &str, note: Option<&str>, below: Element| {
        Element::column(vec![
            parts::setting(label, note, None, true, look),
            below,
            Element::spacer().size(None, Some(12.0)),
        ])
    };
    let open = ui.settings_state.select;
    if let Some(shown) = crate::account::privacy_setting(store, key, look, open) {
        return Some(shown);
    }
    if let Some(shown) = restricted::setting(ctx, key) {
        return Some(shown);
    }
    if let Some(shown) = desktop::setting(ctx, key) {
        return shown;
    }
    if let Some(shown) = sounds::setting(ctx, key) {
        return Some(shown);
    }
    if let Some(shown) = reading::setting(ctx, key).or_else(|| image_cache::setting(ctx, key)) {
        return Some(shown);
    }
    if let Some(shown) = shortcuts::setting(ctx, key) {
        return Some(shown);
    }
    if let Some(shown) = cosmetics::setting(ctx, key) {
        return Some(shown);
    }
    if let Some(shown) = themes::setting(ctx, key) {
        return shown;
    }
    if let Some(shown) = updates::setting(ctx, key) {
        return Some(shown);
    }
    Some(match key {
        "display-name" => display_name(ctx),
        "handle" => parts::setting(
            t("Nombre de usuario"),
            Some(t("No se puede cambiar.")),
            Some(line(
                format!("@{}", store.me_user()?.name),
                16.0,
                look.muted,
            )),
            true,
            look,
        ),
        "accent" => {
            let locked = profile_locked(store);
            Element::column(vec![
                parts::setting(
                    t("Color del avatar"),
                    Some(locked.unwrap_or(
                        t("Tu inicial se muestra sobre este color."),
                    )),
                    None,
                    locked.is_none(),
                    look,
                ),
                swatches(
                    &Swatches {
                        prefix: ACCENT.into(),
                        chosen: store.me_user().map(|me| me.accent),
                        default: None,
                        custom: ui.settings_state.accent_hex.clone(),
                        enabled: locked.is_none(),
                        narrow: width < swatches_width(false),
                        picked: crate::color_picker::picked(ui, ACCENT),
                    },
                    look,
                ),
                Element::spacer().size(None, Some(12.0)),
            ])
        }
        "profile" => parts::setting(
            t("Perfil"),
            Some(t("Pronombres, biografía y cómo te ven los demás.")),
            Some(parts::secondary_button("profile:edit", t("Editar perfil"), true, look)),
            true,
            look,
        ),
        "accounts" => {
            store
                .provider
                .as_ref()
                .filter(|info| info.capabilities.saved_accounts)?;
            let mut content = Vec::new();
            if let Some(notice) = crate::accounts::notice(ui) {
                content.push(rustcord_components::accounts::notice_panel(&notice, look));
            }
            let rows = crate::accounts::manager_rows(ui, store);
            content.push(rustcord_components::accounts::list(
                &rows,
                width,
                look,
            ));
            content.push(parts::secondary_button(
                "acc:add",
                t("Agregar una cuenta"),
                true,
                look,
            ));
            Element::column(content).gap(12.0)
        }
        // The device's session, as a card, and the note on what is kept with it.
        "device" => {
            let remembered = store.remembered?;
            let os = match std::env::consts::OS {
                "linux" => "Linux",
                "windows" => "Windows",
                "macos" => "macOS",
                other => other,
            };
            let (note, action) = if remembered {
                (
                    t("Sesión recordada en el llavero del sistema"),
                    Some(parts::secondary_button(
                        "us:forget-session",
                        t("Olvidar la sesión"),
                        true,
                        look,
                    )),
                )
            } else {
                (
                    t("Sesión solo en memoria: al cerrar Rustcord tendrás que volver a iniciar sesión."),
                    None,
                )
            };
            let mut row = vec![
                rustcord_components::icon(Icon::Screen, look.muted).centered_y(),
                Element::column(vec![
                    rustcord_components::strong_line(format!("{os} • Rustcord"), 16.0, look.text),
                    rustcord_components::text(note, 14.0, look.muted),
                ])
                .gap(4.0)
                .flex(1.0)
                .centered_y(),
            ];
            row.extend(action.map(|action| action.centered_y()));
            let card = parts::card(
                Element::column(vec![
                    Element::row(vec![rustcord_components::strong_line(
                        t("Dispositivo actual"),
                        14.0,
                        look.muted,
                    )])
                    .padding(12.0),
                    Element::panel(Element::spacer())
                        .size(None, Some(1.0))
                        .background(look.border),
                    Element::row(row).gap(12.0).padding(12.0),
                ]),
                look,
            );
            Element::column(vec![
                card,
                rustcord_components::text(
                    t("Tus datos de acceso no se guardan en la caché de mensajes."),
                    14.0,
                    look.muted,
                ),
            ])
            .gap(8.0)
        }
        "forget" => parts::setting(
            t("Borrar lo guardado en este equipo"),
            Some(t("Mensajes, borradores e imágenes guardadas de esta cuenta. Tus ajustes se conservan.")),
            Some(parts::danger_button("us:forget", t("Borrar datos locales"), look)),
            true,
            look,
        ),
        "desktop" => toggle(
            "us:desktop",
            t("Activar las notificaciones de escritorio"),
            Some(t("Avisos del sistema para menciones y mensajes directos.")),
            prefs.desktop_notifications,
        ),
        "notification-content" => toggle(
            "us:notification-content",
            t("Mostrar el contenido del mensaje"),
            Some(t("Apagado, la notificación dice quién escribió y «Nuevo mensaje», sin el texto.")),
            prefs.notification_content,
        ),
        "message-sound" => toggle(
            "us:message-sound",
            t("Nuevo mensaje"),
            Some(t("Un tono breve con cada mención o mensaje directo que avisaría.")),
            prefs.message_sounds,
        ),
        "unread-badges" => toggle(
            "us:unread-badges",
            t("Marcar servidores con mensajes sin leer"),
            Some(t("Una marca junto al servidor mientras tenga mensajes sin leer.")),
            prefs.unread_badges,
        ),
        "mention-badges" => toggle(
            "us:mention-badges",
            t("Contador de menciones"),
            Some(t("Un globo con el número sobre el servidor y en mensajes directos.")),
            prefs.mention_badges,
        ),
        "devices" => devices(ctx),
        "volumes" => volumes(ctx),
        "meter" => meter(ctx),
        "loopback" => {
            let note = if ui.audio.loopback && ui.audio.output_starting {
                t("Abriendo la salida…")
            } else {
                t("Usa auriculares: con altavoces se produce eco.")
            };
            parts::toggle(
                "us:loopback",
                t("Escucharte"),
                Some(note),
                ui.audio.loopback,
                crate::quick_voice::call_holds_output(store).then_some(t(
                    "En una llamada la salida es suya: puedes escucharte de nuevo al colgar.",
                )),
                look,
            )
        }
        "input-profile" => parts::choices(
            "us:profile",
            t("Perfil de entrada"),
            &[
                (
                    "us:profile=voice-isolation".to_owned(),
                    t("Aislamiento de voz"),
                    Some(t("Solo tu hermosa voz: deja que Rustcord corte el ruido")),
                ),
                (
                    "us:profile=studio".to_owned(),
                    t("Estudio"),
                    Some(t("Audio puro: micrófono abierto sin procesamiento")),
                ),
                (
                    "us:profile=custom".to_owned(),
                    t("Personalizar"),
                    Some(t("Modo avanzado: ¡dame todos los controles!")),
                ),
            ],
            &format!("us:profile={}", prefs.input_profile.key()),
            look,
        ),
        "noise-suppression" => {
            let chosen = prefs.effective_processing().suppression;
            let custom = prefs.input_profile == super::InputProfile::Custom;
            let engines = t("RNNoise quita también el teclado y los ruidos que cambian; DeepFilterNet deja la voz más limpia con ruido constante, pero deja pasar más el teclado y usa más procesador, con 20 ms más de retraso; WebRTC es más liviano, para ruido constante como un ventilador.");
            let options = [
                ("us:suppression=rnnoise", "RNNoise", matches!(&chosen, rustcord_media::ProcessingMode::RnNoise)),
                ("us:suppression=deepfilter", "DeepFilterNet", matches!(&chosen, rustcord_media::ProcessingMode::DeepFilter)),
                ("us:suppression=webrtc", "WebRTC", matches!(&chosen, rustcord_media::ProcessingMode::WebRtc)),
                ("us:suppression=off", t("Ninguna"), matches!(&chosen, rustcord_media::ProcessingMode::Off)),
            ];
            let borrowed: Vec<(&str, &str, bool, bool)> = options
                .iter()
                .map(|(id, name, chosen)| (*id, *name, *chosen, true))
                .collect();
            parts::setting(
                t("Supresión de ruido"),
                Some(&if custom {
                    format!("{} {engines}", t("Quita el ruido de fondo. Lo eliges tú."))
                } else {
                    format!("{} {engines}", t("Quita el ruido de fondo. Lo elige el perfil; cambiar algo aquí lo pasa a Personalizar."))
                }),
                Some(select(
                    ctx,
                    "noise-suppression",
                    t("Supresión de ruido"),
                    &borrowed,
                    (width * 0.4).clamp(180.0, 280.0),
                )),
                true,
                look,
            )
        }
        "echo-cancellation" => toggle(
            "us:echo-cancellation",
            t("Cancelación de eco"),
            Some(t("Evita que el micrófono capte lo que suena en tus auriculares.")),
            prefs.effective_processing().echo_cancellation,
        ),
        "system-echo" => {
            let echo = prefs.effective_processing().echo_cancellation;
            let note = match ui.audio.system_echo.filter(|_| echo && prefs.system_echo) {
                Some(true) => t("Se cancela lo que suena en tu salida. Nunca se graba ni se envía."),
                Some(false) => t("No se pudo escuchar tu salida: por ahora se cancela sólo lo que reproduce Rustcord."),
                None => t("Lo que suena en tu salida no se cuela por el micrófono. Nunca se graba ni se envía."),
            };
            parts::toggle(
                "us:system-echo",
                t("Cancelar sonido del equipo"),
                Some(note),
                echo && prefs.system_echo,
                (!echo).then_some(t("Necesita la cancelación de eco.")),
                look,
            )
        }
        "automatic-gain" => toggle(
            "us:automatic-gain",
            t("Control automático de ganancia"),
            Some(t("Mantiene tu voz a un nivel parejo. No cambia el volumen del micrófono.")),
            prefs.effective_processing().automatic_gain,
        ),
        "camera" => camera(ctx),
        // Windows has no GPU engine yet: OpenH264 always.
        "video-hardware" if cfg!(windows) => parts::toggle(
            "us:video-hardware",
            t("Aceleración por hardware"),
            None,
            false,
            Some(t("En Windows, el video todavía se procesa en el procesador.")),
            look,
        ),
        "video-hardware" => toggle(
            "us:video-hardware",
            t("Aceleración por hardware"),
            Some(t("Usa la tarjeta gráfica para el video cuando puede. Apágala si la imagen se ve mal.")),
            prefs.video_hardware,
        ),
        // On, the key is held; off, the voice is detected (the switch sends
        // the action of the other state).
        "input-mode" => toggle(
            if prefs.push_to_talk {
                "us:input=voice"
            } else {
                "us:input=ptt"
            },
            t("Pulsar para hablar"),
            Some(t("Activa tu micrófono solo cuando mantienes presionada una tecla designada.")),
            prefs.push_to_talk,
        ),
        "ptt-key" => {
            prefs.push_to_talk.then_some(())?;
            push_to_talk_key(ctx)
        }
        "ptt-release" => {
            prefs.push_to_talk.then_some(())?;
            stacked(
                t("Tiempo de respuesta de Pulsar para hablar"),
                Some(
                    tr!(
                        "{ms} ms: cuánto sigue pasando el micrófono después de soltar el atajo.",
                        ms = prefs.push_to_talk_release
                    )
                    .as_str(),
                ),
                parts::slider(
                    "us:ptt-release",
                    f32::from(prefs.push_to_talk_release) / f32::from(super::MAX_RELEASE),
                    look,
                )
                .width(width)
                .accessibility_label(t("Tiempo de respuesta de Pulsar para hablar"))
                .render(),
            )
        }
        "sensitivity" => {
            (!prefs.push_to_talk).then_some(())?;
            sensitivity(ctx)
        }
        "image-previews" => Element::column(vec![
            parts::subtitle(t("Muestra imágenes, videos y gatitos divertidos..."), look),
            toggle(
                "us:image-previews",
                t("Vista previa de imágenes"),
                Some(t("Apagado, una imagen se muestra como cualquier otro archivo.")),
                prefs.image_previews,
            ),
        ]),
        "reactions" => toggle(
            "us:reactions",
            t("Mostrar reacciones de emojis"),
            None,
            prefs.reactions,
        ),
        "emoticons" => toggle(
            "us:emoticons",
            t("Convertir automáticamente los emoticonos en tus mensajes a emojis"),
            Some(t("Por ejemplo, cuando escribes :), Rustcord lo convierte a 🙂. El código y los enlaces no cambian.")),
            prefs.convert_emoticons,
        ),
        "avatars" => parts::toggle(
            "us:avatars",
            t("Mostrar los avatares de los usuarios"),
            None,
            !prefs.compact,
            Some(t("Para ocultar los avatares, elige Compacto en Accesibilidad › Mostrar el mensaje del chat.")),
            look,
        ),
        "related-accessibility" => parts::related(
            "us:page=accessibility",
            t("Accesibilidad"),
            t("Ajusta la densidad visual de los mensajes de chat y la IU"),
            look,
        ),
        "related-appearance" => parts::related(
            "us:page=appearance",
            t("Apariencia"),
            t("Cambia el tema de la aplicación"),
            look,
        ),
        "text-size" => stacked(
            t("Tamaño de texto en el chat"),
            Some(t("Ajusta el tamaño de la fuente del chat.")),
            ticked(
                "us:text-size",
                t("Tamaño de texto en el chat"),
                &TEXT_SIZES.map(|size| format!("{size}px")),
                TEXT_SIZES.iter().position(|size| *size == prefs.text_size),
                1,
                width,
                look,
            ),
        ),
        "display" => stacked(
            t("Mostrar el mensaje del chat"),
            Some(t("Cambia la apariencia de los mensajes de chat.")),
            parts::choices(
                "us:display",
                t("Mostrar el mensaje del chat"),
                &[
                    (
                        "us:display=cozy".to_owned(),
                        t("Predeterminado"),
                        Some(t("El nombre y el avatar encabezan cada grupo de mensajes.")),
                    ),
                    (
                        "us:display=compact".to_owned(),
                        t("Compacto"),
                        Some(t("Un mensaje por línea, con la hora y el nombre delante, sin avatares.")),
                    ),
                ],
                if prefs.compact {
                    "us:display=compact"
                } else {
                    "us:display=cozy"
                },
                look,
            ),
        ),
        "group-gap" => stacked(
            t("Espacio entre mensajes de grupo"),
            Some(t("Ajusta el espaciado entre los grupos de mensajes.")),
            ticked(
                "us:group-gap",
                t("Espacio entre mensajes de grupo"),
                &GROUP_GAPS.map(|gap| format!("{gap}px")),
                GROUP_GAPS.iter().position(|gap| *gap == prefs.group_gap),
                3,
                width,
                look,
            ),
        ),
        "zoom" => {
            let nearest = ZOOMS
                .iter()
                .enumerate()
                .min_by_key(|(_, zoom)| zoom.abs_diff(prefs.zoom))
                .map(|(mark, _)| mark);
            stacked(
                t("Nivel de zoom"),
                Some(t("Ajusta el tamaño de la interfaz. También puedes cambiarlo con Ctrl +/-.")),
                ticked(
                    "us:zoom",
                    t("Nivel de zoom"),
                    &ZOOMS.map(|zoom| zoom.to_string()),
                    ui.settings_state.zoom_drag.or(nearest),
                    5,
                    width,
                    look,
                ),
            )
        }
        "role-colors" => {
            let options = [
                ("us:role-colors=on", t("En nombres"), prefs.role_colors, true),
                (
                    "us:role-colors=off",
                    t("No mostrar colores de rol"),
                    !prefs.role_colors,
                    true,
                ),
            ];
            parts::setting(
                t("Colores de rol"),
                Some(t("Elige cómo te gustaría que se vean los colores de rol. Los nombres toman el color de su rol más alto.")),
                Some(select(
                    ctx,
                    "role-colors",
                    t("Colores de rol"),
                    &options,
                    (width * 0.4).clamp(180.0, 280.0),
                )),
                true,
                look,
            )
        }
        "motion" => toggle(
            "us:motion",
            t("Activar el movimiento reducido"),
            Some(t("Reduce la cantidad y la intensidad de las animaciones, los efectos al pasar el cursor y otros efectos en movimiento.")),
            ui.reduced_motion,
        ),
        "language" => {
            let system = tr!(
                "Sistema ({language})",
                language = ui.desktop.system_language.name()
            );
            let options = [
                ("us:lang=system", system.as_str(), prefs.language.is_none()),
                ("us:lang=es", I18n::Spanish.name(), matches!(prefs.language, Some(I18n::Spanish))),
                ("us:lang=en", I18n::English.name(), matches!(prefs.language, Some(I18n::English))),
            ];
            let borrowed: Vec<(&str, &str, bool, bool)> = options
                .iter()
                .map(|(id, name, chosen)| (*id, *name, *chosen, true))
                .collect();
            parts::setting(
                t("Selecciona un idioma"),
                Some(t(
                    "Elige el idioma en el que quieres que se muestre Rustcord. «Sistema» sigue el del equipo. Los mensajes no se traducen.",
                )),
                Some(select(
                    ctx,
                    "language",
                    t("Selecciona un idioma"),
                    &borrowed,
                    (width * 0.4).clamp(180.0, 280.0),
                )),
                true,
                look,
            )
        }
        "clock" => stacked(
            t("Formato de tiempo"),
            Some(t("Cómo se escriben las horas de los mensajes.")),
            parts::choices(
                "us:clock",
                t("Formato de tiempo"),
                &[
                    ("us:clock=12".to_owned(), t("12 horas"), Some(t("2:05 p. m."))),
                    ("us:clock=24".to_owned(), t("24 horas"), Some("14:05")),
                ],
                if prefs.twelve_hour {
                    "us:clock=12"
                } else {
                    "us:clock=24"
                },
                look,
            ),
        ),
        "provider" => {
            let Some(info) = &store.provider else {
                return Some(parts::setting(
                    t("Sin proveedor"),
                    Some(t("Rustcord no está conectado a ningún servicio.")),
                    None,
                    true,
                    look,
                ));
            };
            let mut header = vec![Element::row(vec![
                strong_line(t(&info.name), 16.0, look.text).flex(1.0).centered_y(),
                crate::providers::badge(info.kind, look).centered_y(),
            ])
            .gap(8.0)];
            if let Some(notice) = crate::providers::current_notice(ui) {
                header.push(text(t(notice), 14.0, look.muted));
            }
            let mut details = vec![Element::column(header).gap(6.0).padding(12.0)];
            if let rustcord_provider::AuthenticationState::SignedIn { scopes } = &store.authentication {
                if !scopes.is_empty() {
                    details.push(parts::setting(
                        t("Permisos concedidos"),
                        Some(&scopes.join(", ")),
                        None,
                        true,
                        look,
                    ));
                }
            }
            Element::column(details)
        }
        // What the host can start instead, each with its origin.
        "choose" => {
            crate::providers::choosable(ui).then_some(())?;
            stacked(
                t("Cambiar de proveedor"),
                Some(t("Cada uno dice de dónde vienen sus datos. Cambiar cierra la sesión de este.")),
                Element::column(crate::providers::cards(ui, store, width, look)).gap(8.0),
            )
        }
        "unavailable-providers" => crate::providers::unavailable_notes(ui, look)?,
        "capabilities" => capabilities(ctx)?,
        "related-developer" => {
            simulated(store).then_some(())?;
            parts::related(
                "us:page=developer",
                t("Pruebas de la simulación"),
                t("Corta la conexión o haz fallar un envío para ver cómo responde Rustcord."),
                look,
            )
        }
        "developer" => toggle(
            "us:developer",
            t("Modo desarrollador"),
            Some(&format!(
                "{} {}",
                t("El modo desarrollador muestra elementos del menú contextual que resultan útiles para personas que programan aplicaciones."),
                t("Agrega «Copiar ID» a los menús de mensajes, canales, categorías, servidores, roles y miembros."),
            )),
            prefs.developer,
        ),
        "metrics" => toggle(
            "us:metrics",
            t("Métricas de rendimiento"),
            Some(t("Tiempos de cada cuadro, memoria y CPU, sobre la ventana. También con F12.")),
            ui.show_metrics,
        ),
        "disconnect" => trial(
            t("Simular una desconexión"),
            t("Corta la sesión simulada para ver la reconexión automática."),
            "us:disconnect",
            t("Cortar"),
            look,
        ),
        "fail-send" => trial(
            t("Rechazar el próximo envío"),
            t("El mensaje queda marcado como no enviado, con Reintentar y Descartar."),
            "us:fail-send",
            t("Rechazar"),
            look,
        ),
        "silent-send" => trial(
            t("No responder el próximo envío"),
            t("Pasado el plazo, el mensaje queda como de destino incierto."),
            "us:silent-send",
            t("Silenciar"),
            look,
        ),
        "ring-me" => trial(
            t("Recibir una llamada"),
            t("La persona de tu primer privado te llama: el aviso, el sonido y la notificación."),
            "us:ring-me",
            t("Llamar"),
            look,
        ),
        "answer-elsewhere" => trial(
            t("Atenderla en otro dispositivo"),
            t("La llamada que suena sigue en otro de tus dispositivos."),
            "us:answer-elsewhere",
            t("Atender"),
            look,
        ),
        _ => return None,
    })
}

/// A test of the simulation: what it does and its button.
fn trial(title: &str, note: &str, id: &str, label: &str, look: &Look) -> Element {
    parts::setting(
        title,
        Some(note),
        Some(parts::secondary_button(id, label, true, look)),
        true,
        look,
    )
}

/// A slider with marks; a value between marks shows at none.
fn ticked(
    id: &str,
    label: &str,
    labels: &[String],
    index: Option<usize>,
    usual: usize,
    width: f32,
    look: &Look,
) -> Element {
    let names: Vec<&str> = labels.iter().map(String::as_str).collect();
    parts::ticked_slider(
        id,
        label,
        &names,
        index.unwrap_or(usual),
        usual,
        width,
        look,
    )
}

/// A list to choose from, `us:select:{name}`, drawn as a field: each option
/// its action, its label, whether it is chosen and whether it can be; the
/// ones that cannot be chosen yet stay in it, disabled.
fn select(
    ctx: &Ctx,
    name: &str,
    label: &str,
    options: &[(&str, &str, bool, bool)],
    width: f32,
) -> Element {
    let current = options
        .iter()
        .find(|(_, _, chosen, _)| *chosen)
        .map_or("", |(_, label, _, _)| *label);
    let choices = options
        .iter()
        .map(|(id, text, chosen, enabled)| {
            DropdownOption::new(*id, *text)
                .selected(*chosen)
                .enabled(*enabled)
        })
        .collect();
    ctx.look
        .select(Select::new(
            format!("us:select:{name}"),
            current,
            choices,
            ctx.ui.settings_state.select == Some(name),
            "us:select:dismiss",
            ctx.look.palette(),
        ))
        .size(limn::ControlSize::Spacious)
        .accessibility_label(label)
        .width(width)
        .menu_width(width)
        .height(parts::FIELD_HEIGHT)
        .render()
}

/// The display name: what it is and a button to change it, or the field
/// being written with its buttons under it.
fn display_name(ctx: &Ctx) -> Element {
    let look = ctx.look;
    let (border, focus) = parts::field_outline(look);
    let locked = profile_locked(ctx.store);
    let me = ctx.store.me_user();
    let note = locked.unwrap_or(t(
        "Cómo te ven los demás. Vacío, se muestra tu nombre de usuario.",
    ));
    match &ctx.ui.settings_state.name {
        Some(draft) => Element::column(vec![
            parts::setting(t("Nombre para mostrar"), Some(note), None, true, look),
            Element::row(vec![
                TextField::new(
                    NAME,
                    draft.clone(),
                    me.map_or("", |me| me.name.as_str()),
                    look.palette(),
                )
                .accessibility_label(t("Nombre para mostrar"))
                .height(parts::FIELD_HEIGHT)
                .outline(border, focus)
                .radius(parts::FIELD_RADIUS)
                .render()
                .flex(1.0),
                parts::primary_button("us:name:save", t("Guardar"), true, look),
                parts::secondary_button("us:name:cancel", t("Cancelar"), true, look),
            ])
            .gap(8.0),
            Element::spacer().size(None, Some(12.0)),
        ]),
        None => parts::setting(
            t("Nombre para mostrar"),
            Some(note),
            Some(
                Element::row(vec![
                    line(me.map_or("—", |me| me.shown_name()), 16.0, look.muted)
                        .max_width(220.0)
                        .centered_y(),
                    parts::secondary_button("us:name:edit", t("Editar"), locked.is_none(), look),
                ])
                .gap(16.0),
            ),
            locked.is_none(),
            look,
        ),
    }
}

/// The microphone and the speaker, side by side from 480 px and one under
/// the other below, each with its volume; and
/// finding them again, or putting the volumes back once they changed.
fn devices(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let width = ctx.geometry.column;
    let wide = width >= 480.0;
    let half = if wide { (width - 17.0) / 2.0 } else { width };
    let label = |name: &str| line(name, 14.0, look.text).font_family(limn::FontFamily::SansMedium);
    let side = |output: bool| {
        let list = crate::quick_voice::devices(ui, output);
        let (title, missing, id, name, volume, most) = if output {
            (
                t("Altavoz"),
                t("La salida elegida no está conectada. Conéctala o elige otra."),
                "us:output-volume",
                t("Volumen del altavoz"),
                ui.prefs.output_volume,
                rustcord_media::MAX_VOLUME,
            )
        } else {
            (
                t("Micrófono"),
                t("El micrófono elegido no está conectado. Conéctalo o elige otro."),
                "us:input-volume",
                t("Volumen del micrófono"),
                ui.prefs.input_volume,
                rustcord_media::MAX_INPUT_VOLUME,
            )
        };
        let mut column = vec![label(title), quick::device_select(&list, half, look)];
        if list.missing() {
            column.push(text(missing, 12.0, look.danger));
        }
        column.extend([
            Element::spacer().size(None, Some(4.0)),
            label(name),
            quick::percent_slider(id, name, volume, most, half, look),
        ]);
        Element::column(column).gap(8.0)
    };
    let pair = if wide {
        Element::row(vec![side(false), side(true)]).gap(17.0)
    } else {
        Element::column(vec![side(false), side(true)]).gap(16.0)
    };
    let mut column = vec![
        Element::spacer().size(None, Some(12.0)),
        pair,
        quick::volume_note(look),
    ];
    match &ui.audio.devices {
        Some(Err(error)) => column.push(text(
            tr!("No se pudieron leer los dispositivos: {error}", error),
            13.0,
            look.danger,
        )),
        Some(Ok(found)) => column.push(line(
            tr!("Sistema de audio: {backend}", backend = found.backend),
            12.0,
            look.muted,
        )),
        None => {}
    }
    let link = |id: &str, label: &str, glyph: Icon| {
        Button::new(id, label, look.palette())
            .variant(ButtonVariant::Link)
            .icon(glyph)
            .render()
    };
    let mut links = vec![link(
        "us:devices",
        t("Actualizar dispositivos"),
        Icon::Refresh,
    )];
    if crate::quick_voice::levels_changed(ui) {
        links.push(link(
            "us:reset-levels",
            t("Restablecer niveles"),
            Icon::Sliders,
        ));
    }
    links.push(Element::spacer().flex(1.0));
    column.push(Element::row(links).gap(16.0));
    Element::column(column).gap(8.0)
}

/// «Probar sonido»: one of the call's sounds on the chosen output, also
/// during a call, into its sound.
fn volumes(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let note = if ui.prefs.output_volume == 0.0 {
        t("Con el volumen de salida en 0 % no se oye.")
    } else if ctx.store.call().is_some() {
        t("Suena el sonido de entrar a la llamada, mezclado con ella.")
    } else {
        t("Suena el sonido de entrar a la llamada en la salida elegida.")
    };
    Element::column(vec![
        Element::spacer().size(None, Some(8.0)),
        Element::row(vec![
            parts::secondary_button("us:speaker-test", t("Probar sonido"), true, look),
            line(note, 13.0, look.muted).centered_y(),
            Element::spacer().flex(1.0),
        ])
        .gap(12.0),
    ])
}

/// «Comprobemos el micrófono»: one button, the level as the
/// call would use it, in dBFS, with where voice starts marked on it; and,
/// for comparing the suppressors, the levels before and after processing.
/// In a call it measures the call's own microphone: nothing more is sent.
fn meter(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let button = if ui.audio.testing {
        parts::secondary_button("us:meter", t("Detener la prueba"), true, look)
    } else {
        parts::primary_button("us:meter", t("Empezar la prueba"), true, look)
    };
    let status = if ui.audio.testing && ui.audio.starting {
        t("Abriendo micrófono…").to_owned()
    } else if ui.audio.testing {
        crate::quick_voice::reading(ui).unwrap_or_else(|| t("Escuchando.").to_owned())
    } else {
        String::new()
    };
    let width = ctx.geometry.column;
    let live = ui.audio.testing && !ui.audio.starting;
    let mut rows = vec![parts::subtitle(t("Probar micrófono"), look), text(
        t("Empieza la prueba y di algo: la barra muestra tu nivel, y la marca, dónde empieza a contar como voz. Es local; no se envía nada."),
        14.0,
        look.muted,
    )];
    if ctx.store.call().is_some() {
        rows.push(text(
            t("En la llamada mide el mismo micrófono que usa ella, sin cambiar lo que se envía."),
            13.0,
            look.muted,
        ));
    }
    rows.extend([
        Element::spacer().size(None, Some(4.0)),
        Element::row(vec![
            button,
            line(status, 13.0, look.muted).centered_y(),
            Element::spacer().flex(1.0),
        ])
        .gap(12.0),
        quick::level_meter(
            "us:level",
            ui.audio.levels.last().copied().filter(|_| live),
            crate::quick_voice::threshold(ui),
            width,
            look,
        ),
        Element::spacer().size(None, Some(4.0)),
        line(t("Antes y después del procesamiento"), 13.0, look.text)
            .font_family(limn::FontFamily::SansMedium),
    ]);
    let width = (width - 92.0).max(40.0);
    for (id, label, levels) in [
        ("us:raw-levels", t("Entrada"), &ui.audio.raw_levels),
        ("us:levels", t("Procesada"), &ui.audio.levels),
    ] {
        let levels = if live { levels.as_slice() } else { &[][..] };
        rows.push(
            Element::row(vec![
                line(label, 13.0, look.muted)
                    .size(Some(84.0), None)
                    .centered_y(),
                parts::level_bars(id, levels, width, look).accessible(
                    limn::Access::new(limn::Role::Status)
                        .id(id)
                        .name(tr!("Nivel de {label}", label))
                        .description(if !ui.audio.testing {
                            t("Sin prueba")
                        } else if ui.audio.starting {
                            t("Esperando micrófono")
                        } else {
                            t("Prueba local")
                        }),
                ),
            ])
            .gap(8.0),
        );
    }
    Element::column(rows).gap(8.0)
}

/// The push-to-talk key, its recorder, and whether it also works while
/// another window has focus.
fn push_to_talk_key(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let recording = ui.settings_state.recording.is_some();
    let key = ui.prefs.push_to_talk_key.as_deref();
    // Why the key just recorded was refused.
    let refused = ui
        .settings_state
        .key_notice
        .as_ref()
        .filter(|(action, _)| *action == crate::keybinds::Action::default())
        .map(|(_, why)| why.as_str());
    let note = if recording {
        t("Pulsa la tecla o la combinación. Esc cancela.")
    } else if let Some(why) = refused {
        why
    } else if key.is_none() {
        t("Todavía no tiene atajo: graba uno.")
    } else {
        t("Funciona con la ventana de Rustcord enfocada.")
    };
    let control = Element::row(vec![
        Kbd::new(
            if recording {
                "…"
            } else {
                key.unwrap_or(t("Sin atajo"))
            },
            look.palette(),
        )
        .render()
        .centered_y(),
        parts::secondary_button(
            "us:ptt-record",
            if recording {
                t("Cancelar")
            } else {
                t("Grabar atajo")
            },
            true,
            look,
        ),
    ])
    .gap(12.0);
    let global = match &ui.audio.global_keys {
        GlobalKeys::Off => None,
        GlobalKeys::Waiting => Some(t("Pidiendo el atajo global al escritorio…").to_owned()),
        GlobalKeys::Bound(trigger) if trigger.is_empty() => {
            Some(t("También fuera de Rustcord, con el atajo que asignó el escritorio.").to_owned())
        }
        GlobalKeys::Bound(trigger) => Some(tr!(
            "También fuera de Rustcord: el escritorio lo asignó a {trigger}.",
            trigger
        )),
        GlobalKeys::Unavailable(reason) => Some(tr!(
            "Fuera de Rustcord no funciona: {reason}",
            reason = t(reason)
        )),
    };
    let mut column = vec![parts::setting(
        t("Atajo de teclado «Pulsar para hablar»"),
        Some(note),
        Some(control),
        true,
        look,
    )];
    if let Some(global) = global {
        column.push(text(global, 13.0, look.muted));
    }
    if let Some(line) = &ui.audio.hyprland_bind {
        column.push(text(
            t("Hyprland no asigna la tecla por sí mismo. Agrega esta línea a tu configuración:"),
            13.0,
            look.muted,
        ));
        column.push(
            Element::row(vec![
                text(line.clone(), 13.0, look.text).centered_y(),
                parts::secondary_button("us:ptt-hypr-copy", t("Copiar"), true, look),
            ])
            .gap(12.0),
        );
    }
    // A game elevated on Windows does not get the hook's input without it.
    if cfg!(windows) {
        column.push(text(
            t("Si un juego se ejecuta como administrador, abre Rustcord también como administrador para que el atajo funcione dentro de él."),
            13.0,
            look.muted,
        ));
    }
    if ui.audio.testing {
        column.push(line(
            if ui.audio.transmitting {
                t("Transmitiendo: el micrófono pasa.")
            } else {
                t("En silencio: mantén el atajo para hablar.")
            },
            13.0,
            look.muted,
        ));
    }
    column.push(Element::spacer().size(None, Some(12.0)));
    Element::column(column).gap(4.0)
}

/// The camera to use and «Probar video»: the system asks the first time,
/// and the picture stays on this machine.
fn camera(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let video = &ui.video;
    let chosen = video.device.as_deref();
    let mut options =
        vec![
            DropdownOption::new("us:camera-default", t("Predeterminada del sistema"))
                .icon(Icon::Video)
                .selected(chosen.is_none()),
        ];
    for (id, name) in &video.cameras {
        options.push(
            DropdownOption::new(format!("us:camera={id}"), name.clone())
                .icon(Icon::Video)
                .selected(chosen == Some(id.as_str())),
        );
    }
    let current = chosen.map_or(t("Predeterminada del sistema"), |id| {
        video
            .cameras
            .iter()
            .find(|(found, _)| found == id)
            .map_or(id, |(_, name)| name.as_str())
    });
    let width = ctx.geometry.column.min(480.0);
    let select = look
        .select(Select::new(
            "us:select:camera",
            current,
            options,
            ui.settings_state.select == Some("camera"),
            "us:select:dismiss",
            look.palette(),
        ))
        .icon(Icon::Video)
        .accessibility_label(t("Cámara"))
        .width(width)
        .menu_width(width)
        .render();
    let size = Size::new(width, width * 9.0 / 16.0);
    let picture = match video.camera.live.frame().filter(|_| video.testing) {
        Some((w, h)) => {
            let scale = (size.width / w as f32).min(size.height / h as f32);
            rustcord_components::centered(
                // Mirrored, as a webcam shows the person; not what is sent.
                Element::surface(
                    rustcord_components::voice::ids::CAMERA_SURFACE,
                    t("Tu cámara"),
                )
                .surface_mirrored(true)
                .size(Some(w as f32 * scale), Some(h as f32 * scale)),
            )
        }
        None => rustcord_components::centered(line(
            if video.testing {
                t("Esperando la cámara…")
            } else {
                t("Prueba el video para verte aquí.")
            },
            14.0,
            look.muted,
        )),
    };
    let supported = cfg!(any(target_os = "linux", windows));
    let mut rows = vec![
        Element::spacer().size(None, Some(16.0)),
        line(t("Cámara"), 16.0, look.text).font_family(limn::FontFamily::SansMedium),
        select,
    ];
    if video.cameras.is_empty() {
        rows.push(line(
            t("Las cámaras aparecen cuando el sistema da acceso."),
            12.0,
            look.muted,
        ));
    }
    rows.extend([
        Element::panel(picture)
            .size(Some(size.width), Some(size.height))
            .radius(8.0)
            .background(Color::rgb(0, 0, 0))
            .clip_children(),
        Element::row(vec![
            parts::primary_button(
                "us:video-test",
                if video.testing {
                    t("Detener video")
                } else {
                    t("Probar video")
                },
                supported,
                look,
            ),
            Element::spacer().flex(1.0),
        ]),
        text(
            if supported {
                t("Vista previa local; no envía video. El sistema pide permiso la primera vez.")
            } else {
                t(rustcord_components::voice::NO_VIDEO)
            },
            13.0,
            look.muted,
        ),
    ]);
    Element::column(rows).gap(8.0)
}

/// When the microphone counts as voice: the room's level, which Rustcord
/// follows, or one's own on a slider right under the level heard.
fn sensitivity(ctx: &Ctx) -> Element {
    let (ui, look) = (ctx.ui, ctx.look);
    let prefs = &ui.prefs;
    let width = ctx.geometry.column;
    let threshold = if prefs.speech_gate() {
        rustcord_media::VoiceActivity::GATED_ON
    } else if prefs.auto_sensitivity {
        ui.audio
            .threshold
            .unwrap_or(rustcord_media::VoiceActivity::default().on)
    } else {
        prefs.sensitivity
    };
    let level = ui
        .audio
        .testing
        .then(|| ui.audio.levels.last().copied())
        .flatten();
    let mut column = vec![
        parts::subtitle(t("Sensibilidad de la entrada"), look),
        parts::toggle(
            "us:auto-sensitivity",
            t("Determinar automáticamente la sensibilidad de entrada"),
            Some(if prefs.speech_gate() {
                t("Decide sola: pasa tu voz, y el teclado, la respiración y el ruido de fondo no abren el micrófono.")
            } else if prefs.auto_sensitivity {
                t("Rustcord sigue el ruido de la sala: tu voz tiene que superarlo por 15 dB.")
            } else {
                t("Arrastra el umbral: lo que suene por encima cuenta como voz.")
            }),
            prefs.auto_sensitivity,
            None,
            look,
        ),
    ];
    // Set by hand, the threshold is the thumb of a slider that draws the
    // level on its track, as the official client's; otherwise the meter
    // marks the threshold Rustcord follows.
    column.push(if prefs.auto_sensitivity {
        parts::sensitivity(level, threshold, width, look)
    } else {
        let level = level.unwrap_or(0.0);
        let fill = if level >= threshold {
            look.online
        } else {
            look.idle
        };
        parts::slider("us:sensitivity", prefs.sensitivity, look)
            .width(width)
            .level(level, fill)
            .accessibility_label(t("Umbral de voz"))
            .render()
    });
    let decibels = threshold * -rustcord_media::FLOOR_DBFS + rustcord_media::FLOOR_DBFS;
    let decibels = format!("{decibels:.0}");
    column.push(line(
        if prefs.speech_gate() {
            t("Umbral: lo que suena como voz").to_owned()
        } else if prefs.auto_sensitivity {
            tr!("Umbral: {decibels} dBFS · automático", decibels)
        } else {
            tr!("Umbral: {decibels} dBFS", decibels)
        },
        12.0,
        look.muted,
    ));
    Element::column(column).gap(8.0)
}

/// A group of shortcuts, under its title.
/// What the provider can do, one line each.
fn capabilities(ctx: &Ctx) -> Option<Element> {
    let look = ctx.look;
    let info = ctx.store.provider.as_ref()?;
    let yes_no = |value: bool| if value { t("Sí") } else { "No" };
    let capabilities = info.capabilities;
    let voice = match capabilities.voice {
        VoiceSupport::None => "No",
        VoiceSupport::Simulated => t("Simulada: no se transmite audio"),
        VoiceSupport::Real => t("Sí"),
    };
    let rows = [
        (
            t("Inicio y cierre de sesión"),
            yes_no(capabilities.authentication),
        ),
        (
            t("Lista de servidores"),
            yes_no(capabilities.account_servers),
        ),
        (t("Enviar mensajes"), yes_no(capabilities.send_messages)),
        (
            t("Editar y borrar"),
            yes_no(capabilities.edit_messages && capabilities.delete_messages),
        ),
        (t("Reacciones"), yes_no(capabilities.reactions)),
        (t("Historial"), yes_no(capabilities.history)),
        (t("Mensajes directos"), yes_no(capabilities.direct_messages)),
        (t("Adjuntos"), yes_no(capabilities.attachments)),
        (
            t("Reenvío seguro (sin duplicar)"),
            yes_no(capabilities.idempotent_send),
        ),
        (t("Presencia"), yes_no(capabilities.presence)),
        (t("Editar el perfil"), yes_no(capabilities.edit_profile)),
        (t("Voz"), voice),
    ]
    .into_iter()
    .map(|(label, value)| {
        Element::row(vec![
            line(label, 15.0, look.text).flex(1.0),
            line(value, 15.0, look.muted),
        ])
        .gap(16.0)
        .padding(10.0)
    })
    .collect();
    Some(Element::column(vec![
        parts::subtitle(t("Lo que puede hacer"), look),
        parts::card(Element::column(rows), look),
    ]))
}

/// The quick switcher over `root` while it is open.
pub fn switcher(ui: &Ui, store: &Store, root: Element, look: &Look, window: Size) -> Element {
    let Some(state) = &ui.switcher else {
        return root;
    };
    let width = Modal::fit_width(window.width, 560.0);
    let results: Vec<Element> = destinations(ui, store)
        .into_iter()
        .enumerate()
        .map(|(index, destination)| {
            let icon = match destination.kind {
                ChannelKind::Category => Icon::Server,
                ChannelKind::Direct => Icon::At,
                ChannelKind::Group => Icon::Users,
                _ => Icon::Hash,
            };
            ListRow::new(
                format!("switcher:go={}", destination.id),
                destination.title,
                look.menu_palette(),
            )
            .icon(icon)
            .subtitle(destination.detail)
            .filled()
            .title_ink(if index == state.selected {
                look.accent
            } else {
                look.text
            })
            .trailing_icon(if index == state.selected {
                Icon::ArrowRight
            } else {
                Icon::ChevronRight
            })
            .render()
        })
        .collect();
    let results = if results.is_empty() {
        vec![line(t("Nada coincide."), 13.0, look.muted)]
    } else {
        results
    };
    Modal::new(
        root,
        true,
        t("Ir a…"),
        t("Busca un servidor, un canal o una conversación. ↑↓ para elegir, Enter para ir."),
        "switcher:close",
        look.menu_palette(),
    )
    .width(width)
    .body(
        Element::column(vec![
            SearchField::new(
                SWITCHER_QUERY,
                state.query.clone(),
                t("¿A dónde quieres ir?"),
                look.menu_palette(),
            )
            .shortcut("Ctrl+K")
            .render(),
            ScrollArea::new("switcher:results", Element::column(results).gap(2.0))
                .render()
                .size(None, Some(320.0)),
        ])
        .gap(8.0),
    )
    .initial_focus(SWITCHER_QUERY)
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searching_folds_case_and_accents_and_wants_every_word() {
        assert!(found("ZOOM", &["Zoom"]));
        assert!(found("animacion", &["animaciones"]));
        assert!(found("tamaño texto", &["Tamaño del texto"]));
        assert!(!found("tamaño voz", &["Tamaño del texto"]));
    }

    #[test]
    fn each_page_shows_its_sections_in_discord_s_order() {
        assert_eq!(
            section_keys(SettingsPage::System),
            [
                "desktop",
                "shortcuts",
                "default-keybinds",
                "graphics",
                "updates"
            ]
        );
        assert_eq!(
            section_keys(SettingsPage::Appearance),
            [
                "theme",
                "messages",
                "chat-box",
                "cosmetics",
                "font",
                "panels",
                "reading"
            ]
        );
    }

    // One test per page of Discord's categories: the order is the point.

    #[test]
    fn account_shows_account_info_then_the_session() {
        assert_eq!(
            section_keys(SettingsPage::Account),
            ["account-info", "account-accounts", "account-device"]
        );
    }

    #[test]
    fn privacy_shows_voice_encryption_first_then_rustcord_s_own() {
        assert_eq!(section_keys(SettingsPage::Privacy), ["e2ee", "image-cache"]);
    }

    #[test]
    fn messaging_permissions_show_discord_s_order() {
        assert_eq!(
            section_keys(SettingsPage::Messaging),
            [
                "content-filters",
                "spam-filters",
                "messaging",
                "friend-requests",
                "ignore-block"
            ]
        );
    }

    #[test]
    fn notifications_show_overview_sounds_then_badges() {
        assert_eq!(
            section_keys(SettingsPage::Notifications),
            ["overview", "sounds", "badges"]
        );
    }

    #[test]
    fn registered_games_show_the_current_game_then_the_added_ones() {
        assert_eq!(section_keys(SettingsPage::Games), ["games-now", "games"]);
    }

    #[test]
    fn activity_privacy_shows_what_i_share() {
        assert_eq!(section_keys(SettingsPage::Activity), ["activity"]);
    }

    #[test]
    fn developer_shows_developer_mode_then_rustcord_s_own() {
        assert_eq!(
            section_keys(SettingsPage::Developer),
            ["developer", "provider-notes", "simulation"]
        );
    }

    #[test]
    fn voice_and_video_shows_discord_s_categories_in_their_order() {
        assert_eq!(
            section_keys(SettingsPage::Voice),
            ["voice", "input-profile", "video", "call-sounds", "advanced"]
        );
    }

    #[test]
    fn accessibility_shows_discord_s_categories_in_their_order() {
        assert_eq!(
            section_keys(SettingsPage::Accessibility),
            ["readability", "density", "color", "motion"]
        );
    }

    #[test]
    fn language_and_time_shows_its_two_sections_in_order() {
        assert_eq!(section_keys(SettingsPage::Language), ["language", "clock"]);
    }

    #[test]
    fn the_groups_hold_every_page_once_in_the_order_of_all() {
        let pages: Vec<SettingsPage> = GROUPS
            .iter()
            .flat_map(|(_, _, group)| group.iter())
            .filter_map(|entry| match entry {
                Entry::Page(page) => Some(*page),
                Entry::LogOut => None,
            })
            .collect();
        assert_eq!(pages, SettingsPage::ALL);
    }

    #[test]
    fn every_section_key_is_unique_and_names_a_page_of_its_own() {
        let mut keys: Vec<&str> = sections().map(|section| section.key).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count);
        for page in SettingsPage::ALL {
            assert!(!section_keys(page).is_empty(), "{page:?} has sections");
        }
    }
}
