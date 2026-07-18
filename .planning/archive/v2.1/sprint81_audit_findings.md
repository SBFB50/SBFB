# Sprint 81 Audit Findings

Date: 2026-07-11
Auditor: Claude (ultracode Workflow — 12 tracks fan-out + verification adversariale par finding ; synthese main-thread)
Sprint: 81
Diff: `61412bb..8b3590c` (24 commits in-range ; le brief annoncait 25, cf. S81-I-3)
Verdict: **FAIL** à l'audit (0 P0, 4 P1, 16 P2, 14 P3 — tout P1 = FAIL au
canon ; regle README §7.1 : >=3 P1 => arbitrage PO requis) → **GATE LEVÉ voie A**
(arbitrage PO 2026-07-11) par `fix(sprint81)` `ad53940` ; le P1 restant
S81-G-ESC-1 est routé S82 par construction (escalade dont le livrable est
l'instruction). Cf. §Levée du gate en fin de document.

> Note de nature (a lire avant de traiter le verdict) : le COEUR de S81 est
> solide et verifie live. Les 4 P1 ne sont PAS des regressions fonctionnelles
> du sprint : deux sont des rots PRE-EXISTANTS hors-diff exposes par l'audit
> (hygiene de test E2E web, infra CI GHA), un est l'escalade de process due
> dont le livrable EST son instruction (boot-SEED, a fermer DANS S82), un est
> un defaut de CONSIGNATION du wrap-up (gate T1 non ecrit). Aucun n'exige de
> re-concevoir S81. La disposition depend de l'arbitrage PO « S82 = sprint
> dette » (tranche 2026-07-11) : plusieurs de ces P1/P2 SONT le travail de S82.
> Verdict FAIL mecanique (canon) ; recommandation de traitement en §Conditions.

Methode : audit orchestre en Workflow ultracode (run `wf_a5756400-0fa`). 12
agents de track (Track A eclatee suites Win / Docker / web+operator) + B..K du
canon `prompts/agent/audit-gate-checks.md`, chaque finding passe par une
verification adversariale (3 lentilles evidence/contexte/severite pour P0/P1,
1 sceptique pour P2/P3). Les 4 P1 sont CONFIRMED 0/3 refutations. Deux etapes
n'ont pas abouti dans le Workflow et sont reprises main-thread (Opus 4.8) :
Track A Docker (agent termine sans sortie structuree) et la critique de
completude (credits Fable 5 epuises) — cf. §Track A et §Completude.

---

## Track A — Suites

### A part 1/3 — Rust Windows natif + cargo deny (JOUE, VERT)
- `cargo fmt --all --check` : 0 diff.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0 warning.
- `cargo nextest run --workspace --locked` : **2095 passed / 0 skipped** (99.3s,
  32 binaires) = MATCH EXACT plan §1 + body K `8b3590c`. Aucune regression.
- `cargo test --workspace --locked --doc` : 6 passed / 0 failed.
- `cargo build -p nexus-shell-daemon --release` : ok (7m51).
- `cargo deny check` COMPLET : exit 0, « advisories ok, bans ok, licenses ok,
  sources ok » (4/4). 72 warnings duplicate = les 72 groupes multi-version
  documentes `deny.toml [bans]` (P2-AUDIT-2-RESIDUEL assume warn), 6/6 ignores
  advisories encore vivants, RUSTSEC-2026-0185 absente du graphe, ed25519-dalek
  3.0.0-rc.0 non yankee au 2026-07-11. Base advisory-db du jour (commit 08:12Z).
- Finding : S81-A-1 (P3).

### A part 2/3 — Docker canonique (NON RE-VERIFIE INDEPENDAMMENT A CET AUDIT)
- L'agent Workflow dedie a termine sans sortie structuree (pipeline[1] :
  « subagent completed without calling StructuredOutput »). Docker engine
  present a l'audit (`docker info` = 29.4.3, 0 container) — l'echec est
  cote agent, pas cote env.
- Croisement de garantie retenu (non-bloquant pour le verdict) : Win natif
  2095/2095 VERT (part 1/3) + arithmetique delta +81 EXACTE et auto-consistante
  (Track E) + claim body K « Docker sbfb-ci 2099/2099 0-skip » coherent
  (2095 + 4 `#[cfg(unix)]`, ecart constant a chaque maillon de la chaine E).
- Le count Docker n'affecte aucun P1. Trou de COUVERTURE d'audit explicitement
  consigne, pas un finding. Re-verif Docker dual-platform disponible sur
  demande PO (run canonique sbfb-ci ~15 min).

### A part 3/3 — Frontend web/ + factory-operator + E2E (JOUE)
- web/ : lint 0 err (5 warnings react-refresh pre-existants S10), tsc clean,
  Vitest **412/412** (38 fichiers = attendu), coverage 87.27/79.01/86.02/88.59
  >= seuils 85/78/85/85 (vert solo, cf. S81-A3-5), build, size-limit **6/6**,
  scan FR clean.
- operator (`tools/factory-operator`) : lint, Vitest **201/201** (35 fichiers =
  attendu), build, size **8/8**, E2E Playwright hermetique **10/10** (Operator
  Rust reel rebuild in-run). INTEGRALEMENT vert et exact.
- **E2E web hermetique : ROUGE 3/45** (41 passed + 3 failed + 1 skipped).
- Findings : S81-A3-1 (P1), S81-A3-2 (P1), S81-A3-3 (P2), S81-A3-4 (P3),
  S81-A3-5 (P3).

**Counts observes** : Rust Win 2095 (plan 2095) ; Docker non re-mesure (claim
2099) ; Vitest web 412 (plan 412) ; operator 201 (plan 201) ; operator E2E 10
(plan 10) ; web E2E 41 passed/3 failed/1 skip (attendu stale 41+1, reel-si-vert
44+1, cf. S81-A3-3).

---

## Track B — Security
- 19 verifications, evidence negative exhaustive. **0 finding.**
- 15 unsafe neufs = test-only avec `SAFETY:` ; 157 unwrap = mod tests (1 prod
  pre-existant garde) ; 0 secret (cles test deterministes ; SBFB_IDENTITY_SECRET_HEX
  atteste absent au flip) ; 0 eval/innerHTML/allow-same-origin ; 12 serde(default)
  rationalises.
- Focus S81 VERTS : THREAT v17 sweep S78 x15 exact (0 ref vivante non requalifiee
  hors historiques v12-v14) ; attestation loaded-stage documentee comme self-claim
  (pas de sur-vente byzantine) ; note N/A GUARDRAILS sanitize_diagnostic motivee ;
  §15.4/15.5 zero-n0+flip conformes au deploye ; SI-12 TOCTOU documente THREAT v17
  §16 (disposition, pas re-decouverte).

---

## Track C — Patterns
- Ordre anti-anchoring respecte (opinion diff-first puis confrontation PATTERNS).
- §P73 (Phase K) FIDELE au code sur ses 4 claims (payloads in-frame, discipline
  two-node, seam env->plan, JoinPeers). §P70/P71/P72 non regresses.
- Findings : S81-C-1 (P2), S81-C-2 (P2), S81-C-3 (P2), S81-C-4 (P3), S81-C-5 (P3).
- Tech debt : T20 pose (relay-cert-pinning) mais avec pointeur faux (S81-C-3).

> **Note (S82 Phase H, 2026-07-14).** La prose de S81-C-1/C-2/C-4/C-5 n'a
> jamais ete persistee : seuls les IDs ci-dessus existent (Track C
> anti-anchoring — la sortie brute de l'auditeur n'a pas ete archivee), et
> grep `S81-C-` sur les 17 phase-reviews S81 = 0 hit. La consigne
> « re-extraire des phase-reviews » etait donc inexecutable. Disposition
> Phase H : passe de fidelite PATTERNS<->code RE-DERIVEE (methode Track C,
> Workflow 13 agents : 8 tranches rust+shell + verification adversariale
> par tranche) -> 21 findings verifies (14 P2 + 7 P3 apres re-cotation
> adversariale), TOUS corriges in-phase — corrections et presente note
> voyagent dans le meme commit atomique Phase H ; le round Codex de la
> phase a durci 6 de ces corrections (quorum accept/reject, plain-fetch
> residuel FileUploadBlock, safeParse 2xx-only, kudos ledger non signe,
> zip 8.6.0 au lock, present-vrai "routed"). **S81-C-1/C-2
> SOLDES par re-derivation ; S81-C-4/C-5 subsumes par la meme passe
> (routes Phase H, ledger E :116) ; S81-C-3 re-ancre** (annotation datee
> dans le corps T20 `docs/rust/PATTERNS.md` + harmonisation namespace
> `iroh::tls` du blockquote). §P73 re-verifie FIDELE (2 passes
> independantes). Detail : `sprint82_phase_h_preflight.md` §Passe de
> fidelite.

---

## Track D — Scope
- 8/8 scope cuts verifies TENUS. iroh STRICTEMENT SEUL tenu (51 fichiers
  hors-perimetre du bump = collapses clippy MSRV-gated 1:1 traces
  `collapse_sites.txt`) ; absorption ex-S78 (I/J) actee C1 ; 3 chores
  (bf07960/e7ff73c/9c52cb7) doc/process purs avec decision PO citee.
- Bump toolchain 1.95 ABSENT (workspace 1.94). Bisectabilite A->B tenue
  (materializer `1e7188f` separe et anterieur au bump `c899d54`). 0 bump wire.
- Finding : S81-D-1 (P3, libelle body Phase I).

---

## Track E — Tests Delta
- Audit statique (aucune suite lancee). **0 finding.**
- Chaine Win 2014->2095, somme des 17 deltas par phase = **+81 EXACT** (A..J +70,
  K +11 = 9 K-1/K-2 + 2 fixes review/Codex). Docker 2018->2099 miroir (+4 cfg(unix)
  constant). Vitest web 411->412 (Phase I seule, 1 it() ajoute). Operator 201 +
  E2E 10 inchanges. 0 baisse silencieuse, 0 gonflage test-rot.

---

## Track F — Review Files
- 16 phases committees (A..K dont A2/A3/A4/E2/E3) x 3 artefacts = **48/48 presents**.
  Phase 0 = audit gate S80 (artefacts archives v2.1, commits pre-range). Les 16
  review.md ont UN SEUL `## Verdict: PASS` a l'etat committe (0 PASS-PENDING final)
  + une section `## Codex reconciliation`.
- Authenticite Codex J/K re-jugee au fond : les episodes de blocage executeur sont
  transparents, les deux rounds bruts committes.
- Findings : S81-F-1 (P2), S81-F-2 (P2), S81-F-3 (P2), S81-F-4 (P3), S81-F-5 (P3).

> **Note (S82 Phase J, 2026-07-15).** La prose per-item de S81-F-1..5
> n'a jamais ete persistee : seuls les IDs existent (3e occurrence de
> la classe prose-non-persistee — cf. notes Track C [Phase H, supra]
> et Track H [Phase I, infra] ; cause racine commune : cet audit n'a
> detaille que les 4 P1). Grep `S81-F-` sur tout .planning = 6 hits,
> tous ID-nu ou routage-ledger, 0 prose descriptive — la synthese
> track-level ci-dessus (48/48 artefacts, Verdict unique PASS,
> authenticite Codex J/K) existe en revanche et CORROBORE la
> re-derivation. Disposition Phase J : passe d'hygiene fichiers-review
> RE-DERIVEE (methode Track F `audit-gate-checks.md:114-131`) sur les
> 16 review.md S81 archives (v2.1) + les 9 review.md S82 (a-i) : les
> 25 portent chacun UN SEUL `## Verdict` committe = `## Verdict: PASS`
> (0 PASS-PENDING final) ; triplets complets (S81 48/48 + 2 .txt
> support Phase B ; S82 27/27 + 2 fichiers support acceptance/ledger
> legitimes) ; `## Codex reconciliation` present partout ;
> authenticite Codex J/K conforme (`## Verdict global` = phrasing
> d'output brut Codex, pas un header de gate). Anomalies residuelles
> OBSERVEES, benignes, hors gate, dans fichiers ARCHIVES (registre
> fige, non reecrit — RECORDED-NOT-FIXED) : (a)
> sprint81_phase_b_review.md porte DEUX `## Codex reconciliation`
> (:506 placeholder pre-Codex non retire + :517 reelle) ; (b)
> sprint81_phase_a_review.md:12 garde un header secondaire
> `## Conclusion (pré-Codex) : PASS-PENDING` (le Verdict final :139
> est conforme) ; (c) titres H1 heterogenes entre phases
> (cosmetique) ; (d) titre a3_review « Phase A3a » = split A3a/A3b
> documente au preflight (PLAN-ADAPT), triplet coherent — PAS un
> drift. **S81-F-1/F-2/F-3 (P2) + S81-F-4/F-5 (P3) REQUALIFIES CLOSED
> sans edit.** Detail : `sprint82_phase_j_preflight.md`.

---

## Track G — Carry-Overs
- **36 items de l'inventaire plan §3 statues** : 10 CLOSED (evidence live), 25
  re-routes (rationale + exit condition), 1 ESCALADE BLOQUANTE instruite.
- CLOSED verifies live : carry sharding S77 RIG-ABSENT (`43623a5` + t2_j PASS,
  determinism byte-identique) ; binding loaded-stage K (3 tests fail-closed au
  code + ACTED agregat) ; WAN task-delivery A3/A4 (note run 1 quorum : task a
  atteint le Mac par WAN) ; hot-join E3 ; P2-SIBLING-SYNC-SET C ; strip-relay K
  (volet re-flake CI 3-OS NON verifiable avant push, exit routee) ; quorum LT-7/C10
  ETEINT (agregat top-level PASS bi-axe) ; G1 design_review present dans active/.
- Supply-chain re-verifie live (crates.io/advisory-db 2026-07-11) : ed25519-dalek
  3.0.0 stable existe MAIS iroh 1.0.2 epingle ENCORE ==3.0.0-rc.0 via iroh-base
  => deblocage flip warn->deny IMPOSSIBLE par bump iroh a ce jour ; HICKORY 6
  ignores toujours necessaires ; quinn-proto residuel borne inchange ; yanked=deny
  dormant (rc.0 non yankee) ; G-D5-1 VALIDATED_BLUEPRINT « 0.97 » toujours stale.
- Findings : S81-G-ESC-1 (P1), S81-G-1 (P2), S81-G-2 (P3), S81-G-3 (P3).

---

## Track H — HARDENING
- Coeur du mandat VERT : trigger iroh FIRED consigne Phase G `50f05c1` (last_validated
  2026-06-03->2026-07-08, trigger re-arme, entree datee) ; LOOPBACK last_validated
  2026-07-11 conforme, §3 indexe les 6 routes shard-session ; zones rouges INCHANGEES
  — R-iroh-audit P0 jamais requalifie (0 occurrence dans le diff/artefacts ; upgrade
  != Gate 1, pilote ferme).
- Findings (tous PRE-EXISTANTS, aucun cause par S81) : S81-H-1 (P2), S81-H-2 (P2),
  S81-H-3 (P3).

> **Note (S82 Phase I, 2026-07-14).** La prose de S81-H-1/H-2/H-3 n'a
> jamais ete persistee : seuls les IDs ci-dessus existent (meme classe
> que Track C, cf. note Phase H supra — 2e occurrence, signal meta
> consigne : la cause racine est que cet audit n'a detaille que les 4
> P1 ; les phases S82 restantes qui « re-extraient » doivent d'emblee
> planifier une RE-DERIVATION). Grep `S81-H-[123]` sur tout .planning
> (dont les 30+ phase-reviews archivees v2.1) = 0 hit de prose
> descriptive. Le mapping H-1->THREAT_MODEL / H-2->LOOPBACK /
> H-3->HARDENING_ROADMAP du plan est une INFERENCE (non ancree
> per-item), traitee comme simple cadrage. Disposition Phase I : passe
> de fidelite hardening-drift RE-DERIVEE (Workflow 8 agents : 4
> tranches THREAT x2 / LOOPBACK / HARDENING + verification
> adversariale par tranche). Resultats — THREAT_MODEL : famille
> coordinator-Python-au-present (scope §1.1, DFD §4, §5.6 consent.py,
> note d'epoque §7, parenthese VERSION=5, governor 0.10) + 3 ancres
> numeriques perimees (§14 T-OPERATOR-SPAWN/CSRF, §15.1 public_routes)
> => **S81-H-1 SOLDE par re-derivation** (corriges in-phase).
> LOOPBACK : 4 ancres numeriques perimees §3.1 + 1 chemin de route
> abrege §3 (quarantine) => **S81-H-2 SOLDE** (corriges,
> last_validated 2026-07-14). HARDENING_ROADMAP : 0 drift vivant
> (front-matter, classification hickory :28, §3 historique intangible
> re-verifies au lock) => **S81-H-3 REQUALIFIE CLOSED sans edit**
> (hors annotation :889 cargo-audit, item distinct
> CARGO-AUDIT-CLAIM-HONESTY). Desambiguisation : S80-H-1/2/3/4
> (LOT-LOOPBACK-DOC, CLOSED S81 G) != S81-H-1/2/3 ; homonymes Sprint 9
> H-3 wheel (PATTERNS) et SEC-H-1 (review phase H S81). Detail :
> `sprint82_phase_i_preflight.md`.

---

## Track I — Meta-Process
- Fond PROPRE : 17 commits « Sprint 81 Phase X » au format canonique, 9 sections
  body avec headers canoniques, 0 emoji (24 messages), 0 amend, 0 --no-verify
  (backstop lightcheck en place). Bascule Codex 5.5->5.6 Sol PARFAITEMENT tracee
  (`e7ff73c` : directive PO, slug, gate CLI 0.144.1).
- Findings : S81-I-1 (P2), S81-I-2 (P2), S81-I-3 (P3).

> **Note (S82 Phase J, 2026-07-15).** La prose per-item de S81-I-2 n'a
> jamais ete persistee (seul l'ID existe ; I-3 a un embryon dans
> l'en-tete `Diff:` du present fichier :6) — meme classe
> prose-non-persistee que Tracks C/H/F. RE-DERIVATION Track I
> (`audit-gate-checks.md:162-179`) sur `61412bb..8b3590c` : le « Fond
> PROPRE » de la synthese supra est CONFIRME — 17/17 commits de phase
> portent les 9 headers canoniques + `## Codex verification`, 0 emoji,
> 0 merge, 0 amend ; G8 present 16/17 (15 PLAN-ADAPT + J `43623a5`
> DESIGN-CONFLICT→arbitrage PO Option B ; le 17e `58cef6d`
> [dispositions review Codex de Phase I] porte `## G8 traceability`
> sans token frais et herite du preflight parent `bb6c4f9` — pattern
> legitime d'un commit de dispositions). **S81-I-2 REQUALIFIE
> CLOSED-consigne** : 0 defaut dur ; 2 observations soft consignees
> SANS elire une prose canonique unique (l'intention originale est
> irrecouvrable) : (a) Phase I livree en 2 commits sujet-Phase-I
> (`bb6c4f9` feat + `58cef6d` dispositions post-commit) ; (b)
> monotonie 15/17 PLAN-ADAPT, auto-escaladee par les bodies eux-memes
> (`23f3be8` 2e consecutif → `efb9667` 5e) — signal deja absorbe par
> le kickoff S82 (re-derivation d'emblee). **S81-I-3 CLOSED** :
> l'ecart 25-vs-24 commits = pure convention de borne —
> `git rev-list --count 61412bb..8b3590c` = 24 (borne basse EXCLUE)
> vs 25 en incluant le commit d'ACTIVATION `61412bb` (kickoff+plan
> S81) ; 0 merge, 0 commit perdu, la plage `..` exclusive de l'audit
> est correcte. S81-I-1 deja resolu voie A `ad53940` (correction au
> row Track I de `sprint82_audit_plan.md`). Passe immuable : aucun
> commit amende — les observations se consignent. Detail :
> `sprint82_phase_j_preflight.md`.

---

## Track J — Testability
- Substance du gate SOLIDE : mapping T1 6 sous-tests committe et REEL (23 tests
  verifies par grep, 0 env-gate, classe relay-gated exclue avec libelle corrige) ;
  test de convergence cross-machine README §4 existe (`dispatch_loop.rs:396`, 2
  noeuds iroh, task: incrementale post-subscribe) ; T2 agregat bi-axe PARSE
  (10/10 artefacts), top-level status=PASS ; b3_p2_quorum PASS.
- Defaut de CONSIGNATION sur un point (S81-J-1).
- Findings : S81-J-1 (P1), S81-J-2 (P2), S81-J-3 (P2), S81-J-4 (P2), S81-J-5 (P3).

> **Note (S82 Phase J, 2026-07-15).** La prose per-item de
> S81-J-3/J-4/J-5 n'a jamais ete persistee (IDs supra + tables + un
> gloss d'une ligne `sprint81_verification.md:294-295`) — meme classe
> prose-non-persistee que Tracks C/H/F/I. RE-DERIVATION Track J
> (`audit-gate-checks.md:181-217`) sur le corpus reel. **S81-J-3 (P2)
> SOLDE par ratification** : le vocabulaire palier-level T2
> `ACTED{evidence}` / `MIXED` / `NOT-RUN` (contrat in-artifact
> `sprint81_t2_acceptance.json:6` ; 3 ACTED + 1 MIXED dans l'agregat,
> NOT-RUN dans `sprint81_t2_baseline_098.json:54`) est desormais
> defini au canon README §4 — paragraphe dedie, miroir du patron
> d'amendement T3 Phase B ; ACTED/MIXED repris du contrat verbatim,
> NOT-RUN derive fidelement de l'usage (le contrat listait le token
> sans le formuler). L'agregat compte 13 paliers (11 transport + 2
> sharding) — le « 10/10 paliers » de verification.md:290-291
> comptait vraisemblablement les 10 artefacts externes references,
> pas les paliers. **S81-J-4
> (P2) SOLDE** : couche token foldee dans J-3 ; residuel reel = les
> chemins in-artifact `.planning/active/sprint81_t2_*.json` ne
> resolvent plus post-archivage (les 10 artefacts vivent en
> archive/v2.1/) — ACCEPT-DOCUMENTED P3 : archivage attendu, 0 perte
> de preuve, `scripts/acceptance/.b3_quorum_k.json` resout toujours ;
> l'agregat archive n'est pas reecrit (registre fige). **S81-J-5 (P3)
> SOLDE** : le corpus S81 est propre — verification §Acceptance
> emploie le vocab ferme COMPLET (T1 :267, T2 :288), 0 verdict
> `DIFFERE-*` employe (le seul token concret du corpus reviews S81,
> `sprint81_phase_k_review.md:359` `DIFFERE-materiel`, est une
> meta-reference a l'anti-pattern interdit) ; residuel de
> classe LIVE = tokens nus « T1 = N-A » / « T2 = N/A » dans des
> bodies S82 deja committes (passe immuable) → hygiene going-forward :
> les futurs bodies ecrivent les formes completes
> `N-A-no-frontend-change` / `N-A-no-cross-machine-feature`. Rappel
> 0-perdu : S81-J-1 (P1) resolu voie A `ad53940` (`## Acceptance`
> verification :258) ; S81-J-2 (P2) ROUTED (PARTIAL) — fichier
> calibre Phase C `2931b82`, run reel >=1 du a Phase T (PO-4=C),
> jamais full-CLOSED ici. Detail : `sprint82_phase_j_preflight.md`.

---

## Track K — Docs-Contract Closure
- Cloture docs-contrat LIVREE et conforme au plan §2 sur les 4 livrables : LOOPBACK
  §3 +6 lignes shard-session (last_validated 2026-07-11) ; SHARD_PROTOCOL_SPEC
  §5.1/5.2/§6 collent au code (row /result 9/9) ; lot doc-stale docs/sharding
  requalifie (ref pendante WIRING_SPEC:147 reparee, symboles grep-resolvables,
  gate source-ref) ; `spec_consts_exist` etendu aux 4 types J+K.
- Nouvelles frontieres S81 indexees (test-acteur) : 5 routes loopback shard-session
  (Phase I) + attestation stage (K) presentes GUIDE/llms.txt/LOOPBACK.
- Findings (drift de PROSE seulement, couche machine drift-gatee exacte) :
  S81-K-1 (P2), S81-K-2 (P3).

---

## Summary

| Severity | Count | Items |
|----------|-------|-------|
| P0 | 0 | — |
| P1 | 4 | S81-A3-1, S81-A3-2, S81-G-ESC-1, S81-J-1 |
| P2 | 16 | S81-A3-3, S81-C-1, S81-C-2, S81-C-3, S81-F-1, S81-F-2, S81-F-3, S81-G-1, S81-H-1, S81-H-2, S81-I-1, S81-I-2, S81-J-2, S81-J-3, S81-J-4, S81-K-1 |
| P3 | 14 | S81-A-1, S81-A3-4, S81-A3-5, S81-C-4, S81-C-5, S81-D-1, S81-F-4, S81-F-5, S81-G-2, S81-G-3, S81-H-3, S81-I-3, S81-J-5, S81-K-2 |

### Les 4 P1 en detail

**S81-A3-1 (P1) — Suite E2E web hermetique ROUGE 3/45.**
`web/e2e/browse-search.spec.ts:22` echoue (daemon suppose VIDE) parce que
`app-authoring.spec.ts:69-113` (S79-H) seme deux projets fixtures via
`/api/daemon/publish` SANS cleanup dans le daemon partage single-worker ;
browse-search tourne apres alphabetiquement. Contre-preuve : `browse-search`
solo = 5/5 vert sur daemon frais. **Pre-existant au diff S81** (les 2 specs
< `61412bb`), jamais detecte : aucun wrap-up depuis S77 n'a joue la suite web
complete, et le seul CI qui la cable (GHA) est rouge env (cf. S81-A3-2).
Disposition : fix du cleanup fixtures OU assertions tolerantes a l'etat seme,
puis re-run attendu 44 passed + 1 skip + MAJ du compte de reference.

**S81-A3-2 (P1) — GHA « CI » en FAILURE 100% des 30 derniers runs + claim faux.**
`gh run list --workflow CI --limit 30` = 30/30 failure (2026-05-07 -> 2026-07-03,
y compris les 4 pushes de la fenetre S81). Cause env : build-script `glib-sys
v0.18.1` (deps GTK absentes du runner) tue « Full verification » (clippy) et
« Factory Operator front » (vitest). Consequence : les steps Playwright E2E web
et operator ne tournent dans AUCUN CI (Woodpecker ne cable que les vitest unit) —
le volet « + CI chaque push » du gate de testabilite (README §4) est inoperant
pour T1 E2E, et le claim « CI operationnel : Woodpecker + GHA » (CLAUDE.md) est
inexact pour GHA. **Rot d'infra pre-existant, pas une regression code S81** —
mais artefact de gate inoperant + claim faux = P1 par canon (Track A/J).
Disposition : installer les deps GTK dans les jobs GHA (miroir de l'image
sbfb-ci) OU decabler honnetement GHA des claims + cabler les E2E dans Woodpecker ;
requalifier le claim tant que rouge.

**S81-G-ESC-1 (P1 de process) — Escalade boot-SEED re-drive-on-ingest OVERDUE 3/3, instruite.**
Carry S75 route 3 fois (`e05338f` + `50f05c1:197` « ESCALADE audit gate S81 » +
`8872596`), driver ONE-SHOT confirme `http.rs:1820-1826` (« nothing re-drives
until the next daemon restart »). Evidence NEUVE meme famille cote WORKER
(agregat T2 note run 2 quorum : worker boote 3s avant submit n'a jamais recu
l'entree `task:` en 30s ; convergence +2m08 une fois stable). Regle §6.2.1 :
**plus jamais de report sec** — fermer dans S82 ou re-conception ratifiee PO.
INSTRUCTION S82 (le livrable de cette escalade) : phase dediee « convergence
cold-boot catch-up », design unifie sous UN invariant (le broadcast gossip est
un HINT, l'etat durable synchronise est la VERITE — tout consommateur cold-boot
reconcilie contre le doc synchronise apres formation du neighborhood). Deux
livrables : (1) cote ANCRE = re-drive de `run_boot_seed_driver` a l'ingest
annuaire (hook idempotent par le set pinned, duress-gate herite, borne 1 re-drive
par batch) ; (2) cote WORKER = catch-up des `task:` pendantes au boot (scanner
le doc synchronise pour les entrees pending non claimees et entrer le claim path).
EXIT CONDITIONS machine-checkables : (a) T1 deux-noeuds hermetique — ancre fraiche
+ app presente uniquement dans un annuaire ingere APRES le boot => app pinnee
SANS restart ; (b) worker demarre APRES le submit => execute dans le budget ;
(c) re-jeu live T2 du run-2 => PASS <=30s, artefact JSON committe. ALTERNATIVE :
decision PO fermante ratifiant la semantique restart-only + runbook + THREAT.

**S81-J-1 (P1) — Gate T1 standing non consigne sur une surface web touchee.**
`web/src/api/daemon.ts` modifie Phase I (`ShardSessionViewSchema.rtt_frontier_ms`),
aucun spec e2e cree/etendu, aucun run web `test:e2e` enregistre sur la fenetre S81
(Phase I = lint/tsc/vitest/build/size seulement), aucun token du vocabulaire ferme
{GREEN, RED, N-A-no-frontend-change} dans `verification.md`, pas de section
`## Acceptance` exigee par README §4. Le P1 porte sur la CONSIGNATION machine-lisible
du gate, pas sur un risque produit (`rtt_frontier_ms` rendu par aucun composant,
couvert par 51 Vitest). Disposition : `fix(sprint81)` — jouer `(cd web && npm run
test:e2e)` et consigner `## Acceptance` avec T1 web = son verdict honnete (RED tant
que S81-A3-1 non fixe, GREEN apres), T1-infra = 6 sous-tests mappes, T2 = PASS.

---

## Conditions (recommandation de traitement — arbitrage PO requis, cf. §7.1 >=3 P1)

Le verdict FAIL est mecanique (canon : tout P1 = FAIL). La levee passe par le
traitement des 4 P1. Vu la decision PO « S82 = sprint dette docs-contrat +
refactorisation » (2026-07-11), plusieurs de ces items SONT naturellement le
travail de S82. Recommandation de l'auditeur (a arbitrer) :

- **A fixer en `fix(sprint81)` AVANT Phase A S82** (correction rapide, honnetete
  du record) : **S81-J-1** (section `## Acceptance` honnete) + **S81-A3-1** (le
  fix test qui rend le T1 web GREEN) + les claims faux docs a faible cout
  (**S81-A3-2** volet « requalifier le claim CI », **S81-I-1** ligne plan, **S81-K-1**
  3 lignes SPEC/REFERENCE/WIRING). Ces items sont pre-conditions honnetes a
  l'ouverture de S82.
- **A router S82 (dans le theme dette+refacto, obligatoire)** : **S81-G-ESC-1**
  (escalade — DOIT fermer en S82 par construction) + **S81-A3-2** volet infra
  GTK/Woodpecker + les P2 doc-dette (**S81-C-1/C-2/C-3**, **S81-H-1/H-2**,
  **S81-J-3**, **S81-K-2**) + **S81-G-1** (migration stores worker) + reparation
  relay-gated multi_daemon 4/10.
- **CONDITION au futur push groupe** (arbitrage PO deja documente) : declencher
  `workflow_dispatch integration-nightly` + rust-ci/ci.yml verts sur le tip
  (leve S81-J-2 + volet CI-3-OS de strip-relay).
- **Re-verif Docker dual-platform** disponible sur demande (trou de couverture
  Track A part 2, non-bloquant pour le verdict).

## Completude (critique main-thread — l'agent critic a echoue sur credits)
- Tracks A-K tous joues SAUF Track A Docker (agent echoue ; croisement de garantie
  documente, non-bloquant). Aucun focus S81 du plan §2 ignore.
- Inventaire carries plan §3 : 36 items statues par Track G (exhaustif vs la
  liste). Escalade BLOQUANTE boot-SEED instruite avec design + 3 exit conditions.
- Aucun gap bloquant residuel pour le verdict.

## Carry-Over To Sprint 82
- **S81-G-ESC-1** (boot-SEED cold-boot catch-up) : owner S82, phase dediee ;
  trigger = kickoff S82 ; exit = 3 conditions machine-checkables ci-dessus OU
  re-conception PO fermante.
- **Dette docs-contrat S79/S80 + S81 doc-dette (C-1/C-2/C-3, H-1/H-2, K-1/K-2,
  J-3/J-4, I-1/I-2, G-2, F-*)** : owner S82 (sprint dette nomme, jamais bundle) ;
  exit = chaque item corrige ou requalifie. *(Correction S82 Phase E, 2026-07-14 : la ligne
  d'origine disait « 8 P2 / 11 P3 docs-contract S80 » — mislabel, 8/11 = tally
  de l'audit S79 (joue en Phase 0 de S80) ; l'audit S80 = 4 P2/10 P3, dont
  13/14 deja resolus avant S82. Re-audit par item :
  `sprint82_phase_e_ledger_reconciliation.md`.)*
- **S81-G-1** migration stores worker redb2 : owner S82 ; exit = migration verifiee
  3 noeuds (sibling backup OU store recree + worker fonctionnel 1.0.1).
- **Reparation relay-gated multi_daemon 4/10 + integration-nightly run reel** :
  owner S82 ; trigger = push groupe ; exit = chaque test rouge repare/requalifie +
  1 run nightly lisible.
- **Supply-chain veille iroh** (ed25519-dalek 3.0.0 stable dispo mais iroh 1.0.2
  epingle rc.0) : owner standing ; trigger = release iroh relevant le pin ; exit =
  bump iroh + convergence lock + flip multiple-versions warn->deny.
- **Sharding post-bi-axe** (R-J-6 RunProofs per-worker, J-D5-1 conn_type assertion,
  schemas requetes Mint/Mount/Generate, F2 KV-cache, J1b-3, D3-2, SI-12) : owner
  axe sharding S82+ ; croise l'arbitrage PO benchmarks standards (meme rig).
- **Arbitrages PO calendaires** : Topologie A-vs-B avant 25/08 ; gate n0 15/09 ;
  benchmarks standards Phase L vs S82 ; catalog_len=0 seeder (S81-G-3, question
  design a trancher au kickoff S82).

## Levée du gate (post-audit, voie A — arbitrage PO 2026-07-11)

Le verdict FAIL est mecanique (tout P1 = FAIL). La voie A retenue par le PO
corrige immediatement les P1 d'honnetete/consignation et route le reste S82.
Commit `fix(sprint81)` **`ad53940`** (7 fichiers, 0 `.rs` touche, nextest Win
2095 inchange) :

- **S81-A3-1 (P1) — RÉSOLU** : `web/playwright.config.ts` splitte en 2 projets
  Playwright ; le seul spec semeur (`app-authoring`) tourne en dernier via
  `dependencies`, garantissant un daemon vierge aux assertions empty-state.
  Re-run E2E web = **GREEN 44 passed / 2 skipped (env-gated) / 0 failed,
  EXIT=0** (2026-07-12).
- **S81-J-1 (P1) — RÉSOLU** : section `## Acceptance` machine-lisible ajoutee a
  `sprint81_verification.md` (T1 web GREEN, T1 operator GREEN, T1-infra GREEN,
  T2 PASS). L'honnetete du RED-au-wrap-up est consignee.
- **S81-A3-2 (P1) — volet claim RÉSOLU / volet infra ROUTÉ S82** : `CLAUDE.md`
  requalifie le claim CI (GHA rouge env-GTK ; Woodpecker seul operationnel,
  sans Playwright). La reparation infra GHA (deps GTK) est routee S82.
- **S81-A3-3 / S81-I-1 / S81-K-1 (P2) — RÉSOLUS** : compte E2E web corrige
  (CLAUDE.md), verdict G8 Phase J corrige (plan Track I : DESIGN-CONFLICT),
  drift de prose ShardSessionView corrige (6 sites, privacy preserve, gates
  docs verts).
- **S81-G-ESC-1 (P1) — ROUTÉ S82 (par construction)** : escalade boot-SEED,
  instruction complete (design unifie ancre+worker + 3 exit conditions) portee
  par ce findings ; a fermer en S82 (phase dediee) ou re-conception PO fermante.

**État du gate : LEVÉ pour l'ouverture de S82.** Les P1 restants non résolus
dans ce commit (G-ESC-1 escalade + A3-2 volet infra) sont, par nature ET par
decision PO, du travail du sprint dette S82 — pas des pre-conditions
mecaniques a corriger avant Phase A. Tous les P2/P3 sont routes §Carry-Over.
Re-verif Docker dual-platform : non jouee (PO : croisement Win 2095 +
arithmetique delta +81 + claim Docker 2099 coherent suffit).
