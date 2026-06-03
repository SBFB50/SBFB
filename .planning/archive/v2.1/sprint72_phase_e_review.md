# Phase Review — Sprint 72 Phase E (front UX intentions, provider-routed execution chat)

## Verdict: PASS

Promu `PASS` apres reconciliation Codex (cf. §Codex reconciliation). Le
rapport brut `sprint72_phase_e_codex_review.md` est present et stage :
5 livrables CONFIRME, 0 GAP, 1 PARTIEL ferme par addition (cles i18n
`networkStatus.{rejected,timed_out}`).

(Rigor signal : **2 findings P2** documentes (carry-overs S73/S74). Le P1
i18n et le P3 champ-mort identifies au 1er passage ont ete **CORRIGES et
re-verifies moi-meme** — voir §Findings + §Correctifs verifies. >=1 P2+
requis pour un audit rigoureux : satisfait.)

**Mise a jour (2e passage)** : l'executeur a applique les correctifs P1 +
P3. Re-verification independante effectuee — les deux fixes sont CONFIRMES
(diff lu, cles re-auditees exhaustivement, gates re-verts).
**Mise a jour (3e passage, post-Codex)** : Codex (gpt-5.5, xhigh) a rendu
5 CONFIRME / 0 GAP / 1 PARTIEL ; le PARTIEL (cles
`networkStatus.{rejected,timed_out}` absentes) a ete ferme par addition
(FR+EN), gates re-verts. Verdict promu `PASS`.

Reviewer INDEPENDANT (n'a pas ecrit le code). Diff Phase E front-only :
2 fichiers NEW (`src/lib/executionChat.ts`, `src/pages/ExecutionChat.tsx`) +
4 modifies (`App.tsx`, `components/Sidebar.tsx`, `i18n/locales/{fr,en}.json`).

---

## Staging check (Step 1bis)
- Phase fichiers : 6 — NEW `tools/factory-operator/src/lib/executionChat.ts`,
  NEW `tools/factory-operator/src/pages/ExecutionChat.tsx`, M
  `.../src/App.tsx`, M `.../src/components/Sidebar.tsx`, M
  `.../src/i18n/locales/fr.json`, M `.../src/i18n/locales/en.json`.
- Planning split : `sprint72_phase_e_preflight.md` est untracked (artefact
  preflight) + ce `sprint72_phase_e_review.md`. Doivent etre committes dans
  le commit phase OU un `chore(planning)`. N/A bloquant (artefacts process,
  pattern habituel : stage avec la phase). Aucun mix code/planning illegal.
- Untracked accidentels : 0 (pas de node_modules, .pdb, cache, build).
- Coherence `mod`/fichier : N/A (TS, pas Rust). Imports resolus : `App.tsx`
  importe `ExecutionChat` (NEW), `Sidebar.tsx` ajoute l'entree nav, les
  cles i18n `execute.*` + `nav.execute` sont ajoutees. tsc=0 le confirme.

## Correctifs verifies (2e passage — re-verification independante)
- **P1 RESOLU** : `git diff -- .../i18n/locales/{fr,en}.json` lu moi-meme.
  La cle `sessionError` est desormais presente dans le bloc `execute` des
  DEUX fichiers (fr.json:211 « Erreur lors de la création de la session. »,
  en.json même position « Error creating session. »). Re-audit EXHAUSTIF de
  toutes les cles `t("execute.*")` de `ExecutionChat.tsx` (grep
  `execute\.[a-zA-Z_.]+`) : `title, description, targetLabel,
  conversationHint, empty, placeholder, send, thinking, gateRequired,
  streamError, connectionLost, sessionError, networkInProgress` toutes
  presentes dans le bloc `execute` ; les prefixes dynamiques `intent.*`,
  `intentDesc.*`, `networkStatus.*` tous renseignes (+ `defaultValue` de
  secours sur `networkStatus`). **Aucun chemin de cle-litterale ne subsiste.**
- **P3 RESOLU** : `Streaming.thinking` SUPPRIME de l'interface
  (ExecutionChat.tsx:50-53, ne reste que `text` + `networkStatus`) ; le
  `case "delta"` n'ecrit plus `thinking` (:170-172) ; `case "thinking":`
  est un no-op documente (:174-177, « reasoning chunks not surfaced; the
  empty-state spinner covers the pre-first-token wait »). Plus de champ
  d'etat mort. Le `t("execute.thinking")` du spinner empty-state (:323)
  reste legitime (cle presente). Verifie par lecture.

## Suites (re-verifiees moi-meme, pas sur parole — 2e passage)
- factory-operator `npx tsc -b --noEmit` : **exit 0** (re-run apres fixes).
- factory-operator `npx eslint src/pages/ExecutionChat.tsx
  src/lib/executionChat.ts` : **exit 0, 0 warning** sur les fichiers Phase E.
- factory-operator `npx eslint .` (full) : **exit 0** — 3 warnings, TOUS
  pre-existants (`ui/badge.tsx:52`, `ui/button.tsx:58`, `ui/tabs.tsx:82`,
  react-refresh/only-export-components). **AUCUN warning** des 2 fichiers
  NEW Phase E. Verifie par chemin.
- Vitest : N/A — `tools/factory-operator/package.json` n'a pas de runner
  (scripts `build`=`tsc -b && vite build`, `lint`=`eslint .`). Conforme
  plan §8.3 + preflight.
- scan-en-strings.sh : N/A — le script cible `web/src/` uniquement, pas
  `tools/factory-operator`. Verification FR = revue manuelle (voir
  §Strings FR).
- Rust / size-limit / shell `web/` : non touches par Phase E (front
  Operator standalone). Pas de re-mesure (exemption documentee plan §8.5).

## Commit body validation
- Draft body NON fourni dans la session courante → **CONCERN body-absent**
  (non bloquant pour le verdict review ; le body sera valide Step 4bis au
  moment du commit). Titre cible (plan §8.5) :
  `feat(factory-operator): Sprint 72 Phase E — UX intentions execution (Claude / local / reseau)`.
- Delta tests : Rust 1544 -> 1544 (+0), Vitest 279 -> 279 (+0), size 6/6.
  Coherent : front-only, pas de runner. Le body DOIT citer le preflight
  PLAN-ADAPT (chat-SSE consumer reconstruit, pas greffe).

## Body format validation (Step 4bis, §4.1)
Body redige (`.git/COMMIT_BODY_PHASE_E.txt`) avec les 9 headers `##` EXACTS
(`## Contexte/Fichiers/Delta tests/Verification §7.4/Scope cuts/G8
traceability/Pre-launch protocol/Codex verification/Carry closure`). Header
`## Scope cuts` exact (PAS « respectes ») conforme lightcheck. Section Codex
verification renseignee post-reconciliation (5 CONFIRME / 0 GAP / 1 PARTIEL
ferme). A verifier au staging.

## Modified-file branch coverage (Step 2bis, G9)
Aucun runner front → pas de couverture par test automatise (assume par le
plan §8.3 + preflight : la verification est tsc/eslint + revue manuelle de
la transmission `provider`). La logique critique (auto-reconnect defense,
mapping StreamChunk) est verifiee par lecture adversariale ci-dessous, pas
par un test. C'est une dette structurelle du package (pas de Vitest), pas
une regression Phase E — documentee comme limite (P2-front-no-test).

---

## Points adversariaux (verdict explicite sur chacun)

### 1. Axe EXECUTION correct (D5) — PASS
`createSession`/`sendMessage` envoient `provider: intent` ou `intent ∈
{claude,ollama,network}` (executionChat.ts:53,69) vers `POST
/api/chat/session` et `POST /api/chat/{id}/send`. C'est bien
`ChatSendRequest.provider` (l'axe EXECUTION, `ExecutionTarget::from_provider`
parse `claude`/`ollama`/`local`/`network`), PAS l'axe prompt-adaptation
5-valeurs (`AgentSelector`) ni `/api/prompt?provider=`. Le type
`ExecutionIntent` (3 valeurs) est distinct et documente en tete de fichier.
Conforme D5 + §P55. La page NE touche PAS `AgentChat.tsx` (terminal PTY
preserve) — c'est une page additive `/execute`.

### 2. UX intentions, jamais jargon — PASS
Les CTA sont des INTENTIONS : `execute.intent.{claude,ollama,network}` =
« Exécuter sur Claude / en local / sur le réseau » (fr.json:214-218). Aucun
label visible ne contient `provider`/`ollama`/`network`/`kind`. La string
litterale `provider` n'apparait que dans le body wire (`{ provider: intent }`,
executionChat.ts:54,69) et dans des commentaires de code, jamais dans une
CTA rendue. Note : la valeur technique `ollama`/`network` sert de CLE i18n
(`execute.intent.ollama`) — c'est un identifiant interne, pas un texte
affiche. Conforme UX obligatoire CLAUDE.md.

### 3. Auth EventSource — PASS
`openStream` ouvre `new EventSource('/api/chat/{id}/stream')` — chemin
RELATIF (executionChat.ts:78). `postApi` prefixe `/api${path}` (useApi.ts:58),
aussi relatif. `vite.config.ts:46-52` proxifie `/api` → `http://127.0.0.1:3001`
et injecte `x-sbfb-token` sur chaque `proxyReq`. Grep confirme : **zero**
`http://`, `127.0.0.1`, `localhost`, `:3001` absolu dans les 2 fichiers NEW.
Le browser EventSource ne peut pas poser de header custom — la voie
relative-proxy est la seule auth-correcte. Conforme preflight S3.

### 4. Gate non-bypassable — PASS
Deux chemins gates, tous deux rendus + stream arrete :
- Reponse `send` : `sendMessage` retourne `{ok:false, requires_gate:true}`
  (operator_server.rs:809-813) ; le front lit `res.requires_gate`
  (ExecutionChat.tsx:128), affiche `execute.gateRequired` en bulle `system`,
  `setBusy(false)`, `return` — le stream n'est JAMAIS ouvert.
- Evenement SSE `requires_gate` : `sse_gate` emet le JSON brut
  `{"type":"requires_gate","message":...}` (operator_server.rs:843-849) ;
  le front a la variante `requires_gate` dans son union StreamChunk
  (executionChat.ts:35) et la traite (ExecutionChat.tsx:211-219) : `close()`
  + notice, stream arrete. La securite reste server-side (gate AVANT
  dispatch, operator_server.rs:896-910). Le front ne fait que rendre.
  Detail correct : le backend `StreamChunk` Rust n'a PAS de variante
  `requires_gate` (llm_bridge.rs:42-59) ; `sse_gate` l'emet hors-enum en
  JSON brut, et l'union TS du front est un SUR-ENSEMBLE qui matche la
  realite wire — bonne modelisation.

### 5. EventSource auto-reconnect (PIEGE CRITIQUE) — PASS
La defense est correcte sur TOUS les chemins :
- `close()` (ExecutionChat.tsx:155-159) pose `finished=true`, appelle
  `es.close()` (coupe l'auto-reconnect natif), et null `esRef.current` SEULEMENT
  si `esRef.current === es` (garde anti-ecrasement d'un stream plus recent).
- Chaque chunk terminal (`done` :190, `error` :200, `requires_gate` :212)
  appelle `close()`. Le serveur termine le stream apres `Done` → le browser
  tenterait un reconnect, mais `es.close()` deja appele l'empeche.
- `es.onerror` (:223) : `if (finished) return;` — si un chunk terminal a deja
  ferme, onerror est un no-op (pas de double-notice, pas de re-run). Sinon
  `close()` + `connectionLost`. Empeche le re-run en boucle serveur (le
  PIEGE).
- Cleanup `useEffect` (:102-107) ferme tout stream ouvert a l'unmount
  (StrictMode double-mount dev, `main.tsx` StrictMode). `handleSend` ferme
  aussi tout stale stream avant d'en ouvrir un neuf (:114-115). Pas de fuite.
Verdict : aucun re-run en boucle, aucune fuite EventSource. Le piege est
desamorce proprement.

### 6. PO-14 reseau — PASS
Le bras network backend emet `Debug{label:"network-poll", content:"status: X"}`
par tick (provider_router.rs:409-412), zero Delta, un seul Done (test
`network_provider_submit_poll_yields_single_done` : `dones==1, deltas==0`).
Le front mappe ce Debug en etat « En cours sur le réseau »
(ExecutionChat.tsx:178-187, 306-314) avec un spinner + le statut traduit
(`execute.networkStatus.{pending,dispatched,awaiting_quorum,completed}`),
JAMAIS un curseur de frappe live. `defaultValue: streaming.networkStatus`
(:310-312) protege contre un statut non mappe (pas de cle-litterale ici).
Conforme R5 + PO-14 : pas de fausse promesse live.

### 7. Scope cut model picker — P2 (defendable mais limite UX reelle)
Le front n'envoie PAS de `model` (`createSession` envoie `{provider}` seul ;
`sendMessage` envoie `{message, provider}`). Le backend defaute
`claude-opus-4-8[1m]` (operator_server.rs:311). Consequences par intention :
- `claude` : modele valide → OK.
- `network` : le modele est passe au submit mais le worker reseau choisit
  son backend ; non bloquant.
- `ollama` (« Exécuter en local ») : `GenerationRequest::new("claude-opus-4-8[1m]",
  ..)` → Ollama renvoie « model not found » → `ollama_diagnostic` → un
  `StreamChunk::Error` « Ollama error: model ... » affiche en bulle system.
**Tranche** : c'est un **scope-cut DEFENDABLE** (axe model = D5 separe, hors
Phase E ; preflight l.304-313 + kickoff D5 l'actent). La degradation est
GRACIEUSE (diagnostic clair, pas de crash/hang). MAIS pour un sprint dont le
goal est « UX intentions COMPLETE », la CTA « Exécuter en local » est
non-fonctionnelle out-of-the-box (l'utilisateur doit avoir un modele Ollama
nomme `claude-opus-4-8[1m]`, ce qui n'existe jamais). Classe **P2** (pas P1) :
pas un defaut d'implementation Phase E, mais une limite UX a router en carry
S73/S74 (model picker pour les intentions non-Claude). A documenter
explicitement dans le body « Scope cuts » + sprint73_audit_plan.

### 8. React/TS correctness — PASS (P3 champ-mort RESOLU)
- Pas de fuite EventSource (cf. point 5). Cleanup + esRef garde.
- Pas de stale closure : `accumRef` (mutable ref) accumule les deltas hors
  state ; les `setStreaming`/`setMessages` sont tous fonctionnels
  (`prev => ...`). `handleSend` deps `[input, busy, intent, t]` — correctes
  (toutes les valeurs lues sont la ou des refs/setters stables). Pas de
  warning exhaustive-deps (eslint=0).
- Union discriminee `StreamChunk` narrowee par `switch (chunk.type)` —
  exhaustif sur les 6 variantes, chaque branche accede aux champs valides.
- Pas de `any` explicite. `JSON.parse(ev.data) as StreamChunk` est un cast
  assume (le wire est de confiance, loopback) avec try/catch silencieux sur
  parse invalide (return) — acceptable.
- `key={i}` (index) sur `messages.map` — liste append-only jamais reordonnee
  → benin (pas de bug de reconciliation).
- **P3 RESOLU** (etait : `Streaming.thinking` ecrit jamais lu) : le champ a
  ete supprime de l'interface ; `case "thinking"` est un no-op documente ;
  le spinner empty-state couvre l'attente avant 1er token. Plus de champ mort.

### 9. Strings FR — P1 cle i18n manquante RESOLU
- `lng:"fr"`, `fallbackLng:"fr"` (i18n/index.ts:12-13). Tous les textes
  passent par `t(...)`. Accents corrects dans fr.json (Exécuter, réseau,
  nécessite, privée, démarrer, etc.).
- **DEFAUT** : `ExecutionChat.tsx:142` appelle `t("execute.sessionError")`
  sur le chemin d'echec de `createSession`/`sendMessage` (catch). Or le bloc
  `execute` de fr.json (l.201-230) et en.json **ne definit PAS** de cle
  `sessionError`. (La cle `sessionError` existe seulement sous `chat`,
  fr.json:187 — i18next ne fait PAS de fallback inter-namespace.) Avec la
  config i18next par defaut (pas de `parseMissingKeyHandler`/`saveMissing`),
  `t` retourne la CLE LITTERALE → l'utilisateur voit le texte brut
  `"execute.sessionError"` (anglais technique, non traduit) quand la creation
  de session ou l'envoi echoue (daemon down, 401, etc.). C'est un chemin
  d'erreur reel et atteignable. Viole « strings utilisateur FR » (critere
  §8.4) ET la regle UX (jargon technique visible). Etait **Classe P1**.
- **RESOLU (2e passage)** : la cle `sessionError` a ete ajoutee aux blocs
  `execute` de `fr.json` (« Erreur lors de la création de la session. ») et
  `en.json` (« Error creating session. »). Diff verifie + re-audit exhaustif
  de toutes les cles `execute.*` (cf. §Correctifs verifies) : plus aucun
  chemin de cle-litterale. Root-cause (cle oubliee) corrigee, pas un
  band-aid. tsc+eslint re-verts. **PLUS BLOQUANT.**

---

## Scope cuts verification (kickoff §7 / plan §11)
- #1 packaging/onboarding → S74 : Phase E livre l'ecran de selection
  fonctionnel (in-scope §7 #1 NOTE), AUCUN launcher/installer/doc onboarding.
  Conforme.
- #6 `sbfb-factory search/open/fork` → S74 : 0 fichier touche. Conforme.
- #7 projet cible distinct nexus → S74 : le front NE construit PAS de project
  picker (`createSession` laisse `project_id` au defaut serveur). Conforme —
  evite l'edge vers l'atelier S74 (decision preflight + Phase D review).
- #9/#10 GPU/quorum cross-machine → S75 : non touche. Conforme.
- #12 token-par-token WAN → jamais (PO-14) : respecte (point 6).
- #16 multi-cloud provider/model → hors roadmap : 3 intentions fermees, pas
  de proxy generaliste. Conforme. (Le model picker manquant — point 7 — est
  une absence, pas un ajout hors-scope.)
Aucun fichier ne touche un scope cut. PASS.

## Research grounding (Step 4ter)
- Preflight G8 : `sprint72_phase_e_preflight.md` EXISTE, verdict PLAN-ADAPT,
  5 scans presents (S1a/S1b/S2/S3/S4). S1a nomme >=1 OSS (Open WebUI /
  AnythingLLM + precedent in-repo `e26d9f2`/`c3f4813`). §Plan adaptation
  documente l'evidence (chat-SSE removed `c3f4813`, terminal bypasse
  ExecutionTarget) et l'approche corrigee (build du consumer + selecteur sur
  `/execute`). PASS.
- Le code IMPLEMENTE fidelement le PLAN-ADAPT : nouvelle page `/execute`
  (pas de greffe sur le terminal), selecteur 3-intentions, SSE consumer,
  auth via proxy relatif, `project_id` au defaut. Conforme A→E du preflight.
- Deps : aucune nouvelle dep (S1b clean). EventSource natif. PASS.

## Horizon long-terme + documentation amont
- Design doc : le preflight PLAN-ADAPT tient lieu de design d'execution
  (front glue UI, pas de nouveau module structurant > 1 sprint). Acceptable.
- D5 (3 axes) avec alternatives rejetees : documente kickoff §4 + §P55. PASS.
- Solution la plus poussee : enum-dispatch backend + page additive (preserve
  le terminal PTV) = choix conservateur correct (pas de regression du
  produit existant). PASS.
- Aucune LOC estimee au plan §8 (estimation indicative §10 « pas un
  plafond, pas de LOC §6.7 »). PASS.

## Findings (rigor signal — 2 P2 restants ; 1 P1 + 1 P3 RESOLUS)
- **P1 RESOLU** : `t("execute.sessionError")` referencait une cle i18n
  absente → rendait la cle litterale. Cle ajoutee aux 2 blocs `execute`
  (fr+en), diff verifie, re-audit exhaustif des cles `execute.*` propre.
  tsc+eslint re-verts. Plus bloquant.
- **P3 RESOLU** : champ d'etat mort `Streaming.thinking` supprime ;
  `case "thinking"` no-op documente. Plus de champ mort.
- **P2 (carry)** : intention « Exécuter en local » (Ollama) non-fonctionnelle
  out-of-the-box — le front n'envoie pas de `model`, le backend defaute
  `claude-opus-4-8[1m]` qui n'existe pas dans Ollama → diagnostic Error.
  Degradation gracieuse, scope-cut model-axis defendable (D5), MAIS la CTA
  est inerte sans config Ollama manuelle. Carry-over **S73/S74** (model
  picker pour intentions non-Claude). A documenter au body §Scope cuts +
  `sprint73_audit_plan.md`.
- **P2 (carry)** : `tools/factory-operator` n'a aucun runner de test → la
  logique critique (auto-reconnect defense, mapping SSE, gate-rendering)
  n'est couverte par AUCUN test automatise, seulement par revue manuelle.
  Dette structurelle du package (pas une regression E). Carry : decision PO
  d'ajouter Vitest a l'Operator (infra hors quick win, plan §8.3 le note).

Comptage : **2 findings P2** restants (carry-overs) + 1 P1 RESOLU + 1 P3
RESOLU. >=1 P2+ requis pour un audit rigoureux : satisfait.

## Codex gate (§4.5) — zero exemption
- Status : **DONE** — `sprint72_phase_e_codex_review.md` present et stage
  (output brut `codex exec -o`, non reecrit ; gpt-5.5, reasoning xhigh).
- Resultat : 6 livrables audites — 5 CONFIRME, 0 GAP, 1 PARTIEL.
- Prompt : `.git/CODEX_SPRINT72_PHASE_E.txt`.

## Codex reconciliation
- Rapport Codex lu (brut, non reecrit) : 5 CONFIRME, 0 GAP, 1 PARTIEL.
- PARTIEL (Livrable 5 i18n) : le composant rend
  `execute.networkStatus.${status}` dynamiquement ; le bras Network backend
  (provider_router.rs:409-412) emet un Debug `status: rejected` /
  `status: timed_out` AVANT l'Error terminal, et les cles
  `networkStatus.{rejected,timed_out}` manquaient (fr+en) — un `defaultValue`
  evitait la cle brute mais affichait le mot anglais transitoirement.
  **Ferme par addition** : 2 cles ajoutees a fr.json (« Rejetée » /
  « Expirée ») + en.json (« Rejected » / « Timed out »). Root cause (set de
  statuts incomplet vs provider_router), pas un band-aid.
- Gates re-verts apres l'addition : `tsc -b --noEmit` exit 0, `eslint .`
  exit 0 (3 warnings pre-existants uniquement).
- 0 GAP P0/P1. Les 2 P2 (model-picker non-Claude, Operator sans runner de
  test) restent documentes en carry (body §Scope cuts + sprint73_audit_plan).
- Review promu `## Verdict: PASS`.

## Recommendation
- Ready to commit : **OUI** — review PASS, Codex reconcilie (0 GAP, PARTIEL
  ferme par addition), gates verts. Stager le code Phase E + les 3 artefacts
  (preflight, review, codex_review) dans le commit phase.
- Action immediate : commit `feat(factory-operator): Sprint 72 Phase E`.
- Carry-overs S73 (pour `sprint73_audit_plan.md`) : P2 model-picker
  intentions non-Claude ; P2 factory-operator sans runner de test. Plus les
  carries Phase D heritees (sync-FS-async resolve_daemon,
  diagnostic-poll-generique). (Le P3 champ `thinking` mort est RESOLU, plus
  un carry.)

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA Phase E + description + compteurs
      1544 Rust / 279 Vitest / 6 size inchanges, front-only)
- [ ] Update `MEMORY.md` (ligne index si pivot description changee)
- [ ] Stage `sprint72_phase_e_preflight.md` + ce review + le codex_review
      dans le commit phase (ou un `chore(planning)` adjacent)
