# 03 - Commit Gate

But : empecher les commits techniquement bons mais process-faibles.

## Avant staging

Verifier :

```bash
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
python scripts/agent/agentctl.py precommit-lightcheck --scope staged
```

Si des fichiers non lies sont modifies, ne pas les staged.

## Staging coherence

Le staging doit raconter une seule histoire.

Verifier :

```bash
git diff --cached --stat
git diff --cached --name-only
git diff --cached
```

Bloquer si :

- fichiers planning hors phase melanges avec code ;
- features non liees ;
- caches/build outputs ;
- tests non lies ;
- gros changement de securite sans review ;
- schema/protocole sans preflight S4.

## Sections commit obligatoires

Un commit sprint phase doit contenir exactement 9 sections markdown de niveau
2. Les headers sont :

- `## Context`
- `## Changes`
- `## Tests`
- `## G8 traceability`
- `## Codex verification`
- `## Pre-launch protocol`
- `## Scope cuts`
- `## Carry closure / Unblock`
- `## Risk`

La ligne `Security delta` est obligatoire dans `## Codex verification` meme si
elle dit `None`. Elle doit etre non-vide si le diff touche une surface securite.
Ne pas l'ajouter comme dixieme section.

La section `## Codex verification` doit prouver que le review final contient
exactement `## Verdict: PASS`. `PASS-PENDING` bloque le commit.

La section `## Tests` doit contenir des **deltas cumules** par suite (before ->
after + delta + commande). Les deltas doivent matcher les sorties reelles des
commandes executees. Ne jamais reporter un delta sans avoir lance la suite. Cf.
`prompts/agent/commit-body.md` pour le format de reference.

## Template court

```text
## Context
- ...

## Changes
- path: ...

## Tests
- Rust workspace: <before> -> <after> (+<delta> Phase {X}) via <command>.
- Rust doctests: <before> -> <after> (+<delta>) via <command>.
- Vitest unit: <before> -> <after> (+<delta>) via <command>.
- Playwright: <before> -> <after> (+<delta>) via <command>.
- Frontend build/size/i18n: <result> via <commands>.
- Smoke matrix: <matrix name>.

## G8 traceability
- Preflight: ...
- Review gate: .planning/active/sprint{N}_phase_{X}_review.md contains exactly `## Verdict: PASS`.

## Codex verification
- Codex pass:
- Final review:
- Verification commands:
- Security delta: None.

## Pre-launch protocol
- Format/version impact: none.

## Scope cuts
- Honoured: ...
- Reopened: none.

## Carry closure / Unblock
- ...

## Risk
- ...
```

## Validation hook

Avant `git commit` :

```bash
python scripts/agent/agentctl.py precommit-lightcheck --scope staged
```

Pendant `git commit`, les hooks appellent :

```bash
python scripts/agent/agentctl.py precommit-lightcheck --scope message --message-file .git/COMMIT_EDITMSG
python scripts/agent/agentctl.py auditor-gate --message-file .git/COMMIT_EDITMSG
```

Un bypass doit etre exceptionnel et documente dans le commit suivant ou dans un
artefact `.planning/active/`.

## Interdictions

- Pas de `--no-verify` pour eviter un gate rouge.
- Pas d'amend sauf demande explicite.
- Pas de squash pour cacher un process failure sans trace.
- Pas de claims de tests non executes.
- Pas de "PASS" si le review est `CONCERN` ou `FAIL`.
- Pas de commit si le review est encore `PASS-PENDING`.
- Pas de body phase avec 8 headers, 10 headers, ou sans
  `## Codex verification`.
- Pas de preuve de Phase B pour autoriser une Phase C.
