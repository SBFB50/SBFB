Verdict global Codex : **PARTIEL, non committable avant correction ou re-scoping explicite**.

Tests ciblés verts, mais le livrable 1 ne respecte pas strictement `SYNTHESIS §4.6` : champs ProofCard incomplets, risk factors incomplets, seuil `old_release` divergent.

**Contrat vérifié**
- Phase A = ProofCard compute + endpoint : `.planning/active/sprint68_plan.md:63-83`
- Tests attendus : `.planning/active/sprint68_plan.md:89-107`
- Scope cuts : `.planning/active/sprint68_plan.md:364-377`
- SYNTHESIS ProofCard struct : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:852-891`
- SYNTHESIS formule : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:900-916`
- SYNTHESIS risk factors : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:923-933`

**1. `crates/nexus-coordinator-rs/src/proof_card.rs`**
Statut : **PARTIEL**

Confirmé :
- Fichier nouveau présent : `crates/nexus-coordinator-rs/src/proof_card.rs:1`
- `FORMULA_VERSION = 1` : `crates/nexus-coordinator-rs/src/proof_card.rs:12`
- `ProofCardInput` présent : `crates/nexus-coordinator-rs/src/proof_card.rs:20-33`
- `ProofCard` présent : `crates/nexus-coordinator-rs/src/proof_card.rs:40-50`
- `compute_proof_card()` présent : `crates/nexus-coordinator-rs/src/proof_card.rs:121-123`
- Formule base 30 : `crates/nexus-coordinator-rs/src/proof_card.rs:165`
- Evidence layers : `crates/nexus-coordinator-rs/src/proof_card.rs:168-190`
- Risk deductions : `crates/nexus-coordinator-rs/src/proof_card.rs:193-200`
- Clamp `[0, 100]` : `crates/nexus-coordinator-rs/src/proof_card.rs:203`

Gaps bloquants dans ce livrable :
- SYNTHESIS exige `source` dans `ProofCard` : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:855-860`; le struct Rust `ProofCard` n’a pas ce champ : `crates/nexus-coordinator-rs/src/proof_card.rs:40-50`.
- SYNTHESIS exige `hash.artifact_hash` et `hash.content_hash` : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:861-866`; le Rust ne garde que `archive_hash` et `provenance_hash` : `crates/nexus-coordinator-rs/src/proof_card.rs:54-57`.
- SYNTHESIS autorise `risk.level = "critical"` : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:882-884`; le Rust n’a que `Low`, `Medium`, `High` : `crates/nexus-coordinator-rs/src/proof_card.rs:103-109`.
- SYNTHESIS liste 7 risk factors : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:927-933`; le code ne pousse que `unverified_deploy`, `no_provenance`, `stale_source`, `old_release` : `crates/nexus-coordinator-rs/src/proof_card.rs:148-160`.
- `no_curator`, `single_curator`, `no_open_source` ne sont pas matérialisés comme facteurs ; `curator_count` ne sert qu’aux bonus : `crates/nexus-coordinator-rs/src/proof_card.rs:179-184`, et `is_open_source` ne sert qu’au bonus : `crates/nexus-coordinator-rs/src/proof_card.rs:171-173`.
- SYNTHESIS dit `old_release > 90 jours` : `.planning/research/SYNTHESIS_factory_rrv_protocol.md:932`; le code utilise `180` jours : `crates/nexus-coordinator-rs/src/proof_card.rs:16`.

Impact : API JSON incomplète par rapport au data model canonique, facteurs de risque non visibles, divergence possible avec l’UI Phase D et les apps bridge. **Bloquant avant commit** si le contrat reste “tous les champs SYNTHESIS §4.6”.

**2. `crates/nexus-coordinator-rs/src/lib.rs`**
Statut : **CONFIRME**

- `pub mod proof_card;` ajouté : `crates/nexus-coordinator-rs/src/lib.rs:28`

Impact : non bloquant.

**3. `crates/nexus-shell-daemon-core/src/browse.rs`**
Statut : **CONFIRME**

- Accessor public ajouté : `crates/nexus-shell-daemon-core/src/browse.rs:553-557`
- Il lit `direct_entries` par `project_id` et clone le `BrowseEntry` : `crates/nexus-shell-daemon-core/src/browse.rs:553-556`

Impact : non bloquant.

**4. `crates/nexus-shell-daemon/src/http.rs` endpoint**
Statut : **CONFIRME**, avec dépendance au gap du livrable 1

- Route ajoutée dans `authed_routes` : `crates/nexus-shell-daemon/src/http.rs:262-358`
- Middleware `auth_required` appliqué à `authed_routes` : `crates/nexus-shell-daemon/src/http.rs:428`
- Handler `get_proof_card()` : `crates/nexus-shell-daemon/src/http.rs:1998-2113`
- Browse direct entry : `crates/nexus-shell-daemon/src/http.rs:2002-2004`
- Curator snapshot + déduplication curators : `crates/nexus-shell-daemon/src/http.rs:2005-2016`
- Metadata browse/curator + 404 si projet absent : `crates/nexus-shell-daemon/src/http.rs:2018-2044`
- Provenance DB query : `crates/nexus-shell-daemon/src/http.rs:2047-2069`
- Vérification provenance : `crates/nexus-shell-daemon/src/http.rs:2072-2091`
- Input ProofCard assemblé : `crates/nexus-shell-daemon/src/http.rs:2096-2109`
- Retour 200 JSON ProofCard : `crates/nexus-shell-daemon/src/http.rs:2111-2112`

Impact : handler fonctionnel. Il hérite toutefois du modèle ProofCard incomplet du livrable 1.

**5. `crates/sbfb-manifest/src/lib.rs` allowlist**
Statut : **CONFIRME**

- `"proof_card_get"` ajouté : `crates/sbfb-manifest/src/lib.rs:52-63`
- Validation utilise bien `BRIDGE_METHOD_ALLOWLIST.contains(...)` : `crates/sbfb-manifest/src/lib.rs:85-91`
- Accessor allowlist : `crates/sbfb-manifest/src/lib.rs:99-100`

Impact : non bloquant.

**6. `web/src/bridge/protocol.ts`**
Statut : **CONFIRME** pour l’énumération demandée

- `BridgeMethodSchema` contient `"proof_card_get"` : `web/src/bridge/protocol.ts:20-44`

Note : le plan brut mentionne un “schema Zod ProofCard” à `.planning/active/sprint68_plan.md:81`; ce fichier n’ajoute pas de schéma de data model ProofCard, seulement la méthode bridge. Sous la formulation utilisateur du livrable 6, c’est confirmé.

**7. `web/src/bridge/useBridge.ts`**
Statut : **CONFIRME**

- Case `proof_card_get` ajouté : `web/src/bridge/useBridge.ts:373-383`
- Validation `payload.project_id` : `web/src/bridge/useBridge.ts:374-375`
- Dispatch vers `/api/daemon/proof-card/{project_id}` avec `encodeURIComponent` : `web/src/bridge/useBridge.ts:376-378`
- 404 converti en `{ card: null }` : `web/src/bridge/useBridge.ts:380`
- Erreur non-OK : `web/src/bridge/useBridge.ts:381`

Impact : non bloquant.

**8. `sbfb-bridge.js` méthode `getProofCard` et copies**
Statut : **CONFIRME**

- `web/public/sbfb-bridge.js` : `getProofCard(projectId)` appelle `proof_card_get` : `web/public/sbfb-bridge.js:365-372`
- `examples/sbfb-explorer/sbfb-bridge.js` : même méthode : `examples/sbfb-explorer/sbfb-bridge.js:365-372`
- `examples/sbfb-ideas/sbfb-bridge.js` : même méthode : `examples/sbfb-ideas/sbfb-bridge.js:365-372`
- `git diff --no-index --exit-code web/public/sbfb-bridge.js examples/sbfb-explorer/sbfb-bridge.js` : aucune différence.
- `git diff --no-index --exit-code web/public/sbfb-bridge.js examples/sbfb-ideas/sbfb-bridge.js` : aucune différence.

Impact : non bloquant.

**9. Tests**
Statut : **CONFIRME**

Tests unitaires ProofCard présents :
- `test_proof_card_full_evidence`, score 100 : `crates/nexus-coordinator-rs/src/proof_card.rs:300-307`
- `test_proof_card_minimal`, score 30 : `crates/nexus-coordinator-rs/src/proof_card.rs:309-314`
- `test_proof_card_provenance_boost`, score 50 + SLSA 1 : `crates/nexus-coordinator-rs/src/proof_card.rs:316-325`
- `test_proof_card_risk_no_provenance`, score 15 : `crates/nexus-coordinator-rs/src/proof_card.rs:327-335`
- `test_proof_card_formula_version`, version 1 : `crates/nexus-coordinator-rs/src/proof_card.rs:337-342`
- `test_proof_card_clamp_bounds`, bas clamp 0 : `crates/nexus-coordinator-rs/src/proof_card.rs:344-359`
- `test_proof_card_freshness_states` : `crates/nexus-coordinator-rs/src/proof_card.rs:361-390`
- `test_proof_card_unverified_deploy`, score 20 : `crates/nexus-coordinator-rs/src/proof_card.rs:392-400`

Tests endpoint :
- Happy path HTTP : `crates/nexus-shell-daemon/src/http.rs:6190-6233`
- Not found 404 : `crates/nexus-shell-daemon/src/http.rs:6235-6252`

Test Vitest bridge :
- `proof_card_get` dispatch : `web/src/bridge/__tests__/useBridge.test.ts:424-465`

Commandes exécutées :
- `cargo nextest run -p nexus-coordinator-rs -E "test(proof_card)" --locked` : 8/8 pass.
- `cargo nextest run -p nexus-shell-daemon -E "test(proof_card)" --locked` : 2/2 pass.
- `npm run test:unit -- useBridge` : 1 fichier, 18/18 pass.
- `cargo nextest run -p sbfb-manifest -E "test(bridge_allowlist)" --locked` : 1/1 pass.

Impact : non bloquant pour les tests demandés. Ces tests ne couvrent pas le gap du livrable 1 sur `source`, `artifact_hash`, `content_hash`, `critical`, `no_curator`, `single_curator`, `no_open_source`.

**10. Scope cuts Phase A**
Statut : **CONFIRME**

- Aucun fichier `ProofCard.tsx` trouvé via `rg --files web/src crates examples | rg "ProofCard\\.tsx$|SearchManifest|scan[-_]?secrets|preview\\.rs$|diff\\.rs$"`.
- Recherche ciblée dans les fichiers Phase A : seul faux positif `diffs` dans un commentaire existant `web/src/bridge/protocol.ts:67`.
- Recherche des lignes ajoutées daemon/core : seul faux positif `Self-published` dans fixture de test `crates/nexus-shell-daemon/src/http.rs:6202`.
- Les changements fonctionnels observés sont limités à ProofCard, endpoint, bridge, allowlist, tests : `crates/nexus-shell-daemon/src/http.rs:358`, `crates/nexus-shell-daemon/src/http.rs:1998-2113`, `web/src/bridge/useBridge.ts:373-383`.

Impact : pas de fuite Phase B/C/D détectée.

**Sécurité**
S1 SQL injection : **CONFIRME**
- `project_id` vient du path : `crates/nexus-shell-daemon/src/http.rs:1998-2001`
- Handler appelle `db.get_provenance_by_project(&project_id)` : `crates/nexus-shell-daemon/src/http.rs:2059`
- DB utilise placeholder SQL `?1` : `crates/nexus-coordinator-rs/src/db.rs:740-743`
- DB bind avec `rusqlite::params![project_id]` : `crates/nexus-coordinator-rs/src/db.rs:745`
- Pas de concat SQL dans ce chemin.

S2 Auth middleware : **CONFIRME**
- Route dans `authed_routes` : `crates/nexus-shell-daemon/src/http.rs:262-358`
- `.layer(middleware::from_fn_with_state(auth, auth_required))` appliqué : `crates/nexus-shell-daemon/src/http.rs:428`
- Router merge ensuite `authed_routes` : `crates/nexus-shell-daemon/src/http.rs:430-433`

S3 Nouveau `unsafe` : **CONFIRME**
- Recherche sur lignes ajoutées : aucun `+ ... unsafe`.
- Les seuls `unsafe` trouvés dans les fichiers touchés sont préexistants dans tests env var : `crates/nexus-shell-daemon-core/src/browse.rs:1493` et `crates/nexus-shell-daemon-core/src/browse.rs:1497`.

S4 Secrets / credentials : **CONFIRME**
- Recherche sur lignes ajoutées pour `secret|password|credential|api_key|private_key|BEGIN PRIVATE KEY|sk-...` : aucun hit.
- Le token fixe existant est un token de test documenté : `crates/nexus-shell-daemon/src/http.rs:2130-2135`, pas un secret runtime.

**Résumé final**
- Total livrables : 10
- Confirmés : 9
- Partiels : 1
- Gaps : 0
- Bloquant avant commit : **oui, livrable 1 PARTIEL vs SYNTHESIS §4.6**