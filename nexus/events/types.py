"""
NEXUS -- Event types and core event dataclass.

Defines every event flowing through the system and the immutable
NexusEvent value object that carries context between workers.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any
import uuid


class EventType(str, Enum):
    """All event types recognised by the NEXUS event bus."""

    # Evidence lifecycle
    EVIDENCE_ADDED = "evidence_added"
    EVIDENCE_PROCESSED = "evidence_processed"
    EVIDENCE_CHUNKED = "evidence_chunked"

    # Entity lifecycle
    ENTITY_DISCOVERED = "entity_discovered"
    ENTITY_ENRICHED = "entity_enriched"

    # Monitoring
    MONITORING_RESULT = "monitoring_result"

    # Analysis
    ANALYSIS_COMPLETED = "analysis_completed"

    # Hypothesis
    HYPOTHESIS_CREATED = "hypothesis_created"
    HYPOTHESIS_SCORED = "hypothesis_scored"

    # Contradiction
    CONTRADICTION_FOUND = "contradiction_found"

    # Forensics
    FORENSIC_RESULT = "forensic_result"

    # Suspect
    SUSPECT_SCORED = "suspect_scored"

    # Geo
    LOCATION_GEOCODED = "location_geocoded"

    # Timeline
    TIMELINE_REBUILT = "timeline_rebuilt"

    # Periodic ticks
    TICK_REPORT = "tick_report"
    TICK_BACKUP = "tick_backup"
    TICK_SUMMARY_TREE = "tick_summary_tree"

    # Wiki
    WIKI_UPDATED = "wiki_updated"
    TICK_WIKI_LINT = "tick_wiki_lint"


@dataclass(frozen=True, slots=True)
class NexusEvent:
    """Immutable event flowing through the NEXUS event bus.

    Attributes:
        event_type:      Discriminator for routing.
        case_id:         Investigation case this event belongs to.
        payload:         Arbitrary data specific to the event type.
        event_id:        Unique identifier (UUID4).
        timestamp:       UTC ISO-8601 creation time.
        source_worker:   Name of the worker that published this event.
        parent_event_id: Optional ID of the event that triggered this one,
                         enabling causal-chain tracing.
    """

    event_type: EventType
    case_id: str
    payload: dict[str, Any] = field(default_factory=dict)
    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    timestamp: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )
    source_worker: str = ""
    parent_event_id: str | None = None
