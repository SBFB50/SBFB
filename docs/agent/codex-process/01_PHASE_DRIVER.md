# 01 - Phase Driver

But : implementer une phase sans scope creep et sans perdre le lien avec le
plan.

## Preflight avant code

Une phase sprint normale doit avoir :

- `sprint{N}_kickoff.md`
- `sprint{N}_plan.md`
- `sprint{N}_phase_{X}_preflight.md` ou `pivot_proposal`
- verdict G8 exploitable : `EXECUTE`, `PLAN-ADAPT` ou
  `SCOPE-CUT-CONSISTENT`

Sans G8, produire le preflight avant d'ecrire la premiere ligne de code.
Un `PASS-PENDING` de driver signifie seulement que la phase est prete pour
verification Codex; il ne permet jamais de committer.

## Extraction de scope

Construire une table avant edition :

| Element | Source | Decision |
| --- | --- | --- |
| Objectif phase | plan phase X | obligatoire |
| Fichiers attendus | plan/preflight | autorises |
| Scope cuts | kickoff/plan | interdits |
| Tests attendus | plan/review precedent | obligatoires |
| Surfaces securite | grep S3/S4 | section Security delta |
| Decisions Day 0 | kickoff | non modifiables |

Commandes utiles :

```bash
rg -n "Phase X|Commit cible|Tests plan|Scope cuts|Day 0|D[1-5]" .planning/active/sprint{N}_*.md
git diff --name-only HEAD
git log --all --grep="DEVIATION\|rejected\|scope-cut\|threat-model" --oneline -- <target-files>
```

## Regle d'atomicite

Un commit phase doit avoir un sujet principal.

Exemples :

- OK : route collision fix + tests + docs qui expliquent ce fix.
- Pas OK : route collision fix + identity persistence + nouvelle UI reseau.
- OK si documente : route collision fix + deviation explicite approuvee dans
  `sprint{N}_phase_{X}_preflight.md` ou review.

Si un changement utile est decouvert mais hors scope :

1. le noter comme finding ou carry ;
2. ne pas le coder dans le meme commit ;
3. ou ecrire une deviation explicite avant de coder.

## Security delta obligatoire

Documenter `Security delta` dans le review et dans `## Codex verification` du
commit body si le diff touche :

- auth/token/session ;
- CORS, CSP, CORP, COEP, iframe sandbox ;
- loopback, named pipe, UDS, host/origin checks ;
- body limits, upload/download limits ;
- path traversal, archive extraction, blob serving ;
- signing, canonical bytes, schemas, `*_VERSION`, `DOMAIN_*` ;
- provenance, open-source verification, SBFB bridge ;
- secrets, keystore, identity, panic/duress.

Sans cette evidence, le commit est process-fail meme si les tests passent. Ne
pas ajouter `Security delta` comme dixieme section du body: le body de phase a
exactement 9 sections.

## Verification incrementale

Pendant l'implementation :

```bash
python scripts/agent/agentctl.py verify-on-write --file <changed-file>
```

Avant review :

```bash
cargo fmt --all --check
cargo clippy -p <crate> --all-targets --locked -- -D warnings
cargo test -p <crate> <targeted-test> --locked
cd web && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit -- <target>
```

Avant commit phase, appliquer la verification complete demandee par
`prompts/agent/phase-review.md`, sauf docs-only explicitement justifie. Ensuite
faire la verification Codex finale: le review committable doit contenir
exactement `## Verdict: PASS`, remplacer tout `PASS-PENDING`, et le body doit
contenir exactement 9 sections dont `## Codex verification`.

## Smoke reel

Ne jamais se contenter d'un test unitaire quand le bug est utilisateur.

Exemples :

- SPA reload : tester navigation document directe et navigation in-app.
- daemon app detail : tester iframe, console errors, network responses.
- P2P/gossip : tester chaque machine, pas seulement l'API locale.
- consent/auth : tester route sans token et avec token.

Les matrices detaillees sont dans `04_DOMAIN_SMOKE_MATRICES.md`.
