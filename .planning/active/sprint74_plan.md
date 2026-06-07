# Sprint 74 — Plan d'execution (Atelier fork + Disponibilite/Hosting complet)

**Ecrit** : 2026-06-07 (apres kickoff, avant 1er commit feat/fix).
**Kickoff** : `sprint74_kickoff.md` (D1..D5 gelees ; arbitrages produit
Checkpoint §11 a rendre AVANT de figer Phases E-F).
**Design review** : `sprint74_design_review.md` (G1, D1 ⚠️ D2 ✅ D3 ⚠️ D4 ⚠️
D5 ✅).
**Design produit fige** : `.planning/research/s74_disponibilite_ux_design.md`
(D-DISPO, mockups, copy FR, 5 verrous anti-recentralisation, phasage).
**Roadmap** : v5 Factory Complete Vision, Arc 3.5, sprint **4/6** (atelier
fork + pull-forward LT-5 redundancy persistence).

---

## §1 Etat verifie a l'entree

**Tip master** : `a53b9f6` (`fix(shell): auto-register the same-origin daemon
as the default coordinator`). **37+ ahead origin, rien pousse** (pre-launch
§1.4). 9 hotfixes Cas D locaux depuis l'audit gate S73 (`47b8c59`..`3b7ef54`
daemon #1-#8 + `a53b9f6` shell).

| Suite | Count entree | Source |
|---|---|---|
| Rust nextest (canonique CI Linux) | **1570** | audit S73 Docker sbfb-ci 1570/1570 0-skip |
| Rust nextest (Windows natif) | 1566 | ecart +4 = `#[cfg(unix)]` structurel |
| Vitest (`web/`) | **294** | 289 audit S73 + 5 `bootstrap.test` (hotfix `a53b9f6`) |
| Vitest `factory-operator` | 7 | infra NEW S73 Phase B |
| size-limit | 6/6 | — |
| clippy workspace | 0 warning | audit S73 |
| fmt | exit 0 | audit S73 |
| **`web/` test:coverage** | **ROUGE pre-existant** (79.9/71.89/78.68/78.73 < 85/90/78/85) | **T14 — masque par `\| tail`, CI-enforced, A TRAITER Phase G** |

**Re-mesure obligatoire** : au demarrage reel (Phase A preflight) sur le SHA
post-kickoff. Le compte Rust reste 1570 canonique (le kickoff n'ajoute pas
de test) ; le shell hotfix a deja ajoute +5 Vitest (comptes).

**Etat infra (cartographie kickoff §Sources)** :
- **Triplet provenance** (S73) dans `SearchResult` → alimente le fork.
- **`deploy.rs::publish_announcement`** (#8 `3b7ef54`) = helper canonique
  broadcast→persist-outbox→index→cache. Voie du re-deploy/fork-redeploy.
- **`restore_browse_from_outbox`** (#7, `runtime.rs:1750-1771`) = re-annonce
  au boot prouve. Pattern du pin persistant + re-annonce de seed.
- **`blobs.rs::add_bytes:77-88`** tag deja (skip-GC) ; **`fetch_ticket:140-163`
  ne tag PAS** le blob fetche → **gap pin-cross-noeud** (Phase E).
- **`node.rs`** : FsStore (redb) persiste les blobs ; Router 3 ALPN
  (`blobs/gossip/docs`) → un 4e ALPN `sbfb/seed/0` (Phase E).
- **`deploy.rs:445-457`** : faux-vert NAT (self→Reachable, last_probed_at:None).
- **Gaps** : cablage fork (B-C) ; panneau Disponibilite front (A) ; pin local
  M18+tag+re-annonce (D) ; seed cross-noeud `SeedRequest`+tag+invite+approbation
  +registre (E-F).

---

## §2 Decisions Day 0 (gelees — rappel)

| D | Decision | Implication code |
|---|---|---|
| D1 | Seed cross-noeud = `SeedRequest` Ed25519+JCS sur ALPN dedie `sbfb/seed/0` (demande point-a-point, pas op feed) | NEW `seed.rs` (types+sig+handler ALPN), `blobs.rs` (tag blob fetche), `node.rs:341-344` (.accept) |
| D2 | Pin local = table `keep_online` (M18 local) + tag/protect + re-annonce boot (#7) | `db.rs` M18, `deploy.rs`/`publish.rs`, `runtime.rs:1750-1771`, `blobs.rs`, route `POST /api/daemon/keep-online` |
| D3 | Registre seed = op feed `SeedAnnounced` (raw-op `Value`, 0 bump) + compteur best-effort « Toi + N pairs » TTL | `public_feed.rs` (helper raw-op), `feed_sync.rs` (ingest), `seed_registry`, front panneau |
| D4 | Faux-vert NAT « vu de ton noeud » + invitation revocable (Tailscale) + approbation cote pair (Syncthing) | `browse.rs`/front libelle, NEW invite seed token, handler `SeedRequest` gate approbation, front |
| D5 | Ampleur : A-D surs (fork+dispo+pin local) ; E-F cross-noeud borne ; jamais un faux bouton actif | phasage strict, criteres binaires, A-D livrables si E-F debordent |

---

## §3 Research consulte

- **Seed/replication/approbation (D1/D3/D4)** : Radicle Heartwood (seeding
  policy + delegates ≠ seeders + `--scope followed`), IPFS Cluster (CRDT
  pinset + `replication_factor_min` + allocations/re-allocation), Tailscale
  (invite revocable single-use/reusable expire 30j + quarantine par defaut),
  Syncthing (approbation explicite de pair + introducer), IPFS reprovide
  (provider records 22h/48h → re-annoncer = cout recurrent reel). Convergence :
  ALPN req/resp pour la demande (point-a-point), op feed raw-op pour le fait
  observable, invitation revocable + approbation, seeder ≠ co-auteur.
- **Pin local (D2)** : IPFS pinset + reprovide ; pattern #7 in-repo
  (`restore_browse_from_outbox`). Re-annonce-au-boot prouve.
- **Code lu (file:line)** : 7 cartographies, voir kickoff §Sources +
  `sprint74_design_review.md`.

**Dependances inter-phases** :
```
A (Disponibilite front + rename)  ── independant (primitives S73)
B (fork backend: workspace + clone/blob)  ──┐
                                            ├─→ C (fork → redeploy identite locale)
                                            │
D (pin LOCAL: M18+tag+re-annonce boot)  ────┴─→ E (seed cross-noeud: SeedRequest+tag+invite+approbation)
                                                     └─→ F (re-annonce distante + registre SeedAnnounced + compteur)
G (wrap-up + dette T14 + carries audit)  ── apres A-F
```
A independant (front). B → C (C re-deploie le workspace de B). D (pin local)
precede E (E etend le pin au cross-noeud). E → F (F = re-annonce distante +
registre, le cran le plus aval). **A-D = segment SUR ; E-F = segment cross-noeud
BORNE (R1, D5).** Si E-F debordent, A-D livrables.

---

## Phase A — Disponibilite front (D-DISPO segment 1) + rename « coordinateur »→« noeud »

### A.1 Scope
100 % front, primitives S73 existantes. Carte succes publish (remplace le
`<dl>` Hash/Provenance/Commit `Deploy.tsx:151-174`) + ligne de verite sous
CTA (« Ton noeud signe cette app et la garde en ligne… »), hashs replies
(« Details techniques ») ; **0 champ hote**. Bouton « Disponibilite »
(remplace le badge `blob:<hash>`) → **Sheet lateral** shadcn glass : Section
AUTEUR scellee / Section ETAT (mapping reachable→« En ligne »/unreachable→
« Hors ligne »/unknown→« Verification… ») / Section QUI-LA-GARDE. Toggle
« Garder en ligne » **lecture-seule** (ON honnete, OFF = 2e passe Phase D).
**App DISTANTE** : action « Garder en ligne — soutenir ce projet »
presentationnelle/« Bientot » (seed VOLONTAIRE communautaire, amendement PO
2026-06-07 ; fonctionnel D+F) — remplace le « Ce noeud (consultation) »
lecture-seule du design §6/§7.
Rappel hors-ligne conditionnel (greffe A, 1x/session/app dismissible, mes apps
seulement) + placeholder « app tombee » (greffe D) → `/deploy` prerempli.
**Rename « coordinateur »→« noeud »/« reseau »** (AppShell CoordinatorPicker,
`daemon.ts` var interne, nav, AddCoordinatorDialog, OnboardingEmpty) — copy
D-DISPO §6 (« Aucun noeud actif », « Publier », « Se connecter a ce noeud »).
**5 verrous anti-recentralisation cables UI** (D-DISPO §8). Strings FR §6.

### A.2 Fichiers touches
| Fichier | Role |
|---|---|
| `web/src/pages/Deploy.tsx` (151-174) | Carte succes (remplace `<dl>` brut) + ligne de verite + repli « Details techniques » ; 0 champ hote. |
| `web/src/components/AvailabilitySheet.tsx` (NEW) | Sheet lateral : AUTEUR scelle / ETAT (probe) / QUI-LA-GARDE / Copies de secours (« Bientot » inerte). |
| `web/src/pages/ProjectDetail.tsx` / `BrowsedProject.tsx` | Bouton « Disponibilite » (remplace badge `blob:<hash>`) → ouvre le Sheet ; rappel hors-ligne + app-tombee. |
| `web/src/components/AppShell.tsx` (CoordinatorPicker) | Rename « coordinateur »→« noeud »/« reseau » + copy intentions. |
| `web/src/api/daemon.ts` | Rename var interne ; (lecture seule Phase A, pas de mutation). |
| `web/src/pages/OnboardingEmpty.tsx` + `AddCoordinatorDialog.tsx` | « Aucun noeud actif » + « Se connecter a ce noeud » (D-DISPO §6). |

### A.3 Tests plan (Vitest `web/`)
1. `availability_sheet_renders_author_state_seeders` — 3 sections rendues.
2. `publish_success_card_folds_hashes` — carte succes, hashs replies, 0 champ hote.
3. `availability_state_maps_reachable_unreachable_unknown` — mapping pastille.
4. `offline_reminder_only_for_own_apps_dismissible` — greffe A conditionnelle.
5. `coordinator_renamed_to_node_in_shell` — « noeud »/« reseau », plus de « coordinateur » visible.
6. `keep_online_toggle_readonly_in_phase_a` — toggle ON honnete, pas de mutation.

### A.4 Critere d'acceptation
```
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  bash scripts/scan-en-strings.sh)
```
Panneau rend AUTEUR/ETAT/QUI-LA-GARDE ; rename propage ; 0 champ hote ; 0
faux bouton actif ; strings FR. G1 design_review present (gate Phase A).

### A.5 Commit cible
`feat(shell): Sprint 74 Phase A — availability panel (read) + clean publish card + rename coordinator→node`
Body : 9 sections (Contexte / Fichiers / Delta Vitest +6 / Verification §7.4 /
Scope cuts respectes / G8 traceability / Pre-launch protocol / Codex
verification / Carry closure rename + D-DISPO segment 1).

---

## Phase B — Atelier fork backend : workspace cible + clone forge / reconstruction blob (PO-5)

### B.1 Scope
Notion de **projet cible distinct du repo nexus** (`process::repo_root`
pointe nexus, G17). Depuis le triplet S73 : clone `repo_url@commit_sha`
(forge) **ou** reconstruction depuis le blob `archive_hash` (repli :
`fetch_ticket` + unzip) → nouveau workspace atelier. **Pre-requis
browse-indexing (audit S73, AVANT tout cablage prod)** : **rowid partition
browse/feed** (C.3 — partitionner l'espace rowid avant que le browse-indexing
prod ne clobbe les upserts feed) + **re-application de l'invariant
`is_open_source⇒provenance_hash`** au chemin browse (B.6).

### B.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/sbfb-factory/src/fork.rs` (NEW) ou `process.rs` | `fork_from_search_hit(triplet)` : clone forge OU reconstruction blob → workspace cible distinct nexus. |
| `crates/sbfb-factory/src/process.rs` (`repo_root`, G17) | Notion de projet cible distinct (workspace forke, pas le repo nexus). |
| `crates/nexus-coordinator-rs/src/search.rs` (index_entry, rowid) | **C.3** : partition rowid browse/feed (browse-rows hors espace seq feed). |
| `crates/nexus-shell-daemon/src/http.rs` (index_browse_entry) ou `deploy.rs` | **B.6** : re-appliquer `is_open_source⇒provenance_hash` au browse-index. |

### B.3 Tests plan
1. `fork_from_forge_clones_repo_at_commit` — clone `repo_url@commit_sha` → workspace.
2. `fork_from_blob_reconstructs_archive` — reconstruction `archive_hash` (fetch+unzip) → workspace.
3. `fork_target_workspace_distinct_from_nexus_repo` — `repo_root` ≠ nexus.
4. `browse_rowid_partitioned_from_feed_seq` (C.3) — un upsert feed seq=N ne clobbe pas une browse-row.
5. `browse_index_rejects_open_source_without_provenance` (B.6) — invariant re-applique.

### B.4 Critere d'acceptation
```
cargo nextest run -p sbfb-factory -p nexus-coordinator-rs -p nexus-shell-daemon --locked -E 'test(fork) + test(rowid) + test(provenance)'
```
Hit search → workspace forke (forge OU blob) ; rowid partition ; invariant
provenance re-applique.

### B.5 Commit cible
`feat(factory): Sprint 74 Phase B — fork a network project into a target workspace (forge clone or blob reconstruction)`
Body : 9 sections, delta +5, G8 (PO-5), scope cut #8 (pas de Monaco) respecte,
carry closure C.3 + B.6.

---

## Phase C — Atelier fork → REDEPLOY sous identite locale + boucle UI

### C.1 Scope
`reseau→atelier→redeploy` : le workspace forke (B) est re-deploye via le
helper canonique `deploy.rs::publish_announcement` (#8) → **provenance
re-signee par MON noeud** (le fork EST un nouvel acte d'auteur local ; seeder
≠ co-auteur). Bouton UI « Forker dans l'atelier » + « La remettre en ligne »
(app tombee, greffe D → `/deploy` prerempli, **re-signature coherente fork** —
arbitrage D4/Checkpoint Q3). Templates : static + static-reader + react +
pyodide (4 templates — **PO arbitre Q7 = react/pyodide INCLUS S74**). **OFF-SPRINT-2/2b** (audit S73) : test
non-regression deploy per-app + completer per-app project_id sur /publish
(`http.rs:1004`) + gossip (`runtime.rs:1569`, `publish.rs:39`) — sinon le
re-deploy collisionne (R7).

### C.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/sbfb-factory/src/fork.rs` + `crates/nexus-shell-daemon/src/deploy.rs` | Redeploy du workspace via `publish_announcement` ; provenance re-signee locale. |
| `crates/nexus-shell-daemon/src/http.rs` (1004) + `runtime.rs` (1569) + `publish.rs` (39) | **OFF-SPRINT-2b** : per-app project_id sur /publish + gossip. |
| `web/src/pages/Browse.tsx` / `BrowsedProject.tsx` | Bouton « Forker dans l'atelier » + « La remettre en ligne » (greffe D). |

### C.3 Tests plan
1. `fork_redeploy_resigns_provenance_as_local_node` — provenance auteur = MON noeud.
2. `fork_redeploy_loop_e2e_single_node` — chercher→forker→redeploy (E2E mono-noeud).
3. `deploy_per_app_distinct_browse_cards` (OFF-SPRINT-2) — multi-app → cartes distinctes (non-regression).
4. `publish_and_gossip_use_per_app_project_id` (OFF-SPRINT-2b) — plus de node_id sur /publish+gossip.
5. `remettre_en_ligne_prefills_deploy` (front) — app tombee → `/deploy` prerempli.

### C.4 Critere d'acceptation
```
cargo nextest run -p sbfb-factory -p nexus-shell-daemon --locked -E 'test(fork_redeploy) + test(per_app)'
(cd web && npm run test:unit)
```
Boucle chercher→forker→editer→redeploy prouvee (E2E mono-noeud) ; provenance
du fork = identite locale ; OFF-SPRINT-2/2b traites. Depend de Phase B.

### C.5 Commit cible
`feat(factory): Sprint 74 Phase C — redeploy a fork under local node identity + per-app project_id on publish/gossip`
Body : 9 sections, delta +5, G8 (PO-5), carry closure OFF-SPRINT-2 + 2b,
scope cut #8 respecte.

---

## Phase D — Pin local persistant (D2) : `keep_online` M18 + tag/protect + re-annonce boot + toggle fonctionnel

### D.1 Scope
Migration **M18** `keep_online` (local : `project_id` PK, `enabled`,
`archive_hash`, `pinned_at`) ; toggle « Garder en ligne » **fonctionnel**
(ON/OFF) via `POST /api/daemon/keep-online` (loopback auth) ; tag/protect du
blob selon `enabled` (skip-GC) ; **re-annonce au boot** : `restore_*`
(`runtime.rs:1750-1771`) lit `keep_online` (en plus de l'outbox) + re-broadcast
pour les apps gardees en ligne. OFF → tag retire (le blob peut etre GC'd) +
arret re-annonce (UX « stockee mais plus diffusee »). Carries audit ici :
**H.1 M17 boot-recovery non-silencieuse** (recovery loguee/metree, pas warn
noye) + **H.2 reconstructibilite browse-rows** (documenter / carry si depend
browse-indexing B).

### D.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` (228-303, M-pattern) | **M18** `keep_online` (ADD TABLE local) + getters/setters. |
| `crates/nexus-shell-daemon/src/deploy.rs` / `publish.rs` | Set `keep_online=true` au self-deploy. |
| `crates/nexus-shell-daemon/src/runtime.rs` (1750-1771) | `restore_*` lit `keep_online` + re-annonce ; **H.1** recovery non-silencieuse. |
| `crates/nexus-core-rs/src/blobs.rs` (77-88) | Tag conserve si `enabled`, retire sinon (skip-GC). |
| `crates/nexus-shell-daemon/src/http.rs` | Route `POST /api/daemon/keep-online` (toggle, loopback auth). |
| `web/src/components/AvailabilitySheet.tsx` | Toggle « Garder en ligne » **fonctionnel** (remplace lecture-seule Phase A). |

### D.3 Tests plan
1. `keep_online_toggle_persists_m18` — ON → ligne M18 ; OFF → `enabled=false`.
2. `pinned_app_reannounced_on_boot` — `keep_online=true` → re-annonce au reboot simule (pattern #7).
3. `keep_online_off_removes_tag` — OFF → tag retire (blob GC-eligible).
4. `migration_m18_creates_keep_online_table` — M18 verte (upgrade reel user_version).
5. `m17_boot_recovery_not_silent` (H.1) — echec rebuild post-DROP → log/metrique eleve, pas warn noye.

### D.4 Critere d'acceptation
```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon -p nexus-core-rs --locked -E 'test(keep_online) + test(pinned) + test(m18) + test(boot_recovery)'
```
Toggle ON → app re-annoncee au boot ; OFF → tag retire ; M18 verte ; le pin
survit a un redemarrage du daemon. **PIN LOCAL** (le cross-noeud = E-F).

### D.5 Commit cible
`feat(daemon): Sprint 74 Phase D — persistent local pin (keep_online M18 + blob tag + boot re-announce)`
Body : 9 sections, delta +5, G8 (D2), carry closure H.1 (H.2 documente),
scope cut #5 (pas de timer 22h) respecte.

---

## Phase E — Seed cross-noeud (D1+D4 segment 2) : `SeedRequest` ALPN + fetch+tag+pin + invitation/approbation

### E.1 Scope
**Le pull-forward LT-5 — segment a risque (R1/R2/R3).** NEW protocole ALPN
`sbfb/seed/0` : `SeedRequest`/`SeedResponse` signes **Ed25519+JCS+nonce**
(domain-constant dedie ; composition de `canonical.rs`, [DETER] crypto-spec au
**preflight Phase E**) ride par le Router (`node.rs:341-344`). Le seeder
**verifie sig + invitation (D4) + approuve** (cote pair, modele Syncthing) →
`fetch_ticket` → **tag/protect** (corrige `blobs.rs:140` gap, R3) → persiste
`keep_online` cote seeder (reutilise D2) → re-annonce. Invitation **revocable**
(D4 vol.2, token signe distinct de la cle de noeud, modele Tailscale).
Faux-vert NAT libelle « En ligne (vu de ton noeud) » (D4 vol.1, `deploy.rs:456`).
**Si deborde → arbitrage Checkpoint Q1 (slice de repli).**

**Seed VOLONTAIRE communautaire (amendement PO 2026-06-07)** : en plus de
l'invitation auteur->pair, tout noeud consultant une app publique peut s'auto-elire
seeder (« Garder en ligne — soutenir ce projet », **SANS approbation auteur** —
contenu deja public, sur par content-addressing blake3, seeder!=auteur). **PAS de
`SeedRequest`** (acte unilateral local) : reutilise **D** (pin d'un blob DISTANT
fetche) + **F** (`SeedAnnounced`). C'est le chemin always-on **PRINCIPAL** et le
moins risque -> atterrit **des D+F**, AVANT la crypto d'invitation E. L'invitation
authentifiee E reste le complement « designer MA machine (VPS) / un pair specifique ».
Tests add : `voluntary_seed_distant_public_app_no_approval` + `voluntary_seeder_serves_author_provenance_intact` (content-addressed, pas de re-provenance).

### E.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/seed.rs` (NEW) ou `nexus-core-rs/src/seed.rs` | `SeedRequest`/`SeedResponse` types + signature Ed25519+JCS + nonce + handler ALPN. |
| `crates/nexus-core-rs/src/node.rs` (341-344) | `.accept("sbfb/seed/0", SeedProtocol::new(...))`. |
| `crates/nexus-core-rs/src/blobs.rs` (140-163) | **R3** : tag/protect le blob fetche (`fetch_and_pin` ou tag dans `fetch_ticket`). |
| `crates/nexus-coordinator-rs/src/` (invite pattern) | Token d'invitation seed signe **revocable** (D4 vol.2). |
| `crates/nexus-shell-daemon/src/http.rs` + front | Approbation cote pair (D4 vol.3) + « Inviter un pair de confiance » + libelle « vu de ton noeud ». |

### E.3 Tests plan
1. `seed_request_signature_verified` — sig valide → accepte ; sig falsifiee → rejet.
2. `seed_request_nonce_anti_replay` — replay d'un `SeedRequest` → rejet.
3. `seeder_fetches_tags_pins_blob` (R3) — blob fetche → tagge → survit au GC.
4. `seed_requires_invite_and_approval` (D4) — sans invitation/approbation → rejet.
5. `seeded_app_keeps_author_provenance_intact` (R5) — provenance auteur inchangee apres seed.
6. **`seed_e2e_two_nodes_peer_keeps_app_reachable`** (§P57) — E2E 2 vrais nodes iroh : le pair fetch+tag+pin → app joignable depuis le pair.

### E.4 Critere d'acceptation
```
# Windows natif (feedback_wsl_before_push)
cargo nextest run -p nexus-shell-daemon -p nexus-core-rs --locked -E 'test(seed)'
# Docker Linux
docker run --rm -v "${PWD}:/workspace" -w /workspace sbfb-ci bash -c "cargo nextest run -p nexus-shell-daemon -p nexus-core-rs --locked -E 'test(seed)'"
```
`SeedRequest` authentifie (sig invalide → rejet) ; un pair fetch+tag+pin sur
invitation+approbation ; provenance auteur intacte ; **E2E 2-noeuds reel**.
**Preflight [DETER] crypto-spec OBLIGATOIRE avant 1er Edit.** Depend de Phase D.

### E.5 Commit cible
`feat(daemon): Sprint 74 Phase E — authenticated cross-node seed protocol (SeedRequest ALPN + fetch/tag/pin + invite/approval)`
Body : 9 sections, delta +6, G8 (D1+D4, [DETER] crypto-spec), carry closure
LT-5 (segment 2 amont), scope cut #6 (probe externe NAT differe) respecte.

---

## Phase F — Seed cross-noeud aval (D3) : re-annonce distante persistante + registre `SeedAnnounced` + compteur multi-seed

### F.1 Scope
**Cran le plus aval (D5 — reste « Bientot » si deborde).** Re-annonce
**persistante** par le pair distant **apres reboot** (le seeder relit son
`keep_online` au boot, comme Phase D mais cote pair). Registre op feed
**`SeedAnnounced`** (raw-op `serde_json::Value` : `{ project_id,
seeder_node_id, archive_hash, ts, sig }`, **0 bump** `FEED_FORMAT_VERSION`) →
`feed_sync` ingest → agregat seed + **etat multi-seed** + compteur
**« Toi + N pairs (vus recemment) »** (best-effort TTL, D3/Checkpoint Q5).
Front « Qui la garde en ligne » multi-seed + « Copies de secours »
**fonctionnel** (remplace « Bientot »).

### F.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` (1750-1771) | Re-annonce distante persistante au boot (seeder cote pair). |
| `crates/nexus-shell-daemon/src/public_feed.rs` (82-118) | Helper construire/valider `SeedAnnounced` raw-op (pas une 5e variante d'enum). |
| `crates/nexus-shell-daemon/src/feed_sync.rs` | Ingest `SeedAnnounced` → agregat seed. |
| `crates/nexus-shell-daemon/src/seed_registry.rs` (NEW) ou `browse.rs` | Etat multi-seed + compteur TTL best-effort. |
| `web/src/components/AvailabilitySheet.tsx` | « Qui la garde en ligne » multi-seed + « Copies de secours » fonctionnel. |

### F.3 Tests plan
1. `remote_seeder_reannounces_after_reboot_e2e` (§P57) — pair distant re-annonce apres reboot.
2. `seed_announced_raw_op_no_version_bump` — `FEED_FORMAT_VERSION` reste 1.
3. `seed_announced_ingested_increments_count` — `SeedAnnounced` → compteur multi-seed.
4. `seed_count_best_effort_ttl_expires` — un pair non-vu (TTL) sort du compteur.
5. `multi_seed_state_rendered` (front) — « Toi + N pairs » affiche.

### F.4 Critere d'acceptation
```
cargo nextest run -p nexus-shell-daemon --locked -E 'test(seed_announced) + test(remote_seeder) + test(seed_count)'
(cd web && npm run test:unit)
```
Un pair distant re-annonce apres reboot (E2E) ; `SeedAnnounced` ingere →
compteur ; multi-seed visible ; 0 bump wire. **Si deborde → reste « Bientot »
(D5, arbitrage Checkpoint Q1).** Depend de Phase E.

### F.5 Commit cible
`feat(daemon): Sprint 74 Phase F — remote seed boot re-announce + SeedAnnounced registry (raw-op) + multi-seed counter`
Body : 9 sections, delta +5, G8 (D3), carry closure LT-5 (segment 2 aval),
pre-launch (raw-op, 0 bump), scope cut #4 (failover differe) respecte.

---

## Phase G — Wrap-up + dette (coverage T14 + carries audit S73)

### G.1 Scope
`sprint74_verification.md` (fail-fast rempli) + `sprint75_audit_plan.md`.
**Dette coverage T14** : tests `FileUploadBlock` (35 %→seuils 85/90/78/85) +
**retirer le masquage `| tail`** du fail-fast (verify.sh step12) + **ajouter
`bootstrap.ts` a `coverage.include`** → `test:coverage` ENFIN vert AVANT push
origin (LT-2, R8). **Carries audit S73 non traites A-F** : B.2 quorum zombie
+ statut terminal + test redundancy>1 ; FRESHNESS-RELEASE-UNINDEXED
(ReleasePublished project_name/category indexable) ; D.1 recadrer THREAT_MODEL
§11 (residual loopback, clamp `q`/`offset` si retenu) ; B.5 normaliser
isHttpsUrl 3 ancres (Browse:471, BrowsedProject:367, VerificationDetail:185) +
test multi-vecteur (Checkpoint Q6) ; SEARCH-VIEW-THROW-SKELETON
(`query.isError` → carte d'erreur) ; E.3 renforcer 3 tests C/D ; B.4 + C.4
PATTERNS. `PATTERNS.md` (seed cross-noeud + pin local) + memory + SPRINT_LOG +
CLAUDE.md.

### G.2 Fichiers touches
| Fichier | Role |
|---|---|
| `.planning/active/sprint74_verification.md` (NEW) | Self-report fail-fast (incl. `test:coverage` vert). |
| `.planning/active/sprint75_audit_plan.md` (NEW) | Feuille de route audit S74 pour S75. |
| `web/src/components/blocks/FileUploadBlock.test.tsx` (NEW) + `web/vitest.config.ts` + `web/scripts/verify.sh` (ou fail-fast) | **T14** : tests ≥ seuils, retirer `\| tail`, ajouter `bootstrap.ts` a `coverage.include`. |
| `crates/nexus-coordinator-rs/src/validator.rs` (B.2) | Branche quorum trip pose Rejected (statut terminal) + test redundancy>1. |
| `crates/nexus-coordinator-rs/src/search.rs` + `public_feed.rs` (FRESHNESS) | ReleasePublished indexe project_name/category. |
| `web/src/pages/Browse.tsx` + `BrowsedProject.tsx` + `VerificationDetail.tsx` (B.5) | Normaliser isHttpsUrl sur 3 ancres + branche `query.isError`. |
| `docs/security/THREAT_MODEL.md` (§11, §14) + `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` | D.1 recadre + B.4/C.4 doc + pattern seed/pin. |
| memory + `docs/claude/SPRINT_LOG.md` + `CLAUDE.md` | Etat S74. |

### G.3 Critere d'acceptation
100% fail-fast verts (**incl. `test:coverage` vert, `| tail` retire**) ;
2 docs planning ; carries audit S73 traites ou re-route explicitement ;
PATTERNS + memory a jour.

### G.4 Commit cible
`docs(sprint74): verification + audit plan for Sprint 75 + coverage T14 + S73 audit carries`

---

## §5 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | nextest workspace | `cargo nextest run --workspace --locked` | 0 fail (canonique Linux) | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 fail | |
| 5 | build release | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | A — panneau Disponibilite | `test(availability_sheet_renders_author_state_seeders)` | 3 sections | |
| 7 | A — carte publish 0 champ hote | `test(publish_success_card_folds_hashes)` | hashs replies, 0 hote | |
| 8 | A — rename coordinateur→noeud | `test(coordinator_renamed_to_node_in_shell)` | plus de « coordinateur » | |
| 9 | A — toggle lecture-seule | `test(keep_online_toggle_readonly_in_phase_a)` | ON honnete | |
| 10 | B — fork forge | `test(fork_from_forge_clones_repo_at_commit)` | workspace | |
| 11 | B — fork blob | `test(fork_from_blob_reconstructs_archive)` | workspace | |
| 12 | B — rowid partition (C.3) | `test(browse_rowid_partitioned_from_feed_seq)` | pas de clobber | |
| 13 | B — invariant provenance (B.6) | `test(browse_index_rejects_open_source_without_provenance)` | rejet | |
| 14 | C — redeploy identite locale | `test(fork_redeploy_resigns_provenance_as_local_node)` | auteur = MON noeud | |
| 15 | C — boucle E2E mono-noeud | `test(fork_redeploy_loop_e2e_single_node)` | vert | |
| 16 | C — per-app /publish+gossip (OFF-SPRINT-2b) | `test(publish_and_gossip_use_per_app_project_id)` | plus de node_id | |
| 17 | D — keep_online M18 | `test(keep_online_toggle_persists_m18)` | persiste | |
| 18 | D — re-annonce au boot | `test(pinned_app_reannounced_on_boot)` | re-annonce reboot | |
| 19 | D — OFF retire le tag | `test(keep_online_off_removes_tag)` | GC-eligible | |
| 20 | D — M17 boot-recovery non-silencieuse (H.1) | `test(m17_boot_recovery_not_silent)` | log/metrique | |
| 21 | E — `SeedRequest` sig | `test(seed_request_signature_verified)` | sig falsifiee → rejet | |
| 22 | E — anti-replay | `test(seed_request_nonce_anti_replay)` | replay → rejet | |
| 23 | E — fetch+tag+pin (R3) | `test(seeder_fetches_tags_pins_blob)` | survit GC | |
| 24 | E — invite+approbation (D4) | `test(seed_requires_invite_and_approval)` | sans → rejet | |
| 25 | E — provenance auteur intacte (R5) | `test(seeded_app_keeps_author_provenance_intact)` | inchangee | |
| 26 | E — E2E 2-noeuds reel (§P57) | `test(seed_e2e_two_nodes_peer_keeps_app_reachable)` Windows + Docker | pair joignable | |
| 27 | F — re-annonce distante reboot (§P57) | `test(remote_seeder_reannounces_after_reboot_e2e)` | vert | |
| 28 | F — `SeedAnnounced` 0 bump | `test(seed_announced_raw_op_no_version_bump)` | FEED_FORMAT_VERSION=1 | |
| 29 | F — compteur multi-seed | `test(seed_announced_ingested_increments_count)` | incremente | |
| 30 | A-F — `web/` lint+tsc | `npm run lint && tsc --noEmit` | exit 0 | |
| 31 | A-F — Vitest `web/` | `npm run test:unit` | 0 fail (294 + nouveaux) | |
| 32 | A-F — build + size | `npm run build && npm run size` | 6/6 | |
| 33 | A-F — scan-en-strings | `bash scripts/scan-en-strings.sh` | 0 string EN | |
| 34 | G — **test:coverage vert (T14)** | `npm run test:coverage` (`\| tail` retire) | seuils 85/90/78/85 atteints | |
| 35 | G — `bootstrap.ts` ∈ coverage.include | `grep bootstrap web/vitest.config.ts` | present | |
| 36 | G — B.5 isHttpsUrl 3 ancres | `test(repo_url_anchors_https_guarded)` | multi-vecteur | |
| 37 | G — SEARCH-VIEW isError | `test(search_view_renders_error_card)` | carte d'erreur | |
| 38 | G — B.2 quorum statut terminal | `test(quorum_trip_sets_rejected_status)` | Rejected | |
| 39 | G — 2 docs planning | `test -f sprint74_verification.md + sprint75_audit_plan.md` | present | |

---

## §6 Git plan

| Ordre | Phase | Type | Titre |
|---|---|---|---|
| 1 | kickoff | chore | `chore(planning): Sprint 74 kickoff + plan + design_review` |
| 2 | A | feat | `feat(shell): Sprint 74 Phase A — availability panel (read) + clean publish card + rename coordinator→node` |
| 3 | B | feat | `feat(factory): Sprint 74 Phase B — fork a network project into a target workspace (forge clone or blob reconstruction)` |
| 4 | C | feat | `feat(factory): Sprint 74 Phase C — redeploy a fork under local node identity + per-app project_id on publish/gossip` |
| 5 | D | feat | `feat(daemon): Sprint 74 Phase D — persistent local pin (keep_online M18 + blob tag + boot re-announce)` |
| 6 | E | feat | `feat(daemon): Sprint 74 Phase E — authenticated cross-node seed protocol (SeedRequest ALPN + fetch/tag/pin + invite/approval)` |
| 7 | F | feat | `feat(daemon): Sprint 74 Phase F — remote seed boot re-announce + SeedAnnounced registry (raw-op) + multi-seed counter` |
| 8 | G | docs | `docs(sprint74): verification + audit plan for Sprint 75 + coverage T14 + S73 audit carries` |

Chaque phase code (A-F) : preflight G8 (**E = [DETER] crypto-spec
obligatoire**) → review PASS-PENDING → Codex brut → reconciliation → review
PASS → body 9 sections. **Migration S73 `active/`→`archive/v2.1/` deleguee au
main thread** (NE PAS dans ce kickoff/plan).

---

## §7 Scope cuts (copie kickoff §7)

1 GPU cross-machine → S75. 2 quorum redundancy>1 cross-MACHINE → S75. 3
sharding → S76. 4 re-allocation/failover auto IPFS-Cluster → post-launch
(vision). 5 timer re-annonce 22h → post-launch. 6 probe externe NAT complet →
S75+ (S74 = libelle « vu de ton noeud »). 7 page « Mes seeds » COMPLETE →
post-S74 ; vision+empty-state INCLUS S74 (PO Q4). 8 editeur Monaco → jamais (PO-9).
9 templates react/pyodide → INCLUS S74 (PO Q7, retire des scope cuts).
10 SearchManifest reseau-large → post-launch. 11 compteur nombre exact →
post-S74 (PO Q5 = best-effort « Toi + N pairs »). 12 Tantivy → gate post-S75 (gele). 13 token-par-token WAN →
jamais (PO-14). 14 rate-limit per-client search → Phase G recadre (D.1).

---

## §8 Risks (R1..R8)

Cf. kickoff §9. **R1 scope creep (Eleve/Eleve) → D5 segmentation A-D surs /
E-F borne, arbitrage ampleur PO Checkpoint Q1.** R2 SeedRequest protocole
sous-estime → composition primitives + preflight [DETER] + E2E reel + slice
repli. R3 blob fetche non-tagge → corriger fetch_ticket + test GC. R4
faux-vert NAT → libelle honnete pilote. R5 re-attribution auteur → fork
re-signe local, seed provenance intacte, test. R6 M18/re-annonce casse boot →
additif local + best-effort warn-only + test reboot. R7 OFF-SPRINT-2b
incomplet collisionne fork-redeploy → completer per-app AVANT C. R8 dette
T14 invisible casse GHA au push → Phase G AVANT push origin.

---

## §9 Checkpoint de cloture

- [ ] Fail-fast checklist §5 : 39/39 rows PASS (canonique CI Linux, **incl.
      test:coverage vert apres retrait `| tail`**)
- [ ] Phases A-G landed (A dispo front+rename, B fork backend, C fork-redeploy,
      D pin local, E seed cross-noeud, F registre+compteur, G wrap-up+dette)
- [ ] **Atelier fork prouve** : chercher→forker→editer→REDEPLOY sous identite
      locale (E2E mono-noeud) ; provenance du fork = MON noeud (seeder≠co-auteur)
- [ ] **Disponibilite continue cablee** (D-DISPO) : publish 0 champ hote +
      carte succes + panneau AUTEUR/ETAT/QUI-LA-GARDE + 5 verrous UI
- [ ] **Pin local persistant** : toggle « Garder en ligne » fonctionnel,
      app re-annoncee au boot (survit redemarrage daemon)
- [ ] **Seed cross-noeud (ex-LT-5 pull-forward)** : `SeedRequest` authentifie
      + fetch+tag+pin + invite/approbation + E2E 2-noeuds reel — **OU** slice
      borne + crans aval « Bientot » (arbitrage PO Checkpoint Q1)
- [ ] **Faux-vert NAT honnete** : « En ligne (vu de ton noeud) » pour le self
- [ ] **0 faux bouton actif** (chaque cran cross-noeud non livre = « Bientot »
      inerte)
- [ ] **Rename « coordinateur »→« noeud »/« reseau »** propage (Phase A)
- [ ] **Dette T14 fermee** : tests FileUploadBlock ≥ seuils, `| tail` retire,
      `bootstrap.ts` ∈ coverage.include, test:coverage vert
- [ ] **14 carries audit S73 traites** (B.2, FRESHNESS, ROWID C.3, B.6, H.1,
      H.2, D.1, B.5, SEARCH-VIEW, E.3, B.4, C.4, OFF-SPRINT-2, 2b) ou re-route
- [ ] Pas de bump wire (pre-launch ; `SeedAnnounced` raw-op ; M18 schema local ;
      FEED_FORMAT_VERSION=1)
- [ ] 6/6 phases code G8 (Phase 0 audit done) ; **E = [DETER] crypto-spec** ;
      Codex 6/6 phases code (A-F)
- [ ] `sprint74_verification.md` + `sprint75_audit_plan.md` ecrits
- [ ] PATTERNS (seed cross-noeud + pin local) ; memory + MEMORY.md + SPRINT_LOG
      row S74 + CLAUDE.md a jour

**S74 CLOSED quand** : 39/39 fail-fast verts + atelier fork prouve +
Disponibilite continue cablee (front + pin local) + seed cross-noeud livre
(full OU slice borne avec crans « Bientot ») + dette T14 fermee + 8 commits +
3 fichiers planning. Arc 3.5 (Factory Complete Vision) **4/6** ; S75 (GPU
partage cross-machine) debloque sous reserve de l'audit gate S74.
**LT-5 redundancy persistence honoree en avance** (pull-forward PO).
