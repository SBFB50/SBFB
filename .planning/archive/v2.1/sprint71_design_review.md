# Sprint 71 — Design Review (scoring Day 0 D1..D8)

**Role** : reviewer independant pre-gel (G1, README §6.1.1).
**Mandat** : scorer chaque decision Day 0 sur la qualite de ses
sources et la presence d'alternatives comparees. **Pas de proposition
de solution** — signaler les angles morts seulement. Le planner reste
owner.
**Nature du sprint** : consolidation (Arc 3.5). Les decisions reposent
sur de la lecture-de-code factuelle (file:line), pas sur du SOTA
externe. Le scoring evalue donc la solidite du constat + la comparaison
des alternatives internes, pas la fraicheur de sources academiques.

**Convention de scoring** :
- ✅ constat factuel verifie (file:line) + au moins 1 alternative
  reellement comparee avec raison de rejet.
- ⚠️ constat present mais une hypothese non verifiee OU une alternative
  pas entierement comparee.
- ❌ pas de constat verifiable OU choix contredit par le code present.

---

## Scoring synthese

| D | Titre | Score |
|---|-------|-------|
| D1 | Cle dispatch unique `task:` (B-1) | ✅ |
| D2 | Quorum greedy seed-fixe (B-2, PO-11) | ⚠️ |
| D3 | Gater le SSE, garder bypassPermissions (G2) | ✅ |
| D4 | Modele `claude-opus-4-8[1m]` (G9) | ✅ |
| D5 | Token + Host guard + CORS restreint (G7) | ⚠️ |
| D6 | Timeout subprocess + diagnostic claude (G12) | ✅ |
| D7 | WIP terminal tranche Phase A (G1) | ✅ |
| D8 | Modules morts retires/clarifies (dette) | ⚠️ |

**Rigor signal G4** : 3 ⚠️ sur 8. Au-dessus du seuil minimal (au moins
1 ⚠️). Les 3 ⚠️ portent sur des hypotheses runtime (determinisme GPU,
modele de menace process-local, retrait de code potentiellement
futur), pas sur des constats errones — chacune a un adjustment inline.

---

## D1 — Cle dispatch unique `task:` ✅

**Constat verifie** : `dispatch_loop.rs:35` ecrit `format!("tasks/{}",
entry.task.task_id)` ; `runtime.rs:833` scanne
`get_many_by_prefix(b"task:")` et `runtime.rs:845` fait
`strip_prefix("task:")`. Le decalage `tasks/` vs `task:` est reel et
verifie a la ligne pres — aucune tache dispatchee par ce chemin n'est
vue par le worker.

**Alternatives comparees** : (a) changer le worker pour lire `tasks/`
(rejet : surface de regression plus large — claim/result/cache) ; (b)
lecture tolerante deux prefixes (rejet : code mort permanent +
band-aid). Choix : aligner le writer sur le reader. Bien argumente.

**Angle mort** : aucun bloquant. Verifier au fix qu'aucun **test
existant** n'injecte deja directement la cle `tasks/` (couvert par R4).

**Verdict** : ✅ constat exact, alternatives reelles, choix minimal.

---

## D2 — Quorum greedy seed-fixe ⚠️

**Constat verifie** : `validator.rs:68` passe `entry.payload.result_text`
a `validate_quorum`, qui compte `r.sha256` exact (`validator.rs:115`) ;
`logprobs_hash`/`model_digest` sont inertes (`validator.rs:234-235`,
`[0u8;32]`). La comparaison hash-exact sur sortie stochastique rejette
des workers honnetes. Constat solide. PO-11 (greedy seed-fixe) est une
decision PO actee, pas inventee ici.

**Alternatives comparees** : (a) comparaison floue edit-distance/
embedding (rejet : seuil arbitraire, surface d'attaque, non
reproductible) ; (b) logprobs/watermark maintenant (rejet : inerte +
chantier R&D). Bonne comparaison.

**Angle mort (⚠️)** : la decision **assume que le backend honore un
seed fixe et produit un determinisme bit-exact**. C'est vrai cote CPU
mais **le non-determinisme float GPU** (ordre de reduction, kernels)
peut casser le bit-exact cross-machine, voire cross-run sur le meme
GPU selon le runtime. Cette hypothese n'est pas verifiee contre la doc
Ollama/llama.cpp dans le draft.

**Adjustment inline (acknowledged au kickoff §5 D2)** : le preflight
Phase A/B verifie l'hypothese seed/determinisme contre la doc backend
(context7 si dispo). La preuve B-3 tourne sur **machine dev,
meme-backend** (determinisme garanti) ; le determinisme **cross-GPU
heterogene** est documente comme best-effort et **differe a S75**
(preuve cross-machine reelle). La limite est ecrite dans PATTERNS. Le
quorum reste correct par construction une fois les sorties
deterministes — c'est l'hypothese de determinisme qui est conditionnee,
pas la logique de quorum.

**Verdict** : ⚠️ — decision saine, hypothese determinisme a verifier au
preflight + limite cross-GPU a documenter. Adjustment suffisant.

---

## D3 — Gater le SSE, garder bypassPermissions ✅

**Constat verifie** : `handle_chat_message` (`operator_server.rs:606`)
et `handle_chat_send` (l.687) verifient bien `SENSITIVE_ACTIONS`, MAIS
`handle_chat_stream` (l.735-796) appelle directement
`llm_bridge::spawn_claude_stream` (l.776) sans aucun filtre, et
`spawn_claude_stream` lance `--permission-mode bypassPermissions`
(`llm_bridge.rs:80`). Le SSE est bien le **seul chemin non garde** —
constat exact et precis. Le `handle_action_run` est par ailleurs deja
gate par `ACTION_ALLOWLIST` (l.339), ce qui renforce le constat : seul
le SSE chat manque le filtre.

**Alternatives comparees** : (a) retirer bypassPermissions (rejet :
casse le mode autonome que PO-2 + contrat §4 preservent) ; (b) filtre
front uniquement (rejet : contournable par appel API direct vu CORS
Any). Bonne comparaison, coherente avec PO-2.

**Angle mort** : le filtre `SENSITIVE_ACTIONS` est un match
substring case-insensitive (`lower.contains`) — un message peut
contenir « shell » au sens benin (« shell script tutorial ») et etre
gate a tort (faux positif), ou un message peut exprimer une intention
sensible sans le mot-cle (faux negatif). C'est une limite du
mecanisme existant, pas de la decision D3 (qui se contente d'etendre
le filtre existant au SSE). A noter pour S72+ (gating semantique), pas
bloquant ici.

**Verdict** : ✅ constat exact, scope minimal coherent PO-2. La
faiblesse du filtre substring est pre-existante, hors scope D3.

---

## D4 — Modele `claude-opus-4-8[1m]` ✅

**Constat verifie** : `operator_server.rs:776` passe `"sonnet"` litteral
a `spawn_claude_stream` ; `ChatSendRequest.model` existe (l.665-666,
`#[serde(default)]`) mais est ignore par le SSE. La regle modele
(memory `feedback_model_46.md` : toujours `claude-opus-4-8[1m]`, jamais
d'alias) est gelee et violee. Constat exact.

**Alternatives comparees** : (a) garder sonnet (rejet : viole regle
gelee + alias incorrect) ; (b) router provider/model complet maintenant
(rejet : scope S72). Comparaison nette, frontiere S71/S72 claire.

**Angle mort** : aucun. Verifier que `claude-opus-4-8[1m]` est bien
l'ID accepte par le CLI `claude --model` (vs un alias) — le CLI peut
attendre un format different. Trivial a valider au fix (test 3 Phase C).

**Verdict** : ✅ constat exact, scope borne, regle gelee respectee.

---

## D5 — Token + Host guard + CORS restreint ⚠️

**Constat verifie** : `operator_server.rs:87-90` construit `CorsLayer`
avec `allow_origin(Any).allow_methods(Any).allow_headers(Any)`, sans
middleware d'auth, sur un serveur qui ecrit des fichiers
(`handle_artifact_draft`, `handle_context_pack`) et spawn des process
(SSE). Le pattern correct existe : `daemon_client.rs:64-65`
(`X-SBFB-Token` + `Host: 127.0.0.1`). Constat exact, pattern de
reference reel.

**Alternatives comparees** : (a) garder CORS Any « c'est local »
(rejet : DNS rebinding/CSRF depuis un site tiers ouvert dans le
navigateur → spawn agent) ; (b) auth OS UDS peer creds (differe :
serveur TCP loopback, pas UDS ; coherence avec daemon_client). Bonne
comparaison, menace CSRF/rebinding correctement identifiee.

**Angle mort (⚠️)** : le token+Host loopback protege du CSRF/rebinding
mais **ne protege pas d'un process local malveillant** qui peut lire le
token (variable d'env, fichier, argv). La decision ne dit pas
explicitement que cette surface est hors scope.

**Adjustment inline (acknowledged kickoff §5 D5)** : c'est **le meme
modele de menace que le daemon loopback durci**, deja accepte
projet-wide (le THREAT_MODEL traite « process local hostile » au
niveau sandbox OS du noeud, pas au niveau serveur HTTP). Documenter
explicitement cette frontiere dans le commit/PATTERNS : l'Operator
defend du reseau (CSRF/rebinding/cross-origin), pas du process local
hostile (qui est couvert ailleurs ou hors menace pilote ferme).
Adjustment suffisant.

**Verdict** : ⚠️ — decision correcte, frontiere de menace
« process local » a expliciter. Coherent avec le modele loopback
existant.

---

## D6 — Timeout subprocess + diagnostic claude ✅

**Constat verifie** : `spawn_claude_stream` (`llm_bridge.rs:64-118`)
spawn `claude.cmd`/`claude` (l.74) sans timeout et ne gere l'absence
que par un `Failed to spawn claude: {e}` opaque (l.107). Constat exact.

**Alternatives comparees** : (a) pas de timeout « l'agent gere » (rejet :
hang bloque le stream + fuit le process) ; (b) crate `which` pour
resoudre (rejet : dep inutile, check pre-spawn suffit — conditionne a
la portabilite Windows). Comparaison correcte, parcimonie de deps
respectee.

**Angle mort** : sur Windows, la resolution `claude.cmd` via PATH +
`Command::new` a des subtilites (shim `.cmd` vs `.exe`, `PATHEXT`). Le
check pre-spawn doit etre teste sur Windows (le projet est Win 11). Le
plan le note (« a trancher au preflight Phase C si non portable »).

**Verdict** : ✅ constat exact, parcimonie deps, portabilite Windows
notee.

---

## D7 — WIP terminal tranche Phase A ✅

**Constat verifie** : `stash@{0}` « WIP terminal plaintext-logging
refactor (incomplete) -- S71 Factory » existe (verifie via
`git stash list`). Le HEAD ecrit l'asciicast `.cast`
(`terminal.rs:27,30,133`) de maniere coherente ; le commit `864b005` a
deja livre la persistance asciicast + session list. Le refactor
plaintext est un demi-travail qui cassait le build. Constat exact.

**Alternatives comparees** : (a) laisser le stash flotter (rejet : dette
invisible, interdit par le contrat consolidation) ; (b) terminer le
plaintext sans relire (rejet : decider avant lecture = reflexe).
Decision **conditionnee a la lecture du stash au preflight Phase A**,
defaut « jeter + garder asciicast ». Discipline §6.7 respectee
(decision, pas reflexe).

**Angle mort** : aucun bloquant. La decision est explicitement
conditionnee — le preflight Phase A tranche pour de bon. Bon pattern.

**Verdict** : ✅ constat exact, decision conditionnee correctement,
interdiction de flotter respectee.

---

## D8 — Modules morts retires/clarifies ⚠️

**Constat verifie** : `RedundancyDispatcher` existe
(`redundancy.rs`) ; `execute_build` est defini
(`build_executor.rs:126`) avec une logique reelle (clone→build→sha256)
mais l'intake affirme qu'il n'est jamais appele ; la double notion
provider existe (`process.rs:24` `PROVIDERS = ["claude","codex","gpt",
"local","human"]` vs runtime `LlmBackend`). Les trois constats sont
plausibles et localises.

**Alternatives comparees** : (a) laisser les modules morts « au cas ou »
(rejet : confusion + faux signal) ; (b) unifier provider/backend de
force (rejet : peut-etre deux concepts legitimes — adaptation prompt
vs execution). Comparaison correcte.

**Angle mort (⚠️)** : (1) l'affirmation « `execute_build` jamais
appele » n'est pas prouvee par grep dans le draft — un appelant
indirect (dynamique, via trait object, ou dans un binaire worker)
pourrait exister. (2) Retirer `execute_build`/`RedundancyDispatcher`
risque de supprimer du code qu'un futur **S75 (GPU partage / build
distribue)** voudrait reutiliser — `execute_build` est precisement la
brique « build reproductible » qui sert la verification cross-machine.

**Adjustment inline (acknowledged kickoff §5 D8 + R7)** : le **preflight
Phase B fait le grep d'appelants** avant tout retrait (prouve le
« jamais appele »). Si un appelant S75 est **nommable**, l'item devient
DEPRECATED documente + entree `ROADMAP_COMMITMENTS.md` (les 7 champs),
pas un retrait. Sinon retrait. La distinction provider/backend est
**documentee dans PATTERNS si legitime**, unifiee seulement si
prouvee redondante. Decision finale conditionnee au preflight, pas au
gel. Adjustment suffisant.

**Verdict** : ⚠️ — constats plausibles mais « jamais appele » a prouver
au grep + risque de retrait de code futur-utile. Adjustment (grep +
DEPRECATED conditionnel) suffisant.

---

## Notes transverses

1. **Checklist `[DETER]` crypto/spec** : non applicable — aucune
   primitive crypto/spec nouvelle (D2 greedy seed-fixe est une config
   d'inference, pas une primitive). Le quorum reutilise le hash exact
   existant.
2. **Checklist Rust-first** : non applicable — pas de choix de lib
   runtime nouveau. D6 evite meme une dep (`which`) par parcimonie.
3. **Estimation LOC** : absente du kickoff et du plan, conforme §6.7.
   Dimensionnement par objectif fonctionnel.
4. **Goal §2 → verification.md** : le goal pointe explicitement vers
   `verification.md §Fail-fast checklist` (34 rows) comme critere SMART
   mesurable. Conforme G3.
5. **Arbitrage §11 (mono-sprint vs scindage)** : le kickoff tranche
   honnetement (tenter 1 sprint, point de bascule R8 documente, question
   PO ouverte au checkpoint). C'est un signal de rigueur — la charge
   est evaluee par objectif fonctionnel, pas minimisee.

**Rigor signal global** : SATISFAIT. 3 ⚠️ sur 8, chacune avec
adjustment inline acknowledged au kickoff §5. Les constats sont
verifies file:line (pas de circularite type S19 PoW Hashcash). Les
alternatives sont reellement comparees (au moins 2 par decision). Le
planner reste owner ; aucun veto reviewer.
