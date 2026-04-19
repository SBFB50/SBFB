# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coord-side PII redaction — layer 2 defense-in-depth.

Complète la couche 1 iframe (Sprint 21 phase B `d5b0035`) en
garantissant qu'un prompt user traverse le pipeline coord → worker
toujours redacted, même si la couche iframe n'a pas tourné (client
non-browser, test unit, crash JS). Le prompt redacted est signé
tel quel par le dispatcher — ni les relays ni les workers ne voient
les PII originales sur le wire.

Moteur : pipeline regex Python built-in (pur, déterministe, sans
réseau) couvrant les 7 entités critiques (`EMAIL_ADDRESS`,
`PHONE_NUMBER`, `CREDIT_CARD`, `IBAN_CODE`, `US_SSN`, `IP_ADDRESS`,
`URL`). Enrichissement optionnel via
`presidio-analyzer[gliner]>=2.2.362` +
`knowledgator/gliner-pii-edge-v1.0` chargé best-effort au premier
`redact()` — si le modèle HF cache n'est pas local et qu'on est
hors-ligne, le `PiiRedactor` degrade silencieusement vers le regex
engine seul + log structlog `pii_redactor_gliner_unavailable`. Le
coord boot toujours, même en env minimal.

Policy `~/.sbfb/pii_redaction_policy.toml` hot-reload pattern
TokenRotator S18 + pow_policy_loader S20 phase coord :
- 50 ms mtime debounce (protège contre editor multi-save bursts).
- Malformed-reload guard : TOML invalide → garde l'ancienne policy
  + warning.
- File-deletion guard : fichier disparu → garde l'ancienne policy
  (failing-closed, jamais zéro-redaction par accident).

Design doc complet :
.planning/research/S21_phase_C_output_filter_design.md
"""

from __future__ import annotations

import re
import threading
import time
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import structlog

_log = structlog.get_logger(__name__)

DEFAULT_MODEL_NAME = "knowledgator/gliner-pii-edge-v1.0"
DEFAULT_ENABLED_ENTITIES: tuple[str, ...] = (
    "EMAIL_ADDRESS",
    "PHONE_NUMBER",
    "CREDIT_CARD",
    "IBAN_CODE",
    "US_SSN",
    "IP_ADDRESS",
    "URL",
    "PERSON",
    "LOCATION",
)
DEFAULT_CONFIDENCE_THRESHOLD = 0.5
DEFAULT_REDACTION_FORMAT = "<{entity_type}_{N}>"
_MTIME_DEBOUNCE_SECS = 0.05


# Regex built-ins. Volontairement simples et lisibles — la couche
# GLiNER rattrape les cas edge (noms propres, adresses, locations
# contextuelles) quand elle est chargeable.
_REGEX_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "EMAIL_ADDRESS",
        re.compile(
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
        ),
    ),
    (
        "PHONE_NUMBER",
        # E.164 international + US formats
        re.compile(
            r"(?:\+?\d{1,3}[\s-]?)?(?:\(?\d{3}\)?[\s-]?)\d{3}[\s-]?\d{4}",
        ),
    ),
    (
        "CREDIT_CARD",
        # 13-19 chiffres avec separators optionnels (Visa/MC/Amex/Discover).
        # Luhn check appliqué dans `_extract_regex_spans` pour éliminer
        # les faux positifs sur les suites numériques arbitraires.
        re.compile(r"\b(?:\d[ -]?){13,19}\b"),
    ),
    (
        "IBAN_CODE",
        re.compile(
            r"\b[A-Z]{2}\d{2}[A-Z0-9]{4,30}\b",
        ),
    ),
    (
        "US_SSN",
        re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
    ),
    (
        "IP_ADDRESS",
        re.compile(
            r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}"
            r"(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b"
        ),
    ),
    (
        "URL",
        re.compile(r"https?://[^\s<>\"]+"),
    ),
)


def _luhn_valid(number: str) -> bool:
    """Luhn checksum validation for CREDIT_CARD false-positive filter."""
    digits = [int(c) for c in number if c.isdigit()]
    if not 13 <= len(digits) <= 19:
        return False
    checksum = 0
    for i, d in enumerate(reversed(digits)):
        if i % 2 == 1:
            d *= 2
            if d > 9:
                d -= 9
        checksum += d
    return checksum % 10 == 0


@dataclass
class RedactionPolicy:
    """In-memory shape of `pii_redaction_policy.toml` + defaults."""

    confidence_threshold: float = DEFAULT_CONFIDENCE_THRESHOLD
    enabled_entities: tuple[str, ...] = field(default_factory=lambda: tuple(DEFAULT_ENABLED_ENTITIES))
    redaction_format: str = DEFAULT_REDACTION_FORMAT

    @classmethod
    def from_toml(cls, text: str) -> "RedactionPolicy":
        data = tomllib.loads(text)
        section = data.get("default", {}) if isinstance(data, dict) else {}
        enabled = section.get("enabled_entities", DEFAULT_ENABLED_ENTITIES)
        return cls(
            confidence_threshold=float(section.get("confidence_threshold", DEFAULT_CONFIDENCE_THRESHOLD)),
            enabled_entities=tuple(enabled),
            redaction_format=str(section.get("redaction_format", DEFAULT_REDACTION_FORMAT)),
        )


@dataclass
class _Span:
    start: int
    end: int
    entity_type: str
    score: float


class PiiRedactor:
    """Redact PII entities from a prompt before wire signing.

    Usage minimal (regex engine seul, rapide, offline-friendly) ::

        redactor = PiiRedactor()
        safe = redactor.redact("email me at alice@x.com")

    Usage complet (Presidio AnalyzerEngine + GLiNER pour
    PERSON / LOCATION / enrichissement contextuel) : les imports
    Presidio et la construction AnalyzerEngine sont lazy — le
    premier `redact()` déclenche le chargement (qui peut prendre
    plusieurs secondes pour la première fois). Si le chargement
    échoue pour une raison quelconque (modèle HF cache absent,
    spaCy model absent, import error), le redactor degrade vers
    le regex engine seul et log `pii_redactor_*_unavailable`.
    """

    def __init__(
        self,
        *,
        model_name: str = DEFAULT_MODEL_NAME,
        policy_path: Path | None = None,
        enable_presidio: bool = True,
    ) -> None:
        self._model_name = model_name
        self._policy_path = policy_path
        self._enable_presidio = enable_presidio
        self._policy = RedactionPolicy()
        self._policy_mtime: float | None = None
        self._last_reload_check: float = 0.0
        self._lock = threading.Lock()
        self._analyzer: Any = None
        self._presidio_tried = False
        self._degraded = False
        if policy_path is not None:
            self._reload_policy_locked()

    def redact(self, text: str) -> str:
        """Return a redacted copy of ``text``. Empty/None → empty."""
        if not text:
            return text or ""
        self._maybe_reload_policy()
        enabled = set(self._policy.enabled_entities)
        spans: list[_Span] = []
        spans.extend(self._extract_regex_spans(text, enabled))
        if self._enable_presidio:
            self._ensure_presidio()
            if self._analyzer is not None:
                spans.extend(self._extract_presidio_spans(text, enabled))
        spans = _dedupe_spans(spans)
        if not spans:
            return text
        return self._rewrite(text, spans)

    def reload_policy(self) -> None:
        """Force reload now (skip mtime debounce)."""
        with self._lock:
            self._last_reload_check = 0.0
            # Force re-read même si mtime identique.
            self._policy_mtime = None
            self._reload_policy_locked()

    @property
    def degraded(self) -> bool:
        """True si Presidio / GLiNER non chargeable → regex-only."""
        return self._degraded

    @property
    def policy(self) -> RedactionPolicy:
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
                    "pii_redactor_policy_deleted_keep_last",
                    path=str(self._policy_path),
                )
                self._policy_mtime = None
            return
        if self._policy_mtime is not None and mtime <= self._policy_mtime:
            return
        try:
            text = self._policy_path.read_text(encoding="utf-8")
            new_policy = RedactionPolicy.from_toml(text)
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "pii_redactor_policy_malformed_keep_last",
                path=str(self._policy_path),
                error=str(exc),
            )
            return
        self._policy = new_policy
        self._policy_mtime = mtime
        _log.info(
            "pii_redactor_policy_loaded",
            path=str(self._policy_path),
            confidence_threshold=new_policy.confidence_threshold,
            entities=list(new_policy.enabled_entities),
        )

    def _ensure_presidio(self) -> None:
        if self._presidio_tried:
            return
        with self._lock:
            if self._presidio_tried:
                return
            self._presidio_tried = True
            self._analyzer = self._build_presidio()

    def _build_presidio(self) -> Any:
        try:
            from presidio_analyzer import AnalyzerEngine
        except ImportError as exc:
            _log.warning(
                "pii_redactor_presidio_missing",
                error=str(exc),
            )
            self._degraded = True
            return None
        try:
            engine = AnalyzerEngine()
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "pii_redactor_analyzer_engine_unavailable",
                error=str(exc),
            )
            self._degraded = True
            return None
        try:
            from presidio_analyzer.predefined_recognizers import (
                GLiNERRecognizer,
            )

            entity_mapping = {
                "person": "PERSON",
                "name": "PERSON",
                "organization": "LOCATION",
                "location": "LOCATION",
                "address": "LOCATION",
                "email": "EMAIL_ADDRESS",
                "phone": "PHONE_NUMBER",
                "credit_card": "CREDIT_CARD",
                "iban": "IBAN_CODE",
                "ssn": "US_SSN",
                "ip_address": "IP_ADDRESS",
                "url": "URL",
            }
            gliner = GLiNERRecognizer(
                model_name=self._model_name,
                entity_mapping=entity_mapping,
                flat_ner=False,
                multi_label=True,
                map_location="cpu",
            )
            engine.registry.add_recognizer(gliner)
            _log.info(
                "pii_redactor_gliner_loaded",
                model=self._model_name,
            )
        except Exception as exc:  # noqa: BLE001
            _log.warning(
                "pii_redactor_gliner_unavailable",
                error=str(exc),
            )
            self._degraded = True
        return engine

    def _extract_regex_spans(
        self,
        text: str,
        enabled: set[str],
    ) -> list[_Span]:
        spans: list[_Span] = []
        for entity_type, pattern in _REGEX_PATTERNS:
            if entity_type not in enabled:
                continue
            for match in pattern.finditer(text):
                raw = match.group(0)
                if entity_type == "CREDIT_CARD" and not _luhn_valid(raw):
                    continue
                spans.append(
                    _Span(
                        start=match.start(),
                        end=match.end(),
                        entity_type=entity_type,
                        score=1.0,
                    )
                )
        return spans

    def _extract_presidio_spans(
        self,
        text: str,
        enabled: set[str],
    ) -> list[_Span]:
        if self._analyzer is None:
            return []
        try:
            results = self._analyzer.analyze(
                text=text,
                language="en",
                entities=list(enabled),
                score_threshold=self._policy.confidence_threshold,
            )
        except Exception as exc:  # noqa: BLE001
            _log.warning("pii_redactor_analyze_failed", error=str(exc))
            return []
        return [
            _Span(
                start=int(r.start),
                end=int(r.end),
                entity_type=str(r.entity_type),
                score=float(r.score),
            )
            for r in results
        ]

    def _rewrite(self, text: str, spans: list[_Span]) -> str:
        # Numéroter par type en ordre d'apparition, puis remplacer
        # du end vers le start pour préserver les offsets amont.
        order = sorted(spans, key=lambda s: (s.start, s.end))
        numbering: dict[str, int] = {}
        placeholders: dict[tuple[int, int, str], str] = {}
        counts: dict[str, int] = {}
        for s in order:
            numbering[s.entity_type] = numbering.get(s.entity_type, 0) + 1
            counts[s.entity_type] = counts.get(s.entity_type, 0) + 1
            placeholders[(s.start, s.end, s.entity_type)] = self._policy.redaction_format.format(
                entity_type=s.entity_type,
                N=numbering[s.entity_type],
            )
        rewritten = text
        for s in sorted(order, key=lambda x: x.start, reverse=True):
            placeholder = placeholders[(s.start, s.end, s.entity_type)]
            rewritten = rewritten[: s.start] + placeholder + rewritten[s.end :]
        _log.info(
            "pii_redacted",
            counts=counts,
            degraded=self._degraded,
        )
        return rewritten


def _dedupe_spans(spans: list[_Span]) -> list[_Span]:
    """Fusionne les spans chevauchants en gardant le plus large
    (ou le plus confiant en cas d'egalité)."""
    if not spans:
        return []
    sorted_spans = sorted(spans, key=lambda s: (s.start, -(s.end - s.start)))
    merged: list[_Span] = []
    for s in sorted_spans:
        if not merged:
            merged.append(s)
            continue
        last = merged[-1]
        if s.start >= last.end:
            merged.append(s)
            continue
        # Overlap — keep the longer span; tie-break on score.
        len_s = s.end - s.start
        len_last = last.end - last.start
        if len_s > len_last or (len_s == len_last and s.score > last.score):
            merged[-1] = s
    return merged
