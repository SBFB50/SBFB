# Sprint 75 Phase C — Review (node-directory ingest + remote-catalog durability + 3 carries)

Date: 2026-06-09
Verdict: **PASS** (0 P0, 0 P1 ; P2/NIT légitimes corrigés, reste déféré-documenté)

## Méthode

Review adversariale multi-agent (workflow `wvktsvid6`, 5 agents = 5 dimensions :
correctness, security/anti-recentralisation, wire/pre-launch, tests, patterns) sur le
diff working-tree des 12 fichiers Phase C (+1160/-94). Chaque dimension a produit des
findings structurés ; chaque finding P0/P1 passait ensuite par un skeptic adversarial
(consigne : *réfuter*, défaut `isReal=false`). Synthèse : **0 P0/P1 confirmé**. Puis
fail-fast dual-bloc (Rust workspace + Web) re-vert après corrections, et gate Codex.

## Compte des findings

| Sévérité | Brut | Confirmés (skeptics) | Statut |
|---|---|---|---|
| P0 | 0 | 0 | — |
| P1 | 0 | 0 | — |
| P2 | 5 (dont 1 doublon) | — | 3 corrigés, 2 déférés-documentés |
| NIT | ~8 | — | 3 corrigés (regression guards), reste accepté |

Aucun finding n'a atteint P0/P1 — le pipeline ingest réutilise verbatim le gate
`verify_signed_list_ingest` vérifié en Phase B (4 rounds Codex), la durabilité honore D4
(locator re-validé, jamais le contenu), et les 5 verrous tiennent.

## P2 corrigés

1. **Doc rustdoc mal attribuée** — la struct `SeedCountQuery` insérée AVANT
   `async fn seed_count` volait la doc de route (la fn restait sans doc). **Fix** : struct
   déplacée au-dessus avec sa propre doc, doc de route ré-attachée à la fn
   (`http.rs` `seed_count`).
2. **Commentaire producteur périmé** — le commentaire de `publish_directory`
   (`http.rs:1211`) annonçait encore l'ingest arm + le durable-replay comme « Phase C
   deliverables » non faits. **Fix** : réécrit pour la réalité Phase C — l'ingest arm EST
   livré, la durabilité est CONSUMER-side (re-pull locator), le re-announce PRODUCER est
   Phase E.
3. **`seed_count` narrow sur `own_hash` même quand keep_online DÉSACTIVÉ** — DBQ-1
   préserve délibérément le hash M18 à la désactivation, donc sans garde un app désactivé
   se voyait scopé à une version que le nœud ne sert plus. **Fix** : le narrow sur
   `own_hash` est gardé par `self_seeding` (`http.rs` `seed_count`).

## NIT corrigés (regression guards cheap-value)

- **Pin du wire string `"nodedirectory"`** — `browse_source_serializes_lowercase` assert
  désormais la sérialisation serde du nouveau variant (le contrat que le Zod frontend
  consomme).
- **Test web du nouveau enum** — `daemon.test.ts` : un /browse avec `source:"nodedirectory"`
  parse (mixed-version compat).
- **Tests re-pull renforcés** (anti-faux-vert) : `repull_filters_unsubscribed_locator`
  (le filtre `is_subscribed` defense-in-depth sur un locator PRÉSENT mais non abonné,
  verrou 5) + `repull_tolerates_bad_locator` (le chemin toléré sur lequel repose la
  durabilité : un locator inutilisable → 0 restauré, pas de panic, store vide). Helper
  `insert_anchor_locator_for_test` ajouté.

## P2/NIT déférés (correctement scopés — pas de fix Phase C)

- **Re-annonce PRODUCER au boot** (un publisher ne re-annonce pas son annuaire après
  reboot) → **Phase E** (driver VPS headless config-driven). Phase C livre la durabilité
  CONSOMMATEUR (re-pull) ; le re-announce timer producteur est explicitement E (plan §E).
- **Blob annuaire re-pull non pinné skip-GC** → **scope cut #3** (GC reaper déféré
  post-launch ; aucun GC ne tourne aujourd'hui, le blob persiste). Pin requis quand le GC
  sera ajouté.
- **`known_entry_count` double-compte un app présent en curator-list ET en annuaire** →
  best-effort assumé (THREAT_MODEL §15 : sur-estimation tolérée, jamais sous-estimation ;
  le content-addressing reste la vérité). Compteur « honest superset », pas exact.
- **Re-pull boot séquentiel, borné N×15s** → pilote-ferme (1-2 ancres ≤ 30s pire cas),
  timeout par-ancre déjà documenté. Parallélisation = follow-up non-bloquant.

## Fail-fast (Windows, post-corrections finales)

- Rust : `fmt --check` ✓ · `clippy --workspace --all-targets -D warnings` 0 ·
  `nextest --workspace` **1724 passed 0 fail** (1714→1724 = +10) · doctests 0 · release ✓.
- Web : tsc 0 · lint 0 err · Vitest **334** (+3) · coverage 86.94/78.73/85.82/88.25 ≥
  seuils · build · size 6/6 · scan FR clean.
- Re-vert après CHAQUE round de fix Codex (4 re-runs complets du bloc Rust, 2 du bloc web).

## Reconciliation Codex

Gate Codex (GPT-5.5, `codex exec`, sortie brute `sprint75_phase_c_codex_review.md` — jamais
réécrite ; l'artefact = le DERNIER round). **7 rounds → OVERALL: PASS** (round 7 :
19 CONFIRMED, 0 GAP).

1. **Round 1 — 2 GAPs réels** (19 CONFIRMED) : (a) `self_seeding` lu par projet et émis
   tel quel → un nœud pinnant la version Y claimait « Toi » pour une requête sur la
   version X ; fix : `self_seeding` n'est vrai pour une requête versionnée que si le pin
   EST cette version (`http.rs seed_count`). (b) Le frontend n'envoyait jamais le
   nouveau query `?archive_hash` → WIRE-2 read-side inerte (leçon S72-D) ; fix :
   `seedCount(baseUrl, pid, archiveHash?)` + `AvailabilitySheet` passe
   `entry.archive_hash` (queryKey 4-éléments). +2 tests Vitest (query présent/omis).
2. **Round 2 — 1 GAP nouveau** (les 2 du round 1 CONFIRMED ; Codex a re-exécuté la suite
   de tests lui-même) : le floor anti-rollback ne survivait pas au reboot
   (`anchors.json` ne persistait que pubkey/ticket ; le re-pull lisait le floor de la
   map RAM vide). Fix : `AnchorLocator.revision` persisté (+`#[serde(default)]`),
   locator `(ticket, revision)`, floor au re-pull = `persisted-1` ; test
   `boot_repull_restores_remote_catalogs` renforcé (assert revision persistée dans
   anchors.json + rejet rollback post-reboot). +1 test.
3. **Round 3 — GAP auto-référentiel** (20 CONFIRMED, 0 GAP code) : Codex flaggait sa
   PROPRE sortie round-2 (verdict FAIL périmé dans `.planning/`) comme artefact stale.
   Résolution process : prompt restreint au code (`crates/` + `web/`), `.planning/` et
   `.git/` explicitement hors scope.
4. **Round 4 — 3 GAPs réels** (15 CONFIRMED, scope code-only) : (a) l'ingest LIVE
   ignorait le floor persisté (un re-pull échoué ouvrait une fenêtre de rollback live) ;
   fix : floor live combinant RAM et locator persisté. (b) omit `?archive_hash` n'était
   pas version-agnostic quand keep-online actif (substitution `own_hash`) ; fix :
   omission = strictement version-agnostic (`None`), pas de substitution silencieuse.
   (c) doc `NodeDirectoryAnnouncement` périmée (« dropped at debug ») ; fix : doc
   réalignée Phase C. +1 test (`live_ingest_respects_persisted_floor_after_failed_repull`).
5. **Round 5 — 2 GAPs hors-phase** (12 CONFIRMED) : apps directory-only visibles mais
   pas rendables/seedables (blob-serve + seed volontaire opèrent sur `direct_entries`).
   Réel mais **scopé Phase D** (pull re-mint depuis `(node_id, archive_hash)`) et
   **Phase F** (action front) — le modèle F-Droid « découvrir puis télécharger » ;
   le plan séquence explicitement ainsi. Résolution : commentaire SCOPE explicite au
   site aggregator + frontière de phase C/D/F ajoutée au prompt Codex (il ne l'avait
   pas) — pas un gaming du verdict, le report est documenté dans le code.
6. **Round 6 — 2 GAPs régressions de mes propres fixes** (15 CONFIRMED) : (a) le floor
   `max(RAM, persisted)` strict rejetait un re-announce live à la MÊME revision après un
   re-pull échoué → catalogue irrécupérable jusqu'au bump publisher ; fix : RAM présent →
   dedup strict `>` (parité curator), RAM vide → floor persisté `>= P` (`P-1` au gate) =
   restauration same-revision OK, rollback toujours rejeté. (b) doc `SeedCountQuery`
   contredisait le code (claim « falls back to own pinned hash » vs code
   version-agnostic) ; fix : doc réalignée. Test étendu : same-revision RESTAURE après
   re-pull échoué + dedup strict re-armé une fois RAM repeuplée.
7. **Round 7 — OVERALL: PASS** : 19 CONFIRMED (gate partagé, subscription-gate, locator
   ≠ contenu, re-pull subscribed-only + re-validation, dispatch câblé, compte additif
   honnête, WIRE-1 producteur+consommateur, WIRE-2 dual-mode, DBQ-1 COALESCE, Zod
   additif, front version-aware, test durabilité non-tautologique), 0 GAP.

Delta tests total Phase C : Rust 1714→**1724** (+10), Vitest 331→**334** (+3).

## Verdict: PASS

0 P0 ; 0 P1 ; P2 review légitimes corrigés (doc + logique seed_count), P2 hors-scope
déférés à Phase D/E / scope cuts avec justification ; NITs cheap-value (regression
guards) traités ; 8 GAPs Codex réels corrigés sur 7 rounds (+ 1 auto-référentiel et
2 hors-phase résolus par scope process documenté). Pas de DESIGN-CONFLICT (PLAN-ADAPT
C.3 du préflight honore D4). Les 5 verrous anti-recentralisation vérifiés contre le
code par review multi-agent ET Codex cross-model.
