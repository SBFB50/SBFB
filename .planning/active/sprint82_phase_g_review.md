# Sprint 82 Phase G — Review

Date : 2026-07-14. Review Workflow ultracode 14 agents (8 dimensions +
6 vérifications adversariales sur findings ; ~1.17M tokens,
opus-4-8[1m]) sur le working tree non committé, complétée par les
corrections in-phase et leurs re-vérifications machine main-thread.
Diff final : 10 fichiers modifiés (crates/nexus-core-rs/src/{lib.rs,
schemas/mod.rs, schemas/shard.rs} ; crates/nexus-shell-daemon/src/
{http.rs, shard_session.rs} ; docs/protocol/SHARD_PROTOCOL_SPEC.md ;
docs/sharding/WIRING_SPEC.md ; docs/rust/PATTERNS.md ;
scripts/{check-frontier-contracts.sh, check-sharding-docs.sh}) +
2 snapshots neufs (shard_group_mint_request.schema.json,
shard_generate_request.schema.json) + artefact preflight.

## Verdict: PASS

Verdicts dimensions : **8/8 clean — 0 P0, 0 P1.** Post-adversarial :
1 P2 CONFIRMÉ + 7 P3 CONFIRMÉS (dont 3 = le même nit §6/§6.1 vu par
3 dimensions) — **tous corrigés in-phase** (détail §Findings),
re-vérifiés machine (snapshot régénéré, fmt clean, nextest schemas
15/15, gates frontier+sharding exit 0). Codex GPT-5.6 Sol round 1 :
9 CONFIRMÉ / 1 PARTIEL / 0 GAP — les 2 écarts du PARTIEL (éditions
texte PATTERNS.md pures) corrigés et re-prouvés machine, cf.
§Codex reconciliation. Promu PASS.

## Fond vérifié — ce qui est correct

Chaque preuve re-jouée indépendamment par les reviewers (pas de
confiance aveugle envers l'orchestrateur) :

- **Move a2 fidèle** (dim 1) : champs/attributs serde identiques aux
  ex-définitions http.rs (comparé champ par champ côté suppression vs
  ajout), handlers consomment sans changement de comportement,
  `#[serde(default)]` préservés (revision, session_id, max_tokens),
  aucun `#[schemars(...)]` n'altère serde, snapshots byte-exacts aux
  structs, `schema_snapshots()` +2 sans doublon, re-exports ordonnés.
- **Contrat inverse des requests prouvé load-bearing** (dim 3) : la
  négation `!mint.contains("revision")` échouerait réellement si on
  annotait `#[schemars(required)]` (démontré par le comportement du
  même attribut sur les réponses) ; `required=[group_id,members]` /
  `[prompt]` exacts dans les snapshots ; les 2 tests génériques
  (`schema_parses_as_valid_json_object`, drift-test) itèrent
  `schema_snapshots()` donc couvrent les 2 nouveaux ; l'absence de
  whitelist SI-3/SI-4 sur les requests est JUSTIFIÉE (input sans
  surface de fuite, ensemble des champs verrouillé par le snapshot).
- **Suites re-exécutées par le reviewer** (dim 2) : 4 gates docs exit 0
  (frontier avec « DOMAIN census [25 frozen] »), nextest schemas 15/15,
  cargo check daemon exit 0, census reproduit = 25 à la main, snapshots
  JSON valides pretty-printed newline finale, 0 fn #[test] ajoutée →
  count invariant structurellement.
- **Scope exact** (dim 4) : les 4 livrables plan couverts ; le backfill
  §3 et le tripwire census sont des adaptations DOCUMENTÉES au
  preflight PLAN-ADAPT ; S80-G-1 = consignation fermante (« 3rd and
  FINAL mention »), compteur §6.2.1 fermé ; frontière NEUVE annoncée
  pour indexation Phase T.
- **Research grounding 7/7** (dim 5) : les 6 corrections PLAN-ADAPT et
  les 7 points d'exécution du preflight correspondent EXACTEMENT au
  code livré ; grep committé §P70 re-exécuté = 25 ; tripwire FAIL à 26
  / clean à 25 ; rouge-avant-vert prouvé 2 fois.
- **Sécurité** (dim 6) : JsonSchema inerte à la deserialisation
  (diff attribut par attribut) ; PROMISE_RE 13 branches = 0 match sur
  TOUT le texte ajouté (y compris descriptions des snapshots) ;
  invariant PATH-authoritative INTACT (http.rs rejette body_id ≠ path
  → 400, handler non touché) ; prose mount sans contradiction
  THREAT_MODEL §16 ; 0 bump wire (aucun canonical/signed/Cargo touché).
- **Docs cellule par cellule** (dim 7) : les 3 tables Request-body §6.1
  exactes contre les structs réelles (types, required, defaults,
  default-1 revision confirmé http.rs, 400 PATH confirmé) ; §3 = 12
  rows = 12 snapshots shard ; cohérence WIRING/PATTERNS.
- **Patterns** (dim 8) : mimétisme fidèle de la voisine
  ShardSessionResultView (ordre /// puis // provenance puis derive),
  provenance vers le passé immuable uniquement, BusyBox-safety de
  chaque ligne shell ajoutée, `## Verdict: PLAN-ADAPT` greppable au
  preflight.

## Findings (1 P2 + 7 P3, tous CONFIRMÉS adversarialement, tous corrigés in-phase)

1. **P2 (dim 7)** — Provenance « S81 K » incorrecte sur les 2 rows
   backfill §3 : `ShardSessionResultView`/`Response` datent de **S81
   Phase I** (`bb6c4f9`, git log -S + struct shard.rs:119/:130) ; le
   preflight portait l'erreur (scan S3), la phase l'avait recopiée.
   **Corrigé** : 2 rows « S81 I » + note datée dans le preflight.
2. **P3 (dims 1+5+7, même nit)** — Pointeurs « §6 » vs « §6.1 »
   incohérents (code/snapshot vs WIRING/PATTERNS). **Corrigé** :
   5 renvois code (shard.rs ×4, shard_session.rs) + 2 renvois internes
   SPEC harmonisés « §6.1 » ; snapshot generate RÉGÉNÉRÉ
   (description-only) et re-drift-gaté vert.
3. **P3 (dim 3)** — Doc-comment d'en-tête du test
   `schemas_publish_required_fields` sous-décrivait la couverture
   (payloads signés seuls). **Corrigé** : mentionne les envelopes
   réponses + le contrat inverse des request bodies.
4. **P3 (dim 6)** — §P70 « never drift » sur-affirmait la garantie du
   tripwire (add+remove net-zero / rename non couverts). **Corrigé** :
   borne honnête explicite (« cannot GROW silently » + classe net-zero
   routée à la review adversariale per-sprint).
5. **P3 (dim 8)** — Message du tripwire n'orientait pas vers la prose
   25/22 in-file (header (2), comment (2b)). **Corrigé** : message
   énumère les 3 surfaces de refresh.
6. **P3 (dim 8)** — Pipe census sans guard `|| true` (latent multi-batch
   sous set -euo pipefail, incohérent avec la sœur FRONTIER). **Corrigé** :
   `{ find … || true; }` — diagnostic de drift plutôt qu'abort set -e.
7. **P3 (dim 7, PRÉ-EXISTANT S82-B, opportuniste consigné)** — SPEC §6
   route result disait « 9 fields » pour 14 réels (les 5 métriques
   benchmark S82 B jamais reportées dans la row). **Corrigé
   in-passing** (même fichier fortement édité par la phase, désormais
   cross-référencé par §3) : « 14 fields » + les 5 métriques listées.

Re-vérifications post-fixes : `UPDATE_SNAPSHOTS=1` → snapshot generate
descriptions-only ; `cargo fmt --all --check` clean ; nextest schemas
15/15 ; check-frontier-contracts + check-sharding-docs exit 0 ;
nextest workspace complet relancé (post-fixes, attendu 2099 —
fixes = doc-comments/docs/shell uniquement).

## Suites §7.4 (jouées pré-review, toutes vertes)

- Rust Windows : fmt 0 ; clippy --workspace --all-targets -D warnings
  0 ; nextest **2099/2099 0-skip** (count invariant vs baseline S82) ;
  doctests OK ; build release daemon OK.
- Docker canonique `sbfb-ci` : fmt 0 ; nextest **2103/2103 0-skip**
  (= Win + 4 `#[cfg(unix)]`, baseline respectée).
- Web : lint + tsc + unit + coverage 87.27/79.01/86.02/88.59 ≥
  85/78/85/85 + build + size 6/6 + scan-en-strings clean (0 fichier
  web/ touché ; bloc joué par règle full-failfast).
- Gates docs : spdx + factory + sharding + frontier exit 0 en GNU ET
  frontier + sharding en BusyBox `bash:5` Docker (surface Woodpecker).
- Rouge-avant-vert : (a) REQUIRED_ANCHORS +3 AVANT l'édit WIRING_SPEC
  → gate FAIL avec les 3 anchors nommées, puis vert après ; (b) census
  muté à 26 → FAIL diagnostic, réel 25 → clean.

## Codex reconciliation

Rapport brut : `sprint82_phase_g_codex_review.md` (output
`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max -o`, NON
réécrit). Round 1 : **10 livrables — 9 CONFIRMÉ / 1 PARTIEL / 0 GAP** ;
« aucun défaut fonctionnel ou de gate n'a été trouvé ». Codex a
re-exécuté indépendamment : nextest schemas 6/6, cargo check daemon,
fmt + git diff --check, les 2 gates GNU ET Docker bash:5/BusyBox,
census indépendant = 25.

Le PARTIEL (livrable 9, PATTERNS §P70) = 2 écarts factuels
documentaires, tous deux corrigés in-phase :

1. **PATTERNS « ShardSessionResultView S81 K » → S81 I** — la même
   erreur de datation que le P2 review (finding 1) survivait dans ma
   nouvelle prose §P70 (occurrence ratée par le fix SPEC). Corrigé ;
   `grep -n 'S81 K' docs/rust/PATTERNS.md` = 0 hit.
2. **PATTERNS couche ÉTIQUETTE « 8 sharding types » → 12** — compteur
   PRÉ-EXISTANT périmé (10 avant la phase), aggravé par les +2 de la
   phase. Corrigé in-passing (même fichier, même classe compteur-stale
   que le « 9 fields » SPEC, consigné).

Re-vérifications post-fixes Codex : 4 gates docs exit 0 (frontier
census [25 frozen], sharding, factory, spdx) ; 0 code touché par ces
2 fixes (PATTERNS.md seul) → suites Rust/web inchangées (nextest
workspace 2099/2099 re-couru après les fixes review, avant les fixes
Codex texte-seuls). Critère d'arrêt boucle atteint : 0 GAP, PARTIEL
soldé par éditions texte re-prouvées machine.
