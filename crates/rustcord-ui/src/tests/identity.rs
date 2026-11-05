//! The cards of what someone does
//! with their times, the marks around names, the badges and accounts of a
//! profile, and the picker with the plan's limits, the packs and the
//! provider's own GIFs. The simulation answers; nothing reaches a service.

use rustcord_components::identity::{ActivityTime, NameEffect, Platform, RoleMark};
use rustcord_components::Look;
use rustcord_model::{
    Activity, ActivityKind, Perks, Platforms, Presence, Status, Sticker, StickerId, StickerPack,
    UserId,
};
use rustcord_provider::{GifAttribution, GifCategory, GifItem, GifPage, GifQuery, GifResult};

use super::*;

const PERSON: UserId = UserId(17);

fn act(host: &mut Host, id: &str) {
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::Clicked { id: id.into() },
        host.now,
    );
    host.run(intents);
}

/// Someone playing for a while and listening to a track, on the web.
fn busy(host: &mut Host) {
    let wall = host.store.wall();
    host.event(Event::PresenceChanged {
        user: PERSON,
        presence: Presence {
            status: Status::Online,
            custom: None,
            custom_emoji: None,
            activities: vec![
                Activity {
                    kind: ActivityKind::Playing,
                    name: "Factorio".into(),
                    detail: Some("Fábrica".into()),
                    state: None,
                    image: None,
                    small_image: None,
                    started: Some(Timestamp(wall.0 - (48 * 60 + 40) * 1000)),
                    ends: None,
                    ..Default::default()
                },
                Activity {
                    kind: ActivityKind::Listening,
                    name: "Radio".into(),
                    detail: Some("Tema".into()),
                    state: Some("Trío".into()),
                    image: None,
                    small_image: None,
                    started: Some(Timestamp(wall.0 - 175_000)),
                    ends: Some(Timestamp(wall.0 + 35_000)),
                    ..Default::default()
                },
            ],
            platforms: Platforms {
                web: true,
                mobile: true,
                ..Default::default()
            },
        },
    });
}

#[test]
fn a_card_shows_what_someone_does_with_its_times_and_where_from() {
    let mut host = Host::new();
    busy(&mut host);
    let card = crate::profile::card_of(&host.ui, &host.store, None, PERSON, &Look::DARK).unwrap();
    let identity = &card.identity;
    assert_eq!(identity.activities.len(), 2);
    assert_eq!(
        identity.activities[0].time,
        Some(ActivityTime::Elapsed(48 * 60 + 40))
    );
    let Some(ActivityTime::Track {
        at,
        length,
        progress,
    }) = &identity.activities[1].time
    else {
        panic!("a track");
    };
    assert_eq!((*at, *length), (175, 210));
    assert!((progress - 0.8333).abs() < 0.01);
    assert_eq!(identity.platforms, [Platform::Mobile, Platform::Web]);
    // While it is shown, it moves each second.
    host.ui.profile = Some(PERSON);
    assert!(activity_tick(&host.ui, &host.store).is_some_and(|after| after.as_millis() <= 1000));
    host.ui.profile = None;
    assert_eq!(activity_tick(&host.ui, &host.store), None);
}

#[test]
fn lists_say_what_someone_does_else_their_status_with_its_emoji() {
    let mut host = Host::new();
    busy(&mut host);
    let presence = host.store.presence(PERSON);
    assert_eq!(
        crate::identity::status_line(&host.ui, &presence, false),
        (Some("Jugando a Factorio".into()), None),
        "what they do first, with no emoji"
    );
    let emoji = rustcord_model::TextPicture {
        token: "<:ok:1>".into(),
        name: "ok".into(),
        image: rustcord_model::RemoteImage {
            url: "https://simulacion.invalid/emoji/ok.png".into(),
            kind: Default::default(),
        },
    };
    let status = Presence {
        status: Status::Online,
        custom: Some("de vacaciones".into()),
        custom_emoji: Some(emoji.clone()),
        ..Presence::default()
    };
    let (line, shown) = crate::identity::status_line(&host.ui, &status, false);
    assert_eq!(line.as_deref(), Some("de vacaciones"));
    assert_eq!(shown.map(|shown| shown.name), Some("ok".into()));
    // An emoji alone.
    let alone = Presence {
        custom: None,
        ..status
    };
    let (line, shown) = crate::identity::status_line(&host.ui, &alone, false);
    assert!(line.is_none() && shown.is_some());
}

#[test]
fn a_private_rows_line_says_what_they_do_else_their_status_with_its_emoji() {
    let mut host = Host::new();
    host.click("home");
    // The simulation's first private conversation is with Bruno.
    let friend = UserId(11);
    let wall = host.store.wall();
    host.event(Event::PresenceChanged {
        user: friend,
        presence: Presence {
            status: Status::Online,
            activities: vec![Activity {
                kind: ActivityKind::Playing,
                name: "Factorio".into(),
                detail: None,
                state: None,
                image: None,
                small_image: None,
                started: Some(wall),
                ends: None,
                ..Default::default()
            }],
            ..Presence::default()
        },
    });
    let painted = host.painted();
    assert!(
        painted.iter().any(|text| text == "Jugando a Factorio"),
        "{painted:?}"
    );
    let emoji = rustcord_model::TextPicture {
        token: "<:ok:1>".into(),
        name: "ok".into(),
        image: rustcord_model::RemoteImage {
            url: "https://simulacion.invalid/emoji/ok.png".into(),
            kind: Default::default(),
        },
    };
    host.event(Event::PresenceChanged {
        user: friend,
        presence: Presence {
            status: Status::Online,
            custom: Some("de vacaciones".into()),
            custom_emoji: Some(emoji.clone()),
            ..Presence::default()
        },
    });
    let painted = host.painted();
    assert!(
        painted.iter().any(|text| text == "de vacaciones"),
        "{painted:?}"
    );
    assert!(wanted_images(&host.ui, &host.store, &host.tree).contains(&emoji.image));
}

#[test]
fn an_emoji_opens_its_card_which_says_what_it_is_and_copies_it() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    let messages = host.store.timeline(channel).unwrap().messages();
    let index = messages
        .iter()
        .position(|message| message.content.starts_with("Quedó andando"))
        .expect("the simulation's line with server emoji");
    let said = messages[index].clone();
    let token = said.text_references[0].source.clone();
    host.tree.scroll_to_index(&timeline_id(channel), index);
    // A click on a server's emoji: its card, under it.
    let id = format!("msg:{}:emoji:0:{token}", said.id);
    host.click(&id);
    // Placed under it on a layout, shown on the next.
    host.frame();
    let painted = host.painted().join(" ");
    assert!(painted.contains(":chispa:"), "{painted}");
    assert!(painted.contains("Un emoji de un servidor."), "{painted}");
    assert!(painted.contains("no está en esta"), "{painted}");
    // Once a server of the session has it, by its id, under the name it
    // has now, not the token's old name.
    let mut origin = host.store.servers()[1].clone();
    origin.emojis.push(rustcord_model::CustomEmoji {
        id: rustcord_model::EmojiId(900_001),
        server: origin.id,
        name: "chispa_nueva".into(),
        insert: ":chispa_nueva:".into(),
        animated: true,
        available: true,
        roles: Vec::new(),
        image: None,
    });
    let from = rustcord_i18n::tr!("De {server}", server = origin.name);
    host.event(Event::ServerChanged(origin));
    let painted = host.painted().join(" ");
    assert!(painted.contains(&from), "{painted}");
    assert!(painted.contains(":chispa_nueva:"), "{painted}");
    // Copying it copies it as written, and closes it.
    let copied = update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::Clicked {
            id: rustcord_components::emoji_card::ids::COPY.into(),
        },
        host.now,
    );
    assert!(copied.contains(&Intent::Copy(token)), "{copied:?}");
    assert!(!host
        .painted()
        .join(" ")
        .contains("Un emoji de un servidor."));
    // A secondary click on a reaction: a standard emoji's card, there.
    let reacted = host
        .store
        .timeline(channel)
        .unwrap()
        .messages()
        .iter()
        .find_map(|message| Some((message.id, message.reactions.first()?.emoji.clone())))
        .expect("a reaction in the simulation");
    let intents = secondary_click(
        &mut host.ui,
        &host.store,
        &format!("react:{}:{}", reacted.0, reacted.1),
        Point::new(300.0, 300.0),
    )
    .expect("the reaction's card");
    host.run(intents);
    let painted = host.painted().join(" ");
    assert!(painted.contains("emoji estándar"), "{painted}");
    assert!(painted.contains(&rustcord_components::emoji::code(&reacted.1)));
    // Another conversation closes it (it came back there).
    let other = host.channel_named("ayuda");
    host.click(&format!("channel:{other}"));
    host.frame();
    assert!(host.ui.emoji_card.is_none());
}

#[test]
fn marks_follow_the_member_and_their_roles_and_badges_open_their_links() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    let server = host.ui.server.unwrap();
    // Ada wears a decoration, a nameplate and a tag; her role has an emoji.
    let ada = UserId(10);
    let marks = crate::identity::marks(&host.ui, &host.store, Some(server), ada, &Look::DARK);
    assert_eq!(
        marks.tag.as_ref().map(|tag| tag.text.as_str()),
        Some("RUST")
    );
    assert!(marks.plate.is_some());
    if host
        .store
        .member_of(server, ada)
        .is_some_and(|m| !m.roles.is_empty())
    {
        assert!(marks.role.is_some());
    }
    let emoji_role = host
        .store
        .server(server)
        .unwrap()
        .roles
        .iter()
        .find(|role| role.style.emoji.is_some())
        .cloned()
        .unwrap();
    assert!(matches!(
        crate::identity::role_mark(&host.ui, &emoji_role),
        Some(RoleMark::Emoji { emoji, .. }) if emoji == "🛡️"
    ));
    // A gradient follows the role's colors only while role colors show.
    let gradient = host
        .store
        .server(server)
        .unwrap()
        .roles
        .iter()
        .any(|role| role.style.gradient.len() == 2);
    assert!(gradient, "a role of three colors");
    // Her profile, read, shows her badges; one opens its link.
    host.ui.profile = Some(ada);
    assert!(host.store.begin_profile(ada, Some(server), 7).allowed());
    for event in host.provider.manage(Command::LoadProfile {
        request: 7,
        user: ada,
        server: Some(server),
        effect: true,
    }) {
        host.event(event);
    }
    let card =
        crate::profile::card_of(&host.ui, &host.store, Some(server), ada, &Look::DARK).unwrap();
    assert_eq!(card.identity.badges.len(), 8);
    assert!(card.identity.badges[0].link);
    // Each badge's id says whose it is, so that two cards keep apart; the
    // tiered one's hint is its catalog's.
    assert_eq!(card.identity.badges[0].id, format!("pp:badge={ada}:0"));
    let tiered = card.identity.badges[5]
        .hint
        .as_ref()
        .expect("the tiered badge's hint");
    assert_eq!(
        (tiered.eyebrow.as_deref(), tiered.title.as_str()),
        (Some("Antigüedad de la cuenta"), "Flor")
    );
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::Clicked {
            id: format!("pp:badge={ada}:0"),
        },
        host.now,
    );
    assert!(matches!(&intents[..], [Intent::OpenUrl(url)] if url.contains("insignias/pionera")));
    // Beside a conversation it is the same badge under its own id.
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::Clicked {
            id: format!("pp:badge=side:{ada}:1"),
        },
        host.now,
    );
    assert!(matches!(&intents[..], [Intent::OpenUrl(url)] if url.contains("insignias/mejora")));
    // She talks in «Sala común»: her card says so, and its button joins
    // it and closes the card.
    let voice = card.identity.voice.as_ref().expect("Ada is in voice");
    assert_eq!((voice.channel.as_str(), voice.count), ("Sala común", 3));
    assert!(voice.joinable && voice.server.is_some() && !voice.own);
    let id = format!("pp:voice={}", voice.key);
    update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::Clicked { id },
        host.now,
    );
    assert_eq!(host.ui.profile, None);
    // Her connected accounts are in the full profile.
    let accounts = crate::identity::connections(&host.store, ada, Some(server));
    assert_eq!(accounts[0].label, "GitHub");
    assert!(accounts[0].verified && accounts[0].link);
    let _ = channel;
}

#[test]
fn the_picker_offers_other_servers_and_packs_with_the_plans_reasons() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    // Before the plan is known only this server's expressions are offered.
    host.click("composer:emoji");
    assert!(
        crate::expressions::custom_items(&host.ui, &host.store, channel)
            .iter()
            .all(|item| item.section.is_none())
    );
    host.event(Event::Perks(Perks {
        emoji_anywhere: true,
        animated_emoji: true,
        stickers_anywhere: false,
        upgrade: Some("Plus".into()),
        ..Perks::default()
    }));
    let emoji = crate::expressions::custom_items(&host.ui, &host.store, channel);
    let elsewhere: Vec<_> = emoji.iter().filter(|item| item.section.is_some()).collect();
    assert!(!elsewhere.is_empty(), "the owned server's emoji");
    assert!(
        elsewhere.iter().any(|item| item.unavailable.is_none()),
        "usable anywhere with the plan"
    );
    // Stickers of another server need the plan; the tab reads the packs.
    let from = host.sent.len();
    act(&mut host, "composer:emoji:tab=stickers");
    assert!(host.sent[from..]
        .iter()
        .any(|command| matches!(command, Command::LoadStickerPacks)));
    host.event(Event::StickerPacks(vec![StickerPack {
        id: 1,
        name: "Saludos".into(),
        stickers: vec![Sticker {
            id: StickerId(9_301),
            name: "Hola".into(),
            image: None,
            available: true,
        }],
    }]));
    let stickers = crate::expressions::sticker_items(&host.ui, &host.store, channel);
    let other = stickers
        .iter()
        .find(|item| item.id == 9_100)
        .expect("the owned server's sticker");
    assert_eq!(
        other.unavailable.as_deref(),
        Some("Necesitas Plus para usar stickers de otros servidores aquí")
    );
    let pack = stickers.iter().find(|item| item.id == 9_301).unwrap();
    assert_eq!(pack.section.as_deref(), Some("Saludos"));
    assert!(pack.unavailable.is_none(), "a pack's goes anywhere");
    // Chosen and sent: the pack's sticker leaves as itself.
    act(&mut host, "composer:emoji:sticker=9301");
    act(&mut host, "composer:sticker:send");
    assert!(host.sent.iter().any(|command| matches!(
        command,
        Command::SendSticker { sticker, .. } if *sticker == StickerId(9_301)
    )));
    // An animated emoji without the plan says why.
    host.event(Event::Perks(Perks::default()));
    let emoji = crate::expressions::custom_items(&host.ui, &host.store, channel);
    assert!(emoji
        .iter()
        .filter(|item| item.section.is_some())
        .all(|item| item.unavailable.is_some()));
}

#[test]
fn the_gif_tab_searches_the_provider_with_categories_and_suggestions() {
    let mut host = Host::new();
    let channel = host.open_channel("general");
    host.click("composer:emoji");
    let from = host.sent.len();
    act(&mut host, "composer:emoji:tab=gif");
    let trending = host.sent[from..].iter().find_map(|command| match command {
        Command::SearchGifs {
            request,
            query: GifQuery::Trending,
            ..
        } => Some(*request),
        _ => None,
    });
    let request = trending.expect("trending on opening the tab");
    assert!(host.sent[from..]
        .iter()
        .any(|command| matches!(command, Command::SuggestGifs { query, .. } if query.is_empty())));
    let item = |name: &str| GifItem {
        id: name.into(),
        title: name.into(),
        creator: None,
        url: format!("https://simulacion.invalid/gifs/{name}.gif"),
        preview: rustcord_model::RemoteImage {
            url: format!("https://simulacion.invalid/gifs/{name}.png"),
            kind: Default::default(),
        },
        thumbnail: None,
    };
    host.event(Event::GifResults(GifResult {
        request,
        query: GifQuery::Trending,
        offset: 0,
        result: Ok(GifPage {
            items: vec![item("hola")],
            next_offset: None,
            attribution: GifAttribution {
                label: "GIF simulados".into(),
                url: String::new(),
                image: None,
            },
            categories: vec![GifCategory {
                name: "gatos".into(),
                image: None,
            }],
        }),
    }));
    let view = crate::gifs::view(&host.ui, &host.store);
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.categories[0].0, "gatos");
    assert!(view.attribution.is_none(), "no address to credit");
    // A category searches its name.
    let from = host.sent.len();
    act(&mut host, "composer:emoji:gif:category=0");
    assert!(host.sent[from..].iter().any(|command| matches!(
        command,
        Command::SearchGifs { query: GifQuery::Search(text), .. } if text == "gatos"
    )));
    // Writing asks what to search for; the answer for it shows.
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &limn::UiAction::InputChanged {
            id: crate::gifs::QUERY.into(),
            value: "ga".into(),
        },
        host.now,
    );
    let suggest = intents.iter().find_map(|intent| match intent {
        Intent::Command(Command::SuggestGifs { request, .. }) => Some(*request),
        _ => None,
    });
    host.run(intents);
    host.event(Event::GifSuggestions {
        request: suggest.unwrap(),
        query: "ga".into(),
        terms: vec!["gato".into(), "gatos".into()],
    });
    assert_eq!(
        crate::gifs::view(&host.ui, &host.store).suggestions,
        ["gato", "gatos"]
    );
    let _ = channel;
}

#[test]
fn an_ignored_person_shows_no_activity_only_the_status() {
    let host = Host::new();
    let playing = rustcord_model::Presence {
        status: rustcord_model::Status::Online,
        custom: Some("de vacaciones".into()),
        activities: vec![rustcord_model::Activity {
            kind: rustcord_model::ActivityKind::Playing,
            name: "Factorio".into(),
            ..Default::default()
        }],
        ..rustcord_model::Presence::default()
    };
    let (line, _) = crate::identity::status_line(&host.ui, &playing, false);
    assert_eq!(line.as_deref(), Some("Jugando a Factorio"));
    let (line, _) = crate::identity::status_line(&host.ui, &playing, true);
    assert_eq!(line.as_deref(), Some("de vacaciones"));
}

#[test]
fn a_profile_takes_its_colors_and_moves_its_own_picture_and_banner_while_shown() {
    use rustcord_model::{ImageKind, ProfileBanner, RemoteImage, Rgb, UserProfile};
    let mut host = Host::new();
    host.open_channel("general");
    let ada = UserId(10);
    let picture = |name: &str, kind| RemoteImage {
        url: format!("https://cdn.example.test/{name}"),
        kind,
    };
    let (face, banner) = (
        picture("ada.png", ImageKind::Artwork),
        picture("ada-banner.png", ImageKind::Banner),
    );
    let (moving_face, moving_banner) = (
        picture("ada.webp", ImageKind::Artwork),
        picture("ada-banner.webp", ImageKind::AttachmentPreview),
    );
    let mut account = host.store.user(ada).unwrap().clone();
    account.avatar = Some(face.clone());
    account.banner = ProfileBanner::Image(banner.clone());
    host.event(Event::UserChanged(account));
    host.ui.profiles.full = Some((ada, None));
    assert!(host.store.begin_profile(ada, None, 7).allowed());
    host.event(Event::ProfileLoaded {
        request: 7,
        profile: UserProfile {
            user: ada,
            theme: Some((Rgb(0x2B_6C_B0), Rgb(0x9F_7A_EA))),
            moving_avatar: Some((face.clone(), moving_face.clone())),
            moving_banner: Some((banner.clone(), moving_banner.clone())),
            ..Default::default()
        },
    });
    // Shown, both are asked for and may play.
    let wanted = wanted_images(&host.ui, &host.store, &host.tree);
    let playable = crate::playable_images(&host.ui, &host.store, &host.tree, false);
    for moving in [&moving_face, &moving_banner] {
        assert!(wanted.contains(moving) && playable.contains(moving));
    }
    // Loaded and played by the host, the card draws them moving, in her colors.
    let still = super::attachments::image();
    host.ui.images.insert(moving_face.clone(), still.clone());
    host.ui.animated.insert(still.sha256().to_owned());
    let card = crate::profile::card_of(&host.ui, &host.store, None, ada, &Look::DARK).unwrap();
    assert!(card.theme.is_some_and(|theme| theme.dark));
    assert_eq!(card.image, Some(still));
    assert!(
        card.avatar_playing && !card.banner_playing,
        "the banner is not loaded"
    );
    // With motion reduced, nothing moves nor is asked for.
    host.ui.reduced_motion = true;
    let card = crate::profile::card_of(&host.ui, &host.store, None, ada, &Look::DARK).unwrap();
    assert!(
        !card.avatar_playing && card.image.is_none(),
        "the still one, not loaded"
    );
    assert!(!wanted_images(&host.ui, &host.store, &host.tree).contains(&moving_face));
    // A picture changed since moves no more: the moving one was another's.
    host.ui.reduced_motion = false;
    let mut account = host.store.user(ada).unwrap().clone();
    account.avatar = Some(picture("nueva.png", ImageKind::Artwork));
    host.event(Event::UserChanged(account));
    assert_eq!(
        crate::profile::moving_pictures(&host.ui, &host.store, ada, None),
        [None, Some(&moving_banner)]
    );
}

#[test]
fn a_name_style_shows_whole_outside_servers_only_its_font_in_them_and_never_hidden() {
    let mut host = Host::new();
    // Camila (simulated) writes her name in a neon style, in a font the
    // simulation has no face for.
    let camila = UserId(12);
    let direct = host
        .store
        .directs()
        .into_iter()
        .find(|channel| channel.kind == ChannelKind::Direct && channel.recipients == [camila])
        .map(|channel| channel.id)
        .unwrap();
    host.click("home");
    host.click(&format!("channel:{direct}"));
    host.tree.scroll_to_end(&timeline_id(direct));
    let name = host.store.display_name(camila, None);
    let in_font = |frame: &Frame| {
        frame
            .commands
            .iter()
            .filter(|command| {
                matches!(command, limn::DrawCommand::Text {
                    text,
                    font_family: limn::FontFamily::Custom(1),
                    ..
                } if *text == name)
            })
            .count()
    };
    let glowing = |frame: &Frame| {
        frame.commands.iter().any(|command| match command {
            limn::DrawCommand::TextEffect { runs, .. } => runs
                .iter()
                .any(|run| matches!(run, limn::DrawCommand::Text { text, .. } if *text == name)),
            _ => false,
        })
    };
    // Her messages' name and the profile beside the conversation.
    let frame = host.frame();
    assert!(in_font(&frame) >= 2, "{}", in_font(&frame));
    assert!(glowing(&frame));
    assert_eq!(wanted_fonts(&frame), vec![1], "its face is asked for");
    // In a server her name keeps its font, in her role's color, without
    // the effect; her card there shows all of it.
    let server = host
        .store
        .servers()
        .iter()
        .find(|server| host.store.member_of(server.id, camila).is_some())
        .map(|server| server.id)
        .unwrap();
    let look = host.ui.look();
    let marks = crate::identity::marks(&host.ui, &host.store, Some(server), camila, &look);
    let style = marks.style.unwrap();
    assert_eq!(
        (style.font, style.effect, style.colors.len()),
        (Some(1), NameEffect::Solid, 0)
    );
    let card = crate::identity::card(&host.ui, &host.store, Some(server), camila, &[], &look);
    let style = card.marks.style.unwrap();
    assert_eq!((style.effect, style.colors.len()), (NameEffect::Neon, 2));
    // Hidden in the settings: neither drawn nor asked for.
    host.ui.prefs.cosmetics.name_styles = false;
    host.click("home");
    host.click(&format!("channel:{direct}"));
    let frame = host.frame();
    assert_eq!((in_font(&frame), glowing(&frame)), (0, false));
    assert!(wanted_fonts(&frame).is_empty());
}
