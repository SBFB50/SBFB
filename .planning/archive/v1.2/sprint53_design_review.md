# Sprint 53 — Design Review (G1)

**Reviewer** : agent Explore independant.
**Source** : sprint53_kickoff.md §4 (D1..D4).

---

## Scoring

D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4, actionnable).

---

## D1 ✅ — Build from source

Workspace Cargo.toml : deps cross-platform, `cfg(unix)` et
`cfg(windows)` correctement gatees. build.rs n'a que du
`#[cfg(windows)]` (winresource). CI release.yml valide la
compilation 3 OS × 3 binaires. Pas de dep XCode SDK dans les
sources. Build from source sur chaque cible est faisable.

## D2 ✅ — Smoke test 3 niveaux

`DaemonRuntime::start()` boote un noeud iroh (`create_node()`),
ecrit `running.json`, bind HTTP loopback, lance gossip subscribe
+ pkarr browse. Graceful shutdown via `tokio::signal::ctrl_c()`.
Les 3 niveaux de test sont realistes.

## D3 ⚠️ — VPS setup minimal

**Finding** : `auth.rs` (loopback hardening S7/S16) impose
`Host: localhost | 127.0.0.1 | [::1]`. Le HTTP API du daemon
sur le VPS ne repondra qu'aux requetes locales.

**Analyse** : ce n'est PAS un blocker pour le smoke test P2P.
Le P2P iroh (gossip, blobs, pkarr DHT) est un transport QUIC
separe du HTTP API loopback. Le HTTP API sert uniquement la
connexion frontend React → daemon local. Sur le VPS, on n'a
PAS besoin du frontend — le daemon participe au reseau P2P
via iroh sans HTTP externe. Le Niveau 2/3 du smoke test
observe les logs iroh et la decouverte de peers, pas l'API HTTP.

**Decision** : D3 maintenue. La restriction localhost est
correcte par design (securite loopback). Si un futur sprint
veut exposer le frontend VPS, il faudra un reverse proxy
(nginx) qui bind localhost — c'est scope cut S54.

## D4 ✅ — unsafe set_var

**Finding reviewer** : 16 appels `set_var`/`remove_var` non
wrapes `unsafe`. Le reviewer affirme "won't compile on 1.94+".

**Correction** : le workspace utilise `edition = "2021"`
(Cargo.toml:21). `std::env::set_var` est safe en edition 2021
meme sur Rust 1.94. L'exigence `unsafe` est liee a l'**edition
2024**, pas a la version du compilateur. Le code compile
aujourd'hui sans erreur.

Le carry P2-REVIEW-B-1-S51 est **proactif** : wrapper
maintenant pour etre pret quand le workspace migrera a edition
2024. A 2/3, il devient MANDATORY S54 si non adresse. Phase C
le resout — le timing est correct.

---

## Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4).

D3 ⚠️ (localhost auth gate VPS) : acknowledge — le P2P iroh
est un transport distinct du HTTP loopback. Le smoke test
observe le P2P (logs iroh, decouverte peers), pas le HTTP API.
Pas de modification D3. Si le frontend VPS est necessaire plus
tard → scope cut S54 (nginx reverse proxy).
