# Sprint 82 — Phase A acceptance (boot-SEED, S81-G-ESC-1)

Gate de testabilité par-sprint (README §4), type **BOOT-SEED (GATE PLEIN)**.
Invariant Day-0 D2 : « broadcast gossip = HINT ; état durable synchronisé =
VÉRITÉ ; tout consommateur cold-boot RÉCONCILIE ».

## Acceptance

### T1 — hermétique 2-nœuds (verdict ∈ {GREEN, RED, N-A-no-frontend-change})

**GREEN.** Prérequis DUR du commit, tous verts (nextest Windows natif) :

- **ANCRE (red→green + revert-proofs)** — `nexus-shell-daemon`
  `http::tests::redrive_on_ingest_pins_configured_app_without_restart` :
  - CONTROL (red) : sans annuaire ingéré, un re-drive ne pinne RIEN (0 pinned,
    pas de tag) — la « first-boot dead window » reproduite.
  - FIX (green) : après ingest de l'annuaire d'un ancre abonné couvrant le pid
    `keep_online`, le re-drive pinne l'app SANS restart (`blob has()==true` +
    tag skip-GC + row `keep_online` = `(true, hash)`).
  - Revert-proofs : cooldown coalesce une 2e ingest immédiate (`None`) ;
    accept-list vide ne re-drive jamais (`None`).
- **WORKER (cadence, revert-proof structurel)** — `nexus-core-rs`
  `doc_sync::tests::cold_boot_config_accelerates_only_the_cold_window` :
  la cadence cold-boot est strictement plus agressive que le backstop steady
  tant que `!warm`, puis relâche à l'EXACT backstop S77 ; `default()` garde
  cold == steady (0 changement observable). Revert `cold_boot_aggressive()` →
  `default()` fait échouer les inégalités strictes. **Portée (review P2-4)** :
  ce test garde le CONTRAT DU CONSTRUCTEUR ; la ligne d'opt-in worker
  (`nexus-worker-core/engine/runtime.rs`, la seule ligne comportementale
  fermant S81-K) n'est prouvée que par le T2 live.
- **WORKER (mécanisme rejoin 2-nœuds)** — `nexus-core-rs`
  `doc_sync::tests::keepalive_rejoins_doc_after_neighbor_loss` : re-join réel
  sur perte de voisin, red→green, exercice de la boucle spawn refactorée
  (`check_interval_for`/`min_rejoin_for`).

**Honnêteté testabilité (leçon Phase A).** Le BÉNÉFICE de convergence de la
cadence cold-boot est une propriété **transport-only** : en in-process, un
`start_sync` en attente résout son dial en arrière-plan dès que l'adresse
apparaît dans le memory-lookup, donc la fréquence de rejoin est indistinguable
hermétiquement (un test de convergence par injection d'adresse retardée passe
GREEN pour les DEUX cadences — vérifié : le contrôle steady convergeait). La
convergence WORKER est donc prouvée par le T2 live, pas par un faux
red-before-green hermétique (cf. PATTERNS §P74).

### T2 — re-jeu live JSON (status ∈ {PASS, BLOCK{diagnosis}, RIG-ABSENT})

**PASS — escalade OVERDUE 3/3 FERMÉE (PO-1=B).** `sprint82_t2_bootseed.json`,
`status=PASS`, `delay_s=18` (< budget 30s), `stage=result-replication`, attribué
au worker froid cible (`result_text="Paris."` + `result_hash`). Re-jeu live
cross-machine `BOOT_AFTER_SUBMIT=1` joué le 2026-07-13 (Mac rallumé par l'opérateur) :
tâche soumise au coordinateur VPS → worker froid booté APRÈS le submit → il a
rattrapé le `task:` déjà pendant et produit le résultat en **18 s** via la cadence
cold-boot agressive + le rattrapage de voisinage. L'attribution passe (le log du
worker froid porte `"result written"` pour ce task_id — Codex P1-3), donc le
résultat est bien produit par LE worker froid cible, pas un concurrent. **La preuve
LIVE (c) PO-1=B est fournie : l'escalade `S81-G-ESC-1` (OVERDUE 3/3) est CLOSED**
(code + T1 hermétique GREEN + T2 live PASS<30s).

Commande de re-jeu (depuis le PC, `rig.local.env` configuré) :
```
BOOT_AFTER_SUBMIT=1 \
  B3_ARTIFACT=.planning/active/sprint82_t2_bootseed.json \
  bash scripts/acceptance/b3_live_pc_vps.sh
```
