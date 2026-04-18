# Sprint 20 Phase C — nexus-phase-auditor review

HEAD pre-commit: `c32ecb3` (Sprint 20 Phase B)
Draft commit body: "feat(sprint20): Phase C — PoW runtime wire gossip subscribe"
Timebox: 45m

## Verdict : PASS-with-carry

(0 P0, 0 P1 — commit non-bloqué. 1 P2 documenté — rigor signal G4 satisfait.
Carry obligatoire dans `sprint20_audit_findings.md` Phase F.)

---

## Dimensions

### Security

- [x] `semgrep` : non disponible, fallback grep effectué sur les 8 fichiers du diff.
- [x] `unsafe` blocks : 0 bloc unsafe nouveau dans le diff.
- [x] `unwrap()` / `expect()` en production :
  - **FINDING P2-C-SEC-1** — `wrap_payload_with_pow` (`http.rs:538`) appelle
    `.expect("PoW policy RwLock poisoned")`. Si un thread tient le write lock
    au moment d'un panic, le handler HTTP panic à son tour (axum → 500 ou crash).
    Incohérence interne au diff : le gossip receive loop (`runtime.rs:841-844`)
    gère le même cas gracieusement via `poisoned.into_inner().clone()`. De même,
    `PowPolicyWatcher::current()` (`pow_policy_loader.rs:204`) utilise
    `.expect("PoW policy RwLock poisoned")` en code non-test.
    Fix attendu Phase F : remplacer les deux `.expect()` en production par le
    pattern déjà présent dans le gossip loop (`.unwrap_or_else(|p| p.into_inner().clone())`).
  - `unwrap()` dans les tests (`#[cfg(test)]`) : légitimes, aucun finding.
- [x] Loopback / PeerCreds : aucune nouvelle route introduite dans `build_router`.
  Les 4 nouveaux champs `DaemonHttpState` (pow_*) enrichissent l'état partagé
  existant, pas la surface de routing. La route `/panic/wipe` est Phase B (déjà
  auditée), non touchée par ce diff. Aucune route non-authentifiée ajoutée.
- [x] Wire format / JCS : `PowEnvelope` est un format binaire custom
  (`[u32 BE proof_len][proof bytes][payload]`), pas JSON — la contrainte JCS
  ne s'applique pas ici. Le payload intérieur (ProjectAnnouncement) passe par
  le canonique existant `to_gossip_bytes` inchangé.
- [x] Path traversal : aucun zip extract, aucun Path::components() check requis
  dans ce diff. Le `path` du watcher est un chemin TOML, pas un path utilisateur.
- [x] Secrets : aucune credential, token, ou clé privée dans le diff.

### Patterns

- [x] **Sprint 19.1 — Primitive / wire / enforcement separation** : respecté.
  La primitive S19 (`pow.rs`, `pow_gossip.rs`) est inchangée. Phase C livre
  exclusivement le wire (loader + runtime hookup). Aucune modification à la
  crypto sous-jacente.
- [x] **TokenRotatorWatcher pattern (Sprint 18 D-1)** : `PowPolicyWatcher` est
  un pattern-copy documenté : watch parent-dir, filter by path, 50 ms debounce,
  malformed-reload garde le dernier bon état, file-deletion garde le dernier bon
  état, thread nommé `sbfb-pow-policy-watch`, `_join` tenu pour teardown propre.
  Tous les comportements testés par les 8 unit tests. Conforme.
- [x] **Sprint 19.2 — Forward-compat PoW proof format** : pas de changement au
  format wire `HashcashChallenge` / `HashcashProof`. Conforme.
- [x] **Pattern SPDX** : les 3 nouveaux fichiers ont le header
  `// SPDX-License-Identifier: AGPL-3.0-or-later`. Conforme.
- [x] **Pattern pre-launch `*_VERSION = 1`** : aucun wire format bumpe. Conforme.
- [x] **Pattern `#[serde(default)]` légitimité** : aucun nouveau `serde` struct
  dans ce diff. Conforme.
- [P2] **Pattern incohérence RwLock poisoned** (cf. Security ci-dessus) :
  le gossip loop utilise le pattern correct (`poisoned.into_inner()`) mais
  `wrap_payload_with_pow` et `PowPolicyWatcher::current()` paniquent. Le pattern
  devrait être unifié — documenter dans audit_findings.

### Working tree audit (G5)

- [x] PHASE : 8 fichiers (3 nouveaux + 5 modifs), tous dans la liste plan §6.
- [x] CRAFT : 0 fichier planning/docs hors-phase.
- [x] DEBT : 0 fichier scope-cut.
- [x] NOISE : 0 fichier accidentel.
- [x] Section "Working tree audit" déclarée présente dans le body commit
  (confirmé dans le contexte fourni).

### Scope-cuts

- [x] Grep effectué sur les additions du diff (`git diff --staged | grep "^+"`)
  pour chacun des items §8 du kickoff.
- [x] Hits de grep sur "PQC" et "ML-DSA" investigués : les occurrences sont dans
  des lignes **pré-existantes** (`relay_pow_policy.rs:40` commentaire
  `// S26 PQC migration`, `PATTERNS.md:1245` commentaire hérité). Aucune
  addition du diff ne touche ces termes. Aucun scope leak.
- [x] Hardware keystore, HPKE, Rate-limit, Client-side redaction, Kudos-weighted,
  Tool-calling sandbox, Redundancy voting, Ephemeral workers, Honeypot Eclipse,
  DNS fallback, Arti Tor, Snowflake, PQC, ML-DSA, ML-KEM : 0 addition dans le diff.

**Verdict scope-cuts : PASS.**

### Tests-delta

- [x] Rust : plan §6 annonçait +10 integration. Livré +18 (+8 unit loader
  `pow_policy_loader::tests::*` + 10 integration `tests/pow_wire.rs`).
  Delta supérieur documenté comme P3-C2 (bonus non-bloquant). Baseline
  post-Phase B → 598/598 passed.
- [x] Divergence documentée dans le body commit : P3-C2 explicite.
- [x] Python / Vitest / Playwright : inchangés (0 delta attendu pour Phase C).
- [x] `cargo fmt --all --check` : clean (déclaré + crédible car diff propre).
- [x] `cargo clippy --workspace --all-targets --locked -- -D warnings` : clean
  (déclaré).

**Verdict tests-delta : PASS. Bonus +8 non-bloquant.**

### Research-grounding

- [x] `notify` crate : déjà tracée workspace Cargo.toml (utilisée par
  `TokenRotatorWatcher` Sprint 18 D-1). Pas de nouvelle version pin dans ce diff.
  Pattern-copy documenté — trace de research existante.
- [x] `tempfile` crate (tests) : déjà workspace, pas nouvelle.
- [x] `hex` crate (tests) : déjà workspace.
- [x] Aucune dépendance Cargo.toml nouvelle ajoutée dans ce diff
  (`git diff --staged -- 'crates/*/Cargo.toml'` retourne 0 lignes).
- [x] Aucune API crypto nouvelle : `PowSolveCache`, `PowVerifyCache`,
  `PowEnvelope` sont les primitives S19 déjà auditées Sprint 19 Phase B.
- [x] `DEFAULT_POW_POLICY` re-exporté depuis `nexus-core-rs` : constante
  existante, pas d'API externe nouvelle.

**Verdict research-grounding : PASS. Aucune trace manquante.**

### Horizon long-terme + documentation amont

- [x] Design doc pour Phase C : **exempt** per plan §10 row 28 ("sauf C = carry").
  Phase C est un carry S19 A-2, pas un nouveau module structurant — la primitive
  existe depuis Sprint 19, la recherche était dans le plan S19.
- [x] Alternatives rejetées : D1..D5 du kickoff couvrent les alternatives pour
  les décisions Sprint 20. Phase C est un carry sans nouvelle décision
  architecturale — alternatives documentées dans S19 design doc
  `S19_phase_B_pow_hashcash_design.md`.
- [x] Solution la plus poussée : le watcher est une copie du pattern TokenRotator
  Sprint 18 (le pattern le plus poussé disponible dans le workspace, lui-même
  issu d'un audit P1). Pas de régression d'approche.
- [x] Aucune estimation LOC dans plan §6 ni kickoff §4. Conforme (plan §6 cite
  "+10 tests" en delta test, pas une estimation LOC).
- [P2] **Divergence plan §6.2 vs réalité** : le plan cite
  `iroh_runtime.rs::GossipClient::subscribe()` comme wire-point dans
  `nexus-shell-daemon-core`. En pratique, `iroh_runtime.rs` dans `-core` est
  le `CuratorRuntime` (nom hérité Sprint 7) et ses `subscribe()` sont des
  appels de gestion de liste d'attention (pas des appels gossip transport). Le
  vrai wire-point est `nexus-shell-daemon/src/runtime.rs::spawn_gossip_subscribe_task`.
  La divergence est documentée dans le body commit (P2-C1) mais elle révèle
  que le plan §6.4 grep-verify était formulé avec le mauvais module cible. Le
  critère d'acceptation aurait dû pointer sur `nexus-shell-daemon/src/runtime.rs`.
  Finding carry à documenter dans `sprint20_audit_findings.md` §Phase-C pour que
  le plan §6 soit corrigé à la prochaine révision de documentation (Phase F).

---

## Findings

- **P2-C-SEC-1** : `wrap_payload_with_pow` (`http.rs:538`) et
  `PowPolicyWatcher::current()` (`pow_policy_loader.rs:204`) utilisent
  `.expect("PoW policy RwLock poisoned")` en code de production. Le gossip
  receive loop (`runtime.rs:841-844`) gère le même cas gracieusement via
  `poisoned.into_inner().clone()`. Incohérence interne au même diff.
  Fix : remplacer les deux `.expect()` production par `.unwrap_or_else(|p| p.into_inner().clone())`.
  Priorité : non-bloquant Phase C (le cas RwLock poisonné est théorique
  avant v1.0), mais doit être levé avant Phase F ou en chore distinct.

- **P2-C-PLAN-1** : Plan §6.2 cite `iroh_runtime.rs` (dans nexus-shell-daemon-core)
  comme wire-point. Le vrai call-site est `runtime.rs` (dans nexus-shell-daemon
  binary). `browse.rs::subscribe()` n'est pas un appel iroh gossip — c'est un
  appel `CuratorRuntime::subscribe()` (gestion d'attention set). La divergence
  est documentée dans le body commit (P2-C1) mais le plan §6.4 grep-verify reste
  mal formulé. Carry-over pour correction documentation Phase F.

- **P3-C2** : Delta tests +18 vs +10 annoncé au plan. Bonus 8 unit tests du
  loader documenté dans body commit. Non-bloquant.

- **P3-C3** : Canary broadcast `main.rs:237` non enveloppé PoW. Documenté dans
  body commit, scope Phase E. Non-bloquant.

---

## Recommendation

**Commit autorisé** sous condition de levée P2-C-SEC-1 avant ou pendant Phase F.

L'option la plus propre : chore de correction inline avant le commit Phase C
(3 lignes : remplacer les deux `.expect()` production par `.unwrap_or_else`).
Si l'exécuteur préfère livrer Phase C as-is et lever en chore distinct avant
Phase F, acceptable — inscrire P2-C-SEC-1 dans `sprint20_audit_findings.md`
avec statut OPEN pour tracking Phase F.

P2-C-PLAN-1 : carry dans `sprint20_audit_findings.md` §Phase-C, correction
de la formulation plan §6.2/6.4 en Phase F (docs only, non-bloquant).

Aucun P0 ni P1 détecté. Le wire est architecturalement correct : un seul
join_topic dans le runtime, receive loop intégralement wrappé par
`verify_envelope` avant dispatch, publish handler wrappé par
`wrap_payload_with_pow`. Canary scope Phase E correctement identifié.
Hot-reload loader conforme au pattern TokenRotatorWatcher.
