//! The local store's key of each account: made
//! at random the first time, kept in the system keyring, never in the file,
//! and deleted when the account is forgotten. Asked off the interface's
//! thread; without a keyring the account's data stays in memory.

use std::sync::Arc;
use std::time::Duration;

use rustcord_platform::secrets::{Saved, SecretError, SecretStore, CACHE_KEYS, FORMER_CACHE_KEYS};
use rustcord_storage::Key;

/// Bounds a keyring that does not answer; an unlock prompt waits this long.
const TIMEOUT: Duration = Duration::from_secs(30);

/// What the keyring kept of an account: its key, whether it was made just
/// now, and the keys of its former names. A former whose answer failed is
/// left out: neither moved nor forgotten this time.
pub(crate) struct Fetched {
    pub key: Key,
    pub fresh: bool,
    pub former: Vec<(String, Option<Key>)>,
}

/// What the keyring answered about an account, by its name in the store.
pub(crate) enum KeyEvent {
    Fetched(String, Result<Fetched, SecretError>),
    Deleted(Result<(), SecretError>),
}

/// `account`'s key, made and saved the first time, and whether it was made
/// now, with the keys of `former` names. A keyring that fails gives none: a
/// new one would make the rows kept so far unreadable. Without a key of its
/// own, the account keeps the first former key found, so its rows open.
pub(crate) async fn fetch(
    keyring: Arc<dyn SecretStore>,
    account: String,
    former: Vec<String>,
) -> Result<Fetched, SecretError> {
    let fetch = async {
        let saved = load(keyring.as_ref(), &account).await?;
        let mut former_keys = Vec::new();
        for name in former {
            if let Ok(saved) = load(keyring.as_ref(), &name).await {
                let key = saved.and_then(|saved| Key::from_text(&saved.secret));
                former_keys.push((name, key));
            }
        }
        if let Some(key) = saved.and_then(|saved| Key::from_text(&saved.secret)) {
            return Ok(Fetched {
                key,
                fresh: false,
                former: former_keys,
            });
        }
        if let Some(key) = former_keys.iter().find_map(|(_, key)| key.clone()) {
            keyring
                .save(CACHE_KEYS, &account, "", "", &key.to_text())
                .await?;
            return Ok(Fetched {
                key,
                fresh: false,
                former: former_keys,
            });
        }
        // None yet, or one that no longer reads: what it sealed is lost
        // either way, and the store drops it on unlocking with this one.
        let key = Key::generate().ok_or(SecretError::FAILED)?;
        keyring
            .save(CACHE_KEYS, &account, "", "", &key.to_text())
            .await?;
        Ok(Fetched {
            key,
            fresh: true,
            former: former_keys,
        })
    };
    tokio::time::timeout(TIMEOUT, fetch)
        .await
        .unwrap_or(Err(SecretError::FAILED))
}

/// `account`'s key under [`CACHE_KEYS`], or under a former name of that
/// service, from where it moves: saved under the current one, then let go
/// of. A move that fails leaves it where it was, read from there next time.
async fn load(keyring: &dyn SecretStore, account: &str) -> Result<Option<Saved>, SecretError> {
    if let Some(saved) = keyring.load(CACHE_KEYS, account).await? {
        return Ok(Some(saved));
    }
    for former in FORMER_CACHE_KEYS {
        let Some(saved) = keyring.load(former, account).await? else {
            continue;
        };
        if keyring
            .save(CACHE_KEYS, account, "", "", &saved.secret)
            .await
            .is_ok()
        {
            let _ = keyring.delete(former, account).await;
        }
        return Ok(Some(saved));
    }
    Ok(None)
}

/// Forget `account`'s key, under the service's current name and its former.
pub(crate) async fn delete(
    keyring: Arc<dyn SecretStore>,
    account: &str,
) -> Result<(), SecretError> {
    let delete = async {
        for former in FORMER_CACHE_KEYS {
            keyring.delete(former, account).await?;
        }
        keyring.delete(CACHE_KEYS, account).await
    };
    tokio::time::timeout(TIMEOUT, delete)
        .await
        .unwrap_or(Err(SecretError::FAILED))
}
