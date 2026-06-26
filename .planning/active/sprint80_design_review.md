# Sprint 80 — Design Review Board (G1)

> **Méthode (ultracode).** Kickoff orchestré en Workflow (`wf_6dec1de4-397`,
> 12 agents Opus 4.8 1M) : 8 agents recherche D1..D8 → G1 design review board
> (perspective indépendante, scoring 0-5 par décision) + 2 lentilles adversariales
> (technique + scope/décisions gelées) → synthèse directeur. **Pas de rubber-stamp** :
> les corrections PO 2026-06-26 surchargent le blueprint ; tous les ancres code
> re-vérifiés ce tour, et **re-confirmés indépendamment par le main thread** avant de
> figer le kickoff (cf. §Réconciliation main-thread).

## Verdict board : CONDITIONAL → PASS

**7/8 PASS** (D1/D4/D5/D6/D7/D8 PASS dont 6 forts) + **1 CONCERN** (D2, résolu par
conditions intégrées) + **0 CONFLICT**. Aucune décision ne contredit un invariant
backend ni une directive PO. Les 2 conditions G1 sont **intégrées comme tranchées** ;
les 2 arbitrages PO load-bearing sont **résolus** (Option B / Option B). Le lot passe
à **PASS**.

## Scoring par décision

| Décision | Score | Verdict | Note |
|---|:--:|---|---|
| **D1** Framework — React 19 | 5/5 | PASS | Incumbence assumée honnêtement ; charnière Solid-2.0-GA datée et confirmée **non remplie** (Beta `v2.0.0-beta.0`). |
| **D2** Composants — Base UI seul + shadcn build-time | 4/5 | **CONCERN→résolu** | Double-source runtime existant RÉDUIT par le greenfield ; conditions intégrées (lint anti-`@radix-ui`, syntaxe CLI corrigée, re-thème oklch). |
| **D3** Motion — `motion/react` | 4/5 | PASS s/réserve→résolu | Override PO du « 0 lib » ; SUPPRIME la dette WAAPI-maison ; réserve promue en **gate BLOQUANT** (size-limit + lint + allowlist). |
| **D4** Auth — cookie HttpOnly | 5/5 | PASS | Pièce maîtresse threat-model-aware ; fermeture CSRF vérifiée au code ; correction de libellé intégrée. |
| **D5** Styling — Tailwind v4 + oklch | 5/5 | PASS | Identification chirurgicale du « design-system test » (preset shadcn `index.css:1-57`, PAS daisyUI). |
| **D6** IA — bi-focal STEER-B/VERIFY-B | 5/5 | PASS | Invariants reflétés 1:1 au code ; couture D6↔D7 nommée (résolue par Option B). |
| **D7** Scope — routes | 5/5 | PASS | Scope honnête evidence-tight ; tension « 0 defer du coeur » arbitrée PO (Option B). |
| **D8** Carries/testabilité | 5/5 | PASS | Cadrage exact ; régression −7/−8 Vitest actée + SSE single-Done re-couvert. |

## Détail par décision (perspective indépendante)

- **D1 — React 19 (5/5 PASS).** Double incumbence vérifiée : `tools/factory-operator/package.json:32-33`
  (react ^19.2.4) + shell `web/` (411 Vitest). Le greenfield jette le CODE, pas le socle techno.
  Condition de réouverture Solid VÉRIFIÉE non remplie (Solid 2.0 = Beta, `@solidjs/signals` « may
  still have breaking changes ») = cible mouvante refusée pour un front solo lignée-OpenBSD. Résidu
  process : re-vérifier le statut GA de Solid 2.0 au preflight de la 1re phase front (seule charnière
  qui flip). React Compiler laissé en preflight = sage.
- **D2 — Base UI seul + shadcn générateur build-time (4/5 CONCERN, résolu).** shadcn N'EST PAS un
  rival mutuellement exclusif de Base UI : générateur copier-coller bâti SUR des primitives headless
  + Tailwind + cva, supportant officiellement Base UI depuis ~fév. 2026 (`shadcn create --base=base`).
  Le double-source RUNTIME existe **déjà aujourd'hui** (`@base-ui/react` + 6 `@radix-ui/*` + `shadcn`
  en deps) — le greenfield le JETTE, D2 réduit le wart. **Conditions intégrées** : (1) **lint
  BLOQUANT « 0 import `@radix-ui` ne survit au runtime »** (sinon single-source = discipline, pas
  structure) ; (2) syntaxe CLI corrigée (`create --base=base`, pas `--primitive=base-ui`) ; (3) tout
  code émis dépouillé du preset de tokens et **re-thémé oklch avant commit** (verrou D2↔D5). **Tranché
  par le supersede factory-ui (Arbitrage PO #2)** : plus d'héritage à respecter → recommandé **wrappers
  Base UI écrits-main** pour ~8 primitives (sobriété/anti-drift), shadcn reste un devtool optionnel.
- **D3 — Motion `motion/react` (4/5 PASS, réserve promue gate).** Override PO #2 du « 0 lib » du
  blueprint. Motion domine anime.js pour un front React 19 écrit par agents (déclaratif state-driven :
  `AnimatePresence`/`layout`/`layoutId` = terrain des transitions STEER↔VERIFY/gate/stream ;
  `useReducedMotion`/`MotionConfig reducedMotion='user'` honore la doctrine S70). Bonus : Motion
  **supprime** la dette WAAPI-maison du blueprint. La sobriété revendiquée n'est **pas mécaniquement
  tenue** sans filet (le front n'a AUCUN script `size` ; les agents écrivent `motion.div` lourd, pas
  `LazyMotion`+`m`). **Réserve promue en 3 verrous BLOQUANTS** : size-limit chiffré (hero ~4,6 kb),
  lint anti-`motion.*`-nu, allowlist figée des 5 signatures.
- **D4 — Auth cookie HttpOnly (5/5 PASS).** Décision la plus solide, threat-model-aware. Vérifié :
  `auth.rs:229-262` lit le token UNIQUEMENT dans le header (`:42`/`:258`), 0 cookie/query ; SSE
  (`:136`) et WS (`:145`) ne posent pas d'en-tête custom → 401/403 en prod. **Correction de libellé
  intégrée** : SameSite=Strict = stoppeur CSRF **primaire** (cross-site ne porte jamais le cookie
  host-only) ; Host = anti-DNS-rebinding (PAS anti-CSRF) ; **Origin-vérifié-seulement-si-présent** à
  GARDER (un SSE same-origin omet souvent Origin — l'exiger casserait le SSE). Bootstrap `?token` via
  launcher (qui connaît le token) préserve le bearer vs un `GET /` qui distribuerait le cookie à tout
  appelant loopback. Pas de `Secure` sur http loopback (exception 127.0.0.1 potentially-trustworthy à
  vérifier à l'impl).
- **D5 — Tailwind v4 + oklch (5/5 PASS).** Identification chirurgicale evidence-backed du « design-system
  à écarter » (correction PO #3) : PAS daisyUI (jamais présent côté Operator) mais le **preset de tokens
  shadcn GitHub-dark** de `index.css:1-3,8-57` (imports `tw-animate-css` + `shadcn/tailwind.css` + hex
  `#0d1117`/`#58a6ff`/`--sidebar-*`/`--chart-1..5`). Réconciliation parfaite : shadcn-composants (D2,
  runtime Base UI) vs preset-de-tokens-shadcn (D5, jeté) = deux couches distinctes. Operator hors
  `BLOB_SERVE_CSP` (`csp.rs:33`) → styling non bridé. Dette corpus v4 CSS-first nommée → mitigation lint.
- **D6 — Bi-focal STEER-B/VERIFY-B (5/5 PASS).** Triangulé (blueprint §4 + paradigme R&D + brief +
  wireframes importés). Invariants reflétés 1:1 au code : MUR `:35`, rail J1 ← `dirty/staged` listes
  `:419-420`, knowledge `chat_history_authoritative:false:437`, slot ÉTAT machine-d'états ne disant
  jamais PASS. Aucun agent `ux` enregistré → UX-honnêteté routée dans `nexus-phase-review-deep` + scan
  front anti-PASS. **Couture D6↔D7 résolue par Arbitrage PO #1** : VERIFY-B (bande gates + diff-viewer)
  n'est plus aspirationnel — les routes F+G le rendent réel dans S80.
- **D7 — Scope routes (5/5 PASS).** Evidence-tight : table de routes (`:122-146`) sans `/api/gates`
  ni `GET /` ; `dirty/staged` = listes de noms ; seul git diff de contenu = commits passés
  (`/api/sprint-history/diff/{sha}:144`) ; `run_gate_csp_authoring(workspace)` (`gates.rs:386`) n'a
  pour seul appelant que la pipeline publish CLI (`pipeline.rs:55`). Exposer un gate « live » exige de
  **redéfinir la sémantique** (quel workspace ? quand ?) = design. **Arbitrage PO #1 = Option B** :
  ces 2 routes (`GET /api/git/diff` working-tree + `GET /api/gates`) ENTRENT dans S80 (phases F+G) →
  VERIFY-plein livré, honore « 0 defer du coeur ».
- **D8 — Carries/testabilité (5/5 PASS).** S80 = sprint front qui ne ferme **aucun** P1 (assumé). 2
  carries P1 in-vivo (sharding RIG-ABSENT, app-authoring `Not evidenced`) restent ouverts (backend/
  cross-pair, hors-portée front). 8 P2/11 P3 → carry sprint dette Rust/docs nommé (P2-8 + P3-6 routés
  S80). Gate de testabilité conforme : T1 hermétique BLOQUANT (5 sous-tests) ; T2 JSON committé
  déterministe ; `RIG-ABSENT` illégitime (Operator 100 % loopback). CONFIRMED_ISSUE adversarial résolu :
  régression −7/−8 Vitest actée + SSE single-Done re-couvert.

## Lentilles adversariales — constats et résolution

- **[Technique] D1 incumbence / charnière Solid (NO_ISSUE/MINOR)** — blueprint assume l'incumbence
  honnêtement ; Solid 2.0 Beta non-GA confirmé (WebSearch juin 2026). → Conserver React 19 ; re-vérifier
  le statut GA au preflight ; clause de veille v2.1.
- **[Technique] D2 réconciliation shadcn (NEEDS_PO/MINOR)** — double-source existe déjà, le greenfield
  le JETTE ; shadcn×Base UI confirmé (changelog janv./fév. 2026). Résidu : le `shadcn add` ré-injecte le
  preset de tokens. → Lint anti-`@radix-ui` + re-thème oklch obligatoire ; arbitrage générateur-vs-wrappers
  (résolu par supersede factory-ui : wrappers main recommandés).
- **[Technique] D3 dette motion vs sobriété (NEEDS_PO/MINOR)** — l'ajout d'une lib est mandaté (PO #2),
  pas une dérive ; mais le « 4,6 kb maîtrisé » est NON-ENFORCÉ (aucun script `size`). → 3 verrous
  promus BLOQUANTS.
- **[Technique] D4 fermeture CSRF (NO_ISSUE/MINOR)** — vérifié au code : cross-site → 403 (Origin) /
  401 (cookie absent sous SameSite=Strict). Imprécision de rédaction (Host crédité comme anti-CSRF). →
  Libellé corrigé (cf. D4).
- **[Technique] D7+D6 séquencement VERIFY-aspirationnel (NEEDS_PO/MAJOR)** — risque de fabriquer une
  bande gates sur données absentes. → **Résolu par Arbitrage PO #1 (Option B)** : on CONSTRUIT les routes
  (F+G) ; le terminal-PTY-as-VERIFY couvre le bootstrap jusqu'à la livraison du diff-viewer (H). T1 ne
  doit asserter le diff-viewer/panneau gates qu'**après** F→H.
- **[Scope] Operator hors CSP (NO_ISSUE/MINOR)** — VÉRIFIÉ TENU : `BLOB_SERVE_CSP` absent
  d'`operator_server.rs` (`csp.rs:8-14` = 2 consumers daemon + gate). → Router une CSP self-origin
  minimale (`default-src 'self'`) comme défense en profondeur (Phase A).
- **[Scope] Factory hors daemon (NO_ISSUE/MINOR)** — TENU : tous les ajouts dans `sbfb-factory`,
  `Cargo.toml` ne dépend pas du daemon, `tower-http ["cors","fs"]` déjà actif (`:162`). → Confirmer en
  preflight Phase A : 0 route au daemon.
- **[Scope] Browser = client (NO_ISSUE/MINOR)** — TENU : SPA servie par `ServeDir` Rust (pattern daemon
  `http.rs:512`), pas de Tauri/Electron ; build Vite = build, pas runtime.
- **[Scope] Socle gelé S70 `@sbfb/factory-ui` non adressé (NEEDS_PO/MAJOR)** — `CLAUDE.md:495-496`
  impose un socle readonly partagé ; il existe (`tools/factory-ui`, exports `./readonly` + `./operator`)
  mais l'Operator actuel ne l'importe PAS (orphelin). → **Résolu par Arbitrage PO #2 (Option B)** :
  supersede explicite et tracé ; jeter `tools/factory-ui` ; re-planifier la fondation Viewer/Operator
  en S81. D2 libéré de toute contrainte d'héritage.
- **[Scope] 0 defer du coeur vs VERIFY différé (NEEDS_PO/MAJOR)** — tension avec la directive PO. →
  **Résolu par Arbitrage PO #1 (Option B)** : VERIFY-plein dans S80.
- **[Scope] Régression de couverture greenfield (CONFIRMED_ISSUE/MAJOR)** — jeter factory-operator (+
  factory-ui) retire leurs Vitest réels (`executionChat.test.ts` single-Done PO-14 + `ExecutionChat.test.tsx`).
  → Phase I re-couvre l'intention single-Done ; delta acté ; total interdit de descendre silencieusement.
- **[Scope] refonte cosmétique honnête (NO_ISSUE/MINOR)** — (d) tient (0 P1 fermé). Caveat : D4
  (auth-transport) est un VRAI changement de surface d'attaque, pas un re-skin → exiger une revue
  lens-sécurité. Note : avec Option B, S80 n'est PLUS « cosmétique » (backend VERIFY + auth) — assumé.
- **[Scope] carry 8 P2/11 P3 vs « dette = phases » (NEEDS_PO/MINOR)** — exception à la règle d'absorption ;
  défendable (items backend/docs hors-thème front). → Recommandé carry vers sprint dette Rust/docs nommé
  (preflight D8) ; acter dans `sprint81_audit_plan §3`.
- **[Scope] AGPL-3.0 (NO_ISSUE/MINOR)** — toutes deps permissives (MIT/OFL), Geist vendoré 0 CDN. →
  Confirmer en preflight que le manifeste ne réintroduit aucune dep non-permissive.

## Arbitrages PO (résolution des 2 constats MAJEURS)

1. **Cœur S80 = bi-focal COMPLET (Option B).** VERIFY-plein entre dans S80 : 2 routes backend
   (`GET /api/git/diff` working-tree calculé Rust + `GET /api/gates` sémantique gate-live) + diff-viewer
   bespoke + panneau gates front (phases F→H). Honore « 0 defer du coeur ». Conséquence assumée : S80
   plus gros (backend + front) ; sémantique gate-live = travail de design (preflight Phase G).
2. **Socle gelé S70 `@sbfb/factory-ui` = SUPERSEDÉ (Option B).** `CLAUDE.md:495-496` explicitement
   supersédé et tracé ; jeter `tools/factory-ui` (orphelin) ; fondation Viewer/Operator re-planifiée
   from scratch en S81. D2 libéré de toute contrainte d'héritage.

## Réconciliation main-thread (vérification indépendante des faits load-bearing)

Avant de figer le kickoff, le main thread a re-vérifié les 4 faits porteurs :
- **`auth.rs` header-only** : `AUTH_HEADER="x-sbfb-token"` (`:42`), `auth_required` (`:229`) → 401
  « missing or invalid X-SBFB-Token » (`:258`), 0 cookie. CONFIRMÉ → l'auth-cookie est nécessaire.
- **Operator hors CSP** : `BLOB_SERVE_CSP` n'apparaît que dans `gates.rs` (le gate authoring), **absent
  d'`operator_server.rs`**. CONFIRMÉ.
- **Routes** : table `:123-146` — **aucune** `/api/gates` ni `/api/git/diff` working-tree (seul
  `/api/sprint-history/diff/{sha}` = commits passés). CONFIRMÉ → VERIFY-plein nécessite bien 2 routes.
- **Socle orphelin** : `tools/factory-ui` existe (`package.json:6-8` exports `./readonly` + `./operator`)
  ; `tools/factory-operator/src/` ne l'importe pas. CONFIRMÉ → supersede sans casse d'usage.

## Statut

**PASS** — le lot de décisions est adopté, les 2 conditions G1 (D2/D3) intégrées comme verrous BLOQUANTS,
les 2 arbitrages PO tranchés (B/B). Phase A (préalable backend auth cookie) peut s'ouvrir après ce kickoff.
Les questions de preflight (défauts recommandés au kickoff §Questions ouvertes) seront confirmées au
preflight de chaque phase concernée.
