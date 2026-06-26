# Sprint 80 — Plan d'audit (gate Phase 0 : audit du Sprint 79)

Ce document pilote l'**audit gate S79**, à jouer en **Phase 0 de S80** avant toute
Phase A. L'auditeur ingère le diff complet de S79 (phases A→I, commits
`9297f08..<tip Phase I>`) et joue les 9 tracks ci-dessous. Verdict attendu :
PASS / CONDITIONAL PASS / FAIL avec findings P0–P3. PASS exige ≥1 P2+ documenté.

---

## 1. Périmètre

S79 = capacité Factory « app-authoring » (anime.js 4.5.0 + daisyUI 5.5.23) :
knowledge packs versionnés + prompt-kind `app-authoring` + gate CSP déterministe
Rust + self-check runtime + couche docs-contrat. 8 phases feature (A–H) + Phase I
(GUIDE docs-contract closure + wrap-up). Tip = commit Phase I.

## 2. Les 9 tracks

1. **Suites** : Win nextest + Docker canonique `sbfb-ci` rust:1.94 (fmt 0 sous les
   2 toolchains) + Vitest + E2E hermétique. Confirmer le delta annoncé (Rust 1991
   → 1994 ; +`factory_csp_contract` 3 tests). Re-jouer les 3 doc-lints
   (`check-sharding-docs.sh`, `check-frontier-contracts.sh`, `check-factory-docs.sh`).
2. **Sécurité** : (a) gate CSP `run_gate_csp_authoring` **non-délégable** (hors
   `--skip-gates`) — vérifier qu'aucun chemin ne le contourne ; (b) source CSP
   **unique** `BLOB_SERVE_CSP` (`csp.rs:33`) — pas de re-hardcode, miroir
   `csp-contract.json` testé ; (c) self-check runtime = filet, jamais autorité de
   publish ; (d) `check-factory-docs.sh` strictement grep/compare (jamais
   `eval`/source d'un `.md`) ; (e) packs hashés (blake3 recompute hermétique),
   re-extraction manuelle (pas d'auto-fetch, `connect-src 'none'`).
3. **Patterns** : §P70 (cadence docs-contrat) + §P71 (self-check runtime) cohérents
   avec le code ; AGENT_SYSTEM §7 portable.
4. **Scope** : Day-0 figées tenues (kickoff 177–234) ; 0 nouvelle primitive en I ;
   0 bump wire ; scellage 100 % Factory ; connaissance consommée jamais autoritaire
   (`chat_history_authoritative=false`, 0 verdict PASS).
5. **Tests** : non-vacuité — le gate `check-factory-docs.sh` détecte bien un faux
   (drift de source-ref, marqueur d'honnêteté manquant, ligne hors-bornes) ;
   l'exemple `include!` casse bien le build si la policy CSP drifte (anti-rot).
6. **Review files** : `sprint79_phase_{a..i}_{preflight,review,codex_review}.md`
   présents, verdicts cohérents, Codex brut non-réécrit.
7. **Carry-overs** : voir §3.
8. **HARDENING** : surface app-authoring ajoutée au modèle de menace (sandbox
   scellée, statique/runtime split) — vérifier qu'aucune dispense CSP/COEP/COOP/
   Ed25519 n'a été introduite.
9. **Meta-process** : G8 par phase (PLAN-ADAPT récurrents — vérifier qu'ils sont
   evidence-based et ne touchent aucune Day-0 ; 2+ consécutifs = signal méta à
   commenter) ; gate de testabilité §4 (T1 BLOQUANT + T2 JSON) tenu au wrap-up.

## 3. Carries à escalader

- **P1 — Gate CSP `run_gate_csp_authoring` non câblé sur `redeploy` /
  `deploy-workspace` / `deploy-from-repo`** (NOUVEAU — trouvé par l'audit gate S79,
  cf. `sprint79_audit_findings.md` §P1-1). Le gate « non-délégable » n'est appelé
  qu'à `pipeline.rs:52` (verbe `publish`) ; `atelier.rs:70 redeploy()` (cœur de la
  boucle d'authoring fork→edit→iterate) et les routes daemon `deploy.rs:65/233`
  publient des octets d'app SANS rejouer le gate. La claim Day-0 #1 « scellage 100 %
  Factory » est matériellement fausse pour la boucle de redeploy. **Atténué** : la CSP
  runtime blob-serve (`csp.rs:33`, inchangée, `connect-src 'none'`) reste la frontière
  d'isolation effective — pas d'évasion réseau. **Tension décision gelée** : câbler le
  gate *côté daemon* contredirait « Factory = outil client externe, hors daemon » (D2).
  **À trancher PO en Phase 0 S80** : (a) fix client-side `redeploy` (appeler le gate
  dans `atelier::redeploy` avant POST) ± (b) amendement assumé de la formulation Day-0
  (« le gate Factory est un lint d'auteur best-effort sur les verbes de publish du
  client ; le daemon neutre applique la CSP runtime inconditionnellement »). Condition
  du CONDITIONAL PASS S79.
- **P1 — Sharding S77 PROVISIONAL** (toujours ouvert) : orchestrateur de session
  in-vivo + benchmark live cross-machine 2-machines = RIG-ABSENT. Factory-first a
  différé S78 ; cf. `sprint78_audit_plan.md` §7/§10. Décider S80 : ouvrir
  l'orchestrateur sharding ou poursuivre Factory.
- **P1 — app-authoring in-vivo `Not evidenced`** : parcours auteur réel → gate →
  self-check → publish → rendu cross-pair JAMAIS exercé in-vivo ; efficacité
  générative du prompt-kind / copilote Ollama non mesurée. Construit + testé
  hermétiquement, pas « déployé-et-éprouvé ».
- **P2 — couverture étiquette ~21 familles wire** (Phase B) : registre
  `// FRONTIER:` incrémental ne gate que les primitives annotées (1 aujourd'hui) ;
  les ~21 familles wire non-schématisées restent un carry. Ne PAS prétendre « tout
  est gaté ».
- **Track Testabilité standing** : chaque sprint doit livrer T1 BLOQUANT-vert + T2
  artefact JSON ; vérifier la non-régression à chaque push.
- **P2 — TEST-ISOLATION-SBFB-HOME** (hérité S77) : isolation `SBFB_HOME` des tests.
- **Doc-lint sémantique** : le volet (5) de `check-factory-docs.sh` vérifie
  l'**existence de ligne** des refs `PRIMITIVES.md:N`, pas que la ligne **supporte
  encore la claim** (revue LLM adversariale) — limite assumée à re-confirmer.

## 4. Note

Audit platform-agnostique pour la part docs/script/test ; re-run dual-platform
(Win + Docker) déjà exigé au wrap-up Phase I AVANT push. L'audit gate S79 ne
recode pas — il vérifie et écrit `sprint79_audit_findings.md`.
