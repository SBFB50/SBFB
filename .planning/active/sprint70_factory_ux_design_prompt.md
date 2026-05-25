# Prompt Claude Design — Factory Viewer + Factory Operator

## Contexte projet

SBFB (nexus-grid) est une plateforme P2P décentralisée de compute et
d'hébergement d'apps. Chaque app est une archive web (zip avec
index.html) distribuée sur le réseau et rendue dans un iframe sandbox.

Le shell principal (React + Tailwind + shadcn/ui) utilise un dark theme
GitHub-inspired : fond `#0d1117`, surface `#161b22`, cartes `#1c2128`,
border `#30363d`, accent `#58a6ff`, texte blanc, muted `#8b949e`,
vert `#3fb950`, rouge `#f85149`, jaune `#d29922`.

## Deux produits à designer

### 1. Factory Viewer — app SBFB sandboxée (lecture seule)

**Rôle** : vitrine protocole accessible à tous les utilisateurs du
réseau. Affiche les apps en développement ou publiées avec leurs
artefacts de preuve.

**Contraintes** :
- App SBFB statique (HTML/CSS/JS pur, pas de framework lourd)
- Tourne dans un iframe sandbox (`allow-scripts`, sans `allow-same-origin`)
- Communique uniquement via postMessage bridge (3 méthodes :
  `browse_list`, `search`, `proof_card_get`)
- AUCUN accès à localhost, aucun token, aucun endpoint Operator
- Aucune action d'écriture (pas de build, commit, push, sign)

**Écrans** :
1. **Home** — liste des apps trouvées (nom, version, catégorie, statut
   publication), barre de recherche, filtres
2. **Detail** — fiche app : versions, changelog, preview exportée, Proof
   Card (provenance Ed25519, commit source, hash archive), source links

**UX** : le Viewer montre et vérifie, il ne crée rien. L'utilisateur qui
veut agir est renvoyé vers Factory Operator.

### 2. Factory Operator — outil local privilégié (action-gated)

**Rôle** : cockpit développeur pour coder, tester, builder, signer,
publier et piloter le process agent depuis le noeud local.

**Stack** : Vite + React + TypeScript + Tailwind + shadcn/ui.

**Backend** : `sbfb-factory operator serve` (axum, 13 endpoints JSON) :
- `GET /api/status` — état sprint courant
- `GET /api/lint` — résultats lint planning
- `GET /api/audit/{rev}` — audit commit
- `GET /api/prompt/{kind}` — prompt assemblé par kind
- `GET /api/context` — context file brut
- `POST /api/context-pack` — nouveau context-pack
- `GET /api/providers` — providers disponibles
- `POST /api/actions/run` — exécuter action allowlistée
- `GET /api/actions/log` — journal des actions
- `POST /api/artifacts/draft` — brouillon artefact (path guard)
- `POST /api/chat/session` — créer session agent chat
- `POST /api/chat/message` — envoyer message dans session
- `GET /api/chat/{id}/log` — log session chat

**Pages / vues** (sidebar navigation) :

1. **Sprint Overview** — statut sprint en temps réel : numéro, phases
   avec badges état (done/active/pending), artefacts présents/manquants,
   verdicts par gate, compteurs tests
2. **Sélecteur d'agents** — "Qui code ?" + "Qui vérifie ?" en dropdown
   humain : Claude, Codex, GPT, Agent local, Humain
3. **Assistant de phase** — workflow guidé par intentions métier :
   - "Préparer la phase" → preflight
   - "Relire la phase" → phase-review
   - "Vérifier avant validation" → phase-auditor
   - "Préparer le message de commit" → commit-body
   - "Transmettre à un autre agent" → handoff
   - "Auditer le sprint" → audit-gate
4. **Lint Operator** — résultats lint visuels (warnings/errors avec
   fichiers concernés)
5. **Auditeur de commit** — entrer un SHA, voir l'audit (sections
   présentes/manquantes, review check, codex check)
6. **Transfert agent** — importer/générer prompt de base, assembler
   context-pack, copier/ouvrir agent cible
7. **Context Pack Builder** — nouveau contexte complet pour un
   provider/rôle cible
8. **Centre d'actions** — file d'actions proposées/validées par
   l'opérateur, exécution limitée status/lint/audit/prompt/draft
9. **Agent Chat** — discussion libre opérateur-agent avec transcript
   local et actions liées
10. **Journal d'actions** — log des actions Operator

**Principe UX obligatoire** : les CTA principaux sont en français et
expriment des intentions métier ("Préparer la phase", "Relire", etc.),
jamais des commandes techniques (`sbfb-factory`, `--kind preflight`,
`provider local`). Les commandes techniques apparaissent dans un
panneau "Détails techniques" repliable.

**Layout** : sidebar gauche avec icônes + labels, zone contenu à droite,
status bar en bas (HEAD tip + sprint number). Dark theme cohérent avec
le shell SBFB. Responsive.

## Design system partagé

Les deux produits partagent des composants de lecture :
- `StatusBadge` — badge coloré (done/active/pending/error)
- `VerdictChip` — chip verdict (PASS/FAIL/CONCERN/PENDING)
- `ProofCard` — carte preuve Ed25519 (commit source, hash, signature)
- `SprintTimeline` — timeline phases du sprint
- `ChangelogPanel` — changelog versions
- `PreviewList` — liste previews exportées

Le Viewer utilise UNIQUEMENT ces composants de lecture.
L'Operator les utilise aussi, plus ses extensions privilégiées.

## Interdits sécurité Viewer

Le bundle/source Viewer ne doit contenir AUCUN des termes suivants :
`localhost`, `X-SBFB-Token`, `/api/actions`, `/api/chat`,
`/api/context-pack`, `git commit`, `git push`, `child_process`,
`powershell`, `cmd.exe`, `factory-ui/operator`.

## Livrables attendus

1. Mockup Factory Viewer Home + Detail (2 écrans)
2. Mockup Factory Operator avec sidebar et au moins 4 pages clés :
   Sprint Overview, Assistant de phase, Agent Chat, Centre d'actions
3. Design system : palette, composants partagés, icônes
4. Responsive breakpoints (mobile-first)
