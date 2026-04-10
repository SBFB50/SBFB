# NEXUS -- Database Schema Reference

> Auto-generated from DDL in `nexus/db/sqlite_db.py`, `nexus/gov/db.py`, and `nexus/compute/db.py`.
> SQLite with WAL mode, FTS5, and foreign keys enabled.

---

## Table of Contents

1. [Core Investigation Tables (21 tables)](#1-core-investigation-tables)
2. [Government Monitoring Tables (16 tables)](#2-government-monitoring-tables)
3. [Distributed Compute Tables (6 tables)](#3-distributed-compute-tables)
4. [FTS Virtual Tables (4 tables)](#4-fts-virtual-tables)
5. [All Indexes](#5-all-indexes)
6. [Foreign Key Relationships](#6-foreign-key-relationships)

---

## 1. Core Investigation Tables

Source: `nexus/db/sqlite_db.py`

### cases

Central record for each investigation.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| name | TEXT | NOT NULL |
| reference | TEXT | |
| description | TEXT | |
| status | TEXT | DEFAULT 'active' |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### evidence

Individual pieces of evidence attached to a case.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| title | TEXT | NOT NULL |
| evidence_type | TEXT | NOT NULL |
| source | TEXT | |
| source_date | DATETIME | |
| ingestion_date | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| reliability | INTEGER | DEFAULT 50 |
| file_path | TEXT | |
| raw_text | TEXT | |
| summary | TEXT | |
| metadata | TEXT | |
| status | TEXT | DEFAULT 'pending' |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### entities

Named entities extracted from evidence (persons, organizations, locations, etc.).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| name | TEXT | NOT NULL |
| entity_type | TEXT | NOT NULL |
| aliases | TEXT | |
| description | TEXT | |
| first_seen | DATETIME | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### entity_mentions

Links an entity to the evidence where it was found.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| entity_id | TEXT | NOT NULL, REFERENCES entities(id) |
| evidence_id | TEXT | NOT NULL, REFERENCES evidence(id) |
| context | TEXT | |
| confidence | REAL | DEFAULT 0.8 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### hypotheses

Investigation hypotheses with ACH scoring.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| title | TEXT | NOT NULL |
| description | TEXT | NOT NULL |
| status | TEXT | DEFAULT 'active' |
| current_score | REAL | DEFAULT 50.0 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### hypothesis_snapshots

Score history for each hypothesis over time.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| hypothesis_id | TEXT | NOT NULL, REFERENCES hypotheses(id) |
| score | REAL | NOT NULL |
| supporting | TEXT | |
| contradicting | TEXT | |
| reasoning | TEXT | |
| trigger | TEXT | |
| model_used | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### analysis_runs

Record of each analysis pipeline execution.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| run_type | TEXT | NOT NULL |
| trigger | TEXT | |
| status | TEXT | DEFAULT 'running' |
| model_used | TEXT | |
| input_summary | TEXT | |
| output_summary | TEXT | |
| duration_sec | REAL | |
| tokens_used | INTEGER | |
| started_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| completed_at | DATETIME | |

---

### monitoring_jobs

Scheduled web monitoring queries (SearXNG, Robin, Wayback).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| job_type | TEXT | NOT NULL |
| query | TEXT | NOT NULL |
| entity_id | TEXT | REFERENCES entities(id) |
| interval_hours | INTEGER | DEFAULT 24 |
| is_active | BOOLEAN | DEFAULT 1 |
| last_run | DATETIME | |
| next_run | DATETIME | |
| results_count | INTEGER | DEFAULT 0 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### monitoring_results

Individual results returned by monitoring jobs.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| job_id | TEXT | NOT NULL, REFERENCES monitoring_jobs(id) |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| url | TEXT | |
| title | TEXT | |
| snippet | TEXT | |
| source_engine | TEXT | |
| relevance_score | REAL | |
| is_new | BOOLEAN | DEFAULT 1 |
| is_duplicate | BOOLEAN | DEFAULT 0 |
| reviewed | BOOLEAN | DEFAULT 0 |
| found_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### alerts

System alerts and notifications per case.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| alert_type | TEXT | NOT NULL |
| severity | TEXT | DEFAULT 'info' |
| title | TEXT | NOT NULL |
| message | TEXT | NOT NULL |
| related_id | TEXT | |
| is_read | BOOLEAN | DEFAULT 0 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### reports

Generated investigation reports (PDF, etc.).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| report_type | TEXT | NOT NULL |
| status | TEXT | DEFAULT 'generating' |
| file_path | TEXT | |
| file_size | INTEGER | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| completed_at | DATETIME | |

---

### locations

Geocoded locations linked to entities and cases.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| entity_id | TEXT | REFERENCES entities(id) |
| name | TEXT | NOT NULL |
| address | TEXT | |
| lat | REAL | |
| lon | REAL | |
| location_type | TEXT | DEFAULT 'other' |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### audit_log

Immutable, hash-chained audit trail for every action.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| timestamp | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| actor | TEXT | NOT NULL |
| action | TEXT | NOT NULL |
| target_type | TEXT | |
| target_id | TEXT | |
| summary | TEXT | NOT NULL |
| details | TEXT | |
| cycle_number | INTEGER | |
| entry_hash | TEXT | |
| previous_hash | TEXT | |

---

### summary_clusters

RAPTOR summary clusters grouping related evidence.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| title | TEXT | |
| summary | TEXT | |
| evidence_ids | TEXT | |
| embedding_centroid | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### suspects

Scored suspect profiles linked to entities.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| entity_id | TEXT | NOT NULL, REFERENCES entities(id) |
| suspicion_score | REAL | DEFAULT 0.0 |
| graph_score | REAL | DEFAULT 0.0 |
| evidence_score | REAL | DEFAULT 0.0 |
| contradiction_score | REAL | DEFAULT 0.0 |
| profile_score | REAL | DEFAULT 0.0 |
| hypothesis_score | REAL | DEFAULT 0.0 |
| known_motive | TEXT | |
| alibi_status | TEXT | DEFAULT 'unknown' |
| criminal_record | TEXT | |
| relationship_to_victim | TEXT | |
| notes | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(case_id, entity_id)`

---

### suspect_snapshots

Score history for each suspect over time.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| suspect_id | TEXT | NOT NULL, REFERENCES suspects(id) |
| suspicion_score | REAL | NOT NULL |
| graph_score | REAL | |
| evidence_score | REAL | |
| contradiction_score | REAL | |
| profile_score | REAL | |
| hypothesis_score | REAL | |
| trigger | TEXT | |
| reasoning | TEXT | |
| model_used | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### case_summaries

Top-level case summary (one per case).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id), UNIQUE |
| summary | TEXT | |
| cluster_ids | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### event_log

EventBus persistence -- every event published through the reactive system.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| event_type | TEXT | NOT NULL |
| case_id | TEXT | NOT NULL |
| payload | TEXT | |
| source_worker | TEXT | |
| parent_event_id | TEXT | |
| status | TEXT | DEFAULT 'pending' |
| processed_by | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| processed_at | DATETIME | |

---

### contradictions

Detected factual contradictions between pieces of evidence.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| evidence_1_id | TEXT | REFERENCES evidence(id) |
| evidence_2_id | TEXT | REFERENCES evidence(id) |
| evidence_1_title | TEXT | |
| evidence_2_title | TEXT | |
| contradiction_type | TEXT | DEFAULT 'factual' |
| severity | TEXT | DEFAULT 'medium' |
| description | TEXT | NOT NULL |
| likely_correct | TEXT | |
| reasoning | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(case_id, evidence_1_id, evidence_2_id, contradiction_type)`

---

### wiki_pages

Auto-generated investigation wiki pages.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| page_path | TEXT | NOT NULL |
| page_type | TEXT | NOT NULL |
| title | TEXT | NOT NULL |
| content_hash | TEXT | |
| last_compiled | DATETIME | |
| source_ids | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(case_id, page_path)`

---

### investigation_memory

Persistent memory of key insights discovered during investigation.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| case_id | TEXT | NOT NULL, REFERENCES cases(id) |
| insight_type | TEXT | NOT NULL |
| source_event_type | TEXT | NOT NULL |
| importance | REAL | DEFAULT 0.5 |
| confidence | REAL | DEFAULT 0.7 |
| summary | TEXT | NOT NULL |
| full_context | TEXT | |
| related_entities | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

## 2. Government Monitoring Tables

Source: `nexus/gov/db.py`

### gov_politicians

French politicians tracked by the monitoring system.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| name | TEXT | NOT NULL |
| slug | TEXT | NOT NULL |
| chamber | TEXT | NOT NULL |
| party | TEXT | |
| role | TEXT | |
| constituency | TEXT | |
| photo_url | TEXT | |
| official_url | TEXT | |
| hatvp_url | TEXT | |
| active | INTEGER | DEFAULT 1 |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_positions

Public positions taken by politicians (votes, declarations, statements).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| subject | TEXT | NOT NULL |
| position_type | TEXT | NOT NULL |
| position_text | TEXT | NOT NULL |
| stance | TEXT | |
| source_url | TEXT | NOT NULL |
| source_type | TEXT | |
| date | DATE | |
| session | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_contradictions

Detected contradictions between two positions of the same politician.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| position_a_id | TEXT | NOT NULL, REFERENCES gov_positions(id) |
| position_b_id | TEXT | NOT NULL, REFERENCES gov_positions(id) |
| subject | TEXT | NOT NULL |
| description | TEXT | NOT NULL |
| severity | TEXT | DEFAULT 'medium' |
| source_verified | INTEGER | DEFAULT 0 |
| metadata | TEXT | |
| detected_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_scan_log

Log of each scraping/sync scan with checkpoint/resume support.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| scan_type | TEXT | NOT NULL |
| status | TEXT | DEFAULT 'running' |
| items_found | INTEGER | DEFAULT 0 |
| items_new | INTEGER | DEFAULT 0 |
| error_message | TEXT | |
| started_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| completed_at | DATETIME | |
| current_phase | TEXT | DEFAULT '' |
| phase_offset | INTEGER | DEFAULT 0 |
| checkpoint_data | TEXT | DEFAULT '{}' |

---

### gov_mandates

Political mandates (deputy, senator, mayor, etc.).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| type | TEXT | NOT NULL |
| title | TEXT | |
| institution | TEXT | |
| constituency | TEXT | |
| start_date | DATE | |
| end_date | DATE | |
| is_current | INTEGER | DEFAULT 0 |
| parliamentary_group | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_parties

Political parties.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| name | TEXT | NOT NULL |
| short_name | TEXT | UNIQUE |
| color | TEXT | |
| description | TEXT | |
| leader | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_party_memberships

Tracks which politician belongs to which party over time.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| party_id | TEXT | NOT NULL, REFERENCES gov_parties(id) |
| start_date | DATE | |
| end_date | DATE | |
| is_current | INTEGER | DEFAULT 0 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_affairs

Political scandals and legal affairs involving politicians.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| title | TEXT | NOT NULL |
| description | TEXT | |
| status | TEXT | DEFAULT 'enquete' |
| category | TEXT | |
| involvement | TEXT | DEFAULT 'direct' |
| source_url | TEXT | |
| date_start | DATE | |
| date_end | DATE | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_declarations

HATVP asset/interest declarations.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| type | TEXT | NOT NULL |
| qualite | TEXT | |
| departement | TEXT | |
| date_publication | DATE | |
| date_depot | DATE | |
| url | TEXT | |
| status | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_laws

Legislative texts tracked through parliament.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| uid | TEXT | UNIQUE |
| title | TEXT | NOT NULL |
| short_title | TEXT | |
| procedure | TEXT | |
| status | TEXT | |
| initiator_ref | TEXT | |
| date_initial | DATE | |
| date_promulgation | DATE | |
| legislature | TEXT | |
| amendments_count | INTEGER | DEFAULT 0 |
| amendments_adopted | INTEGER | DEFAULT 0 |
| articles_initial | INTEGER | DEFAULT 0 |
| articles_final | INTEGER | DEFAULT 0 |
| duration_days | INTEGER | DEFAULT 0 |
| source_url | TEXT | |
| jo_url | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_press

Press articles mentioning politicians.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| title | TEXT | NOT NULL |
| url | TEXT | UNIQUE |
| source_name | TEXT | |
| published_at | DATETIME | |
| summary | TEXT | |
| sentiment | TEXT | |
| politicians_mentioned | TEXT | |
| subjects | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_social_posts

Social media posts by politicians (Twitter, Facebook, Instagram, TikTok, YouTube).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| platform | TEXT | NOT NULL |
| post_id | TEXT | |
| content | TEXT | |
| url | TEXT | |
| media_type | TEXT | |
| media_url | TEXT | |
| posted_at | DATETIME | |
| likes | INTEGER | DEFAULT 0 |
| shares | INTEGER | DEFAULT 0 |
| comments | INTEGER | DEFAULT 0 |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(platform, post_id)`

---

### gov_transcriptions

Audio/video transcriptions from parliament sessions or media appearances.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| source_type | TEXT | NOT NULL |
| source_url | TEXT | |
| politician_id | TEXT | REFERENCES gov_politicians(id) |
| title | TEXT | |
| transcription | TEXT | |
| timestamped_text | TEXT | |
| duration_seconds | INTEGER | |
| language | TEXT | DEFAULT 'fr' |
| model_used | TEXT | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_factchecks

Fact-check records for claims made by politicians.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| claim | TEXT | NOT NULL |
| claim_date | DATE | |
| claimant | TEXT | |
| politician_id | TEXT | REFERENCES gov_politicians(id) |
| rating | TEXT | |
| review_url | TEXT | |
| reviewer | TEXT | |
| review_date | DATE | |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### gov_external_ids

Cross-reference identifiers linking politicians to external sources (Wikidata, NosDonnees, AN, Senat...).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| politician_id | TEXT | NOT NULL, REFERENCES gov_politicians(id) |
| source | TEXT | NOT NULL |
| external_id | TEXT | NOT NULL |
| confidence | REAL | DEFAULT 1.0 |
| metadata | TEXT | |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(source, external_id)`

---

### gov_alerts

Government monitoring alerts and notifications.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| alert_type | TEXT | NOT NULL |
| title | TEXT | NOT NULL |
| description | TEXT | |
| severity | TEXT | DEFAULT 'info' |
| politician_id | TEXT | REFERENCES gov_politicians(id) |
| event_id | TEXT | |
| is_read | INTEGER | DEFAULT 0 |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

## 3. Distributed Compute Tables

Source: `nexus/compute/db.py`

### compute_nodes

GPU contributor registry -- each node donating compute power.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| name | TEXT | NOT NULL |
| gpu_model | TEXT | NOT NULL |
| vram_mb | INTEGER | NOT NULL |
| platform | TEXT | DEFAULT '' |
| ollama_version | TEXT | DEFAULT '' |
| status | TEXT | DEFAULT 'idle' |
| connected_at | DATETIME | |
| last_heartbeat | DATETIME | |
| tasks_completed | INTEGER | DEFAULT 0 |
| tasks_errored | INTEGER | DEFAULT 0 |
| avg_tokens_per_sec | REAL | DEFAULT 0.0 |
| trust_score | INTEGER | DEFAULT 50 |
| api_key_hash | TEXT | NOT NULL |
| ip_hash | TEXT | NOT NULL |
| public_key | TEXT | DEFAULT '' |
| current_model | TEXT | DEFAULT '' |
| assigned_model | TEXT | DEFAULT '' |
| model_status | TEXT | DEFAULT '' |
| model_pull_started_at | DATETIME | |
| metadata | TEXT | DEFAULT '{}' |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**24 columns** -- privacy-aware design: IP stored as SHA-256 hash, API key stored as SHA-256 hash.

---

### compute_model_transitions

Tracks cluster-wide model tier transitions (e.g., upgrading from 7B to 26B).

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| old_model | TEXT | DEFAULT '' |
| new_model | TEXT | DEFAULT '' |
| old_tier | TEXT | DEFAULT '' |
| new_tier | TEXT | DEFAULT '' |
| total_vram_gb | REAL | DEFAULT 0.0 |
| nodes_online | INTEGER | DEFAULT 0 |
| nodes_ready | INTEGER | DEFAULT 0 |
| transition_state | TEXT | DEFAULT 'transitioning' |
| started_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| completed_at | DATETIME | |

---

### compute_tasks

LLM inference task queue for distributed processing.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| task_type | TEXT | NOT NULL |
| prompt | TEXT | NOT NULL |
| system_prompt | TEXT | DEFAULT '' |
| model | TEXT | DEFAULT '' |
| status | TEXT | DEFAULT 'pending' |
| priority | INTEGER | DEFAULT 5 |
| assigned_to | TEXT | REFERENCES compute_nodes(id) |
| assigned_at | DATETIME | |
| completed_at | DATETIME | |
| result | TEXT | |
| result_validated | INTEGER | DEFAULT 0 |
| validation_score | REAL | DEFAULT 0.0 |
| timeout_seconds | INTEGER | DEFAULT 300 |
| require_logprobs | INTEGER | DEFAULT 0 |
| calibration_prompt | TEXT | DEFAULT '' |
| source_worker | TEXT | DEFAULT '' |
| parent_task_id | TEXT | DEFAULT '' |
| error_message | TEXT | DEFAULT '' |
| retry_count | INTEGER | DEFAULT 0 |
| max_retries | INTEGER | DEFAULT 3 |
| execution_mode | TEXT | DEFAULT 'local' |
| metadata | TEXT | DEFAULT '{}' |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**25 columns** -- supports task validation, retries, logprobs verification, and parent/child chaining.

---

### compute_results

Validated results returned by compute nodes.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| task_id | TEXT | NOT NULL, REFERENCES compute_tasks(id) |
| node_id | TEXT | NOT NULL, REFERENCES compute_nodes(id) |
| result_text | TEXT | NOT NULL |
| tokens_generated | INTEGER | DEFAULT 0 |
| generation_time_ms | INTEGER | DEFAULT 0 |
| model_digest | TEXT | DEFAULT '' |
| logprobs | TEXT | DEFAULT '' |
| signature | TEXT | DEFAULT '' |
| validated | INTEGER | DEFAULT 0 |
| validation_method | TEXT | DEFAULT '' |
| metadata | TEXT | DEFAULT '{}' |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

---

### compute_badges

Achievement badges awarded to contributor nodes.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| node_id | TEXT | NOT NULL, REFERENCES compute_nodes(id) |
| badge_id | TEXT | NOT NULL |
| badge_name | TEXT | NOT NULL |
| awarded_at | DATETIME | DEFAULT CURRENT_TIMESTAMP |

**Table constraint:** `UNIQUE(node_id, badge_id)`

---

### compute_uptime_log

Connection session history for each compute node.

| Column | Type | Constraints |
|--------|------|-------------|
| id | TEXT | PRIMARY KEY |
| node_id | TEXT | NOT NULL, REFERENCES compute_nodes(id) |
| connected_at | DATETIME | NOT NULL |
| disconnected_at | DATETIME | |
| duration_seconds | INTEGER | DEFAULT 0 |

---

## 4. FTS Virtual Tables

SQLite FTS5 virtual tables for full-text search. Each is backed by a content table and kept in sync via INSERT/UPDATE/DELETE triggers.

### evidence_fts

Source: `nexus/db/sqlite_db.py`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS evidence_fts USING fts5(
    title, raw_text, summary, source,
    content=evidence, content_rowid=rowid
);
```

| Indexed Column | Source Table Column |
|----------------|---------------------|
| title | evidence.title |
| raw_text | evidence.raw_text |
| summary | evidence.summary |
| source | evidence.source |

**Triggers:** `evidence_fts_insert`, `evidence_fts_update`, `evidence_fts_delete`

---

### gov_positions_fts

Source: `nexus/gov/db.py`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS gov_positions_fts USING fts5(
    subject, position_text, content=gov_positions, content_rowid=rowid
);
```

| Indexed Column | Source Table Column |
|----------------|---------------------|
| subject | gov_positions.subject |
| position_text | gov_positions.position_text |

**Triggers:** `gov_positions_ai`, `gov_positions_ad`, `gov_positions_au`

---

### gov_press_fts

Source: `nexus/gov/db.py`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS gov_press_fts USING fts5(
    title, summary, content=gov_press, content_rowid=rowid
);
```

| Indexed Column | Source Table Column |
|----------------|---------------------|
| title | gov_press.title |
| summary | gov_press.summary |

**Triggers:** `gov_press_ai`, `gov_press_ad`, `gov_press_au`

---

### gov_transcriptions_fts

Source: `nexus/gov/db.py`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS gov_transcriptions_fts USING fts5(
    title, transcription, content=gov_transcriptions, content_rowid=rowid
);
```

| Indexed Column | Source Table Column |
|----------------|---------------------|
| title | gov_transcriptions.title |
| transcription | gov_transcriptions.transcription |

**Triggers:** `gov_transcriptions_ai`, `gov_transcriptions_ad`, `gov_transcriptions_au`

---

## 5. All Indexes

### Core Investigation Indexes (28)

Source: `nexus/db/sqlite_db.py`

| Index Name | Table | Column(s) |
|------------|-------|-----------|
| idx_evidence_case | evidence | case_id |
| idx_entities_case | entities | case_id |
| idx_hypotheses_case | hypotheses | case_id |
| idx_snapshots_hyp | hypothesis_snapshots | hypothesis_id |
| idx_monitoring_case | monitoring_jobs | case_id |
| idx_alerts_case_read | alerts | case_id, is_read |
| idx_analysis_case | analysis_runs | case_id |
| idx_reports_case | reports | case_id |
| idx_locations_case | locations | case_id |
| idx_locations_entity | locations | entity_id |
| idx_audit_case | audit_log | case_id |
| idx_audit_timestamp | audit_log | timestamp |
| idx_audit_action | audit_log | action |
| idx_clusters_case | summary_clusters | case_id |
| idx_suspects_case | suspects | case_id |
| idx_suspect_snapshots | suspect_snapshots | suspect_id |
| idx_evidence_case_type | evidence | case_id, evidence_type |
| idx_evidence_case_status | evidence | case_id, status |
| idx_entities_case_type | entities | case_id, entity_type |
| idx_mentions_evidence | entity_mentions | evidence_id |
| idx_mentions_entity | entity_mentions | entity_id |
| idx_monitoring_results_job | monitoring_results | job_id |
| idx_monitoring_results_case | monitoring_results | case_id |
| idx_event_log_status | event_log | status |
| idx_event_log_type | event_log | event_type |
| idx_event_log_case | event_log | case_id |
| idx_contradictions_case | contradictions | case_id |
| idx_contradictions_evidence | contradictions | evidence_1_id, evidence_2_id |
| idx_memory_case | investigation_memory | case_id |
| idx_wiki_pages_case | wiki_pages | case_id |
| idx_wiki_pages_path | wiki_pages | case_id, page_path |

### Government Indexes (32)

Source: `nexus/gov/db.py`

| Index Name | Table | Column(s) |
|------------|-------|-----------|
| idx_gov_positions_politician | gov_positions | politician_id |
| idx_gov_positions_date | gov_positions | date |
| idx_gov_positions_subject | gov_positions | subject |
| idx_gov_positions_type | gov_positions | position_type |
| idx_gov_contradictions_politician | gov_contradictions | politician_id |
| idx_gov_contradictions_subject | gov_contradictions | subject |
| idx_gov_scan_log_type | gov_scan_log | scan_type |
| idx_gov_politicians_chamber | gov_politicians | chamber |
| idx_gov_politicians_party | gov_politicians | party |
| idx_gov_politicians_slug | gov_politicians | slug |
| idx_gov_mandates_politician | gov_mandates | politician_id |
| idx_gov_mandates_current | gov_mandates | is_current |
| idx_gov_mandates_type | gov_mandates | type |
| idx_gov_party_memberships_politician | gov_party_memberships | politician_id |
| idx_gov_affairs_politician | gov_affairs | politician_id |
| idx_gov_affairs_status | gov_affairs | status |
| idx_gov_declarations_politician | gov_declarations | politician_id |
| idx_gov_laws_uid | gov_laws | uid |
| idx_gov_laws_status | gov_laws | status |
| idx_gov_press_url | gov_press | url |
| idx_gov_press_published | gov_press | published_at |
| idx_gov_social_politician | gov_social_posts | politician_id |
| idx_gov_social_platform | gov_social_posts | platform |
| idx_gov_social_posted | gov_social_posts | posted_at |
| idx_gov_transcriptions_politician | gov_transcriptions | politician_id |
| idx_gov_transcriptions_source | gov_transcriptions | source_type |
| idx_gov_factchecks_politician | gov_factchecks | politician_id |
| idx_gov_external_ids_politician | gov_external_ids | politician_id |
| idx_gov_external_ids_source | gov_external_ids | source, external_id |
| idx_gov_alerts_type | gov_alerts | alert_type |
| idx_gov_alerts_read | gov_alerts | is_read |

### Compute Indexes (16)

Source: `nexus/compute/db.py`

| Index Name | Table | Column(s) |
|------------|-------|-----------|
| idx_compute_nodes_status | compute_nodes | status |
| idx_compute_nodes_trust | compute_nodes | trust_score |
| idx_compute_nodes_api_key | compute_nodes | api_key_hash |
| idx_compute_tasks_status | compute_tasks | status |
| idx_compute_tasks_priority | compute_tasks | priority, created_at |
| idx_compute_tasks_assigned | compute_tasks | assigned_to, status |
| idx_compute_tasks_type | compute_tasks | task_type |
| idx_compute_results_task | compute_results | task_id |
| idx_compute_results_node | compute_results | node_id |
| idx_compute_nodes_model_status | compute_nodes | model_status |
| idx_compute_nodes_assigned_model | compute_nodes | assigned_model |
| idx_compute_transitions_state | compute_model_transitions | transition_state |
| idx_compute_badges_node | compute_badges | node_id |
| idx_compute_uptime_node | compute_uptime_log | node_id |
| idx_compute_uptime_connected | compute_uptime_log | connected_at |

---

## 6. Foreign Key Relationships

All foreign keys use `TEXT` references (UUID4 strings). SQLite enforces these when `PRAGMA foreign_keys = ON`.

```
cases
 |-- evidence.case_id
 |    |-- entity_mentions.evidence_id
 |    |-- contradictions.evidence_1_id
 |    |-- contradictions.evidence_2_id
 |-- entities.case_id
 |    |-- entity_mentions.entity_id
 |    |-- locations.entity_id
 |    |-- suspects.entity_id
 |-- hypotheses.case_id
 |    |-- hypothesis_snapshots.hypothesis_id
 |-- analysis_runs.case_id
 |-- monitoring_jobs.case_id
 |    |-- monitoring_results.job_id
 |-- monitoring_results.case_id
 |-- alerts.case_id
 |-- reports.case_id
 |-- locations.case_id
 |-- audit_log.case_id
 |-- summary_clusters.case_id
 |-- suspects.case_id
 |    |-- suspect_snapshots.suspect_id
 |-- case_summaries.case_id
 |-- contradictions.case_id
 |-- wiki_pages.case_id
 |-- investigation_memory.case_id

monitoring_jobs.entity_id --> entities.id

gov_politicians
 |-- gov_positions.politician_id
 |    |-- gov_contradictions.position_a_id
 |    |-- gov_contradictions.position_b_id
 |-- gov_contradictions.politician_id
 |-- gov_mandates.politician_id
 |-- gov_party_memberships.politician_id
 |-- gov_affairs.politician_id
 |-- gov_declarations.politician_id
 |-- gov_social_posts.politician_id
 |-- gov_transcriptions.politician_id  (nullable)
 |-- gov_factchecks.politician_id      (nullable)
 |-- gov_external_ids.politician_id
 |-- gov_alerts.politician_id          (nullable)

gov_parties
 |-- gov_party_memberships.party_id

compute_nodes
 |-- compute_tasks.assigned_to         (nullable)
 |-- compute_results.node_id
 |-- compute_badges.node_id
 |-- compute_uptime_log.node_id

compute_tasks
 |-- compute_results.task_id
```

---

## Summary

| Domain | Tables | FTS Tables | Indexes | Total Columns |
|--------|--------|------------|---------|---------------|
| Core Investigation | 21 | 1 | 31 | 183 |
| Government Monitoring | 16 | 3 | 31 | 173 |
| Distributed Compute | 6 | 0 | 15 | 82 |
| **Total** | **43** | **4** | **77** | **438** |

All tables use `TEXT PRIMARY KEY` (UUID4), `DATETIME DEFAULT CURRENT_TIMESTAMP` for creation tracking, and `TEXT` for JSON-serialized metadata fields.
