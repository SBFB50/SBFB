# Sprint 81 Phase E2 — Review (Workflow ultracode + agent de synthèse)

> Phase E2 « PLAN B C8 zéro-n0 » (nom canonique, regex README §4
> `Phase [A-Z]+[0-9]?` ; « E' » = alias prose), split-off code-neuf de
> E-core (`efb9667`). Contrat = `sprint81_phase_e2_preflight.md` (verdict
> **PLAN-ADAPT**, 6 adaptations load-bearing §blockquote + 7 contraintes §4
> + plan de tests §5). Périmètre = préflight E §7.3 a-e : runbook +
> templates + CODE NEUF override discovery gated-env (`presets::Minimal` +
> re-push `memory_lookup` + `PkarrPublisher`/`PkarrResolver::builder` HTTP
> [Option B] + `RelayMode::Custom`) + relais/pkarr self-hosted VPS +
> acceptance zéro-n0 T2. Arbre SALE, HEAD `efb9667`, diff NON committé
> (6 fichiers modifiés + 4 untracked : module neuf `discovery_override.rs`,
> runbook neuf `IROH_SELFHOST_OPS.md`, T2 JSON, préflight). Review menée sur
> le diff COMPLET + grounding vendored `iroh-1.0.1` / repo, synthèse de
> **7 dimensions** (D1 correctness / D2 branch-coverage / D3 préflight /
> D4 sécurité deep / D5 livrables+docs / D6 wire+invariants / D7 patterns) +
> vérifications adversariales, réconciliées ici. **LECTURE SEULE — 0
> cargo/npm relancé** (pipeline Docker sbfb-ci en parallèle, contention
> cargo interdite ; suites §7.4 fournies au contexte).

## Verdict: PASS

> **Diff Phase E2 substantivement CONFORME au contrat préflight
> PLAN-ADAPT.** 0 P0. **0 P1.** 2 P2 CONFIRMÉS (couverture de branche, 0
> défaut runtime) + une douzaine de P3 (fixes cheap / body / carry). Le
> PLAN-ADAPT in-phase — fake pkarr **PUT+GET** maison au lieu de
> `DnsPkarrServer` — a été **vérifié À LA SOURCE par 6 dimensions** et se
> révèle CORRECT et strictement SUPÉRIEUR au plan initial : le serveur pkarr
> de `test-utils` ne route QUE `put()` (`iroh-1.0.1/src/test_utils.rs:310`),
> donc il ne peut PAS exercer le chemin de résolution PROD (Option B
> `PkarrResolver` GET) ; le fake PUT+GET raw-tokio l'exerce EXACTEMENT,
> `run_relay_server` (test-utils) reste pour le vrai relais iroh. Les 6
> adaptations et les 7 contraintes §4 sont TOUTES honorées, les invariants
> non négociables TOUS tenus (presets::N0 défaut préservé sémantiquement,
> 0 bump wire, iroh strictement seul via dev-dep resolver-v2, verrous
> S74/S75 intacts, duress non re-gaté). Un finding P2 de D5 (frontière
> docs-contrat §6.12 « opérateur ») a été **REFUTED** en vérification et
> n'apparaît donc PAS comme un fait ci-dessous.
>
> **PASS-PENDING = 0 fix obligatoire, mais un lot de fixes cheap à trancher
> AVANT Codex.** Le seul finding que la mission ciblait explicitement
> (D2-1 : branche `Url::parse` erreur jamais exercée, P2, corrigeable par 1
> assertion) mérite un fix-in-phase ; les autres P2/P3 sont soit
> documentables au body avec justification (D2-2 structurel), soit des
> corrections doc/commentaire d'une ligne (metrics loopback, ACME ALPN-01,
> commentaire Cargo.toml). Une fois le lot fix-in-phase appliqué et le reste
> tranché au body/carry, le gate Codex peut promouvoir `## Verdict: PASS`.
>
> Séquence : appliquer les fixes 1-4 → documenter D2-2 + P3-body → carry
> G/K → Codex → réconciliation → promotion PASS → commit `feat(core)`.

## Périmètre et staging

`git status --short` = **10 lignes** (6 modifiés + 4 untracked planning/doc),
0 fichier parasite :

- `Cargo.lock` (purement additif : 7 crates de TEST via feature `test-utils`
  — `tokio-rustls-acme`, `reloadable-core`/`-state`,
  `rustls-cert-file-reader`/`-read`/`-reloadable-resolver`, `sha1 0.11.0` ;
  0 version existante bougée ; `sha1`→`sha1 0.10.6` = désambiguïsation
  mécanique due à `sha1 0.11.0`).
- `crates/nexus-core-rs/Cargo.toml` (dev-dep `iroh features=["test-utils"]`
  sous `[dev-dependencies]` + commentaire rationale anti-migration).
- `crates/nexus-core-rs/src/lib.rs` (`pub mod discovery_override` +
  re-exports alphabétiques, 0 collision).
- `crates/nexus-core-rs/src/node.rs` (restructuration
  `create_node_with_protocols` : résolution `load_discovery_override()`
  AVANT builder ; branche `Some` = `presets::Minimal` +
  `apply_zero_n0_discovery` ; branche `None` = chemin N0 préservé ; fn
  `apply_zero_n0_discovery` prod ; test Tier B
  `zero_n0_two_nodes_converge_via_self_hosted_stack` + helper
  `run_fake_pkarr_relay` PUT+GET).
- `crates/nexus-core-rs/src/relay_config.rs` (refactor DRY : extraction
  `enforce_url_policy` pub(crate), `validate_relay_url` byte-identique).
- `docs/release/PKARR_RELAY_OPS.md` (blockquote portée pubky/Mainline =
  canari + cross-ref `IROH_SELFHOST_OPS`).
- `?? crates/nexus-core-rs/src/discovery_override.rs` — module neuf
  (`ZERO_N0_ENV`/`ZERO_N0_PKARR_RELAYS_ENV`/`DiscoveryPlan`/
  `load_discovery_override`/`validate_zero_n0_pkarr_url` + 6 tests Tier A).
- `?? docs/release/IROH_SELFHOST_OPS.md` — runbook neuf zéro-n0.
- `?? .planning/active/sprint81_t2_e2_zero_n0.json` — artefact T2 (palier
  binaires PASS + palier live RIG-ABSENT tracé).
- `?? .planning/active/sprint81_phase_e2_preflight.md` — artefact préflight
  (contrat, ne se review pas lui-même).

## Vérification trois blocs

Suites §7.4 **fournies au contexte (déjà jouées), auditées en cohérence —
non relancées** :

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  **2047/2047 0-skip** (baseline E 2040 → **+7 EXACT** : 6 Tier A
  `discovery_override` + 1 Tier B `node.rs`) ; doctests 6/6 core + 1 worker ;
  release build `nexus-shell-daemon` vert. (1er run = crash rustc infra
  `STATUS_STACK_BUFFER_OVERRUN` « iroh_blobs rlib » = classe contention cargo
  documentée ; re-run complet vert.)
- **Docker sbfb-ci** rust:1.94 : **EN COURS au lancement de la review** —
  résultat réconcilié par le main thread AVANT commit.
- **Front (web + operator)** : lint 0 err + tsc + 411/411 web + 201/201
  Vitest operator + gates 6/6 + E2E Playwright 10/10 + coverage
  87.27/79.01/86.02/88.59 + build + size 6/6 + scan-en-strings clean.
  **AUCUN fichier front touché** (`git status` = 0 `web/`/`tools/`) →
  insensible par construction. 4 flakys de charge
  (`AddAnchorDialog`/`GpuConsentDialog`) requalifiés solo 22/22 puis full
  411/411 = classe `vitest_env_variance`.

**Delta tests** : **+7 Rust net**, cohérent partout, 0 zombie, aucun test
`legacy-decode` (E2 ne redéfinit aucun format persisté). Aucun re-test
interdit par le préflight §5 (handshakes seed/shard non ré-implémentés).

## Table des findings retenus

| id | sév | claim | evidence | disposition |
|---|---|---|---|---|
| **D2-1** | **P2** | La branche erreur `Url::parse` de `validate_zero_n0_pkarr_url` n'a aucun garde de régression : tous les tests fournissent des chaînes parseables qui échouent en aval sur la politique scheme/loopback. Mission-ciblée, fix cheap (1 assertion). | `discovery_override.rs:200` (branche) vs `:379-398` (test omet le cas parse) | **fix-in-phase** |
| **D2-2** | **P2** | Le choix de base-preset `presets::Minimal` de la branche `Some` (`node.rs`) n'est pas couvert E2E : le test Tier B hand-build lui-même `Endpoint::builder(presets::Minimal)` et appelle `apply_zero_n0_discovery` directement, court-circuitant `create_node_with_protocols`/`load_discovery_override`. Une régression `Minimal`→`N0` au site prod re-câblerait n0 par-dessus l'override SANS qu'aucun test échoue. Structurel, non fermable hermétiquement (le fake est http-loopback, rejeté par `enforce_url_policy` https-only). | `node.rs:349` (sélection `Minimal`, non testée) vs `:853-854` (Tier B hard-code Minimal) | **document-in-body** |
| **D2-3** | P3 | La case-insensitivity du chemin ON du gate est non testée : ON accepte `1`/`true` (lowercase) seul ; `TRUE`/`True` non exercé (OFF exerce `FALSE`). Une régression étranglant le ON à `trimmed=="true"` ferait tomber `TRUE` en erreur non-reconnue silencieusement. | `discovery_override.rs:139` vs `:352` (`"true"` lowercase seul) | fix-in-phase (cheap) |
| **E2-METRICS** | P3 | Runbook §4.1 n'écrit pas `metrics_bind_addr` sur la loopback ; par `iroh-relay main.rs` le défaut dérive de `http_bind_addr` (`[::]:80`) → metrics = `[::]:9090` (TOUTES interfaces), alors que §2 (table ports) et §7 affirment « metrics loopback only ». Table firewall n'ouvre pas 9090 (defense-in-depth), mais la claim est factuellement fausse. Le plus security-relevant des P3. (Fusion E2-D4-2 + D5-2.) | `IROH_SELFHOST_OPS.md:83`/`:240` (claim loopback) vs template §4.1 sans `metrics_bind_addr` | fix-in-phase (ajouter `metrics_bind_addr = "127.0.0.1:9090"`) |
| **D5-4** | P3 | Le commentaire rationale de la dev-dep `test-utils` (`Cargo.toml`) sur-cite `DnsPkarrServer` comme brique fournie, alors que le test Tier B ne l'utilise PAS (fake maison PUT+GET, car pkarr `test-utils` = PUT-only). L'import load-bearing réel = `run_relay_server`. | `Cargo.toml:146` vs `iroh-1.0.1/src/test_utils.rs:310` | fix-in-phase (commentaire) |
| **D5-3** | P3 | Runbook §4.1 commente `http_bind_addr = [::]:80` comme « ACME HTTP-01 » ; `iroh-relay` utilise `tokio-rustls-acme` `letsencrypt` = challenge TLS-ALPN-01 sur `:443`, jamais HTTP-01 sur `:80`. La valeur de config est correcte (copier-coller sans danger) ; seule l'attribution du commentaire est imprécise. | `IROH_SELFHOST_OPS.md:115` vs `iroh-relay-1.0.1/src/main.rs:651` | document-in-body (ou fix commentaire) |
| **D1-P3-1** | P3 | Dans le test Tier B, la porte de publication (`assert!` non-vide du store) est satisfaite par un paquet d'ep2 alors que le message accuse ep1. PAS un bug : la vraie preuve reste `ep2.connect(ep1.id())` (404 → resolve Err → timeout → ROUGE). Imprécision d'attribution de diagnostic. | `node.rs:891`/`:896` | document-in-body |
| **E2-D4-1** | P3 | `validate_zero_n0_pkarr_url` n'exige ni host non-vide ni path `/pkarr` : `https:///pkarr` (host vide) passe la garde. Pas un risque de redirection (host vide n'atteint aucune infra) ; robustesse pure, échec révélé loud à l'acceptance. Tolérance analogue à la garde relais S18. | `discovery_override.rs:199-207` | document-in-body |
| **D5-5** | P3 | `IROH_SELFHOST_OPS` §7/§8 renvoient à « THREAT_MODEL Phase G » (ligne zéro-n0 15.x) non encore atterrie au commit E2 (routée carry G). Cross-ref pendante ; pattern de carry honnête (docs, hors gate `check-frontier-contracts.sh`). | `IROH_SELFHOST_OPS.md:242`/`:249` ; préflight carry G `:410` | carry-G / document-in-body |
| **D7-2** | P3 | Candidat pattern neuf à router K : fonction de décision PURE env→Plan\|erreur comme SEAM testable quand le `Builder` iroh est opaque (pas de getter pré-bind), couplée fail-loud + politique d'URL partagée via `pub(crate) enforce_url_policy`. Aucun §P5x/P7x existant ne le capture. | `discovery_override.rs:131` + `relay_config.rs:212` | carry-K (proposer §P6x/P7x au wrap-up) |
| **D7-1** | P3 | Le test Tier B two-node n'est PAS dans le test-group `two-node-convergence` (filtre scopé `package(nexus-shell-daemon)`), MAIS c'est CONFORME au précédent `nexus-core-rs` (`two_nodes_fetch_blob_via_ticket`, `two_nodes_sync_via_share_import` idem hors-groupe, couverts par `profile.ci retries=1`). Élargir le groupe = durcissement OPTIONNEL (délais internes 30s+30s, worst-case 60s < kill 90s). | `.config/nextest.toml:39` ; `node.rs:804` ; `blobs.rs:410`/`docs.rs:614` | document-in-body |
| **D7-3** | P3 | Incohérence cosmétique de wrapping : branche `None` enveloppe l'erreur `load_relay_map` en `NexusError::Endpoint("invalid relay config: …")` ; le chemin zéro-n0 propage l'erreur brute. Les deux restent des `NexusError::Endpoint` fail-loud, 0 perte d'info. | `node.rs:361` vs `discovery_override.rs:173` | document-in-body |
| **E2-D4-3 / D6-DOC** | P3 | Le dev-dep `test-utils` élargit le graphe de compilation de TEST workspace-wide (axum + ACME + rcgen + rustls-cert-* + reloadable-*) sous `cargo test --workspace`/`clippy --all-targets`. Coût compile réel, PAS une régression : resolver v2 + `force_staging` env-only garantissent 0 fuite release + 0 bascule N0→staging. Anticipé préflight §5. + nuance doc : la raison la plus fondamentale du no-leak = les dev-deps d'une dépendance ne sont jamais dans un graphe release (indépendant du resolver). | `Cargo.toml:142-154` ; `Cargo.lock` (7 crates) ; `iroh-1.0.1/src/endpoint.rs:1970` | document-in-body (0 action, transparence blast-radius) |

**REFUTED (n'apparaît PAS comme fait)** : `D5-1` (P2) — « la surface env
zéro-n0 (`SBFB_ZERO_N0*` + `relays.json`) est une frontière docs-contrat
§6.12 non enregistrée ». Vérification : FAUX. Le test-acteur §6.12 énumère
des acteurs MACHINE lisant des contrats que le code ÉMET/SERT (nœud=wire,
client externe=API loopback servie, app=CSP, LLM=knowledge pack). Un
opérateur humain qui POSE une env var selon un runbook n'en est aucun : les
env vars sont des INPUTS que le code LIT (`discovery_override.rs:132`), le
lecteur machine étant `nexus-core-rs` lui-même. Le gate
`check-frontier-contracts.sh` est scopé `crates/`+`web/src/`, `docs/`
explicitement exclu → un runbook `docs/release/*` est hors surface-frontière.
`relays.json` = format S18 pré-existant qu'E2 documente. La discipline
correcte pour une config env = §6.9 named-const, **satisfaite** (`ZERO_N0_ENV`
+ `ZERO_N0_PKARR_RELAYS_ENV` = `pub const`) ET déjà portée au carry K
(préflight `:421`). Agir sur ce « fix » injecterait une frontière §6.12
fantôme polluant le registre Track K.

## Restitution des dimensions

| Dim | Portée | Jugement local | Réconciliation |
|---|---|---|---|
| **D1** — correctness | Sémantique des 5 concerns mission (branche None identique / ordre Vec lookup / parsing / fake pkarr / refactor relay) | **PASS** | Tous vérifiés à la source, 0 bug P0/P1. Branche None byte-identique (champs disjoints du Builder), ordre Vec neutre (resolve concurrent), fake PUT+GET ne peut faire faux-vert (parse corrompu → resolve Err → timeout borné → ROUGE), refactor byte-identique. 1 P3 (attribution diagnostic). |
| **D2** — branch-coverage | +7 tests, hermétisme, drift réel, branches | **CONCERN** | +7 recompté exact, 0 zombie. Tier B NON faux-vert (Minimal ne câble rien n0, MemoryLookup fraîche vide, résolution uniquement via pkarr GET). 3 gaps de branche : **D2-1** (parse-error jamais exercé, P2 CONFIRMED, mission-ciblé) + **D2-2** (Some-arm non couvert E2E, P2 CONFIRMED, structurel) + D2-3 (case-insensitivity ON, P3). |
| **D3** — préflight PLAN-ADAPT | 6 adaptations §blockquote + 7 contraintes §4 + §7.3 a-e + Option B | **PASS** | 6/6 adaptations honorées, 7/7 contraintes appliquées, PLAN-ADAPT in-phase (fake PUT+GET) vérifié STRICTEMENT SUPÉRIEUR au plan initial (test-utils pkarr = PUT-only → DnsPkarrServer aurait résolu via DNS = Option A, PAS notre chemin prod Option B). Option B câblée code+runbook. 3 P3 = confirmations positives (PLAN-ADAPT supérieur, sonde T2 offline-non-revérifiable honnête, dev-dep no-leak prouvé). |
| **D4** — sécurité deep | Env poisoning / duress / verrous S74-S75 / fuite adresse / runbook durci / chemin défaut | **PASS** | Garde `SBFB_ZERO_N0_PKARR_RELAYS` = MÊME `enforce_url_policy` que S18 (0 asymétrie) ; intégrité bornée crypto (pkarr hostile = censor/stale, jamais forge → carry G) ; duress non re-gaté (grep : commentaires seuls) ; verrous S74/S75 intacts (grep vide) ; `AddrFilter::relay_only()` par défaut → 0 fuite IP ; `force_staging` env-only → dev-dep ne flippe pas N0→staging ; trust prod WebPKI-only préservé. 3 P3 (E2-D4-1 host-vide, E2-METRICS, E2-D4-3 blast-radius). |
| **D5** — livrables + docs | Exactitude runbook vs sources vendored + shape T2 + trilingue | **PASS** | Toutes les claims config VÉRIFIABLES exactes vs `iroh-relay-1.0.1` vendored (struct `Config`/`TlsConfig`/`CertMode` PascalCase, ports 80/443/7842/9090, `--dev` 3340, `[[bin]]`) + `iroh-dns-server-1.0.1` via docs.rs (`cert_mode` snake_case, `[mainline] enabled=false`, `[http]` 8080). T2 shape conforme A3/A4/E, palier PASS re-vérifiable, RIG-ABSENT traçable, scoping_note distingue survives-VPS-death S75. **D5-1 (P2) REFUTED** (frontière §6.12 fantôme). 4 P3 doc (metrics, ACME, DnsPkarrServer over-cite, THREAT_MODEL Phase G pending). |
| **D6** — wire + invariants + deps | 0 bump wire, Cargo.lock additif, dev-dep isolé, N0 défaut, PLAN-ADAPT | **PASS** | 0 bump par construction (23 `DOMAIN_*_V1` intacts, `sbfb/seed/0`+`sbfb/shard/1` verbatim, tous `*_FORMAT_VERSION=1` ; `DiscoveryPlan` runtime-only sans serde ; `TEST_ALPN` `#[cfg(test)]`). Cargo.lock purement additif (7 crates test, `sha1` relabel = désambiguïsation, pins `=1.0.1/=0.101.0/=0.103.0` intacts). Dev-dep `[dev-dependencies]` seul + resolver=2 → 0 fuite release. N0 défaut préservé (`force_staging` env-only vérifié). 7 findings = confirmations d'invariants (refute-candidates) + 1 P3 doc-nuance resolver. |
| **D7** — patterns + conventions | Constantes nommées, missing_docs, SAFETY, EnvSnapshot, DRY, nextest-group | **PASS** | Constantes nommées uniques, items publics documentés (missing_docs+clippy-D), SAFETY sur chaque `unsafe env`, `EnvSnapshot`/`ENV_GUARD` conforme aux frères, `enforce_url_policy` DRY byte-identique. PLAN-ADAPT correct à la source. 3 P3 : nextest-group (conforme précédent), pattern neuf → carry K, wrapping cosmétique. |

**Convergence** : 6 PASS + 1 CONCERN (D2). Aucun finding bloquant-avant-commit
(0 P0/P1). Les 2 P2 de D2 sont CONFIRMÉS en vérification, sans upgrade : D2-1
= fix cheap mission-ciblé (fix-in-phase), D2-2 = limitation structurelle
non-fermable hermétiquement (document-in-body honnête). Le seul P2 potentiel
qui aurait pu être bloquant (D5-1) est REFUTED. → verdict global
**PASS-PENDING**, jamais CONCERN (aucun P1 exigeant un fix) ni FAIL.

## Vérifications à la source (positives, load-bearing)

- **PLAN-ADAPT vérifié STRICTEMENT SUPÉRIEUR** :
  `iroh-1.0.1/src/test_utils.rs:310` route `/pkarr/{key}` en `put()` SEUL —
  aucun GET. `DnsPkarrServer` aurait résolu via `DnsAddressLookup` (chemin
  Option A, `presets.rs:131-134 #[cfg(not(wasm_browser))]`), PAS le chemin
  prod Option B `PkarrResolver` HTTP GET. Le fake PUT+GET raw-tokio exerce
  donc EXACTEMENT le chemin de résolution prod. `run_relay_server`
  (test-utils) gardé pour le vrai relais iroh in-process (non fakeable
  trivialement) → dev-dep justifié.
- **Tier B NON faux-vert** : `presets::Minimal` ne câble rien n0
  (crypto-provider seul), `MemoryLookup` fraîche vide par endpoint,
  `RelayMode::Custom` vers le relais in-process → seule résolution possible =
  pkarr GET, seul dial = relais custom. Toute corruption de parse HTTP →
  `SignedPacket::from_relay_payload` échoue → resolve Err → timeout 30s
  borné → ROUGE.
- **Chemin défaut byte-identique** : `Endpoint::builder(preset)` applique le
  preset à la CONSTRUCTION ; le réordonnancement `secret_key` vs
  `relay_mode` est inerte (champs disjoints du `Builder`,
  `endpoint.rs:129-150`). Vec `address_lookup` N0 inchangé. Seul delta log =
  champ additif `zero_n0=false`.
- **Fuite adresse** : `PkarrPublisherBuilder` défaut = `AddrFilter::relay_only()`
  (`pkarr.rs:168`, doc « avoids leaking IP addresses ») ;
  `apply_zero_n0_discovery` ne pose PAS `.addr_filter()` → relay-only
  conservé, même posture que N0.
- **Refactor `enforce_url_policy`** : byte-identique pour `what="relay url"`
  (messages scheme + loopback identiques, gate `dev_mode` identique,
  `pub(crate)` correctement partagé). NE touche PAS le loader canari
  `load_quorum_resolvers_from_env`.

## Dispositions

**À corriger AVANT commit (fix-in-phase)** — aucun n'est obligatoire (0
P0/P1), tous cheap et recommandés :

1. **D2-1 (P2)** — ajouter une assertion dans
   `pkarr_url_policy_matches_relay_policy` (ou via `load_discovery_override`)
   pilotant une chaîne génuinement non-parseable
   (`validate_zero_n0_pkarr_url("not a url").unwrap_err()` contient « not a
   valid URL »), fermant la branche `Url::parse` erreur
   (`discovery_override.rs:200-204`). **Mission-ciblé, préflight §5 le
   nommait « fail-loud malformé ».**
2. **D2-3 (P3)** — dans le même module de test, ajouter le cas ON
   case-insensitive (`SBFB_ZERO_N0=TRUE` / `True` → gate ON), miroir du
   `FALSE` déjà présent côté OFF.
3. **E2-METRICS (P3, le plus security-relevant)** — ajouter
   `metrics_bind_addr = "127.0.0.1:9090"` au template `config.toml` du
   `IROH_SELFHOST_OPS.md` §4.1, pour rendre vraie la claim « metrics loopback
   only » (§2/§7).
4. **D5-4 (P3)** — corriger le commentaire dev-dep `Cargo.toml` : citer
   `run_relay_server` comme brique load-bearing ; `DnsPkarrServer` non
   utilisé (pkarr `test-utils` = PUT-only).

**À trancher au body (documenter avec justification, non bloquant)** :

- **D2-2 (P2)** — documenter honnêtement au body que le choix `Minimal` de
  la branche `Some` (`node.rs:349`) n'est pas couvert E2E (limitation
  structurelle : le fake http-loopback est rejeté par `enforce_url_policy`,
  la couverture E2E hermétique du site prod est non-fermable) ; le seam
  testable `apply_zero_n0_discovery` EST couvert. L'invariant N0-défaut
  concerne la branche `None` (préservée) ; le risque résiduel est la
  sélection Some non-testée.
- **D5-3 (P3)** — corriger l'attribution ACME (`:80` = services HTTP /
  captive portal, ACME = TLS-ALPN-01 sur `:443`) au commentaire §4.1 + §2,
  ou noter au body ; conclusion host-dédié inchangée.
- **D1-P3-1, E2-D4-1, D7-1, D7-3, E2-D4-3/D6-DOC** (P3) — documenter au body
  (attribution diagnostic Tier B ; host-vide edge robustesse ; nextest-group
  conforme précédent ; wrapping cosmétique ; blast-radius compile-test 0
  action).

**À router en carry** :

- **D5-5 (P3) → carry G** — cross-ref `THREAT_MODEL §15.x zéro-n0` pendante
  jusqu'à l'atterrissage de Phase G (déjà tracé préflight carry G).
- **D7-2 (P3) → carry K** — proposer au wrap-up un §P6x/P7x « fonction de
  décision PURE env→Plan comme seam testable + fail-loud coupling + politique
  d'URL partagée `pub(crate)` ».

**À laisser tel quel** :

- D6 refute-candidates (7) — confirmations d'invariants, aucune action.
- D5-1 — **REFUTED**, ne pas agir (frontière §6.12 fantôme ; discipline §6.9
  named-const déjà satisfaite + portée carry K).

**Checklist body (sections canoniques + delta cumulé)** : (1) 6 adaptations
préflight honorées ; (2) 7 contraintes §4 appliquées ; (3) +7 Rust net (6
Tier A + 1 Tier B), Win 2040→2047, Docker attendu +7 ; (4) PLAN-ADAPT
in-phase fake PUT+GET vérifié SUPÉRIEUR (test-utils pkarr PUT-only) ; (5)
Option B `PkarrResolver` HTTP câblée code+runbook ; (6) acceptance zéro-n0 =
artefact T2 (palier binaires PASS-par-sonde + palier live RIG-ABSENT tracé,
fenêtre C8 01/08) ; (7) invariants : presets::N0 défaut sémantiquement
préservé, 0 bump wire (23 `DOMAIN_*_V1`, ALPN verbatim, paquet pkarr
FORMAT-invariant), iroh strictement seul (dev-dep `test-utils` sous
`[dev-dependencies]` resolver-v2, 0 fuite release, `force_staging` env-only),
verrous S74/S75 5/5, duress non re-gaté ; (8) Cargo.lock purement additif (7
crates de TEST, 0 version bougée) ; (9) carries G(threat relocation + SPOF +
silent-loss + hickory-0119 + THREAT_MODEL Phase G) / K(pattern seam +
age_witness + convention env) / I-J(RTT — jamais « shard SAUVÉ ») ; (10)
docs-contrat : **aucune frontière machine §6.12 créée** (D5-1 REFUTED — env
= INPUT lu par le code, pas contrat servi ; runbook `docs/` hors gate) →
aucune étiquette test-acteur due.

## Residual Risk

- **D2-2 latent** : le site prod `node.rs:349` (`Minimal`) + l'appel
  `apply_zero_n0_discovery` ne sont pas couverts E2E hermétiquement ; une
  future régression `Minimal`→`N0` ou un `apply_*` droppé re-câblerait n0
  silencieusement. Correct AUJOURD'HUI, gap structurel non-fermable
  hermétiquement (fake http-loopback rejeté par la politique https-only). Le
  body ne doit pas sur-vendre la couverture E2E.
- **Palier T2 binaires = sonde live non re-vérifiable hors-ligne** :
  `selfhost_binaries_available=PASS` repose sur une sonde crates.io/tarball
  (existence `iroh-dns-server 1.0.1` + README GET+PUT + `[[bin]]`) ;
  cohérent avec préflight §6 qui la flaggait « NON indépendamment
  confirmée ». Le split existence(probe)/convergence(RIG-ABSENT) est honnête ;
  une sonde fausse surfacerait au provisioning, bien avant l'EOL 30/09.
- **SPOF opérateur zéro-n0** : le mode CONCENTRE relais+pkarr+ancre sur
  l'infra opérateur ; l'acceptance T2 prouve (une fois jouée) la résilience
  à l'EOL n0, PAS à la mort du VPS opérateur — COMPLÉMENTAIRE (non superset)
  de survives-VPS-death S75. Routé carry G (`≥2 relais pkarr distincts
  non-n0`).
- **`tls_pinning` T20 non-câblé** : le hook amont existe (1.0.1) mais la
  posture runtime reste WebPKI-only ; `apply_zero_n0_discovery` prod ne pose
  PAS `ca_tls_config` (le `insecure_skip_verify` reste `#[cfg(test)]`) →
  trust intact, câblage `PinValidator` = carry G, NON fermé par ce diff.
- **Env session (05/07)** : contention cargo (ne PAS lancer nextest workspace
  Win pendant Docker/Codex) ; flaky Docker 2-nœuds sous run parallèle =
  classe Phase C requalifiable solo. Non pertinent pour les Tier A purs ;
  pertinent pour l'acceptance E2 LIVE.

---

**PASS-PENDING** : phase substantivement conforme au contrat préflight
PLAN-ADAPT, minimale et load-bearing par conception, PLAN-ADAPT in-phase
vérifié SUPÉRIEUR à la source par 6 dimensions, invariants TOUS tenus. 0 P0,
0 P1. 2 P2 CONFIRMÉS (D2-1 fix-in-phase mission-ciblé + D2-2 document-in-body
structurel) + P3 (fix cheap / body / carry). 1 P2 REFUTED (D5-1 frontière
§6.12 fantôme, n'apparaît pas comme fait). Une fois les fixes 1-4 appliqués
et D2-2 + P3-body documentés, le gate Codex peut promouvoir en
`## Verdict: PASS`.


---

## Codex reconciliation (2026-07-05, post-review)

Verification croisee Codex GPT 5.5 round 1 (`codex exec -o`, artefact
brut `sprint81_phase_e2_codex_review.md`) : **6/8 CONFIRME, 0 GAP,
2 PARTIELS** — tous deux reconcilies, 0 fix code Rust :

1. **PARTIEL Livrable 3 (node.rs chemin defaut)** — Codex releve
   (a) `relay_mode` desormais applique avant `secret_key` et (b) le
   champ `zero_n0` ajoute au log final « iroh endpoint ready ».
   Disposition : **faux positif semantique** sur (a) — le `Builder`
   iroh est un accumulateur pur (chaque `.method()` ne fait que
   setter un champ, l'application reelle se produit au `bind()`,
   cf. vendored `endpoint.rs` ; commutativite verifiee par la review
   dimension D1) ; (b) est un delta d'observabilite VOULU (posture
   discovery diagnosticable au boot), local-only, documente au
   commit body. Le critere du prompt (« semantiquement inchange »)
   est tenu ; sa reformulation « strictement identique au diff »
   etait plus forte que l'intention.
2. **PARTIEL Livrable 7 (units systemd en prose)** — fonde.
   **Fixe root-cause in-phase** : 2 unit files complets installables
   livres (`deploy/iroh-relay.service` +
   `deploy/iroh-dns-server.service`, pattern hardening
   `nexus-shell-daemon.service` S75 + delta unique
   `CAP_NET_BIND_SERVICE`), runbook §4.3 re-pointe dessus.
   Fichiers non compiles (docs/deploy) → aucune suite Rust a
   rejouer ; fmt/clippy/nextest/doctests/release et le pipeline
   Docker sbfb-ci restent representatifs (2047 Win 0-skip /
   2051 Docker complet).

Nuance Codex sur le lock (« desambiguisation textuelle sha1 »)
confirmee : additif pur, 0 version existante bougee. Critere d'arret
de boucle atteint (0 GAP, PARTIELs reconcilies/documentes) —
review promue **PASS**.
