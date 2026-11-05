//! The English of every interface text, keyed by its Spanish, one file per
//! area of the code. A text used in several areas may be in each of their
//! files, always with the same English.

type Entries = &'static [(&'static str, &'static str)];

mod calls;
mod common;
mod components_channels;
mod components_expressions;
mod components_messages;
mod components_people;
mod components_servers;
mod desktop;
mod provider;
mod session;
mod state_core;
mod ui_account;
mod ui_admin;
mod ui_chat;
mod ui_server_settings;
mod ui_settings;
mod ui_social;
// Voice.
mod message_content;
mod voice_panel;
// Files in apps' modals.
mod app_files;
// Emoji and states.
mod emoji_states;
// Video and documents.
mod calls_panel;
mod profile_edit;
mod video_documents;
mod voice_protocol;
// Reading settings and pictures on disk.
mod reading;
// Profile card and cosmetics.
mod profile_card;
// Rules and onboarding.
mod linux_desktop;
mod onboarding;
// Threads, forums and stages.
mod threads_stages;
// Channels and voice administration.
mod admin_channels_voice;
// Desktop session and games.
mod desktop_session;
// Editable shortcuts.
mod shortcuts;
// Own themes, main color and transparency.
mod themes;
// Server administration and members.
mod admin_server_members;
// Why an invitation was refused.
mod events;
mod invitation_refusals;
// AutoMod.
mod automod;
// Rustcord's own updates.
mod updates;
mod windows;
// Windows' media and desktop.
mod windows_media;
// The image and video viewer.
mod media_viewer;
// Streams.
mod streams;
// People's rows in the column.
mod voice_rows;
// The «Activo ahora» column of the friends page.
mod active_now;
// Activity privacy, connections and the developer's settings.
mod settings_apps;
// Settings › Apariencia, Accesibilidad, Sistema and Idioma y hora.
mod settings_look;
// Streamer mode.
mod streamer;
// Discord's settings shown dimmed, and the sections around them.
mod settings_pending;
// The card of the files of a message being sent.
mod uploads_progress;
// The server's menu and its settings, as Discord's.
mod server_admin;

/// Each file's entries, by the file's name.
pub(crate) const ALL: &[(&str, Entries)] = &[
    ("common", common::ENTRIES),
    ("streamer", streamer::ENTRIES),
    ("settings_pending", settings_pending::ENTRIES),
    ("components_messages", components_messages::ENTRIES),
    ("components_expressions", components_expressions::ENTRIES),
    ("components_channels", components_channels::ENTRIES),
    ("components_servers", components_servers::ENTRIES),
    ("components_people", components_people::ENTRIES),
    ("ui_settings", ui_settings::ENTRIES),
    ("ui_account", ui_account::ENTRIES),
    ("ui_server_settings", ui_server_settings::ENTRIES),
    ("ui_admin", ui_admin::ENTRIES),
    ("ui_chat", ui_chat::ENTRIES),
    ("ui_social", ui_social::ENTRIES),
    ("provider", provider::ENTRIES),
    ("state_core", state_core::ENTRIES),
    ("session", session::ENTRIES),
    ("desktop", desktop::ENTRIES),
    ("calls", calls::ENTRIES),
    ("voice_panel", voice_panel::ENTRIES),
    ("message_content", message_content::ENTRIES),
    ("app_files", app_files::ENTRIES),
    ("emoji_states", emoji_states::ENTRIES),
    ("video_documents", video_documents::ENTRIES),
    ("profile_edit", profile_edit::ENTRIES),
    ("voice_protocol", voice_protocol::ENTRIES),
    ("calls_panel", calls_panel::ENTRIES),
    ("reading", reading::ENTRIES),
    ("profile_card", profile_card::ENTRIES),
    ("onboarding", onboarding::ENTRIES),
    ("linux_desktop", linux_desktop::ENTRIES),
    ("threads_stages", threads_stages::ENTRIES),
    ("admin_channels_voice", admin_channels_voice::ENTRIES),
    ("desktop_session", desktop_session::ENTRIES),
    ("shortcuts", shortcuts::ENTRIES),
    ("themes", themes::ENTRIES),
    ("admin_server_members", admin_server_members::ENTRIES),
    ("invitation_refusals", invitation_refusals::ENTRIES),
    ("events", events::ENTRIES),
    // AutoMod.
    ("automod", automod::ENTRIES),
    ("updates", updates::ENTRIES),
    ("windows", windows::ENTRIES),
    ("windows_media", windows_media::ENTRIES),
    // The image and video viewer.
    ("media_viewer", media_viewer::ENTRIES),
    // Streams.
    ("streams", streams::ENTRIES),
    // People's rows in the column.
    ("voice_rows", voice_rows::ENTRIES),
    // The «Activo ahora» column of the friends page.
    ("active_now", active_now::ENTRIES),
    // Activity privacy, connections and the developer's settings.
    ("settings_apps", settings_apps::ENTRIES),
    // Settings › Apariencia, Accesibilidad, Sistema and Idioma y hora.
    ("settings_look", settings_look::ENTRIES),
    // The card of the files of a message being sent.
    ("uploads_progress", uploads_progress::ENTRIES),
    // The server's menu and its settings, as Discord's.
    ("server_admin", server_admin::ENTRIES),
];
