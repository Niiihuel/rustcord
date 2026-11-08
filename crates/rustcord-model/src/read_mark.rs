use serde::{Deserialize, Serialize};

use crate::{MessageId, Timestamp};

/// An account-local reading choice. A reminder never rewinds a provider cursor.
/// Revisions order local choices, not provider confirmations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadMark {
    #[serde(default)]
    pub through: Option<MessageId>,
    /// The exact message chosen by the user; it may no longer be available.
    #[serde(default)]
    pub unread_from: Option<MessageId>,
    #[serde(default)]
    pub revision: u64,
    /// The newest pin the account saw when it opened the channel's pins;
    /// zero for none. Not an `Option`: the restore's memory credit counts
    /// every mark's size.
    #[serde(default)]
    pub pins_seen: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_kept_before_pins_were_seen_still_reads() {
        let mark: ReadMark = serde_json::from_str(r#"{"through":7,"revision":2}"#).unwrap();
        assert_eq!(mark.through, Some(MessageId(7)));
        assert_eq!(mark.pins_seen, Timestamp(0));
    }
}
