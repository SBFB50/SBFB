import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402
"""
NEXUS -- Page Benchmark LIVE.

Dashboard temps reel du benchmark en cours.
Reload = reconnexion immediate aux donnees live.
"""

import json
import time
import streamlit as st
import pandas as pd
import requests
from pathlib import Path
from datetime import datetime
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats

render_system_stats()

API = "http://localhost:8000"
BENCH_DIR = Path(__file__).resolve().parent.parent.parent / "data" / "benchmark"

CASES = {
    "kulik": {"dir": "kulik", "name": "Affaire Elodie Kulik (2002)"},
    "gsk": {"dir": "golden-state-killer", "name": "Golden State Killer (1974-86)"},
    "moreau": {"dir": "affaire-moreau", "name": "Affaire Moreau (fictif)"},
}


def api_call(method, path, **kwargs):
    kwargs.setdefault("timeout", 300)
    try:
        r = getattr(requests, method)(f"{API}{path}", **kwargs)
        return r.json() if r.status_code < 400 else None
    except Exception:
        return None


def find_bench_case():
    """Find any active benchmark case in the system."""
    cases = api.list_cases() or []
    for c in cases:
        ref = c.get("reference", "")
        if "BENCH" in ref.upper() or "KULIK" in ref.upper() or "GSK" in ref.upper() or "MOREAU" in ref.upper():
            return c
    # Return most recent case if any
    return cases[0] if cases else None


# =====================================================================
st.title("Benchmark NEXUS — Live")

health = api.check_health()
if not health:
    st.error("API indisponible")
    st.stop()

st.markdown("---")

# =====================================================================
# DETECT OR CREATE BENCHMARK
# =====================================================================

bench_case = find_bench_case()

if bench_case:
    case_id = bench_case["id"]
    case_name = bench_case["name"]
    st.success(f"Connecte a: **{case_name}** (`{case_id[:12]}`)")
else:
    st.warning("Aucun benchmark en cours.")

    # Launch panel
    st.subheader("Lancer un benchmark")
    available = {}
    for key, info in CASES.items():
        manifest_path = BENCH_DIR / info["dir"] / "manifest.json"
        if manifest_path.exists():
            available[key] = info

    if available:
        selected = st.selectbox("Case", list(available.keys()), format_func=lambda k: available[k]["name"])
        manifest = json.loads((BENCH_DIR / CASES[selected]["dir"] / "manifest.json").read_text(encoding="utf-8"))
        case_data = manifest.get("case", {})
        st.caption(f"{len(manifest.get('evidence', []))} preuves, {len(set(e.get('wave',1) for e in manifest.get('evidence',[])))} vagues")

        if st.button("Creer le dossier et injecter la vague 1", type="primary"):
            resp = api_call("post", "/api/cases", json={
                "name": case_data.get("name", selected),
                "reference": case_data.get("reference", f"#BENCH-{selected.upper()}"),
                "description": case_data.get("description", ""),
            })
            if resp:
                cid = resp["id"]
                st.session_state["bench_case_id"] = cid
                # Inject wave 1 evidence
                wave1 = [e for e in manifest.get("evidence", []) if e.get("wave", 1) == 1]
                bar = st.progress(0)
                for i, ev in enumerate(wave1):
                    fp = BENCH_DIR / CASES[selected]["dir"] / ev.get("file", "")
                    if fp.exists():
                        text = fp.read_text(encoding="utf-8")
                        api_call("post", f"/api/cases/{cid}/evidence/text", json={
                            "title": ev.get("title", fp.stem),
                            "text": text,
                            "source": ev.get("source", "Benchmark"),
                        })
                    bar.progress((i + 1) / len(wave1))
                st.success(f"Vague 1 injectee ({len(wave1)} preuves)")
                st.rerun()
    st.stop()


# =====================================================================
# LIVE DASHBOARD — always shows current state
# =====================================================================

case_id = bench_case["id"]

# --- Stats ---
stats = api.get_case_stats(case_id) or {}

c1, c2, c3, c4, c5 = st.columns(5)
c1.metric("Preuves", stats.get("evidence", 0))
c2.metric("Entites", stats.get("entities", 0))
c3.metric("Hypotheses", stats.get("hypotheses", 0))
c4.metric("Alertes", stats.get("alerts", 0))
c5.metric("Monitoring", stats.get("monitoring_jobs", 0))

st.markdown("---")

# --- Tabs ---
tab_hyp, tab_ent, tab_ev, tab_graph, tab_contra, tab_audit, tab_inject = st.tabs([
    "Hypotheses", "Entites", "Preuves", "Graphe", "Contradictions", "Audit", "Injecter vagues"
])

# --- HYPOTHESES ---
with tab_hyp:
    hypotheses = api.list_hypotheses(case_id) or []
    if hypotheses:
        for h in sorted(hypotheses, key=lambda x: x.get("current_score", 0), reverse=True):
            score = h.get("current_score", 50)
            title = h.get("title", "?")
            color = "#4ecdc4" if score > 50 else "#e74c3c" if score < 25 else "#f39c12"
            st.markdown(
                f'<div style="margin:6px 0">'
                f'<b>{title}</b> <span style="color:#888">({h.get("status","?")})</span></div>'
                f'<div style="background:#2d2d2d;border-radius:4px;height:24px">'
                f'<div style="background:{color};width:{max(score,2)}%;height:100%;border-radius:4px;'
                f'text-align:right;padding-right:8px;color:white;font-size:13px;line-height:24px">'
                f'{score:.0f}%</div></div>',
                unsafe_allow_html=True,
            )

        # Evolution chart
        all_evo = []
        for h in hypotheses:
            evo = api.get_hypothesis_evolution(h["id"]) or []
            for p in evo:
                all_evo.append({"Date": p.get("date", ""), "Score": p.get("score", 50), "Hypothese": h.get("title", "?")[:40]})
        if all_evo:
            df = pd.DataFrame(all_evo)
            try:
                df["Date"] = pd.to_datetime(df["Date"])
                pivot = df.pivot_table(index="Date", columns="Hypothese", values="Score", aggfunc="last").ffill()
                st.line_chart(pivot, height=300)
            except Exception:
                pass
    else:
        st.info("Aucune hypothese — lancez une analyse")
        if st.button("Lancer analyse"):
            api_call("post", f"/api/cases/{case_id}/analyze", json={"trigger": "benchmark"})
            st.success("Analyse lancee en background")

# --- ENTITIES ---
with tab_ent:
    entities = api.list_entities(case_id) or []
    if entities:
        by_type = {}
        for e in entities:
            t = e["entity_type"]
            by_type.setdefault(t, []).append(e["name"])

        for t in ["person", "location", "vehicle", "phone", "organization", "date", "other"]:
            if t in by_type:
                names = by_type[t]
                st.markdown(f"**{t.capitalize()}** ({len(names)})")
                st.write(", ".join(names[:30]))
        st.caption(f"Total: {len(entities)} entites")
    else:
        st.info("Aucune entite")

# --- EVIDENCE ---
with tab_ev:
    evidence = api.list_evidence(case_id) or []
    if evidence:
        rows = []
        for e in evidence:
            rows.append({
                "Titre": e.get("title", "?")[:60],
                "Type": e.get("evidence_type", "?"),
                "Status": e.get("status", "?"),
                "Source": (e.get("source") or "")[:30],
                "Fiabilite": e.get("reliability", 50),
            })
        st.dataframe(pd.DataFrame(rows), use_container_width=True, hide_index=True)

        with st.expander(f"Details ({len(evidence)} preuves)"):
            for e in evidence:
                st.markdown(f"**{e.get('title', '?')}**")
                summary = e.get("summary") or ""
                if summary:
                    st.caption(summary[:300])
                st.markdown("---")
    else:
        st.info("Aucune preuve")

# --- GRAPH ---
with tab_graph:
    graph_data = api.get_graph(case_id)
    if graph_data and (graph_data.get("nodes") or graph_data.get("edges")):
        nodes = graph_data.get("nodes", [])
        edges = graph_data.get("edges", [])

        c_n, c_e = st.columns(2)
        c_n.metric("Noeuds", len(nodes))
        c_e.metric("Aretes", len(edges))

        # Stats by type
        graph_stats = api.get_graph_stats(case_id)
        if graph_stats:
            cols_stats = st.columns(len(graph_stats))
            for i, (label, count) in enumerate(graph_stats.items()):
                cols_stats[i].metric(label, count)

        # Interactive graph visualization
        try:
            from frontend.components.graph_viewer import render_graph
            render_graph(graph_data, height=500)
        except Exception as exc:
            st.warning(f"Visualisation graphe indisponible: {exc}")
            st.json(graph_data)
    else:
        st.info("Graphe vide — les entites n'ont pas encore ete synchronisees vers Neo4j")

# --- CONTRADICTIONS ---
with tab_contra:
    audit_entries = api.list_audit_log(case_id, action="contradiction_found") or []
    if audit_entries:
        for c in audit_entries:
            details = c.get("details")
            desc = ""
            if details:
                try:
                    parsed = json.loads(details) if isinstance(details, str) else details
                    desc = parsed.get("description", "")
                except Exception:
                    desc = str(details)[:300]
            st.markdown(f"**{c.get('timestamp', '?')[:19]}** — {desc[:300]}")
    else:
        st.info("Aucune contradiction detectee")
        if st.button("Lancer detection de contradictions"):
            api_call("get", f"/api/cases/{case_id}/contradictions")
            st.info("Detection lancee")

# --- AUDIT ---
with tab_audit:
    ACTION_ICONS = {
        "evidence_added": "📄", "evidence_ingested_auto": "🤖",
        "entity_discovered": "👤", "hypothesis_created": "💡",
        "hypothesis_scored": "📊", "contradiction_found": "⚡",
        "monitoring_result": "🔍", "query_generated": "🔎",
        "self_questioning": "🧠", "analysis_completed": "✅",
    }
    audit = api.list_audit_log(case_id, limit=30) or []
    if audit:
        for e in audit:
            icon = ACTION_ICONS.get(e.get("action", ""), "📌")
            ts = (e.get("timestamp") or "")[:19]
            st.markdown(f"{icon} **{ts}** [{e.get('actor','?')}] {e.get('summary','')[:80]}")
    else:
        st.caption("Aucune entree d'audit")

# --- INJECT MORE WAVES ---
with tab_inject:
    # Find which case this is
    ref = bench_case.get("reference", "")
    case_key = None
    for k, v in CASES.items():
        manifest_path = BENCH_DIR / v["dir"] / "manifest.json"
        if manifest_path.exists():
            m = json.loads(manifest_path.read_text(encoding="utf-8"))
            if m.get("case", {}).get("reference") == ref or k.lower() in bench_case.get("name", "").lower():
                case_key = k
                break

    if case_key:
        manifest = json.loads((BENCH_DIR / CASES[case_key]["dir"] / "manifest.json").read_text(encoding="utf-8"))
        all_evidence = manifest.get("evidence", [])
        current_count = stats.get("evidence", 0)
        waves = sorted(set(e.get("wave", 1) for e in all_evidence))

        st.write(f"**{current_count}/{len(all_evidence)}** preuves injectees")

        for w in waves:
            wave_ev = [e for e in all_evidence if e.get("wave", 1) == w]
            waves_data = manifest.get("waves", {})
            if isinstance(waves_data, dict):
                wm = waves_data.get(w, waves_data.get(str(w), {}))
            else:
                wm = {}
            wave_name = wm.get("name", f"Vague {w}") if isinstance(wm, dict) else f"Vague {w}"

            injected = current_count >= sum(len([e for e in all_evidence if e.get("wave", 1) <= w]))

            col_w, col_btn = st.columns([3, 1])
            col_w.write(f"**Vague {w}** — {wave_name} ({len(wave_ev)} preuves) {'✅' if injected else ''}")

            if not injected:
                if col_btn.button(f"Injecter V{w}", key=f"inject_v{w}"):
                    bar = st.progress(0)
                    for i, ev in enumerate(wave_ev):
                        fp = BENCH_DIR / CASES[case_key]["dir"] / ev.get("file", "")
                        if fp.exists():
                            text = fp.read_text(encoding="utf-8")
                            api_call("post", f"/api/cases/{case_id}/evidence/text", json={
                                "title": ev.get("title", fp.stem),
                                "text": text,
                                "source": ev.get("source", "Benchmark"),
                            })
                        bar.progress((i + 1) / len(wave_ev))
                    st.success(f"Vague {w} injectee")
                    st.rerun()

        st.markdown("---")
        if st.button("Lancer analyse complete"):
            api_call("post", f"/api/cases/{case_id}/analyze", json={"trigger": "benchmark"})
            st.success("Analyse lancee")

        if st.button("Generer hypotheses"):
            api_call("post", f"/api/cases/{case_id}/hypotheses/generate")
            st.success("Generation lancee")

        if st.button("Re-evaluer toutes les hypotheses"):
            api_call("post", f"/api/cases/{case_id}/evaluate-all")
            st.success("Re-evaluation lancee")
    else:
        st.info("Case non identifie — injectez manuellement via l'onglet Preuves")

# =====================================================================
# SCORING (ground truth comparison)
# =====================================================================

st.markdown("---")
st.subheader("Score")

# Try to find manifest for this case
scoring_manifest = None
for k, v in CASES.items():
    mp = BENCH_DIR / v["dir"] / "manifest.json"
    if mp.exists() and k.lower() in bench_case.get("name", "").lower():
        scoring_manifest = json.loads(mp.read_text(encoding="utf-8"))
        break

if scoring_manifest:
    gt = scoring_manifest.get("ground_truth", {})
    scoring_cfg = scoring_manifest.get("scoring", {})
    target_kw = [kw.lower() for kw in gt.get("target_keywords", [])]
    hypotheses = api.list_hypotheses(case_id) or []
    entities = api.list_entities(case_id) or []
    found_names = [e["name"].lower() for e in entities]

    scores = {}

    # 1. Entities /20
    hits = sum(1 for t in target_kw if any(t in n for n in found_names))
    scores["Entites"] = min(20, int(20 * hits / max(len(target_kw), 1)))

    # 2. Hypothesis top 3 /20
    top3 = sorted(hypotheses, key=lambda x: x.get("current_score", 0), reverse=True)[:3]
    top3_text = " ".join([h.get("title", "") + " " + h.get("description", "") for h in top3]).lower()
    hyp_kw = [kw.lower() for kw in scoring_cfg.get("correct_hypothesis_keywords", target_kw)]
    scores["Hypothese top3"] = 20 if any(kw in top3_text for kw in hyp_kw) else 0

    # 3. Contradictions /20
    contras = api.list_audit_log(case_id, action="contradiction_found") or []
    n_expected = len(scoring_manifest.get("expected_contradictions", []))
    scores["Contradictions"] = min(20, int(20 * len(contras) / max(n_expected, 1)))

    # 4. Score > 40% /20
    best = 0
    for h in hypotheses:
        txt = (h.get("title", "") + " " + h.get("description", "")).lower()
        if any(kw in txt for kw in hyp_kw):
            best = max(best, h.get("current_score", 0))
    scores["Score > 40%"] = 20 if best > 40 else int(20 * best / 40)

    # 5. Timeline + Geo /20
    timeline = api.get_timeline(case_id) or []
    map_data = api_call("get", f"/api/cases/{case_id}/map") or {}
    locs = map_data.get("locations", []) if isinstance(map_data, dict) else []
    scores["Timeline+Geo"] = min(10, len(timeline) * 2) + min(10, len(locs) * 3)

    total = sum(scores.values())

    # Display
    sc = st.columns(6)
    sc[0].metric("TOTAL", f"{total}/100")
    for i, (name, val) in enumerate(scores.items()):
        sc[i + 1].metric(name, f"{val}/20")

    # Ground truth
    with st.expander("Verite (spoiler)"):
        for p in gt.get("perpetrators", []):
            name = p["name"] if isinstance(p, dict) else str(p)
            st.write(f"- {name}")
        for f in gt.get("key_facts", []):
            st.write(f"- {f}")
else:
    st.caption("Pas de manifest de scoring pour ce dossier")

# =====================================================================
# Actions
# =====================================================================

st.markdown("---")
col_del, col_refresh = st.columns(2)
with col_refresh:
    if st.button("Rafraichir", use_container_width=True):
        st.rerun()
with col_del:
    if st.button("Supprimer ce dossier", type="secondary", use_container_width=True):
        api_call("delete", f"/api/cases/{case_id}")
        st.success("Supprime")
        st.rerun()
