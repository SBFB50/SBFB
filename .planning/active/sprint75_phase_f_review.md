# Sprint 75 Phase F — Review (node-centric Browse : nodes list + node catalog + add-anchor)

Date : 2026-06-10. HEAD de base : `491b3c8` (diff uncommitted Phase F).
Process : Workflow multi-agent 5 dimensions adversariales (correctness,
security, wire, tests, patterns) → skeptics refute-by-default sur tout P0/P1
(2 skeptics par finding, majorité requise) → synthèse main thread. 7 agents,
~994k tokens.

## Verdict: PASS

0 P0. 1 P1 émis et **confirmé 2/2 par les skeptics**, corrigé in-phase.
33 findings au total (1 P1 + 4 P2 + 17 P3 + 11 NIT) : **20 corrigés in-phase**
(norme anti-faux-vert), 1 P2 déféré scopé (duress, pré-existant déjà routé),
le reste = NIT/P3 documentés sans action (détail ci-dessous). Promu PASS
après la gate Codex (3 rounds, cf. §Codex reconciliation : 3 GAP réels
corrigés in-phase, round 3 = 21 CONFIRMED 0 GAP OVERALL: PASS).

## Dimensions

| Dimension | P0/P1 | Findings mineurs | Notes |
|---|---|---|---|
| correctness | 0 | 9 | sélection seed_voluntary saine ; renforts UX state (toggle pendant fetch, échos, flash cold-start) |
| security | 0 | 4 | verrous 1-5 tenus et test-pinnés ; piège latent is_open_source curator hardcodé fermé |
| wire | 0 | 6 | /browse BYTE-IDENTIQUE confirmé ; 0 bump ; enveloppe /nodes strict / rows tolérantes vérifiées |
| tests | 1 (lock-4b faux-vert) | 5 | les suites re-exécutées par l'agent ; le P1 = classe T1/S75-A |
| patterns | 0 | 8 | doc-honnêteté (self_pin_enabled exacte) ; copy FR ; jargon « seeder » purgé |

## P1 corrigé in-phase (1, confirmé skeptics 2/2)

1. **[P1 tests] lock-4b : l'exclusion nodedirectory du croisement éditeur
   n'était PAS prouvée — test faux-vert** (classe T1 hijack-test-faux-vert,
   S75 Phase A). La fixture plaçait l'entrée nodedirectory APRÈS les annonces
   d'éditeur : `find()` retournant le premier match, supprimer le filtre
   d'exclusion laissait toute la suite verte (mutation survivante tracée par
   les 2 skeptics). Le scénario décisif manquait : une row catalogue dont la
   SEULE présence /browse est non-direct doit rendre AUCUN badge — sinon
   toute app légitime découverte par annuaire serait faussement « Version
   dérivée » (browse.rs:803 hardcode `is_open_source:false` sur la 3e
   boucle ; le tri prod ne garantit PAS éditeur-d'abord). FIX : fixture
   réordonnée (non-direct EN TÊTE, l'exclusion devient load-bearing) +
   3e app `PID_ORPHAN` catalogue-only + test
   `lock-4b decisif : une row catalogue sans annonce d'editeur ne porte
   AUCUN badge` + commentaires de fixture réécrits honnêtes. En complément
   (piège latent security #11) : le prédicat éditeur exige désormais
   `source === "direct"` strictement (la boucle CURATOR hardcode AUSSI
   `is_open_source:false` et la fixture pinne ce cas avec une entrée curator
   porteuse de hash).

## Findings corrigés in-phase (19 P2/P3/NIT)

2. **[P2 correctness+patterns ×2] SupportButton (NodeCatalog) sans
   invalidations React Query** — divergeait du miroir AvailabilitySheet
   in-phase. FIX : `useQueryClient` + invalidations `daemon-browse` +
   `seed-count` dans onSuccess.
3. **[P2 tests] Le bras prod principal (direct + version correspondante)
   jamais sélection-pinné** — FIX : le test discriminator force le bras
   Ticket (502 « could not fetch » ≠ 404) + variante UPPERCASE même bras.
4. **[P2 tests] Négatifs Q7/WEB-1 non probants (assertion à la frame 0)** —
   FIX : gates de probance (attendre l'émission de la requête seed-count
   avant d'asserter l'absence ; WEB-1 gate « toggle redevenu enabled » =
   signal post-réconciliation).
5. **[P3 correctness] Fallback agnostique : une direct hash-sans-ticket
   pouvait laisser épingler une AUTRE version que celle affichée** — FIX :
   le fallback annuaire est narrowé par le hash de la carte directe
   (`requested.or(direct_hash_no_ticket)`) ; pré-F ce shape était un 400,
   jamais un pin divergent ; test 400 ajouté.
6. **[P3 correctness+wire+patterns ×3] Lignes « en attente » sur-promettant
   pour les subscriptions curator-pures** (attention set unique Q3/DQ3) —
   FIX copy honnête : « Abonnement actif — aucun catalogue annonce pour
   l'instant. » (sans promesse d'annonce future) ; le discriminateur fin
   (a-déjà-produit-une-CuratorList) reste un carry S76.
7. **[P3 correctness+NIT wire] Toggle ON cliquable pendant la réconciliation
   (flash menteur pour un OFF persisté)** — FIX : disabled pendant
   `seedCountQuery.isLoading` (résiduel isError = défaut ON documenté, pas
   de régression vs pré-F).
8. **[P3 correctness+wire ×2] Badge/copy Q7 non gatés sur `archive_hash`
   (compte version-agnostique possible)** — FIX : gate `!!entry.archive_hash`
   sur le badge ET la copy « noeud d'origine » ; copy own conservée pour
   `isOwn` (le nœud d'origine, c'est toi) ; test dédié 2 cas.
9. **[P3 security] Piège latent verrou 4 : le filtre éditeur acceptait les
   rows source=curator** (hardcodées `is_open_source:false` par
   l'aggrégateur) — FIX : `source === "direct"` strict + commentaire + pinné
   par la fixture orpheline (cf. P1).
10. **[P3 security] `navigate()` non encodé depuis un project_id d'annuaire
    distant** — FIX : `encodeURIComponent` sur le call-site neuf
    (catalog-open) ; les sinks pré-existants (Browse.tsx) inchangés
    (hors-diff).
11. **[P3 wire] Verrou 4 résiduel : VerificationDetail pouvait rendre
    « Signature valide » pour une AUTRE version que la row cliquée** (record
    keyé par projectId, tripwire hashMismatch désarmé à provenanceHash=null)
    — FIX pris in-phase (cœur de l'exigence PO) : prop additive optionnelle
    `expectedArtifactHash` + avertissement « version différente »
    (`version-mismatch-warning`) quand `record.artifact_hash` diffère ;
    appelants historiques inchangés ; 2 tests (présence sur mismatch,
    absence sur match).
12. **[P3 wire] Rows catalogue dupliquées → clés React en collision** — FIX :
    `dedupeCatalog` (première occurrence par (pid, hash), doc du rationale).
13. **[P3 tests] Version-exactitude de la requête Q7 non assertée** — FIX :
    le test positif asserte `archive_hash=<hash>` dans l'URL seed-count pour
    les 2 cartes.
14. **[P3 tests] Branche isError /nodes non testée (classe
    SEARCH-VIEW-THROW-SKELETON)** — FIX : test drift Zod → « Erreur reseau ».
15. **[P3 patterns] Jargon « seeder » dans un texte UI + copy divergente** —
    FIX : badge « Joignable via un pair » aligné sur la copy
    AvailabilitySheet.
16. **[P3 correctness] Échos in-session non réinitialisés au changement
    d'app sans remount** — FIX : `useEffect` reset sur `entry.project_id`.
17. **[P3 correctness] Flash possible du ColdStart pendant le chargement des
    subscriptions** — FIX : `isEmpty` exige `!curatorsQuery.isLoading`.
18. **[NIT correctness+security ×2] Casse hex non normalisée
    (seed_voluntary requested_hash + seed_count self_seeding
    case-sensitive vs peer_count normalisé)** — FIX : lowercase une fois en
    tête des 2 handlers (lecon hex-case Phase D) + test UPPERCASE.
19. **[NIT patterns ×2] Commentaire test attribuant le rejet d'une clé
    ABSENTE à `.strict()` (mécanisme Zod erroné) + overclaim « the front now
    always passes »** — FIX : les 2 commentaires réécrits exacts.

## Findings déférés / sans action (scopés)

- **[P2 security] Duress gates de `seed_voluntary` + `set_keep_online`** —
  gap PRÉEXISTANT S74 (les handlers ne sont pas re-créés par F, seul le
  SELECT de seed_voluntary change) ; déjà routé **carry audit S76** (lot
  dette duress) par la review Phase E ; Phase F en augmente l'exposition UX,
  signal consigné pour la priorisation S76.
- **[NIT correctness] 404 « no source for the requested app version » rendu
  aussi pour un project_id totalement inconnu** quand un hash est demandé —
  sémantique acceptable (la version demandée est introuvable), distinction
  coûteuse pour zéro valeur produit ; sans action.
- **[NIT wire] Asymétrie 400/404 sur l'arête direct-avec-hash-sans-ticket en
  mode version-pinnée** — comportement documenté au handler ; sans action.
- **[NIT tests] Fidélité mineure des fixtures (URL provenance non assertée)**
  — sans action (couvert par le mock par pathname).
- **[NIT patterns] `truncateHex` dupliqué (3e/4e occurrence)** — pattern
  repo existant ; un refactor cross-fichiers = bruit hors-scope ; sans
  action.
- **[NIT patterns] Test addAnchor rangé dans `describe("listNodes")`** —
  cosmétique ; sans action.

## Fail-fast

- Itération pré-review : nextest ciblé 11/11 ; Vitest ciblé 77/77 (6
  fichiers) ; tsc 0.
- Complet pré-fixes review : fmt 0 (après reformat) ; clippy
  `--workspace --all-targets` 0 ; **nextest --workspace 1750/1750 0-skip**
  (1748 → +2) ; doctests 0 ; release OK ; web COMPLET vert pipefail : lint 0
  erreur, tsc 0, **Vitest 361/361** (334 → +27), coverage
  87.06/78.82/85.92/88.38 ≥ 85/78/85/85, build OK, size 6/6, scan FR clean.
  (1 défaut attrapé par le 1er run : import `waitFor` inutilisé — retiré,
  re-run pipefail vert.)
- Complet post-fixes review : fmt 0 ; clippy `--workspace --all-targets` 0 ;
  **nextest --workspace 1750/1750 0-skip** (les extensions review vivent
  dans les tests existants, +0 net vs pré-fixes) ; doctests 0 ; release OK ;
  web vert (Vitest 365/365 → puis 366 post-GAP-R1 → **367/367** post-R2,
  coverage finale 87.17/79.01/85.92/88.5 ≥ 85/78/85/85, build, size 6/6,
  scan FR). Les fixes Codex R1/R2 sont ts/tsx-only : le bloc Rust complet
  vert couvre l'état FINAL byte-identique de `http.rs`.
- 1 défaut de lint attrapé en route (setState synchrone dans un effect,
  règle cascading-renders) → remplacé par le pattern React canonique
  « adjust state during render » ; lint final 0 erreur.

## Codex reconciliation

Gate Codex GPT-5.5 (`codex exec`, sortie brute
`sprint75_phase_f_codex_review.md`) — **3 rounds** :
- **Round 1 : 17 CONFIRMED + 1 GAP → OVERALL: FAIL.** GAP confirmé et
  corrigé : la réinitialisation des échos in-session d'AvailabilitySheet ne
  suivait que `project_id` — un swap de VERSION du même projet gardait
  « Tu gardes ce projet en ligne » pour d'autres octets (le seed est
  versionné, WIRE-2). FIX : la clé suivie devient la PAIRE
  `project_id:archive_hash` + test `version_swap_drops_in_session_echo`
  (rerender même instance, l'écho tombe, le CTA revient). Boucle web
  complète re-verte (366/366).
- **Round 2 : 20 CONFIRMED + 2 GAP → OVERALL: FAIL.** (a) `callDaemon`
  jetait le body `{"error": ...}` des non-2xx — le dialog AddAnchor ne
  pouvait pas montrer la raison du daemon. FIX : extraction best-effort du
  `error` dans la `reason` (suffixe « — <raison> », préfixe `HTTP <status>`
  préservé pour les assertions existantes), toutes les surfaces DaemonResult
  en profitent ; assertion `bad key` ajoutée. (b) `/nodes` rendait le CTA
  cold-start quand `/curators` répondait non-data (subscriptions INCONNUES
  collapsées à vides). FIX : le cold-start exige des subscriptions
  CONNUES-vides (`kind === "data"`) + test dédié. Boucle web complète
  re-verte (367/367).
- **Round 3 : 21 CONFIRMED, 0 GAP → OVERALL: PASS.** Tous les livrables
  vérifiés evidence file:line, dont : schémas/routes API (enveloppe stricte,
  rows tolérantes, archive_hash optionnel, self_pin_enabled nullable requis),
  /nodes (rows annoncées + en-attente + cold-start), AddAnchorDialog (inerte,
  normalisation, alias subscribe, invalidations, raison daemon), routes lazy,
  /browse additif, NodeCatalog (source-pas-autorité, prédicat éditeur exact
  anti-fuite cross-version, Q7 scopé ancre+version, surfaces
  Ouvrir/Provenance/Garder-en-ligne), AvailabilitySheet (précédences WEB-1,
  reset pid:hash, Q7 gated), Rust (discriminateur + branches 400/404
  préservées, resolver want_hash 3 call-sites, self_pin_enabled tri-état
  brut, BrowseStatus trois variants INCHANGÉ des deux côtés), verrou lock-1,
  couverture tests web+Rust, suites re-exécutées par Codex lui-même.

Verdict final promu PASS post-Codex round 3. 0 P0, 0 P1 résiduels ; 1 P1
review + 3 GAP Codex corrigés in-phase ; 20 P2/P3/NIT review corrigés
in-phase, 1 P2 déféré scopé + NIT/P3 documentés (cf. §Findings déférés).
