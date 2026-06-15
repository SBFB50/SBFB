**Verdict**

Correctif FIX-A fonctionnellement **CONFIRME** : les replays d’outbox ne rebroadcastent plus les octets PoW stockés verbatim, ils normalisent le payload puis re-wrappent. Seul **GAP mineur** : le test attendu sous le nom exact `replay_refreshes_own_ticket_preserving_hash` n’existe pas ; le comportement est couvert par `replay_remints_own_ticket_to_current_address`.

1. **CONFIRME — outbox non-wrappée**
`publish_announcement` construit le payload à `crates/nexus-shell-daemon/src/deploy.rs:673`, broadcast l’enveloppe live à `deploy.rs:676-680`, puis envoie `GossipCmd::Outbox(payload)` hors du bloc de wrap à `deploy.rs:692-695`. Donc isolé ou wrap live échoué, l’envoi outbox est tenté. Nuance : l’échec du channel est ignoré (`let _ =`), donc c’est un “persist attempt” best-effort côté architecture.

2. **CONFIRME — helper re-mint + re-wrap**
`mint_ticket_for_hash` vérifie le blob local puis utilise `my_endpoint_addr()` à `runtime.rs:1789-1796`. `remint_and_wrap_for_replay` normalise, parse l’annonce, ne re-mint que si `ann.node_id == node.node_id()` à `runtime.rs:1847-1850`, extrait le hash du ticket et remplace seulement si le mint réussit à `runtime.rs:1850-1859`, puis re-wrappe dans tous les cas via `wrap_payload_with_pow_static` à `runtime.rs:1863-1864`. Le cas sans ticket rejoue aussi : si `archive_ticket` est `None`, le bloc `if let Some` est sauté et le wrap reste exécuté à `runtime.rs:1850-1864`.

3. **CONFIRME — 4 sites couplés**
Les trois replays outbox utilisent `remint_and_wrap_for_replay` puis `broadcast(fresh)` : browse_request `runtime.rs:1513-1529`, NeighborUp `runtime.rs:1556-1572`, republish périodique `runtime.rs:1658-1674`. Le boot restore appelle `restore_browse_from_outbox` à `runtime.rs:1450-1451`, qui passe par `normalize_outbox_payload` à `runtime.rs:2037-2044`. `keep_online_allows_rebroadcast` normalise aussi à `runtime.rs:1980-1989`. Aucun de ces sites ne rebroadcast `stored.clone()` / ancien envelope verbatim.

4. **CONFIRME — fenêtre PoW inchangée**
`POW_FORMAT_VERSION` reste `1` à `crates/nexus-core-rs/src/pow.rs:85`, `MAX_PROOF_AGE_SECS` reste `1_800` à `pow.rs:105-109`, et `verify_at` rejette toujours `age > MAX_PROOF_AGE_SECS` à `pow.rs:411-425`. Le chemin entrant réseau appelle toujours `verify_envelope` à `runtime.rs:1488-1499`, et `verify_envelope` appelle `verify_at` sur slow path à `crates/nexus-core-rs/src/pow_gossip.rs:281-317`. La seule modif core vue est un assert `SESSION_WINDOW < MAX_PROOF_AGE_SECS` à `pow_gossip.rs:91-98`, pas un affaiblissement.

5. **CONFIRME — normalize_outbox_payload transition**
Nouveau format payload direct accepté à `runtime.rs:1808-1812`; legacy PoW-wrapé décodé structurellement à `runtime.rs:1813-1818`; junk retourne `None` à `runtime.rs:1820`. Le commentaire borne explicitement ça à l’état persisté local, pas au wire format, à `runtime.rs:1799-1807`.

6. **CONFIRME — mint_ticket_for_hash partagé**
`http::mint_blob_ticket` ne duplique plus la logique : il décode le hash puis délègue à `runtime::mint_ticket_for_hash` à `crates/nexus-shell-daemon/src/http.rs:1639-1653`. Le helper commun conserve les garanties : blob local présent, adresse courante, `BlobTicket::new(...)` à `runtime.rs:1789-1796`.

7. **CONFIRME — pas de bump wire**
`FEED_FORMAT_VERSION = 1` à `crates/nexus-coordinator-rs/src/public_feed.rs:18-20`, `PROJECT_ANNOUNCEMENT_VERSION = 1` à `crates/nexus-shell-daemon-core/src/publish.rs:20-24`, `ANNOUNCEMENT_VERSION = 1` à `crates/nexus-shell-daemon-core/src/iroh_runtime.rs:89-92`, `POW_FORMAT_VERSION = 1` à `pow.rs:80-85`. Le wire reste une `PowEnvelope::encode(&proof, payload)` à `runtime.rs:1758-1770`.

8. **GAP mineur — tests**
Les tests prouvent le fix, mais le nom exact attendu `replay_refreshes_own_ticket_preserving_hash` est absent. Couverture réelle :
`replay_restamps_pow_so_a_fresh_receiver_accepts` vérifie acceptation fresh receiver à `runtime.rs:2503-2543`; `replay_remints_own_ticket_to_current_address` vérifie re-mint et hash préservé à `runtime.rs:2548-2620`; anti-hijack à `runtime.rs:2626-2695`; mint failure garde le ticket et restampe à `runtime.rs:2701-2755`; normalize nouveau+legacy+junk à `runtime.rs:2461-2499`; publish outbox payload non-wrappé à `http.rs:2864-2914`; boot restore nouveau+legacy à `runtime.rs:2797-2939`; keep_online nouveau+legacy à `runtime.rs:2760-2793` et `runtime.rs:2943-2984`.

Tests ciblés exécutés et passés :
`cargo test -p nexus-shell-daemon --locked replay_`, `normalize_outbox_payload`, `publish_announcement_persists_to_outbox_for_replay`, `browse_boot_restore`, `keep_online`.
