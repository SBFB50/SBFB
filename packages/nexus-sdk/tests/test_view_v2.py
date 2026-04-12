"""Unit tests for the Sprint 9 Phase E TabView v2 schema additions.

Covers: TabViewV2, TabBlockFileUpload, AnyTabView discriminated union,
file_upload_block() constructor helper, and the cross-language canonical
fixture shared with the Vitest/Zod parser on the frontend.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from nexus_sdk.view import (
    AnyTabView,
    TabBlockFileUpload,
    TabView,
    TabViewV1,
    TabViewV2,
    file_upload_block,
)
from pydantic import TypeAdapter, ValidationError

SNAPSHOT_V2_PATH = Path(__file__).parent / "snapshots" / "tabview_v2_canonical.json"

# TypeAdapter lets us validate against the Annotated union directly.
_AnyTabViewAdapter: TypeAdapter[AnyTabView] = TypeAdapter(AnyTabView)


# ---------------------------------------------------------------------------
# 1. test_v1_descriptor_validates_under_v1_schema
# ---------------------------------------------------------------------------


def test_v1_descriptor_validates_under_v1_schema() -> None:
    """Existing v1 TabView descriptors remain valid under the v1 parser."""
    tv = TabView(
        schema_version=1,
        tab_name="legacy",
        title="Legacy tab",
        blocks=[],
    )
    assert tv.schema_version == 1
    assert isinstance(tv, TabViewV1)


# ---------------------------------------------------------------------------
# 2. test_v1_descriptor_validates_under_anytabview
# ---------------------------------------------------------------------------


def test_v1_descriptor_validates_under_anytabview() -> None:
    """A schema_version=1 payload is accepted by the AnyTabView union."""
    payload = {"schema_version": 1, "tab_name": "legacy_any"}
    result = _AnyTabViewAdapter.validate_python(payload)
    assert isinstance(result, TabViewV1)
    assert result.schema_version == 1


# ---------------------------------------------------------------------------
# 3. test_v2_descriptor_with_file_upload_block_parses
# ---------------------------------------------------------------------------


def test_v2_descriptor_with_file_upload_block_parses() -> None:
    """A schema_version=2 payload containing a file_upload block parses cleanly."""
    payload = {
        "schema_version": 2,
        "tab_name": "docs",
        "blocks": [
            {
                "kind": "file_upload",
                "label": "Upload",
                "accept": ["image/*"],
                "max_size_bytes": 1024,
            }
        ],
    }
    result = TabViewV2.model_validate(payload)
    assert result.schema_version == 2
    assert len(result.blocks) == 1
    block = result.blocks[0]
    assert isinstance(block, TabBlockFileUpload)
    assert block.label == "Upload"
    assert block.accept == ["image/*"]
    assert block.max_size_bytes == 1024


# ---------------------------------------------------------------------------
# 4. test_v2_descriptor_parsed_as_v2_instance
# ---------------------------------------------------------------------------


def test_v2_descriptor_parsed_as_v2_instance() -> None:
    """model_validate on a v2 payload returns a TabViewV2 instance."""
    result = TabViewV2.model_validate({"schema_version": 2, "tab_name": "check_type"})
    assert isinstance(result, TabViewV2)


# ---------------------------------------------------------------------------
# 5. test_v2_file_upload_block_rejected_under_v1
# ---------------------------------------------------------------------------


def test_v2_file_upload_block_rejected_under_v1() -> None:
    """A file_upload block inside a schema_version=1 descriptor raises ValidationError.

    TabBlockV1 does not include file_upload in its discriminated union, so
    the discriminator will find no matching arm and Pydantic raises.
    """
    payload = {
        "schema_version": 1,
        "tab_name": "bad",
        "blocks": [
            {
                "kind": "file_upload",
                "label": "Denied",
                "accept": ["*/*"],
                "max_size_bytes": 1024,
            }
        ],
    }
    with pytest.raises(ValidationError):
        TabView.model_validate(payload)


# ---------------------------------------------------------------------------
# 6. test_v2_extra_forbid_preserved_on_all_blocks
# ---------------------------------------------------------------------------


def test_v2_extra_forbid_preserved_on_all_blocks() -> None:
    """extra="forbid" is active on TabBlockFileUpload — unknown fields raise."""
    with pytest.raises(ValidationError):
        TabBlockFileUpload(
            label="x",
            accept=["*/*"],
            max_size_bytes=1024,
            unexpected_key="nope",  # type: ignore[call-arg]
        )


# ---------------------------------------------------------------------------
# 7. test_v2_extra_field_raises_validation_error
# ---------------------------------------------------------------------------


def test_v2_extra_field_raises_validation_error() -> None:
    """extra="forbid" is active on TabViewV2 itself — unknown top-level fields raise."""
    with pytest.raises(ValidationError):
        TabViewV2(
            schema_version=2,
            tab_name="x",
            unknown_top_level="bad",  # type: ignore[call-arg]
        )


# ---------------------------------------------------------------------------
# 8. test_cross_lang_fixture_v2_roundtrip_python_side
# ---------------------------------------------------------------------------


def test_cross_lang_fixture_v2_roundtrip_python_side() -> None:
    """Python-side half of the cross-language v2 round-trip guard.

    Reads ``tabview_v2_canonical.json``, parses it via AnyTabView
    (discriminated on schema_version), dumps back to dict, and asserts
    equality with the original payload.

    The Zod/TypeScript side of this guard lives in the Vitest suite that
    imports the same JSON file and calls the v2 schema parser.
    """
    assert SNAPSHOT_V2_PATH.exists(), (
        f"v2 canonical fixture missing at {SNAPSHOT_V2_PATH} — "
        "check that the file was committed alongside test_view_v2.py"
    )

    raw = SNAPSHOT_V2_PATH.read_text(encoding="utf-8")
    payload = json.loads(raw)

    result = _AnyTabViewAdapter.validate_python(payload)
    assert isinstance(result, TabViewV2), (
        f"Expected TabViewV2 but got {type(result).__name__}. schema_version in fixture must be 2."
    )

    dumped = result.model_dump()
    assert dumped == payload, (
        "v2 canonical fixture failed Pydantic round-trip. "
        "A change in view.py (field rename, default, new kind) drifted "
        f"from the committed fixture at {SNAPSHOT_V2_PATH}. "
        "If intentional, regenerate the fixture AND the Vitest snapshot "
        "so both languages stay aligned."
    )


# ---------------------------------------------------------------------------
# 9. test_v2_file_upload_block_constructor_helper
# ---------------------------------------------------------------------------


def test_v2_file_upload_block_constructor_helper() -> None:
    """file_upload_block() returns a validated TabBlockFileUpload with correct fields."""
    block = file_upload_block(
        label="Déposer un fichier",
        accept=["application/pdf"],
        max_size_bytes=10 * 1024 * 1024,
    )
    assert isinstance(block, TabBlockFileUpload)
    assert block.kind == "file_upload"
    assert block.label == "Déposer un fichier"
    assert block.accept == ["application/pdf"]
    assert block.max_size_bytes == 10 * 1024 * 1024


# ---------------------------------------------------------------------------
# 10. test_v2_file_upload_block_accept_validation
# ---------------------------------------------------------------------------


def test_v2_file_upload_block_accept_validation() -> None:
    """file_upload_block() with accept=['image/*'] stores the value correctly."""
    block = file_upload_block(label="Images", accept=["image/*"])
    assert block.accept == ["image/*"]


# ---------------------------------------------------------------------------
# 11. test_v2_file_upload_block_max_size_bytes_validation
# ---------------------------------------------------------------------------


def test_v2_file_upload_block_max_size_bytes_validation() -> None:
    """file_upload_block() stores max_size_bytes as provided; default is 50 MiB."""
    default_block = file_upload_block(label="default")
    assert default_block.max_size_bytes == 50 * 1024 * 1024

    custom_block = file_upload_block(label="small", max_size_bytes=4096)
    assert custom_block.max_size_bytes == 4096


# ---------------------------------------------------------------------------
# 12. test_schema_version_literal_enforced
# ---------------------------------------------------------------------------


def test_schema_version_literal_enforced() -> None:
    """schema_version=3 (unknown) raises ValidationError on both v1 and v2 parsers.

    Validates that neither TabView (Literal[1]) nor TabViewV2 (Literal[2])
    accepts an arbitrary integer, and that AnyTabView rejects it too since
    no discriminator arm matches.
    """
    with pytest.raises(ValidationError):
        TabView(schema_version=3, tab_name="x")  # type: ignore[arg-type]

    with pytest.raises(ValidationError):
        TabViewV2(schema_version=3, tab_name="x")  # type: ignore[arg-type]

    with pytest.raises(ValidationError):
        _AnyTabViewAdapter.validate_python({"schema_version": 3, "tab_name": "x"})
