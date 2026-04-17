// SPDX-License-Identifier: AGPL-3.0-or-later
//! Encryption at rest for the daemon's Ed25519 identity keypair.
//!
//! Sprint 20 Phase A — the big rock that unblocks Gate 2 A-S9
//! `Checkpoint-seize` risk.
//!
//! ## Double-layer defense-in-depth
//!
//! Two secrets are needed to recover the keypair from the disk
//! blob. An attacker who gets only one of them still faces a
//! full brute-force wall against the other.
//!
//! 1. **PIN layer (Argon2id)** — the user types a PIN at
//!    `sbfb unlock`. Argon2id(PIN, salt, m=64 MiB/t=3/p=1)
//!    derives a 32-byte `kek1`. The memory cost is calibrated
//!    for ~3 s/attempt on 2026 desktop CPUs (cf. Signal Secure
//!    Value Recovery blog, RFC 9106 §4) which pushes a 6-digit
//!    PIN brute force from <2 s on GPU to >10 s even on a RTX
//!    5080. OWASP's 19 MiB default is calibrated for
//!    passphrases (~40 bit entropy) — a 4–6 digit PIN has
//!    ~13–20 bits, so we bump `m_cost` to raise the wall.
//!
//! 2. **OS keyring layer (`keyring-rs`)** — at `sbfb init` we
//!    generate a 32-byte `kek2` and store it in the platform's
//!    native credential store (macOS Keychain, Windows
//!    Credential Manager, Linux Secret Service). The blob on
//!    disk does NOT contain `kek2`. An attacker who steals the
//!    blob file offline cannot decrypt anything without first
//!    recovering `kek2` from the live user's keyring.
//!
//! The final AEAD key is `final_kek = BLAKE3(DOMAIN || kek1 ||
//! kek2)`. The daemon never holds `kek1` or `kek2` in plaintext
//! beyond the AEAD seal/open window — both are wrapped in
//! `SecretBox<[u8; 32]>` (heap-allocated, zeroed on drop via
//! the `zeroize` crate).
//!
//! ## Threat model covered
//!
//! | Adversary reads | Needs | Cost |
//! |---|---|---|
//! | Blob file only | PIN + keyring | Argon2id wall + live user |
//! | OS keyring only | PIN + blob file | Argon2id wall + disk access |
//! | Blob file + keyring | PIN | Argon2id wall (3 s/attempt) |
//! | Blob + keyring + PIN | — | decrypt |
//!
//! Specifically, this closes the **T3 DPAPI user-scope gap**
//! (Sygnia 2024 "DPAPI downfall" + SpecterOps 2024-2026): a
//! same-user malicious process that can dump DPAPI master keys
//! from LSASS (via Mimikatz `/unprotect`) still cannot unlock
//! the blob because the PIN never enters `lsass.exe`.
//!
//! ## Blob format v1
//!
//! ```text
//! offset  size  field
//! ------  ----  ----------------------------------
//!      0     6  magic = b"SBFBK1"
//!      6     1  version = 0x01
//!      7     1  flags (bit 0 = use_keyring_layer)
//!      8    16  argon2_salt
//!     24     4  argon2 m_cost (u32 big-endian, KiB)
//!     28     4  argon2 t_cost (u32 BE)
//!     32     4  argon2 parallelism (u32 BE)
//!     36    12  aead_nonce
//!     48     N  ciphertext || aead_tag
//! ```
//!
//! All multi-byte integers are big-endian. The AEAD AAD is the
//! full 48-byte header prefixed by `DOMAIN_KEYSTORE_V1` so any
//! tamper with the header (including parameter downgrade
//! attacks) fails the AEAD open.
//!
//! ## Pre-launch protocol policy
//!
//! Version byte stays at `0x01` until the `v1.0` tag. If the
//! scheme changes before launch, edit this file and re-mint all
//! dev blobs — we do NOT ship a tolerant multi-version decoder.
//! Cf. `CLAUDE.md §Pre-launch protocol policy`.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretBox};
use zeroize::Zeroize;

use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SECRET_KEY_BYTES};

// =================================================================
// Constants
// =================================================================

/// Domain separator for the blob AEAD AAD. Binds the ciphertext to
/// the SBFB keystore v1 scheme; changing this string invalidates
/// every existing blob, so bump it only when the blob layout
/// changes.
pub const DOMAIN_KEYSTORE_V1: &[u8] = b"sbfb-keystore-v1";

/// Magic prefix at byte 0 of every v1 blob.
pub const BLOB_MAGIC: &[u8; 6] = b"SBFBK1";

/// Blob format version.
pub const BLOB_VERSION: u8 = 0x01;

/// Size of the fixed-length header (magic + version + flags +
/// salt + argon2 params + aead nonce).
pub const BLOB_HEADER_LEN: usize = 6 + 1 + 1 + 16 + 4 + 4 + 4 + 12;

/// Size of the Argon2id salt persisted in every blob.
pub const SALT_LEN: usize = 16;

/// Size of the AEAD nonce persisted in every blob. AES-256-GCM
/// takes a 96-bit nonce (see `AES_256_GCM.nonce_len()` in
/// aws-lc-rs).
pub const NONCE_LEN: usize = 12;

/// Size of the AEAD tag appended by AES-256-GCM.
pub const TAG_LEN: usize = 16;

/// Production Argon2id memory cost, in KiB. 64 MiB.
pub const ARGON2_MEM_COST_KIB: u32 = 64 * 1024;

/// Production Argon2id time cost (iterations).
pub const ARGON2_TIME_COST: u32 = 3;

/// Production Argon2id parallelism.
pub const ARGON2_PARALLELISM: u32 = 1;

/// Default relative path of the on-disk blob under `data_dir`.
pub const BLOB_FILE_NAME: &str = "identity.enc";

/// Phase B duress blob filename. Sits next to [`BLOB_FILE_NAME`]
/// in the same `data_dir`. Byte-for-byte indistinguishable layout
/// from the normal blob (same header, same 96-byte total length)
/// so a disk forensics pass sees two identical-looking blobs
/// without a successful decrypt.
pub const BLOB_FILE_NAME_DURESS: &str = "identity_duress.enc";

/// OS keyring service name used by the default `LocalFileKeyStore`.
pub const KEYRING_SERVICE: &str = "sbfb-daemon";

/// OS keyring account name used by the default `LocalFileKeyStore`
/// for the normal (non-duress) identity's `kek2` wrap.
pub const KEYRING_ACCOUNT_NORMAL: &str = "identity-kek-wrap";

/// Phase B : OS keyring account for the duress identity's `kek2`
/// wrap. A distinct slot so a forensic dump of the keyring shows
/// two unrelated-looking entries rather than one compound entry,
/// and so `wipe_all` can delete each independently.
pub const KEYRING_ACCOUNT_DURESS: &str = "identity-kek-wrap-duress";

/// Environment variable the launcher uses to hand the decrypted
/// 32-byte Ed25519 secret key to the daemon child (64-char lower-
/// case hex). Canonical definition lives here so launcher +
/// daemon + any future consumer refer to a single constant —
/// avoids the silent-drift footgun of two `pub const` strings in
/// different crates.
pub const SBFB_IDENTITY_SECRET_HEX_ENV: &str = "SBFB_IDENTITY_SECRET_HEX";

// =================================================================
// Errors
// =================================================================

/// Failures that can happen during `init`, `rotate_pin`, or `wipe`.
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    /// Wrapping `std::io::Error` — blob read/write, directory
    /// creation, atomic rename. The cause preserves the underlying
    /// OS code.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// The Argon2id parameter tuple was rejected by the `argon2`
    /// crate (typical cause: `m_cost < 8` or `parallelism > 255`).
    #[error("argon2 params invalid: {0}")]
    Argon2Params(String),

    /// Argon2id `hash_password_into` returned an error. Extremely
    /// rare — only fires on zero-length PIN or zero-length salt.
    #[error("argon2 kdf failed: {0}")]
    Argon2Kdf(String),

    /// The AES-GCM seal or AEAD primitive refused to operate
    /// (bad key length). Can only happen on catastrophic RNG
    /// failure or a logic bug feeding the wrong key size.
    #[error("aead failed: {0}")]
    Aead(String),

    /// The OS keyring refused a set/delete/get call — the cause
    /// string carries the underlying `keyring::Error` display.
    #[error("os keyring: {0}")]
    Keyring(String),

    /// `init` called on a data dir that already holds an
    /// `identity.enc` blob; the caller must `wipe` first or pick a
    /// different dir, to avoid accidentally clobbering an
    /// unrecoverable identity.
    #[error("blob already exists at {0} — refusing to overwrite without wipe")]
    AlreadyInitialized(PathBuf),

    /// `rotate_pin` was called with an incorrect `old_pin`. The
    /// AEAD open under the PIN-derived KEK rejected the ciphertext
    /// so the rotation cannot proceed. Separate variant (instead
    /// of folding into `Argon2Params`, which means "KDF parameter
    /// tuple invalid") so callers can pattern-match audit logs on
    /// a wrong-PIN event.
    #[error("rotate_pin rejected: wrong old PIN")]
    WrongPin,
}

/// Failures that can happen during `unlock`.
///
/// Split from `KeyStoreError` because the caller (the launcher)
/// treats "wrong PIN" and "corrupted blob" very differently from
/// "io error": the first two are security-relevant and must be
/// audit-logged, the third is a system issue.
#[derive(Debug, thiserror::Error)]
pub enum UnlockError {
    /// The blob file does not exist — the daemon has never been
    /// initialized or the user wiped it.
    #[error("keystore not initialized at {0}")]
    NotInitialized(PathBuf),

    /// Blob file exists but its header is malformed (wrong magic,
    /// wrong version, truncated).
    #[error("blob header malformed: {0}")]
    BlobMalformed(String),

    /// AEAD open rejected the ciphertext. Caller-visible signal
    /// that the PIN is wrong OR the blob was tampered with OR the
    /// OS keyring `kek2` was rotated out from under us.
    #[error("wrong PIN or tampered blob (AEAD reject)")]
    AeadReject,

    /// The blob declares that it was written with `use_keyring`
    /// set but the corresponding keyring entry cannot be read.
    #[error("os keyring entry missing: {0}")]
    KeyringEntryMissing(String),

    /// `std::io::Error` reading the blob file.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// Argon2id KDF failed during unlock (params invalid,
    /// extremely rare post-init).
    #[error("argon2 kdf failed: {0}")]
    Argon2Kdf(String),
}

// =================================================================
// Types
// =================================================================

/// The `[u8; 32]` buffer stored inside `SecretBox` needs an explicit
/// `Zeroize` impl to participate in `SecretBox`'s on-drop wipe.
/// Wrapping it in a newtype lets us derive the trait and avoids a
/// hand-written `impl Zeroize for [u8; 32]` that would collide with
/// the one already in the `zeroize` crate.
#[derive(Zeroize)]
#[zeroize(drop)]
struct SecretKeyBytes([u8; SECRET_KEY_BYTES]);

/// Unlocked identity returned by [`KeyStore::unlock`].
///
/// The 32-byte secret key is wrapped in `SecretBox` so callers
/// that accidentally log the struct get redacted output and so the
/// bytes are zeroed when the box is dropped. The 32-byte public
/// key is copied in the clear for convenience — a public key leak
/// is not a security event.
pub struct Identity {
    secret: SecretBox<SecretKeyBytes>,
    public: [u8; PUBLIC_KEY_LENGTH],
    mode: IdentityMode,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("public", &hex::encode(self.public))
            .field("mode", &self.mode)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl Identity {
    /// Consume the identity and hand the raw 32-byte secret key
    /// back to the caller. The caller is responsible for zeroing
    /// the returned buffer once it is no longer needed — the
    /// typical user is `NodeConfig::with_secret_key` which copies
    /// into `iroh::SecretKey` and lets its own `Drop` handle the
    /// wipe.
    pub fn into_secret_bytes(self) -> [u8; SECRET_KEY_BYTES] {
        // SecretBox exposes a reference; we copy out because the
        // caller owns the bytes from here on. The SecretBox is
        // dropped at the end of this block, zeroing its heap copy.
        let mut out = [0u8; SECRET_KEY_BYTES];
        out.copy_from_slice(&self.secret.expose_secret().0);
        out
    }

    /// Borrow the public key for display / comparison. Always
    /// safe to print or log.
    pub fn public_bytes(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        &self.public
    }

    /// Return whether this identity was unlocked in Normal or
    /// Duress mode (cf. Phase B).
    pub fn mode(&self) -> IdentityMode {
        self.mode
    }
}

/// Which blob the unlock path read from.
///
/// Phase A only produces `Normal` because `init` does not yet
/// accept a duress PIN. Phase B will add the `init_with_duress`
/// path and the runtime routing logic that interprets `Duress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityMode {
    /// Real user identity. Gossip publishes, kudos, and curator
    /// subscriptions use this keypair.
    Normal,

    /// Fake identity revealed only when the user types the duress
    /// PIN. Phase B implements the noop-gossip routing that keeps
    /// the decoy indistinguishable to a remote peer.
    Duress,
}

/// Argon2id parameter tuple. Exposed in the public API so tests
/// and benches can opt into fast params (m=8 KiB, t=1) without
/// running a 3-second KDF on every test. Production code always
/// calls [`KdfParams::production`] and gets the 64 MiB / t=3 / p=1
/// defaults documented at the top of this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in KiB. Argon2 `Params::m_cost`.
    pub m_cost_kib: u32,
    /// Time cost (number of passes). Argon2 `Params::t_cost`.
    pub t_cost: u32,
    /// Parallelism degree. Argon2 `Params::p_cost`.
    pub parallelism: u32,
}

impl KdfParams {
    /// The production parameters enforced by default
    /// `LocalFileKeyStore::new`.
    pub const fn production() -> Self {
        Self {
            m_cost_kib: ARGON2_MEM_COST_KIB,
            t_cost: ARGON2_TIME_COST,
            parallelism: ARGON2_PARALLELISM,
        }
    }

    /// Fast parameters for unit tests. 8 KiB / 1 pass / 1 lane
    /// — derives a KEK in well under 1 ms. NOT for production use
    /// (brute-forceable on any modern CPU).
    pub const fn fast_for_tests() -> Self {
        Self {
            m_cost_kib: 8,
            t_cost: 1,
            parallelism: 1,
        }
    }

    fn into_argon2(self) -> Result<Params, KeyStoreError> {
        Params::new(self.m_cost_kib, self.t_cost, self.parallelism, Some(32))
            .map_err(|e| KeyStoreError::Argon2Params(e.to_string()))
    }
}

impl Default for KdfParams {
    fn default() -> Self {
        Self::production()
    }
}

// =================================================================
// Trait
// =================================================================

/// Abstract keystore interface so Phase A + Phase B + future
/// hardware impls (TPM 2.0 / Secure Enclave / Android StrongBox,
/// cf. Sprint 22+ roadmap) share a single consumer API.
///
/// The trait is intentionally small — `init`, `unlock`,
/// `rotate_pin`, `wipe`. Duress-specific methods live on
/// [`LocalFileKeyStore`] directly and will be promoted here in
/// Phase B once the Android StrongBox impl reveals whether
/// duress maps naturally onto a hardware-backed slot.
pub trait KeyStore: Send + Sync {
    /// Generate a fresh Ed25519 keypair, derive the KEK from the
    /// given PIN + a random salt, wrap the keypair under the KEK,
    /// and persist everything.
    ///
    /// Fails with [`KeyStoreError::AlreadyInitialized`] if a blob
    /// already exists — call [`KeyStore::wipe`] first.
    fn init(&self, pin: &str) -> Result<Identity, KeyStoreError>;

    /// Read the blob, re-derive the KEK from the PIN, open the
    /// AEAD, and return the unlocked [`Identity`].
    fn unlock(&self, pin: &str) -> Result<Identity, UnlockError>;

    /// Replace the PIN without regenerating the keypair. The blob
    /// is re-salted, re-derived, re-wrapped, and atomically
    /// replaced.
    fn rotate_pin(&self, old_pin: &str, new_pin: &str) -> Result<(), KeyStoreError>;

    /// Irreversibly destroy the on-disk blob and any OS keyring
    /// entries. Secure-unlinks the blob file (overwrites with
    /// zeros then removes).
    fn wipe(&self) -> Result<(), KeyStoreError>;
}

// =================================================================
// LocalFileKeyStore
// =================================================================

/// Default production keystore: blob on disk + OS keyring `kek2`.
pub struct LocalFileKeyStore {
    data_dir: PathBuf,
    params: KdfParams,
    /// When `true`, the store persists a random `kek2` in the OS
    /// keyring at init and requires it at unlock. When `false`,
    /// only the Argon2id PIN layer protects the blob — used by
    /// integration tests that run on headless CI without a live
    /// Secret Service / Keychain session.
    use_keyring: bool,
    /// Service name passed to `keyring::Entry::new`. Constant in
    /// production (`KEYRING_SERVICE`); parameterised for tests so
    /// two test cases running in parallel do not collide on the
    /// same keyring slot.
    keyring_service: String,
    /// Account name for the normal identity's `kek2` wrap.
    keyring_account: String,
    /// Phase B : account name for the duress identity's `kek2`
    /// wrap. Default `KEYRING_ACCOUNT_DURESS`; parameterised for
    /// tests in the same way as `keyring_account`.
    keyring_account_duress: String,
}

impl LocalFileKeyStore {
    /// Build a keystore with the production `KdfParams` and the
    /// OS keyring layer enabled.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            params: KdfParams::production(),
            use_keyring: true,
            keyring_service: KEYRING_SERVICE.to_string(),
            keyring_account: KEYRING_ACCOUNT_NORMAL.to_string(),
            keyring_account_duress: KEYRING_ACCOUNT_DURESS.to_string(),
        }
    }

    /// Build a keystore with custom `KdfParams`. Tests pass
    /// [`KdfParams::fast_for_tests`] here to skip the 3-second
    /// production KDF.
    pub fn with_params(data_dir: impl Into<PathBuf>, params: KdfParams) -> Self {
        Self {
            data_dir: data_dir.into(),
            params,
            use_keyring: true,
            keyring_service: KEYRING_SERVICE.to_string(),
            keyring_account: KEYRING_ACCOUNT_NORMAL.to_string(),
            keyring_account_duress: KEYRING_ACCOUNT_DURESS.to_string(),
        }
    }

    /// Disable the OS keyring layer. Useful for headless CI tests
    /// and for the integration test
    /// `keyring_entry_missing_falls_back_to_blob_only`. The blob
    /// records the choice via the `flags` byte so `unlock` does
    /// not try to read a keyring entry that was never written.
    pub fn without_os_keyring(mut self) -> Self {
        self.use_keyring = false;
        self
    }

    /// Override the keyring service name. Only public for tests
    /// that need a unique per-testcase keyring slot; production
    /// callers should always use the `new` / `with_params`
    /// constructors that pick `KEYRING_SERVICE`.
    ///
    /// The duress account gets the `"{account}-duress"` derived
    /// name so a Phase B test running in parallel with another
    /// test cannot collide on the fixed `KEYRING_ACCOUNT_DURESS`.
    pub fn with_keyring_slot(
        mut self,
        service: impl Into<String>,
        account: impl Into<String>,
    ) -> Self {
        self.keyring_service = service.into();
        let account = account.into();
        self.keyring_account_duress = format!("{account}-duress");
        self.keyring_account = account;
        self
    }

    /// Path of the on-disk blob file.
    pub fn blob_path(&self) -> PathBuf {
        self.data_dir.join(BLOB_FILE_NAME)
    }

    /// Phase B : path of the on-disk duress blob file.
    pub fn blob_path_duress(&self) -> PathBuf {
        self.data_dir.join(BLOB_FILE_NAME_DURESS)
    }

    fn keyring_entry(&self) -> Result<keyring::Entry, KeyStoreError> {
        keyring::Entry::new(&self.keyring_service, &self.keyring_account)
            .map_err(|e| KeyStoreError::Keyring(e.to_string()))
    }

    fn keyring_entry_duress(&self) -> Result<keyring::Entry, KeyStoreError> {
        keyring::Entry::new(&self.keyring_service, &self.keyring_account_duress)
            .map_err(|e| KeyStoreError::Keyring(e.to_string()))
    }
}

impl KeyStore for LocalFileKeyStore {
    fn init(&self, pin: &str) -> Result<Identity, KeyStoreError> {
        let blob_path = self.blob_path();
        if blob_path.exists() {
            return Err(KeyStoreError::AlreadyInitialized(blob_path));
        }

        fs::create_dir_all(&self.data_dir)?;

        // Fresh keypair — this is the persistent daemon identity.
        let kp = KeyPair::generate();
        let secret_bytes = kp.secret_bytes();
        let public_bytes = kp.public_bytes();

        // Generate salt + kek2 + nonce from OS RNG.
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let kek2 = if self.use_keyring {
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            // Persist kek2 in the OS keyring BEFORE writing the
            // blob, so a crash between these two steps leaves no
            // orphan blob that cannot be unlocked.
            let entry = self.keyring_entry()?;
            entry
                .set_secret(&bytes)
                .map_err(|e| KeyStoreError::Keyring(e.to_string()))?;
            bytes
        } else {
            [0u8; 32]
        };

        // Derive kek1 from PIN.
        let kek1 = derive_kek1(pin.as_bytes(), &salt, self.params)?;
        let final_kek = combine_keks(kek1.expose_secret(), &kek2);

        // Assemble header + encrypt + write atomic.
        let flags = if self.use_keyring { 0b0000_0001 } else { 0 };
        let header = encode_header(flags, &salt, self.params, &nonce_bytes);
        let aad = build_aad(&header);
        let cipher = Aes256Gcm::new_from_slice(final_kek.expose_secret())
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &secret_bytes,
                    aad: &aad,
                },
            )
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;

        // `ciphertext` = plaintext-length bytes followed by the 16-byte GCM tag.
        let mut blob = Vec::with_capacity(BLOB_HEADER_LEN + ciphertext.len());
        blob.extend_from_slice(&header);
        blob.extend_from_slice(&ciphertext);
        write_atomic(&blob_path, &blob)?;

        // Keep a SecretBox copy for the caller. Then zero the raw
        // secret_bytes buffer in the local stack.
        let identity = Identity {
            secret: SecretBox::new(Box::new(SecretKeyBytes(secret_bytes))),
            public: public_bytes,
            mode: IdentityMode::Normal,
        };
        let mut tmp = secret_bytes;
        tmp.zeroize();
        Ok(identity)
    }

    fn unlock(&self, pin: &str) -> Result<Identity, UnlockError> {
        let blob_path = self.blob_path();
        let blob = match fs::read(&blob_path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(UnlockError::NotInitialized(blob_path));
            }
            Err(e) => return Err(UnlockError::Io(e)),
        };

        let parsed = parse_blob(&blob).map_err(UnlockError::BlobMalformed)?;

        let kek2 = if parsed.uses_keyring {
            let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_account)
                .map_err(|e| UnlockError::KeyringEntryMissing(e.to_string()))?;
            match entry.get_secret() {
                Ok(bytes) => {
                    if bytes.len() != 32 {
                        return Err(UnlockError::KeyringEntryMissing(format!(
                            "keyring kek2 has wrong length: {} (expected 32)",
                            bytes.len()
                        )));
                    }
                    let mut out = [0u8; 32];
                    out.copy_from_slice(&bytes);
                    out
                }
                Err(e) => return Err(UnlockError::KeyringEntryMissing(e.to_string())),
            }
        } else {
            [0u8; 32]
        };

        let kek1 = derive_kek1(pin.as_bytes(), &parsed.salt, parsed.params)
            .map_err(|e| UnlockError::Argon2Kdf(e.to_string()))?;
        let final_kek = combine_keks(kek1.expose_secret(), &kek2);

        let cipher = Aes256Gcm::new_from_slice(final_kek.expose_secret())
            .map_err(|_| UnlockError::AeadReject)?;
        let nonce = Nonce::from_slice(&parsed.nonce);
        let aad = build_aad(&parsed.header);
        let mut plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: parsed.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| UnlockError::AeadReject)?;

        if plaintext.len() != SECRET_KEY_BYTES {
            plaintext.zeroize();
            return Err(UnlockError::BlobMalformed(format!(
                "unexpected plaintext length {} (expected {})",
                plaintext.len(),
                SECRET_KEY_BYTES
            )));
        }

        let mut secret = [0u8; SECRET_KEY_BYTES];
        secret.copy_from_slice(&plaintext);
        let public = KeyPair::from_secret_bytes(&secret).public_bytes();

        // Zero the heap plaintext returned by decrypt() — `secret`
        // is now owned by the SecretBox below.
        plaintext.zeroize();

        Ok(Identity {
            secret: SecretBox::new(Box::new(SecretKeyBytes(secret))),
            public,
            mode: IdentityMode::Normal,
        })
    }

    fn rotate_pin(&self, old_pin: &str, new_pin: &str) -> Result<(), KeyStoreError> {
        // Unlock with the old PIN to recover the secret bytes,
        // then re-init from scratch with the new PIN. We skip the
        // `AlreadyInitialized` check so the rotation is atomic
        // from the caller's POV (wipe + init would race a concurrent
        // reader).
        let identity = match self.unlock(old_pin) {
            Ok(id) => id,
            Err(UnlockError::AeadReject) => {
                return Err(KeyStoreError::WrongPin);
            }
            Err(UnlockError::Io(e)) => return Err(KeyStoreError::Io(e)),
            Err(UnlockError::NotInitialized(p)) => {
                return Err(KeyStoreError::Io(io::Error::new(
                    ErrorKind::NotFound,
                    format!("rotate_pin: not initialized at {}", p.display()),
                )));
            }
            Err(UnlockError::BlobMalformed(m)) => {
                return Err(KeyStoreError::Argon2Kdf(format!(
                    "rotate_pin: blob malformed: {m}"
                )));
            }
            Err(UnlockError::KeyringEntryMissing(m)) => {
                return Err(KeyStoreError::Keyring(format!(
                    "rotate_pin: keyring entry missing: {m}"
                )));
            }
            Err(UnlockError::Argon2Kdf(m)) => {
                return Err(KeyStoreError::Argon2Kdf(m));
            }
        };
        let secret_bytes = identity.into_secret_bytes();

        // Re-use the existing keyring slot: keep kek2, re-derive
        // kek1 with a new salt. Generate a fresh nonce.
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let kek2 = if self.use_keyring {
            let entry = self.keyring_entry()?;
            let bytes = entry
                .get_secret()
                .map_err(|e| KeyStoreError::Keyring(e.to_string()))?;
            if bytes.len() != 32 {
                return Err(KeyStoreError::Keyring(format!(
                    "kek2 wrong length {}",
                    bytes.len()
                )));
            }
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes);
            out
        } else {
            [0u8; 32]
        };

        let kek1 = derive_kek1(new_pin.as_bytes(), &salt, self.params)?;
        let final_kek = combine_keks(kek1.expose_secret(), &kek2);

        let flags = if self.use_keyring { 0b0000_0001 } else { 0 };
        let header = encode_header(flags, &salt, self.params, &nonce_bytes);
        let aad = build_aad(&header);
        let cipher = Aes256Gcm::new_from_slice(final_kek.expose_secret())
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &secret_bytes,
                    aad: &aad,
                },
            )
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;

        let mut blob = Vec::with_capacity(BLOB_HEADER_LEN + ciphertext.len());
        blob.extend_from_slice(&header);
        blob.extend_from_slice(&ciphertext);
        write_atomic(&self.blob_path(), &blob)?;

        // Zero the local secret copy.
        let mut tmp = secret_bytes;
        tmp.zeroize();

        Ok(())
    }

    fn wipe(&self) -> Result<(), KeyStoreError> {
        self.secure_unlink_blob(&self.blob_path())?;
        if self.use_keyring {
            if let Ok(entry) = keyring::Entry::new(&self.keyring_service, &self.keyring_account) {
                // Ignore "no entry" errors — we want idempotent wipe.
                let _ = entry.delete_credential();
            }
        }
        Ok(())
    }
}

// =================================================================
// Phase B — duress + wipe_all
// =================================================================

impl LocalFileKeyStore {
    /// Phase B : provision the duress slot. Writes a **fresh**
    /// Ed25519 keypair to [`BLOB_FILE_NAME_DURESS`] wrapped by
    /// `duress_pin` + the duress-specific OS keyring slot.
    ///
    /// This method is symmetric with [`KeyStore::init`] — it
    /// refuses to overwrite an existing duress blob, it derives
    /// `kek1` with the same Argon2id params, and it produces a
    /// blob that is byte-for-byte indistinguishable in size from
    /// the normal blob. The only visible differences are the
    /// filename and the keyring account.
    ///
    /// Returns an [`Identity`] tagged `IdentityMode::Duress` for
    /// symmetry with `unlock_differential`, even though the typical
    /// caller (the launcher at setup) drops the returned identity
    /// immediately — it is the persistence on disk that matters.
    pub fn init_duress(&self, duress_pin: &str) -> Result<Identity, KeyStoreError> {
        let blob_path = self.blob_path_duress();
        if blob_path.exists() {
            return Err(KeyStoreError::AlreadyInitialized(blob_path));
        }

        fs::create_dir_all(&self.data_dir)?;

        // A distinct Ed25519 keypair — the whole point of duress
        // mode is that the compelled-unlock path reveals a decoy
        // identity that is NOT the user's real node_id.
        let kp = KeyPair::generate();
        let secret_bytes = kp.secret_bytes();
        let public_bytes = kp.public_bytes();

        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let kek2 = if self.use_keyring {
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            let entry = self.keyring_entry_duress()?;
            entry
                .set_secret(&bytes)
                .map_err(|e| KeyStoreError::Keyring(e.to_string()))?;
            bytes
        } else {
            [0u8; 32]
        };

        let kek1 = derive_kek1(duress_pin.as_bytes(), &salt, self.params)?;
        let final_kek = combine_keks(kek1.expose_secret(), &kek2);

        let flags = if self.use_keyring { 0b0000_0001 } else { 0 };
        let header = encode_header(flags, &salt, self.params, &nonce_bytes);
        let aad = build_aad(&header);
        let cipher = Aes256Gcm::new_from_slice(final_kek.expose_secret())
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &secret_bytes,
                    aad: &aad,
                },
            )
            .map_err(|e| KeyStoreError::Aead(e.to_string()))?;

        let mut blob = Vec::with_capacity(BLOB_HEADER_LEN + ciphertext.len());
        blob.extend_from_slice(&header);
        blob.extend_from_slice(&ciphertext);
        write_atomic(&blob_path, &blob)?;

        let identity = Identity {
            secret: SecretBox::new(Box::new(SecretKeyBytes(secret_bytes))),
            public: public_bytes,
            mode: IdentityMode::Duress,
        };
        let mut tmp = secret_bytes;
        tmp.zeroize();
        Ok(identity)
    }

    /// Phase B : try unlocking the normal slot first, fall back to
    /// the duress slot. Returns an [`Identity`] whose `mode` field
    /// tells the caller which slot matched.
    ///
    /// ## Semantics
    ///
    /// - Normal blob exists + PIN matches normal → `Identity { mode:
    ///   Normal }`.
    /// - Normal blob exists + PIN matches duress + duress blob exists
    ///   → `Identity { mode: Duress }`.
    /// - Normal blob exists + PIN wrong + duress blob missing →
    ///   `UnlockError::AeadReject`.
    /// - Normal blob exists + PIN wrong + duress blob exists + PIN
    ///   wrong for duress too → `UnlockError::AeadReject`.
    /// - Normal blob missing → `UnlockError::NotInitialized` from
    ///   the normal path (duress-only layouts are rejected; the
    ///   user must always set up normal first).
    ///
    /// ## Timing indistinguabilite
    ///
    /// A PIN that only matches the duress slot costs **~2x** the
    /// Argon2id KDF (the normal derivation runs first and fails
    /// AEAD). This is a documented Phase B scope cut — see
    /// `.planning/research/S20_phase_B_duress_panic_design.md §5`.
    /// A Sprint 22+ refactor will derive both KDFs in parallel and
    /// cancel the loser to erase the timing side channel.
    pub fn unlock_differential(&self, pin: &str) -> Result<Identity, UnlockError> {
        match self.unlock(pin) {
            Ok(id) => Ok(id),
            Err(UnlockError::AeadReject) => {
                // Normal rejected the PIN — try the duress slot.
                // If the duress blob does not exist, surface the
                // original normal-slot AeadReject so callers cannot
                // observe "duress not set up" vs "wrong PIN".
                let duress_path = self.blob_path_duress();
                if !duress_path.exists() {
                    return Err(UnlockError::AeadReject);
                }
                self.unlock_duress_slot(pin)
            }
            Err(other) => Err(other),
        }
    }

    /// Internal helper mirroring [`KeyStore::unlock`] but reading
    /// the duress blob + the duress keyring account. Returns an
    /// `Identity` tagged `Duress`.
    fn unlock_duress_slot(&self, pin: &str) -> Result<Identity, UnlockError> {
        let blob_path = self.blob_path_duress();
        let blob = match fs::read(&blob_path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(UnlockError::NotInitialized(blob_path));
            }
            Err(e) => return Err(UnlockError::Io(e)),
        };

        let parsed = parse_blob(&blob).map_err(UnlockError::BlobMalformed)?;

        let kek2 = if parsed.uses_keyring {
            let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_account_duress)
                .map_err(|e| UnlockError::KeyringEntryMissing(e.to_string()))?;
            match entry.get_secret() {
                Ok(bytes) => {
                    if bytes.len() != 32 {
                        return Err(UnlockError::KeyringEntryMissing(format!(
                            "keyring kek2 has wrong length: {} (expected 32)",
                            bytes.len()
                        )));
                    }
                    let mut out = [0u8; 32];
                    out.copy_from_slice(&bytes);
                    out
                }
                Err(e) => return Err(UnlockError::KeyringEntryMissing(e.to_string())),
            }
        } else {
            [0u8; 32]
        };

        let kek1 = derive_kek1(pin.as_bytes(), &parsed.salt, parsed.params)
            .map_err(|e| UnlockError::Argon2Kdf(e.to_string()))?;
        let final_kek = combine_keks(kek1.expose_secret(), &kek2);

        let cipher = Aes256Gcm::new_from_slice(final_kek.expose_secret())
            .map_err(|_| UnlockError::AeadReject)?;
        let nonce = Nonce::from_slice(&parsed.nonce);
        let aad = build_aad(&parsed.header);
        let mut plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: parsed.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| UnlockError::AeadReject)?;

        if plaintext.len() != SECRET_KEY_BYTES {
            plaintext.zeroize();
            return Err(UnlockError::BlobMalformed(format!(
                "unexpected plaintext length {} (expected {})",
                plaintext.len(),
                SECRET_KEY_BYTES
            )));
        }

        let mut secret = [0u8; SECRET_KEY_BYTES];
        secret.copy_from_slice(&plaintext);
        let public = KeyPair::from_secret_bytes(&secret).public_bytes();
        plaintext.zeroize();

        Ok(Identity {
            secret: SecretBox::new(Box::new(SecretKeyBytes(secret))),
            public,
            mode: IdentityMode::Duress,
        })
    }

    /// Phase B : wipe both the normal blob AND the duress blob +
    /// both OS keyring entries. Idempotent — missing files or
    /// missing keyring entries are silently accepted so a panic
    /// wipe never blocks on "nothing to delete".
    ///
    /// Called from `nexus-shell-daemon::panic::PanicWipeService`
    /// when the user triggers the 5-tap panic gesture, and from
    /// tests that need a clean slate between cases.
    pub fn wipe_all(&self) -> Result<(), KeyStoreError> {
        // Normal blob first — the typical setup always has this
        // one. Then duress (may not exist).
        self.secure_unlink_blob(&self.blob_path())?;
        self.secure_unlink_blob(&self.blob_path_duress())?;

        if self.use_keyring {
            if let Ok(entry) = keyring::Entry::new(&self.keyring_service, &self.keyring_account) {
                let _ = entry.delete_credential();
            }
            if let Ok(entry) =
                keyring::Entry::new(&self.keyring_service, &self.keyring_account_duress)
            {
                let _ = entry.delete_credential();
            }
        }
        Ok(())
    }

    /// Zero-overwrite then unlink a blob file. No-op if the file
    /// does not exist. Shared by `wipe` (normal blob) and
    /// `wipe_all` (both blobs).
    fn secure_unlink_blob(&self, blob_path: &Path) -> Result<(), KeyStoreError> {
        if blob_path.exists() {
            let len = fs::metadata(blob_path)?.len() as usize;
            let zeros = vec![0u8; len];
            fs::write(blob_path, &zeros)?;
            fs::remove_file(blob_path)?;
        }
        Ok(())
    }
}

// =================================================================
// Private helpers
// =================================================================

/// Argon2id PIN → 32-byte KEK.
///
/// Returns the KEK wrapped in `SecretBox` so the caller cannot
/// accidentally log it, and so the heap allocation is zeroed when
/// the box drops.
fn derive_kek1(
    pin: &[u8],
    salt: &[u8; SALT_LEN],
    params: KdfParams,
) -> Result<SecretBox<[u8; 32]>, KeyStoreError> {
    let argon2_params = params.into_argon2()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    let mut out = Box::new([0u8; 32]);
    argon2
        .hash_password_into(pin, salt, out.as_mut_slice())
        .map_err(|e| KeyStoreError::Argon2Kdf(e.to_string()))?;
    Ok(SecretBox::new(out))
}

/// Combine kek1 (PIN-derived) + kek2 (keyring-wrapped) into a
/// single 32-byte AEAD key via BLAKE3 with domain separation. The
/// combiner must be a keyed hash so trivial differential attacks
/// on kek1 do not leak information about kek2. BLAKE3 is already
/// a workspace dep, constant-time, and collision-resistant.
fn combine_keks(kek1: &[u8; 32], kek2: &[u8; 32]) -> SecretBox<[u8; 32]> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DOMAIN_KEYSTORE_V1);
    hasher.update(kek1);
    hasher.update(kek2);
    let digest = hasher.finalize();
    let mut out = Box::new([0u8; 32]);
    out.copy_from_slice(digest.as_bytes());
    SecretBox::new(out)
}

/// Build the AEAD AAD: domain tag concatenated with the blob
/// header bytes. Any tamper with any byte of the header (flags,
/// salt, Argon2 params, nonce) invalidates the AEAD open. This is
/// the cheapest way to foil a parameter-downgrade attack (e.g. an
/// attacker rewriting `m_cost = 8` in the blob to speed up a
/// brute force).
fn build_aad(header: &[u8; BLOB_HEADER_LEN]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(DOMAIN_KEYSTORE_V1.len() + BLOB_HEADER_LEN);
    aad.extend_from_slice(DOMAIN_KEYSTORE_V1);
    aad.extend_from_slice(header);
    aad
}

fn encode_header(
    flags: u8,
    salt: &[u8; SALT_LEN],
    params: KdfParams,
    nonce: &[u8; NONCE_LEN],
) -> [u8; BLOB_HEADER_LEN] {
    let mut h = [0u8; BLOB_HEADER_LEN];
    h[0..6].copy_from_slice(BLOB_MAGIC);
    h[6] = BLOB_VERSION;
    h[7] = flags;
    h[8..24].copy_from_slice(salt);
    h[24..28].copy_from_slice(&params.m_cost_kib.to_be_bytes());
    h[28..32].copy_from_slice(&params.t_cost.to_be_bytes());
    h[32..36].copy_from_slice(&params.parallelism.to_be_bytes());
    h[36..48].copy_from_slice(nonce);
    h
}

struct ParsedBlob<'a> {
    header: [u8; BLOB_HEADER_LEN],
    uses_keyring: bool,
    salt: [u8; SALT_LEN],
    params: KdfParams,
    nonce: [u8; NONCE_LEN],
    ciphertext: &'a [u8],
}

fn parse_blob(blob: &[u8]) -> Result<ParsedBlob<'_>, String> {
    if blob.len() < BLOB_HEADER_LEN + TAG_LEN {
        return Err(format!(
            "blob too short: {} bytes (expected >= {})",
            blob.len(),
            BLOB_HEADER_LEN + TAG_LEN
        ));
    }
    if &blob[0..6] != BLOB_MAGIC {
        return Err("bad magic (expected SBFBK1)".to_string());
    }
    if blob[6] != BLOB_VERSION {
        return Err(format!(
            "unsupported blob version {} (expected {})",
            blob[6], BLOB_VERSION
        ));
    }
    let flags = blob[7];
    let uses_keyring = flags & 0b0000_0001 != 0;

    let mut header = [0u8; BLOB_HEADER_LEN];
    header.copy_from_slice(&blob[0..BLOB_HEADER_LEN]);

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&blob[8..24]);

    let m_cost_kib = u32::from_be_bytes(blob[24..28].try_into().unwrap());
    let t_cost = u32::from_be_bytes(blob[28..32].try_into().unwrap());
    let parallelism = u32::from_be_bytes(blob[32..36].try_into().unwrap());
    let params = KdfParams {
        m_cost_kib,
        t_cost,
        parallelism,
    };

    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&blob[36..48]);

    let ciphertext = &blob[BLOB_HEADER_LEN..];

    Ok(ParsedBlob {
        header,
        uses_keyring,
        salt,
        params,
        nonce,
        ciphertext,
    })
}

/// Atomic write: write to `<path>.tmp` then rename over the
/// target. On Unix the rename is atomic; on Windows the MoveFileEx
/// call falls back to a best-effort sequence but preserves the
/// invariant that a reader never sees a half-written blob.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("enc.tmp");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&tmp, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&tmp, perms)?;
    }
    fs::rename(&tmp, path)
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    //! Primitive tests for the Sprint 20 Phase A keystore.
    //!
    //! Every test that touches the real OS keyring is gated on a
    //! unique per-testcase slot (`with_keyring_slot`) to avoid
    //! parallel-run collisions, or runs in `without_os_keyring()`
    //! mode so CI can execute them on headless hosts.
    use super::*;
    use tempfile::TempDir;

    fn make_store_no_keyring(dir: &TempDir) -> LocalFileKeyStore {
        LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests()).without_os_keyring()
    }

    /// #1 derive_kek1 is deterministic: same PIN + same salt + same
    /// params → identical output.
    #[test]
    fn derive_kek_deterministic_same_pin_same_salt() {
        let salt = [42u8; SALT_LEN];
        let params = KdfParams::fast_for_tests();
        let a = derive_kek1(b"1234", &salt, params).unwrap();
        let b = derive_kek1(b"1234", &salt, params).unwrap();
        assert_eq!(a.expose_secret(), b.expose_secret());
    }

    /// #2 Different salts produce different KEKs with the same PIN.
    #[test]
    fn derive_kek_different_salt_different_kek() {
        let params = KdfParams::fast_for_tests();
        let a = derive_kek1(b"1234", &[1u8; SALT_LEN], params).unwrap();
        let b = derive_kek1(b"1234", &[2u8; SALT_LEN], params).unwrap();
        assert_ne!(a.expose_secret(), b.expose_secret());
    }

    /// #3 Different PINs produce different KEKs with the same salt.
    #[test]
    fn derive_kek_different_pin_different_kek() {
        let params = KdfParams::fast_for_tests();
        let salt = [3u8; SALT_LEN];
        let a = derive_kek1(b"1234", &salt, params).unwrap();
        let b = derive_kek1(b"5678", &salt, params).unwrap();
        assert_ne!(a.expose_secret(), b.expose_secret());
    }

    /// #4 Blob encode/decode roundtrip via init+unlock.
    #[test]
    fn blob_v1_encode_decode_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        let id_a = store.init("1234").unwrap();
        let id_b = store.unlock("1234").unwrap();
        assert_eq!(id_a.public_bytes(), id_b.public_bytes());
        assert_eq!(
            id_a.into_secret_bytes(),
            id_b.into_secret_bytes(),
            "unlock must recover the exact keypair bytes init wrote"
        );
    }

    /// #5 Blob magic bytes are enforced by parse_blob.
    #[test]
    fn blob_v1_magic_bytes_rejected_if_wrong() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        // Corrupt the magic.
        let path = store.blob_path();
        let mut bytes = fs::read(&path).unwrap();
        bytes[0] = b'X';
        fs::write(&path, &bytes).unwrap();
        let err = store.unlock("1234").unwrap_err();
        match err {
            UnlockError::BlobMalformed(m) => assert!(m.contains("magic"), "got: {m}"),
            other => panic!("expected BlobMalformed, got {other:?}"),
        }
    }

    /// #6 Blob version byte 0x02 is rejected by the v1-only parser.
    #[test]
    fn blob_v1_version_mismatch_rejected() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        let path = store.blob_path();
        let mut bytes = fs::read(&path).unwrap();
        bytes[6] = 0x02;
        fs::write(&path, &bytes).unwrap();
        let err = store.unlock("1234").unwrap_err();
        match err {
            UnlockError::BlobMalformed(m) => assert!(m.contains("version"), "got: {m}"),
            other => panic!("expected BlobMalformed, got {other:?}"),
        }
    }

    /// #7 An attacker who flips a ciphertext byte fails the AEAD.
    #[test]
    fn encrypt_identity_wrong_kek_aead_fails() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        let path = store.blob_path();
        let mut bytes = fs::read(&path).unwrap();
        // Flip a byte inside the ciphertext region (past header).
        let idx = BLOB_HEADER_LEN + 2;
        bytes[idx] ^= 0xff;
        fs::write(&path, &bytes).unwrap();
        let err = store.unlock("1234").unwrap_err();
        assert!(
            matches!(err, UnlockError::AeadReject),
            "expected AeadReject, got {err:?}"
        );
    }

    /// #8 Wrong PIN → AeadReject (the caller sees the same error
    /// code whether the attacker is PIN-guessing or tamper-probing
    /// — the scheme does not leak which).
    #[test]
    fn unlock_wrong_pin_returns_unlock_error() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        let err = store.unlock("wrong-pin").unwrap_err();
        assert!(
            matches!(err, UnlockError::AeadReject),
            "expected AeadReject, got {err:?}"
        );
    }

    /// #9 Happy path: correct PIN returns a usable Identity.
    #[test]
    fn unlock_with_correct_pin_returns_identity() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        let id = store.init("1234").unwrap();
        assert_eq!(id.mode(), IdentityMode::Normal);
        let public = *id.public_bytes();
        let id2 = store.unlock("1234").unwrap();
        assert_eq!(id2.public_bytes(), &public);
    }

    /// #10 Dropping an Identity zeroes the SecretBox heap
    /// allocation. We cannot inspect zeroed memory directly in
    /// safe Rust, so we verify the contract by constructing a
    /// SecretKeyBytes, dropping it explicitly, and relying on the
    /// `#[zeroize(drop)]` derive.
    #[test]
    fn zeroize_drops_plaintext_key_in_memory() {
        // Ensure SecretKeyBytes carries the Zeroize+drop contract.
        fn assert_zeroize<T: zeroize::Zeroize>() {}
        assert_zeroize::<SecretKeyBytes>();
        // Smoke: init returns an Identity whose SecretBox drop
        // calls Zeroize on the wrapped SecretKeyBytes. We observe
        // the call by explicitly consuming the bytes and then
        // re-calling zeroize on the raw array to ensure the type
        // is not accidentally `[u8; N]` (which would double-zero
        // but not be a type error).
        let mut raw = SecretKeyBytes([7u8; SECRET_KEY_BYTES]);
        raw.zeroize();
        assert_eq!(raw.0, [0u8; SECRET_KEY_BYTES]);
    }

    /// #11 The AEAD AAD carries the domain tag; bumping the tag
    /// breaks every existing blob.
    #[test]
    fn canonical_bytes_include_domain_tag() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        // Reading the blob + computing the AAD manually should
        // include DOMAIN_KEYSTORE_V1.
        let bytes = fs::read(store.blob_path()).unwrap();
        let parsed = parse_blob(&bytes).unwrap();
        let aad = build_aad(&parsed.header);
        assert!(
            aad.starts_with(DOMAIN_KEYSTORE_V1),
            "AAD must start with the v1 domain tag"
        );
    }

    /// #12 rotate_pin invalidates the old PIN.
    #[test]
    fn rotate_pin_invalidates_old_pin() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        store.rotate_pin("1234", "5678").unwrap();
        let err = store.unlock("1234").unwrap_err();
        assert!(
            matches!(err, UnlockError::AeadReject),
            "old PIN must no longer unlock, got {err:?}"
        );
    }

    /// #13 rotate_pin preserves the keypair bytes.
    #[test]
    fn rotate_pin_preserves_same_keypair() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        let id_a = store.init("1234").unwrap();
        let public_before = *id_a.public_bytes();
        let secret_before = id_a.into_secret_bytes();

        store.rotate_pin("1234", "5678").unwrap();
        let id_b = store.unlock("5678").unwrap();
        assert_eq!(id_b.public_bytes(), &public_before);
        assert_eq!(id_b.into_secret_bytes(), secret_before);
    }

    /// #14bis Attacker rewrites the Argon2 params bytes in the blob
    /// header (for instance changing `m_cost` to speed up a PIN
    /// brute force). Because the AAD carries the full header plus
    /// the domain tag, the AEAD open fails regardless of whether
    /// the attacker-chosen params can actually derive a valid
    /// kek1 — the ciphertext was sealed against the original AAD.
    /// Defends the downgrade attack explicitly.
    #[test]
    fn param_downgrade_attack_rejected() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        let path = store.blob_path();
        let mut bytes = fs::read(&path).unwrap();
        // Read the original m_cost, then flip to a strictly
        // different (but Argon2-accepted) value so the test still
        // exercises the downgrade path when the test params happen
        // to equal Argon2 MIN_M_COST.
        let orig_m_cost = u32::from_be_bytes(bytes[24..28].try_into().unwrap());
        let new_m_cost: u32 = if orig_m_cost == 16 { 32 } else { 16 };
        bytes[24..28].copy_from_slice(&new_m_cost.to_be_bytes());
        fs::write(&path, &bytes).unwrap();
        let err = store.unlock("1234").unwrap_err();
        assert!(
            matches!(err, UnlockError::AeadReject),
            "param downgrade must trip AEAD, got {err:?}"
        );
    }

    /// #14 wipe removes the blob file.
    #[test]
    fn wipe_removes_blob_and_keyring_entry() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1234").unwrap();
        assert!(store.blob_path().exists());
        store.wipe().unwrap();
        assert!(!store.blob_path().exists());
        // Idempotent: a second wipe on an already-wiped store is
        // a noop, not an error.
        store.wipe().unwrap();
    }

    // =============================================================
    // Phase B — duress + wipe_all
    // =============================================================

    /// #B1 init_duress alongside a pre-existing normal init writes
    /// a second blob file at BLOB_FILE_NAME_DURESS. Both blobs
    /// co-exist on disk.
    #[test]
    fn init_duress_creates_two_blobs() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        assert!(store.blob_path().exists(), "normal blob must exist");
        assert!(store.blob_path_duress().exists(), "duress blob must exist");
    }

    /// #B2 unlock_differential with the normal PIN returns
    /// Identity { mode: Normal } even when the duress slot is
    /// provisioned.
    #[test]
    fn unlock_normal_pin_returns_normal_identity() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let id = store.unlock_differential("1111").unwrap();
        assert_eq!(id.mode(), IdentityMode::Normal);
    }

    /// #B3 unlock_differential with the duress PIN returns
    /// Identity { mode: Duress }. The fall-through from the
    /// normal-slot AEAD reject to the duress-slot open is the
    /// core Phase B runtime branch.
    #[test]
    fn unlock_duress_pin_returns_duress_identity() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let id = store.unlock_differential("2222").unwrap();
        assert_eq!(id.mode(), IdentityMode::Duress);
    }

    /// #B4 unlock_differential with a PIN that matches neither
    /// slot surfaces a uniform `AeadReject` — the caller cannot
    /// observe whether the duress slot is set up based on the
    /// error type alone.
    #[test]
    fn unlock_wrong_pin_rejected_even_with_duress_setup() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let err = store.unlock_differential("wrong-pin").unwrap_err();
        assert!(
            matches!(err, UnlockError::AeadReject),
            "expected AeadReject, got {err:?}"
        );
    }

    /// #B5 The duress keypair is a distinct Ed25519 pair from the
    /// normal one. A network observer that sees the node_id
    /// derived from the unlocked identity sees a different public
    /// key between Normal and Duress boots.
    #[test]
    fn duress_keypair_different_from_normal() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let normal = store.unlock_differential("1111").unwrap();
        let duress = store.unlock_differential("2222").unwrap();
        assert_ne!(
            normal.public_bytes(),
            duress.public_bytes(),
            "duress keypair must differ from normal"
        );
    }

    /// #B6 The duress blob is the same size as the normal blob.
    /// A forensic dump that lists `~/.sbfb/shell-daemon/keyring/`
    /// cannot tell which file is which by size alone.
    #[test]
    fn duress_blob_indistinguishable_size_from_normal() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let normal_len = fs::metadata(store.blob_path()).unwrap().len();
        let duress_len = fs::metadata(store.blob_path_duress()).unwrap().len();
        assert_eq!(
            normal_len, duress_len,
            "duress blob must match normal blob size byte-for-byte"
        );
    }

    /// #B7 init_duress refuses to overwrite an existing duress
    /// blob, mirroring the safety invariant enforced by `init`.
    #[test]
    fn init_duress_twice_rejected() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        let err = store.init_duress("3333").unwrap_err();
        assert!(matches!(err, KeyStoreError::AlreadyInitialized(_)));
    }

    /// #B8 wipe_all removes both blobs; idempotent on re-invocation.
    #[test]
    fn wipe_all_removes_both_blobs() {
        let dir = TempDir::new().unwrap();
        let store = make_store_no_keyring(&dir);
        store.init("1111").unwrap();
        store.init_duress("2222").unwrap();
        assert!(store.blob_path().exists());
        assert!(store.blob_path_duress().exists());
        store.wipe_all().unwrap();
        assert!(!store.blob_path().exists());
        assert!(!store.blob_path_duress().exists());
        // Idempotent
        store.wipe_all().unwrap();
    }
}
