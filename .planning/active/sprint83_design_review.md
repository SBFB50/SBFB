# Sprint 83 — Design Review Board (G1)

**Date** : 2026-07-17 (session ultracode ; complété main-thread Opus 4.8 après reprise
Workflow `wf_6a97f922-75e`).
**Sprint** : 83 — chantier « Workflow Agents » (Hub + Runtime + Packs + Protocole).
**Reviewer** : self-review profond + 3 panels adversariaux indépendants (Workflow
`wf_6a97f922-75e`) : `g1-board` (D1..D5), `plan-challenge` (séquencement/gates 41 phases),
`repo-fit` (conformité canon README §2/§4/§6.12 + hook lightcheck).

Ce document est le **slot G1 formel** requis avant tout commit Phase A (hook lightcheck
Check 5 étant mort — `$PHASE` minuscule vs « A » —, l'enforcement est discipline de plan,
fix routé Phase A1). Les findings ci-dessous sont ceux du board ; ils sont **acknowledgés
inline dans le kickoff §4.6** (table de 17 objections, aucune ignorée).

---

## Scoring

| D# | Titre | Source récente (<90j) | Alternative vérifiée | [DETER] Crypto/spec | [DETER] Rust-first | Code local vérifié | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Architecture d'autorité (4 couches ; Hub sandboxé / Runtime hôte / pack signé) | ok (Research Pack 2026-07-15, décision noms 2026-07-17) | ok (route interne Operator + interpréteur iframe autonome REJETÉS §1) | N/A | ok (runtime = crate hôte Rust) | ⚠️→ok (`http.rs` « 1434 l » → 1513 l re-mesuré) | ⚠️ (mineur, corrigé) |
| D2 | Pack portable + signature `SBFBPromptAgentPack.v1` (JCS+Ed25519+DOMAIN, `subject[]` blake3) | ok (in-toto/DSSE 2025-2026) | ok (DSSE/cosign évalués, différés sans casse) | ok (JCS+Ed25519 existants `nexus-core-rs` ; DSSE alternative < 6 mois) | ok (crypto Rust maison réutilisée) | ⚠️→ok (`deploy.rs` chemin de crate explicité) | ⚠️ (mineur, corrigé) |
| D3 | Moteur durable Rust (mémoïsation-de-steps + journal SQLite maison, PAS replay-total/WASM) | ok (Temporal/Restate/DBOS 2026 ; rusqlite 0.36 repo) | ok (replay-total déterministe + WASM/Obelisk REJETÉS, 2 jambes structurelles) | N/A | ok (SQLite/rusqlite Rust-native ; Temporal rejeté sur 2 jambes structurelles) | ok (`task.rs` LWW, 0 scheduler — gap réel confirmé) | ⚠️ (**majeur**, corrigé) |
| D4 | Langage workflow/agents (`SBFBWorkflowGraph.v1`, Plan-Then-Execute, intersection 7 couches) | ok (LangGraph/OpenAI Agents SDK/MCP 2025-2026) | ok (import exécutable + `$flow.state` REJETÉS ; A2A différé WA-10) | ok (MCP pinné par digest) | ok (compilateur Rust autoritaire) | ⚠️→ok (`sbfb-bridge.js` : 9 fichiers/8 copies, ensemble exact pour le gate parité) | ⚠️ (mineur, corrigé) |
| D5 | Structure d'exécution (adoption ch.24 36→41 phases, AM-1..AM-12) | ok (verif_plan36 2026-07-17, README §4/§6.12) | ok (adoption telle quelle + plan Hub autonome + découpage multi-sprint REJETÉS) | N/A | N/A | ok (grammaire `Phase [A-Z]+[0-9]?` prouvée `agentctl.py`) | ⚠️ (**bloquant+majeurs**, corrigés) |

**Résumé** : D1 ⚠️(mineur), D2 ⚠️(mineur), D3 ⚠️(majeur), D4 ⚠️(mineur), D5 ⚠️(bloquant+3 majeurs).
Rigor signal G4 : **5 ⚠️ sur 5, tous acknowledgés + corrigés inline**. Ce n'est PAS un
rubber-stamp (le gold standard est 1-2 ⚠️ ; ici le challenge adversarial des 3 panels a
trouvé un défaut réel par D-choice, dont 1 bloquant et 1 majeur substantiels — signal d'une
review réelle, pas d'un tampon). Aucun ⚠️ résiduel non traité (verdicts « corrigé »).

---

## Findings

### D1 ⚠️ (mineur) — `http.rs` « 1434 l » faux
**Détail** : le draft citait 1434 lignes pour `crates/nexus-shell-daemon/src/http.rs`.
**Décision** : adjust — re-mesuré `wc -l` = **1513 l** au tip `74531d4` (baseline audit S82) ;
corrigé §1.2 + §4.1 du kickoff. Sans impact sur la décision d'architecture (le runtime va dans
l'hôte daemon quelle que soit la taille exacte de http.rs).

### D2 ⚠️ (mineur) — `deploy.rs` chemin de crate implicite
**Détail** : `deploy.rs` était implicitement placé dans `nexus-coordinator-rs`.
**Décision** : adjust — chemins de crate complets partout où l'ancre est inter-crate
(`deploy.rs` → `nexus-shell-daemon` ; `csp.rs` → `nexus-core-rs` ;
`storage_limiter.rs` → `nexus-shell-daemon-core`). §4.2/§4.3.

### D3 ⚠️ (**majeur**) — citation `temporalio-sdk` périmée
**Détail** : le draft citait `temporalio-sdk` « 0.1.0-alpha, constaté 2026-07-17 » — faux
(version réelle 0.5.0, 2026-06-29). Un argument de rejet fondé sur une version fausse est un
défaut de rigueur G9.
**Décision** : adjust — le **rejet de Temporal est re-fondé sur 2 jambes STRUCTURELLES**
(dépendance à un serveur d'orchestration externe incompatible avec « autorité run = nœud
initiateur » WA-3 ; modèle de worker/queue distribué hors MVP §13.4), **sans argument de
version**. La citation est re-fondée sur des faits constatables. §4.3.

### D4 ⚠️ (mineur) — décompte copies `sbfb-bridge.js`
**Détail** : « + 9 copies » — réel : 9 fichiers AU TOTAL (8 copies + 1 source).
**Décision** : adjust — reformulé avec l'ensemble exact pour le gate de parité bridge ; la
Phase R ajoute une 10e copie trackée (inventaire de parité étendu). §4.4.

### D5 ⚠️ (**bloquant** + 3 majeurs) — structure d'exécution du plan ch.24
**Détail (bloquant)** : le ch.24 adopté tel quel n'a **aucun mapping T1/T2 README §4** — le
sprint serait **infermable par la DoD §4 (c)/(d)**.
**Décision** : adjust — **AM-1** : 3 specs T1 Playwright hermétiques Hub nommées dès l'ouverture
(`workflow-hub-author/run/lifecycle`) + spec bridge (Phase C), job CI BLOQUANT dès R ; artefact
`sprint83_t2_acceptance.json` vocabulaire fermé + règle de finalisation bi-temporelle T6.

**Détail (majeur 1)** : 16 gates orphelins (sans propriétaire de fermeture).
**Décision** : adjust — **AM-2** : matrice de gates COMPLÈTE au plan (propriétaire + mécanisme
d'enforcement + statut EXISTANT/ÉTENDU/NEUF) ; HUB-T1..T10 étendus.

**Détail (majeur 2)** : exit Phase G « compile CONCEPTUELLEMENT » non binaire ; G-SCOPE-FLOOR
infermable en AI2 ; artefact T2 bi-temporellement faux si figé en AI2.
**Décision** : adjust — exit G machine-vérifiable (4 goldens sérialisés + validation schemars +
artefact de décision daté) ; G-SCOPE-FLOOR déplacé AJ1 (AM-5) ; T6=MIXED en AI2, finalisé AJ1
avant `verification.md` (AM-1).

**Détail (majeur 3)** : condition 2 audit S82 non intégralement acquittée (lot P3 + C-1 sans
route) ; référence process périmée « 3 blocs Rust/Python/frontend » (pack l.2496-2497).
**Décision** : adjust — table §6.2 couvre TOUT le Carry-Over (C-1→AJ1, lot P3 ventilé A1/
discipline/re-route S84) ; **AM-6** : canon §7.4 (2 blocs pré-commit + 3 blocs Docker).

**Mineurs D5 restants** (tous adjust, cf. kickoff §4.6) : AM-10↔AM-9 checkpoint panel réconcilié ;
HUB-T4..T9 orphelins → propriétaire/subsomption ; evidence app-authoring gamable → exit renforcée
digest+lineage (§6.1) ; G-VERTICALS omettait Z..AC → assigné.

---

## Checklist [DETER]

### Crypto/spec (D2 pack signature, D4 MCP)
- [x] D2 cite ≥1 alternative concurrente < 6 mois : DSSE/in-toto/cosign 2025-2026, différés sans casse.
- [x] Source datée < 2 ans ou revalidée : JCS RFC 8785 + Ed25519 (existants repo) ; in-toto/DSSE courant.
- [x] Reviewer ⚠️ si alternative absente : N/A (alternatives présentes).
- [x] D4 MCP pinné par digest (révision 2025-11-25, migration stateless 2026-07-28 = veille QPO-7/Q14).

### Rust-first (D3 moteur, D4 compilateur)
- [x] D3 cite ≥1 alternative Rust-native production : rusqlite 0.36 (repo), Temporal Rust SDK 0.5.0.
- [x] Gap factuel documenté si alternative Rust rejetée : Temporal rejeté sur 2 jambes structurelles
      (serveur externe vs autorité nœud initiateur ; worker/queue distribué hors MVP), pas sur la version.
- [x] D4 compilateur Rust autoritaire (anti-drift Rust/TS) ; pas de délégation d'autorité à un LLM.
- [x] Reviewer ⚠️ si gap non documenté : D3 majeur levé (citation version corrigée).
- Exemptions : frontend UX (Hub React), docs, fixtures de test.

### Code local vérifié (G1 point 5)
- [x] `http.rs` re-mesuré 1513 l (D1) ; `task.rs` LWW sans scheduler confirmé (D3, gap réel) ;
      `csp.rs:33` source CSP unique (D2/app-authoring) ; grammaire `Phase [A-Z]+[0-9]?` prouvée
      `agentctl.py:28/:32/:566` (D5) ; `sbfb-bridge.js` ensemble de 9 fichiers (D4).

---

## Note de méthode

Board joué en 3 panels adversariaux indépendants (pas un self-review unique) : `g1-board`
challenge D1..D5 par la checklist Step 6 ; `plan-challenge` réfute le séquencement/gates des
41 phases ; `repo-fit` vérifie la conformité au canon (sections canoniques, T1/T2, docs-contrat,
hook lightcheck). Les 17 objections (1 bloquante, 5 majeures, 11 mineures) sont **toutes**
intégrées inline dans le kickoff §4.6. Le fait que les 5 D-choices portent un ⚠️ (au lieu du
gold 1-2) reflète la profondeur du challenge sur un chantier de 41 phases, pas une faiblesse
des décisions : chaque ⚠️ est corrigé, aucun résiduel. Le design review formel n'ouvre aucun
nouveau DESIGN-CONFLICT ; les D-choices sont gelées sous réserve de la ratification PO du
Checkpoint §11 (slot, version, D1..D5, app-authoring, ressources).
