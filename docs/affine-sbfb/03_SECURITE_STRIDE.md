# Securite STRIDE

Ce fichier resume le threat model. La source complete reste:
`docs/security/THREAT_MODEL.md`.

## Spoofing

Risque: une app, un peer ou un appel loopback se fait passer pour un acteur
legitime.

Defenses:

- node_id Ed25519;
- bearer token local;
- Host / Origin allowlist;
- signatures de tasks, claims, results.

## Tampering

Risque: modification d'une archive, d'une task, d'un resultat ou d'une annonce.

Defenses:

- BLAKE3 sur blobs;
- provenance;
- canonical bytes;
- signatures Ed25519;
- verification cote daemon/coordinator.

## Repudiation

Risque: un acteur nie avoir publie, reclame ou produit un resultat.

Defenses:

- `TaskEntry` signee;
- `ClaimEntry` signee;
- `ResultEntry` signee;
- audit events;
- kudos ledger.

## Information disclosure

Risque: fuite via iframe, storage, keypair, consentement, resultats.

Defenses:

- iframe sandboxee;
- CSP;
- bridge whitelist;
- separation broker/executor;
- state local separe.

## Denial of service

Risque: flood gossip, clone abusif, bridge spam, worker sature.

Defenses:

- rate limits;
- caps de consentement;
- clone timeout;
- quarantine;
- allowlist;
- worker consent levels.

## Elevation of privilege

Risque: une app passe de l'iframe au shell, ou l'executor remonte au broker.

Defenses:

- bridge minimal;
- pas d'acces direct au filesystem;
- broker/executor split;
- executor sans keypair;
- peer creds UDS / Named Pipe.

## Points sensibles a surveiller

- keypair au repos;
- blob-serve dans le broker;
- surface des capabilities futures;
- P2P public: detectability inherente;
- Babel/liseuses: ne pas introduire une chaine DRM ou GAFAM.
