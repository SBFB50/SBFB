# Architecture SBFB

## Vision simple

SBFB se lit comme quatre zones:

```text
UI locale -> broker local -> protocole P2P -> compute volontaire
```

## UI locale

- `nexus-launcher`: lance le daemon et ouvre le shell.
- `web/`: shell React, Browse, apps, consentement.
- `sbfb bridge`: canal minimal entre iframe et shell.

Important: le launcher ne doit pas devenir le proprietaire P2P. Le daemon
garde le reseau.

## Broker local

- `nexus-shell-daemon`: processus long vivant.
- `nexus-shell-daemon-core`: auth, publish, browse, consent, files, canary.
- `nexus-coordinator-rs`: dispatch, validation, kudos ledger.

Le broker garde:

- identite Ed25519;
- token local;
- annonce P2P;
- validation des resultats;
- etat de consentement;
- browse et blob-serve.

## Protocole P2P

- `nexus-core-rs`: iroh, crypto, canonical bytes, Task, Claim, Result.
- `ProjectAnnouncement`: annonce de projet/app.
- `TaskEntry`: demande signee.
- `ClaimEntry`: worker qui reserve une task.
- `ResultEntry`: resultat signe.

## Compute

- `nexus-worker`: binaire contributeur.
- `nexus-worker-core`: consent, allowlist, rate-limit, execution.
- `nexus-executor`: process compute isole.

Invariant: l'executor n'a pas la keypair du broker.

## Carte courte

Voir aussi: `docs/architecture/WHITEBOARD_SBFB.md`.
