# Sprint 47 — Design Review (G1)

**Reviewer** : agent independant (recherche codebase).
**Date** : 2026-05-01.
**Referentiel** : `sprint47_kickoff.md` §4 D1..D4.

---

## D1 — Invite ID collision fix : node_id prefix — ⚠️

**Verification** :
- `DaemonHttpState.node_id` (http.rs L76) est un `String` hex
  64 chars, derive de la keypair Ed25519 iroh. Accessible dans
  tous les handlers via `State(state)`.
- `invite_api.rs` L17 : `INVITE_COUNTER` est un `AtomicU64`
  static — pas de coordination cross-process, collision confirmee.
- L80 : `format!("inv-{now}-{seq}")` — le fix `inv-{node_id_8}-
  {now}-{seq}` est trivial et backwards-compatible.
- `uuid` crate n'est PAS dans le workspace Cargo.toml — ajouterait
  une dep pour un fix simple.

**Finding** : schema compat — si un ledger stocke des invites par
string match exact, le changement de format est un break. Acceptable
pre-v1.0 (pre-launch protocol policy).

**Scoring** : ⚠️ source verifiee, alternative comparee, schema
compat point a noter pre-v1.0.

---

## D2 — Integration tests deploy.rs + apps.rs : fixture — ⚠️

**Verification** :
- `mk_state()` (http.rs L1679) cree un iroh Node reel via
  `create_node().await` avec `blobs_store()` fonctionnel — le
  happy path deploy_private est POSSIBLEMENT testable (meilleur
  que prevu dans le kickoff).
- `BrowseAggregator::add_direct_entry()` existe (deploy.rs L365)
  — peut etre utilise pour peupler le browse dans les tests apps.
- apps.rs `make_entry()` (L186) est un helper test reutilisable.

**Finding** : la feasibility du deploy_private happy path est
sous-estimee dans le kickoff. Le scope cut pourrait etre re-evalue
au G8 Phase B. `add_direct_entry()` simplifie le setup apps.

**Scoring** : ⚠️ approche correcte, feasibility meilleure que
documentee.

---

## D3 — Python modules suppression : audit-then-delete — ⚠️

**Verification** :
- `coordinator.py` L56-82 importe **14 modules** activement.
- 19 autres modules existent dans le namespace — verifier via
  grep systematique si certains sont dead code.
- Risque non nul de "0 module supprimable" si tous les modules
  candidats sont importes transitivement.

**Finding** : l'audit systematique (grep -r cross-package) doit
etre fait AVANT le premier Edit de Phase A. Le resultat "0 module
supprimable + evidence documentee" est une resolution valide du
carry mais n'est pas anticipe dans le risk register.

**Scoring** : ⚠️ approche correcte, outcome zero-delete non
documente dans R2.

---

## D4 — Happy path tests consent/files : mk_state() enrichi — ⚠️

**Verification** :
- **Finding critique** : les handlers consent.rs et files.rs
  lisent/ecrivent via `sbfb_home()` (variable d'environnement
  `SBFB_HOME` ou `~/.sbfb/`). Ils ne sont PAS state-dependent
  via DaemonHttpState.
- L'enrichissement mk_state() avec des tmpdir fields est
  INSUFFISANT — il faut setter `SBFB_HOME` dans le test harness
  AVANT la construction du router.
- Pattern : `std::env::set_var("SBFB_HOME", tmpdir.path())` dans
  chaque test ou dans un fixture shared.

**Finding** : le plan §C.1/§C.2 suppose un enrichissement
DaemonHttpState alors que les handlers utilisent `sbfb_home()`.
Corriger l'approche : setter `SBFB_HOME` env var dans le harness.

**Scoring** : ⚠️ choix technique correct (tests happy path),
detail implementation incorrect (state vs env var).

---

## Rigor signal G4

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ⚠️.
Rigor signal G4 satisfait (4 ⚠️ sur 4, 0 ❌).

Aucun ⚠️ ne bloque l'execution — tous sont des precisions
d'implementation resolvables au G8 preflight de chaque phase.
Le finding D4 (sbfb_home vs state) est le plus important et
doit etre acknowledge dans le kickoff.
