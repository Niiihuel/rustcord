//! What the interface
//! knows of them, which the host keeps current; the header's button of an
//! optional one, with the question it asks over a call; and what that
//! button and Ajustes › Actualizaciones ask for. A mandatory update never
//! reaches this window: the host shows its screen
//! (`rustcord_components::updates::screen`) before anything opens.

use limn::{Element, Icon, UiAction};
use rustcord_components::channel_menu::{Entry, Item};
use rustcord_components::updates::{ids, voice_confirm, Stage, UpdateView};
use rustcord_components::Look;
use rustcord_i18n::t;
use rustcord_model::Timestamp;
use rustcord_state::Store;

use crate::{Intent, Ui};

/// What the interface knows of updates.
#[derive(Clone, Debug, PartialEq)]
pub struct Updates {
    /// Where this copy is installed, when Rustcord's installer put it there
    /// and an update can replace it; `None` for a build from source.
    pub installation: Option<String>,
    /// Look for a newer Rustcord when it starts.
    pub at_start: bool,
    /// A check is on its way.
    pub checking: bool,
    /// A check or an install is on its way: the buttons wait for it.
    pub busy: bool,
    /// When the last check ended, and why it failed if it did.
    pub checked: Option<Timestamp>,
    pub error: Option<String>,
    /// The newer release, and where its installation stands.
    pub offered: Option<UpdateView>,
    /// The latest release the last check found, newer or not: its version
    /// and notes, for Ajustes › Actualizaciones to say what it brought.
    pub latest: Option<(String, String)>,
    /// The header's button was pressed in a call: the question whether to
    /// leave it shows.
    pub voice_confirm: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            installation: None,
            // On until turned off.
            at_start: true,
            checking: false,
            busy: false,
            checked: None,
            error: None,
            offered: None,
            latest: None,
            voice_confirm: false,
        }
    }
}

impl Updates {
    /// The notes Ajustes › Actualizaciones shows, with their version: the
    /// update offered's, else those of the version installed when the last
    /// check found it is the latest.
    pub fn shown_notes(&self) -> Option<(&str, &str)> {
        if let Some(offered) = &self.offered {
            return Some((offered.version.as_str(), offered.notes.as_str()));
        }
        self.latest
            .as_ref()
            .filter(|(version, _)| version == env!("CARGO_PKG_VERSION"))
            .map(|(version, notes)| (version.as_str(), notes.as_str()))
    }
}

impl Updates {
    /// The release offered can be installed now: this copy is Rustcord's
    /// installer's, and nothing is under way.
    pub fn installable(&self) -> bool {
        self.installation.is_some()
            && self
                .offered
                .as_ref()
                .is_some_and(|offered| matches!(offered.stage, Stage::Offered | Stage::Failed(_)))
    }

    /// What the header's button asks for where the update stands: install
    /// it (or try again), or start the version installed.
    fn proceed(&self) -> Vec<Intent> {
        if self.installable() {
            vec![Intent::InstallUpdate]
        } else if matches!(&self.offered, Some(offered) if offered.stage == Stage::Installed) {
            vec![Intent::RestartForUpdate]
        } else {
            Vec::new()
        }
    }
}

/// What the header's button, its question and the settings ask for;
/// `None` for other actions.
pub(crate) fn update(ui: &mut Ui, store: &Store, action: &UiAction) -> Option<Vec<Intent>> {
    let UiAction::Clicked { id } = action else {
        return None;
    };
    let updates = &mut ui.updates;
    Some(match id.as_str() {
        ids::CHECK if !updates.checking && !updates.busy => vec![Intent::CheckForUpdates],
        ids::INSTALL | ids::RETRY if updates.installable() => vec![Intent::InstallUpdate],
        ids::RESTART if matches!(&updates.offered, Some(offered) if offered.stage == Stage::Installed) =>
        {
            vec![Intent::RestartForUpdate]
        }
        ids::CHECK | ids::INSTALL | ids::RETRY | ids::RESTART => Vec::new(),
        // Installing closes Rustcord, and the call with it: Discord asks first.
        ids::BUTTON if store.call().is_some() && !updates.proceed().is_empty() => {
            updates.voice_confirm = true;
            Vec::new()
        }
        ids::BUTTON => updates.proceed(),
        ids::VOICE_CONFIRM => {
            updates.voice_confirm = false;
            updates.proceed()
        }
        ids::VOICE_CANCEL => {
            updates.voice_confirm = false;
            Vec::new()
        }
        // Its page on GitHub, with the pictures and videos of its notes.
        ids::ON_GITHUB => match updates.shown_notes() {
            Some((version, _)) => vec![Intent::OpenUrl(
                rustcord_components::updates::release_page(version),
            )],
            None => Vec::new(),
        },
        ids::AT_START => {
            updates.at_start = !updates.at_start;
            vec![Intent::CheckUpdatesAtStart(updates.at_start)]
        }
        _ => return None,
    })
}

/// Where the update offered stands, for the header's button; only for a
/// copy that can install it: a build from source hears of it in the
/// settings alone.
pub(crate) fn header_stage(ui: &Ui) -> Option<Stage> {
    let updates = &ui.updates;
    updates
        .installation
        .as_ref()
        .and(updates.offered.as_ref())
        .map(|offered| offered.stage.clone())
}

/// The header's button as an item of a narrow header's «más», whose title
/// needs its room: while it can be pressed, the same action and words.
pub(crate) fn more_entry(ui: &Ui) -> Option<Entry> {
    let stage = header_stage(ui)?;
    let mut item = Item::new(
        ids::BUTTON,
        match &stage {
            Stage::Offered => t("Hay una actualización disponible"),
            Stage::Installed => t("¡Actualización lista!"),
            Stage::Failed(_) => t("No se pudo actualizar"),
            // Ajustes › Actualizaciones shows its progress.
            Stage::Downloading { .. } | Stage::Installing | Stage::Restarting => return None,
        },
    );
    item.icon = Some(Icon::Custom(rustcord_components::glyphs::DOWNLOAD));
    if let Stage::Failed(reason) = stage {
        item.detail = Some(reason);
    }
    Some(item.into())
}

/// The question before an update leaves the call, over `root`, while it is
/// asked.
pub(crate) fn overlay(ui: &Ui, root: Element, look: &Look) -> Element {
    if ui.updates.voice_confirm {
        voice_confirm(root, look)
    } else {
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click(ui: &mut Ui, id: &str) -> Option<Vec<Intent>> {
        update(ui, &Store::new(), &UiAction::Clicked { id: id.into() })
    }

    fn offered(stage: Stage) -> Option<UpdateView> {
        Some(UpdateView {
            version: "1.2.0".into(),
            installed: "1.1.0".into(),
            notes: "Notas.".into(),
            stage,
        })
    }

    #[test]
    fn the_notes_shown_are_the_offered_ones_else_the_installed_version_s() {
        let mut ui = Ui::new();
        assert_eq!(ui.updates.shown_notes(), None);
        assert_eq!(click(&mut ui, ids::ON_GITHUB), Some(Vec::new()));
        ui.updates.latest = Some(("0.0.1".into(), "Vieja.".into()));
        assert_eq!(ui.updates.shown_notes(), None, "not this version's");
        let installed = env!("CARGO_PKG_VERSION");
        ui.updates.latest = Some((installed.into(), "Lo nuevo.".into()));
        assert_eq!(ui.updates.shown_notes(), Some((installed, "Lo nuevo.")));
        ui.updates.offered = offered(Stage::Offered);
        assert_eq!(ui.updates.shown_notes(), Some(("1.2.0", "Notas.")));
        assert_eq!(
            click(&mut ui, ids::ON_GITHUB),
            Some(vec![Intent::OpenUrl(
                "https://github.com/Niiihuel/rustcord/releases/tag/v1.2.0".into()
            )])
        );
    }

    #[test]
    fn an_update_is_installed_only_by_a_copy_that_can_and_only_once() {
        let mut ui = Ui::new();
        assert_eq!(click(&mut ui, ids::INSTALL), Some(Vec::new()));
        ui.updates.offered = offered(Stage::Offered);
        // A build from source only hears of it.
        assert_eq!(click(&mut ui, ids::INSTALL), Some(Vec::new()));
        ui.updates.installation = Some("/home/ada/.local/share/rustcord/app".into());
        assert_eq!(
            click(&mut ui, ids::INSTALL),
            Some(vec![Intent::InstallUpdate])
        );
        ui.updates.offered = offered(Stage::Downloading { have: 1, whole: 2 });
        assert_eq!(click(&mut ui, ids::INSTALL), Some(Vec::new()));
        ui.updates.offered = offered(Stage::Failed("No.".into()));
        assert_eq!(
            click(&mut ui, ids::RETRY),
            Some(vec![Intent::InstallUpdate])
        );
    }

    #[test]
    fn checks_are_asked_one_at_a_time_and_at_start_is_kept() {
        let mut ui = Ui::new();
        assert_eq!(
            click(&mut ui, ids::CHECK),
            Some(vec![Intent::CheckForUpdates])
        );
        ui.updates.checking = true;
        assert_eq!(click(&mut ui, ids::CHECK), Some(Vec::new()));
        assert!(ui.updates.at_start, "on by default");
        assert_eq!(
            click(&mut ui, ids::AT_START),
            Some(vec![Intent::CheckUpdatesAtStart(false)])
        );
        assert!(!ui.updates.at_start);
        assert_eq!(click(&mut ui, "us:close"), None);
    }

    #[test]
    fn the_header_button_is_only_for_a_copy_that_can_install_and_asks_for_its_stage() {
        let mut ui = Ui::new();
        assert_eq!(header_stage(&ui), None, "nothing offered");
        ui.updates.offered = offered(Stage::Offered);
        assert_eq!(header_stage(&ui), None, "a build from source");
        assert_eq!(click(&mut ui, ids::BUTTON), Some(Vec::new()));
        ui.updates.installation = Some("/opt".into());
        assert_eq!(header_stage(&ui), Some(Stage::Offered));
        for (stage, asked) in [
            (Stage::Offered, vec![Intent::InstallUpdate]),
            (Stage::Downloading { have: 1, whole: 2 }, Vec::new()),
            (Stage::Installing, Vec::new()),
            (Stage::Restarting, Vec::new()),
            (Stage::Installed, vec![Intent::RestartForUpdate]),
            (Stage::Failed("No.".into()), vec![Intent::InstallUpdate]),
        ] {
            ui.updates.offered = offered(stage.clone());
            assert_eq!(header_stage(&ui), Some(stage.clone()));
            // A narrow header's «más» has it while it can be pressed.
            let item = more_entry(&ui).and_then(|entry| match entry {
                Entry::Item(item) => Some(item.id),
                _ => None,
            });
            assert_eq!(item.is_some(), !asked.is_empty(), "{stage:?}");
            assert_eq!(click(&mut ui, ids::BUTTON), Some(asked), "{stage:?}");
            assert!(!ui.updates.voice_confirm, "no call, no question");
        }
    }
}
