# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 22 Phase E — watermark canari-input spot-check primitive.

Defense against ``C-ComputeTheft`` silent-model-swap : a worker that
silently runs a cheaper model than the one the task requested will
fail known-answer probes inserted 1/N tasks by the coordinator. The
probe is indistinguishable from a real task on the wire — the worker
cannot selectively cheat on "real" tasks because it cannot tell
which ones are canaries.

**Design-distinct** from watermark-output Kirchenbauer ICML 2023
(green-list logit biasing), which is vulnerable to the BIRA attack
(arXiv 2509.23019, Sept 2025). Input-side prompt-probe uses no
logit manipulation, is model-agnostic, and is immune to rewrite
attacks at the token level.

Three collaborating objects :

:class:`CanaryInputSet`
    Rotatable pydantic model carrying a list of
    :class:`CanaryPrompt` entries (prompt + expected_answer +
    optional per-entry tolerance) plus an Ed25519 signature of the
    canonical JSON of the payload. The signature is **local
    integrity only** (file-watcher reload + anti-tamper on disk),
    not a wire P2P primitive — parallel pattern to the Sprint 18
    Phase D ``TokenRotator`` signed ``tokens.json``.

:class:`CanaryInputInjector`
    Pre-dispatch sampling hook. :meth:`should_inject` returns True
    ~1/N times (N = ``inject_rate``, default 100), at which point
    :meth:`next_prompt` yields the next probe in round-robin order.
    Round-robin (rather than random) ensures every prompt gets
    exercised — a single poisoned entry cannot silently bias
    detection toward false negatives.

:class:`CanaryInputObserver`
    Post-result hook. :meth:`observe` looks up the probe by
    ``prompt_id``, computes the normalized Levenshtein similarity
    between ``expected_answer`` and the worker's actual output
    (via ``rapidfuzz.distance.Levenshtein.normalized_similarity``,
    reuse of the Sprint 21 Phase C EED echo pattern), and records
    a :class:`DivergenceRecord` in a bounded ring buffer if
    ``similarity < tolerance``. Durable alerting lands in Sprint 23
    B1 Guardrails refactor — this sprint delivers the primitive
    only.

:class:`CanaryInputManager`
    Glue that owns the file-watched policy TOML and the signed set
    on disk, plus the Injector+Observer pair. Hot-reload mirrors
    :class:`~nexus_coordinator.output_filter.OutputFilter` :
    50 ms mtime debounce + malformed-reload guard + file-deletion
    keep-last.

Refs :
- plan §8 (Sprint 22) + kickoff §4 D4
- ``docs/security/HARDENING_ROADMAP.md §3 S22`` ligne 294-296
- reuse ``nexus_core.sign_bytes`` / ``verify_bytes`` (Sprint 14
  Phase A raw Ed25519 surface) for local set integrity
"""

from __future__ import annotations

import json
import random
import threading
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path

import nexus_core
import structlog
from pydantic import BaseModel, ConfigDict
from rapidfuzz.distance import Levenshtein

_log = structlog.get_logger(__name__)


CANARY_INPUT_SET_VERSION = 1
"""Wire-level version tag on the signed :class:`CanaryInputSet`.

Pre-launch protocol policy (cf. ``CLAUDE.md`` + memory
``nexus_grid_pivot.md §Decisions actees``) : stays at 1 until tag
``v1.0``. Any design change redefines the v1 canonical, no
tolerant multi-version decoder is introduced. Kept explicit here
so a future post-v1.0 bump has a single constant to touch.
"""

DEFAULT_INJECT_RATE = 100
"""Default 1-per-N sampling rate. Operators tune via TOML policy."""

DEFAULT_TOLERANCE = 0.85
"""Default Levenshtein similarity threshold below which a worker
answer triggers a divergence alert. Mirrors the Sprint 21 Phase C
``eed_threshold`` on ``OutputFilter`` — empirical tuning is
deployment-specific so the default is overridable per-prompt and
per-policy.
"""

DEFAULT_ROTATION_FREQUENCY_DAYS = 30
"""Advisory rotation cadence. Not enforced by the library — the
operator runs ``nexus-coordinator canary rotate`` on their own
schedule. The field is surfaced in the policy so monitoring
tooling (Sprint 23+) can warn on staleness.
"""

_MTIME_DEBOUNCE_SECS = 0.05


# ---------------------------------------------------------------------------
# Wire models
# ---------------------------------------------------------------------------


class CanaryPrompt(BaseModel):
    """A single known-answer probe.

    ``prompt_id`` is a short stable identifier the :class:`Observer`
    uses to correlate a worker result back to its probe. The probe
    itself is a plain string + an expected answer the Observer
    compares against via normalized Levenshtein. The optional
    ``tolerance`` lets a probe override the global default — useful
    for long free-form answers where 0.85 is too strict.
    """

    model_config = ConfigDict(frozen=True)

    prompt_id: str
    prompt: str
    expected_answer: str
    tolerance: float = DEFAULT_TOLERANCE


class CanaryInputSet(BaseModel):
    """Rotatable signed bundle of :class:`CanaryPrompt` probes.

    The signature is computed over :meth:`signable_json` — the
    canonical (``sort_keys=True``) JSON of ``{version, created_at_unix,
    prompts}``. The signer pubkey is embedded as ``coord_pubkey_hex``
    so a verifier reading the file from disk can validate it without
    a sidecar pubkey store, and the :meth:`_effective_set_path`
    reload path can reject a set signed by an unexpected key.
    """

    model_config = ConfigDict(frozen=False)

    version: int = CANARY_INPUT_SET_VERSION
    created_at_unix: int
    prompts: list[CanaryPrompt]
    coord_pubkey_hex: str
    signature_hex: str

    def signable_json(self) -> str:
        """Canonical JSON used as the signed message."""
        payload = {
            "version": self.version,
            "created_at_unix": self.created_at_unix,
            "prompts": [p.model_dump() for p in self.prompts],
        }
        return json.dumps(payload, sort_keys=True)


@dataclass
class DivergenceRecord:
    """A single worker-answer divergence recorded by the Observer."""

    prompt_id: str
    observed_at_unix: int
    similarity: float
    expected_answer: str
    observed_answer: str
    worker_pubkey_hex: str | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "prompt_id": self.prompt_id,
            "observed_at_unix": self.observed_at_unix,
            "similarity": self.similarity,
            "expected_answer": self.expected_answer,
            "observed_answer": self.observed_answer,
            "worker_pubkey_hex": self.worker_pubkey_hex,
        }


@dataclass
class CanaryInputPolicy:
    """In-memory shape of ``canary_input_policy.toml`` + defaults.

    ``set_path`` is optional: if the TOML omits it, the caller
    (typically :class:`CanaryInputManager`) supplies a default
    under :func:`nexus_coordinator.paths.canary_input_set_path`.
    """

    enabled: bool = True
    inject_rate: int = DEFAULT_INJECT_RATE
    default_tolerance: float = DEFAULT_TOLERANCE
    rotation_frequency_days: int = DEFAULT_ROTATION_FREQUENCY_DAYS
    set_path: str | None = None

    @classmethod
    def from_toml(cls, text: str) -> "CanaryInputPolicy":
        data = tomllib.loads(text) if text else {}
        default = data.get("default", {}) if isinstance(data, dict) else {}
        return cls(
            enabled=bool(default.get("enabled", True)),
            inject_rate=int(default.get("inject_rate", DEFAULT_INJECT_RATE)),
            default_tolerance=float(default.get("default_tolerance", DEFAULT_TOLERANCE)),
            rotation_frequency_days=int(
                default.get("rotation_frequency_days", DEFAULT_ROTATION_FREQUENCY_DAYS),
            ),
            set_path=default.get("set_path") or None,
        )


# ---------------------------------------------------------------------------
# Sign + verify helpers
# ---------------------------------------------------------------------------


def build_canary_input_set(
    prompts: list[CanaryPrompt],
    coord_secret: bytes,
    coord_pubkey: bytes,
    *,
    now_unix: int | None = None,
) -> CanaryInputSet:
    """Produce a :class:`CanaryInputSet` with a fresh Ed25519 signature.

    The caller is expected to hand over ``(secret, public)`` from
    :attr:`~nexus_coordinator.keystore.LoadedKeypair` — no key
    derivation happens here. Signing is done via ``nexus_core.sign_bytes``
    (Sprint 14 Phase A raw surface) on the canonical
    :meth:`CanaryInputSet.signable_json` bytes.
    """
    created = int(time.time()) if now_unix is None else now_unix
    unsigned = CanaryInputSet(
        version=CANARY_INPUT_SET_VERSION,
        created_at_unix=created,
        prompts=prompts,
        coord_pubkey_hex=coord_pubkey.hex(),
        signature_hex="",
    )
    sig = nexus_core.sign_bytes(unsigned.signable_json().encode("utf-8"), coord_secret)
    return unsigned.model_copy(update={"signature_hex": bytes(sig).hex()})


def verify_canary_input_set(
    canary_set: CanaryInputSet,
    *,
    expected_pubkey: bytes | None = None,
) -> None:
    """Verify the signature on a :class:`CanaryInputSet`. Raises on failure.

    If ``expected_pubkey`` is supplied, the embedded
    ``coord_pubkey_hex`` must match — this is the guard against a
    file on disk signed by a rogue key that happens to produce a
    valid signature for its own pubkey.
    """
    if canary_set.version != CANARY_INPUT_SET_VERSION:
        raise ValueError(
            f"unsupported canary_input_set version {canary_set.version!r}, expected {CANARY_INPUT_SET_VERSION}",
        )
    try:
        sig_bytes = bytes.fromhex(canary_set.signature_hex)
        pubkey_bytes = bytes.fromhex(canary_set.coord_pubkey_hex)
    except ValueError as exc:
        raise ValueError(f"bad hex in canary_input_set: {exc}") from exc
    if expected_pubkey is not None and pubkey_bytes != expected_pubkey:
        raise ValueError("canary_input_set signed by unexpected pubkey")
    message = canary_set.signable_json().encode("utf-8")
    nexus_core.verify_bytes(message, sig_bytes, pubkey_bytes)


def save_canary_input_set(canary_set: CanaryInputSet, path: Path) -> None:
    """Write a :class:`CanaryInputSet` to disk as pretty JSON.

    Parent directory is created on demand. No temp-swap atomic
    rename — the file is only ever read by the same process that
    writes it (single-coordinator per-user invariant).
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(canary_set.model_dump_json(indent=2), encoding="utf-8")


def load_canary_input_set(
    path: Path,
    *,
    expected_pubkey: bytes | None = None,
) -> CanaryInputSet:
    """Read + verify a signed :class:`CanaryInputSet` from disk."""
    text = path.read_text(encoding="utf-8")
    data = json.loads(text)
    canary_set = CanaryInputSet.model_validate(data)
    verify_canary_input_set(canary_set, expected_pubkey=expected_pubkey)
    return canary_set


# ---------------------------------------------------------------------------
# Injector + Observer
# ---------------------------------------------------------------------------


class CanaryInputInjector:
    """Pre-dispatch sampling hook.

    ``should_inject`` + ``next_prompt`` are split so the caller can
    decide to inject without necessarily committing to a prompt (e.g.
    when no active set is loaded). The round-robin index and counters
    are guarded by a lock — the Dispatcher is async and may call this
    from multiple concurrent submit paths.
    """

    def __init__(
        self,
        canary_set: CanaryInputSet | None,
        *,
        inject_rate: int = DEFAULT_INJECT_RATE,
        rng: random.Random | None = None,
    ) -> None:
        self._canary_set = canary_set
        self._inject_rate = max(1, int(inject_rate))
        self._rng = rng if rng is not None else random.Random()
        self._lock = threading.Lock()
        self._rr_index = 0
        self._seen_count = 0
        self._injected_count = 0

    def should_inject(self) -> bool:
        with self._lock:
            self._seen_count += 1
            if self._canary_set is None or not self._canary_set.prompts:
                return False
            if self._inject_rate <= 1:
                self._injected_count += 1
                return True
            draw = self._rng.randint(1, self._inject_rate)
            if draw == 1:
                self._injected_count += 1
                return True
            return False

    def next_prompt(self) -> CanaryPrompt | None:
        with self._lock:
            if self._canary_set is None or not self._canary_set.prompts:
                return None
            idx = self._rr_index % len(self._canary_set.prompts)
            self._rr_index += 1
            return self._canary_set.prompts[idx]

    def set_canary_set(self, new_set: CanaryInputSet | None) -> None:
        with self._lock:
            self._canary_set = new_set
            self._rr_index = 0

    def set_inject_rate(self, new_rate: int) -> None:
        with self._lock:
            self._inject_rate = max(1, int(new_rate))

    @property
    def inject_rate(self) -> int:
        with self._lock:
            return self._inject_rate

    @property
    def stats(self) -> dict[str, int]:
        with self._lock:
            return {"seen": self._seen_count, "injected": self._injected_count}


class CanaryInputObserver:
    """Post-result hook. Records divergences in a bounded ring."""

    def __init__(
        self,
        canary_set: CanaryInputSet | None,
        *,
        default_tolerance: float = DEFAULT_TOLERANCE,
        ring_capacity: int = 100,
    ) -> None:
        self._canary_set = canary_set
        self._default_tolerance = default_tolerance
        self._ring_capacity = max(1, int(ring_capacity))
        self._lock = threading.Lock()
        self._ring: list[DivergenceRecord] = []
        self._observed_count = 0
        self._alerts_count = 0

    def observe(
        self,
        prompt_id: str,
        observed_answer: str,
        *,
        worker_pubkey_hex: str | None = None,
        now_unix: int | None = None,
    ) -> bool:
        """Evaluate an incoming worker answer against the probe.

        Returns ``True`` when similarity < tolerance (divergence
        recorded), ``False`` otherwise. An unknown ``prompt_id`` or
        a missing set both return ``False`` without recording —
        observer state is only mutated on real hits.
        """
        with self._lock:
            self._observed_count += 1
            if self._canary_set is None:
                return False
            prompt = next(
                (p for p in self._canary_set.prompts if p.prompt_id == prompt_id),
                None,
            )
            if prompt is None:
                return False
            tolerance = prompt.tolerance if prompt.tolerance is not None else self._default_tolerance
            similarity = float(
                Levenshtein.normalized_similarity(
                    prompt.expected_answer,
                    observed_answer or "",
                ),
            )
            if similarity >= tolerance:
                return False
            ts = int(time.time()) if now_unix is None else int(now_unix)
            record = DivergenceRecord(
                prompt_id=prompt_id,
                observed_at_unix=ts,
                similarity=similarity,
                expected_answer=prompt.expected_answer,
                observed_answer=observed_answer or "",
                worker_pubkey_hex=worker_pubkey_hex,
            )
            self._ring.append(record)
            if len(self._ring) > self._ring_capacity:
                self._ring = self._ring[-self._ring_capacity :]
            self._alerts_count += 1
            return True

    def recent_divergences(self, limit: int = 50) -> list[DivergenceRecord]:
        limit = max(0, int(limit))
        with self._lock:
            if limit == 0:
                return []
            return list(self._ring[-limit:])

    def set_canary_set(self, new_set: CanaryInputSet | None) -> None:
        with self._lock:
            self._canary_set = new_set

    @property
    def stats(self) -> dict[str, int]:
        with self._lock:
            return {
                "observed": self._observed_count,
                "alerts": self._alerts_count,
                "ring_size": len(self._ring),
            }


# ---------------------------------------------------------------------------
# Manager glue (policy hot-reload + signed-set reload)
# ---------------------------------------------------------------------------


@dataclass
class _ReloadState:
    """Mutable tracking state used by the manager's reload tick."""

    policy_mtime: float | None = None
    set_mtime: float | None = None
    last_reload_check: float = 0.0


class CanaryInputManager:
    """Owns the file-watched policy + signed set, wires Injector+Observer.

    The Coordinator instantiates one per project and exposes it via
    ``coord.canary_input`` so the Dispatcher and Validator can pull
    the primitive off a single object. Hot-reload mirrors the Sprint
    21 Phase C :class:`OutputFilter` pattern (50 ms mtime debounce +
    malformed-reload guard + file-deletion keep-last).

    The manager intentionally does NOT spawn a background thread —
    reload is pulled on every ``maybe_inject`` / ``observe_result``
    call. That keeps the component test-friendly (no lifecycle to
    manage in fixtures) and the call overhead is negligible
    (``stat()`` + compare).
    """

    def __init__(
        self,
        *,
        policy_path: Path | None = None,
        canary_set_path: Path | None = None,
        coord_pubkey: bytes | None = None,
        rng: random.Random | None = None,
    ) -> None:
        self._policy_path = policy_path
        self._canary_set_path = canary_set_path
        self._coord_pubkey = coord_pubkey
        self._policy = CanaryInputPolicy()
        self._reload_state = _ReloadState()
        self._lock = threading.Lock()

        if policy_path is not None and policy_path.exists():
            # Safe to call `_locked` suffix method here without acquiring
            # the lock: __init__ runs single-threaded by construction —
            # no other reference to ``self`` exists yet for a concurrent
            # reload to race against.
            self._reload_policy_locked()

        initial_set: CanaryInputSet | None = None
        target = self._effective_set_path()
        if target is not None and target.exists():
            try:
                initial_set = load_canary_input_set(
                    target,
                    expected_pubkey=self._coord_pubkey,
                )
                self._reload_state.set_mtime = target.stat().st_mtime
            except Exception as exc:  # noqa: BLE001
                _log.warning(
                    "canary_input_set_load_failed",
                    path=str(target),
                    error=str(exc),
                )

        self._injector = CanaryInputInjector(
            initial_set,
            inject_rate=self._policy.inject_rate,
            rng=rng,
        )
        self._observer = CanaryInputObserver(
            initial_set,
            default_tolerance=self._policy.default_tolerance,
        )

    # -- public surface -------------------------------------------------

    @property
    def policy(self) -> CanaryInputPolicy:
        return self._policy

    @property
    def injector(self) -> CanaryInputInjector:
        return self._injector

    @property
    def observer(self) -> CanaryInputObserver:
        return self._observer

    @property
    def current_set(self) -> CanaryInputSet | None:
        """Return the :class:`CanaryInputSet` currently active (or None)."""
        return self._injector._canary_set  # noqa: SLF001

    def maybe_inject(self) -> CanaryPrompt | None:
        """Decide + fetch a probe in one call. Returns None when no injection."""
        self._maybe_reload()
        if not self._policy.enabled:
            return None
        if not self._injector.should_inject():
            return None
        return self._injector.next_prompt()

    def observe_result(
        self,
        prompt_id: str,
        observed_answer: str,
        *,
        worker_pubkey_hex: str | None = None,
    ) -> bool:
        self._maybe_reload()
        return self._observer.observe(
            prompt_id,
            observed_answer,
            worker_pubkey_hex=worker_pubkey_hex,
        )

    def rotate(self, new_set: CanaryInputSet) -> None:
        """Swap in a fresh :class:`CanaryInputSet` + persist to disk.

        Verification happens before any side effect so a bad
        signature cannot wipe the previous set.
        """
        verify_canary_input_set(new_set, expected_pubkey=self._coord_pubkey)
        with self._lock:
            self._injector.set_canary_set(new_set)
            self._observer.set_canary_set(new_set)
            target = self._effective_set_path()
            if target is not None:
                save_canary_input_set(new_set, target)
                try:
                    self._reload_state.set_mtime = target.stat().st_mtime
                except FileNotFoundError:
                    self._reload_state.set_mtime = None

    def update_inject_rate(self, new_rate: int) -> None:
        with self._lock:
            self._policy.inject_rate = max(1, int(new_rate))
            self._injector.set_inject_rate(self._policy.inject_rate)

    # -- internal -------------------------------------------------------

    def _effective_set_path(self) -> Path | None:
        if self._canary_set_path is not None:
            return self._canary_set_path
        if self._policy.set_path:
            return Path(self._policy.set_path).expanduser()
        return None

    def _maybe_reload(self) -> None:
        now = time.monotonic()
        if now - self._reload_state.last_reload_check < _MTIME_DEBOUNCE_SECS:
            return
        self._reload_state.last_reload_check = now
        with self._lock:
            self._reload_policy_locked()
            self._reload_set_locked()

    def _reload_policy_locked(self) -> None:
        if self._policy_path is None:
            return
        try:
            mtime = self._policy_path.stat().st_mtime
        except FileNotFoundError:
            if self._reload_state.policy_mtime is not None:
                _log.warning(
                    "canary_input_policy_deleted_keep_last",
                    path=str(self._policy_path),
                )
                self._reload_state.policy_mtime = None
            return
        if self._reload_state.policy_mtime is not None and mtime <= self._reload_state.policy_mtime:
            return
        try:
            text = self._policy_path.read_text(encoding="utf-8")
            new_policy = CanaryInputPolicy.from_toml(text)
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "canary_input_policy_malformed_keep_last",
                path=str(self._policy_path),
                error=str(exc),
            )
            return
        self._policy = new_policy
        self._reload_state.policy_mtime = mtime
        self._injector.set_inject_rate(new_policy.inject_rate)
        _log.info(
            "canary_input_policy_loaded",
            path=str(self._policy_path),
            inject_rate=new_policy.inject_rate,
            default_tolerance=new_policy.default_tolerance,
            enabled=new_policy.enabled,
        )

    def _reload_set_locked(self) -> None:
        target = self._effective_set_path()
        if target is None:
            return
        try:
            mtime = target.stat().st_mtime
        except FileNotFoundError:
            return
        if self._reload_state.set_mtime is not None and mtime <= self._reload_state.set_mtime:
            return
        try:
            new_set = load_canary_input_set(
                target,
                expected_pubkey=self._coord_pubkey,
            )
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "canary_input_set_reload_failed",
                path=str(target),
                error=str(exc),
            )
            return
        self._injector.set_canary_set(new_set)
        self._observer.set_canary_set(new_set)
        self._reload_state.set_mtime = mtime
        _log.info(
            "canary_input_set_loaded",
            path=str(target),
            prompts=len(new_set.prompts),
        )


# ---------------------------------------------------------------------------
# Seed probes (used by the `canary rotate` CLI on first rotation)
# ---------------------------------------------------------------------------


DEFAULT_SEED_PROMPTS: tuple[tuple[str, str, str], ...] = (
    ("canary.arith.01", "What is 17 plus 42? Answer with a number only.", "59"),
    ("canary.arith.02", "What is 8 times 9? Answer with a number only.", "72"),
    ("canary.fact.01", "What is the chemical symbol for gold? Answer with the symbol only.", "Au"),
    ("canary.fact.02", "How many continents are there? Answer with a number only.", "7"),
    ("canary.geo.01", "What is the capital of France? Answer with the city name only.", "Paris"),
)
"""Factory-default probes the operator is expected to replace.

These are deliberately short and unambiguous so the Levenshtein
similarity signal stays clean. Operators running in production
should curate a bespoke set — the public defaults are guessable
by an adversary who cloned the repo, and a motivated one could
cache them. The CLI ``rotate --output`` path lets operators bake
their own set instead of the seed list.
"""


__all__ = [
    "CANARY_INPUT_SET_VERSION",
    "DEFAULT_INJECT_RATE",
    "DEFAULT_ROTATION_FREQUENCY_DAYS",
    "DEFAULT_SEED_PROMPTS",
    "DEFAULT_TOLERANCE",
    "CanaryInputInjector",
    "CanaryInputManager",
    "CanaryInputObserver",
    "CanaryInputPolicy",
    "CanaryInputSet",
    "CanaryPrompt",
    "DivergenceRecord",
    "build_canary_input_set",
    "load_canary_input_set",
    "save_canary_input_set",
    "verify_canary_input_set",
]
