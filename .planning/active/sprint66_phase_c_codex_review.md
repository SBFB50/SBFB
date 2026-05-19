Pas de GAPs bloquants.

- **P2 documente**: le test `test_feed_republish_at_boot` verifie
  que le daemon boote sans erreur apres insertion d'entries feed
  en SQLite entre deux boots, mais n'asserte pas directement que
  les entries sont presentes dans iroh-docs (verification via doc
  handle non exposee par DaemonRuntime). Le test couvre le path
  sans panic/error, pas la presence E2E dans iroh-docs.

- **P2 documente**: `provenance_cross_node_verified` et
  `provenance_cross_node_tampered` testent les deux cas de
  verification cross-node (valid + tampered node_id). Le path
  hex decode invalid (non-hex ou longueur != 32) retourne
  ("failed", false) sans panic, couvert par le pattern matching
  exhaustif dans get_provenance.

Constats propres :
- Le republish au boot scope le MutexGuard dans un bloc {} avant
  l'await (clippy await_holding_lock satisfait).
- feed_join cap 10 + retain(is_finished) empeche l'accumulation
  non-bornee de handles.
- La verification cross-node extrait le pubkey depuis record.node_id
  au lieu de state.pow_keypair, permettant la verification de
  provenance signee par n'importe quel noeud du reseau.
- Le champ status est ajoute a la reponse HTTP JSON uniquement,
  pas au struct ProvenanceRecord (pre-launch protocol respecte).
- useBridge.ts conserve le backward compat 404 → status "absent".

Tests lances :
  cargo nextest run --workspace --locked    1342 passed
  (cd web && npm run test:unit)             269 passed
