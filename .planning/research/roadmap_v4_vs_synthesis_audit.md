# Audit Roadmap v4 vs Synthese Canon

**Date :** 2026-05-19
**Auditeur :** Claude (session de consolidation)
**Documents compares :**
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md` (DRAFT, 2172 lignes)
- `.planning/research/SYNTHESIS_factory_rrv_protocol.md` (CANON, 1994 lignes)

---

## 1. Les 16 decisions de la synthese §9.1

| # | Decision (synthese) | Reflet dans la roadmap v4 | Verdict |
|---|---------------------|---------------------------|---------|
| D1 | FTS5 d'abord, Tantivy gate post-S75 | P11 (principes) + D-GEL-5 + scope cut #9 | OK |
| D2 | Factory hors daemon (crate sbfb-factory) | P7 (revise) + P14 + D-GEL-10 + D-GEL-11 + S67 Phase B | OK |
| D3 | node_id retire de SBFB.json (Option D) | P12 + D-GEL-9 + S67 Phase A item 3 | OK |
| D4 | Feed raw-op extensible (pas de bump) | P9 + D-GEL-7 + S67/S70 explicites | OK |
| D5 | Babel = premier dogfood Factory (S69) | S69 entier + D-GEL-6 | OK |
| D6 | @protocole avant @dev avant @web | S67 Phase C (FTS5 daemon + RRV @dev simultanes) + scope cut #11 | **DIVERGENCE MINEURE** (voir ci-dessous) |
| D7 | SBFB.json v2 (schema_version: 2) | S67 Phase A item 3 + sbfb-manifest crate | OK |
| D8 | Gates Factory prefixees FG (FG0-FG10) | Section transversale FG0-FG10 | OK |
| D9 | CuratorVouched minimal en S65 (Option D faisabilite) | Synthese dit "minimal en S65", roadmap v4 dit S65 Phase A mais aussi S67 Phase A item 4 | **DIVERGENCE** (voir ci-dessous) |
| D10 | S66 OBLIGATOIRE avant S69 | S66 complet, D-HIDDEN-1/2 + dependances explicites | OK |
| D11 | Score de completude de preuve (pas "trust score") | P6 + S68 Phase A ProofCard formule | OK |
| D12 | Scope cuts Babel : pas de reviews, pas de task mock pour canari | S69 Phase A (scope cuts Babel) + D-GEL-6 | OK |
| D13 | Preview ephemere via daemon API, pas via Factory | S67 Phase A item 5 (preview/load) | OK |
| D14 | Pas de signature Ed25519 decomposee en S67-S69 | Pas mentionne explicitement dans la roadmap | **MANQUE** |
| D15 | SearchManifest = domain tag gele une fois deploye | S70 Phase A implicite (wire format) mais pas de mention explicite du gel | **MANQUE** |
| D16 | formula_version dans ProofCard | S68 Phase A — `formula_version: u16` present | OK |

### Detail des divergences

**D6 — Ordonnancement @protocole -> @dev -> @web.** La synthese (§4.2) recommande clairement de commencer par @protocole (FTS5 daemon sur les donnees existantes) AVANT @dev (index local workspace). La roadmap v4 planifie les deux dans le meme sprint S67 (Phase C = FTS5 daemon + RRV @dev ensemble). Ce n'est pas une contradiction directe puisque Phase C met bien le FTS5 daemon en premier dans la section, mais la co-localisation dans la meme phase dilue la priorite nette de la synthese. Le risque R-V4-6 de la roadmap adresse partiellement ceci en disant que Phase C est reportable en S68 si S67 deborde. **Severite : FAIBLE.**

**D9 — CuratorVouched "minimal en S65".** La synthese (D9) dit "CuratorVouched minimal en S65 (Option D faisabilite)" avec reference doc#7 §5. Mais S65 est DONE dans la roadmap, et la section S65 ne mentionne PAS CuratorVouched comme livrable. Les types CuratorVouched sont planifies en S67 Phase A item 4 de la roadmap. La decision D9 de la synthese parle d'une implementation qui n'a pas eu lieu en S65. La roadmap est plus realiste (S67), la synthese est en retard par rapport a la realite. **Severite : FAIBLE — la synthese reflete une recommandation qui n'a pas ete suivie ; la roadmap est correcte factuellement.**

**D14 — Pas de signature Ed25519 decomposee.** La synthese gele cette decision (deployer via deploy-from-repo monolithique, pas de signature client decomposee). La roadmap v4 ne mentionne nulle part cette decision ni ne la contredit — elle est simplement absente de la section "decisions gelees". **Severite : FAIBLE — absence d'enonce, pas de contradiction.**

**D15 — SearchManifest domain tag gele.** La synthese gele le domain tag `DOMAIN_SEARCH_MANIFEST_V1`. La roadmap v4 mentionne le domain constant dans S70 Phase A mais ne le marque pas explicitement comme "gele une fois deploye" dans les decisions gelees. **Severite : FAIBLE — implicite dans le design, pas explicitement codifie.**

---

## 2. Ordonnancement @protocole -> @dev -> @web (synthese §4.2)

| Etape synthese | Sprint synthese | Sprint roadmap v4 | Conforme ? |
|----------------|-----------------|---------------------|------------|
| @protocole (FTS5 daemon) | S67-S68 | S67 Phase C (daemon FTS5) | OUI |
| @dev (index local workspace) | S67-S69 en parallele avec Factory | S67 Phase C (RRV @dev dans sbfb-factory) | OUI |
| @web (SearXNG sidecar) | S72+ post-pilote | Scope cut #11 (hors scope S65-S75) | OUI (plus conservateur) |

**Verdict : CONFORME.** La roadmap respecte l'ordonnancement. Elle est meme plus conservatrice sur @web (hors scope total au lieu de S72+).

Note : la synthese §4.2 dit "@protocole d'abord" puis "@dev en parallele avec Factory des S67". La roadmap fusionne les deux dans S67 Phase C. C'est conforme car la synthese dit explicitement "en parallele" (Etape 2, S67-S69).

---

## 3. Les 4 primitives manquantes (synthese §2.5)

| Primitive | Route | Sprint synthese | Sprint roadmap v4 | Conforme ? |
|-----------|-------|-----------------|--------------------|------------|
| Feed read paginee | `GET /api/daemon/feed/entries` | S67 | S67 Phase A item 2 | OUI |
| Preview ephemere | `POST /api/v1/preview/load` | S68 | S67 Phase A item 5 | **AVANCEE** (mieux) |
| node_id optionnel dans deploy | Modification deploy.rs | S67 | S67 Phase A item 3 | OUI |
| CuratorVouched/CuratorDisendorsed dans feed | PublicFeedOperation | S67 | S67 Phase A item 4 | OUI |

**Verdict : CONFORME.** Les 4 primitives P0 sont planifiees. La preview/load est meme avancee de S68 a S67 Phase A (amelioration).

### Primitives P1 de la synthese

| Primitive | Route | Sprint synthese | Sprint roadmap v4 | Conforme ? |
|-----------|-------|-----------------|--------------------|------------|
| Manifest extraction | `GET /api/v1/project/{id}/manifest` | S67 | **NON PLANIFIEE** explicitement | **MANQUE** |
| Provenance list (batch) | `GET /api/v1/provenance/list` | S70 | **NON PLANIFIEE** explicitement | **MANQUE** |
| Search FTS5 | `GET /api/daemon/search` | S67-S70 | S67 Phase C | OUI |

**Severite des manques P1 :** FAIBLE. Ce sont des primitives P1, pas P0. La manifest extraction est partiellement couverte par sbfb-manifest crate + deploy.rs import. La provenance list est un raccourci pour RRV indexation batch — contournable via le feed replay.

### Primitives P2 de la synthese

| Primitive | Sprint synthese | Sprint roadmap v4 | Conforme ? |
|-----------|-----------------|---------------------|------------|
| Feed entry par hash | S68+ | NON PLANIFIEE | **MANQUE (P2)** |
| Webhook/subscribe feed (SSE/long-poll) | S68+ | NON PLANIFIEE (Annexe F mentionne le flux d'evenements comme "non planifie avant S68+") | **COHERENT** (reporte dans les deux) |

---

## 4. Proof Cards — bon sprint ?

| Document | Sprint Proof Cards |
|----------|--------------------|
| Synthese §11 Phase 3 | **S69** ("Phase A : ProofCard data model + computation dans coordinator") |
| Roadmap v4 | **S68** ("Phase A — ProofCard data model + computation + API") |

**DIVERGENCE.** La synthese place les Proof Cards en S69. La roadmap v4 les avance en S68. La comparaison v3 vs v4 dans la roadmap (ligne 2056) confirme : "Proof Cards : S71 (arc 3) -> S68 (arc 2)".

La synthese §11 est en retard par rapport au pivot v4. La synthese date du 2026-05-19 mais reflete la sequence §11 "Phase 3 : Proof Cards + Babel dogfood (S69)" qui correspond au plan d'avant le pivot co-dev. La roadmap v4 a avance les Proof Cards car elles dependent de FTS5 (S67) mais pas de SearchManifest, donc S68 est viable.

**Severite : MOYENNE — la synthese §11 ne reflete pas le resequencement v4. La roadmap est plus coherente avec les dependances reelles.** L'ordonnancement de la roadmap v4 est le bon.

---

## 5. Tests d'acceptance Babel (25 items) — distribution

### Distribution synthese §14.4 vs roadmap v4

| Test # | Synthese §14.4 | Roadmap v4 (section finale) | Conforme ? |
|--------|----------------|-----------------------------|------------|
| 1-8 | S67 (~4j) | #1-4: S69/B, #5-7: S69/A, #8: S69/C | **DIVERGENCE** |
| 3-4, 11-12, 19 | S68 (~3j) | #3-4: S69/B, #11: S69/C, #12: S69/A, #19: S69/A | **DIVERGENCE** |
| 13-18, 20 | S69 (~4j) | #12-16: S69/A, #17-18: S69/B, #19: S69/A | Partiellement conforme |
| 21-25 | S70-S71 | #20: S67/B, #21-22: S69/B, #23-24: S68/A, #25: S69/D | **DIVERGENCE** |

**DIVERGENCE SIGNIFICATIVE.** La synthese distribue les 25 tests sur S67 (#1-8), S68 (#3,4,11,12,19), S69 (#13-18,20), et S70-S71 (#21-25). La roadmap v4 concentre presque tous les tests en S69, avec seulement #20 en S67 et #23-24 en S68.

Analyse : la roadmap v4 reporte les tests d'acceptance 1-19 en S69 car c'est le sprint Babel (le test de l'app reelle). La synthese les distribue sur les sprints ou les primitives sous-jacentes sont implementees (ex: test "deploy sans node_id" en S67 quand node_id est rendu optionnel). Les deux approches sont valides — la synthese teste les primitives tot, la roadmap teste l'app Babel quand elle existe.

**Severite : FAIBLE pragmatiquement, MOYENNE en rigueur.** La roadmap v4 est plus pragmatique (les tests Babel ont besoin de Babel). La synthese est plus rigoureuse (tester chaque primitive des son implementation). Les tests #23-24 (Proof Cards) en S68 Phase A dans la roadmap sont coherents avec le deplacement des Proof Cards en S68.

Le test #6 de la synthese ("app generee contient planning sprint") n'apparait PAS dans la roadmap v4. La roadmap v4 renumerate certains tests differemment.

---

## 6. Carries S66 — gestion

| Carry item | Synthese | Roadmap v4 | Conforme ? |
|------------|----------|------------|------------|
| P2-FEED-JOIN-HANDLE-LEAK (2/3) | Annexe E prerequis S66 Phase C | S66 Phase C | OUI |
| P2-ORPHAN-REPUBLISH-RECOVERY (2/3) | Annexe E prerequis S66 Phase C | S66 Phase C | OUI |
| P2-PROVENANCE-404-BRIDGE (3/3) | doc#9 §5 (bloqueur publish path) | S68 Phase B | OUI |
| P2-VERIFY-LOCAL-KEY-ONLY (3/3) | doc#9 §7 implicite | S69 Phase C | OUI |
| P2-PLAYWRIGHT-SPECS-STALE (partie 2) | Non mentionne dans synthese | S69 Phase D | N/A (hors synthese) |
| P2-A-1 rand blocker | Non mentionne dans synthese | Monitoring continu | N/A |
| P2-AUDIT-2 iroh transitives | Non mentionne dans synthese | Monitoring continu (Gate 1) | N/A |
| P2-G-1 exe lock | Non mentionne dans synthese | Monitoring continu | N/A |
| P2-THREAT-MODEL-FEED-SURFACE | Non mentionne dans synthese | Hors scope S65-S75 (note : "A adresser en S67 Phase A ou S70") | N/A |
| LT-2 Radicle | Non mentionne dans synthese | Trigger-dependent | N/A |

**Verdict : CONFORME.** Tous les carries pertinents sont geres. Les carries non mentionnes dans la synthese sont bien dans la section carry de la roadmap.

---

## 7. Sequence §11 de la synthese vs phases de la roadmap

| Etape synthese §11 | Contenu | Correspondance roadmap v4 | Conforme ? |
|--------------------|---------|---------------------------|------------|
| Pre-requis S66 | Persistence | S66 (5 phases A-E) | OUI |
| Phase 1 (S67) | Primitives daemon + @protocole FTS5 | S67 (4 phases A-D) | **PARTIEL** |
| Phase 2 (S68) | sbfb-factory + templates + @dev | S68 Proof Cards + Publish Gate + UX | **DIVERGENCE** |
| Phase 3 (S69) | Proof Cards + Babel dogfood | S69 Babel Canari + Pilote | **DIVERGENCE** |
| Phase 4 (S70-S72) | SearchManifest + Gouvernance UI | S70 SearchManifest + S71 Gouvernance + S72 Reserve | **PARTIEL** |
| Phase 5 (S73+) | @web + Hardening | S73-S75 Bridge + Proof Pack + Release | **PARTIEL** |

### Divergences detaillees

**Phase 1 vs S67 :** La synthese §11 Phase 1 met FTS5 daemon search dans S67 Phase A. La roadmap v4 met FTS5 daemon dans S67 Phase C. La synthese §11 ne mentionne pas RRV @dev dans S67, mais §4.2 dit "Etape 2 (S67-S69, en parallele)" — la roadmap integre RRV @dev dans S67 Phase C. **Compatible mais reorganise.**

**Phase 2 vs S68 :** La synthese §11 Phase 2 est "sbfb-factory + templates + @dev" en S68. La roadmap v4 S68 est "Proof Cards + Publish Gate + UX Confiance". Factory est deja en S67 (Phases B-D) dans la roadmap. La synthese a un decalage d'un sprint pour Factory. **DIVERGENCE MOYENNE** — la roadmap avance Factory d'un sprint par rapport a la synthese §11.

**Phase 3 vs S69 :** La synthese place ProofCard en S69 Phase A. La roadmap v4 les met en S68 Phase A. Babel dogfood reste en S69 dans les deux. **DIVERGENCE MOYENNE** — Proof Cards avancees d'un sprint dans la roadmap.

**Phase 4 :** La synthese dit "S70 Gouvernance Full UI, S71 SearchManifest, S72 Discovery". La roadmap v4 dit "S70 SearchManifest, S71 Gouvernance + Factory Hardening, S72 Reserve". L'ordre Gouvernance/SearchManifest est inverse. **DIVERGENCE** — la roadmap met SearchManifest avant Gouvernance, la synthese fait l'inverse.

**Phase 5 :** La synthese dit "S73 Factory hardening + templates + 2eme app, S74 Babel translation beta, S75 Pack produit". La roadmap dit "S73 Bridge avance + Domain Packs, S74 Proof Pack Structure, S75 Pack Produit". La synthese mentionne Babel translation beta en S74 ; la roadmap la deplace en S72 (reserve) ou S73. **DIVERGENCE FAIBLE.**

**Synthese severite globale §11 : MOYENNE.** La sequence §11 de la synthese est un plan anterieu au pivot v4 complet. La roadmap v4 reorganise significativement les sprints S67-S72 par rapport a la synthese §11. Les deux documents sont du meme jour (2026-05-19) mais la synthese §11 ne reflete pas le resequencement final.

---

## 8. Gates FG0-FG10 — implantation dans les bons sprints

| Gate | Synthese §3.4 (sprint cible implicite) | Roadmap v4 (section transversale) | Conforme ? |
|------|----------------------------------------|-----------------------------------|------------|
| FG0 Classification | S67 (Factory Foundation) | S67 Phase B | OUI |
| FG1 Scope | S67 | S67 Phase B | OUI |
| FG2 Template | S67 | S67 Phase B | OUI |
| FG3 Manifest | S67 | S67 Phase A (sbfb-manifest) | OUI |
| FG4 Diff | S68 | S68 Phase B | OUI |
| FG5 Sandbox | S68 | S68 Phase B | OUI |
| FG6 Secrets/deps | S68 | S68 Phase B | OUI |
| FG7 Preview | S68 | S68 Phase D | OUI |
| FG8 Provenance | S67-S68 | S68 Phase D | OUI |
| FG9 Publish | S69 | S69 Phase B | OUI |
| FG10 Review | S69 | S69 Phase E | OUI |

**Verdict : CONFORME.** Tous les gates FG0-FG10 sont planifies dans les bons sprints. Les LOC estimes dans la synthese (~1840 total) sont coherents avec les estimations de la roadmap.

La synthese mentionne des gates manquantes (limite taille archive, verification bridge runtime, invariant template_hash, verification entrypoint, deduplication). La roadmap v4 ne les mentionne pas explicitement mais les couvre implicitement :
- Limite taille archive : dans FG5/FG6 (synthese §3.4, doc#8 §4.1) — **MANQUE dans la roadmap**
- Verification bridge runtime : **MANQUE explicite dans la roadmap**
- Invariant template_hash FG2/FG8 : **MANQUE explicite dans la roadmap**
- Verification entrypoint : couverte par preview/load (index.html check)
- Deduplication : couverte par BLAKE3 content hash

**Severite : FAIBLE.** Les 3 manques sont des enrichissements recommandes par la synthese, pas des gates critiques.

---

## 9. Risques P0 de la synthese §13.1

| Risque P0 (synthese) | Roadmap v4 couverture | Conforme ? |
|-----------------------|----------------------|------------|
| Factory n'existe pas, publish path depend d'elle | Plan B dans S69 Phase E (Babel a la main si Factory glisse) + R-V4-1 | OUI |
| Feed ReleasePublished non auto-insere dans deploy-from-repo | S65 Phase A l'a resolve (deploy->feed wiring) + S69 Phase B verification | **NOTE** : la synthese dit "~40 LOC dans deploy.rs, combler en S67-S68" mais S65 Phase A l'a deja fait. |
| Persistence blobs volatile (MemStore) | S66 Phase B (FsStore) | OUI |
| node_id obligatoire dans SBFB.json | S67 Phase A item 3 | OUI |

**Risque P0 "Feed ReleasePublished" :** La synthese (doc#9 §5) le marque P0 "a combler en S67-S68". Mais la roadmap v4 S65 dit deja "deploy->feed wiring ReleasePublished" dans Phase A. Si S65 Phase A l'a reellement implemente, le risque est deja resolu. La synthese est en retard.

**Verdict : CONFORME.** Tous les risques P0 sont couverts.

---

## 10. Questions ouvertes Q1-Q17 (synthese §10)

| # | Question | Roadmap v4 reponse | Verdict |
|---|----------|---------------------|---------|
| Q1 | Factory dans workspace nexus ou repo sibling ? | D-GEL-10 : dans le workspace, crate separe. Extraction post-S75. | **RESOLU** |
| Q2 | Premier template : static-minimal, static-storage, ou HTML pur ? | S67 Phase B : `static-minimal` + `static-storage` | **RESOLU** |
| Q3 | Copier comme binaire externe ou logique interne ? | S67 Phase B : logique interne (include_str!, substitution) | **RESOLU** (implicitement) |
| Q4 | Embeddings @dev : absents MVP ou Ollama local ? | Scope cut #12 : sqlite-vec derriere feature flag, pas dans scope MVP | **RESOLU** |
| Q5 | Format exact factory.provenance.json ? | S67 Phase B/D : format SBFB (schema_version, hashes, signature) | **PARTIELLEMENT RESOLU** — format defini, choix in-toto vs SBFB non explicite |
| Q6 | Niveau confirmation utilisateur deps.install ? | Non adresse explicitement | **OUVERT** |
| Q7 | Tag perimetre RRV : tape ou selectionne UI ? | Non adresse explicitement | **OUVERT** |
| Q8 | @web defaut ou consentement ? | Scope cut #11 : @web hors scope S65-S75 | **REPORTE** (acceptable) |
| Q9 | Groupes prives : feed chiffre ou feed separe ? | Scope cut #4 : hors scope S65-S75 | **REPORTE** (acceptable) |
| Q10 | Publication resultat prive vers public ? | Scope cut #4 | **REPORTE** |
| Q11 | Seuil consentement GPU Babel continu ? | Non adresse | **OUVERT** |
| Q12 | project_id pour plusieurs apps d'un meme daemon ? | Non adresse explicitement | **OUVERT** |
| Q13 | Babel canari : repo public requis ou local-only ? | S69 Phase B : deploy-from-repo (implique repo public) | **RESOLU** (implicitement) |
| Q14 | Page React /factory : page shell, Factory serveur, ou CLI only ? | S68 Phase C : page shell React (Option A) + scope cut #13 (pas de serveur local) | **RESOLU** |
| Q15 | Flux d'evenements daemon (WebSocket/named pipe) ? | Annexe F de la synthese reference dans la roadmap. Non planifie avant S68+. | **REPORTE** (coherent) |
| Q16 | Tantivy ~0.22 ou ~0.23 (MSRV 1.94) ? | Hors scope (Tantivy = gate conditionnel) | **REPORTE** (coherent) |
| Q17 | SearchManifest dans feed : nouveau FEED_FORMAT_VERSION ? | D-GEL-7 + P9 : NON, raw-op extensible | **RESOLU** |

**Bilan :** 8 resolues, 4 reportees (coherent avec scope cuts), 4 ouvertes (Q6, Q7, Q11, Q12).

**Severite des questions ouvertes :** Les 4 ouvertes sont P2-P3, non bloquantes pour S67. Q12 (project_id multi-apps) est P1 et pourrait devenir un probleme en S67 si Factory genere plusieurs apps sur le meme daemon. **Recommandation : adresser Q12 dans le plan S67.**

---

## Table recapitulative des divergences

| # | Divergence | Synthese dit | Roadmap v4 dit | Severite | Impact |
|---|-----------|--------------|----------------|----------|--------|
| DIV-1 | Proof Cards sprint | S69 (§11 Phase 3) | S68 (Phase A) | MOYENNE | Roadmap est plus coherente avec les dependances. Pas un probleme. |
| DIV-2 | Sequence §11 Phase 2 vs S68 | "sbfb-factory + templates + @dev" en S68 | S67 contient Factory, S68 contient Proof Cards | MOYENNE | Synthese §11 en retard par rapport au resequencement v4. |
| DIV-3 | Tests acceptance Babel distribution | #1-8 en S67, #3/4/11/12/19 en S68 | Presque tout en S69 | FAIBLE | Roadmap plus pragmatique (teste Babel quand Babel existe). |
| DIV-4 | Gouvernance vs SearchManifest ordre | S70 Gouvernance, S71 SearchManifest | S70 SearchManifest, S71 Gouvernance | MOYENNE | Ordre inverse. Les deux sont valides (pas de dependance stricte). |
| DIV-5 | CuratorVouched en S65 vs S67 | D9 dit "minimal en S65" | S65 ne le contient pas, planifie en S67 Phase A | FAIBLE | La realite (S65 DONE sans CuratorVouched) prime. |
| DIV-6 | Decision D14 (signature decomposee) | Gelee, documentee | Non mentionnee dans les decisions gelees | FAIBLE | Absence, pas contradiction. |
| DIV-7 | Decision D15 (domain tag gele) | Gelee, documentee | Non mentionnee explicitement comme gelee | FAIBLE | Implicite dans le design. |
| DIV-8 | Primitives P1 manquantes | Manifest extraction + Provenance list | Non planifiees explicitement | FAIBLE | P1, contournables. |
| DIV-9 | Gates manquantes (taille archive, bridge runtime, template_hash invariant) | Identifiees dans §3.4 | Non explicites dans la roadmap | FAIBLE | Enrichissements recommandes. |
| DIV-10 | Feed ReleasePublished auto-insert | P0 "a combler S67-S68" | Deja fait en S65 Phase A | FAIBLE | Synthese en retard vs realite. |
| DIV-11 | Test acceptance #6 (planning sprint) | Present dans la liste des 25 | Absent de la distribution finale de la roadmap | FAIBLE | Test mineur manquant. |
| DIV-12 | Q12 project_id multi-apps | P1, sprint S67 | Non adresse | FAIBLE-MOYENNE | Pourrait devenir probleme en S67. |

---

## Verdict

### Metriques

| Metrique | Valeur |
|----------|--------|
| Decisions D1-D16 | 13/16 presentes, 3 absentes (D14, D15 mineures, D9 decalee) |
| Primitives P0 (4) | 4/4 planifiees |
| Primitives P1 | 1/3 planifiee (FTS5), 2 manquantes |
| Gates FG0-FG10 | 11/11 planifiees dans les bons sprints |
| Risques P0 | 4/4 couverts |
| Questions ouvertes | 8 resolues, 4 reportees, 4 ouvertes (dont 1 P1) |
| Carries S66 | Tous geres |
| Divergences totales | 12 identifiees (0 HAUTE, 4 MOYENNE, 8 FAIBLE) |

### Analyse

Les deux documents sont tres largement coherents. Les divergences sont principalement dues au fait que la synthese §11 (sequence de travail recommandee) n'a pas ete mise a jour avec le resequencement final de la roadmap v4 (Proof Cards avancees en S68, Factory avancee en S67, ordre Gouvernance/SearchManifest inverse). Ce sont des divergences de planning, pas d'architecture ni de design.

Les manques identifies (D14, D15, primitives P1, gates enrichies, Q12) sont mineurs et non bloquants pour le demarrage de S66/S67.

La roadmap v4 est globalement plus coherente que la synthese §11 en termes d'ordonnancement car elle respecte mieux les dependances reelles (FTS5 avant Proof Cards, Factory avant Babel, SearchManifest comme extension reseau post-Proof Cards).

### Verdict final

**PASS** — avec 4 recommandations mineures :

1. **R1 (MOYENNE)** : Harmoniser la section §11 de la synthese avec le sequencement final de la roadmap v4, ou marquer §11 comme "plan initial, supersede par la roadmap v4".
2. **R2 (FAIBLE)** : Ajouter D14 et D15 aux decisions gelees de la roadmap v4 pour completude.
3. **R3 (FAIBLE)** : Adresser Q12 (project_id multi-apps) dans le plan S67 avant que Factory ne genere plusieurs apps.
4. **R4 (FAIBLE)** : Ajouter les 3 gates enrichies (taille archive, bridge runtime, template_hash invariant) dans la section transversale FG de la roadmap.

---

*Audit effectue sur les documents du 2026-05-19. Les deux documents sont du meme jour, ce qui explique les divergences residuelles de synchronisation.*
