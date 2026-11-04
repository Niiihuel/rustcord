//! Someone's full profile, read from the undocumented profile route, with
//! the names and icons of the games its `widgets` show (`GET /games`) and
//! the wishlist its `wishlist_settings` name (`GET /wishlists/{id}`), as the
//! official web client asks for them (seen 2026-11-04). Unofficial and not
//! verified against Discord: only the shapes of local fixtures are known to parse.

use rustcord_i18n::es;
use rustcord_model::{
    ImageKind, ProfileBanner, ProfileGame, RecentActivity, RecentKind, RemoteImage, Rgb, ServerId,
    Timestamp, UserId, UserProfile, WishlistItem, MAX_MUTUALS, MAX_PROFILE_GAMES, MAX_RECENT,
    MAX_WISHLIST,
};
use rustcord_provider::{Command, Event, RequestId, Snapshot};
use serde_json::Value;

use crate::{wire, LoginError};

/// Whether the account may change its profile as `command` asks: in a
/// server, only one it is in and not restricted in. Refused here before
/// anything is sent.
pub(crate) fn edit_allowed(snapshot: &Snapshot, command: &Command) -> Result<(), LoginError> {
    let me = snapshot.me.as_ref().ok_or_else(wire::invalid)?.id;
    match command {
        Command::EditProfile(_) => Ok(()),
        Command::EditServerProfile { server, .. } => {
            let member = snapshot
                .members
                .iter()
                .find(|m| m.server == *server && m.user == me)
                .ok_or_else(|| LoginError::new(es("Ya no estás en ese servidor.")))?;
            match member.restrictions.at(crate::messages::wall_now()) {
                Some(_) => Err(LoginError::new(es(
                    "Tu cuenta está restringida en este servidor: no puedes cambiar tu perfil ahí.",
                ))),
                None => Ok(()),
            }
        }
        _ => Err(wire::invalid()),
    }
}

/// The profile answer is read up to this size.
pub(crate) const MAX_RESPONSE: usize = 256 * 1024;

/// The route of `user`'s profile, as seen in `server`. Mutual servers and
/// friends are asked for only about someone else.
pub(crate) fn path(user: UserId, server: Option<ServerId>, mutuals: bool) -> String {
    let mut path = format!(
        "/users/{user}/profile?with_mutual_guilds={mutuals}&with_mutual_friends={mutuals}&with_mutual_friends_count=false"
    );
    if let Some(server) = server {
        path.push_str(&format!("&guild_id={server}"));
    }
    path
}

/// Text the profile shows, cut to `chars`; empty is not reported.
pub(crate) fn text(value: &Value, chars: usize) -> Option<String> {
    let text = value.as_str()?.trim();
    (!text.is_empty()).then(|| {
        text.chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(chars)
            .collect()
    })
}

fn accent(value: &Value) -> Option<Rgb> {
    value
        .as_u64()
        .filter(|color| *color <= 0xFF_FFFF)
        .map(|color| Rgb(color as u32))
}

/// A banner field: missing is unknown, `null` is none, a valid hash is one.
fn banner(value: Option<&Value>, path: &str) -> ProfileBanner {
    match value {
        None => ProfileBanner::Unknown,
        Some(Value::Null) => ProfileBanner::Absent,
        Some(hash) => wire::cdn(path, true, hash)
            .map(ProfileBanner::Image)
            .unwrap_or_default(),
    }
}

/// The first of two places that reports a banner, else the second.
fn either(first: ProfileBanner, second: ProfileBanner) -> ProfileBanner {
    match first {
        ProfileBanner::Image(_) => first,
        _ if second != ProfileBanner::Unknown => second,
        _ => first,
    }
}

/// A list of the mutuals: `None` unless it is an array within bounds.
fn mutual(value: &Value) -> Option<&[Value]> {
    value
        .as_array()
        .filter(|list| list.len() <= MAX_MUTUALS)
        .map(Vec::as_slice)
}

/// The game collection `widgets` show: the favorite games, then the ones
/// played, each once; `None` when the profile has no widgets.
/// shortcut: only those two kinds are known; others (if any) are left out
/// until one is seen.
fn games(widgets: &Value) -> Option<Vec<ProfileGame>> {
    let mut ids: Vec<u64> = Vec::new();
    for kind in ["favorite_games", "played_games"] {
        let listed = widgets
            .as_array()?
            .iter()
            .filter(|widget| widget["data"]["type"] == kind)
            .filter_map(|widget| widget["data"]["games"].as_array())
            .flatten()
            .filter_map(|game| wire::id(&game["game_id"]).ok());
        for id in listed {
            if !ids.contains(&id) && ids.len() < MAX_PROFILE_GAMES {
                ids.push(id);
            }
        }
    }
    Some(
        ids.into_iter()
            .map(|id| ProfileGame {
                id,
                ..Default::default()
            })
            .collect(),
    )
}

/// The answer to [`games_path`] is read up to this size: a game is about 6 KiB.
pub(crate) const MAX_GAMES_RESPONSE: usize = 1024 * 1024;

/// The route that names `games`, as the official client asks for them.
pub(crate) fn games_path(games: &[ProfileGame]) -> String {
    let ids: Vec<String> = games
        .iter()
        .map(|game| format!("game_ids={}", game.id))
        .collect();
    format!("/games?{}", ids.join("&"))
}

/// The names and icons `answer` gives `games`; one it leaves out stays unnamed.
pub(crate) fn name_games(games: &mut [ProfileGame], answer: &Value) {
    for found in answer.as_array().into_iter().flatten() {
        let Some(game) = wire::id(&found["id"])
            .ok()
            .and_then(|id| games.iter_mut().find(|game| game.id == id))
        else {
            continue;
        };
        game.name = text(&found["name"], 128).unwrap_or_default();
        game.icon = found["icon_hash"]
            .as_str()
            .filter(|hash| hash.len() == 32 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|hash| RemoteImage {
                url: format!(
                    "https://cdn.discordapp.com/app-icons/{}/{hash}.png?size=48&keep_aspect_ratio=false",
                    game.id
                ),
                kind: ImageKind::Artwork,
            });
    }
}

/// The route of `user`'s recent activity, as the full profile asks for it.
pub(crate) fn recent_path(user: UserId) -> String {
    format!("/content-inventory/users/{user}/outbox")
}

/// A picture of a recent activity: an `https` one on the hosts its entries
/// use (Spotify's covers, Discord's media proxy and CDN), as given.
fn recent_picture(url: &str) -> Option<RemoteImage> {
    let allowed = [
        "https://i.scdn.co/",
        "https://media.discordapp.net/",
        "https://cdn.discordapp.com/",
    ];
    (allowed.iter().any(|host| url.starts_with(host))
        && url.len() <= 512
        && !url.chars().any(|c| c.is_control() || c.is_whitespace()))
    .then(|| RemoteImage {
        url: url.to_owned(),
        kind: ImageKind::Artwork,
    })
}

/// A watched entry's large asset: a media proxy path (`mp:`) or a Spotify
/// cover (`spotify:`), else an asset of its application.
fn watched_picture(asset: &str, application: Option<u64>) -> Option<RemoteImage> {
    if let Some(path) = asset.strip_prefix("mp:") {
        return recent_picture(&format!("https://media.discordapp.net/{path}"));
    }
    if let Some(id) = asset.strip_prefix("spotify:") {
        return recent_picture(&format!("https://i.scdn.co/image/{id}"));
    }
    let safe = |text: &str| !text.is_empty() && text.bytes().all(|b| b.is_ascii_alphanumeric());
    (safe(asset) && !asset.contains(':'))
        .then_some(application?)
        .and_then(|app| {
            recent_picture(&format!(
                "https://cdn.discordapp.com/app-assets/{app}/{asset}.png?size=160"
            ))
        })
}

/// What `user`'s outbox (`answer`) says they did lately, in its order, as the
/// official client's full profile shows it: games played (and the most
/// played this week), music listened to, what was watched and activities
/// launched; nothing live now, nothing past its `expires_at`. Each one's time
/// is its ID's.
pub(crate) fn recent(answer: &Value, now: Timestamp) -> Vec<RecentActivity> {
    let text = |value: &Value| text(value, 256).filter(|text| !text.trim().is_empty());
    let trait_of = |entry: &Value, kind: u64| {
        entry["traits"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|found| found["type"].as_u64() == Some(kind))
            .cloned()
    };
    answer["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let at = wire::created(UserId(wire::id(&entry["id"]).ok()?))?;
            let expired = wire::timestamp(&entry["expires_at"]).is_ok_and(|end| end.0 <= now.0);
            // Live now: the profile shows it as current activity instead.
            let live =
                trait_of(entry, 3).is_some_and(|found| found["is_live"].as_bool() == Some(true));
            if expired || live {
                return None;
            }
            let extra = &entry["extra"];
            let application = wire::id(&extra["application_id"]).ok();
            let (kind, title, subtitle, image, link) = match entry["content_type"].as_u64()? {
                1 | 3 => (
                    RecentKind::Game,
                    text(&extra["game_name"])?,
                    None,
                    None,
                    None,
                ),
                5 => {
                    let media = &extra["entries"][0]["media"];
                    let track = media["external_id"].as_str().filter(|id| {
                        !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric())
                    });
                    (
                        RecentKind::Music,
                        text(&media["title"])?,
                        text(&media["artists"][0]["name"]),
                        media["image_url"].as_str().and_then(recent_picture),
                        track.map(|id| format!("https://open.spotify.com/track/{id}")),
                    )
                }
                2 => (
                    RecentKind::Watched,
                    text(&extra["media_title"])?,
                    text(&extra["media_subtitle"]),
                    extra["media_assets_large_image"]
                        .as_str()
                        .and_then(|asset| watched_picture(asset, application)),
                    extra["url"]
                        .as_str()
                        .filter(|url| url.starts_with("https://") && url.len() <= 512)
                        .map(str::to_owned),
                ),
                8 => (
                    RecentKind::Activity,
                    text(&extra["activity_name"])?,
                    None,
                    None,
                    None,
                ),
                _ => return None,
            };
            Some(RecentActivity {
                kind,
                title,
                subtitle,
                image,
                at,
                link,
            })
        })
        .take(MAX_RECENT)
        .collect()
}

/// The wishlist `wishlist_settings` names, when the profile shares one.
pub(crate) fn wishlist_id(value: &Value) -> Option<u64> {
    value["wishlist_settings"]
        .as_object()?
        .keys()
        .find_map(|key| wire::id(&Value::from(key.as_str())).ok())
}

/// The route of wishlist `id`, as a profile asks for it.
pub(crate) fn wishlist_path(id: u64) -> String {
    format!("/wishlists/{id}?source=user_profile")
}

/// The items of `user`'s wishlist in `answer`, each with its shop picture:
/// the preview it gives on Discord's CDN, else its product's still.
pub(crate) fn wishlist(answer: &Value, user: UserId) -> Option<Vec<WishlistItem>> {
    if wire::id(&answer["user_id"]).ok()? != user.0 {
        return None;
    }
    let items = answer["wishlist_items"].as_array()?;
    Some(
        items
            .iter()
            .filter_map(|item| {
                let id = wire::id(&item["sku_id"]).ok()?;
                let preview = item["collectibles_item"]["thumbnailPreviewSrc"]
                    .as_str()
                    .filter(|url| {
                        url.starts_with("https://cdn.discordapp.com/")
                            && url.len() <= 512
                            && !url.chars().any(|c| c.is_control() || c.is_whitespace())
                    })
                    .map(|url| RemoteImage {
                        url: url.to_owned(),
                        kind: ImageKind::AttachmentPreview,
                    });
                Some(WishlistItem {
                    id,
                    name: text(&item["sku_name"], 128).unwrap_or_default(),
                    picture: preview.or_else(|| Some(crate::identity::collectible(id, "static"))),
                })
            })
            .take(MAX_WISHLIST)
            .collect(),
    )
}

/// The people the profile names, then the profile itself. The answer must
/// be about `user` and, inside `server`, about their membership there.
pub(crate) fn profile(
    value: &Value,
    request: RequestId,
    user: UserId,
    server: Option<ServerId>,
    mutuals: bool,
) -> Result<Vec<Event>, LoginError> {
    let account = &value["user"];
    if wire::id(&account["id"])? != user.0 {
        return Err(wire::invalid());
    }
    let global = &value["user_profile"];
    let mut profile = UserProfile {
        user,
        server,
        created: wire::created(user),
        about: text(&global["bio"], 1024).or_else(|| text(&account["bio"], 1024)),
        pronouns: text(&global["pronouns"], 64),
        banner: either(
            banner(global.get("banner"), &format!("banners/{user}")),
            banner(account.get("banner"), &format!("banners/{user}")),
        ),
        accent: accent(&global["accent_color"]).or_else(|| accent(&account["accent_color"])),
        badges: crate::identity::badges(&value["badges"], &value["guild_badges"]),
        connections: crate::identity::connections(&value["connected_accounts"]),
        games: games(&value["widgets"]),
        ..Default::default()
    };
    let member = &value["guild_member"];
    if let (Some(server), true) = (server, member.is_object()) {
        if !member["user"].is_null() && wire::id(&member["user"]["id"])? != user.0 {
            return Err(wire::invalid());
        }
        let scoped = &value["guild_member_profile"];
        if !scoped["guild_id"].is_null() && wire::id(&scoped["guild_id"])? != server.0 {
            return Err(wire::invalid());
        }
        let path = format!("guilds/{server}/users/{user}/banners");
        profile.nick = text(&member["nick"], 128);
        profile.server_about = text(&scoped["bio"], 1024).or_else(|| text(&member["bio"], 1024));
        profile.server_pronouns = text(&scoped["pronouns"], 64);
        profile.server_banner = either(
            banner(scoped.get("banner"), &path),
            banner(member.get("banner"), &path),
        );
        profile.joined = wire::timestamp(&member["joined_at"]).ok();
    }
    // The server's emoji the texts name, drawn in place.
    profile.pictures = crate::emoji::pictures(
        [&profile.about, &profile.server_about]
            .into_iter()
            .flatten()
            .map(String::as_str),
    );
    let mut events = Vec::new();
    if mutuals {
        profile.mutual_servers = mutual(&value["mutual_guilds"]).and_then(|list| {
            list.iter()
                .map(|guild| wire::id(&guild["id"]).map(ServerId).ok())
                .collect()
        });
        // A friend who cannot be shown by name makes the list unknown, not shorter.
        let friends: Option<Vec<_>> = mutual(&value["mutual_friends"])
            .and_then(|list| list.iter().map(|friend| wire::user(friend).ok()).collect());
        if let Some(friends) = friends {
            profile.mutual_friends = Some(friends.iter().map(|friend| friend.id).collect());
            events.extend(friends.into_iter().map(Event::UserChanged));
        }
    }
    events.push(Event::ProfileLoaded { request, profile });
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const HASH: &str = "0123456789abcdef0123456789abcdef";

    fn loaded(events: &[Event]) -> &UserProfile {
        match events.last() {
            Some(Event::ProfileLoaded { profile, .. }) => profile,
            _ => panic!("profile last"),
        }
    }

    #[test]
    fn recent_activity_keeps_games_music_and_what_was_watched_in_order() {
        // 2026-09 in the IDs' clock.
        let id = |ms: u64| ((ms - 1_420_070_400_000) << 22).to_string();
        let at = 1_790_000_000_000_u64;
        let answer = serde_json::json!({"entries": [
            {"id": id(at), "content_type": 5, "traits": [], "extra": {"entries": [{"media": {
                "provider": 1, "title": "444 - Remix", "artists": [{"name": "Yan Block"}],
                "image_url": "https://i.scdn.co/image/ab67616d0000b273", "external_id": "4uLU6hMCjMI75M1A2tKUQC"}}]}},
            {"id": id(at - 1000), "content_type": 1, "traits": [{"type": 2, "duration_seconds": 600}],
                "extra": {"application_id": "356875570916753438", "game_name": "Minecraft", "platform": 0}},
            {"id": id(at - 2000), "content_type": 1, "traits": [{"type": 3, "is_live": true}],
                "extra": {"game_name": "Live now"}},
            {"id": id(at - 3000), "content_type": 2, "traits": [], "extra": {
                "application_id": "1", "media_title": "Frieren", "media_subtitle": "Season 1, Episode 5",
                "media_assets_large_image": "mp:external/abc/https/img.example/x.png",
                "url": "https://www.crunchyroll.com/watch/x"}},
            {"id": id(at - 4000), "content_type": 6, "extra": {}},
            {"id": id(at - 5000), "content_type": 5, "extra": {"entries": [{"media": {
                "title": "Bad cover", "image_url": "javascript:alert(1)"}}]}},
            {"id": id(at - 6000), "content_type": 8, "expires_at": "2020-01-01T00:00:00+00:00",
                "extra": {"activity_name": "Expired"}}
        ]});
        let found = recent(&answer, Timestamp(at as i64 + 60_000));
        let titles: Vec<&str> = found.iter().map(|entry| entry.title.as_str()).collect();
        assert_eq!(titles, ["444 - Remix", "Minecraft", "Frieren", "Bad cover"]);
        let song = &found[0];
        assert_eq!(song.kind, RecentKind::Music);
        assert_eq!(song.subtitle.as_deref(), Some("Yan Block"));
        assert_eq!(song.at, Timestamp(at as i64));
        assert_eq!(
            song.link.as_deref(),
            Some("https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC")
        );
        assert!(song
            .image
            .as_ref()
            .is_some_and(|image| image.url.starts_with("https://i.scdn.co/")));
        assert_eq!(
            (found[1].kind, found[1].image.is_none()),
            (RecentKind::Game, true)
        );
        assert_eq!(
            found[2].image.as_ref().map(|image| image.url.as_str()),
            Some("https://media.discordapp.net/external/abc/https/img.example/x.png")
        );
        assert!(found[3].image.is_none(), "an unsafe picture is left out");
        assert_eq!(recent_path(UserId(7)), "/content-inventory/users/7/outbox");
    }

    #[test]
    fn the_route_asks_for_mutuals_only_about_someone_else() {
        assert_eq!(
            path(UserId(43), Some(ServerId(10)), true),
            "/users/43/profile?with_mutual_guilds=true&with_mutual_friends=true&with_mutual_friends_count=false&guild_id=10"
        );
        assert!(path(UserId(42), None, false).ends_with(
            "with_mutual_guilds=false&with_mutual_friends=false&with_mutual_friends_count=false"
        ));
    }

    #[test]
    fn a_full_answer_keeps_the_server_profile_apart_and_names_mutual_friends() {
        let value = json!({
            "user": {"id":"43","username":"peer","bio":"old bio","banner":null,"accent_color":255},
            "user_profile": {"bio":"  Hola  ","pronouns":"elle","banner":HASH,"accent_color":16711680},
            "guild_member": {"user":{"id":"43"},"nick":"Apodo","joined_at":"2024-03-12T10:00:00Z","roles":[]},
            "guild_member_profile": {"guild_id":"10","bio":"En este servidor","pronouns":null,"banner":null},
            "mutual_guilds": [{"id":"10","nick":null},{"id":"11"}],
            "mutual_friends": [{"id":"44","username":"friend"}]
        });
        let events = profile(&value, 7, UserId(43), Some(ServerId(10)), true).unwrap();
        assert!(matches!(&events[0], Event::UserChanged(user) if user.id == UserId(44)));
        let loaded = loaded(&events);
        assert_eq!(loaded.about.as_deref(), Some("Hola"));
        assert!(loaded.pictures.is_empty(), "no emoji named");
        assert_eq!(loaded.pronouns.as_deref(), Some("elle"));
        assert!(
            matches!(&loaded.banner, ProfileBanner::Image(image) if image.url.contains("banners/43/"))
        );
        assert_eq!(loaded.accent, Some(Rgb(0xFF0000)));
        assert_eq!(loaded.nick.as_deref(), Some("Apodo"));
        assert_eq!(loaded.server_about.as_deref(), Some("En este servidor"));
        assert_eq!(loaded.server_pronouns, None);
        assert_eq!(loaded.server_banner, ProfileBanner::Absent);
        assert!(
            loaded.joined.is_some() && loaded.created.is_none(),
            "43 >> 22 is 0"
        );
        assert_eq!(
            loaded.mutual_servers,
            Some(vec![ServerId(10), ServerId(11)])
        );
        assert_eq!(loaded.mutual_friends, Some(vec![UserId(44)]));
        assert!(loaded.answers(UserId(43), Some(ServerId(10))));
    }

    #[test]
    fn the_emoji_a_bio_names_come_as_pictures_the_servers_too() {
        let value = json!({
            "user": {"id":"43","username":"peer"},
            "user_profile": {"bio":"hola <a:Rana:1234567890123456789>\nchau 🌱"},
            "guild_member": {"user":{"id":"43"},"roles":[]},
            "guild_member_profile": {"guild_id":"10","bio":"<:rust:22> aquí"}
        });
        let events = profile(&value, 7, UserId(43), Some(ServerId(10)), true).unwrap();
        let loaded = loaded(&events);
        let tokens: Vec<&str> = loaded.pictures.iter().map(|p| p.token.as_str()).collect();
        assert_eq!(tokens, ["<a:Rana:1234567890123456789>", "<:rust:22>"]);
        assert!(loaded.answers(UserId(43), Some(ServerId(10))));
    }

    #[test]
    fn unknown_stays_apart_from_none_and_floods_are_unknown() {
        let value = json!({
            "user": {"id":"43","username":"peer"},
            "mutual_guilds": [],
            "mutual_friends": vec![json!({"id":"44","username":"f"}); MAX_MUTUALS + 1]
        });
        let events = profile(&value, 1, UserId(43), None, true).unwrap();
        let loaded = loaded(&events);
        assert_eq!(
            loaded.mutual_servers,
            Some(vec![]),
            "an empty list is none in common"
        );
        assert_eq!(loaded.mutual_friends, None, "too many is not reported");
        assert_eq!(loaded.banner, ProfileBanner::Unknown);
        assert_eq!(events.len(), 1);
        let missing = json!({"user": {"id":"43","username":"peer"},
            "mutual_friends": [{"id":"bad"}]});
        let loaded = profile(&missing, 1, UserId(43), None, true).unwrap();
        let loaded = self::loaded(&loaded);
        assert_eq!(loaded.mutual_servers, None, "missing is unknown");
        assert_eq!(loaded.mutual_friends, None, "a malformed friend is unknown");
        let own = profile(&value, 1, UserId(43), None, false).unwrap();
        assert_eq!(self::loaded(&own).mutual_servers, None, "not asked");
    }

    #[test]
    fn the_game_collection_lists_favorites_then_played_once_and_names_them() {
        let value = json!({
            "user": {"id":"43","username":"peer"},
            "widgets": [
                {"id":"1","data":{"type":"played_games","games":[
                    {"game_id":"30","comment":null,"tags":[]},
                    {"game_id":"10","comment":null,"tags":[]},
                    {"game_id":"bad"}]}},
                {"id":"2","data":{"type":"favorite_games","games":[
                    {"game_id":"10","comment":"Main","tags":["obsessed"]}]}},
                {"id":"3","data":{"type":"something_else","games":[{"game_id":"99"}]}}
            ],
            "wishlist_settings": {"77": {"visibility": 1}}
        });
        let events = profile(&value, 1, UserId(43), None, true).unwrap();
        let mut games = loaded(&events).games.clone().unwrap();
        let ids: Vec<u64> = games.iter().map(|game| game.id).collect();
        assert_eq!(
            ids,
            [10, 30],
            "favorites first, each once, known kinds only"
        );
        assert_eq!(games_path(&games), "/games?game_ids=10&game_ids=30");
        name_games(
            &mut games,
            &json!([{"id":"30","name":"Celeste","icon_hash":HASH},
                {"id":"10","name":"Factorio","icon_hash":"../x"}]),
        );
        assert_eq!(games[1].name, "Celeste");
        assert_eq!(
            games[1].icon.as_ref().unwrap().url,
            format!("https://cdn.discordapp.com/app-icons/30/{HASH}.png?size=48&keep_aspect_ratio=false")
        );
        assert!(games[0].icon.is_none(), "a malformed hash is no icon");
        assert_eq!(wishlist_id(&value), Some(77));
        assert_eq!(wishlist_path(77), "/wishlists/77?source=user_profile");
        let none = json!({"user": {"id":"43","username":"peer"}});
        let events = profile(&none, 1, UserId(43), None, true).unwrap();
        assert_eq!(loaded(&events).games, None, "no widgets is unknown");
        assert_eq!(wishlist_id(&none), None);
    }

    #[test]
    fn a_wishlist_keeps_its_owners_items_with_their_pictures() {
        let answer = json!({
            "id": "77", "user_id": "43",
            "wishlist_items": [
                {"sku_id":"5","sku_name":"Ondas","collectibles_item":{"type":1,
                    "thumbnailPreviewSrc":"https://cdn.discordapp.com/media/v1/collectibles-shop/abc"}},
                {"sku_id":"6","sku_name":"Nevermore","collectibles_item":{"type":2,
                    "thumbnailPreviewSrc":"https://elsewhere.test/x.png"}},
                {"sku_id":"nope"}
            ]
        });
        let items = wishlist(&answer, UserId(43)).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Ondas");
        assert_eq!(
            items[0].picture.as_ref().unwrap().url,
            "https://cdn.discordapp.com/media/v1/collectibles-shop/abc"
        );
        assert_eq!(
            items[1].picture.as_ref().unwrap().url,
            "https://cdn.discordapp.com/media/v1/collectibles-shop/6/static",
            "only Discord's CDN, else the product's still"
        );
        assert_eq!(wishlist(&answer, UserId(44)), None, "someone else's");
    }

    #[test]
    fn an_answer_about_someone_or_somewhere_else_is_refused_and_text_is_bounded() {
        let other = json!({"user": {"id":"44","username":"peer"}});
        assert!(profile(&other, 1, UserId(43), None, true).is_err());
        let elsewhere = json!({"user": {"id":"43","username":"peer"},
            "guild_member": {"user":{"id":"43"}}, "guild_member_profile": {"guild_id":"11"}});
        assert!(profile(&elsewhere, 1, UserId(43), Some(ServerId(10)), true).is_err());
        let someone = json!({"user": {"id":"43","username":"peer"},
            "guild_member": {"user":{"id":"44"}}});
        assert!(profile(&someone, 1, UserId(43), Some(ServerId(10)), true).is_err());
        let long = json!({"user": {"id":"43","username":"peer","bio":"x".repeat(5000)},
            "user_profile": {"pronouns":"p".repeat(500)}});
        let events = profile(&long, 1, UserId(43), None, true).unwrap();
        let loaded = loaded(&events);
        assert_eq!(loaded.about.as_ref().map(|t| t.chars().count()), Some(1024));
        assert_eq!(
            loaded.pronouns.as_ref().map(|t| t.chars().count()),
            Some(64)
        );
    }
}
