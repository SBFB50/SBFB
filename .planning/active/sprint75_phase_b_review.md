# Sprint 75 Phase B — Review (NodeDirectoryEntry + generic ingest gate + authoring route)

Date: 2026-06-09
Verdict: **PASS** (après corrections review + reconciliation Codex)

## Méthode

Review adversariale multi-agent (workflow `wtx2v2jm7`, 9 agents) sur le diff
working-tree des 7 fichiers Phase B : 5 dimensions (correctness, security,
wire-scope/anti-recentralisation, tests, patterns) → skeptics adversariaux sur
chaque finding P0/P1 (consigne : *réfuter* le finding, défaut `real=false`) →
synthèse. Puis fail-fast dual-bloc Windows (Rust + Web) et gate Codex.

## Compte des findings

| Sévérité | Brut | Après skeptics | Statut |
|---|---|---|---|
| P0 | 0 | 0 | — |
| P1 | 4 | **3 distincts confirmés** (1 → NIT) | **tous corrigés** |
| P2 | 4 | 4 | 3 corrigés, 1 documenté/accepté |
| NIT | 11 | 11 | cheap-value corrigés, reste accepté |

Le 4ᵉ P1 (test cross-domain) a été rétrogradé **NIT** par le skeptic (faux
sévérité : comportement correct, sur-assertion de nommage) — corrigé quand même
(norme projet anti-faux-vert).

## P1 confirmés — corrections

### P1-1 — Persistance revision morte en production (SECURITY + WIRE, même cause)
**Constat (vérifié skeptic).** `next_directory_revision` lisait UNIQUEMENT
`state.sbfb_home`, qui vaut `None` dans le `DaemonHttpState` de production
(`runtime.rs:910`). Sans fallback, la fonction renvoyait `1` à chaque boot et
n'écrivait jamais `directory_revision.json`. Conséquence : le contrôle
anti-rollback (le champ `revision`, son fichier, la garde monotone) **inerte en
prod** ; pire, une fois l'ingest Phase C câblé, chaque re-publish à `revision=1`
s'auto-rejette chez les pairs (`RevisionRollback`), rendant le catalogue
write-once. Les 3 tests monotone passaient via `mk_state_with_sbfb_home`
(test-only), masquant le trou.
**Fix.** `next_directory_revision` résout désormais le home comme tout autre
route persistante (consent.rs / files.rs) :
`state.sbfb_home.clone().or_else(nexus_shell_daemon_core::auth::sbfb_home)`
(`$SBFB_HOME` / `~/.sbfb`). Doc-comment corrigé (l'ancien « (unit tests) » était
factuellement faux). **Tests** : route-tests isolés sous un home tempdir (sinon
le fallback polluerait le vrai `~/.sbfb`) ; `publish_directory_revision_survives_logical_restart`
(2 états sur le même home = restart logique) ; **`publish_directory_revision_falls_back_to_sbfb_home_env`**
(state `sbfb_home:None` + `$SBFB_HOME` = la forme de prod, garde anti-régression
du `or_else` ; isolé sous nextest process-par-test).

### P1-2 — Une app over-cap 500 la route (SECURITY)
**Constat (vérifié skeptic).** `publish_directory` copiait les champs verbatim
puis `NodeDirectoryEntry::sign` cappe au sign : une app locale avec une
description >280 o (normal) faisait `sign() → Err`, la route renvoyait 500 — le
nœud ne pouvait **plus jamais** publier son catalogue tant que cette app est
hébergée. Le chemin deploy/publish n'impose aucun cap producteur.
**Fix.** `publish_directory` tronque chaque champ à son `NODE_DIRECTORY_*_MAX` sur
une frontière UTF-8 (`truncate_on_char_boundary`) avant le push, et n'inscrit que
les apps **pullables** (skip `archive_hash` vide). Une description trop longue est
clampée, l'app apparaît quand même. **Test** :
`publish_directory_truncates_oversized_fields` (description > cap → 200 + entrée
tronquée + signature vérifie).

### P1-3 (→ NIT) — Test cross-domain n'assertait que l'inégalité d'octets (TESTS)
**Constat.** `node_directory_cross_domain_replay_rejected` n'assertait que
`assert_ne!` sur les octets (tautologie), sans jamais forger une signature
sous le mauvais domaine ni appeler `verify_signature`. Skeptic : comportement
correct, mais nom sur-promet (norme anti-faux-vert S75 Phase A).
**Fix.** Renommé `node_directory_cross_domain_bytes_differ` (honnête, comme
curator/seed) + **nouveau** `node_directory_cross_domain_signature_rejected` :
forge une signature sous `DOMAIN_CURATOR_LIST_V1`, la staple dans un
`NodeDirectoryEntry`, assert `verify_signature().is_err()` — la seule assertion
qui exerce le domain tag *dans* `verify_signature`.

## P2 — corrections

- **P2 announce log-spam jusqu'à Phase C** — la dispatch gossip reconnaît
  désormais un `NodeDirectoryAnnouncement` (`is_node_directory_announcement`,
  discriminateur partial-parse) et le **drop à `debug!`** (au lieu du `warn!`
  du bras curator). Test `is_node_directory_announcement_discriminates`. Doc
  `NodeDirectoryAnnouncement` corrigée.
- **P2 commentaire parité sur-attribué** — `generic_ingest_helper_parity` garde
  la **type-symétrie** du gate partagé ; la préservation de comportement du bras
  curator est gardée par les tests réseau `two_nodes_reject_*`. Commentaires au
  site du refactor + au test + sur `SignedListIngestError` corrigés.
- **P2 announce live-only (pas d'outbox-persist)** — commentaire ajouté au site
  du broadcast : la durabilité (outbox persist + branche directory dans
  `remint_and_wrap_for_replay`) et l'ingest sont des livrables **Phase C**.
- **P2 verrou-4 non asserté au niveau route (clés indépendantes en test)** —
  *accepté + documenté*. Le verrou-4 structurel (`node_id == envelope == signer`)
  EST testé via `verify_signature`. La cohérence filtre-clé (`node.node_id()`
  z-base-32) vs clé-signataire (`pow_keypair.public_bytes()`) est un invariant de
  prod (même secret, encodage différent) documenté par commentaire au site du
  filtre ; le harness de test utilise des clés indépendantes (limite structurelle
  de `mk_state`), non corrigeable sans refactor du harness.

## NIT — traités

- **Per-field caps non testés indépendamment** → `each_per_field_cap_independently_enforced`
  (project_id/project_name/category/description, miroir `curator.rs:625`).
- **Empty `archive_hash` signé dans le catalogue** → filtré (apps non-pullables
  exclues).
- **Cohérence node_id/pow_keypair implicite** → commentaire au site du filtre.
- Reste (sign() sans check version = parité curator verbatim ; archive_hash hex
  non validé = content-addressing neutralise ; corrupt-file→1 = best-effort
  documenté ; nommage envelope pubkey ×3 = pré-existant cosmétique) : **acceptés**
  comme consistent-with-precedent ou neutralisés par le content-addressing.

## Fail-fast (Windows, pré-corrections — re-mesure post-corrections en cours)

- Rust : `fmt --check` ✓ · `clippy --workspace --all-targets -D warnings` ✓ ·
  `nextest --workspace` **1703 passed 0 fail** (1682→1703 = +21, avant les +6
  tests de correction) · doctests ✓ · release ✓.
- Web : 331 Vitest · coverage 86.91/78.63/85.82/88.23 · build · size 6/6 · scan FR.
- Re-mesure post-corrections (+6 tests) + Codex = ci-dessous / reconciliation.

## Reconciliation Codex

Gate Codex (GPT-5.5, `codex exec`, sortie brute `sprint75_phase_b_codex_review.md`
— jamais réécrite). **3 GAPs réels** (0 P0), tous corrigés ; 5 items « Verified »
(refactor curator ordre préservé, domaine disjoint + test négatif réel, attribution
sign/verify, duress avant sign, 0 bump version).

1. **GAP 1 — revision non-atomique (race concurrente).** `next_directory_revision`
   faisait read→+1→write sans lock : deux `POST /directory/publish` concurrents
   (runtime multi-thread) pouvaient lire la même valeur et signer deux directories
   à la même revision (le 2ᵉ rejeté comme rollback). **Fix** : `static
   REVISION_LOCK` (Mutex process-wide) sérialise le read-modify-write → revisions
   strictement distinctes/croissantes. **Test** :
   `publish_directory_concurrent_revisions_are_distinct` (`multi_thread`, join! →
   {1,2}).
2. **GAP 2 — `archive_hash` tronqué = content-address cassé + verify non
   validant.** Tronquer un hash produit un hash *différent* non-fetchable, et
   `verify_signature` n'imposait pas le format. **Fix** : `is_valid_archive_hash`
   (vide OU exactement 64 hex minuscules) imposé au **sign ET verify**
   (`check_catalog_field_caps`) ; la route **skip** les entrées au hash invalide
   (ne tronque plus le hash, tronque seulement les champs d'affichage). **Test** :
   `archive_hash_format_enforced` (uppercase/non-hex/mauvaise-longueur rejetés au
   sign ; forge à hash junk rejetée au verify).
3. **GAP 3 — discriminateur trop tolérant (régression dispatch curator).** serde
   ignore les champs inconnus → un hybride `{v,node,curator,ticket}` parsait comme
   directory et était droppé avant le bras curator (suppression silencieuse d'une
   annonce curator légitime). **Fix** : `is_node_directory_announcement` =
   parse-directory **ET PAS** parse-curator → l'hybride reste sur le chemin curator
   (comportement pré-Phase-B). **Test** : `is_node_directory_announcement_rejects_hybrid`.

4. **GAP rounds 2-3 — vecteur de spoof gossip (`own_entries` poisonné).** Codex
   round 2 a trouvé un NOUVEAU GAP : `own_entries` fait confiance à
   `BrowseEntry.node_id`, mais un `ProjectAnnouncement` gossipé contrôle
   `ann.node_id` (inséré verbatim par `handle_project_announcement`). Un pair
   annonce `node_id == victime` → `own_entries(my)` le sélectionne →
   `/directory/publish` le **signe dans l'annuaire de la victime** (viole verrou
   4). **Fix v1** (blob-presence) : n'annoncer que les apps dont le blob est
   détenu localement. **Codex round 3** a raffiné : le blob-presence prouve la
   *possession*, pas l'*authoring* — un attaquant peut forger `node_id == victime`
   + un hash **déjà détenu** par la victime (hash public) → métadonnées
   attacker-controlled signées (et >256 telles entrées → 500). **Fix v2 (racine)** :
   le dispatch gossip LIVE **rejette toute annonce dont `ann.node_id == notre
   node_id`** (`announcement_claims_own_node_id`) — un pair ne peut jamais
   légitimement s'annoncer comme nous (nos apps arrivent par deploy direct +
   boot-restore, jamais par gossip-receive) ; le boot-restore bypasse. + cap
   MAX_ENTRIES anti-500 + blob-presence (defense-in-depth). **Tests** :
   `live_gossip_drops_self_node_id_spoof` + `publish_directory_excludes_spoofed_unheld_blob`.
   Doc `own_entries` corrigée (2 couches). **Codex round 4 : verdict final.**

Delta tests post-Codex : +7 (rounds 1-3, 32 nouveaux au total Phase B). Fail-fast
re-vert après chaque round (3 crates + workspace).

## Verdict: PASS

0 P0 ; 3 P1 review + GAPs Codex (4 rounds) corrigés ; P2 corrigés/documentés ;
NITs cheap-value traités. Pas de DESIGN-CONFLICT.
