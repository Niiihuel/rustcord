//! Discord's streamer mode: what it hides and silences while on, the notice
//! over the window, and its switch turned by OBS or XSplit running.
//!
//! Its settings are «Apariencia → Modo de transmisión en vivo». Hiding the
//! personal information covers the account's and the connections' pages and
//! the private notes; the invitation links are dotted where they show (still
//! copied whole); the sounds and the notifications of the system stay off.

use limn::Element;
use rustcord_components::Look;

use crate::{Intent, Prefs, Ui};

/// How streamer mode is set, kept between runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Streamer {
    pub on: bool,
    /// Turned on while OBS or XSplit runs, and off once they end. Off until
    /// turned on: it reads the names of the running programs.
    pub auto: bool,
    pub hide_personal: bool,
    pub hide_invites: bool,
    pub mute_sounds: bool,
    pub mute_notifications: bool,
}

impl Default for Streamer {
    /// Discord's: off, and all it hides while on.
    fn default() -> Self {
        Self {
            on: false,
            auto: false,
            hide_personal: true,
            hide_invites: true,
            mute_sounds: true,
            mute_notifications: true,
        }
    }
}

impl Streamer {
    pub fn hides_personal(&self) -> bool {
        self.on && self.hide_personal
    }

    pub fn hides_invites(&self) -> bool {
        self.on && self.hide_invites
    }

    pub fn silences_sounds(&self) -> bool {
        self.on && self.mute_sounds
    }

    pub fn silences_notifications(&self) -> bool {
        self.on && self.mute_notifications
    }

    /// Its switches, by the key each keeps and the action that flips it.
    fn switches(&mut self) -> [(&'static str, &mut bool); 6] {
        [
            ("on", &mut self.on),
            ("auto", &mut self.auto),
            ("personal", &mut self.hide_personal),
            ("invites", &mut self.hide_invites),
            ("sounds", &mut self.mute_sounds),
            ("notifications", &mut self.mute_notifications),
        ]
    }
}

/// What this run remembers of it, not kept.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    /// The notice's × hid it until the mode is turned on again.
    pub notice_hidden: bool,
    /// OBS or XSplit turned it on: their end turns it off.
    pub auto_on: bool,
}

pub(crate) fn kept(prefs: &Prefs) -> Vec<(&'static str, Option<String>)> {
    let mut streamer = prefs.streamer;
    streamer
        .switches()
        .into_iter()
        .map(|(key, on)| (setting(key), Some(on.to_string())))
        .collect()
}

pub(crate) fn restore(prefs: &mut Prefs, setting_of: &impl Fn(&str) -> Option<String>) {
    let mut usual = Streamer::default();
    let usual = usual.switches().map(|(_, on)| *on);
    for ((key, on), usual) in prefs.streamer.switches().into_iter().zip(usual) {
        *on = match setting_of(setting(key)).as_deref() {
            Some("true") => true,
            Some("false") => false,
            _ => usual,
        };
    }
}

/// The name a switch is kept by.
fn setting(key: &str) -> &'static str {
    match key {
        "on" => "streamer",
        "auto" => "streamer_auto",
        "personal" => "streamer_hide_personal",
        "invites" => "streamer_hide_invites",
        "sounds" => "streamer_mute_sounds",
        _ => "streamer_mute_notifications",
    }
}

/// The notice's «Desactivar» and ×, and the settings' switches,
/// `us:streamer={key}`.
pub(crate) const OFF: &str = "streamer:off";
pub(crate) const DISMISS: &str = "streamer:dismiss";

/// A click on one of its controls; `None` when it is not one.
pub(crate) fn clicked(ui: &mut Ui, id: &str) -> Option<Vec<Intent>> {
    if id == DISMISS {
        ui.streamer.notice_hidden = true;
        return Some(Vec::new());
    }
    let key = if id == OFF {
        "on"
    } else {
        id.strip_prefix("us:streamer=")?
    };
    let mut streamer = ui.prefs.streamer;
    let (key, on) = streamer
        .switches()
        .into_iter()
        .find(|(found, _)| *found == key)?;
    *on = !*on;
    let value = *on;
    ui.prefs.streamer = streamer;
    if key == "on" {
        // Turned by hand: OBS ending leaves it as the person chose.
        ui.streamer = Session::default();
    }
    Some(vec![Intent::SaveSetting {
        key: setting(key).into(),
        value: Some(value.to_string()),
    }])
}

/// OBS or XSplit started (`true`) or ended, as the host sees the programs
/// while [`Streamer::auto`] is on.
pub fn broadcasting(ui: &mut Ui, running: bool) -> Vec<Intent> {
    let streamer = &mut ui.prefs.streamer;
    let turn = if running {
        !streamer.on
    } else {
        streamer.on && ui.streamer.auto_on
    };
    if !streamer.auto || !turn {
        return Vec::new();
    }
    streamer.on = running;
    ui.streamer = Session {
        notice_hidden: false,
        auto_on: running,
    };
    vec![Intent::SaveSetting {
        key: setting("on").into(),
        value: Some(running.to_string()),
    }]
}

/// The notice over the window while it is on, until its × hides it.
pub(crate) fn notice(ui: &Ui, look: &Look) -> Option<Element> {
    (ui.prefs.streamer.on && !ui.streamer.notice_hidden).then(|| {
        rustcord_components::streamer::notice(OFF, DISMISS, crate::title_bar::room(ui), look)
    })
}

/// `link` as it may show: dotted while the invitations are hidden.
pub(crate) fn link(ui: &Ui, link: &str) -> String {
    if ui.prefs.streamer.hides_invites() {
        rustcord_components::streamer::dotted(link)
    } else {
        link.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_switches_are_kept_and_obs_turns_it_on_and_off_only_as_it_did() {
        let mut ui = Ui::default();
        assert!(!ui.prefs.streamer.on && ui.prefs.streamer.hide_personal);
        assert!(clicked(&mut ui, "us:streamer=personal").is_some());
        assert!(!ui.prefs.streamer.hide_personal);
        let kept: std::collections::HashMap<_, _> = kept(&ui.prefs).into_iter().collect();
        let mut back = Prefs::default();
        restore(&mut back, &|key| kept.get(key).cloned().flatten());
        assert_eq!(back.streamer, ui.prefs.streamer);
        // Without its switch, OBS changes nothing.
        assert!(broadcasting(&mut ui, true).is_empty());
        clicked(&mut ui, "us:streamer=auto");
        broadcasting(&mut ui, true);
        assert!(ui.prefs.streamer.on && ui.streamer.auto_on);
        assert!(notice(&ui, &ui.look()).is_some());
        clicked(&mut ui, DISMISS);
        assert!(notice(&ui, &ui.look()).is_none());
        broadcasting(&mut ui, false);
        assert!(
            !ui.prefs.streamer.on,
            "OBS turned it on, its end turns it off"
        );
        // Turned on by hand, OBS ending leaves it on.
        clicked(&mut ui, OFF);
        assert!(ui.prefs.streamer.on && !ui.streamer.notice_hidden);
        broadcasting(&mut ui, true);
        broadcasting(&mut ui, false);
        assert!(ui.prefs.streamer.on);
        assert_eq!(link(&ui, "discord.gg/abc"), "discord.gg/•••");
        assert!(ui.prefs.streamer.silences_sounds());
    }
}
