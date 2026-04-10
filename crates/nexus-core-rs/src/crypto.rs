//! Cryptographic primitives for SBFB.
//!
//! This module wraps the raw `ed25519-dalek` and `blake3` crates
//! behind a small, stable API that the rest of nexus-core-rs and
//! the Python bindings call into. The goal is to:
//!
//! - Keep all Ed25519 signing/verification through a single code
//!   path so Sprint 4's kudos ledger, the coordinator's invite
//!   link generator and the worker's task result signer all share
//!   the exact same bytes format.
//! - Provide a BLAKE3 chain-hash primitive for append-only
//!   ledgers (kudos per-project, audit log).
//! - Be completely self-contained: nothing in here touches iroh,
//!   tokio, or any async runtime. All functions are synchronous
//!   and `Send + Sync`.
//!
//! ## Key format
//!
//! Both secret and public keys are 32-byte arrays. We expose them
//! as `[u8; 32]` publicly and convert to the dalek types internally
//! so callers (especially the Python side) never see dalek types.
//!
//! ## Signature format
//!
//! 64 bytes (Ed25519 standard). Exposed as `[u8; 64]`.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use blake3::Hasher as Blake3Hasher;
use ed25519_dalek::{
    Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey, SECRET_KEY_LENGTH,
    SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::{NexusError, Result};

/// Length of a public key in bytes.
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// Length of a secret key in bytes.
pub const SECRET_KEY_BYTES: usize = SECRET_KEY_LENGTH;

/// Length of an Ed25519 signature in bytes.
pub const SIGNATURE_BYTES: usize = SIGNATURE_LENGTH;

/// Length of a BLAKE3 hash in bytes (standard 32-byte digest).
pub const BLAKE3_BYTES: usize = 32;

/// A SBFB signing key. Wraps an Ed25519 secret key.
///
/// Keep this value private. If it leaks, an attacker can sign
/// arbitrary tasks/results/invites as if they were you, and the
/// only recovery is key rotation.
#[derive(Debug, Clone)]
pub struct KeyPair {
    signing: SigningKey,
}

impl KeyPair {
    /// Generate a fresh random keypair using the OS RNG.
    pub fn generate() -> Self {
        let mut secret = [0u8; SECRET_KEY_BYTES];
        OsRng.fill_bytes(&mut secret);
        let signing = SigningKey::from_bytes(&secret);
        KeyPair { signing }
    }

    /// Load a keypair from a 32-byte secret key.
    ///
    /// The public key is derived, no separate storage needed.
    pub fn from_secret_bytes(bytes: &[u8; SECRET_KEY_BYTES]) -> Self {
        let signing = SigningKey::from_bytes(bytes);
        KeyPair { signing }
    }

    /// Return the 32-byte secret key for storage.
    ///
    /// Callers should write this to an OS-level secret store
    /// (keyring) in production and a permission-restricted file
    /// in dev. Never serialize to a doc/gossip/blobs channel.
    pub fn secret_bytes(&self) -> [u8; SECRET_KEY_BYTES] {
        self.signing.to_bytes()
    }

    /// Return the 32-byte public key for sharing.
    pub fn public_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.signing.verifying_key().to_bytes()
    }

    /// Sign an arbitrary message and return the 64-byte signature.
    pub fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        self.signing.sign(message).to_bytes()
    }

    /// Load a keypair from a file, or generate a fresh one and
    /// persist it if the file doesn't exist.
    ///
    /// The file format is a raw 32-byte binary blob — no ASCII
    /// encoding, no newline, just the secret key bytes. The file
    /// is created with 0600 permissions on Unix (owner read/write
    /// only). On Windows the default ACL is used because Rust's
    /// std `fs::Permissions` does not expose Windows ACLs.
    ///
    /// This is the canonical way for coordinators and workers to
    /// maintain a stable Ed25519 identity across restarts.
    pub fn load_or_generate(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        match fs::read(path) {
            Ok(bytes) => {
                if bytes.len() != SECRET_KEY_BYTES {
                    return Err(NexusError::Crypto(format!(
                        "key file {} has {} bytes, expected {}",
                        path.display(),
                        bytes.len(),
                        SECRET_KEY_BYTES,
                    )));
                }
                let mut secret = [0u8; SECRET_KEY_BYTES];
                secret.copy_from_slice(&bytes);
                Ok(KeyPair::from_secret_bytes(&secret))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let kp = KeyPair::generate();
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent).map_err(NexusError::Io)?;
                    }
                }
                fs::write(path, kp.secret_bytes()).map_err(NexusError::Io)?;
                set_owner_only_perms(path).ok(); // best-effort on Unix
                Ok(kp)
            }
            Err(e) => Err(NexusError::Io(e)),
        }
    }
}

/// Tighten file permissions to owner-read/write on Unix. No-op on
/// Windows (the default ACL inherits from the parent directory).
#[cfg(unix)]
fn set_owner_only_perms(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_owner_only_perms(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Verify an Ed25519 signature against a public key and message.
///
/// Returns `Ok(())` on valid signature, or
/// [`NexusError::Crypto`] with a descriptive reason on any
/// failure (bad key length, bad signature length, signature
/// does not match).
pub fn verify(
    public_key: &[u8; PUBLIC_KEY_LENGTH],
    message: &[u8],
    signature: &[u8; SIGNATURE_BYTES],
) -> Result<()> {
    let verifying = VerifyingKey::from_bytes(public_key)
        .map_err(|e| NexusError::Crypto(format!("bad public key: {e}")))?;
    let sig = DalekSignature::from_bytes(signature);
    verifying
        .verify(message, &sig)
        .map_err(|e| NexusError::Crypto(format!("signature verification failed: {e}")))
}

/// Compute the BLAKE3 hash of a single byte slice.
///
/// Returns the 32-byte digest. For hashing multiple chunks
/// efficiently, use [`Blake3Chain`] directly.
pub fn blake3_hash(data: &[u8]) -> [u8; BLAKE3_BYTES] {
    *blake3::hash(data).as_bytes()
}

/// Append-only hash chain, used by the SBFB kudos ledger.
///
/// Starts from an optional genesis hash (defaulting to all zeros)
/// and produces a new hash for each entry by concatenating the
/// previous hash with the new entry bytes and hashing the result.
///
/// Formally: `H_0 = genesis`, `H_{i+1} = BLAKE3(H_i || entry_i)`.
///
/// This gives each entry a position-dependent hash that depends
/// on every prior entry, so tampering with any past entry
/// invalidates every subsequent hash — the standard append-only
/// ledger integrity property.
#[derive(Debug, Clone)]
pub struct Blake3Chain {
    head: [u8; BLAKE3_BYTES],
}

impl Blake3Chain {
    /// Start a new chain from the zero-hash genesis.
    pub fn new() -> Self {
        Blake3Chain {
            head: [0u8; BLAKE3_BYTES],
        }
    }

    /// Start a chain from an explicit genesis hash (e.g. to
    /// resume an existing ledger from disk).
    pub fn from_head(head: [u8; BLAKE3_BYTES]) -> Self {
        Blake3Chain { head }
    }

    /// Return the current head hash.
    pub fn head(&self) -> [u8; BLAKE3_BYTES] {
        self.head
    }

    /// Append an entry, advancing the chain. Returns the new head.
    pub fn append(&mut self, entry: &[u8]) -> [u8; BLAKE3_BYTES] {
        let mut hasher = Blake3Hasher::new();
        hasher.update(&self.head);
        hasher.update(entry);
        self.head = *hasher.finalize().as_bytes();
        self.head
    }
}

impl Default for Blake3Chain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generate_is_random() {
        let a = KeyPair::generate();
        let b = KeyPair::generate();
        assert_ne!(a.secret_bytes(), b.secret_bytes());
        assert_ne!(a.public_bytes(), b.public_bytes());
    }

    #[test]
    fn sign_then_verify_roundtrip() {
        let kp = KeyPair::generate();
        let msg = b"hello SBFB";
        let sig = kp.sign(msg);
        verify(&kp.public_bytes(), msg, &sig).expect("valid signature");
    }

    #[test]
    fn verify_rejects_wrong_message() {
        let kp = KeyPair::generate();
        let sig = kp.sign(b"original");
        let result = verify(&kp.public_bytes(), b"tampered", &sig);
        assert!(result.is_err());
    }

    #[test]
    fn verify_rejects_wrong_public_key() {
        let kp = KeyPair::generate();
        let other = KeyPair::generate();
        let msg = b"message";
        let sig = kp.sign(msg);
        let result = verify(&other.public_bytes(), msg, &sig);
        assert!(result.is_err());
    }

    #[test]
    fn from_secret_bytes_recovers_identical_keypair() {
        let original = KeyPair::generate();
        let restored = KeyPair::from_secret_bytes(&original.secret_bytes());
        assert_eq!(original.public_bytes(), restored.public_bytes());

        // Signatures from the same secret key are deterministic
        // in Ed25519, so we can byte-compare them directly.
        let msg = b"reproducibility test";
        assert_eq!(original.sign(msg), restored.sign(msg));
    }

    #[test]
    fn blake3_hash_is_deterministic() {
        let data = b"the quick brown fox";
        let h1 = blake3_hash(data);
        let h2 = blake3_hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn blake3_hash_changes_on_input() {
        let h1 = blake3_hash(b"a");
        let h2 = blake3_hash(b"b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn chain_starts_at_zero_genesis() {
        let chain = Blake3Chain::new();
        assert_eq!(chain.head(), [0u8; BLAKE3_BYTES]);
    }

    #[test]
    fn chain_advances_on_append() {
        let mut chain = Blake3Chain::new();
        let genesis = chain.head();
        let h1 = chain.append(b"entry 1");
        assert_ne!(h1, genesis);

        let h2 = chain.append(b"entry 2");
        assert_ne!(h2, h1);
    }

    #[test]
    fn chain_is_reproducible_from_same_sequence() {
        let mut a = Blake3Chain::new();
        a.append(b"x");
        a.append(b"y");
        a.append(b"z");

        let mut b = Blake3Chain::new();
        b.append(b"x");
        b.append(b"y");
        b.append(b"z");

        assert_eq!(a.head(), b.head());
    }

    #[test]
    fn chain_detects_order_tampering() {
        let mut a = Blake3Chain::new();
        a.append(b"alice");
        a.append(b"bob");

        let mut b = Blake3Chain::new();
        b.append(b"bob");
        b.append(b"alice");

        // Swapping entry order MUST produce a different head.
        assert_ne!(a.head(), b.head());
    }

    #[test]
    fn chain_from_head_resumes_ledger() {
        let mut original = Blake3Chain::new();
        original.append(b"first");
        let checkpoint = original.head();
        original.append(b"second");
        let final_head = original.head();

        // Simulate restart: load from checkpoint, replay "second"
        let mut resumed = Blake3Chain::from_head(checkpoint);
        resumed.append(b"second");

        assert_eq!(resumed.head(), final_head);
    }

    #[test]
    fn load_or_generate_creates_new_file_on_first_call() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("subdir").join("key.bin");
        assert!(!path.exists());

        let kp = KeyPair::load_or_generate(&path).expect("first call creates");
        assert!(path.exists());
        let written = fs::read(&path).unwrap();
        assert_eq!(written.len(), SECRET_KEY_BYTES);
        assert_eq!(written, kp.secret_bytes().as_ref());
    }

    #[test]
    fn load_or_generate_reads_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("key.bin");

        let first = KeyPair::load_or_generate(&path).unwrap();
        let second = KeyPair::load_or_generate(&path).unwrap();

        assert_eq!(first.public_bytes(), second.public_bytes());
        // Deterministic Ed25519: same secret = same signatures.
        let msg = b"persistence check";
        assert_eq!(first.sign(msg), second.sign(msg));
    }

    #[test]
    fn load_or_generate_rejects_wrong_size_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.bin");
        fs::write(&path, b"short").unwrap();

        let err = KeyPair::load_or_generate(&path).unwrap_err();
        matches!(err, NexusError::Crypto(_));
    }
}
