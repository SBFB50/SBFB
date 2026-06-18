# Sprint 76 — Phase G preflight (G8) — wrap-up + clôture Arc 3.5 6/6

> Produit par un Workflow ultracode (fan-out 5 scans read-only +
> synthèse adversariale). 6 agents, ~590K tokens subagent, 90 tool-uses.
> Phase G = wrap-up DOC-ONLY (0 code feature). Source de vérité du verdict.

## Verdict : **PLAN-ADAPT**

Le plan §10 G.1–G.5 tient dans sa substance. Deux corrections evidence-based
(qui ne touchent **aucune** décision Day-0, donc PLAN-ADAPT et pas
DESIGN-CONFLICT, pas d'arbitrage user requis) :

- **(A) Acceptance LIVE = DIFFERE-trace-user, jamais 38/38.** Le checkpoint
  plan §1 (« 38/38 fail-fast verts dont acceptance LIVE B-3 row #26 + quorum
  row #30, gate AVANT push ») traite les rows LIVE comme bloquantes-vertes,
  en tension directe avec la posture honnête DIFFERE-matériel-opérateur déjà
  adoptée par les reviews C/D et par `verification.md §3/§5`. Ces deux rows
  exigent le matériel opérateur (PC RTX 5080 + VPS Hetzner + Mac + SSH/WAN)
  **absent de l'environnement de session** → l'agent ne peut PAS les claim
  sans faux-vert. Posture à écrire (précédent S74 dual-platform) :
  **36/38 rows verts en session + 2 LIVE (#26/#30) DIFFERE-trace-user +
  row #6 Docker canonique re-joué à la recovery AVANT push.**

- **(B) Harness palier 2 non-runnable → ajouter un paramètre `REDUNDANCY`.**
  `scripts/acceptance/b3_live_pc_vps.sh` hardcode `redundancy_factor:1`
  (ligne 119) et son en-tête dit seulement que le palier 2 « reuses this
  harness » sans exposer d'override ni de 2e worker. Recommandation FORTE :
  ajouter un paramètre `REDUNDANCY` (défaut 1) passé à `redundancy_factor`
  dans le submit + une section d'enrôlement d'un 2e worker homogène (Mac,
  même MODEL/quant) pour rendre le **row #30 réellement exécutable par le
  user**. Sinon, consigner row #30 comme acceptance-user-différée
  non-runnable-sans-ajout. → **Option retenue : ajouter le paramètre**
  (option la plus profonde, rend le palier 2 falsifiable par l'opérateur).

## Les 4 dimensions factuelles — VERTES

1. **Test-count reconcilié sans faux-vert.** Rust Win nextest
   1763 +4(A) +8(B) +10(C) +4(D) +10(E) +5(F) = **1804** (somme deltas 41 ;
   chaque intermédiaire de §2 exactement reproduit). Vérif git indépendante :
   compter les `#[test]`/`#[tokio::test]` ajoutés par commit donne A+4
   (`ce43894`) / B+8 (`6904cdd`) / C+10 (`1cc28e7`) / D+4 (`d75ae77`) / E+10
   (`768e235`) / F+5 (`a547de6`) — **IDENTIQUE** aux deltas annoncés, donc
   aucune phase ne claim un test non écrit. Vitest 379 +7 +10 +0 +0 +1 +1 =
   **398**. Réalité (+41 Rust / +19 Vitest) dépasse l'estimate plan
   (~28 / ~10) ; acceptable (tildes `~` + dépassement concentré C/E expliqué
   par les fix root-cause documentés D-bridge-dedup / E-gen_time_ms).
2. **0 bump wire prouvé** sur tout le range `3faee6e..HEAD` : tous les
   `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` / `SCHEMA_VERSION` = 1 ;
   `RuntimeTuple` strictement additif `#[serde(default)]` Option ; `canonical.rs`
   0 changement ; 0 delta dépendance.
3. **6 carries hérités S75** attendus par §10 G.1 présents/tracés à leur source.
4. **3 carries 2-reports FERMÉS en Phase B** (`6904cdd`, Codex 12/12) :
   CARRY-3 (B2 downgrade `trustworthy_open_source` à l'ingress
   `handle_project_announcement`), LOOPBACK-TIERS (B7 7 routes §3), PULL-3
   (B3 `build_seed_fetch_chain` failover ordonné). → sortent du registre.

## Carries reconduits S77 (pour `sprint77_audit_plan.md` §3)

| # | Carry | Compteur | Exemption / note |
|---|---|---|---|
| 1 | **SYBIL-SEEDER-TAIL** | 2/3 → 3/3 si non fait | **EXEMPTION NOMMÉE « dépendance interne sharding »** (S77 touche dial-set/topology, le sampling se regroupe ; résiduel availability-only non-sécuritaire, ancre slot-0 non-crowdable). Seul 2-report reconduit. Sans exemption = MANDATORY. |
| 2 | REVISION-HOME-DURABILITY | 1/3 → 2/3 | pas d'exemption ; mitigé systemd SBFB_HOME épinglé ; pas exploitable pre-launch |
| 3 | KNOWN-ENTRY-OVERCOUNT | 1/3 → 2/3 | pas d'exemption ; superset HONNÊTE ; dedup (pid,hash) requis seulement si UX « N apps découvrables » |
| 4 | seeder `catalog_len:0` | 1/3 → 2/3 | pas d'exemption ; bloqué sur **arbitrage PO design** (section « seeded » distincte non-autoritaire vs verrou-4 seeder≠éditeur + F-Droid) |
| 5 | RE-DRIVE-ON-INGEST | 1/3 → 2/3 | pas d'exemption ; remède opérateur (restart) ; lié SeedAnnounced/PULL-3, peut se fermer en cascade |
| 6 | T-NN+3 (canonical_bytes dup JCS) | open S70 | absorbable au prochain sprint touchant JCS crypto |
| 7 | **P3-D-3** (send-failure un-mark `seen.remove` sans test) | 1/3 NOUVEAU | route EXPLICITE `sprint76_phase_d_review.md:303` ; ABSENT de §10 G.1 → à ajouter |
| 8 | **MEDIAN-DE-GROUPE** anti-gaming (D4-Q option a non implémentée) | DOC-P2 NOUVEAU | sanity-bound per-entry livré ; option médiane différée ; ABSENT de §10 G.1 → à ajouter |
| — | SYBIL-FORGE-COHERENTE résiduel quorum | surveillé | documenté §15.2/§15.3 ; PAS un carry actionnable seul (coût Sybil pré-existant PoW/AgeWitness + pilote fermé) |

**Externes inchangés** (escalade G7 < 3 reports) : P2-A-1 rand (exemption upstream),
P2-AUDIT-2 iroh (pin 0.98), T-NN+2 iframe Rust-wasm (§P34), P3-OS-1. LT-2 Radicle
ARMÉ dry-run privé FAIT (flip = PO hors-sprint). LT-3/LT-4/LT-7 hors-sprint. LT-5
RÉSORBÉ (quorum DB-Rust, NE PAS re-coder dispatcher Python).
**NE PAS reconduire** (landés/fermés) : P3-THREAT-MODEL-COHORT-ROW (§15.2 landée
THREAT_MODEL.md:895-916), les 3 carries 2-reports (fermés B), P3-D-4 (log slice
cosmétique « No action required »), P3 éditoriaux Phase F (corrigés en-phase).

## Cibles documentaires (file → section → contenu ; garde-fous anti-duplication)

1. **THREAT_MODEL.md** → bloc historique de versions (après v8.1, ~l.998) :
   AJOUTER « v9 (Sprint 76 Phase G, 2026-06-17) » consolidant §15.2 (quorum
   cross-machine cohorte advisory) + §15.3 (dashboard) + pointeur acceptance
   compute LIVE. **NE PAS recréer de rows STRIDE** : §15.1/§15.2/§15.3 existent
   déjà ET la row duress-frères B1 est DÉJÀ marquée FERMÉ §15.1:884.
2. **docs/rust/PATTERNS.md** → AJOUTER **§P62** (prochain libre) wrap-up
   « Sprint 76 — task-routing modèle ENTIER cross-machine prouvé » + renvoi
   tech-debt vers §P60.3 (TOPLOC étage-2 = S77). **NE PAS recréer** §P60.1/.2/.3
   ni §P61 (déjà en place).
3. **docs/shell/PATTERNS.md** → AJOUTER **### P38 — Sprint 76 : dashboard
   contributeur (front)** (format SANS préfixe §, dernière = P37). Pattern front
   réutilisable Phase E (ContributorCard / route /api/v1/contributor / GPU-heures
   locales honnêtes non-attestées).
4. **docs/claude/SPRINT_LOG.md** → table v2.1 (OPEN) : INSÉRER row S76 ligne 18
   (AVANT row 75), gabarit EXACT = row 75, 1 ligne ultra-dense. Colonnes
   | Sprint | État | Tip clôture | Nb commits | Docs |.
5. **CLAUDE.md** « ## Etat actuel » : (a) l.168 « Sprints 0-75 CLOSED, S76 a
   ouvrir » → « 0-76 CLOSED, S77 a ouvrir » ; (b) bloc « S76 DONE … Arc 3.5
   Factory Complete Vision 6/6 COMPLET » juste après le bloc S75 (l.288) ;
   (c) compteur ~2129 → ~2204 ; (d) annoncer S77 sharding.
6. **roadmap_v5** → (a) bloc « LIVRAISON 2026-06-17 (S76 Phase G) » après le bloc
   S75 (~l.31) actant Arc 3.5 6/6 CLOS + S77 sharding ouvert ; (b) annotation
   courte sur le décalage §3 déjà acknowledge (l.21-22), NE PAS le contredire.
7. **sprint76_verification.md** → finaliser colonne Observed (36 verts session +
   #26/#30 DIFFERE-trace-user + #6 Docker recovery) ; convertir §5 TODO.
8. **sprint77_audit_plan.md** → CRÉER (tracks Phase 0 S77 : duress résolu confirmé
   + 6 carries hérités + P3-D-3 + MEDIAN-DE-GROUPE + surfaces compute/quorum/
   dashboard/quant + candidats P1 sharding).

## Execute checklist (ordre)

1. Finaliser `verification.md` colonne Observed (36/38 session + 2 LIVE différé + #6 recovery).
2. Recovery Docker (`wsl --shutdown` + restart Docker Desktop) puis fail-fast
   SÉQUENTIEL (Win seul puis Docker seul, jamais concurrents — wedge S76-C).
   `http.rs:8528` doit être clean sous canonique 1.94 (NE PAS reformater local 1.95).
3. Bloc frontend complet (lint/tsc/unit/coverage/build/size/scan).
4. Ajouter paramètre `REDUNDANCY` au harness `b3_live_pc_vps.sh` (option B retenue).
5. Créer `sprint77_audit_plan.md`.
6. MAJ THREAT_MODEL.md (bloc versions v9, pas de row dupliquée).
7. MAJ rust/PATTERNS.md (§P62, pas de re-création).
8. MAJ shell/PATTERNS.md (### P38).
9. MAJ SPRINT_LOG.md (row S76 ligne 18).
10. MAJ CLAUDE.md (0-76 CLOSED + bloc S76 + Arc 3.5 6/6 + ~2204 + S77).
11. MAJ roadmap_v5 (bloc livraison + annotation §3).
12. MAJ memory nexus_grid_pivot.md tip + index MEMORY.md (court, < 200 char/ligne ;
    MEMORY.md déjà 24.9KB > limite → déplacer le détail dans le topic file).
13. Commit `feat(daemon): Sprint 76 Phase G — wrap-up + cross-machine compute
    acceptance + Arc 3.5 close` ; body 9 sections. PIÈGE : trim whitespace EOF
    du `codex_review.md` avant stage (lightcheck `git diff --cached --check`).

## Notes environnement

- **Docker WSL wedge (S76-C)** : 2 builds Rust lourds concurrents → OOM linker
  MSVC. Recovery = `wsl --shutdown` + restart Docker Desktop ; suites
  SÉQUENTIELLES (Win seul, puis Docker seul). Docker re-joué = gate AVANT push.
- **fmt drift `http.rs:8528`** : faux-positif rustfmt local 1.95 vs canonique
  1.94 (pas de `rust-toolchain.toml`). `http.rs` non touché par Phase G. fmt
  canonique = Docker `rust:1.94`. NE PAS reformater localement.
