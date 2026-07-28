// SPDX-License-Identifier: AGPL-3.0-or-later
//! The deterministic echo target — the first effect target of the engine.
//!
//! An effect target is the OUTSIDE of the transaction: whatever it does is
//! not covered by the store's atomicity, which is exactly why the B2
//! frontier is irreducible. The echo target models that faithfully instead
//! of pretending: its side output is an fsync'd append to a plain text file
//! OUTSIDE the SQLite store, so a side output survives the death of the
//! process that produced it and can be counted afterwards by a different
//! process. An in-memory counter would have made the harness prove nothing.
//!
//! Determinism: the receipt is the JCS canonicalisation of the input, so the
//! same input always yields byte-identical output, on any machine, with no
//! clock and no randomness involved.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::hash::{HashError, IdempotencyKey};

/// Field separator inside one side-output line.
///
/// Neither field can contain it: the receipt is JCS, which escapes control
/// characters, and the key components are checked by
/// [`crate::hash::validate_identity_component`], which refuses EVERY control
/// character (and, since ex-review r3 L1-R3-01, every invisible or bidi format
/// character too — a wider domain, so this separator argument still holds).
/// Note what this does NOT say — the ids are not "ASCII by
/// construction"; they are free-form paths whose control characters are
/// REFUSED at a gate. That distinction is the whole point: the earlier
/// "by construction" wording described an invariant nothing enforced.
const FIELD_SEPARATOR: char = '\t';

/// Name of the echo target, part of every step request it serves.
pub const ECHO_TARGET_NAME: &str = "echo";

/// An effect target failed.
#[derive(Debug, thiserror::Error)]
pub enum TargetError {
    /// The side-output log could not be read or appended to.
    #[error("effect target io: {0}")]
    Io(#[from] std::io::Error),
    /// The input could not be canonicalised into a receipt.
    #[error("effect target canonicalisation: {0}")]
    Hash(#[from] HashError),
    /// A side-output line does not have the `key<TAB>receipt` shape.
    #[error("side-output log corrupt at line {line}")]
    Corrupt {
        /// 1-based line number.
        line: usize,
    },
}

/// One observable side output: the effect actually reached the outside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideOutput {
    /// Rendered idempotency key, see [`IdempotencyKey::as_line`].
    pub key_line: String,
    /// What the target returned for it.
    pub receipt: String,
}

/// Something a step can address.
pub trait EffectTarget {
    /// Stable target name, part of the hashed step request.
    fn name(&self) -> &str;

    /// Whether the target collapses two applications of the SAME
    /// idempotency key into one side output.
    ///
    /// This is a CLAIM the engine trusts to decide whether a crashed,
    /// unconfirmed effect may be re-applied. A target that claims it without
    /// honouring it produces a double side output — the lying-target red
    /// control does exactly that on purpose.
    fn is_idempotent(&self) -> bool;

    /// Stable identity of the DURABLE STORE this target deduplicates in.
    ///
    /// [`EffectTarget::name`] is not enough: two honest targets can share a
    /// name and deduplicate in two different backends. Resuming against the
    /// second one would pass every guard — same name, same idempotency claim,
    /// same key — and still produce a second side output, because the first
    /// backend is where the receipt lives. The engine journals this scope at
    /// intent time and refuses a resume whose token differs.
    ///
    /// The engine compares TOKENS; it cannot verify that equal tokens mean
    /// the same physical store. An implementation whose token is not globally
    /// discriminating (the file target below returns its path AS INJECTED, so
    /// the same relative path from two working directories collides) weakens
    /// the guarantee to exactly what the token distinguishes.
    fn dedup_scope(&self) -> String;

    /// Apply the effect and return its receipt.
    fn apply(
        &mut self,
        key: &IdempotencyKey,
        input: &BTreeMap<String, String>,
    ) -> Result<String, TargetError>;
}

/// Deterministic echo target writing its side outputs to an injected path.
#[derive(Debug, Clone)]
pub struct FileEchoTarget {
    log_path: PathBuf,
    deduplicates: bool,
    declares_idempotent: bool,
}

impl FileEchoTarget {
    /// The honest idempotent target: an already-applied key returns the
    /// recorded receipt and appends nothing.
    pub fn idempotent(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: log_path.into(),
            deduplicates: true,
            declares_idempotent: true,
        }
    }

    /// RED CONTROL — claims idempotency, does not deduplicate. Proves that
    /// "one observable side output" comes from the target's deduplication and
    /// not from the engine alone.
    ///
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn lying(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: log_path.into(),
            deduplicates: false,
            declares_idempotent: true,
        }
    }

    /// An honest non-idempotent target: it does not deduplicate and does not
    /// claim to. The engine must refuse to retry an unconfirmed effect
    /// against it.
    pub fn non_idempotent(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: log_path.into(),
            deduplicates: false,
            declares_idempotent: false,
        }
    }

    /// Path of the side-output log.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
}

impl EffectTarget for FileEchoTarget {
    fn name(&self) -> &str {
        ECHO_TARGET_NAME
    }

    fn is_idempotent(&self) -> bool {
        self.declares_idempotent
    }

    fn dedup_scope(&self) -> String {
        self.log_path.display().to_string()
    }

    fn apply(
        &mut self,
        key: &IdempotencyKey,
        input: &BTreeMap<String, String>,
    ) -> Result<String, TargetError> {
        let key_line = key.as_line();

        if self.deduplicates {
            for existing in read_side_outputs(&self.log_path)? {
                if existing.key_line == key_line {
                    return Ok(existing.receipt);
                }
            }
        }

        // Same canonicalisation door as the digests: one serialization path
        // for the whole crate, so a value the digest would refuse cannot
        // sneak in through the receipt.
        let receipt = crate::hash::canonical_string(input)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{key_line}{FIELD_SEPARATOR}{receipt}")?;
        // The side output must outlive an abrupt process exit: without this
        // the line could still be sitting in a buffer when the writer dies,
        // and the harness would credit the engine for a durability the
        // target never provided.
        file.sync_all()?;

        Ok(receipt)
    }
}

/// Read every side output recorded so far. Usable from a process that did
/// not produce them — that is how the out-of-process harness counts what a
/// killed writer left behind.
///
/// A missing log is an empty list, not an error: no effect has been applied
/// yet.
pub fn read_side_outputs(path: &Path) -> Result<Vec<SideOutput>, TargetError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(TargetError::Io(e)),
    };
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;

    let mut out = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let (key_line, receipt) = line
            .split_once(FIELD_SEPARATOR)
            .ok_or(TargetError::Corrupt { line: index + 1 })?;
        out.push(SideOutput {
            key_line: key_line.to_string(),
            receipt: receipt.to_string(),
        });
    }
    Ok(out)
}

/// How many side outputs were recorded for one idempotency key. The
/// falsifiable quantity of the whole spike: it must be at most 1.
pub fn side_output_count(path: &Path, key: &IdempotencyKey) -> Result<usize, TargetError> {
    let key_line = key.as_line();
    Ok(read_side_outputs(path)?
        .into_iter()
        .filter(|s| s.key_line == key_line)
        .count())
}
