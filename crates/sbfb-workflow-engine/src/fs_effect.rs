// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem effects bound to a WORKTREE STATE precondition (Sprint 83
//! Phase K): an FS mutation planned against one repository state refuses
//! to run against another — a CAS on the worktree, the `git update-ref
//! --stdin` verify-then-apply pattern applied to files.
//!
//! ## Host neutrality
//!
//! The engine reads NO git and shells NO command here: the EXPECTED state
//! is injected at construction, and the ACTUAL state comes from an
//! injected [`WorktreeStateProvider`] (host-side, the adapter that shells
//! `git rev-parse` / `git status` lives with the host — Phase O). The
//! crate's charter (no env, no home, no hidden path) holds.
//!
//! ## Confinement (THREAT §19.2 T-RUN-FS-EFFECT-ESCAPE)
//!
//! The target path is worktree-RELATIVE: absolute paths and `..`
//! components are refused syntactically, then the CANONICALISED parent
//! must stay under the canonicalised root — a symlink in the PATH resolves
//! outside and is refused. The FINAL component is checked separately: if it
//! already exists as a symlink, the effect is refused rather than followed
//! (else a leaf symlink `wt/out -> /etc/x` would let `File::create` write
//! through it, outside the tree). Refusal happens BEFORE any byte is
//! written.
//!
//! ## Honest TOCTOU bound
//!
//! The precondition is verified immediately before the write, in the same
//! single-writer pass — there is no OS transaction over "compare worktree
//! state, then write". An EXTERNAL mutator racing inside that window can
//! still interleave; what the precondition closes is acting on a plan
//! made against a state that is ALREADY gone. Stated, not glossed over.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::activity::{Activity, ActivityError, EffectContext, PollOutcome};

/// The worktree state an FS effect is bound to. All three components are
/// DIGESTS/IDS computed host-side; the engine only compares them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeState {
    /// Commit id HEAD points at.
    pub head: String,
    /// Digest of the index.
    pub index_digest: String,
    /// Digest of the working tree's tracked content.
    pub worktree_digest: String,
}

/// Where the ACTUAL worktree state comes from — injected, so the engine
/// stays host-neutral and the precondition is testable hermetically.
pub trait WorktreeStateProvider: std::fmt::Debug {
    /// The state of the worktree NOW.
    fn current_state(&self) -> Result<WorktreeState, ActivityError>;
}

/// A fixed provider: what tests (and a host that already resolved the
/// state this pass) inject.
#[derive(Debug, Clone)]
pub struct FixedWorktreeState(pub WorktreeState);

impl WorktreeStateProvider for FixedWorktreeState {
    fn current_state(&self) -> Result<WorktreeState, ActivityError> {
        Ok(self.0.clone())
    }
}

/// Write one file inside a worktree, guarded by the state precondition.
/// Input contract: the single inbound port carries
/// `{"path": <worktree-relative>, "content": <string>}`.
///
/// Idempotent BY CONTENT: the same request writes the same bytes at the
/// same path, and an already-identical file is left untouched — that is
/// what the declared idempotency claims, nothing more.
#[derive(Debug)]
pub struct WorktreeFsWriteActivity {
    /// FROZEN at construction: the canonical (absolute) form when the root
    /// resolves, the raw spelling otherwise (ex-Codex r4
    /// K-FS-ROOT-FREEZE-02 — one resolution, not two live resolutions that
    /// a cwd change or a symlink retarget could make disagree between the
    /// scope guard and `confine`).
    worktree_root: PathBuf,
    provider: Box<dyn WorktreeStateProvider + Send>,
    expected: WorktreeState,
    /// Ready outputs stashed by `start`, keyed by the rendered key.
    ready: BTreeMap<String, serde_json::Value>,
}

impl WorktreeFsWriteActivity {
    /// An FS-write activity over `worktree_root`, bound to `expected`.
    /// The root is frozen here (see the field doc): callers should hand an
    /// EXISTING root; a root that only resolves later keeps its raw
    /// spelling in the durable scope, and a resume that then resolves it
    /// canonically fails closed at `TargetScopeChanged` — EXCEPT when the
    /// raw spelling already IS the canonical absolute form, where the
    /// token (same string, same tree) simply stays stable and the state
    /// precondition carries the semantics (clause-E declared tokens) — and
    /// EXCEPT a resume whose root is ALSO absent at construction with the
    /// same RELATIVE raw spelling: both incarnations then share one raw
    /// scope while `confine` resolves each against ITS ambient cwd — there
    /// the state precondition is the remaining guard (defense-in-depth),
    /// which is why an existing root is the supported input.
    pub fn new(
        worktree_root: impl Into<PathBuf>,
        provider: Box<dyn WorktreeStateProvider + Send>,
        expected: WorktreeState,
    ) -> Self {
        let worktree_root = worktree_root.into();
        let worktree_root = worktree_root.canonicalize().unwrap_or(worktree_root);
        Self {
            worktree_root,
            provider,
            expected,
            ready: BTreeMap::new(),
        }
    }

    /// Parse the effect input into `(relative path, content)`.
    fn parse_request(input: &BTreeMap<String, String>) -> Result<(String, String), ActivityError> {
        if input.len() != 1 {
            return Err(ActivityError::Refused(format!(
                "fs write takes exactly one inbound port, got {}",
                input.len()
            )));
        }
        let jcs = input.values().next().expect("len checked");
        let value: serde_json::Value = serde_json::from_str(jcs)
            .map_err(|e| ActivityError::Refused(format!("input is not JSON: {e}")))?;
        let path = value
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ActivityError::Refused("missing string field `path`".to_string()))?;
        let content = value
            .get("content")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ActivityError::Refused("missing string field `content`".to_string()))?;
        Ok((path.to_string(), content.to_string()))
    }

    /// Confinement: refuse absolute paths and `..` syntactically, then
    /// require the canonicalised PARENT to stay under the canonicalised
    /// root (resolves symlinks — an escaping link lands outside and is
    /// refused). Returns the absolute target path.
    fn confine(&self, relative: &str) -> Result<PathBuf, ActivityError> {
        let rel = Path::new(relative);
        if rel.is_absolute() {
            return Err(ActivityError::Refused(format!(
                "absolute path refused: {relative}"
            )));
        }
        for component in rel.components() {
            match component {
                Component::Normal(_) => {}
                Component::CurDir => {}
                _ => {
                    return Err(ActivityError::Refused(format!(
                        "path component refused in: {relative}"
                    )));
                }
            }
        }
        // Re-canonicalise the FROZEN base: identity in the nominal case
        // (the frozen form is already canonical), fail-closed refusal when
        // the tree vanished. A base frozen RAW (root absent at
        // construction) resolves HERE against the live cwd — the one
        // remaining live resolution, and its durable scope keeps the raw
        // spelling: a later resume re-freezes canonically and fails closed
        // at `TargetScopeChanged` (see `new`).
        let root = self
            .worktree_root
            .canonicalize()
            .map_err(|e| ActivityError::Refused(format!("worktree root does not resolve: {e}")))?;
        let target = root.join(rel);
        let parent = target
            .parent()
            .ok_or_else(|| ActivityError::Refused("path has no parent".to_string()))?;
        let parent = parent
            .canonicalize()
            .map_err(|e| ActivityError::Refused(format!("target parent does not resolve: {e}")))?;
        if !parent.starts_with(&root) {
            return Err(ActivityError::Refused(format!(
                "path escapes the worktree: {relative}"
            )));
        }
        let file_name = target
            .file_name()
            .ok_or_else(|| ActivityError::Refused("path has no file name".to_string()))?;
        let final_target = parent.join(file_name);
        // The parent is canonicalised, but the LEAF is not — an existing
        // leaf symlink would be followed by `File::create`/`read` and land
        // outside the tree. And a special file (FIFO, socket, device) would
        // make the idempotent-collapse `fs::read` BLOCK or turn the effect
        // into an IPC. Accept only: NOT-FOUND (the nominal first write) or
        // an existing REGULAR file (an idempotent overwrite). Refuse
        // everything else.
        match std::fs::symlink_metadata(&final_target) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(ActivityError::Refused(format!(
                    "path resolves to a symlink, refused: {relative}"
                )));
            }
            Ok(meta) if meta.is_file() => {}
            Ok(_) => {
                return Err(ActivityError::Refused(format!(
                    "path resolves to a non-regular file, refused: {relative}"
                )));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(ActivityError::Refused(format!(
                    "target leaf does not resolve: {e}"
                )));
            }
        }
        Ok(final_target)
    }
}

impl Activity for WorktreeFsWriteActivity {
    fn name(&self) -> &str {
        "worktree-fs-write"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn dedup_scope(&self) -> String {
        // The scope binds the worktree root AND the EXPECTED state this
        // effect was planned against (head/index/worktree digests). Without
        // the state, a resume that reconstructs the activity against a
        // DIFFERENT expected state would pass every durable guard (the
        // request hash covers node/name/input, not the precondition) and
        // mutate a tree the plan was never made for. Folding it into the
        // scope makes such a resume fail closed at `TargetScopeChanged`
        // (ex-Codex K-FS-STATE-BIND-01). The root is the FROZEN
        // constructor form — canonical absolute when it resolved
        // (ex-Codex r3 K-FS-ROOT-REL: a relative root kept raw would map
        // to a different tree after a resume in another working directory
        // while keeping the same scope string), raw otherwise (see `new`
        // for the honest bound of that fallback). No live resolution here
        // (ex-Codex r4 K-FS-ROOT-FREEZE-02). Canonical JCS, not a
        // separator join — injective on the strings, a state containing
        // the separator byte cannot shift field boundaries (ex-Codex r4
        // K-SCOPE-PATH-ENC-03) ; honest bound: `display()` of a non-UTF-8
        // path is LOSSY (exotic Unix spellings can collide before JCS) —
        // same OS mono-user trust boundary as the rest of the store.
        let subject = serde_json::json!({
            "root": self.worktree_root.display().to_string(),
            "expected_head": self.expected.head,
            "expected_index_digest": self.expected.index_digest,
            "expected_worktree_digest": self.expected.worktree_digest,
        });
        crate::hash::canonical_string(&subject).unwrap_or_else(|_| {
            // Unreachable (no null/non-finite floats); a stable fallback
            // keeps the method total rather than panicking.
            format!("{}", self.worktree_root.display())
        })
    }

    fn start(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        // Precondition FIRST, before any parse-independent work touches
        // the tree: a plan made against a state that is gone must not
        // mutate anything (K.3 `fs_effect_bound_to_head_index_worktree`).
        let actual = self.provider.current_state()?;
        if actual != self.expected {
            return Err(ActivityError::PreconditionMismatch(format!(
                "worktree diverged: expected head {}, found {}",
                self.expected.head, actual.head
            )));
        }
        let (relative, content) = Self::parse_request(&ctx.input)?;
        let target = self.confine(&relative)?;

        // Idempotent collapse: identical content already in place appends
        // nothing and rewrites nothing.
        let already = std::fs::read(&target)
            .ok()
            .is_some_and(|bytes| bytes == content.as_bytes());
        if !already {
            let mut file = std::fs::File::create(&target)?;
            file.write_all(content.as_bytes())?;
            // The mutation must outlive an abrupt process exit — same
            // rationale as every side output of this crate.
            file.sync_all()?;
        }
        self.ready.insert(
            ctx.key.as_line(),
            serde_json::json!({
                "path": relative,
                "bytes": content.len(),
            }),
        );
        Ok(())
    }

    fn poll(&mut self, ctx: &EffectContext) -> Result<PollOutcome, ActivityError> {
        match self.ready.get(&ctx.key.as_line()) {
            Some(output) => {
                let mut facts = BTreeMap::new();
                facts.insert("kind".to_string(), "fs-write".to_string());
                Ok(PollOutcome::Ready {
                    output: output.clone(),
                    facts,
                })
            }
            None => Err(ActivityError::UnknownAttempt),
        }
    }

    fn cancel(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        self.ready.remove(&ctx.key.as_line());
        Ok(())
    }
}
