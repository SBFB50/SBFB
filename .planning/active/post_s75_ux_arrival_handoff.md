# Handoff — UX d'arrivée hybride (décision PO 2026-06-11, à implémenter)

> Décision PO prise en session de test live post-S75 (PC+Mac+VPS) :
> **Option C hybride, avec une limite de refresh pour éviter les spams.**
> NON implémentée — ce document est le design d'entrée pour la session qui
> l'attaquera (lot UX du kickoff S76, ou mini-cycle dédié sur demande PO ;
> l'audit gate S75 en Phase 0 reste premier). Preflight G8 OBLIGATOIRE avant
> code : nouveau comportement d'ingest = nouvelle surface.

## 1. La décision (verbatim intention PO)

- Le chemin d'arrivée : *j'arrive → je vois les nœuds du réseau → je
  m'abonne → leurs projets apparaissent → j'ouvre (téléchargement à la
  demande) → je peux « garder en ligne »*.
- **Rien n'est jamais pré-installé** (mécanique actuelle confirmée au PO :
  fiches de découverte seulement, octets fetchés au 1er « Ouvrir » puis
  cache local). Ce point est ACQUIS, ne pas le changer.
- **Hybride** : la grille `/browse` = MES sources (own + abonnés) ; les
  fiches poussées non sollicitées vont dans une section SÉPARÉE clairement
  étiquetée « Découvert sur le réseau » — jamais mélangées.
- **Page `/nodes` enrichie** : en plus des abonnés, lister les nœuds
  OBSERVÉS (annuaires entendus par gossip sans abonnement) avec CTA
  « S'abonner ».
- **Limite de refresh anti-spam** (exigence PO explicite) : l'ingest des
  annonces non sollicitées est rate-limité.

## 2. Design proposé (chemins de code exacts, état 2026-06-11 `173426e`)

### 2.1 Daemon — registre « nœuds observés » (RAM-only, borné)
- Aujourd'hui le bras directory DROPPE les `NodeDirectoryAnnouncement` de
  pubkeys non-abonnées (gate partagé `verify_signed_list_ingest`,
  subscription-gated, `iroh_runtime.rs`). NE PAS ingérer leur CATALOGUE
  (contenu non sollicité) — retenir des MÉTADONNÉES seulement :
  `observed_directories: {node_id, revision, app_count, last_seen}`.
- Signature Ed25519 TOUJOURS vérifiée avant rétention (le PoW gossip
  existant reste le 1er filtre).
- **Bornes anti-spam (la « limite de refresh » PO)** — pattern SeedRegistry
  S75-D (SEED-1/SEED-2) : cap nombre de nœuds observés (ex. 256, eviction
  stalest) ; TTL (ex. 48h, purge paresseuse) ; **rate-limit par node_id**
  (1 mise à jour acceptée / fenêtre, ex. 60 s — vérifier le throttle
  existant `process_directory_announcement_bytes_throttled` Phase C comme
  base) ; clamp `last_seen = min(now, claimed)` IN-REGISTRY (§P59.2 :
  enforce dans la primitive, pas chez l'appelant).

### 2.2 Route `/api/daemon/nodes` — clé additive `observed`
- Enveloppe actuelle `{nodes:[…]}` avec Zod `.strict()` ENVELOPPE côté
  front : ajouter `observed:[{node_id, revision, app_count, last_seen}]`
  = MAJ schéma front DANS LE MÊME COMMIT (loopback-local, pas un wire P2P,
  0 bump ; précédent `self_pin_enabled` F).

### 2.3 Daemon — flag `from_subscribed` sur /browse (serialize-only)
- La grille front ne peut pas distinguer un `direct` d'un nœud abonné d'un
  `direct` d'inconnu : `node_id` est `#[serde(skip)]`. Exposer
  `from_subscribed: bool` via le flatten view `BrowseEntryView` (pattern
  `is_own`, §P58.2 — ZÉRO churn des sites de construction), calculé à la
  sérialisation : `entry.node_id ∈ attention set || is_own`.

### 2.4 Front
- `/browse` : grille principale = `is_own || source ∈ {curator,
  nodedirectory} || (direct && from_subscribed)` ; le reste → section
  « Découvert sur le réseau » séparée, cappée à l'affichage (ex. 24 fiches,
  plus récentes d'abord), copy honnête « annoncé sur le réseau — non
  sollicité ». La dédup `dedupeBrowseEntries` (`fdb4fb1`) s'applique aux
  deux groupes.
- `/nodes` : section « Nœuds découverts sur le réseau » (observed), CTA
  S'abonner = `addAnchor` EXISTANT ; copy « s'annonce sur le réseau —
  abonne-toi pour voir son catalogue ». Lignes en-attente existantes
  inchangées.

### 2.5 Garde-fous
- Verrous 1-5 inchangés (rien de pré-rempli, rien de hard-codé, additive
  jamais substitutive — la section découverte est l'AMBIANT opt-in visuel,
  la grille reste le superset de MES sources). 0 bump `*_FORMAT_VERSION`,
  0 nouveau DOMAIN, 0 dep. Tests : Rust (registre borné : cap/TTL/
  rate-limit/eviction/clamp + flag from_subscribed) + Vitest (split
  grille/section, nodes observed + CTA).

## 3. État de la topo de test live (au moment du handoff)

- PC Windows : daemon `--web-root web/dist` port 7654, node `fe7a4898…`,
  4 apps, annuaire rev10. Mac : daemon root `~/sbfb-test`, web-root
  `/tmp/sbfb-dist`, port 49798, node `182dfeb9…`, abonné PC+VPS (config
  AVANT boot — un subscribe post-boot ne re-joint PAS le swarm gossip,
  bootstrap figé au boot `runtime.rs:1080` — piège connu). VPS : ancre
  systemd active, seede sbfb-explorer.
- 2 hotfixes Cas D committés cette session : `fdb4fb1` (dédup grille) +
  `173426e` (keep-online réconcilié au mount). HEAD `173426e`, 19 ahead,
  RIEN POUSSÉ (Docker canonique déjà vert 1759/1759 ; push = décision PO).
