# STORE_MIGRATION_OPS — migration on-disk redb 2→4 (S81 Phase F)

> Runbook minimal opérateur pour la migration du store iroh au bump
> iroh-docs 0.98 → 0.101 (redb 2.x → 4.1). La procédure de flip LIVE
> complète (ordre des nœuds, gel publish/ingest, convergence par nœud)
> est un livrable **Phase H** — ce document couvre uniquement la
> mécanique de migration et son rollback.

## Ce qui se passe à l'ouverture

- `blobs.db` : **aucune migration**. Le fichier est déjà au format
  redb v3 et toutes les tables iroh-blobs sont non-tuple — il s'ouvre
  tel quel sous iroh-blobs 0.103.
- `docs.redb` : migration **automatique et one-way** à l'ouverture
  (feature défaut `redb-v2-migration` d'iroh-docs). Mécanisme :
  temp-file + swap — jamais in-place. Coût disque ~2× pendant la
  migration. À la fin, l'original est conservé en
  `docs.redb.backup-redb-v2-tuples` à côté du fichier migré.

## Règles opérateur

1. **Snapshot tar AVANT toute migration réelle** (`NEXUS_GRID_ROOT`
   complet : `docs.redb` + `blobs/` + `coordinator.db*` + `node_key`).
   Snapshots Phase B : Windows pris ; **Mac PENDING — aucun boot Mac
   avant son snapshot**.
2. **Rollback** (one-way) : restaurer le tar, OU renommer
   `docs.redb.backup-redb-v2-tuples` par-dessus `docs.redb`.
3. **Crash pendant la migration** : une fenêtre existe entre le rename
   et le persist — `docs.redb` est alors absent et un reboot créerait
   un store vide. Le daemon détecte ce cas (S81 Phase F) : si le backup
   existe alors qu'un replica M8 est introuvable, le boot **refuse** le
   recreate avec un message diagnostic. Remède : restaurer le backup
   (règle 2) puis rebooter.
4. **Après une migration VÉRIFIÉE** (boot OK, namespaces présents,
   apps servies) : **supprimer** `docs.redb.backup-redb-v2-tuples`.
   Tant qu'il traîne, le garde-fou ci-dessus reste armé et un replica
   légitimement absent refusera de s'auto-recréer (fail-loud
   récupérable, jamais une perte). Le backup contient l'ancien
   `NamespaceSecret` — le supprimer est aussi une mesure d'hygiène
   secrets.
5. Un fichier temporaire `docs.db.migrate*` orphelin dans le dossier
   iroh signale une migration interrompue avant le swap — supprimable
   sans risque (l'original n'a pas encore bougé).
