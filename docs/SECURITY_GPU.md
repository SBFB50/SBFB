# NEXUS Distributed GPU Compute -- Security Model

Documentation de securite du systeme de calcul GPU distribue.
Chaque seuil et constante provient directement du code source.

---

## Table des matieres

1. [Authentification](#1-authentification)
2. [Vie privee](#2-vie-privee)
3. [Rate Limiting](#3-rate-limiting)
4. [Couche 1 -- Signatures Ed25519](#4-couche-1----signatures-ed25519)
5. [Couche 2 -- Whitelist des digests modeles](#5-couche-2----whitelist-des-digests-modeles)
6. [Couche 3 -- Fingerprinting Logprob](#6-couche-3----fingerprinting-logprob)
7. [Score de confiance (Trust Score)](#7-score-de-confiance-trust-score)
8. [Spot-checking BOINC-style](#8-spot-checking-boinc-style)
9. [Isolation des donnees](#9-isolation-des-donnees)
10. [Modele de menaces](#10-modele-de-menaces)

---

## 1. Authentification

### Generation de cle API

A l'enregistrement (`POST /api/compute/register`), le serveur genere une cle API unique :

```python
# nexus/compute/db.py, _generate_api_key()
secrets.token_urlsafe(32)
```

`secrets.token_urlsafe(32)` produit 32 octets d'entropie cryptographique, encodes en base64url, soit 43 caracteres. La cle est retournee **une seule fois** dans la reponse d'enregistrement et n'est jamais stockee en clair.

### Stockage par hachage SHA-256

Seul le hash SHA-256 de la cle API est persiste en base :

```python
# nexus/compute/db.py, _hash_api_key()
hashlib.sha256(api_key.encode()).hexdigest()
```

La colonne `api_key_hash TEXT NOT NULL` dans `compute_nodes` stocke ce hash. Un index (`idx_compute_nodes_api_key`) accelere la recherche par hash.

### Parsing du Bearer token

L'authentification se fait via le header `Authorization: Bearer <api_key>` :

```python
# nexus/api/compute.py, _get_authenticated_node()
if not authorization.lower().startswith("bearer "):
    raise HTTPException(status_code=401, ...)

api_key = authorization[7:].strip()
```

- **Case-insensitive** : `authorization.lower().startswith("bearer ")` accepte `Bearer`, `bearer`, `BEARER`, etc.
- **Trimmed** : `.strip()` supprime les espaces parasites autour de la cle.
- Le serveur hash la cle recue et compare avec `api_key_hash` en base via `get_node_by_api_key()`.
- Un noeud avec `status == "banned"` recoit une erreur 403 meme avec une cle valide.

---

## 2. Vie privee

### Hachage des adresses IP

Les adresses IP ne sont **jamais stockees en clair**. Elles sont hachees avant persistance :

```python
# nexus/compute/db.py, _hash_ip()
hashlib.sha256(ip.encode()).hexdigest()
```

La colonne `ip_hash TEXT NOT NULL` dans `compute_nodes` ne contient que le hash SHA-256. Ce hash est aussi utilise comme cle du rate limiter (voir section 3).

### Pas de donnees personnelles dans les prompts

Les taches envoyees aux noeuds GPU contiennent uniquement du texte politique public (debats parlementaires, textes de loi, votes). Aucun identifiant de politicien ou donnee personnelle n'est inclus dans les prompts distribues. Voir section 9 (Isolation des donnees) pour les details.

---

## 3. Rate Limiting

Le rate limiter est in-memory, par IP hashee :

```python
# nexus/api/compute.py
_RATE_LIMIT_PER_MINUTE = 100
```

**Mecanisme :**

- Chaque requete enregistre un timestamp dans `_rate_limits[ip_hash]`.
- Les entrees de plus de 60 secondes sont nettoyees a chaque appel (`now - t < 60`).
- Si le nombre de requetes recentes depasse 100, le serveur retourne `HTTP 429 Rate limit exceeded (100 req/min)`.
- Le nettoyage est passif (a chaque requete) -- pas de thread de garbage collection.
- Les hashs IP sans requetes recentes sont supprimes du dictionnaire (`_rate_limits.pop(ip_hash, None)`).

**Endpoints proteges :** `/register`, `/heartbeat`, `/task`, `/result`, `/model/ready` -- tous les endpoints ou un noeud interagit directement.

**Limitation :** Le rate limiter est in-memory et ne survit pas aux redemarrages du serveur. Il n'est pas distribue (un seul serveur NEXUS).

---

## 4. Couche 1 -- Signatures Ed25519

**Objectif :** Prouver QUI a soumis le resultat (non-repudiation cryptographique).

### Generation de paire de cles

```python
# nexus/compute/crypto.py, generate_keypair()
private_key = Ed25519PrivateKey.generate()
# Encodage PEM (PKCS8) pour la cle privee
# SubjectPublicKeyInfo pour la cle publique
```

La cle publique (PEM) est envoyee au serveur lors de l'enregistrement (`public_key_pem` dans `NodeRegisterRequest`). La cle privee reste sur le noeud contributeur.

### Construction du payload (`_build_payload`)

Le payload signe est un JSON deterministe :

```python
# nexus/compute/crypto.py, _build_payload()
data = {
    "task_id": task_id,
    "result": result_text[:2000],  # Troncature a 2000 caracteres
    "model_digest": model_digest,
    "node_id": node_id,
}
return json.dumps(data, sort_keys=True, ensure_ascii=True).encode("utf-8")
```

- **Tri des cles** (`sort_keys=True`) : garantit un ordre deterministe quel que soit le langage.
- **ASCII** (`ensure_ascii=True`) : evite les variations d'encodage Unicode.
- **Troncature** (`result_text[:2000]`) : seuls les 2000 premiers caracteres sont signes, pour des raisons de performance. Le reste du resultat n'est pas couvert par la signature.

### Signature (cote worker)

```python
# nexus/compute/crypto.py, sign_result()
signature = key.sign(payload)  # Ed25519 natif
return base64.b64encode(signature).decode("ascii")
```

La signature base64 est envoyee dans le champ `signature` de `TaskResultRequest`.

### Verification (cote serveur)

```python
# nexus/compute/crypto.py, verify_signature()
key.verify(signature, payload)  # Leve une exception si invalide
```

- Si la bibliotheque `cryptography` n'est pas installee : **degradation gracieuse** -- la verification est bypassed (`return True`).
- Si `public_key_pem` ou `signature_b64` est vide : retourne `False`.
- Echec de signature : `trust_delta = -50`, **ban immediat**.

---

## 5. Couche 2 -- Whitelist des digests modeles

**Objectif :** Verifier QUEL modele est charge sur le noeud (SHA-256 des poids).

### Obtention du digest

Le digest est le hash SHA-256 du fichier de poids du modele, obtenu via l'API Ollama (`/api/show`). Ce hash est unique par modele et version.

### Enregistrement dans la whitelist

```python
# nexus/compute/verification.py, register_digest()
_DIGEST_WHITELIST[model] = digest
```

La whitelist est un dictionnaire en memoire (`model_name -> SHA-256 digest`), peuple au demarrage par scan du GPU de confiance ou configuration manuelle.

### Verification

```python
# nexus/compute/verification.py, verify_digest()
```

| Cas | Resultat | Raison |
|-----|----------|--------|
| Whitelist vide | `True` | `no_whitelist` |
| Digest manquant | `False` | `missing_digest` |
| Modele absent de la whitelist | `True` | `model_not_in_whitelist` |
| Digest correspond | `True` | `digest_match` |
| Digest ne correspond pas | `False` | `digest_mismatch` |

Echec de digest (`digest_mismatch` ou `missing_digest`) : `trust_delta = -50`, **ban immediat**.

---

## 6. Couche 3 -- Fingerprinting Logprob

**Objectif :** Verifier que le BON modele a reellement execute la tache (pas seulement charge).

Base sur la recherche LLMmap (USENIX Security 2025) : les distributions de logprob sont uniques par modele pour des prompts calibres.

### Prompts de calibration

8 prompts predetermines, orientes politique francaise :

```python
# nexus/compute/verification.py, CALIBRATION_PROMPTS
1. "La capitale de la France est"
2. "Le president de la Republique en 2026 est"
3. "L'article 49.3 permet au gouvernement de"
4. "Le nombre de deputes a l'Assemblee nationale est"
5. "La devise de la France est"
6. "Le Senat est compose de"
7. "La Constitution de la Cinquieme Republique date de"
8. "Le Premier ministre est nomme par"
```

### Taux d'echantillonnage

```python
# nexus/compute/verification.py, should_calibrate()
return random.random() < 0.10  # 10% des taches
```

Un prompt de calibration aleatoire est envoye avec 10% des taches. Le noeud doit retourner les logprobs de sa reponse.

### Profils de reference

Les profils de reference (`_LOGPROB_PROFILES`) sont des dictionnaires `{token: logprob}` calibres sur le GPU de confiance pour chaque couple `(modele, prompt)`.

### Verification

```python
# nexus/compute/verification.py, verify_logprobs()
threshold: float = 0.5  # Difference absolue maximale
```

**Algorithme :**
1. Pour chaque token du profil de reference, comparer avec le logprob reporte.
2. Calculer la difference absolue maximale (`max_diff`).
3. Si `max_diff > 0.5` : echec.
4. Si aucun token ne correspond (`matched == 0`) : echec.

| Cas | Resultat | Raison |
|-----|----------|--------|
| Pas de profils configures | `True` | `no_profiles_configured` |
| Modele non profile | `True` | `model_not_profiled` |
| Prompt non profile | `True` | `prompt_not_profiled` |
| Logprobs manquants | `False` | `missing_logprobs` |
| Aucun token commun | `False` | `no_matching_tokens` |
| Divergence > 0.5 | `False` | `logprob_divergence` |
| Match | `True` | `logprob_match` |

**Consequence d'un echec logprob :** `trust_delta = -5`, pas de ban immediat. Le resultat est quand meme accepte (`passed: True`) mais le noeud est marque comme suspect, ce qui augmente son taux de spot-check (voir section 8).

---

## 7. Score de confiance (Trust Score)

Chaque noeud possede un `trust_score` (colonne `INTEGER DEFAULT 50` dans `compute_nodes`), borne entre 0 et 100 :

```python
# nexus/compute/db.py, update_node_trust()
new_score = max(0, min(100, row[0] + delta))
```

### Deltas de confiance

| Evenement | Delta | Source |
|-----------|-------|--------|
| Tache acceptee (toutes verifications OK) | **+1** | `ResultVerifier.verify()` -- `trust_delta = 1` par defaut |
| Spot-check reussi | **+5** | Documente dans `verification.py` header |
| Spot-check echoue | **-20** | Documente dans `verification.py` header |
| Echec logprob (Couche 3) | **-5** | `ResultVerifier.verify()` |
| Resultat trop court (< 5 chars) | **-5** | `TaskDispatcher.validate_result()` |
| Echec signature (Couche 1) | **-50** + ban | `ResultVerifier.verify()` |
| Echec digest (Couche 2) | **-50** + ban | `ResultVerifier.verify()` |

### Auto-ban

- Le seuil d'auto-ban est document a `trust_score < 20`.
- L'echec des Couches 1 ou 2 declenche un ban **immediat** (via `ban_node()`), qui met `trust_score = 0` et `status = 'banned'`.
- Un noeud banni voit ses taches assignees remises en file d'attente (`status = 'pending'`).
- Un noeud avec `status == "banned"` recoit une erreur `HTTP 403` sur tous les endpoints authentifies.

---

## 8. Spot-checking BOINC-style

Filet de securite : un pourcentage des resultats est re-execute sur le GPU de confiance (local) pour comparaison semantique.

### Taux de spot-check par niveau de confiance

```python
# nexus/compute/verification.py, spot_check_needed()
# nexus/compute/dispatcher.py, _get_spot_check_rate()
if trust_score >= 80:
    rate = 0.01   # 1% pour les noeuds de confiance
elif trust_score >= 50:
    rate = 0.05   # 5% standard (score initial = 50)
else:
    rate = 0.20   # 20% pour les noeuds suspects
```

| Plage trust_score | Taux spot-check | Profil |
|-------------------|-----------------|--------|
| >= 80 | 1% | Noeud de confiance |
| >= 50 (defaut) | 5% | Standard |
| < 50 | 20% | Suspect |

### Mecanisme

1. Apres acceptation d'un resultat, `spot_check_needed()` tire aleatoirement selon le taux.
2. Si spot-check necessaire, un evenement `COMPUTE_SPOT_CHECK_NEEDED` est publie sur l'EventBus.
3. La tache originale (prompt tronque a 500 caracteres) est re-executee localement.
4. Les resultats sont compares semantiquement (pas de correspondance exacte).
5. Spot-check reussi : `+5` au trust score. Echec : `-20`.

### Feedback loop

Un noeud suspect (trust < 50) est spot-checke 4x plus souvent qu'un noeud standard, ce qui accelere sa rehabilitation ou son ban progressif. Un noeud qui accumule des echecs descendra sous le seuil de ban.

---

## 9. Isolation des donnees

### Contenu des prompts distribues

Les taches distribuees aux noeuds GPU contiennent exclusivement du texte politique public :

- Debats parlementaires (comptes rendus officiels)
- Textes de loi et propositions
- Resultats de votes publics
- Resumes d'actualite politique

### Ce qui n'est PAS distribue

- **Aucun identifiant de politicien** dans les prompts (pas de base de donnees interne)
- **Aucune donnee personnelle** d'utilisateurs ou contributeurs
- **Aucune URL interne** du serveur (le endpoint `/hybrid/status` retourne `exo_url=""` explicitement)

### Self-worker

Le `SelfWorker` (GPU embarque du serveur) accede directement a la base de donnees sans passer par HTTP. Il n'a pas besoin d'authentification API :

- Il est enregistre avec le nom `_self_worker_` et l'IP `127.0.0.1`.
- Il pull les taches directement depuis la DB, pas via l'API REST.
- Ses resultats sont stockes directement, sans passer par la verification 3 couches (il est le GPU de confiance).

---

## 10. Modele de menaces

### Attaques prevenues

| Menace | Defense | Couche |
|--------|---------|--------|
| **Vol/rejeu de cle API** | SHA-256 hachage, cle montree une fois | Auth |
| **Brute force** | 256 bits d'entropie (32 bytes urlsafe), rate limit 100/min | Auth + Rate limit |
| **Usurpation d'identite** | Signature Ed25519 sur chaque resultat | Couche 1 |
| **Substitution de modele** (charger un petit modele au lieu du gros) | Digest SHA-256 des poids, compare a la whitelist | Couche 2 |
| **Proxy de modele** (relayer vers un autre service) | Fingerprinting logprob unique par modele | Couche 3 |
| **Resultats inventes** (ne pas executer le prompt) | Spot-checking BOINC re-execute sur GPU de confiance | Spot-check |
| **Contribution de spam** | Validation longueur minimale (5 chars), rate limit | Validation |
| **Tracking d'IP** | SHA-256 unidirectionnel, IP brute jamais persistee | Privacy |
| **Escalade apres ban** | status="banned" verifie dans le middleware auth, 403 systematique | Auth |
| **DDoS** | Rate limit in-memory 100 req/min par IP | Rate limit |

### Compromis acceptes

| Compromis | Raison |
|-----------|--------|
| **Degradation gracieuse sans `cryptography`** | Si le package n'est pas installe, les signatures sont bypassees (`return True`). Ceci est un choix delibere pour permettre le fonctionnement sans la dependance optionnelle. |
| **Troncature du payload signe a 2000 chars** | Performance : signer la totalite d'une reponse de 16K tokens serait couteux. Un attaquant pourrait theoriquement modifier la fin d'un resultat au-dela de 2000 caracteres sans invalider la signature. |
| **Rate limiter in-memory** | Ne survit pas aux redemarrages. Suffisant pour un serveur unique (pas de clustering NEXUS). |
| **Logprob sampling a 10%** | Compromis performance/detection : verifier les logprobs de chaque tache serait trop couteux. Un attaquant suffisamment patient pourrait passer 90% de taches frauduleuses non verifiees, mais le spot-checking et l'accumulation de trust penalties finissent par le detecter. |
| **Whitelist manuelle** | La whitelist des digests doit etre peuplee manuellement ou par scan au demarrage. Si elle est vide, la Couche 2 est bypassee (`no_whitelist`). |
| **Pas de chiffrement des prompts en transit** | Les prompts sont envoyes en clair via HTTP. Le deploiement en production doit utiliser HTTPS (TLS) au niveau du reverse proxy. |
| **Modele absent de la whitelist accepte** | Si un modele n'a pas ete enregistre dans la whitelist, son digest est accepte avec le flag `model_not_in_whitelist`. Ceci permet aux nouveaux modeles de fonctionner sans reconfiguration. |
| **Self-worker bypass verification** | Le GPU local est considere comme GPU de confiance par definition. Ses resultats ne passent pas par la verification 3 couches. |

### Flux de verification complet

```
Resultat soumis par un noeud
    |
    v
[Validation basique] -- result_text < 5 chars? --> REJET (-5 trust)
    |
    v
[Couche 1: Ed25519] -- Signature invalide? --> BAN (-50 trust, status=banned)
    |                  -- Pas de signature?  --> skip (not_provided)
    v
[Couche 2: Digest]  -- Digest mismatch?    --> BAN (-50 trust, status=banned)
    |                  -- Pas de whitelist?  --> skip (no_whitelist)
    v
[Couche 3: Logprob] -- Divergence > 0.5?   --> ACCEPT (-5 trust, suspect)
    |                  -- Pas de calibration --> skip (not_calibrated)
    v
[ACCEPT] (+1 trust)
    |
    v
[Spot-check?] -- random < rate(trust) --> Re-execution sur GPU de confiance
                                           Pass: +5 trust / Fail: -20 trust
```
