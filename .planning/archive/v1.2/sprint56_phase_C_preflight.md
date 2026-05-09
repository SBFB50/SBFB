# Sprint 56 Phase C — preflight G8

Date : 2026-05-09 | HEAD : `056dcc7` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, context7 systematic — respecte (aucune lib externe nouvelle, approach OSS validee)
- feedback_context7_systematic.md : N/A — pas de nouvelle lib/API externe, bridge est du code interne SBFB

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (Figma plugins, VS Code webviews, Sandstorm), APPROACH-ALIGNED — le pattern postMessage bridge avec methodes typees + correlation IDs est le standard pour iframes sandboxees — clean
- S1b deps : 0 nouvelle dep (Zod, axum, serde_json deja workspace), 0 delta — clean
- S2 historiques : 4 fichiers cibles scannes (http.rs, protocol.ts, useBridge.ts, sbfb-bridge.js), git log historique + archive v1.2 S18-S20 (DEVIATION Ed25519/threat-model, non-pertinent bridge), memory feedback clean — clean
- S3 threat model : fast-path verified — phase n'introduit pas de nouveau composant securite ni wire format P2P. Bridge postMessage est protocole interne iframe-host, pas wire P2P. HARDENING_ROADMAP pas d'entree S56 specifique — clean
- S4 wire format : fast-path — aucun fichier canonical.rs/schemas dans perimetre. VERSION=1 preservee. Day 0 D3 (bridge extensions 5 methodes) respectee par construction — clean

## Note implementation

Les endpoints `/app/{name}/state/{key}` (GET/POST) references par
les methodes bridge existantes `storage_get`/`storage_set` n'existent
pas dans le daemon Rust (http.rs ne contient aucune route `/app/`).
Gap pre-existant probable vestige du coordinator Python supprime S51.
Phase C ajoute `storage_list` et `storage_delete` qui necessitent un
backend storage. Approche MVP : HashMap in-memory dans DaemonHttpState
+ 4 routes (GET list, GET single, POST set, DELETE) pour avoir un
storage fonctionnel. Les 2 endpoints "existants" (get/set) sont
egalement a creer pour que le bridge soit coherent end-to-end.
Cela reste dans le scope D3 "bridge extensions" — les methodes
storage_get/storage_set existaient dans le bridge frontend mais sans
backend. Phase C complete le circuit.

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~2m / 3 projets OSS consultes / finding : APPROACH-ALIGNED
- S1b : ~30s / 0 libs scannees (pas de nouvelle dep) / finding : clean
- S2 : ~1m / 4 fichiers + archive v1.2 scannes / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase C.
