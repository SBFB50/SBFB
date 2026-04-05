#!/usr/bin/env python
"""NEXUS Benchmark -- Affaire MOREAU

Injects the benchmark cold case scenario through the FastAPI API,
wave by wave, triggering analysis between waves.

Usage:
    python tests/run_benchmark.py [--api-url http://localhost:8000] [--wave N]
    python tests/run_benchmark.py --no-analyze          # skip LLM analysis
    python tests/run_benchmark.py --wave 3              # inject only wave 3
    python tests/run_benchmark.py --pause 30            # 30s between waves
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests

# ---------------------------------------------------------------------------
# ANSI color helpers (no dependency on colorama)
# ---------------------------------------------------------------------------

_COLORS = {
    "reset": "\033[0m",
    "bold": "\033[1m",
    "dim": "\033[2m",
    "red": "\033[91m",
    "green": "\033[92m",
    "yellow": "\033[93m",
    "blue": "\033[94m",
    "magenta": "\033[95m",
    "cyan": "\033[96m",
    "white": "\033[97m",
    "bg_red": "\033[41m",
    "bg_green": "\033[42m",
    "bg_yellow": "\033[43m",
    "bg_blue": "\033[44m",
}


def _c(text: str, *styles: str) -> str:
    """Wrap *text* in ANSI escape codes."""
    prefix = "".join(_COLORS.get(s, "") for s in styles)
    return f"{prefix}{text}{_COLORS['reset']}"


def _header(msg: str) -> None:
    width = 64
    border = "=" * width
    print(f"\n{_c(border, 'cyan', 'bold')}")
    print(_c(f"  {msg}", "cyan", "bold"))
    print(f"{_c(border, 'cyan', 'bold')}")


def _subheader(msg: str) -> None:
    print(f"\n{_c('--- ' + msg + ' ---', 'blue', 'bold')}")


def _ok(msg: str) -> None:
    print(f"  {_c('[OK]', 'green', 'bold')} {msg}")


def _fail(msg: str) -> None:
    print(f"  {_c('[FAIL]', 'red', 'bold')} {msg}")


def _warn(msg: str) -> None:
    print(f"  {_c('[WARN]', 'yellow', 'bold')} {msg}")


def _info(msg: str) -> None:
    print(f"  {_c('[i]', 'blue')} {msg}")


def _progress(current: int, total: int, label: str) -> None:
    bar_len = 30
    filled = int(bar_len * current / total) if total > 0 else 0
    bar = _c("#" * filled, "green") + _c("-" * (bar_len - filled), "dim")
    pct = f"{100 * current / total:.0f}%" if total > 0 else "0%"
    print(f"  [{bar}] {pct}  {label}", flush=True)


# ---------------------------------------------------------------------------
# API helpers
# ---------------------------------------------------------------------------

BASE_URL = "http://localhost:8000"
BENCHMARK_DIR = Path(__file__).resolve().parent.parent / "data" / "benchmark" / "affaire-moreau"

# Generous timeout for LLM-powered endpoints
TIMEOUT_SHORT = 30  # seconds -- CRUD operations
TIMEOUT_LONG = 600  # seconds -- analysis (LLM work)


class APIError(Exception):
    """Raised on non-2xx responses or connection failures."""


def _api(
    method: str,
    path: str,
    *,
    base: str = BASE_URL,
    json_body: Any = None,
    params: Optional[Dict] = None,
    timeout: int = TIMEOUT_SHORT,
) -> Any:
    """Make an HTTP request and return parsed JSON (or None for 204)."""
    url = f"{base.rstrip('/')}{path}"
    try:
        resp = requests.request(
            method,
            url,
            json=json_body,
            params=params,
            timeout=timeout,
        )
    except requests.ConnectionError:
        raise APIError(
            f"Connexion impossible a {base}. Le backend NEXUS est-il demarre?"
        )
    except requests.Timeout:
        raise APIError(f"Timeout ({timeout}s) pour {method} {path}")

    if resp.status_code == 204:
        return None
    if resp.status_code >= 400:
        detail = ""
        try:
            detail = resp.json().get("detail", resp.text)
        except Exception:
            detail = resp.text
        raise APIError(f"HTTP {resp.status_code} sur {method} {path}: {detail}")

    return resp.json()


# ---------------------------------------------------------------------------
# Core operations
# ---------------------------------------------------------------------------

def create_case(base: str, manifest: Dict) -> str:
    """Create the benchmark case and return its ID."""
    case_def = manifest["case"]
    data = _api(
        "POST",
        "/api/cases",
        base=base,
        json_body={
            "name": case_def["name"],
            "reference": case_def.get("reference"),
            "description": case_def.get("description"),
        },
    )
    return data["id"]


def inject_evidence(
    base: str,
    case_id: str,
    ev_meta: Dict,
    text: str,
) -> Dict:
    """Submit one text evidence item and return the API response."""
    return _api(
        "POST",
        f"/api/cases/{case_id}/evidence/text",
        base=base,
        json_body={
            "title": f"[{ev_meta['id']}] {ev_meta['title']}",
            "text": text,
            "source": ev_meta.get("source"),
        },
    )


def trigger_analysis(base: str, case_id: str) -> str:
    """Trigger a full analysis and return the run_id."""
    data = _api(
        "POST",
        f"/api/cases/{case_id}/analyze",
        base=base,
        timeout=TIMEOUT_SHORT,
    )
    return data["run_id"]


def poll_analysis(base: str, run_id: str, poll_interval: int = 10) -> Dict:
    """Poll an analysis run until it completes or fails.

    Returns the final run record.
    """
    spinner = ["|", "/", "-", "\\"]
    idx = 0
    while True:
        run = _api("GET", f"/api/analysis/{run_id}", base=base)
        status = run.get("status", "unknown")
        sym = spinner[idx % len(spinner)]
        print(
            f"\r  {_c(sym, 'yellow')} Analyse en cours (status={status})...  ",
            end="",
            flush=True,
        )
        if status in ("completed", "failed"):
            print()  # newline after spinner
            return run
        time.sleep(poll_interval)
        idx += 1


def fetch_stats(base: str, case_id: str) -> Dict:
    """Fetch case statistics."""
    try:
        return _api("GET", f"/api/cases/{case_id}/stats", base=base)
    except APIError:
        return {}


def fetch_entities(base: str, case_id: str) -> List[Dict]:
    try:
        return _api("GET", f"/api/cases/{case_id}/entities", base=base) or []
    except APIError:
        return []


def fetch_hypotheses(base: str, case_id: str) -> List[Dict]:
    try:
        return _api("GET", f"/api/cases/{case_id}/hypotheses", base=base) or []
    except APIError:
        return []


def fetch_alerts(base: str, case_id: str) -> List[Dict]:
    try:
        return _api("GET", f"/api/cases/{case_id}/alerts", base=base) or []
    except APIError:
        return []


def fetch_contradictions(base: str, case_id: str) -> List[Dict]:
    try:
        return _api(
            "GET",
            f"/api/cases/{case_id}/contradictions",
            base=base,
            timeout=TIMEOUT_LONG,
        ) or []
    except APIError:
        return []


def fetch_evidence_list(base: str, case_id: str) -> List[Dict]:
    try:
        return _api("GET", f"/api/cases/{case_id}/evidence", base=base) or []
    except APIError:
        return []


# ---------------------------------------------------------------------------
# Display helpers
# ---------------------------------------------------------------------------

def print_wave_stats(
    base: str,
    case_id: str,
    wave_num: int,
) -> None:
    """Print live stats after a wave is complete."""
    _subheader(f"Stats apres vague {wave_num}")

    stats = fetch_stats(base, case_id)
    if stats:
        for key, val in stats.items():
            _info(f"{key}: {val}")
    else:
        _warn("Pas de stats disponibles")

    # Entities summary
    entities = fetch_entities(base, case_id)
    persons = [e for e in entities if e.get("entity_type") == "person"]
    locations = [e for e in entities if e.get("entity_type") == "location"]
    _info(
        f"Entites: {len(entities)} total, "
        f"{len(persons)} personnes, {len(locations)} lieux"
    )
    if persons:
        names = ", ".join(p["name"] for p in persons[:10])
        _info(f"  Personnes: {names}")

    # Hypotheses
    hypotheses = fetch_hypotheses(base, case_id)
    _info(f"Hypotheses: {len(hypotheses)}")
    for h in hypotheses:
        score = h.get("current_score", "?")
        status = h.get("status", "?")
        _info(
            f"  {_c(h['title'], 'bold')}: "
            f"score={_c(str(score), 'yellow')} status={status}"
        )

    # Alerts
    alerts = fetch_alerts(base, case_id)
    critical = [a for a in alerts if a.get("severity") == "critical"]
    warnings = [a for a in alerts if a.get("severity") == "warning"]
    _info(
        f"Alertes: {len(alerts)} total, "
        f"{_c(str(len(critical)), 'red')} critiques, "
        f"{_c(str(len(warnings)), 'yellow')} warnings"
    )
    for a in critical:
        print(f"    {_c('!!', 'red', 'bold')} {a.get('title', '?')}")


# ---------------------------------------------------------------------------
# Final report: compare results vs expectations
# ---------------------------------------------------------------------------

def print_final_report(
    base: str,
    case_id: str,
    manifest: Dict,
) -> None:
    """Compare obtained results with the manifest expectations."""
    _header("RAPPORT FINAL -- Affaire MOREAU")

    # ---- Evidence injected ----
    evidence_list = fetch_evidence_list(base, case_id)
    expected_count = len(manifest["evidence"])
    actual_count = len(evidence_list)
    _subheader("Preuves injectees")
    if actual_count == expected_count:
        _ok(f"{actual_count}/{expected_count} preuves injectees")
    else:
        _warn(f"{actual_count}/{expected_count} preuves injectees")

    # ---- Entities ----
    entities = fetch_entities(base, case_id)
    _subheader("Entites detectees")
    persons = [e for e in entities if e.get("entity_type") == "person"]
    _info(f"{len(entities)} entites ({len(persons)} personnes)")
    # Key persons the system should have found
    expected_persons = [
        "Elise Moreau",
        "Marc Duval",
        "Sophie Laurent",
        "Romain Fabre",
        "Karim Belhadj",
        "Julien Tessier",
        "Yann Chevalier",
    ]
    found_names = {p["name"].lower() for p in persons}
    # Also check aliases
    for p in persons:
        if p.get("aliases"):
            for alias in p["aliases"]:
                found_names.add(alias.lower())
    for expected in expected_persons:
        if any(expected.lower() in fn for fn in found_names):
            _ok(f"Personne trouvee: {expected}")
        else:
            _fail(f"Personne manquante: {expected}")

    # ---- Hypotheses vs expected ----
    hypotheses = fetch_hypotheses(base, case_id)
    _subheader("Hypotheses vs attendus")
    expected_hyps = manifest.get("expected_hypotheses", [])
    hyp_titles_lower = {h["title"].lower(): h for h in hypotheses}

    for exp_h in expected_hyps:
        lo, hi = exp_h["expected_final_score_range"]
        exp_title = exp_h["title"]
        exp_id = exp_h["id"]

        # Fuzzy match: search for keyword overlap
        matched = _match_hypothesis(exp_title, hypotheses)
        if matched:
            score = matched.get("current_score", 0)
            in_range = lo <= score <= hi
            status_str = (
                _c(f"score={score:.1f}", "green")
                if in_range
                else _c(f"score={score:.1f}", "red")
            )
            range_str = f"[{lo}-{hi}]"
            marker = _c("OK", "green", "bold") if in_range else _c("HORS RANGE", "red", "bold")
            print(
                f"  [{exp_id}] {exp_title}\n"
                f"        Obtenu: {status_str}  Attendu: {range_str}  {marker}\n"
                f"        Match: \"{matched['title']}\""
            )
        else:
            _fail(f"[{exp_id}] {exp_title} -- non trouvee dans les hypotheses generees")

    # ---- Contradictions vs expected ----
    _subheader("Contradictions vs attendues")
    contradictions = fetch_contradictions(base, case_id)
    expected_contradictions = manifest.get("expected_contradictions", [])

    _info(f"{len(contradictions)} contradiction(s) detectee(s) par le systeme")
    _info(f"{len(expected_contradictions)} contradiction(s) attendue(s)")

    found_count = 0
    for exp_c in expected_contradictions:
        cid = exp_c["id"]
        desc = exp_c["description"]
        matched = _match_contradiction(desc, contradictions)
        if matched:
            found_count += 1
            _ok(f"[{cid}] {desc[:80]}...")
        else:
            _fail(f"[{cid}] {desc[:80]}...")

    # ---- Summary score ----
    _subheader("Score global du benchmark")
    total_checks = 0
    passed_checks = 0

    # Evidence injection
    total_checks += 1
    if actual_count == expected_count:
        passed_checks += 1

    # Key persons
    for expected in expected_persons:
        total_checks += 1
        if any(expected.lower() in fn for fn in found_names):
            passed_checks += 1

    # Hypotheses in range
    for exp_h in expected_hyps:
        total_checks += 1
        matched = _match_hypothesis(exp_h["title"], hypotheses)
        if matched:
            lo, hi = exp_h["expected_final_score_range"]
            score = matched.get("current_score", 0)
            if lo <= score <= hi:
                passed_checks += 1

    # Contradictions found
    for exp_c in expected_contradictions:
        total_checks += 1
        if _match_contradiction(exp_c["description"], contradictions):
            passed_checks += 1

    pct = (100 * passed_checks / total_checks) if total_checks > 0 else 0
    color = "green" if pct >= 70 else ("yellow" if pct >= 40 else "red")
    print(
        f"\n  {_c('RESULTAT', 'bold')}: "
        f"{_c(f'{passed_checks}/{total_checks}', color, 'bold')} checks passes "
        f"({_c(f'{pct:.0f}%', color, 'bold')})"
    )

    # Alerts summary
    alerts = fetch_alerts(base, case_id)
    _info(f"Alertes totales generees: {len(alerts)}")


def _match_hypothesis(expected_title: str, hypotheses: List[Dict]) -> Optional[Dict]:
    """Fuzzy-match an expected hypothesis title against generated ones.

    Strategy: extract key name tokens from the expected title and
    find the hypothesis whose title contains the most of them.
    """
    # Extract significant keywords (names, roles)
    keywords = _extract_keywords(expected_title)
    if not keywords:
        return None

    best: Optional[Dict] = None
    best_score = 0
    for h in hypotheses:
        h_lower = h["title"].lower()
        h_desc = (h.get("description") or "").lower()
        combined = h_lower + " " + h_desc
        score = sum(1 for kw in keywords if kw in combined)
        if score > best_score:
            best_score = score
            best = h

    # Require at least 1 keyword match
    return best if best_score >= 1 else None


def _match_contradiction(
    expected_desc: str,
    contradictions: List[Dict],
) -> Optional[Dict]:
    """Fuzzy-match an expected contradiction against detected ones."""
    keywords = _extract_keywords(expected_desc)
    if not keywords:
        return None

    best: Optional[Dict] = None
    best_score = 0
    for c in contradictions:
        c_text = json.dumps(c, ensure_ascii=False).lower()
        score = sum(1 for kw in keywords if kw in c_text)
        if score > best_score:
            best_score = score
            best = c

    # Require at least 2 keyword matches for contradictions
    return best if best_score >= 2 else None


def _extract_keywords(text: str) -> List[str]:
    """Extract meaningful keywords (proper nouns, numbers, key terms)."""
    # Lowercase stopwords to ignore
    stopwords = {
        "de", "du", "des", "le", "la", "les", "un", "une", "et", "en", "a",
        "au", "aux", "ce", "cette", "par", "pour", "que", "qui", "sur",
        "son", "sa", "ses", "dit", "vs", "avec", "dans", "est", "pas",
        "plus", "vers", "il", "elle", "ils", "elles", "se", "ne", "ni",
        "ou", "mais", "donc", "car", "si", "non", "oui", "aucun",
        "--", "-", "auteur", "auteurs", "hypothese",
    }
    tokens = text.lower().replace("'", " ").replace("(", " ").replace(")", " ").split()
    return [t.strip(".,;:!?\"'") for t in tokens if t.strip(".,;:!?\"'") not in stopwords and len(t) > 2]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="NEXUS Benchmark -- Affaire MOREAU",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Exemples:\n"
            "  python tests/run_benchmark.py\n"
            "  python tests/run_benchmark.py --wave 2\n"
            "  python tests/run_benchmark.py --no-analyze --pause 5\n"
        ),
    )
    parser.add_argument(
        "--api-url",
        default=BASE_URL,
        help="URL de base de l'API NEXUS (defaut: http://localhost:8000)",
    )
    parser.add_argument(
        "--wave",
        type=int,
        default=0,
        help="Injecter uniquement cette vague (0 = toutes)",
    )
    parser.add_argument(
        "--no-analyze",
        action="store_true",
        help="Ne pas lancer d'analyse LLM entre les vagues",
    )
    parser.add_argument(
        "--pause",
        type=int,
        default=10,
        help="Pause en secondes entre chaque vague (defaut: 10)",
    )
    parser.add_argument(
        "--poll-interval",
        type=int,
        default=10,
        help="Intervalle de polling pour l'analyse en secondes (defaut: 10)",
    )
    args = parser.parse_args()

    base = args.api_url

    # Enable ANSI on Windows
    _enable_ansi_windows()

    _header("NEXUS Benchmark -- Affaire MOREAU")

    # ------------------------------------------------------------------
    # 1. Health check
    # ------------------------------------------------------------------
    _subheader("Verification de l'API")
    try:
        health = _api("GET", "/api/health", base=base)
        _ok(f"API accessible ({base})")
        if isinstance(health, dict):
            for k, v in health.items():
                _info(f"  {k}: {v}")
    except APIError as exc:
        _fail(str(exc))
        return 1

    # ------------------------------------------------------------------
    # 2. Load manifest
    # ------------------------------------------------------------------
    manifest_path = BENCHMARK_DIR / "manifest.json"
    if not manifest_path.exists():
        _fail(f"Manifest introuvable: {manifest_path}")
        return 1
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    _ok(f"Manifest charge: {len(manifest['evidence'])} preuves, 4 vagues")

    # ------------------------------------------------------------------
    # 3. Create case
    # ------------------------------------------------------------------
    _subheader("Creation du dossier")
    try:
        case_id = create_case(base, manifest)
        _ok(f"Dossier cree: {_c(case_id, 'cyan')}")
        _info(f"Nom: {manifest['case']['name']}")
        _info(f"Reference: {manifest['case'].get('reference', 'N/A')}")
    except APIError as exc:
        _fail(f"Impossible de creer le dossier: {exc}")
        return 1

    # ------------------------------------------------------------------
    # 4. Inject evidence wave by wave
    # ------------------------------------------------------------------
    waves_to_run = [1, 2, 3, 4]
    if args.wave > 0:
        if args.wave not in waves_to_run:
            _fail(f"Vague invalide: {args.wave}. Valeurs possibles: 1, 2, 3, 4")
            return 1
        waves_to_run = [args.wave]

    total_injected = 0
    total_errors = 0
    wave_timings: Dict[int, float] = {}

    for wave_num in waves_to_run:
        wave_meta = manifest["waves"][str(wave_num)]
        evidence_in_wave = [
            e for e in manifest["evidence"] if e["wave"] == wave_num
        ]

        _header(f"VAGUE {wave_num}: {wave_meta['name']}")
        _info(wave_meta["description"])
        _info(f"{len(evidence_in_wave)} preuve(s) a injecter")

        wave_start = time.time()

        # -- Inject each evidence item --
        for idx, ev in enumerate(evidence_in_wave, start=1):
            file_path = BENCHMARK_DIR / ev["file"]
            if not file_path.exists():
                _fail(f"Fichier manquant: {ev['file']}")
                total_errors += 1
                continue

            text = file_path.read_text(encoding="utf-8")
            label = f"[{ev['id']}] {ev['title']}"

            try:
                result = inject_evidence(base, case_id, ev, text)
                ev_id = result.get("id", "?")
                _ok(f"{label}  (id={ev_id[:8]}..., {len(text)} chars)")
                total_injected += 1
            except APIError as exc:
                _fail(f"{label}: {exc}")
                total_errors += 1

            _progress(idx, len(evidence_in_wave), f"Vague {wave_num}")

        # -- Trigger analysis --
        if not args.no_analyze:
            _subheader(f"Analyse apres vague {wave_num}")
            try:
                run_id = trigger_analysis(base, case_id)
                _info(f"Analyse lancee (run_id={run_id[:8]}...)")
                run = poll_analysis(base, run_id, poll_interval=args.poll_interval)
                status = run.get("status", "?")
                duration = run.get("duration_sec")
                if status == "completed":
                    dur_str = f" en {duration:.1f}s" if duration else ""
                    _ok(f"Analyse terminee{dur_str}")
                else:
                    _fail(f"Analyse echouee (status={status})")
                    summary = run.get("output_summary")
                    if summary:
                        _info(f"  Detail: {summary[:200]}")
            except APIError as exc:
                _fail(f"Erreur analyse: {exc}")

        # -- Print stats --
        print_wave_stats(base, case_id, wave_num)

        wave_elapsed = time.time() - wave_start
        wave_timings[wave_num] = wave_elapsed
        _info(f"Vague {wave_num} completee en {wave_elapsed:.1f}s")

        # -- Pause between waves --
        remaining_waves = [w for w in waves_to_run if w > wave_num]
        if remaining_waves and args.pause > 0:
            _info(f"Pause de {args.pause}s avant vague {remaining_waves[0]}...")
            for sec in range(args.pause, 0, -1):
                print(
                    f"\r  {_c(str(sec), 'yellow')}s restantes...  ",
                    end="",
                    flush=True,
                )
                time.sleep(1)
            print("\r" + " " * 40 + "\r", end="")

    # ------------------------------------------------------------------
    # 5. Final report
    # ------------------------------------------------------------------
    print_final_report(base, case_id, manifest)

    # Timing summary
    _subheader("Temps d'execution")
    total_time = sum(wave_timings.values())
    for wn, wt in wave_timings.items():
        _info(f"Vague {wn}: {wt:.1f}s")
    print(
        f"\n  {_c('Total:', 'bold')} {total_time:.1f}s  |  "
        f"{_c(str(total_injected), 'green')} injectees  |  "
        f"{_c(str(total_errors), 'red') if total_errors else '0'} erreurs"
    )

    _info(f"Case ID: {case_id}")
    _info(f"Dashboard: http://localhost:8501 (selectionner '{manifest['case']['name']}')")

    return 0 if total_errors == 0 else 1


# ---------------------------------------------------------------------------
# Windows ANSI support
# ---------------------------------------------------------------------------

def _enable_ansi_windows() -> None:
    """Enable ANSI escape sequences on Windows 10+ consoles."""
    if sys.platform != "win32":
        return
    try:
        import ctypes
        kernel32 = ctypes.windll.kernel32  # type: ignore[attr-defined]
        # STD_OUTPUT_HANDLE = -11
        handle = kernel32.GetStdHandle(-11)
        # ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004
        mode = ctypes.c_ulong()
        kernel32.GetConsoleMode(handle, ctypes.byref(mode))
        kernel32.SetConsoleMode(handle, mode.value | 0x0004)
    except Exception:
        pass  # Fallback: ANSI codes will be ignored or garbled


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    sys.exit(main())
