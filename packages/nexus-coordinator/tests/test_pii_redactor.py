# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coord-side `PiiRedactor` tests (Sprint 21 phase coord-side).

Ces tests exercent le regex engine pur Python built-in du
`PiiRedactor` — indépendant de Presidio / GLiNER pour la
prédictibilité CI. L'enrichissement Presidio GLiNER est chargé
best-effort au runtime (cf. design doc §2.1 + pii_redactor.py
docstring) et testé séparément en e2e si le modèle HF est
disponible localement.
"""

from __future__ import annotations

import time
from pathlib import Path

from nexus_coordinator.pii_redactor import (
    DEFAULT_ENABLED_ENTITIES,
    PiiRedactor,
    RedactionPolicy,
)


def _make_regex_only_redactor(policy_path: Path | None = None) -> PiiRedactor:
    """Instancie un PiiRedactor sans chargement Presidio (regex-only).

    Évite le chargement d'AnalyzerEngine (qui peut échouer en CI
    sans spaCy model) et rend les tests déterministes. L'option
    `enable_presidio=False` est documentée dans `PiiRedactor`.
    """
    return PiiRedactor(policy_path=policy_path, enable_presidio=False)


# ---- Test 1 : redact_email_phone_name ----------------------------


def test_redact_email_phone_name() -> None:
    """Entités regex basic (email + phone) détectées + anonymized.

    Le regex engine built-in ne détecte pas "name" (PERSON) sans
    GLiNER. Le test vérifie que les entités regex-based (email,
    phone) sont bien redactées avec des placeholders typés.
    """
    redactor = _make_regex_only_redactor()
    text = "Contact alice@example.com or call +1 555-123-4567 for details."
    redacted = redactor.redact(text)
    assert "alice@example.com" not in redacted
    assert "555-123-4567" not in redacted
    assert "<EMAIL_ADDRESS_1>" in redacted
    assert "<PHONE_NUMBER_1>" in redacted


# ---- Test 2 : redact_gate2_apps_strict_mode ----------------------


def test_redact_gate2_apps_strict_mode(tmp_path: Path) -> None:
    """Policy override gate2_apps = confidence_threshold 0.3,
    tout redact.

    Le regex engine retourne toujours score=1.0 donc le seuil
    n'affecte pas les entités regex-based. Le test vérifie que le
    seuil configuré est bien lu depuis la policy (plus bas = plus
    strict pour Presidio quand chargé). On valide que la policy
    reflète le threshold demandé + toutes les entités restent
    actives quand le seuil est bas.
    """
    policy_file = tmp_path / "pii_policy.toml"
    policy_file.write_text(
        """
[default]
confidence_threshold = 0.3
enabled_entities = [
    "EMAIL_ADDRESS",
    "PHONE_NUMBER",
    "CREDIT_CARD",
    "IBAN_CODE",
    "US_SSN",
    "IP_ADDRESS",
    "URL",
]
""",
        encoding="utf-8",
    )
    redactor = _make_regex_only_redactor(policy_path=policy_file)
    # La policy est bien chargée avec le threshold strict.
    assert redactor.policy.confidence_threshold == 0.3
    # Toutes les entités regex-based sont détectées (validation que
    # aucun entity n'est désactivée par inadvertance dans strict mode).
    text = (
        "Email: bob@test.org, "
        "IBAN: DE89370400440532013000, "
        "SSN: 123-45-6789, "
        "IP: 192.168.1.1, "
        "URL: https://example.com/path"
    )
    redacted = redactor.redact(text)
    for raw in (
        "bob@test.org",
        "DE89370400440532013000",
        "123-45-6789",
        "192.168.1.1",
        "https://example.com/path",
    ):
        assert raw not in redacted, f"leak: {raw!r} in {redacted!r}"


# ---- Test 3 : policy_hot_reload ----------------------------------


def test_policy_hot_reload(tmp_path: Path) -> None:
    """Modifier policy.toml runtime → reload appliqué.

    Pattern identique au `TokenRotator` S18 + `pow_policy_loader`
    S20 phase coord : mtime debounce 50 ms, reload sur mtime
    forward, garde l'ancienne si malformed ou fichier absent.
    """
    policy_file = tmp_path / "pii_policy.toml"
    policy_file.write_text(
        """
[default]
confidence_threshold = 0.5
enabled_entities = ["EMAIL_ADDRESS"]
""",
        encoding="utf-8",
    )
    redactor = _make_regex_only_redactor(policy_path=policy_file)
    # Avant reload : seul EMAIL_ADDRESS est actif, PHONE_NUMBER ne
    # devrait pas être redacté.
    text = "email foo@bar.com and phone 555-000-1111"
    first = redactor.redact(text)
    assert "<EMAIL_ADDRESS_1>" in first
    assert "555-000-1111" in first  # pas redacté — non enabled

    # Réécriture policy avec PHONE_NUMBER ajouté. On avance l'mtime
    # de façon explicite pour contourner le debounce + la précision
    # filesystem potentiellement faible.
    time.sleep(0.1)
    policy_file.write_text(
        """
[default]
confidence_threshold = 0.5
enabled_entities = ["EMAIL_ADDRESS", "PHONE_NUMBER"]
""",
        encoding="utf-8",
    )
    # Force un re-stat (touche mtime forward).
    new_mtime = time.time() + 1.0
    import os

    os.utime(policy_file, (new_mtime, new_mtime))

    redactor.reload_policy()
    second = redactor.redact(text)
    assert "<EMAIL_ADDRESS_1>" in second
    assert "<PHONE_NUMBER_1>" in second
    assert "555-000-1111" not in second


# ---- Sanity : malformed + deleted policy guards -------------------


def test_policy_malformed_keeps_last_good(tmp_path: Path) -> None:
    policy_file = tmp_path / "pii_policy.toml"
    policy_file.write_text(
        """
[default]
confidence_threshold = 0.5
enabled_entities = ["EMAIL_ADDRESS"]
""",
        encoding="utf-8",
    )
    redactor = _make_regex_only_redactor(policy_path=policy_file)
    assert redactor.policy.confidence_threshold == 0.5

    # Écrire un TOML malformed → doit garder l'ancienne policy.
    policy_file.write_text("not [ valid toml", encoding="utf-8")
    new_mtime = time.time() + 1.0
    import os

    os.utime(policy_file, (new_mtime, new_mtime))

    redactor.reload_policy()
    assert redactor.policy.confidence_threshold == 0.5  # inchangé


def test_policy_deleted_keeps_last_good(tmp_path: Path) -> None:
    policy_file = tmp_path / "pii_policy.toml"
    policy_file.write_text(
        """
[default]
enabled_entities = ["EMAIL_ADDRESS"]
""",
        encoding="utf-8",
    )
    redactor = _make_regex_only_redactor(policy_path=policy_file)
    policy_file.unlink()
    # Force reload — doit garder last good (fail-closed).
    redactor.reload_policy()
    assert redactor.policy.enabled_entities == ("EMAIL_ADDRESS",)


# ---- Sanity : credit_card Luhn filter false-positives -----------


def test_credit_card_luhn_rejects_false_positive(tmp_path: Path) -> None:
    """Suite de 13-19 chiffres qui n'est PAS un numéro CC valide Luhn
    ne doit pas être redactée.

    On désactive PHONE_NUMBER dans la policy de ce test pour isoler
    le comportement du filtre Luhn sur CREDIT_CARD — la regex phone
    capture des sous-séquences qui se chevauchent avec les 16
    chiffres (16 ≥ 10) et polluerait le signal.
    """
    policy_file = tmp_path / "pii_policy.toml"
    policy_file.write_text(
        """
[default]
enabled_entities = ["CREDIT_CARD"]
""",
        encoding="utf-8",
    )
    redactor = _make_regex_only_redactor(policy_path=policy_file)
    # 16 chiffres qui ne passent pas Luhn checksum.
    text = "Order reference 1111111111111111 for shipping"
    redacted = redactor.redact(text)
    # 1111111111111111 ne passe pas Luhn → pas redacté.
    assert "1111111111111111" in redacted
    # 4111111111111111 est un test Visa valide Luhn → redacté.
    text2 = "Card 4111111111111111 expires 12/26"
    redacted2 = redactor.redact(text2)
    assert "4111111111111111" not in redacted2
    assert "<CREDIT_CARD_1>" in redacted2


# ---- Sanity : default policy = full defaults -------------------


def test_default_policy_matches_contract() -> None:
    policy = RedactionPolicy()
    assert policy.confidence_threshold == 0.5
    assert tuple(policy.enabled_entities) == tuple(DEFAULT_ENABLED_ENTITIES)
    assert "EMAIL_ADDRESS" in policy.enabled_entities
    assert "PERSON" in policy.enabled_entities
