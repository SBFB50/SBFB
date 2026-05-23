# Gate 1 — Protocole de test pilote ferme

**Version :** 1.0
**Sprint :** S69 Phase D
**Testeurs cibles :** 2-3 personnes (pilote ferme, pas public)
**Critere de passage :** 9/9 criteres Go dans le tableau recapitulatif

---

## Table des matieres

1. [Instructions d'installation](#1-instructions-dinstallation)
2. [Verification d'integrite des binaires](#2-verification-dintegrite-des-binaires)
3. [Procedures de test](#3-procedures-de-test)
4. [Formulaire de feedback](#4-formulaire-de-feedback)
5. [Rapport de bugs](#5-rapport-de-bugs)
6. [Contacts](#6-contacts)

---

## 1. Instructions d'installation

### 1.1 Pre-requis

- Connexion internet (pour la decouverte P2P initiale)
- Navigateur web moderne (Firefox, Chrome, Edge)
- 200 Mo d'espace disque libre
- Port 4919 (HTTP loopback) libre

### 1.2 Windows

1. Telecharger `sbfb-installer.exe` depuis le lien fourni
2. Verifier l'integrite du binaire (section 2)
3. Double-cliquer sur `sbfb-installer.exe`
4. Suivre l'assistant d'installation (Next → Install → Finish)
5. Le lanceur `sbfb-launcher.exe` est ajoute au menu Demarrer
6. Lancer SBFB depuis le menu Demarrer → SBFB
7. Le daemon demarre et le navigateur s'ouvre sur `http://127.0.0.1:4919`

### 1.3 macOS

1. Telecharger `sbfb-launcher.dmg` depuis le lien fourni
2. Verifier l'integrite du binaire (section 2)
3. Ouvrir le `.dmg` et glisser SBFB dans Applications
4. Au premier lancement, clic droit → Ouvrir (contournement Gatekeeper)
5. Le daemon demarre et le navigateur s'ouvre sur `http://127.0.0.1:4919`

### 1.4 Linux

1. Telecharger `sbfb-launcher` depuis le lien fourni
2. Verifier l'integrite du binaire (section 2)
3. Rendre executable : `chmod +x sbfb-launcher`
4. Lancer : `./sbfb-launcher`
5. Le daemon demarre et le navigateur s'ouvre sur `http://127.0.0.1:4919`

---

## 2. Verification d'integrite des binaires

Avant toute installation, verifier que le binaire telecharge correspond
au hash officiel. Si le hash ne correspond pas, ne pas installer et
signaler immediatement.

| Plateforme | Fichier | BLAKE3 |
|------------|---------|--------|
| Windows | `sbfb-installer.exe` | `<A_REMPLIR_AVANT_DISTRIBUTION>` |
| macOS | `sbfb-launcher.dmg` | `<A_REMPLIR_AVANT_DISTRIBUTION>` |
| Linux | `sbfb-launcher` | `<A_REMPLIR_AVANT_DISTRIBUTION>` |

**Commande de verification :**

```bash
# Avec b3sum (https://github.com/BLAKE3-team/BLAKE3)
b3sum sbfb-installer.exe

# Ou avec sha256sum en fallback
sha256sum sbfb-installer.exe
```

Les hash SHA256 de fallback seront fournis avec le lien de
telechargement si `b3sum` n'est pas disponible.

---

## 3. Procedures de test

Chaque procedure correspond a un critere Gate 1 de la roadmap v4.
Executer dans l'ordre — certains tests dependent des precedents.

### Test 1 — Installation

**Critere Gate 1 :** 2/3 testeurs installent sans aide.

**Go :** Installation reussie sans assistance exterieure.
**No-Go :** Echec d'installation ou besoin d'aide technique.

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 1.1 | Telecharger le binaire pour votre plateforme | Fichier telecharge |
| 1.2 | Verifier le hash BLAKE3 (section 2) | Hash correspond |
| 1.3 | Installer selon les instructions (section 1) | Installation terminee sans erreur |
| 1.4 | Lancer SBFB | Le navigateur s'ouvre sur `http://127.0.0.1:4919` |
| 1.5 | Verifier que la page d'accueil s'affiche | Page Browse visible avec barre de navigation |

**Resultat :** ______ (Go / No-Go)
**Notes :** ______

---

### Test 2 — Connexion P2P

**Critere Gate 1 :** 2 noeuds se voient en < 5 min.

**Go :** Decouverte mutuelle en moins de 5 minutes.
**No-Go :** Aucune connexion apres 15 minutes.

**Pre-requis :** 2 testeurs en ligne simultanement (meme LAN ou WAN).

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 2.1 | Les 2 testeurs lancent SBFB | Daemon actif sur chaque machine |
| 2.2 | Aller dans l'onglet Network | Page Network affichee |
| 2.3 | Attendre la decouverte P2P (max 5 min) | Au moins 1 peer apparait dans la liste |
| 2.4 | Noter le temps de decouverte | Temps < 5 min |

**Resultat :** ______ (Go / No-Go)
**Temps de decouverte :** ______ min
**Notes :** ______

---

### Test 3 — Deploy app depuis source

**Critere Gate 1 :** 1 testeur deploie depuis source.

**Go :** App deployee visible dans Browse.
**No-Go :** Deploy echoue.

**Pre-requis :** `sbfb-factory` installe (fourni avec le package),
`git` disponible.

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 3.1 | Ouvrir un terminal | Terminal pret |
| 3.2 | Creer un projet : `sbfb-factory create --template static --name test-app --output ./test-app` | Repertoire `test-app` cree avec `index.html` et `SBFB.json` |
| 3.3 | Initialiser le repo : `cd test-app && git init && git add -A && git commit -m "init"` | Repo Git initialise |
| 3.4 | Valider le projet : `sbfb-factory validate .` | Validation PASS |
| 3.5 | Pousser le repo vers un hebergeur (GitHub, Codeberg, etc.) | Repo accessible via URL HTTPS |
| 3.6 | Publier : `sbfb-factory publish . --repo-url https://github.com/<user>/test-app` | Publication reussie, hash affiche |
| 3.7 | Ouvrir Browse dans le navigateur | L'app `test-app` apparait dans la liste |
| 3.8 | Cliquer sur l'app | L'app s'ouvre dans un iframe sandbox |

**Resultat :** ______ (Go / No-Go)
**Hash de l'app :** ______
**Notes :** ______

---

### Test 4 — Babel via Factory

**Critere Gate 1 :** Babel creee avec Factory, deployee, visible Browse.

**Go :** Babel deployee et affichee dans Browse.
**No-Go :** Factory echoue ou Babel non visible.

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 4.1 | Creer Babel : `sbfb-factory create --template static-reader --name babel --output ./babel` | Repertoire `babel` cree avec `index.html`, `SBFB.json`, `sbfb-bridge.js` |
| 4.2 | (Optionnel) Editer `babel/index.html` pour ajouter du contenu | Contenu personnalise visible |
| 4.3 | Initialiser le repo : `cd babel && git init && git add -A && git commit -m "init"` | Repo Git initialise |
| 4.4 | Valider : `sbfb-factory validate .` | Validation PASS |
| 4.5 | Previsualiser : `sbfb-factory preview .` | Preview chargee, URL affichee |
| 4.6 | Ouvrir la preview dans le navigateur | L'app Babel s'affiche correctement |
| 4.7 | Pousser le repo vers un hebergeur | Repo accessible via URL HTTPS |
| 4.8 | Publier : `sbfb-factory publish . --repo-url https://github.com/<user>/babel` | Publication reussie, hash et provenance affiches |
| 4.9 | Ouvrir Browse | Babel apparait dans la liste |
| 4.10 | Cliquer sur Babel | Le reader s'affiche dans l'iframe |

**Resultat :** ______ (Go / No-Go)
**Hash Babel :** ______
**Notes :** ______

---

### Test 5 — Feed sync

**Critere Gate 1 :** Feed synchronise entre 2+ noeuds.

**Go :** Le feed est identique sur les 2 noeuds.
**No-Go :** Divergence ou corruption du feed.

**Pre-requis :** 2 noeuds connectes (Test 2 PASS), 1 app deployee
(Test 3 ou 4 PASS).

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 5.1 | Sur le noeud A : deployer une app (Test 3 ou 4) | App deployee, visible dans Browse de A |
| 5.2 | Sur le noeud B : ouvrir Browse | L'app deployee par A apparait dans la liste de B |
| 5.3 | Verifier le contenu | Le nom, la description et la categorie sont identiques |
| 5.4 | Sur le noeud B : ouvrir l'app | L'app s'affiche correctement |
| 5.5 | Noter le temps de propagation | Temps raisonnable (< 2 min) |

**Resultat :** ______ (Go / No-Go)
**Temps de propagation :** ______ min
**Notes :** ______

---

### Test 6 — Restart

**Critere Gate 1 :** Daemon redemarrage propre.

**Go :** Redemarrage sans perte de donnees.
**No-Go :** State corrompu apres redemarrage.

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 6.1 | Fermer SBFB (fermer la fenetre ou Ctrl+C sur le terminal) | Daemon s'arrete proprement |
| 6.2 | Relancer SBFB | Daemon redemarre |
| 6.3 | Ouvrir Browse | Les apps precedemment deployees sont toujours visibles |
| 6.4 | Ouvrir l'onglet Curators | Les listes de curators sont preservees |
| 6.5 | Verifier Network | La reconnexion P2P s'effectue |

**Resultat :** ______ (Go / No-Go)
**Notes :** ______

---

### Test 7 — Stabilite 24h

**Critere Gate 1 :** Daemon tourne 24h sans crash.

**Go :** 24h de fonctionnement continu sans crash, OOM ou freeze.
**No-Go :** Crash, OOM ou freeze.

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 7.1 | Lancer SBFB et noter l'heure de demarrage | Daemon actif |
| 7.2 | Laisser tourner 24h (machine allumee, connexion internet) | Daemon toujours actif apres 24h |
| 7.3 | Apres 24h : ouvrir Browse | Page s'affiche normalement |
| 7.4 | Verifier Network | Peers visibles |
| 7.5 | Deployer une app (Test 3) | Deploy reussit |
| 7.6 | Verifier les logs pour erreurs | Pas de panic, pas de OOM |

**Commande logs :**
```bash
# Windows (PowerShell)
Get-Content "$env:LOCALAPPDATA\sbfb\daemon.log" -Tail 50

# macOS / Linux
tail -50 ~/.local/share/sbfb/daemon.log
```

**Resultat :** ______ (Go / No-Go)
**Heure debut :** ______
**Heure fin :** ______
**Anomalies observees :** ______
**Notes :** ______

---

### Test 8 — Search RRV trouve Babel

**Critere Gate 1 :** `search?q=babel` retourne Babel.

**Go :** Babel apparait dans les resultats de recherche.
**No-Go :** Search vide.

**Pre-requis :** Babel deployee (Test 4 PASS).

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 8.1 | Ouvrir Browse dans le navigateur | Page Browse affichee |
| 8.2 | Utiliser la barre de recherche : taper "babel" | Resultats affiches |
| 8.3 | Verifier que Babel apparait | Babel presente dans la liste |
| 8.4 | Tester une recherche partielle : "bab" | Babel apparait (FTS5 prefix) |
| 8.5 | Tester une recherche par categorie : "content" | Babel apparait (categorie content) |

**Resultat :** ______ (Go / No-Go)
**Notes :** ______

---

### Test 9 — Proof Card

**Critere Gate 1 :** Proof Card Babel affichee.

**Go :** Proof Card visible avec score et couches de preuve.
**No-Go :** Proof Card absente ou score incorrect.

**Pre-requis :** Babel deployee (Test 4 PASS).

| Etape | Action | Resultat attendu |
|-------|--------|------------------|
| 9.1 | Ouvrir Browse → cliquer sur Babel | Page detail du projet |
| 9.2 | Localiser la section Proof Card | Proof Card visible |
| 9.3 | Verifier le score (0-100) | Score affiche, coherent |
| 9.4 | Developper les couches de preuve | 6 couches visibles (provenance, hash, signature, source, manifest, curator) |
| 9.5 | Verifier les risk factors | Facteurs de risque affiches avec badges |
| 9.6 | Verifier la provenance Ed25519 | Signature provenance presente |

**Resultat :** ______ (Go / No-Go)
**Score affiche :** ______
**Notes :** ______

---

## 4. Formulaire de feedback

Remplir ce tableau apres avoir execute les 9 tests.

| # | Critere | Resultat | Bloqueur ? | Notes |
|---|---------|----------|------------|-------|
| 1 | Installation | Go / No-Go | Oui / Non | |
| 2 | Connexion P2P | Go / No-Go | Oui / Non | |
| 3 | Deploy app | Go / No-Go | Oui / Non | |
| 4 | Babel via Factory | Go / No-Go | Oui / Non | |
| 5 | Feed sync | Go / No-Go | Oui / Non | |
| 6 | Restart | Go / No-Go | Oui / Non | |
| 7 | Stabilite 24h | Go / No-Go | Oui / Non | |
| 8 | Search RRV | Go / No-Go | Oui / Non | |
| 9 | Proof Card | Go / No-Go | Oui / Non | |

**Verdict global :** ______ (PASS si 9/9 Go, FAIL sinon)

**Informations testeur :**
- Plateforme : ______ (Windows / macOS / Linux)
- Version OS : ______
- Date du test : ______
- Duree totale : ______

---

## 5. Rapport de bugs

Pour chaque bug rencontre, remplir une entree :

| Champ | Valeur |
|-------|--------|
| Test # | |
| Etape | |
| Description | |
| Comportement attendu | |
| Comportement observe | |
| Reproductible ? | Oui / Non / Intermittent |
| Logs pertinents | |
| Capture d'ecran | (joindre si possible) |

Envoyer les rapports par le canal de communication convenu avec
le mainteneur.

---

## 6. Contacts

- **Mainteneur :** canal direct convenu avant le pilote
- **Repo source :** lien fourni aux testeurs
- **Issues :** signaler via le rapport de bugs (section 5)

Le pilote est ferme — ne pas distribuer les binaires en dehors
du groupe de test.
