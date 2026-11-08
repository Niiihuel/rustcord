//! Profiles and the account's status, from plain props: the card of a person
//! (opened from the member list or from a message's author), the account's
//! own popout over its panel with the status menu and how long a status
//! lasts, someone's full profile (a card beside its tabs), the dialog that
//! sets the custom status, the menus of the microphone's and the
//! headphones' arrows, and the full-window profile editor with its live
//! preview and a read-only account view.
//!
//! Measures follow Discord's, as `docs/discord-reference.md` notes them: a
//! popout (300 px on #0a0a0c) with an 80 px mark over its banner, its
//! actions in cards of 32 px rows, a full profile of a 400 px card beside its
//! tabs in a 960 px panel, a 310 px status menu
//! with 200 px menus of durations, 235 px device
//! menus and a 510.5 px status dialog. The editor adapts its form and preview
//! to one or two columns, with fixed close and save controls outside the scroll.

use limn::{
    Button, ButtonVariant, Color, Component, ComponentPalette, CornerRadii, DismissButton,
    DropdownMenu, DropdownMenuEntry, DropdownMenuItem, Element, FontFamily, Icon, IconButton,
    Modal, Point, Popover, PopoverPlacement, ScrollArea, Size, Skeleton, Slider, Tab, Tabs,
    TabsVariant, TextArea, TextField, Tooltip,
};
use rustcord_i18n::{es, t, tr};
use rustcord_model::{Rgb, Status, Timestamp, UserId};

use crate::server_settings::{self as parts, Swatches, UnsavedBar};
use crate::voice::{menu_entries, menu_width, MenuEntry};
use crate::{centered, emoji, gap, icon, line, strong_line, text, time, Look};

/// The actions these pieces report. Those that name a message, a status or
/// a device are built with the helpers or the prefixes here.
mod side;
pub(crate) mod theme;
pub use side::{side_panel, SIDE_WIDTH};
pub use theme::ProfileTheme;

pub mod ids {
    /// The account's popout, opened from its panel.
    pub const DISMISS: &str = "pp:dismiss";
    /// Opens the profile editor, from here or from the settings.
    pub const EDIT: &str = "profile:edit";
    /// The bubble of the custom status: opens its dialog.
    pub const CUSTOM: &str = "pp:custom";
    pub const COPY_ID: &str = "pp:copy-id";
    pub const COPY_NAME: &str = "pp:copy-name";
    /// Followed by a status's key, and by `:` and a duration's key when it
    /// lasts a while: `pp:status=idle:1h`.
    pub const STATUS: &str = "pp:status=";
    /// Rows that only open a submenu start with this: choosing them does
    /// nothing.
    pub const SUBMENU: &str = "pp:sub:";
    /// The popout's status row, which opens the menu of statuses.
    pub const STATUS_MENU: &str = "pp:status-menu";
    pub const STATUS_MENU_DISMISS: &str = "pp:status-menu:dismiss";
    pub const SWITCH_ACCOUNT: &str = "pp:switch-account";
    /// The accounts under «Cambiar de cuenta»: `pp:account={id}` switches to
    /// one, `pp:manage-accounts` opens «Gestionar cuentas».
    pub const ACCOUNT: &str = "pp:account=";
    pub const MANAGE_ACCOUNTS: &str = "pp:manage-accounts";
    pub const ACCOUNT_MENU_DISMISS: &str = "pp:account-menu:dismiss";

    /// The arrows of the microphone and the headphones: the quick voice
    /// panel of each, [`crate::quick_voice`].
    pub const INPUT: &str = "pp:input";
    pub const OUTPUT: &str = "pp:output";

    /// The dialog of the custom status.
    pub const CUSTOM_CLOSE: &str = "pp:custom:close";
    pub const CUSTOM_TEXT: &str = "pp:custom:text";
    pub const CUSTOM_EMOJI: &str = "pp:custom:emoji";
    /// Followed by the emoji chosen.
    pub const CUSTOM_EMOJI_PICK: &str = "pp:custom:emoji=";
    pub const CUSTOM_EMOJI_DISMISS: &str = "pp:custom:emoji:dismiss";
    pub const CUSTOM_EXPIRY: &str = "pp:custom:expiry";
    /// Followed by a [`super::ClearAfter`] key.
    pub const CUSTOM_EXPIRY_PICK: &str = "pp:custom:expiry=";
    pub const CUSTOM_EXPIRY_DISMISS: &str = "pp:custom:expiry:dismiss";
    pub const CUSTOM_SAVE: &str = "pp:custom:save";
    pub const CUSTOM_CANCEL: &str = "pp:custom:cancel";

    /// The card of a message's author.
    pub const AUTHOR_DISMISS: &str = "pp:author:dismiss";
    /// The card of someone of the member list.
    pub const MEMBER_DISMISS: &str = "profile:dismiss";
    /// The round buttons over a card's banner, each followed by the person's
    /// id: the friendship's (which opens its menu, or sends a request when
    /// there is none) and «…». Their menus close on `…:dismiss`.
    pub const CARD_RELATION: &str = "pp:relation:";
    pub const CARD_MORE: &str = "pp:more:";
    pub const CARD_RELATION_DISMISS: &str = "pp:relation:dismiss";
    pub const CARD_MORE_DISMISS: &str = "pp:more:dismiss";
    /// «Compartir perfil» and «Denunciar perfil de usuario» of a card's «…»,
    /// each followed by the person's id.
    pub const SHARE: &str = "pp:share:";
    pub const REPORT: &str = "pp:report:";
    /// A card's mutual friends or servers: followed by the person's id and
    /// `:friends` or `:servers`, they open the full profile on that tab.
    pub const MUTUAL: &str = "pp:mutual:";
    /// The «+» beside a card's roles opens the checklist of roles; its
    /// `…:dismiss` closes it. Followed by the person's id.
    pub const CARD_ROLES: &str = "pp:roles:";
    pub const CARD_ROLES_DISMISS: &str = "pp:roles:dismiss";
    /// The field under a card that writes to its person: followed by their id.
    pub const MESSAGE: &str = "pp:message:";
    /// Opens someone's full profile: the prefix and their id.
    pub const FULL: &str = "pp:full:";
    pub const FULL_CLOSE: &str = "pp:full:close";
    /// Followed by a [`super::FullTab`] key: `pp:full:tab=servers`.
    pub const FULL_TAB: &str = "pp:full:tab=";
    /// «Ver biografía completa» of a card: the full profile, from the bio.
    pub const FULL_BIO: &str = "pp:full-bio:";
    /// The note's icon of a card: the full profile, where the note is.
    pub const FULL_NOTE: &str = "pp:full-note:";
    /// A card's «Colección de juegos» and the panel's «Ver todo» of the
    /// wishlist: the full profile, from them.
    pub const FULL_GAMES: &str = "pp:full-games:";
    pub const FULL_WISHLIST: &str = "pp:full-wishlist:";
    /// Shows or hides the profile beside a direct conversation.
    pub const SIDE_TOGGLE: &str = "pp:side:toggle";
    /// The body of the profile beside a direct conversation, scrolling
    /// above its button: followed by the person's handle.
    pub const SIDE_BODY: &str = "pp:side:body:";
    /// The body of a card, scrolling between its head and its buttons;
    /// see [`card_body`].
    pub const CARD_BODY: &str = "pp:card:body:";
    /// The effect over a card, which gives way under the pointer.
    pub const CARD_EFFECT: &str = "pp:card:effect";

    /// The profile settings: the example button of the preview.
    pub const EXAMPLE: &str = "profile:example";
    /// The tabs of the profile: the account's, and the servers'.
    pub const TAB_MAIN: &str = "profile:tab:main";
    pub const TAB_SERVERS: &str = "profile:tab:servers";
    /// The decorations Rustcord cannot change yet: their buttons do nothing.
    pub const DECORATION: &str = "profile:decoration";
    pub const NAMEPLATE: &str = "profile:nameplate";
    pub const EFFECT: &str = "profile:effect";
    /// «Cambiar marco»: `profile:frame` is the framing of a picked picture.
    pub const PROFILE_FRAME: &str = "profile:frame-pick";
    pub const NAME: &str = "profile:name";
    pub const PRONOUNS: &str = "profile:pronouns";
    pub const ABOUT: &str = "profile:about";
    /// Prefixes the color picker's ids: see [`crate::server_settings::Swatches`].
    pub const COLOR: &str = "profile:color";
    pub const COLOR_HEX: &str = "profile:color:hex";
    /// The selector of profiles at the top of the editor's column.
    pub const SERVERS: &str = "profile:servers";
    pub const SERVERS_DISMISS: &str = "profile:servers:dismiss";
    /// Prefixes the bar's ids: `profile:reset` and `profile:save`.
    pub const BAR: &str = "profile";
    pub const RESET: &str = "profile:reset";
    pub const SAVE: &str = "profile:save";
    /// The pictures: choose one through the system's selector, or take it away.
    pub const AVATAR_PICK: &str = "profile:avatar:pick";
    pub const AVATAR_REMOVE: &str = "profile:avatar:remove";
    pub const BANNER_PICK: &str = "profile:banner:pick";
    pub const BANNER_REMOVE: &str = "profile:banner:remove";
    /// The name the account goes by in the server whose profile is edited.
    pub const NICK: &str = "profile:nick";
    /// Edits the profile of a server: the prefix and the server's id.
    pub const SCOPE: &str = "profile:scope=";
    /// Where the pictures start, to scroll them into view.
    pub const PICTURES: &str = "profile:pictures";
    /// The picture being framed, dragged over to move the frame, and the
    /// slider that brings it closer.
    pub const FRAME: &str = "profile:frame";
    pub const FRAME_ZOOM: &str = "profile:frame:zoom";

    /// The name of the author of `message`, which opens their card.
    pub fn author_name(message: u64) -> String {
        format!("pp:name:{message}")
    }

    /// The mark of the author of `message`, which opens their card.
    pub fn author_mark(message: u64) -> String {
        format!("pp:avatar:{message}")
    }

    /// The body of the card of `handle` (unique per account): one for each
    /// person, since the window keeps a scroll per ID.
    pub fn card_body(handle: &str) -> String {
        format!("{CARD_BODY}{handle}")
    }
}

/// A popout's width: the official client's 300 px.
pub const WIDTH: f32 = 300.0;
/// The room between a member list's row and its card; 8 px from an
/// author's name, as the official popout stands.
pub const LIST_GAP: f32 = 16.0;
/// The bio a card shows: three lines, about 60 px, as the official
/// client's popout; longer ones lead to the full profile.
const BIO_LINES: usize = 3;
const BIO_CHARS: usize = 120;
const BIO_HEIGHT: f32 = 60.0;
const BANNER: f32 = 105.0;
const AVATAR: f32 = 80.0;
/// The ring of the card's color around the mark.
const RING: f32 = 6.0;
/// Inside a card, left and right.
const INSET: f32 = 16.0;
/// A person's card: its corners, and its head, 140 px as the official
/// popout's. The mark (80 px at (16, 61)) overlaps the banner by 44 px and
/// hangs 36 px below it; its 6 px ring is cut into the banner, so the ring's
/// box starts at (10, 55).
const CARD_RADIUS: f32 = 8.0;
/// A themed popout's frame, the gradient around its veil, and its footer's
/// height, where the gradient ends; the full profile's frame.
const POPOUT_FRAME: f32 = 2.0;
const POPOUT_FOOTER: f32 = 62.0;
const FULL_FRAME: f32 = 4.0;
const HEAD: f32 = 140.0;
const MARK_TOP: f32 = 61.0;
/// The status dot of a card's mark: 16 px, in a 28 px disc of the ground.
const DOT: f32 = 16.0;
/// The skeleton of a card being read: its height, and its four text lines
/// (from the card's 268 px body).
const SKELETON_HEIGHT: f32 = 350.0;
const SKELETON_LINES: [f32; 4] = [241.0, 201.0, 228.0, 161.0];
/// The status menu beside the popout, and the menus of durations beside it.
pub const STATUS_MENU_WIDTH: f32 = 310.0;
pub const DURATION_MENU_WIDTH: f32 = 200.0;
/// The dialog of the custom status, and its menu of when it goes.
pub const DIALOG_WIDTH: f32 = 510.5;
const EXPIRY_MENU_WIDTH: f32 = 274.0;
/// The settings' profile: the preview's gap from the form, and the narrowest
/// content column that puts the form and the preview side by side (a 360 px
/// form beside the 300 px preview); narrower ones stack them.
const PREVIEW_GAP: f32 = 36.0;
const SIDE_BY_SIDE: f32 = 360.0 + PREVIEW_GAP + WIDTH;

/// The longest display name, pronouns, «Sobre mí» and custom status.
pub const NAME_MAX: usize = 32;
pub const PRONOUNS_MAX: usize = 40;
pub const ABOUT_MAX: usize = 190;
pub const CUSTOM_MAX: usize = 128;

/// The statuses the account can choose: the status, its key in the ids, and
/// what it means when that is not plain.
pub const STATUSES: [(Status, &str, Option<&str>); 4] = [
    (Status::Online, "online", None),
    (Status::Idle, "idle", None),
    (
        Status::DoNotDisturb,
        "dnd",
        Some(es("Los demás ven que no quieres interrupciones.")),
    ),
    (
        Status::Invisible,
        "invisible",
        Some(es("Apareces sin conexión.")),
    ),
];

/// How long a chosen status lasts: its key in the ids, what the menu says,
/// and its minutes. Choosing the status itself keeps it until changed.
pub const DURATIONS: [(&str, &str, i64); 5] = [
    ("15m", es("Durante 15 minutos"), 15),
    ("1h", es("Durante 1 hora"), 60),
    ("8h", es("Durante 8 horas"), 8 * 60),
    ("24h", es("Durante 24 horas"), 24 * 60),
    ("3d", es("Durante 3 días"), 3 * 24 * 60),
];

/// The key of a status in the ids.
pub fn status_key(status: Status) -> &'static str {
    STATUSES
        .iter()
        .find(|(candidate, _, _)| *candidate == status)
        .map_or("online", |(_, key, _)| key)
}

/// The status a key names.
pub fn status_of(key: &str) -> Option<Status> {
    STATUSES
        .iter()
        .find(|(_, candidate, _)| *candidate == key)
        .map(|(status, _, _)| *status)
}

/// When a custom status goes by itself.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ClearAfter {
    #[default]
    Day,
    FourHours,
    Hour,
    HalfHour,
    Never,
}

impl ClearAfter {
    pub const ALL: [Self; 5] = [
        Self::Day,
        Self::FourHours,
        Self::Hour,
        Self::HalfHour,
        Self::Never,
    ];

    pub fn minutes(self) -> Option<i64> {
        match self {
            Self::Day => Some(24 * 60),
            Self::FourHours => Some(4 * 60),
            Self::Hour => Some(60),
            Self::HalfHour => Some(30),
            Self::Never => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Day => "24h",
            Self::FourHours => "4h",
            Self::Hour => "1h",
            Self::HalfHour => "30m",
            Self::Never => "never",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.key() == key)
    }

    /// What its menu says, set at `now`: "24 horas (mañana a las 07:22)".
    pub fn label(self, now: Timestamp) -> String {
        let length = match self {
            Self::Day => t("24 horas"),
            Self::FourHours => t("4 horas"),
            Self::Hour => t("1 hora"),
            Self::HalfHour => t("30 minutos"),
            Self::Never => return t("No eliminar").into(),
        };
        let until = now.plus_minutes(self.minutes().unwrap_or_default());
        let when = if time::day(until) == time::day(now) {
            time::clock(until)
        } else {
            until_label(until, now)
        };
        format!("{length} ({when})")
    }
}

/// When something that ends at `until` ends, seen at `now`: "hoy a las
/// 07:52", "mañana a las 07:22" or "el 17/10 a las 07:22".
pub fn until_label(until: Timestamp, now: Timestamp) -> String {
    let (day, today) = (time::day(until), time::day(now));
    let clock = time::clock(until);
    if day == today {
        tr!("hoy a las {clock}", clock)
    } else if Some(day) == today.succ_opt() {
        tr!("mañana a las {clock}", clock)
    } else {
        tr!(
            "el {date} a las {clock}",
            date = day.format(t("%d/%m")),
            clock
        )
    }
}

/// The mark of a status, `size` across, on `ground`: a dot while online, a
/// crescent while idle, a dot with a bar while not to be disturbed, a ring
/// while invisible or offline.
pub fn status_mark(status: Status, look: &Look, size: f32, ground: Color) -> Element {
    let color = look.status(status);
    let disc = |diameter: f32, fill: Color| {
        Element::panel(Element::spacer())
            .size(Some(diameter), Some(diameter))
            .radius(diameter / 2.0)
            .background(fill)
    };
    match status {
        Status::Online => disc(size, color),
        Status::Idle => Element::stack(vec![disc(size, color), disc(size * 0.62, ground)])
            .size(Some(size), Some(size)),
        Status::DoNotDisturb => Element::stack(vec![
            disc(size, color),
            crate::centered(
                Element::panel(Element::spacer())
                    .size(Some(size * 0.6), Some((size * 0.22).max(2.0)))
                    .radius(size * 0.11)
                    .background(ground),
            ),
        ])
        .size(Some(size), Some(size)),
        Status::Invisible | Status::Offline => Element::panel(Element::spacer())
            .size(Some(size), Some(size))
            .radius(size / 2.0)
            .border(color, (size * 0.28).max(2.0)),
    }
}

/// `child`, `size` across, with the mark of `status` over its bottom right
/// corner, ringed in `ground` so it reads on any color.
fn marked(child: Element, size: f32, status: Status, look: &Look, ground: Color) -> Element {
    let dot = (size * 0.25).max(10.0);
    let ring = (dot * 0.25).max(2.0);
    let mark = Element::panel(status_mark(status, look, dot, ground))
        .padding(ring)
        .radius(dot / 2.0 + ring)
        .background(ground);
    Element::stack(vec![
        child,
        Element::column(vec![
            Element::spacer().flex(1.0),
            Element::row(vec![Element::spacer().flex(1.0), mark]),
        ]),
    ])
    .size(Some(size), Some(size))
}

/// What a card shows of a person.
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: String,
    pub image: Option<limn::RasterImage>,
    /// Its picture moves: drawn on the surface the host plays `image` on.
    pub avatar_playing: bool,
    /// A decoded wide image; a missing or loading banner retains the accent.
    pub banner: Option<limn::RasterImage>,
    /// The banner moves, as `avatar_playing`.
    pub banner_playing: bool,
    /// The profile's own colors: its card framed and tinted in them.
    pub theme: Option<ProfileTheme>,
    /// Without a picture, the banner's color: the profile's accent when it
    /// has one, else the mark's dominant color. `None` paints `raised`.
    pub banner_tint: Option<Color>,
    /// The unique handle, without a leading @.
    pub handle: String,
    /// Their color: the banner, and behind their initial.
    pub accent: Color,
    pub status: Status,
    /// Whether the provider publishes presence; otherwise no status is displayed.
    pub presence: bool,
    /// The line they set beside their status.
    pub custom: Option<String>,
    /// A server's emoji before that line, as a picture.
    pub custom_emoji: Option<crate::identity::TextPicture>,
    pub pronouns: Option<String>,
    pub about: Option<String>,
    /// What they are doing: "Jugando a Factorio".
    pub activity: Option<String>,
    /// Their roles where they are seen, highest first, in their colors.
    pub roles: Vec<(String, Color)>,
    pub bot: bool,
    /// What keeps them from taking part where they are seen: "Aislado
    /// hasta hoy a las 18:30".
    pub restriction: Option<String>,
    /// Since when, the account first and then the server it is seen in.
    /// Only what is known.
    pub member_since: Vec<Since>,
    /// What they share with the account, once known: "2 servidores en común".
    pub mutual: Option<String>,
    /// The action that opens a direct conversation with them, if any: the
    /// full profile's «Enviar mensaje».
    pub message_action: Option<String>,
    /// The action that opens their full profile, if any.
    pub full_action: Option<String>,
    /// The account's private note on them, where it keeps notes.
    pub note: Option<NoteField>,
    /// What they do, where from, their badges and marks.
    pub identity: crate::identity::CardIdentity,
    /// Their profile is being read and none of it is kept yet: the card is
    /// its skeleton, [`card_skeleton`].
    pub loading: bool,
    /// The friendship's round button on the banner; `None` hides it: the
    /// account's own card, bots, blocked people, a provider without requests.
    pub relation: Option<Relation>,
    /// The «…» button's action and its menu's entries; `None` hides it, as
    /// Discord's when nothing in it applies (the account's own card, at times).
    pub more: Option<(String, Vec<MenuEntry>)>,
    /// The card's menu that is open, if any: its entries are the ones above.
    pub open_menu: Option<CardMenu>,
    /// The account blocks or ignores them: a notice under the name.
    pub notice: Option<Notice>,
    /// Who they share with the account, as the card's row shows it:
    /// nothing while unknown or loading, or when there is none.
    pub mutual_friends: Mutual,
    pub mutual_servers: Mutual,
    /// Per role of `roles`: the action that takes it away, when the account
    /// may. Only what the checklist lists can be taken away.
    pub role_removals: Vec<Option<String>>,
    /// The action of the roles' «+» and the checklist it opens, when the
    /// account may change them.
    pub role_menu: Option<(String, Vec<MenuEntry>)>,
    /// The field under the card that writes to them, when the account may.
    pub message: Option<MessageField>,
    /// The account's own card offers «Editar perfil».
    pub edit_profile: bool,
    /// The account's own card: its bubble sets the custom status, and says
    /// «Añadir estado» while there is none.
    pub set_custom: bool,
    /// The preview of the profile's editor offers an example button instead.
    pub example_button: bool,
    /// The games of their collection, all of them: a card shows four.
    pub games: Vec<GameIcon>,
    /// What they wish for: the profile beside a conversation shows three.
    pub wishlist: Vec<WishTile>,
}

/// A game of someone's collection, as its card shows it: its icon, and its
/// name for a reader.
#[derive(Clone, Debug, PartialEq)]
pub struct GameIcon {
    pub name: String,
    pub icon: Option<limn::RasterImage>,
}

/// Something someone wishes for, as their profile shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct WishTile {
    pub name: String,
    pub picture: Option<limn::RasterImage>,
}

/// The friendship with someone, as the round button on a card's banner says
/// it: none (the button sends a request), a friend, a request sent,
/// or one received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Friendship {
    None,
    Friend,
    Outgoing,
    Incoming,
}

/// The friendship button of a card: its state, the action it sends (the
/// request when there is none, else the toggle of its menu), and the entries
/// of that menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub state: Friendship,
    pub action: String,
    pub menu: Vec<MenuEntry>,
}

/// The menus a card opens from its buttons: the friendship's, «…», and the
/// roles checklist beside the roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardMenu {
    Relation,
    More,
    Roles,
}

/// The field under a card that writes to its person: what is
/// written so far; Enter sends it.
#[derive(Clone, Debug, PartialEq)]
pub struct MessageField {
    pub id: String,
    pub text: String,
}

/// Why a card's person is set apart: the account blocks them, or ignores them.
#[derive(Clone, Debug, PartialEq)]
pub enum Notice {
    Blocked,
    Ignored {
        /// The action of «Dejar de ignorar».
        unignore: String,
    },
}

/// The account's private note on someone, as its field shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteField {
    /// The field's own: Enter keeps what is written.
    pub id: String,
    pub text: String,
    /// Still being read: the field waits.
    pub loading: bool,
    /// Why it cannot be written now, when it cannot.
    pub denied: Option<String>,
}

/// «Nota»: a field only the account sees, saved with Enter; on a themed
/// card, on its note's veil.
fn note_section(note: &NoteField, profile: &Profile, look: &Look) -> Element {
    let placeholder = if note.loading {
        t("Cargando la nota…")
    } else {
        t("Haz clic para agregar una nota")
    };
    // Discord's note reads as its placeholder until it is written: no fill,
    // no outline but the focus's.
    let palette = ComponentPalette {
        raised: profile.theme.map_or(Color::TRANSPARENT, |theme| theme.note),
        border: Color::TRANSPARENT,
        muted: look.dim,
        ..look.menu_palette()
    };
    let field = TextField::new(note.id.clone(), note.text.clone(), placeholder, palette)
        .outline(Color::TRANSPARENT, look.accent)
        .accessibility_label(t("Nota"));
    let field = if note.loading || note.denied.is_some() {
        field.disabled()
    } else {
        field
    };
    let hint = note
        .denied
        .clone()
        .unwrap_or_else(|| t("Enter la guarda.").into());
    section(
        t("Nota (solo visible para ti)"),
        Element::column(vec![
            field.render().font_size(14.0),
            text(hint, 12.0, look.faint),
        ])
        .gap(4.0),
        look,
    )
}

/// A small picture of a server or a person: their image, else their initial
/// on their color.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub name: String,
    pub accent: Color,
    pub image: Option<limn::RasterImage>,
}

/// One date of «Miembro desde»: since when, and where. `face` is the
/// server's; the account has none.
#[derive(Clone, Debug, PartialEq)]
pub struct Since {
    /// "Cuenta", or the server's name.
    pub scope: String,
    pub face: Option<Face>,
    /// "3 may 2019".
    pub date: String,
}

/// A loaded list of a full profile: not reported, on its way, or known.
#[derive(Clone, Debug, PartialEq)]
pub enum Mutual {
    Unknown,
    Loading,
    Known(Vec<Face>),
}

/// The tabs of the full profile's column beside its card, in Discord's
/// order: what they do, then the friends and the servers in common.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FullTab {
    #[default]
    Activity,
    Friends,
    Servers,
}

impl FullTab {
    pub const ALL: [Self; 3] = [Self::Activity, Self::Friends, Self::Servers];

    pub fn key(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Servers => "servers",
            Self::Friends => "friends",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.key() == key)
    }
}

/// What the full profile shows beyond the card.
#[derive(Clone, Debug, PartialEq)]
pub struct FullProfile {
    pub profile: Profile,
    /// The server the card is seen in: its roles and date are of it.
    pub server: Option<String>,
    /// Since when the account and they are friends, when the provider says:
    /// "3 may 2019".
    pub friends_since: Option<String>,
    pub mutual_servers: Mutual,
    pub mutual_friends: Mutual,
    pub tab: FullTab,
    /// The accounts elsewhere they show.
    pub connections: Vec<crate::identity::ConnectionRow>,
    /// Their profile is being read and none of it is kept yet: the activity
    /// tab shows its lines instead of claiming there is none.
    pub loading: bool,
    /// What they did lately («Actividad reciente»), in the service's order.
    pub recent: Vec<RecentCard>,
}

/// What a recent activity was, for the glyph of its card's last line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecentGlyph {
    #[default]
    Game,
    Music,
    Watched,
    Activity,
}

/// One card of «Actividad reciente».
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecentCard {
    pub glyph: RecentGlyph,
    pub title: String,
    pub subtitle: Option<String>,
    /// Its picture, once loaded; the glyph on a plain square until then.
    pub image: Option<limn::RasterImage>,
    /// How long ago, as a person reads it: «hace 2 semanas».
    pub when: String,
}

/// A card of «Actividad reciente», as Discord's: on the raised ground, 12 in,
/// a 60 px picture then its title (14 semibold), its subtitle (12) and a line
/// with its kind's glyph and how long ago (12, subtle).
fn recent_card(card: &RecentCard, look: &Look) -> Element {
    let glyph = match card.glyph {
        RecentGlyph::Game | RecentGlyph::Activity => Icon::Gamepad,
        RecentGlyph::Music => Icon::MusicNote,
        RecentGlyph::Watched => Icon::Video,
    };
    let picture = match &card.image {
        Some(image) => Element::raster_image(image.clone().cover())
            .size(Some(60.0), Some(60.0))
            .radius(4.0),
        None => Element::panel(crate::centered(
            icon(glyph, look.muted).size(Some(24.0), Some(24.0)),
        ))
        .size(Some(60.0), Some(60.0))
        .radius(4.0)
        .background(raised(look, 24)),
    };
    let mut details = vec![strong_line(&card.title, 14.0, look.text).shrink_to_fit()];
    if let Some(subtitle) = &card.subtitle {
        details.push(line(subtitle, 12.0, look.muted).shrink_to_fit());
    }
    details.push(
        Element::row(vec![
            icon(glyph, look.faint)
                .size(Some(12.0), Some(12.0))
                .centered_y(),
            line(&card.when, 12.0, look.faint).centered_y(),
        ])
        .gap(4.0),
    );
    Element::row(vec![
        picture,
        Element::column(details).gap(4.0).flex(1.0).centered_y(),
    ])
    .gap(12.0)
    .padding(12.0)
    .radius(8.0)
    .background(raised(look, 16))
}

/// «Actividad reciente»: its heading (12, subtle) and its cards, 8 apart.
fn recent_section(cards: &[RecentCard], look: &Look) -> Option<Element> {
    if cards.is_empty() {
        return None;
    }
    let mut parts = vec![strong_line(t("Actividad reciente"), 12.0, look.muted)];
    parts.extend(cards.iter().map(|card| recent_card(card, look)));
    Some(Element::column(parts).gap(8.0))
}

/// A popout's ground: Discord's floating black (#0a0a0c, the default look's
/// `card`) on a dark look; a light look keeps its white menus.
fn ground(look: &Look) -> Color {
    if look.canvas.r > 128 {
        look.menu
    } else {
        look.card
    }
}

/// The popout's ground with the text laid over it at `alpha` of 255, opaque so
/// it reads the same over a banner: Discord's cards and bubbles (#17181b,
/// 16) and its badges' box (#1a1a1d, 20) on #0a0a0c.
fn raised(look: &Look, alpha: u8) -> Color {
    let (under, ink) = (ground(look), look.text);
    let mix = |under: u8, ink: u8| {
        (f32::from(under) + (f32::from(ink) - f32::from(under)) * f32::from(alpha) / 255.0).round()
            as u8
    };
    Color::rgb(
        mix(under.r, ink.r),
        mix(under.g, ink.g),
        mix(under.b, ink.b),
    )
}

/// The fill of a box on `profile`'s card: [`raised`] at `alpha`, or on a
/// themed card its boxes' veil.
fn boxed(profile: &Profile, look: &Look, alpha: u8) -> Color {
    profile
        .theme
        .map_or_else(|| raised(look, alpha), |theme| theme.boxed)
}

/// Discord's hairlines inside a popout: its grey at `alpha` of 255 (41 is
/// its 16 %, 51 its 20 %, 61 its 24 %).
fn hairline(look: &Look, alpha: u8) -> Color {
    Look::alpha(look.muted, alpha)
}

/// `content` floating against `anchor` on a popout's surface, `WIDTH` wide,
/// while there is content: Discord's ground, 8 px corners, a 1 px ring and its
/// shadow. Escape and a click outside report `dismiss`.
pub fn popout(
    anchor: Element,
    content: Option<Element>,
    dismiss: &str,
    placement: PopoverPlacement,
    look: &Look,
) -> Element {
    let Some(content) = content else {
        return anchor;
    };
    let surface = Element::panel(content)
        .size(Some(WIDTH), None)
        .background(ground(look))
        .border(look.border, 1.0)
        .radius(CARD_RADIUS)
        .shadow(look.shadow, Point::new(0.0, 12.0), 24.0, 0.0)
        .clip_children();
    Element::popover(anchor, surface, dismiss, placement).popover_gap(8.0)
}

/// [`popout`] for a person's card: the account's and someone else's are the
/// same surface.
pub fn card_popout(
    anchor: Element,
    content: Option<Element>,
    dismiss: &str,
    placement: PopoverPlacement,
    look: &Look,
) -> Element {
    popout(anchor, content, dismiss, placement, look)
}

/// A thought bubble: the custom status, or an invitation to set one. It
/// opens the dialog when given an action.
fn bubble(
    content: &str,
    emoji: Option<&crate::identity::TextPicture>,
    invitation: bool,
    action: Option<&str>,
    look: &Look,
) -> Element {
    let mut parts = vec![gap(4.0, 0.0)];
    if invitation {
        parts.push(
            icon(Icon::Add, look.dim)
                .size(Some(16.0), Some(16.0))
                .centered_y(),
        );
    }
    // A server's emoji before the line: 16 px, 32 alone, as the official
    // client's popout draws it.
    if let Some(emoji) = emoji {
        let side = if content.is_empty() { 32.0 } else { 16.0 };
        parts.push(crate::identity::picture(emoji, side, look).centered_y());
    }
    if !content.is_empty() {
        // 14/18 as Discord's: formatted text takes a line height.
        parts.push(
            Element::formatted_text(vec![limn::TextSpan::new(
                content,
                if invitation { look.dim } else { look.text },
                14.0,
                FontFamily::SansItalic,
            )])
            .font_size(14.0)
            .line_height(18.0)
            .flex(1.0),
        );
    }
    parts.push(gap(4.0, 0.0));
    // Discord's: 8 above and below, 12 at the sides, 16 px corners.
    let body = Element::row(parts).gap(4.0).padding(8.0);
    let body = match action {
        Some(id) => Element::clickable(id, body)
            .hover_background(look.selected)
            .accessible(limn::Access::new(limn::Role::Button).name(if invitation {
                t("Establecer estado personalizado").to_owned()
            } else {
                tr!("Estado personalizado: {content}", content)
            })),
        None => Element::panel(body),
    };
    body.radius(16.0)
        .background(raised(look, 16))
        .border(hairline(look, 41), 1.0)
}

/// The bubble of a card's head: the custom status, or with `action` (the
/// account's own) `invitation` to set one; `action` opens the dialog.
fn status_bubble(
    profile: &Profile,
    action: Option<&str>,
    invitation: &str,
    look: &Look,
) -> Option<Element> {
    match custom_status(profile) {
        Some((custom, emoji)) => Some(bubble(custom, emoji, false, action, look)),
        None => action.map(|_| bubble(invitation, None, true, action, look)),
    }
}

/// What someone set beside their status: its line and a server's emoji
/// before it, when they set either.
fn custom_status(profile: &Profile) -> Option<(&str, Option<&crate::identity::TextPicture>)> {
    let emoji = profile.custom_emoji.as_ref();
    match profile.custom.as_deref() {
        Some(custom) => Some((custom, emoji)),
        None => emoji.map(|emoji| ("", Some(emoji))),
    }
}

/// The sizes of a head: a card's (80 px mark at (16, 61) under a 105 px
/// banner, a 16 px status) or the full profile's (120 at (32, 80) under 140).
#[derive(Clone, Copy)]
struct HeadSize {
    width: f32,
    banner: f32,
    avatar: f32,
    ring: f32,
    left: f32,
    top: f32,
    dot: f32,
    /// At least this tall: the mark hangs below the banner.
    height: f32,
}

const CARD_HEAD: HeadSize = HeadSize {
    width: WIDTH,
    banner: BANNER,
    avatar: AVATAR,
    ring: RING,
    left: INSET,
    top: MARK_TOP,
    dot: DOT,
    height: HEAD,
};

/// The top of a card: the banner, its color when it has no picture,
/// and the mark over its bottom edge with the ring cut into the banner, the
/// decoration 1.2 times the mark, the status dot, and `bubble` beside it.
/// A two-line bubble makes the head taller, as Discord's does.
fn card_head(profile: &Profile, bubble: Option<Element>, look: &Look) -> Element {
    sized_card_head(profile, CARD_HEAD, 12.0, bubble, look)
}

/// [`card_head`] of `size`, its round buttons `right` from its right edge.
fn sized_card_head(
    profile: &Profile,
    size: HeadSize,
    right: f32,
    bubble: Option<Element>,
    look: &Look,
) -> Element {
    let mut layers = bubbled_head(profile, size, bubble, look);
    // Over everything, so the buttons take their clicks.
    layers.extend(banner_buttons(profile, right, look));
    Element::stack(layers).size(Some(size.width), None)
}

/// The layers of a head of `size` with `bubble` beside its mark and the two
/// dots that lead to it, as Discord's thought bubble: 9 right of the mark,
/// its top 3 above the mark's middle, as measured on the popout.
fn bubbled_head(
    profile: &Profile,
    size: HeadSize,
    bubble: Option<Element>,
    look: &Look,
) -> Vec<Element> {
    let mut layers = vec![sized_head(profile, size, look)];
    if let Some(bubble) = bubble {
        let (x, top) = (
            size.left + size.avatar + 9.0,
            size.top + size.avatar / 2.0 - 3.0,
        );
        let dot = |side: f32| {
            Element::panel(Element::spacer())
                .size(Some(side), Some(side))
                .radius(side / 2.0)
                .background(raised(look, 16))
                .border(hairline(look, 41), 1.0)
        };
        layers.push(at(x - 3.0, top - 15.0, dot(10.0)));
        layers.push(at(x + 7.0, top - 7.0, dot(20.0)));
        layers.push(Element::column(vec![
            gap(0.0, top),
            Element::row(vec![
                gap(x, 0.0),
                bubble.shrink_to_fit(),
                gap(size.left, 0.0),
            ]),
        ]));
    }
    layers
}

/// The surface the host plays `image`'s animation on, filling its box.
fn moving(image: &limn::RasterImage) -> Element {
    Element::surface(
        crate::message::animation_surface(image.sha256()),
        image.alt(),
    )
    .surface_fit(limn::SurfaceFit::Cover)
}

/// A head of `size`: the banner, the mark ringed in the popout's ground with
/// its status and decoration. Its height is fixed, the mark drawn past it.
/// Each picture moves while it plays; a themed banner without one is its
/// two colors, and the ring the card's veil over the primary.
fn sized_head(profile: &Profile, size: HeadSize, look: &Look) -> Element {
    let banner = Element::panel(match &profile.banner {
        Some(image) if profile.banner_playing => moving(image).size(None, Some(size.banner)),
        Some(image) => Element::raster_image(image.clone().cover()).size(None, Some(size.banner)),
        None => Element::spacer(),
    })
    .size(None, Some(size.banner));
    let banner = match &profile.theme {
        Some(theme) => banner.vertical_gradient([theme.primary, theme.secondary]),
        None => banner.background(profile.banner_tint.unwrap_or(look.raised)),
    }
    .clip_children();
    let face = match &profile.image {
        Some(image) if profile.avatar_playing => Element::stack(vec![
            crate::avatar_with_image(&profile.name, profile.accent, size.avatar, None),
            moving(image)
                .size(Some(size.avatar), Some(size.avatar))
                .radius(size.avatar / 2.0),
        ])
        .size(Some(size.avatar), Some(size.avatar)),
        image => {
            crate::avatar_with_image(&profile.name, profile.accent, size.avatar, image.clone())
        }
    };
    let rim = profile.theme.map_or(ground(look), |theme| theme.ring);
    let ring = Element::panel(face)
        .padding(size.ring)
        .radius(size.avatar / 2.0 + size.ring)
        .background(rim);
    // A column keeps the banner its height: a stack child would take the head's.
    let mut layers = vec![
        Element::column(vec![banner]),
        at(size.left - size.ring, size.top - size.ring, ring),
    ];
    if let Some(decoration) = &profile.identity.marks.decoration {
        let across = size.avatar * 1.2;
        let (centre, middle) = (size.left + size.avatar / 2.0, size.top + size.avatar / 2.0);
        layers.push(at(
            centre - across / 2.0,
            middle - across / 2.0,
            Element::raster_image(decoration.clone()).size(Some(across), Some(across)),
        ));
    }
    // The status over the decoration, in the mark's bottom right corner, ringed
    // in the popout's ground so it reads over the picture.
    if profile.presence {
        let side = size.dot + 12.0;
        let mark = Element::panel(status_mark(profile.status, look, size.dot, rim))
            .padding(6.0)
            .radius(side / 2.0)
            .background(rim);
        layers.push(at(
            size.left + size.avatar - side,
            size.top + size.avatar - side,
            mark,
        ));
    }
    Element::stack(layers).size(Some(size.width), Some(size.height))
}

/// The top of the custom status dialog: a banner in the profile's color, the
/// mark and the bubble beside it (a card has [`card_head`]).
fn head(profile: &Profile, bubble: Option<Element>, look: &Look) -> Element {
    let (tall, across) = (BANNER, AVATAR);
    let ring = across + 2.0 * RING;
    let banner = Element::panel(match &profile.banner {
        Some(image) => Element::raster_image(image.clone().cover()).size(None, Some(tall)),
        None => Element::spacer(),
    })
    .size(None, Some(tall))
    .background(profile.accent)
    .clip_children();
    let face = crate::identity::decorated(
        |side| crate::avatar_with_image(&profile.name, profile.accent, side, profile.image.clone()),
        across,
        profile.identity.marks.decoration.clone(),
    );
    let face = if profile.presence {
        marked(face, across, profile.status, look, look.menu)
    } else {
        face
    };
    let face = Element::panel(face)
        .padding(RING)
        .radius(ring / 2.0)
        .background(look.menu);
    let beside = match bubble {
        Some(bubble) => Element::column(vec![gap(0.0, ring / 2.0 + 6.0), bubble]),
        None => Element::spacer(),
    };
    Element::stack(vec![
        Element::column(vec![banner, gap(0.0, ring / 2.0 + 6.0)]),
        Element::column(vec![
            gap(0.0, tall - ring / 2.0),
            Element::row(vec![
                gap(INSET, 0.0),
                face,
                beside.flex(1.0),
                gap(INSET, 0.0),
            ])
            .gap(8.0),
        ]),
    ])
}

/// A small mark beside a name, such as BOT.
fn tag(label: &str, look: &Look) -> Element {
    Element::panel(
        Element::text(label)
            .font_family(FontFamily::SansBold)
            .font_size(9.0)
            .foreground(look.on_accent),
    )
    .padding(3.0)
    .radius(3.0)
    .background(look.accent)
}

/// The display name in `face`, `size` px, as its owner styled it.
fn shown_name(profile: &Profile, face: FontFamily, size: f32, look: &Look) -> Element {
    crate::identity::styled(
        Element::truncated_text(profile.name.clone())
            .font_family(face)
            .font_size(size)
            .foreground(look.text),
        size,
        (look.text, &[]),
        profile.identity.marks.style.as_ref(),
        look,
    )
}

/// The name, 20 px (24 in the full profile), and the handle with the
/// pronouns under it, then what they share with the account. The
/// account's own card offers to copy the handle; someone else's card, with
/// `note`, the icon of the account's note on them, which opens their full
/// profile where it is written, as the official popout does.
fn names(profile: &Profile, copy: bool, note: Option<&str>, size: f32, look: &Look) -> Element {
    let mut top = vec![shown_name(profile, FontFamily::SansBold, size, look)
        .shrink_to_fit()
        .centered_y()];
    if let (Some(action), Some(field)) = (note, &profile.note) {
        let hint = if field.text.is_empty() {
            t("Agregar una nota").to_owned()
        } else {
            tr!("Nota: {note}", note = field.text.as_str())
        };
        top.push(
            IconButton::new(action, Icon::Edit, look.menu_palette())
                .variant(ButtonVariant::Ghost)
                .tooltip(hint)
                .ink(look.muted)
                .render()
                .centered_y(),
        );
    }
    if let Some(chip) = &profile.identity.marks.tag {
        top.push(crate::identity::tag_chip(chip, 14.0, look).centered_y());
    }
    if profile.bot {
        top.push(tag("BOT", look).centered_y());
    }
    if copy {
        top.push(
            IconButton::new(ids::COPY_NAME, Icon::Copy, look.menu_palette())
                .variant(ButtonVariant::Ghost)
                .tooltip(t("Copiar nombre de usuario"))
                .ink(look.muted)
                .render()
                .centered_y(),
        );
    }
    top.push(Element::spacer().flex(1.0));
    let mut second = profile.handle.clone();
    if let Some(pronouns) = &profile.pronouns {
        second = format!("{second} • {pronouns}");
    }
    let mut column = vec![Element::row(top).gap(6.0), text(second, 14.0, look.muted)];
    if let Some(mutual) = &profile.mutual {
        column.push(
            Element::row(vec![
                icon(Icon::Users, look.muted)
                    .size(Some(14.0), Some(14.0))
                    .centered_y(),
                line(mutual.clone(), 13.0, look.muted)
                    .shrink_to_fit()
                    .centered_y(),
            ])
            .gap(6.0),
        );
    }
    Element::column(column).gap(2.0)
}

/// A titled part of a profile: «Miembro desde», «Roles», under Discord's
/// 12 px medium muted heading, 8 above its body.
fn section(title: &str, body: Element, look: &Look) -> Element {
    Element::column(vec![
        line(title, 12.0, look.muted).font_family(FontFamily::SansMedium),
        body,
    ])
    .gap(8.0)
}

/// A hairline between the parts of a card.
fn divider(look: &Look) -> Element {
    Element::panel(Element::spacer())
        .size(None, Some(1.0))
        .background(look.border)
}

/// A face `size` across: their picture, or their initial on their color.
fn face(face: &Face, size: f32) -> Element {
    crate::avatar_with_image(&face.name, face.accent, size, face.image.clone())
}

/// A 4 px dot between two parts of a line.
fn bullet(color: Color) -> Element {
    Element::panel(Element::spacer())
        .size(Some(4.0), Some(4.0))
        .radius(2.0)
        .background(color)
        .centered_y()
}

/// «Miembro desde» in one line, as Discord's: each date after its mark (the
/// app's for the account, the server's face), a dot between. Where each
/// date is from is said to a reader.
fn member_since(entries: &[Since], look: &Look) -> Element {
    let mut parts = Vec::new();
    for since in entries {
        if !parts.is_empty() {
            parts.push(bullet(look.muted));
        }
        let mark = match &since.face {
            Some(server) => face(server, 16.0),
            None => icon(Icon::Custom(crate::MARK), look.muted).size(Some(16.0), Some(16.0)),
        };
        parts.push(
            Element::row(vec![
                mark.centered_y(),
                line(since.date.clone(), 14.0, look.text).centered_y(),
            ])
            .gap(8.0)
            .accessible(
                limn::Access::default().description(format!("{}: {}", since.scope, since.date)),
            ),
        );
    }
    Element::row(parts).gap(8.0).wrap()
}

/// What the person does: their activity cards, or else the line their
/// presence gives.
fn activity_parts(profile: &Profile, width: f32, look: &Look) -> Vec<Element> {
    let identity = &profile.identity;
    let voice = identity
        .voice
        .as_ref()
        .map(|card| crate::identity::voice_card(card, width, look));
    if identity.activities.is_empty() {
        return voice
            .into_iter()
            .chain(
                profile
                    .activity
                    .iter()
                    .map(|activity| text(activity.clone(), 13.0, look.text)),
            )
            .collect();
    }
    vec![Element::column(
        voice
            .into_iter()
            .chain(
                identity
                    .activities
                    .iter()
                    .map(|card| crate::identity::activity_card(card, width, look)),
            )
            .collect(),
    )
    .gap(8.0)]
}

/// Someone's bio as Markdown, `width` across. A card (`clamp`) shows three
/// lines, as the official client's popout does, and leads to the full
/// profile for the rest; the full profile shows all of it.
fn bio(profile: &Profile, clamp: bool, width: f32, look: &Look) -> Option<Element> {
    let about = profile.about.as_ref()?;
    let mut bio = crate::identity::with_pictures(about, &profile.identity.pictures, width, look);
    let long = about.lines().count() > BIO_LINES || about.chars().count() > BIO_CHARS;
    if clamp && long {
        bio = Element::column(vec![
            bio.max_height(BIO_HEIGHT).clip_children(),
            match &profile.full_action {
                Some(action) => Element::clickable(
                    action.replacen(ids::FULL, ids::FULL_BIO, 1),
                    line(t("Ver biografía completa"), 13.0, look.accent),
                )
                .hover_foreground(look.text),
                None => Element::spacer().size(None, Some(0.0)),
            },
        ])
        .gap(4.0);
    }
    Some(bio)
}

/// A round button over a card's banner, 32 px on a dark disc with a white
/// icon, its tooltip the label.
fn banner_button(id: &str, glyph: Icon, label: &str, look: &Look) -> Element {
    let glyph = icon(glyph, Color::rgb(255, 255, 255)).size(Some(16.0), Some(16.0));
    let disc = Element::panel(centered(glyph))
        .size(Some(32.0), Some(32.0))
        .radius(16.0)
        .background(Color::rgba(0, 0, 0, 143))
        .border(Color::rgba(255, 255, 255, 20), 1.0);
    let button = Element::clickable(id, disc)
        .accessible(limn::Access::new(limn::Role::Button).name(label.to_owned()));
    Tooltip::new(button, id, label, look.menu_palette()).render()
}

/// The menu a card's button opens, `placement` from the button; closed, the
/// anchor alone. Its width and rows are the person menu's.
fn card_menu(
    anchor: Element,
    entries: Vec<MenuEntry>,
    open: bool,
    dismiss: &str,
    placement: PopoverPlacement,
    look: &Look,
) -> Element {
    let width = menu_width(&entries);
    DropdownMenu::new(
        anchor,
        menu_entries(entries, look),
        open,
        dismiss,
        look.menu_palette(),
    )
    .context_menu_style()
    .context_metrics(crate::look::MENU_METRICS)
    .width(width)
    .placement(placement)
    .render()
}

/// The friendship's button and «…» of a profile, each with its menu, drawn
/// by `button` and opening `placement` from it. In the full profile
/// (`full`), «…» leaves out «Ver perfil completo», which is where it is.
fn person_buttons(
    profile: &Profile,
    button: &dyn Fn(&str, Icon, &str, &Look) -> Element,
    placement: PopoverPlacement,
    full: bool,
    look: &Look,
) -> Vec<Element> {
    let mut buttons = Vec::new();
    if let Some(relation) = &profile.relation {
        let (glyph, label) = match relation.state {
            Friendship::None => (Icon::Add, t("Añadir amigos")),
            Friendship::Friend => (Icon::Check, t("Amigo")),
            Friendship::Outgoing => (Icon::Clock, t("Solicitud de amistad enviada")),
            Friendship::Incoming => (Icon::Clock, t("Solicitud de amistad entrante")),
        };
        let open = profile.open_menu == Some(CardMenu::Relation);
        let entries = if open {
            relation.menu.clone()
        } else {
            Vec::new()
        };
        buttons.push(card_menu(
            button(&relation.action, glyph, label, look),
            entries,
            open,
            ids::CARD_RELATION_DISMISS,
            placement,
            look,
        ));
    }
    if let Some((id, entries)) = &profile.more {
        let open = profile.open_menu == Some(CardMenu::More);
        let mut entries: Vec<MenuEntry> = if open { entries.clone() } else { Vec::new() };
        if full {
            entries.retain(
                |entry| !matches!(entry, MenuEntry::Item(id, ..) if id.starts_with(ids::FULL)),
            );
            if matches!(entries.first(), Some(MenuEntry::Separator)) {
                entries.remove(0);
            }
        }
        buttons.push(card_menu(
            button(id, Icon::More, t("Más"), look),
            entries,
            open,
            ids::CARD_MORE_DISMISS,
            placement,
            look,
        ));
    }
    buttons
}

/// The round buttons over a card's banner, 8 from its top and `right` from
/// its right: the friendship's and «…», each with its menu. Nothing when
/// neither shows.
fn banner_buttons(profile: &Profile, right: f32, look: &Look) -> Option<Element> {
    let buttons = person_buttons(
        profile,
        &banner_button,
        PopoverPlacement::BottomEnd,
        false,
        look,
    );
    if buttons.is_empty() {
        return None;
    }
    Some(Element::column(vec![
        gap(0.0, 8.0),
        Element::row(vec![
            Element::spacer().flex(1.0),
            Element::row(buttons).gap(8.0),
            gap(right, 0.0),
        ]),
    ]))
}

/// A square button of the full profile, as Discord's: 32 px, 8 px corners,
/// `fill` (at rest and under the pointer) and a hairline, its tooltip the
/// label.
fn full_button(id: &str, glyph: Icon, label: &str, fill: (Color, Color), look: &Look) -> Element {
    let square = Element::panel(centered(
        icon(glyph, look.text).size(Some(16.0), Some(16.0)),
    ))
    .size(Some(32.0), Some(32.0))
    .radius(8.0)
    .background(fill.0)
    .border(hairline(look, 41), 1.0);
    let button = Element::clickable(id, square)
        .radius(8.0)
        .hover_background(fill.1)
        .accessible(limn::Access::new(limn::Role::Button).name(label.to_owned()));
    Tooltip::new(button, id, label, look.menu_palette()).render()
}

/// `child` at (`x`, `y`) of the stack it is in, placed by spacers.
fn at(x: f32, y: f32, child: Element) -> Element {
    Element::column(vec![gap(0.0, y), Element::row(vec![gap(x, 0.0), child])])
}

/// The palette of a skeleton on a card's or a modal's surface: its shapes
/// pulse between two strengths of the text color. limn's `raised` is opaque
/// and only five levels off the menu's, so the shapes vanished at the bottom
/// of each pulse.
fn skeleton_palette(look: &Look) -> ComponentPalette {
    ComponentPalette {
        raised: Look::alpha(look.text, 16),
        hover: Look::alpha(look.text, 30),
        ..look.menu_palette()
    }
}

/// The four text lines of a skeleton, 10 px apart, each as wide as its share.
fn skeleton_lines(id: &str, look: &Look) -> Element {
    Element::column(
        SKELETON_LINES
            .iter()
            .enumerate()
            .map(|(n, width)| {
                Skeleton::new(format!("{id}:line:{n}"), skeleton_palette(look))
                    .width(*width)
                    .height(12.0)
                    .radius(4.0)
                    .render()
            })
            .collect(),
    )
    .gap(10.0)
}

/// The card while its profile is read: the shapes of the card's parts where
/// they will be, on its surface, as Discord's skeleton. `id` keeps
/// their ids stable across frames.
pub fn card_skeleton(id: &str, look: &Look) -> Element {
    let palette = skeleton_palette(look);
    let block = |part: &str, width: f32, height: f32, radius: f32| {
        Skeleton::new(format!("{id}:{part}"), palette)
            .width(width)
            .height(height)
            .radius(radius)
            .render()
    };
    // The 6 px ring is the card's own surface, inside the 80 px avatar.
    let avatar = Element::panel(
        Skeleton::circle(format!("{id}:avatar"), palette, AVATAR - 2.0 * RING).render(),
    )
    .padding(RING)
    .radius(AVATAR / 2.0)
    .background(ground(look));
    let chips = Element::row(vec![
        block("chip:0", 52.0, 16.0, 4.0),
        gap(16.0, 0.0),
        block("chip:1", 72.0, 16.0, 4.0),
        gap(16.0, 0.0),
        block("chip:2", 48.0, 16.0, 4.0),
    ]);
    let divider = Element::panel(Element::spacer())
        .size(Some(WIDTH), Some(1.0))
        .background(look.border);
    Element::stack(vec![
        at(0.0, 0.0, block("banner", WIDTH, BANNER, 0.0)),
        at(INSET, 65.0, avatar),
        at(INSET, 157.0, block("name", 147.0, 18.0, 4.0)),
        at(INSET, 183.0, block("handle", 94.0, 12.0, 4.0)),
        at(INSET, 211.0, chips),
        at(0.0, 239.0, divider),
        at(INSET, 256.0, skeleton_lines(id, look)),
    ])
    .size(Some(WIDTH), Some(SKELETON_HEIGHT))
    // An id is what lets the platform hear it: without one the status is dropped.
    .accessible(
        limn::Access::new(limn::Role::Status)
            .id(id)
            .name(t("Cargando perfil")),
    )
}

/// The card of someone else: the head with its buttons, then the
/// name with the notice of an ignored or blocked person, what restricts
/// them, their mutual friends and servers, what they do, their bio, their
/// roles, and the note when there is no full profile to open. Its body
/// scrolls between the head and the footer when the window is too short for
/// all of it (a popover is never taller than the window); the effect gives
/// way while the pointer is over the card. While its profile is being read,
/// it is its [`card_skeleton`].
pub fn card(profile: &Profile, look: &Look) -> Element {
    if profile.loading {
        return card_skeleton(&format!("profile:loading:{}", profile.handle), look);
    }
    let bubble = status_bubble(
        profile,
        profile.set_custom.then_some(ids::CUSTOM),
        t("Añadir estado"),
        look,
    );
    let note = profile
        .full_action
        .as_ref()
        .map(|action| action.replacen(ids::FULL, ids::FULL_NOTE, 1));
    // A themed card draws in its own colors, inside its frame.
    let themed = profile.theme.map(|theme| (theme, theme.look(look)));
    let look = themed.as_ref().map_or(look, |(_, look)| look);
    let edge = if themed.is_some() { POPOUT_FRAME } else { 0.0 };
    let width = WIDTH - 2.0 * (INSET + edge);
    let mut body = vec![card_names(profile, note.as_deref(), false, look)];
    if let Some(notice) = &profile.notice {
        body.push(card_notice(notice, look));
    }
    if let Some(restriction) = &profile.restriction {
        body.push(restriction_badge(restriction, look));
    }
    body.extend(card_mutuals(profile, look));
    body.extend(card_about(profile, width, look));
    body.extend(card_roles(profile, look));
    // Without a full profile to open (the account's own), the field stays.
    if let Some(note) = profile.note.as_ref().filter(|_| note.is_none()) {
        body.push(note_section(note, profile, look));
    }
    // Discord's spacing: the name 12 under the head, its parts 12 apart, 16
    // around the field at the bottom.
    let head = HeadSize {
        width: WIDTH - 2.0 * edge,
        ..CARD_HEAD
    };
    let mut parts = vec![
        sized_card_head(profile, head, 12.0, bubble, look),
        ScrollArea::new(
            ids::card_body(&profile.handle),
            Element::column(vec![
                gap(0.0, 12.0),
                Element::row(vec![
                    gap(INSET, 0.0),
                    Element::column(body).gap(12.0).flex(1.0),
                    gap(INSET, 0.0),
                ]),
                gap(0.0, 16.0),
            ]),
        )
        // Discord's thin scrollbar, shown under the pointer.
        .appearance(look.scrollbar(false))
        .render()
        .flex(1.0),
    ];
    let footer = card_footer(profile, look);
    let below = if footer.is_some() { POPOUT_FOOTER } else { 0.0 };
    parts.extend(footer);
    let card = Element::column(parts).hover_region();
    let card = match &themed {
        Some((theme, _)) => theme.frame(card, BANNER, below, edge, CornerRadii::same(CARD_RADIUS)),
        None => card,
    };
    crate::identity::with_effect(
        card,
        WIDTH,
        profile.identity.effect.as_ref(),
        Some(ids::CARD_EFFECT),
    )
}

/// The footer of a card: Discord's 46 px field to write to its person, whose
/// hint names them, or on the account's own card «Editar perfil».
fn card_footer(profile: &Profile, look: &Look) -> Option<Element> {
    let row = match &profile.message {
        Some(field) => {
            let hint = tr!("Enviar un mensaje a @{name}", name = profile.name);
            let palette = ComponentPalette {
                raised: Color::rgba(0, 0, 0, 31),
                border: hairline(look, 51),
                muted: look.dim,
                ..look.menu_palette()
            };
            TextField::new(field.id.clone(), field.text.clone(), hint.clone(), palette)
                .height(46.0)
                .radius(8.0)
                .accessibility_label(hint)
                .render()
                .font_size(14.0)
                .padding(15.0)
                .flex(1.0)
        }
        None if profile.example_button => {
            Button::new(ids::EXAMPLE, t("Botón de ejemplo"), look.accent_palette())
                .render()
                .flex(1.0)
        }
        None if profile.edit_profile => {
            Button::new(ids::EDIT, t("Editar perfil"), look.accent_palette())
                .icon(Icon::Edit)
                .render()
                .flex(1.0)
        }
        None => return None,
    };
    Some(Element::column(vec![
        Element::row(vec![gap(INSET, 0.0), row, gap(INSET, 0.0)]),
        gap(0.0, 16.0),
    ]))
}

/// Badges a card's name row shows before «+N».
const BADGES_SHOWN: usize = 6;

/// The name of a card: the display name at 20 px semibold, its note's icon
/// (or, on the account's own, `copy` the handle) while the pointer is on the
/// card, and the BOT mark; then the handle, the pronouns, the server tag and
/// the badges in their box, in as many lines as they need.
fn card_names(profile: &Profile, note: Option<&str>, copy: bool, look: &Look) -> Element {
    names_sized(profile, note, copy, 20.0, look)
}

/// [`card_names`] with the name `size` px: 24 in the full profile.
fn names_sized(
    profile: &Profile,
    note: Option<&str>,
    copy: bool,
    size: f32,
    look: &Look,
) -> Element {
    let mut top = vec![shown_name(profile, FontFamily::SansSemibold, size, look)
        .shrink_to_fit()
        .centered_y()];
    // The mark of an app right after its name, as Discord's.
    if profile.bot {
        top.push(tag("BOT", look).centered_y());
    }
    if let (Some(action), Some(field)) = (note, &profile.note) {
        let hint = if field.text.is_empty() {
            t("Agregar nota (solo visible para ti)")
        } else {
            t("Editar nota (solo visible para ti)")
        };
        top.push(
            IconButton::new(action, Icon::Edit, look.menu_palette())
                .variant(ButtonVariant::Ghost)
                .tooltip(hint)
                .ink(look.muted)
                .render()
                .centered_y()
                .reveal_on_hover(format!("pp:card:{}:note", profile.handle)),
        );
    }
    if copy {
        top.push(
            IconButton::new(ids::COPY_NAME, Icon::Copy, look.menu_palette())
                .variant(ButtonVariant::Ghost)
                .tooltip(t("Copiar nombre de usuario"))
                .ink(look.muted)
                .render()
                .centered_y()
                .reveal_on_hover(format!("pp:card:{}:copy", profile.handle)),
        );
    }
    let mut second = vec![text(profile.handle.clone(), 14.0, look.text).centered_y()];
    if let Some(pronouns) = &profile.pronouns {
        second.push(
            Element::panel(Element::spacer())
                .size(Some(4.0), Some(4.0))
                .radius(2.0)
                .background(look.text)
                .centered_y(),
        );
        second.push(text(pronouns.clone(), 14.0, look.text).centered_y());
    }
    if let Some(chip) = &profile.identity.marks.tag {
        second.push(card_tag(chip, look).centered_y());
    }
    if !profile.identity.badges.is_empty() {
        // Discord's box: 22 tall, 8 px corners, a hairline around 20 px
        // badges 2 apart.
        let badges =
            crate::identity::badge_items(&profile.identity.badges, 20.0, BADGES_SHOWN, look);
        second.push(
            Element::panel(Element::row(vec![
                gap(2.0, 0.0),
                // Badges without a picture are their names: they may wrap.
                Element::row(badges).gap(2.0).wrap().shrink_to_fit(),
                gap(2.0, 0.0),
            ]))
            .padding(1.0)
            .radius(8.0)
            .background(boxed(profile, look, 20))
            .border(hairline(look, 61), 1.0)
            .centered_y(),
        );
    }
    Element::column(vec![
        Element::row(top).gap(6.0),
        Element::row(second).gap(6.0).wrap(),
    ])
}

/// The server tag beside a name: its badge and letters on a raised pill,
/// 24 px tall.
fn card_tag(tag: &crate::identity::TagChip, look: &Look) -> Element {
    let mut inside = Vec::new();
    if let Some(badge) = &tag.badge {
        inside.push(
            Element::raster_image(badge.clone())
                .size(Some(14.0), Some(14.0))
                .centered_y(),
        );
    }
    inside.push(text(tag.text.clone(), 12.0, look.text).centered_y());
    let hint = match &tag.server {
        Some(server) => tr!("Etiqueta de servidor de {server}", server),
        None => t("Etiqueta de servidor").to_owned(),
    };
    Element::panel(Element::row(vec![
        gap(8.0, 0.0),
        Element::row(inside).gap(4.0).centered_y(),
        gap(8.0, 0.0),
    ]))
    .size(None, Some(24.0))
    .radius(8.0)
    .background(look.raised)
    .border(look.border, 1.0)
    .accessible(limn::Access::default().description(format!("{hint}: {}", tag.text)))
}

/// What the account does with someone it blocks or ignores, as a panel under
/// the name: for an ignored one, a link to lift it.
fn card_notice(notice: &Notice, look: &Look) -> Element {
    let parts = match notice {
        Notice::Blocked => vec![text(t("Bloqueaste a este usuario"), 14.0, look.muted)],
        Notice::Ignored { unignore } => vec![
            text(t("Ignoraste a este usuario"), 14.0, look.muted),
            text(" • ", 14.0, look.muted),
            Element::clickable(
                unignore.clone(),
                line(t("Dejar de ignorar"), 14.0, look.accent),
            )
            .hover_foreground(look.text),
        ],
    };
    Element::panel(Element::column(vec![
        gap(0.0, 8.0),
        Element::row(vec![
            gap(12.0, 0.0),
            Element::row(parts).wrap(),
            gap(12.0, 0.0),
        ]),
        gap(0.0, 8.0),
    ]))
    .radius(8.0)
    .background(look.raised)
}

/// The mutual friends and servers of a card, each part opening the full
/// profile on its tab: up to three faces overlapping, then the
/// count. Nothing while they are unknown or there are none.
fn card_mutuals(profile: &Profile, look: &Look) -> Option<Element> {
    let known = |mutual: &Mutual| match mutual {
        Mutual::Known(faces) if !faces.is_empty() => Some(faces.clone()),
        _ => None,
    };
    let (friends, servers) = (
        known(&profile.mutual_friends),
        known(&profile.mutual_servers),
    );
    if friends.is_none() && servers.is_none() {
        return None;
    }
    let base = profile
        .full_action
        .as_ref()?
        .replacen(ids::FULL, ids::MUTUAL, 1);
    let both = friends.is_some() && servers.is_some();
    let size = if both { 12.0 } else { 14.0 };
    let mut parts = Vec::new();
    if let Some(faces) = &friends {
        let words = match faces.len() {
            1 => t("1 amigo en común").to_owned(),
            n => tr!("{n} amigos en común", n),
        };
        parts.push(mutual_part(
            format!("{base}:friends"),
            faces,
            words,
            size,
            look,
        ));
    }
    if both {
        parts.push(
            Element::panel(Element::spacer())
                .size(Some(4.0), Some(4.0))
                .radius(2.0)
                .background(look.muted)
                .centered_y(),
        );
    }
    if let Some(faces) = &servers {
        let words = match faces.len() {
            1 => t("1 servidor en común").to_owned(),
            n => tr!("{n} servidores en común", n),
        };
        // Discord names the servers without their icons.
        parts.push(mutual_part(
            format!("{base}:servers"),
            &[],
            words,
            size,
            look,
        ));
    }
    Some(Element::row(parts).gap(6.0).wrap())
}

/// One part of the mutuals: its faces (16 px, the first three overlapping) and
/// its words, as a button that opens the full profile on its tab.
fn mutual_part(id: String, faces: &[Face], words: String, size: f32, look: &Look) -> Element {
    const FACE: f32 = 16.0;
    const STEP: f32 = 10.0;
    let shown: Vec<Element> = faces
        .iter()
        .take(3)
        .enumerate()
        .map(|(index, known)| at(index as f32 * STEP, 0.0, face(known, FACE)))
        .collect();
    let width = FACE + shown.len().saturating_sub(1) as f32 * STEP;
    let mut parts = Vec::new();
    if !shown.is_empty() {
        parts.push(
            Element::stack(shown)
                .size(Some(width), Some(FACE))
                .centered_y(),
        );
    }
    parts.push(text(words.clone(), size, look.muted).centered_y());
    Element::clickable(id, Element::row(parts).gap(6.0))
        .accessible(limn::Access::new(limn::Role::Button).name(words))
}

/// The parts of a card under its mutuals: the bio (see [`bio`]), the bot's
/// line, the game collection and what the person does (their voice card
/// first), in Discord's order and without headings.
fn card_about(profile: &Profile, width: f32, look: &Look) -> Vec<Element> {
    let mut body: Vec<Element> = bio(profile, true, width, look).into_iter().collect();
    if profile.bot {
        body.push(text(t("Cuenta automatizada (bot)."), 12.0, look.faint));
    }
    body.extend(game_card(profile, 12.0, boxed(profile, look, 16), look));
    body.extend(activity_parts(profile, width, look));
    body
}

/// Games a card's collection shows, the last dimmed under «+N» when there
/// are more.
const GAMES_SHOWN: usize = 4;

/// «Colección de juegos», as Discord's card: its title (`size` px: 12 in a
/// popout, 14 beside a conversation) and up to four 24 px icons on the
/// right, on `fill`; it opens the full profile. Nothing without games.
fn game_card(profile: &Profile, size: f32, fill: Color, look: &Look) -> Option<Element> {
    if profile.games.is_empty() {
        return None;
    }
    const ICON: f32 = 24.0;
    let more = profile.games.len().saturating_sub(GAMES_SHOWN);
    let icons: Vec<Element> = profile
        .games
        .iter()
        .take(GAMES_SHOWN)
        .enumerate()
        .map(|(index, game)| {
            // A grey square under each, the picture's ground while it loads.
            let mut layers = vec![Element::panel(Element::spacer())
                .size(Some(ICON), Some(ICON))
                .radius(4.0)
                .background(hairline(look, 61))];
            if let Some(image) = &game.icon {
                layers.push(
                    Element::raster_image(image.clone().cover())
                        .size(Some(ICON), Some(ICON))
                        .radius(4.0),
                );
            }
            if more > 0 && index == GAMES_SHOWN - 1 {
                // Discord's half-faded icon under black at 60 %.
                layers.push(
                    Element::panel(centered(
                        line(format!("+{more}"), 12.0, Color::rgb(224, 224, 227))
                            .font_family(FontFamily::SansMedium),
                    ))
                    .size(Some(ICON), Some(ICON))
                    .radius(4.0)
                    .background(Color::rgba(0, 0, 0, 204)),
                );
            }
            Element::stack(layers)
                .size(Some(ICON), Some(ICON))
                .accessible(limn::Access::default().description(game.name.clone()))
        })
        .collect();
    let title = t("Colección de juegos");
    let card = Element::panel(Element::row(vec![
        line(title, size, look.text)
            .font_family(FontFamily::SansMedium)
            .centered_y(),
        Element::spacer().flex(1.0),
        Element::row(icons).gap(4.0).centered_y(),
    ]))
    .padding(8.0)
    .radius(8.0)
    .background(fill);
    Some(match &profile.full_action {
        Some(action) => Element::clickable(action.replacen(ids::FULL, ids::FULL_GAMES, 1), card)
            .radius(8.0)
            .hover_background(look.hover)
            .accessible(limn::Access::new(limn::Role::Button).name(title.to_owned())),
        None => card,
    })
}

/// The roles of a card, as chips in as many lines as they need. Each the
/// account may take away has a «×»; a «+» (or «Añadir roles» with none) opens
/// the checklist of roles when the account may give them.
fn card_roles(profile: &Profile, look: &Look) -> Option<Element> {
    let mut chips: Vec<Element> = profile
        .roles
        .iter()
        .enumerate()
        .map(|(index, (name, color))| {
            card_role(
                name,
                *color,
                profile
                    .identity
                    .role_marks
                    .get(index)
                    .and_then(Option::as_ref),
                profile
                    .role_removals
                    .get(index)
                    .and_then(|removal| removal.as_deref()),
                profile.theme.map(|theme| theme.role),
                look,
            )
        })
        .collect();
    if let Some((id, entries)) = &profile.role_menu {
        let open = profile.open_menu == Some(CardMenu::Roles);
        let menu = if open { entries.clone() } else { Vec::new() };
        let anchor = if profile.roles.is_empty() {
            Button::new(id.clone(), t("Añadir roles"), look.menu_palette())
                .variant(ButtonVariant::Ghost)
                .icon(Icon::Add)
                .render()
        } else {
            square_button(
                id,
                Icon::Add,
                t("Añadir roles"),
                t("Añadir roles").to_owned(),
                look,
            )
        };
        chips.push(card_menu(
            anchor,
            menu,
            open,
            ids::CARD_ROLES_DISMISS,
            PopoverPlacement::BottomStart,
            look,
        ));
    }
    (!chips.is_empty()).then(|| Element::row(chips).gap(4.0).wrap())
}

/// A role of a card: Discord's 24 px outlined chip with its 12 px color dot,
/// its picture or emoji and its name, and the «×» that takes it away when
/// `remove` names the action. A themed card's `pill` fills and outlines it.
fn card_role(
    name: &str,
    color: Color,
    mark: Option<&crate::identity::RoleMark>,
    remove: Option<&str>,
    pill: Option<(Color, Color)>,
    look: &Look,
) -> Element {
    let mut inside = vec![gap(2.0, 0.0), parts::color_dot(color, 12.0).centered_y()];
    if let Some(mark) = mark {
        inside.push(crate::identity::role_mark(mark, 14.0, look).centered_y());
    }
    inside.push(text(name, 12.0, look.text).centered_y());
    match remove {
        Some(action) => inside.push(square_button(
            action,
            Icon::Close,
            t("Eliminar rol"),
            tr!("Quitar el rol {role}", role = name),
            look,
        )),
        None => inside.push(gap(8.0, 0.0)),
    }
    let chip = Element::panel(Element::row(inside).gap(4.0).centered_y())
        .size(None, Some(24.0))
        .radius(8.0);
    match pill {
        Some((fill, outline)) => chip.background(fill).border(outline, 1.0),
        None => chip.border(hairline(look, 51), 1.0),
    }
}

/// A 24 px square button with an icon of 12 px: its tooltip is `hint`, and
/// `name` what a reader hears of it.
fn square_button(id: &str, glyph: Icon, hint: &str, name: String, look: &Look) -> Element {
    let button = Element::clickable(
        id,
        centered(icon(glyph, look.muted).size(Some(12.0), Some(12.0))).size(Some(24.0), Some(24.0)),
    )
    .radius(12.0)
    .hover_background(look.hover)
    .accessible(limn::Access::new(limn::Role::Button).name(name));
    Tooltip::new(button, id, hint, look.menu_palette()).render()
}

/// A warning pill: a clock and what restricts them.
fn restriction_badge(label: &str, look: &Look) -> Element {
    Element::panel(
        Element::row(vec![
            icon(Icon::Clock, look.danger).centered_y(),
            text(label, 13.0, look.text).flex(1.0).centered_y(),
        ])
        .gap(6.0),
    )
    .padding(8.0)
    .radius(8.0)
    .background(look.raised)
    .border(look.danger, 1.0)
    .accessible(limn::Access::new(limn::Role::Status).name(label.to_owned()))
}

/// A list of the full profile, as known: unknown is never shown as none.
/// Each in it is a row of Discord's: a 40 px face and the name.
fn mutual_list(mutual: &Mutual, look: &Look) -> Element {
    match mutual {
        Mutual::Unknown => text(t("No disponible en esta conexión."), 14.0, look.muted),
        Mutual::Loading => text(t("Cargando…"), 14.0, look.muted),
        Mutual::Known(faces) if faces.is_empty() => text(t("Ninguno en común."), 14.0, look.muted),
        Mutual::Known(faces) => Element::column(
            faces
                .iter()
                .map(|known| {
                    Element::row(vec![
                        face(known, 40.0).centered_y(),
                        strong_line(known.name.clone(), 16.0, look.text)
                            .shrink_to_fit()
                            .centered_y(),
                    ])
                    .gap(12.0)
                    .padding(8.0)
                })
                .collect(),
        ),
    }
}

/// The label of a tab of the full profile: «Actividad», and the mutuals with
/// their count once known, as Discord's «9 amigos en común».
fn tab_label(tab: FullTab, full: &FullProfile) -> String {
    let count = |list: &Mutual| match list {
        Mutual::Known(faces) => Some(faces.len()),
        _ => None,
    };
    match tab {
        FullTab::Activity => t("Actividad").to_owned(),
        FullTab::Friends => match count(&full.mutual_friends) {
            Some(1) => t("1 amigo en común").to_owned(),
            Some(n) => tr!("{n} amigos en común", n),
            None => t("Amigos en común").to_owned(),
        },
        FullTab::Servers => match count(&full.mutual_servers) {
            Some(1) => t("1 servidor en común").to_owned(),
            Some(n) => tr!("{n} servidores en común", n),
            None => t("Servidores en común").to_owned(),
        },
    }
}

/// The full profile's panel: as wide as Discord's (960) and as tall as the
/// window, its card 400 wide, 48 in from the panel's top and sides and 32
/// from the column of tabs beside it; a column narrower than 280 goes under
/// the card.
/// Discord's full profile: 962 wide, up to 800 high, 48 from the window's
/// top and bottom, radius 16 and a 1 px border.
const FULL_PANEL: f32 = 962.0;
const FULL_TALL: f32 = 800.0;
const FULL_RADIUS: f32 = 16.0;
const FULL_CARD: f32 = 400.0;
const FULL_EDGE: f32 = 48.0;
const FULL_GUTTER: f32 = 32.0;
const FULL_TABS_MIN: f32 = 280.0;
/// The card's head: a 140 px banner, the 120 px mark at (32, 80) in an 8 px
/// ring with its 24 px status.
const FULL_HEAD: HeadSize = HeadSize {
    width: FULL_CARD,
    banner: 140.0,
    avatar: 120.0,
    ring: 8.0,
    left: 32.0,
    top: 80.0,
    dot: 24.0,
    height: 208.0,
};

/// The full profile's card: the head, the name at 24 px, «Enviar mensaje»
/// with the friendship's button and «…», then the bio and Discord's
/// sections, at least `height` tall so it reaches the panel's foot.
fn full_card(full: &FullProfile, height: f32, look: &Look) -> Element {
    let profile = &full.profile;
    // A themed card draws in its own colors, inside its frame.
    let themed = profile.theme.map(|theme| (theme, theme.look(look)));
    let look = themed.as_ref().map_or(look, |(_, look)| look);
    let edge = if themed.is_some() { FULL_FRAME } else { 0.0 };
    let head = HeadSize {
        width: FULL_CARD - 2.0 * edge,
        ..FULL_HEAD
    };
    let width = head.width - 2.0 * FULL_HEAD.left;
    let mut buttons = Vec::new();
    if let Some(action) = &profile.message_action {
        let button = match &themed {
            Some((theme, _)) => Button::new(
                action.clone(),
                t("Enviar mensaje"),
                theme.button_palette(look),
            )
            .variant(ButtonVariant::Raised),
            None => Button::new(action.clone(), t("Enviar mensaje"), look.accent_palette()),
        };
        buttons.push(button.icon(Icon::Chat).render());
    }
    let fill = profile
        .theme
        .map_or((hairline(look, 31), look.hover), |theme| {
            (theme.secondary_button[0], theme.secondary_button[1])
        });
    buttons.extend(person_buttons(
        profile,
        &|id, glyph, label, look| full_button(id, glyph, label, fill, look),
        PopoverPlacement::BottomStart,
        true,
        look,
    ));
    let mut body = Vec::new();
    if !buttons.is_empty() {
        body.push(Element::row(buttons).gap(8.0));
    }
    if let Some(notice) = &profile.notice {
        body.push(card_notice(notice, look));
    }
    if let Some(restriction) = &profile.restriction {
        body.push(restriction_badge(restriction, look));
    }
    body.extend(bio(profile, false, width, look));
    if profile.bot {
        body.push(text(t("Cuenta automatizada (bot)."), 12.0, look.faint));
    }
    if !profile.member_since.is_empty() {
        body.push(section(
            t("Miembro desde"),
            member_since(&profile.member_since, look),
            look,
        ));
    }
    if let Some(since) = &full.friends_since {
        body.push(section(
            t("Amigos desde el"),
            text(since.clone(), 14.0, look.text),
            look,
        ));
    }
    if let Some(roles) = full.server.as_ref().and_then(|_| card_roles(profile, look)) {
        body.push(section(t("Roles"), roles, look));
    }
    if !full.connections.is_empty() {
        body.push(section(
            t("Conexiones"),
            crate::identity::connections(&full.connections, look),
            look,
        ));
    }
    if let Some(note) = &profile.note {
        body.push(note_section(note, profile, look));
    }
    let bubble =
        custom_status(profile).map(|(custom, emoji)| bubble(custom, emoji, false, None, look));
    let content = Element::column(vec![
        Element::stack(bubbled_head(profile, head, bubble, look)).size(Some(head.width), None),
        Element::row(vec![
            gap(FULL_HEAD.left, 0.0),
            Element::column(vec![
                gap(0.0, 12.0),
                names_sized(profile, None, false, 24.0, look),
                gap(0.0, 24.0),
                Element::column(body).gap(24.0),
                gap(0.0, 32.0),
            ])
            .flex(1.0),
            gap(FULL_HEAD.left, 0.0),
        ]),
    ]);
    let card = match &themed {
        Some((theme, _)) => theme.frame(
            content,
            head.banner,
            0.0,
            edge,
            CornerRadii::top(FULL_RADIUS),
        ),
        None => Element::panel(content)
            .background(ground(look))
            .border(look.border, 1.0)
            .corner_radii(CornerRadii::top(16.0))
            .clip_children(),
    };
    crate::identity::with_effect(
        card.size(Some(FULL_CARD), None).min_size(FULL_CARD, height),
        FULL_CARD,
        profile.identity.effect.as_ref(),
        None,
    )
}

/// The column beside the full profile's card: Discord's tabs, then what they
/// do (their activities, else a word that there are none), or the friends
/// or servers in common.
fn full_tabs(full: &FullProfile, width: f32, look: &Look) -> Element {
    let profile = &full.profile;
    // shortcut: widths estimated from the characters, as the friends' tabs
    // do; Discord's 30 between them, less when the column is narrow.
    let tabs: Vec<Tab> = FullTab::ALL
        .into_iter()
        .map(|tab| {
            let label = tab_label(tab, full);
            let width = label.chars().count() as f32 * 7.8 + 8.0;
            Tab::new(format!("{}{}", ids::FULL_TAB, tab.key()), label, width)
                .active(tab == full.tab)
        })
        .collect();
    let taken: f32 = tabs.iter().map(|tab| tab.width).sum();
    let spacing = ((width - taken - 4.0) / 2.0).clamp(8.0, 22.0);
    let palette = ComponentPalette {
        border: hairline(look, 41),
        ..look.menu_palette()
    };
    let tabs = Tabs::new(tabs, palette)
        .variant(TabsVariant::Underlined)
        .accent(look.text)
        .font_size(15.0)
        .id("pp:full:tabs")
        .height(44.0)
        .gap(spacing)
        .render();
    let shown = match full.tab {
        // While the read is in flight nothing is known yet: its lines, never
        // «none».
        FullTab::Activity if full.loading => {
            skeleton_lines(&format!("profile:loading:{}:about", profile.handle), look)
        }
        FullTab::Activity => {
            let mut parts = Vec::new();
            if !profile.identity.platforms.is_empty() && profile.presence {
                parts.push(crate::identity::platforms(
                    &profile.identity.platforms,
                    look,
                ));
            }
            let activities = activity_parts(profile, width, look);
            let recent = recent_section(&full.recent, look);
            if activities.is_empty() && recent.is_none() {
                parts.push(text(t("Sin actividad por ahora."), 14.0, look.muted));
            }
            parts.extend(activities);
            // Sections 24 apart, as Discord's.
            if let Some(recent) = recent {
                parts.push(gap(0.0, 16.0));
                parts.push(recent);
            }
            Element::column(parts).gap(8.0)
        }
        FullTab::Friends => mutual_list(&full.mutual_friends, look),
        FullTab::Servers => mutual_list(&full.mutual_servers, look),
    };
    Element::column(vec![tabs, shown]).gap(16.0)
}

/// Someone's full profile over `root`, as Discord's: a rounded panel
/// centred on a 72 % scrim, 962 wide and up to 800 high, their card on the
/// left and the tabs beside it, the close button in its top right corner.
/// Escape and a click outside close it.
pub fn full_profile(root: Element, full: &FullProfile, window: Size, look: &Look) -> Element {
    let panel = window.width.min(FULL_PANEL);
    let tall = (window.height - 2.0 * FULL_EDGE).clamp(0.0, FULL_TALL);
    let beside = panel - 2.0 * FULL_EDGE - FULL_CARD - FULL_GUTTER;
    let body = if beside >= FULL_TABS_MIN {
        Element::row(vec![
            gap(FULL_EDGE, 0.0),
            ScrollArea::new(
                "pp:full:card",
                Element::column(vec![
                    gap(0.0, FULL_EDGE),
                    full_card(full, tall - FULL_EDGE, look),
                ]),
            )
            .appearance(look.scrollbar(false))
            .render()
            .size(Some(FULL_CARD), None),
            gap(FULL_GUTTER, 0.0),
            ScrollArea::new(
                "pp:full:side",
                Element::column(vec![
                    gap(0.0, FULL_EDGE),
                    full_tabs(full, beside, look),
                    gap(0.0, FULL_EDGE),
                ]),
            )
            .appearance(look.scrollbar(false))
            .render()
            .flex(1.0),
            gap(FULL_EDGE, 0.0),
        ])
    } else {
        // Too narrow for two columns: the tabs under the card, one scroll.
        ScrollArea::new(
            "pp:full:card",
            Element::column(vec![
                gap(0.0, FULL_EDGE),
                Element::row(vec![
                    Element::spacer().flex(1.0),
                    Element::column(vec![
                        full_card(full, 0.0, look),
                        full_tabs(full, FULL_CARD, look),
                    ])
                    .gap(FULL_GUTTER)
                    .size(Some(FULL_CARD), None),
                    Element::spacer().flex(1.0),
                ]),
                gap(0.0, FULL_EDGE),
            ]),
        )
        .appearance(look.scrollbar(false))
        .render()
    };
    let close = DismissButton::new(format!("{}:close", ids::FULL_CLOSE), look.menu_palette())
        .target_size(32.0)
        .dispatch_as(ids::FULL_CLOSE)
        .render();
    let close = Element::panel(close)
        .radius(8.0)
        .background(look.raised)
        .border(look.border, 1.0);
    let surface = Element::stack(vec![
        body,
        Element::column(vec![
            gap(0.0, 8.0),
            Element::row(vec![Element::spacer().flex(1.0), close, gap(8.0, 0.0)]),
        ]),
    ])
    .size(Some(panel), Some(tall))
    .radius(FULL_RADIUS)
    .background(Look::alpha(look.canvas, 255))
    .border(look.border, 1.0)
    .clip_children()
    .accessible(
        limn::Access::new(limn::Role::Dialog)
            .id(format!("{}:dialog", ids::FULL_CLOSE))
            .name(t("Perfil").to_owned()),
    );
    // The close button takes the focus, as the dialog it replaces gave it.
    Element::popover(root, surface, ids::FULL_CLOSE, PopoverPlacement::Center)
        .popover_initial_focus(format!("{}:close", ids::FULL_CLOSE))
        .popover_scrim(Color::rgba(0, 0, 0, 184))
        .popover_backdrop_dismiss(true)
}

/// The account's own popout.
#[derive(Clone, Debug, PartialEq)]
pub struct OwnPopout {
    pub profile: Profile,
    /// Until when the chosen status lasts, as a person reads it: "hasta
    /// mañana a las 08:00". `None` while it lasts until changed.
    pub until: Option<String>,
    /// Why the custom status cannot be set, when it cannot.
    pub custom_denied: Option<String>,
    /// Why the profile cannot be changed, when it cannot.
    pub edit_denied: Option<String>,
    /// The menu of statuses is open beside the status row.
    pub status_open: bool,
    /// Developer mode is on: the account's ID can be copied.
    pub copy_id: bool,
    /// The accounts «Cambiar de cuenta» lists, the open one first; `None`
    /// when the provider remembers no accounts.
    pub switch_accounts: Option<Vec<SwitchItem>>,
}

/// An account «Cambiar de cuenta» lists.
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchItem {
    pub id: UserId,
    pub name: String,
    pub picture: Option<limn::RasterImage>,
    /// The open account: marked, and choosing it only closes the menu.
    pub active: bool,
}

/// A row of the popout's groups, as Discord's: 32 px, a 16 px icon, the label
/// 14 px medium in the muted ink, and what goes after it.
fn action_row(id: &str, glyph: Element, label: &str, after: Vec<Element>, look: &Look) -> Element {
    let mut parts = vec![
        Element::row(vec![crate::centered(glyph)])
            .size(Some(16.0), Some(16.0))
            .centered_y(),
        line(label, 14.0, look.muted)
            .font_family(FontFamily::SansMedium)
            .shrink_to_fit()
            .centered_y(),
    ];
    parts.extend(after.into_iter().map(Element::centered_y));
    Element::clickable(id, Element::row(parts).gap(8.0).padding(8.0))
        .size(None, Some(32.0))
        .radius(8.0)
        .hover_background(look.hover)
        .accessible(limn::Access::new(limn::Role::MenuItem).name(label.to_owned()))
}

/// An icon of a popout's rows: 16 px in the muted ink.
fn row_icon(glyph: Icon, look: &Look) -> Element {
    icon(glyph, look.muted).size(Some(16.0), Some(16.0))
}

/// Rows on Discord's card (#17181b), a hairline 8 under and over each
/// between two, 4 in from the rows' sides.
fn group(rows: Vec<Element>, look: &Look) -> Element {
    let mut children = Vec::new();
    for row in rows {
        if !children.is_empty() {
            children.push(Element::row(vec![
                gap(4.0, 0.0),
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(hairline(look, 51))
                    .flex(1.0),
                gap(4.0, 0.0),
            ]));
        }
        children.push(row);
    }
    Element::panel(Element::column(children).gap(8.0).padding(8.0))
        .radius(8.0)
        .background(raised(look, 16))
}

/// The surface of a menu beside a row.
fn menu_surface(rows: Vec<Element>, width: f32, look: &Look) -> Element {
    Element::panel(Element::column(rows).gap(2.0).menu_navigation())
        .padding(6.0)
        .size(Some(width), None)
        .background(look.menu)
        .border(look.border, 1.0)
        .radius(8.0)
        .shadow(look.shadow, Point::new(0.0, 9.0), 22.0, 0.0)
}

const ACCOUNT_MENU_WIDTH: f32 = 240.0;

/// The accounts beside «Cambiar de cuenta»: each with its picture and name,
/// the open one checked, and «Gestionar cuentas» under a line.
fn account_menu(accounts: &[SwitchItem], look: &Look) -> Element {
    let mut rows = Vec::new();
    for account in accounts {
        let mut parts = vec![
            crate::avatar_with_image(&account.name, look.accent, 24.0, account.picture.clone())
                .centered_y(),
            line(account.name.clone(), 14.0, look.text)
                .flex(1.0)
                .centered_y(),
        ];
        if account.active {
            parts.push(icon(Icon::Check, look.accent).centered_y());
        }
        rows.push(menu_item(
            format!("{}{}", ids::ACCOUNT, account.id.0),
            parts,
            &account.name,
            account.active,
            look,
        ));
    }
    rows.push(
        Element::panel(Element::spacer())
            .size(None, Some(1.0))
            .background(look.border),
    );
    let manage = t("Gestionar cuentas");
    rows.push(menu_item(
        ids::MANAGE_ACCOUNTS.to_owned(),
        vec![line(manage, 14.0, look.text).flex(1.0).centered_y()],
        manage,
        false,
        look,
    ));
    menu_surface(rows, ACCOUNT_MENU_WIDTH, look)
}

/// One row of a popout's menu: its parts in a line, hovered as the others.
fn menu_item(id: String, parts: Vec<Element>, name: &str, selected: bool, look: &Look) -> Element {
    Element::clickable(id, Element::row(parts).gap(8.0).padding(8.0))
        .size(None, Some(40.0))
        .radius(4.0)
        .hover_background(look.hover)
        .accessible(
            limn::Access::new(limn::Role::MenuItem)
                .name(name.to_owned())
                .selected(selected),
        )
}

/// The menu of statuses, opened beside the popout's status row: each status
/// but online opens how long it lasts while the pointer is over it.
fn status_menu(current: Status, look: &Look) -> Element {
    let mut rows = Vec::new();
    for (status, key, description) in STATUSES {
        let label = crate::look::status_label(status);
        let mut copy = vec![line(label, 15.0, look.text)];
        if let Some(description) = description {
            copy.push(text(t(description), 12.0, look.muted));
        }
        let mut parts = vec![
            Element::row(vec![crate::centered(status_mark(
                status, look, 10.0, look.menu,
            ))])
            .size(Some(16.0), Some(16.0))
            .centered_y(),
            Element::column(copy).gap(2.0).flex(1.0).centered_y(),
        ];
        if current == status {
            parts.push(icon(Icon::Check, look.accent).centered_y());
        }
        if status != Status::Online {
            parts.push(icon(Icon::ChevronRight, look.muted).centered_y());
        }
        let row = Element::clickable(
            format!("{}{key}", ids::STATUS),
            Element::row(parts).gap(10.0).padding(8.0),
        )
        // A description may take two lines: the row grows with it.
        .min_size(0.0, if description.is_some() { 56.0 } else { 40.0 })
        .radius(4.0)
        .hover_background(look.hover)
        .accessible(
            limn::Access::new(limn::Role::MenuItem)
                .name(label)
                .selected(current == status),
        );
        if status == Status::Online {
            rows.push(row);
            rows.push(
                Element::panel(Element::spacer())
                    .size(None, Some(1.0))
                    .background(look.border),
            );
            continue;
        }
        let durations = DURATIONS
            .iter()
            .map(|(duration, label, _)| {
                DropdownMenuEntry::Item(DropdownMenuItem::new(
                    format!("{}{key}:{duration}", ids::STATUS),
                    t(label),
                ))
            })
            .chain([DropdownMenuEntry::Item(DropdownMenuItem::new(
                format!("{}{key}:forever", ids::STATUS),
                t("Para siempre"),
            ))])
            .collect();
        rows.push(
            DropdownMenu::new(
                row,
                durations,
                true,
                format!("{}{key}", ids::SUBMENU),
                look.menu_palette(),
            )
            .width(DURATION_MENU_WIDTH)
            .placement(PopoverPlacement::RightStart)
            .render()
            .popover_on_hover(),
        );
    }
    menu_surface(rows, STATUS_MENU_WIDTH, look)
}

/// The account's popout over its panel, as Discord's: its card's head with a
/// bubble to set the custom status, its name and bio, then a card of
/// «Editar perfil» and the status with its menu, and one of switching
/// accounts and copying its id.
pub fn own_popout(popout: &OwnPopout, look: &Look) -> Element {
    let profile = &popout.profile;
    let action = popout.custom_denied.is_none().then_some(ids::CUSTOM);
    let bubble = status_bubble(profile, action, t("Establecer estado personalizado"), look);
    let mut body = vec![card_names(profile, None, true, look)];
    // Its own bio whole: there is no full profile of one's own to lead to.
    body.extend(bio(profile, false, WIDTH - 2.0 * INSET, look));
    let edit = action_row(
        ids::EDIT,
        row_icon(Icon::Edit, look),
        if popout.edit_denied.is_some() {
            t("Ver perfil")
        } else {
            t("Editar perfil")
        },
        Vec::new(),
        look,
    );
    let edit = match &popout.edit_denied {
        Some(reason) => Tooltip::new(edit, ids::EDIT, reason.clone(), look.menu_palette()).render(),
        None => edit,
    };
    let mut after = Vec::new();
    if profile.status == Status::Invisible {
        let warning = row_icon(Icon::WarningTriangle, look).foreground(look.idle);
        after.push(
            Tooltip::new(
                Element::panel(warning).accessibility_label(t("Atención")),
                "pp:invisible-warning",
                t("Mientras estés invisible, los demás no ven tu estado ni tu actividad."),
                look.menu_palette(),
            )
            .render(),
        );
    }
    after.push(Element::spacer().flex(1.0));
    if let Some(until) = &popout.until {
        after.push(line(until.clone(), 12.0, look.muted));
    }
    after.push(row_icon(Icon::ChevronRight, look));
    let status = action_row(
        ids::STATUS_MENU,
        status_mark(profile.status, look, 12.0, raised(look, 16)),
        crate::look::status_label(profile.status),
        after,
        look,
    );
    // The pointer or the focus on the row opens the status menu, and keeps it
    // while the durations nested in it are used; a click keeps it open.
    let status = Element::popover(
        status,
        status_menu(profile.status, look),
        ids::STATUS_MENU_DISMISS,
        PopoverPlacement::RightStart,
    );
    let status = if popout.status_open {
        status
    } else {
        status.popover_on_hover()
    };
    let mut actions = vec![edit];
    if profile.presence {
        actions.push(status);
    }
    body.push(group(actions, look));
    let mut account = Vec::new();
    if let Some(accounts) = &popout.switch_accounts {
        let row = action_row(
            ids::SWITCH_ACCOUNT,
            row_icon(Icon::Custom(crate::user_settings::PERSON), look),
            t("Cambiar de cuenta"),
            vec![
                Element::spacer().flex(1.0),
                row_icon(Icon::ChevronRight, look),
            ],
            look,
        );
        // The pointer on the row opens its accounts, as on the status row.
        account.push(
            Element::popover(
                row,
                account_menu(accounts, look),
                ids::ACCOUNT_MENU_DISMISS,
                PopoverPlacement::RightStart,
            )
            .popover_on_hover(),
        );
    }
    if popout.copy_id {
        account.push(action_row(
            ids::COPY_ID,
            row_icon(Icon::Copy, look),
            t("Copiar ID del usuario"),
            Vec::new(),
            look,
        ));
    }
    if !account.is_empty() {
        body.push(group(account, look));
    }
    // Discord's spacing: the name 12 under the head, 16 at the sides, 20 under
    // the last card.
    Element::column(vec![
        card_head(profile, bubble, look),
        Element::column(vec![
            gap(0.0, 12.0),
            Element::row(vec![
                gap(INSET, 0.0),
                Element::column(body).gap(12.0).flex(1.0),
                gap(INSET, 0.0),
            ]),
            gap(0.0, 20.0),
        ]),
    ])
}

/// An audio device as the lists show it.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub id: String,
    pub name: String,
    /// The one the system uses unless told otherwise.
    pub default: bool,
}

/// The custom status dialog.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomStatusDialog {
    /// The account, as the preview shows it.
    pub profile: Profile,
    pub emoji: Option<String>,
    pub text: String,
    pub emoji_open: bool,
    pub clear_after: ClearAfter,
    pub expiry_open: bool,
    /// The wall clock, for when the status goes.
    pub now: Timestamp,
}

/// The emoji and the text as one line: what the status will say.
pub fn custom_line(emoji: Option<&str>, text: &str) -> Option<String> {
    let text = text.trim();
    match (emoji, text.is_empty()) {
        (Some(emoji), true) => Some(emoji.to_owned()),
        (Some(emoji), false) => Some(format!("{emoji} {text}")),
        (None, true) => None,
        (None, false) => Some(text.to_owned()),
    }
}

/// Why `line` cannot be the custom status, when it cannot.
pub fn custom_error(line: Option<&str>) -> Option<String> {
    line.filter(|line| line.chars().count() > CUSTOM_MAX)
        .map(|_| {
            tr!(
                "El estado puede tener hasta {max} caracteres.",
                max = CUSTOM_MAX
            )
        })
}

/// The dialog that sets the custom status over `root`: a preview whose
/// bubble says what is written, the field with its emoji, when it goes,
/// and saving.
pub fn custom_status_dialog(
    root: Element,
    dialog: &CustomStatusDialog,
    window_width: f32,
    look: &Look,
) -> Element {
    let palette = look.menu_palette();
    let said = custom_line(dialog.emoji.as_deref(), &dialog.text);
    let error = custom_error(said.as_deref());
    let preview = {
        let bubble = bubble(
            said.as_deref().unwrap_or(t("¿Qué estás haciendo?")),
            None,
            said.is_none(),
            None,
            look,
        );
        Element::panel(Element::column(vec![
            head(&dialog.profile, Some(bubble), look),
            Element::column(vec![names(&dialog.profile, false, None, 20.0, look)]).padding(INSET),
        ]))
        .size(Some(WIDTH), None)
        .radius(12.0)
        .background(look.menu)
        .border(look.border, 1.0)
        .clip_children()
    };
    let face = match &dialog.emoji {
        Some(emoji) => Element::text(emoji.clone())
            .font_family(FontFamily::Sans)
            .font_size(20.0)
            .foreground(look.text),
        None => icon(Icon::Emoji, look.muted),
    };
    let emoji_button = Element::clickable(ids::CUSTOM_EMOJI, crate::centered(face))
        .size(Some(40.0), Some(40.0))
        .radius(20.0)
        .hover_background(look.hover)
        .accessible(limn::Access::new(limn::Role::Button).name(t("Elegir un emoji")));
    let emoji_button = Popover::new(
        emoji_button,
        emoji::picker(ids::CUSTOM_EMOJI_PICK, t("Emoji del estado"), look),
        dialog.emoji_open,
        ids::CUSTOM_EMOJI_DISMISS,
        palette,
    )
    .width(emoji::WIDTH + 12.0)
    .placement(PopoverPlacement::BottomStart)
    .render();
    let field = Element::row(vec![
        gap(4.0, 0.0),
        emoji_button.centered_y(),
        Element::text_input(
            ids::CUSTOM_TEXT,
            dialog.text.clone(),
            t("¿Qué estás haciendo?"),
        )
        .foreground(look.text)
        .font_family(FontFamily::Sans)
        .font_size(16.0)
        .padding(8.0)
        .accessibility_label(t("Estado"))
        .flex(1.0)
        .centered_y(),
    ])
    .size(None, Some(49.5));
    let field = Element::panel(field)
        .radius(8.0)
        .background(look.canvas)
        .border(
            if error.is_some() {
                look.danger
            } else {
                look.border
            },
            1.0,
        );
    let mut body = vec![
        Element::row(vec![
            Element::spacer().flex(1.0),
            preview,
            Element::spacer().flex(1.0),
        ]),
        gap(0.0, 12.0),
        line(t("Estado"), 16.0, look.text),
        field,
    ];
    if let Some(error) = &error {
        body.push(text(error.clone(), 13.0, look.danger));
    }
    let when = match dialog.clear_after.minutes() {
        Some(minutes) => tr!(
            "Eliminar {when}",
            when = until_label(dialog.now.plus_minutes(minutes), dialog.now)
        ),
        None => t("No eliminar").to_owned(),
    };
    let expiry = Element::clickable(
        ids::CUSTOM_EXPIRY,
        Element::row(vec![
            line(when, 16.0, look.text).shrink_to_fit().centered_y(),
            icon(Icon::ChevronDown, look.text).centered_y(),
        ])
        .gap(8.0)
        .padding(4.0),
    )
    .radius(4.0)
    .hover_background(look.hover)
    .accessible(limn::Access::new(limn::Role::Button).name(t("Borrar después de")));
    let mut choices = vec![DropdownMenuEntry::Heading(t("Eliminar después").into())];
    choices.extend(ClearAfter::ALL.into_iter().map(|choice| {
        DropdownMenuEntry::Item(
            DropdownMenuItem::new(
                format!("{}{}", ids::CUSTOM_EXPIRY_PICK, choice.key()),
                choice.label(dialog.now),
            )
            .selected(choice == dialog.clear_after),
        )
    }));
    let expiry = DropdownMenu::new(
        expiry,
        choices,
        dialog.expiry_open,
        ids::CUSTOM_EXPIRY_DISMISS,
        palette,
    )
    .width(EXPIRY_MENU_WIDTH)
    .placement(PopoverPlacement::BottomStart)
    .render();
    let footer = Element::row(vec![
        expiry.centered_y(),
        Element::spacer().flex(1.0),
        Button::new(ids::CUSTOM_CANCEL, t("Cancelar"), palette)
            .variant(ButtonVariant::Link)
            .render()
            .centered_y(),
        Button::new(ids::CUSTOM_SAVE, t("Guardar"), look.accent_palette())
            .width(106.0)
            .enabled(error.is_none())
            .render()
            .centered_y(),
    ])
    .gap(12.0)
    .size(None, Some(42.0));
    Modal::new(
        root,
        true,
        t("Establecer tu estado"),
        "",
        ids::CUSTOM_CLOSE,
        palette,
    )
    .width(Modal::fit_width(window_width, DIALOG_WIDTH))
    .body(Element::column(body).gap(8.0))
    .footer(footer)
    .initial_focus(ids::CUSTOM_TEXT)
    .render()
}

/// What the profile editor shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Editor {
    /// The account as the preview shows it, with what was changed here.
    pub preview: Profile,
    /// What the fields hold.
    pub name: String,
    pub pronouns: String,
    pub about: String,
    pub accent: Rgb,
    /// The custom color field, while open.
    pub hex: Option<String>,
    /// Why the provider refused the last change.
    pub failure: Option<String>,
    /// Why the profile cannot be changed at all, when it cannot.
    pub denied: Option<String>,
    /// The servers, for the name the account goes by in each.
    pub servers: Vec<EditorServer>,
    pub servers_open: bool,
    /// Whether the per-server tab is the open one.
    pub servers_tab: bool,
    /// The bar of unsaved changes, while something differs.
    pub bar: Option<UnsavedBar>,
    /// The avatar and the banner, and what may be done with them.
    pub pictures: EditorPictures,
    /// The server whose profile is edited; `None` for the one shown everywhere.
    pub scope: Option<EditorScope>,
    /// Why the username and the email are not changed here, said beside them.
    pub credentials: Option<String>,
    /// Where the color picker's thumbs stand while it is used.
    pub picked: Option<limn::Hsv>,
}

/// The pictures of the profile being edited and what may be done with them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorPictures {
    /// The banner as it would show; without it, its color.
    pub banner: Option<limn::RasterImage>,
    /// Whether there is one to take away.
    pub has_avatar: bool,
    pub has_banner: bool,
    /// The picture being chosen: the avatar (`Some(true)`) or the banner.
    pub picking: Option<bool>,
    /// Why they cannot change, when they cannot.
    pub denied: Option<String>,
    /// A line under them, as the simulation's.
    pub note: Option<String>,
    /// Why a new banner cannot be chosen, as the plan says; one there may
    /// still be taken away.
    pub banner_denied: Option<String>,
    /// The picture last chosen, to frame it.
    pub framing: Option<EditorFraming>,
}

/// A picture chosen for the profile, whole, and the part of it kept. All
/// shares are of the picture's width and height.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorFraming {
    pub image: limn::RasterImage,
    /// Its width over its height.
    pub aspect: f32,
    /// The part kept: left, top, width and height.
    pub frame: [f32; 4],
    /// Where the frame's middle is asked to be.
    pub x: f32,
    pub y: f32,
    /// How close, from 0 (the whole frame that fits) to 1.
    pub zoom: f32,
    /// An avatar's frame: it shows round.
    pub round: bool,
    /// Whether it moves and stays moving, or why not.
    pub note: Option<String>,
}

/// The server whose profile is edited.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorScope {
    /// Its name.
    pub server: String,
    /// The name the account goes by there.
    pub nick: String,
    /// Why that name cannot change, when it cannot.
    pub nick_denied: Option<String>,
    /// Why «Sobre mí» there cannot change, when it cannot.
    pub about_denied: Option<String>,
}

/// What is wrong with the editor's fields, by field: each blocks saving.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorErrors {
    pub name: Option<String>,
    pub pronouns: Option<String>,
    pub about: Option<String>,
}

impl EditorErrors {
    /// The errors of fields holding these values.
    pub fn of(name: &str, pronouns: &str, about: &str) -> Self {
        let too_long = |value: &str, max: usize| value.trim().chars().count() > max;
        Self {
            name: too_long(name, NAME_MAX).then(|| {
                tr!(
                    "El nombre puede tener hasta {max} caracteres.",
                    max = NAME_MAX
                )
            }),
            pronouns: too_long(pronouns, PRONOUNS_MAX).then(|| {
                tr!(
                    "Los pronombres pueden tener hasta {max} caracteres.",
                    max = PRONOUNS_MAX
                )
            }),
            about: too_long(about, ABOUT_MAX).then(|| {
                tr!(
                    "«Sobre mí» puede tener hasta {max} caracteres.",
                    max = ABOUT_MAX
                )
            }),
        }
    }

    pub fn any(&self) -> bool {
        self.name.is_some() || self.pronouns.is_some() || self.about.is_some()
    }
}

/// A server of the profile choice, marked as the rail marks it.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorServer {
    /// What changes the name the account goes by there.
    pub action: String,
    pub name: String,
    pub nick: Option<String>,
    /// Its icon, once loaded; its initials on `accent` without one.
    pub image: Option<limn::RasterImage>,
    pub initials: String,
    pub accent: Color,
}

/// A server's icon in a menu row: its picture, or its first initial on its
/// color, rounded as the rail rounds it.
fn server_mark(server: &EditorServer) -> Element {
    match &server.image {
        Some(image) => Element::raster_image(image.clone().cover()).radius(5.0),
        None => Element::panel(crate::centered(
            Element::text(
                server
                    .initials
                    .chars()
                    .next()
                    .map(String::from)
                    .unwrap_or_default(),
            )
            .font_family(limn::FontFamily::SansMedium)
            .font_size(9.0)
            .foreground(Color::rgb(255, 255, 255)),
        ))
        .radius(5.0)
        .background(server.accent),
    }
}

/// A form section as Discord's settings lay it out: its title (with the
/// «Nitro» pill beside it when asked, and greyed when `faint`), its
/// description lines, then its controls 16 px under them.
fn form_section(
    title: &str,
    pill: bool,
    faint: bool,
    description: Vec<String>,
    body: Vec<Element>,
    width: f32,
    look: &Look,
) -> Element {
    let mut head =
        vec![strong_line(title, 16.0, if faint { look.faint } else { look.text }).centered_y()];
    if pill {
        head.push(nitro_pill(look));
    }
    let mut top = vec![Element::row(head).gap(8.0)];
    top.extend(
        description
            .into_iter()
            .map(|sentence| text(sentence, 14.0, look.muted).size(Some(width), None)),
    );
    Element::column(vec![
        Element::column(top).gap(8.0),
        Element::column(body).gap(8.0),
    ])
    .gap(16.0)
}

/// The «Nitro» mark of the settings that need it.
fn nitro_pill(look: &Look) -> Element {
    Element::row(vec![
        gap(6.0, 0.0),
        line(t("Nitro"), 11.0, look.muted).centered_y(),
        gap(6.0, 0.0),
    ])
    .size(None, Some(18.0))
    .radius(9.0)
    .background(look.raised)
    .centered_y()
}

/// A control with what is said under it: an error, or a reason it is off.
fn with_note(control: Element, note: Option<(String, Color)>, width: f32) -> Element {
    match note {
        Some((note, color)) => Element::column(vec![
            control,
            text(note, 13.0, color).size(Some(width), None),
        ])
        .gap(8.0),
        None => control,
    }
}

/// The count of «Sobre mí», in the field's corner, red past its maximum.
fn about_field(
    id: &str,
    value: &str,
    hint: &str,
    label: &str,
    height: f32,
    look: &Look,
) -> Element {
    let count = value.trim().chars().count();
    Element::column(vec![
        TextArea::new(id, value.to_owned(), hint.to_owned(), look.palette())
            .accessibility_label(label.to_owned())
            .height(height)
            .render(),
        Element::row(vec![
            Element::spacer().flex(1.0),
            line(
                format!("{count}/{ABOUT_MAX}"),
                12.0,
                if count > ABOUT_MAX {
                    look.danger
                } else {
                    look.muted
                },
            ),
        ]),
    ])
    .gap(4.0)
}

/// «Cambiar» and «Quitar» of one picture: the first chooses a file, loading
/// while the selector is open; the second shows only with one to take away.
fn picture_actions(
    pick: &'static str,
    remove: &'static str,
    has: bool,
    picking: bool,
    pictures: &EditorPictures,
    look: &Look,
) -> Element {
    let avatar = pick == ids::AVATAR_PICK;
    let (change, take) = if avatar {
        (t("Cambiar avatar"), t("Quitar avatar"))
    } else {
        (t("Cambiar banner"), t("Quitar banner"))
    };
    let can = pictures.denied.is_none();
    let chooses = avatar || pictures.banner_denied.is_none();
    let mut actions = vec![Button::new(pick, change, look.accent_palette())
        .enabled(can && chooses && pictures.picking.is_none())
        .loading(picking)
        .render()];
    if has {
        actions.push(
            Button::new(remove, take, look.menu_palette())
                .variant(ButtonVariant::Link)
                .enabled(can && pictures.picking.is_none())
                .render(),
        );
    }
    Element::row(actions).gap(12.0)
}

/// The avatar: its buttons, the formats, what limits it, and the framing
/// while an avatar is being framed.
fn avatar_section(view: &Editor, width: f32, look: &Look) -> Element {
    let pictures = &view.pictures;
    let hint = if view.scope.is_some() {
        t("Se muestran en este servidor en lugar de los de tu cuenta. PNG, JPEG, GIF o WebP de hasta 8 MB; después de elegirla, la encuadras.")
    } else {
        t("PNG, JPEG, GIF o WebP de hasta 8 MB: el avatar cuadrado, el banner en 5:2. Después de elegirla, la encuadras.")
    };
    let mut description = vec![hint.to_owned()];
    description.extend(pictures.denied.clone());
    description.extend(pictures.note.clone());
    let mut body = vec![picture_actions(
        ids::AVATAR_PICK,
        ids::AVATAR_REMOVE,
        pictures.has_avatar,
        pictures.picking == Some(true),
        pictures,
        look,
    )];
    if let Some(framing) = pictures.framing.as_ref().filter(|framing| framing.round) {
        body.push(editor_framing(framing, width, look));
    }
    form_section(t("Avatar"), false, false, description, body, width, look).anchor(ids::PICTURES)
}

/// The banner: its buttons, the reason it may not be chosen, and the
/// framing while a banner is being framed.
fn banner_section(view: &Editor, width: f32, look: &Look) -> Element {
    let pictures = &view.pictures;
    let mut body = vec![picture_actions(
        ids::BANNER_PICK,
        ids::BANNER_REMOVE,
        pictures.has_banner,
        pictures.picking == Some(false),
        pictures,
        look,
    )];
    if let Some(framing) = pictures.framing.as_ref().filter(|framing| !framing.round) {
        body.push(editor_framing(framing, width, look));
    }
    let reason = pictures.banner_denied.clone();
    form_section(
        t("Banner del perfil"),
        view.scope.is_none(),
        reason.is_some(),
        reason.into_iter().collect(),
        body,
        width,
        look,
    )
}

/// A profile setting Rustcord cannot change yet: greyed, with a disabled button.
fn unavailable_section(
    title: &str,
    button: &'static str,
    label: &str,
    width: f32,
    look: &Look,
) -> Element {
    form_section(
        title,
        true,
        true,
        vec![t("Rustcord todavía no puede cambiar esto.").to_owned()],
        // In a row, so the button keeps its own width.
        vec![Element::row(vec![Button::new(
            button,
            label,
            look.menu_palette(),
        )
        .variant(ButtonVariant::Secondary)
        .enabled(false)
        .render()])],
        width,
        look,
    )
}

/// The fields of the account's own profile, in Discord's order.
fn main_sections(view: &Editor, width: f32, look: &Look) -> Vec<Element> {
    let palette = look.palette();
    let errors = EditorErrors::of(&view.name, &view.pronouns, &view.about);
    let name = TextField::new(
        ids::NAME,
        view.name.clone(),
        view.preview.handle.clone(),
        palette,
    )
    .height(40.0)
    .accessibility_label(t("Nombre para mostrar"))
    .render();
    let pronouns = TextField::new(
        ids::PRONOUNS,
        view.pronouns.clone(),
        t("Agrega tus pronombres"),
        palette,
    )
    .height(40.0)
    .accessibility_label(t("Pronombres"))
    .render();
    let about = about_field(
        ids::ABOUT,
        &view.about,
        t("Cuenta algo sobre ti."),
        t("Biografía"),
        136.0,
        look,
    );
    vec![
        form_section(
            t("Nombre para mostrar"),
            false,
            false,
            Vec::new(),
            vec![with_note(
                name,
                errors.name.clone().map(|note| (note, look.danger)),
                width,
            )],
            width,
            look,
        ),
        form_section(
            t("Pronombres"),
            false,
            false,
            Vec::new(),
            vec![with_note(
                pronouns,
                errors.pronouns.clone().map(|note| (note, look.danger)),
                width,
            )],
            width,
            look,
        ),
        avatar_section(view, width, look),
        unavailable_section(
            t("Decoración de avatar"),
            ids::DECORATION,
            t("Cambiar decoración"),
            width,
            look,
        ),
        unavailable_section(
            t("Placa de nombre"),
            ids::NAMEPLATE,
            t("Cambiar placa de nombre"),
            width,
            look,
        ),
        unavailable_section(
            t("Efecto de perfil"),
            ids::EFFECT,
            t("Cambiar efecto"),
            width,
            look,
        ),
        unavailable_section(
            t("Marco de perfil"),
            ids::PROFILE_FRAME,
            t("Cambiar marco"),
            width,
            look,
        ),
        banner_section(view, width, look),
        form_section(
            t("Color del banner"),
            false,
            false,
            Vec::new(),
            vec![parts::swatches(
                &Swatches {
                    prefix: ids::COLOR.into(),
                    chosen: Some(view.accent),
                    default: None,
                    custom: view.hex.clone(),
                    enabled: true,
                    narrow: width < parts::swatches_width(false),
                    picked: view.picked,
                },
                look,
            )],
            width,
            look,
        ),
        form_section(
            t("Biografía"),
            false,
            false,
            vec![t("Puedes usar Markdown y enlaces si quieres.").to_owned()],
            vec![with_note(
                about,
                errors.about.clone().map(|note| (note, look.danger)),
                width,
            )],
            width,
            look,
        ),
    ]
    .into_iter()
    .chain(view.credentials.as_ref().map(|reason| {
        // What the account signs in with, and why it cannot change here.
        form_section(
            t("Nombre de usuario"),
            false,
            false,
            vec![reason.clone()],
            vec![text(view.preview.handle.clone(), 15.0, look.text)],
            width,
            look,
        )
    }))
    .collect()
}

/// The name and «Sobre mí» the account has in one server, and its pictures.
fn scoped_sections(view: &Editor, scope: &EditorScope, width: f32, look: &Look) -> Vec<Element> {
    let palette = look.palette();
    let errors = EditorErrors::of(&scope.nick, "", &view.about);
    let nick = TextField::new(
        ids::NICK,
        scope.nick.clone(),
        view.preview.handle.clone(),
        palette,
    )
    .height(40.0)
    .enabled(scope.nick_denied.is_none())
    .accessibility_label(t("Apodo"))
    .render();
    let about = about_field(
        ids::ABOUT,
        &view.about,
        t("Cuenta algo para este servidor."),
        t("Sobre mí en este servidor"),
        136.0,
        look,
    );
    let about_note = scope
        .about_denied
        .clone()
        .map(|reason| (reason, look.muted))
        .or(errors.about.clone().map(|note| (note, look.danger)));
    vec![
        form_section(
            t("Apodo"),
            false,
            false,
            Vec::new(),
            vec![with_note(
                nick,
                errors
                    .name
                    .clone()
                    .map(|note| (note, look.danger))
                    .or(scope.nick_denied.clone().map(|reason| (reason, look.muted))),
                width,
            )],
            width,
            look,
        ),
        avatar_section(view, width, look),
        banner_section(view, width, look),
        form_section(
            t("Biografía"),
            false,
            false,
            vec![t("Puedes usar Markdown y enlaces si quieres.").to_owned()],
            vec![with_note(about, about_note, width)],
            width,
            look,
        ),
    ]
}

/// The read-only account: what is shown of the profile, without controls.
fn read_only_values(view: &Editor, look: &Look) -> Vec<Element> {
    vec![
        profile_value(t("Nombre para mostrar"), Some(&view.preview.name), look),
        profile_value(t("Nombre de usuario"), Some(&view.preview.handle), look),
        profile_value(t("Pronombres"), view.preview.pronouns.as_deref(), look),
        profile_value(t("Sobre mí"), view.preview.about.as_deref(), look),
        text(
            t("La edición de este perfil todavía no está disponible."),
            12.0,
            look.muted,
        ),
    ]
}

/// The sections, one line apart: a hairline between each two.
fn ruled(sections: Vec<Element>, look: &Look) -> Vec<Element> {
    let mut ruled = Vec::new();
    for section in sections {
        if !ruled.is_empty() {
            ruled.push(divider(look));
        }
        ruled.push(section);
    }
    ruled
}

/// The per-server tab's picker: the servers, and the one whose profile is
/// edited.
fn server_picker(view: &Editor, look: &Look) -> Element {
    let current = view.scope.as_ref().map_or_else(
        || t("Elige un servidor").to_owned(),
        |scope| scope.server.clone(),
    );
    let selector = Element::clickable(
        ids::SERVERS,
        Element::row(vec![
            strong_line(current, 16.0, look.text)
                .shrink_to_fit()
                .centered_y(),
            Element::spacer().flex(1.0),
            icon(Icon::ChevronDown, look.muted).centered_y(),
        ])
        .gap(6.0)
        .padding(10.0),
    )
    .radius(6.0)
    .background(look.raised)
    .hover_background(look.hover)
    .accessible(limn::Access::new(limn::Role::Button).name(t("Elegir servidor")));
    let mut profiles = vec![DropdownMenuEntry::Heading(
        t("Perfil en cada servidor").into(),
    )];
    profiles.extend(view.servers.iter().map(|server| {
        let item = DropdownMenuItem::new(server.action.clone(), server.name.clone())
            .leading(server_mark(server))
            .selected(
                view.scope
                    .as_ref()
                    .is_some_and(|scope| scope.server == server.name),
            );
        DropdownMenuEntry::Item(match &server.nick {
            Some(nick) => item.annotation(nick.clone()),
            None => item,
        })
    }));
    DropdownMenu::new(
        selector,
        profiles,
        view.servers_open,
        ids::SERVERS_DISMISS,
        look.menu_palette(),
    )
    .width(280.0)
    .render()
}

/// The tabs of the profile: the account's, and one per server when there are
/// servers.
fn profile_tabs(view: &Editor, look: &Look) -> Element {
    let tab = |id: &'static str, label: &str, active: bool| {
        // shortcut: widths estimated from the characters, as the full profile's tabs do.
        let width = label.chars().count() as f32 * 7.5 + 12.0;
        Tab::new(id, label.to_owned(), width).active(active)
    };
    let mut tabs = vec![tab(ids::TAB_MAIN, t("Perfil principal"), !view.servers_tab)];
    if !view.servers.is_empty() {
        tabs.push(tab(
            ids::TAB_SERVERS,
            t("Perfiles por servidor"),
            view.servers_tab,
        ));
    }
    Tabs::new(tabs, look.menu_palette())
        .variant(TabsVariant::Underlined)
        .accent(look.accent)
        .id("profile:tabs")
        .height(32.0)
        .gap(20.0)
        .render()
}

/// The form: what failed to save, the tab's picker, then its sections.
fn form(view: &Editor, width: f32, look: &Look) -> Element {
    let mut parts = Vec::new();
    if view.denied.is_some() {
        parts.extend(ruled(read_only_values(view, look), look));
    } else {
        if let Some(failure) = &view.failure {
            parts.push(
                text(
                    tr!("No se pudo guardar: {failure}", failure),
                    13.0,
                    look.danger,
                )
                .accessible(limn::Access::new(limn::Role::Alert).name(failure.clone())),
            );
        }
        if view.servers_tab {
            parts.push(server_picker(view, look));
            match &view.scope {
                Some(scope) => parts.extend(ruled(scoped_sections(view, scope, width, look), look)),
                None => parts.push(
                    text(
                        t("Elige un servidor para cambiar cómo te ven allí."),
                        14.0,
                        look.muted,
                    )
                    .size(Some(width), None),
                ),
            }
        } else {
            parts.extend(ruled(main_sections(view, width, look), look));
        }
    }
    Element::column(parts).gap(24.0).size(Some(width), None)
}

/// The preview: the card as the others see it, its footer a plain example.
fn preview_column(view: &Editor, look: &Look) -> Element {
    let can = view.denied.is_none();
    Element::column(vec![
        Element::text(if can { t("Vista previa") } else { t("Perfil") })
            .font_size(16.0)
            .font_family(FontFamily::SansMedium)
            .foreground(look.text),
        Element::panel(card(&view.preview, look))
            .size(Some(WIDTH), None)
            .radius(8.0)
            .background(look.menu)
            .border(look.border, 1.0)
            .clip_children(),
    ])
    .gap(8.0)
    .size(Some(WIDTH), None)
}

/// The profile of the settings' page, as Discord's: its tabs, then the form
/// beside a preview that stays where it is while the form scrolls. Narrower
/// than [`SIDE_BY_SIDE`], the preview comes under the form.
pub fn profiles_page(view: &Editor, size: Size, look: &Look) -> Element {
    let can = view.denied.is_none();
    let side_by_side = size.width >= SIDE_BY_SIDE;
    let form_width = if side_by_side {
        size.width - PREVIEW_GAP - WIDTH
    } else {
        size.width
    };
    let preview = preview_column(view, look);
    let content = if side_by_side {
        Element::row(vec![
            ScrollArea::new("profile:form", form(view, form_width, look))
                .render()
                .flex(1.0),
            gap(PREVIEW_GAP, 0.0),
            preview,
        ])
    } else {
        ScrollArea::new(
            "profile:form",
            Element::column(vec![form(view, form_width, look), gap(0.0, 32.0), preview]),
        )
        .render()
        .flex(1.0)
    };
    let mut rows = Vec::new();
    if can {
        rows.push(profile_tabs(view, look));
        rows.push(divider(look));
        rows.push(gap(0.0, 16.0));
    }
    rows.push(content.flex(1.0));
    if let Some(bar) = view.bar.filter(|_| can) {
        rows.push(gap(0.0, 12.0));
        rows.push(parts::unsaved_bar_for(ids::BAR, bar, look).size(Some(size.width), None));
        rows.push(gap(0.0, 16.0));
    }
    Element::column(rows).size(Some(size.width), Some(size.height))
}

/// The picture last chosen, whole, with the part kept framed over it:
/// dragging over it (or its arrows) moves the frame, the slider brings it
/// closer.
fn editor_framing(view: &EditorFraming, width: f32, look: &Look) -> Element {
    const TALLEST: f32 = 220.0;
    let aspect = if view.aspect.is_finite() && view.aspect > 0.0 {
        view.aspect
    } else {
        1.0
    };
    let shown_width = width.min(TALLEST * aspect).max(1.0);
    let shown_height = shown_width / aspect;
    let [left, top, wide, high] = view.frame;
    let (frame_width, frame_height) = (wide * shown_width, high * shown_height);
    let frame = Element::panel(Element::spacer())
        .size(Some(frame_width), Some(frame_height))
        .radius(if view.round {
            frame_width.min(frame_height) / 2.0
        } else {
            4.0
        })
        .border(look.accent, 2.0)
        .translate(Point::new(left * shown_width, top * shown_height));
    let picture = Element::clickable(
        ids::FRAME,
        Element::stack(vec![
            Element::raster_image(view.image.clone()).size(Some(shown_width), Some(shown_height)),
            // A column keeps the frame at its own size, from the corner.
            Element::column(vec![frame]),
        ])
        .size(Some(shown_width), Some(shown_height)),
    )
    .area_slider(view.x, view.y, 0.0)
    .radius(6.0)
    .clip_children()
    .accessible(
        limn::Access::new(limn::Role::Slider)
            .name(t("Encuadre"))
            .value(view.x)
            .value_text(format!(
                "{}%, {}%",
                (view.x * 100.0).round(),
                (view.y * 100.0).round()
            )),
    );
    let zoom = Slider::new(ids::FRAME_ZOOM, view.zoom, look.palette())
        .ink(look.accent)
        .accessibility_label(t("Zoom del encuadre"))
        .render();
    let mut parts = vec![
        strong_line(t("Encuadre"), 15.0, look.text),
        Element::row(vec![
            Element::spacer().flex(1.0),
            picture,
            Element::spacer().flex(1.0),
        ]),
        Element::row(vec![
            line(t("Zoom"), 13.0, look.muted).centered_y(),
            zoom.flex(1.0).centered_y(),
        ])
        .gap(12.0),
        text(
            t("Arrastra sobre la imagen para elegir qué parte se ve; el zoom la acerca."),
            12.0,
            look.muted,
        )
        .size(Some(width), None),
    ];
    if let Some(note) = &view.note {
        parts.push(text(note.clone(), 12.0, look.muted).size(Some(width), None));
    }
    Element::column(parts).gap(8.0)
}

/// Missing optional data means it was not supplied; it does not prove the user
/// left that field empty on the service.
fn profile_value(label: &str, value: Option<&str>, look: &Look) -> Element {
    let value = value.filter(|value| !value.trim().is_empty());
    Element::column(vec![
        strong_line(label, 13.0, look.muted),
        text(
            value.unwrap_or(t("No disponible")),
            15.0,
            if value.is_some() {
                look.text
            } else {
                look.faint
            },
        ),
    ])
    .gap(6.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{controls, frame, texts};

    const LOOK: Look = Look::DARK;

    fn ada() -> Profile {
        Profile {
            identity: Default::default(),
            image: None,
            avatar_playing: false,
            banner: None,
            banner_playing: false,
            banner_tint: None,
            theme: None,
            name: "Ada".into(),
            handle: "ada".into(),
            accent: Color::rgb(90, 120, 200),
            status: Status::Online,
            presence: true,
            custom: Some("leyendo".into()),
            custom_emoji: None,
            pronouns: Some("ella".into()),
            about: Some("Escribo programas.".into()),
            activity: Some("Jugando a Factorio".into()),
            roles: vec![
                ("Moderación".into(), Color::rgb(200, 80, 80)),
                ("Equipo".into(), Color::rgb(80, 200, 80)),
            ],
            bot: false,
            message_action: Some("profile:dm:10".into()),
            message: Some(MessageField {
                id: format!("{}10", ids::MESSAGE),
                text: String::new(),
            }),
            restriction: None,
            member_since: Vec::new(),
            mutual: None,
            full_action: None,
            note: None,
            loading: false,
            relation: None,
            more: None,
            open_menu: None,
            notice: None,
            mutual_friends: Mutual::Unknown,
            mutual_servers: Mutual::Unknown,
            role_removals: vec![None, None],
            role_menu: None,
            edit_profile: false,
            set_custom: false,
            example_button: false,
            games: Vec::new(),
            wishlist: Vec::new(),
        }
    }

    #[test]
    fn a_card_too_tall_for_the_window_scrolls_its_body_and_keeps_its_buttons() {
        let tall = Profile {
            roles: (0..40)
                .map(|n| (format!("Rol número {n}"), Color::rgb(80, 80, 200)))
                .collect(),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        // A popover is never taller than its window: the card gets less.
        let (_, laid) = frame(card(&tall, &LOOK), Size::new(WIDTH, 420.0));
        let body = laid
            .regions
            .iter()
            .find(|region| {
                region.id == ids::card_body("ada") && region.kind == limn::HitKind::Scroll
            })
            .expect("the body scrolls")
            .rect;
        let button = laid
            .regions
            .iter()
            .find(|region| region.id == format!("{}10", ids::MESSAGE))
            .expect("the field to write stays")
            .rect;
        assert!(body.y + body.height <= button.y, "{body:?} {button:?}");
        assert!(button.y + button.height <= 420.0, "inside the window");
        // With room, the whole card shows and nothing scrolls away.
        let (_, roomy) = frame(card(&ada(), &LOOK), Size::new(WIDTH, 2000.0));
        assert!(controls(&roomy).contains(&format!("{}10", ids::MESSAGE)));
    }

    #[test]
    fn each_persons_card_opens_at_the_top_of_its_body() {
        let tall = |name: &str, handle: &str| Profile {
            name: name.into(),
            handle: handle.into(),
            roles: (0..40)
                .map(|n| (format!("Rol número {n}"), Color::rgb(80, 80, 200)))
                .collect(),
            ..ada()
        };
        let size = Size::new(WIDTH, 420.0);
        let (mut tree, laid) = frame(card(&tall("Ada", "ada"), &LOOK), size);
        let body = laid
            .regions
            .iter()
            .find(|region| region.id == ids::card_body("ada"))
            .unwrap()
            .rect;
        tree.handle_event(limn::UiEvent::ScrollDirect {
            position: limn::Point::new(body.x + 20.0, body.y + 20.0),
            delta_y: -160.0,
        });
        tree.layout(size);
        assert!(tree.scroll_offset(&ids::card_body("ada")) > 0.0);
        // Straight from Ada's row to Bruno's: his opens at its top.
        tree.set_root(card(&tall("Bruno", "bruno"), &LOOK));
        let laid = tree.layout(size);
        assert!(laid
            .regions
            .iter()
            .any(|region| region.id == ids::card_body("bruno")));
        assert_eq!(tree.scroll_offset(&ids::card_body("bruno")), 0.0);
    }

    #[test]
    fn the_note_is_an_icon_that_opens_the_full_profile() {
        let note = |text: &str| NoteField {
            id: "pp:note:10".into(),
            text: text.into(),
            loading: false,
            denied: None,
        };
        let someone = Profile {
            full_action: Some(format!("{}10", ids::FULL)),
            note: Some(note("")),
            ..ada()
        };
        let (_, laid) = frame(card(&someone, &LOOK), Size::new(WIDTH, 900.0));
        let shown = controls(&laid);
        assert!(
            shown.contains(&format!("{}10", ids::FULL_NOTE)),
            "{shown:?}"
        );
        assert!(
            !shown.contains(&"pp:note:10".to_owned()),
            "no field on the card"
        );
        // Without a full profile to open, the field stays.
        let alone = Profile {
            note: Some(note("amiga")),
            ..ada()
        };
        let (_, laid) = frame(card(&alone, &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&"pp:note:10".to_owned()));
    }

    #[test]
    fn a_long_bio_shows_three_lines_on_the_card_and_leads_to_the_full_profile() {
        let long = Profile {
            about: Some("uno\ndos\ntres\ncuatro\ncinco".into()),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        let (_, laid) = frame(card(&long, &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&format!("{}10", ids::FULL_BIO)));
        let at = |wanted: &str| {
            laid.commands.iter().find_map(|command| match command {
                limn::DrawCommand::Text { text, position, .. } if text == wanted => {
                    Some(position.y)
                }
                _ => None,
            })
        };
        let (first, link) = (at("uno").unwrap(), at("Ver biografía completa").unwrap());
        assert!(
            link - first <= BIO_HEIGHT + 6.0,
            "the rest is cut under three lines: {first} {link}"
        );
        // A short one shows whole, with nothing more to see.
        let short = Profile {
            full_action: long.full_action.clone(),
            ..ada()
        };
        let (_, laid) = frame(card(&short, &LOOK), Size::new(WIDTH, 900.0));
        assert!(!texts(&laid).contains(&"Ver biografía completa".to_owned()));
    }

    #[test]
    fn a_card_shows_who_someone_is_their_roles_and_a_way_to_write() {
        let (_, laid) = frame(
            Element::row(vec![
                Element::column(vec![card(&ada(), &LOOK)]).size(Some(WIDTH), None),
                Element::spacer().flex(1.0),
            ]),
            Size::new(800.0, 700.0),
        );
        let painted = texts(&laid);
        for expected in [
            "Ada",
            "ada",
            "ella",
            "leyendo",
            "Escribo programas.",
            "Moderación",
            "Equipo",
        ] {
            assert!(
                painted.iter().any(|text| text == expected),
                "{expected}: {painted:?}"
            );
        }
        // The card has no headings for its bio or its roles.
        for gone in ["Sobre mí", "Roles"] {
            assert!(
                !painted.iter().any(|text| text == gone),
                "{gone}: {painted:?}"
            );
        }
        assert!(controls(&laid).contains(&format!("{}10", ids::MESSAGE)));
    }

    #[test]
    fn the_footer_writes_to_someone_with_their_name_or_edits_the_own_profile() {
        let (_, laid) = frame(card(&ada(), &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&format!("{}10", ids::MESSAGE)));
        assert!(texts(&laid).contains(&"Enviar un mensaje a @Ada".to_owned()));
        assert!(!controls(&laid).contains(&ids::EDIT.to_owned()));
        // The account's own card edits its profile instead, full width.
        let own = Profile {
            message: None,
            edit_profile: true,
            set_custom: true,
            example_button: false,
            games: Vec::new(),
            wishlist: Vec::new(),
            full_action: None,
            ..ada()
        };
        let (_, laid) = frame(card(&own, &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&ids::EDIT.to_owned()));
        assert!(texts(&laid).contains(&"Editar perfil".to_owned()));
        // Its bubble opens the custom status, and invites to set one.
        assert!(controls(&laid).contains(&ids::CUSTOM.to_owned()));
        let unset = Profile {
            custom: None,
            ..own
        };
        let (_, laid) = frame(card(&unset, &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&ids::CUSTOM.to_owned()));
        assert!(texts(&laid).contains(&"Añadir estado".to_owned()));
        // Neither where the account cannot write to them; someone else's
        // bubble is only read, and none without a status.
        let none = Profile {
            message: None,
            ..ada()
        };
        let (_, laid) = frame(card(&none, &LOOK), Size::new(WIDTH, 900.0));
        assert!(!controls(&laid)
            .iter()
            .any(|id| id.starts_with(ids::MESSAGE) || id == ids::EDIT || id == ids::CUSTOM));
        let quiet = Profile {
            custom: None,
            ..ada()
        };
        let (_, laid) = frame(card(&quiet, &LOOK), Size::new(WIDTH, 900.0));
        assert!(!texts(&laid).contains(&"Añadir estado".to_owned()));
    }

    #[test]
    fn the_bubble_hugs_a_short_status_and_wraps_a_long_one_inside_the_card() {
        let bubble = |custom: &str| {
            let own = Profile {
                custom: Some(custom.into()),
                set_custom: true,
                ..ada()
            };
            let (_, laid) = frame(card(&own, &LOOK), Size::new(WIDTH, 900.0));
            laid.regions
                .iter()
                .find(|region| region.id == ids::CUSTOM)
                .map(|region| region.rect)
                .expect("the bubble")
        };
        // 9 right of the 80 px mark, 16 in: as wide as its words, as Discord's.
        let short = bubble("leyendo");
        assert!(
            (short.x - 105.0).abs() < 1.0 && short.width < 100.0,
            "{short:?}"
        );
        let long = bubble(&"una línea bastante larga ".repeat(4));
        assert!(
            long.x + long.width <= WIDTH - INSET + 0.5 && long.height > short.height,
            "{long:?}"
        );
    }

    #[test]
    fn a_card_shows_its_restriction_and_mutuals_but_not_the_dates() {
        let profile = Profile {
            restriction: Some("Aislado hasta hoy a las 18:30".into()),
            member_since: vec![Since {
                scope: "Cuenta".into(),
                face: None,
                date: "3 de mayo de 2019".into(),
            }],
            mutual_friends: Mutual::Known(vec![taller(), taller()]),
            mutual_servers: Mutual::Known(vec![taller()]),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        let (_, laid) = frame(
            Element::row(vec![
                Element::column(vec![card(&profile, &LOOK)]).size(Some(WIDTH), None),
                Element::spacer().flex(1.0),
            ]),
            Size::new(800.0, 900.0),
        );
        let painted = texts(&laid);
        for expected in [
            "Aislado hasta hoy a las 18:30",
            "2 amigos en común",
            "1 servidor en común",
        ] {
            assert!(
                painted.iter().any(|t| t == expected),
                "{expected}: {painted:?}"
            );
        }
        // The dates are the full profile's, not the card's.
        for gone in ["Miembro desde", "3 de mayo de 2019"] {
            assert!(!painted.iter().any(|t| t == gone), "{gone}: {painted:?}");
        }
        let shown = controls(&laid);
        assert!(
            shown.contains(&format!("{}10:friends", ids::MUTUAL)),
            "{shown:?}"
        );
        assert!(
            shown.contains(&format!("{}10:servers", ids::MUTUAL)),
            "{shown:?}"
        );
    }

    #[test]
    fn mutuals_read_12_px_beside_each_other_and_14_px_alone_and_nothing_unknown() {
        let size_of = |profile: &Profile, wanted: &str| {
            let (_, laid) = frame(card(profile, &LOOK), Size::new(WIDTH, 900.0));
            laid.commands.iter().find_map(|command| match command {
                limn::DrawCommand::Text {
                    text, font_size, ..
                } if text == wanted => Some(*font_size),
                _ => None,
            })
        };
        let both = Profile {
            mutual_friends: Mutual::Known(vec![taller(), taller()]),
            mutual_servers: Mutual::Known(vec![taller()]),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        assert_eq!(size_of(&both, "2 amigos en común"), Some(12.0));
        let friends = Profile {
            mutual_servers: Mutual::Known(Vec::new()),
            ..both.clone()
        };
        assert_eq!(size_of(&friends, "2 amigos en común"), Some(14.0));
        // Unknown or loading mutuals show nothing, and none is no mutual.
        for mutual in [Mutual::Unknown, Mutual::Loading, Mutual::Known(Vec::new())] {
            let quiet = Profile {
                mutual_friends: mutual.clone(),
                mutual_servers: mutual,
                ..both.clone()
            };
            let (_, laid) = frame(card(&quiet, &LOOK), Size::new(WIDTH, 900.0));
            assert!(!controls(&laid).iter().any(|id| id.starts_with(ids::MUTUAL)));
        }
    }

    #[test]
    fn a_card_head_has_the_friendship_and_more_buttons_as_their_states_allow() {
        let relation = |state, action: &str, menu: Vec<MenuEntry>| Relation {
            state,
            action: action.into(),
            menu,
        };
        let shown = |profile: &Profile| {
            let (_, laid) = frame(card(profile, &LOOK), Size::new(WIDTH, 900.0));
            controls(&laid)
        };
        // Someone without a friendship: the button sends the request.
        let stranger = Profile {
            relation: Some(relation(Friendship::None, "friend:10:add", Vec::new())),
            more: Some((format!("{}10", ids::CARD_MORE), Vec::new())),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        let buttons = shown(&stranger);
        assert!(buttons.contains(&"friend:10:add".to_owned()), "{buttons:?}");
        assert!(
            buttons.contains(&format!("{}10", ids::CARD_MORE)),
            "{buttons:?}"
        );
        // A friend's button opens the menu of the friendship, when it is open.
        let friend = Profile {
            relation: Some(relation(
                Friendship::Friend,
                &format!("{}10", ids::CARD_RELATION),
                vec![MenuEntry::Item(
                    "friend:10:remove".into(),
                    "Eliminar amigo".into(),
                    None,
                )],
            )),
            open_menu: Some(CardMenu::Relation),
            ..stranger.clone()
        };
        let (_, laid) = frame(card(&friend, &LOOK), Size::new(WIDTH, 900.0));
        assert!(controls(&laid).contains(&"friend:10:remove".to_owned()));
        assert!(texts(&laid).contains(&"Eliminar amigo".to_owned()));
        // Nothing of the buttons on one's own card, bots or blocked people.
        let own = Profile {
            relation: None,
            more: None,
            full_action: None,
            ..ada()
        };
        assert!(!shown(&own)
            .iter()
            .any(|id| id.starts_with(ids::CARD_RELATION) || id.starts_with(ids::CARD_MORE)));
    }

    #[test]
    fn a_card_name_shows_six_badges_and_counts_the_rest() {
        let badges = (0..8)
            .map(|n| crate::identity::BadgeChip {
                id: format!("pp:badge=7:{n}"),
                description: format!("Insignia {n}"),
                image: None,
                link: true,
                hint: None,
            })
            .collect();
        let profile = Profile {
            identity: crate::identity::CardIdentity {
                badges,
                ..Default::default()
            },
            ..ada()
        };
        let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 900.0));
        let shown = controls(&laid);
        assert!(shown.contains(&"pp:badge=7:5".to_owned()), "{shown:?}");
        assert!(!shown.contains(&"pp:badge=7:6".to_owned()), "{shown:?}");
        assert!(texts(&laid).contains(&"+2".to_owned()));
    }

    #[test]
    fn an_ignored_person_is_noticed_with_a_way_to_lift_it_and_a_blocked_one_has_none() {
        let ignored = Profile {
            notice: Some(Notice::Ignored {
                unignore: "friend:10:unignore".into(),
            }),
            ..ada()
        };
        let (_, laid) = frame(card(&ignored, &LOOK), Size::new(WIDTH, 900.0));
        let painted = texts(&laid);
        assert!(painted.contains(&"Ignoraste a este usuario".to_owned()));
        assert!(painted.contains(&"Dejar de ignorar".to_owned()));
        assert!(controls(&laid).contains(&"friend:10:unignore".to_owned()));
        let blocked = Profile {
            notice: Some(Notice::Blocked),
            ..ada()
        };
        let (_, laid) = frame(card(&blocked, &LOOK), Size::new(WIDTH, 900.0));
        assert!(texts(&laid).contains(&"Bloqueaste a este usuario".to_owned()));
        assert!(!controls(&laid).iter().any(|id| id.starts_with("friend:")));
    }

    #[test]
    fn roles_on_a_card_come_with_a_remove_button_and_a_plus_for_the_checklist() {
        let action = "ss:member-of:1:10:role:7";
        let profile = Profile {
            role_removals: vec![Some(action.into()), None],
            role_menu: Some((
                format!("{}10", ids::CARD_ROLES),
                vec![MenuEntry::Role {
                    id: action.into(),
                    name: "Moderación".into(),
                    color: None,
                    has: true,
                }],
            )),
            ..ada()
        };
        let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 900.0));
        let shown = controls(&laid);
        assert!(shown.contains(&action.to_owned()), "{shown:?}");
        assert!(
            shown.contains(&format!("{}10", ids::CARD_ROLES)),
            "{shown:?}"
        );
        // Someone with no roles reads the plus as «Añadir roles».
        let none = Profile {
            roles: Vec::new(),
            role_removals: Vec::new(),
            ..profile.clone()
        };
        let (_, laid) = frame(card(&none, &LOOK), Size::new(WIDTH, 900.0));
        assert!(texts(&laid).contains(&"Añadir roles".to_owned()));
        // Without the account's permission, neither shows.
        let plain = Profile {
            role_removals: vec![None, None],
            role_menu: None,
            ..profile
        };
        let (_, laid) = frame(card(&plain, &LOOK), Size::new(WIDTH, 900.0));
        assert!(!controls(&laid)
            .iter()
            .any(|id| id.starts_with(ids::CARD_ROLES)));
        assert!(!controls(&laid).contains(&action.to_owned()));
    }

    fn taller() -> Face {
        Face {
            name: "Taller".into(),
            accent: Color::rgb(200, 120, 60),
            image: None,
        }
    }

    fn full_texts(servers: Mutual, friends: Mutual, tab: FullTab) -> Vec<String> {
        let full = FullProfile {
            connections: Default::default(),
            profile: ada(),
            server: Some("Taller".into()),
            friends_since: None,
            mutual_servers: servers,
            mutual_friends: friends,
            tab,
            loading: false,
            recent: Vec::new(),
        };
        texts(
            &frame(
                full_profile(Element::spacer(), &full, Size::new(900.0, 900.0), &LOOK),
                Size::new(900.0, 900.0),
            )
            .1,
        )
    }

    #[test]
    fn the_full_profile_never_shows_unknown_mutuals_as_none() {
        let about = full_texts(Mutual::Unknown, Mutual::Loading, FullTab::Activity);
        for expected in [
            "Actividad",
            "Servidores en común",
            "Amigos en común",
            "Jugando a Factorio",
            "Roles",
            "Moderación",
        ] {
            assert!(about.iter().any(|t| t == expected), "{expected}: {about:?}");
        }
        let servers = full_texts(Mutual::Unknown, Mutual::Loading, FullTab::Servers);
        assert!(servers
            .iter()
            .any(|t| t == "No disponible en esta conexión."));
        assert!(!servers.iter().any(|t| t == "Ninguno en común."));
        assert!(
            !servers.iter().any(|t| t == "Jugando a Factorio"),
            "one tab at a time"
        );
        // The card beside the tabs stays: its roles are not a tab's.
        assert!(servers.iter().any(|t| t == "Moderación"), "{servers:?}");
        let friends = full_texts(Mutual::Unknown, Mutual::Loading, FullTab::Friends);
        assert!(friends.iter().any(|t| t == "Cargando…"), "{friends:?}");
        // Known: the tabs count them, a known empty list says so.
        let friends = full_texts(
            Mutual::Known(vec![taller()]),
            Mutual::Known(Vec::new()),
            FullTab::Friends,
        );
        assert!(friends.iter().any(|t| t == "1 servidor en común"));
        assert!(friends.iter().any(|t| t == "0 amigos en común"));
        assert!(
            friends.iter().any(|t| t == "Ninguno en común."),
            "{friends:?}"
        );
        assert!(!friends
            .iter()
            .any(|t| t == "No disponible en esta conexión."));
        let servers = full_texts(
            Mutual::Known(vec![taller()]),
            Mutual::Known(Vec::new()),
            FullTab::Servers,
        );
        assert!(servers.iter().any(|t| t == "Taller"), "{servers:?}");
        assert!(servers.iter().any(|t| t == "T"), "its initial as its face");
    }

    #[test]
    fn the_full_profile_reports_its_tabs_and_offers_to_write() {
        let full = FullProfile {
            connections: Default::default(),
            profile: ada(),
            server: None,
            friends_since: None,
            mutual_servers: Mutual::Unknown,
            mutual_friends: Mutual::Unknown,
            tab: FullTab::Activity,
            loading: false,
            recent: Vec::new(),
        };
        let (_, laid) = frame(
            full_profile(Element::spacer(), &full, Size::new(900.0, 900.0), &LOOK),
            Size::new(900.0, 900.0),
        );
        let ids = controls(&laid);
        for tab in FullTab::ALL {
            let id = format!("{}{}", ids::FULL_TAB, tab.key());
            assert!(ids.contains(&id), "{id}: {ids:?}");
            assert_eq!(FullTab::from_key(tab.key()), Some(tab));
        }
        assert!(ids.contains(&"profile:dm:10".to_owned()));
    }

    #[test]
    fn the_full_profile_is_a_card_beside_its_tabs_with_discords_sections() {
        let more = vec![
            MenuEntry::Item(
                format!("{}10", ids::FULL),
                "Ver perfil completo".into(),
                None,
            ),
            MenuEntry::Separator,
            MenuEntry::Item("friend:10:block".into(), "Bloquear".into(), None),
        ];
        let full = FullProfile {
            connections: Default::default(),
            profile: Profile {
                full_action: Some(format!("{}10", ids::FULL)),
                more: Some((format!("{}10", ids::CARD_MORE), more)),
                open_menu: Some(CardMenu::More),
                member_since: vec![
                    Since {
                        scope: "Cuenta".into(),
                        face: None,
                        date: "3 may 2019".into(),
                    },
                    Since {
                        scope: "Taller".into(),
                        face: Some(taller()),
                        date: "4 mar 2025".into(),
                    },
                ],
                ..ada()
            },
            server: Some("Taller".into()),
            friends_since: Some("18 may 2024".into()),
            mutual_servers: Mutual::Unknown,
            mutual_friends: Mutual::Unknown,
            tab: FullTab::Activity,
            loading: false,
            recent: Vec::new(),
        };
        let at = |laid: &limn::Frame, wanted: &str| {
            laid.commands
                .iter()
                .find_map(|command| match command {
                    limn::DrawCommand::Text { text, position, .. } if text == wanted => {
                        Some(*position)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{wanted} is painted"))
        };
        let window = Size::new(1280.0, 900.0);
        let (_, laid) = frame(
            full_profile(Element::spacer(), &full, window, &LOOK),
            window,
        );
        let painted = texts(&laid);
        for expected in [
            "Miembro desde",
            "Amigos desde el",
            "18 may 2024",
            "Bloquear",
        ] {
            assert!(
                painted.iter().any(|t| t == expected),
                "{expected}: {painted:?}"
            );
        }
        // «…» is the card's, less the profile it is already in.
        assert!(
            !painted.iter().any(|t| t == "Ver perfil completo"),
            "{painted:?}"
        );
        // Both dates on one line, the card's; the tabs beside it.
        let (account, server) = (at(&laid, "3 may 2019"), at(&laid, "4 mar 2025"));
        assert_eq!(account.y, server.y);
        let (name, tab) = (at(&laid, "Ada"), at(&laid, "Actividad"));
        assert!(
            tab.x > name.x + FULL_CARD - 64.0 && tab.y < name.y,
            "{name:?} {tab:?}"
        );
        // The custom status in the popout's bubble, beside the 120 px mark, over the name.
        let status = at(&laid, "leyendo");
        assert!(
            status.x > name.x + 120.0 + 9.0 && status.y < name.y - 32.0,
            "{status:?} {name:?}"
        );
        assert!(laid.commands.iter().any(|command| matches!(command,
            limn::DrawCommand::FillRect { rect, color, radius }
                if *radius == 16.0 && *color == raised(&LOOK, 16) && rect.x > name.x + 120.0 && rect.y < status.y
        )));
        // Too narrow for two columns: the tabs go under the card.
        let narrow = Size::new(700.0, 900.0);
        let (_, laid) = frame(
            full_profile(Element::spacer(), &full, narrow, &LOOK),
            narrow,
        );
        let (name, tab) = (at(&laid, "Ada"), at(&laid, "Actividad"));
        assert!(tab.y > name.y, "{name:?} {tab:?}");
    }

    #[test]
    fn a_card_without_a_banner_paints_its_color_across_the_top() {
        // The banner's color is the profile's (or the mark's), else the raised ground.
        let painted = |tint: Option<Color>| {
            let profile = Profile {
                banner_tint: tint,
                ..ada()
            };
            let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 800.0));
            assert!(!laid
                .commands
                .iter()
                .any(|command| matches!(command, limn::DrawCommand::RasterImage { .. })));
            laid.commands.iter().any(|command| {
                matches!(
                    command,
                    limn::DrawCommand::FillRect { rect, color, .. }
                        if *color == tint.unwrap_or(LOOK.raised)
                            && (rect.width - WIDTH).abs() < 1.0
                            && rect.height == BANNER
                )
            })
        };
        let blue = Color::rgb(90, 120, 200);
        assert!(painted(Some(blue)));
        assert!(painted(None));
    }

    #[test]
    fn many_roles_wrap_onto_more_lines_as_chips() {
        let roles: Vec<_> = (0..8)
            .map(|index| (format!("Rol número {index}"), Color::rgb(200, 80, 80)))
            .collect();
        let profile = Profile {
            roles: roles.clone(),
            ..ada()
        };
        let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 1200.0));
        let top = |name: &str| {
            laid.commands
                .iter()
                .find_map(|command| match command {
                    limn::DrawCommand::Text { text, position, .. } if text == name => {
                        Some(*position)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{name} is painted"))
        };
        let (first, last) = (top("Rol número 0"), top("Rol número 7"));
        assert!(last.y > first.y, "{first:?} {last:?}");
        for (name, _) in &roles {
            let right = top(name).x + name.chars().count() as f32 * 7.0;
            assert!(right <= WIDTH, "{name} stays inside the card: {right}");
        }
    }

    #[test]
    fn profile_banner_covers_the_card_without_changing_its_height() {
        let image = limn::RasterImage::new(
            "b".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "Banner de prueba",
        )
        .unwrap();
        for width in [272.0, WIDTH] {
            let profile = ada();
            let (_, before) = frame(card(&profile, &LOOK), Size::new(width, 800.0));
            let with_image = Profile {
                banner: Some(image.clone()),
                ..profile
            };
            let (_, loaded) = frame(card(&with_image, &LOOK), Size::new(width, 800.0));
            let rects: Vec<_> = loaded
                .commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::RasterImage {
                        image: actual,
                        rect,
                        ..
                    } if actual == &image.clone().cover() => Some(*rect),
                    _ => None,
                })
                .collect();
            assert_eq!(rects.len(), 1);
            assert!((rects[0].width - width).abs() < 1.0, "{:?}", rects[0]);
            assert_eq!(rects[0].height, BANNER);
            assert_eq!(
                before.regions, loaded.regions,
                "loading artwork must not move actions"
            );
        }
    }

    #[test]
    fn the_own_popout_offers_the_custom_status_editing_and_the_id() {
        let popout = OwnPopout {
            profile: Profile {
                image: None,
                custom: None,
                status: Status::Invisible,
                ..ada()
            },
            until: Some("hasta las 10:00".into()),
            custom_denied: None,
            edit_denied: None,
            status_open: true,
            copy_id: true,
            switch_accounts: Some(vec![SwitchItem {
                id: UserId(7),
                name: "Otra".into(),
                picture: None,
                active: false,
            }]),
        };
        let (_, laid) = frame(
            Element::row(vec![
                Element::column(vec![own_popout(&popout, &LOOK)]).size(Some(WIDTH), None),
                Element::spacer().flex(1.0),
            ]),
            Size::new(800.0, 700.0),
        );
        let ids = controls(&laid);
        for expected in [
            ids::CUSTOM,
            ids::EDIT,
            ids::COPY_ID,
            ids::STATUS_MENU,
            ids::SWITCH_ACCOUNT,
            "pp:status=dnd",
        ] {
            assert!(ids.contains(&expected.to_owned()), "{expected}: {ids:?}");
        }
        let painted = texts(&laid);
        // At 300 px its words wrap: read them as one line.
        let words = painted.join(" ");
        let words: Vec<&str> = words.split_whitespace().collect();
        assert!(
            words.join(" ").contains("Establecer estado"),
            "the bubble invites to set one: {painted:?}"
        );
        assert!(painted.contains(&"Invisible".to_owned()));
        assert!(painted.contains(&"hasta las 10:00".to_owned()));
    }

    #[test]
    fn a_custom_status_reads_as_its_emoji_and_text_and_has_a_limit() {
        assert_eq!(
            custom_line(Some("🌙"), "  durmiendo "),
            Some("🌙 durmiendo".into())
        );
        assert_eq!(custom_line(None, "   "), None);
        assert!(custom_error(Some(&"x".repeat(CUSTOM_MAX))).is_none());
        assert!(custom_error(Some(&"x".repeat(CUSTOM_MAX + 1))).is_some());
        assert_eq!(ClearAfter::from_key("4h"), Some(ClearAfter::FourHours));
        assert!(ClearAfter::Never.label(Timestamp(0)) == "No eliminar");
    }

    #[test]
    fn the_editor_counts_what_is_written_about_oneself_and_says_when_it_is_too_long() {
        let view = Editor {
            preview: Profile {
                message: None,
                ..ada()
            },
            name: "Ada".into(),
            pronouns: "ella".into(),
            about: "x".repeat(ABOUT_MAX + 1),
            accent: Rgb(0x34_98_DB),
            hex: None,
            failure: None,
            denied: None,
            servers: Vec::new(),
            servers_open: false,
            servers_tab: false,
            bar: Some(UnsavedBar::default()),
            pictures: EditorPictures::default(),
            scope: None,
            credentials: Some("Pide tu contraseña.".into()),
            picked: None,
        };
        let (_, laid) = frame(
            profiles_page(&view, Size::new(1280.0, 2000.0), &LOOK),
            Size::new(1280.0, 2000.0),
        );
        let painted = texts(&laid);
        assert!(painted.contains(&format!("{}/{ABOUT_MAX}", ABOUT_MAX + 1)));
        assert!(painted
            .iter()
            .any(|text| text.contains("«Sobre mí» puede tener hasta 190")));
        let ids = controls(&laid);
        for expected in [
            ids::NAME,
            ids::PRONOUNS,
            ids::ABOUT,
            ids::SAVE,
            ids::TAB_MAIN,
        ] {
            assert!(ids.contains(&expected.to_owned()), "{expected}: {ids:?}");
        }
    }

    fn editing(bar: Option<UnsavedBar>) -> Editor {
        Editor {
            preview: Profile {
                message: None,
                name: "Ada Nueva".into(),
                pronouns: Some("elle".into()),
                about: Some("Ahora en Rust.".into()),
                roles: Vec::new(),
                ..ada()
            },
            name: "Ada Nueva".into(),
            pronouns: "elle".into(),
            about: "Ahora en Rust.".into(),
            accent: Rgb(0x34_98_DB),
            hex: None,
            failure: None,
            denied: None,
            servers: Vec::new(),
            servers_open: false,
            servers_tab: false,
            bar,
            pictures: EditorPictures::default(),
            scope: None,
            credentials: None,
            picked: None,
        }
    }

    #[test]
    fn each_server_of_the_profile_choice_shows_its_icon_or_its_initial() {
        let server = |id: u64, name: &str, image: Option<limn::RasterImage>| EditorServer {
            action: format!("{}{id}", ids::SCOPE),
            name: name.into(),
            nick: None,
            image,
            initials: name.chars().take(2).collect(),
            accent: Color::rgb(200, 40, 40),
        };
        let mut view = editing(None);
        view.servers = vec![
            server(1, "Taller", Some(chosen())),
            server(2, "Laboratorio", None),
        ];
        view.servers_open = true;
        view.servers_tab = true;
        let window = Size::new(1280.0, 800.0);
        let (_, laid) = frame(profiles_page(&view, window, &LOOK), window);
        let icon = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::RasterImage { rect, image, .. }
                    if *image == chosen().cover() =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("the server's own icon");
        assert_eq!(
            (icon.width, icon.height),
            (limn::ICON_SIZE, limn::ICON_SIZE)
        );
        assert!(
            texts(&laid).contains(&"L".to_owned()),
            "its initial without one"
        );
        assert!(
            !laid.commands.iter().any(|command| matches!(
                command,
                limn::DrawCommand::Icon {
                    icon: Icon::Server,
                    ..
                }
            )),
            "no generic server glyph"
        );
    }

    #[test]
    fn the_editor_offers_saving_only_with_changes_and_its_preview_shows_them() {
        for (window, wide) in [
            (Size::new(1280.0, 800.0), true),
            (Size::new(400.0, 800.0), false),
        ] {
            let (_, laid) = frame(profiles_page(&editing(None), window, &LOOK), window);
            let ids = controls(&laid);
            assert!(!ids.contains(&ids::SAVE.to_owned()) && !ids.contains(&ids::RESET.to_owned()));
            let painted = texts(&laid);
            // The preview is the card itself, with what is being written;
            // a narrow window stacks it under the form, out of view.
            let preview: &[&str] = if wide {
                &["Vista previa", "ada", "elle", "Ahora en Rust."]
            } else {
                &[]
            };
            for expected in preview {
                assert!(
                    painted.iter().any(|t| t == expected),
                    "{expected}: {painted:?}"
                );
            }

            let (_, laid) = frame(
                profiles_page(&editing(Some(UnsavedBar::default())), window, &LOOK),
                window,
            );
            let ids = controls(&laid);
            assert!(ids.contains(&ids::SAVE.to_owned()) && ids.contains(&ids::RESET.to_owned()));
            let painted = texts(&laid);
            // Narrow, the bar's message is cut to the room the buttons leave.
            let message = "¡Cuidado! ¡Tienes cambios sin guardar!";
            assert!(
                !wide || painted.iter().any(|t| t == message),
                "{message}: {painted:?}"
            );
        }
    }

    fn chosen() -> limn::RasterImage {
        limn::RasterImage::new(
            "e".repeat(64),
            limn::RasterImageFormat::Png,
            3,
            std::sync::Arc::from("AAAA"),
            "Imagen elegida",
        )
        .unwrap()
    }

    #[test]
    fn a_chosen_picture_is_framed_over_its_whole_and_says_whether_it_moves() {
        let framing = EditorFraming {
            image: chosen(),
            aspect: 2.0,
            frame: [0.5, 0.0, 0.5, 1.0],
            x: 0.75,
            y: 0.5,
            zoom: 0.0,
            round: true,
            note: Some("Se usa sólo el primer cuadro.".into()),
        };
        let (_, laid) = frame(
            Element::column(vec![editor_framing(&framing, 300.0, &LOOK)]),
            Size::new(320.0, 600.0),
        );
        let picture = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::RasterImage { rect, .. } => Some(*rect),
                _ => None,
            })
            .unwrap();
        assert_eq!((picture.width, picture.height), (300.0, 150.0));
        // The frame is drawn over the part kept: the right half, round.
        let outline = laid
            .commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::StrokeRect {
                    rect,
                    color,
                    radius,
                    ..
                } if *color == LOOK.accent => Some((*rect, *radius)),
                _ => None,
            })
            .unwrap();
        let (rect, radius) = outline;
        assert_eq!(
            (
                rect.x - picture.x,
                rect.y - picture.y,
                rect.width,
                rect.height
            ),
            (150.0, 0.0, 150.0, 150.0)
        );
        assert_eq!(radius, 75.0);
        for id in [ids::FRAME, ids::FRAME_ZOOM] {
            assert!(laid.regions.iter().any(|region| region.id == id), "{id}");
        }
        assert!(texts(&laid)
            .iter()
            .any(|text| text.contains("primer cuadro")));
    }

    #[test]
    fn a_banner_the_plan_does_not_allow_is_not_chosen_but_may_be_taken_away() {
        let mut view = editing(None);
        view.pictures = EditorPictures {
            has_banner: true,
            banner_denied: Some("Necesitas Nitro para usar un banner".into()),
            ..EditorPictures::default()
        };
        let window = Size::new(1280.0, 2400.0);
        let (_, laid) = frame(profiles_page(&view, window, &LOOK), window);
        let ids = controls(&laid);
        assert!(ids.contains(&ids::AVATAR_PICK.to_owned()), "{ids:?}");
        assert!(ids.contains(&ids::BANNER_REMOVE.to_owned()));
        assert!(!ids.contains(&ids::BANNER_PICK.to_owned()));
        assert!(texts(&laid)
            .iter()
            .any(|text| text == "Necesitas Nitro para usar un banner"));
    }

    /// Where the text `wanted` starts, as drawn.
    fn at(laid: &limn::Frame, wanted: &str) -> limn::Point {
        laid.commands
            .iter()
            .find_map(|command| match command {
                limn::DrawCommand::Text { position, text, .. } if text == wanted => Some(*position),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{wanted} is not drawn"))
    }

    #[test]
    fn the_profile_page_lists_discords_sections_in_order_and_greys_what_rustcord_cannot_change() {
        let mut view = editing(None);
        view.preview.example_button = true;
        let window = Size::new(696.0, 2000.0);
        let (_, laid) = frame(profiles_page(&view, window, &LOOK), window);
        let painted = texts(&laid);
        let mut from = 0;
        for title in [
            "Nombre para mostrar",
            "Pronombres",
            "Avatar",
            "Decoración de avatar",
            "Placa de nombre",
            "Efecto de perfil",
            "Marco de perfil",
            "Banner del perfil",
            "Color del banner",
            "Biografía",
        ] {
            let found = painted[from..]
                .iter()
                .position(|text| text == title)
                .unwrap_or_else(|| panic!("{title} out of order: {painted:?}"));
            from += found + 1;
        }
        assert!(painted.contains(&"Botón de ejemplo".to_owned()));
        assert!(!painted.contains(&"Editar perfil".to_owned()));
        assert!(painted.contains(&"Rustcord todavía no puede cambiar esto.".to_owned()));
        let ids = controls(&laid);
        for expected in [
            ids::TAB_MAIN,
            ids::NAME,
            ids::AVATAR_PICK,
            ids::BANNER_PICK,
            ids::ABOUT,
            ids::EXAMPLE,
        ] {
            assert!(ids.contains(&expected.to_owned()), "{expected}: {ids:?}");
        }
        assert!(
            !ids.contains(&ids::TAB_SERVERS.to_owned()),
            "no servers, no tab"
        );
        for disabled in [
            ids::DECORATION,
            ids::NAMEPLATE,
            ids::EFFECT,
            ids::PROFILE_FRAME,
        ] {
            assert!(
                !ids.contains(&disabled.to_owned()),
                "{disabled} is clickable"
            );
        }
    }

    #[test]
    fn the_preview_stays_beside_the_form_when_wide_and_comes_under_it_when_narrow() {
        let view = editing(None);
        let wide = Size::new(696.0, 900.0);
        let (_, laid) = frame(profiles_page(&view, wide, &LOOK), wide);
        let (form, preview) = (at(&laid, "Nombre para mostrar"), at(&laid, "Vista previa"));
        assert!((preview.x - (696.0 - WIDTH)).abs() < 1.0, "{preview:?}");
        assert!(preview.y <= form.y + 80.0, "beside: {preview:?} {form:?}");

        let narrow = Size::new(400.0, 2400.0);
        let (_, laid) = frame(profiles_page(&view, narrow, &LOOK), narrow);
        let (form, preview) = (at(&laid, "Nombre para mostrar"), at(&laid, "Vista previa"));
        assert!((preview.x - form.x).abs() < 1.0, "{preview:?} {form:?}");
        assert!(preview.y > form.y + 200.0, "under: {preview:?} {form:?}");
    }

    #[test]
    fn the_per_server_tab_asks_for_a_server_and_then_shows_its_own_sections() {
        let mut view = editing(None);
        view.servers = vec![EditorServer {
            action: format!("{}1", ids::SCOPE),
            name: "Taller".into(),
            nick: None,
            image: None,
            initials: "TA".into(),
            accent: Color::rgb(200, 40, 40),
        }];
        view.servers_tab = true;
        let window = Size::new(1280.0, 1000.0);
        let (_, laid) = frame(profiles_page(&view, window, &LOOK), window);
        assert!(laid
            .regions
            .iter()
            .any(|region| region.id == ids::TAB_SERVERS));
        assert!(
            texts(&laid).contains(&"Elige un servidor para cambiar cómo te ven allí.".to_owned())
        );
        assert!(!texts(&laid).contains(&"Nombre para mostrar".to_owned()));

        view.scope = Some(EditorScope {
            server: "Taller".into(),
            nick: "Ada del taller".into(),
            nick_denied: None,
            about_denied: None,
        });
        let (_, laid) = frame(profiles_page(&view, window, &LOOK), window);
        let painted = texts(&laid);
        for title in ["Apodo", "Avatar", "Banner del perfil", "Biografía"] {
            assert!(
                painted.iter().any(|text| text == title),
                "{title}: {painted:?}"
            );
        }
        assert!(!painted.contains(&"Color del banner".to_owned()));
        assert!(controls(&laid).contains(&ids::NICK.to_owned()));
    }

    #[test]
    fn the_card_being_read_is_a_skeleton_in_the_shapes_of_its_parts() {
        let (_, laid) = frame(
            card_skeleton("profile:loading:ada", &LOOK),
            Size::new(WIDTH, SKELETON_HEIGHT),
        );
        let fills: Vec<limn::Rect> = laid
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect();
        let shown = |x: f32, y: f32, width: f32, height: f32| {
            fills
                .iter()
                .any(|rect| (rect.x, rect.y, rect.width, rect.height) == (x, y, width, height))
        };
        // The banner; the avatar's 80 px box at (16, 65), its ring, and the pulse inside.
        assert!(shown(0.0, 0.0, WIDTH, BANNER), "{fills:?}");
        assert!(shown(INSET, 65.0, AVATAR, AVATAR), "{fills:?}");
        assert!(shown(22.0, 71.0, 68.0, 68.0), "{fills:?}");
        for (x, y, width, height) in [
            (16.0, 157.0, 147.0, 18.0),
            (16.0, 183.0, 94.0, 12.0),
            (16.0, 211.0, 52.0, 16.0),
            (84.0, 211.0, 72.0, 16.0),
            (172.0, 211.0, 48.0, 16.0),
            (0.0, 239.0, WIDTH, 1.0),
            (16.0, 256.0, 241.0, 12.0),
            (16.0, 278.0, 201.0, 12.0),
            (16.0, 300.0, 228.0, 12.0),
            (16.0, 322.0, 161.0, 12.0),
        ] {
            assert!(
                shown(x, y, width, height),
                "{x},{y} {width}x{height}: {fills:?}"
            );
        }
        // Faint text, not limn's opaque raised: it stays visible at the bottom of the pulse.
        let banner = laid.commands.iter().find_map(|command| match command {
            limn::DrawCommand::FillRect { rect, color, .. } if rect.height == BANNER => {
                Some(*color)
            }
            _ => None,
        });
        assert_eq!(banner, Some(Look::alpha(LOOK.text, 16)));
    }

    #[test]
    fn a_card_whose_profile_is_read_is_its_skeleton_until_it_arrives() {
        let reading = Profile {
            loading: true,
            ..ada()
        };
        let (_, laid) = frame(card(&reading, &LOOK), Size::new(WIDTH, SKELETON_HEIGHT));
        assert!(laid
            .access
            .iter()
            .any(|node| node.name == "Cargando perfil"));
        assert!(!texts(&laid).iter().any(|text| text == "Ada"));
        assert!(!controls(&laid).contains(&"profile:dm:10".to_owned()));
        let (_, loaded) = frame(card(&ada(), &LOOK), Size::new(WIDTH, 900.0));
        assert!(!loaded
            .access
            .iter()
            .any(|node| node.name == "Cargando perfil"));
        assert!(controls(&loaded).contains(&format!("{}10", ids::MESSAGE)));
    }

    #[test]
    fn the_about_tab_of_a_profile_being_read_shows_lines_not_no_data() {
        // Nothing of them is known: no about, roles, activity or pronouns.
        let bare = Profile {
            about: None,
            pronouns: None,
            activity: None,
            roles: Vec::new(),
            custom: None,
            ..ada()
        };
        let full = |loading: bool| FullProfile {
            connections: Default::default(),
            profile: Profile {
                loading,
                ..bare.clone()
            },
            server: None,
            friends_since: None,
            mutual_servers: Mutual::Loading,
            mutual_friends: Mutual::Loading,
            tab: FullTab::Activity,
            loading,
            recent: Vec::new(),
        };
        let (_, reading) = frame(
            full_profile(
                Element::spacer(),
                &full(true),
                Size::new(900.0, 900.0),
                &LOOK,
            ),
            Size::new(900.0, 900.0),
        );
        assert!(!texts(&reading)
            .iter()
            .any(|text| text == "Sin actividad por ahora."));
        let widths: Vec<f32> = reading
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect { rect, .. } if rect.height == 12.0 => Some(rect.width),
                _ => None,
            })
            .collect();
        for width in SKELETON_LINES {
            assert!(widths.contains(&width), "{widths:?}");
        }
        // Once the read is over with nothing to show, it says so.
        let (_, settled) = frame(
            full_profile(
                Element::spacer(),
                &full(false),
                Size::new(900.0, 900.0),
                &LOOK,
            ),
            Size::new(900.0, 900.0),
        );
        assert!(texts(&settled)
            .iter()
            .any(|text| text == "Sin actividad por ahora."));
        // What they did lately: its heading and cards, and no «none».
        let lately = FullProfile {
            recent: vec![RecentCard {
                glyph: RecentGlyph::Music,
                title: "444 - Remix".into(),
                subtitle: Some("Yan Block".into()),
                image: None,
                when: "hace 2 semanas".into(),
            }],
            ..full(false)
        };
        let (_, laid) = frame(
            full_profile(Element::spacer(), &lately, Size::new(900.0, 900.0), &LOOK),
            Size::new(900.0, 900.0),
        );
        let said = texts(&laid);
        for text in [
            "Actividad reciente",
            "444 - Remix",
            "Yan Block",
            "hace 2 semanas",
        ] {
            assert!(said.iter().any(|said| said == text), "{text}: {said:?}");
        }
        assert!(!said.iter().any(|text| text == "Sin actividad por ahora."));
    }

    #[test]
    fn the_full_profile_is_discord_s_rounded_modal_not_the_window_s_height() {
        let bare = ada();
        let full = FullProfile {
            connections: Default::default(),
            profile: bare,
            server: None,
            friends_since: None,
            mutual_servers: Mutual::Loading,
            mutual_friends: Mutual::Loading,
            tab: FullTab::Activity,
            loading: false,
            recent: Vec::new(),
        };
        // As measured in Discord: 962×717 at 1440×813, 962×800 at 1920×1080.
        for (window, expected) in [
            (Size::new(1440.0, 813.0), (962.0, 717.0)),
            (Size::new(1920.0, 1080.0), (962.0, 800.0)),
        ] {
            let (_, laid) = frame(
                full_profile(Element::spacer(), &full, window, &LOOK),
                window,
            );
            let dialog = laid
                .access
                .iter()
                .find(|node| node.id == format!("{}:dialog", ids::FULL_CLOSE))
                .expect("the dialog")
                .rect;
            assert_eq!((dialog.width, dialog.height), expected, "{window:?}");
            assert_eq!(
                dialog.x,
                (window.width - expected.0) / 2.0,
                "centred across"
            );
            assert_eq!(dialog.y, (window.height - expected.1) / 2.0, "and down");
        }
    }

    fn games(count: usize) -> Vec<GameIcon> {
        (0..count)
            .map(|n| GameIcon {
                name: format!("Juego {n}"),
                icon: None,
            })
            .collect()
    }

    fn rect_of(laid: &limn::Frame, id: &str) -> limn::Rect {
        laid.regions
            .iter()
            .find(|region| region.id == id)
            .unwrap_or_else(|| panic!("{id} is laid out"))
            .rect
    }

    fn font_of(laid: &limn::Frame, wanted: &str) -> Option<f32> {
        laid.commands.iter().find_map(|command| match command {
            limn::DrawCommand::Text {
                text, font_size, ..
            } if text == wanted => Some(*font_size),
            _ => None,
        })
    }

    #[test]
    fn a_card_shows_four_games_the_last_under_the_rest_s_count_and_no_wishlist() {
        let profile = Profile {
            full_action: Some(format!("{}10", ids::FULL)),
            games: games(6),
            wishlist: vec![WishTile {
                name: "Ondas".into(),
                picture: None,
            }],
            ..ada()
        };
        let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 900.0));
        let painted = texts(&laid);
        assert!(painted.contains(&"+2".to_owned()), "{painted:?}");
        assert_eq!(font_of(&laid, "Colección de juegos"), Some(12.0));
        assert!(!painted.iter().any(|t| t == "Lista de deseados"));
        // Discord's 40 px card across the body, which opens the full profile.
        let games = rect_of(&laid, &format!("{}10", ids::FULL_GAMES));
        assert_eq!((games.width, games.height), (WIDTH - 2.0 * INSET, 40.0));
        // Without games there is no card.
        let (_, none) = frame(card(&ada(), &LOOK), Size::new(WIDTH, 900.0));
        assert!(!texts(&none).iter().any(|t| t == "Colección de juegos"));
    }

    #[test]
    fn the_profile_beside_a_conversation_is_discords_panel() {
        let profile = Profile {
            full_action: Some(format!("{}10", ids::FULL)),
            activity: None,
            member_since: vec![Since {
                scope: "Cuenta".into(),
                face: None,
                date: "7 ene 2020".into(),
            }],
            mutual_friends: Mutual::Known(vec![taller(), taller(), taller()]),
            mutual_servers: Mutual::Known(vec![taller(), taller()]),
            games: games(5),
            wishlist: (0..4)
                .map(|n| WishTile {
                    name: format!("Deseo {n}"),
                    picture: None,
                })
                .collect(),
            ..ada()
        };
        let height = 675.0;
        let (_, laid) = frame(side_panel(&profile, &LOOK), Size::new(SIDE_WIDTH, height));
        let painted = texts(&laid);
        for expected in [
            "Ada",
            "3 amigos en común",
            "2 servidores en común",
            "Escribo programas.",
            "Miembro desde",
            "7 ene 2020",
            "Colección de juegos",
            "+1",
            "Lista de deseados",
            "Ver todo",
            "Ver perfil completo",
        ] {
            assert!(
                painted.iter().any(|t| t == expected),
                "{expected}: {painted:?}"
            );
        }
        assert_eq!(font_of(&laid, "Colección de juegos"), Some(14.0));
        // Inside: 8 of margin and a 2 px edge, then the body 16 in, 268 wide.
        let inner = 8.0 + 2.0;
        let games = rect_of(&laid, &format!("{}10", ids::FULL_GAMES));
        assert_eq!(
            (games.x, games.width, games.height),
            (inner + INSET, WIDTH - 2.0 * INSET, 40.0)
        );
        // The button under the scroll: 40 tall, 16 from the card's bottom.
        let full = rect_of(&laid, &format!("{}10", ids::FULL));
        assert_eq!(
            (full.x, full.y, full.width, full.height),
            (inner + INSET, height - inner - 16.0 - 40.0, 268.0, 40.0)
        );
        assert!(games.y + games.height < full.y, "{games:?} {full:?}");
        // Three of the four wishes, 78 px squares spread across the card.
        let tiles: Vec<limn::Rect> = laid
            .commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillRect { rect, .. }
                    if rect.width == 78.0 && rect.height == 78.0 =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .collect();
        assert_eq!(tiles.len(), 3, "{tiles:?}");
        assert_eq!(tiles[0].x, inner + INSET + 8.0);
        assert_eq!(tiles[2].x + 78.0, inner + WIDTH - INSET - 8.0);
        assert!(controls(&laid).contains(&format!("{}10", ids::FULL_WISHLIST)));
    }

    /// The vertical gradients painted: where, and their two ends.
    fn gradients(laid: &limn::Frame) -> Vec<(limn::Rect, Color, Color)> {
        laid.commands
            .iter()
            .filter_map(|command| match command {
                limn::DrawCommand::FillVerticalGradient { rect, top, bottom } => {
                    Some((*rect, *top, *bottom))
                }
                _ => None,
            })
            .collect()
    }

    /// Whether `laid` fills something in `color`.
    fn fills(laid: &limn::Frame, color: Color) -> bool {
        laid.commands.iter().any(|command| {
            matches!(command,
                limn::DrawCommand::FillRect { color: fill, .. }
                | limn::DrawCommand::FillCornerRect { color: fill, .. } if *fill == color)
        })
    }

    #[test]
    fn a_themed_profile_is_framed_in_its_gradient_on_each_surface() {
        let theme = ProfileTheme::new(Rgb(0x2B_6C_B0), Rgb(0x9F_7A_EA));
        let themed = Profile {
            theme: Some(theme),
            full_action: Some(format!("{}10", ids::FULL)),
            ..ada()
        };
        let full = |profile: Profile| FullProfile {
            connections: Default::default(),
            profile,
            server: None,
            friends_since: None,
            mutual_servers: Mutual::Unknown,
            mutual_friends: Mutual::Unknown,
            tab: FullTab::Activity,
            loading: false,
            recent: Vec::new(),
        };
        let window = Size::new(1000.0, 900.0);
        let surfaces = |profile: &Profile| {
            [
                (
                    card(profile, &LOOK),
                    Size::new(WIDTH, 800.0),
                    POPOUT_FRAME,
                    BANNER,
                ),
                (
                    side_panel(profile, &LOOK),
                    Size::new(SIDE_WIDTH, 800.0),
                    2.0,
                    BANNER,
                ),
                (
                    full_profile(Element::spacer(), &full(profile.clone()), window, &LOOK),
                    window,
                    FULL_FRAME,
                    FULL_HEAD.banner,
                ),
            ]
        };
        let plain = Profile {
            full_action: themed.full_action.clone(),
            ..ada()
        };
        for (element, size, ..) in surfaces(&plain) {
            assert!(gradients(&frame(element, size).1).is_empty(), "unthemed");
        }
        for (element, size, edge, banner) in surfaces(&themed) {
            let laid = frame(element, size).1;
            let found = gradients(&laid);
            let blend = |(_, top, bottom): &&(limn::Rect, Color, Color)| {
                (*top, *bottom) == (theme.primary, theme.secondary)
            };
            // The banner without a picture, and the frame from its foot down.
            let mut banners: Vec<_> = found.iter().filter(blend).collect();
            banners.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
            assert_eq!(banners.len(), 2, "{found:?}");
            assert_eq!(banners[0].0.height, banner);
            assert!(
                (banners[1].0.y - banners[0].0.y - banner).abs() < 1.0,
                "{banners:?}"
            );
            // `edge` of the frame shows beside the banner.
            assert!((banners[1].0.width - banners[0].0.width - 2.0 * edge).abs() < 1.0);
            assert!(fills(&laid, theme.veil), "the veil over it");
        }
        // Its «Enviar mensaje» in the primary made legible.
        let laid = frame(
            full_profile(Element::spacer(), &full(themed.clone()), window, &LOOK),
            window,
        )
        .1;
        assert!(fills(&laid, theme.button[0]));
        assert!(!fills(&laid, LOOK.accent), "{:?}", LOOK.accent);
    }

    #[test]
    fn moving_pictures_play_on_their_surfaces() {
        let raster = |digit: &str| {
            limn::RasterImage::new(
                digit.repeat(64),
                limn::RasterImageFormat::Png,
                3,
                std::sync::Arc::from("AAAA"),
                "Imagen",
            )
            .unwrap()
        };
        let (face, banner) = (raster("c"), raster("d"));
        let surfaces = |playing: bool| {
            let profile = Profile {
                image: Some(face.clone()),
                banner: Some(banner.clone()),
                avatar_playing: playing,
                banner_playing: playing,
                ..ada()
            };
            let (_, laid) = frame(card(&profile, &LOOK), Size::new(WIDTH, 800.0));
            laid.commands
                .iter()
                .filter_map(|command| match command {
                    limn::DrawCommand::Surface { id, rect, .. } => Some((id.clone(), *rect)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert!(surfaces(false).is_empty(), "still, they are pictures");
        let playing = surfaces(true);
        let at = |image: &limn::RasterImage| {
            playing
                .iter()
                .find(|(id, _)| *id == crate::message::animation_surface(image.sha256()))
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{playing:?}"))
        };
        assert_eq!((at(&banner).width, at(&banner).height), (WIDTH, BANNER));
        assert_eq!((at(&face).width, at(&face).height), (AVATAR, AVATAR));
    }
}
