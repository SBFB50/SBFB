Verdict global — **PARTIEL**. Aucun bypass P0/P1 ni autre exemple actuellement sous-déclaré. Restent 2 gaps documentaires, un garde incomplet et une divergence TS/Rust latente.

1. **CONFIRMÉ — choke-point unique**

- Source vérifiée à [useBridge.ts:168](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:168), contrôle de source à [useBridge.ts:173](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:173), autorité à [useBridge.ts:177](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:177), avant `pii_redact` à [useBridge.ts:199](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:199) et avant `dispatch` à [useBridge.ts:222](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:222).
- `dispatchPiiRedact` et `dispatch` sont privés à [useBridge.ts:321](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:321) et [useBridge.ts:333](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:333). Aucun autre consommateur production de `BridgeRequestSchema` n’existe.

2. **PARTIEL — algèbre correcte, “EXACTEMENT” faux hors domaine normal**

- Expiration `>=`, révocation, intersection des couches présentes, `None`/`null` neutre, borne allowlist et empty-declared sont symétriques à [grant_v0.rs:55](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/grant_v0.rs:55), [grant_v0.rs:80](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/grant_v0.rs:80), [authority.ts:51](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:51) et [authority.ts:77](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:77).
- La borne off-allowlist tient des deux côtés à [grant_v0.rs:93](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/grant_v0.rs:93) et [authority.ts:84](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:84).
- Divergence acceptée validate-vs-drop correctement documentée : Rust rejette à [lib.rs:123](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/lib.rs:123), TS retire les noms invalides à [authority.ts:133](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:133).
- Divergence latente non documentée : Rust impose `u64` à [grant_v0.rs:48](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/grant_v0.rs:48), alors que TS accepte tout `number` à [authority.ts:31](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:31). `NaN` ou `Infinity` rendent `nowMs >= expiresAtMs` faux et donc le grant vivant. Aucun impact production Phase C — le resolver émet uniquement `null` à [authority.ts:143](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:143) — mais le miroir n’est exact que pour des millisecondes finies et représentables.

3. **CONFIRMÉ — getter-TOCTOU et refus synchrone**

- Clone-scan-send du même snapshot à [useBridge.ts:273](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:273).
- Le test est réellement rouge avant : l’ancien code conservait l’objet à getter, puis `JSON.stringify(sent)` le relisait et révélait `needs`. Le discriminateur est à [useBridge.test.ts:760](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/useBridge.test.ts:760), avec compteur secondaire à [useBridge.test.ts:763](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/useBridge.test.ts:763).
- L’usurpation produit une réponse immédiate et zéro fetch à [intersection_authoritative.test.ts:59](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/intersection_authoritative.test.ts:59).

4. **CONFIRMÉ — parité fonctionnelle 16/16**

- Rust 16 à [lib.rs:77](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/lib.rs:77), TS 16 à [protocol.ts:20](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/protocol.ts:20), dispatch 15 cases plus `pii_redact`.
- Le bloc extrait `[a-z0-9_]+`, refuse les extractions vides et compare les ensembles à [check-frontier-contracts.sh:286](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:286) et [check-frontier-contracts.sh:304](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:304).
- Reproduction indépendante : `TS_COUNT=16`, `RS_COUNT=16`, ensembles égaux ; suppression unilatérale simulée de `search` détectée.
- P3 documentaire : le prologue annonce encore « Four deterministic checks » à [check-frontier-contracts.sh:8](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:8), alors que le script en contient cinq.

5. **PARTIEL — rows demandées correctes, documentation globale encore contradictoire**

- Row I correcte à [THREAT_MODEL.md:186](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:186). Les cinq rows sont présentes à [THREAT_MODEL.md:188](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:188), avec revoke-inflight et resolve-failopen honnêtement résiduels.
- Getter-TOCTOU fermé et encodage valeur/nom accepté à [THREAT_MODEL.md:1688](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1688). `reserved-events.ts` est bien au passé à [reserved-events.ts:85](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/reserved-events.ts:85).
- GAP docs-contrat : le DFD du même threat model énumère toujours seulement `task_submit / storage_get / storage_set / event` à [THREAT_MODEL.md:99](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:99).
- Autres contrats publics toujours stales : trois méthodes seulement à [SBFB_JSON_V2.md:48](/C:/Users/FlowUP/Documents/Code/nexus/docs/protocol/SBFB_JSON_V2.md:48) et [EXTERNAL_AUDIT_SCOPE.md:108](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/EXTERNAL_AUDIT_SCOPE.md:108).
- La nouvelle autorité `storage_version` d’Ideas Hub manque encore dans [README.md:263](/C:/Users/FlowUP/Documents/Code/nexus/README.md:263) et [README_EN.md:163](/C:/Users/FlowUP/Documents/Code/nexus/docs/community/README_EN.md:163).

6. **CONFIRMÉ — résolution/fallback**

- URL, fetch, statut HTTP, JSON et resolver sont tous dans le `try/catch`; tout échec retourne le grandfather à [BrowsedProject.tsx:303](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/BrowsedProject.tsx:303).
- `bridge` absent ou `null` grandfathere ; toute valeur non-null produit un declared set fail-closed à [authority.ts:155](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/authority.ts:155).
- La fenêtre permissive jusqu’à résolution est explicitement passée au hook à [BrowsedProject.tsx:328](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/BrowsedProject.tsx:328) et documentée à [THREAT_MODEL.md:192](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:192).

Round 2 — **CLEAN**

- Off-allowlist : HOLDS Rust/TS.
- Révocation live→revoked→appel suivant : HOLDS à [intersection_authoritative.test.ts:140](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/intersection_authoritative.test.ts:140), sous le résidu d’effet passif ratifié.
- Regex `[a-z0-9_]+` et libellé non-vacuity : HOLDS à [check-frontier-contracts.sh:293](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:293).
- Policy `null`, cinq rows, absence d’overclaim mutation-tested : HOLDS.
- E2E « grandfather UNTIL resolved » : HOLDS à [bridge-grant-intersection.spec.ts:16](/C:/Users/FlowUP/Documents/Code/nexus/web/e2e/bridge-grant-intersection.spec.ts:16).
- `bridge:null` : HOLDS.

Round 3 — **PARTIEL documentaire**

- Comptes correctement avancés après Round 4 : +8 Rust/2137 et +38 web/528 à [sprint83_phase_c_review.md:17](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_review.md:17), [sprint83_phase_c_review.md:21](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_review.md:21) et [sprint83_phase_c_review.md:30](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_review.md:30). Le +28/518 Round 3 est donc honnêtement supplanté par les dix nouveaux cas.
- Plan corrigé à [sprint83_plan.md:432](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:432).
- Résidu effet passif correctement documenté à [useBridge.ts:132](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/useBridge.ts:132) et [THREAT_MODEL.md:191](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:191).
- GAP : [sprint83_phase_c_codex_review.md:23](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_codex_review.md:23), [ligne 30](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_codex_review.md:30) et [ligne 40](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_c_codex_review.md:40) portent encore les conclusions Round 2 désormais fausses : ancien libellé mutation, plan `179`, anciens comptes. À remplacer ou étiqueter explicitement comme snapshot Round 2 avant commit.

Round 4

- **(a) PARTIEL** : le correctif source est exact à [sbfb-ideas/SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/SBFB.json:8), avec appel à [app.js:337](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/app.js:337), transitivité à [sbfb-bridge.js:320](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/sbfb-bridge.js:320), [ligne 333](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/sbfb-bridge.js:333) puis `_call("storage_version")` à [ligne 274](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/sbfb-bridge.js:274). La narration v21 est correcte, mais les docs publiques listées au verdict 5 restent stales.
- **(b) PARTIEL** : le test est réellement rouge-avant/vert-après, mais le garde n’est pas un vrai contrôle helper→méthode. À [example_manifest_declarations.test.ts:160](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/example_manifest_declarations.test.ts:160), il compare uniquement l’ensemble des valeurs `_call("x")` avec l’ensemble des valeurs de la table. Un nouveau helper `readStorage() -> _call("storage_get")` ne change aucun ensemble : le drift-test passe, mais `.readStorage()` n’est pas détecté. Une association transitive erronée `onStorageUpdate -> autre méthode déjà connue` passe aussi. De plus, les regex excluent les chiffres à [ligne 118](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/example_manifest_declarations.test.ts:118) et [ligne 167](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/example_manifest_declarations.test.ts:167), et `appSources` ne lit que les `.js` top-level à [ligne 95](/C:/Users/FlowUP/Documents/Code/nexus/web/src/bridge/__tests__/example_manifest_declarations.test.ts:95), pas HTML inline, sous-répertoires, `.mjs` ou `.ts`.
- **(c) CONFIRMÉ — audit courant complet** :

| App déclarante | Déclaré | Atteint indépendamment | Verdict |
|---|---|---|---|
| compute-tester | `task_submit`, `task_result` à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/compute-tester/SBFB.json:8) | [app.js:59](/C:/Users/FlowUP/Documents/Code/nexus/examples/compute-tester/app.js:59), [app.js:78](/C:/Users/FlowUP/Documents/Code/nexus/examples/compute-tester/app.js:78) | couvre |
| sbfb-factory-viewer | 4 méthodes à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-factory-viewer/SBFB.json:8) | `proof_card_get`, `browse_list` à [app.js:91](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-factory-viewer/app.js:91), [app.js:148](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-factory-viewer/app.js:148) | sur-déclare, couvre |
| sbfb-ideas | 6 méthodes à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-ideas/SBFB.json:8) | `storage_list/set/delete`, `identity_pubkey`, `storage_version` transitif | couvre après fix |
| daisyui-animejs-showcase | `[]` à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/daisyui-animejs-showcase/SBFB.json:8) | aucun helper, `_call` ou BridgeRequest dans les sources expédiées | couvre |
| velocity-scroll-demo | `[]` à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/velocity-scroll-demo/SBFB.json:8) | aucun appel dans le JS inline | couvre |
| workflow-hub | `[]` à [SBFB.json:8](/C:/Users/FlowUP/Documents/Code/nexus/examples/workflow-hub/SBFB.json:8) | uniquement l’événement custom non-BridgeRequest à [a11y.js:81](/C:/Users/FlowUP/Documents/Code/nexus/examples/workflow-hub/a11y.js:81) | couvre |

Les quatre SDK vendored des exemples sont actuellement byte-identiques. Aucun autre exemple déclarant ne sous-déclare. `sbfb-explorer` n’a aucune clé `bridge` à [SBFB.json:6](/C:/Users/FlowUP/Documents/Code/nexus/examples/sbfb-explorer/SBFB.json:6) et relève bien du grandfather.

Vérifications exécutables : `tsc --noEmit`, ESLint ciblé, `cargo fmt --all --check` et `git diff --check` verts. Vitest n’a pas pu démarrer dans la sandbox read-only (`EPERM` sur `web/node_modules/.vite-temp`). Le gate Bash complet n’a pas été confirmé ici ; son bloc parité a été reproduit indépendamment avec mutation unilatérale en mémoire.

