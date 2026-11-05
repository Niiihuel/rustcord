//! The faces of name fonts (`NameStyle::font`): each fetched once a name in
//! it is shown, from where its provider says (`Provider::name_font`), with
//! the pictures' credential-free client; kept on disk with the account's
//! artwork as it was downloaded, sealed as a picture is; and handed to the
//! host as TrueType or OpenType, WOFF2 decoded here, off the interface's
//! thread.

use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::Instant;

use rustcord_model::ImageKind;
use tokio::task::JoinHandle;

use super::{download, Images, RETRY_DELAY};

/// A face's file at most: the largest Discord serves is 264 KiB.
const FONT_BYTES: usize = 2 * 1024 * 1024;
/// Faces downloading at once, at most; the rest wait for a later frame.
const CONCURRENT: usize = 2;
/// Fonts looked at in one call, at most.
const WANTED: usize = 64;

enum State {
    Loading(JoinHandle<()>),
    /// Given to the host, or with no face to give.
    Done,
    /// It did not load: asked again from then on.
    Failed(Instant),
}

pub(crate) struct Fonts {
    state: HashMap<u16, State>,
    sender: mpsc::Sender<(u16, Option<Arc<[u8]>>)>,
    results: mpsc::Receiver<(u16, Option<Arc<[u8]>>)>,
}

impl Default for Fonts {
    fn default() -> Self {
        let (sender, results) = mpsc::channel();
        Self {
            state: HashMap::new(),
            sender,
            results,
        }
    }
}

impl Fonts {
    /// Stop the downloads under way, as a logout does.
    pub(crate) fn clear(&mut self) {
        self.state.retain(|_, state| match state {
            State::Loading(task) => {
                task.abort();
                false
            }
            _ => true,
        });
    }
}

/// The TrueType or OpenType face in `bytes`: decoded from WOFF2, or as it is.
pub(crate) fn face(bytes: &[u8]) -> Option<Vec<u8>> {
    match bytes.get(..4)? {
        b"wOF2" => wuff::decompress_woff2(bytes).ok(),
        [0, 1, 0, 0] | b"OTTO" | b"true" => Some(bytes.to_vec()),
        _ => None,
    }
}

impl Images {
    /// The faces of the name fonts of `wanted` that arrived since the last
    /// call, each once, and the download of those still missing: a font
    /// its provider has no face for is never asked again; one that did not
    /// load, after a while.
    pub fn load_fonts(&mut self, wanted: &[u16]) -> Vec<(u16, Arc<[u8]>)> {
        let mut ready = Vec::new();
        while let Ok((id, face)) = self.fonts.results.try_recv() {
            let state = match face {
                Some(face) => {
                    ready.push((id, face));
                    State::Done
                }
                None => Instant::now()
                    .checked_add(RETRY_DELAY)
                    .map_or(State::Done, State::Failed),
            };
            self.fonts.state.insert(id, state);
        }
        // As a picture: on disk only for the account's own session, and
        // nothing it may keep is asked while its disk opens.
        let (disk, opening) = if self.disk.trusted {
            (self.disk.current(), self.disk.opening())
        } else {
            (None, false)
        };
        let Some(client) = self.client.clone().filter(|_| !opening) else {
            return ready;
        };
        for &id in wanted.iter().take(WANTED) {
            match self.fonts.state.get(&id) {
                Some(State::Loading(_) | State::Done) => continue,
                Some(State::Failed(at)) if *at > Instant::now() => continue,
                _ => {}
            }
            let loading = self.fonts.state.values();
            if loading
                .filter(|state| matches!(state, State::Loading(_)))
                .count()
                >= CONCURRENT
            {
                break;
            }
            let Some((url, address)) = self.font_url(id) else {
                self.fonts.state.insert(id, State::Done);
                continue;
            };
            let (client, disk, sender, wake) = (
                client.clone(),
                disk.clone(),
                self.fonts.sender.clone(),
                self.wake.clone(),
            );
            let task = self.runtime.spawn(async move {
                // Kept from an earlier run: the network is not asked.
                let kept = match disk.clone() {
                    Some(disk) => {
                        let address = address.clone();
                        tokio::task::spawn_blocking(move || {
                            disk.lock().ok()?.read(ImageKind::Artwork, &address)
                        })
                        .await
                        .ok()
                        .flatten()
                    }
                    None => None,
                };
                let fetched = kept.is_none();
                let bytes = match kept {
                    Some(bytes) => Some(bytes),
                    None => download(&client, url, FONT_BYTES).await.ok(),
                };
                let face = match bytes {
                    Some(bytes) => tokio::task::spawn_blocking(move || {
                        let found = face(&bytes)?;
                        // Kept for the next run once it proved to be a face.
                        if let (true, Some(disk)) = (fetched, disk) {
                            if let Ok(mut disk) = disk.lock() {
                                disk.write(ImageKind::Artwork, &address, &bytes);
                            }
                        }
                        Some(Arc::from(found))
                    })
                    .await
                    .ok()
                    .flatten(),
                    None => None,
                };
                if sender.send((id, face)).is_ok() {
                    wake();
                }
            });
            self.fonts.state.insert(id, State::Loading(task));
        }
        ready
    }

    /// Where the face of `id` is, as the provider says: over HTTPS, without
    /// credentials, as a picture's address is checked.
    fn font_url(&self, id: u16) -> Option<(url::Url, String)> {
        let raw = self.hosted.as_ref()?.name_font(id)?;
        let origin = url::Url::parse(&raw).ok()?.origin().ascii_serialization();
        let loopback = false;
        #[cfg(test)]
        let loopback = loopback || self.loopback;
        let url = super::validated_resource_url(&raw, &[origin.as_str()], loopback)?;
        Some((url, raw))
    }
}
