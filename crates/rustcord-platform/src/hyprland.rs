//! Hiding the window in the tray on Hyprland, which neither hides a Wayland
//! surface nor honors minimizing: the window goes to a special workspace of
//! its own and comes back to the one in view, through Hyprland's command
//! socket. Elsewhere the caller hides or minimizes.
//!
//! Blocking, briefly: call it off the interface's thread.
//!
//! Also the binds that hand keys to Rustcord's global shortcuts, which
//! Hyprland never assigns by itself: added to its Lua state while they are
//! wanted, again after each reload of its config, and taken away after.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(1);
const WORKSPACE: &str = "special:rustcord";

/// Hyprland's command socket for this session, when Rustcord runs under it.
pub fn socket() -> Option<PathBuf> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    if signature.is_empty()
        || !signature
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    let dir = crate::local_socket::runtime_dir()?;
    Some(dir.join("hypr").join(signature).join(".socket.sock"))
}

/// Send the window of process `pid` out of sight.
pub fn hide(socket: &Path, pid: u32) -> std::io::Result<()> {
    dispatch(
        socket,
        &format!("movetoworkspacesilent {WORKSPACE},pid:{pid}"),
        &format!(
            r#"hl.dsp.window.move({{ workspace = "{WORKSPACE}", follow = false, window = "pid:{pid}" }})"#
        ),
    )
}

/// Bring it back to the workspace in view, and focus it.
pub fn show(socket: &Path, pid: u32) -> std::io::Result<()> {
    let active = request(socket, "j/activeworkspace")?;
    let id = serde_json::from_str::<serde_json::Value>(&active)
        .ok()
        .and_then(|workspace| workspace["id"].as_i64())
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    dispatch(
        socket,
        &format!("movetoworkspace {id},pid:{pid}"),
        &format!(r#"hl.dsp.window.move({{ workspace = {id}, window = "pid:{pid}" }})"#),
    )?;
    dispatch(
        socket,
        &format!("focuswindow pid:{pid}"),
        &format!(r#"hl.dsp.focus({{ window = "pid:{pid}" }})"#),
    )
}

/// A dispatcher in the classic syntax, or, where Hyprland is configured in
/// Lua (0.55 and later) and refuses that, the same one as a Lua call.
fn dispatch(socket: &Path, classic: &str, lua: &str) -> std::io::Result<()> {
    if ok(request(socket, &format!("dispatch {classic}"))?).is_ok() {
        return Ok(());
    }
    ok(request(socket, &format!("dispatch {lua}"))?)
}

/// A key handed to one of Rustcord's global shortcuts: `keys` in Hyprland's
/// Lua form (`CTRL + SHIFT + t`, `mouse:275`), `shortcut` the portal's id.
/// A `release` bind fires when the key is let go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalBind {
    pub keys: String,
    pub shortcut: String,
    pub release: bool,
}

/// The binds in Hyprland while this lives. They never take the key from the
/// window that has the focus (a game keeps it). Dropped, they are removed.
pub struct GlobalBinds {
    stop: Arc<AtomicBool>,
}

impl GlobalBinds {
    /// Add `binds` for the shortcuts of the portal's `app` and keep them
    /// across reloads of the config, on a thread of their own. `None` when a
    /// key or a shortcut has characters a bind cannot carry.
    pub fn start(socket: PathBuf, app: &str, binds: Vec<GlobalBind>) -> Option<Self> {
        let safe = |text: &str, extra: &[u8]| {
            !text.is_empty()
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || extra.contains(&byte))
        };
        if !safe(app, b"-_.")
            || binds
                .iter()
                .any(|bind| !safe(&bind.keys, b"_+: ") || !safe(&bind.shortcut, b"-_"))
        {
            return None;
        }
        let add = add_binds(app, &binds);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let _ = std::thread::Builder::new()
            .name("rustcord-hyprland-binds".into())
            .spawn(move || keep(&socket, &add, &stopped));
        Some(Self { stop })
    }
}

impl Drop for GlobalBinds {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// The Lua that replaces Rustcord's binds with `binds`.
fn add_binds(app: &str, binds: &[GlobalBind]) -> String {
    let mut lua = format!("{REMOVE} rustcord_binds = {{}}");
    for bind in binds {
        let name = format!("{app}:{}", bind.shortcut);
        let release = if bind.release { ", release = true" } else { "" };
        lua.push_str(&format!(
            r#" table.insert(rustcord_binds, hl.bind("{}", hl.dsp.global("{name}"), {{ non_consuming = true{release}, description = "{name}" }}))"#,
            bind.keys
        ));
    }
    lua
}

/// The Lua that takes Rustcord's binds away, if any are there.
const REMOVE: &str =
    "if rustcord_binds then for _, bind in pairs(rustcord_binds) do bind:remove() end end";

/// Add the binds, add them again whenever Hyprland reloads its config (which
/// drops them), and remove them once stopped.
fn keep(socket: &Path, add: &str, stop: &AtomicBool) {
    let eval = |lua: &str| match request(socket, &format!("eval {lua}")) {
        Ok(answer) if answer.trim() == "ok" => {}
        Ok(answer) => eprintln!(
            "rustcord: Hyprland refused the shortcut binds: {}",
            answer.trim()
        ),
        Err(error) => eprintln!("rustcord: Hyprland's socket: {error}"),
    };
    eval(add);
    let events = socket.with_file_name(".socket2.sock");
    let connect = || {
        let stream = UnixStream::connect(&events).ok()?;
        stream.set_read_timeout(Some(TIMEOUT)).ok()?;
        Some(BufReader::new(stream))
    };
    let mut reader = connect();
    let mut line = String::new();
    while !stop.load(Ordering::Relaxed) {
        let Some(events) = reader.as_mut() else {
            std::thread::sleep(TIMEOUT);
            reader = connect();
            continue;
        };
        // A line cut by the timeout stays in `line` until its end arrives.
        match events.read_line(&mut line) {
            Ok(0) => reader = None,
            Ok(_) => {
                if line.starts_with("configreloaded>>") {
                    eval(add);
                }
                line.clear();
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => {
                reader = None;
                line.clear();
            }
        }
    }
    eval(REMOVE);
}

fn ok(answer: String) -> std::io::Result<()> {
    if answer.trim() == "ok" {
        Ok(())
    } else {
        Err(std::io::Error::other("Hyprland refused"))
    }
}

fn request(socket: &Path, command: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    stream.write_all(command.as_bytes())?;
    let mut answer = String::new();
    stream.take(64 * 1024).read_to_string(&mut answer)?;
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    #[test]
    fn the_window_goes_to_a_special_workspace_and_back_to_the_one_in_view() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".socket.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let mut heard = Vec::new();
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut command = [0u8; 256];
                let read = stream.read(&mut command).unwrap();
                let command = String::from_utf8_lossy(&command[..read]).into_owned();
                let answer = if command == "j/activeworkspace" {
                    r#"{"id": 3, "name": "3"}"#
                } else {
                    "ok"
                };
                stream.write_all(answer.as_bytes()).unwrap();
                heard.push(command);
            }
            heard
        });
        hide(&path, 42).unwrap();
        show(&path, 42).unwrap();
        assert_eq!(
            server.join().unwrap(),
            [
                "dispatch movetoworkspacesilent special:rustcord,pid:42",
                "j/activeworkspace",
                "dispatch movetoworkspace 3,pid:42",
                "dispatch focuswindow pid:42",
            ]
        );
    }

    #[test]
    fn a_lua_configured_hyprland_hears_the_same_commands_as_lua_calls() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".socket.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let mut heard = Vec::new();
            for _ in 0..7 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut command = [0u8; 256];
                let read = stream.read(&mut command).unwrap();
                let command = String::from_utf8_lossy(&command[..read]).into_owned();
                let answer = if command == "j/activeworkspace" {
                    r#"{"id": 3, "name": "3"}"#
                } else if command.starts_with("dispatch hl.") {
                    "ok"
                } else {
                    "error: ')' expected"
                };
                stream.write_all(answer.as_bytes()).unwrap();
                heard.push(command);
            }
            heard
        });
        hide(&path, 42).unwrap();
        show(&path, 42).unwrap();
        assert_eq!(
            server.join().unwrap(),
            [
                "dispatch movetoworkspacesilent special:rustcord,pid:42",
                r#"dispatch hl.dsp.window.move({ workspace = "special:rustcord", follow = false, window = "pid:42" })"#,
                "j/activeworkspace",
                "dispatch movetoworkspace 3,pid:42",
                r#"dispatch hl.dsp.window.move({ workspace = 3, window = "pid:42" })"#,
                "dispatch focuswindow pid:42",
                r#"dispatch hl.dsp.focus({ window = "pid:42" })"#,
            ]
        );
    }

    #[test]
    fn rustcords_binds_go_in_after_each_reload_and_out_when_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".socket.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let events = UnixListener::bind(dir.path().join(".socket2.sock")).unwrap();
        let (each, heard_one) = std::sync::mpsc::channel();
        let commands = std::thread::spawn(move || {
            let mut heard = Vec::new();
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut command = [0u8; 4096];
                let read = stream.read(&mut command).unwrap();
                heard.push(String::from_utf8_lossy(&command[..read]).into_owned());
                stream.write_all(b"ok").unwrap();
                each.send(()).unwrap();
            }
            heard
        });
        let binds = vec![
            GlobalBind {
                keys: "CTRL + SHIFT + t".into(),
                shortcut: "push-to-talk".into(),
                release: false,
            },
            GlobalBind {
                keys: "CTRL + SHIFT + t".into(),
                shortcut: "push-to-talk-release".into(),
                release: true,
            },
        ];
        let kept = GlobalBinds::start(path.clone(), "rustcord", binds.clone()).unwrap();
        heard_one.recv().unwrap();
        let (mut stream, _) = events.accept().unwrap();
        stream
            .write_all(b"workspace>>2\nconfigreloaded>>\n")
            .unwrap();
        // Added again after the reload; then dropped, they go.
        heard_one.recv().unwrap();
        drop(kept);
        let heard = commands.join().unwrap();
        let add = format!("eval {}", add_binds("rustcord", &binds));
        assert_eq!(heard, [add.clone(), add, format!("eval {REMOVE}")]);
        assert!(heard[0].contains(
            r#"hl.bind("CTRL + SHIFT + t", hl.dsp.global("rustcord:push-to-talk-release"), { non_consuming = true, release = true, description = "rustcord:push-to-talk-release" })"#
        ));
        // Nothing that could break out of the Lua string is accepted.
        let quoted = GlobalBind {
            keys: r#"t"), os.execute("x"#.into(),
            shortcut: "push-to-talk".into(),
            release: false,
        };
        assert!(GlobalBinds::start(path, "rustcord", vec![quoted]).is_none());
    }
}
