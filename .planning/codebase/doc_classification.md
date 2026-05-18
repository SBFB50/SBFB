# Classification des documents .planning/ — Audit exhaustif

**Date** : 2026-05-18
**Methode** : Lecture integrale de chaque fichier, croisement avec CLAUDE.md et MEMORY.md
**Scope** : `.planning/research/` (54 fichiers), `.planning/codebase/` (13 fichiers), `.planning/active/` (16 fichiers), `.planning/*.md` (6 fichiers), `.planning/archive/` (structure)

---

## 1. `.planning/codebase/` — 13 fichiers

### 1.1 Documents a jour (post-S64, ecrits 2026-05-18)

| Fichier | Sujet | Etat | Notes |
|---|---|---|---|
| `ARCHITECTURE.md` | Architecture Rust workspace 12 crates, layered headless-first | **CANON** | Source de verite architecture. Analyse date 2026-05-18, reflete le code post-S64. |
| `STRUCTURE.md` | Arborescence fichiers, repertoires, conventions nommage | **CANON** | Source de verite structure. Analyse date 2026-05-18. |
| `frontend_architecture.md` | Stack React 19, routing, state, bridge, composants, pages | **CANON** | Source de verite frontend. Analyse date 2026-05-18. |
| `protocol_wire_formats.md` | iroh 0.98, wire types, crypto, canonical bytes, 14 domaines | **CANON** | Source de verite protocole. Analyse date 2026-05-18. |
| `security_posture.md` | Threat model STRIDE+LINDDUN, 5 personas, defenses en couches | **CANON** | Source de verite securite. Analyse date 2026-05-18. |
| `tests_quality.md` | Distribution tests, CI/CD, commandes, Criterion benches | **CANON** | Source de verite tests. Analyse date 2026-05-18. |
| `planning_history.md` | Timeline projet, versions, decisions gelees, sprint history | **CANON** | Source de verite historique. Analyse date 2026-05-18. |
| `APPS_BRIDGE_DOCS.md` | Bridge SDK, apps exemple, iframe sandbox, manifest SBFB.json | **CANON** | Source de verite apps/bridge. Analyse date 2026-05-18. |

### 1.2 Documents obsoletes (pre-pivot, date 2026-04-06)

| Fichier | Sujet | Etat | Remplace par | Action |
|---|---|---|---|---|
| `STACK.md` | Stack Python 3.13 + FastAPI + Neo4j + Docker + Ollama | **SUPERSEDED** | Nouveau `ARCHITECTURE.md` (2026-05-18) + CLAUDE.md §Stack | **Supprimer** — decrit l'ancien NEXUS cold-case, pas SBFB |
| `CONVENTIONS.md` | Conventions Python/TypeScript pre-pivot (snake_case, pytest) | **SUPERSEDED** | Les conventions Rust actuelles sont dans `ARCHITECTURE.md` + `docs/rust/PATTERNS.md` | **Supprimer** — decrit des conventions Python qui n'existent plus |
| `TESTING.md` | Patterns pytest pre-pivot (233 tests Python) | **SUPERSEDED** | Nouveau `tests_quality.md` (2026-05-18) | **Supprimer** — plus aucun test Python n'existe |
| `CONCERNS.md` | Tech debt pre-pivot (FTS5, GLiNER singleton, Neo4j) | **SUPERSEDED** | Le code reference n'existe plus (supprime S51) | **Supprimer** — tous les fichiers references sont supprimes |
| `INTEGRATIONS.md` | Integrations pre-pivot (Ollama Python, SearXNG, Robin, Neo4j) | **SUPERSEDED** | `protocol_wire_formats.md` + `ARCHITECTURE.md` | **Supprimer** — decrit des integrations SearXNG/Robin/Neo4j qui n'existent plus |

**Resume** : 8 CANON (post-S64), 5 SUPERSEDED (pre-pivot). Les 5 obsoletes doivent etre supprimes ou archives car ils decrivent un codebase qui n'existe plus.

---

## 2. `.planning/research/` — 54 fichiers

### 2.1 Pre-pivot NEXUS cold-case (avril 2026, avant 2026-04-10)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `SUMMARY.md` | 2026-04-06 | Resume reactive event-driven pour NEXUS cold-case | **FOSSIL** | Archiver — decrit architecture asyncio/Python pour le NEXUS supprime |
| `REACTIVE_ARCHITECTURE.md` | 2026-04-06 | EventBus asyncio, SpiderFoot pattern, VRAM scheduler | **FOSSIL** | Archiver — 455 lignes de design Python pour un codebase supprime |
| `DISTRIBUTED_GPU_RESEARCH.md` | 2026-04-09 | Task queue distribue, GPU registry, WebSocket vs polling | **FOSSIL** | Archiver — design serveur central FastAPI, pre-P2P |
| `PIXEL_ART_AI_RESEARCH.md` | 2026-04-09 | Diffusion models pixel art, sprite sheets, SDXL Turbo | **FOSSIL** | Archiver — feature never started, unrelated to SBFB |
| `AWWWARDS_DESIGN_RESEARCH.md` | 2026-04-10 | Dashboard design premium, animations Motion, shadcn | **RESEARCH** | Garder — patterns CSS/Motion/shadcn potentiellement reutilisables pour le shell React actuel |

### 2.2 Design docs S19-S21 (hardening, avril 2026)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `S19_phase_B_pow_hashcash_design.md` | 2026-04-16 | Design PoW Hashcash gossip subscribe | **RESEARCH** | Garder — documente decisions architecturales code vivant dans `pow.rs` |
| `S19_phase_C_tls_cert_pinning_design.md` | 2026-04-16 | Design TLS cert pinning relays | **RESEARCH** | Garder — documente code vivant `tls_pinning.rs` |
| `S19_phase_D_delayed_upload_queue_design.md` | 2026-04-16 | Design delayed upload queue (anti-traffic correlation) | **RESEARCH** | Garder — documente code vivant |
| `S19_phase_E_pkarr_relay_design.md` | 2026-04-16 | Design pkarr relay docker self-hosted | **RESEARCH** | Garder — reference deployment infra |
| `S20_phase_B_duress_panic_design.md` | 2026-04-16 | Design duress PIN + panic wipe | **RESEARCH** | Garder — documente code vivant `keystore.rs` duress mode |
| `S20_phase_D_structured_output_design.md` | 2026-04-18 | Design structured output Ollama | **RESEARCH** | Garder — documente design worker-core |

### 2.3 Research S21 — PII/rate-limit/output filter (avril 2026)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `S21_research_pii_sdk_options.md` | 2026-04-18 | Pre-research PII SDK options | **RESEARCH** | Garder — reference decision D2 PII |
| `S21_research_rust_first_alignment.md` | 2026-04-18 | Analyse Rust-first vs JS+Python PII | **RESEARCH** | Garder — justifie le rejet Rust-first iframe |
| `S21_research_backbone_resolution.md` | 2026-04-18 | Resolution backbone GLiNER edge model | **RESEARCH** | Garder — resolution factuelle divergence documentee |
| `S21_research_ort_wasm_alternatives.md` | 2026-04-18 | ONNX Runtime wasm alternatives | **RESEARCH** | Garder — justifie rejet Rust-first wasm NER |
| `S21_phase_B_iframe_pii_sdk_design.md` | 2026-04-19 | Design iframe PII redaction SDK | **RESEARCH** | Garder — documente architecture pii/sdk vivante dans `web/src/sdk/pii/` |
| `S21_phase_C_output_filter_design.md` | 2026-04-19 | Design output filter coord-side | **RESEARCH** | Garder — documente `output_filter.rs` vivant |
| `S21_phase_D_quarantine_design.md` | 2026-04-19 | Design quarantine queue | **RESEARCH** | Garder — documente `quarantine_queue.rs` vivant |
| `S21_research_p2p_compute_scoring_systems.md` | 2026-04-19 | Scoring BOINC/Folding@home/Golem | **RESEARCH** | Garder — reference pour kudos v2 futur |
| `S21_research_fair_allocation_mechanisms.md` | 2026-04-19 | Quadratic funding, anti-whale mechanisms | **RESEARCH** | Garder — reference pour LT-1/LT-3 |

### 2.4 Research S22-S24 (Sybil, process, agents)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `S22_contribution_family_sybil_matrix.md` | 2026-04-20 | Matrice Sybil 8 familles contribution | **RESEARCH** | Garder — reference LT-3 post-Gate-3 |
| `S23_to_S29_agents_sudo_integration_matrix.md` | 2026-04-20 | Matrice integration openai-agents + sudo | **RESEARCH** | Garder — reference LT-4 post-v1.0 |
| `S24_process_review_2026-04-21.md` | 2026-04-21 | Diagnostic factuel process S16-S23 | **RESEARCH** | Garder — meta-process, reference pour evolution workflow |
| `S23_S24_process_evolution_analysis.md` | 2026-04-22 | Diff process S23→S24 | **RESEARCH** | Garder — meta-process, reference historique |

### 2.5 Day-0 open questions + NLnet

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `DAY0_OPEN_QUESTIONS.md` | 2026-04-16 | Parametres arbitraires S20-S25 | **FOSSIL** | Archiver — tous les sprints S20-S25 sont clos, les decisions sont prises. Le document ne sert plus a informer aucun kickoff futur. |
| `nlnet_application_draft.txt` | 2026-04-27 | Brouillon candidature NLnet NGI0 | **RESEARCH** | Garder — reference pour future candidature (deadline juin 2026) |

### 2.6 Sprint 33 multi-noeud + S49 migration Rust

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `sprint33_multinode_research.md` | 2026-04-27 | Research multi-noeud pre-S33 | **FOSSIL** | Archiver — S33 clos depuis longtemps, multi-noeud est operationnel (6 tests, 2 E2E) |
| `S49_coordinator_rust_migration.md` | 2026-05-01 | Inventaire migration Python→Rust | **FOSSIL** | Archiver — migration terminee (S50-S51), plus de Python dans le projet |

### 2.7 Frontend research (avril 2026)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `frontend_ux_protocol_analysis.md` | 2026-04-27 | Inventaire API surface + gaps UX frontend | **RESEARCH** | Garder — l'inventaire des routes daemon/coordinator reste informatif |
| `frontend_inspiration_catalog.md` | 2026-04-27 | Catalogue projets open source inspiration | **RESEARCH** | Garder — reference design |
| `spacedrive_deep_dive.md` | 2026-04-27 | Deep dive Spacedrive patterns UX | **RESEARCH** | Garder — reference patterns reutilisables |
| `frontend_vision_v2.md` | 2026-04-27 | Vision frontend "depuis le protocole" | **RESEARCH** | Garder — vision produit pas encore implemented |
| `3d_space_simulation_catalog.md` | 2026-04-27 | Catalogue simulations 3D (black holes, nebula) | **FOSSIL** | Archiver — feature decorative jamais implementee, hors roadmap |

### 2.8 Research post-v1.0 apps et features (mai 2026)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `pre_v1_apps_protocol_explorer_ideas_hub.md` | 2026-05-07 | Spec Protocol Explorer + Ideas Hub | **SUPERSEDED** | Les apps sont livrees (S57-S58). Le doc decrit un "a faire" qui est fait. Garder en RESEARCH pour reference mais plus canon. |
| `gpu_pooling_distributed_inference.md` | 2026-05-10 | Research GPU pooling (Petals, Exo, Prima) | **RESEARCH** | Garder — reference pour future pooling GPU |
| `vote_triggered_task_dispatch.md` | 2026-05-10 | Research vote→task dispatch (DAO patterns) | **RESEARCH** | Garder — reference pour Ideas Hub evolution |
| `p2p_storage_replication_iroh_docs.md` | 2026-05-10 | Research storage P2P iroh-docs | **RESEARCH** | Garder — reference pour app storage evolution |
| `community_code_validation_p2p.md` | 2026-05-10 | Revue code decentralisee (Radicle patterns) | **RESEARCH** | Garder — reference pour LT-2 Radicle |
| `babel_translation_protocol.md` | 2026-04-27 | Spec Babel bibliotheque multilingue P2P | **SUPERSEDED** par `s73_s75_factory_babel_research.md` | Garder en RESEARCH comme reference historique originale, mais le doc S73-S75 est plus recent et complet. |

### 2.9 Research vision produit (mai 2026)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `chat_ia_reseau_recherche_reseau_rnd.md` | 2026-05-12 | RRV = Recherche Reseau Verifiable | **RESEARCH** | Garder — vision produit fondatrice pour RRV |
| `p2panda_public_protocol_briques.md` | 2026-05-13 | Briques p2panda utiles au protocole public | **RESEARCH** | Garder — reference architecture |
| `public_verifiable_feed_roadmap.md` | 2026-05-13 | Roadmap 6 sprints feed verifiable S61-S66 | **SUPERSEDED** par la nouvelle roadmap S65-S75 | Le scope original (6 sprints S61-S66) est absorbe dans la nouvelle roadmap etendue S65-S75. Garder en RESEARCH pour reference historique. |
| `iroh_no_internet_babel_anti_censure.md` | 2026-05-16 | Iroh sans Internet, anti-censure | **RESEARCH** | Garder — reference factuelle pour claims anti-censure |
| `sbfb_project_factory_rrv_oss_research.md` | 2026-05-17 | Project Factory + RRV local-first + OSS reuse | **RESEARCH** | Garder — base de l'arc S73-S75 |
| `rrv_scoped_search_compute_groups.md` | 2026-05-16 | RRV scoped search, compute groups | **RESEARCH** | Garder — base de l'arc S70-S72 |
| `sbfb_cross_domain_use_cases.md` | 2026-05-17 | Use cases cross-domain SBFB | **RESEARCH** | Garder — vision produit long-terme |
| `sbfb_rrv_code_factory_vision_pitch.md` | 2026-05-17 | Vision SBFB + RRV + Code Factory (pitch) | **CANON** | Source de verite vision produit. Ce document est le texte fondateur qui explique le "pourquoi" de la roadmap post-v1.0. |

### 2.10 Nouveaux docs S65-S75 (2026-05-18)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `s65_contrat_public_research.md` | 2026-05-18 | Research contrat public S65 | **CANON** | Source de verite research S65. Inventaire exhaustif des textes de confiance + gaps + plan de phases. |
| `s66_durabilite_research.md` | 2026-05-18 | Research durabilite S66 | **CANON** | Source de verite research S66. Inventaire exhaustif persistence iroh + gaps + corrections. |
| `s67_gouvernance_confiance_research.md` | 2026-05-18 | Research gouvernance confiance S67 | **CANON** | Source de verite research S67. Analyse exhaustive systeme curator. |
| `s68_s69_preuves_pilote_research.md` | 2026-05-18 | Research proof packs S68 + pilote ferme S69 | **CANON** | Source de verite research S68-S69. |
| `s70_s72_rrv_research.md` | 2026-05-18 | Research RRV (Recherche Reseau Verifiable) S70-S72 | **CANON** | Source de verite research S70-72. |
| `s73_s75_factory_babel_research.md` | 2026-05-18 | Research Code Factory + Babel S73-S75 | **CANON** | Source de verite research S73-75. Supersede `babel_translation_protocol.md` pour les details implementation. |
| `s65_s75_cross_cutting_research.md` | 2026-05-18 | Dependances, risques, sequencage 11 sprints | **CANON** | Source de verite cross-cutting roadmap S65-S75. |

### 2.11 Docs de recherche specifiques S73-S75 (dans research/)

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `STACK.md` (dans research/) | 2026-05-18 | Stack technique S73-S75 Factory | **CANON** | Source de verite stack pour l'arc Factory. Attention : NE PAS confondre avec le `codebase/STACK.md` obsolete. |
| `FEATURES.md` (dans research/) | 2026-05-18 | Feature landscape Factory | **CANON** | Table stakes + differentiators pour l'arc Factory. |
| `ARCHITECTURE.md` (dans research/) | 2026-05-18 | Architecture patterns Factory | **CANON** | Patterns broker, sandbox, template engine. Attention : NE PAS confondre avec `codebase/ARCHITECTURE.md`. |
| `PITFALLS.md` (dans research/) | 2026-05-18 | Pitfalls domaine Factory | **CANON** | Pieges critiques identifies pour l'arc Factory. |

---

## 3. `.planning/active/` — 16 fichiers

Tous les fichiers dans `active/` sont des artefacts du sprint 64 (le dernier clos) et du sprint 65 (le prochain).

| Fichier | Etat | Action |
|---|---|---|
| `sprint64_kickoff.md` | **ACTIVE** | Archiver vers `.planning/archive/v2.0/` a la cloture S65 |
| `sprint64_plan.md` | **ACTIVE** | idem |
| `sprint64_design_review.md` | **ACTIVE** | idem |
| `sprint64_phase_A_preflight.md` | **ACTIVE** | idem |
| `sprint64_phase_A_review.md` | **ACTIVE** | idem |
| `sprint64_phase_B_preflight.md` | **ACTIVE** | idem |
| `sprint64_phase_B_review.md` | **ACTIVE** | idem |
| `sprint64_phase_C_preflight.md` | **ACTIVE** | idem |
| `sprint64_phase_C_review.md` | **ACTIVE** | idem |
| `sprint64_phase_D_preflight.md` | **ACTIVE** | idem |
| `sprint64_phase_D_review.md` | **ACTIVE** | idem |
| `sprint64_phase_E_preflight.md` | **ACTIVE** | idem |
| `sprint64_phase_E_review.md` | **ACTIVE** | idem |
| `sprint64_verification.md` | **ACTIVE** | idem |
| `sprint64_audit_plan.md` | **ACTIVE** | Audit plan du S64 = materiel S65 |
| `sprint65_audit_plan.md` | **ACTIVE** | Prepare pour le prochain sprint |

---

## 4. `.planning/*.md` — 6 fichiers racine

| Fichier | Date | Sujet | Etat | Action |
|---|---|---|---|---|
| `README.md` | ~S16+ | PARA pattern, layout, cycle de vie sprint | **CANON** | Source de verite organisation `.planning/`. A mettre a jour (mentionne `codebase/` snapshot 2026-04-06, remplacer par 2026-05-18). |
| `NEXUS_GOV_ROADMAP.md` | pre-pivot | Roadmap plateforme politique autonome | **FOSSIL** | Archiver — decrit le projet pre-pivot NEXUS cold-case (supprime S51). `planning_history.md` le documente deja comme obsolete. |
| `OPEN_SOURCE_ROADMAP.md` | pre-pivot | Roadmap open source + financement pre-pivot | **FOSSIL** | Archiver — decrit strategies pour le NEXUS cold-case, pas SBFB. |
| `DISTRIBUTED_GPU_ROADMAP.md` | pre-pivot | Roadmap GPU distribue pre-pivot | **FOSSIL** | Archiver — architecture serveur central, incompatible avec le design P2P pur SBFB. |
| `roadmap_v1.0_alexandria.md` | 2026-04-26 | Roadmap v1.0 tag + repo public + VPS | **SUPERSEDED** | v1.0 est taguee. Le doc decrit un etat passe. Garder en RESEARCH pour reference historique. |
| `roadmap_v1_migration_rust.md` | 2026-04-29 | Roadmap migration Python→Rust S38-S56 | **SUPERSEDED** | Migration terminee (S50-S51). Le doc decrit un plan complete. Garder en RESEARCH pour reference historique. |

---

## 5. `.planning/archive/` — Structure

| Repertoire | Sprints | Contenu | Etat |
|---|---|---|---|
| `archive/v1.0/` | S0-S13 | 59 fichiers (kickoff, plan, verification, audit) | **ARCHIVE** — intact |
| `archive/v1.1/` | S14-S15 | 10 fichiers | **ARCHIVE** — intact |
| `archive/v1.2/` | S16-S60+ | 100+ fichiers (S16 a S64 non encore archive) | **ARCHIVE** — les fichiers S64 active/ doivent y migrer a la cloture S65 |

**Note** : Les fichiers `active/sprint64_*` doivent etre deplaces vers `archive/v2.0/` (ou `archive/v1.2/` si v2.0 n'est pas encore officiellement ouvert) lors du kickoff S65.

---

## 6. Analyse des supersessions et conflits

### 6.1 `public_verifiable_feed_roadmap.md` vs nouvelle roadmap S65-S75

**Verdict : SUPERSEDED.**

Le document `public_verifiable_feed_roadmap.md` (2026-05-13) decrit une roadmap de 6 sprints (S61-S66) pour le feed verifiable. Les sprints S61-S64 sont maintenant clos. La nouvelle roadmap S65-S75 etend le scope (11 sprints, 3 arcs) et remplace la vision originale. Le document original reste utile comme reference historique (il contient l'inventaire "ce qui existe" et "ce qui manque" au moment de sa redaction), mais n'est plus la source de verite pour le planning futur.

### 6.2 `babel_translation_protocol.md` vs `s73_s75_factory_babel_research.md`

**Verdict : PARTIELLEMENT SUPERSEDED.**

Le document original (2026-04-27) contient la vision fondatrice de Babel (gap verifie, pitch, corpus Gutenberg). Le nouveau doc S73-S75 (2026-05-18) est plus detaille sur l'implementation technique (structure app, bridge SDK, NLLB integration). Les deux documents ont une valeur complementaire :
- `babel_translation_protocol.md` = vision produit originale (garder en RESEARCH)
- `s73_s75_factory_babel_research.md` = research technique implementation (CANON)

### 6.3 `codebase/STACK.md` (2026-04-06) vs `codebase/ARCHITECTURE.md` (2026-05-18)

**Verdict : SUPERSEDED completement.**

L'ancien `STACK.md` decrit Python 3.13 + FastAPI + Neo4j + Docker + Ollama. Le nouveau `ARCHITECTURE.md` decrit Rust 1.94 + axum + iroh 0.98 + SQLite. Ce sont deux codebase entierement differentes (pre-pivot vs post-pivot).

### 6.4 `codebase/TESTING.md` (2026-04-06) vs `codebase/tests_quality.md` (2026-05-18)

**Verdict : SUPERSEDED completement.** L'ancien decrit 233 tests pytest, le nouveau 1659 tests (1344 Rust + 265 Vitest + 44 Playwright + 6 size-limit).

### 6.5 `codebase/CONVENTIONS.md` (2026-04-06) vs realite code

**Verdict : SUPERSEDED.** Decrit des conventions Python pour un codebase qui est desormais 100% Rust + TypeScript. Il n'existe pas de `CONVENTIONS.md` a jour — les conventions Rust sont dans `docs/rust/PATTERNS.md` et les conventions TS dans le code lui-meme (ESLint config). **Recommandation** : ecrire un nouveau `CONVENTIONS.md` pour le codebase actuel.

### 6.6 Confusion nommage `research/STACK.md` vs `codebase/STACK.md`

**Alerte** : deux fichiers `STACK.md` existent :
- `.planning/research/STACK.md` (2026-05-18) — stack technique pour l'arc S73-S75 Factory
- `.planning/codebase/STACK.md` (2026-04-06) — stack pre-pivot (OBSOLETE)

La collision de noms peut creer de la confusion. **Recommandation** : supprimer le `codebase/STACK.md` obsolete, ou le renommer.

### 6.7 Confusion nommage `research/ARCHITECTURE.md` vs `codebase/ARCHITECTURE.md`

Meme probleme. Deux fichiers `ARCHITECTURE.md` :
- `.planning/research/ARCHITECTURE.md` — architecture Factory broker/sandbox
- `.planning/codebase/ARCHITECTURE.md` — architecture globale Rust workspace

Pas de conflit de contenu (scopes differents), mais le nommage identique peut creer de la confusion.

---

## 7. Recommandations

### 7.1 Actions immediates (menage)

1. **Supprimer 5 fichiers codebase obsoletes** :
   - `.planning/codebase/STACK.md` (pre-pivot Python)
   - `.planning/codebase/CONVENTIONS.md` (pre-pivot Python)
   - `.planning/codebase/TESTING.md` (pre-pivot pytest)
   - `.planning/codebase/CONCERNS.md` (pre-pivot tech debt)
   - `.planning/codebase/INTEGRATIONS.md` (pre-pivot integrations)

2. **Archiver 3 roadmaps pre-pivot** (deplacer vers `archive/v1.0/` ou un sous-dossier `archive/pre-pivot/`) :
   - `.planning/NEXUS_GOV_ROADMAP.md`
   - `.planning/OPEN_SOURCE_ROADMAP.md`
   - `.planning/DISTRIBUTED_GPU_ROADMAP.md`

3. **Archiver 6 research FOSSIL** (deplacer vers un sous-dossier `research/archive/` ou `archive/research/`) :
   - `research/SUMMARY.md`
   - `research/REACTIVE_ARCHITECTURE.md`
   - `research/DISTRIBUTED_GPU_RESEARCH.md`
   - `research/PIXEL_ART_AI_RESEARCH.md`
   - `research/DAY0_OPEN_QUESTIONS.md`
   - `research/sprint33_multinode_research.md`
   - `research/S49_coordinator_rust_migration.md`
   - `research/3d_space_simulation_catalog.md`

### 7.2 Document a creer

**Nouveau `codebase/CONVENTIONS.md`** pour le codebase actuel (Rust + TypeScript). Le template GSD existe. Devrait couvrir :
- Conventions nommage Rust (snake_case fonctions, PascalCase types, UPPER_SNAKE constants)
- Conventions nommage TypeScript (PascalCase composants, camelCase hooks/stores)
- Import organization (Rust: std → external → workspace → local; TS: react → external → local)
- Error handling (Rust: `anyhow::Result` + `.context()`, pas de `.unwrap()` hors tests)
- Commit format (`feat(scope): Sprint N Phase X -- titre`)
- Patterns communs (`docs/rust/PATTERNS.md` reference)

### 7.3 Organisation `research/` (sous-dossiers par arc)

La recommendation est de NE PAS creer de sous-dossiers thematiques. Raison :
- Les fichiers sont prefixes par sprint (S19, S21, S22, etc.) ce qui suffit pour le tri chronologique
- Les nouveaux docs S65-S75 sont prefixes par sprint range (`s65_`, `s66_`, `s70_s72_`, etc.)
- Les docs de vision produit ont des noms explicites (`babel_`, `rrv_`, `sbfb_`)
- Un sous-dossier `research/archive/` pour les FOSSIL suffit

**Structure recommandee** :
```
research/
  archive/                              # FOSSIL (8 fichiers)
  S19_phase_B_pow_hashcash_design.md    # RESEARCH vivant
  S19_phase_C_tls_cert_pinning_design.md
  ...
  s65_contrat_public_research.md        # CANON sprint research
  s66_durabilite_research.md
  ...
  sbfb_rrv_code_factory_vision_pitch.md # CANON vision
  STACK.md                              # CANON (arc Factory)
  FEATURES.md                           # CANON (arc Factory)
  ARCHITECTURE.md                       # CANON (arc Factory)
  PITFALLS.md                           # CANON (arc Factory)
```

### 7.4 Roadmap formelle

La roadmap formelle `roadmap_v2_public_trust_rrv_factory.md` n'existe pas encore comme fichier. Elle devrait etre placee a :

**`.planning/roadmap_v2_public_trust_rrv_factory.md`** (racine `.planning/`, au meme niveau que les anciennes roadmaps)

Son contenu devrait synthetiser :
- `s65_s75_cross_cutting_research.md` (sequencage, carries, risques)
- Les 3 arcs (Confiance publique S65-S69, RRV S70-S72, Factory+Babel S73-S75)
- Les gates et points de decision entre arcs
- L'etat actuel (S64 CLOSED, S65 prochain)

### 7.5 Mettre a jour `.planning/README.md`

- Remplacer "snapshot 2026-04-06" par "snapshot 2026-05-18"
- Ajouter `v2.0/` dans le layout archive
- Mentionner les 8 nouveaux docs codebase + 7 docs research S65-S75

---

## 8. Statistiques resumees

| Categorie | Canon | Research | Superseded | Fossil | Active | Total |
|---|---|---|---|---|---|---|
| `codebase/` | 8 | 0 | 5 | 0 | 0 | 13 |
| `research/` | 12 | 28 | 3 | 8 | 0 | 51 |
| `research/` (S65-S75 specifiques) | 4 | 0 | 0 | 0 | 0 | 4 |
| `active/` | 0 | 0 | 0 | 0 | 16 | 16 |
| racine `.planning/` | 1 | 0 | 2 | 3 | 0 | 6 |
| **Total** | **25** | **28** | **10** | **11** | **16** | **90** |

**Actions requises** : 5 suppressions codebase + 8 archivages research FOSSIL + 3 archivages roadmaps pre-pivot + 1 document a creer (CONVENTIONS.md) + 1 README a mettre a jour + 1 roadmap formelle a creer.

---

*Classification audit: 2026-05-18*
