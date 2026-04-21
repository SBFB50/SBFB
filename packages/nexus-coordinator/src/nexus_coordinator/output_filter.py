# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coord-side output filter — invisible text scan + prompt echo detection.

Applied sur le ``payload.content`` du ``ResultEntry`` d'un worker,
APRÈS ``Verifier.verify_entries`` (3-layer signature + model digest
+ logprob fingerprint) et AVANT ``Dispatcher.mark_completed`` +
``KudosLedger.credit``. Un verdict négatif convertit le
``ValidationEvent`` en ``result_rejected`` sans crediter kudos
(pattern identique à un 3-layer verify fail).

Deux classes d'attaques couvertes :

1. **Invisible text steganography** — zero-width U+200B, PUA
   U+E000-U+F8FF, Tag chars U+E0020-U+E007F cachés dans la
   réponse. Scanner ré-implémenté localement depuis llm-guard
   0.3.16 `InvisibleText` (cf. design doc §2.2 + pivot log
   2026-04-19 : drop llm-guard pour transitive-pin
   presidio-analyzer incompatible). Whitelist Cf : RLO/LRO/
   PDF/LRE/RLE + directional isolates (U+2066-U+2069) conservés
   pour i18n légitime (Arabe, Hébreu, texte bidi).

2. **Prompt leak** (PLeak CCS'24 arXiv 2405.06823) — la réponse
   echo tout ou partie du ``system_prompt``. Trois niveaux
   cumulés (le premier qui déclenche bloque) :
   - **Exact Match** : ``system_prompt in model_output``.
   - **Substring Match** : présence d'une tranche >= 40 chars
     du system_prompt dans l'output.
   - **EED (Extended Edit Distance)** :
     ``rapidfuzz.distance.Levenshtein.normalized_similarity``
     seuil **0.85** configurable. Seuil empirique tuné sur le
     corpus PLeak CCS'24 — gardé configurable par construction
     parce qu'il n'y a pas de one-size-fits-all sans tuning par
     déploiement.

Policy `~/.sbfb/output_filter_policy.toml` hot-reload pattern
identique à `PiiRedactor` (mtime 50 ms debounce, malformed-reload
guard, file-deletion guard — failing-closed).

Design doc complet :
.planning/research/S21_phase_C_output_filter_design.md
"""

from __future__ import annotations

import threading
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path

import structlog
from rapidfuzz.distance import Levenshtein

_log = structlog.get_logger(__name__)

DEFAULT_EED_THRESHOLD = 0.85
DEFAULT_SUBSTRING_MIN_LEN = 40
_MTIME_DEBOUNCE_SECS = 0.05


# ------ InvisibleText ranges ------
#
# Reproduction fidèle de llm-guard 0.3.16 `InvisibleText` scanner.
# Source : docs.protectai.github.io/llm-guard/input_scanners/
# invisible_text/ + code source llm_guard/input_scanners/
# invisible_text.py.
#
# Blocs strippés :
# - Zero-width / joiner / directional : U+200B-U+200F, U+2060,
#   U+FEFF (BOM). Exception : les directional format chars
#   utilisés pour i18n bidi (U+202A-U+202E, U+2066-U+2069) sont
#   whitelist — langue Arabe / Hébreu les utilise.
# - Private Use Area : U+E000-U+F8FF (BMP) + U+F0000-U+FFFFD
#   (Plane 15) + U+100000-U+10FFFD (Plane 16).
# - Tag chars : U+E0020-U+E007F (ASCII tag block, steganography
#   vector rez0 CCS'24).

_STRIPPED_ZW = frozenset(
    chr(cp)
    for cp in (
        0x200B,
        0x200C,
        0x200D,
        0x200E,
        0x200F,
        0x2060,
        0xFEFF,
    )
)
# Whitelisted Cf (Format) chars — i18n légitime, ne PAS stripper.
_WHITELIST_CF = frozenset(
    chr(cp)
    for cp in (
        0x202A,  # LRE
        0x202B,  # RLE
        0x202C,  # PDF
        0x202D,  # LRO
        0x202E,  # RLO
        0x2066,  # LRI
        0x2067,  # RLI
        0x2068,  # FSI
        0x2069,  # PDI
    )
)


def _is_pua(char: str) -> bool:
    cp = ord(char)
    return 0xE000 <= cp <= 0xF8FF or 0xF0000 <= cp <= 0xFFFFD or 0x100000 <= cp <= 0x10FFFD


def _is_tag_char(char: str) -> bool:
    cp = ord(char)
    return 0xE0020 <= cp <= 0xE007F


def scan_invisible_text(text: str) -> tuple[str, bool, float]:
    """Strip invisible chars from ``text``. Return (sanitized,
    is_valid, risk_score) comme llm-guard 0.3.16 `InvisibleText`.

    - ``is_valid = True`` si aucun char strippé.
    - ``risk_score = 1.0`` si au moins un char strippé, sinon
      ``0.0``.

    Whitelist Cf (bidi i18n) préservée.
    """
    if not text:
        return text or "", True, 0.0
    out_chars: list[str] = []
    stripped_any = False
    for ch in text:
        if ch in _WHITELIST_CF:
            out_chars.append(ch)
            continue
        if ch in _STRIPPED_ZW or _is_pua(ch) or _is_tag_char(ch):
            stripped_any = True
            continue
        out_chars.append(ch)
    sanitized = "".join(out_chars)
    risk_score = 1.0 if stripped_any else 0.0
    return sanitized, not stripped_any, risk_score


@dataclass
class FilterVerdict:
    """Result of :meth:`OutputFilter.filter`."""

    is_valid: bool
    reason: str  # "invisible_text" | "prompt_echo_exact"
    #             | "prompt_echo_substring" | "prompt_echo_eed"
    #             | "ok"
    risk_score: float
    sanitized_output: str


@dataclass
class OutputFilterPolicy:
    """In-memory shape of `output_filter_policy.toml` + defaults."""

    enabled: bool = True
    strip_zero_width: bool = True
    strip_pua: bool = True
    strip_tag_chars: bool = True
    whitelist_cf: bool = True
    exact_match: bool = True
    substring_match_min_len: int = DEFAULT_SUBSTRING_MIN_LEN
    eed_threshold: float = DEFAULT_EED_THRESHOLD

    @classmethod
    def from_toml(cls, text: str) -> "OutputFilterPolicy":
        data = tomllib.loads(text) if text else {}
        default = data.get("default", {}) if isinstance(data, dict) else {}
        invisible = data.get("invisible_text", {}) if isinstance(data, dict) else {}
        prompt_echo = data.get("prompt_echo", {}) if isinstance(data, dict) else {}
        return cls(
            enabled=bool(default.get("enabled", True)),
            strip_zero_width=bool(invisible.get("strip_zero_width", True)),
            strip_pua=bool(invisible.get("strip_pua", True)),
            strip_tag_chars=bool(invisible.get("strip_tag_chars", True)),
            whitelist_cf=bool(invisible.get("whitelist_cf", True)),
            exact_match=bool(prompt_echo.get("exact_match", True)),
            substring_match_min_len=int(prompt_echo.get("substring_match_min_len", DEFAULT_SUBSTRING_MIN_LEN)),
            eed_threshold=float(prompt_echo.get("eed_threshold", DEFAULT_EED_THRESHOLD)),
        )


class OutputFilter:
    """Scan a worker ``payload.content`` before coord delivery.

    Usage ::

        filter = OutputFilter(policy_path=Path("~/.sbfb/output_filter_policy.toml"))
        verdict = filter.filter(system_prompt, user_prompt, model_output)
        if not verdict.is_valid:
            # reject result, mark_failed
            ...
    """

    def __init__(
        self,
        *,
        policy_path: Path | None = None,
    ) -> None:
        self._policy_path = policy_path
        self._policy = OutputFilterPolicy()
        self._policy_mtime: float | None = None
        self._last_reload_check: float = 0.0
        self._lock = threading.Lock()
        if policy_path is not None:
            self._reload_policy_locked()

    def filter(
        self,
        system_prompt: str,
        user_prompt: str,
        model_output: str,
    ) -> FilterVerdict:
        """Return verdict with optionally sanitized output.

        Ordering : invisible text strip first (always sanitizes,
        fails if anything was strippé — keeps the sanitized
        output so callers can log/audit but still flag the
        result as invalid), then prompt-echo detection cascade
        (exact → substring → EED).
        """
        self._maybe_reload_policy()
        if not self._policy.enabled:
            return FilterVerdict(
                is_valid=True,
                reason="ok",
                risk_score=0.0,
                sanitized_output=model_output or "",
            )

        sanitized, is_clean, risk_score = scan_invisible_text(model_output or "")
        if not is_clean:
            return FilterVerdict(
                is_valid=False,
                reason="invisible_text",
                risk_score=risk_score,
                sanitized_output=sanitized,
            )

        # Prompt-echo cascade — only against system_prompt (user
        # prompt echo isn't a leak, it's a legitimate repeat).
        sp = (system_prompt or "").strip()
        if sp:
            # Exact match
            if self._policy.exact_match and sp in sanitized:
                return FilterVerdict(
                    is_valid=False,
                    reason="prompt_echo_exact",
                    risk_score=1.0,
                    sanitized_output=sanitized,
                )
            # Substring match (any slice >= min_len)
            min_len = self._policy.substring_match_min_len
            if min_len > 0 and len(sp) >= min_len:
                for start in range(0, len(sp) - min_len + 1):
                    slice_ = sp[start : start + min_len]
                    if slice_ in sanitized:
                        return FilterVerdict(
                            is_valid=False,
                            reason="prompt_echo_substring",
                            risk_score=0.95,
                            sanitized_output=sanitized,
                        )
            # EED (normalized Levenshtein similarity)
            similarity = Levenshtein.normalized_similarity(sp, sanitized)
            if similarity >= self._policy.eed_threshold:
                return FilterVerdict(
                    is_valid=False,
                    reason="prompt_echo_eed",
                    risk_score=float(similarity),
                    sanitized_output=sanitized,
                )

        return FilterVerdict(
            is_valid=True,
            reason="ok",
            risk_score=risk_score,
            sanitized_output=sanitized,
        )

    def reload_policy(self) -> None:
        """Force reload now (skip mtime debounce)."""
        with self._lock:
            self._last_reload_check = 0.0
            self._policy_mtime = None
            self._reload_policy_locked()

    @property
    def policy(self) -> OutputFilterPolicy:
        return self._policy

    # ---- internal helpers ----

    def _maybe_reload_policy(self) -> None:
        if self._policy_path is None:
            return
        now = time.monotonic()
        if now - self._last_reload_check < _MTIME_DEBOUNCE_SECS:
            return
        self._last_reload_check = now
        with self._lock:
            self._reload_policy_locked()

    def _reload_policy_locked(self) -> None:
        if self._policy_path is None:
            return
        try:
            mtime = self._policy_path.stat().st_mtime
        except FileNotFoundError:
            if self._policy_mtime is not None:
                _log.warning(
                    "output_filter_policy_deleted_keep_last",
                    path=str(self._policy_path),
                )
                self._policy_mtime = None
            return
        if self._policy_mtime is not None and mtime <= self._policy_mtime:
            return
        try:
            text = self._policy_path.read_text(encoding="utf-8")
            new_policy = OutputFilterPolicy.from_toml(text)
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "output_filter_policy_malformed_keep_last",
                path=str(self._policy_path),
                error=str(exc),
            )
            return
        self._policy = new_policy
        self._policy_mtime = mtime
        _log.info(
            "output_filter_policy_loaded",
            path=str(self._policy_path),
            eed_threshold=new_policy.eed_threshold,
            substring_match_min_len=new_policy.substring_match_min_len,
            enabled=new_policy.enabled,
        )


class OutputSafetyGuardrail:
    """Guardrail adapter wrapping :class:`OutputFilter`.

    Tripwires when the filter verdict is invalid (invisible text or
    prompt echo detected). Implements the ``Guardrail`` ABC from
    ``nexus_coordinator.guardrails``.
    """

    def __init__(self, output_filter: OutputFilter) -> None:
        self._filter = output_filter

    @property
    def name(self) -> str:
        return "output_safety"

    @property
    def direction(self) -> str:
        return "output"

    async def check(self, ctx: object, value: str) -> object:
        from nexus_coordinator.guardrails import GuardrailOutcome

        verdict = self._filter.filter(
            getattr(ctx, "system_prompt", ""),
            getattr(ctx, "user_prompt", ""),
            value,
        )
        return GuardrailOutcome(
            passed=verdict.is_valid,
            tripwire=not verdict.is_valid,
            guardrail_name=self.name,
            evidence={"reason": verdict.reason, "risk_score": verdict.risk_score},
        )

    async def on_tripwire(self, ctx: object, outcome: object) -> None:
        pass


def _register_output_guardrail() -> None:
    from nexus_coordinator.guardrails import Guardrail

    Guardrail.register(OutputSafetyGuardrail)


_register_output_guardrail()


__all__ = [
    "FilterVerdict",
    "OutputFilter",
    "OutputFilterPolicy",
    "OutputSafetyGuardrail",
    "scan_invisible_text",
]
