import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402
"""
NEXUS -- Page Benchmark.

Lance et visualise les benchmarks sur vrais cold cases resolus.
Score /100 par case: entites, hypotheses, contradictions, timeline, geo.
"""

import json
import time
import streamlit as st
import pandas as pd
import requests
from pathlib import Path
from datetime import datetime

API = "http://localhost:8000"
BENCH_DIR = Path(__file__).resolve().parent.parent.parent / "data" / "benchmark"

CASES = {
    "kulik": {"dir": "kulik", "name": "Affaire Elodie Kulik (2002, France)"},
    "gsk": {"dir": "golden-state-killer", "name": "Golden State Killer (1974-86, USA)"},
    "moreau": {"dir": "affaire-moreau", "name": "Affaire Moreau (fictif)"},
}


def api(method, path, **kwargs):
    kwargs.setdefault("timeout", 300)
    try:
        r = getattr(requests, method)(f"{API}{path}", **kwargs)
        if r.status_code < 400:
            return r.json()
        return {"error": r.status_code, "detail": r.text[:200]}
    except requests.Timeout:
        return {"error": "timeout"}
    except Exception as e:
        return {"error": str(e)}


# =====================================================================
st.title("Benchmark NEXUS")
st.caption("Evaluation sur vrais cold cases resolus — le systeme recoit les pieces d'enquete brutes et doit converger vers la verite.")

# Health check
health = api("get", "/api/health")
if not health or health.get("error"):
    st.error(f"API indisponible: {health}")
    st.stop()
st.success(f"API OK — v{health.get('version', '?')}")

st.markdown("---")

# =====================================================================
# Case selection
# =====================================================================

st.subheader("Selectionner un cold case")

available = {}
for key, info in CASES.items():
    manifest_path = BENCH_DIR / info["dir"] / "manifest.json"
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        n_evidence = len(manifest.get("evidence", []))
        n_waves = len(set(e.get("wave", 1) for e in manifest.get("evidence", [])))
        available[key] = {
            "name": info["name"],
            "manifest": manifest,
            "n_evidence": n_evidence,
            "n_waves": n_waves,
            "dir": BENCH_DIR / info["dir"],
        }

if not available:
    st.error("Aucun benchmark disponible dans data/benchmark/")
    st.stop()

cols = st.columns(len(available))
for i, (key, info) in enumerate(available.items()):
    with cols[i]:
        gt = info["manifest"].get("ground_truth", {})
        st.metric(info["name"], f"{info['n_evidence']} preuves")
        st.caption(f"{info['n_waves']} vagues")
        if gt.get("perpetrators"):
            with st.expander("Verite (spoiler)"):
                perps = gt["perpetrators"]
                names = [p["name"] if isinstance(p, dict) else str(p) for p in perps]
                st.write(f"**Coupables:** {', '.join(names)}")
                for f in gt.get("key_facts", []):
                    st.write(f"- {f}")

selected = st.selectbox("Case", list(available.keys()), format_func=lambda k: available[k]["name"])
case_info = available[selected]
manifest = case_info["manifest"]

st.markdown("---")

# =====================================================================
# Options
# =====================================================================

st.subheader("Options")
c1, c2, c3 = st.columns(3)
run_analysis = c1.checkbox("Lancer analyse LLM apres chaque vague", value=True)
timeout_inject = c2.number_input("Timeout injection (s)", value=300, min_value=30)
timeout_analysis = c3.number_input("Timeout analyse (s)", value=600, min_value=60)

# =====================================================================
# Run benchmark
# =====================================================================

st.markdown("---")

if st.button("Lancer le benchmark", type="primary", use_container_width=True):

    progress = st.progress(0)
    status = st.empty()
    log_container = st.container()
    results_container = st.container()

    logs = []
    def log(msg):
        logs.append(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}")
        log_container.code("\n".join(logs[-30:]), language="text")

    # --- Create case ---
    status.info("Creation du dossier...")
    case_data = manifest.get("case", {})
    resp = api("post", "/api/cases", json={
        "name": case_data.get("name", selected),
        "reference": case_data.get("reference", f"#BENCH-{selected}"),
        "description": case_data.get("description", "Benchmark case"),
    })
    if not resp or resp.get("error"):
        st.error(f"Echec creation dossier: {resp}")
        st.stop()

    case_id = resp["id"]
    log(f"Dossier cree: {case_id[:12]}")

    # --- Inject evidence by wave ---
    evidence_list = manifest.get("evidence", [])
    waves = sorted(set(e.get("wave", 1) for e in evidence_list))
    total_steps = len(evidence_list) + (len(waves) if run_analysis else 0)
    step = 0

    wave_results = []

    for wave_num in waves:
        wave_evidence = [e for e in evidence_list if e.get("wave", 1) == wave_num]
        waves_data = manifest.get("waves", {})
        if isinstance(waves_data, dict):
            wave_meta = waves_data.get(wave_num, waves_data.get(str(wave_num), {}))
        elif isinstance(waves_data, list):
            wave_meta = next((w for w in waves_data if w.get("wave") == wave_num), {})
        else:
            wave_meta = {}
        if not isinstance(wave_meta, dict):
            wave_meta = {}

        log(f"\n=== VAGUE {wave_num}: {wave_meta.get('name', '')} ({len(wave_evidence)} preuves) ===")
        status.info(f"Vague {wave_num}/{len(waves)} — injection de {len(wave_evidence)} preuves...")

        injected = 0
        for ev in wave_evidence:
            file_path = case_info["dir"] / ev.get("file", "")
            if not file_path.exists():
                log(f"  SKIP {ev.get('title', '?')} — fichier manquant")
                step += 1
                continue

            text = file_path.read_text(encoding="utf-8")
            resp = api("post", f"/api/cases/{case_id}/evidence/text", json={
                "title": ev.get("title", file_path.stem),
                "text": text,
                "source": ev.get("source", "Benchmark"),
            }, timeout=timeout_inject)

            if resp and not resp.get("error"):
                injected += 1
                log(f"  OK {ev.get('title', '?')[:50]}")
            else:
                log(f"  FAIL {ev.get('title', '?')[:50]} — {resp}")

            step += 1
            progress.progress(step / total_steps)

        log(f"  {injected}/{len(wave_evidence)} preuves injectees")

        # --- Analysis after wave ---
        if run_analysis:
            status.info(f"Vague {wave_num} — analyse LLM en cours...")
            log(f"  Analyse lancee...")

            resp = api("post", f"/api/cases/{case_id}/analyze", json={"trigger": "benchmark"})
            if resp and resp.get("run_id"):
                run_id = resp["run_id"]
                t0 = time.time()
                while time.time() - t0 < timeout_analysis:
                    time.sleep(10)
                    run_status = api("get", f"/api/analysis/{run_id}")
                    if run_status and run_status.get("status") != "running":
                        break
                elapsed = time.time() - t0
                final_status = run_status.get("status", "?") if run_status else "timeout"
                log(f"  Analyse: {final_status} ({elapsed:.0f}s)")
            else:
                log(f"  Analyse FAIL: {resp}")

            step += 1
            progress.progress(step / total_steps)

        # --- Collect wave stats ---
        stats = api("get", f"/api/cases/{case_id}/stats") or {}
        hypotheses = api("get", f"/api/cases/{case_id}/hypotheses") or []
        entities = api("get", f"/api/cases/{case_id}/entities") or []

        wave_result = {
            "wave": wave_num,
            "name": wave_meta.get("name", f"Vague {wave_num}"),
            "evidence": stats.get("evidence", 0),
            "entities": stats.get("entities", 0),
            "hypotheses": len(hypotheses),
            "alerts": stats.get("alerts", 0),
            "top_hypotheses": [
                {"title": h["title"][:60], "score": h["current_score"]}
                for h in sorted(hypotheses, key=lambda x: x.get("current_score", 0), reverse=True)[:5]
            ],
            "persons_found": [e["name"] for e in entities if e["entity_type"] == "person"][:15],
        }
        wave_results.append(wave_result)

        log(f"  Stats: {stats.get('evidence', 0)} preuves, {stats.get('entities', 0)} entites, {len(hypotheses)} hypotheses")
        for h in wave_result["top_hypotheses"][:3]:
            log(f"    {h['score']:5.1f}% | {h['title']}")

    progress.progress(1.0)

    # =====================================================================
    # SCORING
    # =====================================================================

    status.info("Calcul du score...")
    log("\n=== SCORING ===")

    scoring = manifest.get("scoring", {})
    ground_truth = manifest.get("ground_truth", {})
    final_stats = api("get", f"/api/cases/{case_id}/stats") or {}
    final_entities = api("get", f"/api/cases/{case_id}/entities") or []
    final_hypotheses = api("get", f"/api/cases/{case_id}/hypotheses") or []
    final_contradictions = manifest.get("expected_contradictions", [])

    scores = {}

    # 1. Entities found /20
    target_persons = [kw.lower() for kw in ground_truth.get("target_keywords", [])]
    found_names = [e["name"].lower() for e in final_entities]
    entity_hits = sum(1 for t in target_persons if any(t in n for n in found_names))
    entity_score = min(20, int(20 * entity_hits / max(len(target_persons), 1)))
    scores["Entites cles"] = entity_score
    log(f"  Entites: {entity_hits}/{len(target_persons)} trouvees -> {entity_score}/20")

    # 2. Correct hypothesis in top 3 /20
    hyp_keywords = [kw.lower() for kw in scoring.get("correct_hypothesis_keywords", ground_truth.get("target_keywords", []))]
    top3 = sorted(final_hypotheses, key=lambda x: x.get("current_score", 0), reverse=True)[:3]
    top3_text = " ".join([h.get("title", "") + " " + h.get("description", "") for h in top3]).lower()
    hyp_match = any(kw in top3_text for kw in hyp_keywords)
    scores["Hypothese top 3"] = 20 if hyp_match else 0
    log(f"  Hypothese correcte dans top 3: {'OUI' if hyp_match else 'NON'} -> {scores['Hypothese top 3']}/20")

    # 3. Contradictions detected /20
    detected_contras = api("get", f"/api/cases/{case_id}/audit?action=contradiction_found") or []
    n_detected = len(detected_contras)
    n_expected = len(final_contradictions)
    contra_score = min(20, int(20 * n_detected / max(n_expected, 1)))
    scores["Contradictions"] = contra_score
    log(f"  Contradictions: {n_detected}/{n_expected} -> {contra_score}/20")

    # 4. Correct hypothesis score > 40% /20
    best_correct = 0
    all_hyp_text = [(h, (h.get("title", "") + " " + h.get("description", "")).lower()) for h in final_hypotheses]
    for h, text in all_hyp_text:
        if any(kw in text for kw in hyp_keywords):
            best_correct = max(best_correct, h.get("current_score", 0))
    score_above_40 = best_correct > 40
    scores["Score > 40%"] = 20 if score_above_40 else int(20 * best_correct / 40)
    log(f"  Meilleur score hypothese correcte: {best_correct:.0f}% -> {scores['Score > 40%']}/20")

    # 5. Timeline + geo /20
    timeline = api("get", f"/api/cases/{case_id}/timeline") or []
    locations = api("get", f"/api/cases/{case_id}/map") or {}
    loc_list = locations.get("locations", []) if isinstance(locations, dict) else []
    timeline_score = min(10, len(timeline) * 2)
    geo_score = min(10, len(loc_list) * 3)
    scores["Timeline + Geo"] = timeline_score + geo_score
    log(f"  Timeline: {len(timeline)} events ({timeline_score}/10) | Geo: {len(loc_list)} lieux ({geo_score}/10) -> {scores['Timeline + Geo']}/20")

    total = sum(scores.values())
    log(f"\n  SCORE TOTAL: {total}/100")

    status.success(f"Benchmark termine — Score: {total}/100")

    # =====================================================================
    # DISPLAY RESULTS
    # =====================================================================

    with results_container:
        st.markdown("---")
        st.subheader(f"Resultats — {case_info['name']}")

        # Score
        sc1, sc2, sc3, sc4, sc5, sc6 = st.columns(6)
        sc1.metric("TOTAL", f"{total}/100")
        for i, (name, val) in enumerate(scores.items()):
            [sc2, sc3, sc4, sc5, sc6][i].metric(name, f"{val}/20")

        # Wave progression
        st.markdown("### Progression par vague")
        wave_df = pd.DataFrame([{
            "Vague": f"V{r['wave']}",
            "Nom": r["name"][:30],
            "Preuves": r["evidence"],
            "Entites": r["entities"],
            "Hypotheses": r["hypotheses"],
        } for r in wave_results])
        st.dataframe(wave_df, use_container_width=True, hide_index=True)

        # Hypotheses
        if final_hypotheses:
            st.markdown("### Hypotheses finales")
            for h in sorted(final_hypotheses, key=lambda x: x.get("current_score", 0), reverse=True):
                score = h.get("current_score", 0)
                title = h.get("title", "?")
                color = "#4ecdc4" if score > 40 else "#e74c3c" if score < 20 else "#f39c12"
                st.markdown(
                    f'<div style="margin:4px 0">'
                    f'<span style="font-weight:bold">{title}</span></div>'
                    f'<div style="background:#2d2d2d;border-radius:4px;height:20px">'
                    f'<div style="background:{color};width:{score}%;height:100%;border-radius:4px;'
                    f'text-align:right;padding-right:6px;color:white;font-size:12px;line-height:20px">'
                    f'{score:.0f}%</div></div>',
                    unsafe_allow_html=True,
                )

        # Entities
        st.markdown("### Entites cles trouvees")
        persons = [e for e in final_entities if e["entity_type"] == "person"]
        if persons:
            st.write(", ".join([p["name"] for p in persons[:20]]))

        # Ground truth comparison
        st.markdown("### Comparaison avec la verite")
        gt = manifest.get("ground_truth", {})
        if gt:
            col_truth, col_found = st.columns(2)
            with col_truth:
                st.markdown("**Verite:**")
                for p in gt.get("perpetrators", []):
                    name = p["name"] if isinstance(p, dict) else str(p)
                    st.write(f"- {name}")
                for f in gt.get("key_facts", []):
                    st.write(f"- {f}")
            with col_found:
                st.markdown("**Trouve par NEXUS:**")
                if final_hypotheses:
                    best = sorted(final_hypotheses, key=lambda x: x.get("current_score", 0), reverse=True)[0]
                    st.write(f"Hypothese principale: **{best['title']}** ({best['current_score']:.0f}%)")
                st.write(f"Entites: {len(final_entities)} | Personnes: {len(persons)}")

        # Save results
        bench_results = {
            "case": selected,
            "timestamp": datetime.now().isoformat(),
            "score": total,
            "scores": scores,
            "waves": wave_results,
            "case_id": case_id,
        }
        results_json = json.dumps(bench_results, ensure_ascii=False, indent=2, default=str)

        st.download_button(
            "Telecharger les resultats (JSON)",
            data=results_json,
            file_name=f"bench_{selected}_{datetime.now().strftime('%Y%m%d_%H%M')}.json",
            mime="application/json",
        )

        # Cleanup option
        st.markdown("---")
        if st.button("Supprimer le dossier de benchmark"):
            api("delete", f"/api/cases/{case_id}")
            st.success("Dossier supprime")

# =====================================================================
# Previous results
# =====================================================================

st.markdown("---")
st.subheader("Resultats precedents")

results_dir = Path(__file__).resolve().parent.parent.parent / "docs"
result_files = sorted(results_dir.glob("BENCHMARK-*.json"), reverse=True)
if result_files:
    for f in result_files[:5]:
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
            score = data.get("score", "?")
            case_name = data.get("case", "?")
            ts = data.get("timestamp", "?")[:16]
            st.write(f"- **{case_name}** — {score}/100 — {ts} — `{f.name}`")
        except Exception:
            pass
else:
    st.caption("Aucun resultat de benchmark precedent.")
