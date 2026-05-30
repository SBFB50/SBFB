# Sprint 71 — Kickoff (contrat d'entree)

**Ecrit** : 2026-05-30 par la session de cadrage (agent kickoff).
**Theme** : Assainissement compute + securite Factory + reconciliation
du bloc off-sprint. Sprint de **consolidation d'ouverture d'arc**
(Arc 3.5 Factory Complete Vision, roadmap v5).
**Roadmap CANON** : `.planning/roadmap_v5_factory_complete_vision.md`.
**Intake** : `.planning/active/sprint71_intake.md`.
**Nature** : phases elargies, zero feature speculative. Chaque item
reference un bug P0/P1, un carry, ou un test E2E manquant identifie
(regle anti-derive §6.2.1 Regle 1).

---

## 1. Sources research consultees (pre-gel)

Les decisions Day 0 ci-dessous (§5) ne reposent pas sur de la
recherche externe SOTA — c'est un sprint de consolidation dont les
items sont des bugs ancres dans le code present, pas des choix
technologiques nouveaux. Les sources consultees sont donc internes,
factuelles, lecture-de-code :

| Source | Usage | Date |
|--------|-------|------|
| `git log 201b24d..HEAD` (12 commits ahead origin) | Cartographie du bloc off-sprint (G5) | 2026-05-30 |
| `git stash list` (3 stashes, `stash@{0}` WIP terminal) | Decision G1 Phase A | 2026-05-30 |
| `dispatch_loop.rs:35` vs `runtime.rs:833,845` | Constat B-1 (cle dispatch) | 2026-05-30 |
| `validator.rs:68,91,115` | Constat B-2 (quorum hash exact via `result_text`) | 2026-05-30 |
| `llm_bridge.rs:74,80` + `operator_server.rs:87-90,776` | Constats G2/G7/G9/G12 securite Factory | 2026-05-30 |
| `daemon_client.rs:64-65` (X-SBFB-Token + Host guard) | Pattern auth correct a propager (G7) | 2026-05-30 |
| `docs/agent/RRV_FACTORY_CONTRACT.md §4` | Contrat Operator a amender (PO-2) | 2026-05-30 |
| `roadmap_v5 §1` (PO-1..PO-14) | Decisions PO gelees, non-rebattues | 2026-05-30 |
| Memory `feedback_model_46.md` | Regle modele `claude-opus-4-8[1m]` (G9) | 2026-05-30 |
| Memory `feedback_kudos_non_monetary.md` | Kudos non-monetaire (PO-12, pas touche S71) | 2026-05-30 |

**Decision crypto/spec nouvelle ?** Non. PO-11 (greedy seed-fixe)
est une decision PO actee dans la roadmap, pas un choix crypto a
sourcer ici. La checklist `[DETER]` crypto/spec (§6.1.1) ne
s'applique pas — aucune primitive crypto/spec nouvelle n'est
introduite. Les 3 deps off-sprint (`portable-pty`, `async-stream`,
`futures`) passent au preflight G8/S1b en Phase B (G13), pas au gel
Day 0.

---

## 2. Constat d'entree

### 2.1 D'ou on part — situation de process NON-STANDARD

Sprint 71 ne demarre PAS sur un tip propre auditeable. La situation
est decrite en detail dans `intake §1`. Resume :

- **Tip** : `d5ddb95` (+ `d4bcceb chore(planning)` redirect roadmap
  v5). **12 commits ahead d'origin/master** — rien n'est pousse.
- **S70 clos** au commit `201b24d`. MAIS ~14 commits
  `feat/fix(factory)` + `docs(community)` ont lande APRES, **hors
  cycle sprint** (`e26d9f2..d5ddb95` cote factory,
  `1e8f6da..046ea3b` cote docs community). `git diff --stat
  201b24d..HEAD` = 33 fichiers, +5574/-682. Zero preflight, zero
  review, zero Codex, zero body 9 sections.
- **`sprint70_audit_findings.md` n'existe pas.** L'audit gate de
  cloture S70 n'a jamais tourne et le tip a depuis diverge — on ne
  peut donc pas rejouer l'audit gate S70 sur un tip propre.
- `.planning/active/` contient encore les docs S70 + le
  `sprint71_audit_plan.md` herite. A migrer vers `archive/v2.1/`
  a l'ouverture (chore, hors phase feat).
- **WIP terminal** : refactor `.cast`→`.log` incomplet dans
  **`git stash@{0}`** « WIP terminal plaintext-logging refactor
  (incomplete) -- S71 Factory ». Le HEAD actuel ecrit toujours
  l'asciicast `.cast` (`terminal.rs:27,30,133`). Tree propre,
  `cargo check -p sbfb-factory` OK au HEAD.

### 2.2 Compteurs tests entree (tip `d5ddb95`)

Baseline declaree (source : memory `nexus_grid_pivot.md` + CLAUDE.md
etat S70) : ~1486 Rust / 279 Vitest / 6 size-limit. Le bloc
off-sprint a ajoute des surfaces **largement non-testees** (G6 :
`terminal.rs` 0 test, `sprint_history.rs` 0/1047 lignes,
`operator_server` unit 0, `process.rs` 0, spawn LLM/PTY 0). Les
compteurs exacts d'entree seront re-mesures et inscrits dans
`plan.md §1` (etat verifie) au demarrage de l'execution, sur le SHA
reel post-migration chore.

### 2.3 Pre-launch protocol policy (rappel)

Rien n'est pousse vers origin (12 ahead). La **reconciliation locale
est libre** : pas de bump de version wire, edition du canonical
autorisee. `TASK_FORMAT_VERSION` et les `*_ANNOUNCEMENT_VERSION`
restent a 1. La correction B-1 (cle dispatch) **change le wire
applicatif** (cle de doc `tasks/{id}` vs prefixe `task:`) mais
**aucun noeud tiers ne parle ce protocole en prod** — la correction
est libre, pas de migration, pas de decodeur range.

### 2.4 Verdict audit gate du sprint precedent

**Inexistant a l'entree.** L'audit gate S70 n'a pas tourne (§2.1).
La strategie de reconciliation (§3 ci-dessous, Phase 0) produit
retroactivement `sprint70_audit_findings.md` en absorbant le bloc
off-sprint comme dette d'entree. C'est une **deviation documentee du
pattern §3.1**, validee PO (PO-3, decision 2026-05-30).

---

## 3. Phase 0 — Audit-absorb du bloc off-sprint (deviation §3 documentee)

**Decision PO 2026-05-30 (PO-3 reconciliation complete).** Le pattern
standard §3.1 (Phase 0 = audit du sprint precedent sur tip propre)
ne s'applique pas tel quel : le tip a diverge et l'audit S70 n'a
jamais tourne. A la place :

1. **Audit-absorb** : la session fraiche S71 ingere le diff
   `201b24d..HEAD`, le traite comme **dette d'entree**, et ecrit
   `sprint70_audit_findings.md` — un audit retroactif couvrant a la
   fois S70 (livraisons normales) ET le bloc off-sprint (~14 commits
   sans cycle). Verdict attendu : **CONDITIONAL** (les P0/P1 du bloc
   off-sprint = G2/G7/G9/G12/B-1/B-2 sont precisement le scope des
   phases A-D de ce sprint, donc reconcilies in-sprint, pas en
   `fix(sprint70)` prealable).
2. **Reconciliation** : les phases S71 produisent les artefacts
   process manquants du bloc off-sprint — retro-review (dimensions
   §4.5), retro-Codex (exec brut), retro-audit, + la couverture de
   tests manquante (G6). Documentes dans `.planning/active/`.
3. **Audit gate S71 de sortie** : valide a la fois la reconciliation
   du bloc off-sprint ET les nouvelles phases. C'est le
   `sprint71_audit_plan.md` ecrit en Phase E qui pilotera la session
   fraiche S72.

**Pourquoi ne PAS rejouer l'audit S70 d'abord puis empiler S71 ?**
Parce que le bloc off-sprint et les corrections S71 touchent les
**memes fichiers** (`operator_server.rs`, `llm_bridge.rs`,
`validator.rs`). Auditer puis re-corriger en deux passes dupliquerait
le travail. L'audit-absorb fusionne les deux : on absorbe la dette
en entree, on la corrige dans les phases, on valide a la sortie.

---

## 4. Goal en une phrase

> Sprint 71 **assainit la couche compute** (B-1 routage dispatch reel,
> B-2 quorum deterministe greedy seed-fixe, B-3 premier E2E
> cross-process coordinator→worker→Ollama→validation), **durcit la
> securite Factory** (G2 gating SSE/bypassPermissions, G7 CORS+token,
> G9 modele `opus-4-8`, G12 timeout+diagnostic), et **reconcilie le
> bloc de ~14 commits off-sprint** (retro-review + retro-Codex +
> retro-audit + tests), debloquant l'arc S72-S76 (Factory front-end de
> tout SBFB). **Critere SMART : 100% des rows fail-fast verts au
> `sprint71_verification.md §Fail-fast checklist`, mesure binaire au
> Phase E wrap-up.** La verification.md fail-fast checklist (24-32
> rows executables, cf. plan §5) est la source of truth mesurable.

---

## 5. Decisions Day 0 (D1..D8 gelees)

### D1 — Cle de dispatch unique : prefixe `task:` (B-1)

**Retenu** : aligner le dispatcher sur le lecteur worker. Le worker
lit `get_many_by_prefix(b"task:")` (`runtime.rs:833,845`) avec
`strip_prefix("task:")`. Le dispatch loop ecrit aujourd'hui
`format!("tasks/{}", task_id)` (`dispatch_loop.rs:35`). On change le
**dispatcher** pour ecrire la cle `format!("task:{}", task_id)` afin
qu'aucune autre lecture worker (rodee, testee) ne casse. La cle
devient `task:{task_id}`, alignee bout en bout.

**Rejete** :
- *Changer le worker pour lire `tasks/`* : touche le chemin worker
  le plus rode (claim, result, completed_task_ids cache) — surface
  de regression bien plus large que le seul writer. Rejete.
- *Supporter les deux prefixes (lecture tolerante)* : ajoute du code
  mort permanent (le writer n'ecrira jamais `tasks/` apres le fix) et
  masque le bug au lieu de le fermer. Anti-band-aid (§6.3). Rejete.

**Implications code** : `dispatch_loop.rs:35` (1 ligne), + un test
qui prouve qu'une cle ecrite par le dispatcher est lue par le scan
worker (round-trip de cle, pas in-process injection).

### D2 — Quorum deterministe : greedy seed-fixe, pas tolerance floue (B-2, PO-11)

**Retenu** : pour rendre le quorum hash-exact utilisable, on impose
le **determinisme a la source** plutot que d'assouplir la comparaison.
Les taches verifiables soumettent une requete d'inference greedy
(`temperature=0`, seed fixe) au backend Ollama/llama.cpp. Deux
workers honnetes produisent alors le meme `result_text`, donc le meme
hash, et le quorum `validate_quorum` (`validator.rs:84-145`, compte
exact de `r.sha256` = hash de `result_text`) accepte. PO-11 acte :
greedy seed-fixe pour taches verifiables ; logprobs/watermark reste
**V2** (aujourd'hui inerte : `logprobs_hash=[0u8;32]`,
`model_digest` non discriminant — `validator.rs:234-235`).

**Rejete** :
- *Comparaison floue (edit distance / embedding similarity)* :
  introduit un seuil arbitraire non-deterministe, ouvre une surface
  d'attaque (un worker malveillant vise « assez proche »), et n'est
  pas reproductible cross-machine. Rejete (cf. roadmap §5 « validation
  stochastique non resolue »).
- *logprobs/watermark maintenant* : la machinerie est inerte et son
  design (verification des logprobs cross-backend) est un chantier
  R&D a part entiere. Hors scope consolidation. Differe V2.

**Implications code** : chemin de soumission worker→backend (forcer
greedy+seed pour les taches `verifiable`), + un champ/flag de tache
indiquant le mode deterministe, + test E2E B-3 qui prouve deux
workers → meme hash → quorum atteint. Le `validator.rs` lui-meme
n'a pas besoin de changer (le compte exact devient correct une fois
les sorties deterministes).

### D3 — Securite Factory : gater le SSE, pas desactiver le pilotage (G2, PO-2)

**Retenu** : le pilotage agent embarque (terminal PTY + chat SSE)
est **garde et gate**, pas supprime (PO-2). Aujourd'hui
`handle_chat_message`/`handle_chat_send` verifient bien
`SENSITIVE_ACTIONS` (`operator_server.rs:606,687`) MAIS le SSE
`handle_chat_stream` (`operator_server.rs:735-796`) **court-circuite**
ce filtre : il appelle directement `spawn_claude_stream` avec
`--permission-mode bypassPermissions` (`llm_bridge.rs:80`). On
applique le meme filtre SENSITIVE_ACTIONS au SSE : si le dernier
message user contient une action sensible (`shell`/`commit`/`push`/
`PASS`), le stream renvoie `requires_gate` au lieu de spawner un
agent autonome. `bypassPermissions` reste mais derriere le gate +
confirmation, jamais sur un chemin non filtre.

**Rejete** :
- *Retirer `bypassPermissions` entierement* : casse le mode « prompt
  de base + discussion agent autonome » que le contrat preserve
  (RRV_FACTORY_CONTRACT §4 UX). Le PO l'a explicitement choisi garde
  (PO-2). Rejete.
- *Filtre cote front uniquement* : le serveur ecrit des fichiers et
  spawn des process — un filtre front est contournable par appel API
  direct (CORS Any, §D5). La defense doit etre serveur. Rejete.

**Implications code** : `operator_server.rs:735-796` (appliquer le
filtre avant `spawn_claude_stream`), amendement
`RRV_FACTORY_CONTRACT.md §4` (autoriser explicitement le pilotage
agent local privilegie **gate**).

### D4 — Modele Operator : `claude-opus-4-8[1m]`, pas `sonnet` (G9)

**Retenu** : le modele est hardcode `"sonnet"` dans le SSE
(`operator_server.rs:776`) — viole la regle modele (memory
`feedback_model_46.md` : toujours `claude-opus-4-8[1m]`, jamais
d'alias). On cable `ChatSendRequest.model` (deja porte mais ignore,
`operator_server.rs:665-666`) avec defaut `claude-opus-4-8[1m]`. Les
stubs `handle_chat_message` (« Agent integration pending »,
`operator_server.rs:641`) et `handle_chat_send` (log only) sont
clarifies : soit cables sur le vrai chemin SSE, soit documentes
comme endpoints legacy a retirer.

**Rejete** :
- *Garder `sonnet` « ca marche »* : viole une regle gelee
  explicite + le modele n'est meme pas l'alias correct. Rejete.
- *Passer le routage provider/model complet maintenant* : c'est le
  scope **S72** (ProviderRouter). S71 cable seulement le defaut
  correct + le passage de `model`, pas le router multi-provider.
  Differe S72.

**Implications code** : `operator_server.rs:776` (lire `req.model`
avec defaut), `default_provider`/defaut model, clarification des deux
stubs.

### D5 — Auth serveur Operator : token + Host guard, CORS restreint (G7)

**Retenu** : le serveur Operator expose CORS `Any` sans auth
(`operator_server.rs:87-90`) alors qu'il **ecrit des fichiers et
spawn des process**. On applique le pattern deja correct du
`daemon_client` (`daemon_client.rs:64-65` : header `X-SBFB-Token` +
`Host: 127.0.0.1`). Le serveur exige un token local (genere au
demarrage, transmis a l'UID front) et restreint CORS a l'origine
locale connue. Loopback-only, pattern loopback durci deja en place
cote daemon.

**Rejete** :
- *CORS `Any` « c'est local »* : un site web tiers ouvert dans le
  navigateur peut POST `/api/actions/run` ou `/api/chat/.../stream`
  (DNS rebinding / CSRF) et faire spawner un agent. Inacceptable pour
  un serveur qui ecrit/spawn. Rejete.
- *Auth OS (UDS peer creds)* : le serveur est HTTP TCP loopback, pas
  UDS ; aligner sur le pattern token+Host deja eprouve cote daemon
  evite d'inventer un second mecanisme. Differe (le daemon utilise
  deja UDS/NP peer creds ; l'Operator reste sur token+Host pour
  coherence avec `daemon_client`).

**Implications code** : `operator_server.rs:80-107` (CorsLayer
restreint + middleware token + Host guard), generation/transmission
du token au front.

### D6 — Spawn subprocess : timeout + diagnostic `claude` resolu (G12)

**Retenu** : `spawn_claude_stream` (`llm_bridge.rs:64-118`) spawn
sans timeout et resout `claude.cmd`/`claude` via PATH sans verifier
sa presence ni diagnostiquer l'absence. On ajoute (1) un timeout
configurable sur le subprocess (kill si depasse), (2) une
verification de resolution de l'executable avec un message
diagnostic clair (« claude CLI introuvable dans le PATH ») au lieu
d'un `Failed to spawn` opaque (`llm_bridge.rs:107`).

**Rejete** :
- *Pas de timeout « l'agent gere »* : un subprocess agent qui hang
  bloque le stream et fuit un process. Un serveur de production doit
  borner. Rejete.
- *which-crate pour resoudre* : ajouter une dep pour ce que
  `Command::new` + un check pre-spawn fait deja suffit. Pas de
  nouvelle dep. Rejete (sauf si le check pre-spawn s'avere non
  portable Windows — a trancher au preflight Phase C).

**Implications code** : `llm_bridge.rs:64-118` (timeout wrapper +
pre-spawn resolution check + diagnostic).

### D7 — WIP terminal : trancher en Phase A, jamais laisser flotter (G1)

**Retenu** : le `stash@{0}` (refactor `.cast`→`.log` incomplet) est
**tranche en Phase A**. Decision par defaut proposee : **jeter le
stash et garder l'asciicast** (`.cast`) qui est l'etat HEAD coherent
(`terminal.rs:27,30,133` ecrivent l'asciicast complet, et le commit
`864b005` a deja livre la persistance asciicast + session list). Le
refactor plaintext etait un demi-travail qui cassait le build ; sa
valeur (logs lisibles) ne justifie pas l'incoherence
lecture/ecriture d'extension qu'il introduit. **Decision finale prise
en Phase A apres lecture du stash** (preflight Phase A) — si le
plaintext est presque fini et coherent, le terminer ; sinon, jeter.

**Rejete** :
- *Laisser le stash en attente* : un refactor incomplet qui flotte
  est de la dette invisible qui repollue le prochain sprint. Le
  contrat de consolidation interdit de laisser flotter (intake §4
  « Ne jamais laisser flotter »). Rejete.
- *Terminer le plaintext sans relire* : decider avant d'avoir lu le
  stash = reflexe, pas decision (§6.7). Le preflight Phase A lit le
  stash d'abord. Decision conditionnee.

**Implications code** : Phase A — soit `git stash drop stash@{0}`
(garde HEAD `.cast`), soit terminer le cablage `PlainTextWriter` +
aligner les 3 sites d'extension (`list_sessions` filtre, serve
endpoint, label UI). Pas d'etat intermediaire.

### D8 — Modules morts : retirer ou cabler, pas laisser dormir (dette)

**Retenu** : trois dettes structurelles du bloc compute sont
tranchees en Phase B :
- `RedundancyDispatcher` (`redundancy.rs`) — **module mort**. Verifier
  qu'aucun chemin vivant ne l'appelle, puis retirer (ou documenter
  DEPRECATED si un appelant futur S75 est nomme).
- `execute_build` (`build_executor.rs:126`) — **jamais appele**.
  Trancher : cabler dans le chemin worker reel, ou retirer si la
  logique build est subsumee ailleurs.
- **Double notion « provider »** : string adaptation-prompt
  (`process.rs:24` : `PROVIDERS = ["claude","codex","gpt","local",
  "human"]`) vs runtime `LlmBackend` (Ollama/llama.cpp). Clarifier
  que ce sont **deux axes distincts** (provider de prompt-adaptation
  vs backend d'execution) et le documenter dans PATTERNS, ou unifier
  si redondant.

**Rejete** :
- *Laisser les modules morts « au cas ou »* : code mort = surface de
  confusion + faux signal de capacite. Le sprint consolidation les
  ferme (§6.2.1). Rejete.
- *Unifier provider/backend de force* : ce sont peut-etre deux
  concepts legitimes (qui adapte le prompt vs qui execute). Decision
  conditionnee a la lecture Phase B : documenter la distinction si
  legitime, unifier seulement si redondant.

**Implications code** : Phase B — `redundancy.rs`,
`build_executor.rs:126`, `process.rs:24`, PATTERNS.md.

### Acknowledged review findings (G1)

Scoring (renseigne par `sprint71_design_review.md`) :
**D1 ✅, D2 ⚠️, D3 ✅, D4 ✅, D5 ⚠️, D6 ✅, D7 ✅, D8 ⚠️.**
Rigor signal G4 satisfait (3 ⚠️ sur 8 — sources factuelles
internes, pas de SOTA externe a sourcer).

- **D2 ⚠️** : greedy seed-fixe assume que le backend Ollama/llama.cpp
  honore reellement un seed fixe et produit un determinisme
  bit-exact cross-machine (GPU non-determinisme float possible).
  Decision : **adjust** — Phase A/B documente la limite « determinisme
  greedy = meme machine/meme backend version garanti ; cross-GPU
  best-effort, fallback redundancy=1 ou meme-modele-meme-quant
  documente ». Le test B-3 tourne sur la machine dev (meme backend),
  pas cross-GPU heterogene — la preuve cross-GPU reelle est S75.
- **D5 ⚠️** : le pattern token+Host loopback protege du CSRF/rebinding
  mais ne protege pas d'un process local malveillant qui lit le token.
  Decision : **adjust** — c'est le meme modele de menace que le daemon
  loopback durci (deja accepte projet-wide) ; documenter que la
  surface « process local hostile » est hors scope (sandbox OS niveau
  noeud, pas niveau serveur HTTP).
- **D8 ⚠️** : trancher `execute_build` (cabler vs retirer) depend de la
  lecture du chemin worker reel ; risque de retirer du code qu'un
  futur S75 (GPU partage) voudrait. Decision : **adjust** — si un
  appelant S75 est nommable, DEPRECATED + entree
  `ROADMAP_COMMITMENTS.md` ; sinon retrait. Decision finale au
  preflight Phase B.

---

## 6. Plan phases outline (Phase 0 + A..E)

Consolidation = phases elargies. Detail ligne-par-ligne dans
`sprint71_plan.md`.

- **Phase 0 — Audit-absorb** (deviation §3 documentee). Ingere
  `201b24d..HEAD`, ecrit `sprint70_audit_findings.md` (audit
  retroactif S70 + bloc off-sprint). Pas un commit feat — produit
  l'artefact d'audit + migre les docs S70 `active/`→`archive/v2.1/`
  (chore separe).

- **Phase A — Compute routing + 1er E2E + decision WIP terminal.**
  Fix B-1 (cle dispatch alignee `task:`). Premier E2E cross-process
  coordinator→worker→Ollama→validation (B-3, inexistant aujourd'hui).
  Decision/execution WIP terminal G1 (D7). **Critere : la 1ere tache
  dispatchee est reellement vue et executee par un worker reel,
  prouve par test.**

- **Phase B — Quorum deterministe + nettoyage compute.** B-2 greedy
  seed-fixe (D2) prouve par le quorum (deux workers honnetes →
  meme hash → accepte). Retirer/clarifier modules morts (D8 :
  RedundancyDispatcher, execute_build, double notion provider). Passer
  les 3 deps off-sprint au preflight G8/S1b CVE (G13). **Critere :
  quorum redundancy>1 accepte sur sortie deterministe, modules morts
  resolus.**

- **Phase C — Securite Factory.** G2 gater SSE/bypassPermissions (D3),
  G9 modele `opus-4-8` (D4), G7 CORS+token (D5), G12 timeout+diagnostic
  (D6). Amendement `RRV_FACTORY_CONTRACT.md §4` (PO-2). **Critere :
  le SSE refuse une action sensible non gardee, le serveur rejette une
  requete sans token, le modele est `opus-4-8`, le spawn timeout.**

- **Phase D — Reconciliation process du bloc off-sprint.** Retro-review
  (dimensions §4.5) + retro-Codex (exec brut) + retro-audit du code
  off-sprint (G5). Couverture de tests des surfaces non-testees (G6 :
  `terminal.rs`, `sprint_history.rs`, `operator_server` unit,
  `process.rs`, spawn LLM/PTY). **Critere : chaque surface off-sprint
  a un review + un test ; G5/G6 fermes.**

- **Phase E — Wrap-up.** `sprint71_verification.md` (fail-fast rempli)
  + `sprint71_audit_plan.md` (pour S72) + PATTERNS.md (Rust + shell) +
  memory update. **Critere : 100% fail-fast verts, 2 docs planning,
  PATTERNS a jour, memory a jour.**

**Reservation dette §6.2.1 Regle 1** : S71 est un sprint de
**consolidation post-arc** — l'integralite du sprint est de la dette /
fix / reconciliation / tests manquants. Aucune feature speculative.
La regle « au moins une phase dette » est satisfaite a 100%.

---

## 7. Items carry/dette

Reclassification explicite des carry-overs (source : CLAUDE.md §Etat
actuel « Carry S71 reconduits » + intake §2).

| ID | Description | Classification | Reports | Action S71 |
|----|-------------|----------------|---------|------------|
| B-1 | Cle dispatch `tasks/{id}` vs `task:` | **scope integre Phase A** | nouveau | Fixe Phase A (D1) |
| B-2 | Quorum hash exact rejette sampling | **scope integre Phase B** | nouveau | Greedy seed-fixe Phase B (D2) |
| B-3 | Zero E2E cross-process compute | **scope integre Phase A** | nouveau | 1er E2E Phase A |
| G2 | SSE court-circuite SENSITIVE_ACTIONS | **scope integre Phase C** | nouveau (off-sprint) | Gate SSE Phase C (D3) |
| G9 | Modele hardcode `sonnet` | **scope integre Phase C** | nouveau (off-sprint) | `opus-4-8` Phase C (D4) |
| G7 | CORS Any + zero auth | **scope integre Phase C** | nouveau (off-sprint) | Token+CORS Phase C (D5) |
| G12 | Spawn sans timeout | **scope integre Phase C** | nouveau (off-sprint) | Timeout Phase C (D6) |
| G5 | Bloc off-sprint non reconcilie | **scope integre Phase D** | nouveau | Retro-review/Codex/audit Phase D |
| G6 | Surfaces off-sprint non testees | **scope integre Phase D** | nouveau | Tests Phase D |
| G1 | WIP terminal `.cast`→`.log` stash | **scope integre Phase A** | nouveau | Decision Phase A (D7) |
| G13 | 3 deps off-sprint au preflight | **scope integre Phase B** | nouveau | Preflight G8/S1b Phase B |
| P2-A-1 | rand blocker upstream | **carry confirme** | 4+ | Exemption externe (blocker amont) |
| P2-AUDIT-2 | pre-release transitives iroh | **carry confirme** | 3+ | Herite pin 0.98 (blocker amont) |
| T-NN+2 | iframe Rust-wasm | **carry confirme** | 3+ | PATTERNS §P34 (differe) |
| P2-F-3 | prompt file coupling | **carry confirme** | 2/3 | Differe S72 (non bloquant) |
| LT-2 | Radicle sortie cap G7 | **carry confirme** | — | Trigger PENDING (tag v1.0 pas pousse) |
| LT-5 | redundancy persistence | **carry confirme** | — | Differe (ex-P2-D-1, S26 reclass) |
| LT-7 | self-hosted build worker quorum E2E | **lie B-3** | — | Partiel via E2E Phase A (worker quorum reste S75) |

**Items a 3 reports sans exemption** : aucun. P2-A-1 et P2-AUDIT-2
sont exemptes par blocker externe (dep amont). T-NN+2 (iframe
Rust-wasm) est exempte (PATTERNS §P34, depend d'un upstream wasm).
P2-F-3 est a 2/3 (non escalade). Les items B-*/G-* sont **integres au
plan**, pas reportes (conformement §6.2.1 Regle 2 : un gap reel
identifie entre dans le plan).

---

## 8. Scope cuts (exhaustif)

Ce que S71 ne fera PAS, et pour quel sprint c'est garde :

1. **ProviderRouter multi-LLM** (trait `ProviderRouter`,
   ClaudeProvider/OllamaProvider/NetworkProvider) → **S72** (quick win).
   S71 cable seulement le defaut modele correct (D4), pas le router.
2. **Chat Factory cable sur routage de taches reseau** → **S72**.
3. **Pont feed-distant → reindex FTS5 a chaud** (fraicheur recherche)
   → **S73**.
4. **Enrichissement `SearchResult`** (repo_url+commit_sha+archive_hash
   +provenance_hash) → **S73**.
5. **Barre de recherche shell cablee** sur `GET /api/daemon/search`
   → **S73**.
6. **Decision SearchManifest** (recherche reseau opt-in propagee) →
   **S73** (selon audit S72).
7. **Commandes `sbfb-factory search/open/fork`** (Factory tire du
   reseau) → **S74**.
8. **Notion de projet cible** distinct du repo nexus (`process::repo_root`
   pointe toujours nexus, G17) → **S74**.
9. **Templates etendus** (react, pyodide) → **S74** (decide au kickoff).
10. **GPU partage volontaire prouve cross-machine** (consent 4 niveaux +
    caps + panneau « offrir ma puissance ») → **S75**. S71 prouve le
    routage 1-tache cross-process, pas le GPU partage.
11. **Quorum redundancy>1 prouve cross-MACHINE reel** (B-3 leve) →
    **S75**. S71 prouve cross-PROCESS sur machine dev (meme backend).
12. **Sharding pipeline « gros modele »** (Petals/Parallax, iroh QUIC
    streams, scheduler latency-aware) → **S76 STRETCH** (peut glisser).
13. **logprobs/watermark verification** (model_digest discriminant,
    logprobs_hash reel) → **V2 compute** (post-S75). S71 = greedy
    seed-fixe uniquement (PO-11).
14. **Dashboard contributeur kudos non-monetaire per-task** → **S75**.
15. **@dev index tree-sitter** → **S71+ post-Gate 1** (pas bloquant,
    pas dans ce sprint).
16. **Packaging produit Factory** (launcher conscient, doc install
    operateur, PO-4) → **S74** (onboarding atelier).

---

## 9. Tracabilite scope

Mapping des items « hors S71 » de l'intake §2 vers leur sprint/phase
de prise en charge :

| Item intake (hors S71) | Sprint cible | Phase |
|------------------------|--------------|-------|
| G3/G4/G8/G10/G17/G18/G19/G23 (atelier, Viewer, contrats UI, socle readonly, UX intentions) | S72-S74 | — |
| G14 (secret_scanner) | S72+ (dette pair) | — |
| G15 (canonical bytes T-NN+3) | S72+ | — |
| G16 (E2E publish) | S72+ | — |
| G20/G21/G22 | S71 dette pair / S72+ | reparti |
| ProviderRouter (roadmap S72) | S72 | A |
| Recherche reseau cablee (roadmap S73) | S73 | — |
| Atelier rouvrir/forker (roadmap S74) | S74 | — |
| GPU partage cross-machine (roadmap S75) | S75 | — |
| Sharding (roadmap S76) | S76 | STRETCH |

Items « NOT » du sprint precedent (S70) repris : N/A — S70 n'a pas
produit de `verification.md §scope cuts respectes` exploitable (le
bloc off-sprint a court-circuite la cloture). L'audit-absorb Phase 0
reconstitue la liste.

---

## 10. Risk register (R1..R8)

| # | Risque | Likelihood | Impact | Mitigation |
|---|--------|------------|--------|------------|
| R1 | Greedy seed-fixe non bit-exact cross-GPU (float non-determinisme) | Moyen | Eleve (B-2 quorum) | D2 ⚠️ : preuve B-3 sur machine dev meme-backend ; cross-GPU best-effort documente, fallback redundancy=1 ou meme-quant ; preuve cross-GPU reelle differee S75 |
| R2 | E2E B-3 flaky (Ollama runtime requis, GPU local) | Moyen | Moyen | Test gate sur disponibilite Ollama (skip propre si absent) + seed fixe pour reproductibilite ; documenter prerequis runtime |
| R3 | Charge S71 trop lourde pour 1 sprint (compute + securite + reconciliation) | Eleve | Eleve | Voir §11 arbitrage. Phases elargies, scinder en S71/S71-bis si l'arbitrage le recommande au checkpoint |
| R4 | Fix B-1 casse un test in-process existant qui injectait `tasks/` | Moyen | Moyen | Grep tous les sites qui lisent/ecrivent la cle avant le fix ; aligner les tests sur `task:` |
| R5 | Gating SSE casse le mode « discussion agent autonome » preserve (PO-2) | Moyen | Moyen | D3 : ne gater que les messages contenant SENSITIVE_ACTIONS, pas tout le stream ; tester le happy-path non-sensible |
| R6 | Token Operator casse le front existant (UI off-sprint suppose CORS Any) | Moyen | Moyen | Transmettre le token au front dans le meme commit ; tester le bootstrap front→serveur |
| R7 | Retrait `execute_build`/`RedundancyDispatcher` supprime du code qu'un futur S75 voulait | Faible | Moyen | D8 ⚠️ : si appelant S75 nommable → DEPRECATED + ROADMAP_COMMITMENTS, sinon retrait ; decision au preflight Phase B |
| R8 | Reconciliation Phase D sous-estimee (14 commits, +5500 lignes a retro-auditer) | Eleve | Eleve | Phase D la plus large ; si depasse, c'est le declencheur du scindage §11 (reconciliation Factory → S71-bis) |

R3 et R8 sont les deux risques qui pilotent l'arbitrage §11.

---

## 11. ARBITRAGE — tient-en-1-sprint vs scinder

**Question** : la charge « assainissement compute (B-1/B-2/B-3 + E2E
cross-process) » + « reconciliation Factory (G5/G6, ~5500 lignes
off-sprint) » + « securite Factory (G2/G7/G9/G12) » est-elle tenable
en UN sprint de consolidation a phases elargies ?

**Evaluation par objectif fonctionnel** (pas par LOC) :

- **Bloc compute (Phases A+B)** : B-1 est un fix d'une ligne + tests ;
  B-2 greedy seed-fixe touche le chemin de soumission worker + un flag
  de tache + un E2E ; B-3 est le premier E2E cross-process. C'est un
  objectif fonctionnel **coherent et borne** : « une tache route et
  s'execute reellement, le quorum accepte deux sorties deterministes ».
  Tenable en 2 phases.
- **Bloc securite (Phase C)** : G2/G7/G9/G12 sont 4 corrections
  localisees dans 2 fichiers (`operator_server.rs`, `llm_bridge.rs`)
  + un amendement contrat. Objectif borne. Tenable en 1 phase.
- **Bloc reconciliation (Phase D)** : c'est le **point d'incertitude**.
  Retro-review (11 dimensions) + retro-Codex + retro-audit de ~14
  commits (+5500 lignes) + ecrire la couverture de tests de 5 surfaces
  a 0 test (`terminal.rs`, `sprint_history.rs` 1047 lignes,
  `operator_server` unit, `process.rs`, spawn LLM/PTY). C'est le plus
  gros morceau, et il est intrinsequement long (auditer du code qu'on
  n'a pas ecrit + le tester).

**Recommandation** : **tenter en 1 sprint, avec un point de bascule
explicite.** Les Phases A-B-C sont l'assainissement minimal qui
**debloque tout l'arc** (sans B-1 corrige, S72-S76 n'ont pas de socle
compute). Elles sont prioritaires et bornees. La Phase D
(reconciliation) est la variable d'ajustement :

- Si Phases A-B-C livrent dans le budget de session et que la Phase D
  reconciliation tient, S71 reste mono-sprint.
- **Point de bascule** : si la Phase D depasse (R8 se realise — la
  retro-audit + tests de 5500 lignes off-sprint excede ce qu'une
  session de consolidation peut absorber), **scinder** : S71 ferme sur
  assainissement compute + securite (A-B-C) avec une reconciliation
  **partielle** (retro-review + retro-Codex faits, tests prioritaires),
  et **S71-bis (ou S72 absorbe)** finit la couverture de tests +
  retro-audit complet du bloc off-sprint. L'arc S72-S76 decale d'un
  cran si necessaire.

**Pourquoi ne pas scinder d'emblee ?** Parce que A-B-C + une
reconciliation au moins partielle est exactement le « assainir avant
d'empiler » qu'exige la verite d'ingenierie (roadmap §2). Scinder
d'emblee risque de livrer le compute sans jamais boucler la
reconciliation Factory (la dette off-sprint resterait ouverte). Mieux
vaut viser le tout, avec un fallback honnete documente.

**Question PO ouverte (checkpoint §12)** : valider le point de bascule
R8 — si la Phase D deborde, le PO accepte-t-il (a) une reconciliation
partielle en S71 + completion S71-bis/S72, ou (b) prefere-t-il scinder
d'emblee S71 (compute+securite) / S71-bis (reconciliation Factory) et
decaler l'arc d'un cran ?

---

## 12. Audit gate pattern — rappel

- **Phase 0** : deviation documentee (§3) — audit-absorb du bloc
  off-sprint, produit `sprint70_audit_findings.md` retroactif (S70 +
  off-sprint). Validee PO-3 2026-05-30.
- **Phase de sortie (E)** : produit les deux livrables obligatoires
  dans un commit `docs(sprint71)` :
  `sprint71_verification.md` (self-report fail-fast rempli) +
  `sprint71_audit_plan.md` (feuille de route pour la session fraiche
  S72). Sans ces deux fichiers, le sprint ne peut pas etre ferme (§3.3).
- L'audit gate S71 de sortie valide **a la fois** la reconciliation du
  bloc off-sprint ET les nouvelles phases (§3).

---

## 13. Checkpoint de validation (allege — PO deja consulte)

Le PO a deja acte PO-1..PO-14 cette session (roadmap v5 §1). Le
checkpoint est donc allege : une seule question reellement ouverte +
confirmations rapides.

1. **[OUVERTE — arbitrage §11]** Point de bascule R8 : si la Phase D
   (reconciliation) deborde, (a) reconciliation partielle S71 +
   completion S72/S71-bis, ou (b) scindage d'emblee S71 compute+securite
   / S71-bis reconciliation avec decalage d'arc ? **Reponse par defaut
   proposee : (a).**
2. **[CONFIRME PO-11]** D2 greedy seed-fixe pour B-2, logprobs/watermark
   en V2 — OK ?
3. **[CONFIRME PO-2]** D3 garder+gater le SSE (pas retirer
   bypassPermissions), amender le contrat §4 — OK ?
4. **[CONFIRME PO-3]** Phase 0 audit-absorb (pas de rejeu audit S70 sur
   tip propre) — OK ?
5. **[A trancher Phase A — D7]** WIP terminal : defaut « jeter le stash,
   garder asciicast `.cast` » sauf si le preflight Phase A montre un
   plaintext presque fini et coherent — OK pour deleguer a la Phase A ?
