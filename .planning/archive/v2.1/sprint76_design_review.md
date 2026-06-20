# Sprint 76 — Design Review Board (G1)

**Date** : 2026-06-15
**Sprint** : 76 — GPU partagé volontaire, prouvé cross-machine (Arc 3.5 Factory Complete Vision 6/6)
**Reviewer** : self-review profond (auto-challenge systématique des 5 décisions Day-0 S76)

---

## Scoring

| D# | Titre | Source récente (<90j) | Alternative rejetée (source) | [DETER] Crypto/spec | [DETER] Rust-first | Code vérifié | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Surface « offrir ma puissance » — reuse consent 4 niveaux + caps + GPU monitor | ok (BOINC/FAH/Petals/Salad WebFetch 2026-06-15 ; Petals `--public_name` confirmé) | ok (BOINC `global_prefs` rejeté avec source ; flag `enabled` séparé rejeté ; vast.ai self-test rejeté ; HTTP worker endpoint rejeté) | N/A | N/A | ⚠️ | ⚠️ |
| D2 | E2E cross-machine task-routing compute (lève B-3) | ⚠️ (iroh-docs/BOINC/Petals/Ray/GPUStack consultés 2026-06-15 mais artefacts stables anciens ; pas de source primaire <90j sur le point de décision « pull vs push ») | ok (Ray Serve GCS, GPUStack RPC, Petals/Parallax DHT, BOINC server, loopback HTTP — 5 rejets sourcés) | N/A | ok (iroh 0.98 Rust pinné réutilisé ; transports concurrents Hivemind/Lattica rejetés comme dépendance dupliquée) | ok | ⚠️ |
| D3 | Quorum redundancy>1 sur sorties DÉTERMINISTES — cohorte homogène exact-match + TOPLOC en réserve | ⚠️ (TOPLOC arXiv 2501.16007 = 2025-01 >90j ; Thinking Machines/SGLang = 2025-09 >90j ; Ingonyama 2024-09 ; arXiv 2601.14277 <90j mais c'est une source D5) | ok (batch-invariant kernels, match sémantique, zkLLM, no-redundancy Petals, logits-complets — 5 rejets sourcés daté) | ok (TOPLOC <6 mois à l'écriture initiale, alternative concurrente Parallax « coming next », source <2 ans) | ok (Ollama/llama.cpp Rust backends réutilisés ; fork kernels rejeté) | ⚠️ | ⚠️ |
| D4 | Dashboard contributeur — comptabilité kudos non-monétaire per-task | ok (BOINC CreditNew/CreditOptions, FAH QRB, Gridcoin, EigenTrust — tous WebFetch 2026-06-15) | ok (Gridcoin monétaire rejeté=décision gelée, EigenTrust Sybil rejeté avec papier, FAH QRB rejeté, statu quo rejeté, champ wire signé rejeté) | N/A | N/A | ok | ✅ |
| D5 | Quantization 4-bit documentée (GGUF doc-only ; runtime quant déjà présent ; 70B=S77) | ok (arXiv 2601.14277 = 2026-01-11 <90j ; bartowski GGUF + llama-cpp-2 docs.rs 2026-06-15) | ok (AWQ/GPTQ/EXL2/bitsandbytes/Q2_K-défaut/tensor-split-en-S76 — 6 rejets sourcés) | N/A | ok (llama-cpp-2 Rust binding ; vLLM/ExLlamaV2/Transformers Python rejetés avec gap factuel in-process) | ok | ✅ |

**Résumé** : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅ — Rigor signal G4 satisfait (3 ⚠️ sur 5).

Le gold standard est 1-2 ⚠️ ; ici 3 ⚠️ — au-dessus de la zone rubber-stamp, le board a fait son travail. Les 3 ⚠️ sont des findings réels et corrigibles inline, pas des rejets de décision. Aucune des 5 décisions n'est invalidée ; toutes restent les bons choix produit/architecture. Les ⚠️ portent sur (D1) un bug de wiring latent + une décision PO non tranchée surfacée comme « risque » plutôt qu'arbitrée ; (D2) l'absence de source primaire récente sur le point de décision et le double-comptage du risque de convergence WAN ; (D3) une recency réelle des sources crypto-déterminisme + une affirmation « code vérifié » partiellement périmée (le champ `model_digest`/`logprobs_hash` existe déjà, ce que la recherche présente comme net-new).

---

## Findings

### D1 ⚠️ — Le wiring `/api/v1` vs `/consent/set` et la décision « le worker co-localisé honore-t-il `consent.json` » sont présentés comme « risques à trancher » alors que l'un est un bug bloquant et l'autre est LA décision Day-0 de D1

**Detail** :

Le bloc D1 a une analyse de réutilisation excellente et 4 rejets bien sourcés (item 2 satisfait). Mais l'item 5 « code vérifié / changement faisable » est ⚠️ pour deux raisons concrètes vérifiées dans le code :

1. **Le préfixe de route est laissé en « question ouverte »** (`web/src/api/consent.ts` POST `/consent/set` vs daemon `http.rs:423 /api/v1/consent/set`). Si la page consent est inerte en prod packagée, alors le pilier #1 de D1 (« écrire le niveau choisi dans `consent.json` via la route existante `/consent/set` ») ne fonctionne **pas** — la faisabilité du changement n'est donc pas établie, elle est conditionnée à un fait non vérifié. Un design review board ne peut pas valider « faisable » sur une route dont on ignore si elle est branchée.

2. **La vraie décision Day-0 de D1 — « le worker co-localisé doit-il lire le `consent.json` utilisateur (L≥2) au lieu du `Whitelist[own_doc]` hardcodé » (`local_worker.rs:307-308`, vérifié : `consent.level = ConsentLevel::Whitelist` + `allowed_project_ids.insert(project_id)`) — est rangée en « Risque/Question ouverte #2 » au lieu d'être tranchée.** C'est pourtant le cœur de « offrir ma puissance » : sans ce changement, activer L4 dans le dialog ne fait rien servir de public (le worker reste verrouillé sur son propre doc). Une décision Day-0 ne peut pas déléguer son cœur à un « à trancher au plan ».

**Decision** : adjust. Au kickoff §4, transformer les deux « risques » en décisions explicites :
- (a) **Le bug `/api/v1` est un pré-requis bloquant de D1, pas un risque** — l'inscrire comme première tâche de la phase D1 (vérifier `web/vite.config.ts` + réconcilier le préfixe) avec critère d'acceptance « POST consent depuis le front packagé écrit `consent.json` ». S'il s'avère que c'est un proxy Vite dev qui masque un trou prod, c'est un fix(sprint76) légitime dans la phase, pas un report.
- (b) **Trancher la sémantique d'enrôlement maintenant** : `OwnProjects`/`Whitelist`(L1) = OFF least-privilege (le worker co-localisé garde son `Whitelist[own_doc]` actuel) ; `OpenSource`/`All`(L≥2) = le worker co-localisé **lit le `consent.json` utilisateur**. Verrou : `All`(L4) reste un opt-in double-confirmé (cohérent avec le `threatNote` « risque maximum » déjà affiché). Écrire ça comme décision, pas comme option. (Nuance de nommage à corriger au passage : la recherche parle de « L1-L4 » mais l'enum réel est `OwnProjects/OpenSource/Whitelist/All` — `OwnProjects` est le least-privilege, pas `Whitelist` ; le kickoff doit utiliser les noms réels de `consent.rs:397-413` pour éviter une confusion d'implémentation.)

---

### D2 ⚠️ — Pas de source primaire <90j sur le point de décision « pull vs push », et le risque de convergence WAN est sous-évalué par rapport à un constat d'acceptance déjà observé

**Detail** :

Le bloc D2 est solide sur les rejets (Ray/GPUStack/Petals/BOINC/loopback, 5 rejets factuels — item 2 ok) et sur le Rust-first (iroh 0.98 réutilisé, transports concurrents rejetés — item 4 ok). L'E2E test et le result-sync bridge cités existent bien (vérifié : `runtime.rs:3629 e2e_network_execute_gate_real_http_no_frontier_mock`, `result_sync.rs:142 spawn_result_subscribe`, `runtime.rs:692` le câble — item 5 ok). Le ⚠️ porte sur l'item 1 et sur l'honnêteté du risque :

1. **Item 1 ⚠️ — aucune source < 90 jours n'étaye le point de décision.** Toutes les références D2 (BOINC wiki, Petals v2.0.0 de 2023, Ray 2.55.1, iroh-docs) sont des artefacts stables anciens consultés aujourd'hui ; c'est légitime pour « code OSS lu », mais le board ne voit aucune source primaire récente confirmant que « pull node-centrique sur doc répliqué » reste l'état de l'art en 2026 face à des schedulers récents. La décision est correcte (et de toute façon contrainte par iroh 0.98 gelé), mais la checklist source-récente n'est pas satisfaite au sens strict.

2. **Le risque de convergence WAN est minimisé.** D2 note « à vérifier au preflight que `result:` réplique en < 30s sur WAN » — mais la mémoire S75 contient un **constat d'acceptance déjà observé** : `SeedAnnounced ne converge pas cross-noeud (peer_count:0 ~10 min)`. D2 distingue (à juste titre) la réplication de DOC du gossip de feed, mais présente cette distinction comme une mitigation acquise alors que c'est précisément l'hypothèse non vérifiée sur laquelle repose tout le palier 1. Un board ne peut pas laisser « le chemin compute utilise un autre chemin que le seed qui ne converge pas » comme affirmation rassurante sans la marquer comme hypothèse-à-falsifier en premier.

**Decision** : adjust. Au kickoff §4 :
- (a) Reformuler la justification « pull vs push » : la décision n'est pas étayée par une source récente, elle est **dérivée de la contrainte gelée iroh 0.98 + du modèle S75 prouvé** — l'écrire ainsi (decision = forced-by-frozen-stack, pas « SOTA pull »), ce qui est plus honnête et tout aussi valide.
- (b) **Élever la convergence `result:` cross-machine WAN au rang de premier critère d'acceptance falsifiable de la phase B-3**, en référençant explicitement le constat S75 `SeedAnnounced peer_count:0`. L'acceptance doit ouvrir par : « mesurer le délai de réplication `result:` PC→VPS sur WAN réel ; si > timeout du gate (150×200ms), c'est un BLOCK à diagnostiquer, pas un timeout à rallonger ». Sinon le risque de répéter un faux-vert (l'acceptance « passe » parce qu'on a élargi le timeout) est réel.

---

### D3 ⚠️ — Sources crypto-déterminisme >90j ET affirmation « code vérifié » partiellement périmée : `model_digest` et `logprobs_hash` existent DÉJÀ dans `ResultPayload`, ce que D3 présente comme net-new

**Detail** :

D3 est le bloc le plus rigoureux sur le fond (6 rejets sourcés et datés, item 2 ok ; TOPLOC satisfait [DETER] crypto item 3 avec alternative <6 mois à l'écriture + source <2 ans ; Ollama/llama.cpp Rust item 4 ok). Le ⚠️ vient de deux écarts factuels que le board doit signaler — c'est exactement le rôle anti-complaisance :

1. **Item 1 ⚠️ (recency) — les sources crypto-déterminisme sont toutes >90 jours.** TOPLOC = arXiv 2501.16007 (jan 2025, vérifié), Thinking Machines = sept 2025 (vérifié via Simon Willison/LMSYS), SGLang = 2025-09-22, Ingonyama = 2024-09. Aucune n'est <2026-03-17. La seule source <90j du bloc (arXiv 2601.14277, jan 2026) appartient à D5, pas à D3. La décision reste correcte (les faits physiques FP/batch-invariance ne périment pas), mais la case « source récente » n'est pas honnêtement cochable.

2. **Item 5 ⚠️ (code vérifié) — l'« Implication code » centrale de D3 est partiellement périmée.** D3 propose « ajout **additif** `model_digest`/quant côté `Task`… `#[serde(default)]`, net-new » et présente `PendingResultPersist.result_hash` comme « le futur emplacement d'un proof TOPLOC (étage 2) ». Or le code actuel contient **déjà** :
   - `ResultPayload.model_digest: [u8; 32]` (`task.rs:374`, vérifié) — doc-commenté « layer 2 of the verification stack, compared against a whitelist of known-good model digests » ;
   - `ResultPayload.logprobs_hash: [u8; 32]` (`task.rs:383`, vérifié) — doc-commenté « **layer 3 of the verification stack** », i.e. l'emplacement TOPLOC-shaped existe déjà et s'appelle `logprobs_hash`, pas `result_hash` ;
   - mais l'implémentation à `runtime.rs:1082` calcule `model_digest = blake3(task.model.as_bytes())` = **hash du NOM du modèle**, pas des octets du GGUF (le doc-comment dit « exact model file » — discordance doc/impl pré-existante).

   Conséquence : la recherche a lu la **struct `Task`** (`task.rs:74-213`) et n'a pas vu que le digest vit dans `ResultPayload` (`task.rs:349-435`). Le routing « cohorte homogène par digest » n'est donc pas un « ajout additif de champ » — c'est (i) corriger `model_digest` pour qu'il hashe le fichier GGUF et non le nom, et (ii) advertir ce digest dans la capability worker, ce qui est un travail différent et plus subtil que « ajouter un champ ». Présenter ça comme net-new additif sous-estime la faisabilité réelle.

**Decision** : adjust. Au kickoff §4 :
- (a) Marquer honnêtement la recency : « décision étayée par des faits physiques FP stables (sources 2024-2025), pas par une publication <90j ; la seule source <90j [arXiv 2601.14277] relève de D5 ».
- (b) **Réécrire l'Implication code D3** : ne PAS dire « ajout additif `model_digest` à `Task` ». Dire : « `ResultPayload.model_digest` (`task.rs:374`) et `logprobs_hash` (`task.rs:383`) EXISTENT déjà comme couches 2/3 de vérification ; `model_digest` est actuellement `blake3(model_name)` (`runtime.rs:1082`), pas le digest du fichier GGUF (discordance avec son doc-comment). Le routing cohorte-homogène S76 doit (i) décider si on durcit `model_digest` vers un hash de fichier GGUF [P1, ou doc-note si hors-scope], (ii) advertir le tuple (model_digest, quant, runtime_family) dans la capability worker. `logprobs_hash`, pas `result_hash`, est le slot TOPLOC de l'étage 2 ». Cette correction rend la frontière S76/S77 plus nette et évite de re-coder un champ existant.
- (c) Conserver la Q3 « résultat attendu honnête = exact-match tient en cohorte homogène, diverge sur GPU hétérogène » comme critère d'acceptance écrit (anti faux-vert T1) — c'est déjà bien dans le bloc, le garder.

---

## Notes pour D4 et D5 (✅ — pas de finding, mais points à porter au plan)

- **D4 ✅** : bloc exemplaire. Les 5 rejets sont sourcés (Gridcoin=violation décision gelée, EigenTrust=papier Stanford + faiblesse Sybil citée, FAH QRB, statu quo, champ wire signé). Le trou anti-gaming réel (`tokens_generated` self-déclaré hors-quorum, vérifié `kudos_ledger.rs:56,73` + `task.rs:363`) est correctement identifié et chiffré (`log_utility` compresse <10× mais ne supprime pas). La granularité per-task native (vérifié : `credit()` une ligne par task, `prev_hash` per-project `kudos_ledger.rs:64`) est exacte. **À porter au plan** : la Q1 (durcir `amount` = `log_utility(median(tokens_generated))` du groupe d'accord vs documenter le trou en P2) est une décision PO légitime, pas un défaut de la décision Day-0. Aucun ajustement kickoff requis.
- **D5 ✅** : bloc exemplaire. arXiv 2601.14277 (jan 2026, <90j, **vérifié réel** — confirme Q4_K_M ~1pt MMLU). Tous les rejets runtime (AWQ/GPTQ/EXL2/bitsandbytes) ont le gap factuel in-process Rust requis par [DETER] Rust-first. Le verdict « 70B sur 1-2 cartes 16GB GPU-pur = impossible (42.5>32), donc objectif intrinsèquement S77 » est honnête et la table d'empreintes est juste. Le scope « doc-only, runtime quant déjà présent inchangé » est vérifié (`llama_cpp.rs:149-150` ne câble que `with_n_gpu_layers`, pas `split_mode`/`devices`). **À porter au plan** : la décision « câbler le tensor-split mono-machine en S76 OU le laisser S77 » est tranchée correctement (S77, doc-only en S76) ; le préflight pourra ré-évaluer si le delta est trivial, mais le défaut par défaut est sain. Aucun ajustement kickoff requis.

---

**Findings résolus dans ce review** : 3 ⚠️ (D1, D2, D3), tous **adjust** (corrections inline à appliquer au kickoff §4). 0 **acknowledge**. Le board confirme que les 5 décisions Day-0 sont les bons choix ; les ajustements portent sur l'honnêteté des cases recency/code-vérifié et sur la promotion de deux « risques » au rang de décisions/acceptances bloquantes (D1 wiring + sémantique enrôlement ; D2 convergence WAN ; D3 réécriture de l'Implication code autour des champs `model_digest`/`logprobs_hash` déjà présents).

---

## Checklist [DETER]

### Crypto/spec (D3 — quorum déterminisme / TOPLOC)
- [x] D-choice crypto cite >=1 alternative concurrente < 6 mois (TOPLOC <6 mois à l'écriture initiale, alternative Parallax « coming next »)
- [x] Source datée < 2 ans ou revalidée (TOPLOC arXiv 2501.16007, source <2 ans)
- [x] Reviewer ⚠️ si alternative absente — ⚠️ posé sur la **recency** (toutes sources crypto-déterminisme >90j) et sur le **code vérifié** (`model_digest`/`logprobs_hash` déjà présents), pas sur l'absence d'alternative

### Rust-first
- [x] D2 (runtime transport) cite >=1 alternative Rust-native production : iroh 0.98 réutilisé ; transports concurrents Hivemind/Lattica rejetés comme dépendance dupliquée
- [x] D3 (runtime inférence) cite Ollama/llama.cpp Rust backends réutilisés ; fork kernels rejeté avec gap factuel
- [x] D5 (runtime quant) cite llama-cpp-2 Rust binding ; vLLM/ExLlamaV2/Transformers Python rejetés avec gap factuel in-process
- [x] Gap factuel documenté pour chaque alternative Rust rejetée
- [x] Reviewer ⚠️ si gap non documenté — aucun gap Rust-first non documenté ; les ⚠️ D1/D2/D3 ne portent pas sur cet axe (D1 item 5 wiring/sémantique, D2 item 1 recency + risque WAN, D3 item 1 recency + item 5 champs pré-existants)
- Exemptions : CI tooling, frontend UX (D1 panneau « offrir ma puissance »), docs (D5 doc-only quantization), tests fixtures
