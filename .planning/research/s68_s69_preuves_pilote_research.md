# Recherche S68 "Pack De Preuves Release" + S69 "Pilote Ferme"

**Date :** 2026-05-18
**Confiance globale :** MEDIUM-HIGH (code verifie + ecosysteme documente)
**Mode :** Ecosystem + Feasibility

---

## Table des matieres

1. [Etat actuel des preuves dans le codebase](#1-etat-actuel-des-preuves-dans-le-codebase)
2. [Recherche externe — Proof packs et attestation de release](#2-recherche-externe)
3. [Proposition de structure proof pack SBFB](#3-proposition-proof-pack-sbfb)
4. [Etat actuel de l'installation / onboarding](#4-etat-actuel-installation-onboarding)
5. [Recherche externe — Pilotes fermes](#5-recherche-externe-pilotes-fermes)
6. [Checklist pilote SBFB](#6-checklist-pilote-sbfb)
7. [Dependances S68 vers S69](#7-dependances-s68-s69)
8. [Plans de phases proposes](#8-plans-de-phases)
9. [Risques et mitigations](#9-risques-et-mitigations)
10. [Sources et confiance](#10-sources-et-confiance)

---

## 1. Etat actuel des preuves dans le codebase

### 1.1 Provenance SLSA L1 — CE QUI EXISTE

**Fichier :** `crates/nexus-coordinator-rs/src/provenance.rs` (212 lignes)

Le systeme de provenance actuel est fonctionnel et couvre SLSA Build L1 :

| Element | Implementation | Evidence |
|---------|---------------|----------|
| `ProvenanceRecord` | struct signe Ed25519 | `provenance.rs:17-29` |
| Canonical bytes | `DOMAIN_PROVENANCE_V1 \|\| 0x00 \|\| JCS(fields)` | `provenance.rs:102-124` |
| Champs signes | schema_version, repo_url, commit_sha, artifact_hash, node_id, timestamp | `provenance.rs:110-117` |
| `generate_provenance()` | cree + signe un record | `provenance.rs:31-58` |
| `verify_provenance()` | reconstruit canonical, verifie Ed25519 | `provenance.rs:60-90` |
| `provenance_blake3_hex()` | hash du JSON pretty-print | `provenance.rs:96-100` |
| app_version | champ optionnel, PAS dans les bytes signes | `provenance.rs:27-28` |
| Tests | 5 tests (generate+verify, wrong key, tamper, blake3 determinism, app_version stability) | `provenance.rs:127-211` |

**Verdict SLSA :** Le projet est a SLSA Build L1 **quand le build tourne localement**. La provenance existe, contient les informations requises (source, commit, artifact hash), et est signee. Mais :

- Le build tourne sur la machine du developpeur (pas un build platform heberge) : empeche L2.
- Pas de separation entre le signataire et le builder : empeche L3.

### 1.2 Deploy verifie — LE FLOW COMPLET

**Fichier :** `crates/nexus-shell-daemon/src/deploy.rs` (753 lignes)

Le pipeline deploy-from-repo est complet :

```
1. Validation repo_url (HTTPS)
2. Validation commit_sha (40 hex) si fourni
3. HEAD request reachability (10s timeout)
4. git clone --depth 1 --single-branch (30s timeout)
5. git fetch + checkout si commit_sha specifie
6. Clone size < 500 MB check
7. SBFB.json → node_id match daemon
8. index.html exists
9. git rev-parse HEAD → commit_sha
10. zip_directory() → zip bytes (exclut .git/, symlinks)
11. BLAKE3(zip) → artifact_hash
12. generate_provenance() → ProvenanceRecord Ed25519
13. contributor attestation (best-effort)
14. provenance.json injecte dans le zip
15. iroh-blobs store → content hash
16. persist provenance in coordinator DB
17. broadcast ProjectAnnouncement via gossip
18. Return { deployed, hash, provenance_hash, commit_sha }
```

**Gaps pour un proof pack :**
- Le provenance.json est injecte dans le zip mais PAS exporte separement
- Pas de checksum SHA256 du binaire (le build utilise BLAKE3 intern mais pas de fichier `.sha256` public)
- Pas de SBOM genere au moment du deploy
- Pas de snapshot de l'etat du feed au moment du deploy

### 1.3 Feed hash-chain publique

**Fichier :** `crates/nexus-coordinator-rs/src/public_feed.rs`
**Spec :** `docs/protocol/PUBLIC_FEED_SPEC.md` (484 lignes, complete §1-12)

Le feed est mature :

| Element | Etat |
|---------|------|
| FeedEntry signe Ed25519 + DOMAIN_FEED_V1 | Fait |
| Hash-chain BLAKE3 per-author | Fait |
| Multi-author SSB-model | Fait |
| verify_entry() + verify_chain() | Fait |
| Rate limiter 5/min/author | Fait |
| PoW 16-bit BLAKE3 | Fait |
| Timestamp guard 30j futur | Fait |
| 15 vecteurs adversariaux documentes (§10) | Fait (S64) |
| New node bootstrap procedure (§11) | Fait |
| Test vectors deterministes (§8) | Fait |
| Replay depuis genesis → PublicRegistryView | Fait |

**Gap pour proof pack :** Il manque un **export snapshot** de l'etat du feed (derniere entry, hash, seq, nombre d'entrees per-author). Le feed est queryable mais pas exportable en un fichier standalone verifiable.

### 1.4 Warrant canary

**Fichier :** `crates/nexus-shell-daemon-core/src/canary/` (mod.rs + frost.rs + dkg.rs + ceremony.rs + duress_ack.rs)

| Element | Etat |
|---------|------|
| CanarySigned signe DOMAIN_WARRANT_CANARY_V1 | Fait |
| FROST K-of-N threshold (RFC 9591) | Fait (K=2/N=3 par defaut) |
| DuressAck heartbeat quotidien | Fait |
| CANARY.txt dans le repo | Fait (date: 2026-04-15) |
| verify-canary.sh script | Fait |
| CI canary-monthly.yml (weekly cron check + freshness) | Fait |

**Gap pour proof pack :** Le CANARY.txt est signe et verifiable, mais le **status du canary** n'est pas inclus dans un bundle de preuves exporte. Le canary est de 33 jours (2026-04-15 → 2026-05-18), il faudra un refresh avant le pilote.

### 1.5 CI pipeline et artefacts

| Workflow | Artefacts produits | Signing |
|----------|-------------------|---------|
| `ci.yml` | Aucun artefact (verification only) | N/A |
| `release.yml` | Binaires 3 OS x 3 crates + SHA256 + in-toto SLSA v1 + cosign sig | OIDC cosign keyless |
| `build-worker.yml` | Binaire worker 7 targets | Aucun signing |
| `supply-chain.yml` | Rapports cargo-deny + pip-audit + audit-ci | N/A |
| `canary-monthly.yml` | Verification signature + freshness | N/A |
| `mirror-codeberg.yml` | Mirror push Codeberg | N/A |

**`release-attest.sh`** est le script cle. Il produit :
- `<binary>-<os>-<arch>[.exe]` — l'artefact
- `<binary>-<os>-<arch>[.exe].sha256` — checksum SHA256
- `<binary>-<os>-<arch>[.exe].intoto.jsonl` — attestation SLSA v1 (in-toto Statement)
- `<binary>-<os>-<arch>[.exe].intoto.jsonl.sig` — signature cosign (si COSIGN_EXPERIMENTAL=1)

Le script pin `SOURCE_DATE_EPOCH` au timestamp du commit pour des builds reproductibles.

**Gaps majeurs :**
- Pas de SBOM genere dans le pipeline release
- Pas de cargo-deny output inclus dans le release
- Pas de verification de reproductibilite (diffoscope)
- Le `build-worker.yml` ne signe PAS les binaires (seul `release.yml` signe)
- GitHub Attestation API pas encore utilisee
- Pas de transparency log public (Rekor entry non verifiable publiquement post-release)

### 1.6 Git tag signing

Le CLAUDE.md mentionne "tag v1.0 pose localement, pas encore pousse vers origin". Le tag existe mais il n'y a **aucune evidence de signature GPG/SSH du tag** dans le workflow. Les tags sont des annotated tags simples, pas des signed tags.

### 1.7 Synthese des gaps

| Gap | Impact | Priorite proof pack |
|-----|--------|---------------------|
| Pas d'export feed snapshot standalone | Un tiers ne peut pas verifier l'etat du feed hors connexion | HAUTE |
| Pas de SBOM | Pas de visibilite supply chain pour un verifieur externe | HAUTE |
| Pas de signed git tag | Faible confiance dans l'authenticite du tag | MOYENNE |
| Pas de verification reproductibilite | Impossible de prouver que le binaire vient du source | MOYENNE |
| Build local = pas SLSA L2 | La provenance est auto-attestee, pas attestee par un CI heberge | HAUTE |
| Canary a 33 jours | Valide mais en retard, refresh necessaire avant pilote | HAUTE |
| Pas de proof pack CLI tool | Aucun outil pour generer/verifier un bundle complet | HAUTE |

---

## 2. Recherche externe

### 2.1 SLSA — Niveaux et progression

**Confiance : HIGH** (documentation officielle slsa.dev)

| Niveau | Exigences | SBFB actuel |
|--------|-----------|-------------|
| L1 | Provenance existe (qui, quoi, quand) | OUI — ProvenanceRecord Ed25519 |
| L2 | Build heberge + provenance signee par la plateforme | PARTIEL — release.yml signe via cosign OIDC mais deploy-from-repo est local |
| L3 | Builds isoles + cles inaccessibles aux etapes user | NON — pas de separation signer/builder |

**SLSA v1.1** (version courante avril 2025) distingue Build Track et Source Track. SBFB a interet a documenter son niveau par track :
- **Build Track L2** est atteignable via CI : le `release.yml` avec cosign OIDC est deja presque L2 (il signe, il tourne sur GHA). Il manque la verification que les attestations sont signees par GHA et pas par le dev.
- **Source Track** n'a pas encore de niveaux formels dans SLSA v1.1, mais SBFB couvre deja l'essentiel (repo public, commits signes par Keyoxide, deploy-from-repo).

**Recommandation :** Documenter le niveau SLSA comme "Build L1 (deploys locaux), Build L2 (releases CI via release.yml + cosign)". Ne pas pretendre L3.

### 2.2 Sigstore / cosign / Rekor

**Confiance : MEDIUM-HIGH** (deja partiellement integre)

SBFB utilise deja cosign dans `release-attest.sh` avec OIDC keyless. Ce qui manque :

| Element | Etat SBFB | Valeur ajoutee |
|---------|-----------|----------------|
| Signature cosign OIDC | FAIT (release.yml) | Lie le build a l'identite GHA |
| Rekor transparency log | NON EXPLICITE | Chaque signature cosign est automatiquement loguee dans Rekor si cosign OIDC est utilise |
| Verification Rekor entry | PAS DE TOOLING | Un verifieur externe devrait pouvoir chercher l'entry Rekor par artifact hash |
| `cosign verify-blob` instructions | MANQUANT | Pas de doc pour un tiers qui veut verifier |

**Rekor v2** (GA 2025) integre le witnessing directement et est plus scalable. Les entries existantes de SBFB (si le tag a ete push et le release workflow a tourne) sont dans le Rekor public.

**Recommandation :** Ajouter au proof pack :
1. Le Rekor entry UUID pour chaque artefact signe
2. Un script `verify-release.sh` qui fait `cosign verify-blob`
3. Les instructions pour chercher dans le transparency log

### 2.3 in-toto

**Confiance : HIGH** (deja utilise dans release-attest.sh)

Le `release-attest.sh` genere deja des attestations in-toto Statement v1 au format SLSA Provenance v1. La structure est :

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{"name": "...", "digest": {"sha256": "..."}}],
  "predicateType": "https://slsa.dev/provenance/v1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://github.com/SBFB50/SBFB/build-types/release-attest-bash@v1",
      "externalParameters": {"binary": "...", "os": "...", "arch": "..."},
      "internalParameters": {"SOURCE_DATE_EPOCH": "...", "profile": "release"},
      "resolvedDependencies": [{"uri": "git+...", "digest": {"sha1": "..."}}, {"name": "Cargo.lock", "digest": {"sha256": "..."}}]
    },
    "runDetails": {"builder": {"id": "..."}, "metadata": {"invocationId": "...", "startedOn": "..."}}
  }
}
```

**Ce qui manque pour un proof pack complet :**
- Layout in-toto (qui definit les etapes autorisees et les fonctionnaires) — PAS necessaire pour SLSA L1/L2 mais utile pour la credibilite
- Link metadata pour chaque etape du pipeline (clone, build, test, sign) — OVER-ENGINEERING pour 2-3 testeurs

**Recommandation :** Ne PAS implementer de layout in-toto complet. Le format Statement v1 + SLSA Provenance v1 est suffisant. Concentrer l'effort sur la verification plutot que sur plus de metadata.

### 2.4 Reproducible builds

**Confiance : MEDIUM** (partiellement implemente)

`release-attest.sh` pin `SOURCE_DATE_EPOCH` au timestamp du commit. C'est la base de la reproductibilite Rust. Mais :

| Aspect | Etat |
|--------|------|
| SOURCE_DATE_EPOCH | FAIT |
| Cargo.lock commit | FAIT |
| codegen-units=1, lto=fat, strip=symbols | FAIT (dans les internalParameters de l'attestation) |
| Verification diffoscope | PAS FAIT |
| rebuilderd integration | PAS FAIT |
| Multi-build comparison | PAS FAIT |

**Realite Rust :** La reproductibilite bit-a-bit est difficile en Rust a cause des chemins absolus embeds dans les debug info, des timestamps LLVM, et des variations de version de linker. `strip=symbols` aide beaucoup mais ne garantit pas 100%.

**Recommandation pour S68 :** Ajouter un test de reproductibilite **best-effort** : deux builds consecutifs dans le CI, comparer les SHA256. Si different, logger les differences mais ne pas bloquer. Documenter l'ecart. Ne PAS pretendre "reproductible" si ce n'est pas bit-a-bit.

### 2.5 SBOM (Software Bill of Materials)

**Confiance : HIGH** (ecosysteme Rust mature)

Deux outils viables pour Rust :

| Outil | Format | Maturite |
|-------|--------|----------|
| `cargo-sbom` | SPDX + CycloneDX 1.4/1.5/1.6 | 0.10.0, actif |
| `cargo-cyclonedx` | CycloneDX | Maintenu par OWASP, recovery apres abandon |

**Recommandation :** Utiliser `cargo-sbom` pour generer un CycloneDX 1.6 JSON. L'inclure dans le proof pack. Le workflow CI peut generer et attacher le SBOM au GitHub Release.

Commande :
```bash
cargo install cargo-sbom --locked
cargo sbom --output-format cyclonedx-json > sbom.cdx.json
```

### 2.6 GitHub Artifact Attestation API

**Confiance : MEDIUM** (API GA depuis 2025)

GitHub Artifact Attestations utilise Sigstore sous le capot et stocke les attestations dans le registre GitHub. Disponible pour les repos publics sur tous les plans.

**Avantage :** `gh attestation verify` permet a un tiers de verifier sans installer cosign.

**Recommandation :** Ajouter `actions/attest-build-provenance@v2` dans `release.yml` apres le build. Cela double la provenance (cosign + GitHub Attestation) sans effort significatif.

### 2.7 Debian release process — inspiration

**Confiance : HIGH** (pratique etablie)

Le modele Debian pour les fichiers Release :
```
Release (signe GPG)
  └── Packages (checksums SHA256 de chaque .deb)
        └── .deb individuel
```

**Transposition SBFB :** Le proof pack est l'equivalent du fichier `Release` signe — un document unique qui reference tous les artefacts avec leurs checksums, lie a une identite cryptographique.

---

## 3. Proposition de structure proof pack SBFB

### 3.1 Fichier racine : `proof-pack-v<version>.json`

```json
{
  "schema_version": 1,
  "sbfb_version": "1.0.0",
  "created_at": "2026-05-XX T..Z",
  "creator_node_id": "<Ed25519 hex>",

  "git": {
    "repo_url": "https://github.com/SBFB50/SBFB",
    "commit_sha": "<40 hex>",
    "tag": "v1.0.0",
    "tag_signed": true,
    "mirror_urls": [
      "https://codeberg.org/SBFB/SBFB"
    ]
  },

  "artifacts": [
    {
      "name": "nexus-launcher-linux-x86_64",
      "blake3": "<64 hex>",
      "sha256": "<64 hex>",
      "size_bytes": 12345678,
      "slsa_attestation": "nexus-launcher-linux-x86_64.intoto.jsonl",
      "cosign_signature": "nexus-launcher-linux-x86_64.intoto.jsonl.sig",
      "rekor_entry_uuid": "<uuid>"
    }
  ],

  "provenance": {
    "schema_version": 1,
    "method": "deploy-from-repo",
    "slsa_level": "L1 (local) / L2 (CI)",
    "domain_separation": "nexus-provenance-v1"
  },

  "feed_snapshot": {
    "total_entries": 42,
    "last_seq": 42,
    "last_entry_hash": "<BLAKE3 hex>",
    "authors_count": 3,
    "genesis_hash": "genesis",
    "snapshot_at": "2026-05-XX T..Z"
  },

  "canary": {
    "date": "2026-05-XX",
    "next_update": "2026-06-XX",
    "status": "valid",
    "headline": "...",
    "pubkey_hex": "<Ed25519 hex>",
    "signature_hex": "<Ed25519 hex>",
    "signing_model": "FROST K=2/N=3"
  },

  "supply_chain": {
    "cargo_deny_clean": true,
    "npm_audit_clean": true,
    "sbom_file": "sbom.cdx.json",
    "sbom_format": "CycloneDX 1.6",
    "cargo_lock_sha256": "<hex>",
    "package_lock_sha256": "<hex>"
  },

  "test_results": {
    "rust_tests": 1326,
    "vitest_tests": 265,
    "size_limit_checks": 6,
    "all_green": true,
    "ci_run_id": "<GHA run id>",
    "ci_run_url": "https://github.com/SBFB50/SBFB/actions/runs/..."
  },

  "signature": "<Ed25519 hex over canonical bytes of this JSON>"
}
```

### 3.2 Arborescence du proof pack

```
proof-pack-v1.0.0/
  proof-pack-v1.0.0.json           # Fichier racine signe
  proof-pack-v1.0.0.json.sig       # Signature detachee (Ed25519)
  sbom.cdx.json                    # CycloneDX SBOM
  CANARY.txt                       # Copie du warrant canary
  cargo-deny-report.txt            # Sortie cargo-deny (audit clean)
  feed-snapshot.json               # Export de l'etat du feed
  artifacts/
    nexus-launcher-linux-x86_64           # Binaire
    nexus-launcher-linux-x86_64.sha256    # Checksum
    nexus-launcher-linux-x86_64.intoto.jsonl  # SLSA attestation
    nexus-launcher-linux-x86_64.intoto.jsonl.sig  # cosign sig
    nexus-launcher-windows-x86_64.exe
    nexus-launcher-windows-x86_64.exe.sha256
    ...
  verify.sh                        # Script de verification autonome
```

### 3.3 Script `verify.sh` — verification autonome

Le script doit etre executable par un tiers avec uniquement :
- `bash`, `jq`, `sha256sum`
- Le proof pack decompresse
- Optionnel : `cosign` pour la verification Sigstore

```bash
#!/bin/bash
# Etapes :
# 1. Verifier les checksums SHA256 de chaque artefact
# 2. Verifier la coherence BLAKE3 (si blake3 CLI disponible)
# 3. Verifier la signature Ed25519 du proof-pack JSON (via nexus-shell-daemon canary verify pattern)
# 4. Verifier les attestations in-toto (structure JSON valide)
# 5. Verifier le CANARY.txt (signature + freshness)
# 6. Optionnel : cosign verify-blob pour les sigs Sigstore
# 7. Afficher le resume
```

### 3.4 CLI `sbfb proof-pack generate`

Sous-commande du daemon ou du launcher :

```
nexus-shell-daemon proof-pack generate \
  --version 1.0.0 \
  --artifacts-dir dist/ \
  --output proof-pack-v1.0.0/
```

Cette commande :
1. Lit l'etat du feed local (snapshot)
2. Lit le CANARY.txt du repo
3. Copie les artefacts du dist/
4. Genere le SBOM via `cargo-sbom`
5. Execute `cargo-deny check` et capture le rapport
6. Genere le JSON racine
7. Signe le JSON avec la cle du noeud
8. Ecrit tout dans le dossier output

### 3.5 CLI `sbfb proof-pack verify`

```
nexus-shell-daemon proof-pack verify \
  --input proof-pack-v1.0.0/ \
  --pubkey <hex>
```

Verification complete :
1. Signature du JSON racine
2. Checksums de chaque artefact
3. Canary freshness (< 45 jours)
4. Coherence feed snapshot (dernier hash verifiable si le noeud est connecte)
5. SBOM present et parseable
6. Rapport textuel passe/echoue

---

## 4. Etat actuel de l'installation / onboarding

### 4.1 Launcher

**Fichier :** `crates/nexus-launcher/src/main.rs` (684 lignes)

Le launcher est mature :

| Feature | Etat |
|---------|------|
| Spawn nexus-shell-daemon start | Fait |
| Wait for running.json (15s timeout) | Fait |
| Open browser automatique | Fait |
| Tray icon (muda + tray-icon) | Fait |
| Token rotation (24h cycle) | Fait |
| Auth server | Fait |
| Driver check NVIDIA (CVE) | Fait |
| Unlock/init subcommands (identity encryption) | Fait |
| web-root resolution (env, bundled, dev) | Fait |
| Stale running.json detection + cleanup | Fait |
| Error msgbox Windows | Fait |

### 4.2 Installeur cross-platform

**Fichier :** `Packager.toml` + `scripts/build-installer.sh`

| Plateforme | Format | Config |
|------------|--------|--------|
| Windows | NSIS .exe (currentUser, EN+FR) | `[nsis]` dans Packager.toml |
| Linux | .deb + .AppImage | `[deb]` + `[appimage]` |
| macOS | .dmg | `[dmg]` avec positions fenetre |

**Binaires empaquetes :** `nexus-launcher` (main) + `nexus-shell-daemon`
**Ressources :** `web/dist` → copie dans `web/` de l'installeur

**Gap majeur :** L'installeur est configure mais **jamais teste en conditions reelles par un tiers**. Le build-installer.sh est un script qui tourne, mais personne n'a jamais installe SBFB sur une machine qui n'est pas celle du dev.

### 4.3 Onboarding page

**Fichier :** `web/src/pages/OnboardingEmpty.tsx` (111 lignes)

La page est **obsolete** — elle reference l'ancien modele pre-launcher :
- Montre des commandes `uv run --package nexus-coordinator` (Python, pre-pivot)
- Dialogue "Ajouter un coordinateur" via URL (ancien modele multi-coordinateur)
- Ne correspond plus a l'architecture actuelle (launcher → daemon → browser auto)

**Gap critique pour le pilote :** Un nouveau testeur qui lance l'installeur ne verra probablement PAS cette page (le daemon demarre automatiquement et le browser s'ouvre). Mais si le daemon ne demarre pas, il n'y a aucune page d'aide utile.

### 4.4 Join ticket mechanism

**Fichier :** `crates/nexus-coordinator-rs/src/invite.rs` (236 lignes)

Le systeme d'invite existe :
- `InviteLedger::mint()` — cree un invite avec scope (worker/observer), expiry, max_uses
- `InviteLedger::revoke()` — revoque
- `InviteLedger::get/list()` — consultation
- `tasks_doc_ticket` — optionnel, lien vers le document de taches

**Gap :** Le systeme invite est une brique DB, mais il n'y a **pas d'endpoint HTTP expose** pour distribuer un invite a un nouveau testeur. Le flow "invite un ami" n'existe pas dans le frontend. Le join de feed se fait via `POST /api/daemon/feed/join` avec un ticket iroh-docs obtenu manuellement.

### 4.5 Premier lancement — flow actuel

```
1. Utilisateur installe via NSIS / .deb / .dmg
2. Lance nexus-launcher
3. Launcher genere auth_token dans ~/.sbfb/auth_token
4. Launcher spawn nexus-shell-daemon start
5. Daemon boot: iroh endpoint, SQLite, HTTP server
6. Daemon ecrit running.json
7. Launcher detecte running.json, ouvre le browser
8. Browser → http://127.0.0.1:<port>
9. Shell React affiche Browse (vide si pas de peers)
10. ??? (pas de guide pour connecter a un peer)
```

**Gap critique :** L'etape 10 est un mur. Un nouveau noeud n'a aucun moyen de decouvrir des peers sans un ticket de feed ou une adresse de noeud. Il n'y a pas de "seed node" pre-configure, pas d'URL de bootstrap, pas d'ecran "Entrez le ticket de votre ami".

---

## 5. Recherche externe — Pilotes fermes

### 5.1 Patterns de pilotes fermes

**Confiance : MEDIUM** (best practices generiques, pas specifique P2P)

| Pattern | Source | Applicable SBFB |
|---------|--------|-----------------|
| **TestFlight (Apple)** | Invite par email, 10k testeurs max, crash reports auto | Partiellement — SBFB n'a pas d'infrastructure centralisee |
| **Firefox Test Pilot** | Opt-in, 100k+ participants, A/B testing, feature flags | Non applicable — SBFB n'a pas de telemetrie |
| **Signal Beta** | APK/TestFlight distribution, debug logs manuels, forum feedback | Bon modele — distribution manuelle, feedback textuel |
| **Tailscale early access** | Invite-only, pilot select customers, puis public beta | Bon modele — petit groupe, progressif |

### 5.2 Modele recommande pour SBFB

Le pilote SBFB doit respecter les contraintes du projet :
- **Zero telemetrie** (pas de crash reporting centralise, pas de analytics)
- **Zero cloud** (pas de Google Forms, pas de Sentry)
- **Coherence philosophique** (le feedback utilise le reseau SBFB lui-meme)

**Modele propose : "Pilote par tickets"**

```
1. Mainteneur genere N invites (2-3)
2. Envoie par canal securise (Signal, email chiffre)
3. Invite contient : installeur + ticket feed + guide PDF
4. Testeur installe, entre le ticket, rejoint le reseau
5. Feedback via app Ideas Hub deployee sur le reseau
6. Crash logs locaux (launcher.log, daemon logs dans ~/.sbfb/logs/)
7. Testeur partage logs par email si probleme
```

### 5.3 Crash reporting sans telemetrie

**Confiance : MEDIUM**

Pas besoin d'un Sentry/GlitchTip pour 2-3 testeurs. Les mecanismes existants suffisent :

| Mecanisme | Etat SBFB |
|-----------|-----------|
| `launcher.log` (JSON, rolling daily) | FAIT |
| `launcher-panic.log` (panic hook) | FAIT |
| Daemon structured logs (JSON, tracing-appender) | FAIT |
| SecurityEvent JSONL audit trail | FAIT |
| ETW events (Windows) | FAIT |

**Recommandation :** Ajouter un bouton "Exporter les logs" dans le tray menu ou le frontend. Ce bouton compresse les logs des 7 derniers jours dans un zip que le testeur peut envoyer par email. Pas de telemetrie, pas de upload automatique.

### 5.4 Criteres go/no-go

Inspires du modele Tailscale (pilot → public beta → GA) :

| Critere | Go | No-Go |
|---------|-----|--------|
| Installation | 2/3 testeurs installent sans aide | 0/3 reussit ou 2/3 ont besoin d'aide |
| Premier lancement | Daemon demarre et browser s'ouvre en < 30s | Crash au demarrage ou browser ne s'ouvre pas |
| Connexion P2P | 2 noeuds se voient en < 5 min | Aucun noeud ne se connecte apres 15 min |
| Deploy app | 1 testeur deploie Protocol Explorer depuis source | Deploy echoue ou provenance invalide |
| Feed sync | Feed synchronise entre 2+ noeuds | Divergence ou corruption |
| Restart clean | Daemon redemarrage propre apres kill | Crash, state corrompu, ou impossibilite de redemarrer |
| Stabilite 24h | Daemon tourne 24h sans crash ni memory leak > 2x | Crash, OOM, ou freeze |

---

## 6. Checklist pilote SBFB

### 6.1 Prerequisites techniques

| # | Prerequis | Sprint | Etat |
|---|-----------|--------|------|
| 1 | Proof pack generable et verifiable | S68 | A FAIRE |
| 2 | Canary fresh (< 30 jours) | S68 | STALE (33j) |
| 3 | Tag v1.0 pousse + signe | Pre-S68 | TAG LOCAL SEULEMENT |
| 4 | Installeur teste sur machine propre (VM) | S69 | A FAIRE |
| 5 | Page onboarding mise a jour | S69 | A FAIRE (obsolete) |
| 6 | Mecanisme join ticket fonctionnel E2E | S69 | PARTIEL (DB OK, pas d'endpoint HTTP, pas d'UI) |
| 7 | Bouton "Exporter logs" | S69 | A FAIRE |
| 8 | Documentation testeur (PDF/MD) | S69 | A FAIRE |
| 9 | CI vert sur master | Continu | Oui |
| 10 | Feed auth-tier check (P2-FEED-INSERT-NO-AUTH-TIER) | S65+ | CARRY |

### 6.2 Matrice de test

| Scenario | Windows 11 | Ubuntu 24.04 | macOS 14+ |
|----------|------------|--------------|-----------|
| Installation NSIS/.deb/.dmg | Testeur A | Testeur B | Testeur C (si dispo) |
| Premier lancement | X | X | X |
| Join feed via ticket | X | X | - |
| Browse apps (voit Protocol Explorer) | X | X | - |
| Deploy-from-repo app perso | X | - | - |
| Feed sync bidirectionnel | X+B | B+A | - |
| Restart daemon | X | X | - |
| Stabilite 24h | X | X | - |
| Desinstallation propre | X | X | - |

**Profils testeurs :**
- **Testeur A :** Dev Python/JS, a l'aise CLI, Windows (le profil cible early adopter)
- **Testeur B :** Sysadmin Linux, serveur distant (VPS), teste la reachability WAN
- **Testeur C :** (optionnel) Non-technique, macOS, teste le "est-ce que ca marche juste en cliquant"

### 6.3 Mecanisme de feedback

**Canal principal :** App Ideas Hub deployee sur le reseau SBFB

Avantages :
- Dogfooding — le feedback utilise le produit
- Zero dependance cloud
- Les testeurs voient les idees des autres et votent

**Canal secondaire :** Email chiffre ou Signal groupe prive

Pour les problemes urgents (crash au demarrage, impossible de se connecter) ou l'envoi de logs.

**Format de rapport de bug :**
```
## Titre
## Etapes pour reproduire
1. ...
2. ...
3. ...
## Comportement attendu
## Comportement observe
## Logs (joindre le zip exporte)
## Environnement (OS, version, derriere NAT/VPN ?)
```

### 6.4 Plan de communication

**Ce qu'on promet :**
- Version pre-release non definitive
- Bugs attendus et normaux
- Feedback pris en compte
- Confidentialite : pas de partage des logs sans consentement
- Le testeur peut quitter a tout moment

**Ce qu'on NE promet PAS :**
- Stabilite
- Retrocompatibilite des donnees entre versions
- Support 24/7
- Que le produit soit "fini"

---

## 7. Dependances S68 vers S69

```
S68 (Proof Pack)                     S69 (Pilote Ferme)
================                     ==================
                                     
A. Proof pack structure              Prerequis: proof pack existe
   + CLI generate                    → Testeurs recoivent le proof pack
                                       comme piece de confiance
                                     
B. Attestation build CI              Prerequis: release workflow produit
   (SBOM + GitHub Attestation)       des artefacts signes verifiables
                                     → Installeurs distribues aux testeurs
                                     
C. Feed snapshot + canary            Prerequis: canary fresh + feed
   refresh                           verifiable
                                     → Testeurs peuvent verifier le feed
                                     
D. Proof pack verification           Prerequis: verify.sh + CLI verify
   tool                              → Testeur B (sysadmin) verifie
                                       independamment
```

**Dependances strictes :**
- S69 Phase A (checklist) depend de S68 Phase A (structure proof pack)
- S69 Phase B (installeur teste) depend de S68 Phase B (artefacts signes dans CI)
- S69 Phase C (feedback) ne depend PAS de S68 (peut etre prepare en parallele)

**Dependances carry S65+ :**
- **P2-FEED-INSERT-NO-AUTH-TIER** (MANDATORY) : doit etre fait AVANT le pilote. Un testeur malveillant pourrait injecter des entries de feed sans verification.
- **P2-FEED-JOIN-HANDLE-LEAK** : fuite de JoinHandle au feed_join. Risque de memory leak apres 24h de fonctionnement.

---

## 8. Plans de phases

### 8.1 S68 — Pack De Preuves Release (4 phases A-D)

#### Phase A : Structure proof pack + CLI generate

**Objectif :** Un dossier proof-pack/ generable par commande CLI.

Contenu :
1. Definir le schema `ProofPackManifest` (Rust struct, serde JSON)
2. Implementer `sbfb proof-pack generate` dans nexus-shell-daemon
   - Lit feed state (last_seq, last_entry_hash, authors_count)
   - Copie CANARY.txt
   - Copie artefacts du dist/
   - Genere le JSON racine
   - Signe avec la cle du noeud (DOMAIN a definir, ex: `DOMAIN_PROOF_PACK_V1`)
3. Tests : generation, signature, round-trip parse
4. **Feed snapshot export** : endpoint `GET /api/daemon/feed/snapshot` qui retourne le JSON du snapshot

**Estimation :** 1 phase standard

#### Phase B : Attestation build CI + SBOM

**Objectif :** Le pipeline release produit des artefacts avec attestation GitHub + SBOM.

Contenu :
1. Ajouter `cargo-sbom` au CI : generer `sbom.cdx.json`
2. Ajouter `actions/attest-build-provenance@v2` dans `release.yml`
3. Capturer la sortie `cargo-deny check` dans un fichier rapport
4. Documenter le Rekor entry UUID dans le release notes
5. Signer le tag git avec SSH key (pas GPG — plus simple, compatible GitHub)

**Estimation :** 1 phase standard

#### Phase C : Feed snapshot + canary refresh

**Objectif :** Le proof pack contient un feed verifiable et un canary fresh.

Contenu :
1. Publier un nouveau CANARY.txt (maintainer signe manuellement)
2. Implementer `feed-snapshot.json` : export complet de l'etat du feed
   - Toutes les entries OU resume (total, last seq, per-author stats)
   - Signature du snapshot par le noeud
3. Verifier que `verify_chain()` fonctionne sur le snapshot exporte
4. Test E2E : generer proof pack → verifier → assertion OK

**Estimation :** 1 phase standard

#### Phase D : Outil de verification externe

**Objectif :** Un script `verify.sh` et une sous-commande CLI que n'importe qui peut utiliser.

Contenu :
1. `scripts/verify-proof-pack.sh` — bash portable
   - Verifie checksums SHA256
   - Verifie signature du manifest JSON
   - Verifie freshness canary (< 45j)
   - Optionnel : `cosign verify-blob` si cosign installe
2. `sbfb proof-pack verify --input <dir> --pubkey <hex>`
   - Verification complete en Rust
   - Rapport textuel passe/echoue
3. Documentation : `docs/release/PROOF_PACK.md`
   - Structure du proof pack
   - Comment verifier (3 methodes : script, CLI, manuellement)
   - Interpretation des resultats

**Estimation :** 1 phase standard

### 8.2 S69 — Pilote Ferme (5 phases A-E)

#### Phase A : Checklist prerequisites + invite mechanism

**Objectif :** Tout est pret pour inviter des testeurs.

Contenu :
1. Verifier que tous les carry P2 critiques sont resolus (ou explicitement acceptes)
2. Implementer l'endpoint HTTP pour distribuer un ticket de feed
   - `POST /api/v1/pilot/invite` → genere un invite token + feed ticket
   - `GET /api/v1/pilot/join?token=<token>` → rejoint le feed
3. Mettre a jour la page OnboardingEmpty :
   - Enlever les commandes Python obsoletes
   - Ajouter "Entrez votre ticket d'invitation"
   - Ajouter un indicateur de connexion P2P
4. Script de generation des invites pour le mainteneur

**Estimation :** 1 phase lourde (beaucoup de surface)

#### Phase B : Installeur cross-platform teste

**Objectif :** Les installeurs fonctionnent sur des machines propres.

Contenu :
1. Tester l'installeur Windows NSIS dans une VM Windows 11 propre
   - Fresh install, aucun prerequis pre-installe
   - Verifier que le launcher demarre, le daemon boot, le browser s'ouvre
2. Tester le .deb sur Ubuntu 24.04 LTS VM
3. Tester le .dmg sur macOS (si machine dispo, sinon accepter le gap)
4. Fix les bugs d'installation trouves
5. Documenter les prerequisites systeme (versions OS minimum, ports necessaires)

**Estimation :** 1 phase (beaucoup de tests manuels + fixes)

#### Phase C : Feedback collector integre

**Objectif :** Les testeurs peuvent reporter des problemes via le reseau SBFB.

Contenu :
1. Deployer Ideas Hub sur le reseau comme app "Pilot Feedback"
2. Ajouter un bouton "Exporter les logs" dans le tray menu
   - Compresse les 7 derniers jours de logs dans un zip
   - Ouvre le file picker pour sauvegarder
3. Ajouter un bouton "Rapport de bug" dans le frontend
   - Formulaire structure (titre, etapes, comportement attendu/observe)
   - Envoie via `bridge.setStorage()` dans l'app feedback
4. Documentation testeur : guide PDF/MD "Comment participer au pilote"

**Estimation :** 1 phase standard

#### Phase D : Scenarios de test guides

**Objectif :** Chaque testeur a un parcours structure a suivre.

Contenu :
1. Document "Scenarios de test pilote" avec checklist :
   - Scenario 1 : Installation + premier lancement (10 min)
   - Scenario 2 : Rejoindre le reseau via ticket (10 min)
   - Scenario 3 : Naviguer Browse, voir Protocol Explorer (5 min)
   - Scenario 4 : Deploy-from-repo d'un hello-world (15 min)
   - Scenario 5 : Feed sync — observer les entries d'un autre noeud (10 min)
   - Scenario 6 : Verifier la provenance d'une app (5 min)
   - Scenario 7 : Restart daemon + verifier que l'etat persiste (5 min)
   - Scenario 8 : Laisser tourner 24h (passif)
2. Formulaire de resultat par scenario (passe/echoue/commentaire)
3. Envoyer le document aux testeurs avant le debut du pilote

**Estimation :** 1 phase legere (doc principalement)

#### Phase E : Analyse go/no-go

**Objectif :** Decision honnete basee sur les resultats du pilote.

Contenu :
1. Collecter tous les retours (Ideas Hub + emails)
2. Categoriser : bugs critiques / UX / cosmetics / suggestions
3. Matrice go/no-go (voir section 5.4)
4. Document "Bilan pilote" :
   - Ce qui a fonctionne
   - Ce qui a casse
   - Bugs a corriger avant go-live
   - Decision : go / go-with-fixes / no-go
5. Si go : planifier le push du tag + annonce publique (S70+)
6. Si no-go : lister les sprints necessaires avant retry

**Estimation :** 1 phase legere (analyse + decision)

---

## 9. Risques et mitigations

### 9.1 Risques S68

| Risque | Probabilite | Impact | Mitigation |
|--------|-------------|--------|------------|
| SBOM generation echoue (deps complexes workspace) | MEDIUM | LOW | Fallback : cargo-deny output seul, pas de SBOM |
| Reproductibilite bit-a-bit impossible (Rust) | HIGH | LOW | Documenter comme best-effort, ne pas pretendre |
| Canary refresh oublie | LOW | HIGH | Mettre en Phase C, verifier dans la checklist |
| GitHub Attestation API change | LOW | LOW | Cosign suffit comme backup |

### 9.2 Risques S69

| Risque | Probabilite | IMPACT | Mitigation |
|--------|-------------|--------|------------|
| Testeur ne reussit pas a installer | MEDIUM | HIGH | Phase B teste sur VM propre avant envoi |
| NAT / firewall bloque P2P | HIGH | HIGH | Relais iroh devrait contourner, documenter les workarounds |
| Aucun peer visible apres join | MEDIUM | HIGH | Pre-deployer un seed node permanent (VPS Helsinki) |
| Daemon crash apres 24h | MEDIUM | MEDIUM | Monitor avec le P2-FEED-JOIN-HANDLE-LEAK fix |
| Testeur ne donne pas de feedback | MEDIUM | MEDIUM | Guide structure + rappel apres 1 semaine |
| Carries P2 non resolus cassent le pilote | MEDIUM | HIGH | Triage strict pre-pilote : lister les must-fix |

### 9.3 Risque inter-sprints

Le plus grand risque est de **commencer S69 sans que S68 soit complete**. Le proof pack est la piece de confiance qui justifie que SBFB est serieux. Sans proof pack, le pilote ressemble a "installe ce truc et fais confiance".

---

## 10. Sources et confiance

### Sources primaires (code)

| Source | Confiance |
|--------|-----------|
| `provenance.rs` (212 lignes) | HIGH — lu exhaustivement |
| `deploy.rs` (753 lignes) | HIGH — lu (100 premieres lignes, structure claire) |
| `public_feed.rs` + PUBLIC_FEED_SPEC.md | HIGH — lu exhaustivement |
| `canary/` (mod + frost + dkg + ceremony + duress_ack) | HIGH — reference cartographie |
| `release-attest.sh` (136 lignes) | HIGH — lu exhaustivement |
| `release.yml`, `ci.yml`, `supply-chain.yml`, `canary-monthly.yml` | HIGH — lu exhaustivement |
| `main.rs` launcher (684 lignes) | HIGH — lu exhaustivement |
| `Packager.toml` + `build-installer.sh` | HIGH — lu exhaustivement |
| `invite.rs` (236 lignes) | HIGH — lu exhaustivement |
| `OnboardingEmpty.tsx` (111 lignes) | HIGH — lu exhaustivement |

### Sources externes

| Source | Confiance | URL |
|--------|-----------|-----|
| SLSA v1.1 spec | HIGH | [slsa.dev/spec/v1.1/levels](https://slsa.dev/spec/v1.1/levels) |
| Sigstore/cosign documentation | HIGH | [docs.sigstore.dev](https://docs.sigstore.dev/cosign/signing/overview/) |
| Rekor transparency log | HIGH | [docs.sigstore.dev/logging/overview/](https://docs.sigstore.dev/logging/overview/) |
| GitHub Artifact Attestations | MEDIUM-HIGH | [docs.github.com/actions/security-for-github-actions](https://docs.github.com/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds) |
| in-toto specification | HIGH | [github.com/in-toto/specification](https://github.com/in-toto/docs/blob/master/in-toto-spec.md) |
| cargo-sbom | MEDIUM | [crates.io/crates/cargo-sbom](https://crates.io/crates/cargo-sbom) |
| CycloneDX Rust Cargo | MEDIUM | [github.com/CycloneDX/cyclonedx-rust-cargo](https://github.com/CycloneDX/cyclonedx-rust-cargo) |
| Reproducible builds (rebuilderd) | MEDIUM | [reproducible-builds.org](https://reproducible-builds.org/) |
| Rekor v2 GA | MEDIUM | [blog.sigstore.dev/rekor-v2-ga/](https://blog.sigstore.dev/rekor-v2-ga/) |
| Beta testing best practices | LOW-MEDIUM | [userpilot.com/blog/beta-testing-feedback-form](https://userpilot.com/blog/beta-testing-feedback-form-template-best-practices-and-examples/) |
| Signal beta process | LOW | [support.signal.org/hc/en-us/articles/360007318471-Signal-Beta](https://support.signal.org/hc/en-us/articles/360007318471-Signal-Beta) |
| Tailscale release stages | MEDIUM | [tailscale.com/kb/1167/release-stages](https://tailscale.com/kb/1167/release-stages) |

### Confiance par domaine

| Domaine | Confiance | Justification |
|---------|-----------|---------------|
| Proof pack structure | HIGH | Base sur code existant + standards SLSA/in-toto |
| SLSA level assessment | HIGH | Spec officielle + code verifie |
| SBOM generation | MEDIUM | Pas teste dans ce workspace, outils tiers |
| Installeur cross-platform | MEDIUM | Config existe, jamais teste par un tiers |
| Pilote patterns | MEDIUM | Best practices generiques, pas specifiques P2P |
| Crash reporting | HIGH | Mecanismes existants suffisants |
| Go/no-go criteria | MEDIUM | Adaptes de Tailscale/Firefox, a calibrer |
