# Sprint 75 Phase E — Review (headless VPS anchor)

Date : 2026-06-10. HEAD de base : `41b13e3` (diff uncommitted Phase E).
Process : Workflow multi-agent 5 dimensions adversariales (correctness,
security, wire, tests, patterns) → skeptics refute-by-default sur tout P0/P1
(2 skeptics par finding, majorité requise) → synthèse main thread. 9 agents,
~1.06M tokens.

## Verdict: PASS

0 P0. 2 P1 émis et **confirmés 2/2 par les skeptics**, corrigés in-phase.
~22 findings P2/P3/NIT (dédupliqués entre dimensions : le clamp uppercase et
la rustdoc fusionnée sont flaggés par 3-4 dimensions chacun) : 17 corrigés
in-phase (norme anti-faux-vert), 5 déférés scopés avec owner explicite.

## Dimensions

| Dimension | P0/P1 | Findings mineurs | Notes |
|---|---|---|---|
| correctness | 1 (systemd env) | 5 | oneshot/re-pin/refactor/résolution/anti-double-émission tous vérifiés sains |
| security | 1 (duress driver) | 4 | verrous 1-5 tenus structurellement ; caps dial intacts ; mint gate détention |
| wire | 0 | 6 | 0 bump, 0 DOMAIN, /browse intact, route publish byte-identique post-refactor |
| tests | 0 | 7 | les 8 tests neufs jugés PROBANTS (non tautologiques) ; renforts demandés |
| patterns | 0 | 9 | locks lexicaux OK ; doc-honnêteté (claims vs code) |

## P1 corrigés in-phase (2, confirmés skeptics 2/2)

1. **[P1 correctness+patterns] Unit systemd sans `SBFB_HOME`/`HOME`** — sur
   systemd stock (User= sans `$HOME`), `resolve_token_from_disk` échoue au
   boot → crash-loop DOA ; et sans home résoluble, `read_directory_revision`
   retourne 0 pour toujours → la re-annonce producteur (la feature phare) est
   inerte sur sa cible, et chaque publish re-signe revision=1 que le dedup
   strict des abonnés rejette comme rollback. FIX : `Environment=SBFB_HOME=
   /var/lib/nexus-grid/.sbfb` + `StateDirectoryMode=0700` + instructions
   d'install corrigées (`install -d` avant le cp) + commentaire des DEUX
   arbres d'état écrits.
2. **[P1 security] `run_boot_seed_driver` non gaté duress** — seule surface
   d'émission du diff sans le gate : en mode Duress, le driver aurait fetché,
   muté `keep_online` et émis des `SeedAnnounced` signés du keypair DECOY sur
   la config RÉELLE de l'opérateur (le launcher duress ne change que
   l'identité, pas le data root) — corrélation decoy↔réel, déni plausible
   cassé. FIX : short-circuit duress en tête du driver (miroir des frères
   gatés) + test `boot_seed_driver_noop_in_duress` (0 pin, 0 tag).

## Findings corrigés in-phase (15 P2/P3/NIT)

3. **[P2 ×4 dims] Clamp `[seed]` acceptait l'hex MAJUSCULE** jamais
   résoluble en aval (lookups lowercase exacts) — même classe que le fix
   SeedRegistry Phase D. FIX : normalisation `make_ascii_lowercase()` avant
   le retain dans `clamped()` + cas mixte dans
   `invalid_seed_project_ids_dropped_at_load` (survit NORMALISÉE).
4. **[P2 patterns] `RestrictAddressFamilies` omettait `AF_NETLINK`** requis
   par iroh/netwatch + getifaddrs Linux (ancre relay-only ou cassée). FIX :
   ajouté.
5. **[P2 security] Hardening systemd incomplet** — FIX : `SystemCallFilter=
   @system-service` + `SystemCallErrorNumber` + `ProtectProc=invisible` +
   `ProcSubset=pid` + `PrivateDevices` + `UMask=0077` +
   `StateDirectoryMode=0700`.
6. **[P2 patterns+wire] Doc de la route `/seed/request` promettait une
   exemption d'invite « self-designation »** que le handler S74 ne possède
   pas (rejet `no-invite` inconditionnel). FIX : doc corrigée aux 2 endroits
   (invite TOUJOURS requise ; `serde(default)` = tolérance runtime
   documentée).
7. **[P2 tests] Fuite d'isolation : le driver spawné par les tests runtime
   atteignait le VRAI `~/.sbfb`** via le fallback `auth::sbfb_home`. FIX
   root-cause : `DaemonStartOptions.sbfb_home` (None = résolution UNE fois au
   boot ; les tests runtime injectent leur tempdir), porté dans
   `DaemonHttpState.sbfb_home`.
8. **[P2 tests] Prédicat anti-double-émission sans couverture** — FIX :
   extrait en fn pure `seed_already_announced` + test 5 cas.
9. **[P2 tests] Route `/seed/request` : duress + rejets locaux non testés** —
   FIX : `seed_request_peer_noop_in_duress` + `seed_request_peer_rejects_
   local_errors` (400 malformed / 400 self / 404 unknown / 409 blob non
   détenu).
10. **[P2 correctness] Fenêtre morte premier-boot du driver one-shot** — FIX
    (option doc) : documentée dans la rustdoc du driver + remède opérateur
    dans `config.toml.example` (`POST /api/daemon/seed` à chaud ou restart) ;
    le re-drive-on-ingest est un carry S76 (cf. déférés).
11. **[P3 ×3 dims] Rustdoc de `next_directory_revision` fusionnée sur
    `read_directory_revision`** + claim « the route is the only writer »
    périmée + claim `sbfb_home: None` prod périmée post-fix 7. FIX : blocs
    re-scindés et réécrits (2 writers in-process via le cœur partagé).
12. **[P3 security+wire+patterns] Garde anti-self-designation contournable
    par la forme base32** — FIX : comparaison des identités PARSÉES
    (`peer_id.to_string() == state.node_id`) + message d'erreur ajusté.
13. **[P3 wire] `SEED_REQUEST_TIMEOUT_SECS` 60s sous-dimensionné** vs le
    budget 120s du même transfert ailleurs ; 504 pouvait masquer un seed
    réussi avec invite single-use consommée. FIX : aligné sur
    `DIRECTORY_PULL_TIMEOUT_SECS` + note « 504 ≠ échec, vérifier via
    seed-count ».
14. **[P3 patterns ×2] Overclaim « first-ever discovery »** (le re-emit est
    boot-only : un abonné qui rejoint après reste en attente d'overlap) —
    FIX : qualifié aux 2 endroits + résiduel consigné. **Doc du driver
    suggérant un failover cross-tier inexistant** — FIX : « first applicable
    source ONLY, no cross-tier failover (PULL-3, S76) ».
15. **[P3 security] Auto-pin lexicographic-first sans vérif provenance** —
    FIX doc : trust boundary explicitée au point de résolution (l'ancre
    abonnée EST le gate ; BLAKE3 seule vérif à l'auto-seed) + routé au carry
    S76 avec le sampling anti-Sybil.
16. **[P3 patterns] Instructions d'install de l'unit** (`useradd` sans
    répertoire → `cp` échoue) + header `/opt` contradictoire de
    `config.toml.example`. FIX : `install -d` + header aligné
    `$NEXUS_GRID_ROOT`.
17. **[P3 tests ×2 + NIT ×3]** Dedup `seen` figé (doublon config → pinned==1
    dans `boot_repins`) ; priorité de résolution figée par
    `boot_driver_prefers_keep_online_hash_over_directory` (row M18 jamais
    écrasé par un annuaire divergent, 0 fetch) ; bras mismatch multi commenté
    « defensively unreachable » ; ordre du task boot inversé (re-annonce
    locale AVANT l'acquisition réseau) ; « z-base-32 » → « lowercase-hex ».

## Findings déférés scopés (5, owner explicite)

- **[P2] Re-drive-on-ingest du driver one-shot** (option a du finding 10 :
  re-résoudre les pids configurés restants quand un `NodeDirectoryEntry`
  s'ingère) — changement comportemental, **carry audit S76** ; le remède
  opérateur est documenté in-phase.
- **[P2] Duress gates des frères PRÉEXISTANTS** (`seed_voluntary`,
  `reannounce_seeds_at_boot` — gap S74, hors diff Phase E) — **carry audit
  S76** (lot dette duress).
- **[P3] Exemption same-key dans le handler seed** (si réellement voulue) =
  changement de surface wire S74 — **hors phase**, la correction doc est le
  fix in-phase ; doc héritée `seed.rs:111-116` à réaligner au même moment.
- **[P3] Failover cross-tier du driver** (ticket mort → pas de bascule
  multi-provider ; le ticket d'une entry restaurée d'outbox pointe NOTRE
  adresse) — même classe que **PULL-3, carry audit S76** (doc inline livrée).
- **[NIT] Validation live de l'unit systemd sur le VPS réel**
  (`systemd-analyze security`, bind QUIC sous seccomp) — **Phase G**
  (acceptance cross-machine survives-VPS-death).

## Fail-fast

- Itération : run filtré 8 tests Phase E (pré-fixes) 8/8 PASS ; crates ciblés
  639/639.
- Complet pré-fixes : fmt 0 (après reformat) ; clippy `--all-targets` 0
  (3 `needless_borrow` corrigés — `&state` double emprunt post-refactor) ;
  doctests 0 ; release OK ; **web COMPLET vert** : lint 0 err, tsc 0, Vitest
  334/334, coverage 86.94/78.73/85.82/88.25 ≥ seuils (1er run coverage = 1
  fail flake env. `vitest_env_variance` child-process, re-run propre
  334/334 GREEN ; 0 fichier web dans le diff), build OK, size 6/6, scan FR
  clean.
- Un run nextest workspace intermédiaire a été invalidé par une course de
  compilation (run filtré concurrent recompilant les binaires sous le
  nextest en cours — TIMEOUT artefactuels) ; non significatif.
- Complet post-fixes review : fmt 0 ; clippy `--all-targets` 0 ; **nextest
  workspace 1748/1748, 0 skip** (1735 → +13 = 8 tests initiaux + 5 tests
  review) ; doctests 0 ; release OK. NOTE ENV : en cours de gating, la
  pile réseau hôte s'est dégradée (classe S74 : `create_node` hang 90s,
  reproduit sur HEAD sans le diff via stash/pop — preuve env, pas code) ;
  rétablie par reboot machine, suite complète re-jouée VERTE post-reboot.
- Post-fix Codex round 1 (boot_driver_handle abort+join) : re-run complet
  consigné au commit body.

## Codex reconciliation

Gate Codex GPT-5.5 (`codex exec`, sortie brute
`sprint75_phase_e_codex_review.md`) :
- **Round 1 : 28 CONFIRMED + 1 GAP → OVERALL: FAIL.** GAP confirmé et
  corrigé in-phase : le task boot était détaché (`tokio::spawn` sans
  handle) — un shutdown pendant un pull (jusqu'à 120s/app) laissait du
  travail réseau vivant et faisait échouer la reclamation de l'`Arc<Node>`.
  FIX : `DaemonRuntime.boot_driver_handle` retenu + `abort()`+join dans
  `shutdown()` AVANT la reclamation du node (abort sûr : aucun lock sync
  tenu à travers un await dans le driver, écritures DB atomiques).
  Boucle complète re-jouée après le fix : fmt 0 / clippy 0 / nextest
  **1748/1748 0-skip** / doctests 0 / release OK.
- **Round 2 : 19 CONFIRMED, 0 GAP → OVERALL: PASS.** Tous les livrables
  vérifiés evidence file:line, dont : `[seed]` default-empty + lowercase
  clamp, plumbing main/runtime + sbfb_home once-at-boot, séquencement
  boot (replay-wait → re-announce d'abord → driver), **handle retenu +
  abort+join avant la reclamation du node (le GAP round 1)**, refactor
  publish_directory observationnellement identique, revision read/write
  split + gate state-driven, duress gates (driver + route), acquisition
  first-applicable + multi-provider bare-hash, prédicat anti-double-
  émission, route requester (parse/self-guard/mint-gate/sign/timeout/
  nonce-echo), `request_seed` dead_code retiré, unit systemd (2 roots
  épinglés + AF_NETLINK + hardening), 13 tests neufs.

Verdict final promu PASS post-Codex round 2. 0 P0, 0 P1 résiduels ;
2 P1 review + 1 GAP Codex corrigés in-phase ; 17 P2/P3/NIT corrigés
in-phase, 5 déférés scopés (cf. §Findings déférés).
