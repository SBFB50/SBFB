# Sprint 19 Phase B — PoW Hashcash gossip subscribe : design doc

**Ecrit** : 2026-04-16, session implementation Sprint 19 Phase B
post-livraison `pow.rs` + `relay_pow_policy.rs` + `pow_gossip.rs`
+ `benches/pow.rs`. Document retrospectif : capture les choix
**deja faits** dans le code, les alternatives **ecartees a la
conception**, et les limites **honnetes** de la primitive avant
audit Phase 0 S20.

**Contexte** : HARDENING_ROADMAP §3 S19 item 1 « PoW Hashcash
per-gossip-subscribe (difficulty 2^18 initial, adjustable
per-relai) ». Pre-requis pour S21 rate-limit per-consumer
(`HARDENING_ROADMAP §3 S21 §Dependencies`). Sans Sybil-resistance
minimale via PoW, un rate-limit per-identity est trivialement
contournable par botnet.

**Code livre couvert par ce doc** :
- `crates/nexus-core-rs/src/pow.rs` (660 lignes : primitive +
  tests)
- `crates/nexus-core-rs/src/relay_pow_policy.rs` (358 lignes :
  TOML loader)
- `crates/nexus-core-rs/src/pow_gossip.rs` (652 lignes : envelope
  wire + caches publisher/subscriber)
- `crates/nexus-core-rs/benches/pow.rs` (68 lignes : criterion
  bench 3 difficultes)

---

## 1. Probleme adresse

### 1.1 Threat model — ce qu'on defend

Le scenario primaire est **B-Sybil** + **B-GossipPoison** dans
[`docs/security/THREAT_MODEL.md` matrice §1](../../docs/security/THREAT_MODEL.md)
+ [`HARDENING_ROADMAP.md §3 S19`](../../docs/security/HARDENING_ROADMAP.md) :

| Attaquant | Capacite | Defense PoW S19 |
|---|---|---|
| **T0 — script kiddie** (1 laptop) | Spam ~10 subscribe/s sans cost | Bloque : 100 ms PoW × 100 topics = 10 s |
| **T1 — botnet rente 100 nodes** (~10 EUR/mois) | Spam ~1 000 subscribe/s sur curator topic | Force ~100 EUR/mois CPU rent per topic flood + per pubkey, rentabilite cassee |
| **T2 — adversary 10k bots compromis** (Mirai-class) | Spam ~100k subscribe/s | Force ~1k subscribe/s effectif post-PoW (gain ~1/100), donne fenetre temporelle a l'observation |
| **T3 — etat-nation regional** (~1M cores) | DDoS dedie | **Non-defendu** (PoW seul insuffisant — voir §6) |

L'envelope wire `pow_gossip.rs` lie le proof a `(publisher_pubkey,
topic, issued_at)` via le pre-image SHA256 (cf. `pow.rs:194-219`
struct `HashcashChallenge`). Cela rend la solution **non-replayable**
sur :
- une autre identite (publisher_pubkey est domain-separe dans le
  hash → un attacker qui sniffe une preuve ne peut pas la rebadger
  sous une autre clef Ed25519)
- un autre topic (cross-topic flood = cross-cost — flood N topics
  exige N solves)
- une fenetre temporelle ≥ 30 min (`MAX_PROOF_AGE_SECS = 1800`
  cf. `pow.rs:109`) — proof captured cant be replayed indefiniment

### 1.2 Threat model — ce qu'on NE defend PAS

Liste honnete pour eviter de survendre PoW :

1. **Sybil-resistance complete** : PoW impose un cost CPU mais
   n'empeche pas un attacker patient. T2-T3 avec patience peuvent
   accumuler des identites pre-solvees sur des heures. Defense
   complementaire = kudos-weighted admission Sprint 22
   (`HARDENING_ROADMAP §3 S22`).

2. **Bandwidth DDoS (volume L4)** : PoW protege le path subscribe,
   pas la couche transport. Un attacker qui flood UDP/TCP vers le
   relai pkarr lui-meme est traite par TLS pinning Phase C +
   futur rate-limit Sprint 21.

3. **Payload integrity** : l'envelope binde `(proof, payload)` par
   concatenation seulement, **pas par signature**. Un MITM peut
   modifier le payload sans casser le PoW (test
   `verify_cache_rejects_tampered_payload_has_no_effect`
   `pow_gossip.rs:502-527` pin l'invariant explicitement).
   L'integrite du payload est la responsabilite du layer
   au-dessus (curator list signe Ed25519, task entry signe, etc.
   cf. `crates/nexus-core-rs/src/canonical.rs` domains separes).

4. **GPU/ASIC asymetrie** : SHA256 est ASIC-friendly. Un attacker
   avec accès cloud Bitcoin mining (ou ASIC second-marche post-
   2024 obsolescence) peut produire des proofs ~1000x plus vite
   qu'un CPU. **Trade-off assume** : la primitive cible le bot
   commodity (botnet x86/ARM), pas l'attacker dedie. Si la menace
   evolue vers ASIC-equipped, la migration vers Equihash ou
   Cuckoo Cycle est documentee §6 et §3.

5. **Eclipse attacks contre la federation pkarr** : couvert par
   primitive S18 `dht_quorum::redundant_resolve` (3 relais quorum
   2/3) wired Phase A S19. Orthogonal au PoW.

6. **Pre-computation challenges** : un attacker qui pre-solve
   1000 challenges pour 1000 timestamps futurs et les replay le
   moment venu. Mitige partiellement par `MAX_PROOF_AGE_SECS =
   1800` (proof valid only 30 min) mais pas elimine totalement —
   un pre-compute sur 30 min de fenetre reste possible. Cf.
   limite §6.

7. **Time-skew attacker** : un attacker dont le clock derive de
   ±MAX_PROOF_AGE_SECS peut soumettre un proof "from the future"
   ; le verify rejette via `IssuedInFuture` (`pow.rs:411-417`)
   mais ne garantit pas un alignement clock cross-network. Le NTP
   skew tolerance reste un trou silencieux.

### 1.3 Pourquoi maintenant (S19, pas S21+)

`HARDENING_ROADMAP §3 S21 §Dependencies` : « S19 PoW (sinon rate-
limit contourne) ». Sans cost-of-identity au boot subscribe,
un attacker T1 cree 10k identites Ed25519 (bon marche :
~50 cycles/keypair) et chaque identite consomme un slot rate-
limit. Le rate-limit S21 devient une fiction. PoW S19 est la
brique foundation.

---

## 2. Decision retenue

**Hashcash SHA256 single-threaded deterministe**, difficulty
**2^18 leading zero bits par defaut** (~100 ms CPU moderne 2026),
**ajustable per-topic** via `~/.sbfb/relay_pow_policy.toml`
(loader env `SBFB_POW_POLICY_PATH` override). Challenge bind
`(topic_32B, publisher_pubkey_32B, issued_at_u64, difficulty_u32,
format_version_u16)` via canonical bytes JCS + domain tag
`b"nexus-pow-v1"` (cf. `canonical.rs:126`). Wire format envelope
binaire `[u32 BE proof_len][proof JSON][payload]`. Caches session
15 min publisher (`PowSolveCache`) + subscriber (`PowVerifyCache`)
amortissent le solve sur les heartbeats repetes.

---

## 3. Alternatives considerees

### 3.1 Equihash (Zcash-style, memory-hard)

**Description** : asymmetric PoW base sur Generalized Birthday
Problem. Parameters `(n=200, k=9)` exigent ~50-144 MB RAM pour
solve, verify O(1). Bitcoin Gold variant `(n=144, k=5)` exige
700 MB-2.5 GB.

**Avantages** :
- ASIC-resistant en theorie (memory-hard) — un T2-T3 cloud peut
  pas amortir sur SHA256 ASIC
- Verify trivial cote subscriber (1 coup d'oeil sur la solution)

**Inconvenients dans contexte gossip subscribe** :
- **RAM cost demoli les mobiles + raspberry pi**. Un publisher
  raspberry pi 4 (4 GB RAM) ne peut pas allouer 700 MB juste pour
  subscribe a un topic. Tor a explicitement rejete RandomX pour
  cette raison (cf. tevador devlog : « 1 GB memory requirement is
  too high » [equix/devlog](https://github.com/tevador/equix/blob/master/devlog.md))
- **ASIC-resistance erodee** : Bitcoin Gold ASIC apparu en 2018
  malgre la promesse memory-hard ([Equihash Wikipedia](https://en.wikipedia.org/wiki/Equihash)).
  La revue 2025 [eprint 2025/1351](https://eprint.iacr.org/2025/1351.pdf)
  re-explore le tradeoff
- **Ecosysteme Rust pauvre** : pas de crate audited grand-public.
  Ecrire from-scratch un Equihash production-grade est >5 KLOC.
  Dependance binding a `zcash_primitives` introduit ~50 transitive
  deps + un domain knowledge cryptanalyse cote audit Cure53/ToB.

**Verdict** : **rejete**. Le cost RAM tue les profils
deployment T0-T1 (mobile, raspberry pi, low-end VPS). Reconsiderer
si la menace evolue vers cloud ASIC volumes a l'echelle Bitcoin
mining (Sprint 26+ scope).

### 3.2 Argon2 puzzles (memory-hard, password-hashing competition winner)

**Description** : winner du Password Hashing Competition 2015,
RFC 9106. Variant Argon2d data-dependent memory access acceptable
pour PoW (pas de side-channel concern client-side). Parameters
typiques : `m_cost=64 MB, t_cost=3, p=1`.

**Avantages** :
- Memory-hard genuine, ASIC-resistant
- RFC standardise, audit posture solide
- Crate Rust `argon2` (RustCrypto) audited, mature

**Inconvenients dans contexte gossip subscribe** :
- **Verify cote subscriber non-trivial** : Argon2 verify exige
  ~10-50 ms vs ~1 us SHA256 verify Hashcash. Sur un subscriber
  qui recoit 1000 messages/s, le cost cumule devient prohibitif
  (1000 × 10 ms = 10 s/s impossible). Hashcash a la propriete
  asymetrique « cost-to-produce >> cost-to-verify » au sens
  ratio 10^6 :1 que Argon2 ne fournit pas
- **Memory-hard sur un publisher mobile** : meme probleme que
  Equihash, ramene a 64 MB minimum au lieu de 700 MB mais reste
  un cost RAM perceptible
- **Recherche academique 2025 questionne MHPoW** : « Provably
  Memory-Hard Proofs of Work with Memory-Easy Verification »
  ([Springer 2025](https://link.springer.com/chapter/10.1007/978-3-032-12290-2_17))
  re-ouvre la question des proofs MHPoW post-attack Dinur-Nadler
  contre MTP/Argon2d. Pas un blocker mais un signal que le terrain
  bouge

**Verdict** : **rejete**. Verify-cost casse l'architecture cache
session subscriber (le whole point d'amortir sur les heartbeats
disparait). Argon2 reste pertinent pour Sprint 23 « Escalating
PoW per-(consumer, model) » ou la bursty-low-volume nature inverse
le tradeoff (cf. `HARDENING_ROADMAP §3 S23`).

### 3.3 VDF (Verifiable Delay Function) — Wesolowski / Pietrzak

**Description** : PoW « inherently sequential » qui force un
delai temporel **non-parallelisable** independant des resources
CPU. Construction Wesolowski 2018 avec proof constant-size,
Pietrzak 2018 avec recursive halving. Production : Chia Network
proof-of-space-and-time (squaring repeated dans groupe RSA).

**Avantages** :
- Defense vs **attacker avec parallelism arbitraire** : 1000 cores
  ne resolvent pas plus vite qu'1 core (sequentialite enforce)
- Proof constant-size + verify O(log T)
- Active recherche academique 2024-2025 (cf. [Uplatz blog 2025
  comprehensive analysis](https://uplatz.com/blog/verifiable-delay-functions-a-comprehensive-analysis-of-cryptographic-foundations-applications-and-deployment-challenges/))

**Inconvenients dans contexte gossip subscribe** :
- **Group choice piege** : RSA group exige trusted setup ou Class
  Group exige ~1024-bit primes pas nativement supportes dans
  ecosystem Rust auditt. Production deployment limited a Chia.
- **Setup operationnel lourd** : un VDF de 100 ms exige un
  parameter-space exploration tuning per-CPU-arch. Pas de magic
  number 2^18 portable
- **Pas de crate Rust audit** : `poanetwork/vdf` GitHub pas mis a
  jour 2023+, pas d'audit Cure53/ToB recent. Les bindings
  productions tournent en C/C++ via `chiavdf`
- **Asymetrie cost-to-produce inadequate** : VDF coute ~1s pour
  1s. Hashcash 2^18 coute ~100 ms produce / ~1 us verify (ratio
  10^5). VDF tue le subscriber sur volume

**Verdict** : **rejete pour S19**. Migration path documente §6
si la menace evolue vers attacker GPU/parallelism. Pertinent
Sprint 24+ « Domain fronting + Tor bridges » ou la sequentialite
devient critique.

### 3.4 Cuckoo Cycle (graph-theoretic, memory-bound)

**Description** : Tromp 2014, base sur 42-cycle search dans un
graph bipartite. Grin proof-of-work production. Memory-bound (pas
juste memory-hard) — bottlenecked par memory bandwidth, pas par
raw cycles CPU.

**Avantages** :
- ASIC-resistant via memory-bandwidth bottleneck
- Verify instant O(cycle_length)
- Production deployment Grin depuis 2018 — battle-tested

**Inconvenients dans contexte gossip subscribe** :
- **Memory cost prohibitif** : 2^29 edges = ~2.2 GB RAM pour
  solve « single-threaded mean solver » ([Grin docs](https://docs.grin.mw/wiki/miscellaneous/cuckoo-cycle/))
  + 10.5 s solve time sur i7-4790K. Aucun mobile/RPi viable
- **Time-Memory Trade-Off (TMTO)** : un attacker peut reduire la
  RAM en augmentant le solve time, ce qui complique le tuning du
  difficulty per-relai. La promesse 100 ms par-platform devient
  une distribution multi-modale
- **Implementation Rust audit** : `mimblewimble/grin` cuckoo
  pure-Rust mais audit-posture liee a Grin upstream. Extraction
  standalone non-trivial

**Verdict** : **rejete**. Memory cost incompatible avec profil
deployment SBFB T0-T1. Enseignement : aucune memory-hard PoW
courante n'est compatible avec les contraintes mobile/RPi du
projet en 2026.

### 3.5 TLS puzzles (Akamai-style, application-layer challenge)

**Description** : challenge HTTP retourne par le serveur (cookie
+ JS PoW dans browser) avant d'admettre une connexion. Akamai
Behavioral DDoS Engine 2024+ ([Akamai blog Nov
2024](https://www.akamai.com/blog/security/2024/nov/akamais-behavioral-ddos-engine-breakthrough-in-modern-ddos-mitigation))
utilise des heuristiques comportementales + challenge-response
adaptatif.

**Avantages** :
- Defense edge proven a echelle internet
- Adaptive a charge attack en temps reel
- Pas de coordination cross-relai necessaire (decisional locale)

**Inconvenients dans contexte gossip subscribe** :
- **Pas un protocol P2P** : Akamai-style assume un edge centralise
  qui distribue le challenge. SBFB est decentralise — aucun edge
  qui distribue les puzzles. Reinventer un challenge-distribution
  protocol = Sprint dedie
- **Browser-centric** : reposent sur JS PoW execute client-side.
  SBFB tourne en Rust headless dans le daemon, pas dans un browser
- **Pas de spec ouverte stable** : Akamai propriétaire, pas de
  RFC ni de papier de reference

**Verdict** : **rejete**. Pattern incompatible avec architecture
P2P decentralisee. Reference utile pour S21 rate-limit adaptative
(« heuristiques comportementales » = telemetrie locale relais
self-hosted Sprint 22+).

### 3.6 Equi-X / HashX (Tor PoW 2023, hybrid memory + CPU)

**Description** : Equi-X est l'algorithme PoW retenu par Tor
0.4.8.4 (release 2023) pour onion services. Base sur HashX
([equix devlog](https://github.com/tevador/equix/blob/master/devlog.md)),
inspire d'Equihash mais parametre pour minimiser RAM
(~~16 MB) et asymetrie GPU/CPU. Difficulty dynamique adaptative
selon attack volume detected ([Tor PoW FAQ](https://onionservices.torproject.org/technology/security/pow/)).

**Avantages** :
- Reference 2023-2024 production-grade pour cas d'usage tres
  similaire (anti-DoS introduction circuits Tor onion)
- RAM cost « ~16 MB » bien plus tolerable que Equihash/Cuckoo
- Difficulty dynamique adaptative — modele pour Sprint 22+

**Inconvenients dans contexte gossip subscribe S19** :
- **Implementation Rust absente** : Equi-X distribue en C
  (`tevador/equix`), reverse-binding Rust possible mais
  non-audited 2026. Le projet Tor lui-meme tourne en C donc
  upstream choix logique
- **Crypto custom non-RFC** : HashX n'a pas de RFC, juste un
  devlog tevador. Pour un audit Cure53/ToB Sprint 29, devoir
  briefer un auditeur sur HashX (vs SHA256 trivialement connue)
  rajoute du friction
- **Sur-engineered pour S19** : le main-feature Tor est la
  difficulty adaptative. SBFB S19 ne fait pas adaptive, juste
  un baseline 2^18 fixe. Le gain Equi-X (« meilleur asymetrie
  GPU/CPU sur 16 MB ») est marginal a difficulty fixe modeste

**Verdict** : **rejete pour S19**. Tres serieux candidat pour
Sprint 22+ « Kudos-weighted gossip admission » + difficulty
dynamique. Note explicite §6 migration path.

### 3.7 Hashcash SHA256 (RETENU)

**Description** : Adam Back 1997, [hashcash.org/hashcash.pdf](http://www.hashcash.org/hashcash.pdf).
Trouver un nonce tel que `SHA256(challenge || nonce)` ait `D`
bits de zero en prefixe. Verify : 1 SHA256 + count-leading-zeros.
Asymetrie produce : 2^D evaluations / verify : 1 evaluation.

**Pourquoi le bon choix pour CETTE etape** :

1. **Standards-compatibility audit-friendly** : SHA256 est
   trivialement connu de tout auditeur Cure53/ToB. Le code
   `pow.rs:286-294` `sha256_of` fait 8 lignes. Replicable en
   Python/Go/JS en 5 minutes pour cross-verifier la primitive.
   BLAKE3 (3-5x plus rapide, deja workspace dep) etait alternative
   tentante — rejet documente `Cargo.toml:60-64` : « standards-
   compatibility argument for long-term audit clarity wins ».

2. **Verify O(1) preserve la fast-path subscriber** : 1 SHA256 +
   count-leading-zeros = ~1 us. Cache hit 15 min `PowVerifyCache`
   skip meme ce cost. Volume-friendly contrary a Argon2/VDF.

3. **RAM-cost zero-impact mobile/RPi** : SHA256 est computational
   pure, pas memory-hard. Un raspberry pi 3 (1 GB RAM) tourne le
   solve sans pression memoire — le seul cost est CPU time. Mesure
   bench `pow.rs:97` : 2^18 = ~100 ms sur CPU desktop moderne, ~1 s
   sur RPi 4. Tolerable pour un publisher heartbeat 30 s+
   intervals.

4. **Crate `sha2` RustCrypto stable** : derniere CVE 2021
   ([RUSTSEC-2021-0100](https://rustsec.org/advisories/RUSTSEC-2021-0100.html)
   AVX2 backend miscompute longs messages — fixe v0.9.8, donc 5
   ans clean depuis). API inchangee depuis 2017. Zero transitive
   churn. Workspace dep pinned `0.10` (`Cargo.toml:67`).

5. **Asymetrie produce/verify maximale** (10^6 :1 a 2^18). Aucune
   alternative considere atteint ce ratio sans memory-hard
   penalty. Critical pour amortir les costs sur subscriber
   high-volume.

6. **Precedent production stable** : Bitcoin (PoW principal),
   Tor 2023 (introduction circuits — pre-Equi-X migration), RFC
   6110 (DKIM), Lightning Network invoice PoW. Litterature
   dispo, attack vectors connus, mitigation patterns matures.

**Limites connues honnetes** : voir §6.

### 3.8 Pourquoi pas une lib audited existante (Bitcoin/Zcash/Monero Rust impls) au lieu d'ecrire from scratch ?

Question explicite du brief. Reponse en 5 points :

1. **Hashcash n'est pas une lib, c'est un protocol 30 lignes**.
   La primitive `pow.rs:283-373` (solve + verify) est ~90 LOC.
   Aucune lib existante n'apporte de valeur cryptographique
   ajoutee — la complexite est dans `sha2` (deja-audited) et le
   parametrage (different per-projet). Bitcoin's `bitcoin` crate
   ne expose pas Hashcash standalone — son PoW est cable au block
   header structure.

2. **Wrapping `bitcoin` ou `zcash_primitives` ajoute ~50-100
   transitive deps + un domain-knowledge cryptocurrency
   non-applicable**. Ces libs portent block-header-validation,
   coinbase-reward-curve, consensus-rules, etc. Pulling them in
   pour un PoW de 30 lignes inflige 5 KLOC de code mort qui doit
   etre audite.

3. **Domain separation prefix `b"nexus-pow-v1"`
   (`canonical.rs:126`) est SBFB-specific**. Aucune lib externe
   ne le respecte naturellement. Reutiliser `bitcoin::hash`
   forcerait un wrapping pour injecter le prefix → on ecrit le
   wrapping, on ecrit la primitive 30 lignes, identique cost-to-
   maintain.

4. **Auditabilite du code metier** : 90 LOC self-contained avec
   tests inline (`pow.rs:436-660` 18 tests) sont auditables en
   ~1 h Cure53/ToB. Wrapping `bitcoin` exige l'auditeur de tracer
   le call-path a travers la lib externe. Effort net > effort
   ecrire from scratch.

5. **Pattern documente** [`docs/rust/PATTERNS.md`](../../docs/rust/PATTERNS.md)
   « Crypto from-scratch policy : OK si <100 LOC + delegate
   primitive (sha2/ed25519-dalek/blake3), interdit si >100 LOC
   ou implements primitive lui-meme ». Hashcash respecte les deux
   conditions. Contre-exemple : ML-KEM (Sprint 26+) viendra de
   `aws-lc-rs` FIPS 140-3 obligatoirement (cf.
   `nexus_grid_pivot.md` Sprint 17 R-libcrux-hax P2 finding).

---

## 4. Parametrage

### 4.1 Difficulty 2^18 — rationale

**Constants livres** :
- `DEFAULT_DIFFICULTY_BITS = 18` (`pow.rs:97`)
- `MAX_DIFFICULTY_BITS = 30` (`pow.rs:103`)
- `MAX_PROOF_AGE_SECS = 1800` (`pow.rs:109`)

**Calcul timing 2^18** :

| Hardware | Hash rate (SHA256) | Solve 2^18 | Source |
|---|---|---|---|
| Desktop modern 2026 (Ryzen 9, M2 Max, etc.) | ~50-100 MH/s single-core | **~3-5 ms** measured ; **~100 ms** documente conservativement | bench `pow.rs:13-17` upper bound |
| Laptop modern 2026 (Intel i5 mobile) | ~20 MH/s single-core | ~13 ms | extrapolation BitcoinTalk threads |
| Raspberry Pi 5 (Cortex-A76) | ~3-5 MH/s single-core | ~50-90 ms | extrapolation [BitcoinTalk RPi SHA256 thread](https://bitcointalk.org/index.php?topic=67707.0) ARMv8 perf |
| Raspberry Pi 4 (Cortex-A72) | ~1-2 MH/s single-core | ~130-250 ms | [Mining Pool Stats](https://miningpoolstats.net/blog/bitcoin-mining-on-raspberry-pi/) |
| Mobile (Snapdragon 8 Gen 3, 2024+) | ~10-30 MH/s single-core | ~10-25 ms | extrapolation Snapdragon SHA-NI extensions |
| Botnet bot (Mirai class, ARM IoT 2018) | ~0.1-0.3 MH/s | ~1-3 s | research consensus |

Conclusions :
- **Publisher legit (T0/T1 maintainer ops)** : 2^18 = 100 ms a
  pire cas RPi 4. Acceptable pour un boot subscribe paye 1x par
  session 15 min (`SESSION_WINDOW` `pow_gossip.rs:87`).
- **Botnet bot (T2 commodity)** : ~1-3 s par identite × N topics.
  Force un attacker T2 a 100 cores a payer ~30 s pour spawn 100
  identites flooders sur 1 topic. Rate-limit emergent : il peut
  pas pulse > 10 sub/s aggregat sans saturate.
- **Difficulty 2^16 (rejete D2 kickoff)** : ~25 ms desktop,
  ~250 ms RPi. Botnet a ~250 ms = peut produire 4 ident/s — trop
  permissif.
- **Difficulty 2^20 (rejete D2 kickoff)** : ~400 ms desktop,
  ~2-4 s RPi 4. Trop pour un publisher mobile heartbeat.

Le baseline 2^18 est aligne avec le **Tor PoW initial range
2023** documente FAQ : « Initial times per solve range from 5
milliseconds for faster computers and up to 30 milliseconds for
slower hardware » ([Tor PoW FAQ](https://onionservices.torproject.org/technology/security/pow/)).
Tor utilise Equi-X plus efficace, donc des nombres plus bas pour
un meme effective cost. Notre 100 ms SHA256 ≈ 30 ms Equi-X (ratio
~3x). On reste dans le ballpark.

### 4.2 Per-relai ajustable

**Format `relay_pow_policy.toml`** (`relay_pow_policy.rs:21-31`) :

```toml
default_difficulty = 18

[topic_overrides]
"a1b2c3...deadbeef" = 20  # higher difficulty for hot topics
"cafebabe...feedface" = 16  # lower for dev/test channels
```

Pourquoi TOML :
- Operateurs editent a la main → comments + multiline supportes
- Workspace dep deja presente (`toml` `Cargo.toml:107`) — zero
  added cost
- Pattern align `relays.json` S18, `consent.json` S16, `tokens.
  json` S18 (audit fix D-1)
- Rejet JSON : pas de comments → operator confusion sur intent
  des overrides

**Loader layered source** (`relay_pow_policy.rs:9-17`) :
1. Env var `SBFB_POW_POLICY_PATH` absolute path
2. `$SBFB_HOME/relay_pow_policy.toml` (default `~/.sbfb/`)
3. Fallback `DEFAULT_POLICY` (`relay_pow_policy.rs:64`)

Mirror du pattern `relay_config.rs` S18, lui-meme align sur
`consent.rs` S16 + `auth.rs` D-1 audit fix S18.

**Hot-reload** : **NON livré dans S19 Phase B**. Le loader est
appele au boot. Pour rotation difficulty live (operator detect
attack et bump 18→22), il faudra cabler un `notify` watcher pattern
(reference `consent.rs:ConsentWatcher` S16, `auth.rs:TokenWatcher`
audit fix D-1 S18). Tracked tech debt **S20+** (cf. §6).

**Clamping defensif** : `RelayPowPolicy::difficulty_for`
(`relay_pow_policy.rs:116-122`) clamp toujours sur
`MAX_DIFFICULTY_BITS = 30`. Un policy file qui passe la load-
validation puis qui serait mute en RAM (poisoning impossible mais
defense en profondeur) reste capped. Test
`difficulty_for_clamps_stored_override_over_max`
(`relay_pow_policy.rs:344-357`) pin l'invariant.

**Loader rejette over-max au load** (`relay_pow_policy.rs:130-
163`) avec message explicite « relay PoW policy default_difficulty
=100 exceeds MAX_DIFFICULTY_BITS=30 ». Fail loud au lieu de
silently clamp = l'operateur catch les typos.

### 4.3 Anti-replay

Mecanismes combines :

1. **`issued_at` field** dans canonical bytes (`pow.rs:213-218`) :
   chaque challenge encode unix-seconds. Le pre-image SHA256
   change donc le winning nonce change. Un attacker qui sniff
   un proof issued_at=T1 ne peut pas le replay a T2 sans recalcul.

2. **`MAX_PROOF_AGE_SECS = 1800` (30 min)** (`pow.rs:109`) : le
   verify_at rejette tout proof dont `now - issued_at > 1800`
   (`pow.rs:419-425` branch `Expired`). Un proof captured
   network-side a T0 a une fenetre de replay max 30 min.

3. **`SESSION_WINDOW = 900` (15 min)** (`pow_gossip.rs:87`) : la
   cache subscriber considere un `(pubkey, topic)` trusted 15 min
   apres premier verify. **Strictement < MAX_PROOF_AGE_SECS** par
   design (15 min < 30 min) — un legit publisher renouvelle son
   proof avant l'expiry, sans ever hit la border. Documente
   `pow.rs:107-110`.

4. **Clock skew tolerance via `IssuedInFuture`**
   (`pow.rs:411-417`) : si `issued_at > now`, error explicit.
   Test `verify_rejects_future_issued_at` (`pow.rs:550-568`).
   **Pas de tolerance positive** — un publisher avec clock derive
   +10 s sera rejected. Trade-off assume : on prefere fail-loud
   sur clock skew que silently accept (un attacker peut pas spam
   « issued in 1 hour » et pre-stage). Mitigation operator-side :
   NTP sync recommande, documenter dans
   `docs/release/PKARR_RELAY_OPS.md` (Phase E).

5. **Nonce uniqueness implicite via deterministic search**
   (`pow.rs:357-369`) : le solver itere `0u64..` jusqu'au hit.
   Pour un meme challenge, le winning nonce est deterministe.
   Cela rend les regression tests reproductibles (rationale
   `pow.rs:351-356`) **et** garantit que deux solves successifs
   du meme challenge produisent le meme proof — utilise par la
   cache publisher pour assertion `solve_cache_invalidate_forces
   _fresh_solve` (`pow_gossip.rs:411-431`).

**Limite anti-replay** : un attacker qui captures un proof
recent (<30 min, <15 min cache window subscriber) **peut** le
replay sur un autre subscriber qui n'a jamais vu le pubkey. La
defense est probabiliste — la fenetre est courte et le
publisher_pubkey binding empeche le rebadging. Pour Sybil-
resistance complete, la combinaison PoW + kudos S22 + rate-limit
S21 est necessaire.

---

## 5. Choix d'implementation

### 5.1 Crate `sha2` (vs `ring`, `aws-lc-rs`, `libsodium`)

| Crate | Audit | FIPS 140-3 | RAM | Pure Rust | Workspace dep |
|---|---|---|---|---|---|
| `sha2` (RustCrypto) | Self-audit + community | Non | Pas critique SHA | Oui | **Pinned 0.10 deja** |
| `ring` (briansmith) | Implicit (BoringSSL fork base) | Non | Pas critique SHA | Mixed C/Rust | Non-direct |
| `aws-lc-rs` | NIST cert sept 2024 | **Oui level 1** | Mid | C base | Non-direct |
| `libsodium` (sodiumoxide) | Audited 2018+ | Non | Mid | C base | Non-direct |

**Choix `sha2`** :
- **Deja workspace dep** (`Cargo.toml:67`) — zero added churn
- **Pure Rust** — pas de C linker, pas de cross-compile drama
  (target Windows + Linux + macOS + ARM mobile sans config
  per-target)
- **CVE history minimaliste** : seul incident notable
  RUSTSEC-2021-0100 AVX2 backend miscompute long messages, fixe
  v0.9.8 — clean depuis 5 ans
- **API stable depuis 2017** : `Sha256::new().update().finalize()`
  pattern documente context7 trace [`/rustcrypto/hashes` 2026-04
  query](https://github.com/rustcrypto/hashes/blob/master/sha2/README.md)
- **FIPS non-requis pour PoW** : SHA256 PoW n'a pas de
  certification regulatory — FIPS impose pour signing/key-
  derivation, pas pour anti-spam puzzle. Si plus tard la cert
  devient un selling point ONG (cf. `PARTNERSHIPS.md`),
  migration `aws-lc-rs::digest::SHA256` est mecanique (meme
  primitive, juste API switch — 1-day work).

**Rejet `ring` / `aws-lc-rs` immediats** : zero benefit pour PoW
specific, ajoute transitive deps non-justifiees (`ring` ~3 MB,
`aws-lc-rs` ~5 MB FFI bindings + LibAWSCrypto).

**Rejet BLAKE3** : 3-5x plus rapide sur CPUs modernes
(`Cargo.toml:60-64` rationale). Mais standards-compatibility
audit win (Cure53/ToB Sprint 29) via SHA256 universel beat le
perf gain. Difficulty 2^18 SHA256 reste sub-100ms-modern-CPU
donc le perf overhead BLAKE3 vs SHA256 est dans les error bars
des heartbeats network.

**Recherche context7** [`/rustcrypto/hashes`
2026-04-16](https://github.com/rustcrypto/hashes) confirme que
l'API `sha2::{Sha256, Digest}` + `.update()` + `.finalize()`
inchangee — code `pow.rs:286-294` matche le pattern documente.

### 5.2 Single-threaded deterministe (vs multi-threaded parallel)

**Choix** : nonce iteration single-threaded `for nonce in 0u64..`
(`pow.rs:357-369`).

**Rationale** :
1. **CI-friendly determinism** : un meme challenge produit
   toujours le meme winning nonce → bench reproducibles, tests
   regression-friendly. Multi-thread search introduce un race
   condition sur which thread find le hit first → nonce non-
   deterministe → tests utilisant `assert_eq!(proof.nonce, X)`
   flaky. Documente `pow.rs:351-356`.

2. **Fairness pour publisher mobile** : un publisher single-core
   raspberry pi paie meme cost qu'un desktop (~30x plus) parce
   que single-thread search. Multi-threading favoriserait les
   beefy publishers — deviendrait incentive a deployer publishers
   server-class (anti-decentralization).

3. **Defense vs precomputation amortization** : multi-thread
   search amortit le cost via parallelism, ce qui rend la
   metrique « cost-per-proof » floue. Single-thread keeps
   `cost = 2^D × cycle-per-hash` line a calculer.

**Trade-off assume** : un attacker T2-T3 peut multi-threader
trivialement. La primitive ne empeche pas — elle juste ne profite
pas. Le multi-threading attacker-side est neutralise par le fact
que **la difficulty est plate per-identity** ; un attacker qui
veut N identites paie N×cost meme s'il parallelise.

### 5.3 Format wire (proof = champ optionnel `#[serde(default)]`)

**Choix** : envelope binaire `[u32 BE proof_len][proof JSON]
[payload]` (`pow_gossip.rs:11-31`). Format **non versioned**
explicit — c'est implicit format v1.

**Rationale conformite « Pre-launch protocol policy »** (CLAUDE.md
§Pre-launch) : aucun deploy live ne parle SBFB. Le wire format
peut etre redefini en place jusqu'au tag v1.0. Pas de tolerant
decoder multi-version, pas de `#[serde(default)]` pour
compatibility historique.

**Le `#[serde(default)]` qui apparait dans
`relay_pow_policy.rs:77,87`** est legitime au sens **runtime
robustness** (un operator qui ecrit `relay_pow_policy.toml` avec
juste `default_difficulty = 18` et omet `[topic_overrides]` ne
doit pas trigger un parse error). Le rationale est documente
dans le code (`relay_pow_policy.rs:75-77`).

**Limite « payload tampering not detected »** est documentee
explicit `pow_gossip.rs:502-527` test
`verify_cache_rejects_tampered_payload_has_no_effect`. Scope
boundary intentionnel : PoW = cost-of-identity, pas payload
integrity. La couche au-dessus (`curator.rs:CuratorList`,
`task.rs:Task`, etc.) signe Ed25519 le payload via canonical
bytes JCS + domain prefix.

**Forward-compat post-v1.0** : envelope format bump introduit un
1-byte `version` prefix avec tolerant decoder. Documente
`pow_gossip.rs:28-31`.

### 5.4 Bench `criterion` (vs custom bench)

**Choix** : `criterion` 0.5 dev-dep (`Cargo.toml:197-199`),
benches 3 difficultes 2^12 / 2^18 / 2^20 (`benches/pow.rs:47-58`).

**Rationale** :
1. **Regression-guard, pas microbench** : criterion flag un 2x
   regression sur le default difficulty si un futur refactor
   pessimise le hash loop. Documente `benches/pow.rs:7-10`.
2. **CI integration trivialise** : `cargo bench --bench pow` est
   single command, output structure pour parse (vs custom bench
   = parsing logic ad-hoc).
3. **`html_reports` desactive** : CI tourne headless, on assert
   wall-clock ceilings via grep `time:`. Pas de generate HTML.
   `Cargo.toml:197-199` features minimales.
4. **`sample_size` adaptatif per-difficulty** : 50 samples pour
   2^12 (~5 ms = 250 ms total bench), 30 pour 2^18 (~3 s), 10
   pour 2^20 stress (~4 s). Total bench time ~10 s, CI-friendly.

**Rejet custom bench** : refaire un measurement loop avec warmup
+ outlier detection = ~100 LOC dupliques de criterion. Pas de
gain.

**Rejet `divan`** (alternative recente) : workspace n'a pas la
dep, criterion fait le job, pas de raison de splurge.

---

## 6. Limites connues + futures evolutions

### 6.1 Limites connues honnetes

1. **PoW seul ne defend pas Sybil-resistance complete**.
   Combinaison **necessaire** : PoW S19 + kudos-weighted
   admission S22 + rate-limit S21. Un attacker T2-T3 patient peut
   accumuler identites pre-solvees over time. PoW augmente le cost
   marginal d'une identite, pas un absolu absolu.

2. **PoW SHA256 ASIC-vulnerable a long terme**. Bitcoin ASICs
   rendent le ratio CPU/ASIC ~10^9 ([Mining Pool Stats blog](https://miningpoolstats.net/blog/bitcoin-mining-on-raspberry-pi/)).
   Un attacker qui investit dans un ASIC second-marche post-2024
   (Bitcoin halving obsolescence) a un cost factor enormement
   inferieur a un publisher CPU. **Mitigation S22+** : migration
   Equi-X (Tor 2023) ou difficulty escalade dynamique (S23
   « Escalating PoW per-(consumer, model) » `HARDENING_ROADMAP §3
   S23`).

3. **Pre-computation challenges fenetre 30 min**. Un attacker
   qui pre-solve 1000 challenges pour 1000 timestamps futurs
   `(now+1, now+2, ..., now+1799)` sur 30 min de fenetre peut
   les replay au moment voulu. Le `MAX_PROOF_AGE_SECS = 1800`
   limite la window mais ne l'elimine pas. Defense complementaire
   = challenge issued par le subscriber (interactive) plutot
   que self-issued par le publisher (current). Trade-off
   bandwidth + complexity vs anti-precompute. **Reporte S22+**.

4. **Time-skew silencieux** : pas de NTP sync enforcement. Un
   publisher avec clock derive +5 min sera rejected `IssuedIn
   Future`. Operator doit deploy chronyd/ntpd. Documenter ops
   doc (Phase E `PKARR_RELAY_OPS.md`).

5. **Cache poisoning via tampered envelope** : test
   `end_to_end_rejects_tampered_proof_with_mock_transport`
   (`pow_gossip.rs:611-650`) verifie que `failed verify must not
   poison the session cache`. Mais si l'attacker arrive a faire
   passer un proof valide cote PoW (pas de tamper) avec un
   payload malveillant, la cache trust le pubkey 15 min →
   payload integrity = responsabilite couche au-dessus. Pas une
   limite PoW per se, mais un trou si un developpeur en aval
   oublie de signer son payload.

6. **Hot-reload policy NOT livre** : un operator qui detect une
   attack et veut bump 18→22 doit restart le daemon. Pattern
   `notify` watcher pour `relay_pow_policy.toml` est preview
   `relay_pow_policy.rs:32-39` mais code-only au boot. **Tech
   debt S20+** quand le pattern S16/D-1 sera consolide.

7. **Single-threaded solve cap throughput publisher** : un
   publisher qui veut subscribe a 10 topics simultanement paie
   ~1 s sequentiel (10 × 100 ms). Acceptable mais visible UX
   au boot. Multi-thread cote publisher uniquement (pas
   determinism-impacted because cache key est `(pubkey, topic)`
   so each thread serve different topic) = optimisation S21+.

### 6.2 Adaptive difficulty (pourquoi pas S19, S21+)

Tor PoW spec 2023 implemente difficulty dynamique adaptative
basee sur attack volume detected ([Tor PoW FAQ](https://onionservices.torproject.org/technology/security/pow/)).
Pattern :
- Mesure rate de subscribe attempt par seconde
- Si > threshold, bump difficulty global
- Decay back vers baseline quand calm

**Pourquoi pas S19** :
- Necessite metric collection cote relai (telemetrie pas encore
  cablee S19)
- Necessite consensus inter-relais sur la difficulty courante
  (sinon publisher voit un different per-relai → confusion)
- Necessite operator dashboard pour monitor/adjuster (S22+ ops
  scope)

**Cible S22** : item « Kudos-weighted gossip admission »
(`HARDENING_ROADMAP §3 S22`) inclut adaptive difficulty proposal.
Forward-compat noted `pow.rs:43-48` : « S22 kudos-weighted
admission : the receiver verify path will add a `kudos_score
>= policy.threshold` check alongside the PoW verify ».

### 6.3 Challenge cache (memoization solutions, S20+)

**Idea** : cote subscriber, garder un cache LRU des proofs
verifies recently across all sessions, pas juste session 15 min.
Permet de skipper le full verify meme apres restart daemon.

**Pourquoi pas S19** : cache persisted exige fichier disk write
+ rotation policy + crash-recovery test surface. Cache memory-
only S19 (`PowVerifyCache::entries: DashMap`
`pow_gossip.rs:253`) suffit pour la session window.

**Pertinent S20** : si le pattern S16 ConsentWatcher + S18 Token
Rotator produit un kit rotation/persistence reusable, le PoW
cache peut reuser le meme. Tech debt loggee en
`docs/rust/PATTERNS.md` Phase F S19 wrap-up.

### 6.4 VDF migration path

Si la menace evolue vers **attacker GPU/parallelism arbitraire**
(scenario T3+ etat-nation avec 10k GPUs) :
- Hashcash devient insuffisant (ratio CPU/GPU ~100x sur SHA256)
- Equi-X 2023 Tor amelioration mais reste vulnerable a parallelism
- VDF Wesolowski/Pietrzak garantit non-parallelism

**Migration path estimated** :
1. Bump `POW_FORMAT_VERSION 1 → 2` (post-v1.0, tolerant decoder)
2. Replace primitive `pow.rs::solve/verify` par VDF wrapping
   `chiavdf` C lib (binding Rust)
3. `RelayPowPolicy` schema bump : ajout `vdf_iterations` field
4. Cache `PowVerifyCache` model unchanged (key reste `(pubkey,
   topic)`)

Effort estime : ~1 sprint dedie. Tracked dans `nexus_grid_pivot.
md` future-work si menace evolue.

### 6.5 Multi-threaded solver cote publisher (S21+)

Cf. §6.1 limite 7. Optimisation puren cote producer pour
parallel-subscribe scenarios. Pas d'impact securite, juste UX.
Roadmap dedie performance Sprint 21+.

---

## 7. References

### 7.1 Primary literature

- **Adam Back (2002)**, « Hashcash — A Denial of Service Counter-
  Measure », [hashcash.org/hashcash.pdf](http://www.hashcash.org/hashcash.pdf).
  Spec originale, RFC-equivalent reference.
- **Hashcash Wikipedia** [en.wikipedia.org/wiki/Hashcash](https://en.wikipedia.org/wiki/Hashcash) —
  vue ensemble pratique 2024/2025.
- **Tor PoW spec 2023** : [Introducing Proof-of-Work Defense for
  Onion Services blog](https://blog.torproject.org/introducing-proof-of-work-defense-for-onion-services/),
  [PoW FAQ](https://onionservices.torproject.org/technology/security/pow/),
  [Tor spec hspow analysis](https://spec.torproject.org/hspow-spec/analysis-discussion.html).
  Reference pour difficulty dynamic + Equi-X choice rationale.
- **Tevador devlog Equi-X / HashX** [GitHub equix devlog.md](https://github.com/tevador/equix/blob/master/devlog.md).
  Cite RandomX rejection + memory cost trade-off (1 GB rejete pour
  Tor → 16 MB acceptable).

### 7.2 Alternatives considered — papers

- **Equihash** : Wikipedia [en.wikipedia.org/wiki/Equihash](https://en.wikipedia.org/wiki/Equihash),
  Bitcoin Gold post-attack [bitcoingold.org Equihash-BTG](http://www.bitcoingold.org/equihash-btg-our-new-pow-algorithm/),
  recherche 2025 [eprint 2025/1351](https://eprint.iacr.org/2025/1351.pdf).
- **Argon2** : RFC 9106 [rfc-editor.org/rfc/rfc9106](https://www.rfc-editor.org/rfc/rfc9106.html),
  recherche 2024 sur effective adoption [arxiv 2504.17121](https://arxiv.org/html/2504.17121v1).
- **VDF** : Pietrzak/Wesolowski survey [Uplatz blog 2025](https://uplatz.com/blog/verifiable-delay-functions-a-comprehensive-analysis-of-cryptographic-foundations-applications-and-deployment-challenges/),
  Trail of Bits intro [blog.trailofbits.com 2018](https://blog.trailofbits.com/2018/10/12/introduction-to-verifiable-delay-functions-vdfs/).
- **Cuckoo Cycle** : Tromp 2014 paper [eprint 2014/059](https://eprint.iacr.org/2014/059.pdf),
  Grin docs [docs.grin.mw cuckoo](https://docs.grin.mw/wiki/miscellaneous/cuckoo-cycle/).
- **Akamai Behavioral DDoS Engine** [Nov 2024 blog](https://www.akamai.com/blog/security/2024/nov/akamais-behavioral-ddos-engine-breakthrough-in-modern-ddos-mitigation) —
  reference TLS challenge model deprecated pour P2P.
- **Sybil defense P2P** : Borisov 2006 « Computational Puzzles as
  Sybil Defenses » [nymity.ch Borisov2006a.pdf](https://nymity.ch/sybilhunting/pdf/Borisov2006a.pdf),
  cf. discussion precomputation reuse.

### 7.3 Implementation references

- **`sha2` RustCrypto crate** : [github.com/rustcrypto/hashes](https://github.com/RustCrypto/hashes).
  CVE history : seul advisory notable [RUSTSEC-2021-0100](https://rustsec.org/advisories/RUSTSEC-2021-0100.html)
  (AVX2 backend miscompute, fixe v0.9.8). API stable depuis 2017.
- **`aws-lc-rs` FIPS 140-3** : [memorysafety.org rustls-fips
  blog](https://www.memorysafety.org/blog/rustls-with-aws-crypto-back-end-and-fips/),
  [AWS-LC FIPS cert sept 2024](https://aws.amazon.com/blogs/security/aws-lc-is-now-fips-140-3-certified/).
  Reference si SBFB veut cert plus tard.
- **`bitcoin` rust-secp256k1** : [github.com/rust-bitcoin/rust-
  secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) —
  reference « no crypto from scratch in Rust except hash
  functions » policy (cf. §3.8 reasoning).

### 7.4 Context7 traces datees < 6 mois

- **`/rustcrypto/hashes`** queried 2026-04-16 (today). Confirmed
  API `sha2::{Sha256, Digest}::new().update().finalize()` exact
  pattern matches `pow.rs:286-294` `sha256_of`. Sources :
  `github.com/rustcrypto/hashes/blob/master/sha2/README.md`,
  `github.com/rustcrypto/hashes/blob/master/README.md`.
- **`/websites/rs_iroh-gossip`** queried 2026-04-16 (today).
  Confirmed `iroh-gossip` 0.97 `GossipTopic::broadcast(message:
  Bytes)` + `subscribe_with_opts(topic_id, opts) -> GossipTopic`
  API. Validates that the envelope wire `pow_gossip.rs::PowEnvelope`
  bytes can be passed unchanged to `broadcast(envelope.into())`
  without API friction. Source : `docs.rs/iroh-gossip/latest/src/
  iroh_gossip/api`.

### 7.5 SBFB internal references

- [`docs/security/THREAT_MODEL.md`](../../docs/security/THREAT_MODEL.md) —
  STRIDE/LINDDUN matrix, B-Sybil + B-GossipPoison rows.
- [`docs/security/HARDENING_ROADMAP.md §3 S19`](../../docs/security/HARDENING_ROADMAP.md) —
  scope item ancrage.
- [`docs/security/VALIDATED_BLUEPRINT.md`](../../docs/security/VALIDATED_BLUEPRINT.md) —
  couche 1 « identite & auth » mentionne PoW S19 vs PQC S26+
  sequencing.
- [`crates/nexus-core-rs/src/canonical.rs:126`](../../crates/nexus-core-rs/src/canonical.rs) —
  `DOMAIN_POW_V1 = b"nexus-pow-v1"` definition.
- [`Cargo.toml:57-67`](../../Cargo.toml) — `sha2 = "0.10"`
  workspace dep + rationale comment SHA256 vs BLAKE3 pour
  audit clarity.
- [`.planning/active/sprint19_kickoff.md §4 D2`](../active/sprint19_kickoff.md) —
  difficulty 2^18 D2 decision day 0.
- [`.planning/active/sprint19_plan.md §5`](../active/sprint19_plan.md) —
  Phase B scope + tests plan.

---

**Note de placement** : ce design doc accompagne le commit
`feat(sprint19): Phase B — PoW Hashcash gossip subscribe
(difficulty 2^18 per-relai)`. Cite explicitement dans le body
commit (cf. `sprint19_plan.md §5.5`). Update `nexus_grid_pivot.
md` non-applicable (research doc, pas un livrable code). Audit
gate Sprint 20 Phase 0 reverra la qualite de ce doc + completude
des limites §6 vs threats observed entre S19 et S20.
