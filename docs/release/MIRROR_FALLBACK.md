# Mirror fallback — code source redundancy

**Ecrit** : Sprint 18 Phase E3 (2026-04-15)
**Scope** : disaster-recovery mirror du repo GitHub source vers
Codeberg (Forgejo). Pattern prepare pour activation Radicle au
go-live v1.0.

---

## 1. Rationale

Si `github.com/SBFB50/SBFB` devient inaccessible (account
suspension, DMCA takedown, subpoena seizure, service outage,
rate limit trigger), le code source reste accessible via un
mirror Codeberg push-synchronise automatiquement a chaque push
GitHub.

> **Notes de cohabitation** :
>
> - **Orgs distincts** : org GitHub = `SBFB50`, org Codeberg =
>   `SBFB` (namespace `SBFB` etait disponible sur Codeberg sans
>   le suffixe `50`). Les URLs sont donc `github.com/SBFB50/SBFB`
>   et `codeberg.org/SBFB/SBFB` — meme nom de repo, orgs
>   differents.
> - **`git push --mirror` est destructif** : le workflow pousse
>   **toutes les refs** du source en mode mirror, ce qui **supprime
>   silencieusement cote Codeberg toute ref absente cote GitHub**
>   (branche, tag). Ne **jamais** creer une branche manuellement
>   cote Codeberg — elle serait wipee au prochain push GitHub.
>   Toutes les modifications doivent passer par GitHub (source of
>   truth).

**Status actuel (2026-04-15)** : **pre-launch protocol policy**
(cf. `CLAUDE.md §Pre-launch protocol policy`). Le repo GitHub
est **prive**. Le mirror Codeberg est egalement **prive** —
disaster-recovery maintainer uniquement, pas encore anti-subpoena
public.

**Bascule v1.0 go-live** : au premier tag `v1.0`, les deux repos
passent en public en un clic chacun + on active le mirror Radicle
(phase 3, voir §3). A ce moment la, la valeur bascule de
"disaster-recovery maintainer" vers "anti-subpoena public" au
benefice des users externes qui cloneront/forkeront.

**Radicle differe** : Radicle Heartwood est un reseau P2P
**public par design**, pas de repos prives. Publier maintenant
exposerait du code pre-launch. Radicle sera active en meme temps
que le flip public v1.0. Pattern et doc setup deja prepares.

---

## 2. Clone depuis Codeberg (fallback)

Quand GitHub est inaccessible :

```bash
git clone https://codeberg.org/SBFB/SBFB.git
```

Tant que le repo est prive (pre-launch) : necessite compte
Codeberg + accès maintainer grant. Apres v1.0 go-live : public,
clone anonyme ok.

Verification que le mirror est a jour :

```bash
# Côté GitHub (maintainer only pre-launch)
gh api repos/SBFB50/SBFB/commits/master --jq .sha

# Côté Codeberg
git ls-remote https://codeberg.org/SBFB/SBFB.git master | awk '{print $1}'

# Les 2 SHA doivent matcher (workflow push apres chaque GitHub push)
```

---

## 3. Flip sequence au v1.0 go-live

**A executer le jour du tag `v1.0`** (tout documente ici —
pattern Radicle pre-research par session precedente,
`gsaslis/mirror-to-radicle@v0.2.0` avril 2026 surface API
requiert 5 secrets, pas 1).

### 3.1 Flip visibilite GitHub + Codeberg (~2 min)

1. GitHub : `github.com/SBFB50/SBFB/settings/general` → Danger
   Zone → Change visibility → Public
2. Codeberg : `codeberg.org/SBFB/SBFB/settings` → Make public

### 3.2 Setup Radicle maintainer + machine account (~25 min, VM Linux)

Radicle Heartwood = pas de binaire Windows natif avril 2026,
necessite VM Linux ou WSL2. **Deux identites Radicle distinctes**
a creer : identite personnelle maintainer (signe `rad init`,
jamais exportee) + identite machine account (CI-only, poussee
en GHA secrets).

```bash
# 1. Install Radicle Heartwood (Linux VM ou WSL2)
curl -sSf https://radicle.xyz/install | sh

# 2. Identite maintainer personnelle (proprietaire projet Radicle).
#    Passphrase choisie = coffre-fort, jamais en secret GHA.
rad auth --alias sbfb-maintainer

# 3. Initialiser le repo sur Radicle depuis un clone propre.
git clone https://github.com/SBFB50/SBFB.git
cd SBFB
rad init --name SBFB --default-branch master --public \
         --description "SBFB — decentralized P2P compute and hosting"
# Copier le RID affiche (format rad:z...) pour etape 6.

# 4. Identite machine account (CI-only, push rights uniquement).
rad auth --alias sbfb-ci-mirror
# Noter le passphrase et la public key affiches — vont en GHA
# secrets etape 6.

# 5. Export machine-account private key base64 pour GHA.
base64 -w0 ~/.radicle/keys/sbfb-ci-mirror
```

### 3.3 Secrets GHA Radicle (5 entrees)

`github.com/SBFB50/SBFB/settings/secrets/actions` :

| Secret | Source |
|---|---|
| `RADICLE_IDENTITY_PASSPHRASE` | passphrase `rad auth sbfb-ci-mirror` |
| `RADICLE_IDENTITY_PRIVATE_KEY` | `base64 -w0 ~/.radicle/keys/sbfb-ci-mirror` |
| `RADICLE_IDENTITY_PUBLIC_KEY` | contenu `~/.radicle/keys/sbfb-ci-mirror.pub` |
| `RADICLE_REPOSITORY_ID` | RID affiche par `rad init` (format `rad:z...`) |
| `RADICLE_PROJECT_NAME` | `SBFB` (constant, mais secret pour coherence) |

**Alias `sbfb-ci-mirror`** inline dans le workflow YAML, pas
secret (non-sensible + auditabilite).

### 3.4 Ajout workflow `.github/workflows/mirror-radicle.yml`

Pin action au SHA exact (bumping = security review
explicite) :

```yaml
name: Mirror to Radicle

on:
  push:
    branches: [master]
  schedule:
    - cron: "0 3 * * *"  # fallback daily safety net
  workflow_dispatch:

permissions:
  contents: read

jobs:
  mirror:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Mirror to Radicle
        uses: gsaslis/mirror-to-radicle@514707f3fc8411f91331f00d7524c76584c10d78
        with:
          radicle-identity-alias: sbfb-ci-mirror
          radicle-identity-passphrase: ${{ secrets.RADICLE_IDENTITY_PASSPHRASE }}
          radicle-identity-private-key: ${{ secrets.RADICLE_IDENTITY_PRIVATE_KEY }}
          radicle-identity-public-key: ${{ secrets.RADICLE_IDENTITY_PUBLIC_KEY }}
          radicle-repository-id: ${{ secrets.RADICLE_REPOSITORY_ID }}
          radicle-project-name: sbfb
```

### 3.5 Canary update — ajouter `mirror_urls:` a `CANARY.txt`

Extend le texte signe (regenerer signature via `sbfb canary
publish --headline "SBFB warrant canary v1.0 go-live — mirrors
live"`) avec :

```
mirror_urls:
  github: https://github.com/SBFB50/SBFB
  codeberg: https://codeberg.org/SBFB/SBFB
  radicle: rad:z<RID-from-3.2>
```

Verifier nouvelle signature passe `scripts/verify-canary.sh`
avant commit.

### 3.6 Verification mirror Radicle post-first-run

Depuis n'importe quel node Linux avec Radicle installe :

```bash
rad clone rad:z<RID>
cd SBFB
git log --oneline | head
# Les commits doivent matcher GitHub master.
```

Ou navigateur : `https://app.radicle.xyz/nodes/<seed>/<RID>`.

### 3.7 Docs & tracking

1. Update `docs/release/MIRROR_FALLBACK.md §1 Status` →
   `v1.0 flipped, anti-subpoena public active`
2. Fermer item tracking `sprint18_audit_plan.md` §Radicle-v1.0
3. Enregistrer la date + signature rotation dans
   `docs/security/THREAT_MODEL.md §5.7 Key storage`

### 3.8 Rotation machine-account key (post-v1.0)

Compromise de la cle machine account = attaquant peut injecter
commits visibles aux seeds Radicle, mais ne peut pas reecrire
l'historique (chain of trust -> maintainer identity). Rotation :

1. Repeter `rad auth --alias sbfb-ci-mirror-2`
2. `rad id update` — remove old delegate, add new
3. Replace les 5 secrets GHA en batch
4. Log date rotation dans `THREAT_MODEL.md §5.7`

---

## 4. Maintainer setup Codeberg (reference — already done 2026-04-15)

Etapes executees au setup initial, documentees pour rotation /
nouveau maintainer :

1. Compte Codeberg : `codeberg.org/user/sign_up`
2. Creer repo vide `SBFB/SBFB` :
   - private, branche default `master`, pas d'init README
3. Generer Personal Access Token :
   `codeberg.org/user/settings/applications`
   - Accès : "Tout (public, privé et limité)"
   - Scope : `repository` **Read + Write** uniquement (moindre
     privilège)
4. Stocker token en GitHub Actions secret :
   - `github.com/SBFB50/SBFB/settings/secrets/actions` →
     New repository secret → `CODEBERG_TOKEN`
5. Workflow `.github/workflows/mirror-codeberg.yml` pousse
   automatiquement a chaque push GitHub (any branch + all tags).

---

## 5. Secret rotation

Le token Codeberg peut expirer ou etre compromis. Rotation :

1. Regenerer un nouveau token (etape 3 §4 ci-dessus)
2. Mettre a jour secret GitHub `CODEBERG_TOKEN` avec la nouvelle
   valeur
3. Revoquer l'ancien token sur Codeberg

**Frequence recommandee** : rotation proactive tous les 12 mois,
ou immediate en cas d'incident suspect (leak suspected, runner
compromise, maintainer device compromise).

---

## 6. Threat model fit

**Protects against** :

- GitHub account suspension / shadowban (political, ToS)
- GitHub.com service outage / DNS blackhole
- DMCA takedown attack on GitHub
- Subpoena seizure of GitHub infrastructure
- Regional ISP blocking github.com (Iran, China, Russia)

**Does NOT protect against** :

- Compromise of maintainer GitHub credentials (attacker pushes
  malicious code, mirror faithfully replicates it to Codeberg)
  → mitigated by `docs/release/REPRODUCIBLE_BUILDS.md` SLSA
  provenance + warrant canary (CANARY.txt Ed25519 signed)
- Compromise of `CODEBERG_TOKEN` secret (attacker can push to
  Codeberg mirror) → limited blast radius (mirror-only, no
  write to source of truth), mitigated by §5 rotation
- Codeberg.org service outage (mirror inaccessible) → future
  Radicle phase 3 covers this (orthogonal failure modes)
- Branche / tag cree manuellement cote Codeberg (pas via push
  GitHub) → sera supprime silencieusement au prochain run du
  workflow (`git push --mirror` strict), cf. §1 Notes de cohabitation

---

## 7. Related docs

- `.github/workflows/mirror-codeberg.yml` — workflow implementation
- `docs/release/REPRODUCIBLE_BUILDS.md` — SLSA provenance (Sprint 18 B)
- `CANARY.txt` + `scripts/verify-canary.sh` — warrant canary
  (Sprint 18 E2)
- `docs/security/THREAT_MODEL.md` — project-wide threat model
- `.planning/active/sprint18_plan.md §Phase E3` — pivot rationale
  2026-04-15 (Radicle → Codeberg dual-phased)
