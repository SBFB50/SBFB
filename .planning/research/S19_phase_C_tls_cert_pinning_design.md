# Sprint 19 Phase C — TLS cert pinning relays : design doc

**Ecrit** : 2026-04-16 (pre-implementation, design from scratch).
**Sprint** : 19, Phase C (suite Phase A DHT wire + Phase B PoW Hashcash).
**Tip master a la redaction** : `1a606a3` (post-S18 audit gate leve).
**Document parent** : `.planning/active/sprint19_kickoff.md` §4 D3,
`.planning/active/sprint19_plan.md` §6.

---

## 1. Probleme adresse

### 1.1 Threat model TLS de la chaine relai SBFB

La couche transport iroh 0.97 multiplexe deux chemins entre noeuds :
(a) QUIC direct hole-punche end-to-end avec un protocole TLS 1.3
"raw public key" (RFC 7250) base sur l'identite Ed25519 du peer —
**pas concerne** par ce design doc, l'identite est verifiee
cryptographiquement contre le `NodeId` connu et il n'y a pas de
WebPKI dans la boucle ; (b) le **fallback relay HTTPS** par lequel
les paquets transitent quand le hole-punching echoue (NAT
symetrique, CGNAT, firewall corporate). C'est sur ce **chemin (b)**
que repose ce design.

Le client iroh ouvre une connexion **WebSocket sur HTTPS** vers
chaque relay configure (preset n0 : `relay.iroh.network`,
`use1-1.relay.iroh.network`, etc., ou la federation custom S18
Phase C `~/.sbfb/relays.json`). Le handshake TLS est valide selon
**la chaine WebPKI standard du systeme** : root CAs OS + extension
intermediates. Aucun pinning, aucune contrainte CA — le scheme est
exactement celui d'un navigateur generique.

Adversaires concernes (cf. `docs/security/THREAT_MODEL.md §3`
adversary model + `.planning/archive/v1.2/sprint17_phase_A_
adversary_taxonomy.md` T0-T5) :

| Adversaire | Capacite WebPKI relevante | Scenario relai |
|---|---|---|
| **T2 — ISP / state-mandated MITM** | Force une CA nationale (ex. Trustcor, Symantec post-distrust, ou un nouveau acteur 2026) a emettre un cert valide WebPKI pour `relay.iroh.network` | Intercept tout traffic relai d'une region, deanonymise correlation NAT-traversal |
| **T3 — CA compromise** | Compromise interne d'une CA reconnue WebPKI (cas Comodo 2011, DigiNotar 2011, plus recemment incidents 2024-2026 Mozilla CA program incident reports) | Meme effet T2, sans coercion etatique necessaire |
| **T4 — Hostile relay operator** | Un relai dans la federation custom S18 emis par un operateur malveillant ou pris en charge (legal warrant) | Re-emet legitimement son cert mais change la cle ; ou redirige DNS vers un autre serveur sous son controle |
| **T5 — BGP hijack + fraudulent issuance** | BGP hijack d'un prefixe relai + Domain Validation cert opportuniste (Let's Encrypt + redirected validation) | Cert valide WebPKI pour relai dont l'attaquant ne controle pas le serveur reel |

### 1.2 Pourquoi WebPKI standard ne suffit PAS pour SBFB

**Question legitime** : "iroh signe end-to-end via Ed25519 raw
public key, le relai ne voit que des bytes opaques chiffres QUIC
— pourquoi en faire plus ?"

Reponses :

1. **Metadata leakage via TLS termination relai** : meme si la
   payload QUIC reste end-to-end chiffree, le relai voit les
   metadonnees de routing (qui parle a qui, quand, combien). Un
   MITM TLS du WebSocket relai expose ces metadonnees a
   l'attaquant — utile pour traffic analysis, correlation,
   identification d'un activiste qui parle a un journaliste.
   Cf. VALIDATED_BLUEPRINT couche 10 "opsec metadata".

2. **Downgrade vers un faux relai compromet le hole-punching
   path discovery** : le relai sert aussi de canal pour les STUN-
   like address probes iroh. Un MITM peut injecter des `EndpointAddr`
   forges qui dirigent les peers vers un faux noeud d'attaquant
   pendant le bootstrap, avant que la verification cryptographique
   du `NodeId` au handshake QUIC kick in. C'est un Eclipse-by-relay
   ; complementaire de l'Eclipse-by-DHT ferme par S18 + Phase A
   S19.

3. **Tampering du protocole relay HTTP** : iroh's relay HTTP API
   evolue (handshake, magic bytes, version negotiation). Un MITM
   peut force-downgrade vers une version sans certaines
   protections, ou injecter des messages controle qui forcent le
   client a re-dial un relai attaquant.

4. **Pre-launch context** : SBFB n'a pas encore de noeuds tiers
   en prod. Le seul WebPKI-validation-only setup avant tag v1.0
   est equivalent a "trust on first connection" sans audit. Le
   pinning permet de figer cette baseline et de detecter une
   reinitialisation suspecte de la chaine TLS.

5. **CA ecosystem 2026 sous pression** : multiple incidents
   recents (cf. WebSearch results §7 — Mozilla CA program
   distrusts 2024-2026, Let's Encrypt 90 → 45 jours rotation
   accelere les opportunites de mis-issuance). Le modele "trust
   any CA" devient progressivement moins defendable.

### 1.3 Ancrage HARDENING_ROADMAP

Cf. `docs/security/HARDENING_ROADMAP.md §3 S19` : "TLS cert
pinning relays (iroh upstream contrib)" — item explicit Sprint 19.
VALIDATED_BLUEPRINT couche 3 "transport anonyme" reference
indirectement. Cf. aussi `.planning/archive/v1.2/sprint17_phase_B_
p2p_attack_surface.md` section "BGP hijack + fraudulent issuance".

---

## 2. Decision retenue

**Pin SPKI SHA-256 base64url** pour chaque relai HTTPS configure,
charge depuis `~/.sbfb/relay-pins.json` (pattern S18 `relays.json`
+ S18 `tokens.json`), avec **hot-reload via `notify` file-watcher
50ms debounce** (pattern `TokenRotator` S18 Phase D). **Fail-open
loud-warn** si pinset absent (pre-launch convivialite) ;
**fail-closed** si pinset present mais relai absent du pinset
(une fois opt-in, on enforce strict). Verification injectee dans
le **client TLS WebSocket relai d'iroh** via `rustls`
`with_custom_certificate_verifier` au niveau du
`relay::client::ClientBuilder` — **necessite contribution
upstream iroh** car l'API actuelle n'expose le hook qu'en
`cfg(test)` (cf. §5.1).

---

## 3. Alternatives considerees

### 3.1 Full cert pin (DER hash entier)

**Description** : pin la valeur SHA-256 du certificat X.509
serialize complet (DER). Verification : recompute SHA-256(cert_der)
au handshake, compare au pin store. C'est le pattern le plus
simple a implementer.

**Avantages** :
- Aucune extraction ASN.1 a faire (juste hash bytes bruts)
- Detecte n'importe quel changement (cle, expiration date,
  extension SAN), pas seulement la cle publique
- Resilient a la roll de cle si on accepte la rotation

**Inconvenients** :
- **Let's Encrypt rotate l'integralite du cert tous les 90 jours**
  (et passe a **45 jours fin 2025** — confirme par
  `letsencrypt.org/2025/12/02/from-90-to-45`), meme si la cle est
  reutilisee. Pin full-cert = re-pin obligatoire tous les 90j a
  l'echelle de la federation = friction operator + UX cassee si
  le user ne re-fetch pas les pins assez vite
- Necessite un canal de communication out-of-band fiable pour
  push les nouveaux pins a tous les noeuds avant expiry (sinon
  fenetre de panne globale) — exactement le scenario operationnel
  qui a tue HPKP (cf. §3.6)
- Force operator a coordonner cert renewal + pin rotation
  simultanement, **single point of failure operationnelle**

**Verdict** : **rejete**. Pin full-cert est l'antipattern HPKP
qu'il faut eviter en 2026 ([Lotushints 2026/03 "Certificate
Pinning Pitfalls: Why Rotation Breaks Apps"](https://www.lotushints.com/2026/03/certificate-pinning-pitfalls-why-rotation-breaks-apps/)).
SBFB n'a aucune raison de s'imposer ce cout vs SPKI pin.

### 3.2 CA chain pin (intermediate or root only)

**Description** : pin uniquement la CA emettrice (ex. ISRG Root
X1 pour Let's Encrypt) ou un intermediate (ex. R3, R10). Cert
end-entity verifie selon WebPKI standard, mais la chaine doit
remonter a un pin connu. C'est ce que recommande
[community.letsencrypt.org HPKP best practices](https://community.letsencrypt.org/t/hpkp-best-practices-if-you-choose-to-implement/4625).

**Avantages** :
- Survit la rotation 90j (et 45j) Let's Encrypt sans intervention
- Operations coordonnees uniquement quand la CA elle-meme rotate
  son intermediate (rare, evenement annonce mois en avance)
- Compatible avec emergency cert re-issuance (perte de cle private
  serveur sans perdre la CA)

**Inconvenients** :
- **Une CA compromise = full bypass du pin**. Cf. menace T3
  §1.1 : si l'attaquant force ISRG (ou autre CA pinned) a emettre
  un cert pour `relay.iroh.network`, le pin valide naivement.
  C'est exactement le scenario CA-compromise contre lequel on
  veut se proteger
- Repose sur la securite operationnelle de la CA (HSM, audits
  WebTrust, CA program Mozilla compliance) — mais ce sont des
  hypotheses qu'on a deja en mode WebPKI standard. CA pin ajoute
  **zero garantie nouvelle** vs WebPKI standard
- Pas de defense contre CA newly-added-to-trust-store qui devient
  malveillante 6 mois plus tard

**Verdict** : **rejete**. Ne ferme pas la menace T3 (CA compromise),
qui est precisement le threat model que ce design adresse. Equivalent
operationnellement a faire confiance a WebPKI sans pinning.

### 3.3 DANE / TLSA records (RFC 6698)

**Description** : publier le hash de la cle publique (ou du cert
entier) dans un enregistrement DNS `_443._tcp.relay.iroh.network
TLSA 3 1 1 <sha256-spki>`. Le client resout le DNS, valide
DNSSEC, compare au cert presente au handshake TLS.

**Avantages** :
- Pas besoin d'un canal out-of-band : le DNS est le canal
- Standard IETF mature (RFC 6698 + 7671 + 7673)
- Operator-facing : juste un DNS record a maintenir
- Resilient a la roll cle car operator publie le nouveau hash
  AVANT de roll le cert (overlap pattern)

**Inconvenients** :
- **Necessite DNSSEC end-to-end** (resolveur recursif + zone
  signee + chain of trust). Adoption DNSSEC residentielle ~30%
  globalement 2026, recurseurs ISP rarement valident. Cf.
  [Wikipedia DANE deployment](https://en.wikipedia.org/wiki/DNS-based_Authentication_of_Named_Entities)
  — Mozilla et Google ont **explicitement refuse** d'implementer
  DANE dans Firefox/Chrome citant DNSSEC absent
- **Conflit philosophique avec le design pkarr de SBFB** :
  l'ensemble de la stack iroh + SBFB est explicitement
  **DHT-based discovery** (pkarr relay distribue, pas de
  DNS root). Introduire DANE ressuscite la dependance DNSSEC
  qu'on a evite par design
- Overhead operations : configurer DNSSEC sur le domaine
  `iroh.network` ou equivalent operator-self-hoste, gerer la KSK
  rotation, monitoring
- Bibliotheques Rust DANE-validating quasi-inexistantes en 2026
  (pas de crate equivalente a `webpki` pour DANE), implementer
  from scratch est risque securite eleve
- Un futur draft IETF [draft-ietf-dance-client-auth](https://datatracker.ietf.org/doc/draft-ietf-dance-client-auth/)
  etend DANE a l'auth client mais reste experimental, adoption
  marginale 2026

**Verdict** : **rejete**. Conflit strategique direct avec le
choix architectural pkarr/DHT. Si SBFB voulait s'appuyer sur
DNS, on aurait choisi DNS depuis le depart. DANE comme
**complement long-terme** a explorer si DHT-based discovery
prouvee insuffisante (cf. §6 evolutions futures).

### 3.4 Cert Transparency log monitoring (passive)

**Description** : ne **pas** bloquer au handshake. A la place, un
processus background interroge regulierement les logs CT publics
(`crt.sh`, Cloudflare, Let's Encrypt) pour detecter toute emission
nouvelle de cert pour les domaines des relais SBFB. Si un cert
non-prevu apparait, alerte (log warn + notification UI + telemetry
warrant canary cf. S18 Phase E2).

**Avantages** :
- Aucun risque de fail-closed accidentel (passif par design)
- Detecte mis-issuance meme par des CA legitimes pinnees ailleurs
- Compatible avec rotation 90j/45j Let's Encrypt sans friction
- Standard largement deploye 2026 — toutes les CAs grand public
  doivent log ([Cloudflare CT monitoring](https://developers.cloudflare.com/ssl/edge-certificates/additional-options/certificate-transparency-monitoring/))
- Recommandation officielle post-HPKP (cf. [GF.dev "HPKP is Dead"](https://gf.dev/learn/hpkp-is-dead))

**Inconvenients** :
- **Detection != prevention** : un MITM actif transitoire (5 min,
  enough pour exfiltrer une session sensible) peut s'achever avant
  que la prochaine query CT log detecte l'emission
- Fenetre de detection typiquement 24-48h (logs propagation)
- Necessite infra serveur ou service tiers pour query CT — ne
  scale pas a un noeud user solo
- Repose sur le fait que les CAs malveillantes log honnetement —
  un attaquant qui contraint une CA peut aussi obtenir l'emission
  hors-log (CA non-conforme CA/B Forum mais ca arrive)

**Verdict** : **non-standalone, complement utile** a explorer
**Sprint 20+** comme couche supplementaire (defense en profondeur
post-S19). Pas un substitut au SPKI pin pour Sprint 19.
Documenter en §6 evolutions futures.

### 3.5 SPKI hash pin (RETENU)

**Description** : pin le hash SHA-256 du **Subject Public Key Info**
DER-encoded de chaque cert relai (RFC 7469 §2.4 spec — meme si le
RFC HPKP lui-meme est deprecated, la primitive de hash reste
canonical). Encodage base64url (RFC 7469 utilise base64 standard ;
on utilise base64url pour UX file/JSON friendly, sans padding).

Format pin entry stockee :
```json
{
  "relay_url": "https://relay.iroh.network",
  "spki_sha256": "qO3R-...",  // 43 chars base64url no-padding
  "added_at": "2026-04-16T10:00:00Z",
  "source": "Bootstrap"
}
```

**Avantages** :
- **Survit Let's Encrypt rotation 90j (et 45j)** car la cle reste
  la meme entre renewals si l'operator configure key reuse
  (Caddy, certbot avec `--reuse-key`, acme.sh `--always-force-new-domain-key`
  off, etc.). SPKI = `(algorithm, public_key_bytes)`, identique
  d'un cert au suivant pour le meme keypair
- Detecte les changements de cle (intentionnels = key roll
  legitime, ou non-intentionnels = compromise)
- Independant du fournisseur de cert (Let's Encrypt, ZeroSSL,
  internal CA, self-signed) tant que l'operator publie le SPKI
- **Standard documente RFC 7469 §2.4** ; spec stable, librairies
  Rust matures (cf. §5.3 `x509-parser` / `spki` / `rustls-pki-types`)
- Compatible avec **backup pin** pattern RFC 7469 §4.3 (publier
  N pins, accepter si l'un matche — rotation strategy §4.3
  ci-dessous)

**Inconvenients** :
- Necessite que l'operator du relai active **key reuse** sur ses
  renewals — par defaut Let's Encrypt regenere la cle
  ([community.letsencrypt.org "Reuse Private Key in Cert renewal"](https://community.letsencrypt.org/t/reuse-private-key-in-cert-renewal/239192)).
  Implication : la doc bootstrap pin §4.2 doit demander a chaque
  operator relai SBFB d'activer `--reuse-key` ou equivalent
- Si l'operator perd la cle privee = re-pin obligatoire pour tous
  les clients = mini-HPKP-incident contained au seul relai
  affecte (mitige par backup pin §4.3 + multi-relai S18 federation
  fallback)
- Ne defend pas contre une compromise du serveur lui-meme (root
  attaquant qui vole `/etc/letsencrypt/live/relay.example.org/
  privkey.pem`) — limite documentee §6

**Verdict** : **RETENU**. Compromise raisonnable : ferme T2/T3/T4/T5
au cout d'une coordination operator key-reuse + un mecanisme
rotation honnete (§4.3).

### 3.6 Comparaison HPKP (deprecie) — lecons retenues

HPKP (RFC 7469 cote serveur HTTP header) est **mort cliniquement**
depuis 2017 — Chrome a depreciate, Mozilla a remove fin 2018.
Sources : [Wikipedia HPKP](https://en.wikipedia.org/wiki/HTTP_Public_Key_Pinning),
[Qualys "Is HTTP Public Key Pinning Dead?"](https://blog.qualys.com/product-tech/2016/09/06/is-http-public-key-pinning-dead),
[The SSL Store "Industry Experts Say Don't Use Key Pinning"](https://www.thesslstore.com/blog/industry-experts-say-dont-use-key-pinning-hpkp/).

**Pourquoi HPKP a echoue** :

1. **Footgun killing the site permanently** : un pin mal-configure
   (typo, mauvais hash, cle perdue) bloque tout client qui a vu
   le header pendant `max-age` (souvent 60 jours). Recovery
   impossible sans attendre l'expiry — site bricked
2. **TTL trop long par defaut** : RFC suggere `max-age=5184000` (60
   jours) — fenetre de panne irrecuperable enorme
3. **Aucune autorite manuelle de revocation** du pin cote browser
4. **Adoption marginale** : seules quelques mega-sites (Google,
   Facebook) avaient les moyens de gerer la complexite operations.
   Le mecanisme generique etait accessible a millions de sites
   qui n'avaient ni les besoins ni les competences. Source : co-
   auteur RFC Ryan Sleevi a publiquement renounce le standard
5. **Pas de ramp-up** : une fois pinne, pinne immediatement —
   aucune phase d'observation pour valider la coherence

**Ce que le design SBFB en retient (anti-pattern checklist)** :

| Probleme HPKP | Mitigation design SBFB |
|---|---|
| TTL bloquant 60 jours | **Pas de TTL navigateur-style** : le pinset est purement local file `~/.sbfb/relay-pins.json` ; user peut editer/supprimer/ajouter en realtime, hot-reload `notify` watcher (§4.4). Recovery = "ouvre le fichier et corrige" |
| Footgun kill-the-site | **Fail-open loud-warn** par defaut (pas de pinset = WebPKI standard). User doit explicitement opt-in en peuplant le file. Une fois opt-in, fail-closed strict per relay-url, mais l'utilisateur peut toujours deplacer le file ailleurs (recovery <30s) |
| Adoption massive de mecanisme dangereux | SBFB pinning est **operator-shipped** : les bootstrap pins sont fournis dans la release du daemon (§4.2), pas configurables par n'importe quel site distant via header |
| Pas de revocation manuelle | **Hot-reload realtime** + user edit = revocation manuelle native |
| Pas de ramp-up | **Backup pin RFC 7469 §4.3** : pinset accepte multiple SPKI pour le meme relai, permet pre-publication de la prochaine cle pendant 30 jours overlap (§4.3) |
| Coordonner serveur + browser sync | SBFB coordonne **client = daemon SBFB ; serveur = relai operator**. Les deux dans la meme communaute (operator pkarr/ONG-run S22+), coordination realiste vs millions de browsers anonymes |

**Conclusion** : HPKP a echoue car deploye comme **mecanisme
generique web-scale**. SBFB applique la primitive cryptographique
SPKI pin dans un **contexte operationnel completement different**
(file local user-editable, federation operator-coordinated, recovery
1-line). Les criticismes HPKP ne s'appliquent pas mecaniquement.

---

## 4. Format pin & rotation

### 4.1 Format `~/.sbfb/relay-pins.json`

Schema JSON, intentionnellement minimal (pattern S18 `relays.json`
+ `tokens.json`) :

```json
{
  "version": 1,
  "pins": [
    {
      "relay_url": "https://relay.iroh.network",
      "spki_sha256": "qO3R-7xPFnVJqDh8K7Yz9aBcDe-fGhIjKlMnOpQrStU",
      "added_at": "2026-04-16T10:00:00Z",
      "source": "Bootstrap",
      "expires_at": null
    },
    {
      "relay_url": "https://relay.iroh.network",
      "spki_sha256": "BACKUP-pin-pre-publication-rotation-overlap",
      "added_at": "2026-04-16T10:00:00Z",
      "source": "Bootstrap",
      "expires_at": null
    }
  ]
}
```

- `version: 1` — pre-launch policy, freeze a 1 jusqu'au tag v1.0
  (cf. CLAUDE.md §"Pre-launch protocol policy")
- `pins[]` — liste plate, **plusieurs pins peuvent partager le
  meme `relay_url`** (= backup pin pour rotation overlap §4.3)
- `spki_sha256` — base64url no-padding (43 chars), compute via
  `BASE64URL_NOPAD(SHA256(SPKI_DER))` ou SPKI_DER est la
  serialization ASN.1 DER du `SubjectPublicKeyInfo` X.509
  (RFC 5280 §4.1.2.7, RFC 7469 §2.4)
- `source: "Bootstrap" | "UserOverride"` — provenance du pin :
  bootstrap (livre dans la release SBFB), ou ajoute manuellement
  par l'utilisateur. Pas d'effet validation, juste audit trail
- `expires_at: null | RFC3339` — permet a un operator de marquer
  un pin destine a expirer (rotation planifiee). Si non-null et
  passe → log warn et le pin est considere valide jusqu'a
  echeance, post-echeance fail-closed pour ce pin (les autres
  pins du meme `relay_url` continuent)

Struct Rust (preview) :

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RelayPinsFile {
    pub version: u8,
    pub pins: Vec<RelayPin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RelayPin {
    pub relay_url: String,
    pub spki_sha256: String,  // base64url no-padding, 43 chars
    pub added_at: String,     // RFC3339
    pub source: PinSource,
    pub expires_at: Option<String>,  // RFC3339 ou None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinSource {
    Bootstrap,
    UserOverride,
}
```

Permission file UNIX `0600`, parent dir `0700` (pattern `auth_token`,
`tokens.json` S18). Sur Windows, ACL DACL standard utilisateur
courant uniquement.

### 4.2 Bootstrap pins

Bootstrap pins = SPKI hashes pre-publies dans la release SBFB pour
les relais connus au moment du sprint (S19 = 3 relais n0 + 2
fallback custom S18).

**Procedure d'extraction documentee** dans
`docs/release/RELAY_PIN_BOOTSTRAP.md` (livrable Phase C plan §6.2) :

```bash
# Extract SPKI base64url-no-pad SHA-256 d'un cert LIVE
openssl s_client -connect relay.iroh.network:443 -servername relay.iroh.network \
  </dev/null 2>/dev/null \
  | openssl x509 -pubkey -noout \
  | openssl pkey -pubin -outform DER \
  | openssl dgst -sha256 -binary \
  | basenc --base64url --wrap=0 \
  | tr -d '='
```

Verification croisee : refaire le calcul depuis un emplacement
different (e.g. depuis un noeud cloud + un noeud residentiel) pour
detecter MITM **a l'extraction** elle-meme. Si les deux calculs
divergent, alerte.

Bootstrap inclus dans le binary daemon via `include_str!` ou
`build.rs` (debat Phase C : statique-compile vs runtime-fetch) :

- **Statique-compile** : pin file embedded a build-time, garanti
  byte-for-byte coherent avec le binary version. Risque : un
  rebuild force pour chaque rotation operator. Acceptable
  pre-launch (rotation rare, sprint cadence)
- **Runtime fetch** : daemon fetch un pin manifest depuis HTTPS
  signed par cle operator au boot. Necessite infra de signature +
  HTTPS sans le-meme-pin-bootstrap problem (chicken-and-egg).
  **Reject** pour Sprint 19, peut-etre Sprint 22+ avec gossip
  signed de pin manifests via Ed25519 keys d'operator

**Choix Sprint 19** : **statique-compile** dans le binary +
override file `~/.sbfb/relay-pins.json` user-editable qui prend
precedence (pattern S18 env > file > default).

### 4.3 Rotation strategy

**Le probleme honnete** : si une cle relai roule (operator forced
a roll, perte cle privee, rotation planifiee), comment annoncer
le nouveau pin AVANT que l'ancienne expire chez les clients ?
Sans canal communication fiable, on a un mini-HPKP-incident
localise.

**Reponse 3 couches** :

1. **Backup pin pre-publication (RFC 7469 §4.3)** : le pinset
   accepte deja **plusieurs pins par `relay_url`**. L'operator
   peut publier `pin_v2` 30 jours **avant** le swap, le bootstrap
   release suivante embarque les deux (`pin_v1` + `pin_v2`), les
   clients qui update voient les deux pins. Au jour J du swap,
   le serveur presente le cert avec la cle v2 → pin_v2 matche.
   L'ancienne pin_v1 est removed dans la release N+2 (60 jours
   plus tard)

2. **User-override** : pour le user qui ne veut pas attendre la
   release SBFB suivante, il peut editer `~/.sbfb/relay-pins.json`
   manuellement et ajouter un pin via `source: "UserOverride"`.
   Hot-reload (§4.4) prend effet en <50ms

3. **Communication channel** : **honestement, on n'a pas encore de
   canal fiable** pour annoncer un key roll relai a tous les
   clients SBFB en temps quasi-reel. Les options envisagees :

   - **Gossip iroh signed message** : un message gossip "PIN_
     ROTATION_NOTICE" signe par operator key Ed25519, broadcast
     sur un topic dedie. **Probleme** : un noeud qui n'est pas
     encore connecte au reseau (premier boot apres rotation) ne
     verra pas le message, fail-closed sur le relai. Acceptable
     si N relais redondants + quorum
   - **Warrant canary mensuel S18 Phase E2** : etendre le canary
     manifest pour inclure une section "active_pins" signee par
     SBFB project key. Update mensuel = max 30 jours latency
     pour propagation. **Decision** : reporter S20+ comme
     extension warrant canary (Phase B canary v2)
   - **Out-of-band** : signature publique sur un site web SBFB
     (sbfb.network/pins.txt avec PGP detached sig). User fait
     `curl + verify + cp`. **Decision Sprint 19** : c'est le
     fallback documente §RELAY_PIN_BOOTSTRAP.md. Pas elegant,
     pas scalable, mais marche pre-launch et single-operator
     scenario

   **Honnete** : **on ne sait pas encore comment annoncer un
   key roll proprement a l'echelle "100k noeuds installes
   partout dans le monde"**. Le design Sprint 19 est dimensionne
   pour le contexte pre-launch + first weeks post-launch
   (operator-only). La generalisation est un sujet ouvert
   Sprint 22-30 (cf. §6 evolutions futures + HARDENING_ROADMAP
   §3 S25 "decentralized PKI bootstrap").

### 4.4 Hot-reload via `notify` watcher

Reuse direct du pattern `TokenRotator` S18 Phase D (cf.
`crates/nexus-shell-daemon-core/src/auth.rs:421-607`) :

```rust
pub struct PinValidator {
    pins: Arc<RwLock<PinSet>>,
    _watcher: Option<RecommendedWatcher>,
}

impl PinValidator {
    pub fn from_file_with_watch(path: PathBuf) -> Result<Self, PinError> {
        let initial = load_from_disk(&path)?;
        let pins = Arc::new(RwLock::new(initial));
        let pins_clone = pins.clone();
        let path_clone = path.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            // Debounce 50ms inline (pattern S18 ConsentWatcher)
            std::thread::sleep(Duration::from_millis(50));
            match res {
                Ok(_event) => {
                    match load_from_disk(&path_clone) {
                        Ok(new) => {
                            *pins_clone.write().unwrap() = new;
                            tracing::info!("relay-pins.json reloaded");
                        }
                        Err(e) => tracing::warn!(
                            "relay-pins.json reload failed: {e} ; keeping previous pinset"
                        ),
                    }
                }
                Err(e) => tracing::warn!("notify watcher error: {e}"),
            }
        })?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        Ok(Self { pins, _watcher: Some(watcher) })
    }

    pub fn validate(&self, relay_url: &str, cert_der: &[u8]) -> Result<(), PinError> {
        let pinset = self.pins.read().unwrap();
        let actual_spki = extract_spki_sha256(cert_der)?;
        let candidates: Vec<&RelayPin> = pinset.pins.iter()
            .filter(|p| p.relay_url == relay_url)
            .collect();
        if candidates.is_empty() {
            // Fail-open if pinset entirely empty (no opt-in yet)
            // Fail-closed if pinset has other relays but not this one
            if pinset.pins.is_empty() {
                tracing::warn!(
                    relay_url = %relay_url,
                    "relay-pins.json empty — falling back to WebPKI (loud warn)"
                );
                return Ok(());
            }
            return Err(PinError::NoPin(relay_url.to_string()));
        }
        // RFC 7469 §2.6 : ANY pin matches → accept
        let now = chrono::Utc::now();
        for pin in candidates {
            if let Some(exp) = &pin.expires_at {
                let exp_dt: chrono::DateTime<chrono::Utc> = exp.parse()?;
                if now > exp_dt {
                    continue;  // pin expired, skip
                }
            }
            if pin.spki_sha256 == actual_spki {
                return Ok(());
            }
        }
        Err(PinError::SpkiMismatch {
            relay_url: relay_url.to_string(),
            actual: actual_spki,
        })
    }
}
```

**Atomicite** : reload atomic via `Arc<RwLock<PinSet>>` swap, jamais
d'etat partiellement chargee (load_from_disk returns either
complete PinSet or Err — keep previous on Err).

**Debounce 50ms** : evite les multi-fires sur les editors qui font
write+rename (vim, atomic save). Pattern direct du `ConsentWatcher`
S16 (cf. `crates/nexus-worker-core/src/consent.rs`).

---

## 5. Choix d'implementation

### 5.1 Hook iroh 0.97 — ou injecter

**Etat actuel iroh 0.97** (verifie via context7 `/websites/rs_iroh`
2026-04-16) :

- Le QUIC client TLS config est genere par `tls::TlsConfig::
  make_client_config(alpns, keylog)` qui utilise **deja** un
  `with_custom_certificate_verifier(self.server_verifier.clone())`
  — **mais ce verifier est l'iroh raw-public-key verifier
  (TLS 1.3 RFC 7250), pas un WebPKI verifier**. C'est la bonne
  abstraction : iroh fait son propre TLS basee sur l'identite
  Ed25519 du peer, pas WebPKI. Hors scope Phase C
- Le **relay client** (`relay::client::ClientBuilder`) utilise
  un transport HTTPS WebSocket distinct, qui s'appuie sur
  `reqwest`/`hyper` + rustls **WebPKI standard**. C'est ICI
  qu'on injecte le pin
- L'API `relay::client::ClientBuilder` expose
  `.insecure_skip_cert_verify(skip)` mais **uniquement** sous
  `#[cfg(any(test, feature = "test-utils"))]` — pas en API
  publique

**Consequence critique** : **iroh 0.97 n'expose PAS de hook
public pour custom relay cert verifier**. On a 3 options :

#### Option A — Contribution upstream iroh (PREFERREE)

Ouvrir une PR upstream qui ajoute :
```rust
impl relay::client::ClientBuilder {
    pub fn custom_cert_verifier(
        mut self,
        verifier: Arc<dyn rustls::client::danger::ServerCertVerifier>
    ) -> Self { ... }
}
```

**Avantages** : design propre, beneficie a tout l'ecosysteme iroh,
zero forking maintenance. **Inconvenients** : timeline upstream
inconnue (review iroh ~2-6 semaines historiquement), bloque la
phase C tant qu'integre.

#### Option B — Forked connect path (FALLBACK Sprint 19)

Wrapper local qui re-implemente `magicsock::transports::relay::
actor::create_relay_builder` avec un custom rustls config injecte
manuellement. Necessite de copier ~150 LOC de iroh internals,
maintenance burden a chaque iroh update. Marqueur TODO pour
remplacer par Option A des qu'upstream merge.

#### Option C — Skip pinning Sprint 19, attendre upstream

**Reject** : repousse Sprint 20+ alors que HARDENING_ROADMAP §3
S19 liste l'item explicitement. Aussi, on perd l'opportunite de
pousser le PR upstream avec une demande concrete documentee.

**Decision retenue** : **Option B en Sprint 19** (forked connect
path) **+ Option A draft PR upstream Phase F** (livre une PR
draft a iroh meme si pas merge ce sprint, pour signal et
discussion). Le forked path est marque tech-debt explicite dans
`PATTERNS.md` avec une issue tracking pour switcher des qu'iroh
0.98+ expose le hook.

Phase C plan §6.2 doit refleter cette realite : le commit Phase
C contient le forked path + comment "TODO upstream PR iroh#XXXX"
+ doc CONTRIBUTING.md note pour onboarding contributeur.

### 5.2 rustls custom `ServerCertVerifier`

API rustls 0.23 (verifie via context7 `/websites/rs_rustls_0_23_37_
rustls` 2026-04-16) — **stable et bien documentee** :

```rust
use rustls::client::danger::{ServerCertVerifier, HandshakeSignatureValid};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};

#[derive(Debug)]
pub struct PinningServerVerifier {
    pin_validator: Arc<PinValidator>,
    fallback: Arc<rustls::client::WebPkiServerVerifier>,
}

impl ServerCertVerifier for PinningServerVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, Error> {
        // 1. WebPKI standard d'abord (chain trust + expiry +
        //    DNS name match) — defense en profondeur, pas de
        //    bypass
        self.fallback.verify_server_cert(
            end_entity, intermediates, server_name, ocsp_response, now
        )?;

        // 2. Pin SPKI custom — fail-closed if pinset present
        //    et relai pinned ; fail-open if pinset vide
        let server_url = format!("https://{}", match server_name {
            ServerName::DnsName(dns) => dns.as_ref(),
            _ => return Err(Error::General("non-DNS server name".into())),
        });
        self.pin_validator
            .validate(&server_url, end_entity.as_ref())
            .map_err(|e| Error::General(format!("pin: {e}")))?;

        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(...) -> Result<HandshakeSignatureValid, Error> {
        self.fallback.verify_tls12_signature(...)
    }
    fn verify_tls13_signature(...) -> Result<HandshakeSignatureValid, Error> {
        self.fallback.verify_tls13_signature(...)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.fallback.supported_verify_schemes()
    }
}
```

**Defense en profondeur** : pin **complete** WebPKI, ne le
**remplace pas**. Si WebPKI fail (cert expired, name mismatch),
on rejette avant meme d'evaluer le pin. Si WebPKI ok mais pin
mismatch, on rejette. Les deux check doivent passer.

**Crypto provider** : `rustls::crypto::ring::default_provider()`
(ring) — meme provider que iroh utilise actuellement. Alternative
[`aws-lc-rs`](https://github.com/aws/aws-lc-rs) (FIPS 140-3
certified) recommandee par VALIDATED_BLUEPRINT couche 1 long-
terme — mais ring est OK Sprint 19, switch a aws-lc-rs Sprint 26+
PQC migration window.

**CVE awareness 2026** :
- `CVE-2026-31812` (quinn-proto DoS via QUIC transport params,
  fix 0.11.14+) — **n'affecte PAS rustls TLS validation** mais
  est dans la stack iroh transport. Documente dans
  `cargo-deny.toml` advisory check S18 baseline. Verifier au
  Phase C qu'iroh 0.97 pinne quinn-proto >= 0.11.14
- Pas de CVE active confirme sur rustls 0.23.x ou
  webpki/rustls-webpki au 2026-04-16 (cf. rustsec.org checks
  cargo-deny job S18 Phase A)

### 5.3 SPKI extract — code path

**Choix de la crate** : `rustls-pki-types` + `x509-parser`.

**Pourquoi pas la crate `spki` directement** : `spki` (RustCrypto)
est elegant mais ajoute une dep crypto-rs ecosystem en plus de
rustls — duplication. `x509-parser` est deja dans l'ecosysteme
rustls indirectement, zero-copy, fuzzed. SPKI extract = extracting
le champ `tbsCertificate.subjectPublicKeyInfo` du `CertificateDer`,
ASN.1 DER-encoded (cf. [x509-parser docs](https://docs.rs/x509-parser/latest/x509_parser/x509/struct.SubjectPublicKeyInfo.html)).

```rust
use sha2::{Digest, Sha256};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use x509_parser::prelude::*;

pub fn extract_spki_sha256(cert_der: &[u8]) -> Result<String, PinError> {
    let (_, cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| PinError::ParseCert(e.to_string()))?;
    // tbs_certificate.subject_pki est le SubjectPublicKeyInfo
    // DER-encoded raw bytes
    let spki_der = cert.tbs_certificate.subject_pki.raw;
    let hash = Sha256::digest(spki_der);
    Ok(URL_SAFE_NO_PAD.encode(hash))
}
```

**Test vector** : un cert PEM de test stocke
`crates/nexus-core-rs/tests/fixtures/relay_test_cert.pem` avec un
SPKI hash hardcoded check (regenerable via la procedure openssl
§4.2). Permet test reproductible sans relais reel.

### 5.4 Fail-close vs fail-open policy

| Etat | Comportement | Rationale |
|---|---|---|
| Pinset absent (fichier `~/.sbfb/relay-pins.json` n'existe pas) | **Fail-open** + `tracing::warn!` au boot ("PIN_NOT_CONFIGURED — falling back to WebPKI standard") | Pre-launch convivialite ; user qui a pas opt-in garde le comportement S18 byte-for-byte. Future S22+ peut basculer en fail-close si l'ecosysteme mature |
| Pinset vide (`pins: []`) | Equivalent a absent : fail-open + warn | Difference cosmetique uniquement |
| Pinset present + relai listed + match | **Accept** | Cas nominal |
| Pinset present + relai listed + mismatch | **Reject** + log error + retry sur autre relai (federation S18) | Suspicion MITM, on coupe |
| Pinset present + relai NOT listed (mais d'autres relais oui) | **Reject** (NoPin) | User a opt-in sur certains relais ; ajouter un nouveau relai sans le pin = potentiellement nouveau relai legitime, mais on prefere fail-closed et demander a user de confirmer en ajoutant le pin manuellement |
| Pin avec `expires_at` passe | Skip ce pin specifique. Si tous les pins du relai expires → equivalent a NoPin (Reject) | Permet rotation planifiee |

**Note importante** : la decision "fail-open vs fail-close" est
**parametrable** Phase C+ via `~/.sbfb/relay-pins.json` un champ
top-level `enforcement_mode: "permissive" | "strict"`. Sprint 19
hardcode `permissive` par defaut, le champ est present dans le
schema mais ignored ; Sprint 22+ active le toggle UI.

---

## 6. Limites connues + futures evolutions

### 6.1 Limites Sprint 19

- **Ne defend pas contre** : compromise du serveur relai lui-meme
  (root attaquant qui vole `/etc/letsencrypt/live/.../privkey.pem`
  ET le binaire iroh-relay). Le pin sera "valide" car le serveur
  presente la vraie cle. Defense necessite operator ops hygiene
  (HSM relai, monitoring intrusion) — hors scope SBFB
- **DoS via key-roll force** : un attaquant qui compromise un
  operator relai temporairement et le force a roll sa cle peut
  causer une fenetre de panne pour tous les clients qui n'ont
  pas update leurs pins. Mitigation : multi-relai federation S18
  (3 relais, fallback automatique), backup pin pre-publie §4.3
- **Pre-launch only honest scope** : le mecanisme est dimensionne
  pour ~10 relais et ~quelques 100 noeuds installes. Scaling
  10k+ relais necessite pin discovery decentralise (DHT-based,
  cf. §6.3)
- **Pas de revocation cross-noeuds** : si SBFB project decouvre
  qu'un pin specifique est compromis (e.g. operator a publie sa
  cle privee par accident), il n'y a pas de mecanisme push-revoke
  vers tous les noeuds existants. Force de release SBFB suivante
  + canary update mensuel + user manual edit
- **Forked iroh internals** : Option B §5.1 introduit ~150 LOC
  copy-paste de iroh internals que SBFB doit maintenir compatible
  a chaque iroh upgrade. Tech debt explicite, tracked

### 6.2 Cert Transparency monitoring (Sprint 20+)

Ajouter un background job daemon-side qui interroge crt.sh pour
les domaines des relais SBFB enrolled dans le pinset, alerte si
un cert non-prevu apparait (nouveau cert non-precede par une
publication out-of-band du operator). Pattern complementaire au
pin SPKI : pin = active defense au handshake, CT monitoring =
passive detection des emissions non-autorisees, meme si le client
n'a jamais connecte au relai compromis. Reference : [Cloudflare
CT Monitoring](https://developers.cloudflare.com/ssl/edge-certificates/additional-options/certificate-transparency-monitoring/),
[MDN CT](https://developer.mozilla.org/en-US/docs/Web/Security/Certificate_Transparency).

### 6.3 Decentralized PKI long-terme (Sprint 25+)

Si le DHT-based discovery pkarr prouve insuffisant pour communiquer
les rotations de pin (latency, fiabilite), envisager :

- **DANE/TLSA via DNSSEC** : reintroduit la dependance DNSSEC
  qu'on a evite, mais modulo une zone signee dediee
  `_relays.sbfb.network` peut servir de fallback authoritative
- **Gossip-signed pin manifests** : un operator publie un pin
  rotation announcement signe Ed25519 sur un topic gossip dedie
  `sbfb/pins/v1`, les clients re-fetch et merge dans leur pinset.
  Necessite un PoW Hashcash (cf. Sprint 19 Phase B) pour eviter
  le spam. Ferme la boucle "communication channel honnete" du
  §4.3
- **Warrant canary v2** : etendre le manifest canary mensuel
  S18 Phase E2 pour inclure une section `active_relay_pins`
  signee SBFB project key. Update mensuel = max 30j latency

### 6.4 Contribution upstream iroh (Sprint 19 Phase F livrable optionnel)

Draft PR `iroh#XXXX` proposant l'API publique :
```rust
relay::client::ClientBuilder::custom_cert_verifier(
    Arc<dyn rustls::client::danger::ServerCertVerifier>
)
```

Avec rationale : "applications utilisant iroh comme transport
veulent appliquer leur propre policy de validation cert relai
(SPKI pinning, internal CA, DANE) sans skip total via
`insecure_skip_cert_verify`". Test cases : SPKI pin, DANE, custom
internal CA. Si merge, Sprint 20+ supprime le forked path.

### 6.5 Migration aws-lc-rs (Sprint 26+ PQC)

VALIDATED_BLUEPRINT couche 1 recommande aws-lc-rs (FIPS 140-3) vs
ring pour les operations crypto sensibles. Le custom cert
verifier de Phase C est ecrit contre `rustls::client::danger`
trait — provider-agnostic. Switch ring → aws-lc-rs Sprint 26+
n'exige aucune modification du code Phase C.

---

## 7. References

### 7.1 RFCs et specs

- **RFC 7469** — *Public Key Pinning Extension for HTTP*
  (HPKP, deprecated 2018 mais SPKI hash spec §2.4 toujours
  canonical) :
  https://datatracker.ietf.org/doc/html/rfc7469
  — Section 2.4 (SPKI Fingerprint) et Section 4.3 (Backup Pins)
  sont les deux sections directement applicables au design SBFB
- **RFC 5280** — *X.509 PKI Certificate and CRL Profile* §4.1.2.7
  SubjectPublicKeyInfo encoding :
  https://datatracker.ietf.org/doc/html/rfc5280
- **RFC 6698** — *DANE TLSA* (alternative rejetee §3.3) :
  https://www.rfc-editor.org/rfc/rfc6698
- **RFC 7671** — *DANE Operational Guidance* :
  https://www.rfc-editor.org/rfc/rfc7671.html
- **RFC 4648** — base64 / base64url encoding :
  https://datatracker.ietf.org/doc/html/rfc4648

### 7.2 Documentation et articles

- **OWASP Cheat Sheet — Certificate and Public Key Pinning** :
  https://owasp.org/www-community/controls/Certificate_and_Public_Key_Pinning
- **GF.dev — "HPKP is Dead — What Replaced It and Why"** (post-
  HPKP recommendations 2024) :
  https://gf.dev/learn/hpkp-is-dead
- **Qualys — "Is HTTP Public Key Pinning Dead?"** (Ivan Ristic) :
  https://blog.qualys.com/product-tech/2016/09/06/is-http-public-key-pinning-dead
- **The SSL Store — "Industry Experts Say Don't Use Key Pinning
  (HPKP)"** (Ryan Sleevi quote) :
  https://www.thesslstore.com/blog/industry-experts-say-dont-use-key-pinning-hpkp/
- **Wikipedia — HTTP Public Key Pinning** (timeline deprecation) :
  https://en.wikipedia.org/wiki/HTTP_Public_Key_Pinning
- **Lotushints 2026/03 — "Certificate Pinning Pitfalls: Why
  Rotation Breaks Apps"** (article 2026 sur le footgun rotation) :
  https://www.lotushints.com/2026/03/certificate-pinning-pitfalls-why-rotation-breaks-apps/
- **Let's Encrypt — "Decreasing Certificate Lifetimes to 45 Days"**
  (2025-12-02, impact pin design 2026) :
  https://letsencrypt.org/2025/12/02/from-90-to-45
- **Let's Encrypt community — HPKP best practices** :
  https://community.letsencrypt.org/t/hpkp-best-practices-if-you-choose-to-implement/4625
- **Tor Blog — "Detecting Certificate Authority compromises and
  web browser collusion"** :
  https://blog.torproject.org/detecting-certificate-authority-compromises-and-web-browser-collusion/
- **Cloudflare — Certificate Transparency Monitoring docs** :
  https://developers.cloudflare.com/ssl/edge-certificates/additional-options/certificate-transparency-monitoring/
- **MDN Web Docs — Certificate Transparency** :
  https://developer.mozilla.org/en-US/docs/Web/Security/Certificate_Transparency
- **Wikipedia — DANE deployment status** :
  https://en.wikipedia.org/wiki/DNS-based_Authentication_of_Named_Entities

### 7.3 Traces context7 (avril 2026)

Toutes consultees 2026-04-16 :

- `/websites/rs_iroh` — query "Endpoint builder TLS config rustls
  custom ServerCertVerifier injection for relay client" :
  - Resultat critique : `make_client_config` utilise deja
    `with_custom_certificate_verifier(self.server_verifier.clone())`
    pour QUIC TLS raw-public-key (RFC 7250) — c'est l'auth iroh
    Ed25519, pas WebPKI
  - `relay::client::ClientBuilder::insecure_skip_cert_verify` est
    `cfg(any(test, feature = "test-utils"))` — **pas d'API publique
    pour custom validator**, confirme la decision §5.1 Option B +
    upstream PR
  - `ActiveRelayActor::create_relay_builder` et `RelayConnectionOptions`
    structurent le hook potentiel pour upstream PR

- `/websites/rs_iroh` — query "RelayClient ClientBuilder TLS
  certificate verification insecure_skip_cert_verify expose custom
  verifier" : confirme l'API actuelle limitee, suggere le path
  d'extension upstream

- `/websites/rs_rustls_0_23_37_rustls` — query "custom
  ServerCertVerifier dangerous_configuration with_custom_
  certificate_verifier 2026" :
  - `DangerousClientConfigBuilder::with_custom_certificate_verifier
    (Arc<dyn ServerCertVerifier>)` — API stable rustls 0.23.x
  - `ServerCertVerifier` trait : signature complete documentee
    (`verify_server_cert`, `verify_tls12_signature`,
    `verify_tls13_signature`, `supported_verify_schemes`)
  - `WebPkiServerVerifier` peut etre wrappe (delegation pattern)
    pour faire defense-en-profondeur (WebPKI + pin)

### 7.4 CVE 2026 stack TLS

- **CVE-2026-31812** — quinn-proto DoS via QUIC transport params,
  fix 0.11.14+ :
  https://advisories.gitlab.com/pkg/cargo/quinn-proto/CVE-2026-31812/
  https://www.miggo.io/vulnerability-database/cve/CVE-2026-31812
  — Severity High, n'affecte pas TLS validation mais doit etre
  pinne au cargo-deny.toml advisory-check S18 baseline
- **rustsec.org advisories** (2026 baseline check) :
  https://rustsec.org/advisories/
  — Pas d'advisory active rustls 0.23.x au 2026-04-16
- **rustls vulnerability docs** (manual reference) :
  https://docs.rs/rustls/latest/rustls/manual/_02_tls_vulnerabilities/index.html

### 7.5 Sources crates Rust

- **x509-parser** (zero-copy ASN.1 parser, fuzzed, recommande
  Sprint 19) :
  https://docs.rs/x509-parser
  https://docs.rs/x509-parser/latest/x509_parser/x509/struct.SubjectPublicKeyInfo.html
- **rustls-pki-types** (types compat rustls ecosystem) :
  https://docs.rs/rustls-pki-types/latest/rustls_pki_types/
- **spki** (RustCrypto, alternative §5.3 non-retenue) :
  https://docs.rs/spki

### 7.6 Cross-refs internes SBFB

- `.planning/active/sprint19_kickoff.md` §4 D3 — decision Day 0
- `.planning/active/sprint19_plan.md` §6 — Phase C scope
- `crates/nexus-core-rs/src/relay_config.rs` — pattern loader
  `relays.json` S18 (reuse pour `relay-pins.json`)
- `crates/nexus-shell-daemon-core/src/auth.rs:421-607` — pattern
  `TokenRotator` + `notify` watcher S18 Phase D (reuse direct)
- `crates/nexus-worker-core/src/consent.rs` — pattern
  `ConsentWatcher` S16 (reuse pour debounce 50ms)
- `docs/security/THREAT_MODEL.md` §3 adversary model + §1.1 trust
  boundaries
- `.planning/archive/v1.2/sprint17_phase_B_p2p_attack_surface.md`
  — section BGP hijack + fraudulent issuance (T5)
- `.planning/archive/v1.2/sprint17_phase_D_hardening_roadmap.md`
  §3 S19 — TLS cert pinning relays explicit item
- `.planning/archive/v1.2/sprint17_validated_blueprint.md` couche
  3 "transport anonyme" + couche 1 PQC long-terme aws-lc-rs

---

**Statut design doc** : *draft pre-implementation Phase C*. Les
choix Day 0 (D3 SPKI hash retenu) sont figes ; les details
d'implementation (forked vs upstream, signature exacte du custom
verifier, format JSON `enforcement_mode` toggle Sprint 22+) sont
ouverts a discussion lors de la session fraiche Phase C qui re-
verifiera context7 iroh + rustls a la date d'execution (potentielle
nouvelle version iroh entre 2026-04-16 et le commit Phase C).
