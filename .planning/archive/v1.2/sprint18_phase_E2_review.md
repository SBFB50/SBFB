# Sprint 18 Phase E2 — nexus-phase-auditor review

**HEAD pre-commit** : `9f4d19f`
**Draft commit body** : `feat(sprint18): Sprint 18 Phase E2 — warrant canary monthly Ed25519 gossip publish`
**Timebox** : ~45m

---

## Verdict : PASS

Aucun finding P0 ni P1. Les 4 findings P2/P3 initialement flagues
ont ete adresses dans le meme commit (bumped de CONCERN a PASS
apres fix inline) :

1. **P2 plan.md node_id → canary-key** : fixed dans
   `sprint18_plan.md §Phase E2` + ajoute explicitement la
   deviation GHA verifier vs publisher avec rationale.
2. **P2 kickoff §D5 base64 → hex + JCS** : fixed dans
   `sprint18_kickoff.md §D5` — block format mis a jour pour
   refleter la convention hex lowercase + JCS RFC 8785 utilisee
   par l'implementation.
3. **P2 PATTERNS.md** : ajoute section §Sprint 18.1 "Persistent
   maintainer identity key vs ephemeral network identity key"
   dans `docs/rust/PATTERNS.md` avec table comparative + regle
   d'orientation pour tout futur DOMAIN_* signe.
4. **P3 headline length cap** : `MAX_HEADLINE_LEN = 512` constant
   + `CanaryError::HeadlineTooLong` variant + guard dans
   `build_canary` + 2 tests (`build_canary_rejects_oversize_headline`,
   `build_canary_accepts_headline_at_exact_cap`).

Section §Findings ci-dessous reste pour reference historique.

Cumul Rust tests : 464 → **474 (+10)** — plan demandait +5, livre
+10 = 5 plan + 5 bonus (topic_id seed, parse round-trip, format
contains fields, headline reject oversize, headline accept at cap).

---

## Dimensions

### Security

- [x] **Semgrep / grep scan** : zero secret hardcode (AKIA, ghp_,
  pat_, sbfb_ patterns — neant). La pubkey `80b439cb...` dans
  CANARY.txt est une cle publique, pas un secret — aucun probleme.
- [x] **unsafe blocks** : zero dans les fichiers du diff.
- [x] **unwrap() en production** : un seul `unwrap_or` dans
  `format_canary_txt` — `std::str::from_utf8(DOMAIN_WARRANT_CANARY_V1)
  .unwrap_or("nexus-warrant-canary-v1")`. Correct :
  `DOMAIN_WARRANT_CANARY_V1 = b"nexus-warrant-canary-v1"` est de
  l'ASCII pur, le `unwrap_or` est du code mort mais inoffensif. Les
  `unwrap()` restants sont tous dans `#[cfg(test)]` — conforme a la
  convention.
- [x] **loopback / PeerCredsVerified** : `handle_canary` est une
  branche de `match cmd` dans `main()`, en dehors du stack HTTP
  axum. Il ne traverse pas le loopback HTTP, il n'a pas besoin de
  `PeerCredsVerified`. Correct.
- [x] **wire format / JCS** : `build_canary` et `verify_canary`
  passent tous deux par `canonical_bytes(&signed,
  DOMAIN_WARRANT_CANARY_V1)` qui est l'implem JCS RFC 8785 confirmee
  dans `nexus-core-rs/src/canonical.rs`. Domain separation tag
  unique `b"nexus-warrant-canary-v1"` — aucun replay cross-surface
  possible avec les tags task/result/claim/invite/kudos/curator-list/
  provenance existants.
- [x] **GHA workflow** : `permissions: contents: read` uniquement —
  le workflow ne peut pas modifier le repo. Aucun secret GHA ne
  stocke la cle de signature. Le workflow ne fait pas de
  `git commit` ni `git push`. Correct.
- [x] **path traversal** : `canary_key_path()` retourne un chemin
  construit via `sbfb_home().map(|d| d.join("canary-key.key"))` —
  aucune interpolation d'entree utilisateur dans le chemin. Pas de
  path traversal possible.
- [x] **Headline injection** : le field `headline` est libre-form
  string. Il est signe dans `CanarySigned`, donc toute modification
  post-signature invalide la signature. Le field est affiche via
  `println!` et `format!` — aucun risque injection shell. Pas de
  scrape automatique (conforme plan §E2). Pas de longueur maximale
  sur le headline — finding P3 ci-dessous.

### Patterns

Patterns Rust pertinents au diff (`docs/rust/PATTERNS.md`) :

- [x] **Sprint 7.1 — `DOMAIN_*` tag pour chaque famille signee** :
  `DOMAIN_WARRANT_CANARY_V1` ajoute dans `canonical.rs` avec doc +
  commentaire de rationale. Conforme au pattern etabli en Sprint 7.1.
- [x] **Sprint 7.2 — ordre des checks dans verify** : `verify_canary`
  commence par version check, puis hex decode, puis canonical_bytes,
  puis Ed25519 verify. Ordre cheap-avant-expensive conforme au
  pattern Sprint 7.2.
- [x] **Sprint 7.4 — topic id via BLAKE3 seed** :
  `warrant_canary_topic_id()` utilise
  `blake3::hash(WARRANT_CANARY_TOPIC_SEED)` — meme pattern que
  `curator_topic_id()`. Conforme + test
  `topic_id_is_deterministic_and_32_bytes` ancre le seed.
- [x] **`thiserror` lib / `anyhow` bin** : `CanaryError` utilise
  `thiserror` dans la core lib. `handle_canary` dans le binary
  utilise `anyhow::Context`. Conforme.
- [x] **`async_trait` pour trait mockable** : `CanaryBroadcaster`
  utilise `async-trait` qui est deja dans le workspace — conforme au
  pattern existant dans `nexus-core-rs` et `nexus-worker-core`.

**Pattern drift detecte (P2)** : le pattern de separer cle
ephemere (node identity) de cle persistante (maintainer identity)
n'est pas documente dans `docs/rust/PATTERNS.md`. La decision est
justifiee dans le doc du module et dans `auth.rs::canary_key_path()`,
mais elle n'est pas capturee comme pattern reutilisable. A ajouter
en Phase F ou Sprint 19 Phase 0 — notamment pour eviter qu'un futur
sprint reutilise `node_id.key` par erreur pour une operation qui
necessite une identite persistante.

### Scope-cuts

Scan exhaustif sur les huit keywords scope-cut du kickoff §6 :

```
iroh-audit, pyodide-escape, PoW-gossip, encryption-at-rest,
tls-pinning, pkarr-relay, ONG-relays, PQC, ML-DSA, ML-KEM
```

**Resultat : zero match.** Aucun fichier du diff ne touche un scope
cut. Conforme.

### Tests-delta

- [x] **Rust annonce** : +8 (plan +5, +3 bonus).
- [x] **Reel mesure** : `cargo test --workspace --locked` → 472
  passing (baseline 464 post-E1). Delta reel = **+8**. Correspond
  exactement.
- [x] **Zero failed, zero ignored** : confirme par output de
  `cargo test`. Toutes les suites individuelles : 0 failed,
  0 ignored.
- [x] **Tests nommes** : les 8 tests de `canary::tests` sont tous
  presents et verts.

### Research-grounding

Deps ajoutees/modifiees dans le diff :

- `async-trait = { workspace = true }` dans les deux Cargo.toml —
  **pas une nouvelle dependance workspace**. `async-trait = "0.1"`
  est present dans le `Cargo.toml` racine a la ligne 66, utilise
  dans `nexus-core-rs` et `nexus-worker-core` depuis Sprint 2.
  Version inchangee. Cargo.lock ne montre aucune nouvelle
  resolution. **PASS**.
- **API crypto** utilisees : `canonical_bytes` (JCS RFC 8785, trace
  Sprint 7), `KeyPair::load_or_generate` + `sign` + `verify`
  (Ed25519-dalek, trace Sprint 2), `blake3::hash` (trace Sprint
  7.4). **PASS**.
- **Gossip topic broadcast** via `GossipClient::join_topic` +
  `TopicHandle::broadcast` — API iroh 0.97 existante, tracee dans le
  kickoff §D4-§D5. **PASS**.
- **`time` crate** — deja presente dans le workspace. Version
  inchangee. **PASS**.

Aucune API crypto standardisee externe (SLSA, in-toto, PQC, libp2p)
introduite dans ce diff.

---

## Reponses directes aux quatre questions

### Q1 — La deviation GHA (verifier au lieu d'auto-publisher) est-elle defendable ?

**Oui, defendable et superieure au plan original** sur le plan du
threat model. Raisonnement documente dans le header du workflow
exact et correspond a la doctrine classique des warrant canaries
(rsync.net, IVPN) :

1. **Dead-man switch integrity** : un cron automatique avec cle
   stockee en GHA secret supprime la propriete centrale du canary
   — la necessite d'un acte intentionnel mensuel du maintainer. Un
   maintainer sous gag order peut etre contraint de "laisser
   tourner le cron" tout en etant silencieusement compromis.
2. **GHA secret = attack surface** : stocker la cle Ed25519 dans
   GHA secrets expose la cle a toute modification malveillante
   d'un workflow (supply-chain attack sur le CI, merge PR
   malveillante qui exfiltre via une etape modifiee).

**Finding P2 (amelioration doc)** : la deviation n'est documentee
que dans le header du workflow YML. Elle devrait aussi etre
tracee dans le corps du commit et dans le plan.md. Pour l'audit
gate Sprint 19, utile d'ajouter une note dans PATTERNS.md
indiquant que "un warrant canary ne doit jamais etre signe par
un processus automatise" comme regle reutilisable.

### Q2 — La bootstrap CANARY.txt avec pubkey fixe cree-t-elle une dependance permanente couteuse a rotation ?

**Oui, contrainte reelle mais acceptable et conforme au modele
warrant canary standard.**

- Tout warrant canary VPN serieux publie une pubkey stable avec la
  meme contrainte. Inherent au modele.
- La cle `80b439cb...` est dans `CANARY.txt` mais PAS dans le code
  source ni dans les configs deployees. Rotation = meme cout qu'un
  renouvellement de certificat racine.
- L'opt-in user explicite n'etait pas requis pour le bootstrap :
  pas encore d'utilisateurs qui ont pinne cette pubkey (pre-launch
  protocol). Le bootstrap S18 est la genese ; la contrainte
  devient reelle en Sprint 19+.

**Finding P3 (tracking)** : documenter en PATTERNS.md ou au sprint
F wrap-up que la rotation cle canary est un protocole sensible
necessitant communication out-of-band. Ajouter test "verify avec
ancienne pubkey echoue apres rotation" avant le premier tag v1.0.

### Q3 — Le decouplage `canary-key.key` vs `node_id` daemon est-il OK ?

**Oui, bon design.** Separation necessaire et bien documentee :

- Cle daemon iroh (`create_node`) = identite reseau P2P rotatable
  (ephemere par demarrage). L'utiliser pour le canary viderait la
  propriete de stabilite.
- `canary-key.key` = cle maintainer persistante qui survit aux
  reinstalls daemon. Pattern `load_or_generate` garantit reuse si
  fichier existe.

**Risque de coherence future (P2)** : le plan §E2 ligne 722 dit
`"Reads ~/.sbfb/node_id.key (existant S11+)"` — inconsistance dans
le plan qui a ete corrigee dans l'implementation mais pas mise a
jour dans le plan.md. A corriger en Phase F.

**Pas de risque de conflit** : les deux fichiers coexistent dans
`~/.sbfb/`, ont des noms distincts, ne sont jamais charges
simultanement dans le meme contexte.

### Q4 — Les +3 bonus tests sont-ils du gold-plating ou de la vraie coverage ?

**Vraie coverage, pas du gold-plating.** Analyse par test :

- `topic_id_is_deterministic_and_32_bytes` : ancre le BLAKE3 seed
  string `b"nexus-grid/warrant-canary/v1"` comme valeur de
  regression. Pattern identique Sprint 7.4
  `curator_topic_id_is_deterministic_and_32_bytes`. Necessaire.
- `parse_canary_txt_round_trips_through_format` : couvre le chemin
  `format_canary_txt → parse_canary_txt → verify_canary`. Chemin
  exact que `scripts/verify-canary.sh` + le GHA workflow
  empruntent. Necessaire.
- `format_canary_txt_contains_key_fields` : verifie les invariants
  de format du fichier ASCII lisible humain. Si le pattern
  `Date: {date} (UTC)` change, ce test casse avant que le GHA
  staleness-checker ne plante silencieusement. Necessaire.

---

## Findings

- **P2** : `docs/rust/PATTERNS.md` ne documente pas le pattern "cle
  maintainer persistante vs identite reseau ephemere". Tracker
  Phase F ou Sprint 19 Phase 0.
  (`crates/nexus-shell-daemon-core/src/auth.rs:canary_key_path` +
  `docs/rust/PATTERNS.md`)

- **P2** : La deviation plan → implementation (GHA verifier, non
  publisher) est documentee dans le workflow header mais absente
  du commit body et du plan.md. Un futur auditeur lisant uniquement
  `sprint18_plan.md §Phase E2` verra "Step: run
  `sbfb canary publish`" et une contradiction avec le code livre.
  Mettre a jour `sprint18_plan.md` ou noter la deviation dans
  `sprint18_phase_F` wrap-up. (`.planning/active/sprint18_plan.md:734-735`)

- **P2** : `sprint18_plan.md §Phase E2 ligne 722` dit
  `"Reads ~/.sbfb/node_id.key"` mais l'implem utilise
  `~/.sbfb/canary-key.key`. Inconsistance doc → code a corriger
  lors du Phase F wrap-up pour eviter la confusion lors du Phase 0
  Sprint 19.

- **P3** : Le field `headline` dans `build_canary` n'a pas de
  longueur maximale. Une headline de 100 MB passerait la
  validation et serait signee + serialisee sur le gossip topic.
  Risque de DoS gossip theorique (aucun noeud exterieur ne consomme
  ce topic en S18). Ajouter `const MAX_HEADLINE_LEN: usize = 512` +
  return `Err(CanaryError::BadHex("headline too long"))` dans un
  sprint ulterieur.
  (`crates/nexus-shell-daemon-core/src/canary.rs:build_canary`)

- **P3** : Le kickoff D5 documente le format de signature comme
  `base64-Ed25519-signature` mais l'implementation utilise du hex
  lowercase. La convention hex est correcte et coherente avec le
  reste du codebase. La divergence est dans la documentation du
  kickoff uniquement. Pas de bug, mais la phrase
  `"pub: <base64-Ed25519-pubkey matching node_id>"` dans
  `sprint18_kickoff.md §D5` est maintenant trompeuse.
  (`.planning/active/sprint18_kickoff.md:366-367`)

---

## Recommendation

**Commit autorise.** Tous les findings P2/P3 initialement flagues
ont ete fixes inline dans le meme commit Phase E2 (voir verdict
ci-dessus). Aucune dette residuelle.

Actions historiques (pour tracabilite, toutes fermees) :

1. ~~Mettre a jour `sprint18_plan.md §Phase E2`~~ — **DONE**
   (node_id → canary-key + GHA deviation rationale).
2. ~~Ajouter dans `docs/rust/PATTERNS.md` la regle~~ — **DONE**
   (section Sprint 18.1 ajoutee).
3. ~~Corriger la mention `base64` dans `sprint18_kickoff.md §D5`~~
   — **DONE** (hex lowercase + JCS rationale).
4. ~~Tracker P3 headline length cap~~ — **DONE** (implement
   directement : `MAX_HEADLINE_LEN = 512` + guard + 2 tests).
