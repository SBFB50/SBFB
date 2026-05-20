# 05 - Automation Backlog

But : transformer les critiques recurrentes en checks. Ce fichier liste les
ameliorations `agentctl.py` qui rendraient le process Codex plus proche du
niveau Claude sans dependre d'un modele.

## A1 - Scope classifier

Probleme : un commit peut annoncer un fix cible et toucher plusieurs features.

Check propose :

- classifier les fichiers staged par groupe : runtime, HTTP API, frontend API,
  UI page, tests, docs, planning, security docs ;
- comparer au titre commit et a la phase plan ;
- bloquer si plus de deux groupes fonctionnels non documentes ;
- accepter si commit body contient `Deviation accepted:` avec path planning.

Niveau : block pour phase commit.

## A2 - Security delta required

Probleme : CORS, CSP, body limit ou auth peuvent changer sans justification.

Check propose :

Detecter dans staged diff :

```text
CorsLayer
allow_headers
allow_methods
Access-Control-Allow-Origin
Cross-Origin-Resource-Policy
Content-Security-Policy
DefaultBodyLimit
auth_required
x-sbfb-token
Origin
Host
sandbox
canonical
DOMAIN_
_VERSION
```

Puis exiger dans commit body :

```text
## Codex verification
- Security delta:
```

Niveau : block si absent, warn si trop vague.

## A3 - Smoke matrix declaration

Probleme : un fix integration peut passer les tests unitaires mais casser le
chemin utilisateur.

Check propose :

- si `web/src` + `http.rs` modifies : exiger `Smoke matrix: daemon-served SPA`;
- si `blob-serve` ou `BrowsedProject` modifies : exiger `Smoke matrix: app detail`;
- si `GossipClient` ou browse aggregator modifies : exiger `Smoke matrix: P2P/gossip`.

Niveau : warn au debut, block apres validation.

## A4 - HTML no-cache precision

Probleme : middleware HTML peut toucher toute la stack mais seulement agir sur
`text/html`.

Check propose :

- si `no_cache_html_middleware` ou `fallback_service` change, exiger un test
  qui prouve qu'une route JSON ne recoit pas le header HTML no-cache.

Niveau : warn ou block selon route publique.

## A5 - UI French accent scan

Probleme : `scan-en-strings.sh` detecte l'anglais, pas le francais degrade.

Check propose :

- dictionnaire minimal pour termes frequents :
  - `Reseau` -> `Réseau`
  - `abonnes` -> `abonnés`
  - `connectes` -> `connectés`
  - `details` -> `détails`
  - `securite` -> `sécurité`

Niveau : warn.

## A6 - Body limit justification

Probleme : changement d'upload/body limit = surface d'attaque.

Check propose :

- si diff contient `DefaultBodyLimit::max`, exiger une ligne dans
  `Security delta` avec :
  - limite exacte ;
  - raison UX/protocole ;
  - mitigation loopback/auth/quota ;
  - test ou carry si non teste.

Niveau : block.

## A7 - Dirty interrupted work guard

Probleme : un tour interrompu laisse des changements qui peuvent etre melanges
avec une nouvelle demande.

Check propose :

- `agentctl context` signale les fichiers modifies depuis le dernier commit ;
- si commit scope ne correspond pas a ces fichiers, afficher un warning fort ;
- option future : fichier `.planning/active/session_dirty_note.md` temporaire.

Niveau : warn.

## A8 - Review cannot PASS silently

Probleme : review complaisant.

Check propose :

- si review file contient `## Verdict: PASS` et aucun `[P2]`, `[P3]`,
  `Exhaustive negative evidence`, ou `No findings rationale`, bloquer.
- si review file contient `PASS-PENDING`, bloquer tout commit phase.
- si le commit body phase n'a pas exactement 9 headers de niveau 2 ou manque
  `## Codex verification`, bloquer.

Niveau : block pour audit gate.
