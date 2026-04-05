#!/usr/bin/env python
"""
NEXUS Benchmark -- Affaire Kulik (progression temporelle)

Avance progressivement la date de recherche pour mesurer
quand le systeme resout l'affaire.

La verite :
- Auteurs : Gregory Wiart (ADN, mort 2003) + Willy Bardon (voix, condamne 30 ans)
- Methode resolution : ADN de parentele (2012) + reconnaissance vocale
"""

import json
import time
import sys
import requests
from datetime import datetime

BASE = "http://localhost:8000"

PHASES = [
    {
        "name": "Phase 1 — Post-crime immediat",
        "before": "2005-01-01",
        "description": "Articles 2002-2004. Le crime vient d'avoir lieu.",
    },
    {
        "name": "Phase 2 — Enquete stagne",
        "before": "2008-01-01",
        "description": "Articles 2002-2007. L'enquete pietine, 5000 ADN sans resultat.",
    },
    {
        "name": "Phase 3 — Avant percee ADN",
        "before": "2012-01-01",
        "description": "Articles 2002-2011. Juste avant l'identification par ADN de parentele.",
    },
    {
        "name": "Phase 4 — Apres identification Wiart",
        "before": "2015-01-01",
        "description": "Articles 2002-2014. Wiart identifie par ADN de parentele, mort en 2003.",
    },
    {
        "name": "Phase 5 — Proces Bardon",
        "before": "2022-01-01",
        "description": "Articles 2002-2021. Bardon identifie, juge, condamne.",
    },
    {
        "name": "Phase 6 — Toutes sources",
        "before": "2026-12-31",
        "description": "Toutes les sources disponibles. Post-condamnation.",
    },
]

TRUTH = {
    "perpetrators": ["Gregory Wiart", "Willy Bardon"],
    "method": "ADN de parentele + reconnaissance vocale",
    "verdict": "Bardon condamne a 30 ans de reclusion criminelle",
}

QUERIES = [
    "Elodie Kulik disparition meurtre Peronne 2002",
    "meurtre Cartigny Somme janvier 2002 enquete",
    "Elodie Kulik ADN enquete suspects",
    "agression mortelle femme Somme Picardie 2002",
    "Elodie Kulik appel pompiers enregistrement",
]


def log(msg: str) -> None:
    """Print with flush for real-time visibility."""
    print(msg, flush=True)


def api(method: str, path: str, **kwargs):
    """Call the NEXUS API. Returns parsed JSON or None on error."""
    kwargs.setdefault("timeout", 300)
    try:
        r = getattr(requests, method)(f"{BASE}{path}", **kwargs)
        if r.status_code >= 400:
            log(f"  [API ERROR] {method.upper()} {path} -> {r.status_code}: {r.text[:200]}")
            return None
        # Some endpoints return 204 with no body
        if r.status_code == 204 or not r.text.strip():
            return {"status": "ok"}
        return r.json()
    except requests.exceptions.Timeout:
        log(f"  [TIMEOUT] {method.upper()} {path}")
        return None
    except Exception as e:
        log(f"  [ERROR] {method.upper()} {path}: {e}")
        return None


def check_truth(hypotheses: list, entities: list) -> dict:
    """Check if the system found the truth."""
    found = {"wiart": False, "bardon": False, "adn_parentele": False}

    # Build searchable text from hypotheses
    all_text = " ".join(
        [
            h.get("title", "") + " " + h.get("description", "")
            for h in hypotheses
        ]
    ).lower()

    # Build searchable text from entity names + descriptions
    all_entity_names = [e.get("name", "").lower() for e in entities]
    all_entity_text = " ".join(
        [e.get("name", "") + " " + (e.get("description", "") or "") for e in entities]
    ).lower()

    combined = all_text + " " + all_entity_text

    if any("wiart" in n for n in all_entity_names) or "wiart" in combined:
        found["wiart"] = True
    if any("bardon" in n for n in all_entity_names) or "bardon" in combined:
        found["bardon"] = True
    if "parentele" in combined or "parentèle" in combined or "familial" in combined:
        found["adn_parentele"] = True

    return found


def run():
    results = []

    # ---------------------------------------------------------------
    # 1. Health check
    # ---------------------------------------------------------------
    log("Verification de l'API...")
    health = api("get", "/api/health")
    if not health:
        log("ERREUR: API non disponible sur http://localhost:8000")
        sys.exit(1)
    log(f"API OK: {health}")

    # ---------------------------------------------------------------
    # 2. Get or create case
    # ---------------------------------------------------------------
    cases = api("get", "/api/cases") or []
    case = None
    for c in cases:
        if "Kulik" in c.get("name", ""):
            case = c
            log(f"Case Kulik existant trouve: {c['id']}")
            break

    if not case:
        log("Creation du case Kulik...")
        case = api(
            "post",
            "/api/cases",
            json={
                "name": "Affaire Elodie Kulik (benchmark progressif)",
                "reference": "#2002-KULIK-BENCH",
                "description": "Meurtre Elodie Kulik janv 2002 Peronne Somme. before:2005-01-01",
            },
        )
        if not case:
            log("ERREUR: Impossible de creer le case")
            sys.exit(1)

    cid = case["id"]
    log(f"Case ID: {cid}")

    # ---------------------------------------------------------------
    # 3. Seed initial evidence if empty
    # ---------------------------------------------------------------
    stats = api("get", f"/api/cases/{cid}/stats")
    if stats and stats.get("evidence", 0) == 0:
        log("Injection de l'evidence initiale...")
        api(
            "post",
            f"/api/cases/{cid}/evidence/text",
            json={
                "title": "Rapport initial - disparition Elodie Kulik jan 2002",
                "text": (
                    "Le 11 janvier 2002 vers 0h30, appel au 18. "
                    "Voix feminine en detresse + voix masculines. "
                    "Vehicule VW Polo calcine a Cartigny. "
                    "Corps Elodie Kulik 24 ans retrouve a proximite. "
                    "Strangulation. ADN masculin (sperme) sans correspondance FNAEG. "
                    "Victime directrice CIC Peronne. "
                    "Soiree chez amis a Ham, departie vers minuit. "
                    "Crime sexuel en reunion probable."
                ),
                "source": "SRPJ Amiens jan 2002",
            },
        )
        log("Evidence initiale injectee")
    else:
        log(f"Evidence existante: {stats.get('evidence', 0)} items")

    # ---------------------------------------------------------------
    # 4. Run phases
    # ---------------------------------------------------------------
    for phase_idx, phase in enumerate(PHASES):
        log(f"\n{'=' * 60}")
        log(f"  {phase['name']}")
        log(f"  Recherche limitee aux articles avant {phase['before']}")
        log(f"{'=' * 60}")

        # 4a. Update case description with new date filter
        api(
            "put",
            f"/api/cases/{cid}",
            json={
                "description": (
                    f"Meurtre Elodie Kulik janv 2002. "
                    f"{phase['description']} before:{phase['before']}"
                )
            },
        )
        log(f"  Description mise a jour (before:{phase['before']})")

        # 4b. Create monitoring jobs if they don't exist yet
        existing_jobs = api("get", f"/api/cases/{cid}/monitoring") or []
        existing_queries = {j.get("query", "") for j in existing_jobs}

        new_jobs_created = 0
        for q in QUERIES:
            query_with_date = f"{q} before:{phase['before']}"
            if query_with_date not in existing_queries and q not in existing_queries:
                result = api(
                    "post",
                    f"/api/cases/{cid}/monitoring",
                    json={
                        "case_id": cid,
                        "query": query_with_date,
                        "job_type": "searxng",
                        "interval_hours": 24,
                    },
                )
                if result:
                    new_jobs_created += 1
        log(f"  {new_jobs_created} nouveaux jobs monitoring crees")

        # 4c. Force execute all monitoring jobs
        jobs = api("get", f"/api/cases/{cid}/monitoring") or []
        log(f"  Execution de {len(jobs)} jobs monitoring...")
        triggered = 0
        for j in jobs:
            resp = api("post", f"/api/monitoring/{j['id']}/run")
            if resp:
                triggered += 1
        log(f"  {triggered}/{len(jobs)} jobs declenches")

        # 4d. Wait for monitoring results to come in
        log("  Attente des resultats SearXNG (45s)...")
        time.sleep(45)

        # 4e. Check monitoring results
        mon_results = api("get", f"/api/cases/{cid}/monitoring/results") or []
        new_results = [
            r
            for r in mon_results
            if not r.get("reviewed") and not r.get("is_duplicate")
        ]
        log(f"  {len(mon_results)} resultats totaux, {len(new_results)} nouveaux")

        # 4f. Auto-ingest top new results as evidence
        ingested = 0
        for r in new_results[:20]:  # max 20 per phase
            ingest_resp = api("post", f"/api/monitoring/results/{r['id']}/ingest")
            if ingest_resp:
                ingested += 1
        if ingested:
            log(f"  {ingested} resultats ingeres comme evidence")

        # 4g. Launch full analysis
        log("  Lancement analyse complete...")
        run_resp = api(
            "post",
            f"/api/cases/{cid}/analyze",
            json={"trigger": "manual"},
        )

        run_id = None
        run_status = None
        if run_resp:
            run_id = run_resp.get("run_id", "")
            log(f"  Analysis run_id: {run_id}")

            # Poll for completion (max 10 min)
            for tick in range(60):
                time.sleep(10)
                run_status = api("get", f"/api/analysis/{run_id}")
                status_val = run_status.get("status", "?") if run_status else "?"
                if status_val != "running":
                    break
                if tick % 3 == 0:
                    log(f"    ... analyse en cours ({(tick + 1) * 10}s)")

            final_status = run_status.get("status", "timeout") if run_status else "timeout"
            log(f"  Analyse terminee: {final_status}")
        else:
            log("  ERREUR: Impossible de lancer l'analyse")

        # 4h. Generate hypotheses explicitly if none exist
        hypotheses = api("get", f"/api/cases/{cid}/hypotheses") or []
        if len(hypotheses) == 0:
            log("  Aucune hypothese -- lancement generation...")
            gen_resp = api("post", f"/api/cases/{cid}/hypotheses/generate")
            if gen_resp:
                log("  Generation d'hypotheses lancee, attente 60s...")
                time.sleep(60)
                hypotheses = api("get", f"/api/cases/{cid}/hypotheses") or []
                log(f"  {len(hypotheses)} hypotheses generees")

        # 4i. Re-evaluate all hypotheses
        if hypotheses:
            log(f"  Re-evaluation de {len(hypotheses)} hypotheses...")
            api("post", f"/api/cases/{cid}/evaluate-all")
            time.sleep(30)
            # Refresh
            hypotheses = api("get", f"/api/cases/{cid}/hypotheses") or []

        # 4j. Collect results
        stats = api("get", f"/api/cases/{cid}/stats") or {}
        entities = api("get", f"/api/cases/{cid}/entities") or []

        truth_check = check_truth(hypotheses, entities)

        phase_result = {
            "phase": phase["name"],
            "before_date": phase["before"],
            "evidence_count": stats.get("evidence", 0),
            "entity_count": stats.get("entities", 0),
            "hypothesis_count": len(hypotheses),
            "monitoring_results": len(mon_results),
            "new_results_ingested": ingested,
            "truth_found": truth_check,
            "hypotheses": [
                {
                    "title": h.get("title", "?"),
                    "score": h.get("current_score", 0),
                    "status": h.get("status", "?"),
                }
                for h in hypotheses
            ],
            "top_entities": [
                e["name"]
                for e in entities
                if e.get("entity_type") == "person"
            ][:15],
        }
        results.append(phase_result)

        # 4k. Print phase summary
        log(f"\n  --- Resultats Phase ---")
        log(f"  Preuves: {stats.get('evidence', 0)}")
        log(f"  Entites: {stats.get('entities', 0)}")
        log(f"  Hypotheses: {len(hypotheses)}")
        for h in hypotheses:
            score = h.get("current_score", 0)
            title = h.get("title", "?")
            log(f"    {score:5.1f}% | {title}")
        log(
            f"  Verite trouvee: "
            f"Wiart={'OUI' if truth_check['wiart'] else 'non'} "
            f"Bardon={'OUI' if truth_check['bardon'] else 'non'} "
            f"ADN={'OUI' if truth_check['adn_parentele'] else 'non'}"
        )
        log(f"  Personnes detectees: {', '.join(phase_result['top_entities'][:10])}")

        if truth_check["wiart"] and truth_check["bardon"]:
            log(f"\n  >>> AFFAIRE RESOLUE a {phase['name']} <<<")
            break

        # 4l. Clean up monitoring jobs for next phase (update queries with new dates)
        if phase_idx < len(PHASES) - 1:
            # Delete old jobs so next phase creates new ones with updated date
            for j in jobs:
                api("delete", f"/api/monitoring/{j['id']}")
            log(f"  {len(jobs)} anciens jobs supprimes pour prochaine phase")

    # ---------------------------------------------------------------
    # 5. Write results
    # ---------------------------------------------------------------
    report = {
        "benchmark": "Affaire Elodie Kulik -- Progression temporelle",
        "timestamp": datetime.now().isoformat(),
        "case_id": cid,
        "truth": TRUTH,
        "phases": results,
    }

    with open("docs/BENCHMARK-KULIK-RESULTS.json", "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    # Write markdown report
    md = [
        "# Benchmark NEXUS -- Affaire Elodie Kulik",
        f"\nDate: {datetime.now().strftime('%Y-%m-%d %H:%M')}",
        f"\nVerite: {TRUTH['perpetrators'][0]} + {TRUTH['perpetrators'][1]}",
        f"Methode: {TRUTH['method']}",
        "\n## Resultats par phase\n",
    ]
    for r in results:
        resolved = (
            "RESOLU"
            if r["truth_found"]["wiart"] and r["truth_found"]["bardon"]
            else "non resolu"
        )
        md.append(f"### {r['phase']} (avant {r['before_date']})")
        md.append(f"- Preuves: {r['evidence_count']}")
        md.append(f"- Entites: {r['entity_count']}")
        md.append(f"- Resultats monitoring: {r['monitoring_results']}")
        md.append(f"- Resultats ingeres: {r['new_results_ingested']}")
        md.append(f"- Hypotheses: {r['hypothesis_count']}")
        for h in r.get("hypotheses", []):
            md.append(f"  - {h['score']:.0f}% | {h['title']}")
        md.append(
            f"- Wiart trouve: {'OUI' if r['truth_found']['wiart'] else 'non'}"
        )
        md.append(
            f"- Bardon trouve: {'OUI' if r['truth_found']['bardon'] else 'non'}"
        )
        md.append(
            f"- ADN parentele: {'OUI' if r['truth_found']['adn_parentele'] else 'non'}"
        )
        md.append(f"- Personnes detectees: {', '.join(r['top_entities'][:10])}")
        md.append(f"- **Status: {resolved}**\n")

    with open("docs/BENCHMARK-KULIK-RESULTS.md", "w", encoding="utf-8") as f:
        f.write("\n".join(md))

    log(f"\n{'=' * 60}")
    log(f"  BENCHMARK TERMINE")
    log(f"  Resultats: docs/BENCHMARK-KULIK-RESULTS.md")
    log(f"  JSON: docs/BENCHMARK-KULIK-RESULTS.json")
    log(f"{'=' * 60}")


if __name__ == "__main__":
    run()
