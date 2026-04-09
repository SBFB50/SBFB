"""
NEXUS GOV -- Pydantic v2 models for French Government Monitoring tables.

For each table:
- *Base:   shared fields (used in Create + full response)
- *Create: input schema for POST endpoints
- *Update: partial update schema (all fields Optional)
- <Name>:  full response schema with id + timestamps
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Literal, Optional

from pydantic import BaseModel


# ============================================================================
# Literal types
# ============================================================================

Chamber = Literal["assemblee", "senat", "gouvernement"]
PoliticianRole = Literal["depute", "senateur", "ministre", "premier_ministre", "president"]
PositionType = Literal["vote", "declaration", "amendment", "question", "patrimoine", "lobby"]
Stance = Literal["pour", "contre", "abstention"]
SourceType = Literal["assemblee_nationale", "senat", "hatvp", "journal_officiel", "media"]
ContradictionSeverity = Literal["low", "medium", "high"]
ScanStatus = Literal["running", "completed", "error"]


# ============================================================================
# Politicians
# ============================================================================

class PoliticianBase(BaseModel):
    name: str
    chamber: Chamber
    party: Optional[str] = None
    role: Optional[PoliticianRole] = None
    constituency: Optional[str] = None
    photo_url: Optional[str] = None
    official_url: Optional[str] = None
    hatvp_url: Optional[str] = None
    active: bool = True


class PoliticianCreate(PoliticianBase):
    pass


class PoliticianUpdate(BaseModel):
    name: Optional[str] = None
    chamber: Optional[Chamber] = None
    party: Optional[str] = None
    role: Optional[PoliticianRole] = None
    constituency: Optional[str] = None
    photo_url: Optional[str] = None
    official_url: Optional[str] = None
    hatvp_url: Optional[str] = None
    active: Optional[bool] = None


class Politician(PoliticianBase):
    id: str
    created_at: datetime
    updated_at: Optional[datetime] = None

    model_config = {"from_attributes": True}


# ============================================================================
# Positions (votes, declarations, amendments, etc.)
# ============================================================================

class PositionBase(BaseModel):
    politician_id: str
    subject: str
    position_type: PositionType
    position_text: str
    stance: Optional[Stance] = None
    source_url: str
    source_type: Optional[SourceType] = None
    date: Optional[str] = None
    session: Optional[str] = None
    metadata: Optional[Any] = None


class PositionCreate(PositionBase):
    pass


class Position(PositionBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Contradictions
# ============================================================================

class ContradictionBase(BaseModel):
    politician_id: str
    position_a_id: str
    position_b_id: str
    subject: str
    description: str
    severity: ContradictionSeverity = "medium"
    source_verified: bool = False


class ContradictionCreate(ContradictionBase):
    pass


class Contradiction(ContradictionBase):
    id: str
    detected_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Scan Log
# ============================================================================

class ScanLog(BaseModel):
    id: str
    scan_type: str
    status: ScanStatus = "running"
    items_found: int = 0
    items_new: int = 0
    error_message: Optional[str] = None
    started_at: datetime
    completed_at: Optional[datetime] = None
    current_phase: str = ""
    phase_offset: int = 0
    checkpoint_data: Any = None

    model_config = {"from_attributes": True}


# ============================================================================
# Stats (virtual — not stored, computed on the fly)
# ============================================================================

class GovernmentStats(BaseModel):
    politicians: int
    positions: int
    contradictions: int
    mandates: int = 0
    parties: int = 0
    party_memberships: int = 0
    affairs: int = 0
    declarations: int = 0
    laws: int = 0
    press: int = 0
    social_posts: int = 0
    transcriptions: int = 0
    factchecks: int = 0
    external_ids: int = 0
    scan_logs: int = 0
    alerts: int = 0
    last_scan: Optional[str] = None
