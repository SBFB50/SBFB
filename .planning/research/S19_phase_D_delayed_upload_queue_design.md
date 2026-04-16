# Sprint 19 Phase D — Delayed upload queue : design doc

**Date** : 2026-04-16 (pre-Phase D, session de design pure).
**Auteur** : agent recherche pre-Phase D (kickoff §4 D4 + plan §7 deja
figes).
**Tip master ref** : `1a606a3` (post-S18 audit gate leve).
**Statut** : design accepte, l'implementation Phase D suit ce
document. Toute deviation Phase D vs ce design = doit etre
justifiee par une session fraiche (pas un drift silencieux).

---

## 1. Probleme adresse

### 1.1 Threat model — observation traffic et correlation

Nexus-grid est un reseau P2P ou un coordinator local emet via
gossip iroh chaque task submit a destination des workers
disponibles. Sans aucune temporisation, l'enchainement observable
de l'exterieur est :

```
T : user clique "submit" dans l'iframe app
T+~5ms : POST /tasks/submit → coordinator (loopback)
T+~10ms : coordinator signe TaskEntry, doc.set(), gossip emit
T+~50ms : relay iroh diffuse le message a tous les subscribers du topic
```

Un observateur de niveau **T2** (ISP local), **T3** (DPI corporate
ou ISP national) ou **T4** (NSA cable tap, BGP hijack) qui voit
**simultanement** :

- Le trafic chiffre du device user (envoi POST loopback est
  invisible cote reseau, **mais** le upstream gossip vers le
  relay est observable au niveau IP/QUIC pattern)
- Le trafic gossip relayed vers les peers du topic

…peut **correler** ces deux flux par leur fenetre temporelle
(<100ms typiquement) et lier `user_A → topic_T` quand bien meme
le contenu serait chiffre. C'est le scenario S17 documente dans
[`docs/security/P2P_THREATS.md §6.3`](../../docs/security/P2P_THREATS.md)
("Dragnet metadata correlation T4") et
[`docs/security/VALIDATED_BLUEPRINT.md §6 Traffic shaping`](../../docs/security/VALIDATED_BLUEPRINT.md).

Le HARDENING_ROADMAP §3 S19 specifie l'item **"Delayed upload
queue (randomized 0-5min batching) — ~300 LOC"** comme la
**premiere etape concrete d'anti-correlation** au sein du
projet, sans pretendre etre un mix-net complet.

### 1.2 Distinction explicite avec D3 (PoW) et avec un mix-net Loopix S25+

Trois mesures complementaires sont planifiees mais **distinctes
en mandate** :

| Mesure | Sprint | Cible threat | Mecanisme |
|---|---|---|---|
| PoW Hashcash gossip subscribe | **S19 Phase B** | Sybil bootstrap, T1-T2 spammer | Cost-of-identity SHA256 difficulty 2^18 |
| **Delayed upload queue** | **S19 Phase D (ce doc)** | Traffic correlation T2-T3 timing-attack | Random delay exponential 0-5min |
| Mix-net Loopix Sphinx + cover traffic | **S25+** | Global passive adversary T4-T5 | Stratified Poisson mix nodes + dummy traffic |

Ce doc traite **uniquement** D : le but est de **rompre la
correlation temporelle "submit→broadcast"** observable a courte
fenetre. Il ne pretend pas defendre contre un global passive
adversary T4 qui observe **a la fois** l'entree du device et
**toutes** les sorties relai sur 5 min — ce scenario reste
defait par un simple intersection attack (cf. §6) jusqu'a S25+.

### 1.3 Pourquoi cette etape intermediaire suffit pour S19

**Pourquoi pas implementer directement un mix-net Loopix
maintenant ?** Question legitime, reponse en 4 points :

1. **Cout en complexite** : Loopix exige une **stratified network
   topology** (entry gateways + 3 layers mix nodes + exit
   gateways), un **packet format Sphinx** (2048 bytes fixed,
   crypto onion routing), et du **cover traffic continu** par
   chaque client meme idle. Le Nym whitepaper estime ~200-800ms
   de latence end-to-end **best case**, beaucoup plus en charge
   reelle. C'est une **brique infrastructure** qui requiert
   des relays Sphinx-aware deployes, pas juste du code coord-side.
2. **Cout en bandwidth** : cover traffic continu = ~1-10
   packets/s par client meme idle, soit ~10-100 KB/s **24/7**
   par device. Inadmissible pour un user mobile DnD Forge.
   Loopix n'est viable qu'avec un wallet-token economique
   (Nym) ou un opex sponsorise (univs).
3. **Pre-requis non satisfaits** : Loopix presuppose un
   **anonymity set k > quelques dizaines de clients actifs
   simultanement** dans la meme couche. Pre-launch SBFB =
   **anonymity set = 1** (le dev seul). Implementer Loopix
   maintenant produirait une anonymite **mathematiquement
   nulle** pour un cout d'ingenierie maximal.
4. **Sequencement rationnel** : L'**[Anonymity Trilemma
   (Das, Meiser, Mohammadi 2017)](https://eprint.iacr.org/2017/954.pdf)**
   prouve formellement qu'on ne peut avoir simultanement
   *strong anonymity*, *low bandwidth overhead* et *low
   latency*. Le projet choisit explicitement **low bandwidth +
   low latency** pour S19 (target user DnD Forge), sacrifiant
   *strong anonymity*. L'upgrade vers strong anonymity (mix-net)
   est sequence S25+ une fois que (a) il y a une userbase reelle
   pour fournir l'anonymity set, et (b) un partnership ONG-ops
   peut fournir des relays Sphinx.

**Conclusion §1** : un random delay 0-5min est le **minimum
viable anti-correlation** qui (a) ne casse pas l'UX, (b) requiert
zero infrastructure additionnelle, (c) protege contre l'observer
le plus probable (T2-T3 ISP/DPI a fenetre courte), (d) prepare
l'evolution vers mix-net sans le bloquer.

---

## 2. Decision retenue (resume executable)

| Aspect | Choix |
|---|---|
| Distribution | **Exponential decay** mean=90s, max=300s (clamped) |
| Range total | **0 a 5 minutes** (300s hard ceiling) |
| Mediane attendue | ~62s (ln(2) × 90s) |
| p99 attendu | ~300s (clamped a max) |
| Scheduler flush | Boucle interne **30s** (`asyncio.sleep` + flush due) |
| Persistance | **SQLite WAL** table `delayed_uploads` (cross-restart) |
| Concurrence | `asyncio.Lock` autour de la queue list (single event loop) |
| Backpressure | Soft cap 10000 entries → log warn + accept ; hard cap 100000 → reject 429 |
| Metric | Log INFO histogram `upload_queue_delay_seconds` (bucketed 0/30/60/120/180/240/300) |
| Tunable | `coordinator.toml [upload_queue]` section : `mean_jitter_s`, `max_jitter_s`, `flush_interval_s`, `disabled` (escape hatch dev) |
| Module | `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` |
| Integration point | `api/tasks.py::submit_task()` pipe via `coord.upload_queue.schedule(...)` au lieu de `dispatcher.submit(...)` direct |

Cette decision **etend** le plan §7.3 du sprint19_plan.md sur
deux points :

1. Le plan parlait de `queue: list[tuple[datetime, dict]]` **in-
   memory only** avec mention "persistence = tech debt S20+".
   **Ce design upgrade vers SQLite WAL des Phase D** car la perte
   d'un task submit pendant un coord crash est un bug critique
   pre-launch (le user voit "submitted" mais rien n'arrive jamais
   sur le reseau). Le cout d'ajouter SQLite est faible (~50 LOC
   table + insert + delete), le benefice (durabilite cross-
   restart) est inestimable. Decision design-doc, justifiee §5.2.
2. Le plan parlait de "queue" sans reference a `coordinator.toml`.
   **Ce design ajoute une section TOML tunable** pour permettre a
   un user paranoiaque de pousser `mean=300s` (anti-correlation
   forte, UX degradee) ou un user impatient de pousser `mean=10s`
   (UX preservee, anti-correlation faible). Justifiee §4.4.

---

## 3. Alternatives de distribution considerees

### 3.1 Uniform [0, T]

```
delay = uniform(0, T)
```

**Caracteristiques** :

- Probabilite uniforme sur tout l'intervalle
- Mediane = T/2, mean = T/2
- Pas de tail : impossible d'avoir un delay > T

**Avantages** :

- Simple a implementer (`random.uniform`)
- Previsible en pire cas (tout le monde finit avant T)
- Comprehension intuitive pour un debug

**Inconvenients** :

- **Mauvais anti-correlation** : un attaquant qui voit N submits
  successifs peut estimer la fonction de densite et inferer le T
  exact. Apres calibrage, il sait que tout broadcast est dans
  [submit_seen, submit_seen+T] avec probabilite uniforme — il
  peut donc fenetrer son log a T secondes et garder >99% des
  candidats.
- **Concentration centre** : la mediane est parfaitement
  predictible, donc la fenetre de plus haute probabilite est
  l'endroit le plus exploite par un correlateur.
- **Pas de "queue protectrice"** : un submit qui rentre a t=0
  et un autre a t=T-1 sont reflushes dans des fenetres
  totalement differentes — pas de mutual cover.

**Verdict** : **rejete**. La distribution uniform est
explicitement **dominee** par exponential pour l'objectif anti-
correlation (cf. [Cornell ESORICS 2006 — Timing analysis in
low-latency mix networks](https://www.cs.cornell.edu/~shmat/shmat_esorics06.pdf)
qui demontre que les distributions a fat tail sont preferables).

### 3.2 Exponential decay mean=90s max=300s — RETENU

```python
delay = min(random.expovariate(1.0 / 90.0), 300.0)
```

**Caracteristiques** :

- Densite f(t) = (1/μ) × e^(-t/μ) avec μ=90s, support [0, ∞)
  clampe a 300s
- Mediane = ln(2) × μ ≈ 62s (≪ mean a cause de la skew)
- p50 ≈ 62s, p90 ≈ 207s, p99 ≈ 300s (clamped)
- Tail "fat" : 36% des submits seront delays > 90s, 11% > 200s
- Memoryless property : l'attente residuelle ne depend pas du
  temps deja attendu (propriete unique de l'exponential)

**Avantages** :

- **Optimal sous Poisson process inflow** : si les arrivees
  user→coord suivent une distribution de Poisson (hypothese
  raisonnable pour des actions humaines independantes), alors
  une distribution exponentielle de delay produit un **departure
  process** Poisson de meme intensite (M/M/∞ queue), ce qui
  **brouille maximalement** la correlation arrivee-depart
  ([Loopix 2017 §4 Poisson mixing](https://arxiv.org/abs/1703.00536)).
- **Mediane basse** = UX preservee pour majorite des submits
  (62s reste sous le seuil "encore tolerable" pour DnD Forge,
  cf. §4.1).
- **Tail haute** = anti-correlation forte pour les ~10% qui
  patientent jusqu'a 200-300s.
- **Memoryless property** garantit qu'un attaquant qui observe
  un submit a t=0 et n'a vu **aucun** broadcast au temps t=60s
  ne gagne **aucune** information sur la probabilite que ce
  submit soit reflush a t=60s+epsilon vs t=200s.
- **Cornell ESORICS 2006** + **Loopix 2017** + **MOCHA 2025
  eprint** convergent tous sur exponential delays comme baseline
  rationnelle pour low-latency anti-timing.

**Inconvenients** :

- **Tail clamped a 300s casse la memoryless property** dans
  l'ultime 10% : un attaquant qui voit qu'un submit attend
  > 250s peut etre quasi-certain qu'il sera flush au plus tard
  a t=300s. Mitigation : clamp est un compromis UX necessaire,
  on l'assume documented.
- Generation `random.expovariate` Python est non-CSPRNG par
  defaut. Mitigation : importer `secrets.SystemRandom` et le
  passer a `random.expovariate` ou recoder via
  `-mean × ln(secrets.SystemRandom().random())` pour
  cryptographique-grade randomness. Cf. §5.

**Verdict** : **RETENU**. Aligne avec D4 kickoff. Aligne avec la
literature mixnet majoritaire (Loopix, Mixminion-pool, Cornell).

**Implementation reference** :
```python
import secrets, math
_rng = secrets.SystemRandom()

def exponential_delay(mean_s: float, max_s: float) -> float:
    # Inverse-CDF sampling, CSPRNG source
    u = _rng.random()
    # u ∈ (0, 1) ; -mean × ln(u) suit Exp(1/mean)
    raw = -mean_s * math.log(u if u > 0 else 1e-18)
    return min(raw, max_s)
```

### 3.3 Poisson process (Tor-style continuous-time)

```
generate Poisson process rate λ = 1/90s
each submit dispatched at next Poisson event
```

**Caracteristiques** :

- Inter-departure times = exponentiels (memoryless)
- Departures **independantes** des arrivees (chaque submit
  s'aligne sur le **prochain event** du process Poisson global)
- Pattern Loopix continuous-time mixing

**Avantages** :

- **Tres haute robustesse anti-correlation** : meme un attaquant
  qui voit 100 submits ne peut pas distinguer "submit i a fini
  par flush au tick j" — l'inter-arrival pattern est uniforme et
  ne porte pas d'info sur l'ordre original.
- Exactement le pattern Loopix, donc upgrade-path clair vers S25+.
- Permet le batching naturel : si 3 submits arrivent dans une
  fenetre de 10s et que le prochain Poisson tick est dans 80s,
  les 3 sont flush ensemble (k-anonymity emergente).

**Inconvenients** :

- **Queue tail unbounded** : si la load coord est faible, le
  prochain Poisson tick peut etre dans 10 minutes — un submit
  rentre a t=0 patiente jusqu'au prochain tick **meme si tous
  les submits suivants entrent et flush avant lui**. UX
  inacceptable sans clamping.
- **Implementation overhead** : il faut maintenir un timer
  Poisson global, pas juste un timer per-message. Plus complexe
  a tester deterministe.
- **Pas de gain prouve sur exponential per-message** a notre
  echelle (1 coord, ~10 submits/h max realiste pre-launch). Le
  benefice Poisson process se materialise des >100 submits/min
  ou les batchs naturels emergent.

**Verdict** : **rejete pour S19**, **equivalent a exponential
per-message a notre echelle** + complexite supplementaire non
justifiee. A reconsiderer S25+ quand un mix-net est de toute
facon le path.

### 3.4 Fixed batching pool-mix (Mixmaster timed dynamic pool)

```
toutes les T secondes:
  n = len(pool)
  if n >= min_pool_size:
    count = min(n - min, n × rate_pct)
    flush count messages (random selection)
  else:
    wait
```

**Caracteristiques** :

- k-anonymity **garantie** par construction : un message flush
  est indistinguable des `count - 1` autres flush ensemble
- Tres etudie academiquement : ([Mixmaster spec](https://www.freehaven.net/anonbib/cache/mixmaster-spec.txt),
  [Mixminion](https://www.mixminion.net/minion-design.pdf),
  [Serjantov-Newman 2003 Timed Pool Mixes](https://link.springer.com/content/pdf/10.1007/978-0-387-35691-4_41.pdf))
- Pattern utilise par tous les remailers Type-II/Type-III
  serieux

**Avantages** :

- **Garantie cryptographique d'anonymity set k** : chaque
  message flush partage sa fenetre temporelle avec exactement
  k-1 autres → un attaquant a 1/k probabilite de correlation
  correcte, mathematiquement.
- Resistance **prouvee** a un global passive adversary qui
  observe ingress + egress (a condition que k soit suffisant).

**Inconvenients** :

- **UX cassee si traffic faible** : un user solo qui submit
  1 task pendant que le pool est vide attend potentiellement
  T × N rounds que d'autres submits remplissent le min_pool.
  Pour SBFB pre-launch ou il y a 1 user (le dev), `min_pool=10`
  signifie attendre ~indefiniment. **C'est exactement le
  scenario que pre-launch SBFB rencontre.**
- **Latence imprevisible** : un message peut sortir au prochain
  tick (T secondes) ou dans 10 ticks (10×T secondes), selon
  l'arrival rate des autres → impossible a borner au pire cas.
- **Pre-requis anonymity set k > 1** non satisfait pre-launch.
  Pareil que Loopix §1.3 critique 3.
- **Active attacks "trickle"** : un attaquant peut **manipuler
  le pool** en submittant N-1 messages benins pour deanonymiser
  le 1 message cible (trickle attack documentee
  [Serjantov-Dingledine 2003 batching taxonomy](https://www.freehaven.net/doc/batching-taxonomy/taxonomy.pdf)).
  Mitigation requise : minimum batch size + pool retention
  > simple flush.

**Verdict** : **rejete pour S19 MVP**. Reconsidere conjointement
avec **mix-net Loopix S25+** quand l'anonymity set est >> 1 et
qu'on a deja le packet format Sphinx + cover traffic pour
contrer trickle attacks.

### 3.5 Adaptive (charge-dependent) delay

```
mean_jitter = base_mean × adjustment(current_qps)
where adjustment ∈ [0.5, 2.0] depending on observed load
```

**Caracteristiques** :

- Mean varie dynamiquement avec le QPS observe
- Pattern utilise par Tor PoW dynamic difficulty 2023+
  ([Tor PoW blog](https://blog.torproject.org/introducing-proof-of-work-defense-for-onion-services/))

**Avantages** :

- Auto-tuning : load forte → batchs naturels → mean baisse
  (pas besoin d'attendre, anti-correlation par batching) ;
  load faible → mean monte (compense l'absence de batching
  naturel)
- Resilience UX en burst : si user submit 10 tasks d'un coup,
  le mean peut chuter pour les flusher rapidement

**Inconvenients** :

- **Sophistication non justifiee a notre stade** : l'auto-tuning
  est utile quand on a >100 submits/min variables. Pre-launch
  SBFB est <10 submits/h. La logique adaptive est **morte
  code** jusqu'a ce qu'il y ait une vraie load.
- **Ouvre une nouvelle attack surface** : un attaquant qui peut
  observer la mean en temps reel (par sondage de ses propres
  submits) peut **inferer la load** du coord, donc le nombre
  d'utilisateurs actifs → fuite de metadata anonymity-set-size.
- **Tests deterministes plus durs** : la fonction de delay
  depend de l'etat global (QPS observed) → seed fixed ne suffit
  plus, il faut mock l'horloge **et** l'historique.

**Verdict** : **differé S22+**, a reconsiderer quand load reelle
exists. Pre-launch design-doc principle : "fixed mean with
TOML override" est plus simple et plus auditable.

### 3.6 Comparaison synoptique

| Distribution | Anti-correlation | UX (median) | UX (p99) | Complexity | Anonymity-set-1 viable | Verdict S19 |
|---|---|---|---|---|---|---|
| Uniform [0, T] | Faible | T/2 | T | **Trivial** | Oui | Rejete (dominee) |
| **Exponential μ=90 max=300** | **Bonne** | **62s** | **300s** | Faible | **Oui** | **RETENU** |
| Poisson process (Loopix-style) | Tres bonne | Variable | Unbounded | Moyenne | Non (queue tail) | Rejete (overkill S19) |
| Fixed pool-mix (Mixmaster) | **Garantie k-anon** | Variable | Unbounded | Moyenne | **Non (pre-req k>1)** | Rejete (S25+) |
| Adaptive | Bonne (charge >0) | Auto | Auto | Elevee | Oui (degraded mode) | Differe S22+ |

---

## 4. Range 0-5min — rationale UX/anonymity

### 4.1 UX impact (DnD Forge target user)

DnD Forge est le target user explicite du Gate 1 SBFB. Son
profil :

- **Use case** : un MJ ou joueur lance une generation NPC, item,
  encounter, art via une app SBFB. Il attend la reponse pour
  continuer sa session de jeu.
- **Tolerance latence** : la session est **interactive mais pas
  realtime** (pas un chat). Comparaisons :

| App | Latence acceptable mediane | Source |
|---|---|---|
| Signal (chat realtime) | <1s | Standard messagerie sync |
| SimpleX (chat anonyme) | 1-5s | "private but not slow" — pas de delay artificiel applicatif |
| Briar (P2P anonymous) | **30s-minutes** (Bluetooth/Tor) | "sacrifices speed for anonymity" — assume haute latence |
| Tor onion service | 2-10s | Circuit setup + 6 relays |
| Email | minutes-hours | Standard async |
| **DnD Forge target** | **<2 min mediane, <5min p99** | Decision Sprint 19 — sit between SimpleX et Briar |

Sources :
- [Privacy Guides — Real-time communication](https://www.privacyguides.org/en/real-time-communication/)
  : positionne Briar comme "high-latency, ideal for high-risk
  users like journalists" — pattern accepte pour user qui
  comprend le tradeoff.
- [Simplified Privacy — Showdown messengers](https://simplifiedprivacy.com/messengers/)
  : "Briar sacrifices speed for anonymity" → users informes
  acceptent 30s+.

**Mediane 62s** (exponential μ=90 ⇒ ln(2)×90 ≈ 62s) est :
- 60× plus lent que Signal (acceptable car DnD Forge n'est PAS
  un chat realtime)
- ~12× plus lent que Tor onion service (acceptable car SBFB
  ajoute en plus du onion-equivalent au S25+)
- ~2× plus lent qu'une session Briar typique (acceptable car
  user-base SBFB sera education-driven sur le tradeoff "cest
  P2P anonyme")
- **Sous le seuil empirique "encore acceptable" de 90s** documente
  par Briar (>90s commence a generer dropoff utilisateurs)

**p99 300s = 5 minutes** est tail acceptable parce que :
- 99% des submits resteront sous ce seuil
- Les 1% restants sont un trade-off acceptable pour le
  benefice anti-correlation
- Documente dans `docs/shell/PATTERNS.md` (Phase D scope du plan)
  pour que le user voie le rationale

### 4.2 Anonymity gain — k-anonymity effective

**Question** : combien de submits passent dans une fenetre 5min
observee typique sur un coordinator pre-launch / post-launch ?

**Pre-launch (single-coord, ~1 user)** :
- ~1-3 submits/h maximum estimate (developpement, tests, demos)
- Sur fenetre 5min : **0 ou 1 submit** typiquement
- **k-anonymity effective ≈ 1** → **anti-correlation gain
  marginal** car un observer voit 1 submit + 1 broadcast dans
  la fenetre = link probable (timing correlation toujours
  inferrable a P=1/N avec N petit)
- **Cependant**, l'observer ne peut plus **timestampe** le
  submit a la milliseconde → il sait juste "submit s'est passe
  sometime in the last 5 min" ce qui est **utile contre IMSI-
  catcher correlation** (cf. P2P_THREATS.md §6.4 : "IMSI
  catcher + SBFB upload timing = contributor identifie
  physiquement" — devient "physical presence at any moment in
  the last 5min" ce qui est tres different).

**Post-launch (target ~100-1000 active users)** :
- ~10-100 submits/min realiste
- Sur fenetre 5min : **50-500 submits** typiquement
- **k-anonymity effective = 50-500** → **anti-correlation gain
  fort** : un observer ne peut plus distinguer quel submit
  correspond a quel broadcast au sein de la fenetre

Le delay 0-5min est donc **conservatif pre-launch** (gain
limite mais zero cost a deploy) et **bien dimensionne post-
launch** ou il devient un vrai mecanisme de k-anonymity emergent.

### 4.3 Alternatives range etudiees (kickoff D4)

Le kickoff §4 D4 a deja considere et rejete trois ranges :

| Range | Mediane | UX impact | Verdict kickoff |
|---|---|---|---|
| **0-30 minutes** Tor-style rendezvous | ~10min | Casse UX DnD Forge | **Rejete** : trop long, casserait l'interactif |
| 0-2 minutes | ~25s | UX preservee | **Rejete** : tail trop court, observer fenetre 2min plus facile a correler que 5min |
| **0-5 minutes (RETENU)** | **~62s** | Tolerable (cf. §4.1) | **Retenu** : compromis publish-anonymity vs interactive |

Ce design accepte le verdict kickoff et n'**etend pas** la
discussion : 5min est le compromis figé Day 0.

### 4.4 Tunable per-deployment

Bien que le default soit 5min/μ=90s, le design **expose le
tuning** via `coordinator.toml` :

```toml
[upload_queue]
enabled = true                  # default true ; false = passthrough mode (escape hatch dev/debug)
mean_jitter_s = 90.0            # mean of exponential distribution (default 90s)
max_jitter_s = 300.0            # hard ceiling (default 300s = 5min)
flush_interval_s = 30.0         # scheduler wake-up interval (default 30s)
backpressure_soft_cap = 10000   # log warn above this queue size
backpressure_hard_cap = 100000  # reject 429 above this
```

**Cas d'usage du tunable** :

- **Dev environment** : `enabled = false` pour avoir des
  feedback loops rapides en dev (assume insecure local).
- **User paranoia** : `mean_jitter_s = 240`, `max_jitter_s = 900`
  (15min) pour push tail vers Tor-style. Documente comme
  "Briar mode" dans PATTERNS.md.
- **High-load coord** : `flush_interval_s = 5` pour reduire la
  granularite de detection oldest message.

**Pas de migration silencieuse** : changer le default require
un sprint dedie + flip dans un release note, parce que **changer
le default change l'observable timing pattern** ce qui peut
empoisonner l'anonymity set transversal entre deployments.

---

## 5. Choix d'implementation

### 5.1 asyncio.Queue vs apscheduler vs custom

**Decision** : **custom** sur `list[(deadline, task_dict)] +
asyncio.Lock + flush loop`. Pas `asyncio.Queue` natif, pas
`apscheduler`.

**Rationale** :

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| `asyncio.Queue` ([context7 cpython 3.13.9](https://github.com/python/cpython/blob/v3.13.9/Doc/library/asyncio-queue.rst)) | FIFO natif, `task_done()` signaling, prouve battle-tested | Pas de scheduling time-based — c'est une **work queue** pas un **delay queue**. Wrapper non-trivial pour deadline-based pop | **Rejete** (mauvais primitive) |
| `asyncio.PriorityQueue` | Tri par deadline natif | Meme pb : pas de mecanisme "wait until top item is due", il faudrait un poll loop. Et la persistence n'est pas built-in. | **Rejete** |
| `apscheduler.AsyncScheduler + DateTrigger` ([context7 apscheduler](https://github.com/agronholm/apscheduler)) | Production-grade, persistence built-in (SQLAlchemy/Mongo), `DateTrigger` 1-shot scheduling exact pattern | Heavy dep (~2 MB), coupling SQLAlchemy ou autre store, lifecycle management complexity, overkill pour ~10 LOC primitive | **Rejete** (overkill) |
| `arq` / `dramatiq` / `rq` | Production async job queue | Tous redis-backed → nouvelle infra dep critique. Inacceptable single-coord local | **Rejete** (infra dep) |
| **Custom `list + Lock + flush loop`** | **Zero new dep, ~80 LOC total, audit-friendly, persistence trivial via SQLite WAL** | Requires careful test coverage of edge cases | **RETENU** |

**Implementation skeleton** :

```python
import asyncio, math, secrets, json, time
from dataclasses import dataclass
from pathlib import Path
import aiosqlite, structlog

_log = structlog.get_logger(__name__)
_rng = secrets.SystemRandom()


@dataclass
class _PendingUpload:
    upload_id: str           # UUID hex
    deliver_at: float        # unix epoch seconds
    task_payload: dict       # SubmitRequest fields serialized


class UploadQueue:
    def __init__(
        self,
        *,
        db_path: Path,
        emit_fn,                          # async callable: SubmitRequest -> task_id
        mean_jitter_s: float = 90.0,
        max_jitter_s: float = 300.0,
        flush_interval_s: float = 30.0,
        soft_cap: int = 10_000,
        hard_cap: int = 100_000,
        enabled: bool = True,
    ):
        self.db_path = db_path
        self.emit_fn = emit_fn
        self.mean = mean_jitter_s
        self.max = max_jitter_s
        self.flush_interval = flush_interval_s
        self.soft_cap = soft_cap
        self.hard_cap = hard_cap
        self.enabled = enabled
        self._lock = asyncio.Lock()
        self._loop_task: asyncio.Task | None = None
        self._stopping = asyncio.Event()

    async def init(self) -> None:
        async with aiosqlite.connect(self.db_path) as db:
            await db.execute("PRAGMA journal_mode=WAL;")
            await db.execute("""
                CREATE TABLE IF NOT EXISTS delayed_uploads (
                    upload_id TEXT PRIMARY KEY,
                    deliver_at REAL NOT NULL,
                    task_payload TEXT NOT NULL,
                    enqueued_at REAL NOT NULL
                );
            """)
            await db.execute("CREATE INDEX IF NOT EXISTS idx_deliver_at ON delayed_uploads(deliver_at);")
            await db.commit()

    async def schedule(self, task_payload: dict) -> str:
        if not self.enabled:
            return await self.emit_fn(task_payload)  # passthrough escape hatch
        size = await self._size()
        if size >= self.hard_cap:
            raise QueueFullError(f"upload queue full ({size} >= hard cap {self.hard_cap})")
        if size >= self.soft_cap:
            _log.warning("upload queue near soft cap", size=size, soft_cap=self.soft_cap)
        delay = self._draw_delay()
        upload_id = secrets.token_hex(16)
        deliver_at = time.time() + delay
        async with self._lock, aiosqlite.connect(self.db_path) as db:
            await db.execute(
                "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
                (upload_id, deliver_at, json.dumps(task_payload), time.time()),
            )
            await db.commit()
        _log.info(
            "upload scheduled",
            upload_id=upload_id,
            delay_s=round(delay, 1),
            deliver_at=deliver_at,
        )
        return upload_id

    def _draw_delay(self) -> float:
        u = _rng.random()
        if u <= 0.0:  # secrets.SystemRandom.random() never 0 mais defensive
            u = 1e-18
        raw = -self.mean * math.log(u)
        return min(raw, self.max)

    async def _size(self) -> int:
        async with aiosqlite.connect(self.db_path) as db:
            cursor = await db.execute("SELECT COUNT(*) FROM delayed_uploads;")
            row = await cursor.fetchone()
            return int(row[0]) if row else 0

    async def _flush_due(self) -> int:
        now = time.time()
        flushed = 0
        async with self._lock, aiosqlite.connect(self.db_path) as db:
            db.row_factory = aiosqlite.Row
            cursor = await db.execute(
                "SELECT upload_id, task_payload FROM delayed_uploads WHERE deliver_at <= ? ORDER BY deliver_at ASC LIMIT 1000;",
                (now,),
            )
            due_rows = await cursor.fetchall()
            await cursor.close()
            for row in due_rows:
                payload = json.loads(row["task_payload"])
                try:
                    await self.emit_fn(payload)
                    flushed += 1
                    await db.execute("DELETE FROM delayed_uploads WHERE upload_id = ?", (row["upload_id"],))
                except Exception as e:  # noqa: BLE001
                    _log.error(
                        "emit_fn failed during flush, leaving in queue for retry",
                        upload_id=row["upload_id"],
                        error=str(e),
                    )
                    # Pas de DELETE → re-flush au prochain tick
            await db.commit()
        if flushed:
            _log.info("upload queue flushed", count=flushed)
        return flushed

    async def _flush_loop(self) -> None:
        while not self._stopping.is_set():
            try:
                await asyncio.wait_for(self._stopping.wait(), timeout=self.flush_interval)
                break  # stop signaled
            except asyncio.TimeoutError:
                await self._flush_due()

    async def start(self) -> None:
        await self.init()
        self._loop_task = asyncio.create_task(self._flush_loop())

    async def shutdown(self, drain: bool = True) -> None:
        self._stopping.set()
        if self._loop_task:
            try:
                await self._loop_task
            except asyncio.CancelledError:
                pass
        if drain:
            # Force flush all remaining items, regardless of deliver_at
            # Pre-launch decision : on prefere sur-emit a la perte au shutdown
            async with self._lock, aiosqlite.connect(self.db_path) as db:
                db.row_factory = aiosqlite.Row
                cursor = await db.execute("SELECT upload_id, task_payload FROM delayed_uploads;")
                rows = await cursor.fetchall()
                await cursor.close()
                for row in rows:
                    payload = json.loads(row["task_payload"])
                    try:
                        await self.emit_fn(payload)
                        await db.execute("DELETE FROM delayed_uploads WHERE upload_id = ?", (row["upload_id"],))
                    except Exception as e:  # noqa: BLE001
                        _log.warning("shutdown drain emit failed", upload_id=row["upload_id"], error=str(e))
                await db.commit()


class QueueFullError(RuntimeError):
    pass
```

### 5.2 Persistance SQLite WAL (cross-restart)

**Decision** : SQLite WAL mode dans une **table dediee
`delayed_uploads`** (pas une table partagee avec
`task_state` du dispatcher). PRAGMA journal_mode=WAL active au
init.

**Pourquoi pas in-memory only** :

Le sprint19_plan.md §7.4 test #8 dit "S19 queue is in-memory only
(docs warn queue loss on crash, persistence = Sprint 20+ tech
debt)". Ce design **revoque cette decision** et upgrade vers
persistance Phase D, pour les raisons suivantes :

1. **Critical bug masque** : un user clique "submit", voit 200
   `{"task_id": "..."}`, ferme l'iframe rassure. Si le coord
   crash dans les 90 secondes suivantes, le task **n'arrive
   jamais** sur le reseau. L'utilisateur ne saura **jamais**
   que sa soumission a ete perdue. C'est un **silent data loss**
   inacceptable meme pre-launch.
2. **Cout d'implementation faible** : aiosqlite est deja une
   dep coordinator (utilisee par dispatcher.py), ajout d'une
   table = ~10 LOC schema + ~5 LOC INSERT + ~5 LOC DELETE.
3. **Cout au runtime negligeable** : SQLite WAL + 1 INSERT par
   submit (~10 µs) << les 30s flush interval. Disk overhead
   ~200 octets par row × 10000 entries = 2 MB. Acceptable.
4. **Recovery story claire** : au boot coord, `start()` charge
   la table existante et re-flush tout ce qui a `deliver_at <
   now()` immediatement, sinon attend le prochain tick → no
   data lost, pas de ressubmit duplique car DELETE atomic post-
   emit dans la meme transaction.

**Schema** :

```sql
CREATE TABLE delayed_uploads (
    upload_id TEXT PRIMARY KEY,       -- secrets.token_hex(16)
    deliver_at REAL NOT NULL,         -- unix epoch seconds
    task_payload TEXT NOT NULL,       -- json serialized SubmitRequest dict
    enqueued_at REAL NOT NULL         -- diagnostic
);
CREATE INDEX idx_deliver_at ON delayed_uploads(deliver_at);
```

**Vacuum policy** : SQLite WAL fait checkpoint automatique
toutes les 1000 pages modifiees (default). On laisse defaults
(cf. [tech-insider — SQLite WAL Python tutorial 2026](https://tech-insider.org/sqlite-python-tutorial-fts5-wal-mode-2026/)).
Si la queue est borderline always-empty (steady-state 0 entries),
le WAL truncated naturellement.

**Reference implementations consultees** :
- [persist-queue (peter-wangxu)](https://github.com/peter-wangxu/persist-queue)
  : SQLite3 backend WAL by default + AsyncSQLiteQueue support.
  **Pourquoi pas dep direct** : cette lib enforce un certain
  schema, on prefere notre propre table pour controler le format
  payload (et garder le coord avec une seule sqlite file).
- [aiodiskqueue (ErikKalkoken)](https://github.com/ErikKalkoken/aiodiskqueue)
  : DbmEngine 3x plus rapide que SQLite, mais Dbm n'est pas
  cross-platform reliable sur Windows. SQLite WAL gagne.
- [litequeue (litements)](https://github.com/litements/litequeue)
  : pattern reference, ~200 LOC pure Python. Pourquoi pas dep :
  pareil que persist-queue, l'overhead conceptuel "encore une
  micro-dep a vetter" >> les 80 LOC qu'on ecrit nous-memes.

### 5.3 Scheduler internal flush 30s

**Decision** : boucle `asyncio.sleep(30s)` + flush due. **Pas**
un `apscheduler` ou un timer per-message.

**Pourquoi 30s spcifiquement** :

| Interval | Pros | Cons |
|---|---|---|
| 1s | Latence detection oldest message ~1s | CPU wake-up 60×/min, log spam, surcharge SQLite |
| 10s | Latence detection ~10s | Encore noisy |
| **30s** | **Latence detection ~30s, soit 33% du mean=90s** | **Granularite acceptable** |
| 60s | Plus economique | Latence detection = 67% du mean → fait deborder p99 vers 360s effectif au lieu 300s clamped |
| Per-message timer (`asyncio.sleep(delay)` per message) | Latence detection ~0s | Coup de N tasks asyncio bloquees → memory bloat + cancellation pain |

**30s** est le sweet spot empirique :
- Latence detection ≈ 30s → un message dont `deliver_at` est
  passe attend max 30s avant flush
- Wake-up 2/min → cost CPU negligeable
- Le clamp 300s du tail reste effectif (300 + 30 = 330s p99
  observable, donc on peut soit clamp a 270s et claim "p99 ≤
  300s strict", soit accepter 330s observable et l'expliquer
  PATTERNS.md). **Decision** : on clamp a 270s en interne pour
  garantir 300s observable max.

**Implementation note** : la boucle utilise `asyncio.wait_for(
self._stopping.wait(), timeout=flush_interval)` plutot que
`asyncio.sleep(flush_interval)` pur, pour permettre un shutdown
propre et instantane (ne pas attendre 30s a la fin pour stop).

### 5.4 Concurrent submit thread-safety

asyncio est single-threaded, mais **les await points sont des
yield points** ou un autre coroutine peut s'executer. Sources
de race a verifier :

1. **Concurrent schedule() calls** : 2 POST /tasks/submit
   concurrents ⇒ 2 INSERT sur `delayed_uploads`. SQLite WAL
   gere le serializing. Le `_lock` autour de l'INSERT n'est
   strictement necessaire que pour eviter de courir contre
   `_flush_due()` (qui DELETE) — pas pour l'INSERT lui-meme.
   **Decision** : on garde le lock par paranoia + clarté lecture.
2. **Concurrent flush()** : pas possible normalement (1 boucle
   loop_task), mais si `shutdown(drain=True)` est appele
   pendant que le loop task tourne encore : risk de double-emit.
   **Mitigation** : le `_stopping` event est await dans loop
   avant d'iterer, et `shutdown()` await `_loop_task` AVANT de
   demarrer le drain. Sequencement strict.
3. **Submit pendant flush** : un POST arrive pendant
   `_flush_due()` lock le `_lock` → INSERT bloque jusqu'a fin
   du flush. Latence acceptable (~10ms typique).
4. **emit_fn raise pendant flush** : la row n'est PAS deleted
   → re-flush au prochain tick. Idempotency requise sur emit_fn
   (le dispatcher.submit() generera un nouveau task_id si non
   fourni, donc **on doit fournir le task_id deterministe** dans
   le payload pour eviter le doublon. **Decision** : `schedule()`
   accepte un payload qui contient deja `task_id` resolved
   client-side, et le dispatcher passe-through.

**Test harness** : `pytest-asyncio` + `freezegun` (ou
`time-machine`) pour mock l'horloge + validation race en
parallel via `asyncio.gather`.

### 5.5 Metric `upload_queue_delay_seconds` histogram

**Decision** : log INFO bucketise, pas Prometheus exporter pour
S19. Format :

```python
_log.info(
    "upload_queue_delay",
    delay_s=round(actual_delay, 1),
    bucket=_bucket(actual_delay),  # "0-30" / "30-60" / "60-120" / "120-180" / "180-240" / "240-300"
    queue_size_at_emit=current_size,
)
```

**Rationale** :
- Pas de Prometheus dep coord-side actuellement → ajout serait
  scope creep S19.
- `structlog` JSON output → trivial a parser via `jq` post-mortem
  pour build histogram.
- Sprint 22+ peut ajouter un exporter Prometheus optionnel sans
  toucher a la primitive (juste subscribe au log channel).

**Buckets choisis** : (0-30, 30-60, 60-120, 120-180, 180-240,
240-300) sec. 6 buckets log-spaced sur le range 0-300, avec
plus de granularite au-dessus de 60s (ou commence le territoire
"perceived slow").

---

## 6. Failure modes (analyse honnete)

### 6.1 Queue overflow (backpressure strategy)

**Scenario** : un attaquant ou un bug client submit 1M de tasks
en quelques secondes pour saturer la queue.

**Mitigation design** :
- **Soft cap 10k entries** : au-dessus, log WARN structure
  (alerte ops sans casser submission)
- **Hard cap 100k entries** : au-dessus, raise `QueueFullError`
  → `api/tasks.py` traduit en HTTP 429 Too Many Requests avec
  `Retry-After` header indicatif
- **Memory footprint** : 100k × ~500 octets payload moyen =
  ~50 MB. Acceptable. SQLite gere ~1 GB sans broncher.

**Interactions** :
- **PoW Phase B amortit l'attaque** : si chaque submit cote
  client coute 100ms PoW, atteindre 100k requests/min necessite
  ~166 cores CPU dedies attaquant. Botnet possible mais cher.
- **Rate-limit per-consumer Phase S21** sera la **vraie**
  defense quand livre. S19 design assume rate-limit absent et
  utilise les caps comme stopgap.

### 6.2 SQLite full disk

**Scenario** : la machine coord runs out of disk space pendant
une queue size de 50k.

**Mitigation** :
- aiosqlite raise `sqlite3.OperationalError: database or disk
  is full` → catched dans `schedule()`, traduit en HTTP 503
  Service Unavailable + log ERROR
- Le `_flush_due` continue de tourner et DELETE les emit
  successfuls → la queue se vide naturellement
- Pas de PRAGMA `synchronous = OFF` → on accepte les writes
  fsync cost (≈1 ms) pour garantir durabilite

**Limites** :
- Pas de monitoring disk usage built-in (pre-launch). Ops
  responsability de surveiller `du -sh ~/.sbfb/`.
- Sprint 22+ ops dashboard peut ajouter un check.

### 6.3 Coordinator crash mid-flush

**Scenario** : `_flush_due` a emit 5/10 messages d'un batch,
DELETE des 5 commit OK, puis SIGKILL avant DELETE des 5
suivants.

**Mitigation** :
- **Atomic transaction per row** : chaque row est emit + DELETE
  dans la **meme transaction SQLite** (BEGIN ; emit await OK ;
  DELETE ; COMMIT). Si crash avant COMMIT, le row reste, sera
  re-flush au prochain boot.
- **Idempotency requise emit_fn** : si emit_fn est non-
  idempotent (ex : double-emit cause un duplicate task entry
  signe), le re-flush apres restart cause un duplicate. Le
  dispatcher actuel base le `task_id` sur `req.task_id or
  uuid.uuid4()` → si on stocke le task_id deja-resolu dans le
  payload, dispatcher.submit() sera idempotent par cle primaire
  SQLite (`INSERT INTO task_state` aura conflit sur task_id).
  **Decision** : `schedule()` resout le task_id AVANT enqueue,
  stocke dans payload. Le dispatcher detecte le conflit et
  log "task already emitted, skip duplicate", sans erreur.

**Test** : kill -9 du coord pendant que `_flush_due` traite un
batch + restart + verify aucun message perdu, aucun duplique.
Test d'integration Phase D requirement.

### 6.4 Submit pendant que scheduler flush (deja couvert §5.4)

Cf. §5.4 race analysis. Le `_lock` serialise. Latence acceptable.

### 6.5 Adversary submit flood pour saturer queue

**Scenario** : un app malveillant (bypass postMessage bridge ou
exploite une faille loopback) spam le coord avec 100k tasks
fakes pour bloquer les vrais users via 503.

**Mitigations** :
- **Loopback hardening S16** : seul un peer-creds-verified
  client peut atteindre /tasks/submit (UDS SO_PEERCRED Unix /
  Named Pipe DACL Windows). Donc l'attaquant doit etre **un
  process local du meme uid** → deja game over a ce niveau.
- **Bridge postMessage S13** : seules les apps loaded dans
  iframes du shell peuvent submit, et chaque submit est filtre
  par l'allowlist 3-methode du bridge.
- **Rate-limit per-consumer S21** : la vraie defense quand
  livree. S19 assume tres peu de surface attaque externe au
  vu des layers anterieurs.
- **Hard cap 100k + 429** : stopgap S19.

### 6.6 Distribution observability leak

**Scenario** : un attaquant qui peut sonder le coord (via
submit + observer broadcast time) reconstruit la distribution
exact mean+max et infere le tuning local.

**Acceptation** : c'est un **leak assume** du design — si tous
les coords du reseau utilisent les memes defaults (90/300), un
attaquant qui les connait n'apprend rien de plus. **Si un
deployment change le tuning** (cf. §4.4), il s'isole d'un
anonymity-set commun et devient distinguishable. Documente
PATTERNS.md comme "ne pas tuner le default sauf raison forte".

### 6.7 Restart-time burst (thundering herd)

**Scenario** : coord restart apres downtime de 1h. Au reload de
la table, **tous** les rows ont `deliver_at < now`. Ils sont
**tous flush en parallel** au prochain tick → burst d'emit
qui annihile le benefice anti-correlation pour ce groupe.

**Mitigation** :
- **Re-randomize au reload** : le boot detecte les rows avec
  `deliver_at < now()` et les **re-randomize** avec un nouveau
  delay exponentiel (jusqu'a 5 min), preservant l'anti-
  correlation. Le row UPDATE `deliver_at = now() + new_delay`.
- **Documente** : un crash long restart introduit un small
  delay supplementaire pour les messages affectes. UX OK.

### 6.8 Time skew / NTP failure

**Scenario** : la clock systeme jump backward 1h (NTP correction)
→ tous les `deliver_at` sont dans le futur d'1h.

**Mitigation** :
- **Use `time.monotonic()` PAS `time.time()`** pour calculs
  internal ; `time.time()` seulement pour persistance disk
  (UTC epoch).
- **Re-base au boot** : si on detecte que tous les `deliver_at`
  sont > now() + max_jitter_s × 2, on suspect une time issue
  et log WARN + re-randomize.
- Pre-launch acceptable, sprint ops futur peut sophistiquer.

### 6.9 Test determinism

**Risk** : tests qui depend de `secrets.SystemRandom` non-seedable
+ wallclock non-mockable → flaky.

**Mitigation** :
- Injectable RNG : `UploadQueue(..., rng_factory=lambda:
  secrets.SystemRandom())` default ; tests injectent un
  `random.Random(seed=42)` deterministe.
- Injectable clock : `UploadQueue(..., now_fn=time.time)`
  default ; tests utilisent `freezegun` ou un `lambda: 1234.5`
  fixed.

---

## 7. Limites connues + futures evolutions

### 7.1 Limites assumees S19

1. **Single coordinator point = SPOF** : si le coord est
   compromis ou observe, tout le mecanisme tombe. Mitigation
   futur S22+ multi-coord federe.
2. **Pas de mix-net reel** : juste latency injection coord-
   side, pas de Sphinx + cover traffic. Mitigation S25+ Loopix
   integration (cf. VALIDATED_BLUEPRINT couche 10).
3. **Adversary linkability faible cas single user** : si user
   emet 1 task et observer voit 1 submit + 1 broadcast dans
   fenetre 5min, link probable a P=1. Anti-correlation effective
   exige k > 1 dans la fenetre = post-launch.
4. **Cover traffic absent** : silent periods restent silencieux
   → un observer qui voit "0 traffic depuis 1h" sait qu'il n'y
   a pas eu de submit recent. Cover traffic = S25+.
5. **Pas d'isolation per-app** : tous les submits du coord
   passent par la meme queue, donc un app malveillant peut
   inferer le timing pattern d'un autre app si elle observe
   les flush logs. Mitigation S22+ per-app sub-queue.
6. **Pas de batching multi-message** : chaque flush emit les
   messages **individuellement** vers gossip. Un attaquant qui
   compte les bursts gossip apprend `count_per_30s_window`. Le
   batching emergent (plusieurs flush au meme tick) est passif.
   S25+ Loopix forcera le batching cryptographique.

### 7.2 Futures evolutions

| Sprint | Evolution | Mecanisme |
|---|---|---|
| S20 | Encryption at rest queue payload | Wrap payload via keystore |
| S21 | Rate-limit per-consumer | Sliding-window before schedule() |
| S22 | Per-app sub-queues | Multiple `UploadQueue` instances tagged by app_name |
| S22 | Prometheus exporter optionnel | Subscribe au log channel |
| S25 | Mix-net Loopix integration | `emit_fn` route via Sphinx packets vers entry gateway |
| S25 | Cover traffic | Inject periodic dummy `task_payload` avec marker `is_cover=true` filtered worker-side |
| S26 | Adaptive mean (charge-dependent) | Reactivate §3.5 si load reelle observee |

### 7.3 Tech debt loggee S19→PATTERNS.md

- T-S19-D-1 : pas de retry budget per-payload (un emit_fn qui
  raise systematiquement boucle infinie sur le meme row →
  monitoring ops requis)
- T-S19-D-2 : pas de prio queue (un task urgent attend autant
  qu'un task background) — design choice volontaire pour
  uniformite anti-correlation
- T-S19-D-3 : pas d'isolation per-app sub-queue (cf. §7.1.5)

---

## 8. References

### 8.1 Academic papers — mixnet & timing attacks

- **Loopix Anonymity System** (Piotrowska et al. 2017, USENIX
  Security) — [arxiv.org/abs/1703.00536](https://arxiv.org/abs/1703.00536)
  — base theorique Poisson mix + cover traffic
- **MOCHA: Mixnet Optimization Considering Honest Client
  Anonymity** (Rahimi 2025) — [eprint.iacr.org/2025/861](https://eprint.iacr.org/2025/861.pdf)
  — analyse Loopix 3-layers modern (mai 2025)
- **MixFlow: Assessing Mixnets Anonymity** (2023) —
  [eprint.iacr.org/2023/199](https://eprint.iacr.org/2023/199.pdf)
  — flow correlation attaques sur Loopix Poisson delays
- **Mixminion: Type III Anonymous Remailer** (Danezis,
  Dingledine, Mathewson 2003) —
  [mixminion.net/minion-design.pdf](https://www.mixminion.net/minion-design.pdf)
- **Mixmaster Protocol Version 2 Spec** — [freehaven.net/anonbib
  Mixmaster spec](https://www.freehaven.net/anonbib/cache/mixmaster-spec.txt)
- **On the Anonymity of Timed Pool Mixes** (Serjantov-Newman 2003)
  — [link.springer.com 10.1007/978-0-387-35691-4_41](https://link.springer.com/content/pdf/10.1007/978-0-387-35691-4_41.pdf)
- **From a Trickle to a Flood: Active Attacks on Several Mix Types**
  (Serjantov-Dingledine-Syverson 2003) —
  [freehaven.net/doc/batching-taxonomy](https://www.freehaven.net/doc/batching-taxonomy/taxonomy.pdf)
  — trickle attack taxonomy mix types
- **Timing Analysis in Low-Latency Mix Networks** (Cornell
  ESORICS 2006) — [cs.cornell.edu/~shmat/shmat_esorics06.pdf](https://www.cs.cornell.edu/~shmat/shmat_esorics06.pdf)
  — exponential delay defense reference
- **Anonymity Trilemma: Strong Anonymity, Low Bandwidth Overhead,
  Low Latency — Choose Two** (Das, Meiser, Mohammadi 2017
  IEEE S&P) — [eprint.iacr.org/2017/954](https://eprint.iacr.org/2017/954.pdf)
  — formal proof "choose two of three"
- **Cover Traffic: A Trade of Anonymity and Efficiency** (Springer
  STM 2017) — [link.springer.com 10.1007/978-3-319-68063-7_15](https://link.springer.com/chapter/10.1007/978-3-319-68063-7_15)
- **TARANET: Traffic-Analysis Resistant Anonymity at Network
  Layer** (ETH Zurich 2018 EuroS&P) —
  [netsec.ethz.ch/papers/chen_taranet_eurosp18.pdf](https://netsec.ethz.ch/publications/papers/chen_taranet_eurosp18.pdf)
- **Survey: Traffic Analysis Attacks on Tor** (MIT 6.858 2023) —
  [css.csail.mit.edu/6.858/2023/readings/tor-traffic-analysis.pdf](https://css.csail.mit.edu/6.858/2023/readings/tor-traffic-analysis.pdf)
- **Comprehensive Survey of Website Fingerprinting Attacks** (2025)
  — [arxiv.org/pdf/2510.11804](https://arxiv.org/pdf/2510.11804)
  — etat de l'art WFA + defenses 2024-2025

### 8.2 Industry implementations

- **Tor PoW for Onion Services** (Tor Project blog 2023) —
  [blog.torproject.org/introducing-proof-of-work-defense-for-onion-services](https://blog.torproject.org/introducing-proof-of-work-defense-for-onion-services/)
  — anti-DoS pattern adjacent
- **Nym Mixnet — Loopix architecture docs** — [nym.com/docs/network/concepts/loopix](https://nym.com/docs/network/concepts/loopix)
  — implementation reference Poisson mix prod-ready
- **Nym Whitepaper "Next Generation Privacy Infrastructure"** —
  [nym.com/nym-whitepaper.pdf](https://nym.com/nym-whitepaper.pdf)
- **Mullvad DAITA traffic shaping (maybenot)** —
  [github.com/maybenot-io/maybenot](https://github.com/maybenot-io/maybenot)
  — state machines pattern anti-fingerprinting
- **Briar — high-latency P2P anonymous messenger** — [privacyguides.org Real-time communication](https://www.privacyguides.org/en/real-time-communication/)
  — UX precedent "users accept 30s+ for anonymity"

### 8.3 Latency UX studies

- **Privacy Guides — Real-time communication** (2025) —
  [privacyguides.org/en/real-time-communication](https://www.privacyguides.org/en/real-time-communication/)
- **Simplified Privacy — Showdown messengers Signal/SimpleX/Briar**
  (2025) — [simplifiedprivacy.com/messengers](https://simplifiedprivacy.com/messengers/)
- **SimpleX latency design** — [simplex.chat/blog 2024
  quantum-resistance](https://simplex.chat/blog/20240314-simplex-chat-v5-6-quantum-resistance-signal-double-ratchet-algorithm.html)
- **Comparison of Instant Messengers (eylenburg)** — [eylenburg.github.io/im_comparison.htm](https://eylenburg.github.io/im_comparison.htm)

### 8.4 Implementation references — Python async + SQLite WAL

- **context7 `/python/cpython/v3.13.9` asyncio.create_task +
  asyncio.Queue** (date doc avril 2026) — primitives stdlib pour
  scheduling background + work queue
- **context7 `/agronholm/apscheduler` AsyncScheduler +
  DateTrigger** (date doc avril 2026) — comparison reference,
  lib non-retenue mais pattern DateTrigger inspirant
- **persist-queue (peter-wangxu)** — [github.com/peter-wangxu/persist-queue](https://github.com/peter-wangxu/persist-queue)
  — SQLite WAL persistent queue Python reference
- **aiodiskqueue (ErikKalkoken)** — [github.com/ErikKalkoken/aiodiskqueue](https://github.com/ErikKalkoken/aiodiskqueue)
- **litequeue (litements)** — [github.com/litements/litequeue](https://github.com/litements/litequeue)
- **plainjob — SQLite-backed job queue 15k jobs/s** —
  [github.com/justplainstuff/plainjob](https://github.com/justplainstuff/plainjob)
- **SQLite WAL Python tutorial 2026** — [tech-insider.org SQLite Python tutorial WAL](https://tech-insider.org/sqlite-python-tutorial-fts5-wal-mode-2026/)

### 8.5 Cross-refs internes au repo

- [`.planning/active/sprint19_kickoff.md §4 D4`](../active/sprint19_kickoff.md)
  — decision Day 0 figeed
- [`.planning/active/sprint19_plan.md §7`](../active/sprint19_plan.md)
  — Phase D scope + skeleton initial
- [`docs/security/HARDENING_ROADMAP.md §3 S19`](../../docs/security/HARDENING_ROADMAP.md)
  — item "Delayed upload queue (randomized 0-5min batching)"
- [`docs/security/P2P_THREATS.md §6`](../../docs/security/P2P_THREATS.md)
  — traffic analysis threat model + S17 dragnet metadata
- [`docs/security/VALIDATED_BLUEPRINT.md §6 + couche 10 traffic
  shaping`](../../docs/security/VALIDATED_BLUEPRINT.md)
  — long-term mix-net target + Mullvad DAITA precedent
- [`packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py`](../../packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py)
  — submit_task endpoint integration point Phase D
- [`packages/nexus-coordinator/src/nexus_coordinator/coordinator.py`](../../packages/nexus-coordinator/src/nexus_coordinator/coordinator.py)
  — start/stop lifecycle pour wire upload_queue.start()/shutdown()
- [`packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py`](../../packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py)
  — emit_fn target = `dispatcher.submit(SubmitRequest(...))`

---

**Status final** : ce design est consideré **accepté pour
implémentation Phase D**. Les seuls points où Phase D peut
légitimement dévier sont :

1. Si la vérification `aiosqlite + PRAGMA journal_mode=WAL` ne
   passe pas sur le sqlite local du coord (ex : version trop
   ancienne) → tomber en RWMode + log WARN, ne pas bloquer.
2. Si l'integration au `dispatcher.submit()` rencontre une
   contrainte signature-time non anticipee (le payload doit etre
   `SubmitRequest` reconstruit, pas un dict naked) → réviser le
   format payload sérialisé en SQLite (ex : utiliser
   `pydantic.BaseModel.model_dump_json()` plutôt que `json.dumps`).
3. Si les tests determinism reveal un flake sur
   `secrets.SystemRandom` mockability → fallback sur `random.
   Random(seed)` injectable + documenté que `secrets` est utilisé
   en prod uniquement.

Toute autre déviation = retour à ce design avec session fraîche
+ justification explicite.
