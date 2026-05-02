# Publish Model — taxonomie des etats de publication SBFB

**Sprint 52 design doc.**
Complement de `SELF_HOSTED_BUILD.md` et du deploy verifie S14.

## 1. Principe

Ce qui tourne en local n'est pas automatiquement ce qui est publie
sur le protocole.

```
Code local modifie  !=  version open source verifiee
Version publiee SBFB = artefact immutable lie a un commit public precis
```

Une release SBFB n'est jamais mutable. Si le dev change son PC
apres publication, ca ne change rien a la version deja publiee.
L'artefact reste lie a :

```
repo_url + commit_sha + artifact_hash + provenance_hash
```

Si le projet local diverge, il y a juste une nouvelle version
non publiee.

## 2. Les 4 etats d'un projet sur le reseau

| Etat | Source | Badge UI | Workers publics | Mutable | Preuve |
|---|---|---|---|---|---|
| **Local Draft** | disque dev, Vite/dev server, daemon local | aucun | non | oui (c'est du dev) | aucune |
| **Unverified Build** | zip uploade sans provenance | "non verifie" | seulement opt-in `is_open_source=false` | non (blob immutable, mais pas de preuve source) | hash artefact seulement |
| **Verified Release** | commit public + provenance SLSA L1 | "open source verifie" | oui, consent L2+ | non (blob + commit + hash lies) | repo_url + commit_sha + artifact_hash + provenance_hash |
| **Stale Source** | repo public disparu/inaccessible | "source indisponible" | selon politique worker | non (artefact reste, confiance degradee) | provenance existe mais non-reverifiable en temps reel |

### 2.1 Local Draft

Le developpeur travaille sur son PC. Il lance `npm run dev` ou
le daemon local. Les modifications ne sont pas publiees. Aucun
worker du reseau ne voit ce code. Aucun badge. C'est du
developpement normal.

### 2.2 Unverified Build

Le developpeur force un zip ou un paquet local sur le reseau
sans passer par deploy-from-repo. Le protocole l'accepte mais :

- `is_open_source = false`
- Provenance absente ou non verifiee
- Visibilite privee / dev / unverified
- Les workers en consent L2 ("open source verifie seulement")
  refusent les taches de ce projet

Cas d'usage : prototypage rapide, apps privees, tests internes.

### 2.3 Verified Release

Le chemin standard pour une publication open source :

```
1. git commit + git push (repo public)
2. sbfb deploy-from-repo --repo <url> --commit <sha>
3. Le daemon clone le repo (--depth 1)
4. Verifie SBFB.json (Keyoxide Ed25519)
5. Build le zip (index.html + assets)
6. Genere provenance.json (SLSA L1 : subject + digest + builder)
7. Publie via iroh-blobs (hash content-addressable)
8. is_open_source = true
```

L'artefact publie est **immutable** :
- Le hash iroh-blobs est derive du contenu — modifier le zip
  change le hash, donc l'URL
- La provenance lie commit_sha + artifact_hash + builder_id
- Tout pair peut re-cloner le repo au meme commit et verifier
  que le zip produit le meme hash

Cf. `sprint14_keyoxide_decision.md` (memory) pour le design
complet du deploy verifie.

### 2.4 Stale Source

Le repo public n'est plus accessible — GitHub a coupe le compte,
le dev a supprime le repo, la forge est down.

L'artefact est toujours sur iroh-blobs, toujours fonctionnel.
Mais la preuve "code sur le reseau = code du repo" n'est plus
reverifiable en temps reel.

**Mitigations par couche de forge** :

| Forge | Role | Resilience |
|---|---|---|
| GitHub | decouverte, PRs, stars | SPOF si unique forge |
| Codeberg mirror | fallback lisible | survit a une coupure GitHub |
| Radicle | fallback anti-censure P2P | survit a toute forge centralisee |

Un projet qui publie sur les 3 forges ne tombe en Stale Source
que si les 3 sont simultanement indisponibles.

**Politique worker face a Stale Source** :
- Workers en consent L2 strict : refusent (pas de preuve source)
- Workers en consent L1 (tolerant) : acceptent si provenance
  existe (le hash est toujours valide, seule la reverifiabilite
  est perdue)
- Le coordinateur peut marquer un projet Stale Source apres N
  echecs consecutifs de clone verification

## 3. Implications pour le self-hosted build

Le reseau SBFB ne compile que des **Verified Release** :
- Commit public pinne (SHA complet 40 hex)
- Lockfile present et hashe
- Repo clonable par les workers
- Provenance generee apres quorum

Jamais un Local Draft. Jamais un Unverified Build.

Ca ferme le vecteur "build bomb depuis un repo local sale" —
l'attaquant doit au minimum publier son code sur un repo public,
ce qui laisse une trace auditable.

## 4. Implications pour l'UI

L'interface shell React affiche le badge correspondant :

```
[open source verifie]  — commit abc123 sur github.com/foo/bar
                          provenance SLSA L1 valide
                          artefact hash: sha256:deadbeef...

[non verifie]          — upload direct, pas de provenance
                          aucune preuve que le code source
                          correspond au zip

[source indisponible]  — provenance existe (commit abc123)
                          mais le repo n'est plus accessible
                          artefact toujours fonctionnel

[local draft]          — visible uniquement dans le daemon
                          local, pas publie sur le reseau
```

## 5. Relation avec la strategie hybride forges

```
Decouverte:        GitHub (stars, issues, PRs, contributeurs)
Mirror lisible:    Codeberg (memes tags, memes checksums)
Mirror P2P:        Radicle (anti-censure, post-v1.0)
CI quotidienne:    Woodpecker VPS (gate officiel hors GitHub)
Release trust:     signatures + checksums + attestations SLSA
Distribution:      GitHub Releases + Codeberg + iroh-blobs
Self-build:        SBFB quorum multi-builder (LT-7 pre-v1.0)
```

Regle simple :
> **GitHub pour etre trouve. SBFB/Codeberg/Radicle pour survivre.**

GitHub peut rester dans la boucle, mais ne doit plus etre
l'autorite finale. Aucun binaire officiel si la CI hors GitHub
n'a pas passe.

## 6. Cycle de vie complet d'une release

```
Dev local (Local Draft)
  |
  v
git push repo public
  |
  v
CI Woodpecker VPS valide le commit    <-- gate obligatoire
  |                                        hors GitHub
  v
GHA optionnel ("second opinion")
  |
  v
sbfb deploy-from-repo                 <-- Verified Release
  commit_sha + Keyoxide + zip + SLSA
  |
  v
iroh-blobs distribution P2P           <-- immutable
  |
  v
[plus tard] SBFB self-build quorum    <-- remplace Woodpecker
  N builders independants                  comme gate
  SHA256 consensus
  attestation signee
```

A chaque etape, le projet peut etre dans un seul des 4 etats.
La progression normale est : Local Draft → Verified Release.
Le passage par Unverified Build est un raccourci pour le
prototypage, pas le chemin de publication officiel.
