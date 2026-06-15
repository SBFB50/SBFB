# Vision — Nœud institutionnel et credentials civiques (« une personne = une voix » sans registre)

**Date** : 2026-06-12
**Statut** : recherche exploratoire issue d'un débat PO. Aucun engagement de sprint. Candidats routés en fin de document.
**Origine** : analyse croisée programme « L'Avenir en commun » (melenchon2027) x SBFB — rapport complet hors repo dans `C:\Users\FlowUP\Desktop\melenchon2027_textes\_analysis\RAPPORT_FINAL.md` (synthèses + 27 annexes). Le rapport conclut à une « complémentarité conflictuelle » avec trois divergences non réductibles : retrait judiciaire des contenus, identité civile « une personne = une voix », débiteur de la garantie. Ce document explore le montage qui **résout la deuxième** (et partiellement la troisième) sans toucher au protocole.
**Lien vision** : prolonge `vision_communs_idees_factory_builders.md` (curation plurielle, listes signées comme unité de gouvernance, budget « pair frais < 1 min »).

---

## 1. La question initiale et son raffinement

Question PO de départ : « un nœud France où chaque nœud accepté sur le réseau est lié à une carte d'identité ». Raffinée en trois itérations :

1. **Version forte (rejetée)** : identité requise pour rejoindre *le réseau*. Architecturalement impossible (aucun point d'entrée où placer le contrôle ; un gate d'admission = le « trône » que les 5 verrous interdisent ; un fork le retire en une ligne) et contraire au threat model (T5 inclut l'État ; le réseau ne distingue pas un État de droit d'un État autoritaire). Accessoirement contraire au programme politique étudié lui-même (anti-fichage, anti-biométrie).
2. **Version native (retenue comme cadre)** : une institution (État, parti, association) opère *ses* nœuds et conditionne *ses services* à l'identité. Analogue exact du Web : HTTP ne demande de carte à personne, impots.gouv.fr oui. L'institution est un curateur/attestateur/opérateur d'ancres **parmi N** — aucun verrou touché (pas de défaut pré-installé, abonnement volontaire, redondance additive, provenance auteur, subscribed-only).
3. **Frontière dure identifiée** : interactif gateable, contenu non gateable. L'institution peut conditionner tout ce qui requiert la coopération de ses serveurs (vote, pétition, démarche, invite de seed M19, compute) ; elle ne peut pas conditionner la *lecture* de ce qu'elle publie (content-addressing BLAKE3 : une archive publique est re-seedable par quiconque, une app client-side tourne localement pour toujours). Cette frontière tombe au bon endroit : communs de la connaissance non gateables (feature), services personnels gateables (besoin).

**Argument du pari de bienveillance** : la centralisation acceptée ici est une centralisation *avec rampe de sortie*. Si l'institution tourne mal, les usagers se désabonnent, cessent de présenter leur attestation, le réseau et les contenus persistent. Le coût de la malveillance chute de « catastrophique » (opérateur unique sans sortie) à « perte des services qu'elle opérait ». Risque résiduel non protocolaire : la recentralisation douce si l'attention set institutionnel devient le défaut de fait — le contrepoids est la pluralité des listes, jamais hiérarchisées.

## 2. Le pattern produit : lecture libre / écriture authentifiée

C'est la grammaire native du protocole, reproduite au niveau applicatif : n'importe qui lit (fetch + rendu, zéro compte), rien ne s'écrit dans le réseau sans signature Ed25519. Produit type : **app de consultation civique** (pétitions, votes internes, idées) publiée par l'institution — lecture et décomptes ouverts à tous, écriture réservée aux porteurs d'un certificat de membre.

### 2.1 État du code (vérifié, HEAD `8dfb4f7`)

Whitelist bridge actuelle : 15 méthodes (`web/src/bridge/protocol.ts:20`) — `task_submit`, `storage_get/set/list/delete/version`, `pii_redact`, `identity_pubkey`, `node_status`, `browse_list`, `provenance_get/verify`, `feed_cursor_get`, `search`, `proof_card_get`.

- **Existe et sert le scénario** : `identity_pubkey` (l'app connaît la pubkey de son nœud), storage local, lecture du feed, search, provenance.
- **Manque n°1 — signature** : aucune méthode « signe ce payload avec la clé du nœud ». Une app affiche une pubkey mais ne peut pas la *prouver*.
- **Manque n°2 — transport cross-nœud des données applicatives** : `storage_set` est local au nœud ; c'est la dette documentée sbfb-ideas (chaque nœud vote dans son coin). Le sandbox (CSP `connect-src 'none'`, origin opaque) interdit tout autre canal — **par conception**, c'est la garantie centrale anti-exfiltration. Toute solution « juste dans le code des apps » bute sur ce mur, et c'est voulu.

### 2.2 Chemin d'implémentation naturel (si un jour priorisé)

1. **Méthode bridge `feed_append_app_op`** : l'app écrit une op applicative dans le feed signé hash-chaîné de *son* nœud. Authentification gratuite (le feed est signé par construction). `FeedEntry.op` est `serde_json::Value` extensible — ajouter `VoteCast` / `ContributionPublished` ne bump pas `FEED_FORMAT_VERSION` (politique pre-launch explicite). Surface sensible : namespacing par app, caps de taille, rate-limit, vraie phase avec preflight.
2. **Agrégation** : le nœud institutionnel (ou quiconque) lit les feeds des porteurs de certificats et compte ; le décompte est republié lisible par tous.
3. **Prérequis connu** : la propagation cross-nœud des ops de feed a un bug observé live (`SeedAnnounced` ne converge pas, peer_count:0 ~10 min — constat acceptance S75, routé audit gate S75/`sprint76_audit_plan.md`). Même machinerie à fiabiliser d'abord.

### 2.3 MVP sans aucun développement plateforme (« transport humain »)

Toute la couche authentification peut vivre dans le code de l'app dès aujourd'hui (crypto pur JS type noble-ed25519, certificats vérifiés in-app, clés app-level dans `storage_set`) ; seul le transport sort du réseau :

1. l'app signe la contribution et l'affiche (QR / bloc à copier) ;
2. l'adhérent l'envoie par un canal existant (site, mail) ;
3. l'institution agrège hors-réseau, vérifie les signatures, **republie le résultat comme blob/app SBFB avec provenance signée** — décompte lisible et re-vérifiable par tous sur le réseau.

Rustique mais intègre. C'est le bon véhicule pour valider l'usage avant d'investir dans 2.2.

## 3. Le système de credentials : certificat auto-porteur

Principe central : **la signature remplace le registre**. La validation est mathématiquement dans le certificat — vérifiable par quiconque avec la seule clé publique d'émission de l'institution, zéro lookup, zéro annuaire.

### 3.1 Cérémonie d'enrôlement présentielle

1. L'app de l'adhérent génère sa paire de clés **sur son appareil** — la clé privée ne quitte jamais le téléphone, personne (institution comprise) ne la voit jamais.
2. Au local : contrôle de la carte d'identité par un humain ; le téléphone affiche la **pubkey** en QR (rien de secret).
3. Un **PC hors connexion** scanne la pubkey, vérifie l'anti-doublon (3.2), signe le certificat « cette clé = membre vérifié en personne, valide jusqu'au 31/12 », le rend en QR.
4. Au vote, l'app joint le certificat au bulletin. Vérification par n'importe qui = une vérification de signature.

Le PC hors-ligne protège la clé d'émission (jamais exposée à Internet). Renforcement : lecture de la **puce NFC de la CNI** (signée par l'État, vérifiable hors-ligne) en plus du contrôle visuel — la puce prouve la carte authentique, l'humain prouve que le porteur est le titulaire ; chacun seul est contournable, la combinaison non.

### 3.2 L'impossibilité cachée et sa résolution : la non-liaison

**Zéro stockage strict est incompatible avec « une personne = une voix »** : sans aucune trace, rien n'empêche une re-délivrance dans un autre local. Résolution minimale qui préserve l'essentiel :

> On stocke « **cette carte a déjà reçu son certificat cette année** » sans jamais stocker « cette carte = cette clé ».

Liste anti-doublon = entrées `(hash salé du numéro de carte, local, date)`, **privée, hors-ligne** chez l'émetteur, remise à zéro chaque année. Le PC ne journalise jamais quelle clé il a signée. Propriété obtenue : même la saisie du PC ne permet pas de relier une personne à ses votes. Ce qui compte pour la vie privée n'est pas le zéro stockage absolu mais la **non-liaison identité↔clé**.

**Anti-pattern identifié (correction d'une intuition PO)** : un registre *public* de clés publiques n'empêche **aucun** doublon — les doublons vivent dans l'espace des personnes, les clés sont non-liables par construction (c'est la feature). Et un registre public de hash d'identité serait un **oracle d'appartenance** (les numéros de carte ne sont pas secrets → n'importe quel employeur teste « mon salarié est-il membre ? »). La liste anti-doublon ne doit jamais être publique.

### 3.3 Ce qui PEUT être public : le journal de transparence

Modèle **Certificate Transparency** : journal public append-only des **numéros de série** émis (ni clés, ni identités). Vérifiable par tous : si une consultation reçoit plus de bulletins valides que de certificats au journal, la fraude est arithmétiquement visible — y compris une fraude de l'émetteur lui-même. Support naturel : blob signé content-addressé hash-chaîné publié sur le réseau, répliqué par les seeders — réécrire l'historique en douce est impossible, n'importe quel militant en tient un miroir.

Architecture de données en trois lignes :
- **Privé hors-ligne** : la liste anti-doublon (le seul secret du système).
- **Public** : clé(s) d'émission, journal de transparence des séries, liste de révocation (minuscule).
- **Inexistant** : liste des membres, annuaire des clés, lien identité↔clé. *On ne peut pas voler ce qui n'existe pas.*

### 3.4 Niveaux de certificats (assurance proportionnée aux enjeux)

| Niveau | Obtention | Coût logistique | Sert à |
|---|---|---|---|
| 1. Membre | QR par mail (base adhérents existante) | Nul | Consultations courantes, idées, pétitions |
| 2. Carte vérifiée à distance | Lecture NFC de la CNI dans l'app | Faible | Votes internes ordinaires |
| 3. Parrainé | 2 co-signatures de membres vérifiés, en réunion | Faible, distribué | Idem niveau 2, sans CNI à puce |
| 4. Vérifié en personne | Cérémonie au local, PC hors-ligne | Élevé | Scrutins statutaires, désignations |

Même app, même vérification ; le certificat porte son niveau, chaque consultation déclare le niveau exigé. Le niveau mail vaut le statu quo des votes internes de partis (un lien dans un mail) — pas pire que l'existant, jamais suffisant pour les scrutins lourds.

**Variante écartée** : KYC selfie + vivacité type Revolut. (a) Course aux armements perdue contre les deepfakes ; (b) prestataire tiers détenant « opinion politique + biométrie » = double catégorie sensible RGPD art. 9, cauchemar CNIL ; (c) contradiction politique frontale — le programme étudié interdit la reconnaissance faciale.

**Parrainage (niveau 3), bornes anti-Sybil obligatoires** : parrains eux-mêmes vérifiés carte (niveau 4), budget 2-3 parrainages/an/membre, **zéro transitivité** (un parrainé ne parraine pas — cohérent avec le refus gelé du web-of-trust transitif côté protocole), révocation en cascade si un parrain s'avère usine à faux comptes. Sans ces trois bornes, une clique s'auto-certifie exponentiellement.

## 4. Anti-intrusion (par actif, du bijou de famille vers la périphérie)

1. **Clé d'émission nationale** (son vol = fabriquer des électeurs illimités) : **signature à seuil FROST 2-sur-3 ou 3-sur-5** (la clé n'existe en entier nulle part ; un fragment volé ou un initié seul ne servent à rien — brique déjà présente dans l'écosystème : warrant canary FROST DKG) + fragments hors-ligne strict (cartes à puce / machines jamais connectées).
2. **Clés des locaux** : déléguées (signées par la clé nationale), **quota d'émission** (N/mois), expiration courte, révocables. Rayon d'explosion borné par construction.
3. **Journal de transparence** : infalsifiable rétroactivement via le réseau lui-même (blob hash-chaîné répliqué).
4. **Les votes — la défense la plus profonde : rendre l'intrusion sans valeur.** Bulletins signés publiés (pseudonymes) + décompte recomptable par quiconque contre la chaîne de certificats = pirater le serveur de dépouillement ne change rien silencieusement (vérifiabilité de bout en bout : on ne protège pas le serveur, on rend le résultat indépendant du serveur).
5. **Téléphone de l'adhérent** : clé en enclave sécurisée + PIN ; le vol est couvert par révocation + expiration — maillon qu'on répare, pas qu'on blinde.
6. **Supply chain de l'app** : couverture native SBFB — provenance signée liant chaque version au commit source, mise à jour = nouveau hash visible, code vérifiable par quiconque. Une classe au-dessus d'un store qui peut remplacer une app silencieusement.

Stack recommandé : **minimum viable** = dédup hors-ligne + niveaux + expiration annuelle + journal de transparence (couvre ~90 % du risque, trois mesures sur quatre quasi gratuites). **Niveau or** (scrutins statutaires) : + FROST, + quotas locaux, + bulletins publics recomptables, + audit par échantillonnage (recontacter 1-2 % des nouveaux inscrits par trimestre — une fraude de masse ne survit pas statistiquement).

## 5. Scénarios de récupération (le test que la non-liaison survit au mode dégradé)

### 5.1 Local compromis (clé volée)

1. L'attaquant signe 500 faux certificats — mais un certificat n'est valide que journalisé : il doit publier les séries → un moniteur voit le quota du local explosé → alarme en jours.
2. Révocation de la clé du local avec **fenêtre suspecte définie par les dates du journal** (les dates *dans* les certificats sont falsifiables par qui a la clé ; les dates du journal append-only, non). Les certificats du local journalisés avant la fenêtre restent valides.
3. **Dommage collatéral** : les vrais adhérents enrôlés dans la fenêtre perdent leur certificat (l'app le leur affiche via la liste de révocation publique). Leur clé à eux n'a jamais été compromise.
4. **Re-délivrance sans doublon ni liaison** : au guichet, le hash de la carte est retrouvé dans l'anti-doublon avec `(local, date)` ∈ fenêtre révoquée → quel que soit le certificat reçu (on ne sait pas lequel, on n'a pas besoin de savoir), il est forcément mort → re-délivrance autorisée sur la **même pubkey** du téléphone, entrée mise à jour, marquée consommée.
5. **Asymétrie victime/fraudeur** : les victimes légitimes ont une trace anti-doublon (elles sont passées par la cérémonie) ; le fraudeur a signé directement avec la clé volée en court-circuitant le guichet — ses clés n'ont aucune trace → le chemin de réparation lui est fermé.
6. Si un scrutin a eu lieu entre-temps : bulletins publics → recomptage en excluant la fenêtre, décompte corrigé publié.

### 5.2 Téléphone perdu

Personne ne sait quel certificat est le sien (feature) → personne ne peut le révoquer à sa place. Pattern des codes de récupération : **à l'enrôlement, l'app affiche le numéro de série, à noter sur papier** — seul moyen d'auto-révocation. Perte du téléphone → communication de la série (ne révèle rien d'autre) → révocation individuelle au journal → re-délivrance (5.1 étape 4). Papier aussi perdu : une re-délivrance auto-déclarée par an, l'ancien certificat restant derrière le verrou du téléphone et expirant au 31/12.

**Principe traversant : à aucun moment — régime normal, crise, récupération — personne n'a besoin de savoir quel certificat appartient à qui.** La réparation s'appuie sur l'anti-doublon (espace des personnes) et le journal (espace des certificats) sans jamais construire le pont entre les deux. C'est le test qu'échouent la plupart des systèmes d'identité.

## 6. Résidus assumés (à dire dans tout pitch, jamais à promettre)

- **Vote signé ≠ vote secret** : pseudonyme mais public (la clé X a voté Y), et les votes d'une même clé sont liables *entre eux*. Acceptable annoncé ; un scrutin à bulletin secret est un autre produit (signatures aveugles / credentials BBS+ « prouver l'appartenance sans révéler quelle clé » — étage d'après, jamais MVP).
- **Prêt/vente de certificat** : la liaison à la clé du téléphone rend la cession coûteuse (céder son vote = céder toute son identité dans l'app), pas impossible. Le vote papier par procuration a le même résidu.
- **Pas de retrait réseau-wide** : l'institution dé-liste de sa vue et de son décompte ; elle n'efface rien chez les autres. Feature pour la résilience, limite pour la modération — à assumer dans les conditions d'utilisation.
- **Pas de garantie de disponibilité absolue** : le réseau est best-effort ; la garantie opposable, c'est le nœud institutionnel qui la porte.

## 7. Préalables et séquencement

1. **Gate absolu : R-iroh-audit P0.** Pilote fermé, décision gelée. Un déploiement militant est le pire premier public pour une pile non auditée (population qui attire T4-T5). Aucun usage réel avant audit externe.
2. **Maintenant possible** : prototype app « Agora » en mode transport humain (2.3) entre 2-3 nœuds de confiance — valide le parcours certificat, l'auditabilité, le journal, sans une ligne de code plateforme.
3. **Si l'usage prend** : prioriser `feed_append_app_op` + fiabilisation propagation feed (déjà au backlog via SeedAnnounced) dans un sprint normal avec preflight/review.
4. **Hors périmètre SBFB** : toute la logistique d'émission (PC hors-ligne, NFC, locaux, FROST institutionnel) appartient à l'institution, pas au protocole — c'est précisément le point : le protocole n'a pas besoin de savoir que tout ça existe.

## 8. Extension PO (2026-06-12) — l'abonnement comme acte de soutien : contributions déclarées par nœud et par projet

Aujourd'hui, s'abonner = suivre (attention + relais gossip automatique des annonces). L'extension proposée : l'abonnement porte un **profil de contribution** — ce que mon nœud donne au nœud suivi — et son équivalent par projet, une **enveloppe de soutien**. Strictement non-monétaire (lexique kudos : don de ressources volontaire, jamais convertible, jamais conditionnel).

### 8.1 Inventaire des façons d'aider (état du code, HEAD `8dfb4f7`)

**Par projet** :

| Geste | Mécanisme | État |
|---|---|---|
| Garder en ligne | `keep_online` M18 + tag skip-GC | Existe |
| Seeder | seed volontaire S74-E ; seed authentifié invite M19 | Existe |
| Servir les octets | multi-provider `fetch_hash_multi` (automatique si seedé) | Existe |
| Kudos | réputation non-monétaire per-project | Existe |
| Curation | inclusion dans une liste signée | Existe |
| Source | bug report / PR / fork + redeploy re-signé (atelier S74) | Existe |
| Compute orienté projet | worker + allowlist | Granularité à confirmer (preflight) |

**Par nœud** :

| Geste | Mécanisme | État |
|---|---|---|
| S'abonner (attention + relais d'annonces) | subscription gossip | Existe |
| Seeder son catalogue | app par app | Existe (pas de geste groupé) |
| Devenir ancre pour lui | re-pull annuaire + `fetch_and_pin` boot (modèle VPS S75) | Existe (config manuelle `[seed]` + `default_curators`) |
| Offrir du GPU | worker consent L1-L4, caps watts/VRAM/heures, allowlist | Existe (granularité par nœud à confirmer) |
| Attester / inviter | attestations Sybil, invites M19 détenues | Existe |
| Re-curation | reprise dans sa liste signée (2e degré) | Existe |
| Relais réseau bas niveau (relay iroh NAT) | infrastructure iroh | Hors-périmètre SBFB actuel |

### 8.2 Le manque : la déclaration groupée

Trois objets produit, tous composables depuis les primitives existantes :

1. **Mirror one-click d'un catalogue de nœud** : énumérer l'annuaire ingéré → `fetch_and_pin_multi` par app, caps (Mo totaux, N apps) DANS la primitive (§P59).
2. **Profil de contribution attaché à l'abonnement (par nœud)** : à l'acte d'abonnement, déclarer seed-catalogue (avec caps), rôle d'ancre, GPU. Stocké en config locale ; publication éventuelle = opt-in séparé.
3. **Enveloppe de soutien par projet** : keep-online + seed + compute en un geste depuis le panneau Disponibilité.

UX : le panneau Disponibilité (S74-A) contient déjà des CTA « Bientôt » inertes — emplacement prévu ; côté nœud, la page `/node/:id` (S75-F).

### 8.3 Garde-fous (non négociables, hérités des verrous)

- **Volontaire, révocable, plafonné dans la primitive** (§P59) — jamais de contribution par défaut (verrou 3 étendu : un profil de contribution vide est le seul défaut compilable).
- **Best-effort, claim ≠ preuve** : une contribution déclarée est une revendication ; seule la livraison des octets prouve (content-addressing) ; les compteurs (« Toi + N pairs ») ne deviennent jamais une autorité.
- **Seeder ≠ auteur** : contribuer ne confère aucune autorité éditoriale ni de gouvernance sur le nœud/projet aidé.
- **Additive, jamais substitutive** (verrou 2) : un nœud ne peut pas exiger une contribution comme condition d'accès à son contenu public — pas de troc codifié contribution↔privilège.
- **Non-monétaire** : aucune conversion contribution↔kudos automatique sans débat PO dédié (risque de réintroduire une monnaie par la bande — cf. `feedback_kudos_non_monetary`).
- **Vie privée / duress** : déclarer publiquement ce qu'on donne est un signal observable (qui soutient qui) — sensible pour un nœud institutionnel ou militant (§1-3 de ce document). Publication opt-in, jamais implicite ; le profil reste sinon config locale.

### 8.4 La gratification : l'impact rendu visible (sans gamification)

La motivation par l'impact visible est le carburant durable des communs volontaires (précédents : métriques des relais Tor — des faits, pas des points —, compteurs Wikipédia, BOINC/Folding@home). Trois étages de vérité, à ne jamais mélanger :

1. **Impact privé (toujours visible pour soi)** : mesures locales du nœud — apps maintenues joignables et durées, octets servis aux pairs, tâches GPU abouties, disponibilité. Autorité : soi-même (« selon ton nœud »). Zéro enjeu de vie privée.
2. **Claims publics (opt-in, §8.3)** : annonces signées (seed, ancre) visibles dans le casier protocolaire (C10) — revendications, jamais autorité (seul le content-addressing prouve la livraison).
3. **Reconnaissance vérifiée (signée par des tiers)** : les kudos compute existent (hash-chaînés, crédités post-guardrail) ; chaînon manquant = le **« merci signé »** (C11) — remerciement Ed25519 de l'auteur/nœud aidé vers ses soutiens, non-transférable, par relation.

Garde-fous (gelés par cohérence avec kudos/fairness) :

- **Jamais de classement** : un leaderboard public = trône global + effet Matthew + cible de farming Sybil (auto-entraide en boucle entre nœuds d'une même personne). Les surfaces publiques n'affichent que claims + remerciements signés par des tiers, jamais de score agrégé comparable.
- **Pondération récence** (philosophie EMA des kudos), pas de trophées cumulatifs à vie — pas d'aristocratie des premiers arrivés.
- **Aucune conversion** : l'impact n'achète ni privilège, ni priorité, ni monnaie (sinon réinvention du stake — cf. `feedback_kudos_non_monetary`).
- **Privé par défaut**, publication par morceaux choisis.

Maquette « Ton impact » (page privée) :

```
┌─ Ton impact ─────────────────────────────────────┐
│  Ce mois-ci, ton nœud a permis :                 │
│  Disponibilité                                   │
│  · 7 apps maintenues joignables (3 nœuds aidés)  │
│  · « Ideas Hub » : en ligne 47 j d'affilée,      │
│    grâce à toi et 2 autres pairs                 │
│  · 1,2 Go servis à 9 pairs                       │
│  Compute                                         │
│  · 312 tâches GPU abouties · kudos du mois ▓▓▓░  │
│  Reconnaissance                                  │
│  · L'auteur d'« Ideas Hub » a signé un merci     │
│    à ton nœud — 12 juin                          │
│  Visibilité : privé — toi seul vois cette page   │
│  [ Publier une partie… ]                         │
└──────────────────────────────────────────────────┘
```

### 8.5 Comptabilité d'impact : prouvable / mesurable / inconnaissable

Réponse à « voir réellement ce qu'un nœud a permis » (ex. Babel : traductions rendues possibles par son GPU ; accès à un projet rendus possibles par son seed). Trois classes de vérité, à ne jamais confondre dans l'UI :

1. **Prouvable par reçus signés — le compute.** Chaîne déjà cryptographique de bout en bout : tâche signée par le soumetteur → résultat signé par le worker → validation (quorum/guardrails) → kudos hash-chaînés. « Ton GPU a produit N résultats validés pour Babel (~X tokens) » = somme de reçus signés exhibables. Limite volontaire : le *compte* des traductions, jamais leur *contenu* (textes des demandeurs = privés, guardrails PII). Preflight : vérifier l'étiquetage par app des tâches pour l'agrégation par projet.
2. **Mesurable localement — l'hébergement, avec contrefactuel honnête.** Le nœud mesure ce qu'il sert : octets par archive, pairs distincts, fenêtres de disponibilité. « Grâce à toi » strict = **fenêtre seul-hébergeur** : croisement historique SeedRegistry x journal de service local (« seul hébergeur du 2 au 14 mai : 4 accès n'ont existé que par toi »). Propriété d'honnêteté structurelle : le registre best-effort sur-estimant les autres seeders, la détection **sous-crédite** en cas de doute — l'erreur va toujours vers la modestie. Garde-fou : compter sans identifier (volume, jamais liste de node_ids lecteurs) ; publication agrégée et opt-in uniquement.
3. **Inconnaissable par conception — les personnes et l'aval.** Le réseau ne connaît que des node_ids (≠ personnes) ; ce qui se passe *dans* l'app est invisible (sandbox `connect-src 'none'` = zéro analytics in-app possible). **Le pacte de vie privée : un réseau qui ne surveille pas ses lecteurs ne peut pas compter ses lecteurs.** Frontière à revendiquer. L'impact humain au-delà de la mesure passe par le merci signé (C11), pas par un compteur.

**Précision (question PO : « strictement tout ce qu'on permet ? »)** — distinguer actes et effets. L'historique des *actes* d'un nœud peut être strict et total : feed signé (émissions) + reçus de compute (prouvables) + journal local de service (chaque octet servi). L'historique de ses *effets* ne le sera jamais, pour trois raisons structurelles : (a) **transitivité intraçable** — un pair servi re-seede à son tour, ces accès aval sont invisibles ; et marquer les octets par fournisseur pour suivre la chaîne changerait le hash, donc casserait le content-addressing lui-même ; (b) **symétrie du curseur (§8.6)** — le réglage de visibilité qui protège un nœud protège les autres : on ne peut pas exiger leurs journaux pour compléter son historique d'effets ; (c) **aval humain** (classe 3). Formule : les effets sont une *mosaïque de preuves* — mesures locales + tout ce que les autres choisissent de signer (claims niveau 2+, attestations C11) ; couverture croissante par coopération volontaire, jamais totale. Le manque résiduel est la trace en creux du choix de ne pas surveiller.

**Cas d'école Babel (question PO : « voir les traductions permises ? »)** — pour le compute, « ce qu'on permet » est un objet précis : la traduction. Le pivot est la **visibilité de la tâche, choisie par le demandeur** (C14) :

- **Tâches privées (défaut)** : tout compter, ne rien lire. (a) *Reçu par hash* : le reçu signé du worker contient le hash du résultat — preuve « j'ai calculé LA traduction d'empreinte H » sans rétention ; vérifiable si le demandeur exhibe un jour son texte. (b) *Métadonnées non-contenu* : paires de langues, tokens, demandeurs distincts. (c) *Attestations des demandeurs* (extension C11 au-delà du seul auteur d'app, selon leur curseur §8.6).
- **Tâches publiques (choix du demandeur)** : campagnes sur corpus publics (docs, textes libres) — le contenu est liable aux reçus, le panneau d'impact devient cliquable : on lit littéralement les traductions produites par son GPU. Gratification maximale sans surveillance : la publicité du corpus est un choix du demandeur.
- **Symétrie des consentements** : le curseur nœud (§8.6) règle « qui je suis », le drapeau tâche règle « ce que je demande » — chacun appartient à celui que la donnée concerne. L'historique d'impact = l'intersection des consentements.
- **Honnêteté threat-model** : le worker voit le texte pendant le calcul (pas de calcul sur chiffré ici) ; rien n'empêche cryptographiquement un worker malveillant de retenir — d'où la rédaction PII côté client AVANT dispatch (`pii_redact`) : protection en amont, pas promesse du worker.

Maquette détail par app :

```
┌─ Ton impact — Babel ─────────────────────────────┐
│  Mai 2026 · reçus signés                         │
│  · 312 traductions calculées (~840 k tokens)     │
│  · 14 paires de langues · 9 demandeurs           │
│  Tâches publiques (choix du demandeur)           │
│  · Docs SBFB fr→en : 41 traductions    [ Lire ]  │
│  · Corpus associatif : 12              [ Lire ]  │
│  Tâches privées : 259 — comptées, jamais lues    │
│    (reçu = hash signé, vérifiable si le          │
│     demandeur l'exhibe un jour)                  │
│  Hébergement (selon ton nœud)                    │
│  · 86 Mo servis · 9 pairs distincts              │
│  · Seul hébergeur du 2 au 14 mai :               │
│    4 accès n'ont existé que par toi              │
│  Attestations reçues                             │
│  · a7f2… atteste : 47 traductions reçues         │
└──────────────────────────────────────────────────┘
```

### 8.6 Le curseur de visibilité : d'anonyme à identifié, au choix de chaque nœud

Critique PO (2026-06-12) : un « merci » horodaté est trop vague — il ne montre pas réellement ce qu'on permet. Deux réponses combinées :

**(a) Le reçu enrichi.** C11 devient une **attestation de contribution** : reçu structuré signé par le bénéficiaire — { contributeur (pubkey ou « anonyme »), app/hash, période, volumes : tâches validées, tokens, octets servis, fenêtres seul-hébergeur }. « Ton impact » affiche des lignes concrètes exhibables une à une.

```
┌─ Attestation de contribution ────────────────────┐
│  Signée par : auteur de « Babel » (z9d2…)        │
│  Contributeur : ton nœud (pseudonyme)            │
│  Période : mai 2026                              │
│  · 312 résultats GPU validés (~840 k tokens)     │
│  · 86 Mo servis · seul hébergeur du 2 au 14 mai  │
│  ✓ signature vérifiée    [ Exporter le reçu ]    │
└──────────────────────────────────────────────────┘
```

**(b) Le curseur, 5 positions par nœud** (orchestre des primitives existantes : claims opt-in §8, Keyoxide multi-forge du deploy vérifié, attestation civile §3, transport Tor phase 1 pour le niveau 0) :

| Niveau | Nom | Ce qui se voit | Gain / coût |
|---|---|---|---|
| 0 | Anonyme | Rien d'attribuable ; aucune annonce ; Tor recommandé (sinon node_id visible des pairs servis au niveau connexion — honnêteté de transport) | Aide invisible : personne ne sait qui remercier |
| 1 | Discret (défaut) | node_id au transport, zéro claim public ; impact visible en privé seulement | Gratification privée complète, zéro exposition |
| 2 | Contributeur public | Claims signés (seed, ancre), « Toi + N pairs », attestations affichables, casier C10 alimenté | Reconnaissance pseudonyme ; expose « qui soutient quoi » |
| 3 | Reconnu | + nom d'affichage, preuves Keyoxide multi-forge | Réputation portable entre communautés |
| 4 | Identifié | + attestation d'identité civile (§3) | Redevabilité maximale — le nœud institutionnel de §1 est un niveau 4 par nature |

Granularité par audience (avancé) : exceptions par nœud suivi (« envers ideas-hub : Reconnu ; envers le reste : Discret »).

**Deux portes à sens unique (avertissement bloquant en UI)** :
1. L'append-only ne se rétracte pas : redescendre le curseur arrête les claims futurs ; ce qui est déjà dans le feed signé y reste.
2. La rétro-liaison : passer de pseudonyme (2-3) à identifié (4) relie rétroactivement tout l'historique de la clé à l'identité. Alternative à proposer dans le même écran : nouvelle clé pour la vie identifiée, historique pseudonyme orphelin — choix éclairé continuité vs cloisonnement.

Trade-off assumé : l'anonymat coûte la reconnaissance, la reconnaissance coûte l'anonymat — le curseur en transforme le défaut subi en choix par nœud. Défaut = Discret (cohérent §8.3).

```
┌─ Visibilité de ton nœud ─────────────────────────┐
│  Anonyme ─ Discret ─ Public ─ Reconnu ─ Identifié│
│             ▲ actuel                             │
│  En « Discret » : ton impact n'est visible que   │
│  par toi ; aucun claim, pas de remerciements.    │
│  Passer en « Public » publiera tes annonces      │
│  signées — l'historique publié ne se retire pas. │
│  Exceptions par nœud suivi : [ Gérer… ]          │
└──────────────────────────────────────────────────┘
```

### 8.7 Annexe UX — maquettes (session 2026-06-12)

Principes transverses : intentions sans jargon (jamais `fetch_and_pin` ni « M18 » à l'écran) ; « Suivre seulement » pré-sélectionné (verrou 3 traduit en UI) ; coût annoncé avant le oui (Mo, heures) ; GPU pointe vers les niveaux de consentement L1-L4 existants (pas de second système) ; un geste pour tout couper à chaque niveau ; visibilité publique = case séparée jamais implicite.

Dialogue d'abonnement enrichi (`/nodes`, `/node/:id`) :

```
┌──────────────────────────────────────────────────┐
│  Suivre ce nœud                               ✕  │
│  atelier-libre · z3f8…c21a                       │
│  14 apps publiées · vu il y a 2 min              │
│  Ses apps apparaîtront dans ta grille.           │
│  ─── Le soutenir (optionnel) ──────────────────  │
│  (•) Suivre seulement                            │
│  ( ) Suivre et soutenir                          │
│      [x] Héberger son catalogue                  │
│          Plafond : [ 2 Go ▾ ]  Apps : [ 10 ▾ ]   │
│          ≈ téléchargement initial 340 Mo         │
│      [ ] Servir d'ancre (re-publie son annuaire  │
│          au démarrage)                           │
│      [ ] Partager du GPU pour ses tâches         │
│      [ ] Rendre ton soutien visible des autres   │
│           [ Annuler ]        [ Suivre ]          │
└──────────────────────────────────────────────────┘
```

Panneau Disponibilité par projet (évolution S74-A, CTA « Bientôt » activés) :

```
┌─ Disponibilité ──────────────────────────────────┐
│  Hébergé par : Toi + 3 pairs (estimation)        │
│  [x] Garder en ligne sur ce nœud                 │
│  [x] Seeder pour les autres                      │
│  [ ] Exécuter ses tâches GPU en priorité         │
│  Espace utilisé : 48 Mo                          │
│  ── Tout arrêter pour cette app ──               │
└──────────────────────────────────────────────────┘
```

Vue de révocabilité « Ce que tu donnes » (nouvelle, sous Network) :

```
┌─ Ce que tu donnes ───────────────────────────────┐
│  Stockage    1,2 Go / plafond 5 Go   ▓▓▓░░░░░    │
│  GPU         0 h cette semaine       niveau L2   │
│  Par nœud                                        │
│  ├ atelier-libre  seed 10 apps · 340 Mo          │
│  │                    [ Modifier ] [ Arrêter ]   │
│  └ ideas-hub      ancre · annuaire seul          │
│                       [ Modifier ] [ Arrêter ]   │
│  Par app                                         │
│  ├ Ideas Hub          garder en ligne · 12 Mo    │
│  └ Protocol Explorer  seed · 36 Mo               │
│  Visibilité : ton soutien n'est pas public       │
└──────────────────────────────────────────────────┘
```

## 9. Candidats routés (à reprendre dans les débats vision / kickoffs futurs)

| # | Candidat | Nature | Dépendance |
|---|---|---|---|
| C1 | Méthode bridge `feed_append_app_op` (ops applicatives signées, namespacées, rate-limitées) | Feature protocole/daemon | Fiabilisation propagation feed (bug SeedAnnounced, audit S75→S76) |
| C2 | Méthode bridge de signature (prouver `identity_pubkey`, challenge-réponse) | Feature bridge, petite | — |
| C3 | Agrégation cross-nœud d'ops applicatives (sbfb-ideas réellement partagé — candidat vision existant, ce document lui donne un cas d'usage civique) | Feature daemon | C1 |
| C4 | Canal de service authentifié app→nœud-éditeur (template `sbfb/seed/0` : ALPN + Ed25519 + JCS + nonce + invite) | Feature protocole, phase sérieuse | Décision PO sur l'ouverture de la surface |
| C5 | Doc/spec « certificat auto-porteur + journal de transparence » comme pattern d'app recommandé (vérification 100 % in-app, zéro changement protocole) | Doc / exemple d'app | Aucune |
| C6 | Étage crypto avancé : credentials anonymes-uniques (BBS+/blind) pour scrutin secret | Recherche long terme | C1-C4 + maturité |
| C7 | Mirror one-click d'un catalogue de nœud (annuaire → `fetch_and_pin_multi`, caps dans la primitive) | Feature daemon + front | Primitives S74/S75 existantes |
| C8 | Profil de contribution attaché à l'abonnement (par nœud : seed/ancre/GPU) + enveloppe de soutien par projet — publication opt-in | Feature produit transverse (§8) | C7 ; décision PO lexique non-monétaire |
| C9 | Granularité de l'allowlist worker par éditeur/projet (orienter son GPU vers un nœud ou une app) | À confirmer en preflight (granularité actuelle inconnue) | — |
| C10 | Casier protocolaire / onglet « Historique » de nœud : timeline signée vérifiable (feed hash-chaîné + annonces + provenance ; chaîne intacte, anti-rollback, équivocation exhibable). Trois formes : onglet `/node/:id`, app SBFB type sbfb-explorer/Factory Viewer (re-vérification des signatures in-app), observatoire *parmi N* (jamais unique). Hors champ par conception : lectures, abonnements, épinglages non annoncés — casier de déclarations, pas surveillance | Feature daemon+front et/ou app | Granularité `feed_cursor_get` par nœud étranger + exposition octets canoniques au bridge, à confirmer preflight |
| C11 | **Attestation de contribution** (ex-« merci signé », enrichie suite critique PO §8.6.a) : reçu structuré signé par le bénéficiaire { contributeur pubkey-ou-anonyme, app/hash, période, volumes tâches/tokens/octets/fenêtres seul-hébergeur } — raw-op feed 0-bump, non-transférable, par relation, affichée dans « Ton impact » (§8.4) et le casier (C10) | Feature protocole légère | Décision PO lexique (cousin des kudos, mêmes interdits monétaires) ; C12 pour les volumes |
| C13 | Curseur de visibilité par nœud (§8.6.b) : 5 niveaux anonyme→identifié orchestrant des primitives existantes (claims opt-in §8, Keyoxide multi-forge, attestation civile §3, transport Tor niveau 0) + exceptions par audience + avertissements porte-à-sens-unique (append-only, rétro-liaison, option nouvelle-clé) | Feature config daemon + front | C8/C11 ; UX avertissements bloquants |
| C14 | Drapeau de visibilité par tâche compute, choisi par le demandeur (§8.5 cas Babel) : privée (défaut — reçus par hash + métadonnées non-contenu) vs publique (contenu liable aux reçus du worker, panneau d'impact cliquable — campagnes de corpus publics) + reçu-par-hash dans les receipts worker + métadonnées paires-de-langues/tokens | Feature wire task + worker + front | Étiquetage app des tâches (C12) ; décision PO surface wire `Task` (pre-launch : édition libre du canonical) |
| C12 | Comptabilité d'impact (§8.5) : compteurs blob-serve (octets + pairs distincts par hash), agrégation par app des reçus worker + kudos, détection « fenêtre seul-hébergeur » (croisement historique SeedRegistry x journal local, erreur conservatrice) | Feature daemon + worker + front | C8/C10 ; étiquetage app des tâches + events upload iroh-blobs à confirmer preflight |

**Note de cohérence vision** : ce montage est l'instanciation concrète de la « zone d'accord textuelle » identifiée par l'analyse croisée (la mesure « clouds de confiance décentralisés, associatifs et pluriels » du livret numérique) et de la ligne stratégique existante — « arriver avec 10-30 apps réellement utiles, pas avec un manifeste ». Une app de consultation civique à certificats auto-porteurs serait précisément une de ces apps. Il ne crée aucune dépendance du protocole envers une institution : tout vit au-dessus, dans la couche attestation + curation, là où le design l'a toujours prévu.
