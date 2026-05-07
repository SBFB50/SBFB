# Sprint 54 — Design Review (G1)

**Reviewer** : agent independant.
**Source** : sprint54_kickoff.md §4 (D1..D4).

---

## Scoring

D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4, actionnable).

---

## D1 ✅ — Edition 2024 via `cargo fix --edition`

`cargo fix --edition 2024` est l'outil officiel de migration
Rust documente dans le Rust Edition Guide. L'analyse pre-gel
a verifie les 5 breaking changes edition 2024 pertinents :
- 0 `extern` block → pas de `unsafe extern` concern
- 0 `unsafe fn` → `unsafe_op_in_unsafe_fn` lint non applicable
- 0 `gen` identifiant → keyword reservation ok
- `-> impl IntoResponse` sont tous des async fn top-level →
  lifetime capture change non applicable
- 70+ `set_var`/`remove_var` → seul changement reel

L'approche atomique (1 commit, tout le workspace) est preferee
a la migration progressive (mixed-edition fragile). La suite de
1206 tests valide la migration.

## D2 ✅ — E2E wire tasks_doc_ticket

L'analyse pre-gel (memory `s54_e2e_sequencing.md`) identifie
precisement les 3 fichiers et le chainon manquant. Le champ
`tasks_doc_ticket: String` dans `MintRequest` suit le pattern
existant des autres champs wire format (serialisation string,
canonical JCS).

La pre-launch policy est respectee : pas de bump version, on
redefini la v1 courante. L'invite est le bon vehicule pour le
ticket (contient deja toutes les infos de connexion worker).

Verification : `iroh_docs::DocTicket` supporte bien `to_string()`
/ `FromStr` pour la serialisation base32. Pattern utilise dans
`crates/nexus-shell-daemon-core/src/iroh_runtime.rs` pour les
tickets blobs (S12).

## D3 ⚠️ — CI infra VPS

**Finding** : le plan prevoit d'installer l'agent Woodpecker en
binaire natif sur le VPS, mais Woodpecker CI necessite un serveur
Woodpecker central (pas seulement un agent). Le VPS sbfb-eu n'a
pas de serveur Woodpecker installe. L'agent seul ne peut pas
executer de pipeline sans serveur.

**Analyse** : deux options :
- (a) Installer serveur + agent Woodpecker sur le meme VPS (8GB
  RAM suffisant, serveur est leger ~100MB RSS)
- (b) Utiliser le VPS uniquement comme runner et heberger le
  serveur Woodpecker sur la machine dev ou un autre VPS

L'option (a) est plus simple pour un smoke test CI. Le serveur
Woodpecker en mode SQLite local est suffisant sans PostgreSQL.

**Decision recommandee** : ack le finding, proceder avec option (a)
serveur + agent sur le meme VPS. Si la RAM est insuffisante pendant
les builds, scope cut a S55.

## D4 ✅ — Dette pair items selection

Les 4 items selectionnes sont tous < 100 LOC individuellement et
ciblent des fichiers differents (pas de conflit) :
- `node_key` perms : ~5 LOC (securite, priorite correcte)
- `gossip params struct` : ~30 LOC (refactoring, supprime le
  `#[allow(clippy::too_many_arguments)]`)
- `periodic republish` : ~15 LOC (timer + jitter dans select!)
- `route collision doc` : docs seulement

Les items exclus (`outbox persistant`, `browse_request rate-limit`)
sont correctement identifies comme necessitant du design — carry S55
justifie.

---

## Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4).

D3 ⚠️ (Woodpecker serveur requis) : acknowledge — proceder avec
option (a) serveur + agent sur le meme VPS sbfb-eu. Le serveur
Woodpecker en mode SQLite local est leger. Si la RAM de 8GB est
insuffisante pendant les builds Rust (~4-6GB peak), scope cut le
serveur Woodpecker a S55 et se contenter du GHA 9/9 + images pin
pour la Phase D.
