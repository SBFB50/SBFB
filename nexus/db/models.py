"""
NEXUS -- Pydantic v2 models for all database tables.

For each table:
- *Base:   shared fields (used in Create + full response)
- *Create: input schema for POST endpoints
- *Update: partial update schema (all fields Optional)
- <Name>:  full response schema with id + timestamps
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, List, Literal, Optional

from pydantic import BaseModel, Field


# ============================================================================
# Cases
# ============================================================================

CaseStatus = Literal["active", "closed", "archived"]


class CaseBase(BaseModel):
    name: str
    reference: Optional[str] = None
    description: Optional[str] = None
    status: CaseStatus = "active"


class CaseCreate(CaseBase):
    pass


class CaseUpdate(BaseModel):
    name: Optional[str] = None
    reference: Optional[str] = None
    description: Optional[str] = None
    status: Optional[CaseStatus] = None


class Case(CaseBase):
    id: str
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Evidence
# ============================================================================

EvidenceType = Literal["pdf", "image", "text", "audio", "url", "manual"]
EvidenceStatus = Literal["pending", "processed", "error"]


class EvidenceBase(BaseModel):
    case_id: str
    title: str
    evidence_type: EvidenceType
    source: Optional[str] = None
    source_date: Optional[datetime] = None
    reliability: int = Field(default=50, ge=0, le=100)
    file_path: Optional[str] = None
    raw_text: Optional[str] = None
    summary: Optional[str] = None
    metadata: Optional[Any] = None
    status: EvidenceStatus = "pending"


class EvidenceCreate(EvidenceBase):
    pass


class EvidenceUpdate(BaseModel):
    title: Optional[str] = None
    evidence_type: Optional[EvidenceType] = None
    source: Optional[str] = None
    source_date: Optional[datetime] = None
    reliability: Optional[int] = Field(default=None, ge=0, le=100)
    file_path: Optional[str] = None
    raw_text: Optional[str] = None
    summary: Optional[str] = None
    metadata: Optional[Any] = None
    status: Optional[EvidenceStatus] = None


class Evidence(EvidenceBase):
    id: str
    ingestion_date: datetime
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Entities
# ============================================================================

EntityType = Literal[
    "person",
    "location",
    "phone",
    "vehicle",
    "organization",
    "date",
    "money",
    "ip",
    "email",
    "account",
    "weapon",
    "drug",
    "other",
]


class EntityBase(BaseModel):
    case_id: str
    name: str
    entity_type: EntityType
    aliases: Optional[List[str]] = None
    description: Optional[str] = None
    first_seen: Optional[datetime] = None
    metadata: Optional[Any] = None


class EntityCreate(EntityBase):
    pass


class EntityUpdate(BaseModel):
    name: Optional[str] = None
    entity_type: Optional[EntityType] = None
    aliases: Optional[List[str]] = None
    description: Optional[str] = None
    first_seen: Optional[datetime] = None
    metadata: Optional[Any] = None


class Entity(EntityBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Entity Mentions
# ============================================================================


class EntityMentionBase(BaseModel):
    entity_id: str
    evidence_id: str
    context: Optional[str] = None
    confidence: float = Field(default=0.8, ge=0.0, le=1.0)


class EntityMentionCreate(EntityMentionBase):
    pass


class EntityMention(EntityMentionBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Hypotheses
# ============================================================================

HypothesisStatus = Literal["active", "refuted", "confirmed", "merged"]


class HypothesisBase(BaseModel):
    case_id: str
    title: str
    description: str
    status: HypothesisStatus = "active"
    current_score: float = Field(default=50.0, ge=0.0, le=100.0)


class HypothesisCreate(BaseModel):
    """Input for creating a hypothesis. case_id comes from the URL path."""
    title: str
    description: str
    status: HypothesisStatus = "active"
    current_score: float = Field(default=50.0, ge=0.0, le=100.0)


class HypothesisUpdate(BaseModel):
    title: Optional[str] = None
    description: Optional[str] = None
    status: Optional[HypothesisStatus] = None
    current_score: Optional[float] = Field(default=None, ge=0.0, le=100.0)


class Hypothesis(HypothesisBase):
    id: str
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Hypothesis Snapshots
# ============================================================================


class HypothesisSnapshotBase(BaseModel):
    hypothesis_id: str
    score: float = Field(ge=0.0, le=100.0)
    supporting: Optional[Any] = None
    contradicting: Optional[Any] = None
    reasoning: Optional[str] = None
    trigger: Optional[str] = None
    model_used: Optional[str] = None


class HypothesisSnapshotCreate(HypothesisSnapshotBase):
    pass


class HypothesisSnapshot(HypothesisSnapshotBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Analysis Runs
# ============================================================================

AnalysisRunType = Literal["full", "incremental", "verification", "extraction", "self_questioning"]
AnalysisTrigger = Literal["manual", "new_evidence", "monitoring", "scheduled", "autonomous_loop"]
AnalysisStatus = Literal["running", "completed", "failed"]


class AnalysisRunBase(BaseModel):
    case_id: str
    run_type: AnalysisRunType
    trigger: Optional[AnalysisTrigger] = None
    status: AnalysisStatus = "running"
    model_used: Optional[str] = None
    input_summary: Optional[str] = None
    output_summary: Optional[str] = None
    duration_sec: Optional[float] = None
    tokens_used: Optional[int] = None


class AnalysisRunCreate(BaseModel):
    """Only the fields needed to kick off a run."""
    case_id: str
    run_type: AnalysisRunType
    trigger: Optional[AnalysisTrigger] = None
    model_used: Optional[str] = None
    input_summary: Optional[str] = None


class AnalysisRunUpdate(BaseModel):
    status: Optional[AnalysisStatus] = None
    output_summary: Optional[str] = None
    duration_sec: Optional[float] = None
    tokens_used: Optional[int] = None
    completed_at: Optional[datetime] = None


class AnalysisRun(AnalysisRunBase):
    id: str
    started_at: datetime
    completed_at: Optional[datetime] = None

    model_config = {"from_attributes": True}


# ============================================================================
# Monitoring Jobs
# ============================================================================

MonitoringJobType = Literal["searxng", "robin", "both"]


class MonitoringJobBase(BaseModel):
    case_id: str
    job_type: MonitoringJobType
    query: str
    entity_id: Optional[str] = None
    interval_hours: int = 24
    is_active: bool = True
    last_run: Optional[datetime] = None
    next_run: Optional[datetime] = None
    results_count: int = 0


class MonitoringJobCreate(BaseModel):
    case_id: str
    job_type: MonitoringJobType
    query: str
    entity_id: Optional[str] = None
    interval_hours: int = 24


class MonitoringJobUpdate(BaseModel):
    job_type: Optional[MonitoringJobType] = None
    query: Optional[str] = None
    entity_id: Optional[str] = None
    interval_hours: Optional[int] = None
    is_active: Optional[bool] = None
    last_run: Optional[datetime] = None
    next_run: Optional[datetime] = None
    results_count: Optional[int] = None


class MonitoringJob(MonitoringJobBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Monitoring Results
# ============================================================================


class MonitoringResultBase(BaseModel):
    job_id: str
    case_id: str
    url: Optional[str] = None
    title: Optional[str] = None
    snippet: Optional[str] = None
    source_engine: Optional[str] = None
    relevance_score: Optional[float] = Field(default=None, ge=0.0, le=100.0)
    is_new: bool = True
    is_duplicate: bool = False
    reviewed: bool = False


class MonitoringResultCreate(MonitoringResultBase):
    pass


class MonitoringResult(MonitoringResultBase):
    id: str
    found_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Alerts
# ============================================================================

AlertType = Literal[
    "new_evidence",
    "score_shift",
    "monitoring_hit",
    "contradiction",
    "new_entity",
]
AlertSeverity = Literal["info", "warning", "critical"]
ReportType = Literal["full", "summary", "timeline"]
ReportStatus = Literal["generating", "completed", "error"]


class AlertBase(BaseModel):
    case_id: str
    alert_type: AlertType
    severity: AlertSeverity = "info"
    title: str
    message: str
    related_id: Optional[str] = None
    is_read: bool = False


class AlertCreate(BaseModel):
    case_id: str
    alert_type: AlertType
    severity: AlertSeverity = "info"
    title: str
    message: str
    related_id: Optional[str] = None


class Alert(AlertBase):
    id: str
    created_at: datetime

    model_config = {"from_attributes": True}


# ============================================================================
# Reports
# ============================================================================


class ReportBase(BaseModel):
    case_id: str
    report_type: ReportType
    status: ReportStatus = "generating"
    file_path: Optional[str] = None
    file_size: Optional[int] = None


class ReportCreate(BaseModel):
    case_id: str
    report_type: ReportType


class Report(ReportBase):
    id: str
    created_at: datetime
    completed_at: Optional[datetime] = None

    model_config = {"from_attributes": True}


# ============================================================================
# Audit Log
# ============================================================================

AuditAction = Literal[
    "evidence_added",
    "hypothesis_scored",
    "entity_discovered",
    "contradiction_found",
    "monitoring_result",
    "query_generated",
    "analysis_started",
    "analysis_completed",
    "alert_created",
    "hypothesis_created",
    "hypothesis_refuted",
    "hypothesis_confirmed",
    "evidence_ingested_auto",
    "self_questioning",
    "investigation_started",
    "investigation_stopped",
    "case_created",
    "case_updated",
]

AuditActor = Literal["system", "user", "autonomous_loop", "monitoring"]


class AuditEntryBase(BaseModel):
    case_id: str
    actor: str
    action: str
    target_type: Optional[str] = None
    target_id: Optional[str] = None
    summary: str
    details: Optional[Any] = None
    cycle_number: Optional[int] = None


class AuditEntry(AuditEntryBase):
    id: str
    timestamp: datetime

    model_config = {"from_attributes": True}
