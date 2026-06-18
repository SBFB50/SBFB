# Roadmap v5 — Factory Complete Vision (Arc 3.5)

**Statut** : CANON (decision PO 2026-05-30).
**Supersede** : la ligne S71 de `roadmap_v4_neutral_protocol_factory_rrv.md`
(« SearchManifest opt-in ou RRV Core selon audit S70 »). Le RRV/recherche
reseau n'est PAS abandonne — il est **absorbe** dans cet arc (S73) au
service de l'atelier Factory.
**Origine** : deux cartographies multi-agents 2026-05-30 (surface Factory
+ couche compute/RRV/OSS LLM distribue). Artefacts d'intake :
`.planning/active/sprint71_intake.md` + `.planning/research/` (sharding).

**AMENDEMENT 2026-06-09 (pivot PO découverte)** : un bug live (découverte
PUSH-éphémère, fenêtre PoW 30 min → apps publiées >30 min invisibles aux
nouveaux pairs, cross-machine Win↔Mac) impose une **découverte PULL
node-centrique** *fondationale* avant le GPU (on ne partage pas du GPU entre
nœuds qu'on ne découvre pas). **Ré-ordonnancement** : **S75 = découverte PULL
node-centrique + ancre VPS** (annuaire `NodeDirectoryEntry` signé répliquable ;
le SearchManifest D3/s73 reste DIFFÉRÉ — l'annuaire n'est PAS le SearchManifest,
cf. `sprint75_pivot_proposal.md`) ; **GPU partagé cross-machine → S76** ;
**sharding pipeline → S77**. Kickoff/plan/design_review/pivot_proposal :
`.planning/active/sprint75_*`. La ligne « S75 — GPU partagé » §3 ci-dessous est
décalée S76 par cet amendement.

**LIVRAISON 2026-06-11 (S75 Phase G)** : l'amendement est exécuté — S75 a
livré la découverte PULL node-centrique complète (FIX-A re-mint,
`NodeDirectoryEntry` + `DOMAIN_NODE_DIRECTORY_V1`, ingest + durabilité
locator `anchors.json`, pull multi-provider, ancre VPS headless systemd,
front node-Browse `/nodes` + `/node/:id`, acceptance survives-VPS-death).
Détail : `sprint75_verification.md`. **Prochain : S76 = GPU partagé
cross-machine** (la ligne « S75 — GPU partagé » §3 se lit S76 ; sharding
S77 inchangé).

**LIVRAISON 2026-06-17 (S76 Phase G) — Arc 3.5 Factory Complete Vision
6/6 CLOS** : S76 a livré le **GPU partagé cross-machine** — le task-routing
du **modèle ENTIER** d'une machine à une autre (panneau « offrir ma
puissance » + enrôlement worker co-localisé D1 ; E2E cross-machine compute
B-3 + cohorte homogène `RuntimeTuple` D2/D3 ; quorum redundancy>1
déterministe + fix prod bridge result-sync dedup `(worker_pubkey,task_id)`
D3 ; dashboard contributeur + anti-gaming sanity-bound D4 ; quantization
4-bit doc-only D5). Arbitrage PO **« personne n'a 2 GPU »** → le
mono-machine 2-GPU est ENTERRÉ ; le multi-GPU réaliste = **cross-machine
2+ machines × 1 GPU = sharding pipeline S77** (feature distincte, pas un
defer). Acceptance LIVE cross-machine = différée-matériel-opérateur
(harness `b3_live_pc_vps.sh` runnable palier 1+2 via `REDUNDANCY`). Détail :
`sprint76_verification.md`. **Prochain : S77 = sharding pipeline** (modèle
70B éclaté cross-machine ; prérequis livré = routing modèle entier + quorum
+ cohorte homogène). Audit gate S76 = `sprint77_audit_plan.md`.

---

## 0. Le pourquoi — Factory ferme la boucle SBFB

Le pitch d'origine de SBFB est « decentralized P2P compute network for
apps ». Le projet a construit, separement :

- une **couche compute P2P** (workers Ollama, coordinator
  dispatcher/validator/quorum, consentement GPU 4 niveaux) ;
- une **couche creation/publication** (Factory : templates, pipeline
  FG, deploy-from-repo, provenance Ed25519) ;
- une **couche decouverte** (browse pkarr, curator lists, FTS5).

Ces trois couches **ne se parlent pas encore**. La « finition totale de
Factory » que demande le PO n'est pas du polish d'UI — c'est **brancher
les trois couches l'une sur l'autre** pour que Factory devienne le
front-end de tout SBFB :

> On **cherche** une app sur le reseau, on l'**ouvre/forke** dans
> l'atelier, on la fait **evoluer** en pilotant un agent — et cet agent
> tourne au choix sur Claude cloud **ou** sur des modeles open-source
> **repartis sur les GPU de noeuds volontaires** (« mini data centers
> entre individus »). On **publie** sur le reseau, on **consulte** les
> preuves.

Bonne nouvelle etablie par la cartographie : **l'infra existe a ~90 %**.
Le travail est du **cablage + reparation + durcissement + preuve
cross-machine**, pas de la fondation.

---

## 1. Decisions PO actees (2026-05-30) — ne pas rebattre dans l'arc

| # | Decision | Choix PO |
|---|----------|----------|
| PO-1 | Forme de l'Operator | **Atelier d'apps complet** (boucle creer→piloter→verifier→publier→consulter sur un projet cible distinct du repo nexus) |
| PO-2 | Pilotage agent embarque (terminal/chat) | **Garder + gater proprement** (confirmation par action sensible, encadrer bypassPermissions, modele opus-4-8, timeout, amender le contrat Operator) |
| PO-3 | Bloc ~14 commits off-sprint (~5500 lignes) | **Reconciliation complete** (retro-review + retro-Codex + retro-audit + tests) |
| PO-4 | Packaging produit | **In-scope** (serveur sert l'UI, launcher conscient de Factory, doc install operateur) |
| PO-5 | Source du fork d'un projet publie | **Les deux** : forge (`repo_url@commit`) si dispo, **blob zip reseau en repli** |
| PO-6 | GPU distribue / « mini data centers » | **Sharding WAN prioritaire** (feature phare) — un gros modele decoupe entre cartes de noeuds distants, en assumant 1-3 tok/s batch/async + la verif des shards |
| PO-7 | Perimetre v2.1 | **Arc complet S71-S76** |
| PO-8 | Templates atelier | static + static-reader, **+ react + python/pyodide** (decide au kickoff S74) |
| PO-9 | Editeur | **pas de Monaco** — l'agent edite, l'operateur supervise (terminal/chat/diff) |
| PO-10 | Viewer | reste **app SBFB sandboxee** (contrat gele), l'atelier y **renvoie** (pas d'iframe blob direct) |
| PO-11 | Validation sorties LLM stochastiques | **greedy seed-fixe** pour taches verifiables (quorum hash exact) ; logprobs/watermark en V2 |
| PO-12 | Incitation GPU | **volontaire + kudos non-monetaire** (zero token crypto — decision gelee maintenue) |
| PO-13 | RRV | cabler **les deux** : shell pour decouvrir, Factory pour reprendre/forker (Factory apprend a **tirer** du reseau) |
| PO-14 | Claude cloud | reste le **pilote conversationnel principal** (gates/verdicts/commit/push via vraie session agent) ; local/reseau servent les sous-taches de fond |

PO-8 a PO-14 sont des **defauts** recommandes, confirmables/affinables au
kickoff du sprint concerne (les sous-features se decident au bon moment,
avec verification de l'etat reel du code — discipline « sessions
fraiches »).

---

## 2. La verite d'ingenierie non-negociable — l'ordre de dependances

Le PO a choisi le sharding « prioritaire » et l'arc complet. **Prioritaire
= livrable engage et dote d'un vrai poids R&D, PAS construit en premier.**
Raison physique : on ne peut pas decouper un modele entre machines
distantes tant qu'une **seule tache** ne sait pas router entre deux
machines. Aujourd'hui elle ne sait pas.

**Constats bloquants (cartographie compute) :**

- **B-1** : le dispatcher ecrit la cle `tasks/{id}` mais le worker lit
  `task:` (`dispatch_loop.rs:35` vs `runtime.rs:845`). **Aucune tache
  dispatchee n'est jamais vue par un worker reel** — le flux compute ne
  marche qu'en test in-process par injection directe.
- **B-2** : le quorum compare le **hash exact** du texte
  (`validator.rs:115`) → deux workers honnetes en sampling sont tous
  rejetes. Inference multi-worker inutilisable sans determinisme.
- **B-3** : **aucune preuve cross-machine** du chemin compute
  (coordinator→worker→Ollama→validation). Feed/blob valides LAN/WAN, pas
  le compute.

**Ordre impose (chaque etage depend du precedent) :**

```
S71 assainir+securite+reconciliation  (fonde TOUT)
   └─ S72 quick win: chat Factory sur routage de taches existant
        └─ S73 recherche reseau cablee (fraicheur + SearchResult enrichi)
             └─ S74 atelier: rouvrir/forker un projet reseau
                  └─ S75 GPU partage PROUVE cross-machine (task-routing)
                       └─ S76 STRETCH: sharding pipeline (le cousin dur)
```

Le sharding (S76) **ne s'empile jamais** avant la preuve cross-machine du
task-routing (S75). Sa **recherche + son design** sont tires en amont
(`.planning/research/`, en cours) pour que S76 soit du build documente,
pas un spike baclé. S76 peut etre **dedouble** (design prouve →
implementation) plutot que comprime, et peut glisser hors v2.1 si non
prouve — sans bloquer la boucle produit (S71-S74) qui apporte 80 % de la
valeur.

---

## 3. L'arc S71-S76

> **NOTE de numérotation (amendement 2026-06-09, exécuté S75-G + S76-G)** :
> le découpage ci-dessous est PRÉ-amendement. Réel : la découverte PULL
> node-centrique s'est intercalée en **S75**, donc « ### S75 — GPU partagé »
> ci-dessous = **S76 (LIVRÉ, Arc 3.5 6/6 clos)** et « ### S76 — STRETCH
> sharding » = **S77 (à ouvrir, feature distincte non-stretch)**. L'arc réel
> est S71-S77. Cf. les deux blocs LIVRAISON en tête de fichier.

### S71 — Assainissement compute + securite + reconciliation

Sprint de **consolidation d'ouverture d'arc** (phases elargies, zero
feature speculative). Regularise le bloc off-sprint AVANT d'empiler.

- Fix **B-1** (cle dispatch) + 1er **E2E cross-process**
  coordinator→worker→Ollama→validation (inexistant aujourd'hui).
- **B-2** validation stochastique : greedy seed-fixe pour taches
  verifiables ; decision logprobs/watermark (aujourd'hui
  `model_digest=blake3(name)`, `logprobs_hash=32 zeros`, inerte).
- Reconcilier la double notion **provider** (string adaptation-prompt
  `process.rs` vs runtime `LlmBackend`). Clarifier/retirer
  `RedundancyDispatcher` mort. Cabler ou retirer `execute_build`.
- **Securite Factory** : gater `bypassPermissions` (filtre
  SENSITIVE_ACTIONS sur le SSE), modele `opus-4-8` (pas `sonnet`),
  CORS restreint + token local, timeout subprocess + diagnostic claude.
- **Reconciliation process** du bloc off-sprint (retro-review + retro-
  Codex + retro-audit) + tests des surfaces non-testees (terminal,
  llm_bridge, sprint_history, endpoints).
- Decision **WIP terminal** `.cast`→`.log` (`stash@{0}`).
- **Phase 0 = audit-absorb** du bloc off-sprint (l'audit gate S70 n'a pas
  tourne et le tip a diverge ; on absorbe la dette en entree, l'audit
  gate S71 de sortie valide reconciliation + phases).

### S72 — Factory provider routing (QUICK WIN)

- Trait `ProviderRouter` cote Factory → `impl Stream<StreamChunk>`.
- `ClaudeProvider` (defaut cloud, inchange) | `OllamaProvider` (local) |
  `NetworkProvider` (`POST /api/v1/tasks/submit` → poll, **async non-
  streaming** — assume le mode polling/await, pas SSE token-par-token).
- Cabler `ChatSendRequest.model/provider` (aujourd'hui ignore).
- UX intentions : « Executer en local / sur le reseau / sur Claude ».

### S73 — Recherche reseau cablee

- Pont **feed-distant → reindex FTS5 a chaud** (`feed_sync.rs:260`) —
  corrige la fraicheur (projets recents invisibles jusqu'au reboot).
- Enrichir `search_index`/`SearchResult` avec
  `repo_url+commit_sha+archive_hash+provenance_hash` (sinon un hit ne
  peut pas declencher un fork).
- Barre de recherche shell cablee sur `GET /api/daemon/search`.
- Decision **SearchManifest** (recherche reseau opt-in propagee) vs
  feed-local-replique, selon audit S72.

### S74 — Atelier : rouvrir/forker un projet reseau

- Commandes `sbfb-factory search/open/fork` (Factory apprend a **tirer**).
- `reseau→atelier` : clone `repo_url@commit_sha` (forge) **ou**
  reconstruction depuis le blob zip (repli) → nouveau workspace.
- **Notion de projet cible** (distinct du repo nexus — `process::repo_root`
  pointe toujours sur nexus aujourd'hui, G17).
- Bouton UI « Forker dans l'atelier ». Boucle prouvee :
  chercher→forker→editer→redeploy sous sa propre identite noeud.
- Templates etendus (react, pyodide). Packaging/onboarding atelier.

### S75 — GPU partage volontaire, prouve cross-machine

- Reutiliser consent 4 niveaux + caps W/VRAM/h + GPU monitor → panneau
  « offrir ma puissance ».
- **E2E cross-machine** du task-routing compute (leve B-3).
- Quorum `redundancy>1` prouve sur sorties deterministes.
- Dashboard contributeur (kudos non-monetaire per-task).
- Quantization 4-bit documentee (70B sur 1-2 cartes 16GB).

### S76 — STRETCH R&D : sharding pipeline « gros modele »

- Spike sharding pipeline-parallel (Petals/Parallax) sur **iroh QUIC
  streams**, sous-groupes **faible latence**.
- Scheduler latency-aware (router selon RTT mesure, grouper peers
  proches — pattern Parallax).
- **Verification/redondance des activations** (le travail ORIGINAL : les
  peers ne sont pas confiants ; l'OSS suppose le contraire). Design dans
  `.planning/research/`.
- Realite assumee : **1-3 tok/s WAN → batch/async, jamais chat live**.
- NE PAS empiler avant preuve S75. Peut etre dedouble ou glisser.

---

## 4. Reutilisation existante (extrait — detail dans l'intake)

Compute : wire `Task/Claim/Result` Ed25519+JCS testes ; pompe worker
complete ; backends `OllamaBackend`+`LlamaCppBackend` (vraie abstraction
runtime) ; consent GPU + caps + UsageTracker ; validator + quorum DB ;
dispatch `POST /api/v1/tasks/submit`. Decouverte : triplet provenance
de bout en bout ; `deploy-from-repo` complet ; `BrowseAggregator` ;
FTS5 bm25. Factory : `StreamChunk` SSE unifie ; `assemble_prompt` plat ;
`ChatSendRequest` porte deja `provider+model`. Point d'insertion provider
unique : `handle_chat_stream` → `spawn_claude_stream`.

---

## 5. Risques majeurs (detail dans l'intake)

- Empilement premature (sharding avant preuve cross-machine).
- Validation stochastique non resolue → compute multi-worker inutilisable.
- Latence sharding WAN surinterpretee (ne JAMAIS promettre du chat live).
- Securite des shards sur peers non-confiants (travail original).
- UX provider incompatible avec dispatch reseau async (assumer le polling).
- Scope creep « atelier complet » dilue le quick win S72 (phaser strict).

---

## 6. Note de coherence avec les decisions gelees

Cet arc **respecte** : Factory = crate externe hors daemon (v4 D2) ;
kudos non-monetaire, zero crypto (vision gelee) ; OS sandbox, pas wasmtime ;
Viewer = app SBFB sandboxee (contrat RRV/Factory §3) ; @protocole d'abord.
Il **amende** : le contrat Operator §4 pour autoriser explicitement le
pilotage agent local privilegie **gate** (PO-2). Politique pre-launch :
rien n'est pousse vers origin (11 commits ahead) → reconciliation locale
sans contrainte de compat.
