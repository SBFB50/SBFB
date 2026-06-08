# Sprint 75 — Audit plan (audit gate de S74, joue en S75 Phase 0)

Audit gate pattern permanent (depuis S7) : S75 Phase 0 = audit adversarial de
S74 (workflow multi-agents anti-anchoring, P0/P1 bloquants). Ce fichier liste
les surfaces a scruter + les carries re-routes depuis S74 Phase G (G.3 « traiter
OU re-router explicitement »).

## Carries re-routes depuis S74 Phase G (a traiter ou re-router en S75)

| Carry | Pourquoi differe | Fix connu |
|---|---|---|
| **FRESHNESS-RELEASE-UNINDEXED** | Changement payload wire `ReleasePublishedPayload` + 16 literals dans le commit wrap-up = risque dispro ; benefice partiel (releases gossipees d'anciens pairs restent non-indexees) | Ajouter `project_name`/`category` `Option` `#[serde(default, skip_serializing_if=Option::is_none)]` (0-bump, S67 precedent) ; `extract_index_fields` lit DEJA `field("project_name")` -> juste `category` de `String::new()` -> `field("category")` ; `deploy.rs` populate depuis params ; +test searchable-by-name. |
| **KEEP-ONLINE-HASH-SOT** | Inert sans GC reaper (le toggle ON re-lit le hash de l'aggregator, pas la colonne M18 ; jamais lu apres toggle aujourd'hui) | `set_keep_online` handler lit `db.get_keep_online().archive_hash` au lieu de l'aggregator ; load-bearing seulement quand le GC gagne un read-access (Phase H). |
| **invite single-use re-credit** (Phase E P3) | consume-avant-fetch = anti-DoS volontaire | re-crediter une invite sur fetch-failed transitoire. |
| **tests E.3 / H.2 / genuinely-shared-blob / R6-DB-error** | polish test, non-bloquant | renforcer 3 tests C/D ; reconstructibilite browse-rows ; tag partage 2 apps ; erreur DB keep_online. |
| **clamp `q`/`offset` search** (D.1) | residual L pre-launch (loopback single-user) | clamp longueur `q` + borne `offset` cote handler search. |
| **`publish_returns_200_and_adds_direct_entry` flaky** | aggregate() fait un vrai dial reseau dans un test unitaire | NEW carry : le rendre deterministe (injecter status cache / pas de dial reel) — meme classe que les autres tests iroh-networked qui hang sur reseau degrade. |

## Surfaces S74 a scruter (audit S75 Phase 0)

- **SeedAnnounced over-count / Sybil** (THREAT_MODEL §15) : best-effort assume ;
  verifier qu'aucun chemin ne traite le compteur comme verite de joignabilite
  (la sonde ETAT + content-addressing restent l'autorite). Registre reseau-large
  (SearchManifest) reste scope cut #10/D3.
- **Croissance feed reprovide** : chaque boot re-emet SeedAnnounced pour toutes
  les lignes keep_online enabled -> le feed grossit (modele IPFS reprovide).
  Pilote-borne ; verifier qu'aucun dedup/cap n'est attendu avant launch.
- **`is_own` via flatten view** : verifier que `BrowseEntryView` n'introduit pas
  de divergence avec les ~25 literals BrowseEntry (le champ est serialize-only).
- **B.2 quorum impossible** : verifier l'arithmetique (`best_now + remaining <=
  majority_threshold`) sur redundancy 2/3/4/5 (pas d'off-by-one terminant tot).
- **Carries audit S73 NON traites en S74** (herites) : voir
  `archive/v2.1/sprint73_audit_findings.md` — re-verifier ceux non fermes.

## Carries externes/long-terme (inchanges)

P2-A-1 rand upstream (exemption externe), P2-AUDIT-2 iroh pre-release
transitives (pin 0.98), T-NN+2 iframe Rust-wasm (PATTERNS §P34), P3-OS-1
operator_server OR duplique, LT-2 Radicle (tag v1.0 non pousse), LT-3/4/5/7.
