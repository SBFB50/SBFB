// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host secret vault (Sprint 83 Phase L1): named secrets sealed at rest,
//! bound to their opaque reference JUST-IN-TIME.
//!
//! ## What this is, and what it is not
//!
//! A workflow pack carries only an OPAQUE `secret_ref` (a slot name, never
//! material — the engine's `sbfb_workflow_engine::secret` seam). This vault
//! is the HOST side of that seam: it stores the actual credential material,
//! encrypted at rest, and binds a reference to its material immediately before
//! the effect call, then lets the material drop (zeroed). The engine never
//! holds it, journals it, or serialises it. **Not wired yet**: no production
//! code constructs a `SecretVault` — the effect adapter that will call
//! [`UnlockedVault::bind`] belongs to the run-surface phase (0 call-site, DS-1).
//!
//! The vault REUSES the keystore's proven crypto (Sprint 20): Argon2id(PIN) →
//! KEK, AES-256-GCM AEAD, `DOMAIN_KEYSTORE_V1` domain separation, `SecretBox`
//! and `zeroize`, atomic 0600 writes. It is NOT a new signing family and NOT a
//! wire format — the census of `DOMAIN_*_V1` stays at 29, and the vault blob
//! is a purely LOCAL at-rest artifact (never gossiped).
//!
//! ## At-rest defense (honest bound — the keyring is not the primary layer)
//!
//! The primary at-rest protection is Argon2id(PIN): the vault KEK is derived
//! from the operator PIN through the same 64 MiB / 3-pass KDF the keystore
//! uses. The OS keyring layer (`kek2`) is defense-in-depth in the keystore and
//! is NOT wired here — its backend runs on the `keyring` crate's MOCK builder
//! in this workspace (no OS-native feature is enabled), so claiming
//! OS-keychain protection would be false. Activating a live OS keychain is a
//! separate supply-chain decision (it pulls new deps on Linux). What THIS
//! vault guarantees is: material is unreadable at rest without the PIN.
//!
//! ## Per-entry sealing
//!
//! Each secret is AES-256-GCM sealed under the vault KEK with a FRESH random
//! nonce, and its AAD binds the entry to `DOMAIN_KEYSTORE_V1`, a vault
//! sub-context, the blob header (salt + KDF params — foils a parameter
//! downgrade) AND the reference NAME (so a ciphertext cannot be moved to a
//! different slot). Plaintext material therefore exists only transiently,
//! during one [`UnlockedVault::bind`]; the only long-lived secret in memory is
//! the KEK, zeroed when the [`UnlockedVault`] drops.
//!
//! ## The CHECK ENTRY — why the sealed set has a head (ex-review P1-1..P1-4)
//!
//! Per-entry AEAD alone left four holes, each of which the first
//! implementation shipped and the review measured:
//!
//! 1. an EMPTY vault (right after `init`, or after deleting the last entry)
//!    had nothing to authenticate, so **any PIN unlocked it** — and a `put`
//!    under a mistyped PIN sealed the secret under the wrong KEK, losing it
//!    irrecoverably with a diagnostic that blamed the CORRECT PIN;
//! 2. the PIN probe opened the FIRST REAL ENTRY, so every unlock decrypted a
//!    real credential into a plain `Vec<u8>` (which does not zeroise) — and
//!    `BTreeMap::iter().next()` made it always the SAME, lexicographically
//!    smallest, hence predictable one;
//! 3. nothing authenticated the SET: an adversary with file-write access could
//!    delete an entry, or replay an OLD `(nonce, ct)` of the same ref, and the
//!    per-entry AAD still verified — so **a rotation was not durable**;
//! 4. two `UnlockedVault` handles each held a full snapshot and each rewrote
//!    the whole body, so a late write from a stale handle **resurrected a
//!    revoked credential**, with no adversary at all.
//!
//! One construct closes all four: a reserved CHECK ENTRY, sealed under the
//! same KEK, whose plaintext is
//! `marker ‖ 0x1f ‖ epoch(8 BE) ‖ 0x1f ‖ blake3(sorted set fingerprint)`.
//! Its reference name ([`VAULT_CHECK_REF`]) carries a control character, which
//! [`validate_ref`] refuses — so it is UNREACHABLE through `put`, `bind`,
//! `rotate` or `delete`, by construction rather than by a name check.
//! [`SecretVault::unlock`] opens it UNCONDITIONALLY, which means: the PIN is
//! proven even on an empty vault, the header is authenticated even on an empty
//! vault, NO real credential is decrypted at unlock, and the set fingerprint
//! catches a deleted or replayed entry. Every write re-reads the blob and
//! refuses fail-closed if the on-disk epoch moved (`ConcurrentModification`),
//! which is what stops a stale handle from resurrecting a deleted secret —
//! EXCEPT after the lost-update collision described in bound 2 below, which is
//! the one state where two handles can share an epoch without sharing a body.
//!
//! Three honest bounds, stated because none is closed. The threat model §19.4
//! states the same three FACTS in the same terms, but NOT in the same order:
//! the `T-SECRET-VAULT-LIFECYCLE` row lists them as (1) rollback, (2) `delete`
//! is not a sector wipe, (3) the re-read→`rename` window — i.e. its (2) and (3)
//! are this doc's 3 and 2. Match them on CONTENT, never on position; a review
//! round shipped a wrong cross-reference by trusting the ordinal (r8 P1-1).
//! Bound 2 below also carries the NUMBERED ordinal **27** in §19.4, which is
//! the stable name to cite from other documents:
//!
//! 1. Rolling the WHOLE FILE back to a consistent older version (blob + its
//!    check entry together, e.g. from a backup or a filesystem snapshot) is
//!    undetectable from inside the file. Closing it needs a monotone counter
//!    kept OUTSIDE the artifact — and the cheapest precedent in this repo is a
//!    SIDECAR (`anchors.json` persists a revision as an anti-rollback floor,
//!    0 dep): TPM or an OS keyring is NOT a prerequisite. The epoch here only
//!    makes PARTIAL tampering (one entry replayed, deleted or added) detectable.
//! 2. The re-read→`rename` window is locked for NEITHER one process nor two.
//!    Two concurrent writers can both pass the epoch check, both compute the
//!    same `next`, and the last `rename` wins — a lost update in which both
//!    blobs carry the SAME epoch, so the rollback between them is undetectable.
//!    The epoch is a version counter, not a token owned by a writer.
//!    AND the consequence goes one step further than "a lost write", which is
//!    why the sentence above about a stale handle is qualified: once the
//!    collision has happened, BOTH handles carry the disk's epoch, so
//!    `refuse_if_stale` passes for each of them even though one holds a stale
//!    BODY. Each then gets one unguarded write, and a later write by one
//!    silently undoes the other's revocation — the ex-P1-4 class, with no
//!    adversary. The blindness lasts until the next commit by either handle —
//!    or until a `reload()`, which realigns the handle IMMEDIATELY and with no
//!    commit at all (Codex round 2: naming only the commit was too narrow).
//!    Closing it needs a lock spanning re-read→`rename` (an advisory directory
//!    lock, or a `create_new` sentinel); that is debt, like the directory fsync
//!    of bound 11.
//! 3. `delete` removes an entry from the SEALED SET; it is not a sector wipe.
//!    The blocks of the previous inode survive on the medium, still sealed under
//!    the KEK. "Revoked" here means "no longer served and no longer in the
//!    committed set", never "the bytes are gone from the disk".

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::keystore::{
    DOMAIN_KEYSTORE_V1, KdfParams, NONCE_LEN, SALT_LEN, derive_kek1,
    write_atomic_with_tmp_extension,
};

/// File name of the vault blob under the data directory.
pub const VAULT_FILE_NAME: &str = "secrets.vault";

/// Magic prefix at byte 0 of every vault blob.
pub const VAULT_BLOB_MAGIC: &[u8; 6] = b"SBFBV1";

/// Vault blob format version. Pre-launch: stays `0x01` until the `v1.0` tag;
/// if the layout changes before launch, edit here and re-mint dev vaults (no
/// tolerant multi-version decoder — `CLAUDE.md §Pre-launch protocol policy`).
pub const VAULT_BLOB_VERSION: u8 = 0x01;

/// Fixed header: magic(6) + version(1) + m_cost(4 BE) + t_cost(4 BE) +
/// parallelism(4 BE) + salt(16).
pub const VAULT_HEADER_LEN: usize = 6 + 1 + 4 + 4 + 4 + SALT_LEN;

/// Byte offset of the salt inside the header. Derived from the fields before it
/// and used by BOTH the encoder and the parser, so the layout has one source of
/// truth: hard-coded `19..35` slices would still COMPILE if [`SALT_LEN`] changed
/// and would panic at runtime instead.
const SALT_OFFSET: usize = 6 + 1 + 4 + 4 + 4;

/// Reserved reference of the CHECK ENTRY (see the module doc). It carries a
/// control character ON PURPOSE: [`validate_ref`] refuses control characters,
/// so no user operation — `put`, `bind`, `rotate`, `delete` — can name it.
/// Unreachability is a property of the validator, not of a name comparison.
pub const VAULT_CHECK_REF: &str = "\u{0}vault-check";

/// Fixed marker at the head of the check entry's plaintext. A wrong-PIN AEAD
/// failure is caught before this is ever read; the marker guards against a
/// check entry that DECRYPTS but carries a foreign shape.
const VAULT_CHECK_MARKER: &[u8] = b"sbfb-vault-check-v1";

/// Vault AAD sub-context, separating vault ciphertexts from the keystore's
/// identity blob under the SHARED `DOMAIN_KEYSTORE_V1` domain. NOT a
/// `DOMAIN_*_V1` signing family — a local at-rest sub-domain, so the census
/// stays at 29.
const VAULT_AAD_CONTEXT: &[u8] = b"secret-vault";

/// Sub-context that derives the vault AEAD key from the Argon2id KEK, so the
/// key that encrypts secrets is domain-separated from any other use of the
/// PIN-derived material.
const VAULT_KEK_CONTEXT: &[u8] = b"secret-vault-kek";

/// Sub-context of the sorted-set fingerprint carried by the check entry.
const VAULT_SET_CONTEXT: &[u8] = b"secret-vault-set";

/// Field separator inside AAD/KEK/check preimages (mirrors the `0x1f` used
/// elsewhere).
///
/// NOT redundant, despite every preimage also carrying a length prefix or a
/// fixed-width field: [`VAULT_AAD_CONTEXT`] (`secret-vault`) is a STRICT PREFIX
/// of both [`VAULT_KEK_CONTEXT`] (`secret-vault-kek`) and [`VAULT_SET_CONTEXT`]
/// (`secret-vault-set`), and this byte is the only thing separating them. Drop it
/// as a "simplification" and the three sub-domains start to overlap.
const SEP: u8 = 0x1f;

/// Longest accepted secret reference. Bounded so a pathological name cannot
/// bloat the vault blob.
pub const SECRET_REF_MAX_LEN: usize = 512;

/// Credential material bound from the vault. Engineered like the engine's
/// `BoundSecret`: no `Serialize`, redacting `Debug`, zeroed on drop.
///
/// The absence of `Serialize` is PINNED here, by the same compile-fail doctest
/// its engine twin carries — the error code is fixed so the test cannot go
/// green on an unrelated compilation error:
///
/// ```compile_fail,E0277
/// use nexus_core_rs::VaultSecret;
///
/// fn must_not_compile(secret: &VaultSecret) -> String {
///     serde_json::to_string(secret).unwrap() // VaultSecret: !Serialize
/// }
/// ```
///
/// Those three close SERIALISATION and accidental logging — not every leak: a caller that
/// `expose()`s the bytes and copies them into a serialisable value defeats them,
/// which is why the at-rest canary carries positive controls instead of trusting
/// the type.
///
/// The daemon's effect adapter WILL hand these bytes to the engine seam
/// (`sbfb_workflow_engine::secret`) for the duration of one effect call. No such
/// adapter exists yet — 0 call-site, and the engine's `EffectContext` has no
/// resolver field. Read §19.4 of the threat model (DS-1) for the bound, not for
/// a promise.
pub struct VaultSecret {
    material: Zeroizing<Vec<u8>>,
}

impl VaultSecret {
    /// Borrow the material for one effect call. There is no owned copy to
    /// stash on a journaled struct.
    pub fn expose(&self) -> &[u8] {
        &self.material
    }

    /// Length of the material (the only fact the redacting `Debug` reveals).
    pub fn len(&self) -> usize {
        self.material.len()
    }

    /// Whether the material is empty.
    pub fn is_empty(&self) -> bool {
        self.material.is_empty()
    }
}

impl std::fmt::Debug for VaultSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VaultSecret(<redacted {} bytes>)", self.material.len())
    }
}

/// One row of an [`UnlockedVault::inventory`]: the slot NAME and the sealed
/// ciphertext length. NEVER the material or its plaintext length beyond the
/// ciphertext size (which includes the GCM tag).
///
/// The length IS an oracle on the format of what is stored (51 bytes reads as an
/// OpenAI `sk-…`, 40 as an AWS secret, 64 hex as an HMAC key), and so are the
/// slot NAMES and their COUNT, which sit in the blob in clear. Bounded, not
/// closed — see the honest bounds in §19.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultInventoryEntry {
    /// Opaque reference (slot name). Safe to display — it is a name.
    pub secret_ref: String,
    /// Length of the sealed ciphertext (material + 16-byte tag).
    pub sealed_len: usize,
}

/// A vault operation failed.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    /// A vault already exists at this path (call once, or unlock).
    #[error("vault already initialised at {0}")]
    AlreadyInitialised(PathBuf),
    /// No vault blob at this path.
    #[error("vault not initialised at {0}")]
    NotInitialised(PathBuf),
    /// The blob is malformed (bad magic, short, unknown version, bad body, or
    /// a missing/foreign check entry).
    #[error("vault blob malformed: {0}")]
    BlobMalformed(String),
    /// The KDF or AEAD refused — a wrong PIN opens as an AEAD authentication
    /// failure. Material-free by contract.
    #[error("vault unlock/seal refused (wrong PIN or tampered blob)")]
    CryptoReject,
    /// The check entry opened, but the sealed set does not match its
    /// fingerprint: an entry was deleted, added or replayed under an
    /// authenticated-but-stale ciphertext. Fail-closed.
    #[error("vault integrity mismatch: the sealed set was tampered with")]
    IntegrityMismatch,
    /// The blob moved under this handle (another writer committed first).
    /// Fail-closed: writing would resurrect this handle's stale snapshot.
    #[error("vault changed on disk (epoch {on_disk} != {held}); re-unlock before writing")]
    ConcurrentModification {
        /// Epoch currently committed on disk.
        on_disk: u64,
        /// Epoch this handle believed it held.
        held: u64,
    },
    /// No slot answers this reference.
    #[error("unknown secret ref: {0}")]
    UnknownRef(String),
    /// A reference name is outside the accepted domain.
    #[error("invalid secret ref: {0}")]
    InvalidRef(String),
    /// KDF parameter error.
    #[error("kdf params: {0}")]
    KdfParams(String),
    /// I/O failure at the vault file.
    #[error("vault io: {0}")]
    Io(#[from] std::io::Error),
}

/// A locked handle to the vault blob on disk. Unlock it with the operator PIN
/// to bind, store, rotate or delete secrets.
#[derive(Debug, Clone)]
pub struct SecretVault {
    path: PathBuf,
    params: KdfParams,
}

impl SecretVault {
    /// A vault at `<data_dir>/secrets.vault` with production KDF parameters.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self::with_params(data_dir, KdfParams::production())
    }

    /// A vault with custom KDF parameters. Tests pass
    /// [`KdfParams::fast_for_tests`] here to skip the production KDF (~82 ms,
    /// measured — see `PATTERNS.md` §T-keystore-bench-reference; the "3 s"
    /// figure some S20 comments still carry is the DESIGN TARGET, not a
    /// measurement).
    pub fn with_params(data_dir: impl Into<PathBuf>, params: KdfParams) -> Self {
        Self {
            path: data_dir.into().join(VAULT_FILE_NAME),
            params,
        }
    }

    /// Path of the vault blob.
    pub fn blob_path(&self) -> &Path {
        &self.path
    }

    /// Create an EMPTY vault sealed under `pin`, carrying its check entry at
    /// epoch 1. Fails if a vault already exists.
    ///
    /// The existence check is the `create_new` of the write itself, not a
    /// prior `exists()` probe (ex-review P1-4: `exists()` then write is
    /// TOCTOU — two concurrent `init` could both pass the probe).
    ///
    /// `create_new` RESERVES the path, so any later failure must UNRESERVE it
    /// (ex-review r2 P2-7). Otherwise a full disk (or a quota, an AV lock, a
    /// crash) between the create and the `sync_all` leaves a truncated file
    /// behind, and from then on `init` answers `AlreadyInitialised` FOREVER
    /// while `unlock` answers `BlobMalformed` — with no `wipe` API to recover
    /// (see the honest bounds in §19.4). The vault would be permanently
    /// unusable, with no adversary and no crash of ours.
    pub fn init(&self, pin: &str) -> Result<(), VaultError> {
        self.init_inner(pin, false)
    }

    /// `inject_write_failure` exists so the cleanup path above is FALSIFIABLE:
    /// no portable test can fill a disk, and an uncovered recovery path is how
    /// P2-7 shipped in the first place. Production has exactly one caller
    /// ([`Self::init`]) and it passes `false`.
    fn init_inner(&self, pin: &str, inject_write_failure: bool) -> Result<(), VaultError> {
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let header = encode_header(&salt, self.params);
        let kek = self.derive_vault_kek(pin, &salt)?;
        let body: VaultBody = BTreeMap::new();
        let blob = render_blob(kek.expose_secret(), &header, &body, 1)?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)
        {
            Ok(file) => {
                let outcome = write_fresh_blob(file, &blob, &self.path, inject_write_failure);
                if outcome.is_err() {
                    // Unreserve the path. Best-effort on purpose: if THIS also
                    // fails there is nothing further to try, and the original
                    // error is the one worth reporting.
                    let _ = std::fs::remove_file(&self.path);
                }
                outcome
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(VaultError::AlreadyInitialised(self.path.clone()))
            }
            Err(e) => Err(VaultError::Io(e)),
        }
    }

    /// Open the vault with `pin`. The returned [`UnlockedVault`] holds the KEK
    /// (zeroed on drop) and the sealed set; plaintext material is produced
    /// only transiently by [`UnlockedVault::bind`].
    ///
    /// The PIN is proven by opening the CHECK ENTRY — unconditionally, even on
    /// an empty vault, and WITHOUT decrypting any real credential (module doc,
    /// ex-review P1-1/P1-2). The set fingerprint is then verified, so a
    /// deleted or replayed entry is refused here rather than silently served
    /// later (ex-review P1-3).
    pub fn unlock(&self, pin: &str) -> Result<UnlockedVault, VaultError> {
        let blob = match std::fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(VaultError::NotInitialised(self.path.clone()));
            }
            Err(e) => return Err(VaultError::Io(e)),
        };
        let ParsedVaultBlob {
            header,
            salt,
            body: body_bytes,
        } = parse_header(&blob)?;
        let kek = self.derive_vault_kek(pin, &salt)?;
        let mut body: VaultBody = serde_json::from_slice(body_bytes)
            .map_err(|e| VaultError::BlobMalformed(format!("body: {e}")))?;
        let epoch = verify_check_entry(kek.expose_secret(), &header, &mut body)?;
        Ok(UnlockedVault {
            vault: self.clone(),
            header,
            kek,
            body,
            epoch,
        })
    }

    fn derive_vault_kek(
        &self,
        pin: &str,
        salt: &[u8; SALT_LEN],
    ) -> Result<SecretBox<[u8; 32]>, VaultError> {
        let kek1 = derive_kek1(pin.as_bytes(), salt, self.params).map_err(|e| match e {
            crate::keystore::KeyStoreError::Argon2Params(m) => VaultError::KdfParams(m),
            other => VaultError::KdfParams(other.to_string()),
        })?;
        // Domain-separate the vault AEAD key from any other use of the KEK.
        let mut hasher = blake3::Hasher::new();
        hasher.update(DOMAIN_KEYSTORE_V1);
        hasher.update(&[SEP]);
        hasher.update(VAULT_KEK_CONTEXT);
        hasher.update(&[SEP]);
        hasher.update(kek1.expose_secret().as_slice());
        // The finalized digest IS the vault KEK, and the hasher retains the last
        // bytes of kek1 it absorbed: both are copies of key material that nothing
        // would otherwise wipe. `Zeroizing` wipes the digest when it drops at the
        // end of this function; `reset()` overwrites what the hasher retained.
        // Mirror site: `keystore::combine_keks` — the class is closed at BOTH,
        // never at one of the two.
        let digest = Zeroizing::new(*hasher.finalize().as_bytes());
        hasher.reset();
        let mut out = Box::new([0u8; 32]);
        out.copy_from_slice(digest.as_slice());
        Ok(SecretBox::new(out))
    }
}

/// An unlocked vault: bind, store, rotate, delete, inventory. Drops its KEK
/// (zeroed) when it goes out of scope.
pub struct UnlockedVault {
    vault: SecretVault,
    header: [u8; VAULT_HEADER_LEN],
    kek: SecretBox<[u8; 32]>,
    /// The sealed set WITHOUT its check entry (stripped at unlock, re-minted
    /// at every write) — so no code path can mistake the check entry for a
    /// user slot.
    body: VaultBody,
    /// Epoch this handle holds. Every write requires the on-disk epoch to
    /// still equal it, then commits `epoch + 1`.
    epoch: u64,
}

impl std::fmt::Debug for UnlockedVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the KEK or the sealed set contents.
        f.debug_struct("UnlockedVault")
            .field("path", &self.vault.path)
            .field("refs", &self.body.len())
            .field("epoch", &self.epoch)
            .finish()
    }
}

impl UnlockedVault {
    /// Epoch currently held by this handle (bumped by every successful write).
    ///
    /// Reads THIS HANDLE'S SNAPSHOT, like [`Self::refs`] and
    /// [`Self::inventory`] — never a proof of what is committed. The disk
    /// comparison lives in [`Self::refuse_if_stale`], which the four writing
    /// and binding entry points run first.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Bind `secret_ref` to its material, JUST-IN-TIME — in TIME *and* in
    /// FRESHNESS.
    ///
    /// The freshness check is not decoration (ex-review r2 R1). The first cut
    /// closed the stale-handle class on the WRITE side only: `bind` served this
    /// handle's snapshot, and since an entry's AAD binds the header and the ref
    /// but NOT the epoch, an old `(nonce, ct)` stays individually valid for the
    /// life of the KEK. So a daemon holding a long-lived handle kept serving a
    /// credential that another process had deleted or rotated — the exact STRIDE
    /// label of `T-SECRET-VAULT-LIFECYCLE` ("rotation qui laisse l'ancien
    /// materiel recuperable"), with no adversary at all. Closing a class means
    /// closing BOTH sides of it.
    ///
    /// The cost is a file read, one 61-byte AEAD open and one blake3:
    /// [`Self::read_disk_epoch`] re-derives NO Argon2 (the KEK is already
    /// held), so this is microseconds against an effect that is about to make a
    /// network call.
    ///
    /// Fail-closed: an unknown reference does not bind, a stale handle does not
    /// bind, so the effect that needed it does not run — EXCEPT after the
    /// lost-update collision of bound 2 (module doc) = bound 27 of §19.4, the one state where a
    /// handle carries the DISK's epoch with a stale BODY. There the freshness
    /// check passes and `bind` serves an entry that another handle revoked.
    /// That residue is the ex-r2-R1 class re-opening with the ex-P1-4 one; it
    /// closes with a lock spanning re-read→`rename`, not here.
    ///
    /// SECOND, ORDINARY window, and it needs NO collision (Codex round 2): the
    /// epoch check and the read of `self.body` are not atomic. A `bind` that
    /// passes `refuse_if_stale` at epoch `n` may finish AFTER another handle has
    /// committed a `delete` at `n+1`, and it serves the entry it already had.
    /// This execution is still LINEARISABLE — the bind is admitted at the epoch
    /// check, before the revocation — so it is a SEMANTICS to state, not a bug
    /// to fix: **a bind admitted before a revocation may complete after it.**
    /// THIRD window, at the system level: even when `bind` finishes BEFORE the
    /// revocation, the `VaultSecret` it already returned is consumed by the
    /// provider AFTER it. Same class, one layer out — and the reason a hard
    /// revocation needs a LEASE held to the point of provider use, not just a
    /// lock inside the vault. A second re-read would shrink the first window
    /// without closing either.
    pub fn bind(&self, secret_ref: &str) -> Result<VaultSecret, VaultError> {
        validate_ref(secret_ref)?;
        self.refuse_if_stale()?;
        let entry = self
            .body
            .get(secret_ref)
            .ok_or_else(|| VaultError::UnknownRef(secret_ref.to_string()))?;
        let material = open_entry(self.kek.expose_secret(), &self.header, secret_ref, entry)
            .map_err(|_| VaultError::CryptoReject)?;
        Ok(VaultSecret { material })
    }

    /// Re-read the committed set into this handle (the recovery from a
    /// [`VaultError::ConcurrentModification`]). No PIN and no Argon2: the KEK is
    /// already held, only the sealed set and its epoch are refreshed.
    ///
    /// Requires EXCLUSIVE access, while [`Self::bind`] only takes `&self`. That
    /// asymmetry is deliberate but it has a consequence the wiring phase must
    /// plan for: a handle SHARED behind `&self` — the natural shape, since
    /// binding does not mutate — cannot call this recovery, and its only way
    /// back is [`SecretVault::unlock`], i.e. a human PIN entry plus a 64 MiB
    /// Argon2 pass. If the adapter ends up holding the vault behind a shared
    /// reference, it needs interior mutability (or an owner that can hand out a
    /// refreshed handle), decided there rather than assumed here.
    pub fn reload(&mut self) -> Result<(), VaultError> {
        let blob = match std::fs::read(&self.vault.path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(VaultError::NotInitialised(self.vault.path.clone()));
            }
            Err(e) => return Err(VaultError::Io(e)),
        };
        let ParsedVaultBlob {
            header,
            body: body_bytes,
            ..
        } = parse_header(&blob)?;
        let mut body: VaultBody = serde_json::from_slice(body_bytes)
            .map_err(|e| VaultError::BlobMalformed(format!("body: {e}")))?;
        let epoch = verify_check_entry(self.kek.expose_secret(), &header, &mut body)?;
        self.header = header;
        self.body = body;
        self.epoch = epoch;
        Ok(())
    }

    /// Store (or overwrite) `secret_ref` with fresh `material`, sealing it and
    /// persisting the vault. The caller's buffer is taken by value and zeroed
    /// on EVERY return path (it is moved into a `Zeroizing` immediately —
    /// ex-review D8: a manual `.zeroize()` at the end is skipped by the `?`
    /// paths).
    pub fn put(&mut self, secret_ref: &str, material: Vec<u8>) -> Result<(), VaultError> {
        let material = Zeroizing::new(material);
        validate_ref(secret_ref)?;
        let entry = seal_entry(
            self.kek.expose_secret(),
            &self.header,
            secret_ref,
            material.as_slice(),
        )
        .map_err(|_| VaultError::CryptoReject)?;
        let previous = self.body.insert(secret_ref.to_string(), entry);
        match self.persist() {
            Ok(()) => Ok(()),
            Err(e) => {
                // Restore this handle's PRE-CALL snapshot, so a caller that
                // handles the error and retries does not carry a phantom entry.
                // Note what this is NOT: it does not re-align the handle with
                // the disk. On `ConcurrentModification` the disk has precisely
                // MOVED — that is why the write was refused — so `refs()` and
                // `inventory()` still describe the pre-call view until
                // `reload()` runs.
                match previous {
                    Some(old) => self.body.insert(secret_ref.to_string(), old),
                    None => self.body.remove(secret_ref),
                };
                Err(e)
            }
        }
    }

    /// Rotate `secret_ref` to fresh material — a fresh nonce and ciphertext
    /// replace the old ones. Fails closed if the ref does not exist, and fails
    /// closed if this handle's snapshot is stale (see [`Self::delete`]) —
    /// EXCEPT after the lost-update collision of bound 2 (module doc) = bound 27 of §19.4, where
    /// this handle carries the DISK's epoch with a stale BODY: the freshness
    /// check passes and `UnknownRef` then means only "my stale copy did not
    /// have it", not "the committed set does not". Same residue as
    /// [`Self::bind`] and [`Self::delete`]; it closes with a lock spanning
    /// re-read→`rename`, not here.
    pub fn rotate(&mut self, secret_ref: &str, new_material: Vec<u8>) -> Result<(), VaultError> {
        // Taken by value: wrap BEFORE the first fallible step, so that NO early
        // return drops the caller's credential un-wiped (ex-review r3 F2 — the
        // guard lived in `put` alone, while `rotate` has three fallible steps
        // above it). The `take` MOVES the heap buffer into `put`, which re-wraps
        // it; nothing is copied, and the emptied wrapper wipes a zero-length Vec.
        let mut new_material = Zeroizing::new(new_material);
        validate_ref(secret_ref)?;
        self.refuse_if_stale()?;
        if !self.body.contains_key(secret_ref) {
            return Err(VaultError::UnknownRef(secret_ref.to_string()));
        }
        self.put(secret_ref, std::mem::take(&mut *new_material))
    }

    /// Delete `secret_ref` from the sealed set and persist. Returns whether a
    /// slot was removed.
    ///
    /// The freshness check comes FIRST, before the "was it even there?"
    /// short-circuit, and that order is the whole point (ex-review r3 F3). Both
    /// this method and [`Self::rotate`] used to answer from this handle's
    /// SNAPSHOT alone — `Ok(false)` / `UnknownRef` with no disk read at all — so
    /// the class the epoch was introduced to close stayed open on the very path
    /// that carries revocation semantics: the daemon holds a handle at epoch 5,
    /// a CLI adds `stripe_live_key` at epoch 6, the operator revokes through the
    /// daemon, gets `Ok(false)`, reads it as "nothing to revoke" and stops. The
    /// credential is still on disk and the next `bind` from a fresh handle
    /// SERVES it. `Ok(false)` must mean "the committed set does not contain it",
    /// never "my stale copy did not".
    ///
    /// That guarantee holds EXCEPT after the lost-update collision of bound 2
    /// (module doc) = bound 27 of §19.4, the one state where a handle carries
    /// the DISK's epoch with
    /// a stale BODY. There the freshness check passes and `Ok(false)` degrades
    /// back to "my stale copy did not have it" — the very scenario above, minus
    /// the adversary. This fail-OPEN is worse than the `bind` residue because it
    /// reports SUCCESS to a revocation gesture. Same closure: a lock spanning
    /// re-read→`rename`.
    pub fn delete(&mut self, secret_ref: &str) -> Result<bool, VaultError> {
        validate_ref(secret_ref)?;
        self.refuse_if_stale()?;
        let Some(removed) = self.body.remove(secret_ref) else {
            return Ok(false);
        };
        match self.persist() {
            Ok(()) => Ok(true),
            Err(e) => {
                self.body.insert(secret_ref.to_string(), removed);
                Err(e)
            }
        }
    }

    /// Refuse fail-closed if the committed epoch has moved past this handle.
    ///
    /// One helper for the whole class, rather than the same three lines copied
    /// into each entry point: the two rounds of review that preceded this one
    /// both found the check present at some call sites and absent at others.
    /// Cheap by design — [`Self::read_disk_epoch`] replays NO Argon2, the KEK is
    /// already held.
    fn refuse_if_stale(&self) -> Result<(), VaultError> {
        let on_disk = self.read_disk_epoch()?;
        if on_disk != self.epoch {
            return Err(VaultError::ConcurrentModification {
                on_disk,
                held: self.epoch,
            });
        }
        Ok(())
    }

    /// The reference names currently in the vault (no material). The check
    /// entry is not among them — it never enters `body`.
    ///
    /// Reads THIS HANDLE'S SNAPSHOT, with no freshness check — unlike
    /// [`Self::bind`], which re-reads the on-disk epoch because it is about to
    /// hand out material. So after another handle revoked a slot, this list can
    /// still name it until [`Self::reload`]. That is a deliberate split: paying
    /// a file read to enumerate NAMES would buy nothing, since the answer is
    /// stale the moment it returns anyway. Never treat this as evidence that a
    /// slot is still live.
    pub fn refs(&self) -> Vec<String> {
        self.body.keys().cloned().collect()
    }

    /// INVENTORY of the vault — slot names and sealed-ciphertext lengths, never
    /// any material.
    ///
    /// Deliberately not called `audit`: this is a point-in-time listing, NOT a
    /// journal. Nothing here records who bound which secret, or when — the vault
    /// emits no `SecurityEvent` at all. "Which run bound `stripe_live_key`, and
    /// when?" has no answer in L1, and calling this method `audit` would have
    /// implied otherwise (the plan's word for this deliverable was "audit"; what
    /// it gets is an inventory, and the gap is declared rather than papered
    /// over). Same snapshot semantics as [`Self::refs`].
    pub fn inventory(&self) -> Vec<VaultInventoryEntry> {
        self.body
            .iter()
            .map(|(secret_ref, entry)| VaultInventoryEntry {
                secret_ref: secret_ref.clone(),
                sealed_len: entry.ct.len(),
            })
            .collect()
    }

    /// Commit the sealed set. Re-reads the blob and refuses fail-closed if its
    /// epoch moved (ex-review P1-4: a stale handle rewriting its whole
    /// snapshot resurrected deleted secrets).
    ///
    /// Same exception as the four public entry points, and this is where it is
    /// BORN: the re-read→`rename` window is not locked, so two writers can both
    /// pass this check, both compute the same `next`, and the last `rename`
    /// wins — after which BOTH carry the disk's epoch over a stale body. Bound 2
    /// (module doc) = bound 27 of §19.4.
    fn persist(&mut self) -> Result<(), VaultError> {
        self.refuse_if_stale()?;
        // `checked_add`, not `saturating_add`: at `u64::MAX` a saturating epoch
        // would stop advancing SILENTLY, and the compare-and-set above would go
        // blind (`on_disk == held` forever), turning the freshness token into a
        // no-op. Unreachable in practice (2^64 writes) and unforgeable by an
        // adversary (the epoch lives inside an AEAD-authenticated plaintext), but
        // a freshness token that fails open is the wrong primitive regardless.
        let next = self
            .epoch
            .checked_add(1)
            .ok_or_else(|| VaultError::BlobMalformed("epoch overflow".to_string()))?;
        let blob = render_blob(self.kek.expose_secret(), &self.header, &self.body, next)?;
        // Scratch name unique PER WRITE, not per process (ex-review r2 R9): the
        // per-process spelling still let two handles of the SAME process splice
        // each other's partial write, and the epoch check cannot arbitrate that
        // — it runs BEFORE the write, so two writers that both pass it can
        // still collide on one scratch path and hand `rename` a spliced file
        // (unrecoverable: `init` refuses to overwrite and there is no re-key).
        // A random suffix makes the collision impossible instead of unlikely.
        let mut scratch_nonce = [0u8; 8];
        OsRng.fill_bytes(&mut scratch_nonce);
        let tmp_extension = format!(
            "vault.tmp.{}.{}",
            std::process::id(),
            u64::from_be_bytes(scratch_nonce)
        );
        write_atomic_with_tmp_extension(&self.vault.path, &blob, &tmp_extension)?;
        self.epoch = next;
        Ok(())
    }

    /// Epoch currently committed on disk, read through the check entry (so a
    /// tampered blob is refused rather than silently compared).
    fn read_disk_epoch(&self) -> Result<u64, VaultError> {
        let blob = match std::fs::read(&self.vault.path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(VaultError::NotInitialised(self.vault.path.clone()));
            }
            Err(e) => return Err(VaultError::Io(e)),
        };
        let ParsedVaultBlob {
            header,
            body: body_bytes,
            ..
        } = parse_header(&blob)?;
        let mut body: VaultBody = serde_json::from_slice(body_bytes)
            .map_err(|e| VaultError::BlobMalformed(format!("body: {e}")))?;
        verify_check_entry(self.kek.expose_secret(), &header, &mut body)
    }
}

/// One sealed secret in the vault body: a fresh nonce and the AES-256-GCM
/// ciphertext (material + 16-byte tag). `deny_unknown_fields` so a body
/// carrying an unexpected member is refused rather than silently accepted
/// (fail-closed deserialisation, pre-launch doctrine).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedEntry {
    nonce: Vec<u8>,
    ct: Vec<u8>,
}

type VaultBody = BTreeMap<String, SealedEntry>;

/// Write the first blob into the file `create_new` just reserved, consuming the
/// handle (so the caller's cleanup is not fighting an open descriptor).
///
/// `inject_failure` is the test seam of [`SecretVault::init_inner`]: it fails
/// AFTER the create and BEFORE any byte is durable, which is exactly the shape
/// of a full disk.
fn write_fresh_blob(
    mut file: std::fs::File,
    blob: &[u8],
    path: &Path,
    inject_failure: bool,
) -> Result<(), VaultError> {
    use std::io::Write;
    if inject_failure {
        return Err(VaultError::Io(std::io::Error::other(
            "injected write failure (test seam)",
        )));
    }
    file.write_all(blob)?;
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Validate a USER reference. Refusing control characters is what makes
/// [`VAULT_CHECK_REF`] unreachable from every user operation.
///
/// That refusal is load-bearing for AVAILABILITY too, not only for
/// unreachability: if it ever fell, `put(VAULT_CHECK_REF, …)` would make
/// [`set_fingerprint`] count the phantom entry and [`render_blob`] OVERWRITE the
/// real check entry, so the next unlock would compute the fingerprint over a
/// truncated body and fail `IntegrityMismatch` PERMANENTLY — with no `wipe` API
/// to recover and `init` refusing to overwrite. Both properties rest on this
/// single `is_control` test.
fn validate_ref(secret_ref: &str) -> Result<(), VaultError> {
    if secret_ref.is_empty() {
        return Err(VaultError::InvalidRef("empty".to_string()));
    }
    if secret_ref.len() > SECRET_REF_MAX_LEN {
        return Err(VaultError::InvalidRef(format!(
            "longer than {SECRET_REF_MAX_LEN} bytes"
        )));
    }
    if let Some(bad) = secret_ref.chars().find(|&c| is_forbidden_in_ref(c)) {
        return Err(VaultError::InvalidRef(format!(
            "contains a control or invisible/bidi format character U+{:04X}",
            bad as u32
        )));
    }
    Ok(())
}

/// Characters a secret reference may never contain.
///
/// TWIN of `sbfb_workflow_engine::hash::is_forbidden_in_identity` — same set,
/// same reasons, kept in two files because the two crates do not depend on one
/// another. Any change MUST be mirrored; both sides pin the set in a test.
///
/// Control characters are what makes [`VAULT_CHECK_REF`] unreachable and keep a
/// ref out of a line-oriented log. The invisible/bidi FORMAT characters are
/// refused for the reading, not the encoding: a slot name is shown to an operator
/// deciding whether to hand a credential over, and `U+202E` makes a name render
/// as a different name without changing a single byte of the tuple (ex-review r3
/// L1-R3-01).
fn is_forbidden_in_ref(c: char) -> bool {
    c.is_control()
        || matches!(c,
            // Soft hyphen — invisible, and NOT a control character.
            '\u{00AD}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            // Line/paragraph separators (Zl/Zp), which `is_control()` misses.
            | '\u{2028}'..='\u{2029}'
            // Word joiner and the invisible math operators.
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
            // Private Use Areas.
            | '\u{E000}'..='\u{F8FF}'
            | '\u{F0000}'..='\u{FFFFD}'
            | '\u{100000}'..='\u{10FFFD}'
            // TAG block: the canonical smuggling vector (invisible ASCII).
            | '\u{E0020}'..='\u{E007F}'
        )
}

fn encode_header(salt: &[u8; SALT_LEN], params: KdfParams) -> [u8; VAULT_HEADER_LEN] {
    let mut h = [0u8; VAULT_HEADER_LEN];
    h[0..6].copy_from_slice(VAULT_BLOB_MAGIC);
    h[6] = VAULT_BLOB_VERSION;
    h[7..11].copy_from_slice(&params.m_cost_kib.to_be_bytes());
    h[11..15].copy_from_slice(&params.t_cost.to_be_bytes());
    h[15..19].copy_from_slice(&params.parallelism.to_be_bytes());
    h[SALT_OFFSET..SALT_OFFSET + SALT_LEN].copy_from_slice(salt);
    h
}

/// The parsed pieces of a vault blob: the fixed header, the salt it carries,
/// and the JSON body bytes that follow.
struct ParsedVaultBlob<'a> {
    header: [u8; VAULT_HEADER_LEN],
    salt: [u8; SALT_LEN],
    body: &'a [u8],
}

fn parse_header(blob: &[u8]) -> Result<ParsedVaultBlob<'_>, VaultError> {
    if blob.len() < VAULT_HEADER_LEN {
        return Err(VaultError::BlobMalformed(format!(
            "blob too short: {} bytes (expected >= {})",
            blob.len(),
            VAULT_HEADER_LEN
        )));
    }
    if &blob[0..6] != VAULT_BLOB_MAGIC {
        return Err(VaultError::BlobMalformed("bad magic".to_string()));
    }
    if blob[6] != VAULT_BLOB_VERSION {
        return Err(VaultError::BlobMalformed(format!(
            "unsupported vault version {} (expected {})",
            blob[6], VAULT_BLOB_VERSION
        )));
    }
    let mut header = [0u8; VAULT_HEADER_LEN];
    header.copy_from_slice(&blob[0..VAULT_HEADER_LEN]);
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&blob[SALT_OFFSET..SALT_OFFSET + SALT_LEN]);
    Ok(ParsedVaultBlob {
        header,
        salt,
        body: &blob[VAULT_HEADER_LEN..],
    })
}

/// AAD binding an entry to the domain, the vault sub-context, the blob header
/// (salt + KDF params) and the reference NAME. Moving a ciphertext to a
/// different slot, or downgrading the KDF params in the header, fails the open.
fn entry_aad(header: &[u8; VAULT_HEADER_LEN], secret_ref: &str) -> Vec<u8> {
    let ref_bytes = secret_ref.as_bytes();
    let mut aad = Vec::with_capacity(
        DOMAIN_KEYSTORE_V1.len() + VAULT_AAD_CONTEXT.len() + VAULT_HEADER_LEN + ref_bytes.len() + 8,
    );
    aad.extend_from_slice(DOMAIN_KEYSTORE_V1);
    aad.push(SEP);
    aad.extend_from_slice(VAULT_AAD_CONTEXT);
    aad.push(SEP);
    aad.extend_from_slice(header);
    aad.push(SEP);
    // Length-prefix the ref so its boundary is unambiguous in the preimage.
    aad.extend_from_slice(&(ref_bytes.len() as u64).to_le_bytes());
    aad.extend_from_slice(ref_bytes);
    aad
}

/// Fingerprint of the sealed set: `blake3` over the SORTED `(ref, nonce,
/// blake3(ct))` triples, each component length-prefixed. Sorted because the
/// body is a `BTreeMap` (iteration order is the key order), length-prefixed so
/// two different sets cannot share a preimage.
fn set_fingerprint(body: &VaultBody) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(VAULT_SET_CONTEXT);
    for (secret_ref, entry) in body {
        for component in [
            secret_ref.as_bytes(),
            entry.nonce.as_slice(),
            blake3::hash(&entry.ct).as_bytes().as_slice(),
        ] {
            hasher.update(&[SEP]);
            hasher.update(&(component.len() as u64).to_le_bytes());
            hasher.update(component);
        }
    }
    *hasher.finalize().as_bytes()
}

/// Plaintext of the check entry: `marker ‖ 0x1f ‖ epoch(8 BE) ‖ 0x1f ‖ set`.
fn check_plaintext(epoch: u64, fingerprint: &[u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(VAULT_CHECK_MARKER.len() + 2 + 8 + 32);
    out.extend_from_slice(VAULT_CHECK_MARKER);
    out.push(SEP);
    out.extend_from_slice(&epoch.to_be_bytes());
    out.push(SEP);
    out.extend_from_slice(fingerprint);
    out
}

/// Seal the check entry for `body` at `epoch` and render the whole blob.
fn render_blob(
    vault_kek: &[u8; 32],
    header: &[u8; VAULT_HEADER_LEN],
    body: &VaultBody,
    epoch: u64,
) -> Result<Vec<u8>, VaultError> {
    let fingerprint = set_fingerprint(body);
    let plaintext = Zeroizing::new(check_plaintext(epoch, &fingerprint));
    let check = seal_entry(vault_kek, header, VAULT_CHECK_REF, plaintext.as_slice())
        .map_err(|_| VaultError::CryptoReject)?;

    let mut on_disk = body.clone();
    on_disk.insert(VAULT_CHECK_REF.to_string(), check);
    let body_bytes = serde_json::to_vec(&on_disk)
        .map_err(|e| VaultError::BlobMalformed(format!("serialise body: {e}")))?;

    let mut blob = Vec::with_capacity(VAULT_HEADER_LEN + body_bytes.len());
    blob.extend_from_slice(header);
    blob.extend_from_slice(&body_bytes);
    Ok(blob)
}

/// Remove the check entry from `body`, open it, and verify the set it claims.
/// Returns the epoch it carries.
///
/// This is the ONLY authentication performed at unlock — no real credential is
/// decrypted (ex-review P1-2), and it runs even when `body` is otherwise empty
/// (ex-review P1-1).
fn verify_check_entry(
    vault_kek: &[u8; 32],
    header: &[u8; VAULT_HEADER_LEN],
    body: &mut VaultBody,
) -> Result<u64, VaultError> {
    let check = body
        .remove(VAULT_CHECK_REF)
        .ok_or_else(|| VaultError::BlobMalformed("missing check entry".to_string()))?;
    let plaintext = open_entry(vault_kek, header, VAULT_CHECK_REF, &check)
        .map_err(|_| VaultError::CryptoReject)?;

    let expected_len = VAULT_CHECK_MARKER.len() + 2 + 8 + 32;
    if plaintext.len() != expected_len {
        return Err(VaultError::BlobMalformed(
            "check entry has an unexpected length".to_string(),
        ));
    }
    if &plaintext[..VAULT_CHECK_MARKER.len()] != VAULT_CHECK_MARKER {
        return Err(VaultError::BlobMalformed(
            "check entry marker mismatch".to_string(),
        ));
    }
    // Verify the SHAPE fully, not halfway: both separators are part of the
    // plaintext this module writes, so both are part of what "a foreign shape"
    // means. Checking the marker and the field widths while ignoring the two
    // `0x1f` bytes accepted a plaintext of the right length whose framing was
    // wrong — the exact reading the marker is documented to prevent.
    if plaintext[VAULT_CHECK_MARKER.len()] != SEP
        || plaintext[VAULT_CHECK_MARKER.len() + 1 + 8] != SEP
    {
        return Err(VaultError::BlobMalformed(
            "check entry framing mismatch".to_string(),
        ));
    }
    let epoch_at = VAULT_CHECK_MARKER.len() + 1;
    let mut epoch_bytes = [0u8; 8];
    epoch_bytes.copy_from_slice(&plaintext[epoch_at..epoch_at + 8]);
    let epoch = u64::from_be_bytes(epoch_bytes);

    let claimed = &plaintext[epoch_at + 9..];
    if claimed != set_fingerprint(body).as_slice() {
        return Err(VaultError::IntegrityMismatch);
    }
    Ok(epoch)
}

fn seal_entry(
    vault_kek: &[u8; 32],
    header: &[u8; VAULT_HEADER_LEN],
    secret_ref: &str,
    material: &[u8],
) -> Result<SealedEntry, ()> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let aad = entry_aad(header, secret_ref);
    let cipher = Aes256Gcm::new_from_slice(vault_kek).map_err(|_| ())?;
    let ct = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: material,
                aad: &aad,
            },
        )
        .map_err(|_| ())?;
    Ok(SealedEntry {
        nonce: nonce_bytes.to_vec(),
        ct,
    })
}

/// Open one sealed entry. The plaintext is returned INSIDE a `Zeroizing`, so
/// every caller — including a probe that drops it immediately — zeroes it
/// (ex-review P1-2: a bare `Vec<u8>` left credential material in freed heap).
fn open_entry(
    vault_kek: &[u8; 32],
    header: &[u8; VAULT_HEADER_LEN],
    secret_ref: &str,
    entry: &SealedEntry,
) -> Result<Zeroizing<Vec<u8>>, ()> {
    if entry.nonce.len() != NONCE_LEN {
        return Err(());
    }
    let aad = entry_aad(header, secret_ref);
    let cipher = Aes256Gcm::new_from_slice(vault_kek).map_err(|_| ())?;
    cipher
        .decrypt(
            Nonce::from_slice(&entry.nonce),
            Payload {
                msg: &entry.ct,
                aad: &aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const CANARY: &[u8] = b"CANARY-VAULT-material-7f2a-MUST-NOT-APPEAR-AT-REST";
    const PIN: &str = "1234";

    fn fast_vault(dir: &TempDir) -> SecretVault {
        SecretVault::with_params(dir.path(), KdfParams::fast_for_tests())
    }

    /// Every byte-oriented at-rest search must ALSO look for the encoding
    /// `serde_json` actually writes for a `Vec<u8>`: a decimal array. A raw
    /// ASCII scan is blind to it (ex-review P1-6d), so this helper searches both.
    ///
    /// TWIN of `sbfb_workflow_engine::secret`'s canary helper — RECOPIED, not
    /// shared (the two crates do not depend on one another), and deliberately
    /// NARROWER: that twin has more arms, including base64, hex and
    /// JSON-escaped renderings, because it searches RENDERED documents. This one
    /// searches a SEALED blob — a binary header plus the `serde_json` encoding
    /// of a `BTreeMap<String, SealedEntry>` — where the only renderings a leak
    /// could take are raw bytes and a decimal array. The divergence is justified
    /// by the surface, not suffered; the rationale is §19.4, row
    /// `T-SECRET-REF-EXFIL`. (An earlier version of this sentence said the
    /// engine had 3 arms and that NEITHER searched base64/hex/JSON-escaped —
    /// both halves were stale, the engine gained those arms in ex-review r2.)
    fn find_material_at_rest(haystack: &[u8], needle: &[u8]) -> Option<String> {
        if haystack
            .windows(needle.len())
            .any(|window| window == needle)
        {
            return Some("raw bytes".to_string());
        }
        let decimal = needle
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let text = String::from_utf8_lossy(haystack);
        if text.contains(&decimal) {
            return Some("json decimal array".to_string());
        }
        None
    }

    #[test]
    fn vault_secret_debug_redacts_exactly() {
        let s = VaultSecret {
            material: Zeroizing::new(CANARY.to_vec()),
        };
        // EXACT equality, not `!contains(CANARY)`: an oracle that only checks
        // for the ASCII canary is blind to a `{:?}` of the byte slice, which
        // would spill the whole credential in decimal (ex-review P1-6b).
        assert_eq!(
            format!("{s:?}"),
            format!("VaultSecret(<redacted {} bytes>)", CANARY.len())
        );
    }

    /// The production constructor really uses PRODUCTION KDF parameters.
    ///
    /// Every test in this module goes through `fast_for_tests` (8 KiB, 1 pass)
    /// for speed, so `SecretVault::new` — the only constructor a host would
    /// call — was executed by NOTHING: swapping `production()` for
    /// `fast_for_tests()` left all 19 vault tests and the whole workspace green
    /// while sealing real vaults under a KEK brute-forceable from a PIN
    /// dictionary (ex-review r3 L1-D5-3). Asserting the FIELD rather than
    /// calling `unlock` keeps this free of a production Argon2 run (~82 ms measured,
    /// not the "3 s" S20 design target).
    #[test]
    fn production_constructor_uses_production_kdf_params() {
        let dir = TempDir::new().unwrap();
        assert_eq!(
            SecretVault::new(dir.path()).params,
            KdfParams::production(),
            "the production vault must not be constructed with test parameters"
        );
    }

    /// G-SECRET-VAULT. Full lifecycle — init, bind (JIT), rotate, delete,
    /// inventory — plus the at-rest guarantee, checked under BOTH encodings,
    /// and the TYPE that provides the zeroisation this test is named after.
    ///
    /// That last assertion exists because the name promised something no line
    /// checked (ex-review r3 L1-D5-1): the wipe itself cannot be observed from
    /// safe Rust, so what is pinned is the WRAPPER that performs it on drop —
    /// the same thing the engine twin pins
    /// (`sbfb_workflow_engine::secret::material_wrappers_are_pinned_to_zeroizing`).
    /// Swap `Zeroizing<Vec<u8>>` for a bare `Vec<u8>` and this stops compiling.
    #[test]
    fn vault_bind_rotate_zeroize() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);

        vault.init(PIN).unwrap();
        assert!(matches!(
            vault.init(PIN),
            Err(VaultError::AlreadyInitialised(_))
        ));

        let cred = "cloud_provider_token";
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            assert_eq!(unlocked.epoch(), 1);
            unlocked.put(cred, CANARY.to_vec()).unwrap();
            assert_eq!(unlocked.epoch(), 2, "a write bumps the epoch");

            let bound = unlocked.bind(cred).unwrap();
            assert_eq!(bound.expose(), CANARY);
            // Pin the wrapper that zeroises on drop, not merely its behaviour.
            fn assert_zeroizing<T: zeroize::Zeroize>(_: &Zeroizing<T>) {}
            assert_zeroizing(&bound.material);
            assert!(matches!(
                unlocked.bind("absent"),
                Err(VaultError::UnknownRef(_))
            ));

            // The check entry is NOT a user slot.
            assert_eq!(unlocked.refs(), vec![cred.to_string()]);
            let inventory = unlocked.inventory();
            assert_eq!(inventory.len(), 1);
            assert_eq!(inventory[0].secret_ref, cred);
            assert!(inventory[0].sealed_len >= CANARY.len() + 16);
        }

        // At-rest: the canary is absent in BOTH encodings. Positive control
        // proves the search is discriminating for each one.
        let on_disk = std::fs::read(vault.blob_path()).unwrap();
        let mut poisoned_raw = on_disk.clone();
        poisoned_raw.extend_from_slice(CANARY);
        assert_eq!(
            find_material_at_rest(&poisoned_raw, CANARY).as_deref(),
            Some("raw bytes")
        );
        // Built with serde_json ITSELF, so the control exercises the exact
        // encoding the vault body uses — a hand-rolled `{:?}` of a `Vec<u8>`
        // inserts spaces after the commas and would silently miss.
        let poisoned_json =
            serde_json::to_string(&serde_json::json!({ "leak": CANARY.to_vec() })).unwrap();
        assert_eq!(
            find_material_at_rest(poisoned_json.as_bytes(), CANARY).as_deref(),
            Some("json decimal array"),
            "the decimal-array arm of the search must be discriminating"
        );
        assert_eq!(
            find_material_at_rest(&on_disk, CANARY),
            None,
            "canary material leaked into the vault blob at rest"
        );

        // Rotate, then re-unlock: the NEW material binds, the old is gone.
        let new_material = b"ROTATED-material-9e1c".to_vec();
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.rotate(cred, new_material.clone()).unwrap();
        }
        {
            let unlocked = vault.unlock(PIN).unwrap();
            assert_eq!(unlocked.bind(cred).unwrap().expose(), &new_material[..]);
            let on_disk = std::fs::read(vault.blob_path()).unwrap();
            assert_eq!(find_material_at_rest(&on_disk, CANARY), None);
        }

        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            assert!(matches!(
                unlocked.rotate("absent", b"x".to_vec()),
                Err(VaultError::UnknownRef(_))
            ));
        }

        // Delete is DURABLE: proven through a FRESH unlock, not just in memory
        // (ex-review D4).
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            assert!(unlocked.delete(cred).unwrap());
            assert!(!unlocked.delete(cred).unwrap());
        }
        {
            let unlocked = vault.unlock(PIN).unwrap();
            assert!(matches!(
                unlocked.bind(cred),
                Err(VaultError::UnknownRef(_))
            ));
            assert!(unlocked.inventory().is_empty());
        }
    }

    #[test]
    fn wrong_pin_is_refused() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.put("k", b"v".to_vec()).unwrap();
        }
        match vault.unlock("9999") {
            Err(VaultError::CryptoReject) => {}
            other => panic!("wrong PIN must be refused, got {other:?}"),
        }
    }

    /// The hole the check entry closes (ex-review P1-1): an EMPTY vault — the
    /// state of EVERY vault right after `init` — used to open under any PIN,
    /// and a `put` then sealed the secret under the wrong KEK, losing it.
    #[test]
    fn wrong_pin_on_empty_vault_is_refused() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        match vault.unlock("9999") {
            Err(VaultError::CryptoReject) => {}
            other => panic!("empty vault must still prove the PIN, got {other:?}"),
        }
        // And after deleting the last entry the vault is empty again.
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.put("only", b"v".to_vec()).unwrap();
            assert!(unlocked.delete("only").unwrap());
        }
        assert!(matches!(
            vault.unlock("0000"),
            Err(VaultError::CryptoReject)
        ));
        assert!(vault.unlock(PIN).is_ok());
    }

    #[test]
    fn tampered_ref_binding_fails() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();
        let mut unlocked = vault.unlock(PIN).unwrap();
        unlocked.put("real", b"secret".to_vec()).unwrap();
        let entry = unlocked.body.get("real").unwrap().clone();
        let kek = unlocked.kek.expose_secret();
        assert!(open_entry(kek, &unlocked.header, "real", &entry).is_ok());
        assert!(
            open_entry(kek, &unlocked.header, "attacker", &entry).is_err(),
            "a ciphertext must not open under a different ref name"
        );
    }

    /// The reserved check reference is unreachable through EVERY user
    /// operation — because `validate_ref` refuses control characters, not
    /// because of a name comparison.
    #[test]
    fn check_entry_is_unreachable_from_user_operations() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();
        let mut unlocked = vault.unlock(PIN).unwrap();

        for result in [
            unlocked.bind(VAULT_CHECK_REF).map(|_| ()),
            unlocked.put(VAULT_CHECK_REF, b"x".to_vec()),
            unlocked.rotate(VAULT_CHECK_REF, b"x".to_vec()),
            unlocked.delete(VAULT_CHECK_REF).map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(VaultError::InvalidRef(_))),
                "the check ref must be refused by validation, got {result:?}"
            );
        }
    }

    /// Ex-review P1-3: replaying an OLD `(nonce, ct)` of the SAME ref used to
    /// verify — the per-entry AAD is identical — so a rotation was not
    /// durable. The set fingerprint now catches it.
    #[test]
    fn replaying_an_old_ciphertext_is_refused() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();
        let cred = "token";

        let stale = {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.put(cred, b"LEAKED-v1".to_vec()).unwrap();
            unlocked.body.get(cred).unwrap().clone()
        };
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.rotate(cred, b"ROTATED-v2".to_vec()).unwrap();
        }

        // Splice the pre-rotation ciphertext back in, keeping the (newer)
        // check entry — the exact adversary of P1-3.
        let blob = std::fs::read(vault.blob_path()).unwrap();
        let ParsedVaultBlob { header, body, .. } = parse_header(&blob).unwrap();
        let mut on_disk: VaultBody = serde_json::from_slice(body).unwrap();
        on_disk.insert(cred.to_string(), stale);
        let mut spliced = Vec::from(&header[..]);
        spliced.extend_from_slice(&serde_json::to_vec(&on_disk).unwrap());
        std::fs::write(vault.blob_path(), &spliced).unwrap();

        match vault.unlock(PIN) {
            Err(VaultError::IntegrityMismatch) => {}
            other => panic!("a replayed ciphertext must be refused, got {other:?}"),
        }
    }

    /// Ex-review P1-3 (second arm): dropping an entry from the body is
    /// detected too — a silent selective delete used to be invisible.
    #[test]
    fn deleting_an_entry_behind_the_vault_is_detected() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();
        {
            let mut unlocked = vault.unlock(PIN).unwrap();
            unlocked.put("a", b"1".to_vec()).unwrap();
            unlocked.put("b", b"2".to_vec()).unwrap();
        }
        let blob = std::fs::read(vault.blob_path()).unwrap();
        let ParsedVaultBlob { header, body, .. } = parse_header(&blob).unwrap();
        let mut on_disk: VaultBody = serde_json::from_slice(body).unwrap();
        assert!(on_disk.remove("a").is_some());
        let mut spliced = Vec::from(&header[..]);
        spliced.extend_from_slice(&serde_json::to_vec(&on_disk).unwrap());
        std::fs::write(vault.blob_path(), &spliced).unwrap();

        assert!(matches!(
            vault.unlock(PIN),
            Err(VaultError::IntegrityMismatch)
        ));
    }

    /// Ex-review r2 R1 — the READ half of the stale-handle class. `bind` used to
    /// serve this handle's snapshot, so a credential deleted (or rotated) by
    /// ANOTHER handle was still handed to the effect: a revoked token kept
    /// leaving for the provider. This test was RED before the freshness guard.
    #[test]
    fn stale_handle_bind_serves_nothing_after_a_revocation() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        // The daemon's long-lived handle.
        let mut daemon = vault.unlock(PIN).unwrap();
        daemon
            .put("cloud_provider_token", b"COMPROMISED".to_vec())
            .unwrap();
        assert_eq!(
            daemon.bind("cloud_provider_token").unwrap().expose(),
            b"COMPROMISED"
        );

        // The operator revokes from a second handle (a CLI, say).
        {
            let mut cli = vault.unlock(PIN).unwrap();
            assert!(cli.delete("cloud_provider_token").unwrap());
        }

        // The daemon must NOT serve the revoked credential any more.
        match daemon.bind("cloud_provider_token") {
            Err(VaultError::ConcurrentModification { .. }) => {}
            Ok(_) => panic!("a revoked credential was served from a stale snapshot"),
            other => panic!("expected ConcurrentModification, got {other:?}"),
        }

        // And the documented recovery works: reload, then the ref is gone.
        daemon.reload().unwrap();
        assert!(matches!(
            daemon.bind("cloud_provider_token"),
            Err(VaultError::UnknownRef(_))
        ));

        // Same class for a ROTATION: the stale handle must not serve the OLD
        // material.
        daemon.put("rotating", b"v1".to_vec()).unwrap();
        {
            let mut cli = vault.unlock(PIN).unwrap();
            cli.rotate("rotating", b"v2".to_vec()).unwrap();
        }
        assert!(matches!(
            daemon.bind("rotating"),
            Err(VaultError::ConcurrentModification { .. })
        ));
        daemon.reload().unwrap();
        assert_eq!(daemon.bind("rotating").unwrap().expose(), b"v2");
    }

    /// Ex-review P1-4: with no epoch, a stale handle's write resurrected a
    /// credential another handle had deleted. Now the stale write is REFUSED.
    #[test]
    fn stale_handle_write_is_refused_not_silently_applied() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        let mut handle_a = vault.unlock(PIN).unwrap();
        handle_a
            .put("leaked_token", b"compromised".to_vec())
            .unwrap();

        // Operator revokes through a second handle (a CLI, say).
        let mut handle_b = vault.unlock(PIN).unwrap();
        assert!(handle_b.delete("leaked_token").unwrap());

        // The daemon's older handle writes something unrelated.
        let err = handle_a
            .put("new_token", b"fresh".to_vec())
            .expect_err("a stale handle must not commit");
        match err {
            VaultError::ConcurrentModification { .. } => {}
            other => panic!("expected ConcurrentModification, got {other:?}"),
        }

        // The revocation SURVIVED, and the refused write left no phantom.
        let unlocked = vault.unlock(PIN).unwrap();
        assert!(matches!(
            unlocked.bind("leaked_token"),
            Err(VaultError::UnknownRef(_))
        ));
        assert!(matches!(
            unlocked.bind("new_token"),
            Err(VaultError::UnknownRef(_))
        ));
    }

    /// A KDF-parameter downgrade in the header fails the AEAD open, because
    /// the header is inside every entry's AAD — including the check entry's,
    /// which is why this now holds on an EMPTY vault too (patron
    /// `keystore::tests::param_downgrade_attack_rejected`).
    #[test]
    fn param_downgrade_in_header_is_refused() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        let mut blob = std::fs::read(vault.blob_path()).unwrap();
        // m_cost lives at bytes 7..11.
        blob[7..11].copy_from_slice(&1u32.to_be_bytes());
        std::fs::write(vault.blob_path(), &blob).unwrap();

        assert!(matches!(vault.unlock(PIN), Err(VaultError::CryptoReject)));
    }

    /// Each vault mints its OWN salt, so two vaults under the same PIN never
    /// share a KEK (and a ciphertext cannot travel between them).
    #[test]
    fn each_vault_has_its_own_salt() {
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        let a = fast_vault(&dir_a);
        let b = fast_vault(&dir_b);
        a.init(PIN).unwrap();
        b.init(PIN).unwrap();

        let salt_of = |v: &SecretVault| {
            let blob = std::fs::read(v.blob_path()).unwrap();
            parse_header(&blob).unwrap().salt
        };
        assert_ne!(salt_of(&a), salt_of(&b));
    }

    /// A body with an unexpected member is refused (fail-closed
    /// deserialisation), not silently accepted.
    #[test]
    fn unknown_entry_field_is_refused() {
        let json = br#"{"k":{"nonce":[1],"ct":[2],"extra":3}}"#;
        let parsed: Result<VaultBody, _> = serde_json::from_slice(json);
        assert!(parsed.is_err(), "deny_unknown_fields must reject `extra`");
    }

    /// The vault's KEK is domain-separated from the keystore's identity KEK:
    /// the same PIN and salt derive DIFFERENT keys on the two paths.
    #[test]
    fn vault_kek_is_domain_separated_from_keystore() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        let salt = [7u8; SALT_LEN];
        let vault_kek = vault.derive_vault_kek(PIN, &salt).unwrap();
        let keystore_kek1 =
            derive_kek1(PIN.as_bytes(), &salt, KdfParams::fast_for_tests()).unwrap();
        assert_ne!(
            vault_kek.expose_secret().as_slice(),
            keystore_kek1.expose_secret().as_slice(),
            "the vault KEK must not equal the raw Argon2id output"
        );

        // KNOWN-ANSWER VECTOR — the only control that makes the derivation
        // FORMULA falsifiable. Without it, `derive_vault_kek` could be replaced
        // by a bare `blake3(kek1)` — no domain, no sub-context, no separators —
        // and the whole suite would stay green, because every test creates AND
        // opens its vault with the same code: any self-consistent derivation
        // passes. The name of this test would then keep asserting a domain
        // separation it no longer pins.
        //
        // Preimage, 66 bytes (fast test params, salt = [7; 16]):
        //   DOMAIN_KEYSTORE_V1 (16) ‖ 0x1f ‖ VAULT_KEK_CONTEXT (16) ‖ 0x1f ‖ kek1 (32)
        // Precedent for pinning bytes against a stored artifact: `pack.rs`.
        assert_eq!(
            hex::encode(vault_kek.expose_secret()),
            "a387c0dc815153d4d3b3e617703333f503fd9ea29e96940dfaef56fba89b31c5",
            "the vault KEK derivation formula changed — if this is intended, \
             re-mint the vector; if not, the domain separation was just dropped"
        );
    }

    /// Ex-review r2 P2-7: `create_new` RESERVES the path, so a failure after it
    /// must unreserve it. Otherwise a full disk on the very first `init` leaves a
    /// truncated file that makes `init` answer `AlreadyInitialised` forever and
    /// `unlock` answer `BlobMalformed` — with no `wipe` to recover from.
    ///
    /// This is why `init_inner` carries a test seam: no portable test can fill a
    /// disk, and an uncovered recovery path is how the defect shipped.
    #[test]
    fn init_that_fails_mid_write_leaves_no_reserved_file() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);

        let err = vault
            .init_inner(PIN, true)
            .expect_err("the injected failure must surface");
        assert!(matches!(err, VaultError::Io(_)), "got {err:?}");

        // The reservation was released: nothing is left behind.
        assert!(
            !dir.path().join(VAULT_FILE_NAME).exists(),
            "a failed init must not leave a reserved (truncated) vault file"
        );

        // And the operator can simply try again — the vault is NOT bricked.
        vault.init(PIN).unwrap();
        let unlocked = vault.unlock(PIN).unwrap();
        assert!(unlocked.refs().is_empty());
    }

    /// A malformed nonce must FAIL CLOSED, never panic. `Nonce::from_slice`
    /// panics on anything but 12 bytes, so the length guard in `open_entry` is
    /// the difference between a refusal and a crash on input an adversary with
    /// file-write access controls.
    ///
    /// Measured reachability, which is NOT what one would assume: tampering with
    /// a USER entry's nonce never reaches that guard, because it changes the set
    /// and the fingerprint refuses the whole blob at `unlock` first
    /// (`IntegrityMismatch`) — defense in depth working as designed. The guard is
    /// reachable through the CHECK ENTRY, whose nonce sits OUTSIDE the
    /// fingerprint (it is stripped before the set is hashed) — i.e. exactly on
    /// the daemon's BOOT path, where a panic would be a crash loop rather than a
    /// refused unlock. Both halves are asserted, each for what it really does.
    #[test]
    fn malformed_nonce_length_is_refused_without_panicking() {
        for bad_nonce in [
            Vec::new(),
            vec![0u8; NONCE_LEN - 1],
            vec![0u8; NONCE_LEN + 1],
        ] {
            // (a) USER entry: caught EARLIER, by the set fingerprint.
            {
                let dir = TempDir::new().unwrap();
                let vault = fast_vault(&dir);
                vault.init(PIN).unwrap();
                {
                    let mut unlocked = vault.unlock(PIN).unwrap();
                    unlocked.put("cred", b"material".to_vec()).unwrap();
                }
                rewrite_entry_nonce(&vault.path, "cred", bad_nonce.clone());
                match vault.unlock(PIN) {
                    Err(VaultError::IntegrityMismatch) => {}
                    other => panic!("expected IntegrityMismatch, got {other:?}"),
                }
            }
            // (b) CHECK ENTRY: reaches the guard, on the BOOT path. Must be an
            // error, never a panic inside `verify_check_entry`.
            {
                let dir = TempDir::new().unwrap();
                let vault = fast_vault(&dir);
                vault.init(PIN).unwrap();
                rewrite_entry_nonce(&vault.path, VAULT_CHECK_REF, bad_nonce.clone());
                match vault.unlock(PIN) {
                    Err(VaultError::CryptoReject) | Err(VaultError::BlobMalformed(_)) => {}
                    other => panic!("expected a fail-closed error, got {other:?}"),
                }
            }
        }
    }

    /// The two `0x1f` separators of the check-entry plaintext are part of its
    /// SHAPE: a plaintext of the right length with wrong framing must be refused
    /// (ex-review r2 — the marker and the widths were checked, the separators
    /// were not).
    #[test]
    fn check_entry_framing_is_verified_not_only_its_marker() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        // Re-seal a check entry whose separators are `0x00` instead of `0x1f`,
        // keeping the marker, the widths and a valid fingerprint.
        let kek = {
            let blob = std::fs::read(&vault.path).unwrap();
            let parsed = parse_header(&blob).unwrap();
            vault.derive_vault_kek(PIN, &parsed.salt).unwrap()
        };
        let blob = std::fs::read(&vault.path).unwrap();
        let ParsedVaultBlob {
            header,
            body: body_bytes,
            ..
        } = parse_header(&blob).unwrap();
        let mut body: VaultBody = serde_json::from_slice(body_bytes).unwrap();

        let mut plaintext = Vec::new();
        plaintext.extend_from_slice(VAULT_CHECK_MARKER);
        plaintext.push(0x00); // was SEP
        plaintext.extend_from_slice(&1u64.to_be_bytes());
        plaintext.push(0x00); // was SEP
        let mut stripped = body.clone();
        stripped.remove(VAULT_CHECK_REF);
        plaintext.extend_from_slice(&set_fingerprint(&stripped));

        let resealed =
            seal_entry(kek.expose_secret(), &header, VAULT_CHECK_REF, &plaintext).unwrap();
        body.insert(VAULT_CHECK_REF.to_string(), resealed);
        let mut rewritten = header.to_vec();
        rewritten.extend_from_slice(&serde_json::to_vec(&body).unwrap());
        std::fs::write(&vault.path, &rewritten).unwrap();

        match vault.unlock(PIN) {
            Err(VaultError::BlobMalformed(m)) => assert!(m.contains("framing"), "got {m}"),
            other => panic!("expected a framing refusal, got {other:?}"),
        }
    }

    /// TWIN parity with `sbfb_workflow_engine::hash::validate_identity_component`
    /// (`IDENTITY_MAX_LEN`), which the engine side pins symmetrically. The two
    /// crates do not depend on one another, so nothing but these two tests keeps
    /// the seam's domain single — and the unreachability of [`VAULT_CHECK_REF`]
    /// rests entirely on the control-character refusal below.
    #[test]
    fn validate_ref_pins_the_shared_seam_domain() {
        assert_eq!(SECRET_REF_MAX_LEN, 512, "twin of IDENTITY_MAX_LEN");
        assert!(matches!(validate_ref(""), Err(VaultError::InvalidRef(_)),));
        assert!(matches!(
            validate_ref(&"a".repeat(SECRET_REF_MAX_LEN + 1)),
            Err(VaultError::InvalidRef(_)),
        ));
        for bad in [
            "with\nnewline",
            "with\u{1f}separator",
            "with\u{0}nul",
            // Invisible / bidi FORMAT characters: refused for the READING, not
            // the encoding (ex-review r3 L1-R3-01). Same list as the engine twin.
            "with\u{202E}override", // = borne HAUTE de `202A..202E`
            "with\u{200B}zero-width",
            "with\u{2066}isolate",
            "with\u{FEFF}bom",
            // One witness per range added in ex-review r4 L1-R4-E2. Mirrors the
            // engine twin exactly: `U+2060` fell in the gap between `202E` and
            // `2066`, i.e. between two ranges that were already present.
            "with\u{2060}word-joiner",
            "with\u{00AD}soft-hyphen",
            "with\u{2028}line-separator",
            "with\u{E000}private-use",
            "with\u{E0041}tag-smuggling",
            "with\u{F0000}supplementary-private-use",
            // Plane 16 — mirrors the engine twin exactly (review r5 L1-R5-D1-03).
            "with\u{100000}plane16-private-use",
            // Bornes des plages multi-points — miroir exact du jumeau moteur
            // (mesure Codex r1 : un temoin ne prouve que SON point). Les NEUF
            // plages multi-points du predicat portent chacune son point bas ET
            // son point haut ; ajouter une plage sans ses deux bornes rouvre la
            // classe (review r7). L'ordre et les libelles sont byte-identiques
            // a `sbfb-workflow-engine/src/hitl.rs` pour que les deux listes se
            // relisent l'une contre l'autre.
            //   200B..200F : bas `200B` (plus haut), haut ici
            "with\u{200F}right-to-left-mark",
            //   202A..202E : haut `202E` (plus haut), bas ici
            "with\u{202A}left-to-right-embedding",
            //   2028..2029 : bas `2028` (plus haut), haut ici
            "with\u{2029}paragraph-separator",
            //   2060..2064 : bas `2060` (plus haut), haut ici
            "with\u{2064}invisible-plus",
            //   2066..2069 : bas `2066` (plus haut), haut ici
            "with\u{2069}pop-directional-isolate",
            //   E000..F8FF : bas `E000` (plus haut), haut ici
            "with\u{F8FF}private-use-end",
            //   F0000..FFFFD : bas `F0000` (plus haut), haut ici
            "with\u{FFFFD}supplementary-private-use-end",
            //   100000..10FFFD : bas `100000` (plus haut), haut ici
            "with\u{10FFFD}plane16-private-use-end",
            //   E0020..E007F : `E0041` (plus haut) est au MILIEU, donc les DEUX
            //   bornes manquaient — c'est la plage du smuggling ASCII invisible
            "with\u{E0020}tag-space",
            "with\u{E007F}cancel-tag",
            // C0 et C1 : `is_control()` couvre la categorie Cc ENTIERE, pas
            // seulement l'ASCII bas. Sans ces deux temoins, remplacer
            // `c.is_control()` par une comparaison `c < ' '` passait.
            "with\ttab",
            "with\u{0085}next-line",
            // Bornes de la categorie Cc, les DEUX blocs : `007F` DEL ferme C0,
            // `0080`..`009F` est C1 en entier. Sans eux, restreindre
            // `is_control()` a `c < ' '` — la forme la plus naturelle d'une
            // reecriture — passait sur DEL et sur tout C1.
            "with\u{7F}delete",
            "with\u{0080}c1-start",
            "with\u{009F}c1-end",
        ] {
            assert!(
                matches!(validate_ref(bad), Err(VaultError::InvalidRef(_))),
                "{bad:?} must be refused"
            );
        }
        assert!(validate_ref(&"a".repeat(SECRET_REF_MAX_LEN)).is_ok());
        validate_ref("cloud_provider_token").unwrap();
        // ACCEPTES, et c'est la moitie qui manquait : le domaine n'est PAS
        // borne a l'ASCII. Un identifiant non-ASCII VISIBLE passe — c'est
        // exactement ce qui laisse ouvert le spoof d'affichage par homoglyphes
        // (borne 24 de §19.4). Sans ces temoins, restreindre le predicat a
        // `c.is_ascii()` — un durcissement plausible — passait sans un rouge,
        // en cassant silencieusement tout identifiant non anglophone.
        // Liste byte-identique au jumeau `sbfb-workflow-engine/src/hitl.rs`.
        validate_ref("caf\u{00E9}_token").unwrap();
        validate_ref("\u{65E5}\u{672C}_token").unwrap();
    }

    /// Ex-review r3 F3: `delete` and `rotate` answered from this handle's
    /// SNAPSHOT before reading the disk, so a revocation issued through a stale
    /// handle was silently INOPERATIVE — `Ok(false)` read as "nothing to revoke"
    /// while the credential stayed on disk and kept being served. This is the
    /// fail-OPEN half of the class the epoch exists to close, and it was
    /// invisible to every other test because they all re-unlock the acting
    /// handle.
    ///
    /// SCOPE, so this test is not cited for more than it proves: it exercises a
    /// handle whose epoch is BEHIND the disk. It says NOTHING about the state of
    /// bound 27 of §19.4, where a lost-update collision leaves the handle
    /// carrying the disk's epoch over a stale BODY — there the guard passes and
    /// `delete` fails OPEN again. That residue is unlocked debt, not a
    /// regression this test would catch.
    #[test]
    fn delete_and_rotate_refuse_a_stale_handle_instead_of_answering_from_the_snapshot() {
        let dir = TempDir::new().unwrap();
        let vault = fast_vault(&dir);
        vault.init(PIN).unwrap();

        // The daemon's long-lived handle, opened when the vault was empty.
        let mut daemon = vault.unlock(PIN).unwrap();

        // A CLI adds a credential the daemon's snapshot has never seen.
        {
            let mut cli = vault.unlock(PIN).unwrap();
            cli.put("stripe_live_key", b"sk-live-REAL".to_vec())
                .unwrap();
        }

        // The operator revokes through the daemon. `Ok(false)` here would read
        // as "there was nothing to revoke" — the credential must NOT be able to
        // survive that answer.
        match daemon.delete("stripe_live_key") {
            Err(VaultError::ConcurrentModification { .. }) => {}
            Ok(false) => panic!(
                "delete answered 'nothing to revoke' from a stale snapshot — the \
                 credential is still on disk and will still be served"
            ),
            other => panic!("expected ConcurrentModification, got {other:?}"),
        }

        // Same class through `rotate`: refusing with `UnknownRef` would blame the
        // slot NAME for a staleness problem, and the rotation would not happen.
        match daemon.rotate("stripe_live_key", b"sk-live-NEW".to_vec()) {
            Err(VaultError::ConcurrentModification { .. }) => {}
            Err(VaultError::UnknownRef(_)) => {
                panic!("rotate blamed the slot name for a stale snapshot")
            }
            other => panic!("expected ConcurrentModification, got {other:?}"),
        }

        // After the documented recovery, both operations work and the revocation
        // is real: a FRESH handle no longer binds the slot.
        daemon.reload().unwrap();
        assert!(daemon.delete("stripe_live_key").unwrap());
        let fresh = vault.unlock(PIN).unwrap();
        assert!(matches!(
            fresh.bind("stripe_live_key"),
            Err(VaultError::UnknownRef(_))
        ));

        // And `Ok(false)` keeps its honest meaning on a FRESH handle: the
        // committed set really does not contain the ref.
        let mut fresh = vault.unlock(PIN).unwrap();
        assert!(!fresh.delete("never_existed").unwrap());
    }

    /// Overwrite one entry's nonce on disk, leaving everything else intact.
    fn rewrite_entry_nonce(path: &Path, secret_ref: &str, nonce: Vec<u8>) {
        let blob = std::fs::read(path).unwrap();
        let ParsedVaultBlob {
            header,
            body: body_bytes,
            ..
        } = parse_header(&blob).unwrap();
        let mut body: VaultBody = serde_json::from_slice(body_bytes).unwrap();
        let entry = body.get_mut(secret_ref).expect("entry present");
        entry.nonce = nonce;
        let mut rewritten = header.to_vec();
        rewritten.extend_from_slice(&serde_json::to_vec(&body).unwrap());
        std::fs::write(path, &rewritten).unwrap();
    }
}
