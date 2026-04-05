#!/usr/bin/env python
"""
NEXUS Benchmark -- Real Cold Cases (Kulik + Golden State Killer)

Injects evidence wave by wave, triggers analysis after each wave,
and scores the system's ability to converge toward the known truth.

The system receives RAW evidence (without the solution) and must:
- Extract entities
- Generate hypotheses
- Detect contradictions
- Build a coherent timeline and geographic model

Scoring: /100 per case
  - Entities found:              /20
  - Correct hypothesis in top 3: /20
  - Contradictions detected:     /20
  - Correct hypothesis > 40%:    /20
  - Timeline + geography:        /20

Usage:
    python tests/bench_real_cases.py
    python tests/bench_real_cases.py --case kulik
    python tests/bench_real_cases.py --case gsk
    python tests/bench_real_cases.py --api-url http://localhost:8000
    python tests/bench_real_cases.py --no-analyze   # skip LLM analysis
    python tests/bench_real_cases.py --timeout 600   # custom analysis timeout
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

BASE_URL = "http://localhost:8000"
BENCHMARK_DIR = Path(__file__).resolve().parent.parent / "data" / "benchmark"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "docs"

TIMEOUT_INJECT = 300     # seconds -- evidence ingestion (includes LLM extraction)
TIMEOUT_ANALYZE = 600    # seconds -- full analysis pipeline
POLL_INTERVAL = 10       # seconds between analysis status polls

CASES = {
    "kulik": {
        "dir": "kulik",
        "display_name": "Affaire Elodie Kulik",
    },
    "gsk": {
        "dir": "golden-state-killer",
        "display_name": "Golden State Killer (EAR/ONS)",
    },
}

# ---------------------------------------------------------------------------
# ANSI color helpers
# ---------------------------------------------------------------------------

_COLORS = {
    "reset": "\033[0m",
    "bold": "\033[1m",
    "dim": "\033[2m",
    "red": "\033[91m",
    "green": "\033[92m",
    "yellow": "\033[93m",
    "blue": "\033[94m",
    "cyan": "\033[96m",
}


def _c(text: str, *styles: str) -> str:
    prefix = "".join(_COLORS.get(s, "") for s in styles)
    return f"{prefix}{text}{_COLORS['reset']}"


def _header(msg: str) -> None:
    border = "=" * 64
    print(f"\n{_c(border, 'cyan', 'bold')}")
    print(_c(f"  {msg}", "cyan", "bold"))
    print(f"{_c(border, 'cyan', 'bold')}")


def _sub(msg: str) -> None:
    print(f"\n{_c('--- ' + msg + ' ---', 'blue', 'bold')}")


def _ok(msg: str) -> None:
    print(f"  {_c('[OK]', 'green', 'bold')} {msg}")


def _fail(msg: str) -> None:
    print(f"  {_c('[FAIL]', 'red', 'bold')} {msg}")


def _warn(msg: str) -> None:
    print(f"  {_c('[WARN]', 'yellow', 'bold')} {msg}")


def _info(msg: str) -> None:
    print(f"  {_c('[i]', 'blue')} {msg}")


def _score_line(label: str, points: float, max_pts: float) -> None:
    pct = (points / max_pts * 100) if max_pts > 0 else 0
    color = "green" if pct >= 70 else ("yellow" if pct >= 40 else "red")
    bar_len = 20
    filled = int(bar_len * pct / 100)
    bar = _c("#" * filled, color) + _c("-" * (bar_len - filled), "dim")
    print(f"  [{bar}] {_c(f'{points:.0f}/{max_pts:.0f}', color, 'bold')}  {label}")


# ---------------------------------------------------------------------------
# API helpers
# ---------------------------------------------------------------------------

class APIError(Exception):
    pass


def _api(
    method: str,
    path: str,
    *,
    base: str = BASE_URL,
    json_body: Any = None,
    timeout: int = TIMEOUT_INJECT,
) -> Any:
    url = f"{base.rstrip('/')}{path}"
    try:
        resp = requests.request(method, url, json=json_body, timeout=timeout)
    except requests.ConnectionError:
        raise APIError(f"Connection refused: {base}. Is the NEXUS backend running?")
    except requests.Timeout:
        raise APIError(f"Timeout ({timeout}s) on {method} {path}")

    if resp.status_code == 204:
        return None
    if resp.status_code >= 400:
        detail = ""
        try:
            detail = resp.json().get("detail", resp.text[:200])
        except Exception:
            detail = resp.text[:200]
        raise APIError(f"HTTP {resp.status_code} on {method} {path}: {detail}")
    return resp.json()


def _api_safe(method: str, path: str, **kwargs) -> Any:
    """API call that returns None on error instead of raising."""
    try:
        return _api(method, path, **kwargs)
    except APIError as e:
        _warn(str(e))
        return None


# ---------------------------------------------------------------------------
# Data fetchers
# ---------------------------------------------------------------------------

def fetch_entities(base: str, case_id: str) -> List[Dict]:
    return _api_safe("GET", f"/api/cases/{case_id}/entities", base=base) or []


def fetch_hypotheses(base: str, case_id: str) -> List[Dict]:
    return _api_safe("GET", f"/api/cases/{case_id}/hypotheses", base=base) or []


def fetch_contradictions(base: str, case_id: str) -> List[Dict]:
    return _api_safe(
        "GET", f"/api/cases/{case_id}/contradictions",
        base=base, timeout=TIMEOUT_ANALYZE,
    ) or []


def fetch_stats(base: str, case_id: str) -> Dict:
    return _api_safe("GET", f"/api/cases/{case_id}/stats", base=base) or {}


def fetch_alerts(base: str, case_id: str) -> List[Dict]:
    return _api_safe("GET", f"/api/cases/{case_id}/alerts", base=base) or []


# ---------------------------------------------------------------------------
# Keyword matching helpers
# ---------------------------------------------------------------------------

_STOPWORDS = {
    "de", "du", "des", "le", "la", "les", "un", "une", "et", "en", "a",
    "au", "aux", "ce", "cette", "par", "pour", "que", "qui", "sur",
    "son", "sa", "ses", "dit", "vs", "avec", "dans", "est", "pas",
    "plus", "vers", "il", "elle", "ils", "elles", "se", "ne", "ni",
    "ou", "mais", "donc", "car", "si", "non", "oui", "the", "of",
    "and", "to", "in", "is", "was", "for", "on", "are", "at",
}


def _keywords(text: str) -> List[str]:
    tokens = text.lower().replace("'", " ").replace("(", " ").replace(")", " ").split()
    return [
        t.strip(".,;:!?\"'")
        for t in tokens
        if t.strip(".,;:!?\"'") not in _STOPWORDS and len(t) > 2
    ]


def _fuzzy_match_hypothesis(expected_title: str, hypotheses: List[Dict]) -> Optional[Dict]:
    kws = _keywords(expected_title)
    if not kws:
        return None
    best, best_score = None, 0
    for h in hypotheses:
        combined = (h.get("title", "") + " " + (h.get("description", "") or "")).lower()
        score = sum(1 for kw in kws if kw in combined)
        if score > best_score:
            best_score = score
            best = h
    return best if best_score >= 1 else None


def _fuzzy_match_contradiction(
    expected_desc: str,
    keywords_list: List[str],
    contradictions: List[Dict],
) -> Optional[Dict]:
    search_kws = keywords_list or _keywords(expected_desc)
    if not search_kws:
        return None
    best, best_score = None, 0
    for c in contradictions:
        c_text = json.dumps(c, ensure_ascii=False).lower()
        score = sum(1 for kw in search_kws if kw in c_text)
        if score > best_score:
            best_score = score
            best = c
    return best if best_score >= 2 else None


def _entity_name_match(expected: str, entity_names: set) -> bool:
    expected_lower = expected.lower()
    for name in entity_names:
        if expected_lower in name or name in expected_lower:
            return True
    return False


# ---------------------------------------------------------------------------
# Scoring engine
# ---------------------------------------------------------------------------

class CaseScorer:
    """Scores system output against manifest expectations."""

    def __init__(self, manifest: Dict):
        self.manifest = manifest
        self.scoring_cfg = manifest["scoring"]
        self.details: Dict[str, Dict] = {}
        self.total = 0.0

    def score_entities(
        self,
        entities: List[Dict],
    ) -> float:
        """Score: /20 for key entities extracted."""
        cfg = self.scoring_cfg["categories"]["entities"]
        max_pts = cfg["points"]

        all_names = set()
        for e in entities:
            all_names.add(e.get("name", "").lower())
            for alias in (e.get("aliases") or []):
                all_names.add(alias.lower())

        required_persons = cfg.get("required_persons", [])
        required_locations = cfg.get("required_locations", [])
        required_other = cfg.get("required_other", [])

        all_required = required_persons + required_locations + required_other
        if not all_required:
            self.details["entities"] = {"points": max_pts, "max": max_pts, "found": [], "missing": []}
            self.total += max_pts
            return max_pts

        found = []
        missing = []
        for req in all_required:
            if _entity_name_match(req, all_names):
                found.append(req)
            else:
                missing.append(req)

        ratio = len(found) / len(all_required) if all_required else 1.0
        points = round(ratio * max_pts, 1)

        self.details["entities"] = {
            "points": points,
            "max": max_pts,
            "found": found,
            "missing": missing,
            "total_entities": len(entities),
        }
        self.total += points
        return points

    def score_hypothesis_ranking(
        self,
        hypotheses: List[Dict],
    ) -> float:
        """Score: /20 if correct hypothesis is in top 3 by score."""
        cfg = self.scoring_cfg["categories"]["hypothesis_ranking"]
        max_pts = cfg["points"]
        target_id = cfg["target_hypothesis_id"]
        max_rank = cfg.get("max_rank", 3)

        # Find expected hypothesis definition
        expected_hyps = self.manifest.get("expected_hypotheses", [])
        target_def = next((h for h in expected_hyps if h["id"] == target_id), None)

        if not target_def or not hypotheses:
            self.details["hypothesis_ranking"] = {
                "points": 0, "max": max_pts,
                "reason": "No hypotheses generated" if not hypotheses else f"Target {target_id} not defined",
            }
            return 0

        # Match target hypothesis
        matched = _fuzzy_match_hypothesis(target_def["title"], hypotheses)
        if not matched:
            self.details["hypothesis_ranking"] = {
                "points": 0, "max": max_pts,
                "reason": f"Target hypothesis '{target_def['title']}' not found in system hypotheses",
                "system_hypotheses": [h.get("title", "?") for h in hypotheses],
            }
            return 0

        # Rank by score (descending)
        sorted_hyps = sorted(hypotheses, key=lambda h: h.get("current_score", 0), reverse=True)
        rank = next(
            (i + 1 for i, h in enumerate(sorted_hyps) if h.get("id") == matched.get("id")),
            len(sorted_hyps),
        )

        in_top = rank <= max_rank
        points = max_pts if in_top else round(max_pts * max_rank / rank, 1)

        self.details["hypothesis_ranking"] = {
            "points": points,
            "max": max_pts,
            "target": target_def["title"],
            "matched": matched.get("title", "?"),
            "rank": rank,
            "in_top_n": in_top,
            "max_rank": max_rank,
        }
        self.total += points
        return points

    def score_contradictions(
        self,
        contradictions: List[Dict],
    ) -> float:
        """Score: /20 for contradictions detected."""
        cfg = self.scoring_cfg["categories"]["contradictions"]
        max_pts = cfg["points"]
        min_found = cfg.get("minimum_found", 2)

        expected = self.manifest.get("expected_contradictions", [])
        if not expected:
            self.details["contradictions"] = {"points": max_pts, "max": max_pts, "reason": "No expected contradictions"}
            self.total += max_pts
            return max_pts

        found = []
        missing = []
        for exp_c in expected:
            kws = exp_c.get("keywords", [])
            match = _fuzzy_match_contradiction(exp_c["description"], kws, contradictions)
            if match:
                found.append(exp_c["id"])
            else:
                missing.append(exp_c["id"])

        ratio = len(found) / len(expected) if expected else 1.0
        points = round(ratio * max_pts, 1)

        self.details["contradictions"] = {
            "points": points,
            "max": max_pts,
            "found": found,
            "missing": missing,
            "total_detected": len(contradictions),
            "minimum_required": min_found,
            "met_minimum": len(found) >= min_found,
        }
        self.total += points
        return points

    def score_hypothesis_value(
        self,
        hypotheses: List[Dict],
    ) -> float:
        """Score: /20 if correct hypothesis score > 40%."""
        cfg = self.scoring_cfg["categories"]["hypothesis_score"]
        max_pts = cfg["points"]
        target_id = cfg["target_hypothesis_id"]
        min_score = cfg.get("minimum_score", 40)

        expected_hyps = self.manifest.get("expected_hypotheses", [])
        target_def = next((h for h in expected_hyps if h["id"] == target_id), None)

        if not target_def or not hypotheses:
            self.details["hypothesis_score"] = {
                "points": 0, "max": max_pts,
                "reason": "No data",
            }
            return 0

        matched = _fuzzy_match_hypothesis(target_def["title"], hypotheses)
        if not matched:
            self.details["hypothesis_score"] = {
                "points": 0, "max": max_pts,
                "reason": f"Target hypothesis not found",
            }
            return 0

        score_val = matched.get("current_score", 0)
        above_min = score_val >= min_score

        if above_min:
            points = max_pts
        elif score_val > 0:
            # Partial credit: proportional to how close to minimum
            points = round(max_pts * score_val / min_score, 1)
        else:
            points = 0

        self.details["hypothesis_score"] = {
            "points": points,
            "max": max_pts,
            "target": target_def["title"],
            "matched": matched.get("title", "?"),
            "actual_score": score_val,
            "minimum_required": min_score,
            "above_minimum": above_min,
        }
        self.total += points
        return points

    def score_timeline_geo(
        self,
        entities: List[Dict],
        hypotheses: List[Dict],
        contradictions: List[Dict],
        all_evidence_text: str,
    ) -> float:
        """Score: /20 for timeline events and geographic links detected.

        This checks that the system correctly extracted temporal and spatial
        relationships from the evidence. Since the system stores these in
        entities, hypotheses, and contradictions, we search across all outputs.
        """
        cfg = self.scoring_cfg["categories"]["timeline_and_geo"]
        max_pts = cfg["points"]

        required_events = cfg.get("required_timeline_events", [])
        required_geo = cfg.get("required_geo_links", [])

        all_requirements = required_events + required_geo
        if not all_requirements:
            self.details["timeline_geo"] = {"points": max_pts, "max": max_pts}
            self.total += max_pts
            return max_pts

        # Build searchable corpus from all system outputs
        corpus_parts = []
        for e in entities:
            corpus_parts.append(e.get("name", ""))
            corpus_parts.append(e.get("description", "") or "")
        for h in hypotheses:
            corpus_parts.append(h.get("title", ""))
            corpus_parts.append(h.get("description", "") or "")
        for c in contradictions:
            corpus_parts.append(json.dumps(c, ensure_ascii=False))
        corpus = " ".join(corpus_parts).lower()

        found_events = []
        missing_events = []
        for event in required_events:
            event_kws = _keywords(event)
            matches = sum(1 for kw in event_kws if kw in corpus)
            if matches >= max(1, len(event_kws) // 2):
                found_events.append(event)
            else:
                missing_events.append(event)

        found_geo = []
        missing_geo = []
        for geo in required_geo:
            geo_kws = _keywords(geo)
            matches = sum(1 for kw in geo_kws if kw in corpus)
            if matches >= max(1, len(geo_kws) // 2):
                found_geo.append(geo)
            else:
                missing_geo.append(geo)

        total_found = len(found_events) + len(found_geo)
        total_required = len(all_requirements)
        ratio = total_found / total_required if total_required > 0 else 1.0
        points = round(ratio * max_pts, 1)

        self.details["timeline_geo"] = {
            "points": points,
            "max": max_pts,
            "events_found": found_events,
            "events_missing": missing_events,
            "geo_found": found_geo,
            "geo_missing": missing_geo,
        }
        self.total += points
        return points

    def get_total(self) -> float:
        return round(self.total, 1)

    def get_report(self) -> Dict:
        return {
            "total_score": self.get_total(),
            "max_score": self.scoring_cfg["total_points"],
            "categories": self.details,
        }


# ---------------------------------------------------------------------------
# Core benchmark runner for one case
# ---------------------------------------------------------------------------

def run_case(
    case_key: str,
    base: str,
    no_analyze: bool = False,
    timeout_analyze: int = TIMEOUT_ANALYZE,
) -> Dict:
    """Run the full benchmark for one case. Returns the result dict."""

    case_cfg = CASES[case_key]
    case_dir = BENCHMARK_DIR / case_cfg["dir"]
    manifest_path = case_dir / "manifest.json"

    _header(f"BENCHMARK: {case_cfg['display_name']}")

    # -- Load manifest --
    if not manifest_path.exists():
        _fail(f"Manifest not found: {manifest_path}")
        return {"error": f"Manifest not found: {manifest_path}"}

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    evidence_list = manifest["evidence"]
    waves = manifest["waves"]
    _ok(f"Manifest loaded: {len(evidence_list)} evidence items, {len(waves)} waves")

    # -- Create case --
    _sub("Creating case")
    try:
        case_data = _api(
            "POST", "/api/cases",
            base=base,
            json_body={
                "name": f"{manifest['case']['name']} (benchmark)",
                "reference": manifest["case"].get("reference"),
                "description": manifest["case"].get("description"),
            },
        )
        case_id = case_data["id"]
        _ok(f"Case created: {case_id}")
    except APIError as e:
        _fail(f"Cannot create case: {e}")
        return {"error": str(e)}

    # -- Inject evidence wave by wave --
    all_evidence_text = ""
    wave_results = []
    wave_numbers = sorted(waves.keys(), key=int)

    for wave_num_str in wave_numbers:
        wave_num = int(wave_num_str)
        wave_meta = waves[wave_num_str]
        wave_evidence = [e for e in evidence_list if e["wave"] == wave_num]

        _header(f"WAVE {wave_num}: {wave_meta['name']}")
        _info(wave_meta["description"])
        _info(f"{len(wave_evidence)} evidence items to inject")

        wave_start = time.time()
        injected = 0
        errors = 0

        for ev in wave_evidence:
            file_path = case_dir / ev["file"]
            if not file_path.exists():
                _fail(f"Missing file: {ev['file']}")
                errors += 1
                continue

            text = file_path.read_text(encoding="utf-8")
            all_evidence_text += " " + text
            label = f"[{ev['id']}] {ev['title']}"

            try:
                result = _api(
                    "POST",
                    f"/api/cases/{case_id}/evidence/text",
                    base=base,
                    json_body={
                        "title": f"[{ev['id']}] {ev['title']}",
                        "text": text,
                        "source": ev.get("source"),
                    },
                    timeout=TIMEOUT_INJECT,
                )
                ev_id = result.get("id", "?")
                _ok(f"{label} (id={str(ev_id)[:8]}..., {len(text)} chars)")
                injected += 1
            except APIError as e:
                _fail(f"{label}: {e}")
                errors += 1

        # -- Trigger analysis after this wave --
        analysis_status = "skipped"
        if not no_analyze:
            _sub(f"Analysis after wave {wave_num}")
            try:
                run_resp = _api(
                    "POST",
                    f"/api/cases/{case_id}/analyze",
                    base=base,
                    json_body={"trigger": "manual"},
                    timeout=TIMEOUT_INJECT,
                )
                run_id = run_resp.get("run_id", "")
                _info(f"Analysis started (run_id={str(run_id)[:8]}...)")

                # Poll for completion
                for tick in range(timeout_analyze // POLL_INTERVAL):
                    time.sleep(POLL_INTERVAL)
                    run_status = _api_safe(
                        "GET", f"/api/analysis/{run_id}", base=base,
                    )
                    status_val = run_status.get("status", "?") if run_status else "?"
                    if status_val not in ("running", "pending"):
                        analysis_status = status_val
                        break
                    if tick % 3 == 0:
                        _info(f"  ... analysis running ({(tick + 1) * POLL_INTERVAL}s)")
                else:
                    analysis_status = "timeout"

                if analysis_status == "completed":
                    _ok(f"Analysis completed")
                else:
                    _warn(f"Analysis ended: {analysis_status}")

            except APIError as e:
                _warn(f"Analysis error: {e}")
                analysis_status = "error"

            # Generate hypotheses if none exist
            hypotheses = fetch_hypotheses(base, case_id)
            if not hypotheses:
                _info("No hypotheses yet -- triggering generation...")
                _api_safe(
                    "POST", f"/api/cases/{case_id}/hypotheses/generate",
                    base=base, timeout=timeout_analyze,
                )
                time.sleep(30)
                hypotheses = fetch_hypotheses(base, case_id)
                _info(f"{len(hypotheses)} hypotheses generated")

            # Re-evaluate all hypotheses
            if hypotheses:
                _info(f"Re-evaluating {len(hypotheses)} hypotheses...")
                _api_safe(
                    "POST", f"/api/cases/{case_id}/evaluate-all",
                    base=base, timeout=timeout_analyze,
                )
                time.sleep(15)

        # -- Collect wave stats --
        wave_elapsed = time.time() - wave_start
        stats = fetch_stats(base, case_id)
        entities = fetch_entities(base, case_id)
        hypotheses = fetch_hypotheses(base, case_id)

        _sub(f"Stats after wave {wave_num}")
        _info(f"Evidence: {stats.get('evidence', 0)}")
        _info(f"Entities: {stats.get('entities', 0)} ({len([e for e in entities if e.get('entity_type') == 'person'])} persons)")
        _info(f"Hypotheses: {len(hypotheses)}")
        for h in hypotheses:
            score = h.get("current_score", 0)
            _info(f"  {score:5.1f}% | {h.get('title', '?')}")
        _info(f"Wave completed in {wave_elapsed:.1f}s")

        wave_results.append({
            "wave": wave_num,
            "name": wave_meta["name"],
            "injected": injected,
            "errors": errors,
            "analysis_status": analysis_status,
            "duration_sec": round(wave_elapsed, 1),
            "stats": stats,
            "entity_count": len(entities),
            "hypothesis_count": len(hypotheses),
            "hypotheses_summary": [
                {"title": h.get("title", "?"), "score": h.get("current_score", 0)}
                for h in hypotheses
            ],
        })

    # -- Final scoring --
    _header(f"SCORING: {case_cfg['display_name']}")

    entities = fetch_entities(base, case_id)
    hypotheses = fetch_hypotheses(base, case_id)
    contradictions = fetch_contradictions(base, case_id)

    scorer = CaseScorer(manifest)

    # 1. Entities /20
    _sub("Entities (/20)")
    pts_ent = scorer.score_entities(entities)
    ent_detail = scorer.details["entities"]
    for f in ent_detail.get("found", []):
        _ok(f)
    for m in ent_detail.get("missing", []):
        _fail(m)
    _score_line("Entities", pts_ent, 20)

    # 2. Hypothesis ranking /20
    _sub("Hypothesis ranking (/20)")
    pts_rank = scorer.score_hypothesis_ranking(hypotheses)
    rank_detail = scorer.details["hypothesis_ranking"]
    if "matched" in rank_detail:
        _info(f"Target: {rank_detail.get('target', '?')}")
        _info(f"Matched: {rank_detail.get('matched', '?')}")
        _info(f"Rank: #{rank_detail.get('rank', '?')} (need top {rank_detail.get('max_rank', 3)})")
    elif "reason" in rank_detail:
        _warn(rank_detail["reason"])
    _score_line("Hypothesis ranking", pts_rank, 20)

    # 3. Contradictions /20
    _sub("Contradictions (/20)")
    pts_contra = scorer.score_contradictions(contradictions)
    contra_detail = scorer.details["contradictions"]
    _info(f"Detected: {contra_detail.get('total_detected', 0)} total")
    for f in contra_detail.get("found", []):
        _ok(f"Contradiction {f} found")
    for m in contra_detail.get("missing", []):
        _fail(f"Contradiction {m} missing")
    _score_line("Contradictions", pts_contra, 20)

    # 4. Hypothesis score /20
    _sub("Hypothesis score (/20)")
    pts_val = scorer.score_hypothesis_value(hypotheses)
    val_detail = scorer.details["hypothesis_score"]
    if "actual_score" in val_detail:
        _info(f"Score: {val_detail['actual_score']:.1f}% (need >= {val_detail['minimum_required']}%)")
        if val_detail.get("above_minimum"):
            _ok("Above minimum threshold")
        else:
            _warn("Below minimum threshold")
    _score_line("Hypothesis score", pts_val, 20)

    # 5. Timeline + geo /20
    _sub("Timeline + Geography (/20)")
    pts_geo = scorer.score_timeline_geo(entities, hypotheses, contradictions, all_evidence_text)
    geo_detail = scorer.details["timeline_geo"]
    for f in geo_detail.get("events_found", []):
        _ok(f"Event: {f}")
    for m in geo_detail.get("events_missing", []):
        _fail(f"Event: {m}")
    for f in geo_detail.get("geo_found", []):
        _ok(f"Geo: {f}")
    for m in geo_detail.get("geo_missing", []):
        _fail(f"Geo: {m}")
    _score_line("Timeline + Geo", pts_geo, 20)

    # -- Total --
    total = scorer.get_total()
    _header(f"TOTAL: {total:.0f}/100 -- {case_cfg['display_name']}")
    color = "green" if total >= 70 else ("yellow" if total >= 40 else "red")
    print(f"\n  {_c(f'{total:.0f}/100', color, 'bold')}\n")

    return {
        "case": case_key,
        "case_name": case_cfg["display_name"],
        "case_id": case_id,
        "timestamp": datetime.now().isoformat(),
        "total_score": total,
        "max_score": 100,
        "scoring": scorer.get_report(),
        "waves": wave_results,
        "summary": {
            "entities": len(entities),
            "hypotheses": len(hypotheses),
            "contradictions": len(contradictions),
        },
    }


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def generate_markdown_report(results: List[Dict]) -> str:
    """Generate the markdown benchmark report."""
    lines = [
        "# NEXUS Benchmark -- Real Cold Cases",
        f"\nDate: {datetime.now().strftime('%Y-%m-%d %H:%M')}",
        f"Cases tested: {len(results)}",
        "",
    ]

    overall_total = sum(r.get("total_score", 0) for r in results)
    overall_max = sum(r.get("max_score", 100) for r in results)
    lines.append(f"## Overall: {overall_total:.0f}/{overall_max}")
    lines.append("")

    for result in results:
        if "error" in result:
            lines.append(f"### {result.get('case', '?')} -- ERROR")
            lines.append(f"```\n{result['error']}\n```\n")
            continue

        name = result["case_name"]
        total = result["total_score"]
        scoring = result.get("scoring", {})
        categories = scoring.get("categories", {})

        status = "PASS" if total >= 70 else ("PARTIAL" if total >= 40 else "FAIL")
        lines.append(f"### {name} -- {total:.0f}/100 ({status})")
        lines.append("")

        # Category breakdown
        lines.append("| Category | Score | Details |")
        lines.append("|----------|-------|---------|")

        for cat_key, cat_label in [
            ("entities", "Entities"),
            ("hypothesis_ranking", "Hypothesis ranking"),
            ("contradictions", "Contradictions"),
            ("hypothesis_score", "Hypothesis score"),
            ("timeline_geo", "Timeline + Geo"),
        ]:
            cat = categories.get(cat_key, {})
            pts = cat.get("points", 0)
            mx = cat.get("max", 20)

            # Build details string
            details_parts = []
            if "found" in cat and "missing" in cat:
                details_parts.append(f"{len(cat['found'])} found, {len(cat['missing'])} missing")
            if "rank" in cat:
                details_parts.append(f"rank #{cat['rank']}")
            if "actual_score" in cat:
                details_parts.append(f"score={cat['actual_score']:.1f}%")
            if "reason" in cat:
                details_parts.append(cat["reason"])

            detail_str = "; ".join(details_parts) if details_parts else "-"
            lines.append(f"| {cat_label} | {pts:.0f}/{mx:.0f} | {detail_str} |")

        lines.append("")

        # Waves summary
        waves = result.get("waves", [])
        if waves:
            lines.append("**Waves:**")
            for w in waves:
                lines.append(
                    f"- Wave {w['wave']} ({w['name']}): "
                    f"{w['injected']} injected, "
                    f"analysis={w['analysis_status']}, "
                    f"{w['duration_sec']}s"
                )
            lines.append("")

        # Hypotheses at end
        last_wave = waves[-1] if waves else {}
        hyps = last_wave.get("hypotheses_summary", [])
        if hyps:
            lines.append("**Final hypotheses:**")
            for h in sorted(hyps, key=lambda x: x.get("score", 0), reverse=True):
                lines.append(f"- {h['score']:.0f}% | {h['title']}")
            lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="NEXUS Benchmark -- Real Cold Cases (Kulik + Golden State Killer)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--api-url", default=BASE_URL,
        help="NEXUS API base URL (default: http://localhost:8000)",
    )
    parser.add_argument(
        "--case", choices=["kulik", "gsk", "all"], default="all",
        help="Which case to run (default: all)",
    )
    parser.add_argument(
        "--no-analyze", action="store_true",
        help="Skip LLM analysis between waves (inject-only mode)",
    )
    parser.add_argument(
        "--timeout", type=int, default=TIMEOUT_ANALYZE,
        help=f"Analysis timeout in seconds (default: {TIMEOUT_ANALYZE})",
    )

    args = parser.parse_args()
    base = args.api_url

    # Enable ANSI on Windows
    _enable_ansi_windows()

    _header("NEXUS Benchmark -- Real Cold Cases")

    # Health check
    _sub("API Health Check")
    try:
        health = _api("GET", "/api/health", base=base)
        _ok(f"API accessible ({base})")
    except APIError as e:
        _fail(str(e))
        return 1

    # Determine which cases to run
    if args.case == "all":
        case_keys = ["kulik", "gsk"]
    else:
        case_keys = [args.case]

    _info(f"Cases to run: {', '.join(case_keys)}")

    # Run benchmarks
    all_results = []
    for case_key in case_keys:
        result = run_case(
            case_key,
            base=base,
            no_analyze=args.no_analyze,
            timeout_analyze=args.timeout,
        )
        all_results.append(result)

    # -- Generate reports --
    _header("GENERATING REPORTS")

    # JSON report
    json_path = OUTPUT_DIR / "BENCHMARK-REAL-CASES.json"
    report_json = {
        "benchmark": "NEXUS Real Cold Cases Benchmark",
        "timestamp": datetime.now().isoformat(),
        "cases": all_results,
        "overall_score": sum(r.get("total_score", 0) for r in all_results),
        "overall_max": sum(r.get("max_score", 100) for r in all_results),
    }
    json_path.write_text(json.dumps(report_json, ensure_ascii=False, indent=2), encoding="utf-8")
    _ok(f"JSON report: {json_path}")

    # Markdown report
    md_path = OUTPUT_DIR / "BENCHMARK-REAL-CASES.md"
    md_content = generate_markdown_report(all_results)
    md_path.write_text(md_content, encoding="utf-8")
    _ok(f"Markdown report: {md_path}")

    # Final summary
    _header("BENCHMARK COMPLETE")
    total = sum(r.get("total_score", 0) for r in all_results)
    max_total = sum(r.get("max_score", 100) for r in all_results)
    for r in all_results:
        name = r.get("case_name", r.get("case", "?"))
        score = r.get("total_score", 0)
        color = "green" if score >= 70 else ("yellow" if score >= 40 else "red")
        print(f"  {_c(f'{score:.0f}/100', color, 'bold')}  {name}")

    print(f"\n  {_c('TOTAL:', 'bold')} {_c(f'{total:.0f}/{max_total}', 'bold')}")
    _info(f"Reports: {md_path}")
    _info(f"JSON:    {json_path}")

    return 0


# ---------------------------------------------------------------------------
# Windows ANSI support
# ---------------------------------------------------------------------------

def _enable_ansi_windows() -> None:
    if sys.platform != "win32":
        return
    try:
        import ctypes
        kernel32 = ctypes.windll.kernel32  # type: ignore[attr-defined]
        handle = kernel32.GetStdHandle(-11)
        mode = ctypes.c_ulong()
        kernel32.GetConsoleMode(handle, ctypes.byref(mode))
        kernel32.SetConsoleMode(handle, mode.value | 0x0004)
    except Exception:
        pass


if __name__ == "__main__":
    sys.exit(main())
