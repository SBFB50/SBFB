# LIVE_FLIP_RUNBOOK — migration live iroh 0.98 → 1.0.1 (S81 Phase H)

> Runbook opérateur du flip LIVE de la flotte (dev Windows + Mac M2 +
> ancre VPS Hetzner) du pin iroh 0.98 vers les pins exacts
> `iroh =1.0.1 / iroh-docs =0.101.0 / iroh-gossip =0.101.0 /
> iroh-blobs =0.103.0`. La mécanique de migration du store et son
> rollback vivent dans [`STORE_MIGRATION_OPS.md`](STORE_MIGRATION_OPS.md) ;
> ce document couvre l'ORDRE, la fenêtre, les vérifications et les
> critères STOP. Chemin de données tranché au preflight H :
> **in-place auto-migration** (prouvé Phase F sur COPIE du store VPS
> réel, `70dd845`) — jamais de wipe, jamais de re-install stock.
> Gate calendaire C8 : Phase H pas faite au **15/09** → plan B
> self-hosted ACTIF (`IROH_SELFHOST_OPS.md`).

## Modèle de la fenêtre (à lire avant d'agir)

Le wire docs-sync/gossip 0.98 ↔ 1.0 est **non-rétrocompatible** :
toute paire mixte est totalement partitionnée, quel que soit l'ordre
de bascule. L'ordre ne borne PAS la partition — ce qui la borne, c'est
**same-day en UNE session** + le fait qu'aucun nœud tiers n'existe
(décision PO C4/C5). Le flip est un **flag-day coordonné**, pas un
rolling upgrade. Ce qui garantit l'absence de perte, c'est le triplet :
**tar per-nœud (non-skippable) + in-place prouvé Phase F +
`node_key` jamais régénéré**.

L'ordre **dev Win → Mac → VPS EN DERNIER** reste l'ordre prescrit,
pour trois raisons opérationnelles : (1) downtime minimal de l'ancre
always-on/seeder ; (2) le seeder ne bascule qu'une fois ses deux
partenaires de re-convergence déjà en 1.0 vérifiés entre eux ; (3) le
VPS est le seul nœud dont la migration s'exécute sur Linux (caveat
`rename(2)` clobber, Phase F) — il passe en dernier, quand le chemin
a déjà été éprouvé deux fois dans la même session.

**Gel publish/ingest** : c'est une **discipline opérateur** (aucun
verrou code n'existe) — ne rien publier, ne rien seeder, ne pas
toucher aux subscriptions pendant toute la session de flip.

## Phase 0 — Préparation (avant la fenêtre)

1. **Nommer le binaire VPS et sa chaîne de build.** Chaîne canonique :
   build Linux dans le conteneur `sbfb-ci` (`rust:1.94`,
   classe bookworm — celle qui a produit le binaire E3 vérifié live),
   `cargo build --release --locked -p nexus-shell-daemon`. Enregistrer
   le **sha256** et l'attestation (`scripts/release-attest.sh`).
   **Ne PAS utiliser `deploy/deploy.sh`** : il cible
   `/opt/nexus-grid/bin` + `systemctl restart nexus-daemon`, qui ne
   sont PAS l'unité S75 (`/usr/local/bin/nexus-shell-daemon` +
   `nexus-shell-daemon.service`). Deploy = `scp` manuel + `systemctl
   restart nexus-shell-daemon`.
2. **Conserver le binaire 0.98 côte-à-côte sur les 3 nœuds** (ex.
   `nexus-shell-daemon.098`) — c'est le geste (b) du rollback.
3. `cargo deny check` sur le commit déployé — un yank RC intervenu
   (`ed25519-dalek 3.0.0-rc.0`, watch S82) doit surfacer comme signal
   connu-carrié, pas comme surprise pendant la fenêtre.
4. **Capturer l'identité de référence par nœud** :
   `scripts/acceptance/flip_convergence_check.sh --capture-baseline`
   → note `EXPECT_NODE_ID` (64 hex). Sur le VPS c'est OBLIGATOIRE
   (les locators abonnés pointent sur ce `node_id`).
5. **Capturer la baseline de convergence par app seedée** : même
   commande avec `ARCHIVE_HASH=<blake3>` → note `BASELINE_SHA256`
   (sha256 de `index.html` servie AVANT le flip).
6. **Vérifier qu'aucune variable `SBFB_IDENTITY_SECRET_HEX` ne traîne
   dans l'environnement de la session** (elle courcircuiterait le
   `node_key` fichier, `runtime.rs`).
7. Fixer le **time-box** de la session et le **déclencheur de
   rollback** : si la santé locale ou la convergence dev↔Mac échoue,
   on rollback les nœuds déjà flippés et on NE TOUCHE PAS au VPS.

## Phase 1 — Snapshots tar (non-skippable, 3 nœuds)

8. Sur chaque nœud, **daemon ARRÊTÉ**, tar du/des root(s) selon
   `STORE_MIGRATION_OPS.md` règle 1 (DEUX roots, checklist
   survivants, vérification de restaurabilité). État : Windows PRIS
   (Phase B), Mac PRIS (2026-07-08), **VPS À PRENDRE dans la
   fenêtre** (`systemctl stop nexus-shell-daemon` d'abord).

## Phase 2 — Flip par nœud (dev Win → Mac → VPS dernier)

9. Annoncer le **gel** (discipline : zéro publish/seed/subscribe
   jusqu'à la fin de session).
10. Par nœud, dans l'ordre :
    a. Déployer le binaire 1.0.1 (service/unit INCHANGÉ,
       `start --headless` sur le VPS).
    b. Redémarrer. **Premier boot** : vérifier **0 crash-loop**
       (`systemctl status` / logs), migration `docs.redb` jouée
       (backup sibling `docs.redb.backup-redb-v2-tuples` présent),
       feed/ides/pins M18 intacts.
    c. **Santé locale** :
       `EXPECT_NODE_ID=<réf> ARCHIVE_HASH=<app locale>
       BASELINE_SHA256=<réf> scripts/acceptance/flip_convergence_check.sh`
       → doit sortir `PASS`. Un BLOCK `identity` = **STOP immédiat +
       rollback 2 gestes** (le `node_id` a divergé — tar tronqué ou
       `node_key` régénéré warn-only).
    d. **Convergence cross-nœud** — à partir du 2e nœud flippé
       seulement (le 1er est partitionné du reste par construction) :
       même commande avec `ARCHIVE_HASH=<app de l'AUTRE nœud 1.0>` →
       `PASS` attendu (browse `reachable` + sha256 byte-identique,
       le couple E3). Échec dev↔Mac = rollback des deux, VPS intact.
11. **VPS en dernier**, même séquence (tar pris en Phase 1 → deploy →
    boot → santé locale avec `REQUIRE_NODE_ID=1 EXPECT_NODE_ID=<réf>`
    OBLIGATOIRES — le mode fail-closed refuse de tourner sans la
    référence d'identité, seul backstop automatique de la
    régénération warn-only → convergence cross-nœud re-vérifiée
    depuis dev ET Mac).

## Phase 3 — Post-flip (même session)

12. **Re-annonce** : le boot driver re-annonce le directory + re-pull
    (le floor `directory_revision.json` re-s'applique) ; vérifier sur
    un pair que l'annuaire du VPS est re-vu.
13. Rejouer les paliers d'acceptance transport. Chaque run du harness
    écrit son artefact nommé par nœud/palier, ex. :
    `FLIP_ARTIFACT=scripts/acceptance/.flip_vps_local.json …` ; les
    verdicts JSON sont ensuite agrégés dans l'artefact T2 committé
    `.planning/active/sprint81_t2_h_live_flip.json` (même façon que
    les paliers E2/E3 — jamais de verdict en prose).
14. **Après convergence saine vérifiée sur les 3 nœuds** : supprimer
    `docs.redb.backup-redb-v2-tuples` sur chaque nœud
    (`STORE_MIGRATION_OPS.md` règle 4 — ré-arme le self-heal normal
    et purge l'ancien `NamespaceSecret`).
15. Consigner le flip dans `THREAT_MODEL.md` §15.5 (déjà rédigée —
    vérifier que les résiduels observés collent) et mettre à jour
    les artefacts de planning.

## Rollback (à tout moment de la fenêtre)

Procédure exacte : `STORE_MIGRATION_OPS.md` règle 2 — **DEUX gestes**
(restore tar du root complet **ET** re-deploy binaire 0.98), tar (pas
rename) sur le VPS, puis vérif `0 crash-loop` + `node_id` == référence
+ apps servies.

## GO / NO-GO (checklist bloquante avant d'ouvrir la fenêtre)

- [ ] Phase F = PASS (gate R2) — SATISFAIT (`70dd845`, T2 F PASS)
- [ ] Snapshots tar Win + Mac PRIS — SATISFAIT (Phase B + 2026-07-08)
- [ ] Snapshot tar VPS pris **daemon arrêté** + restaurabilité
      vérifiée (`node_key` 32 octets, `directory_revision.json`)
- [ ] Binaire 1.0.1 VPS nommé + sha256 + attestation ; binaire 0.98
      conservé côte-à-côte sur les 3 nœuds
- [ ] `EXPECT_NODE_ID` capturé par nœud + `BASELINE_SHA256` capturée
      par app seedée (harness `--capture-baseline`) ; sur le VPS le
      harness tournera en `REQUIRE_NODE_ID=1` (fail-closed)
- [ ] `cargo deny check` joué sur le commit déployé
- [ ] Aucune `SBFB_IDENTITY_SECRET_HEX` dans l'environnement
- [ ] Fenêtre same-day UNE session réservée ; gel annoncé ; time-box
      + déclencheur rollback fixés
- [ ] Harness `flip_convergence_check.sh` présent sur les 3 nœuds

**STOP immédiat (→ rollback) si** : divergence `node_id` au 1er boot |
crash-loop | échec convergence dev↔Mac au 2e flip | `node_key`
absent/tronqué au restore | migration interrompue sans backup ni tar.
