# Doctrine « contrat-pour-LLM » — les couches de surface ancrée et drift-gatée

> **Statut** : recherche hors-sprint / découverte. Aucune décision PO gelée ici, aucune
> ouverture de sprint. Lecture seule sur `.planning/` (ce doc excepté, écrit dans
> `research/` et non `active/` pour éviter la collision avec une session parallèle).
> **Source-ancré** : chaque fait se vérifie dans le code ; ce document n'est pas une
> autorité. Origine de la réflexion :
> `examples/daisyui-animejs-showcase/knowledge/DOCTRINE_NEXT_SESSION_prompt.md`.
>
> La section §10 « État des lieux » est remplie par un **workflow ultracode de découverte**
> qui scanne le code réel pour trouver les couches DÉJÀ en place (et d'autres non listées).

---

## 1. La thèse

**Toute primitive de FRONTIÈRE du projet** — wire format, protocole, API publique, contrat
d'app — *par opposition à* un helper interne — **est un contrat source-ancré, drift-gated,
consommable par un LLM.**

Ce n'est pas « écrire plus de doc ». C'est un **graphe** : des **surfaces ancrées dans la
source** et **gatées contre le drift**. La doc humaine pourrit ; un schéma généré dont le
build casse quand il diverge du code ne pourrit pas. La différence entre les deux est l'objet
de cette doctrine.

**Critère de frontière** (ce qui déclenche le contrat) : la primitive est-elle lue/consommée
par un acteur qui n'est pas le code lui-même ? Un autre nœud (wire), un client externe (API),
une app du réseau (contrat d'app/CSP), un autre agent LLM (prompt-kind, knowledge). Si oui →
frontière → contrat. Un helper purement interne au crate → pas de contrat (le code + les
tests suffisent).

---

## 2. Le graphe — nœuds et arêtes

### Nœuds

| # | Couche | Rôle | Propriété cardinale | Cadence |
|---|---|---|---|---|
| 1 | **CODE** | le QUOI (comportement actuel) | autorité de dernier ressort | continu |
| 2 | **ÉTIQUETTE** (schéma **généré**, drift-gated) | le CONTRAT (forme + invariants) | **toujours à jour** : le build casse si schéma ≠ code | **par phase** |
| 3 | **COMMIT ATOMIQUE** (`feat(scope): Sprint N Phase X`, body à sections imposées, signé/provenance) | le POURQUOI / QUAND / DELTA | **attribuable** + couche de décision | par phase |
| 4 | **GUIDE + `llms.txt`** (synthèse, Truth-Stack `repo>planning>commits>prompts>chat`, règle « Not evidenced ») | l'INDEX navigable | **anti-hallucination** | **clôture (1 phase)** |

### Arêtes

| # | Couche | Rôle | Propriété cardinale | Garde-fou |
|---|---|---|---|---|
| 5 | **Commentaires de provenance in-code** (`// Sprint N Phase X · §P64 · décision PO #N`) | les liens code ↔ décision ↔ contrat, en **rang-1** | **survivent au refactor** | **uniquement en arrière vers de l'immuable** (sprint/phase/§/decision# qui ont eu lieu), **jamais des promesses** |

⚠️ **Leçon STALE-PHASE-K-COMMENTS (carry réel S77)** : des commentaires `// lands in Phase K`
ont **menti** (promesses jamais tenues, devenues fausses). Règle qui en découle : une arête
de provenance ne pointe que vers du **passé immuable**, jamais vers une promesse future. À gater
par un `source-ref-check`.

---

## 3. Règle de cadence (tranchée)

- **ÉTIQUETTE générée → CHAQUE phase**, dans le commit de la primitive. C'est **gratuit** (le
  schéma est généré) et la gate **ne peut pas pourrir** (drift = build rouge).
- **GUIDE (synthèse) → UNE seule phase de clôture**. L'image complète n'est figeable qu'à la fin.
- Ni « une phase de doc par phase » (lourd, redondant), ni « tout à la fin » (faux pendant le
  sprint, bâclé à l'arrivée).
- **Leçon L/M/N de S77** : la Phase **L** (schémas wire générés) aurait dû être **dispersée par
  phase** (chaque primitive livre son étiquette) ; les Phases **M/N** (synthèse + doc-lint +
  honnêteté) étaient **correctement groupées à la fin**.
- **Réalité des types qui churnent en cours de sprint** : non bloquant. L'étiquette étant
  *générée*, on **régénère** à chaque phase qui touche la primitive — le snapshot suit le code,
  la gate reste verte tant que schéma == struct. (cf. §9 Q4)

---

## 4. Les 3 compagnons (prouvés efficaces cette session knowledge-pack)

1. **Priorité des sources** (anti-invention) :
   `schéma/type généré > tests > llms.txt / doc officielle > prose ; jamais inventer`.
   Prouvé : daisyUI **livre** un `llms.txt` officiel ; anime.js des `.d.ts` ; SBFB **génère** via
   `schemars`. Le knowledge pack anime.js l'inscrit noir sur blanc : « Source de vérité = le
   code, pas la prose » (`…/knowledge/README.md`).

2. **Sonde de comportement** (verdict machine, pas capture) :
   toute primitive **vivante** livre une sonde rejouable rendant un **verdict machine**
   (`PASS` / `ADJUST|BLOCK` / `RIG-ABSENT`). C'est exactement le `scripts/acceptance/b3_*.sh`
   du projet. Prouvé hors-projet : `seis-probe`/`gears-probe` ont **trouvé un vrai bug** et
   permis le tuning ; les harnesses `render-check.mjs`/`motion-check.mjs` sont les vérificateurs
   mécaniques de la boucle.

3. **Vérification adversariale des FAITS** :
   un agent **indépendant** confronte le contrat à la source avant qu'il soit cru. Prouvé : la
   revue adversariale a **trouvé le trou** de `check-csp.mjs` (`form-action` / `base-uri`
   manquants + drift de commentaire `esm`→`umd`), documenté dans
   `…/knowledge/factory-integration-hardened.md`.

**(+) Principe génératif** (modèle Idea Engine, `…/knowledge/ideas/IDEAS.md`) :
**la machine génère/réduit/note, l'humain arbitre le goût**. 32 gen-1 → 12 mutants gen-2 →
juges adversariaux 5-dimensions → **shortlist pour curation humaine**. Aucune décision de goût
n'est automatisée ; seules la génération et la notation le sont.

---

## 5. Pourquoi ça sert RRV (l'objectif)

RRV est *garbage-in / garbage-out* : sa qualité = la qualité de la source indexée. Les couches
rendent la source **fiable par construction** :

| Couche | Apport RRV | Mode RRV qui la lit |
|---|---|---|
| **ÉTIQUETTE** | rend le contrat **vérifiable-par-machine** (le « V ») + **fraîcheur garantie** (drift-gate) | `@dev`, `@audit` |
| **`llms.txt` / GUIDE** | **navigable sans halluciner** (Not-evidenced) | `@research`, tous |
| **COMMIT** | couche de décision **attribuable + signée** | `@product`, `@audit` |
| **Arêtes (commentaires)** | liens **zéro-saut en rang-1** code↔décision | `@dev`, `@security` |

Gains mesurables : justesse, **fraîcheur garantie**, anti-hallucination, vérifiabilité,
cohérence multi-agent.

---

## 6. Où ça doit vivre (recommandation, à trancher par le PO)

- **Process ICI** (le repo SBFB) : un **pattern nommé** dans `docs/rust/PATTERNS.md` (+ miroir
  shell si besoin) + un **check « gate-map »** « primitive de frontière → contrat + gate »,
  référencé depuis `docs/claude/README.md`.
- **Process portable Factory** : la **généralisation** dans `docs/agent/AGENT_SYSTEM.md` +
  content-model `docs/factory/knowledge/`. **S79 (capacité app-authoring anime.js+daisyUI) en
  est la 1ʳᵉ instance concrète** — le knowledge pack 5-couches + le contrat CSP factorisé sont
  déjà la forme « étiquette + sonde + priorité des sources ».
- **Dogfood** : sur **S78** (qui produit une primitive **vivante** = l'orchestrateur shard
  live → cas-test idéal de la sonde ; `scripts/acceptance/b3_shard_pipeline.sh` EST déjà la
  sonde). **Garder l'ajout S78 LÉGER** (la règle + sa gate), pas un mini-sprint doc — ne pas
  diluer le P1 sharding (cf. §9 Q3).

---

## 7. Le check « gate-map » (proposition à formaliser)

Un check méta (CI) qui maintient l'invariant « toute primitive de frontière a son contrat +
sa gate ». Il **FAIL** si l'une de ces conditions n'est pas remplie :

1. **Étiquette absente** sur une primitive de frontière déclarée (wire/API/contrat d'app sans
   schéma généré + sans test de drift).
2. **Source-ref non résolu** : un commentaire `// Sprint N Phase X · §P…` pointe vers un
   §/fichier/ancre qui n'existe pas (modèle direct de `check-sharding-docs.sh` :
   `anchor_present`).
3. **Source-ref vers une promesse** : un commentaire pointe vers une phase/décision *future*
   (anti STALE-PHASE-K).
4. **Sonde absente** sur une primitive **vivante** (déclarée comme telle mais sans
   `scripts/acceptance/*.sh` produisant un verdict machine).
5. **Honnêteté** : un doc de frontière sans le marqueur de statut requis (modèle `PROVISIONAL`
   + caveat cardinal de `check-sharding-docs.sh`).

> Le périmètre exact (registre des « primitives de frontière » : explicite via une liste, ou
> implicite via une convention de chemin/annotation) est l'arbitrage central de la
> formalisation — cf. §9 Q2.

---

## 8. Résumé des couches (vue compacte)

```
CODE ──(génère)──▶ ÉTIQUETTE (schéma drift-gated, par phase)
  │                     ▲
  │                     │ build rouge si divergence
  ├──(commit atomique)──┤  POURQUOI/QUAND/DELTA, signé, sections imposées
  │                     │
  ├──(commentaire rang-1, vers le passé immuable)──▶ ARÊTE provenance (gated source-ref)
  │
  └──(synthèse de clôture)──▶ GUIDE + llms.txt (Truth-Stack, Not-evidenced)

Compagnons transverses :
  · priorité des sources (généré > tests > llms.txt > prose ; jamais inventer)
  · sonde de comportement (verdict machine PASS/ADJUST/RIG-ABSENT)
  · vérification adversariale des faits (agent indépendant)
  · génératif : machine note, humain arbitre le goût
```

---

## 9. Questions ouvertes (la tâche de formalisation)

1. **Formaliser la doctrine en une page** — où exactement ? (reco §6 : PATTERNS ici +
   AGENT_SYSTEM Factory). → *à trancher*.
2. **Définir le check gate-map précis** — qu'est-ce qui FAIL ? (proposition §7). Le point dur :
   comment **déclarer** qu'une primitive est « de frontière » (registre explicite vs convention
   de chemin/annotation `// FRONTIER:`). → *à trancher*.
3. **Dogfood léger sur S78 maintenant, ou attendre S79 ?** (reco §6 : ajout LÉGER sur S78 — la
   règle + sa gate — sans diluer le P1 sharding). → *à trancher*.
4. **Cadence étiquette-par-phase vs types qui churnent** — résolu §3 (régénération = OK car
   généré) ; reste à confirmer qu'aucune primitive de frontière n'échappe à la régénération.
5. **Vérifier dans le code ce qui existe DÉJÀ** — pour ne pas réinventer. → **§10 ci-dessous**
   (découverte ultracode).

---

## 10. État des lieux — ce qui existe déjà (découverte ultracode)

> _Source : workflow `doctrine-layer-discovery` (run `wf_1edc7069-f32`). 9 finders
> multi-modaux Opus 4.8 1M → **99 mécanismes** vérifiés adversarialement (existe-t-il ?
> FAIL-il au drift ? câblé en CI ?) → synthèse. 109 agents, ~5 M tokens. Chaque fait
> ci-dessous est ancré dans la source par les finders ; à re-vérifier avant tout codage._

### Verdict d'ensemble

Le graphe-contrat est **réel et solide aux deux extrémités** — la couche **CODE** (verify /
caps / version-gate à l'exécution, testée en CI sur 3 pipelines) et une **vraie ÉTIQUETTE
générée drift-gated** — **mais l'étiquette générée ne couvre QUE le sharding (8 types) +
`TaskResponse`**. `schema_for!` n'apparaît que dans `schemas/{shard,task_response}.rs` ; les
~21 autres familles wire reposent sur le version-gate runtime + tests manuels. Les **sondes**
(`b3_*`) sont la couche la mieux *conçue* (vrai verdict machine) mais **aucune n'est câblée
CI** et aucun consommateur ne bloque sur leur artefact JSON. Et surtout : **il n'existe AUCUN
`source-ref-check` général** au-delà de `docs/sharding/` — les ~356 commentaires de provenance
in-code ne sont gatés par rien, et **l'anti-pattern STALE-PHASE-K est LIVE**.

### Couche par couche (vérifié adversarialement)

| Couche | Statut | Incarné par (réel) | Note honnête |
|---|---|---|---|
| **CODE** | bien-couverte | `canonical_bytes()` + **23 `DOMAIN_*_V1`**, sign/verify caps dual-enforced + ALPN admission-before-bytes, `*_FORMAT_VERSION` version-gate par décodeur (23 familles), `FeedEntry` raw-op | 2 trous : aucun gate **build** ne force le bump de `FORMAT_VERSION` (convention seule ; le test `seed_announced_raw_op_no_version_bump` est tautologique) ; unicité pairwise des 23 `DOMAIN_*_V1` non assertée |
| **ÉTIQUETTE** | partielle | `shard_schema_snapshot_matches_struct` (8 types `schema_for!`), `TaskResponse` snapshot, `schemas_publish_required_fields`, `shard_session_view_schema_is_whitelisted`, `BRIDGE_METHOD_ALLOWLIST` parité Rust↔TS, Zod `.strict()`, `check-spdx.sh` | générée **sharding-only** ; 21 familles wire sans schéma ; web Zod = **miroirs hand-maintained** (0 `ts-rs`/`typeshare`) ; le self-heal write-on-missing peut **re-verdir** un schéma supprimé |
| **COMMIT** | partielle | `agentctl` lightcheck (body 9 sections + regex phase + LOC-ban), auditor-gate `## Verdict: PASS`, gate artefact Codex, `release-attest.sh` SLSA, warrant canary | bloquant **LOCAL seulement** ; **backstop CI `phase-review-cross-check.yml` MORT** (regex `feat\(sprint…\): Phase [A-F]` → 0 match, plafond `[A-F]` alors qu'on est à Phase L) = faux-vert permanent ; signature git non gatée |
| **GUIDE / `llms.txt`** | partielle | `check-sharding-docs.sh` (link-check + `anchor_present` + honnêteté + french-body), registre `§P30..§P69`, Truth-Stack (prose) | **aucun `llms.txt` SBFB** (le seul est celui de daisyUI, ingéré comme *source*) ; `check-sharding-docs` confiné à `docs/sharding/` (liste d'ancres figée) ; `docs/factory/knowledge/**` **absent** ; hashes `MANIFEST.json` des packs **non recomputés** |
| **EDGE** (provenance) | partielle | **~356** commentaires `// Sprint N Phase · §PNN · PO-N` (55+ fichiers crates, 75+ web), `anchor_present` (set fermé), `.semgrep/sbfb.yml` | convention dense **mais non gatée** : **aucun `source-ref-check` générique** ; **STALE-PHASE-K LIVE** (`http.rs:2111` « lands in Phase K » ; `will populate/expose` dans `state.rs`/`cli.rs`/`registry.rs`/`keystore.rs`/`iroh_runtime.rs`) ; semgrep local-only fail-open, `capability_gate.yml` orphelin |
| **COMPANION-A** (priorité sources) | partielle | Truth-Stack 5 rangs + « Not evidenced » (AGENT_SYSTEM §1), `schema_value_matches_core_export` (worker==core), RRV `@modes` | « Not evidenced » = **pure prose non gatée** (seul le flag rang-5 `chat_history_authoritative:false` est testé) ; le test worker==core est un shim délégant |
| **COMPANION-B** (sonde→verdict machine) | partielle | `b3_live_pc_vps.sh` + `b3_shard_pipeline.sh` (JSON `{status: PASS\|BLOCK\|RIG-ABSENT}`, exit 0/1/3, `pass()` inatteignable sans `run_proof`), `phase_h_compute_local.sh`, `count-tests.sh` | **mieux conçue conceptuellement** mais **0 câblage CI** (rig 2-machines ; b3 dit « never run in CI ») ; **aucun consommateur ne parse le JSON** pour bloquer ; `phase_h` n'émet pas le vocabulaire fermé |
| **COMPANION-C** (vérif adversariale) | partielle | **Gate Codex** (artefact brut `codex exec -o`, non-réécrit, CONFIRMÉ/GAP/PARTIEL + evidence), `spec_consts_exist` (`SHARD_PROTOCOL_SPEC.md`↔Rust const, **en CI**), `anchor_present` | meilleure incarnation **mais** gate Codex **local-only** (jamais en CI) ; `spec_consts_exist` sharding-only + ne lie que la *présence* du nom de cap, pas sa valeur |
| **COMPANION-D** (génératif) | absente | — | Idea Engine = **artefacts + prose uniquement** ; `grep scripts/ idea/novelty/judge/dejavu` = rien. Aucun harness ne matérialise même le « génère/note ». Limite assumée par la doctrine, mais 0 code |

### Autres couches/gates déjà en place (non prévues par la doctrine — à intégrer)

La réponse directe à « en trouver d'autres » : **7 mécanismes réels** hors-doctrine.

1. **`BLOB_SERVE_CSP`** (`blob_serve.rs:286`) — constante Rust **source de vérité runtime**
   de la CSP servie à chaque réponse blob-serve. Primitive de frontière **sans étiquette ni
   cross-check** vs `check-csp.mjs` (3 sites CSP dupliqués non gatés). C'est exactement la
   cible du gate CSP S79.
2. **`size-limit` bundle budget** (`web/.size-limit.json`) — **vrai drift-gate fail-fast
   câblé CI** (ci.yml + woodpecker + verify.sh). Contrat anti-bloat octets/chunk.
3. **`agentctl` Check 4 (wire-format warning)** — détecte un stage de `canonical.rs`/`schemas/`
   /`_VERSION`/`DOMAIN_`/`serde(default)` et exige un preflight FULL S4. Seul mécanisme reliant
   un changement de surface wire à une preuve de scan — mais **WARN, non bloquant, local**.
4. **`phase-precommit-lightcheck` Check 2** — résout les chemins fichiers cités dans le **body
   de commit** (amorce d'un source-ref niveau COMMIT, advisory).
5. **SLSA attestation-schema + reproducible-build** (`scripts/ci-smoke/`) — valident la forme
   in-toto/SLSA v1 + `sha256(sujet)==bytes` + 2-builds-identiques. **Vrais drift-gates mais
   ORPHELINS** (non câblés CI ; seul `release-attest.sh` tourne, jamais ses validateurs).
6. **warrant-canary freshness** (`canary-monthly.yml` + `canary/mod.rs`) — dead-man-switch
   Ed25519, gate >45j lié à `CANARY_VALIDITY_DAYS` (câblé GHA).
7. **`.semgrep/sbfb.yml`** — règles de drift architectural ancrées à des incidents réels
   (`sbfb-canonical-bytes-jcs`, zip-traversal, no `todo!()`). FAIL au drift **mais local-only**
   (PostToolUse, fail-open) ; `capability_gate.yml` orphelin.

### Trous nets (couches/gates absentes)

- **`source-ref-check` GÉNÉRIQUE** des commentaires `// Sprint N Phase · §PNN` (anti
  STALE-PHASE-K) — réclamé par la doctrine, **n'existe nulle part**.
- **`llms.txt` SBFB** (index navigable Truth-Stack généré/gaté).
- **gate-map méta** « primitive de frontière → étiquette + gate + sonde » — rien ne FAIL si
  une nouvelle primitive oublie son schéma.
- **codegen Rust→TS** (`ts-rs`/`typeshare` absents) — parité Zod/Rust = miroirs manuels.
- **gate build** forçant un struct wire modifié à bumper sa `FORMAT_VERSION`.
- **cross-check `BLOB_SERVE_CSP` ↔ `check-csp.mjs`** (3 sites dupliqués).
- **backstop CI serveur** des 9-sections-body / verdict-PASS / artefact-Codex (tout
  client-side aujourd'hui).
- **COMPANION-D** (génératif) entièrement hors-code.

### Deux constats actionnables surprenants

- **Gate CI mort, faux-vert permanent** : `phase-review-cross-check.yml` ne matche aucun
  commit (regex obsolète `[A-F]`, convention actuelle = `Sprint N Phase X`). Croyance de
  filet de sécurité serveur = illusoire. (Fix trivial, hors-scope ici.)
- **`BLOB_SERVE_CSP` testé par substring** : les 2 tests ne vérifient qu'un sous-chaîne
  (`connect-src 'none'`) → un drift interne (retrait de `form-action`/`base-uri`) **reste
  vert**. Confirme le diagnostic du design durci S79.

### Le check « gate-map » concret (proposé par la découverte, §7 affiné)

`scripts/check-frontier-contracts.sh`, **câblé en CI exactement comme `check-sharding-docs.sh`
[14] / `check-spdx.sh` [13]**, `set -euo pipefail`, réutilisant 3 patterns déjà éprouvés :

1. **Registre explicite + `anchor_present`** (modèle `check-sharding-docs.sh:69-74`) :
   annotation `// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION` sur chaque type
   wire signé/décodé ; le script asserte que la const `DOMAIN_*_V1`, la const
   `*_FORMAT_VERSION` et un test `*_rejects_unsupported_version` existent. **FAIL** sinon.
2. **Couverture étiquette** (modèle `shard_schema_snapshot_matches_struct`) : comparer les
   `// FRONTIER:` à la table `schema_snapshots()` ; **FAIL** si un type FRONTIER n'a ni
   snapshot `*.schema.json` ni exemption documentée `// FRONTIER-NO-SCHEMA: <raison>`. Attrape
   mécaniquement les 21 familles non-schématisées.
3. **Anti-promesse / source-ref** (le vrai trou EDGE) : grep
   `lands in Phase [A-Z]|will (populate|expose|add|read|land)` → **FAIL** si la phase promise
   est déjà close ; pour les `§PNN` cités in-code, réutiliser `anchor_present
   "docs/rust/PATTERNS.md" "§PNN"`.

**Bonus quasi-gratuit** : ajouter à `check-sharding-docs.sh` (déjà câblé) un cas
`BLOB_SERVE_CSP.contains("form-action")` + parité avec `check-csp.mjs`, fermant les 3 sites CSP.

> **Ce qui FAIL (rouge CI)** : une primitive de frontière sans son trio
> étiquette/version-gate/sonde déclarée ; un schéma généré supprimé sans exemption ; un `§P200`
> fantôme ; un `// lands in Phase K` survivant à la clôture de Phase K.
