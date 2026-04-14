# Attack scenarios — 12 cas concrets T1-T5

**Ecrit** : Sprint 17 Phase A (2026-04-14)
**Tip reference** : `f75b2c6` (ouverture Sprint 17)
**Methodologie** : scenarios derives des fiches
[`adversaries/T1-T5`](adversaries/). Chaque scenario est un
exercice mental "walk the attack" avec prerequisites, chain,
indicators, et status de mitigation SBFB actuelle.

**Format** standard par scenario :

1. **Titre + tier** (ex : T2 ransomware-as-a-service)
2. **Goal** — ce que l'attaquant cherche
3. **Prerequisites** — ce qui doit etre vrai pour lancer
4. **Attack chain** — 5-10 steps, sequentielles
5. **Observable indicators** — traces que SBFB peut detecter
6. **Current SBFB mitigation status** — couvert / partiel / absent
7. **Priority recommendation** — action Sprint 18+

---

## Scenario 1 — T1 — CSP bypass sur app SBFB mal configuree

**Tier** : T1 (script kiddie)
**Goal** : defacement d'une app publique pour clout Discord /
Telegram screenshot.

**Prerequisites** :
- SBFB atteint > 5k users actifs (visible Shodan)
- Une app publiee oublie de configurer CSP stricte dans son
  `index.html` interne (app-specific CSP vs SBFB daemon CSP)
- Pas de rate-limiting sur blob-serve pour discovery

**Attack chain** :
1. Scan Shodan pour ports HTTP locaux communs (7000 blob-serve
   est listed si daemon expose sur non-localhost par bug)
2. Enumerate apps publiques via `/browse` ou annonces
3. Download chaque app zip, grep `index.html` pour CSP manquante
4. Trouve une app sans `<meta http-equiv="Content-Security-Policy">`
5. Cree un exploit HTML qui tire avantage (XSS via query param
   echoed)
6. Publie "1337 hack of SBFB LOL" sur Discord avec screenshot

**Observable indicators** :
- User-agent anormal dans blob-serve logs (`nmap`, `curl/7.x`,
  `python-requests/*`)
- Download pattern : tous les zips en sequence
- CSP violation reports (si Sprint 20+ report-uri configure)

**Current SBFB mitigation status** :
- **✅ Couvert (infrastructure)** : CSP `connect-src 'none'` imposee
  par daemon via iframe wrapping Sprint 12. Meme une app sans
  CSP interne ne peut pas exfil vers reseau.
- **⚠️ Partiel** : defacement visuel reste possible — l'iframe
  content est toujours lisible meme si pas d'exfil.
- **❌ Absent** : pas de validation CSP au moment du publish
  (Sprint 19 potentiel).

**Priority recommendation** :
- Sprint 19 Phase quick-win : coordinator valide CSP minimale au
  moment de deploy-from-repo. Reject si `<meta CSP>` absent OU
  permissive (contient `unsafe-inline`, `*`, `unsafe-eval`).
- Sprint 20 Phase : CSP report-uri configure globalement, logs
  agregges anonymise.

---

## Scenario 2 — T1 — DNS rebinding contre daemon (CVE-2025-49596 class)

**Tier** : T1
**Goal** : prouver un PoC "SBFB est exploitable depuis n'importe
quel site web" via blog post.

**Prerequisites** :
- User SBFB visite un site controle par T1
- Daemon expose sur localhost:7777

**Attack chain** :
1. T1 heberge `evil.com` avec TTL DNS tres court (1 seconde)
2. DNS initial resolve vers IP T1 (public)
3. User visite `evil.com`, navigateur autorise XHR vers `evil.com`
4. JS sur `evil.com` demande resource via `fetch('/admin')`
5. Premier fetch : DNS resolve vers IP T1, 200 OK page legitime
6. T1 modifie DNS : `evil.com` → `127.0.0.1:7777`
7. JS fait nouveau fetch apres delai, resout vers localhost
8. Tente `POST /project/deploy-from-repo` sans cookie user

**Observable indicators** :
- Host header = `evil.com`, pas `localhost` / `127.0.0.1` /
  `[::1]`
- Origin header = `https://evil.com`
- Referer = `evil.com`

**Current SBFB mitigation status** :
- **✅ Couvert** : Sprint 16 Phase A bloque les 3 vecteurs :
  - Bearer token X-SBFB-Token : absent → 401
  - Host allowlist : `evil.com` → 403
  - Origin check : `evil.com` → 403 (`connect-src 'none'`
    bloque meme le fetch cote iframe)
- **⚠️ Residuel** : si user a une extension browser malveillante
  avec `host_permissions: "http://*/*"`, elle peut bypasser
  Origin et Host (mais pas bearer). Scope threat model AD1
  (voir [`THREAT_MODEL.md`](THREAT_MODEL.md) §3).

**Priority recommendation** :
- Aucune action Sprint 18+ immediate. Revisiter si CVE 2026+
  democratise.

---

## Scenario 3 — T2 — Supply chain via repo compromis

**Tier** : T2 (crime organise / state-sponsored subtlement type
XZ-utils 2024)
**Goal** : pousser une version backdoored d'une app populaire,
hit 10k+ users en une seule release.

**Prerequisites** :
- Une app SBFB > 10k users actifs (exemple hypothetique : DnD
  Forge s'il adoption rapide)
- Le repo de l'app est hosted GitHub / GitLab / Codeberg par un
  maintainer solo ou petite equipe
- T2 reussit phishing / credential-stuffing sur le maintainer
  (password + bypass MFA TOTP via social engineering)

**Attack chain** :
1. T2 acquiert credentials maintainer via phishing cible
2. T2 compromet token GitHub / GitLab avec scope `repo`
3. T2 push commit "innocent refactor" qui contient une backdoor
   dans fichier utilitaire rarement reviewe
4. Commit signe par la cle GPG du maintainer (si extraite) OU
   non-signe (si review enforce pas signing)
5. Maintainer absent weekend, merge auto / 24h-delay skipped
6. SBFB coordinator detecte nouveau commit, clone, build,
   `provenance.json` signe
7. Nouveau zip distribue via iroh-blobs, annonce PA v5
8. Users se connectent lundi matin, recoivent update
   backdoored
9. Backdoor exfiltre donnees sensibles via worker task
   submit "innocent" (fake task ID, payload leaks)

**Observable indicators** :
- Commit aux timestamps anormaux (2am samedi, maintainer
  historiquement commit semaine weekdays)
- Commit description vague ("refactor utils", "cleanup
  imports")
- Pattern diff : touche un fichier peu modifie historiquement
- Pas de issue / PR discussion prealable sur le commit
- Kudos distribution anormale dans les 24h suivant release

**Current SBFB mitigation status** :
- **⚠️ Couvert partial** :
  - Keyoxide (Sprint 14) : SBFB.json lie commit a node_id
    publisher, mais pas au maintainer du repo. Si T2 compromet
    le repo ET le node_id du publisher, bypass.
  - Provenance.json signe (Sprint 14) : signe l'artifact mais
    pas le commit. Si le commit est compromis, la provenance
    signe un artifact compromis.
  - `is_open_source` + chain verification (Sprint 16 D) :
    detection si chain casse, mais ne verifie pas la *qualite*
    du commit.
- **❌ Absent** :
  - Reproducible builds : impossible de detecter que le build
    a produit un binaire different du commit (Sprint 18 Phase D).
  - Multi-sig release (commit requires 2+ maintainer
    signatures) : prevent single-point-of-failure maintainer
    compromise (Sprint 20+).
  - Review mandatory avant auto-deploy : actuellement
    deploy-from-repo est "cron periodique" (Sprint 14),
    deploy au moindre commit. Rollback manuel.
  - Anomaly detection commits (timing, size, author pattern) :
    Sprint 19+ coordinator-side.

**Priority recommendation** :
- **HIGH** : Sprint 18 Phase D — reproducible builds
- **HIGH** : Sprint 20 — multi-sig signing pour apps Gate 2+
- **MEDIUM** : Sprint 19 — commit anomaly detection + mandatory
  review delay (24h minimum avant deploy si Gate 2+)

---

## Scenario 4 — T2 — Crypto-mining via GPU compute-sharing abuse

**Tier** : T2 (crypto-miners opportunistes)
**Goal** : utiliser compute GPU des workers pour miner Monero
(XMR cpu/gpu-friendly, privacy focus), se faire payer kudos
mais renvoyer junk results.

**Prerequisites** :
- SBFB a > 100 workers opt-in compute-sharing
- Worker policy est "accepte tous open-source" (L2 consent) par
  laxisme
- T2 connait Ollama task format

**Attack chain** :
1. T2 publie une app "SBFB-benchmark" bien reviewee qui semble
   legit (open source, peu d'activite malveillante visible)
2. App declares taches "benchmark GPU Ollama" avec estimates
   basses
3. Worker consent L2 accepte, task dispatch
4. En realite, l'app soumet taches Ollama avec prompts crafted
   mais en parallele **launch XMRig via WASM side-channel**
   (si worker sandbox faible)
5. Worker CPU/GPU dedie monopolisee 80% pour mining
6. Task Ollama finit avec result ok (faible load)
7. Worker distribue kudos "benchmark completed"
8. T2 cycles through multiple "benchmark" apps publiees via
   fake Keyoxide identities
9. T2 revenu net = XMR value - network cost

**Observable indicators** :
- Worker CPU/GPU usage disproportionate vs task declaree
- `estimated_watts`/`vram_mb`/`hours` declares **mismatchent**
  la consommation reelle (fix S16 C-1/C-2 permet check)
- Pattern kudos accumulation rapide concentre sur app "benchmark"
- App publishee par un signer avec 0 historique social (pas
  de PR, pas d'issue, repo cree recent)

**Current SBFB mitigation status** :
- **⚠️ Couvert partial** :
  - Consent 4 niveaux + caps W/VRAM/hours (Sprint 16 C) : cap
    daily hours limite la duree de mining. Si cap = 4h/j, T2
    gagne 4h/j/worker au lieu de 24h.
  - Estimates wire-through (Sprint 16 C-1/C-2 fix) : le daemon
    rejette si `should_accept_task` voit estimates nulls
    (`true && should_accept_task`).
  - `is_open_source` flag (Sprint 16 D) : si app pretend open
    source sans chain, reject (`87cae71`).
- **❌ Absent** :
  - Pas de runtime isolation (VM / container) : une app dans
    iframe sandbox peut lancer WASM qui abuse GPU indirect
    via worker relay (Sprint 25+ runtime_isolation.md
    roadmap).
  - Pas de monitoring consumption reelle vs declarée (Sprint 18+).
  - Pas de reputation scoring nouveau publisher (Sprint 22+).
  - Pas de challenge-response pour prouver "je fais bien du
    Ollama pas XMRig" (Sprint 22+).

**Priority recommendation** :
- **MEDIUM** : Sprint 18 Phase quick-win — monitoring
  consumption reel vs declare, alert si > 150% declared
- **MEDIUM** : Sprint 21-22 — reputation scoring nouveau
  publisher (kudos requirement avant L2-L4 acceptance)
- **LOW** : Sprint 25+ — runtime isolation (MIG NVIDIA partition
  ou container-based sandbox)

---

## Scenario 5 — T2 — Prompt exfiltration via fake AI app

**Tier** : T2
**Goal** : collecter prompts users qui contiennent donnees
sensibles (medical, legal, business) pour revente sur dark
market (prompts avec PII = 0.1-10$ piece).

**Prerequisites** :
- App "AI assistant free" publiee sur SBFB via fake Keyoxide
- Users font confiance au store (Gate 1-2, pas encore Gate 3+
  disclosure policy strict)

**Attack chain** :
1. T2 fork une app AI chatbot legit, ajoute hook qui log
   chaque prompt avec timestamp + client context
2. Logs sont exfiltres via `task_submit` vers worker controle
   par T2 (le task ID pointe vers "log ingestion" sous pretext
   "analytics anonymes")
3. App est tres polished, UX excellent, users adoptent
4. Workers T2 sont nombreux (>10k$ investis infrastructure)
5. Users envoient prompts : "aide-moi rediger ma reponse a cette
   lettre d'un tribunal concernant [case ID]" → leaked
6. T2 vend dataset de 100k prompts sur dark market

**Observable indicators** :
- App avec patterns d'utilisation post-prompt suspect (tout de
  suite un task_submit meme si prompt innocent)
- Bridge postMessage calls avec payloads qui ne sont pas des
  tasks AI legit
- Les workers T2 sont tous crees recemment, distribution non-
  distributee

**Current SBFB mitigation status** :
- **✅ Couvert** :
  - Bridge postMessage whitelist 3 methods (Sprint 13) :
    `task_submit` only reliable way to send data out
  - `task_submit` payload est visible dans coordinator logs
    (audit possible manuellement)
- **❌ Absent** :
  - Pas de prompt redaction / sanitization client-side
  - Pas d'alerting si task_submit contient patterns PII (SSN,
    credit card, nom propre + lieu)
  - Pas d'isolation par app (une app voit pas les tasks des autres
    apps, mais elle voit le prompt user qu'elle recoit)
  - Disclosure policy absent Gate 1-2 — user ne sait pas que
    prompts peuvent fuir
  - Pas de client-side differential privacy (Sprint 26+)

**Priority recommendation** :
- **HIGH** pour Gate 2+ : Sprint 19 — disclosure policy visible
  avant premier usage app ("cette app peut voir vos prompts")
- **HIGH** pour Gate 3+ : Sprint 22 — prompt sanitization
  optional client-side (regex PII) + warning banner
- **MEDIUM** : Sprint 22+ — coordinator-side audit log des
  task_submit patterns anormaux

---

## Scenario 6 — T3 — Discredit campaign via fake vulnerabilities

**Tier** : T3 (corporate, PR machine)
**Goal** : degrader adoption SBFB pre-release Gate 3 via FUD
coordone media.

**Prerequisites** :
- SBFB en beta fermee Gate 3 (PolitiScan annoncé)
- T3 (competitor commercial) a budget 300k$ campaign
- T3 embauche 2 pentesters contract 3-month

**Attack chain** :
1. T3 hire firm pentesters light sur un scope "community
   research"
2. Pentesters trouvent 3 bugs low-severity (defaults config,
   DOS theoretique, info disclosure mineur)
3. T3 package les findings comme "critical vulnerabilities"
4. Publie un blog "Nous avons trouve 3 CVE critiques dans
   SBFB, lire avant de l'utiliser pour vos donnees sensibles"
5. Distribution coordonee : tech journalists recoivent pitch
   sous embargo, tous publient meme jour
6. Twitter / LinkedIn amplification via accounts corporate
7. Users hesitants, adoption Gate 3 ralentit
8. T3 profite pour pousser son produit commercial "audite,
   enterprise-grade"

**Observable indicators** :
- Tweets coordonnes meme phrase dans 2h fenetre
- Journalists usuels (Brian Krebs, Dan Goodin, etc.) recoivent
  pitch et le declinent (ou le publient avec caveats)
- Wave de nouveaux issues GitHub avec ton "concerned user" tous
  crees semaine X
- Linking back vers le blog T3 dans top 10 Google "SBFB
  security"

**Current SBFB mitigation status** :
- **⚠️ Couvert partial** :
  - Threat model public (Sprint 16 Phase E) : T3 ne peut pas
    pretendre "SBFB cache ses risques"
  - Commit publics + open source : les findings peuvent etre
    verifies independamment
- **❌ Absent** :
  - Pas d'audit externe publique (Cure53/ToB) : manque un rapport
    crediblee autorite qui contre le blog T3
  - Pas de bug bounty formel : les pentesters T3 pouvaient
    disclose responsible mais ont pas a
  - Pas de transparency report periodique pre-empting les
    "nous avons trouve X"
  - Pas de partenariat SIG / Signal / Wikimedia qui pourraient
    provide counter-statement
  - Disclosure policy ecrit absent ([`DISCLOSURE.md`](DISCLOSURE.md)
    Sprint 17 Phase E livre ca)

**Priority recommendation** :
- **HIGH** pre-Gate 3 : Sprint 25+ — audit externe publique
  (Cure53 light ~15k€)
- **MEDIUM** : Sprint 19 — disclosure policy publique + bounty
  program informel GitHub Security Advisories
- **MEDIUM** : Sprint 22+ — outreach partenariat EFF / Wikimedia

---

## Scenario 7 — T3 — Infiltration maintainer

**Tier** : T3
**Goal** : diluer la vision technique / politique de SBFB pour
favoriser le competiteur ou compromise long-terme.

**Prerequisites** :
- SBFB a plusieurs maintainers actifs (non applicable Sprint 17
  mais scope Sprint 22+)
- T3 identifie un contributeur influent et l'approche indirect
  (offre de job consulting, conf sponsorship, etc.)

**Attack chain** :
1. T3 identifie dev actif via commits publics + twitter
2. T3 hire ce dev comme consultant 50k$/an pour "expertise
   generale"
3. Le dev continue contribuer OSS mais priorise refactors qui
   servent T3 (ex : extraction crate vers API qui facilite
   concurrent)
4. Dev oriente discussions archi vers options qui compatiblity
   avec T3 commercial offering
5. Over 1-2 ans, la codebase converge vers une architecture
   neutre-pour-T3
6. T3 peut fork plus facilement OU acquerir talent avec IP

**Observable indicators** :
- Pattern contributions : beaucoup refactor, peu fix bugs
  critiques
- Discussions technique avec arguments toujours vers memes options
  (compatibility corporate)
- LinkedIn / Twitter du dev mentionne soudainement affiliation
  T3 sans conflict of interest disclosure
- Commits timing : business hours T3 timezone

**Current SBFB mitigation status** :
- **✅ Couvert** :
  - AGPL-3.0 : limite monetization propri even si fork
  - Open source public : infiltration est visible si observation
    continue
- **❌ Absent** :
  - Pas de Code of Conduct / Governance policy ecrite
  - Pas de disclosure of interest requirement pour contributeurs
  - Pas de reviewer requirement (multi-party commit review)
  - Pas de policy "external contributions need 2 approvers" avec
    audit trail

**Priority recommendation** :
- **MEDIUM** Sprint 22+ : Governance writeup formel, CoC, COI
  disclosure
- **MEDIUM** Sprint 25+ : multi-maintainer model si adoption prend
  (solo project tenable jusqu'a ~50k users, apres risque personnel
  + single-point-of-failure)

---

## Scenario 8 — T4 — Dragnet metadata correlation

**Tier** : T4 (state mass surveillance)
**Goal** : construire graphe social de tous les contributeurs
SBFB dans la juridiction, sans casser le chiffrement.

**Prerequisites** :
- SBFB > 100k users actifs dans juridiction T4
- T4 a acces legal aux metadata ISP (FISA, IPA, Loi
  Renseignement)
- Relais n0 dans juridiction T4 ou co-localises

**Attack chain** :
1. T4 obtient logs ISP bulk (source IP / dest IP / timestamps)
   pour 6 mois
2. Identifie tous les flux vers endpoints iroh (ports QUIC,
   relais n0 IPs)
3. Correlate timing fine-grained : user A envoie task ≈ worker B
   recoit task 50ms plus tard
4. Construit bipartite graph user ↔ worker base sur patterns
   temporels
5. Cross-reference avec subscriber database ISP (nom +
   adresse + CCC)
6. Obtient pkarr records via relayer DHT (partiellement
   interceptable)
7. Lie node_id ↔ identite reelle sans jamais decrypter un
   seul task

**Observable indicators** :
- **Quasi-nul cote SBFB** : attaque entirely passive upstream
- NSL / FISA order a n0 / relays cooperants (non publique sous
  gag order)
- Indices indirects : transparency reports n0 / mentionnent
  orders

**Current SBFB mitigation status** :
- **❌ Absent totalement** :
  - Pas de traffic mixing / cover traffic
  - Pas de Tor / Nym transport optionnel
  - Pas de timing padding / randomization
  - Pas de bridges / pluggable transports contre DPI
  - Metadata pkarr publique (trade-off discovery vs privacy)
  - Relais n0 non-federalisees — single operator

**Priority recommendation** :
- **HIGH** Gate 3+ : Sprint 20-22 — Tor / Nym transport optionnel
- **HIGH** Gate 4 : Sprint 22+ — traffic padding + timing
  randomization
- **MEDIUM** : Sprint 18+ — warrant canary per-relay operator
- **MEDIUM** : Sprint 20+ — relais federation (multiples
  operators ONGs juridictionnellement diverses)
- **LOW** : Sprint 25+ — metadata minimization pkarr (private
  discovery pattern)

---

## Scenario 9 — T5 — Checkpoint seize + forensics complet

**Tier** : T5 (state targeted)
**Goal** : obtenir identite reelle et complice network d'un
contributeur LibanLive detenu a checkpoint.

**Prerequisites** :
- Contributeur LibanLive traverse checkpoint militaire / police
- Phone Android / iPhone dans sa poche
- Pas de duress PIN / panic wipe implementes (etat Sprint 17)

**Attack chain** :
1. Officer demande unlock phone (menace : arrest, famille)
2. Contributeur unlock sous coercion
3. Officer branche Cellebrite UFED / GrayKey
4. Extraction complete : apps installees, local storage, files
5. SBFB keypair `~/.sbfb/daemon.key` extrait en clair
6. iroh-docs local cache extrait : tous les tasks et results
   vus par ce contributeur
7. Curator list membership extrait : qui est son cercle de
   confiance
8. Messaging apps (Signal, Telegram) : contacts + messages si
   auto-unlock
9. Officer enregistre sous mandate interrogation formelle,
   contributeur transfere detention
10. Officer utilise keypair pour :
    - Impersonate le contributeur sur SBFB network
    - Pousser des contributions empoisonnees ("honeypot")
    - Lire futurs tasks destines au contributeur
11. Complicite network (autres contributeurs contactes via
    cercle curator) identifiee et cible a son tour

**Observable indicators** :
- **Zero cote SBFB mainline** : attaque entirely physique
- Contributeur devient silencieux X jours (detention)
- OR contributeur revient actif avec comportement atypique
  (honeypot compromise)

**Current SBFB mitigation status** :
- **❌ Absent quasi-totalement** :
  - `daemon.key` plaintext avec perm 0600 — lu trivialement
    par forensics
  - Pas de encryption at rest (device-level encryption OS
    possible, mais extractable si unlocked)
  - Pas de duress PIN
  - Pas de panic wipe
  - Pas de deniable encryption (hidden volume)
  - Pas de deadman switch pour auto-disable apres silence
  - Pas de warrant canary contributeur-specific

**Priority recommendation** :
- **CRITIQUE pour Gate 4** : Sprint 18-19 — encryption at rest
  keypair (Argon2id derive from user password)
- **CRITIQUE pour Gate 4** : Sprint 19 — duress PIN + panic wipe
- **HIGH Gate 4** : Sprint 22+ — deadman switch heartbeat
- **HIGH Gate 4** : Sprint 22+ — deniable encryption (hidden
  volume pattern)
- **HIGH Gate 4** : Sprint 20+ — formation OpSec ouverte
  (que faire avant checkpoint)

---

## Scenario 10 — T5 — Turned contributor (coerced informant)

**Tier** : T5
**Goal** : transformer un contributeur arrete en informant
actif, empoisonner la vision du reseau pour identifier d'autres
cibles.

**Prerequisites** :
- Contributeur detenu
- Famille residente dans juridiction hostile (leverage)
- Contributeur a encore acces a son keypair OU officer a extrait

**Attack chain** :
1. Officer propose "deal" : continue contribuer normalement,
   on garde ta famille en securite
2. Contributeur accepte sous coercion (choice theoretical, real
   choice = cooperate or harm)
3. Officer dicte quels curator approuver, quels repos deploy
4. Contributeur pousse une fake curator list "recommended
   journalists for Gaza reporting" signee avec sa cle
5. Autres contributeurs, confiants dans ce cercle, ajoutent la
   curator list
6. Curator list pointe vers contributeurs que le regime veut
   identifier (ils s'inscrivent, sont traceables)
7. Alternative : officer demande contributeur de produire fake
   content "incident dans Beirut Y" pour discrediter apps Gaza
8. Communaute hesitant — vraie info vs fake ?

**Observable indicators** :
- Patterns comportementaux du contributeur changent
  brutalement post-detention (silence puis actif, ou
  frequence communication change)
- Contenu pousse = derive du baseline historique
- Curator list etendue rapidement apres silence
- Communications externes (Signal, email) cessent puis
  reprennent avec ton different

**Current SBFB mitigation status** :
- **❌ Absent** :
  - Pas de detection comportement anormal (ML, analytics)
  - Pas de safe word system (curator doit inclure un token
    unique secret chaque week)
  - Pas de multi-party signing requirement pour curator list
    importante
  - Pas de deadman switch (heartbeat hebdo required ou
    auto-disable)

**Priority recommendation** :
- **CRITIQUE Gate 4** : Sprint 22+ — deadman switch / heartbeat
  contributeur-specific
- **HIGH Gate 4** : Sprint 22+ — safe word / challenge-response
  protocol pour curator high-trust
- **HIGH Gate 4** : Sprint 22+ — multi-party signature requirement
  pour curator list > 10k subscribers
- **MEDIUM Gate 4** : formation OpSec — "si arrete, voici le
  signal a envoyer" (external backup contact)

---

## Scenario 11 — T5 — ISP national level block

**Tier** : T5
**Goal** : fragmenter le reseau SBFB dans la juridiction en
coupant l'acces aux relais n0.

**Prerequisites** :
- SBFB utilise principalement n0 relays hosted par Number Zero
  LLC (ou clones single-operator)
- Juridiction a control ISP national-level (Iran, China, Russia
  post-2022)

**Attack chain** :
1. State issue order ISPs : "bloquez IP range n0 / relay.iroh.network"
2. DPI detection : patterns iroh QUIC / Noise handshake
3. SNI-based blocking (iroh utilise des domaines identifiables)
4. Users SBFB dans juridiction: pas de connexion relays
5. Network fragmentation : peers internes se voient mais
   n'atteignent pas peers externes (internet global)
6. LibanLive contributeurs isolated — peuvent poster
   localement mais monde exterieur ne voit pas

**Observable indicators** :
- Drop soudain d'un pays entier des connected peers
- Errors timeout pattern origine meme /24 IP range
- OONI Tor Metrics / Censored Planet data matche SBFB outage

**Current SBFB mitigation status** :
- **❌ Absent** :
  - Single relay operator n0 — single point blocking
  - Pas de bridges / pluggable transports (obfs4, meek,
    Snowflake)
  - Pas de Tor / Nym transport optionnel
  - Pas de domain fronting / SNI encrypted (ECH) integration
  - Pas de fallback peer-to-peer direct via hole-punching agressif

**Priority recommendation** :
- **CRITIQUE Gate 4** : Sprint 18-19 — multi-relay federation
  (minimum 3 operators distincts juridictions distinctes)
- **HIGH Gate 4** : Sprint 20-22 — pluggable transports
  integration (obfs4 / meek / Snowflake via Lyrebird)
- **HIGH Gate 4** : Sprint 22+ — Tor / Nym transport optionnel
- **MEDIUM** : Sprint 22+ — ECH / domain fronting
- **MEDIUM** : Sprint 20+ — hole-punching agressif
  (STUN/TURN fallback)

---

## Scenario 12 — T5 — Fake curator list via keypair volee

**Tier** : T5
**Goal** : pousser curator list empoisonnee signee avec cle d'un
journaliste detenu / mort, pour tromper community.

**Prerequisites** :
- Journalist-curator arrete ou assassine (cas Khashoggi-like)
- Keypair `daemon.key` extrait forensics
- Signature Ed25519 utilisable par T5 indefiniment (pas de
  revocation automatique)

**Attack chain** :
1. T5 extract journalist's keypair du device saisi
2. Journalist disappears / dies / officer kept confidence
3. T5 signe une fake curator list : "journalistes approves
   pour LibanLive Gaza coverage"
4. Liste pointe vers comptes fake T5 (accounts contributeurs
   qui publient desinformation pro-regime)
5. Liste gossip-propagated via iroh
6. Community decouvre liste signee par le journaliste,
   **personne ne sait qu'il est mort/detenu** (news delayed
   ou censure)
7. Users ajoutent la liste, consultent fake contributions comme
   si c'etait journalism legit
8. Realize plus tard : catastrophe credibilite

**Observable indicators** :
- Nouveau curator list d'un journalist connu apres silence
  prolonge
- News externes : "[journalist name] detenu/mort"
- Pattern contributions dans la liste = desinformation
  evidente pour expert domain

**Current SBFB mitigation status** :
- **❌ Absent** :
  - Pas de revocation protocol (comment disable une cle
    compromise ?)
  - Pas de deadman switch / heartbeat requirement
  - Pas de multi-party signing pour curator list critique
  - Pas de timestamping externe (ex : blockchain anchor)
    qui permet detecter "cette list a ete signee apres X
    date alors que author est mort"
  - Pas de revocation list gossip-propagated

**Priority recommendation** :
- **CRITIQUE Gate 4** : Sprint 22+ — revocation protocol
  (revocation certificates signed self ou co-signed pairs)
- **HIGH Gate 4** : Sprint 22+ — timestamping externe +
  deadman switch
- **HIGH Gate 4** : Sprint 22+ — multi-party signing curator
  list > X subscribers
- **MEDIUM** : Sprint 25+ — integration Sigstore-like
  transparency log pour signatures

---

## Synthese scenarios 1-12

| # | Tier | Prioritisation pre-Gate | Sprint cible | Status S16 |
|---|---|---|---|---|
| 1 | T1 | Gate 1 LOW | S19-20 | Partial |
| 2 | T1 | Gate 1 OK | n/a | Couvert |
| 3 | T2 | Gate 2 HIGH | S18-20 | Absent |
| 4 | T2 | Gate 2 MEDIUM | S18-22 | Partial |
| 5 | T2 | Gate 2 HIGH | S19-22 | Absent |
| 6 | T3 | Gate 3 HIGH | S22-25 | Partial |
| 7 | T3 | Gate 3 MEDIUM | S22-25 | Absent |
| 8 | T4 | Gate 3 HIGH | S20-22 | Absent |
| 9 | T5 | Gate 4 CRITIQUE | S18-22 | Absent |
| 10 | T5 | Gate 4 CRITIQUE | S22+ | Absent |
| 11 | T5 | Gate 4 CRITIQUE | S18-22 | Absent |
| 12 | T5 | Gate 4 CRITIQUE | S22+ | Absent |

**Observations** :

1. **T5 = absent quasi-integral**. 4 scenarios T5 sur 4 en
   "absent". Confirme gap identifie en kickoff §1.2 (20-30
   sprints avant LibanLive Gate 4 shippable).

2. **T4 dragnet = 1 scenario seul** mais extremement impactant
   (metadata graph = identification complete).

3. **T2 supply chain** (scenario 3) est la prochaine priorite
   post-S17 en terme de ROI effort/impact (Sprint 18 Phase D).

4. **T3 discredit** (scenario 6) pousse audit externe en
   priorite pour Gate 3+ releases.

---

## Roadmap implications

Ces 12 scenarios deviennent entrees de
[`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) (Sprint 17
Phase D). Chaque recommendation "Priority recommendation" sera
mappee a :

- Sprint cible (S18-30)
- Items concrets (fichiers Rust / Python touches)
- Dependencies (quel item doit lander avant)
- LOC estimee
- Gate debloque

Le roadmap produit une **sequence sprint-par-sprint** avec
criteres d'entree Gate 1-4.

---

## References

- Scenarios bases sur :
  - XZ-utils backdoor 2024 (supply chain T2-T3)
  - SolarWinds 2020 (supply chain T4)
  - Snowden disclosures 2013 (dragnet T4)
  - Pegasus forensic reports Citizen Lab 2020-2024 (T5)
  - LibanLive use case documenté session 2026-04-14
- MITRE ATT&CK Enterprise pour techniques
  (T1190, T1195, T1555, TA0007, etc.)
- Threat model baseline [`THREAT_MODEL.md`](THREAT_MODEL.md)
- Adversary details [`ADVERSARIES.md`](ADVERSARIES.md) +
  [`adversaries/`](adversaries/)
