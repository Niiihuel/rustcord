//! What the rest of the administration shows: the safety page, the
//! page that reorders channels, the picture and traits on the profile, a
//! role's picture, emoji and gradient, the members' order and filter, and
//! the button that pauses invitations.

use limn::{
    Button, ButtonSize, ButtonVariant, Component, DropdownOption, Element, Icon, Select, TextField,
    Toggle,
};
use rustcord_components::server_admin::{self as admin_parts, ChannelSlot};
use rustcord_components::server_settings::{
    self as parts, ChoiceCard, DropEdge, MenuAction, Swatches,
};
use rustcord_components::{gap, line, text, user_settings};
use rustcord_i18n::{t, tr};
use rustcord_model::{
    ChannelId, ChannelKind, JoinMethod, MediaScan, MemberOrder, Role, UserId, Verification,
    AFK_TIMEOUTS, MAX_TRAITS,
};
use rustcord_provider::ProviderKind;

use super::{View, FIELD_HEIGHT};
use crate::server_settings::rest::{self, ROLE_EMOJI};

impl View<'_> {
    fn simulated_provider(&self) -> bool {
        self.store
            .provider
            .as_ref()
            .is_some_and(|info| info.kind == ProviderKind::Simulated)
    }

    /// A dropdown of these pages, open while `key` names it.
    fn pick(
        &self,
        key: &'static str,
        current: String,
        options: Vec<(String, String, bool)>,
        label: &str,
        enabled: bool,
    ) -> Element {
        // Narrower in a narrow window.
        let width = self.metrics.content.min(380.0);
        self.pick_sized(key, current, options, label, enabled, width)
    }

    /// [`Self::pick`] `width` wide, its list too.
    fn pick_sized(
        &self,
        key: &'static str,
        current: String,
        options: Vec<(String, String, bool)>,
        label: &str,
        enabled: bool,
        width: f32,
    ) -> Element {
        // Discord's settings select, as the account's settings draw it.
        self.look
            .select(Select::new(
                format!("ss:pick:{key}"),
                current,
                options
                    .into_iter()
                    .map(|(value, name, selected)| {
                        DropdownOption::new(format!("ss:set:{key}:{value}"), name)
                            .selected(selected)
                    })
                    .collect(),
                self.settings.rest.select == Some(key),
                "ss:pick-dismiss",
                self.look.palette(),
            ))
            .size(limn::ControlSize::Spacious)
            .accessibility_label(label)
            .width(width)
            .menu_width(width)
            .enabled(enabled)
            .render()
    }

    /// The channels of `kinds`, in the list's order, named as the list
    /// names them: `#` before a text channel's name.
    pub(super) fn channels_of(&self, kinds: &[ChannelKind]) -> Vec<(ChannelId, String)> {
        self.store
            .channel_tree(self.server.id)
            .into_iter()
            .flat_map(|(_, channels)| channels)
            .filter(|channel| kinds.contains(&channel.kind))
            .map(|channel| {
                let name = if channel.kind.is_voice() {
                    channel.name.clone()
                } else {
                    format!("#{}", channel.name)
                };
                (channel.id, name)
            })
            .collect()
    }

    /// A channel to choose, or none, as a dropdown of `key`, `width` wide.
    #[allow(clippy::too_many_arguments)]
    fn channel_pick_sized(
        &self,
        key: &'static str,
        chosen: Option<ChannelId>,
        kinds: &[ChannelKind],
        none: &str,
        label: &str,
        enabled: bool,
        width: f32,
    ) -> Element {
        let channels = self.channels_of(kinds);
        let current = chosen
            .and_then(|chosen| channels.iter().find(|(id, _)| *id == chosen))
            .map_or_else(|| none.to_owned(), |(_, name)| name.clone());
        let mut options = vec![("none".to_owned(), none.to_owned(), chosen.is_none())];
        options.extend(
            channels
                .iter()
                .map(|(id, name)| (id.to_string(), name.clone(), chosen == Some(*id))),
        );
        self.pick_sized(key, current, options, label, enabled, width)
    }

    /// Discord's «Interacción»: the service's own messages and where they
    /// go, how a newcomer is notified, and where who stays idle in voice is
    /// moved.
    pub(super) fn engagement(&self) -> Element {
        let look = self.look;
        let denied = self.abilities.edit_safety.reason();
        let can = denied.is_none();
        let options = rest::shown_options(self.server, &self.settings.profile);
        let messages = options.system_messages;
        let mut children = vec![parts::page_title(
            t("Interacción"),
            Some(t(
                "Administra los ajustes que ayudan a mantener tu servidor activo.",
            )),
            look,
        )];
        if let Some(reason) = denied {
            children.push(parts::denied(reason, look));
        }
        let switch = |id: &str, label: &str, on: bool| {
            user_settings::toggle(id, label, None, on, denied, look)
        };
        children.extend([
            gap(0.0, 8.0),
            parts::legend(
                t("Mensajes del sistema"),
                Some(t("Configurar mensajes de eventos del sistema enviados a tu servidor.")),
                look,
            ),
            Element::column(vec![
                switch(
                    "ss:system-messages:joins",
                    t("Envía un mensaje de bienvenida aleatorio cuando alguien se una a este servidor."),
                    messages.joins,
                ),
                switch(
                    "ss:system-messages:join-replies",
                    t("Sugiere a los miembros que respondan a los mensajes de bienvenida con un sticker."),
                    messages.join_replies,
                ),
                switch(
                    "ss:system-messages:boosts",
                    t("Envía un mensaje cuando alguien mejore este servidor."),
                    messages.boosts,
                ),
                switch(
                    "ss:system-messages:tips",
                    t("Envía sugerencias útiles para la configuración del servidor."),
                    messages.tips,
                ),
                user_settings::setting(
                    t("Canal de mensajes del sistema"),
                    Some(t("Este es el canal al que enviamos mensajes de eventos del sistema. Puedes desactivarlos en cualquier momento.")),
                    Some(self.channel_pick_sized(
                        "system",
                        options.system_channel,
                        &[ChannelKind::Text],
                        t("Sin mensajes del sistema"),
                        t("Canal de mensajes del sistema"),
                        can,
                        238.0,
                    )),
                    can,
                    look,
                ),
            ]),
            parts::rule(look),
            parts::titled(
                t("Ajustes predeterminados de notificaciones"),
                Some(t("Esto determinará si los miembros que no han configurado explícitamente sus ajustes de notificaciones recibirán o no una notificación por cada mensaje enviado en este servidor.")),
                look,
            ),
            user_settings::choices(
                "ss:notify",
                t("Ajustes predeterminados de notificaciones"),
                &[
                    ("ss:set:notify:all".to_owned(), t("Todos los mensajes"), None),
                    ("ss:set:notify:mentions".to_owned(), t("Solo @menciones"), None),
                ],
                if options.mentions_only {
                    "ss:set:notify:mentions"
                } else {
                    "ss:set:notify:all"
                },
                look,
            ),
            text(
                t("Te recomendamos encarecidamente que configures esta opción como solo @menciones para un servidor de comunidad."),
                12.0,
                look.muted,
            ),
            parts::rule(look),
        ]);
        let half = (self.metrics.content - 16.0) / 2.0;
        children.push(
            Element::row(vec![
                Element::column(vec![
                    parts::titled(t("Canal inactivo"), None, look),
                    self.channel_pick_sized(
                        "afk",
                        options.afk_channel,
                        &[ChannelKind::Voice],
                        t("No hay canal inactivo"),
                        t("Canal inactivo"),
                        can,
                        half,
                    ),
                ])
                .gap(8.0)
                .size(Some(half), None),
                Element::column(vec![
                    parts::titled(t("Suspensión por inactividad"), None, look),
                    self.pick_sized(
                        "afk-timeout",
                        afk_label(options.afk_timeout),
                        AFK_TIMEOUTS
                            .iter()
                            .map(|secs| {
                                (
                                    secs.to_string(),
                                    afk_label(*secs),
                                    *secs == options.afk_timeout,
                                )
                            })
                            .collect(),
                        t("Suspensión por inactividad"),
                        can,
                        half,
                    ),
                ])
                .gap(8.0)
                .size(Some(half), None),
            ])
            .gap(16.0),
        );
        children.push(
            text(
                t("Mueve automáticamente a los miembros a este canal y los silencia cuando estuvieron inactivos más que la suspensión por inactividad."),
                14.0,
                look.muted,
            )
            .font_family(limn::FontFamily::SansMedium),
        );
        parts::page("ss:scroll:engagement", self.metrics.content, children)
    }

    /// Discord's «Configuración de seguridad»: what someone needs before
    /// writing, and whose images are filtered, each a level among cards.
    pub(super) fn safety(&self) -> Element {
        let look = self.look;
        let denied = self.abilities.edit_safety.reason();
        let can = denied.is_none();
        let options = rest::shown_options(self.server, &self.settings.profile);
        let select = self.settings.rest.select;
        let mut children = vec![parts::page_title(
            t("Configuración de seguridad"),
            None,
            look,
        )];
        if let Some(reason) = denied {
            children.push(parts::denied(reason, look));
        }
        let cards = |key: &str, chosen: usize, levels: &[(&str, &str)]| {
            levels
                .iter()
                .enumerate()
                .map(|(index, (name, note))| ChoiceCard {
                    id: format!("ss:set:{key}:{index}"),
                    name: (*name).to_owned(),
                    note: (*note).to_owned(),
                    chosen: index == chosen,
                })
                .collect::<Vec<_>>()
        };
        let verification = Verification::ALL
            .iter()
            .position(|level| *level == options.verification)
            .unwrap_or(0);
        let scan = MediaScan::ALL
            .iter()
            .position(|scan| *scan == options.media_scan)
            .unwrap_or(0);
        children.extend([
            gap(0.0, 8.0),
            parts::titled(
                t("Nivel de verificación"),
                Some(t("Los integrantes del servidor deben cumplir los siguientes requisitos antes de poder enviar mensajes en los canales de texto o iniciar una conversación de mensajes directos. Si un miembro tiene un rol asignado y el proceso de incorporación al servidor no está activado, estos criterios no aplican.")),
                look,
            ),
            parts::choice_cards(
                &cards(
                    "verification",
                    verification,
                    &Verification::ALL.map(verification_card),
                ),
                "ss:pick:verification",
                select == Some("verification"),
                can,
                look,
            ),
            parts::rule(look),
            parts::titled(
                t("Filtros de contenido delicado"),
                Some(t("Elige si los miembros del servidor pueden compartir imágenes que los filtros de contenido delicado del servicio detecten. Se aplica a los canales que no tienen restricción de edad.")),
                look,
            ),
            parts::choice_cards(
                &cards("media-scan", scan, &MediaScan::ALL.map(scan_card)),
                "ss:pick:media-scan",
                select == Some("media-scan"),
                can,
                look,
            ),
        ]);
        parts::page("ss:scroll:safety", self.metrics.content, children)
    }

    /// Becoming a community, Discord's «Activa la comunidad»; once one, its
    /// overview: its rules channel and the one for the service's news, and
    /// stopping it being one.
    pub(super) fn community_page(&self) -> Element {
        let look = self.look;
        let options = rest::shown_options(self.server, &self.settings.profile);
        let toggle = self.abilities.toggle_community.reason();
        let needs = t(
            "Una comunidad necesita al menos verificación baja y revisar el contenido de todos.",
        );
        if !options.community {
            let mut children = vec![
                gap(0.0, 40.0),
                line(t("¿Estás creando una comunidad?"), 24.0, look.text)
                    .font_family(limn::FontFamily::SansSemibold)
                    .center_content(),
                text(
                    t("Convierte tu servidor en un servidor de comunidad para acceder a herramientas de administración adicionales que pueden ayudarte a moderar y expandir tu servidor."),
                    16.0,
                    look.muted,
                )
                .center_content(),
                Element::row(vec![
                    Element::spacer().flex(1.0),
                    Button::new("ss:community", t("Activa la comunidad"), look.accent_palette())
                        .enabled(toggle.is_none())
                        .tooltip(toggle.unwrap_or(needs))
                        .render(),
                    Element::spacer().flex(1.0),
                ]),
            ];
            if let Some(reason) = toggle {
                children.push(parts::denied(reason, look));
            }
            children.push(parts::rule(look));
            children.push(text(needs, 14.0, look.muted).center_content());
            return parts::page("ss:scroll:community", self.metrics.content, children);
        }
        let channels = self.abilities.edit_community.reason();
        let mut children = vec![parts::page_title(
            t("Vista general de la comunidad"),
            Some(t("Tiene un canal de reglas y otro donde el servicio avisa novedades a la moderación.")),
            look,
        )];
        if let Some(reason) = channels {
            children.push(parts::denied(reason, look));
        }
        children.extend([
            gap(0.0, 8.0),
            user_settings::setting(
                t("Canal de normas o directivas"),
                Some(t("Donde se leen las reglas del servidor.")),
                Some(self.channel_pick_sized(
                    "rules",
                    options.rules_channel,
                    &[ChannelKind::Text],
                    t("Elige un canal"),
                    t("Canal de normas o directivas"),
                    channels.is_none(),
                    238.0,
                )),
                channels.is_none(),
                look,
            ),
            user_settings::setting(
                t("Canal de actualizaciones de la comunidad"),
                Some(t(
                    "Donde el servicio avisa a la moderación de cambios y novedades.",
                )),
                Some(self.channel_pick_sized(
                    "updates",
                    options.updates_channel,
                    &[ChannelKind::Text],
                    t("Elige un canal"),
                    t("Canal de actualizaciones de la comunidad"),
                    channels.is_none(),
                    238.0,
                )),
                channels.is_none(),
                look,
            ),
        ]);
        if !options.community_ready() {
            children.push(text(
                t("Falta algo de lo que una comunidad necesita: los dos canales, verificación y revisar el contenido de todos."),
                14.0,
                look.danger,
            ));
        }
        children.extend([
            parts::rule(look),
            user_settings::setting(
                t("Desactiva la comunidad"),
                Some(t("El servidor deja de ser una comunidad: pierde la incorporación y el canal de actualizaciones.")),
                Some(small_danger(
                    "ss:community",
                    t("Desactiva la comunidad"),
                    toggle.is_none(),
                    look,
                )),
                toggle.is_none(),
                look,
            ),
        ]);
        parts::page("ss:scroll:community", self.metrics.content, children)
    }

    /// The banner the card shows: the one chosen, else the server's own
    /// unless it is being taken away.
    pub(super) fn shown_banner(&self) -> Option<limn::RasterImage> {
        match &self.settings.profile.banner {
            Some(Some(_)) => self.settings.rest.banner_preview.clone(),
            Some(None) => None,
            None => self.ui.image(&self.server.options.banner),
        }
    }

    /// The icon the card shows: the one chosen, else the server's own
    /// unless it is being taken away.
    pub(super) fn shown_icon(&self) -> Option<limn::RasterImage> {
        match &self.settings.profile.icon {
            Some(Some(_)) => self.settings.rest.icon_preview.clone(),
            Some(None) => None,
            None => self.ui.image(&self.server.icon),
        }
    }

    /// The banner's buttons on the profile, where the provider has banners,
    /// and what the server's boosts say of moving ones.
    pub(super) fn banner_fields(&self) -> Vec<Element> {
        let look = self.look;
        if !self
            .store
            .provider
            .as_ref()
            .is_some_and(|info| info.capabilities.server_banner)
        {
            return Vec::new();
        }
        let denied = self.abilities.edit_banner.reason();
        let can = denied.is_none();
        let has = match &self.settings.profile.banner {
            Some(banner) => banner.is_some(),
            None => self.server.options.banner.is_some(),
        };
        let mut fields = vec![
            parts::rule(look),
            parts::titled(
                t("Banner"),
                Some(if self.simulated_provider() {
                    t("Sobre la lista de canales. En la simulación se guarda su nombre, no la imagen.")
                } else {
                    t("Sobre la lista de canales, en 16:9; se recorta al centro.")
                }),
                look,
            ),
        ];
        match denied {
            Some(reason) => fields.push(parts::denied(reason, look)),
            None => fields.extend(still_note(self.abilities.animated_banner.reason(), look)),
        }
        fields.push(
            Element::row(vec![
                Button::new("ss:banner:pick", t("Cambiar banner"), look.accent_palette())
                    .button_size(ButtonSize::Small)
                    .enabled(can && self.settings.rest.picking.is_none())
                    .render(),
                small_danger("ss:banner:remove", t("Quitar banner"), can && has, look),
            ])
            .gap(8.0),
        );
        fields
    }

    /// The page that reorders channels and moves them between categories.
    pub(super) fn channels(&self) -> Element {
        let look = self.look;
        let rest = &self.settings.rest;
        let denied = self.abilities.reorder_channels.reason();
        let can = denied.is_none();
        let slots = rest::shown_layout(self.settings, self.store);
        let mut children = vec![parts::page_title(
            t("Canales"),
            Some(t("Arrastra los canales para ordenarlos o pasarlos a otra categoría; también desde el menú de cada uno. Se guarda todo junto.")),
            look,
        )];
        if let Some(reason) = denied {
            children.push(parts::denied(reason, look));
        }
        let categories: Vec<(ChannelId, String)> = slots
            .iter()
            .filter(|slot| slot.category)
            .filter_map(|slot| Some((slot.channel, self.store.channel(slot.channel)?.name.clone())))
            .collect();
        let dragged_at = rest
            .dragging
            .and_then(|dragged| slots.iter().position(|slot| slot.channel == dragged));
        let rows: Vec<Element> = slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let channel = self.store.channel(slot.channel)?;
                let id = slot.channel;
                let mut menu = vec![
                    MenuAction::new(format!("ss:chan:{id}:up"), t("Subir")).icon(Icon::ArrowUp),
                    MenuAction::new(format!("ss:chan:{id}:down"), t("Bajar")).icon(Icon::ArrowDown),
                ];
                if !slot.category {
                    menu.push(
                        MenuAction::new(
                            format!("ss:chan:{id}:to:none"),
                            t("Sacar de la categoría"),
                        )
                        .denied(
                            slot.parent
                                .is_none()
                                .then(|| t("Ya está fuera de las categorías.").to_owned()),
                        )
                        .separated(),
                    );
                    for (category, name) in &categories {
                        menu.push(
                            MenuAction::new(
                                format!("ss:chan:{id}:to:{category}"),
                                tr!("Mover a {category}", category = name.to_uppercase()),
                            )
                            .denied(
                                (slot.parent == Some(*category))
                                    .then(|| t("Ya está en esa categoría.").to_owned()),
                            ),
                        );
                    }
                }
                let drop = match (dragged_at, rest.drop_over) {
                    (Some(from), Some(over)) if over == id && from != index => {
                        Some(if from > index {
                            DropEdge::Above
                        } else {
                            DropEdge::Below
                        })
                    }
                    _ => None,
                };
                Some(admin_parts::channel_slot(
                    &ChannelSlot {
                        channel: id,
                        name: channel.name.clone(),
                        category: slot.category,
                        nested: slot.parent.is_some(),
                        glyph: match channel.kind {
                            ChannelKind::Category => Icon::ChevronDown,
                            kind => {
                                crate::channel_list::glyph(kind).unwrap_or(if kind.is_voice() {
                                    Icon::Volume
                                } else {
                                    Icon::Hash
                                })
                            }
                        },
                        movable: can,
                        menu: if can { menu } else { Vec::new() },
                        menu_open: rest.channel_menu == Some(id),
                        drop,
                    },
                    look,
                ))
            })
            .collect();
        if rows.is_empty() {
            children.push(text(t("Este servidor no tiene canales."), 14.0, look.muted));
        } else {
            children.push(Element::column(rows));
        }
        parts::page("ss:scroll:channels", self.metrics.content, children)
    }

    /// Whether the provider keeps the server's picture and traits.
    fn identity(&self) -> bool {
        self.store
            .provider
            .as_ref()
            .is_some_and(|info| info.capabilities.server_identity)
    }

    /// The profile's picture: Discord's «Ícono», with a button that changes
    /// it and, once it has one, one that takes it away.
    pub(super) fn icon_fields(&self) -> Vec<Element> {
        if !self.identity() {
            return Vec::new();
        }
        let look = self.look;
        let denied = self.abilities.edit_identity.reason();
        let can = denied.is_none();
        let has_icon = match &self.settings.profile.icon {
            Some(icon) => icon.is_some(),
            None => self.server.icon.is_some(),
        };
        let note = if self.simulated_provider() {
            tr!(
                "{advice} En la simulación se guarda su nombre, no la imagen.",
                advice = t("Te recomendamos una imagen de al menos 512x512.")
            )
        } else {
            t("Te recomendamos una imagen de al menos 512x512.").to_owned()
        };
        let mut buttons = vec![Button::new(
            "ss:icon:pick",
            t("Cambiar ícono del servidor"),
            look.accent_palette(),
        )
        .button_size(ButtonSize::Small)
        .enabled(can && self.settings.rest.picking.is_none())
        .tooltip(denied.unwrap_or(t("Elegir una imagen")))
        .render()];
        if has_icon {
            buttons.push(small_danger(
                "ss:icon:remove",
                t("Eliminar ícono"),
                can,
                look,
            ));
        }
        let mut fields = vec![
            parts::rule(look),
            parts::titled(t("Ícono"), Some(&note), look),
            Element::row(buttons).gap(8.0),
        ];
        if can {
            fields.extend(still_note(self.abilities.animated_icon.reason(), look));
        }
        fields
    }

    /// The profile's traits, Discord's «Rasgos»: five boxes, three to a row,
    /// each an emoji (or «:nombre:» of one of the server's) and a label.
    pub(super) fn trait_fields(&self, width: f32) -> Vec<Element> {
        if !self.identity() {
            return Vec::new();
        }
        let look = self.look;
        let can = self.abilities.edit_identity.allowed();
        let mut fields = vec![
            parts::rule(look),
            parts::titled(
                t("Rasgos"),
                Some(t("Añade hasta 5 atributos para mostrar los intereses y la personalidad de tu servidor.")),
                look,
            ),
        ];
        let loaded = self.server.options.traits.is_some() || self.settings.profile.traits.is_some();
        if !loaded {
            if self.settings.rest.profile_failed {
                fields.push(
                    Element::row(vec![
                        text(t("No se pudieron leer los rasgos."), 14.0, look.muted).centered_y(),
                        Button::new("ss:profile:retry", t("Reintentar"), look.palette())
                            .variant(ButtonVariant::Link)
                            .render(),
                    ])
                    .gap(8.0),
                );
            } else {
                fields.push(line(t("Cargando…"), 14.0, look.muted));
            }
            return fields;
        }
        let traits = rest::shown_traits(self.settings, self.store);
        let slots: Vec<(String, String)> = (0..MAX_TRAITS)
            .map(|index| {
                traits.get(index).map_or_else(Default::default, |found| {
                    (rest::trait_emoji(found), found.label.clone())
                })
            })
            .collect();
        fields.push(parts::trait_slots(&slots, width, can, look));
        fields
    }

    /// What the server card shows of the traits: each one's emoji and label.
    pub(super) fn trait_chips(&self) -> Vec<String> {
        rest::shown_traits(self.settings, self.store)
            .iter()
            .filter(|found| !found.label.trim().is_empty())
            .map(|found| match rest::trait_emoji(found) {
                emoji if emoji.is_empty() => found.label.trim().to_owned(),
                emoji => format!("{emoji} {}", found.label.trim()),
            })
            .collect()
    }

    /// A role's picture, emoji and gradient, in its editor.
    pub(super) fn role_style_fields(&self, role: &Role, can: bool) -> Vec<Element> {
        let look = self.look;
        let rest = &self.settings.rest;
        if !self
            .store
            .provider
            .as_ref()
            .is_some_and(|info| info.capabilities.role_styles)
        {
            return Vec::new();
        }
        let icons = self.abilities.role_icons.reason();
        let gradients = self.abilities.role_gradients.reason();
        let preview = rest
            .role_icon_preview
            .as_ref()
            .filter(|(of, _)| *of == role.id)
            .map(|(_, preview)| preview.clone());
        let mark = match (&preview, &role.style.emoji, &role.style.icon) {
            (Some(preview), _, _) => {
                Element::raster_image(preview.clone()).size(Some(32.0), Some(32.0))
            }
            (None, Some(emoji), _) => line(emoji.clone(), 24.0, look.text),
            // The saved picture once loaded; `wanted` asks for it.
            (None, None, Some(icon)) => match self.ui.picture(Some(icon)) {
                Some(picture) => Element::raster_image(picture).size(Some(32.0), Some(32.0)),
                None => line(t("Imagen"), 13.0, look.muted),
            },
            (None, None, None) => line(t("Sin ícono"), 13.0, look.muted),
        };
        let gradient_can = can && gradients.is_none();
        let mut fields = vec![parts::divider(look)];
        fields.push(parts::switch_row(
            "ss:gradient",
            t("Degradé"),
            Some(gradients.unwrap_or(t(
                "El nombre pasa del color del rol a uno o dos colores más.",
            ))),
            !role.style.gradient.is_empty(),
            gradient_can,
            look,
        ));
        if let Some(second) = role.style.gradient.first() {
            // How names look with it.
            let ink = self.role_color(role);
            let stops: Vec<limn::Color> = role
                .style
                .gradient
                .iter()
                .map(|color| look.ink(*color))
                .collect();
            fields.push(rustcord_components::identity::inked(
                rustcord_components::strong_line(role.name.clone(), 16.0, ink),
                ink,
                &stops,
            ));
            let custom = |which: u8| {
                rest.gradient_hex
                    .as_ref()
                    .filter(|(of, _)| *of == which)
                    .map(|(_, hex)| hex.clone())
            };
            let narrow = self.editor_width() < parts::swatches_width(true);
            fields.push(parts::field_label(t("Segundo color"), false, look));
            fields.push(parts::swatches(
                &Swatches {
                    prefix: "ss:gradient2".into(),
                    chosen: Some(*second),
                    default: None,
                    custom: custom(2),
                    enabled: gradient_can,
                    narrow,
                    picked: crate::color_picker::picked(self.ui, "ss:gradient2"),
                },
                look,
            ));
            fields.push(parts::field_label(
                t("Tercer color (opcional)"),
                false,
                look,
            ));
            fields.push(parts::swatches(
                &Swatches {
                    prefix: "ss:gradient3".into(),
                    chosen: role.style.gradient.get(1).copied(),
                    // The default swatch is «none»: two colors only.
                    default: Some(*second),
                    custom: custom(3),
                    enabled: gradient_can,
                    narrow,
                    picked: crate::color_picker::picked(self.ui, "ss:gradient3"),
                },
                look,
            ));
        }
        let icon_can = can && icons.is_none();
        fields.extend([
            parts::divider(look),
            parts::field_label(t("Ícono del rol"), false, look),
            text(
                t("Una imagen o un emoji al lado del nombre de quienes lo tienen."),
                14.0,
                look.muted,
            ),
        ]);
        if let Some(reason) = icons {
            fields.push(parts::denied(reason, look));
        }
        fields.push(
            Element::row(vec![
                mark.centered_y(),
                Button::new("ss:role-icon:pick", t("Subir imagen"), look.palette())
                    .icon(Icon::Image)
                    .variant(ButtonVariant::Outline)
                    .enabled(icon_can && rest.picking.is_none())
                    .render()
                    .centered_y(),
                TextField::new(
                    ROLE_EMOJI,
                    role.style.emoji.clone().unwrap_or_default(),
                    t("o un emoji: 🛡️"),
                    look.palette(),
                )
                .height(FIELD_HEIGHT)
                .accessibility_label(t("Emoji del rol"))
                .enabled(icon_can)
                .render()
                .size(Some(150.0), None),
                Button::new("ss:role-icon:remove", t("Quitar"), look.palette())
                    .variant(ButtonVariant::Ghost)
                    .enabled(icon_can && (role.style.has_mark() || preview.is_some()))
                    .render()
                    .centered_y(),
            ])
            .gap(10.0),
        );
        fields
    }

    /// How many have `role`: as the provider counted, else as known here.
    pub(super) fn role_members(&self, role: rustcord_model::RoleId) -> usize {
        self.store
            .role_count(self.server.id, role)
            .map_or_else(|| self.members_with(role), |count| count as usize)
    }

    /// The order and filter of the members page, when the provider has them.
    pub(super) fn member_tools(&self) -> Option<Element> {
        if !self
            .store
            .provider
            .as_ref()
            .is_some_and(|info| info.capabilities.member_queries)
        {
            return None;
        }
        let query = self.settings.rest.query;
        let can = self.abilities.member_queries.allowed();
        let order = self.pick(
            "member-order",
            order_label(query.order).into(),
            MemberOrder::ALL
                .iter()
                .enumerate()
                .map(|(index, order)| {
                    (
                        index.to_string(),
                        order_label(*order).into(),
                        *order == query.order,
                    )
                })
                .collect(),
            t("Ordenar miembros"),
            can,
        );
        let recent = Toggle::new(
            "ss:members:recent",
            t("Últimos 7 días"),
            query.recent,
            self.look.palette(),
        )
        .enabled(can)
        .accessibility_label(t("Solo quienes entraron en los últimos 7 días"))
        .render();
        // One under the other in a narrow window.
        Some(if self.metrics.content >= 380.0 + 10.0 + 170.0 {
            Element::row(vec![order, recent.centered_y()]).gap(10.0)
        } else {
            Element::column(vec![order, Element::row(vec![recent])]).gap(8.0)
        })
    }

    /// The members in the order the provider listed them for an order or
    /// a filter chosen here, once it did; `None` to list everyone known by
    /// name, as without a choice.
    pub(super) fn ordered_members(&self) -> Option<Vec<UserId>> {
        let query = self.settings.rest.query;
        if query == rustcord_model::MemberQuery::default() {
            return None;
        }
        let pages = self.store.member_pages(self.server.id)?;
        (self.abilities.member_queries.allowed() && pages.query == query)
            .then(|| pages.listed.clone())
    }

    /// How `user` joined, as the members page says it.
    pub(super) fn joined_note(&self, user: UserId) -> Option<String> {
        let method = self.store.member_pages(self.server.id)?.joins.get(&user)?;
        let said = match method {
            JoinMethod::Invite(Some(code)) => tr!("entró con la invitación {code}", code),
            JoinMethod::Invite(None) => t("entró con una invitación").into(),
            JoinMethod::VanityLink => t("entró con el enlace propio").into(),
            JoinMethod::Bot => t("lo agregó un bot").into(),
            JoinMethod::Integration => t("entró por una integración").into(),
            JoinMethod::Discovery => t("lo encontró en el directorio").into(),
            JoinMethod::Hub => t("entró por un centro educativo").into(),
            JoinMethod::App => t("entró por una aplicación").into(),
            JoinMethod::Other => return None,
        };
        Some(if self.simulated_provider() {
            tr!("{said} (simulado)", said)
        } else {
            said
        })
    }

    /// The button that pauses or resumes invitations, and the notice that
    /// they are paused.
    pub(super) fn invites_pause(&self) -> Vec<Element> {
        let look = self.look;
        if !self
            .store
            .provider
            .as_ref()
            .is_some_and(|info| info.capabilities.pause_invites)
        {
            return Vec::new();
        }
        let paused = self.server.options.invites_paused;
        let denied = self.abilities.pause_invites.reason();
        let button = Button::new(
            if paused {
                "ss:invites:resume"
            } else {
                "ss:invites:pause"
            },
            if paused {
                t("Reanudar invitaciones")
            } else {
                t("Pausar invitaciones")
            },
            look.palette(),
        )
        .icon(if paused { Icon::Play } else { Icon::Pause })
        .variant(ButtonVariant::Outline)
        .enabled(denied.is_none())
        .tooltip(denied.unwrap_or(if paused {
            t("Que se pueda volver a entrar con invitaciones")
        } else {
            t("Que nadie nuevo entre con invitaciones por ahora")
        }))
        .render();
        let mut out = vec![button];
        if paused {
            out.push(parts::denied(
                t("Las invitaciones están pausadas: nadie nuevo puede entrar con ellas hasta que se reanuden."),
                look,
            ));
        }
        out
    }
}

fn afk_label(secs: u32) -> String {
    match secs {
        60 => t("1 minuto").into(),
        3600 => t("1 hora").into(),
        secs if secs % 60 == 0 => tr!("{minutes} minutos", minutes = secs / 60),
        secs => tr!("{seconds} segundos", seconds = secs),
    }
}

/// A verification level's card: its name and what it asks.
fn verification_card(level: Verification) -> (&'static str, &'static str) {
    match level {
        Verification::None => (t("Ninguno"), t("Sin restricciones")),
        Verification::Low => (
            t("Bajo"),
            t("Debe tener un correo electrónico verificado en su cuenta."),
        ),
        Verification::Medium => (
            t("Medio"),
            t("También debe tener una cuenta de más de 5 minutos."),
        ),
        Verification::High => (
            t("Alto"),
            t("También debe ser miembro de este servidor por más de 10 minutos."),
        ),
        Verification::Highest => (
            t("Más alto"),
            t("Debe tener un teléfono verificado en su cuenta."),
        ),
    }
}

/// A sensitive content filter's card: its name and whose images it filters.
fn scan_card(scan: MediaScan) -> (&'static str, &'static str) {
    match scan {
        MediaScan::Off => (
            t("No filtrar"),
            t("No se filtrarán los mensajes en busca de imágenes delicadas."),
        ),
        MediaScan::WithoutRoles => (
            t("Filtrar mensajes de miembros del servidor sin roles"),
            t("Se filtrarán las imágenes delicadas de los mensajes de quienes no tienen un rol."),
        ),
        MediaScan::Everyone => (
            t("Filtrar mensajes de todos los miembros"),
            t("Se filtrarán las imágenes delicadas de todos los mensajes."),
        ),
    }
}

fn order_label(order: MemberOrder) -> &'static str {
    match order {
        MemberOrder::NewestMembers => t("Últimos en entrar"),
        MemberOrder::OldestMembers => t("Primeros en entrar"),
        MemberOrder::NewestAccounts => t("Cuentas más nuevas"),
        MemberOrder::OldestAccounts => t("Cuentas más antiguas"),
    }
}

/// A small button that takes something away, Discord's: the page's quiet
/// veil with its label in the danger ink.
fn small_danger(id: &str, label: &str, enabled: bool, look: &rustcord_components::Look) -> Element {
    Button::new(id, label, user_settings::veil(look))
        .variant(ButtonVariant::Secondary)
        .button_size(ButtonSize::Small)
        .ink(look.danger)
        .enabled(enabled)
        .render()
}

/// Why a moving picture chosen would be used still, when it would.
fn still_note(reason: Option<&str>, look: &rustcord_components::Look) -> Option<Element> {
    reason.map(|reason| {
        text(
            tr!(
                "{reason}: un GIF se usa quieto.",
                reason = reason.trim_end_matches('.')
            ),
            13.0,
            look.muted,
        )
    })
}
