# 02 - Reviewer Audit

But : produire une critique au niveau Claude, orientee bugs, risques et preuves.

Le reviewer n'est pas la pour reformuler le travail du driver. Il cherche ce
qui ferait echouer le sprint, le commit ou l'utilisateur.
Pour un commit de phase, il joue aussi la verification Codex finale: il doit
remplacer tout `PASS-PENDING` par `PASS`, `CONCERN` ou `FAIL`. Seul
`## Verdict: PASS` est committable.

## Entrees obligatoires

```bash
git status --short --branch
git diff --stat HEAD
git diff --name-only HEAD
git diff HEAD
git log --oneline -20
rg -n "Phase|Scope cuts|Tests plan|Research|Verdict|G8|Carry" .planning/active
```

Lire :

- kickoff sprint ;
- plan sprint ;
- preflight phase ;
- review phase existant, s'il existe ;
- fichiers modifies ;
- docs securite/protocole si les surfaces sont touchees.

## Severites

- `P0` : corruption, fuite secret, bypass auth, execution dangereuse,
  regression protocole irrecoverable.
- `P1` : bug bloquant utilisateur, scope cut viole, test critique manquant,
  changement securite non justifie, data loss possible.
- `P2` : risque reel mais contournable, test incomplet, doc securite faible,
  atomicite imparfaite.
- `P3` : polish, wording, orthographe, maintenance faible risque.

Un review doit commencer par les findings, pas par un resume.

## Dimensions obligatoires

1. Scope et atomicite.
2. Staging coherence.
3. Plan vs diff.
4. Security delta.
5. Protocol/wire invariants.
6. Branch coverage.
7. Tests et smoke utilisateur.
8. Docs et commit body.
9. Carry-over et dette.
10. Regression historique.
11. Identite exacte sprint/phase entre G8, review, sujet et body.

## Anti-PASS facile

`PASS` avec 0 P2+ est suspect.

Le reviewer peut quand meme PASS sans finding seulement s'il documente :

- toutes les dimensions auditees ;
- les commandes lancees ;
- pourquoi chaque risque attendu ne s'applique pas ;
- quelles limites restent residuelles.

Sinon le verdict doit etre `CONCERN`.

## Patterns de critiques attendues

### Scope creep

Comparer le titre de commit au diff :

```bash
git diff --name-only HEAD
git diff --stat HEAD
rg -n "Commit cible|Phase X|Scope cuts" .planning/active/sprint{N}_plan.md
```

Si un fix touche des modules non necessaires, demander :

- est-ce dans le plan ?
- est-ce dans le preflight ?
- est-ce documente comme deviation ?
- est-ce teste comme feature distincte ?

### Security silent delta

Rechercher :

```bash
git diff HEAD | rg -n "CorsLayer|allow_headers|allow_methods|Access-Control-Allow-Origin|Content-Security-Policy|DefaultBodyLimit|token|Origin|Host|sandbox|canonical|VERSION|DOMAIN_"
```

Si present, exiger `Security delta` + test ou justification.

`Security delta` doit etre present dans le review et dans la section
`## Codex verification` du body; ce n'est pas un dixieme header du body.

### Smoke incomplet

Si le bug vient d'un chemin navigateur, exiger preuve navigateur ou Playwright.
Si le bug vient d'un chemin reseau, exiger preuve multi-node ou justification
de blocage.

## Output attendu

Ecrire dans `.planning/active/sprint{N}_phase_{X}_review.md` ou dans un rapport
de process dedie si le travail n'est pas une phase.

Structure :

```markdown
# Sprint N Phase X Review

## Verdict: PASS | CONCERN | FAIL

## Findings
- [P1] path:line - Titre court
  Evidence:
  Impact:
  Fix:

## Scope And Atomicity
## Codex verification
- PASS-PENDING remplace:
- Review final exactement `## Verdict: PASS` si committable:
- Body 9 sections avec `## Codex verification`:
- Security delta:
- Identite sprint/phase:
## Security Delta
## Tests And Smoke
## Protocol Invariants
## Commit Body Integrity
## Carry-Over
## Residual Risk
```

Regle : un `FAIL` ou P1 non corrige bloque le commit.
Regle supplementaire : `PASS-PENDING`, une preuve Phase B pour un commit Phase C,
ou un body sans les 9 sections exactes bloque aussi le commit.
