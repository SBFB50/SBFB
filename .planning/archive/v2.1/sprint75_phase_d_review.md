# Sprint 75 Phase D — Review (multi-provider pull + node identity exposure)

Date : 2026-06-10. HEAD de base : `9f7de7f` (diff uncommitted Phase D).
Process : Workflow multi-agent 5 dimensions adversariales (correctness,
security, wire, tests, patterns) → skeptics refute-by-default sur tout P0/P1
(2 skeptics, majorité requise) → synthèse main thread. 5 agents, ~588k tokens.

## Verdict: PASS

0 P0, 0 P1 (aucun finding sérieux émis par les 5 dimensions — aucune
réfutation nécessaire). 22 findings P2/P3/NIT (dédupliqués : 19 distincts),
triés ci-dessous : 12 corrigés in-phase (norme anti-faux-vert), 7 déférés
scopés avec owner explicite.

## Dimensions

| Dimension | P0/P1 | Findings mineurs | Notes |
|---|---|---|---|
| correctness | 0 | 6 | logique fetch/Q5/eviction saine ; nits doc + dead-code |
| security | 0 | 6 | hex-case = le vrai finding ; verrous 1-5 tenus, BLAKE3 structurel |
| wire | 0 | 2 | /browse byte-identique confirmé ; 0 bump ; enveloppe /nodes pinnée |
| tests | 0 | 6 | renforts demandés (anti-displacement, has_tag, résolveurs, route) |
| patterns | 0 | 2 | doc-honnêteté (« no allocation churn ») + dead-code |

## Findings corrigés in-phase (12)

1. **[P2 security] Hex-case non normalisé dans SeedRegistry** — une seule clé
   Ed25519 pouvait signer sous 2^64 variantes de casse de sa propre pubkey
   (le feed accepte le mixed-case), monopoliser les 64 slots d'un bucket et
   évincer les seeders honnêtes du dial set. FIX racine : normalisation
   lowercase des 3 clés dans `record()` + lectures normalisées
   (`count_recent`, `seeders_recent`) + self-check `eq_ignore_ascii_case` +
   test `seed_registry_normalizes_hex_case`.
2. **[P2 tests] Anti-displacement per-bucket non testé** (branche `_ => return`)
   + **victime stalest non pinnée** — `seed_registry_size_bounded` renforcé :
   `assert_eq!` exacts (== MAX, pas <=), victime = la plus stale (clé 9 évincée,
   clé 10 survit, dans les DEUX caps), newcomer stale droppé sur bucket plein.
3. **[P2 tests] `find_directory_app_by_hash`/`by_project` jamais exercés** —
   NEW test `directory_resolvers_match_hash_and_project` (match exact,
   empty-hash never-match, multi-dirs, miss, skip archive-less).
4. **[P2 tests] Pin skip-GC de `fetch_and_pin_multi` non asserté** —
   `seed_voluntary_directory_only_app` asserte désormais `has_tag`
   (seul test exerçant `fetch_and_pin_multi` ; miroir du test ticket).
5. **[P3 security] Cap providers = convention d'appelant** — même leçon que
   SEED-1 : `MAX_FETCH_PROVIDERS = 16` enforced DANS `fetch_hash_multi`
   (truncate, callers ordonnent best-first donc tronquer garde les
   prioritaires) ; le cap policy daemon (8) reste plus serré.
6. **[P3 tests + 2×NIT] Route /nodes jamais traversée en HTTP + nom de test
   sur-vendeur** — oneshot `GET /api/daemon/nodes` ajouté dans
   `reachable_via_seeder_status` (c) ; test projection renommé
   `nodes_response_pins_envelope_and_grouping` (déviation de nom vs plan
   §D.3 #6 documentée au commit body — le claim du plan est couvert par le
   couple projection-pin + route-level).
7. **[P3 patterns] Doc « no allocation churn » fausse** — scan d'éviction
   réécrit sur références (seule la clé victime est clonée), doc alignée.
8. **[P3 tests] Doc du test fallback sur-vendait l'ordering** — commentaire
   reformulé : prouve résilience dead-provider + intégrité ; l'ordering est
   pinné à la CONSTRUCTION (`fetch_provider_ordering`), la consommation
   in-order = comportement documenté iroh-blobs 0.100.
9. **[2×NIT] `providers.truncate` dead-code** — supprimé (le break applique le
   cap ; la primitive a son propre plafond).
10. **[NIT] Doc `last_sweep` périmée** (horloge d'annonce → receive clock) —
    corrigée.
11. **[P3 doc] Fresh-flood displacement non documenté** — doc
    `MAX_REGISTRY_BUCKETS` étendue : résiduel accepté (poste THREAT_MODEL §15
    row D), policy inverse perdrait aussi, ancre jamais crowdable, decay TTL.
12. **[P2 doc] Sybil lexicographic crowding du dial set** — résiduel
    availability-only documenté dans `directory_pull_providers` (l'ancre
    n'est pas crowdable, BLAKE3 tient) ; mitigation = random sampling.

## Findings déférés scopés (7, owner explicite)

- **[P2] Cross-tier failover** (ticket direct échoue → ne tente pas le tier
  directory pour le même hash) : amélioration availability réelle mais
  changement de comportement du tier S74 existant — **carry audit S76
  (candidat PULL-3)**. Pas une régression : R5a/R5b sont scopés directory-only.
- **[P2 résiduel] Sampling anti-Sybil du seeder tail** — **carry audit S76**
  (doc inline livrée, cf. corrigé 12).
- **[P3] blob-serve GET déclenche des dials sortants** (oracle drive-by +
  amplification N×8 sans in-flight dedup ; classe pré-existante du tier
  ticket, loopback-only) — **Phase G : THREAT_MODEL §15 rows** (directory
  pull public-route, /nodes, SEED-1/2, fresh-flood acceptance).
- **[P3] `seed_voluntary` project_id first-match** (collision multi-ancres +
  shadowing par direct entry sans archive) — **Phase F** : discriminateur
  `archive_hash` optionnel sur `SeedVoluntaryRequest` quand le CTA front câble.
- **[P3] Couplage `NodeSummary.catalog` ↔ struct wire `CatalogApp`** —
  **handoff Phase F** : Zod `.strict()` sur l'enveloppe `{nodes}`, PAS sur les
  rows catalog (ou projection HTTP dédiée) — sinon le premier ajout additif
  0-bump brique la page /nodes.
- **[NIT] THREAT_MODEL §15 non touché dans ce diff** — séquencement conforme
  (précédent S74 : §15 au wrap-up). **Phase G**.
- **[P2 partiel] E2E blob-serve R5a complet non testé** (exigerait un zip
  valide) — choix explicite : le glue R5a est couvert par
  `directory_resolvers_match_hash_and_project` (résolution) +
  `seed_voluntary_directory_only_app` (même chaîne fetch multi-provider E2E) +
  read-back-by-hash identique au tier ticket existant ; l'E2E rendu complet =
  **acceptance Phase G** (C6 cross-machine). Documenté au commit body.

## Fail-fast

- Itération : `cargo nextest run -p nexus-core-rs -p nexus-shell-daemon-core
  -p nexus-shell-daemon` → 976/976.
- Complet pré-fixes : fmt 0 ; clippy 1 erreur (`await_holding_lock` dans le
  test seed_voluntary — fix : bloc lexical, pattern prod) ; nextest workspace
  **1733/1733** ; doctests 0 ; release OK ; web COMPLET vert (Vitest 334,
  coverage 86.94/78.73/85.82/88.25, size 6/6, scan FR) — 0 fichier web touché.
- Complet post-fixes review : re-run en cours, consigné au commit body
  (delta attendu : nextest 1724 → 1735, +11).

## Codex reconciliation

Gate Codex GPT-5.5 (`codex exec`, sortie brute
`sprint75_phase_d_codex_review.md`) : **round 1 → OVERALL: PASS**, 12
CONFIRMED, 0 GAP. Tous les livrables vérifiés evidence file:line, dont :
fetch_hash_multi (empty-reject + MAX_FETCH_PROVIDERS + bare-hash download),
fallback/integrity live-tests re-exécutés par Codex, SEED-1 clamp +
normalisation + sweep receive-clock, SEED-2 double cap + stale-newcomer drop,
seeders_recent prod, ingest (feed_ts, recv_now), /nodes additif authed,
/browse byte-identique (node_id #[serde(skip)]), providers anchor-first/dedup/
self-excl/cap, seed_voluntary Multi (timeout + h==want_hash + keep_online +
SeedAnnounced), blob_serve directory-only (subscribed-advertised only +
timeout + read-back-by-hash), web diff vide + 0 drift version/domain/proof-age.
Contraste process : 1 seul round (vs 7 en Phase C) — le prompt portait les
frontières de phase D/E/F/G, le PLAN-ADAPT consommateur et le scope code-only
dès le round 1 (leçons R3/R5 de Phase C appliquées).

Verdict final promu PASS post-Codex. 0 P0, 0 P1, 12 P2/P3/NIT corrigés
in-phase, 7 déférés scopés (cf. §Findings déférés).
