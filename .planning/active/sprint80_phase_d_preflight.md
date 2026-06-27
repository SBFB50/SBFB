# Sprint 80 — Phase D — Preflight G8

**Phase** : D — Terminal-PTY-as-VERIFY (bootstrap) + diff de commits passes + knowledge advisory + brouillon, plus tous les folds §5.1 du catalogue (surfacage du gisement `sprint_history.rs` deja calcule).
**Date** : 2026-06-27.
**HEAD** : `6991d51` (Phase C DONE — STEER complet + rail altitude-0 + primitive SSE `useTokenStream`).
**Cas** : B (pre-code). **Orchestration** : Workflow ultracode `wf_531a5d92-a06` (11 agents Opus 4.8 1M : 5 scans factuels fan-out + 5 verificateurs adversariaux + 1 synthese). Format : 5 scans factuels (reconcilies en 4 sections) + verification adversariale.
**Verdict** : **PLAN-ADAPT** — le plan Phase D tient livrable par livrable ; 5 adaptations evidence-grounded ; **aucune** decision Day-0 figee touchee ; **0** DESIGN-CONFLICT.

> **Note process (honnetete, pour l'audit gate S81)** : 3 des 5 scans (S1a, S2, S3) ont retourne un `StructuredOutput` **placeholder** (`scan_id:"test"`) malgre un vrai travail d'outils (14/23/20 tool calls, ~290k tokens cumules de lecture reelle). La synthese reste fiable parce que (a) S1b + S4 ont retourne des findings reels detailles ; (b) les 5 verificateurs adversariaux ont fait leurs propres lectures (et adv:S1a/S2/S3 ont explicitement detecte+refute le placeholder puis re-verifie les vraies ancres) ; (c) l'agent de synthese a fait 8 tool calls sur le vrai code ; (d) **le main-thread a re-verifie chaque claim load-bearing ci-dessous contre le code a HEAD `6991d51`** (`PreflightPhase.file`, `AUTHORING_KNOWLEDGE_MANIFESTS`, `tests/daisyui_manifest.rs`, MANIFEST daisyui, `package.json` xterm, `.size-limit.json`). Tous confirmes. Le verdict ne repose donc PAS sur les scans degeneres.

---

## Resume executif

Phase D est une phase de surfacage : elle eleve le terminal xterm PTY en surface VERIFY de bootstrap, branche le diff de **commits passes** (route existante), l'inspecteur knowledge advisory, le brouillon PASS-bloque, le MUR plein-largeur, et **tous les folds §5.1** du catalogue (arbre de procede A1/U1, frise V8, provenance-de-verdict U2, journal/registre des refus S8/U5, inspecteur context-pack S2 + derive de hash D2, carte de conformite commit U3/A9/V10, rejeu de gate U6/V9, tiroir Sessions S7). Le front est greenfield React 19 (`tools/factory-operator`), consommant **uniquement des routes Operator existantes** — toutes verifiees presentes au HEAD.

Cote backend Rust, le plan annonce 2 ajouts « triviaux » dans la crate `sbfb-factory`. La verification de fond les reduit a **UN SEUL edit reel** : (D1) ajouter l'entree daisyui au const `AUTHORING_KNOWLEDGE_MANIFESTS`. (U2) est **deja satisfait** par l'existant (`PreflightPhase.file`), et le « test miroir » daisyui **existe deja** (committe S79 Phase F). Le seul vrai risque d'implementation est le **POIDS** d'xterm (~345KB raw) qui casserait le gate size-limit s'il atterrissait dans `index-*.js` : lazy-load + nouvelle entree `.size-limit.json` sont **non negociables**. Aucun de ces points ne rouvre une Day-0 ni un scope-cut : verdict **PLAN-ADAPT**.

---

## Les 4 scans factuels (reconcilies post-adversarial)

### S1 — Delta SOTA + deps / CVE / licences / poids (S1a + S1b) — **OK, EXECUTE cote deps**

- **[info] Phase D n'ajoute AUCUNE dep npm runtime.** Le trio xterm requis pour le terminal-VERIFY est deja vendore en Phase B : `@xterm/xterm ^6.0.0`, `@xterm/addon-fit ^0.11.0`, `@xterm/addon-web-links ^0.12.0`. *Evidence* : `tools/factory-operator/package.json:26-28` (re-verifie main-thread).
- **[info] Tout MIT — AGPL-3.0-compatible (Day-0 D10).** Les trois paquets xterm installes sont sous licence MIT. R19 / React Compiler = **non-sujet** : xterm est une lib DOM vanilla framework-agnostique (aucun `peerDependencies` React), montee via `ref`+`useEffect`.
- **[info] xterm n'est PAS encore importe dans `src/`.** `grep xterm|asciinema tools/factory-operator/src/` = 0 match ; le bundle actuel `index-*.js`=34523o ne le contient pas. Phase D cree l'usage reel.
- **[P1, confirme] POIDS BLOQUANT.** `node_modules/@xterm/xterm/lib/xterm.mjs`=344970o raw / 88508o gzip. Le budget `app` (`.size-limit.json`, limit 40KB, `gzip:false`) a seulement ~5.5KB de marge (34523/40960). `vite.config.ts` `manualChunks` ne splitte QUE `vendor-react` -> un import **statique** tomberait dans `index-*.js` et **casserait le gate** Day-0 D4/D5. **Remediation** (cf. Adaptations 1-2) : dynamic `import()` depuis la surface VERIFY hors rail altitude-0 + nouvelle entree `.size-limit.json` pour le chunk async.
- **[P2, confirme] CSS xterm.** `xterm.css`=7112o raw / 2519o gzip ; budget `css` limit 20KB, index css actuel 17652o (marge ~2.8KB). Le dynamic import splitte la CSS xterm en chunk async (rolldown) ; sinon relever le budget css.
- **[info] 0 dep ajoutable pour les folds.** Rejeu `.cast` (U6/V9) via `xterm.write()` (asciicast v2 = JSON ligne-a-ligne replayable), PAS `asciinema-player`. Attach WS maison (hook, pattern `useTokenStream` Phase C), PAS `@xterm/addon-attach` (absent du lockfile, non requis). `npm audit --omit=dev` = 0 vulnerabilite (coherent avec lock 349 pkgs).
- **[info] Verrou D3 tient.** 0 `@radix-ui` dans `package.json`/`src/` ; le gate `check-no-radix-runtime.sh` couvre les 3 couches. Le proxy WS `/api/terminal/ws` + bearer est deja cable cote Vite.

*Verdict S1 : volet deps propre et fige (0 nouvelle dep, 3 deps MIT deja vendorees, 0 CVE, R19/Compiler non-sujet). Deux contraintes d'implementation non negociables a graver AVANT le 1er Edit (Adaptations 1-2).*

### S2 — Decisions historiques traversees — **OK** (aucune decision gelee violee)

- **[info] D1 daisyui = decision S79 Phase D (packs hors workspace, hashes par provenance, consommes/jamais autoritaires).** Le commentaire `operator_server.rs:520` dit explicitement « animejs pack only at this revision » : ajouter daisyui est l'application 1:1 de la decision gelee, pas un changement de doctrine. Le const est un point d'edition unique par pack.
- **[info] MUR + `chat_history_authoritative=false` = invariants Day-0 refletes 1:1.** `SENSITIVE_ACTIONS=["shell","commit","push","PASS"]` (`operator_server.rs:37`) ; le declencheur est un substring-match large -> le front RESTITUE `requires_gate`, jamais un bouton Forcer/Override ni un pre-filtre « plus malin ». `chat_history_authoritative:false` present (`:595`, `:854`).
- **[info] Refus d'ecrire un PASS via l'Operator = grave dans le Rust** (`:732`, `:754` : « cannot write PASS verdict via Operator — use review/gate flow »). Le brouillon non-autoritaire qui refuse PASS reflete cet invariant.
- **[concern de sequencement, NON bloquant] Le « pouls gates » du rail precede `/api/gates` (Phase G).** `grep /api/gates` = 0 : la route n'existe pas encore. Phase D doit rendre le pouls en etat **degrade / « non cable »** (placeholder, deja le cas en Phase C), jamais une jauge/verdict UI — coherent avec la garde « 0 verdict calcule UI ». Note de sequencement, pas une violation Day-0.

### S3 — Threat model / surface d'attaque — **OK** (design threat-sound)

- **[info] Aucun nouveau wire P2P, aucune nouvelle route daemon.** Phase D consomme des routes Operator loopback existantes (`/api/terminal/ws`, `/api/sprint-history/*`, `/api/actions/log`, `/api/context-pack`, `/api/audit/{rev}`, `/api/lint`, `/api/chat/{id}/log`). La surface d'attaque heritee est inchangee : auth Host-loopback + Origin-si-present + bearer 2-transports (header X-SBFB-Token puis fallback cookie `sbfb_operator` gate same-origin, Phase A).
- **[info] Le terminal PTY est deja la surface la plus sensible — Phase D ne l'elargit pas cote serveur.** L'operateur tape `git diff`/`status` dans un PTY deja expose par `/api/terminal/ws` ; Phase D en fait un usage front, sans nouveau handler. Tout shell/commit/push reste derriere le MUR `SENSITIVE_ACTIONS`. Le PTY brut permet de taper `git commit` directement : ce contournement est un risque **deja accepte+documente** (operateur = proprietaire du noeud, single-user loopback) — Phase D ne l'aggrave pas (aucun nouveau handler serveur).
- **[info] Knowledge advisory = lecture seule.** L'inspecteur affiche `authoring_knowledge` (chemins hashes via `file_hash()`) + chip hash + bordure pointillee : fraicheur consultative, jamais autoritaire (decision S79-D6). La derive de hash D2 (`◦ derive — relu`) signale un drift on-disk, pas un verdict.
- **[info] CSP self-origin minimale (Operator hors CSP scellee, Day-0 #7) tenue** : le terminal et le rejeu `.cast` rendent dans le DOM via xterm, pas dans un iframe sandbox ; aucun `connect-src` externe ajoute. Le WS `/api/terminal/ws` est same-origin loopback. *Note implementation* : xterm sanitise les sequences d'echappement au rendu (pas d'eval) ; un `.cast` rejoue passe par `xterm.write()` (meme chemin de rendu que le PTY live).

### S4 — Wire format / invariants pre-launch — **OK** (2 ajouts surs ; 1 reduit a no-op)

- **[info] Aucun `*_VERSION` wire n'est dans le perimetre Phase D.** Les wire formats P2P versionnes (`Task`/`ProjectAnnouncement`/`CuratorList`/`FeedEntry`/`*_ANNOUNCEMENT_VERSION`/`KEY_ROTATION_FORMAT_VERSION`) ne sont pas touches. Les 2 ajouts sont dans la crate `sbfb-factory` (Operator loopback), reponses JSON `#[derive(Serialize)]` only (jamais decodees d'un pair) -> question `#[serde(default)]` sans objet, **0 bump**.
- **[info] D1 daisyui = ajout d'un litteral au const interne `AUTHORING_KNOWLEDGE_MANIFESTS`** (`operator_server.rs:521`), PAS un wire format. Le champ JSON `authoring_knowledge` (context-pack + chat session) gagne 1 element ; hash recalcule au runtime via `file_hash()`, jamais pinne en code. Le MANIFEST daisyui existe deja, tracke (corpus 7 couches + `.gitattributes`).
- **[info, CONFIRME main-thread] U2 est DEJA SATISFAIT — 0 edit Rust.** `PreflightPhase` (`sprint_history.rs:144-149`) porte deja `pub file: String`, `#[derive(Serialize)]` only. Le ticket plan « champ file si absent » est resolu par l'existant. **Ne PAS introduire de doublon de champ.** La clicabilite transversale D+H (provenance-de-verdict) reutilise ce champ cote front.
- **[info, CONFIRME main-thread — refutation adversariale] Le « test miroir » daisyui EXISTE DEJA.** `crates/sbfb-factory/tests/daisyui_manifest.rs` a ete committe en **S79 Phase F** ; il recompute `blake3[..16]` des couches promues + garde CRLF. **Ne PAS recreer** (doublon/collision). Le scan S4 d'origine recommandait a tort de le creer ; la verification au disque le refute.
- **[info] Les tests `authoring_knowledge` ne cassent PAS avec un tableau a 2 elements** : ils usent `.iter().find()/.any()` sur le suffixe de chemin, jamais une assertion de longueur. (A re-confirmer en codant : adapter tout test qui asserterait `len()==1`.)

*Verdict S4 : les 2 ajouts annonces se reduisent a UN SEUL edit Rust reel (le const daisyui). 0 bump wire, conforme politique pre-launch.*

---

## Ancres code re-verifiees (HEAD `6991d51`, par le main-thread)

| Ancre citee (plan/prompt) | Etat | Valeur reelle |
|---|---|---|
| `AUTHORING_KNOWLEDGE_MANIFESTS` | ✓ exact | `operator_server.rs:521` = `&["docs/factory/knowledge/animejs/MANIFEST.json"]` ; commentaire :520 « animejs pack only at this revision » ; fn `authoring_knowledge` :526 |
| `SENSITIVE_ACTIONS` (MUR) | ✓ exact | `operator_server.rs:37` = `&["shell","commit","push","PASS"]` |
| `requires_gate` :766-779 | ✗ corrige | **perime post A/B/C/F** -> `924,932,937,1026,1031,1064,1125` (synth ; a re-grep en codant) |
| refus PASS :574/:596 | ✗ corrige | -> `732,754` (« cannot write PASS verdict via Operator — use review/gate flow ») |
| dirty/staged :419-420 | ✗ corrige | -> `577,578` (`dirty_files`, `staged_files`) |
| `chat_history_authoritative` | ✓ present | `:595`, `:854` = `false` |
| `PreflightPhase.file` (U2) | ✓ deja la | `sprint_history.rs:144-149` `pub file: String` (#[derive(Serialize)]) — **U2 = 0 edit** |
| `tests/daisyui_manifest.rs` | ✓ deja la | existe (committe S79 Phase F) — **ne pas recreer** |
| `docs/factory/knowledge/daisyui/MANIFEST.json` | ✓ present | + corpus 8 fichiers (theming/classes-bank/components/synthesis/docs-llms/README/COMPONENTS + `.gitattributes`) |
| Routes terminal/sprint-history/actions/context-pack/audit/lint/chat-log | ✓ presentes | a re-grep les lignes exactes en codant (drift mecanique attendu) |
| Route `/api/gates` | ✗ absente | `grep` = 0 — **Phase G** ; pouls rendu degrade (deja le cas Phase C) |
| xterm dans `package.json` | ✓ present | `:26-28` (trio MIT, deja vendore Phase B) ; **0 import dans src/** |
| Entree `.size-limit.json` xterm | ✗ absente | seulement `app`/`vendor-react`/`css` — **a ajouter** |

---

## Adaptations de plan (PLAN-ADAPT)

> Regle §4.5.7 : chaque adaptation cite une evidence code/OSS concrete, ne touche AUCUNE Day-0 figee. **5 adaptations** -> signal meta note ci-dessous.

1. **Lazy-load xterm (hors rail altitude-0).** Importer xterm en `dynamic import()` depuis la surface VERIFY/terminal **uniquement**, jamais en statique dans le rail/hero. *Evidence* : `xterm.mjs`=344970o raw / 88508o gzip ; `.size-limit.json` `app` limit 40KB `gzip:false`, bundle index 34523o (marge 5.5KB) ; `vite.config.ts` `manualChunks` ne splitte que `vendor-react` -> import statique = +~280-345KB dans `index-*.js` = gate size-limit (Day-0 D4/D5) casse au wrap-up. *Day-0* : aucune touchee.

2. **Ajouter une entree `.size-limit.json` pour le chunk async xterm** (ex. `vendor-xterm`, mesurer le chunk reel produit par rolldown) + verifier que la CSS xterm (7112o) part en chunk async, sinon relever le budget `css`. *Evidence* : `.size-limit.json` n'a que 3 entrees, aucune xterm ; sans entree, le poids du terminal serait silencieusement non-mesure (viole le chiffrage Day-0 D4/D5).

3. **U2 = AUCUN edit Rust.** `PreflightPhase.file` existe deja (`sprint_history.rs:144-149`, `#[derive(Serialize)]`). *Evidence* : verification main-thread au code. Le ticket plan « champ file si absent » est resolu par l'existant — **ne pas introduire de doublon**. La clicabilite transversale D+H (provenance-de-verdict) reutilise ce champ cote front, pas un ajout struct.

4. **Test miroir daisyui = AUCUN fichier a creer.** `crates/sbfb-factory/tests/daisyui_manifest.rs` existe deja (committe S79 Phase F). *Evidence* : `Glob` au disque. Le plan §149 (« backend trivial + test miroir ») a ete ecrit avant l'atterrissage S79-F -> le **seul** edit Rust D1 reel = le const `operator_server.rs:521` + maj commentaire :520. *(A verifier en codant : le test existant couvre-t-il deja daisyui, ou faut-il l'etendre ? S'il ne couvre qu'animejs, etendre l'assertion plutot que recreer.)*

5. **Rafraichir les ancres de ligne perimes post A/B/C/F** (cf. table ci-dessus) : `requires_gate`, refus PASS, dirty/staged, routes. *Evidence* : grep au HEAD. Toutes les routes/champs/constantes EXISTENT ; seules les lignes ont derive (drift mecanique, pas un gap fonctionnel). **Directive : grep frais en codant, ne pas se fier aux lignes de la prose du plan.**

**Signal meta (5 adaptations + 2e PLAN-ADAPT consecutif apres Phase C)** : le plan Phase D a ete redige avant l'atterrissage de S79-F (test daisyui) et avant les decalages de lignes A/B/C/F. Les deux ajouts backend « triviaux » se sont reduits a un seul edit, et plusieurs ancres ont derive. Ce n'est pas un signe de plan errone — c'est le cout normal d'un plan ecrit en avance sur 4 commits ; aucune Day-0 n'est en cause. **Directive a l'implementeur** : faire confiance a l'etat du code (grep frais) plutot qu'a la prose du plan pour les ancres ; ne PAS recreer le test daisyui ; ne PAS ajouter de champ `file` U2.

---

## Scope cuts — conformite des folds §5.1

| Fold | Livre en Phase D | Coupe / differe (conformite) |
|---|---|---|
| **A1/U1** Arbre de procede | Restitution mono read-only sprint->phase->commit->artefact via `/api/sprint-history/{n}` (donnees deja calculees) | **PAS** la timeline-canvas de procede (differee — kickoff §Out) ; arbre read-only seulement |
| **V8** Frise des verdicts | Vue resserree de A1 | — |
| **U2** Provenance-de-verdict (D+H) | Tout verdict cliquable -> ouvre l'artefact `.planning/` source (via `PreflightPhase.file` existant) | RESTITUE, jamais calcule UI ; 0 score/jauge |
| **S8/U5** Journal + registre des refus | `/api/actions/log` (actions allowlistees + rejets PASS/traversal/non-allowlist) | Registre lecture-seule, jamais « reessayer en forcant » |
| **S2 + D2** Context-pack pre-vol + derive de hash | `POST /api/context-pack` + `file_hash()` (`◦ derive — relu`) | Fraicheur ≠ verdict |
| **U3/A9/V10** Carte de conformite commit | 9 sections body + `/api/audit/{rev}` + `/api/lint`, « N manques » | **Jamais une coche ✓** — issues from Rust |
| **U6/V9** Rejeu des passages de gate | `.cast` via `xterm.write()` (0 dep) + `/terminal/sessions` `/{name}` | PAS d'`asciinema-player` |
| **S7** Tiroir Sessions | Liste simple + replay STEER (incl. rejets du mur) via `/chat/{id}/log` | **PAS un board multi-agents / Mission-Control (coupe)** ; persistance disque = S81 |
| **D1** Fix gap daisyui | Const `AUTHORING_KNOWLEDGE_MANIFESTS` + (test deja present, a etendre si besoin) | — |

Scope cuts geles tenus : Apercu scelle / Proof Card = Viewer S81 ; publish reste CLI ; editeur CM6 riche, palette transversale, timeline-canvas, i18next/router complexe = differes ; auto-bascule STEER->VERIFY arrachee au stream **interdite** (state-driven seulement, le rail heberge un selecteur de MODE manuel, l'auto-bascule [fin de tour ET diff/gate frais] est Phase H).

---

## Edits backend Rust requis

**UN SEUL fichier Rust modifie pour le backend Phase D.**

1. **D1 — daisyui dans le const.** `crates/sbfb-factory/src/operator_server.rs:521` : ajouter `"docs/factory/knowledge/daisyui/MANIFEST.json"` au tableau `AUTHORING_KNOWLEDGE_MANIFESTS` (passe a 2 elements) + maj du commentaire `:520` (retirer « animejs pack only at this revision »). Le MANIFEST + corpus existent deja, tracke. Etendre le test `tests/daisyui_manifest.rs` si son assertion ne couvre pas encore l'inclusion daisyui dans le manifest list (verifier en codant).

2. **U2 — AUCUN edit.** `PreflightPhase.file` (`sprint_history.rs:144-149`) existe deja. **Deja satisfait.**

3. **Test daisyui — AUCUN fichier a creer.** `crates/sbfb-factory/tests/daisyui_manifest.rs` existe deja (S79 Phase F). **Ne pas recreer.**

Cote front (greenfield, hors perimetre Rust) : nouvelle entree `.size-limit.json` (chunk xterm async) + dynamic `import()` xterm — cf. Adaptations 1-2.

---

## ## Verdict: PLAN-ADAPT

Le plan Phase D tient livrable par livrable ; les 5 adaptations sont evidence-grounded (poids xterm/size-limit, U2 deja satisfait, test daisyui deja present, ancres de ligne perimes) et **aucune ne touche une decision Day-0 figee, un wire format, ni le threat model**. Les 2 ajouts backend annonces se reduisent a un seul edit Rust reel (const daisyui). **0 DESIGN-CONFLICT.** L'implementeur peut coder en gravant d'abord les 2 contraintes size-limit (lazy-load + entree) avant le 1er Edit du terminal.
