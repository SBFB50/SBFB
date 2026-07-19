# Preflight G8 — Sprint 83 Phase C (autorite du bridge, intersection 7 couches per-call)

Tip evalue : `b2ece81`. Lecture seule stricte (aucun fichier edite). Chaque fait porte une evidence file:line vue par Grep/Read ; les faits verdict-critiques ont ete re-verifies independamment de la synthese des 5 scans.

Produit par le Workflow ultracode `wf_ca52dc3e-402` : 5 scans factuels (S1a OSS prior-art + S1b deps/CVE + S2 decisions historiques + S3 threat model + S4 wire format) + refutation adversariale par scan + synthese. 11 agents Opus 4.8 1M, 0 erreur. Faits porteurs (sbfb-explorer sans clef `bridge` ; ancre THREAT_MODEL:186) re-verifies main-thread.

## Verdict: PLAN-ADAPT

Coder la Phase C est le bon mouvement — la premisse P0 est fondee et aucun invariant Day-0 n'est viole — mais cinq corrections concretes issues du repo modifient l'approche du plan et doivent etre integrees avant le premier Edit.

Convergence des scans : S1a PLAN-ADAPT, S1b EXECUTE (dimension deps/CVE), S2 PLAN-ADAPT, S3 PLAN-ADAPT (GAP), S4 EXECUTE (dimension wire). Les deux EXECUTE sont scopes a leur dimension propre (0 dep nouvelle ; 0 bump wire) ; l'agregat retient PLAN-ADAPT car les corrections S1a/S2/S3 (ancre, carry egress, posture legacy) touchent le comment-coder. **La refutation adversariale n'a rien refute comme confabulation** : tous les symboles/lignes cites existent, avec des derives de numeros de ligne de 0-7 (aucune load-bearing sauf l'ancre THREAT_MODEL, traitee ci-dessous).

## S1a OSS prior-art

**Confirme (evidence).**
- Placement de l'enforcement : `web/src/bridge/useBridge.ts:164` (`if (req.method === "pii_redact")`) `return` a `:176`, AVANT `dispatch()` appele a `:187`. Un check interne a `dispatch()` laisserait pii_redact (l'effet offline/local) non gouverne. Choke-point requis juste apres le source-check `:157`.
- Alignement SOTA : l'invariant ocap/macaroon/UCAN « derive weaker never stronger » == Research Pack §14.1:1236 « chaque couche ne peut qu'attenuer ». Le bridge EST un confused deputy (il porte le bearer authFetch et dispatche n'importe lequel des 16 verbes pour n'importe quelle iframe), cf. `crates/sbfb-manifest/src/lib.rs:52-59`. structuredClone ne peut passer de reference de capacite vive -> un check host per-call est la bonne approximation ocap sous contrainte postMessage. SENSITIVE_ACTIONS substring (`operator_server.rs:37`) correctement NON etendu.

**Refute / recadre.**
- « Composer capability_store.rs comme layer-1 » : SUGGESTION discutable, pas un drop-in. Re-verifie moi-meme : `crates/nexus-coordinator-rs/src/capability_store.rs:11` `KNOWN_CAPABILITIES` = `[biometric_gate, federation_canary, mcp_server_expose, rag_retrieval, streaming_bridge, tool_calling]` = jeu de toggles feature S25/S41 DISJOINT, et cote COORDINATEUR (pas le host web). Recommander un GrantSet host-side NEUF.
- Finding LangGraph CVE (INFO) : leg repo confirme (parse = Zod strict `protocol.ts` `BridgeRequestSchema`, pas de code-deserialization -> classe RCE ne transfere pas) ; les identifiants CVE externes restent UNCERTAIN (non re-verifies en session, sans impact sur le verdict).

**Implication Phase C.** Choke-point unique apres `:157` ; GrantSet host-side en treillis d'attenuation monotone avec stubs inertes pour les couches pack-version/compiled-plan absentes ; revocation macaroon-style revoke-by-id consultee au call-time, jamais cachee (ferme TOCTOU-on-grant, forward-closure L/V/AI).

## S1b Deps-CVE

**Confirme (evidence).** « 0 dep runtime nouvelle pour C » fonde au lock-node : `crates/sbfb-manifest/Cargo.toml:11-14` = serde+serde_json+thiserror ; la logique grant v0 (`grant_v0::tests::{usurpation_refused, expiry_refused, revocation_per_call}`, `sprint83_plan.md:442`) est PURE (ensemble + timestamp), aucune crypto. blake3 1.8.5 / ed25519-dalek 2.2.0 / serde_json 1.0.149 / serde_jcs 0.2.0 / schemars 1.2.1 deja dans Cargo.lock. Cote web : `web/package.json:48` zod ^3.25.76, intersection = module interne, 0 dep npm. Aucune surface CVE nouvelle ; `deny.toml` n'ignore que RUSTSEC-2026-0194/0195 (quick-xml via iroh=1.0.1, hors surface bridge) ; multiple-versions=warn = P2-AUDIT-2-RESIDUEL externe-bloque, inchange par C.

**Refute.** L'ancre task « ligne STALE = THREAT_MODEL.md:179 » : REFUTED — ligne 179 VIDE ; vraie claim a `:186` (cf. §S3 et §Correction docs-contrat).

**Implication Phase C.** Nuance P3 a encoder dans le commit body : si le coder tague une frontiere bridge/grant avec schema (AM-3) ou grant signe, `sbfb-manifest` gagnera des lignes `{ workspace = true }` (schemars/ed25519-dalek/blake3) = **0 nouveau noeud de lock** — a qualifier explicitement pour eviter un faux flag « dep runtime nouvelle » en review/Codex, ou eviter via l'exemption `// FRONTIER-NO-SCHEMA:` deja ratifiee PO (AM-3 Option A).

## S2 Decisions historiques

**Confirme (evidence).** L'evolution 3->16 est faite de commits-feature de ROUTINE (c32d9c7 S13=3 ... 1d2fb36 S76H task_result=16), 0 DEVIATION enregistree ; « 3 methodes » = jeu MVP S13, JAMAIS un plafond. Rendre le dispatch autoritaire EXECUTE l'intention de securite figee (whitelist + validation source + Zod strict), ne la contredit pas -> **pas de DESIGN-CONFLICT**. Precedent de derive DEJA survenu : `lib.rs:61-66` documente S76-B10 (BRIDGE-ALLOWLIST-DRIFT, allowlist a 10 vs dispatch reel) — enjeu re-hausse maintenant que la declaration devient autoritaire.

**Corrections retenues (toutes CONFIRMED par re-verification).**
- P1 : le CARRY d'egress promis a C par la review B (`sprint83_phase_b_review.md:83`) et le code B (`reserved-events.ts:79-88`) est ABSENT des livrables C.2/C.3 (que de l'ingress). Verifie moi-meme : la section Phase C du plan (407-461) ne contient aucun `egress|pushEvent|reserved-event|payloadCarries`.
- P2 : docstring `lib.rs:52-59` (« DECLARATIVE ... NOT the runtime dispatch boundary ... an app cannot reach a method by declaring it, nor lose sandbox protection by omitting it ») DESAVOUE le role d'autorite que C.2 lui donne — a reconcilier, enforcement garde cote HOST (TS).
- P2/P3 : « 3 methodes » stale aussi a `CLAUDE.md:90` et `:602` (frozen-decisions) — re-verifie moi-meme, hors frontiere docs-contrat de C (THREAT_MODEL seul), a tracer pour le wrap-up.

## S3 Threat model

**Confirme (evidence), signal GAP.** Trois legs modifient l'approche :
- **[P2] Ancre fausse** : re-verifie moi-meme — `THREAT_MODEL.md:186` = row I « 3 methodes whitelist (task_submit, storage_get, storage_set) + 1 canal event push ; schema Zod strict | L ». `:179` VIDE, `:180` = header `### 5.2 postMessage bridge`. Le critere binaire C.4 (`sprint83_plan.md:450` « THREAT_MODEL.md:179 = 16 methodes ») est INSATISFIABLE tel quel. Derive +7 = insertion §17/A9 en Phase B.
- **[P1] Carry egress orphelin** : `THREAT_MODEL.md:1683` (T-A11Y-PROJECTION-SMUGGLE) + `reserved-events.ts:88` + `phase_b_review:83` promettent tous que Phase C ferme « l'autorite d'egress typee complete » ; plan C.2/C.3 = zero livrable/test egress.
- **[P1] Regression legacy CONCRETE (renforcee au-dela du scan)** : re-verifie moi-meme — `examples/sbfb-explorer/SBFB.json` (schema_version 2) OMET la clef `bridge` (grep `"bridge"` = 0), or son code invoque **16** methodes via `_call(...)` (task_submit, storage_get/set/list/delete/version, provenance_get/verify, browse_list, search, proof_card_get, node_status, identity_pubkey, feed_cursor_get, pii_redact, task_result). Une intersection fail-closed sur `methode-declaree-par-release` (set declare vide) refuse TOUT et brique le Protocol Explorer.

**Deferrals honnetes.** In-flight TOCTOU (revocation pendant la fenetre async d'un appel deja admis) : `useBridge.ts:187` `void dispatch(...)` async apres le handler sync ; correctement defere L/V/AI par `gate_matrix.md:38` — a documenter en residu T-BRIDGE-REVOKE-INFLIGHT. Etiquette 7-vs-5 couches (INFO). CLAUDE.md hors scope (P3).

## S4 Wire invariants

**Confirme (evidence), signal NO-DELTA.** Aucun invariant wire n'est menace.
- **Census DOMAIN = 25** reproduit exactement au tip (`find crates -name '*.rs' -exec grep -hoE 'const DOMAIN_[A-Z0-9_]+_V[0-9]+' | sort -u | wc -l` = 25, 25 familles distinctes listees). Gate `scripts/check-frontier-contracts.sh:190` `DOMAIN_CENSUS_FROZEN=25`. Grant v0 = etat runtime host-side, 0 nouveau DOMAIN (grep `grant_v0|GrantSet` crates/web = 0 hit -> NEUF).
- **contracts.rs len==7** intact : `crates/sbfb-workflow-core/src/contracts.rs:129` `CONTRACT_NAMES:[&str;7]`, `:143` assert `obj.len()==7`. Hors liste des livrables C.
- **Parite BRIDGE_METHOD_ALLOWLIST Rust==TS==16** : `lib.rs:67-91` = 16, `protocol.ts:20-49` = 16 ; tenue par tests unitaires miroir (`allowlist_mirrors_host_dispatch_schema` `lib.rs:195-196` + `protocol.test.ts:142`). **`check-frontier-contracts.sh` n'a AUCUN check bridge** (grep bridge/BRIDGE_METHOD/protocol.ts = 0 match) -> le drift-gate C.2/C.4 est du NET-NEW, pas une extension.
- **0 bump wire** : FEED_FORMAT_VERSION=1 (`public_feed.rs:20`), PROJECT_ANNOUNCEMENT_VERSION=1 (`publish.rs:24`), ANNOUNCEMENT_VERSION=1 (`iroh_runtime.rs:93`) — enveloppe bridge inchangee, AM-12/G-WIRE-COMPAT scope Phase P.
- **9 sbfb-bridge.js** trackes (re-verifie : `git ls-files | grep -icE 'sbfb-bridge\.js$'` = 9) ; wrappers de commodite sans constante d'allowlist ; 10e anticipe au scaffold Hub.

## Disposition du CARRY egress (Phase B -> C)

**Recommandation : FOLD dans Phase C** (PLAN-ADAPT avec livrable additionnel).
- Ajouter a C.2/C.3 un livrable d'autorite d'egress typee sur `pushEvent` qui FERME le leg **getter-TOCTOU** par **clone-once-scan-send-same-snapshot** : `const snap = structuredClone(payload); if (payloadCarriesCause(snap)) return; postMessage(createEvent(name, snap), ...)`. Aujourd'hui `useBridge.ts:243` poste `createEvent(name, payload)` avec le payload ORIGINAL apres `payloadCarriesCause(payload)` (`:237`) -> 2 invocations de getter. Le snapshot collapse a une seule execution de getter (scanne ET envoye = memes octets geles). Test rouge-avant/vert-apres.
- Le **residu value/nom-encoding** (encodage delibere de valeurs de need sous clef arbitraire / tableau nu / nom d'event) reste HORS-SCOPE (hote appelant de confiance, chemin sanctionne deja needs-free par construction) et EST re-documente comme **residu ACCEPTE**, jamais silencieusement abandonne.
- Alternative si le PO refuse le fold : reclasser formellement `§17 T-A11Y-PROJECTION-SMUGGLE` (`THREAT_MODEL.md:1683`) et `reserved-events.ts:88` de « = Phase C » vers « residu accepte (hote de confiance) » AVANT de coder. Sinon un carry route par la review B s'evapore et l'audit gate le releve (doctrine « dette audit = phases du sprint »).

Severite securite du residu = **L** ; severite processus du carry = **P1**.

## Delta plan (PLAN-ADAPT) — precis, actionnable

1. **Ancre (bloquant C.4)** : remplacer `THREAT_MODEL.md:179` -> `:186` partout (plan 416/432/450 + kickoff 511/629/728) ; ancrer par CONTENU / tag `// FRONTIER`, pas par numero de ligne nu.
2. **Choke-point (P1)** : intersection en UN point unique juste apres `useBridge.ts:157`, AVANT la branche `pii_redact` (`:164`), pour gouverner l'effet offline/local autant que les 15 verbes coord ; test pii_redact-refuse-offline.
3. **Posture legacy direct-load (P1)** : specifier la posture de grant du chemin Browse->iframe ; ajouter cas no-regression (methodes d'une app existante dispatchent toujours) ET cas distinct `no-grant-record -> refuse (fail-closed default)`. **Arbitrage PO requis** (cf. Questions PO — sbfb-explorer ne declare rien mais appelle 16 methodes).
4. **Drift-gate (P2)** : CONSTRUIRE un vrai bloc de parite bridge dans `check-frontier-contracts.sh` (parser `BridgeMethodSchema` + diff `BRIDGE_METHOD_ALLOWLIST`) ; ne pas creer une 4e surface main-synced (deriver d'une source canonique).
5. **Carry egress (P1)** : folder pushEvent (clone-once-scan-send) + test, ou reclasser formellement (cf. section dediee).
6. **Docstring (P2)** : reconcilier `lib.rs:52-59` avec le nouveau role d'input d'intersection ; enforcement garde cote HOST (TS) ; parite sous gate mecanique (enjeu securite, plus courtoisie).
7. **GrantSet (P2)** : treillis host-side NEUF en attenuation monotone + stubs inertes pack-version/compiled-plan ; NE PAS recabler `capability_store.rs` (coordinateur, toggles disjoints).
8. **Etiquette (INFO)** : body de commit = « Phase C = sous-ensemble host-side 5 couches du modele 7 couches ».
9. **TOCTOU in-flight (P2, defere)** : documenter T-BRIDGE-REVOKE-INFLIGHT (L/V/AI) ; preciser que `revocation_per_call` teste le pre-call, pas l'in-flight.

## Invariants wire (0 bump / census 25 / parite) — confirmes

`wire_invariants_ok = true`. Census DOMAIN = **25** (reproduit exactement au tip) — grant v0 host-side, 0 nouveau DOMAIN_*_V1. `contracts.rs` **len==7** intact. Parite **BRIDGE_METHOD_ALLOWLIST Rust==TS==16**. **0 bump wire** (FEED_FORMAT_VERSION / *_ANNOUNCEMENT_VERSION restent a 1 ; enveloppe bridge inchangee). Gate SENSITIVE_ACTIONS substring correctement **NON etendu** (`operator_server.rs:37`). R-iroh-audit P0 non affecte (aucune surface iroh). L'intersection est un gate ADDITIONNEL AVANT les appels loopback authFetch existants -> retrecit strictement la surface d'attaque, ne l'elargit jamais. Pre-launch respecte : ajout de champ/struct host-side libre avant tag v1.0.

## Correction docs-contrat THREAT_MODEL:179

**Cible reelle = ligne 186** (l'ancre `:179` du plan est une ligne VIDE ; `:180` est le header `### 5.2 postMessage bridge`). **N = 16 verbes reels** (`protocol.ts:20-49` == `BRIDGE_METHOD_ALLOWLIST` `lib.rs:67-91`).

Reecrire la row I `:186` :
- Avant : `| I | Bridge leak des donnees cross-iframe | M | 3 methodes whitelist (task_submit, storage_get, storage_set) + 1 canal event push ; schema Zod strict | L |`
- Apres : `| I | Bridge leak des donnees cross-iframe | M | 16 methodes bridge typees + intersection d'autorite host-side per-call (Phase C : caller ∩ declare-par-release ∩ grant ∩ politique locale ∩ consentement) ; schema Zod strict | L |`

Ajouter les rows residuels T-BRIDGE-USURP (fail-closed), T-BRIDGE-GRANT-EXPIRY, T-BRIDGE-LEGACY-DEFAULT, T-BRIDGE-REVOKE-INFLIGHT (defere L/V/AI) ; resoudre §17 T-A11Y-PROJECTION-SMUGGLE (`:1683`) selon la disposition du carry ; §18 entree historique v21. **Surfaces STALE hors frontiere a tracer pour le wrap-up (pas dans le commit C)** : `CLAUDE.md:90` (« Modele de rendu ») et `CLAUDE.md:602` (« Decisions gelees »), toutes deux « 3 methodes » — a signaler au routeur pour correction en lockstep (frozen-decisions).

## Questions PO ouvertes

1. **Posture de grant du chemin direct-load (Browse->iframe)** : `sbfb-explorer` invoque 16 methodes bridge alors que son `SBFB.json` n'a AUCUNE clef `bridge` ; une intersection fail-closed sur `methode-declaree-par-release` — meme avec grant load-time = methodes-declarees — le briquerait (set declare vide). Option A : grandfather direct-load (clef `bridge` absente => set-implicite = toutes les methodes du dispatch ; seules les apps pack declarent un grant d'installation). Option B : exiger que `SBFB.json` declare desormais ses methodes + migration des 9 apps trackees. Option C : clef `bridge` absente = « pas de contrainte-release » (couche inerte), l'autorite venant du grant d'installation/politique. Quelle posture pour v0 ?
2. **Carry egress typee** : folder dans Phase C (livrable pushEvent + fermeture getter-TOCTOU) ou reclasser formellement en residu accepte (hote de confiance) ? Recommandation preflight = folder le leg getter-TOCTOU (cheap) + documenter le residu value/nom-encoding comme accepte.
3. **Etiquette « 7 couches » vs 5 enforcees en C** : simple clarification de wording dans le body (recommande), ou modeliser explicitement les 2 couches absentes pack-version/compiled-plan en stubs inertes dans le code de C ?

---

## Arbitrage PO (resolu)

- **Q1 — posture de grant direct-load** : reponse initiale « B — exiger declaration + migrer 9 apps »,
  **raffinee par le PO en seance** : les 9 apps d'`examples/` sont d'anciens fixtures de test (4 d'entre
  elles bundlent le SDK complet `sbfb-bridge.js` -> 16 `_call(...)` ; ce sont des demos figees) — **ne pas
  les faire evoluer**, focus **workflow-hub**. Posture reconciliee retenue (un mecanisme, deux regimes) :
  - manifeste avec clef `bridge` PRESENTE (⟹ `workflow-hub`, futurs packs) = **fail-closed autoritaire**
    (intersection `methodes-declarees ∩ grant ∩ politique ∩ consentement`, per-call, offline inclus) ;
  - manifeste SANS clef `bridge` / indisponible (⟹ les vieilles apps de test) = **grandfather legacy**
    (permissif sur les 16 verbes du dispatch) -> **0 migration, rien ne casse** ;
  - `workflow-hub` declare son `bridge.methods` dans `SBFB.json` (minimal/honnete : il n'appelle encore
    aucun verbe au scaffold Phase B ; la declaration grandit avec ses usages en R+) = **seule app sous le
    regime fail-closed complet** (le focus) ;
  - preuve d'enforcement = **harness synthetique + app-fixture e2e hermetique dediee**, jamais les vieux exemples ;
  - residu assume pre-launch (une app legacy sans declaration reste permissive) -> `T-BRIDGE-LEGACY-DEFAULT`,
    severite L (apps de test locales, source verifiable, sandbox CSP) ; bascule « toutes les apps declarent »
    = post-v1.0.
- **Q2 — carry egress typee** : **A — folder le getter-TOCTOU dans C** (clone-once-scan-send-same-snapshot
  dans `pushEvent`) + test rouge-avant/vert-apres ; residu value/nom-encoding re-documente comme accepte.
- **Q3 — etiquette 7 vs 5 couches** : **A — clarification wording** (body : « Phase C = sous-ensemble
  host-side 5 couches du modele 7 couches »).

Le code suit le delta 1-9 ci-dessus + cette posture (grandfather absent-key / fail-closed declared).
