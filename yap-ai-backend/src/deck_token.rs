//! Signed per-deck tokens for exported Anki decks.
//!
//! Every generated deck embeds media URLs of the form
//! `https://clips.yap.town/<lang>/<clip>/lo.mp4?d=<token>`. The token names
//! the deck (a fresh id) and carries an HMAC over that id under a secret
//! only the backend holds, so a token can't be forged and a leaked deck can
//! be traced to one mint and, later, revoked individually. Today the clip
//! domain is a plain public bucket that ignores the query string, so the
//! tokens are inert; a Worker that validates them with [`verify`] and
//! consults a denylist can be put in front of the bucket without touching
//! any deck already in the wild. That future is the whole reason the token
//! is signed now rather than being a bare random id: an unsigned scheme
//! could never tell a mint from a forgery after the fact.
//!
//! Wire format: base64url (no padding) of the 16 id bytes followed by the
//! first 16 bytes of `HMAC-SHA256(secret, id)` — 43 characters, safe in a
//! URL query and in an Anki note field. Truncating the tag to 128 bits is
//! standard and leaves forgery infeasible.

use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::sync::LazyLock;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const TAG_LEN: usize = 16;

static SECRET: LazyLock<Option<Vec<u8>>> = LazyLock::new(|| {
    std::env::var("DECK_TOKEN_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .map(String::into_bytes)
});

/// Whether this process holds the signing secret. Without it no deck can be
/// minted; verification is likewise impossible.
pub fn configured() -> bool {
    SECRET.is_some()
}

fn mac(secret: &[u8], deck_id: Uuid) -> HmacSha256 {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(deck_id.as_bytes());
    mac
}

fn encode(secret: &[u8], deck_id: Uuid) -> String {
    let tag = mac(secret, deck_id).finalize().into_bytes();
    let mut raw = Vec::with_capacity(16 + TAG_LEN);
    raw.extend_from_slice(deck_id.as_bytes());
    raw.extend_from_slice(&tag[..TAG_LEN]);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
}

fn decode(secret: &[u8], token: &str) -> Option<Uuid> {
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .ok()?;
    let (id, tag) = raw.split_at_checked(16)?;
    if tag.len() != TAG_LEN {
        return None;
    }
    let deck_id = Uuid::from_slice(id).ok()?;
    // Constant-time comparison of the truncated tag.
    mac(secret, deck_id).verify_truncated_left(tag).ok()?;
    Some(deck_id)
}

/// A fresh deck id and its token, or `None` when no secret is configured.
pub fn mint() -> Option<(Uuid, String)> {
    let secret = SECRET.as_deref()?;
    let deck_id = Uuid::new_v4();
    let token = encode(secret, deck_id);
    Some((deck_id, token))
}

/// The deck a token was minted for, if it was minted by us and is intact.
/// Not called by anything yet — it is the check the clip-domain Worker will
/// run, kept next to `mint` so the two can't drift.
#[cfg_attr(not(test), expect(dead_code))]
pub fn verify(token: &str) -> Option<Uuid> {
    decode(SECRET.as_deref()?, token)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-secret";

    #[test]
    fn round_trips() {
        let deck_id = Uuid::new_v4();
        let token = encode(SECRET, deck_id);
        assert_eq!(token.len(), 43);
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "token must be safe in a URL query without escaping: {token}"
        );
        assert_eq!(decode(SECRET, &token), Some(deck_id));
    }

    #[test]
    fn rejects_tampering_and_other_secrets() {
        let deck_id = Uuid::new_v4();
        let token = encode(SECRET, deck_id);

        // Flip one character of the tag.
        let mut chars: Vec<char> = token.chars().collect();
        let last = chars.len() - 1;
        chars[last] = if chars[last] == 'A' { 'B' } else { 'A' };
        let tampered: String = chars.into_iter().collect();
        assert_eq!(decode(SECRET, &tampered), None);

        // A token for one id can't be re-pointed at another.
        let other = Uuid::new_v4();
        let mut raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&token)
            .unwrap();
        raw[..16].copy_from_slice(other.as_bytes());
        let swapped = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);
        assert_eq!(decode(SECRET, &swapped), None);

        assert_eq!(decode(b"another-secret", &token), None);
        assert_eq!(decode(SECRET, ""), None);
        assert_eq!(decode(SECRET, "not base64!"), None);
    }

    #[test]
    fn unconfigured_process_mints_nothing() {
        // The env var is not set under `cargo test`, so the process-wide
        // secret is absent and both halves refuse rather than sign with
        // something predictable.
        if !configured() {
            assert!(mint().is_none());
            assert!(verify(&encode(SECRET, Uuid::new_v4())).is_none());
        }
    }
}
