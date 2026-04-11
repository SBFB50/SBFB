# Sprint 7 — Audit Plan (à jouer dans une session fraîche)

**Écrit** : 2026-04-11, en fin de Sprint 7, par l'agent qui vient
de livrer les 6 commits feat `2c896a8` → `6f32893`.

**Pourquoi ce document** : `.planning/sprint7_verification.md` est
une checklist fail-fast **self-reportée** par l'agent qui a écrit
le code. Tous les 32 rows passent — mais c'est le même agent qui
les a écrites et qui confirme qu'elles passent. Ce n'est pas une
vérification, c'est une auto-attestation. Le pattern
`sprint_audit_gate.md` rend l'audit structurellement obligatoire
avant d'ouvrir Sprint 8 Phase A.

**Principe** : le fail-fast dit "le code compile et les tests
passent". L'audit indépendant dit "le code fait ce qu'il prétend
faire, la surface testée correspond à la surface exécutée en prod,
et les décisions sont justifiées à la relecture".

---

## 0. Mode d'emploi pour la session fraîche

**Avant de commencer**, l'auditeur (agent ou humain) doit :

1. `git log --oneline master ^2926383` — lire les 8 commits
   Sprint 7 (1 kickoff + 6 feat + 1 doc verification + ce doc)
2. Lire dans cet ordre :
   - `.planning/sprint7_kickoff.md` (kickoff + §4 D1..D5 gelées)
   - `.planning/sprint7_plan.md` §4–9 (phases A..F détaillées)
     et §10 (fail-fast 30 rows cible)
   - `.planning/sprint7_verification.md` (self-report 32 rows)
3. **Ne pas lire** `docs/rust/PATTERNS.md` section "Sprint 7
   canonical" ni `docs/shell/PATTERNS.md` P9 avant d'avoir formé
   un avis sur la policy — l'objectif est de challenger les
   choix, pas les ratifier
4. Tenir un journal `.planning/sprint7_audit_findings.md` au fur
   et à mesure. Format par finding :
   `{track, severity, what, evidence, fix}`
5. Sévérités : **P0** (casse prod / data loss), **P1** (bloque
   Sprint 8), **P2** (tech debt explicite à logger dans
   PATTERNS.md), **P3** (nit, optionnel)

**Timebox suggéré** : 3 h. Audit indépendant, pas re-spec. Si un
track prend plus de 45 min, skipper et noter "timebox" ; la
session fraîche rapporte du signal en priorité sur du volume.

**Format du delivrable final** : une section par track ci-dessous
dans `.planning/sprint7_audit_findings.md`, chacune avec son
verdict PASS / CONCERN / FAIL + la liste des findings. Puis un
**verdict global** (PASS / CONDITIONAL PASS / FAIL) avec les
conditions pour lever un CONDITIONAL. Les P0 + P1 doivent être
corrigés en commits `fix(sprint7): ...` atterissant sur master
**avant** le premier commit Sprint 8 Phase A.

---

## 1. Track A — Intégrité du contrat cross-langue curator

**Question centrale** : le `CuratorListEntry` Rust
(`nexus_core_rs::curator`) et son miroir Zod côté shell
(`web/src/api/daemon.ts::CuratorListEntrySchema`) **valident-ils
exactement les mêmes payloads** ? La seule garantie actuelle est
que les tests cross-lang côté Python roundtripent via la PyO3 —
mais rien ne valide que le Zod côté shell accepterait le MÊME
blob.

### A1 — Structural diff Rust → Zod

**Méthode** :
1. Dans une REPL Rust ad hoc : mint 10 `CuratorListEntry` avec
   variations sur chaque champ (entries vides, entries au cap,
   curator_name unicode, description vide, revision 0, revision
   2**63-1, created_at 0, created_at dans le futur), les sérialiser
   en JSON via serde_json.
2. Pour chaque blob, `CuratorListEntrySchema.safeParse()` côté
   shell — attendu : TOUS passent.
3. Inversement : construire un blob Zod-valide via
   `CuratorListEntrySchema.parse(...)` (ou écrire à la main) qui
   contourne la validation Rust ; chercher un champ que Rust
   accepte mais Zod refuse, ou vice-versa.

**Signal d'audit** :
- P0 si un payload Rust-valide est Zod-rejeté (le shell ne pourrait
  pas afficher une liste que la daemon a verifiée)
- P1 si l'asymétrie est acceptable mais non documentée
- P2 si Zod est strictement plus strict que Rust (acceptable)

### A2 — `curator_pubkey` encoding

Rust sérialise `[u8; 32]` comme JSON array of numbers. Zod schema
déclare `z.array(z.number().int().min(0).max(255)).length(32)`.
Vérifier explicitement que :
- un byte 0 passe
- un byte 255 passe
- un byte -1 est rejeté (négatif)
- un byte 256 est rejeté (hors plage)
- un array de longueur 31 est rejeté
- un array de longueur 33 est rejeté

Probablement trivial à valider, mais c'est LE point où un Zod
approximatif laisserait passer des bytes invalides qui casseraient
un re-sign Rust.

### A3 — Re-signature roundtrip

Le shell ne re-signe PAS de curator lists (Phase E est pull-only).
Mais si un futur Sprint 10 ajoutait un curator-CLI côté Python,
il faudrait que la Python côté `sign_curator_list` produise un
blob que Rust `verify_signature` accepte **ET** que Zod
`CuratorListEntrySchema` parse. Vérifier ce triangle existe au
moins en 1 test cross-lang dans Sprint 7 — si non, c'est un P2
gap à logger.

**Verdict track** : PASS / CONCERN / FAIL sur la cohérence Rust↔Zod.

---

## 2. Track B — Crypto resilience & envelope attacks

### B1 — Attribution split-brain, cas limites

`CuratorListEntry::verify_signature` layer 4 checks (version, cap,
attribution, signature). L'audit teste les croisements que les
unit tests ne couvrent pas :

1. **Envelope pubkey = garbage bytes, payload correct** : que se
   passe-t-il si `entry.curator_pubkey` contient des zéros mais
   `entry.list.curator_pubkey` est le vrai signer ? Est-ce qu'on
   rejette avant le fetch (au stade attribution) ou après ?
2. **Payload pubkey = garbage bytes, envelope correct** : idem
   dans l'autre sens.
3. **Signature valide sur une liste minuscule, payload gonflé
   à la verification** : un attaquant peut-il tailler une liste
   `entries.len() ≤ cap` qui passe la signature, puis modifier
   `entries` pour en ajouter 1 avant de la publier ? Ça doit
   échouer au check signature (la signature couvre `entries`),
   mais vérifier que l'ordre des checks dans le code (version →
   cap → attribution → signature) ne fait pas d'allocation
   proportionnelle à `entries.len()` AVANT le rejet cap.

### B2 — Revision rollback + replay

Scénarios à tester à la main (curator-side + daemon-side) :
1. Daemon stocke rev 5. Attaquant rejoue une ancienne annonce rev 3
   → doit être ignorée sans log alarming (c'est attendu, pas
   hostile).
2. Daemon stocke rev 5. Curator légitime bump à rev 6 et broadcast
   → doit être accepté.
3. Daemon stocke rev 5. Curator re-émet rev 5 avec un `created_at`
   différent mais même entries → doit être ignoré (strict > revision).
4. Daemon démarre à froid. Attaquant broadcast rev `u64::MAX` signé
   valide → est accepté, puis le curator légitime ne peut plus
   jamais le remplacer. **C'est un DoS permanent potentiel.**
   Vérifier si le code prévoit quoi que ce soit contre ça (spoiler :
   non, c'est un **P1** à logger).

### B3 — Timing des checks dans `verify_signature`

Lecture ligne-à-ligne de `curator.rs::verify_signature` : l'ordre
est version → cap → attribution → signature. L'audit se demande si
le **cap check avant signature** est un gain de DoS ou une perte
de info (on ne sait pas si la liste OVERSIZED était signée ou
forgée). Justifier et documenter.

---

## 3. Track C — Gossip ingest pipeline robustness

### C1 — Ordering dans `process_announcement_bytes`

Lire
`crates/nexus-shell-daemon-core/src/iroh_runtime.rs::process_announcement_bytes`
ligne par ligne. 9 steps documentés. Pour chaque step, poser la
question "et si ça échoue ?" et vérifier que le code :
- Ne fait pas de travail non-réversible (write disk, update stats)
  qu'un step ultérieur pourrait vouloir annuler
- Ne bloque pas la loop gossip plus longtemps que nécessaire
- Logge au bon niveau (debug pour les rejets attendus, warn pour
  les anomalies)

### C2 — `handle_announcement` qui swallow les erreurs

`runtime.rs::handle_announcement` silently drops
`AnnouncementAttributionMismatch` en `debug!`. L'argument est
"non-subscribed curator + mismatch partagent la même variante" —
c'est un code smell. Vérifier si on pourrait confondre :
1. Un vrai attaquant qui staple une liste à la mauvaise clé
2. Un curator légitime dont on s'est simplement pas encore abonné

Si oui, split la variante en deux (`NotSubscribed` vs
`AttributionMismatch`). Sinon, ajouter un test qui lock le
comportement.

### C3 — Panic safety dans la gossip task

Est-ce que la task gossip peut paniquer sous une entrée
malformée ? Relire chaque `unwrap()`/`expect()` dans
`iroh_runtime.rs` et `runtime.rs::spawn_gossip_subscribe_task`.
Un panic dans cette task DROP le `Arc<Node>` → empêche
`Arc::try_unwrap` dans `shutdown` → iroh node reste ouvert →
`running.json` reste en place → singleton bug au prochain boot.

### C4 — Backpressure en cas de flood

Si un attaquant broadcast 10 000 annonces/s sur le topic,
`process_announcement_bytes` est appelé séquentiellement dans la
loop gossip (pas de concurrent processing). Conséquence :
- Chaque annonce déclenche un `fetch_ticket` (connexion iroh)
- La loop sérialise → débit max ~ nb probe/s
- DoS : la loop absorbe tout mais la daemon devient
  irresponsive sur les `/curators` / `/browse` HTTP ? Non, l'HTTP
  task est séparée — mais les fetches iroh peuvent saturer
  l'endpoint.

Logger comme **P2** : ajouter un rate limiter (ou un semaphore)
devant `process_announcement_bytes` en Sprint 8 ou 9.

---

## 4. Track D — Singleton enforcement edge cases

### D1 — Pid recycling path

`registry::is_process_alive` appelle `System::new_all()` puis
lookup le pid + compare le nom via `process_name_matches`. Audit :
- Est-ce que `System::new_all()` garantit de voir un process qui
  vient d'être créé il y a 10 ms ? sysinfo 0.32 utilise un refresh
  explicite sur construction — mais sur Windows la latence peut
  être significative. Risk : false negative sur un restart rapide.
- `process_name_matches` normalise hyphen→underscore + lowercase.
  Est-ce qu'il existe un nom de process système Windows qui contient
  `nexus_shell_daemon` mais n'est PAS notre daemon ? (e.g. un
  utilisateur a renommé son own binary.) → test avec un process
  synthétique nommé `nexus_shell_daemon_launcher.exe`.

### D2 — `running.json` atomic write race

Phase A utilise `write_running(tmp) + rename`. Sur Windows NTFS,
`rename` est atomique pour des moves dans le même dossier. Mais
si l'antivirus tient `running.json` ouvert au moment du rename,
le rename échoue → la daemon bail out après avoir déjà spawné
iroh. Est-ce qu'on GC proprement l'iroh Node dans ce path ?

### D3 — Subscriptions.json persistence vs crash

Si la daemon crash entre `attention.insert(pk, ())` et
`persist_subscriptions()`, l'utilisateur voit un état
incohérent. Lire le code et vérifier l'ordre d'operations
(insert-in-RAM → persist) — si persist échoue, l'attention set
contient la clé mais le fichier non. Pire, au prochain boot on
charge le fichier et on perd la subscription. Est-ce tracké /
testé ?

---

## 5. Track E — Pkarr probe correctness + test contamination

### E1 — `probe_reachable` vs pkarr real-world latency

Le timeout 2 s est-il suffisant pour un vrai lookup pkarr en
production ? Sur un home NAT avec relay n0, la première
résolution peut prendre 3-5 s (relay round-trip + UDP
traversal). Conséquence : premier probe → timeout → Unreachable →
TTL cache 60 s → l'utilisateur voit "injoignable" pendant 1 min
pour un projet qui est en fait accessible.

### E2 — `probe_reachable_finds_a_seeded_local_peer` contamination

Le test unit `probe_reachable_finds_a_seeded_local_peer` utilise
2 nodes locaux et seed manuellement le memory_lookup. C'est une
technique valide, mais l'audit teste si l'ordre d'opérations
réel (gossip arrive → fetch_ticket → `memory_lookup.add_endpoint_info(addr)`)
seed assez vite pour qu'un `probe_reachable` subséquent dans le
même process puisse résoudre SANS second seeding. Probablement
oui (iroh-blobs fetch_ticket seed le memory_lookup pour le dial),
mais le test actuel mocke cette étape et skip le code réel.
Lister comme **CONCERN** et recommander un test 3-node (publisher
→ daemon pkarr-only → probe) en Sprint 8.

### E3 — Cache TTL expiration

`BrowseAggregator::cached` utilise `duration_since` avec TTL
`DEFAULT_PROBE_TTL = 60s`. Le test `cached_expires_after_ttl`
utilise 1ms TTL + sleep 10ms. OK. Mais : si l'horloge système
recule (daylight saving sur certains fuseaux, NTP sync),
`duration_since` retourne `Err` → le fallback dans le match
(`_ => None`) traite le cache comme expiré. Safe but
un-documented. Note en **P3**.

---

## 6. Track F — Shell UX dans les états dégradés

### F1 — `DaemonOfflineBanner` re-render sur focus/blur

React Query avec `refetchOnWindowFocus: false` — OK. Mais quand
l'utilisateur démarre la daemon puis revient à la page :
- La query est stale (60 s)
- Pas de refetch auto
- L'utilisateur voit "offline" pendant jusqu'à 60 s

Solution : `refetchInterval: 15_000` + un bouton "Rafraîchir"
visible. Le bouton existe déjà dans Browse (testid
`browse-refresh`). Mais Curators n'en a pas. **P2**.

### F2 — Hex case-sensitivity côté shell

`isValidCuratorPubkey` rejette l'uppercase. Mais un utilisateur
typique copie/colle depuis un README qui peut utiliser
l'uppercase. Proposition : `.toLowerCase()` + validation avant
submission. Le code fait déjà `.toLowerCase()` dans le form
handler — vérifier que c'est le cas et tester.

### F3 — Accessibility des pages Browse/Curators

`CardTitle` est un `<div>` (shadcn vendored). Les pages Phase E
n'ont donc **aucun** `<h2>` / `<h3>` — les screen readers ne
trouveront pas de hiérarchie. Pas un blocker Sprint 7, mais un
**P2** à logger pour un futur sprint accessibility.

### F4 — Toast d'erreur pour la subscribe mutation

`Curators.tsx` store l'erreur dans `formError` (texte inline).
Pas de toast. Le pattern shadcn/sonner serait plus visible. **P3**
nit.

---

## 7. Track G — Coordinator proxy security

### G1 — httpx timeout cumulation

`_forward` crée un nouveau `httpx.AsyncClient` par call. La connect
timeout est 2 s, read 10 s. Pour un call `/browse` avec 50 projets
cachés mais 5 nouveaux à probe, le daemon prend ~10 s worst-case.
Sous une rafale de F5, la coordinator accumule les connexions
→ risque thread pool exhaustion.

**Vérifier** : y a-t-il une limite sur le nombre de clients httpx
concurrents ? Si non, documenter comme **P2** + proposer un
`httpx.Limits(max_connections=10)`.

### G2 — CORS trust boundary

Le daemon a sa propre loopback CORS (tower_http). Le coordinator a
la sienne (FastAPI CORSMiddleware). Chaque règle est correcte
isolément. Mais :
- Si le coordinator redirige vers le daemon (pas actuellement le
  cas), la CORS preflight serait doublée.
- Si un futur sprint expose le coordinator hors loopback (e.g. LAN),
  le CORS regex devrait être resserré.

Note : pas un problème Sprint 7 mais **à documenter** dans
`docs/shell/PATTERNS.md` P9 update.

### G3 — `json_body` forward verbatim

`_forward` passe le body Python dict au daemon. Si un attaquant
peut injecter un champ supplémentaire (par exemple `curator_pubkey_hex`
+ `secret_injection_attempt`), le daemon le voit. Le daemon rejette
les champs inconnus via serde `#[serde(deny_unknown_fields)]` ?
→ Vérifier dans `SubscribeCuratorRequest` si `#[serde(deny_unknown_fields)]`
est posé. Si non, **P2**.

### G4 — `DaemonUnavailable` fuite d'info

Le champ `reason` dans l'envelope unavailable carrie le path du
`running.json` via les logs httpx ? Vérifier ce qui apparaît dans
`body["reason"]` quand le daemon n'est pas là — si ça contient
`%APPDATA%\nexus-grid\shell-daemon\running.json`, c'est un info
leak P3 (pas critique, path connu de toute façon).

---

## 8. Track H — Cross-dependency hygiene

### H1 — sysinfo / iroh / axum / tower-http versions

Vérifier dans `Cargo.toml` workspace :
- iroh pinné à 0.97 (R2 locks 0.8)
- iroh-blobs 0.99
- axum 0.7 (non 0.8)
- tower-http 0.6 features = `["cors"]`
- sysinfo 0.32
- dashmap 6

Est-ce qu'un `cargo update` accidentel bumperait axum à 0.8 ?
Vérifier que le pin est `= "0.7"` (exact) ou `"0.7.*"` (latest
minor). Le plan §12 R2 demande exact pin ; si le workspace utilise
`"0.7"`, c'est caret range et bumpe à 0.7.x mais pas 0.8.

### H2 — httpx absence pré-Sprint 7 ?

L'agent Explore a rapporté "No existing httpx.AsyncClient in the
coordinator codebase" mais `httpx>=0.27` est déjà déclaré dans
`pyproject.toml`. Vérifier cohérence : est-ce qu'un autre module
utilise httpx aujourd'hui ? `grep httpx packages/nexus-coordinator/`.
Si oui, l'affirmation de l'agent était fausse et l'audit doit
s'assurer que la Phase E n'a pas introduit une import collision.

### H3 — `nexus_core` wheel editable install drift

L'incident Phase E était que le wheel PyO3 installé dans `.venv`
avait disparu entre Phase B (install) et Phase E (test run). Cause
probable : un `uv sync` quelque part a écrasé le `.pth` de
l'editable install. Tracer la cause exacte et documenter : soit
- Ajouter `nexus-core-py` en dep dans `pyproject.toml` workspace
- Soit automatiser le `maturin develop` dans un `scripts/setup.sh`

**P1** — c'est un blocker reproductibilité pour la CI de Sprint 8.

---

## 9. Track I — Documentation & traceability

### I1 — `docs/shell/PATTERNS.md` P9 cohérent avec le code

Lire P9 après avoir lu le code Phase E. Est-ce que le pattern
décrit correspond à ce qui est livré ? Contre-exemple courant :
le pattern parle de "httpx.AsyncClient global" mais le code crée
un client par call.

### I2 — `docs/rust/PATTERNS.md` Sprint 7 canonical section

Vérifier que la section inclut :
- `DOMAIN_CURATOR_LIST_V1` + tag name
- `CuratorListEntry::verify_signature` order (version / cap /
  attribution / signature)
- `CURATOR_LIST_MAX_ENTRIES = 256` + rationale R5
- `CURATOR_TOPIC_SEED = b"nexus-grid/curator/v1"` + R6 rollback
- `probe_reachable` using `BLOBS_ALPN` with 2s default timeout

### I3 — Sprint 7 plan §12 scope cuts vs reality

Pour chaque scope cut listé (pkarr publish, app context submit_task,
nexus_command, etc), vérifier par grep qu'**aucune** ligne de code
Sprint 7 ne le touche. C'est la vérif de dérive de scope — un
agent éxécutant peut accidentellement tirer le fil d'un scope cut.

### I4 — Commit messages vs reality

Chaque commit Sprint 7 (`2c896a8`..`6f32893`) contient dans son
body le delta de tests attendu. Vérifier par rejouer
`cargo test` + `pytest` au tip de chaque commit individuel (via
`git stash` + `git checkout`) que les comptes matchent. C'est
onéreux en temps ; timeboxer à 1 commit au hasard.

---

## 10. Verdict global attendu

Deux scénarios possibles quand la session fraîche finit cet audit :

**PASS** : aucun finding P0 ni P1. Les P2/P3 vont dans
`docs/shell/PATTERNS.md` et `docs/rust/PATTERNS.md` tech debt
sections. Sprint 8 Phase A peut démarrer direct.

**CONDITIONAL PASS** : 1 ou 2 findings P1 clairement fixables en
commits `fix(sprint7): ...` dédiés. L'auditeur liste les commits
nécessaires + les critères de lève de condition. Sprint 8 Phase A
ne démarre QU'après les fix + une session de verify rapide.

**FAIL** : ≥ 1 finding P0, ou ≥ 3 findings P1. L'audit demande
une re-conception partielle. Très improbable vu la discipline du
plan + les 147 nouveaux tests, mais c'est le seul cas qui force
un `rollback` ou un rework avant Sprint 8.

---

## 11. Out of scope pour l'audit Sprint 7

L'auditeur ne doit PAS challenger :

- **Le choix D1 HTTP loopback via coordinator proxy** — gelé
  Sprint 7 Day 0 §4, non re-débattable
- **Le choix D2 singleton strict** — idem
- **Le choix D3 curator schema + topic + domain** — gelé
- **Le choix D4 task_submit Option B** — gelé, signature attendue
  Sprint 8 Phase A
- **Le choix D5 @nexus_command frozen** — idem
- **L'absence de publish pkarr** — scope cut §12 explicite
- **L'absence de bootstrap peers VPS** — scope cut §12 explicite

Si l'audit a une raison technique NOUVELLE d'invalider une
décision Day 0, il doit la logger comme **"décision à rouvrir en
Sprint 8 Day 0"** et ne PAS bloquer Sprint 7.

---

## 12. Livrable final attendu

`.planning/sprint7_audit_findings.md` avec :

1. Une section par track (A..I) — verdict + findings
2. Un verdict global (PASS / CONDITIONAL PASS / FAIL)
3. Si CONDITIONAL PASS : liste des commits fix attendus, chacun
   avec son critère de lève
4. Une liste des P2 à logger en tech debt dans `docs/shell/PATTERNS.md`
5. Une liste des P3 laissés sans action
6. Signature : "audité par session {id}, timebox observée {h}h"

**Sans ce fichier, Sprint 8 Phase A ne peut pas démarrer.** C'est
le point non-négociable de `sprint_audit_gate.md`.
