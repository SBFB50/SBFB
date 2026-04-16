# Sprint 19 Phase A — nexus-phase-auditor review

**HEAD pre-commit** : `0c20a39`
**Draft commit title** : `feat(sprint19): Phase A — DHT quorum runtime wire (browse aggregator canary)`
**Timebox** : 18 min
**Auditor** : nexus-phase-auditor (session 2026-04-16)

## Verdict : PASS

**Promu CONCERN → PASS** après intégration des 2 corrections
code recommandées par l'auditeur avant commit :

- P3 #1 fixé : `crates/nexus-core-rs/Cargo.toml` utilise désormais
  `url = { workspace = true }`.
- P3 #3 fixé : doc-comment `pkarr_resolver.rs` ligne 92 pointe
  désormais vers `CaRootsConfig::default` (== `EmbeddedWebPki`)
  au lieu de `::embedded`.
- P3 #2 reporté Phase F comme explicitement recommandé par
  l'auditeur — annotation doc uniquement (`sprint19_plan.md §3.1`
  signature `PkarrRelayClient::new` 2-arg vs pré-session 1-arg).

Verdict initial CONCERN conservé ci-dessous avec les findings
P3 intégraux pour la trace audit. 0 finding P0/P1 trouvé.

## Verdict initial (pré-intégration P3 code) : CONCERN

0 finding P0/P1. 3 findings P3 — commit autorisé après intégration des 2 corrections code (P3 #1 + P3 #3) ; P3 #2 (annotation plan) reporté Phase F.

---

## Dimensions

### Security

- Pas d'unsafe blocks nouveaux.
- Pas de secrets hardcodés (`AKIA`, `ghp_`, etc.).
- `.unwrap()` en prod path : le seul `.unwrap_or("unknown")` dans
  `PkarrQuorumResolver::new` est documenté et logiquement
  inaccessible pour une `Url` HTTPS valide (une URL qui a passé
  `Url::parse` HTTPS aura toujours un host). Tous les autres
  `.unwrap()` dans le diff sont dans `#[cfg(test)]`. **PASS**.
- Loopback / wire / zip : aucun de ces chemins touchés. **PASS**.
- Design "pas d'injection memory_lookup" : tient la route. Le
  raisonnement est correct — `SignedPacket` est signé par la clé
  privée du node cible, un relai malveillant peut seulement
  refuser ou servir un stale, pas forger. Le canary détecte
  exactement ces deux cas. **PASS**.
- Fail-loud sur malformed URL : comportement correct. Un
  opérateur qui écrit `SBFB_PKARR_RELAYS=not-a-url` doit voir le
  daemon refuser de booter plutôt que silencieusement désactiver
  la défense eclipse. **PASS**.
- Pas de TOCTOU : `load_quorum_resolvers_from_env()` lit l'env
  une seule fois au boot, stocké dans un `Arc`. Pas de re-lecture
  ultérieure. **PASS**.
- `ENV_GUARD` Mutex statique dans les tests env : pattern
  identique à `relay_config::tests`, correct pour la
  parallélisation `cargo test`. **PASS**.

### Patterns

- SPDX header `// SPDX-License-Identifier: AGPL-3.0-or-later`
  présent en tête de `pkarr_resolver.rs`. **PASS**.
- `#[derive(Debug)]` supprimé sur `BrowseAggregator` et remplacé
  par `impl fmt::Debug` manuel : justification documentée (éviter
  `Debug` sur `dyn QuorumResolver`). Pattern correct et commenté.
  **PASS**.
- `Arc<dyn QuorumResolver>` : pas d'`unsafe`, pas de raw pointer.
  `Arc<Vec<Arc<dyn ...>>>` est idiomatique pour ce cas (snapshot
  immutable partagé). **PASS**.
- `async-trait` : pattern déjà établi dans le codebase
  (`QuorumResolver` S18), pas de dérive. **PASS**.
- `with_quorum_resolvers(Vec::new())` caller-bug branch documenté
  en doc-comment + warn log. **PASS**.
- **P3 #1 (pattern drift)** : `crates/nexus-core-rs/Cargo.toml`
  ajoute `url = "2"` en direct au lieu de
  `url = { workspace = true }`. Workspace déclare `url = "2.5"`
  à la ligne 73 du root `Cargo.toml`. Les deux crates existantes
  qui utilisent `url` (`nexus-launcher`, `nexus-worker-core`)
  passent par `{ workspace = true }`. `url = "2"` se résoudra en
  `2.5` par Cargo (compatible semver) donc pas de régression
  runtime, mais le pattern "workspace pin bypass" est incohérent
  et ouvre la porte à une future divergence si `url` bump sa
  version majeure. **Doit être corrigé avant commit** en
  `url = { workspace = true }`.

### Scope-cuts

Grep sur le diff pour les mots-clés des scope cuts kickoff §6 :

- `hashcash`, `proof-of-work`, `PoW` : absent du diff. **PASS**.
- `cert.pin`, `tls.pin` : absent. **PASS**.
- `delay.*queue`, `upload.*queue` : absent. **PASS**.
- `docker` : absent. **PASS**.
- `pkarr_relays.json`, `json.*loader` : le diff mentionne
  `~/.sbfb/pkarr_relays.json` dans un doc-comment comme "arrive
  Sprint 20+" — c'est une note forward, aucune implémentation
  introduite. **PASS**.
- `ML-DSA`, `ML-KEM`, `PQC` : absent. **PASS**.
- `rate.limit`, `kudos`, `duress` : absent. **PASS**.

Aucun scope leak détecté.

### Tests-delta

- Rust workspace : annoncé +9 (478 → 487), mesuré
  `cargo test --workspace --locked` : **487 total**. Delta +9
  confirmé. **PASS**.
- Branches critiques couvertes :
  - `probe_and_cache_skips_dial_when_quorum_has_no_majority` :
    vérifie `NoMajority` → `Unreachable` + skip dial (wall-clock
    < 1s). **PASS**.
  - `probe_and_cache_skips_dial_when_all_quorum_resolvers_fail` :
    vérifie `AllFailed` → `Unreachable` + cache. **PASS**.
  - Env loader : `returns_none_when_unset`,
    `parses_comma_separated_urls`, `fails_loud_on_bad_url`.
    **PASS**.
  - Branche `QuorumError::Empty` : couverte par exhaustiveness
    du match mais pas par test dédié — logiquement inaccessible
    post-`is_empty()` guard, justifié dans le commentaire.
    Acceptable.
- Python / Vitest / Playwright : diff ne touche aucun code
  Python, frontend, ni e2e. Delta 0 sur ces suites cohérent.
  **PASS**.

### Research-grounding

- **`url = "2"`** : pas de trace dans `sprint19_plan.md §3` (le
  plan note `url` est "transitive dep of iroh", mais pas de
  lookup context7/WebSearch dédié pour ce crate). Cependant
  `url 2.x` est stable depuis 2019, pas de CVE active, pas de
  version bump — **CONCERN P3** (pas P1 car c'est une dep connue
  inchangée en version effective). Corrigé dans le finding P3 #1.
- **`iroh::address_lookup::pkarr::PkarrRelayClient::new(url, tls_config)`** :
  le plan §3.1 documente la signature à 1 argument
  `new(pkarr_relay_url: Url)`. Le code utilise 2 arguments
  `new(url, tls_config)`. Discordance. Toutefois : (a)
  `cargo check -p nexus-core-rs` compile sans erreur confirmant
  que l'API iroh 0.97 réelle est bien 2-arg, (b)
  `iroh::tls::{CaRootsConfig, default_provider}` sont des exports
  réels iroh 0.97 (confirmé par grep codebase + compile). La
  session a donc corrigé la description pré-session en session.
  Pas P1 car la trace research est présente (context7 `rs_iroh`
  cité §3.1), la correction est implicite dans le code qui
  compile. **P3 #2 : le plan §3.1 devrait être rétroactivement
  noté "API réelle = 2-arg (url, tls_config)" pour qu'un auditeur
  futur ne soit pas bloqué par la discordance** — annotation à
  faire en Phase F avec la déviation `iroh_runtime.rs`.
- **`iroh::tls::CaRootsConfig`** : utilise `CaRootsConfig::default()`
  plutôt que `CaRootsConfig::embedded()` comme le doc-comment le
  mentionne. `default()` et `embedded()` peuvent différer selon
  la version iroh — le code compile donc c'est l'API valide,
  mais le commentaire `"see CaRootsConfig::embedded"` est
  légèrement trompeur. **P3 #3** — doit être corrigé avant commit
  pour ne pas égarer un futur auditeur TLS (Phase C sprint19 ira
  dans cette zone).
- **`SignedPacket::to_relay_payload()`** : documenté dans §3.1
  comme "format stable cross-relay". Aucune trace context7 dédiée
  à cet appel spécifique, mais il compile et les tests passent.
  Acceptable — pas une API crypto standardisée externe, c'est
  l'API interne iroh pkarr.
- Pas de nouvelle dépendance Cargo sans trace research P1.

---

## Findings

- **P3 #1** : `crates/nexus-core-rs/Cargo.toml` ligne 45 —
  `url = "2"` doit être `url = { workspace = true }` pour
  respecter le pattern workspace pin établi par `nexus-launcher`
  et `nexus-worker-core`. Corriger avant commit (1 ligne de diff
  dans `Cargo.toml`, pas de re-test nécessaire — même résolution
  effective 2.5.x).
  **Status : FIXED dans le commit d'intégration Phase A**.
- **P3 #2** : `sprint19_plan.md §3.1` — la description
  pré-session de `PkarrRelayClient::new` mentionne 1 argument.
  L'API réelle iroh 0.97 en exige 2 (`url + tls_config`). Annoter
  la correction dans `sprint19_verification.md` Phase F (même
  traitement que la déviation `iroh_runtime.rs`).
  **Status : reporté Phase F**.
- **P3 #3** : `pkarr_resolver.rs` ligne 92 — doc-comment écrit
  `"CaRootsConfig::embedded"` mais le code appelle
  `CaRootsConfig::default()`. Corriger le commentaire pour éviter
  confusion lors d'un futur audit TLS (Phase C sprint19 ira dans
  cette zone).
  **Status : FIXED dans le commit d'intégration Phase A**.

---

## Recommendation

Commit autorisé avec correction P3 #1 `url = { workspace = true }`
intégrée avant le push (1 ligne, pas de re-test nécessaire — même
résolution effective) + correction P3 #3 doc-comment
`CaRootsConfig::default` (au lieu de `::embedded`). P3 #2 est une
annotation doc à inclure dans `sprint19_verification.md` Phase F.

La déviation `iroh_runtime.rs` est honnête et correctement
motivée : grep confirme que `iroh_runtime.rs` n'a aucun call site
pkarr/resolve, le vrai call site prod unique est bien
`BrowseAggregator::probe_and_cache`. La déviation ne cache rien.

## Commit metadata

- Range : 1 commit `feat(sprint19): Phase A — DHT quorum runtime
  wire (browse aggregator canary)`.
- Delta tests attendu / mesuré : +9 Rust (478 → 487).
- Suites vertes : `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, `cargo test --workspace
  --locked`.
- Pas de modification Python / Vitest / Playwright — suites
  non-exécutées car hors périmètre ; Phase F les ré-exécutera au
  complet.
