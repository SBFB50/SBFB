"""
NEXUS -- Reactive worker implementations.

Each worker wraps an existing NEXUS tool class and adapts it to the
event-driven reactive architecture.  Workers are thin adapters between
the EventBus and the core business logic.
"""

from nexus.events.workers.evidence_ingest import EvidenceIngestWorker
from nexus.events.workers.entity_extractor import EntityExtractorWorker
from nexus.events.workers.summarizer import SummarizerWorker
from nexus.events.workers.chunker_embed import ChunkerEmbedWorker
from nexus.events.workers.neo4j_sync import Neo4jSyncWorker
from nexus.events.workers.osint_recon import OSINTReconWorker
from nexus.events.workers.geo_mapper import GeoMapperWorker
from nexus.events.workers.analysis import AnalysisPipelineWorker
from nexus.events.workers.hypothesis import HypothesisWorker
from nexus.events.workers.contradiction import ContradictionWorker
from nexus.events.workers.forensics import ForensicRouterWorker
from nexus.events.workers.suspect_scorer import SuspectScorerWorker
from nexus.events.workers.query_generator import QueryGeneratorWorker
from nexus.events.workers.self_questioning import SelfQuestioningWorker
from nexus.events.workers.alert import AlertWorker
from nexus.events.workers.summary_tree import SummaryTreeWorker
from nexus.events.workers.timeline import TimelineWorker
from nexus.events.workers.memory import MemoryWorker
from nexus.events.workers.wiki_compiler import WikiCompilerWorker
from nexus.events.workers.wiki_lint import WikiLintWorker

__all__ = [
    "EvidenceIngestWorker",
    "EntityExtractorWorker",
    "SummarizerWorker",
    "ChunkerEmbedWorker",
    "Neo4jSyncWorker",
    "OSINTReconWorker",
    "GeoMapperWorker",
    "AnalysisPipelineWorker",
    "HypothesisWorker",
    "ContradictionWorker",
    "ForensicRouterWorker",
    "SuspectScorerWorker",
    "QueryGeneratorWorker",
    "SelfQuestioningWorker",
    "AlertWorker",
    "SummaryTreeWorker",
    "TimelineWorker",
    "MemoryWorker",
    "WikiCompilerWorker",
    "WikiLintWorker",
]
