# Sprint 18 — Audit plan pour Sprint 19 Phase 0

**Ecrit** : 2026-04-15 (Phase F wrap-up)
**Commit stack a auditer** : range `4f0727b..<wrap-up>` — 9 commits (Phase A supply-chain `<A>` + Phase B repro builds `4ab0211` + Phase C multi-relai `9d0ad7a` + Phase D wire+token `94cccb2` + Phase E1 driver `9f4d19f` + Phase E2 canary `04c9621` + Phase E3 Codeberg mirror `95807b1` + Phase F `<this>`)

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre :
   - memory (`MEMORY.md`, `nexus_grid_pivot.md`, `sprint_audit_gate.md`, `feedback_approach.md`)
   - `git log --oneline 4f0727b..HEAD` (range Sprint 18)
   - `.planning/archive/v1.2/sprint18_kickoff.md` (D1..D5 gelees, **NE PAS rebattre**)
   - `.planning/archive/v1.2/sprint18_plan.md`
   - `.planning/archive/v1.2/sprint18_verification.md`
   - **ce document**
   - Phase reviews `.planning/archive/v1.2/sprint18_phase_{B,C,D,E2,E3}_review.md` (lus **apres** formation opinion own-track)
2. **NE PAS lire** les phase reviews en entier avant d'avoir forme une opinion track par track. Ces docs captent la narration livreur — l'audit doit **challenger** pas confirmer.
3. Timebox suggere : **2-3h**.
4. Livrable : `.planning/active/sprint18_audit_findings.md` (meme layout que `sprint17_audit_findings.md` archive).
5. Commits fix eventuels (P0/P1) doivent atterrir avant le premier commit Sprint 19 Phase A. Format `fix(sprint18): <track>-P<n> — <short>`.

---

## Scope auditable

Sprint 18 livre **~1460 LOC code + ~350 LOC tests + ~440 LOC docs/config** reparti sur 8 phases :

| Track | Phase | Commit | Livrable principal |
|---|---|---|---|
| A | Supply chain CI | `<A>` | cargo-deny + pip-audit + npm audit + wasmtime pin + `.github/workflows/supply-chain.yml` |
| B | Reproducible builds | `4ab0211` | `--locked` + `SOURCE_DATE_EPOCH` + SHA256 SLSA in-toto attestation + `docs/release/REPRODUCIBLE_BUILDS.md` |
| C | Multi-relai + DHT | `9d0ad7a` | `RelayMode::Custom` n0 + 2 fallbacks + DHT pkarr 3-paralleles quorum 2/3 + Phase C review PASS |
| D | Wire TaskEntry + token | `94cccb2` | Coord-side emet `is_open_source` + caps W/VRAM/hours dans `TaskEntry` + X-SBFB-Token rotation auto |
| E1 | Driver check NVD | `9f4d19f` | `nexus-launcher/driver_check.rs` + NVD scrape + cache 24h |
| E2 | Warrant canary | `04c9621` | `canary.rs` 470 LOC + CLI `sbfb canary publish/verify` + `CANARY.txt` bootstrap + `canary-monthly.yml` verifier + `verify-canary.sh` |
| E3 | Codeberg mirror | `95807b1` | `mirror-codeberg.yml` push-mirror + `MIRROR_FALLBACK.md` §1-§7 (pivot Radicle → Codeberg, Radicle differe v1.0) |
| F | Wrap-up | `<this>` | verification + audit plan + migrations + updates CLAUDE/SPRINT_LOG |

Meta-track : **Radicle-v1.0 tracking** (P2-2 reporte E3 review).

---

## Track A — Supply chain CI baseline

**Question centrale** : les 3 gates (cargo-deny, pip-audit, npm audit) sont-ils configures correctement, le pin wasmtime est-il effectif, le workflow est-il fail-closed ?

**Methodes** :

1. Lire `.github/workflows/supply-chain.yml` : verifier que chaque step fait `exit != 0` en cas de CVE Critical/High detectee, pas juste un warning.
2. Lire `crates/Cargo.toml` : grep `wasmtime = "=<version>"` — est-ce exact pin (`=`) ou caret (`^`) ? Version = 43.0.1+ ou LTS 36.0.7+ ? Compare a la decision D2 kickoff.
3. `cargo-deny.toml` existe ? Contenu verifie (bans, licenses, sources, advisories sections completes) ?
4. Tests integration : le workflow a-t-il un test negatif (i.e. un PR qui introduit `openssl 0.1.0` fake = workflow rouge) ?
5. cross-check pip-audit + npm audit comportement identique Rust (exit code sur Critical).

**Trigger audit P0** : un CVE Critical dans une dep actuelle qui passerait le gate sans blocker. Ou wasmtime pin absent (stub accepte mais toutes les features wasmtime desactivees).

**Trigger audit P1** : license blocklist incomplete (AGPL copyleft mal whitelistee cote deps internes).

---

## Track B — Reproducible builds + SLSA in-toto

**Question centrale** : la verification SHA256 est-elle deterministe (2 builds fresh sur la meme machine = meme hash), l'attestation in-toto suit-elle le schema SLSA v1.0 officiel, les signatures Ed25519 verifiables offline ?

**Methodes** :

1. `docs/release/REPRODUCIBLE_BUILDS.md` : verifier que le protocole est **executable** (commandes copy-paste, pas abstraites).
2. Lire `crates/<release-binary>/Cargo.toml` + build scripts : `--locked` enforced ? `SOURCE_DATE_EPOCH` respecte par le linker Windows (MSVC strip timestamps) ?
3. Attestation in-toto : schema version = `https://slsa.dev/provenance/v1` ? Signe avec quelle cle (daemon node_id = ephemere — erreur ; cle release persistante — ok) ?
4. 2 builds local successifs sur la meme revision : `sha256sum` identique ? Documenter 1 test manuel.
5. Cross-platform : repro sur Windows vs Linux CI → hashes differents attendus (platform-specific), mais chaque plateforme seule doit etre deterministe.

**Trigger audit P0** : determinisme casse (2 builds identiques = 2 hashes differents) → aucune valeur SLSA.

**Trigger audit P1** : signature utilise node_id ephemere (perd la propriete "meme signer cross-releases").

---

## Track C — Multi-relai federation + DHT quorum

**Question centrale** : le fallback relais est-il correct (round-robin, timeout, recovery apres panne), le DHT quorum 2/3 refuse-t-il les reponses minoritaires (anti-eclipse partiel), les tests E2E couvrent-ils le happy path ET le degraded mode ?

**Methodes** :

1. Lire `sprint18_phase_C_review.md` entier pour baseline findings.
2. Grep `RelayMode::Custom` dans `crates/` — comptage + verification que tous les points de construction `Endpoint::builder()` l'utilisent (pas de regression sur un chemin qui construirait default Mode).
3. DHT : `tokio::join!` sur 3 pkarr paralleles + quorum counter. Lire le code : si 2/3 convergent sur hash H, accepter ; si 0 ou 1 accord, retry ? Log ? Drop ?
4. Tests : fail-inject 1 relais sur 3 → le noeud continue via les 2 autres ? 2 relais fail → degraded mode flag leve ?
5. Metrics : est-ce que le switch relais leve un metric `relay_failover_count` ?

**Trigger audit P0** : regression sur Endpoint default (utiliser les relais n0 obligatoires au lieu du fallback config-driven) — rend le SBFB dependant total de l'infra n0.

**Trigger audit P1** : quorum accept 1/3 reponse au lieu de rejeter (le code default comportement pkarr) → vector eclipse attack.

---

## Track D — Coord-side wire + X-SBFB-Token rotation

**Question centrale** : `is_open_source` + caps (`estimated_watts`, `estimated_vram_mb`, `estimated_hours`) sont-ils injectes cote coord dans le `TaskEntry` canonical **avant** signature (pas au decode cote worker), token rotation est-elle automatique (pas opt-in) et quelle est la politique de rotation (heure, jour, session) ?

**Methodes** :

1. Lire `sprint18_phase_D_review.md` pour baseline findings.
2. Coord : tracer le code path d'un `POST /project/task/submit` → `task.rs::TaskEntry::build` : les 4 champs sont-ils injectes AVANT `canonical_bytes` ? Un test unit confirme canonical bytes = champs presents.
3. Worker decode : verifier que `is_open_source` est lu en entry (pas ignore, pas defaulted) et que `should_accept_task` de S16 le consomme effectivement pour filtrer selon consent level L2.
4. Token rotation : quelle trigger ? (N minutes inactivite / N requests / cron / launcher restart) ? Est-ce coherent avec modele threat (rotation >=24h recommande si perm-persistent key au repos non-encrypted, cf. RUNTIME_ISOLATION.md).
5. Backward compat S16 pre-rotation : token lu par launcher existant continue de marcher jusqu'a rotation ? (ne doit pas break l'install S16 au premier boot S18).

**Trigger audit P0** : wire bypass (worker accepte un task sans `is_open_source` = default false silencieusement = opacite).

**Trigger audit P1** : token rotation casse le session existant sans fallback (user boot S18 apres S16 = launcher trouve vieux token, reject, boucle).

---

## Track E1 — NVIDIA driver CVE check

**Question centrale** : le scraping NVD est-il respectueux des rate limits (5 req/30s sans API key), le cache 24h fonctionne offline, la comparison driver local vs CVE-affected version n'a pas de false-negative sur les ranges (`<= 470.xx` matche une version patch 470.05 ok) ?

**Methodes** :

1. Lire `crates/nexus-launcher/src/driver_check.rs` : implementation du filter NVD + parsing JSON.
2. Test mocked response : CVE affectant `<= 535.98` — un driver `535.99` doit NOT match (false positive evite), un driver `535.97` doit match (true positive).
3. Cache lockpath : `~/.sbfb/nvd-cache.json` permission 0600 ? JSON schema documente ?
4. Offline fallback : si NVD unreachable (DNS blackhole pre-launch), le launcher boot quand meme (pas de blocage utilisateur) ? Warning log ?
5. Severity filter : seul Critical + High remontent a l'UI (pas tous) ? (pre-launch on peut se permettre de warn sur tout, mais post-launch a revaluer).

**Trigger audit P0** : false-negative dans la comparaison version (un driver vulnerable passe sans warn).

**Trigger audit P1** : cache non respectueux TTL (24h config mais reset a chaque boot = rate-limit NVD declenche).

---

## Track E2 — Warrant canary

**Question centrale** : le signing scheme est-il strict (domain separation, JCS RFC 8785, pubkey stable multi-mois), le workflow GHA est-il VERIFIER (pas signer = preserve dead-man switch property), le fichier `CANARY.txt` est-il human-readable + machine-verifiable au format ASCII pur (pas de binary base64 surprise) ?

**Methodes** :

1. Lire `sprint18_phase_E2_review.md` entier.
2. Grep `DOMAIN_WARRANT_CANARY_V1` dans le codebase : utilise dans `canonical.rs` + `canary.rs` + nulle part ailleurs (pas de leak de domain).
3. Signer `sbfb canary publish --headline X` N=3 fois sur la meme cle : les signatures doivent differer (Ed25519 deterministe → nouveau timestamp → new canonical → new sig), mais toutes doivent verify sous la meme pubkey.
4. `canary-monthly.yml` : verifier qu'il fait `sbfb canary verify` + staleness check 45 jours, **PAS** `sbfb canary publish`. Dead-man switch = maintainer doit intentionnellement refresh le CANARY.txt commit. GHA ne signe jamais.
5. Key rotation : procedure documentee out-of-band ? Si maintainer perd sa canary-key.key, comment re-annoncer la nouvelle pubkey sans fausse alarme ? (doc clear dans PATTERNS.md §Sprint 18.1).

**Trigger audit P0** : le workflow GHA fait `canary publish` automatiquement (dead-man switch casse) → verifier sur commits post-E2, pas juste sur le canary-monthly.yml header.

**Trigger audit P1** : domain separation leak (une signature canary validable comme task/result/claim par erreur).

---

## Track E3 — Codeberg mirror + pivot Radicle

**Question centrale** : le pivot Radicle → Codeberg est-il correctement justifie et trace, le workflow `mirror-codeberg.yml` est-il securise (token via extraheader, permissions minimales, scope PAT Read+Write), la doc `MIRROR_FALLBACK.md §3` est-elle self-contained pour executer le flip Radicle au v1.0 sans re-research ?

**Methodes** :

1. Lire `sprint18_phase_E3_review.md` entier (verdict PASS 0 P0/P1, 4 P2 + 3 P3, tous fixes inline sauf P2-2 Radicle-v1.0 tracking reporte ici — cf. Meta-track ci-dessous).
2. Grep `Radicle` dans `.planning/archive/v1.2/sprint18_*.md` : chaque occurrence mentionne "Radicle differe v1.0" dans la meme phrase ou block §D5 pivot ? Pas d'occurrence orpheline.
3. `mirror-codeberg.yml` : `permissions: contents: read` uniquement, SPDX header AGPL, `set -euo pipefail`, guard `CODEBERG_TOKEN` missing. Auth via `http.extraheader Authorization: token`, pas URL-embedded credential.
4. `MIRROR_FALLBACK.md §3.1-§3.8` : contient commandes rad complete + 5 secrets RADICLE_* + workflow YAML Radicle PIN SHA `gsaslis/mirror-to-radicle@514707f3` + rotation procedure ? Un maintainer peut-il executer sans re-research ?
5. Post-merge smoke test : `95807b1` push → workflow run verte ? `git ls-remote codeberg.org/SBFB/SBFB master` SHA == `95807b1` ?

**Trigger audit P0** : leak token dans workflow logs (scan logs apres premier run reel pour `token <PAT>` pattern).

**Trigger audit P1** : MIRROR_FALLBACK.md §3 non-self-contained (manque une commande critique ou un secret GHA = re-research necessaire au v1.0).

---

## Meta-track — Radicle-v1.0 activation tracking

**Origine** : finding P2-2 du review E3 (`sprint18_phase_E3_review.md`).

**Question centrale** : l'activation Radicle au v1.0 go-live a-t-elle un landing spot concret (deadline, owner, runbook) qui resistera a la cloture du sprint 18 + changements de session ?

**Item** :

```
[Radicle-v1.0 activation]
- Owner : maintainer (FlowUP)
- Deadline : jour du tag v1.0 (probablement release v1.0 sprint)
- Blocker : tag v1.0 prerequisite — flip GitHub+Codeberg public d'abord
- Runbook : docs/release/MIRROR_FALLBACK.md §3 (8 sous-sections 3.1-3.8, self-contained)
- Resources :
  * VM Linux disponible (testee Phase E3, snapshot pre-setup recommande)
  * Action pinned : gsaslis/mirror-to-radicle@514707f3 (v0.2.0, avril 2026)
  * 5 secrets GHA a creer : RADICLE_IDENTITY_{ALIAS,PASSPHRASE,PRIVATE_KEY,PUBLIC_KEY}_KEY + RADICLE_REPOSITORY_ID
- Check post-activation :
  * Workflow mirror-radicle.yml run verte
  * rad clone rad:z<RID> depuis machine tierce recupere master tip
  * Radicle Explorer app.radicle.xyz montre le projet
  * CANARY.txt §mirror_urls extend avec 3 sources (GitHub / Codeberg / Radicle)
  * MIRROR_FALLBACK.md §1 status update → "v1.0 flipped, anti-subpoena public active"
```

**Trigger audit S19 Phase 0** : verifier que cet item existe dans le sprint dans lequel v1.0 tag est prevu. Si v1.0 != S19, re-report ce meta-track au sprint release. Pas de perte si Radicle jamais active, **mais** la promesse "pattern pret en 15 min" du commit E3 doit etre tenable — donc periodiquement `rad` CLI + action doivent etre testees (stale rot prevention).

**Fix si omis** : ajouter au kickoff S19 (ou sprint release v1.0) `§3 items carry/dette` cet item avec deadline + owner.

---

## Track F — Wrap-up coherence

**Question centrale** : les docs Phase F (CLAUDE.md `§Etat actuel`, SPRINT_LOG.md row S18, memory `nexus_grid_pivot.md`) sont-ils coherents entre eux et avec le tip final, la migration PARA est-elle complete (9 files `.planning/active/sprint18_*.md` → `archive/v1.2/`) ?

**Methodes** :

1. `git log --oneline 4f0727b..HEAD` : doit contenir au moins 9 commits S18 directs (Phase A `d7ab281` + Phase B `4ab0211` + Phase C `9d0ad7a` + Phase D `94cccb2` + Phase E1 `9f4d19f` + Phase E2 `04c9621` + Phase E3 `95807b1` + chore open `1f5cf42` + wrap-up F). Note : ~10 commits tooling `chore(claude)` entre Phase A et B sont inclus dans le range mais hors scope S18 (voir note verification.md).
2. `CLAUDE.md §Etat actuel` : mentionne tip final + compteurs tests finals + mention Gate 1 unlock.
3. `SPRINT_LOG.md` : row S18 v1.2 avec Phase stack + theme + etat DONE.
4. Memory `nexus_grid_pivot.md` frontmatter description : tip sync + compteurs post-S18.
5. `ls .planning/active/` : vide (tous les sprint18_*.md archives).
6. `ls .planning/archive/v1.2/sprint18_*.md` : **10 files attendus** (kickoff, plan, verification, audit_plan, 6 phase reviews B/C/D/E1/E2/E3). Note : E1 review produit par nexus-phase-auditor au commit `9f4d19f`.

---

## Criteres verdict

- **PASS** : 0 finding P0, 0 finding P1.
- **CONDITIONAL PASS** : 0 P0, P1 trouve MAIS fix commits livres avant `feat(sprint19): Phase A` = gate leve.
- **FAIL** : P0 non-resolu au demarrage S19 Phase A. Sprint 19 doit attendre fix.

Pattern permanent depuis Sprint 7.
