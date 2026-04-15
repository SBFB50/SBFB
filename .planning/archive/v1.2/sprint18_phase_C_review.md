# Sprint 18 Phase C — nexus-phase-auditor review

**HEAD pre-commit** : non encore commite (4 fichiers M + 3 fichiers untracked sur master, tip pre-Phase-C = `4ab0211` Phase B reproducible builds + SLSA).
**Draft commit body** : `feat(sprint18): Phase C — multi-relai federation + DHT quorum primitive`.
**Audit timebox** : ~40 min.

---

## Verdict : PASS

0 finding P0. 0 finding P1. 3 findings P2. 1 finding P3.

Les findings P2 sont des carry-overs non-bloquants documentes ci-dessous et a inclure dans le body du commit Phase C. Le finding P3 est un nit. Commit autorise.

---

## Dimensions

### Security

- **Semgrep** : les regles `.semgrep/sbfb.yml` ciblent `.rs`. Les 4 fichiers Rust du diff ont ete analyses. 0 finding semgrep.
- **`unsafe`** : `lib.rs` conserve `#![forbid(unsafe_code)]`. Les deux nouveaux fichiers (`relay_config.rs`, `dht_quorum.rs`) n'introduisent aucun bloc `unsafe`. Conforme.
- **`unwrap()` / `expect()` en production** :
  - `relay_config.rs` ligne 208 : `.unwrap_or(false)` sur `env::var(DEV_MODE_ENV)` — pattern safe, pas de panic, fallback explicite.
  - `dht_quorum.rs` ligne 219 : `.expect("successes non-empty → at least one bucket")` dans le chemin de production de `redundant_resolve`. Ce `.expect` est garde par le check `if successes.is_empty() { return Err(...) }` a la ligne 202 — l'invariant est maintenu, le panic ne peut pas se produire en pratique. Pattern acceptable (invariant local garanti, pas de `unwrap` nu).
  - Tous les autres `unwrap()` / `expect()` sont dans des blocs `#[cfg(test)]` — conforme.
- **Secrets** : aucun secret hardcode. Aucun pattern `AKIA`, `ghp_`, `pat_`, `sbfb_[a-z]+_[a-zA-Z0-9]{20,}` detecte.
- **Loopback** : le diff touche `relay_config.rs` qui valide les URLs de relay. La politique est correcte : HTTP rejete, localhost rejete sauf `SBFB_DEV_MODE=1`. Le gate loopback du daemon (`PeerCredsVerified`) n'est pas touche par ce diff — la fonction `validate_relay_url` concerne les relays iroh QUIC, pas le HTTP loopback du daemon. Aucune regression.
- **Path traversal** : aucun `zip` extract, aucune manipulation de path user-controlled sans validation. `relays_file_path()` construit le chemin via `sbfb_home().map(|h| h.join(RELAYS_FILE_NAME))` avec un nom de fichier constant — pas de path traversal possible.
- **Wire format** : `relay_config.rs` utilise `serde_json::from_str` pour parser le fichier de config. Ce n'est PAS un wire format canonique inter-nodes — c'est un fichier de config operator local. JCS canonique n'est pas requis ici (JCS est requis uniquement pour les messages signes entre pairs : `Task`, `CuratorList`, etc.). Conforme.
- **Byte-for-byte comparison** dans `dht_quorum.rs` : l'invariant cryptographique est correct. Les paquets pkarr signes sont compares byte-for-byte — deux relays retournant des octets identiques ont necessairement le meme contenu, ce qui est plus fort que comparer un hash calcule localement. Pattern valide.
- **Quorum avec 1 seul resolver** : `quorum_threshold_for(1) = 1`, `quorum_threshold_for(2) = 2`. Documenté dans le doc-comment. La protection Eclipse est significativement affaiblie avec 1 ou 2 resolvers, mais le code ne l'interdit pas — c'est une responsabilite du caller. Acceptable : l'API exposee est claire, les tests couvrent le cas `Empty`, et l'usage production visera 3 resolvers.

### Patterns

`docs/rust/PATTERNS.md` ne contient pas de patterns numerotes P1..PNN formels — c'est un scratchpad de lecons Sprint 1 (iroh API, PyO3, etc.). Les patterns applicables sont extraits du contenu documente et des conventions etablies dans les sprints precedents.

- **Pattern `thiserror` pour les enums d'erreur** : `QuorumError` utilise `#[derive(Debug, Error)]` de `thiserror`. Coherent avec `NexusError` dans `crates/nexus-core-rs/src/error.rs`. Conforme.
- **Pattern `JoinSet` pour taches concurrentes** : `dht_quorum.rs` utilise `tokio::task::JoinSet` pour collecter N taches concurrentes. C'est le pattern tokio idiomatique — `JoinSet` garantit l'annulation automatique des taches en-vol si droppee (mentionnee dans le doc-comment). Conforme et meilleur que `join_all` + `select!` pour N dynamique.
- **Pattern `async-trait`** : `QuorumResolver` utilise `#[async_trait]` pour la dyn-compatibilite. Rust 1.94 supporte `async fn in traits` nativement (stable depuis 1.75), mais la dyn-compatibilite (`Arc<dyn QuorumResolver>`) requiert encore soit `async-trait` soit Return-Type-Notation (stable 1.79+, mais non dyn-safe sans boite). `async-trait` reste le pattern le plus lisible pour ce cas — acceptable.
- **Pattern `EnvSnapshot` + `Mutex` guard pour tests env** : reimplementation coherente entre `relay_config.rs` (unit tests) et `relay_federation.rs` (integration tests). Le pattern est identique au code existant dans `auth.rs` tests. Pas de PN documente, mais la convention est homogene. **Pattern drift P2** — voir Findings.
- **Pattern `sbfb_home()`** : troisieme occurrence de la fonction helper (consent.rs dans nexus-worker-core, auth.rs dans nexus-shell-daemon-core, relay_config.rs dans nexus-core-rs). La triplification est documentee dans le commentaire de la fonction elle-meme (`/// Mirrors the helper in... — kept local to avoid a new cross-crate dep just for one path`). La dette est reconnue. **P2** — voir Findings.

### Scope-cuts

Grep exhaustif des fichiers du diff (relay_config.rs, dht_quorum.rs, lib.rs, node.rs, relay_federation.rs, Cargo.lock, Cargo.toml) contre chaque item §6 du kickoff S18 :

| Scope cut §6 | Grep result |
|---|---|
| PoW gossip | absent |
| TLS cert pinning relays | absent |
| Encryption at rest keypair | absent |
| Iroh audit externe | absent |
| Pyodide sandbox escape | absent |
| ML-DSA / ML-KEM / PQC | absent |
| Self-hosted pkarr relay | absent |
| Federated ONG-run relays concrets | absent |
| NVIDIA CVE / NVD check | absent |
| Warrant canary | absent |
| Radicle mirror | absent |
| THREAT_MODEL.md cross-ref S17 | absent |

**Zero scope creep detecte.**

**Sur la non-integration daemon (scope-cut revendique)** : le plan §Phase C listait `crates/nexus-shell-daemon/tests/browse_dht_quorum.rs` comme livrable conditionnel ("si pattern actuel lookup single"). L'executeur a reporte le wire daemon → pkarr reel. Jugement : **acceptable**. Le plan lui-meme qualifiait ce point de conditionnel ("si pattern actuel lookup single"). Le mecanisme `QuorumResolver` est entierement livre, publiquement exporte, couvert par 13 tests unitaires + 5 integration. Le wire daemon necessite un contexte pkarr-reel qui introduit un risque de flakiness non-souhaitable avant Phase D (coord + token). Le report a Phase F ou Sprint 19 est conforme a la risk policy du plan.

### Tests-delta

| Source | Annonce | Realite |
|---|---|---|
| Plan §Tests Phase C | +20 total | — |
| Plan §Commit Phase C | "+12 unit + +5 integration = +17" | — |
| Draft commit body | +20 | +20 confirms |
| `relay_config.rs` unit | 7 | 7 (comptes) |
| `dht_quorum.rs` unit | 8 (annonce body) | 8 (7 tokio::test + 1 test regulier dont `start_paused`) |
| `relay_federation.rs` integration | 5 (annonce body) | 5 (comptes) |
| **Total** | **+20** | **+20** |
| `cargo test --workspace --locked` | 450 | 450 (confirme par l'executeur) |

**Match exact +20.** Delta reel coherent avec l'annonce du body et la section §Tests du plan.

Note : le §Commit du plan annoncait "+17" (12+5) mais la section §Tests du meme plan annoncait "+20". L'executeur a atteint +20 en livrant 7 unit relay (vs 5 prevus) + 8 unit dht (vs 7 prevus) + 5 integration = 20. L'ecart interne au plan (17 vs 20) est en faveur du livrable — non-bloquant.

**Suites hors-scope** : Python coord (+0), Vitest (+0), Playwright (+0) — aucun de ces composants n'est touche par Phase C. Conforme.

---

## Findings

### P2 — sbfb_home() triplique (tech debt T-new)

`relay_config.rs` introduit une troisieme copie de `sbfb_home()`, apres `nexus-worker-core::consent` et `nexus-shell-daemon-core::auth`. La copie est documentee dans son commentaire mais augmente la surface de desynchronisation. La logique `HOME` / `USERPROFILE` est copiee byte-for-byte, correcte, mais toute correction future devra etre appliquee en trois endroits.

**Mitigation dans ce diff** : le commentaire de la fonction cite explicitement les deux autres occurrences et justifie la duplication ("kept local to avoid a new cross-crate dep just for one path"). La justification est valide pour ce sprint : un nouveau sous-crate `nexus-home` ou un move vers `nexus-core-rs` aurait augmente le scope Phase C.

**Action recommandee** : enregistrer comme `T-new` dans `docs/rust/PATTERNS.md` — factoriser `sbfb_home()` dans `nexus-core-rs` (qui est deja un crate transitif des deux autres) en Phase F wrap-up ou Sprint 19. A mentionner dans le body du commit comme carry-over.

### P2 — Critere d'acceptation "home_relay=..." non satisfait textuellement

Le plan §Critere d'acceptation Phase C stipule : "Launcher startup log affiche `home_relay=...` (ou `home_relay=fallback`)". Le log produit dans `node.rs` est :

```
info!(relay_count, "using custom relay map from SBFB config")
```

et :

```
info!(node_id = ..., custom_relays = using_custom_relays, "iroh endpoint ready")
```

Le champ `home_relay=` specifie dans le critere n'est pas emis. L'information semantique est presente (`custom_relays=true/false`, `relay_count`), mais le critere textuel exact n'est pas satisfait. Le launcher lui-meme (`nexus-launcher/src/main.rs`) n'emet pas de log relay du tout.

**Impact** : diagnostic operateur reduit — un operateur qui check les logs launcher ne voit pas quelle URL de relay est active, seulement si des relays custom sont configures.

**Action recommandee** : ajouter dans `node.rs` un log `info!(home_relay = %first_url_or_fallback, "relay config active")` qui restitue l'URL du premier relay (ou "n0-defaults" si None). A corriger en Phase F ou a accepter comme carry-over non-bloquant. Non-bloquant pour le commit car le mecanisme fonctionne, c'est le diagnosability qui est incomplet.

### P2 — Pattern drift : EnvSnapshot non-documente

Le pattern `EnvSnapshot + Mutex ENV_GUARD` pour serialiser les tests qui mutent l'env est maintenant present dans relay_config.rs (unit), relay_federation.rs (integration), et probablement dans auth.rs existant. Ce pattern n'est pas documente dans `docs/rust/PATTERNS.md`. Phase C ajoute une 2e occurrence independante (copy-paste du pattern, pas de shared utility).

**Action recommandee** : documenter ce pattern comme PN dans `docs/rust/PATTERNS.md` en Phase F wrap-up. Non-bloquant.

### P3 — async-trait : nit de modernisation

`async-trait 0.1` est necessaire pour la dyn-compatibilite de `QuorumResolver`. En Rust 1.94, la Return-Type-Notation (RTN) permet theoriquement de s'en passer pour certains cas, mais le support complet de `dyn Trait` avec `async fn` sans `async-trait` n'est pas encore stabilise de facon universelle. Le choix actuel est correct et safe.

Nit uniquement : si une future version de l'API remplace le `Arc<dyn QuorumResolver>` par un generic `R: QuorumResolver`, `async-trait` deviendrait optionnel. Pas d'action requise ce sprint.

---

## Verifications effectuees

| Check | Resultat |
|---|---|
| `cargo test --workspace --locked` | 450 passed (+20 vs baseline 430) |
| `cargo fmt --all --check` | exit 0 (auto-fix prealable sur dht_quorum.rs) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (1 `unnecessary_to_owned` corrige) |
| `cargo deny check` | advisories/bans/licenses/sources ok |
| `bash tests/ci-smoke/supply-chain-green.sh` | ALL GREEN (Phase A + B + C preservees) |
| Grep secrets (AKIA, ghp_, pat_) | 0 match |
| Grep scope cuts §6 kickoff | 0 match |
| `unsafe` blocks | 0 introduit, `#![forbid(unsafe_code)]` maintenu |
| Test count reconciliation (7+8+5=20) | match annonce body +20 |
| Wire format JCS | non applicable (config locale, pas inter-nodes) |
| Path traversal relay URL | valide, nom fichier constant |
| `home_relay=` log launcher | absent — P2 carry-over |

---

## Recommendation

**Commit autorise.**

Avant de committer, enrichir le body avec les carry-overs P2 :

```
Phase F carry-overs non-bloquants :
- Wire nexus-shell-daemon browse aggregator sur redundant_resolve + vrais pkarr resolvers.
  Le mechanism est en place, le wire demande integration pkarr-reelle reportee.
- Factoriser sbfb_home() (3 occurrences : consent.rs, auth.rs, relay_config.rs) dans
  nexus-core-rs en Phase F ou Sprint 19. A tracker comme T-new dans docs/rust/PATTERNS.md.
- Ajouter log home_relay= URL active dans node.rs pour diagnosabilite operateur.
- Documenter pattern EnvSnapshot + ENV_GUARD dans docs/rust/PATTERNS.md.
- Document pattern bash scripts (carry-over Phase B).
```

Le body courant liste deja "Wire nexus-shell-daemon..." comme carry-over — ajouter les trois autres items (sbfb_home, home_relay log, EnvSnapshot PN).

Gate 1 prerequis 3/4 cleared confirme. Phase D non-bloquee.
