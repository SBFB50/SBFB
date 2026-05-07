# Sprint 54 Phase C — preflight G8

Date : 2026-05-06 | HEAD : `ed5bbdc` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — wire le chainon manquant E2E au lieu d'un band-aid. Respecte.
- feedback_context7_systematic.md : iroh-docs DocTicket API deja connue du codebase (docs.rs share_write/share_read). N/A nouvelle lib.

## Scans

### S1a — OSS prior art
Phase cable un champ existant (tasks_doc_ticket) dans un wire format existant (invite v2). Pattern standard : capability token qui porte un ticket d'acces a un document distribue. Projets de reference : iroh-docs (le ticket EST le pattern iroh natif), Keybase team invites (portent des tokens d'acces aux repos). APPROACH-ALIGNED.

### S1b — deps
+1 workspace dep : nexus-worker-core ajoutee comme dep de nexus-shell-daemon pour acceder a Invite::mint() et InvitePayload. C'est une dep interne (meme workspace), pas une dep externe. 0 bump externe.

### S2 — decisions historiques
git log `b0656ff` S4 Phase C "invite v2 hard bump" — decision Python pre-pivot (supprimee S50-S51). Le code Rust actuel (worker-core/invite.rs) a deja INVITE_VERSION=2 avec tasks_doc_ticket. Pas de conflit — on cable le cote coordinateur/daemon qui manque. 0 finding.

### S3 — threat model (fast-path)
Le tasks_doc_ticket porte un DocTicket write-access signe par le coordinateur. Le ticket est deja protege par la signature Ed25519 de l'invite (canonical_bytes JCS + DOMAIN_INVITE_V1). Un invite forge ne peut pas injecter un faux ticket. Pas de nouveau composant securite — le ticket utilise l'infra iroh-docs existante. 0 regression threat.

### S4 — wire format (FULL SCAN — touche InvitePayload)
- `INVITE_VERSION = 2` dans worker-core/invite.rs — INCHANGE (le champ tasks_doc_ticket existe deja dans InvitePayload, on ne fait que le remplir cote daemon).
- `DOMAIN_INVITE_V1` dans canonical.rs — INCHANGE.
- Pre-launch policy : pas de bump version, on redefini la v1 courante. Le champ tasks_doc_ticket est deja dans le canonical_bytes signe. Le daemon va simplement le remplir au lieu de laisser None.
- MintRequest (coordinator-rs) : ajout champ tasks_doc_ticket (DB-side, pas wire-format — c'est un champ stockage interne, pas sur le reseau).
- Day 0 preservees : oui.

**S4 verdict** : clean. Le wire format InvitePayload ne change PAS (le champ existe deja). Le changement est cote daemon : remplir le champ au lieu de le laisser vide.

## Plan adaptation (deviation mineure vs plan original)

Le plan §Phase C prevoyait d'ajouter tasks_doc_ticket a MintRequest et InviteRecord. En realite :
- InvitePayload (worker-core) a DEJA le champ tasks_doc_ticket depuis S4
- Le daemon doit appeler Invite::mint() (worker-core) pour generer le wire token signe
- Cela necessite d'ajouter nexus-worker-core comme dep de nexus-shell-daemon
- MintRequest et InviteRecord (coordinator-rs) recoivent le champ pour le stockage DB

Ce n'est pas un PLAN-ADAPT formel (pas d'APPROACH-NAIVE) mais une deviation d'implementation documentee.

## Telemetrie preflight
- Duree totale : ~5m
- S1a : 30s / APPROACH-ALIGNED
- S1b : 20s / +1 workspace dep interne
- S2 : 60s / 1 historical commit S4 (non applicable)
- S3 : fast-path / 30s
- S4 : FULL SCAN / 120s / clean

## Action
Proceder code phase C. Le daemon appelle Invite::mint() via dep nexus-worker-core. Le wire format InvitePayload est inchange.
