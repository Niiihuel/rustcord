//! Ephemeral attachment references, independent of files and transports.

use crate::{ImageKind, RemoteImage};

/// A process-local attachment grant. It never identifies a remote attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UploadToken(pub u64);

/// What names a prepared file's picture: no provider's address starts so.
const PREVIEW: &str = "rustcord-upload:";

impl UploadToken {
    /// The picture of the file prepared under this token, decoded on this
    /// device from its bytes, never fetched.
    pub fn preview(self) -> RemoteImage {
        RemoteImage {
            url: format!("{PREVIEW}{}", self.0),
            kind: ImageKind::AttachmentPreview,
        }
    }

    /// The token whose [`Self::preview`] `image` is.
    pub fn previewed(image: &RemoteImage) -> Option<Self> {
        if image.kind != ImageKind::AttachmentPreview {
            return None;
        }
        image.url.strip_prefix(PREVIEW)?.parse().ok().map(Self)
    }
}

/// The only selected-file data retained by presentation and domain state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingAttachment {
    pub token: UploadToken,
    pub filename: String,
    pub size_bytes: u64,
}

impl PendingAttachment {
    /// Its picture, when its name says it is one Rustcord draws (a pasted
    /// picture is a PNG); one that is not fails to decode and keeps its glyph.
    pub fn preview(&self) -> Option<RemoteImage> {
        let (_, extension) = self.filename.rsplit_once('.')?;
        ["png", "jpg", "jpeg", "gif", "webp"]
            .iter()
            .any(|picture| extension.eq_ignore_ascii_case(picture))
            .then(|| self.token.preview())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pictures_have_a_preview_and_it_names_its_token_back() {
        let file = |name: &str| PendingAttachment {
            token: UploadToken(42),
            filename: name.into(),
            size_bytes: 1,
        };
        let preview = file("imagen.PNG").preview().expect("a picture");
        assert_eq!(UploadToken::previewed(&preview), Some(UploadToken(42)));
        for name in ["notas.txt", "png", "audio.mp3"] {
            assert_eq!(file(name).preview(), None, "{name}");
        }
        // An address from elsewhere never names a prepared file.
        for image in [
            RemoteImage {
                url: "https://cdn.example/rustcord-upload:42".into(),
                kind: ImageKind::AttachmentPreview,
            },
            RemoteImage {
                kind: ImageKind::Artwork,
                ..preview
            },
        ] {
            assert_eq!(UploadToken::previewed(&image), None);
        }
    }
}
