//! What the settings of a channel show, from their state and the store.

use limn::{
    Button, ButtonVariant, Color, Component, ComponentPalette, ControlSize, Element, Empty,
    FontFamily, Icon, Modal, TextArea, TextField, TextSpan,
};
use rustcord_components::channel_settings::{
    self as own, AccessLine, AddChoice, FieldSelect, InviteRow, PrivateCard, TargetItem,
    TargetMark, Visibility, AFTER_CHOICES, AFTER_SLOWMODE, FIELD_GAP, SLOWMODE, TARGET_GAP,
    TOPIC_LIMIT,
};
use rustcord_components::server_menu::{audience_picker, Audience, Choice};
use rustcord_components::server_settings::{
    self as parts, ItemLine, MenuAction, Metrics, SidebarEntry, UnsavedBar, DEFAULT_ROLE_COLOR,
};
use rustcord_components::{gap, line, strong_line, text, Look};
use rustcord_i18n::{t, tr};
use rustcord_model::{
    Channel, ChannelKind, OverwriteTarget, Permissions, RemoteImage, Role, Server, Timestamp,
    UserId, WebhookKind,
};
use rustcord_provider::ProviderKind;
use rustcord_state::{Abilities, Store};

use super::{
    everyone, final_name, permission_lock, sync_commands, target_lock, ChannelSettings, Page,
    ACCESS_QUERY, NAME, TOPIC,
};
use crate::{Ui, ViewContext};

/// The height of the settings' text fields, as Discord's client measures them.
const FIELD_HEIGHT: f32 = 40.0;
/// The topic's field: Discord's editor (233 px) without its 32 px of format
/// buttons.
const TOPIC_HEIGHT: f32 = 201.0;

/// The settings over the whole window, while they are open.
pub fn view(ui: &Ui, store: &Store, context: &ViewContext) -> Option<Element> {
    let settings = ui.channel_settings.as_ref()?;
    let channel = store.channel(settings.channel)?;
    let server = store.server(channel.server?)?;
    let abilities = store.abilities(channel.id)?;
    let look = &ui.look();
    let metrics = Metrics::new(context.window.width, false);
    let view = View {
        settings,
        store,
        channel,
        server,
        abilities: &abilities,
        look,
        metrics,
        advanced: ui.advanced_permissions,
        developer: ui.prefs.developer,
        ui,
    };
    let region = match settings.page {
        Page::Overview => view.overview(),
        Page::Permissions => view.permissions(),
        Page::Forum => {
            super::forum::page(settings, channel, &server.emojis, &abilities, look, metrics)
        }
        Page::Invites => view.invites(context.wall),
        Page::Integrations => view.integrations(),
    };
    let bar = settings.dirty(store).then(|| {
        parts::unsaved_bar_for(
            "cs",
            UnsavedBar {
                saving: settings.saving(),
                nudged: settings.nudged,
            },
            look,
        )
    });
    let root = parts::shell_for(
        "cs",
        context.window,
        metrics,
        view.sidebar(),
        region,
        bar,
        look,
    );
    Some(view.dialogs(root, context.window.width))
}

/// The people the ⊕ popout offers: those not listed yet whose name or
/// handle has what is searched, by name. Only the first few that match:
/// typing narrows them, and the provider is asked for the people it has
/// not sent yet (`lookup`).
fn offered_people(
    settings: &ChannelSettings,
    store: &Store,
    channel: &Channel,
    server: &Server,
) -> Vec<(String, UserId)> {
    let query = settings.add_query.trim().to_lowercase();
    let listed = settings.listed(channel);
    let handle = |user: UserId| {
        store
            .user(user)
            .map_or_else(|| user.to_string(), |account| account.name.clone())
    };
    let mut people: Vec<(String, UserId)> = store
        .members(server.id)
        .iter()
        .filter(|member| !listed.contains(&OverwriteTarget::Member(member.user)))
        .map(|member| {
            (
                store.display_name(member.user, Some(server.id)),
                member.user,
            )
        })
        .filter(|(name, user)| {
            name.to_lowercase().contains(&query) || handle(*user).to_lowercase().contains(&query)
        })
        .collect();
    people.sort_by_key(|(name, _)| name.to_lowercase());
    people.truncate(8);
    people
}

/// The faces the permissions page draws (with their
/// initials): the server's owner, the people with an overwrite, and while
/// the ⊕ popout is open, the people it offers.
pub(crate) fn wanted(ui: &Ui, store: &Store, add: &mut dyn FnMut(Option<&RemoteImage>)) {
    /// How many people with an overwrite ask for their faces.
    const LISTED: usize = 50;
    let Some(settings) = ui
        .channel_settings
        .as_ref()
        .filter(|settings| settings.page == Page::Permissions)
    else {
        return;
    };
    let Some(channel) = store.channel(settings.channel) else {
        return;
    };
    let Some(server) = channel.server.and_then(|server| store.server(server)) else {
        return;
    };
    let id = Some(server.id);
    if let Some(owner) = server.owner {
        add(store.avatar_in(id, owner));
    }
    for user in settings
        .listed(channel)
        .into_iter()
        .filter_map(|target| match target {
            OverwriteTarget::Member(user) => Some(user),
            OverwriteTarget::Role(_) => None,
        })
        .take(LISTED)
    {
        add(store.avatar_in(id, user));
    }
    if settings.adding {
        for (_, user) in offered_people(settings, store, channel, server) {
            add(store.avatar_in(id, user));
        }
    }
}

/// What every part of the view reads.
struct View<'a> {
    settings: &'a ChannelSettings,
    store: &'a Store,
    channel: &'a Channel,
    server: &'a Server,
    abilities: &'a Abilities,
    look: &'a Look,
    metrics: Metrics,
    advanced: bool,
    /// Developer mode: the menus offer to copy ids.
    developer: bool,
    /// For the people's pictures already loaded.
    ui: &'a Ui,
}

impl View<'_> {
    fn category(&self) -> bool {
        self.channel.kind == ChannelKind::Category
    }

    /// The channel as its title names it: `#name`, or its name alone.
    fn title(&self) -> String {
        match self.channel.kind {
            ChannelKind::Text => format!("#{}", self.channel.name),
            _ => self.channel.name.clone(),
        }
    }

    fn sidebar(&self) -> Element {
        let page = |id: &str, label: &str, page: Page| SidebarEntry::Item {
            id: format!("cs:page:{id}"),
            label: label.into(),
            selected: self.settings.page == page,
            danger: false,
        };
        let (icon, parent) = match self.channel.kind {
            ChannelKind::Category => (Icon::FolderFill, false),
            kind => (
                crate::channel_list::glyph(kind).unwrap_or(if kind.is_voice() {
                    Icon::Volume
                } else {
                    Icon::Hash
                }),
                true,
            ),
        };
        let detail = self
            .channel
            .parent
            .filter(|_| parent)
            .and_then(|category| self.store.channel(category))
            .map(|category| category.name.clone());
        let mut entries = vec![
            SidebarEntry::Title {
                icon: Some(icon),
                name: self.channel.name.clone(),
                detail,
            },
            page("overview", t("Vista general"), Page::Overview),
            page("permissions", t("Permisos"), Page::Permissions),
        ];
        if self.channel.kind == ChannelKind::Forum {
            entries.push(page("forum", t("Foro"), Page::Forum));
        }
        // A category has neither invitations nor integrations of its own.
        if !self.category() {
            entries.push(page("invites", t("Invitaciones"), Page::Invites));
            entries.push(page("integrations", t("Integraciones"), Page::Integrations));
        }
        if self.abilities.manage.allowed() {
            entries.push(SidebarEntry::Separator);
            entries.push(SidebarEntry::Item {
                id: "cs:delete".into(),
                label: if self.category() {
                    t("Eliminar categoría")
                } else {
                    t("Eliminar canal")
                }
                .into(),
                selected: false,
                danger: true,
            });
        }
        let footnote = match &self.store.provider {
            Some(info) if info.kind == ProviderKind::Simulated => {
                tr!("{name} · sin conexión real", name = t(&info.name))
            }
            Some(info) => t(&info.name).to_owned(),
            None => t("Sin proveedor").into(),
        };
        parts::sidebar(&entries, &footnote, self.look)
    }

    /// The channel's options can be changed here: with the provider's
    /// support and the right to manage it.
    fn options(&self) -> bool {
        self.abilities.channel_options.allowed()
    }

    fn overview(&self) -> Element {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let denied = self.abilities.manage.reason();
        let can = denied.is_none();
        let mut column = vec![parts::page_title(t("Vista general"), None, look)];
        if let Some(reason) = denied {
            column.push(gap(0.0, 16.0));
            column.push(parts::denied(reason, look));
        }
        let name = settings
            .edit
            .name
            .clone()
            .unwrap_or_else(|| channel.name.clone());
        let (label, placeholder) = match channel.kind {
            ChannelKind::Category => (t("Nombre de la categoría"), t("Nueva categoría")),
            ChannelKind::Voice | ChannelKind::Stage => (t("Nombre del canal"), t("Canal nuevo")),
            _ => (t("Nombre del canal"), t("canal-nuevo")),
        };
        let (border, focus) = rustcord_components::user_settings::field_outline(look);
        // shortcut: limn's field writes 14 px at its largest, Discord's 16.
        let field = TextField::new(NAME, name.clone(), placeholder, look.palette())
            .size(ControlSize::Spacious)
            .height(FIELD_HEIGHT)
            .radius(8.0)
            .outline(border, focus)
            .accessibility_label(label)
            .enabled(can)
            .render();
        let saved_as = final_name(channel.kind, &name);
        let name_note = if saved_as.is_empty() {
            Some(text(
                t("El nombre no puede quedar vacío."),
                12.0,
                look.danger,
            ))
        } else if channel.kind == ChannelKind::Text && saved_as != name {
            // Text channels take their names lowercase and dashed: say how it will be.
            Some(line(
                tr!("Se va a llamar #{saved_as}", saved_as),
                12.0,
                look.muted,
            ))
        } else {
            None
        };
        column.push(gap(0.0, FIELD_GAP));
        column.push(own::field(label, field, name_note, look));
        match channel.kind {
            ChannelKind::Text | ChannelKind::Announcement | ChannelKind::Forum => {
                column.extend(self.text_overview(can))
            }
            ChannelKind::Voice | ChannelKind::Stage => column.extend(self.voice_overview(can)),
            _ => {}
        }
        parts::page(
            "cs:scroll:overview",
            self.metrics.content,
            vec![Element::column(column)],
        )
    }

    /// The slow mode's field, with what it means for this kind of channel.
    fn slowmode(&self, can: bool) -> Element {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let slowmode = settings.edit.slowmode_secs.unwrap_or(channel.slowmode_secs);
        let select = own::field_select(
            &FieldSelect {
                id: "cs:slowmode".into(),
                name: t("Modo lento").into(),
                value: own::slowmode_label(slowmode),
                options: SLOWMODE
                    .iter()
                    .map(|(secs, label)| {
                        (
                            format!("cs:slowmode:{secs}"),
                            t(label).to_owned(),
                            *secs == slowmode,
                        )
                    })
                    .collect(),
                open: settings.slowmode_open,
                dismiss: "cs:slowmode:dismiss".into(),
                enabled: can,
                width: self.metrics.content,
            },
            look,
        );
        let meaning = if channel.kind == ChannelKind::Forum {
            t("Los miembros solo podrán crear una publicación por intervalo, a menos que tengan el permiso de Omitir el modo lento.")
        } else {
            t("Los miembros solo podrán enviar un mensaje y crear un hilo durante este intervalo, a menos que tengan el permiso de Omitir el modo lento.")
        };
        own::field(
            t("Modo lento"),
            select,
            Some(own::note(&[(meaning, false)], look)),
            look,
        )
    }

    /// «Visibilidad del contenido», each choice enabled when what it
    /// changes may be changed.
    fn visibility(&self) -> Element {
        let settings = self.settings;
        let channel = self.channel;
        let chosen = Visibility::of(
            settings.edit.spoiler.unwrap_or(channel.options.spoiler),
            settings.edit.nsfw.unwrap_or(channel.nsfw),
        );
        let allowed = |spoiler, nsfw| {
            super::visibility_allowed(settings, channel, self.abilities, spoiler, nsfw)
        };
        own::visibility(
            chosen,
            [
                allowed(false, false),
                allowed(true, false),
                allowed(false, true),
            ],
            self.look,
        )
    }

    /// The topic (a forum's guidelines), slow mode, content visibility,
    /// announcements and when new threads hide, as Discord's text channel
    /// shows them.
    fn text_overview(&self, can: bool) -> Vec<Element> {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let topic = settings
            .edit
            .topic
            .clone()
            .unwrap_or_else(|| channel.topic.clone())
            .unwrap_or_default();
        let left = TOPIC_LIMIT.saturating_sub(topic.chars().count());
        let (title, placeholder) = if channel.kind == ChannelKind::Forum {
            (
                t("Directivas de publicación"),
                t("¡Crea directivas de publicación!"),
            )
        } else {
            (
                t("Tema del canal"),
                t("¡Hazle saber a todos cómo usar este canal!"),
            )
        };
        let (border, _) = rustcord_components::user_settings::field_outline(look);
        let topic_field = Element::stack(vec![
            TextArea::new(TOPIC, topic, placeholder, look.palette())
                .accessibility_label(title)
                .height(TOPIC_HEIGHT)
                .enabled(can)
                .render(),
            // What is left to write, at the field's lower right.
            Element::column(vec![
                Element::spacer().flex(1.0),
                Element::row(vec![
                    Element::spacer().flex(1.0),
                    line(left.to_string(), 14.0, look.text).font_family(FontFamily::SansSemibold),
                    gap(16.0, 0.0),
                ]),
                gap(0.0, 8.0),
            ]),
        ])
        .size(None, Some(TOPIC_HEIGHT))
        .radius(8.0)
        .border(border, 1.0)
        .clip_children();
        let mut children = vec![
            gap(0.0, FIELD_GAP),
            own::field(title, topic_field, None, look),
        ];
        // An announcement channel has no slow mode.
        if channel.kind != ChannelKind::Announcement {
            children.push(gap(0.0, FIELD_GAP));
            children.push(self.slowmode(can));
        }
        children.push(gap(0.0, AFTER_SLOWMODE));
        children.push(self.visibility());
        if super::announcements(self.store, channel) {
            let kind = settings.edit.kind.unwrap_or(channel.kind);
            children.push(gap(0.0, FIELD_GAP));
            children.push(rustcord_components::user_settings::toggle(
                "cs:announcement",
                t("Canal de anuncios"),
                Some(t("Publica mensajes que llegan a servidores fuera del tuyo: quien sigue este canal recibe en su servidor las publicaciones que elijas.")),
                kind == ChannelKind::Announcement,
                self.abilities.channel_options.reason(),
                look,
            ));
        }
        // A forum hides its posts on its own page.
        if channel.kind != ChannelKind::Forum {
            let minutes = settings
                .edit
                .archive_after
                .or(channel.options.archive_after)
                // Discord's own when it says nothing: three days.
                .unwrap_or(4320);
            let select = own::field_select(
                &FieldSelect {
                    id: "cs:hide".into(),
                    name: t("Ocultar después de inactividad").into(),
                    value: own::archive_label(minutes),
                    options: rustcord_model::ARCHIVE_AFTER
                        .iter()
                        .map(|after| {
                            (
                                format!("cs:hide:{after}"),
                                own::archive_label(*after),
                                *after == minutes,
                            )
                        })
                        .collect(),
                    open: settings.hide_open,
                    dismiss: "cs:hide:dismiss".into(),
                    enabled: self.options(),
                    width: self.metrics.content,
                },
                look,
            );
            children.push(gap(0.0, AFTER_CHOICES));
            children.push(own::field(
                t("Ocultar después de inactividad"),
                select,
                Some(own::note(
                    &[(
                        t("Los hilos nuevos no aparecerán en la lista de canales tras estar inactivos durante el tiempo especificado."),
                        false,
                    )],
                    look,
                )),
                look,
            ));
        }
        if let Some(reason) = self.abilities.channel_options.reason().filter(|_| can) {
            children.push(gap(0.0, FIELD_GAP));
            children.push(parts::denied(reason, look));
        }
        children
    }

    /// Slow mode, content visibility and, apart, the sound, cameras, size
    /// and region of a voice channel, as Discord's shows them.
    fn voice_overview(&self, can: bool) -> Vec<Element> {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let options = self.options();
        let width = self.metrics.content;
        let mut children = vec![
            gap(0.0, FIELD_GAP),
            self.slowmode(can),
            gap(0.0, AFTER_SLOWMODE),
            self.visibility(),
            own::group_divider(look),
        ];
        if let Some(reason) = self.abilities.channel_options.reason().filter(|_| can) {
            children.push(parts::denied(reason, look));
            children.push(gap(0.0, FIELD_GAP));
        }
        // The sound: the least, the usual and the most, in kilobits.
        let (least, usual, most) = super::bitrates(self.store, channel);
        let bitrate = settings.edit.bitrate.unwrap_or(channel.options.bitrate);
        let bitrate = if bitrate == 0 { usual } else { bitrate };
        let span = |value: u32| (value - least) as f32 / (most - least).max(1) as f32;
        let kbps = |value: u32| tr!("{kbps}kbps", kbps = value / 1000);
        let mut marks = vec![(0.0, kbps(least))];
        if usual > least && usual < most {
            marks.push((span(usual), kbps(usual)));
        }
        marks.push((1.0, kbps(most)));
        children.push(own::field(
            t("Velocidad de transferencia"),
            own::marked_slider(
                "cs:bitrate",
                t("Velocidad de transferencia"),
                span(bitrate.clamp(least, most)),
                &marks,
                options,
                width,
                look,
            ),
            Some(own::note(
                &[(
                    &tr!(
                        "¡TODAS LAS VELOCIDADES! Exceder una velocidad de {kbps} kbps puede afectar negativamente a personas con una mala conexión.",
                        kbps = usual / 1000
                    ),
                    false,
                )],
                look,
            )),
            look,
        ));
        // The cameras.
        let full = settings
            .edit
            .full_video
            .unwrap_or(channel.options.full_video);
        children.push(gap(0.0, FIELD_GAP));
        children.push(own::field(
            t("Calidad del video"),
            own::radio(
                "cs:video",
                t("Calidad del video"),
                &[
                    ("cs:video:auto".into(), t("Automática").into(), None, options),
                    ("cs:video:full".into(), "720p".into(), None, options),
                ],
                if full { "cs:video:full" } else { "cs:video:auto" },
                look,
            ),
            Some(own::note(
                &[
                    (
                        t("Establece la calidad de video de la cámara para todos los participantes del canal. Selecciona "),
                        false,
                    ),
                    (t("Automática"), true),
                    (t(" para un rendimiento óptimo."), false),
                ],
                look,
            )),
            look,
        ));
        // How many fit.
        let limit = settings
            .edit
            .user_limit
            .unwrap_or(channel.options.user_limit)
            .min(super::USER_LIMIT);
        let (lead, tail) = if channel.kind == ChannelKind::Stage {
            (
                t("Limita el número de usuarios que pueden conectarse a este canal de escenario. Los usuarios con el permiso "),
                t(" pueden ignorar este límite e incluir a otros usuarios en el canal."),
            )
        } else {
            (
                t("Limita el número de usuarios que pueden conectarse a este canal de voz. Los usuarios con el permiso "),
                t(" pueden ignorar este límite e incluir a otros usuarios en el canal."),
            )
        };
        children.push(gap(0.0, FIELD_GAP));
        children.push(own::field(
            t("Límite de usuarios"),
            own::marked_slider(
                "cs:user-limit",
                t("Límite de usuarios"),
                limit as f32 / super::USER_LIMIT as f32,
                &[(0.0, "∞".into()), (1.0, super::USER_LIMIT.to_string())],
                options,
                width,
                look,
            ),
            Some(own::note(
                &[(lead, false), (t("Mover miembros"), true), (tail, false)],
                look,
            )),
            look,
        ));
        // Where its calls run, when the provider offers regions.
        let regions = super::regions(self.store);
        if !regions.is_empty() {
            let region = settings
                .edit
                .region
                .clone()
                .unwrap_or_else(|| channel.options.region.clone());
            let name = region.as_deref().map_or_else(
                || t("Automática").to_owned(),
                |id| {
                    regions
                        .iter()
                        .find(|(known, _)| *known == id)
                        .map_or_else(|| id.to_owned(), |(_, name)| t(name).to_owned())
                },
            );
            let mut choices = vec![(
                "cs:region:auto".to_owned(),
                t("Automática").to_owned(),
                region.is_none(),
            )];
            choices.extend(regions.iter().map(|(id, name)| {
                (
                    format!("cs:region:{id}"),
                    t(name).to_owned(),
                    region.as_deref() == Some(*id),
                )
            }));
            children.push(gap(0.0, FIELD_GAP));
            children.push(Element::column(vec![
                strong_line(t("Excepción de región"), 16.0, look.text),
                gap(0.0, 4.0),
                text(
                    t("Cualquier persona de este canal intentará conectarse a la región que establezcas, sin importar dónde viva. Las regiones influyen en la calidad de voz y video. Si lo dejas en automático, el servicio decidirá qué funciona mejor."),
                    14.0,
                    look.muted,
                ),
                gap(0.0, 8.0),
                own::field_select(
                    &FieldSelect {
                        id: "cs:region".into(),
                        name: t("Excepción de región").into(),
                        value: name,
                        options: choices,
                        open: settings.region_open,
                        dismiss: "cs:region:dismiss".into(),
                        enabled: options,
                        width,
                    },
                    look,
                ),
            ]));
        }
        children
    }

    /// The channel's invitations, as Discord's list: who made each, its
    /// code, its uses and what is left of it; «crear uno» makes one.
    fn invites(&self, wall: Timestamp) -> Element {
        let look = self.look;
        let server = self.server.id;
        let manage = self
            .store
            .server_abilities(server)
            .map(|abilities| abilities.manage_invites);
        let mut children = vec![
            parts::page_title(t("Invitaciones"), None, look),
            gap(0.0, 16.0),
            Element::row(vec![
                text(
                    t("Aquí tienes una lista de todos los enlaces de invitación activos. Puedes revocar cualquiera o"),
                    14.0,
                    look.muted,
                )
                .shrink_to_fit(),
                gap(4.0, 0.0),
                rustcord_components::user_settings::link(
                    "cs:invite:create",
                    t("crear uno."),
                    t("Crear una invitación"),
                    look,
                ),
            ]),
            gap(0.0, 24.0),
        ];
        if let Some(reason) = manage.as_ref().and_then(|permit| permit.reason()) {
            children.push(parts::denied(reason, look));
            return parts::page(
                "cs:scroll:invites",
                self.metrics.content,
                vec![Element::column(children)],
            );
        }
        match self.store.invites(server) {
            None => children.push(line(t("Cargando…"), 14.0, look.muted)),
            Some(invites) => {
                let rows: Vec<InviteRow> = invites
                    .iter()
                    .filter(|invite| invite.channel == Some(self.channel.id))
                    .map(|invite| InviteRow {
                        code: invite.code.clone(),
                        inviter: invite.inviter.map(|user| {
                            (
                                self.name_of(user),
                                self.accent_of(user),
                                self.image_of(user),
                            )
                        }),
                        uses: invite
                            .uses
                            .map_or_else(|| "?".into(), |uses| uses.to_string()),
                        expires: invite
                            .expires
                            .map_or_else(|| "∞".into(), |when| countdown(when, wall)),
                        revoke: Some(format!("cs:invite:{}:revoke", invite.code)),
                    })
                    .collect();
                if rows.is_empty() {
                    children.push(
                        Empty::new(
                            t("No hay invitaciones"),
                            t("Nadie creó un enlace a este canal todavía."),
                            look.palette(),
                        )
                        .icon(Icon::Users)
                        .framed()
                        .render(),
                    );
                } else {
                    children.push(own::invite_table(&rows, look));
                }
            }
        }
        parts::page(
            "cs:scroll:invites",
            self.metrics.content,
            vec![Element::column(children)],
        )
    }

    /// The webhooks that post in the channel and the channels it follows,
    /// as Discord's cards, with the webhooks listed under them.
    fn integrations(&self) -> Element {
        let look = self.look;
        let server = self.server.id;
        let manage = self
            .store
            .server_abilities(server)
            .map(|abilities| abilities.manage_webhooks);
        let denied = manage.as_ref().and_then(|permit| permit.reason());
        let mut children = vec![
            parts::page_title(t("Integraciones"), None, look),
            gap(0.0, 20.0),
            text(
                t("Personaliza tu servidor con integraciones. Administra los webhooks y canales que sigues que hacen publicaciones en este canal."),
                14.0,
                look.text,
            ),
            gap(0.0, 32.0),
            Element::panel(Element::spacer())
                .size(None, Some(1.0))
                .background(look.border),
            gap(0.0, 32.0),
        ];
        let own: Vec<&rustcord_model::Webhook> = self
            .store
            .webhooks(server)
            .unwrap_or_default()
            .iter()
            .filter(|webhook| webhook.channel == self.channel.id)
            .collect();
        let incoming: Vec<_> = own
            .iter()
            .filter(|webhook| webhook.kind != WebhookKind::Follower)
            .collect();
        let followed = own
            .iter()
            .filter(|webhook| webhook.kind == WebhookKind::Follower)
            .count();
        let webhooks = match incoming.len() {
            1 => t("1 webhook").to_owned(),
            n => tr!("{n} webhooks", n),
        };
        children.push(own::integration_card(
            Icon::Bolt,
            "Webhooks",
            &webhooks,
            Some(("cs:webhook:new", t("Crear Webhook"), denied.is_none())),
            look,
        ));
        for webhook in &incoming {
            let mut notes = Vec::new();
            if let Some(creator) = webhook.creator {
                notes.push(tr!("Creado por {name}", name = self.name_of(creator)));
            }
            children.push(gap(0.0, 8.0));
            children.push(parts::item_row_for(
                "cs",
                &ItemLine {
                    key: format!("webhook:{}", webhook.id),
                    name: webhook.name.clone(),
                    notes,
                    glyph: Icon::Bolt,
                    actions: vec![("delete", Icon::Trash, t("Eliminar"))],
                    denied: denied.map(str::to_owned),
                },
                look,
            ));
        }
        children.push(gap(0.0, 16.0));
        children.push(own::integration_card(
            Icon::Custom(rustcord_components::glyphs::MEGAPHONE),
            t("Canales que sigues"),
            &match followed {
                1 => t("1 canal").to_owned(),
                n => tr!("{n} canales", n),
            },
            None,
            look,
        ));
        if let Some(reason) = denied {
            children.push(gap(0.0, 16.0));
            children.push(parts::denied(reason, look));
        }
        parts::page(
            "cs:scroll:integrations",
            self.metrics.content,
            vec![Element::column(children)],
        )
    }

    fn role(&self, target: OverwriteTarget) -> Option<&Role> {
        let OverwriteTarget::Role(id) = target else {
            return None;
        };
        self.server.roles.iter().find(|role| role.id == id)
    }

    fn role_color(&self, role: Option<&Role>) -> Color {
        self.look.ink(
            role.and_then(|role| role.color)
                .unwrap_or(DEFAULT_ROLE_COLOR),
        )
    }

    fn name_of(&self, user: UserId) -> String {
        self.store.display_name(user, Some(self.server.id))
    }

    fn accent_of(&self, user: UserId) -> Color {
        self.store
            .user(user)
            .map_or(self.look.faint, |account| self.look.ink(account.accent))
    }

    /// The picture `user` shows in this server, once loaded; [`wanted`]
    /// asks for it.
    fn image_of(&self, user: UserId) -> Option<limn::RasterImage> {
        self.ui
            .picture(self.store.avatar_in(Some(self.server.id), user))
    }

    fn handle_of(&self, user: UserId) -> String {
        self.store
            .user(user)
            .map_or_else(|| user.to_string(), |account| account.name.clone())
    }

    /// What a target is called: a role's name, or a person's.
    fn target_name(&self, target: OverwriteTarget) -> String {
        match target {
            OverwriteTarget::Role(_) => self
                .role(target)
                .map_or_else(|| t("Rol eliminado").to_owned(), |role| role.name.clone()),
            OverwriteTarget::Member(user) => self.name_of(user),
        }
    }

    /// The targets listed, as Discord's client orders them: roles from the
    /// highest, `@everyone`, and people by name.
    fn ordered_targets(&self) -> Vec<OverwriteTarget> {
        let everyone = everyone(self.channel);
        let listed = self.settings.listed(self.channel);
        let mut roles: Vec<OverwriteTarget> = listed
            .iter()
            .copied()
            .filter(|target| matches!(target, OverwriteTarget::Role(_)) && *target != everyone)
            .collect();
        roles.sort_by_key(|target| -self.role(*target).map_or(0, |role| role.position));
        let mut people: Vec<(String, OverwriteTarget)> = listed
            .iter()
            .filter_map(|target| match target {
                OverwriteTarget::Member(user) => {
                    Some((self.name_of(*user).to_lowercase(), *target))
                }
                OverwriteTarget::Role(_) => None,
            })
            .collect();
        people.sort_by(|a, b| a.0.cmp(&b.0));
        roles.push(everyone);
        roles.extend(people.into_iter().map(|(_, target)| target));
        roles
    }

    fn permissions(&self) -> Element {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let everyone = everyone(channel);
        let (title, description) = if self.category() {
            (
                t("Ajustes de la categoría"),
                t("Usa los permisos para personalizar quién puede hacer qué en esta categoría."),
            )
        } else {
            (
                t("Permisos del canal"),
                t("Usa los permisos para personalizar quién puede hacer qué en este canal."),
            )
        };
        let mut children = vec![parts::page_title(title, Some(description), look)];
        let denied = self.abilities.manage_permissions.reason();
        if let Some(reason) = denied {
            children.push(parts::denied(reason, look));
        }
        let category = channel
            .parent
            .filter(|_| !self.category())
            .and_then(|parent| self.store.channel(parent));
        if let Some(category) = category {
            children.push(own::sync_card(
                &category.name,
                sync_commands(channel, category).is_empty(),
                denied.is_none(),
                look,
            ));
        }
        let private_enabled = target_lock(self.store, channel, everyone).is_none()
            && permission_lock(channel, Permissions::VIEW).is_none();
        children.push(own::private_card(
            &PrivateCard {
                category: self.category(),
                on: settings.is_private(channel, everyone),
                enabled: private_enabled,
                roles: self.access_roles(),
                members: self.access_members(),
            },
            look,
        ));
        children.push(parts::divider(look));
        children.push(own::advanced_heading(self.advanced, look));
        if self.advanced {
            children.push(self.advanced_view());
        }
        parts::page("cs:scroll:permissions", self.metrics.content, children)
    }

    /// The roles that may see the channel while it is private.
    fn access_roles(&self) -> Vec<AccessLine> {
        let everyone = everyone(self.channel);
        self.ordered_targets()
            .into_iter()
            .filter(|target| {
                *target != everyone
                    && matches!(target, OverwriteTarget::Role(_))
                    && self
                        .settings
                        .shown(self.channel, *target)
                        .state(Permissions::VIEW)
                        == Some(true)
            })
            .map(|target| {
                let OverwriteTarget::Role(id) = target else {
                    unreachable!("only roles are kept")
                };
                AccessLine {
                    name: self.target_name(target),
                    detail: t("Rol").into(),
                    color: self.role_color(self.role(target)),
                    person: false,
                    image: None,
                    note: None,
                    remove: target_lock(self.store, self.channel, target)
                        .is_none()
                        .then(|| format!("cs:access:role:{id}:remove")),
                }
            })
            .collect()
    }

    /// Who may see the channel while it is private: its owner always, then
    /// the people let in.
    fn access_members(&self) -> Vec<AccessLine> {
        let owner = self.server.owner;
        let mut lines: Vec<AccessLine> = owner
            .map(|user| AccessLine {
                name: self.name_of(user),
                detail: self.handle_of(user),
                color: self.accent_of(user),
                person: true,
                image: self.image_of(user),
                note: Some(t("Propietario del servidor").into()),
                remove: None,
            })
            .into_iter()
            .collect();
        lines.extend(self.ordered_targets().into_iter().filter_map(|target| {
            let OverwriteTarget::Member(user) = target else {
                return None;
            };
            let allowed = self
                .settings
                .shown(self.channel, target)
                .state(Permissions::VIEW)
                == Some(true);
            (allowed && Some(user) != owner).then(|| AccessLine {
                name: self.name_of(user),
                detail: self.handle_of(user),
                color: self.accent_of(user),
                person: true,
                image: self.image_of(user),
                note: None,
                remove: target_lock(self.store, self.channel, target)
                    .is_none()
                    .then(|| format!("cs:access:member:{user}:remove")),
            })
        }));
        lines
    }

    /// The list of roles and people with overwrites, and the permissions of
    /// the one chosen.
    fn advanced_view(&self) -> Element {
        let look = self.look;
        let settings = self.settings;
        let channel = self.channel;
        let everyone = everyone(channel);
        let selected = settings.selected.unwrap_or(everyone);
        let items: Vec<TargetItem> = self
            .ordered_targets()
            .into_iter()
            .map(|target| {
                let (key, id) = match target {
                    OverwriteTarget::Role(role) => ("role", role.0),
                    OverwriteTarget::Member(user) => ("member", user.0),
                };
                let base = format!("cs:target:{key}:{id}");
                let mark = match target {
                    _ if target == everyone => TargetMark::Everyone,
                    OverwriteTarget::Role(_) => {
                        TargetMark::Role(self.role_color(self.role(target)))
                    }
                    OverwriteTarget::Member(user) => {
                        TargetMark::Member(self.accent_of(user), self.image_of(user))
                    }
                };
                let mut menu = if target == everyone {
                    Vec::new()
                } else {
                    vec![
                        MenuAction::new(format!("{base}:remove"), t("Eliminar"))
                            .icon(Icon::Trash)
                            .danger()
                            .denied(target_lock(self.store, channel, target)),
                        MenuAction::new(
                            format!("{base}:copy-id"),
                            if key == "role" {
                                t("Copiar ID del rol")
                            } else {
                                t("Copiar ID del usuario")
                            },
                        )
                        .icon(Icon::Copy),
                    ]
                };
                menu.retain(|action| self.developer || !action.id.ends_with(":copy-id"));
                TargetItem {
                    id: base,
                    name: self.target_name(target),
                    mark,
                    selected: target == selected,
                    menu,
                }
            })
            .collect();
        let can_add = self.abilities.manage_permissions.allowed();
        let add = own::add_popout(
            can_add,
            settings.adding && can_add,
            &settings.add_query,
            &self.add_choices(),
            look,
        );
        let list = own::target_list(&items, add, look);
        let mut column = Vec::new();
        let locked = target_lock(self.store, channel, selected);
        if let Some(reason) = locked
            .as_deref()
            .filter(|_| self.abilities.manage_permissions.allowed())
        {
            column.push(parts::denied(reason, look));
        }
        let overwrite = settings.shown(channel, selected);
        for (title, permissions) in own::permission_groups(channel.kind) {
            column.push(parts::permission_heading(t(title), false, look));
            let rows = permissions
                .iter()
                .map(|(permission, label, description)| {
                    let description = if *permission == Permissions::VIEW && selected == everyone {
                        format!(
                            "{} {}",
                            t(description),
                            own::everyone_view_note(channel.kind)
                        )
                    } else {
                        t(description).to_owned()
                    };
                    // The target's lock is said once, over the list.
                    let unheld = locked
                        .is_none()
                        .then(|| permission_lock(channel, *permission))
                        .flatten();
                    let control = own::tri_state(
                        &own::permission_id(*permission),
                        overwrite.state(*permission),
                        locked.is_none() && unheld.is_none(),
                        t(label),
                        look,
                    );
                    parts::permission_entry(
                        t(label),
                        &description,
                        unheld.as_deref(),
                        control,
                        look,
                    )
                })
                .collect();
            column.push(Element::column(rows));
        }
        Element::row(vec![
            list,
            gap(TARGET_GAP, 0.0),
            Element::column(column).gap(12.0).flex(1.0),
            // Discord's client keeps the controls 16 px clear of the column's end.
            gap(16.0, 0.0),
        ])
    }

    /// What the ⊕ popout offers: roles within reach and people, not listed
    /// yet, that match what is searched.
    fn add_choices(&self) -> Vec<AddChoice> {
        let settings = self.settings;
        let channel = self.channel;
        let query = settings.add_query.trim().to_lowercase();
        let listed = settings.listed(channel);
        let rank = self.store.my_rank(self.server.id);
        let mut choices: Vec<AddChoice> = self
            .server
            .assignable_roles()
            .into_iter()
            .filter(|role| {
                role.position < rank
                    && !listed.contains(&OverwriteTarget::Role(role.id))
                    && role.name.to_lowercase().contains(&query)
            })
            .map(|role| AddChoice {
                id: format!("cs:add:role:{}", role.id),
                name: role.name.clone(),
                color: self.role_color(Some(role)),
                person: false,
                image: None,
            })
            .collect();
        let people = offered_people(settings, self.store, channel, self.server);
        choices.extend(people.into_iter().map(|(name, user)| AddChoice {
            id: format!("cs:add:member:{user}"),
            name,
            color: self.accent_of(user),
            person: true,
            image: self.image_of(user),
        }));
        choices
    }

    /// Who the dialog offers to let into a private channel: roles within
    /// reach that cannot see it yet and match what is searched, and, once
    /// something is searched (after an `@` for people only), the people that
    /// match. Who is chosen stays in view.
    fn audience(&self, access: &super::Access) -> Audience {
        let channel = self.channel;
        let query = access.query.trim().to_lowercase();
        let people_only = query.starts_with('@');
        let query = query.trim_start_matches('@');
        let rank = self.store.my_rank(self.server.id);
        let sees = |target: OverwriteTarget| {
            self.settings
                .shown(channel, target)
                .state(Permissions::VIEW)
                == Some(true)
        };
        let roles = if people_only {
            Vec::new()
        } else {
            self.server
                .assignable_roles()
                .into_iter()
                .filter(|role| {
                    role.position < rank
                        && !sees(OverwriteTarget::Role(role.id))
                        && role.name.to_lowercase().contains(query)
                })
                .map(|role| Choice {
                    id: format!("cs:pick:role:{}", role.id),
                    label: role.name.clone(),
                    chosen: access.roles.contains(&role.id),
                })
                .collect()
        };
        let searching = people_only || !query.is_empty();
        let members = (searching || !access.members.is_empty()).then(|| {
            let mut found: Vec<Choice> = self
                .store
                .members(self.server.id)
                .iter()
                .filter(|member| {
                    Some(member.user) != self.server.owner
                        && !sees(OverwriteTarget::Member(member.user))
                })
                .filter_map(|member| {
                    let name = self.name_of(member.user);
                    let chosen = access.members.contains(&member.user);
                    let matches = searching && name.to_lowercase().contains(query);
                    (chosen || matches).then(|| Choice {
                        id: format!("cs:pick:member:{}", member.user),
                        label: name,
                        chosen,
                    })
                })
                .collect();
            found.sort_by_key(|choice| !choice.chosen);
            found.truncate(8 + access.members.len());
            found
        });
        Audience {
            query: access.query.clone(),
            roles,
            members,
        }
    }

    /// The dialogs over the settings: who may see it, taking the category's
    /// permissions, and deleting it.
    fn dialogs(&self, root: Element, window_width: f32) -> Element {
        let look = self.look;
        let settings = self.settings;
        let palette = look.menu_palette();
        let mut root = root;
        if let Some(access) = &settings.access {
            root = Modal::new(
                root,
                true,
                t("Añadir miembros o roles"),
                self.title(),
                "cs:pick:cancel",
                palette,
            )
            .width(Modal::fit_width(window_width, 510.0))
            .body(audience_picker(ACCESS_QUERY, &self.audience(access), look))
            .footer(own::dialog_buttons(
                ("cs:pick:cancel", t("Cancelar")),
                ("cs:pick:done", t("Listo"), true),
                look,
            ))
            .initial_focus(ACCESS_QUERY)
            .render();
        }
        let category = self
            .channel
            .parent
            .and_then(|parent| self.store.channel(parent));
        if let (true, Some(category)) = (settings.sync, category) {
            let question = Element::formatted_text(vec![
                TextSpan::new(t("¿Quieres que "), look.muted, 15.0, FontFamily::Sans),
                TextSpan::new(self.title(), look.text, 15.0, FontFamily::SansBold),
                TextSpan::new(
                    t(" tenga los permisos de "),
                    look.muted,
                    15.0,
                    FontFamily::Sans,
                ),
                TextSpan::new(category.name.clone(), look.text, 15.0, FontFamily::SansBold),
                TextSpan::new(
                    t("? Sus permisos propios se reemplazan."),
                    look.muted,
                    15.0,
                    FontFamily::Sans,
                ),
            ]);
            let danger = ComponentPalette {
                text: Color::rgb(255, 255, 255),
                surface: look.danger,
                hover: look.danger,
                selected: look.danger,
                ..palette
            };
            root = Modal::new(
                root,
                true,
                t("¿Sincronizar permisos?"),
                "",
                "cs:sync:cancel",
                palette,
            )
            .width(Modal::fit_width(window_width, 425.0))
            .body(question)
            .footer(
                Element::column(vec![
                    Button::new(
                        "cs:sync:cancel",
                        t("Mantener los permisos actuales"),
                        palette,
                    )
                    .variant(ButtonVariant::Secondary)
                    .size(ControlSize::Spacious)
                    .render(),
                    Button::new("cs:sync:confirm", t("Sincronizar permisos"), danger)
                        .variant(ButtonVariant::Solid)
                        .size(ControlSize::Spacious)
                        .render()
                        .center_content(),
                ])
                .gap(9.0),
            )
            .initial_focus("cs:sync:cancel")
            .render();
        }
        if !settings.delete {
            return root;
        }
        // Discord's: the name in bold within the question, and the two
        // buttons side by side, as wide as the dialog.
        let (title, lead, name, tail) = if self.category() {
            (
                t("Eliminar categoría"),
                t("¿Realmente quieres eliminar la categoría "),
                self.channel.name.clone(),
                t("? Sus canales quedan sin categoría. Esta acción no se puede deshacer."),
            )
        } else {
            (
                t("Eliminar canal"),
                t("¿Realmente quieres eliminar "),
                self.title(),
                t("? Esta acción no se puede deshacer."),
            )
        };
        let question = Element::formatted_text(vec![
            TextSpan::new(lead, look.muted, 16.0, FontFamily::Sans),
            TextSpan::new(name, look.text, 16.0, FontFamily::SansSemibold),
            TextSpan::new(tail, look.muted, 16.0, FontFamily::Sans),
        ]);
        let danger = ComponentPalette {
            text: Color::rgb(255, 255, 255),
            surface: look.danger,
            hover: look.danger,
            selected: look.danger,
            ..palette
        };
        // Cancelar takes the focus: Enter does not delete.
        Modal::new(root, true, title, "", "cs:delete:cancel", palette)
            .width(Modal::fit_width(window_width, 400.0))
            .body(question)
            .footer(
                Element::row(vec![
                    Button::new("cs:delete:cancel", t("Cancelar"), palette)
                        .variant(ButtonVariant::Secondary)
                        .size(ControlSize::Spacious)
                        .render()
                        .flex(1.0),
                    Button::new("cs:delete:confirm", title, danger)
                        .variant(ButtonVariant::Solid)
                        .size(ControlSize::Spacious)
                        .render()
                        .center_content()
                        .flex(1.0),
                ])
                .gap(8.0),
            )
            .initial_focus("cs:delete:cancel")
            .render()
    }
}

/// What is left of an invitation until `when`, as Discord counts it down:
/// days, hours, minutes and seconds, two digits each.
fn countdown(when: Timestamp, now: Timestamp) -> String {
    let left = (when.0 - now.0).max(0) / 1000;
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        left / 86_400,
        left % 86_400 / 3600,
        left % 3600 / 60,
        left % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invitation_counts_down_in_days_hours_minutes_and_seconds() {
        let now = Timestamp(1_000_000);
        let later = Timestamp(now.0 + ((8 * 24 + 4) * 3600 + 40 * 60 + 49) * 1000);
        assert_eq!(countdown(later, now), "08:04:40:49");
        assert_eq!(
            countdown(now, later),
            "00:00:00:00",
            "an expired one ends at zero"
        );
    }
}
