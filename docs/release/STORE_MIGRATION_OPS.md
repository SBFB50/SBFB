# STORE_MIGRATION_OPS — migration on-disk redb 2→4 (S81 Phase F)

> Runbook minimal opérateur pour la migration du store iroh au bump
> iroh-docs 0.98 → 0.101 (redb 2.x → 4.1). La procédure de flip LIVE
> complète (ordre des nœuds, gel publish/ingest, convergence par nœud)
> vit dans [`LIVE_FLIP_RUNBOOK.md`](LIVE_FLIP_RUNBOOK.md) (Phase H) —
> ce document couvre uniquement la mécanique de migration et son
> rollback.

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

1. **Snapshot tar AVANT toute migration réelle, daemon ARRÊTÉ**
   (un tar à chaud déchire le WAL SQLite/redb). Le daemon écrit
   **DEUX roots** (cf. `deploy/nexus-shell-daemon.service`) : le tar
   doit couvrir les deux, ou vérifier que `SBFB_HOME` est niché sous
   `NEXUS_GRID_ROOT` (cas VPS : `/var/lib/nexus-grid/.sbfb` — un tar
   du root le capte ; sur dev Win/Mac, vérifier où `SBFB_HOME`
   résout AVANT de tar). Checklist des survivants requis dans
   l'archive : `node_key` (l'IDENTITÉ — 32 octets exactement ; un
   fichier tronqué régénère un `node_id` neuf en warn-only,
   `runtime.rs::load_or_generate_node_key`), `coordinator.db` +
   `-wal`/`-shm`, `docs.redb`, `blobs.db` + `blobs/`, `anchors.json`,
   `subscriptions.json`, `config.toml`, et sous `.sbfb/` :
   `auth_token`, `tokens.json`, `directory_revision.json` (le floor
   anti-rollback — sa perte casse la ré-annonce monotone).
   Vérifier la **restaurabilité** (extract jetable : `node_key`
   présent et 32 octets, `directory_revision.json` présent).
   Snapshots : Windows PRIS (Phase B) ; **Mac PRIS 2026-07-08**
   (`sbfb-snapshots/s81-phase-b/mac-nexus-grid-pre-s81h.tar.gz`,
   contenu vérifié `node_key` + `coordinator.db` + `docs.redb` +
   `blobs.db`, aucun process SBFB actif au tar) ; **VPS À PRENDRE au
   flip** (Phase H, daemon arrêté).
2. **Rollback** (one-way) : **DEUX gestes, jamais un seul** —
   (a) restaurer le tar du root complet **ET** (b) re-déployer le
   **binaire 0.98** conservé côte-à-côte. La migration `docs.redb`
   est AUTOMATIQUE à l'ouverture : restaurer le tar puis rebooter
   sous 1.0.1 **re-migre immédiatement** et rejoue le flip raté au
   lieu de l'annuler. Sur le **VPS Linux, utiliser le TAR, pas le
   rename** du backup (`rename(2)` clobber silencieux — caveat
   Phase F) ; le rename `docs.redb.backup-redb-v2-tuples` →
   `docs.redb` reste un raccourci acceptable sur Win/Mac seulement,
   et toujours accompagné du geste (b).
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

## Inventaire des classes de stores (pattern S82 Phase T)

Chaque classe migre DIFFÉREMMENT ; l'artefact T2 par classe
(`sprint82_t2_store_migration.json` pour les stores worker) juge chaque
classe selon SA mécanique, jamais une règle unique.

| Classe | Fichier(s) | Mécanique de migration | Garde downgrade |
|---|---|---|---|
| iroh docs (daemon/worker standalone) | `docs.redb` | automatique one-way à l'ouverture (redb-v2-migration, temp+swap, sibling backup) | non (one-way, rollback = DEUX gestes, cf. règles ci-dessus) |
| iroh blobs | `blobs.db` | aucune (déjà v3) | n/a |
| local-worker (on-demand) | home local-worker | wipe + reprovision FRESH à chaque spawn (jamais de migration in-place) | n/a par construction |
| coordinator | `coordinator.db` (SQLite) | migrations M1..M19 forward à l'ouverture | non historique |
| **workflow run store (S83 Phase I)** | fichier SQLite injecté par l'hôte (`sbfb-workflow-engine`) | `rusqlite_migration` forward-only à l'ouverture (`user_version`, chaque phase APPENDE un `M::up`, jamais d'édition) ; upcast des events à l'INTERPRÉTATION (jamais de réécriture des rows) | **OUI, double et fail-closed** : `user_version` > migrations connues refuse l'OPEN (`UnsupportedSchemaVersion`) ; `format_version` > courant refuse la LECTURE (`NewerThanSupported`). Backup = `VACUUM INTO` temp+rename ; restore valide le backup AVANT remplacement (inventaire tables+colonnes par version + PK en ensemble exact + FK mono-colonne exacte run_id REFERENCES runs(run_id) + index UNIQUE non-partiel colonnes exactes + quick_check ; non compares : types, NOT NULL, CHECK, defauts, collations, violations FK preexistantes, colonnes surnumeraires) et purge les siblings `-wal`/`-shm` AVANT la copie (anti-corruption), cf. THREAT_MODEL §19 T-RUN-RESTORE-ROLLBACK. |

L'artefact T2 par classe du run store (palier WA-T5.3) reste `NOT-RUN`
jusqu'à AI2 (plan S83 §I.4) ; les ancres hermétiques committées sont les
tests `store_upgrade_replay_*` + `store_backup_restore_round_trip`
(`crates/sbfb-workflow-engine/tests/run_store.rs`).
