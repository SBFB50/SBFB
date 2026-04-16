# Relay pin bootstrap et rotation

Ce document décrit comment extraire, distribuer et faire tourner les
empreintes SPKI SHA-256 des relais iroh que la couche TLS pinning
(Sprint 19 Phase C, [`nexus_core_rs::tls_pinning`](../../crates/nexus-core-rs/src/tls_pinning.rs))
utilise pour valider les certificats présentés par chaque relai lors
du fallback HTTPS WebSocket.

Le lecteur est supposé connaître :

- la différence entre le chemin QUIC direct iroh (auth end-to-end
  Ed25519 via RFC 7250 raw public key — **pas** concerné par ce
  document) et le chemin relai HTTPS WebSocket (validation WebPKI
  classique, durci par ce module) — cf. design doc `.planning/
  research/S19_phase_C_tls_cert_pinning_design.md` §1.1 ;
- le format `~/.sbfb/relays.json` (Sprint 18 Phase C federation)
  qui liste les relais fédérés et leur URL ;
- la politique pre-launch `*_VERSION = 1` figée jusqu'au tag v1.0
  (`CLAUDE.md §Pre-launch protocol policy`).

---

## 1. Pipeline d'extraction SPKI — commande de référence

Pour obtenir l'empreinte SPKI SHA-256 base64url no-padding d'un
relai live, exécuter depuis n'importe quel poste (la commande
ne requiert aucun secret) :

```bash
RELAY=relay.iroh.network
openssl s_client -connect "${RELAY}:443" -servername "${RELAY}" \
  </dev/null 2>/dev/null \
  | openssl x509 -pubkey -noout \
  | openssl pkey -pubin -outform DER \
  | openssl dgst -sha256 -binary \
  | basenc --base64url --wrap=0 \
  | tr -d '='
```

Sortie attendue : une chaîne de **43 caractères** (SHA-256 de 32
octets encodés en base64url sans padding — 32 × 4/3 = 42,67 arrondi
vers le haut).

Cette commande est **la source canonique** — tout désaccord entre
sa sortie et la valeur stockée dans `~/.sbfb/relay-pins.json`
implique soit un MITM à l'extraction, soit une rotation de clé
non encore propagée.

### 1.1 Vérification croisée anti-MITM-à-l'extraction

Un attaquant position MITM sur le poste qui extrait peut présenter
un faux certificat au `openssl s_client`. Mitigation minimale :

1. Lancer la commande depuis **deux réseaux distincts** (p. ex.
   une VM cloud + un poste résidentiel, ou fibre + 4G) ;
2. Comparer les sorties — elles **doivent** être identiques
   byte-for-byte ;
3. Si divergence, suspendre le bootstrap et reporter sur le canal
   de sécurité du projet (`SECURITY.md`, canary).

Cette double-extraction n'élimine pas tous les scénarios (un MITM
systémique sur la CA racine peut rendre les deux extractions
cohérentes entre elles mais fausses), mais ferme les cas MITM
locaux sur un seul réseau.

### 1.2 Extraction depuis un cert PEM hors-ligne

Pour un cert déjà exporté (p. ex. récupéré via `kubectl get secret`
sur un cluster qui héberge un relai auto-hébergé cf. Sprint 19
Phase E) :

```bash
openssl x509 -in cert.pem -pubkey -noout \
  | openssl pkey -pubin -outform DER \
  | openssl dgst -sha256 -binary \
  | basenc --base64url --wrap=0 \
  | tr -d '='
```

Cette variante est déterministe et ne dépend que du contenu du
fichier — c'est la forme qu'utilisent les tests unitaires du
module [`tls_pinning`](../../crates/nexus-core-rs/src/tls_pinning.rs)
(`extract_spki_sha256_from_pem_matches_openssl_pipeline`) pour
vérifier la conformité du code Rust à ce pipeline.

---

## 2. Format `~/.sbfb/relay-pins.json`

Le fichier est un JSON utf-8 de version 1 (pré-launch, gelé) :

```json
{
  "version": 1,
  "pins": [
    {
      "relay_url": "https://relay.iroh.network",
      "spki_sha256": "Aq1c_N_zjopBnfg-mcHBozX8dgA64izVtd_zgdDioXs",
      "added_at": "2026-04-16T10:00:00Z",
      "source": "Bootstrap",
      "expires_at": null
    },
    {
      "relay_url": "https://use1-1.relay.iroh.network",
      "spki_sha256": "backup_spki_base64url_no_pad_43_chars",
      "added_at": "2026-04-16T10:00:00Z",
      "source": "Bootstrap",
      "expires_at": null
    }
  ]
}
```

### 2.1 Champs

| Champ | Type | Sémantique |
|---|---|---|
| `version` | `u8` | Toujours `1` pré-launch. Un v99 illégal déclenche `PinError::UnknownVersion`. |
| `pins` | `array` | Liste plate. **Plusieurs entrées par `relay_url` sont autorisées** — c'est le motif backup-pin RFC 7469 §4.3 pour les rotations. |
| `pins[i].relay_url` | `string` | URL complète avec schéma. Comparaison stricte `==` à l'URL passée à `PinValidator::validate()`. |
| `pins[i].spki_sha256` | `string` | 43 chars base64url no-padding, sortie de §1. |
| `pins[i].added_at` | `string` | RFC 3339. Trace d'audit — aucune incidence runtime. |
| `pins[i].source` | `"Bootstrap" \| "UserOverride"` | Provenance. Aucune incidence validation — seulement audit. |
| `pins[i].expires_at` | `string \| null` | RFC 3339 ou absent. Si présent et passé, ce pin précis est ignoré à la validation. |

Le parseur est strict (`serde(deny_unknown_fields)`) : tout champ
inattendu fait rejeter le fichier à la charge. C'est voulu — si
un futur format étend le schéma, les anciens clients doivent
tomber fort plutôt que d'ignorer silencieusement.

### 2.2 Permissions fichier

Le fichier ne contient **aucun secret** (seulement des empreintes
publiques) — il n'y a donc pas de contrainte `0600` stricte. Le
PinValidator ne tente pas de forcer les permissions : c'est au
launcher / à l'opérateur de poser le fichier où il veut.

Pour cohérence avec `tokens.json` et `consent.json`, la
convention recommandée est `0644` sur Unix, DACL standard
utilisateur sur Windows (donc world-readable mais non-world-
writable). Si un opérateur veut un contrôle plus strict (multi-
user server), `0600` est acceptable — la lib ne regarde pas.

### 2.3 Chemin par défaut

L'ordre de résolution du chemin du fichier est :

1. `$SBFB_RELAY_PINS_FILE` si défini et non-vide (override test/ops) ;
2. `$SBFB_HOME/relay-pins.json` si `SBFB_HOME` est défini ;
3. `$HOME/.sbfb/relay-pins.json` sur Unix ;
4. `$USERPROFILE\.sbfb\relay-pins.json` sur Windows.

Ce même ordre est utilisé pour `relays.json`, `tokens.json` et
`consent.json` — un opérateur qui pointe `SBFB_HOME` vers un
tmpfs ou un volume chiffré a tous ses fichiers de config au
même endroit.

---

## 3. Bootstrap pins livrés avec la release

La release du daemon nexus-shell-daemon N.M.P embarque un jeu de
pins pour les relais iroh connus au moment du sprint où le tag
est coupé. Ce jeu est :

- **calculé offline** par un maintainer, via la commande §1 ;
- **vérifié en croisé** par au moins 2 maintainers depuis des
  réseaux distincts (cf. §1.1) ;
- **inclus dans le binaire** via `include_str!` sur le fichier
  source versionné dans `crates/nexus-core-rs/assets/
  bootstrap_relay_pins.json` (livré Sprint 20 ou quand le premier
  set de relais stabilisés existe — Sprint 19 pré-launch n'a
  **aucun** pin bootstrap embarqué).

Le jeu embarqué devient le pinset par défaut si l'utilisateur n'a
pas créé son propre `~/.sbfb/relay-pins.json`. Si un pinset user
existe, il prend précédence (pattern S18 `env > file > default`
inversé ici en `file > default` puisque l'env override pointe
déjà un file).

### 3.1 État pré-launch S19

Sprint 19 ferme **sans** jeu bootstrap embarqué : le fichier
source `assets/bootstrap_relay_pins.json` n'existe pas encore.
Conséquence : si un utilisateur boot le daemon sans fichier
`~/.sbfb/relay-pins.json`, `PinValidator::from_file_with_watch`
charge un pinset vide → `validate()` **fail-open** + log warn
`"relay-pins.json empty or missing — falling back to WebPKI
validation only"`.

Ce choix délibéré évite de shipper des pins non-vérifiés par
assez de maintainers avant le tag v1.0 (contribuerait paradoxal
un risque supply-chain via la release elle-même). Sprint 20+
livrera le jeu bootstrap une fois le processus de co-signature
maintainer mature.

---

## 4. Rotation strategy — backup pin pré-publication

Quand un opérateur de relai prévoit de faire tourner sa clé
privée (perte HSM, rotation calendaire, audit), la procédure
RFC 7469 §4.3 s'applique :

### 4.1 T-30 jours — pré-publication

L'opérateur génère la **nouvelle** clé privée + CSR, et publie
son empreinte SPKI `spki_v2` **30 jours avant** la bascule,
via :

- mise à jour du fichier `assets/bootstrap_relay_pins.json`
  dans le repo SBFB → release N+1 embarque les **deux** pins
  (ancien `spki_v1` + nouveau `spki_v2`, tous deux `expires_at:
  null` pendant la période d'overlap) ;
- annonce out-of-band : mise à jour du `CANARY.txt` mensuel
  (Sprint 18 Phase E2) avec une section `active_relay_pins` (à
  spec'er Sprint 20+ `warrant canary v2` ;
  pour Sprint 19 = annonce manuelle via le SECURITY.md du repo).

Les clients qui mettent à jour vers la release N+1 voient les
deux pins et acceptent donc **soit** l'ancien cert (encore en
production), **soit** le nouveau cert (pré-déployé en staging).

### 4.2 T0 — bascule

L'opérateur bascule le serveur sur la nouvelle clé + nouveau
cert. Les clients qui ont déjà la release N+1 valident via le
pin v2. Ceux qui ont encore la release N continuent de valider
via le pin v1 **mais vont échouer** quand le serveur ne présente
plus l'ancien cert.

Stratégie de coexistence maximale :

- garder les deux certs simultanément sur le serveur pendant
  une fenêtre (cert-switch-on-SNI ou dual listen-port), **ET**
- accepter que certains clients retard de release refusent
  temporairement de se connecter au relai (ils utiliseront un
  relai fallback via la federation Sprint 18).

### 4.3 T+60 jours — nettoyage

L'opérateur signale que la bascule est ferme. Release N+2
retire le pin v1 (ou le marque `expires_at` dans le passé
pour qu'il soit skippé à la validation). Retour en régime
monoculture sur v2.

### 4.4 Rotation d'urgence (clé compromise)

Si une clé privée est connue compromise :

1. **Immédiat** : publier un CANARY warrant amendment hors-
   cycle signé (Sprint 18 Phase E2 canary key) annonçant la
   révocation du pin ;
2. **Immédiat** : release hot-fix N+1 qui retire le pin compromis
   et ajoute le nouveau (pas d'overlap — l'ancienne clé n'est
   plus trust-worthy) ;
3. **Immédiat** : user-override disponible via l'édition manuelle
   de `~/.sbfb/relay-pins.json` avec `source: "UserOverride"` — le
   hot-reload `notify` watcher applique en <50 ms sans restart
   daemon ;
4. **Post-mortem** : Publier un rapport d'incident dans
   `.planning/active/sprint{N}_incident_spki_rotation_{date}.md`
   + update `docs/security/THREAT_MODEL.md`.

Pas de DoS massif tolérable ici — un cert compromis est un
bypass complet du pinning pour les clients qui ne rotent pas
à temps. Préférer un hot-fix en urgence vs. attendre le cycle
sprint.

---

## 5. User-override workflow

Un utilisateur qui veut ajouter un pin manuellement (nouveau
relai auto-hébergé Phase E, ou mismatch bootstrap soupçonné) :

```bash
# 1. Extraire le SPKI (cf. §1)
NEW_PIN=$(openssl s_client -connect my-relay.example.org:443 \
  -servername my-relay.example.org </dev/null 2>/dev/null \
  | openssl x509 -pubkey -noout \
  | openssl pkey -pubin -outform DER \
  | openssl dgst -sha256 -binary \
  | basenc --base64url --wrap=0 | tr -d '=')

# 2. Éditer le pinset (ajout, pas remplacement)
#    Le hot-reload watcher détecte < 50ms
jq --arg url "https://my-relay.example.org" \
   --arg hash "$NEW_PIN" \
   '.pins += [{
       "relay_url": $url,
       "spki_sha256": $hash,
       "added_at": now | todate,
       "source": "UserOverride",
       "expires_at": null
   }]' ~/.sbfb/relay-pins.json > /tmp/pins.new \
  && mv /tmp/pins.new ~/.sbfb/relay-pins.json

# 3. Vérifier le reload côté daemon
tail -n 20 ~/.sbfb/logs/shell-daemon.log | grep "relay-pins.json reloaded"
```

Si le fichier n'existe pas encore, le créer à partir de §2.

Le hot-reload est **atomique** : le pinset entier est swappé via
`Arc<RwLock<_>>` en une seule opération — jamais d'état
partiellement chargé. Si le nouveau JSON est invalide, le watcher
log un warn et **garde le pinset précédent** plutôt que de
fail-open en swappant un pinset vide.

---

## 6. Cross-références

- Module Rust : [`nexus_core_rs::tls_pinning`](../../crates/nexus-core-rs/src/tls_pinning.rs)
- Design doc : [`.planning/research/S19_phase_C_tls_cert_pinning_design.md`](../../.planning/research/S19_phase_C_tls_cert_pinning_design.md)
- Federation relais : [`nexus_core_rs::relay_config`](../../crates/nexus-core-rs/src/relay_config.rs) + `~/.sbfb/relays.json`
- Threat model TLS : [`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §3 adversaries T2-T5
- Warrant canary (pour l'annonce de rotation) :
  [`docs/security/WARRANT_CANARY.md`](../security/WARRANT_CANARY.md) (Sprint 18 Phase E2)
- pkarr relay self-hosted (qui génère les certs à pinner) :
  [`docs/release/PKARR_RELAY_OPS.md`](./PKARR_RELAY_OPS.md) (Sprint 19 Phase E, à livrer)
- Tech debt T20 forked iroh connect path : [`docs/rust/PATTERNS.md`](../rust/PATTERNS.md) §T20
