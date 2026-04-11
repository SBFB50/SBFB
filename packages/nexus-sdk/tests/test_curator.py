"""Cross-language tests for the Sprint 7 Phase B curator primitives.

These tests exercise the Python ↔ Rust boundary of the new
``sign_curator_list`` / ``verify_curator_list_entry`` PyO3 bindings.
The full Rust-side coverage lives in
``crates/nexus-core-rs/src/curator.rs``; this module focuses on the
surface Python code will actually reach into:

- a Python caller builds a ``CuratorList`` dict, asks the Rust
  crypto layer to sign it, and can verify the result on the same
  side
- a list signed on the Rust side survives a roundtrip through
  JSON serialization and still verifies in Python
- attribution split-brain, oversized entries, and version mismatch
  are rejected identically to the Rust-only tests (regression
  against a PyO3 binding that silently downgrades errors)

All tests depend on the ``nexus_core`` wheel being installed in the
active environment. The MEMORY note for this project records the
``maturin develop --release`` invocation that produces the wheel.
"""

from __future__ import annotations

import json

import nexus_core  # provided by the nexus-core-py wheel (PyO3)
import pytest


# ---------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------


def _mint_keypair() -> tuple[bytes, bytes]:
    """Return a fresh ``(secret_bytes, public_bytes)`` tuple.

    ``generate_secret`` comes from the Rust side and returns a
    dict with the 32-byte secret and 32-byte public halves; we
    unpack it to plain ``bytes`` so the rest of the tests do not
    have to keep the dict around.
    """
    pair = nexus_core.generate_secret()
    secret = bytes(pair["secret"])
    public = bytes(pair["public"])
    assert len(secret) == 32
    assert len(public) == 32
    return secret, public


def _sample_list_dict(public: bytes, revision: int = 1) -> dict:
    """Build a deterministic curator list dict referencing a
    hypothetical FlowUP curation set with two project entries."""
    return {
        "version": 1,
        "curator_pubkey": list(public),
        "curator_name": "FlowUP Curation",
        "created_at": 1_712_345_678,
        "revision": revision,
        "entries": [
            {
                "project_id": "a" * 64,
                "project_name": "gov",
                "category": "gov",
                "description": "Signal processing and intelligence tooling",
            },
            {
                "project_id": "b" * 64,
                "project_name": "coldcase",
                "category": "investigation",
                "description": "Cold case investigation toolkit",
            },
        ],
    }


# ---------------------------------------------------------------
# Happy path
# ---------------------------------------------------------------


def test_sign_then_verify_roundtrip() -> None:
    secret, public = _mint_keypair()
    list_dict = _sample_list_dict(public)
    signed_json = nexus_core.sign_curator_list(
        json.dumps(list_dict, sort_keys=True),
        secret,
    )

    # The signed blob is valid JSON that verifies under the same
    # binding.
    signed = json.loads(signed_json)
    assert signed["list"]["version"] == 1
    assert len(signed["signature"]) == 64
    assert signed["curator_pubkey"] == list(public)

    nexus_core.verify_curator_list_entry(signed_json)


def test_signed_entry_survives_json_reserialization() -> None:
    # A Python caller that re-serializes the entry (as a FastAPI
    # response or a gossip broadcast payload) must still produce
    # a blob that verifies.
    secret, public = _mint_keypair()
    signed_json = nexus_core.sign_curator_list(
        json.dumps(_sample_list_dict(public), sort_keys=True),
        secret,
    )
    parsed = json.loads(signed_json)
    reserialized = json.dumps(parsed, sort_keys=True)
    nexus_core.verify_curator_list_entry(reserialized)


# ---------------------------------------------------------------
# Negative path — the bindings must surface clear errors
# ---------------------------------------------------------------


def test_verify_rejects_tampered_entries() -> None:
    secret, public = _mint_keypair()
    signed_json = nexus_core.sign_curator_list(
        json.dumps(_sample_list_dict(public), sort_keys=True),
        secret,
    )
    tampered = json.loads(signed_json)
    tampered["list"]["entries"][0]["project_name"] = "TAMPERED"
    with pytest.raises(RuntimeError):
        nexus_core.verify_curator_list_entry(json.dumps(tampered))


def test_verify_rejects_attribution_split_brain() -> None:
    # The envelope curator_pubkey does not match list.curator_pubkey
    # anymore. This is the split-brain bug the Sprint 2 audit found
    # and fixed in ClaimEntry; the same mitigation must hold for
    # curator lists.
    secret, public = _mint_keypair()
    _, other_public = _mint_keypair()
    signed_json = nexus_core.sign_curator_list(
        json.dumps(_sample_list_dict(public), sort_keys=True),
        secret,
    )
    tampered = json.loads(signed_json)
    tampered["curator_pubkey"] = list(other_public)
    with pytest.raises(RuntimeError):
        nexus_core.verify_curator_list_entry(json.dumps(tampered))


def test_sign_rejects_mismatched_pubkey_in_payload() -> None:
    # The caller hands the binding a list whose curator_pubkey
    # does not match the signing secret — almost always a caller
    # bug. The binding must refuse before producing a signature.
    secret, _ = _mint_keypair()
    _, other_public = _mint_keypair()
    list_dict = _sample_list_dict(other_public)
    with pytest.raises(RuntimeError):
        nexus_core.sign_curator_list(json.dumps(list_dict, sort_keys=True), secret)


def test_sign_rejects_oversized_entries_dos_cap() -> None:
    # Sprint 7 plan R5 mitigation: a curator trying to ship a
    # pathologically large list must be rejected client-side so
    # the broadcast never leaves the machine.
    secret, public = _mint_keypair()
    list_dict = _sample_list_dict(public)
    list_dict["entries"] = [
        {
            "project_id": f"{i:064x}",
            "project_name": f"p{i}",
            "category": "misc",
            "description": "",
        }
        for i in range(257)  # CURATOR_LIST_MAX_ENTRIES + 1
    ]
    with pytest.raises(RuntimeError):
        nexus_core.sign_curator_list(json.dumps(list_dict, sort_keys=True), secret)


def test_verify_rejects_future_version() -> None:
    # A Phase A shell-daemon sees a hand-crafted entry with a
    # future version — the version check must fire before the
    # signature check, otherwise forward-compat assumptions break.
    secret, public = _mint_keypair()
    signed_json = nexus_core.sign_curator_list(
        json.dumps(_sample_list_dict(public), sort_keys=True),
        secret,
    )
    tampered = json.loads(signed_json)
    tampered["list"]["version"] = 99
    with pytest.raises(RuntimeError):
        nexus_core.verify_curator_list_entry(json.dumps(tampered))


def test_verify_surfaces_bad_json_as_value_error() -> None:
    # Malformed JSON is a caller bug distinct from a crypto
    # failure. The binding surfaces it as ValueError so Python
    # callers can catch the two cases separately.
    with pytest.raises(ValueError):
        nexus_core.verify_curator_list_entry("not valid json")
