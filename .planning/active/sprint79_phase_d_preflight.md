# Sprint 79 Phase D — Preflight G8

## Verdict: PLAN-ADAPT

Le coeur du design Phase D est sain et alignable (champ ADDITIF `authoring_knowledge {path, hash}` REFERENCE — jamais inline, jamais autoritaire — modele exact `process_docs`). Aucun arbitrage PO Day-0 n'est requis (D1/D6/D9 tous tenus), donc **pas de DESIGN-CONFLICT**. Mais le texte litteral du plan n'est pas executable tel quel sur 3 points concrets, chacun appuye par du code/filesystem reel : (a) daisyui/MANIFEST.json n'existe pas a la Phase D ; (b) `handle_chat_session` reconstruit son propre pack et n'herite pas ; (c) le commentaire de provenance in-code doit eviter la regex anti-promesse du gate. D'ou **PLAN-ADAPT**, pas EXECUTE nu.

## Resume executif (5 lignes)

1. Design valide : path+hash REFERENCE additif, modele `process_docs` (operator_server.rs:404-409), conforme a l'etat de l'art OSS (MCP : URI-ref pour gros/reuse, pas inline) — les couches anime.js (781KB+314KB) restent referencees, pas embarquees.
2. Integrite reelle : le hash blake3[..8] est verifie-par-recalcul (test delta +2 ferme la boucle au niveau context-pack ; `tests/animejs_manifest.rs` couvre deja le 16-hex des couches) — pas un hash decoratif.
3. Autorite : `chat_history_authoritative=false` present et inchange aux DEUX handlers (operator_server.rs:416 + :669), garde par test existant ; le champ ne porte que des metadonnees, pas de texte d'instruction → aucun vecteur d'injection.
4. Day-0 tenus : D1 (knowledge sous docs/factory/knowledge/, hashe, hors workspace), D6 (consomme/affiche, anti-PASS), D9 (0 bump wire, 0 dep — blake3 1.8.5 single-version, serde_json single ; context-pack = JSON operator local, pas wire P2P).
5. 3 adaptations d'implementation requises (animejs-seul, dual-write chat-session, commentaire passe-immuable) — actionnables, non-bloquantes, sans toucher l'arbitrage PO.

## S1a OSS prior-art

**Verdict S1a : EXECUTE (APPROACH-ALIGNED).** Surfacer un knowledge pack comme REFERENCE path+hash (pas du contenu inline) dans un context-pack de bootstrap est exactement la best-practice du spec MCP (« return resource links/URIs for large or frequently reused content; embed inline only for small content needed in the conversation »). Les couches anime.js sont 781KB `docs.json` + 314KB `primitives.json` ; les embarquer inline ferait exploser la fenetre de contexte pour zero benefice. Le plan fait exactement cela : fiche distillee `app-authoring.md` + couches lourdes REFERENCEES par path+hash.

- **Integrite (un hash qui ne protege rien s'il n'est jamais re-verifie)** : design sain car le hash EST re-verifie. `tests/animejs_manifest.rs` (Phase A, 9297f08) recompute blake3[..16] par couche et asserte l'egalite avec `MANIFEST.hashes` (rouge sur drift d'octet). Le test delta +2 ('hash recompute match') ferme la boucle au niveau context-pack. C'est la best-practice content-addressable (Git/IPFS/pip) : le hash donne l'integrite UNIQUEMENT recompute a la lecture.
- **Autorite (knowledge consomme devenant faussement autoritaire)** : c'est le seul vrai pitfall documente OSS 2026 (skill/reference traite comme ground-truth = vecteur d'injection). Le plan le defend : module CONSOMME/AFFICHE jamais autoritaire, `chat_history_authoritative=false` PRESERVE aux deux endpoints (operator_server.rs:416 + :669), D6 anti-PASS. Le champ est metadonnee (path+hash+exists), pas du texte d'instruction injecte → ne peut pas porter de directive vers l'autorite.
- Ancres : `handle_context_pack` operator_server.rs:355-427 CONFIRME ; `process_docs` :404-409 CONFIRME (4 `file_hash`, chemins litteraux) ; `file_hash` :340-353 CONFIRME ({path, hash:[..8], exists}).

## S1b deps

**Verdict S1b : EXECUTE.** Phase D ajoute ZERO dependance. blake3 deja declaree (crates/sbfb-factory/Cargo.toml:14 `blake3 = { workspace = true }`), deja utilisee fully-qualified a operator_server.rs:344 (`blake3::hash`) dans `file_hash`. serde_json deja workspace (Cargo.toml:22). Graphe transitif inchange : Cargo.lock resout un SEUL blake3 1.8.5 (count=1), serde_json 1.0.149 single, walkdir 2.5.0 single — pas de collision majeure (la lecon schemars 0.8/1.2 de S72 ne se reproduit pas). Aucun import nouveau requis. Aucun code reseau, aucun parser, aucune surface d'attaque. D9 honore.

## S2 historique

**Verdict S2 : EXECUTE.** La structure du context-pack a ete fondee en un seul commit (69e3a06, S70 Phase D) : chaque section EST une reference de fichier hashee {path, hash[..8], exists}, JAMAIS inline. `authoring_knowledge` est strictement homologue a `process_docs`. Le Truth Stack (RRV_FACTORY_CONTRACT.md:53-84 + AGENT_SYSTEM.md:12-27) regit l'autorite : le pack surface des fichiers repo rang-1 hashes et n'accorde aucune autorite ; MANIFEST.json est un fichier repo rang-1, conforme. `chat_history_authoritative=false` jamais mis a true dans aucun commit (reverse-commit clean). Le predecesseur immediat Phase C (95aba5b, meme domaine) a pose le precedent 'connaissance CONSOMMEE/AFFICHEE jamais autoritaire, 0 bump wire'. `authoring_knowledge` n'existe qu'en planning, jamais en code → net-new sans precedent a contredire. Le seul commit threat-model sur operator_server.rs (a0337c6, S71 Phase C) durcit CSRF/auth/spawn, orthogonal au contenu du pack. operator_server.rs intact depuis Phase C (95aba5b..HEAD vide) → ancres exactes a HEAD=16d84c2.

*Note S2 : derive de lettrage kickoff (context-pack='Phase C', kickoff:87-91) vs plan (context-pack='Phase D' apres re-lettrage 7d0225d). Meme scope ; suivre le LETTRAGE DU PLAN.*

## S3 threat

**Verdict S3 : EXECUTE (pas de faille structurelle).** La primitive est un POINTEUR HASHE additif lecture-seule, pas une nouvelle surface de confiance ni un nouvel actor. 5 questions resolues :
1. **Path traversal** : SANS RISQUE si chemins LITTERAUX hardcodes (modele `process_docs`, specifie plan:260-262 + D1). La seule entree-derivee existante (`specialized_kind`) est deja double-gardee (process.rs:51-83 + check `..`/`/`/`\` operator_server.rs:378). GARDE-FOU REVIEW : rejeter toute implementation derivant le path d'une entree requete.
2. **Integrite** : hash blake3[..8] = fingerprint generation-time, verifie-par-recalcul, prouve non-decoratif par le test +1. Conforme au design Operator existant.
3. **Autorite** : `chat_history_authoritative=false` aux 2 handlers (:416, :669) ; anti-PASS artifact-draft (:543-579) independant et intouche.
4. **Scellage** : `BLOB_SERVE_CSP` (blob_serve.rs:286) intouche (Phase E) ; le champ n'accorde aucune dispense CSP/FG5/FG6/FG8 ; lint authoring ADDITIF.
5. **SKILL.md routing** = doc de process (Markdown consomme au boot), aucun code execute, aucune surface reseau — risque faible.

Aucune regression sur T-OPERATOR-CSRF / T-OPERATOR-SPAWN (token+Host+Origin + gate SENSITIVE_ACTIONS hors-chemin, intacts). Residuel loopback-sans-peer-creds (THREAT_MODEL §14 :847-851) non aggrave (champ lecture-seule d'empreintes de docs deja lisibles par un process local du meme user).

## S4 wire

**Verdict S4 : EXECUTE.** Le context-pack est une reponse JSON HTTP LOCALE de l'Operator (`serde_json::json!` retournee en `Json<serde_json::Value>` sur POST /api/context-pack loopback, operator_server.rs:358/388-418), PAS un wire format P2P. Un champ additif n'impacte aucun `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` / `FEED_FORMAT_VERSION` — toutes les constantes wire vivent dans nexus-core-rs/nexus-shell-daemon-core/nexus-coordinator-rs (toutes =1), sbfb-factory a 0 constante wire propre (seul `DOMAIN_PROVENANCE_V1` importe dans gates.rs:6, intouche). `#[serde(default)]` non applicable (reponse macro-construite, jamais deserialisee en struct ; la seule struct Deserialize `ContextPackRequest` :286-295 est deja `#[serde(default)]` partout, inchangee). Pre-launch policy (CLAUDE.md:519-541) regit les wire formats seulement ; le JSON operator local n'y figure pas. Consommateurs schema-free : api-client.ts:43-44 type `Record<string,unknown>`, ContextPackBuilder.tsx:49-54 lit `{pack:string}` avec fallback `JSON.stringify` — aucun Zod `.strict()` (contraste S73 Phase E /search) → cle additive consumer-safe.

## Verification adversariale (drift des ancres + avocat du diable)

**Verifie moi-meme contre le code reel, pas les scans :**

- **DRIFT 1 (CONFIRME, materiel) — handle_chat_session ne reutilise PAS handle_context_pack.** Lecture directe operator_server.rs:657-671 : `handle_chat_session` construit son PROPRE literal `json!` reduit (base/universal/handoff + runtime_context {head,sprint,phase} ; PAS process_docs, agent_system, active_artifacts). Aucun appel a `handle_context_pack`, aucun helper partage. La formule du plan ligne 263 'herite via le meme pack' est IMPRECISE. Le champ doit etre ecrit aux DEUX blocs `json!` (:410-region ET :657-671). Non-bloquant mais le modele mental du plan est faux → adaptation 2.
- **DRIFT 2 (CONFIRME) — daisyui/MANIFEST.json absent a la Phase D.** `ls docs/factory/knowledge/` = seul `animejs/` present ; `daisyui/` = No such file or directory. Source encore sous `examples/daisyui-animejs-showcase/knowledge/daisyui/MANIFEST.json` (935 oct), promu Phase F. `file_hash:352` degrade en {path, exists:false} (0 panic) → non-bloquant, mais l'entree daisyui est une reference morte non-testable a la Phase D → adaptation 1.
- **Avocat du diable — anti-promesse gate (PROMISE_RE).** check-frontier-contracts.sh ligne 62 : la regex flagge `Phase [A-Z0-9]+ (will|adds|ships)`, `lands? in Phase`, `inert until Phase`. Scope find ligne 73 = `crates web/src` → operator_server.rs (sous crates/) EST scanne ; `.claude/` (SKILL.md) ne l'est PAS. Un commentaire de provenance `// Phase D adds authoring_knowledge` ferait ECHOUER le build → adaptation 3 (passe-immuable seulement). Forme clean validee : `// Sprint 79 Phase D - decision D#: ...`.
- **Largeur de hash (clarte test)** : `file_hash:347` emet blake3(MANIFEST.json bytes)[..8] (8 hex du fichier MANIFEST lui-meme), distinct du blake3[..16] des couches DANS `MANIFEST.hashes` (deja couvert par tests/animejs_manifest.rs). Le test +2 doit recalculer le 8-hex et garder les deux tests distincts.
- **Routing tables identiques** : preflight SKILL.md:115-125 et review SKILL.md:49-59 portent la table 5-lignes identique + la regle dual-edit. Insertion 'UI/animation/design app SBFB' = append propre aux 2 tables, miroir de la ligne `lib externe → context7`.

**design_conflict_found = false** : chaque ancre cassee degrade gracieusement et le plan est executable avec ces 3 notes. Aucun Day-0 (D1/D6/D9) n'est touche.

## Adaptations de plan (PLAN-ADAPT)

1. **animejs-seul a la Phase D.** Ne pointer `authoring_knowledge` QUE vers `docs/factory/knowledge/animejs/MANIFEST.json`. La ligne daisyui + son assertion de hash arrivent a la promotion daisyui (Phase F). Variante acceptable : pointer les deux + tester explicitement `exists:false` pour daisyui — privilegier animejs-seul (pas de reference morte). *Evidence : ls docs/factory/knowledge/ = seul animejs/ ; file_hash:352 gracieux ; plan:330,336 promotion Phase F.*
2. **Dual-write du champ.** Ecrire `authoring_knowledge` dans LES DEUX blocs `json!` : `handle_context_pack` (apres operator_server.rs:410) ET `handle_chat_session` (dans :657-671). Ne pas se fier a 'herite via le meme pack'. *Evidence : :657-671 = literal distinct, aucun helper partage.*
3. **Commentaire de provenance passe-immuable.** Forme `// Sprint 79 Phase D - decision D#: ...`. INTERDIT : `// Phase F will populate daisyui`, `// daisyui lands in Phase F`, `// Phase D adds authoring_knowledge`. *Evidence : PROMISE_RE check-frontier-contracts.sh:62 ; scope crates/ inclut operator_server.rs ; plan §3bis impose deja le passe immuable.*

## P2/P3 ouverts a surveiller en review

- **P3 ordonnancement (daisyui)** : verifier qu'aucune reference morte non-testee n'est creee (file_hash:352 degrade proprement, pas de menace).
- **P3 dual-write** : verifier que `authoring_knowledge` est present dans les DEUX reponses (context-pack ET chat_session), sinon l'invariant 'herite' est rompu.
- **P3 largeur de hash** : test +2 recompute blake3(MANIFEST.json)[..8] == champ.hash, distinct du [..16] des couches.
- **P3 coherence SKILL.md** : nouvelle ligne ajoutee IDENTIQUEMENT aux 2 tables (regle dual-edit preflight:123-125 / review:57-59).
- **P2/P3 residuel Operator (hors-perimetre)** : loopback sans UDS/peer-creds (THREAT_MODEL §14 :847-851) non aggrave par Phase D.
- **INFO garde-fou securite** : si le path est derive d'une entree requete (pas litteral hardcode), ESCALADER (path-traversal).

## Decision EXECUTER vs DEMANDER

**EXECUTER.** Regle §7.1 (bootstrap pre-flight, cas B post-design) : une phase dont le design est arbitre et fige (D1/D6/D9 Day-0 tenus, 0 DESIGN-CONFLICT) et dont les seules deviations sont des ajustements d'implementation evidence-based (sans toucher un arbitrage PO) relevent d'EXECUTER, pas de DEMANDER. Les 3 adaptations sont des corrections d'approche concretes (PLAN-ADAPT au sens §4.5.7), pas des questions ouvertes a l'arbitrage PO. Aucun des findings ne requiert une decision PO : daisyui (ordonnancement de phases deja figé D→F), dual-write (fidelite a l'invariant 'herite' du plan), commentaire (regle §3bis deja ecrite). Proceder au code Phase D avec les 3 adaptations appliquees.
