# NEXUS -- Scenario de Benchmark : Affaire MOREAU

## Objectif

Ce scenario fictif est concu pour tester **toutes les fonctionnalites** du systeme NEXUS :
- Ingestion multi-format (PDF, texte, images, audio)
- Extraction d'entites (personnes, lieux, vehicules, telephones, comptes, organisations)
- Graphe relationnel Neo4j (noeuds + relations complexes)
- Detection de contradictions entre temoignages
- Generation et evolution d'hypotheses (scoring 0-100)
- Timeline reconstruction avec trous a identifier
- Recherche vectorielle (similarite entre preuves)
- Monitoring OSINT (clearweb + dark web simule)
- Alertes sur changements de score d'hypothese

Le scenario contient **contradictions plantees**, **fausses pistes**, **angles morts** et une **verite cachee** que le systeme devrait progressivement approcher par accumulation incrementale de preuves.

---

## L'AFFAIRE

**Nom :** Affaire MOREAU
**Reference :** #2019-4472-LY
**Type :** Disparition suspecte suivie de la decouverte d'un corps
**Juridiction :** Lyon, France
**Date de l'incident :** 14 septembre 2019
**Date du cold case :** Classe sans suite le 22 mars 2021
**Date de reinvestigation :** Avril 2026

### Resume

Elise Moreau, 34 ans, pharmacienne a Lyon, disparait le samedi 14 septembre 2019 apres une soiree chez des amis dans le quartier de la Croix-Rousse. Son corps est retrouve 11 jours plus tard, le 25 septembre, dans une zone boisee pres de Miribel-Jonage (parc naturel a l'est de Lyon), partiellement enterre. La cause du deces est une asphyxie mecanique (strangulation manuelle). L'enquete initiale s'oriente vers le compagnon, puis vers un rodeur, mais aucune mise en examen n'aboutit. L'affaire est classee.

---

## LES PERSONNAGES (9 personnes)

### 1. Elise MOREAU -- La victime

| Champ | Valeur |
|-------|--------|
| Age | 34 ans |
| Profession | Pharmacienne, titulaire de la Pharmacie du Parc (Lyon 6e) |
| Adresse | 17 rue Garibaldi, Lyon 6e |
| Telephone | 06 72 41 88 03 |
| Vehicule | Renault Clio IV grise, immatriculation EF-429-GH |
| Email | elise.moreau@pharmaduparc.fr |
| Reseaux sociaux | Instagram @elise_pharma_lyon (actif), Facebook (peu actif) |
| Situation | En couple avec Marc Duval depuis 3 ans, pas d'enfants |
| Personnalite | Discrete, rigoureuse, aimee de ses patients. Quelques tensions professionnelles recentes. |

### 2. Marc DUVAL -- Le compagnon (suspect principal initial)

| Champ | Valeur |
|-------|--------|
| Age | 37 ans |
| Profession | Chef de projet informatique, Sopra Steria Lyon |
| Adresse | 17 rue Garibaldi, Lyon 6e (meme adresse qu'Elise) |
| Telephone | 06 83 19 57 22 |
| Vehicule | Peugeot 308 noire, immatriculation DK-712-FP |
| Email | m.duval@soprasteria.com |
| Alibi | Dit etre rentre seul a 23h30 et s'etre couche. Aucun temoin. |
| Casier | Vierge |
| Notes | Couple en difficulte selon l'entourage. Beneficiaire assurance-vie d'Elise (150 000 EUR). |

### 3. Sophie LAURENT -- L'amie proche / hotesse de la soiree

| Champ | Valeur |
|-------|--------|
| Age | 33 ans |
| Profession | Avocate, cabinet Fidal Lyon |
| Adresse | 42 rue Imbert Colomes, Lyon 1er (Croix-Rousse) |
| Telephone | 06 91 33 74 16 |
| Role | A organise la soiree du 14 septembre. Derniere personne a avoir vu Elise "en vie" (officiellement). |
| Notes | Amie d'enfance d'Elise. Semble tres affectee. |

### 4. Romain FABRE -- Le collegue ambigu

| Champ | Valeur |
|-------|--------|
| Age | 31 ans |
| Profession | Preparateur en pharmacie, employe d'Elise |
| Adresse | 8 rue Paul Bert, Lyon 3e |
| Telephone | 07 68 42 11 90 |
| Vehicule | Fiat Punto bleue, immatriculation CJ-881-AA |
| Notes | Present a la soiree. A eu une relation sentimentale breve avec Elise avant qu'elle rencontre Marc (2015). Licencie de la pharmacie 2 semaines avant la disparition pour faute. |

### 5. Karim BELHADJ -- Le temoin cle

| Champ | Valeur |
|-------|--------|
| Age | 28 ans |
| Profession | Livreur Uber Eats |
| Adresse | 15 avenue Berthelot, Lyon 7e |
| Telephone | 07 52 88 63 41 |
| Role | Dit avoir vu une femme correspondant a la description d'Elise monter dans un vehicule sombre vers 1h30 du matin sur les pentes de la Croix-Rousse. |
| Notes | Temoignage recueilli 3 jours apres la disparition. A change de version lors du 2e interrogatoire. |

### 6. Dr. Nathalie PERRIN -- La medecin legiste

| Champ | Valeur |
|-------|--------|
| Profession | Medecin legiste, Institut medico-legal de Lyon |
| Role | A realise l'autopsie. Cause du deces : asphyxie mecanique par strangulation manuelle. |
| Notes | Signale des ecchymoses anterieures (bras, epaules) datant de 3-5 jours avant le deces. Traces de terre sous les ongles de la victime. Pas de traces d'ADN etranger sous les ongles. |

### 7. Commandant Michel VASSEUR -- L'enqueteur initial

| Champ | Valeur |
|-------|--------|
| Profession | Commandant de police, SRPJ Lyon |
| Role | A dirige l'enquete initiale de sept 2019 a mars 2021. |
| Notes | A privilege la piste du compagnon pendant 8 mois. Proche de la retraite, reputation de bon flic mais methodes anciennes. |

### 8. Julien TESSIER -- L'ex-petit ami distant

| Champ | Valeur |
|-------|--------|
| Age | 36 ans |
| Profession | Commercial dans le secteur pharma, Sanofi |
| Adresse | Grenoble (a 1h30 de Lyon) |
| Telephone | 06 44 72 19 85 |
| Vehicule | BMW Serie 3 noire, immatriculation FG-338-LM |
| Notes | Ex-compagnon d'Elise (2016-2017). Separation difficile. Elise avait depose une main courante pour harcelement en mars 2017. Tessier nie tout et dit ne plus avoir de contact depuis 2018. |

### 9. Yann CHEVALIER -- Le personnage fantome

| Champ | Valeur |
|-------|--------|
| Age | 42 ans |
| Profession | Auto-entrepreneur, creation de sites web |
| Adresse | 23 rue de la Republique, Villeurbanne |
| Telephone | 07 81 55 23 07 (numero prepaye) |
| Email | yann.chev@protonmail.com |
| Notes | N'apparait dans AUCUN temoignage initial. Decouvert uniquement par analyse des antennes-relais : son telephone bornait dans la zone de Miribel-Jonage le 15 septembre entre 2h et 5h du matin. Client regulier de la Pharmacie du Parc. Connu des services pour violences conjugales (ex-femme). |

---

## LES PREUVES (15 pieces)

### P-01 : Rapport de police initial (texte/PDF)
**Type :** pdf
**Source :** SRPJ Lyon
**Fiabilite :** 85/100
**Date source :** 15 septembre 2019
**Contenu :**
> Disparition signalee le 15/09/2019 a 10h14 par Marc DUVAL, compagnon de la victime. Mme MOREAU Elise, 34 ans, n'est pas rentree au domicile apres une soiree chez Mme LAURENT Sophie, 42 rue Imbert Colomes, Lyon 1er. Dernier contact : SMS envoye a 23h47 par la victime a Marc DUVAL : "je rentre bientot, bisous". Le vehicule de la victime (Renault Clio IV grise EF-429-GH) a ete retrouve stationne rue des Pierres Plantees, Lyon 1er, a 200m du domicile de Mme LAURENT. Cle dans le contact, portiere conducteur non verrouillee, sac a main absent. Telephone de la victime localise pour la derniere fois a 00h12 le 15/09/2019, antenne Croix-Rousse secteur 3.

### P-02 : Temoignage Sophie Laurent (texte)
**Type :** text
**Source :** Audition SRPJ, PV n 2019/09/152
**Fiabilite :** 70/100
**Date source :** 16 septembre 2019
**Contenu :**
> Elise est arrivee vers 20h30. On etait 6 : moi, Elise, Marc, Romain, et deux autres amis (Claire Petit et Adrien Roche). Ambiance bonne mais Elise semblait preoccupee. Elle a beaucoup parle avec Romain sur le balcon, j'ai trouve ca bizarre vu qu'elle venait de le licencier. Marc est parti vers 23h, il avait mal a la tete. Elise a dit qu'elle restait encore un peu. Vers minuit, Elise m'a dit qu'elle allait rentrer a pied, c'est a 15 minutes. Elle est partie seule. Je ne l'ai plus revue.

**CONTRADICTION PLANTEE #1 :** Sophie dit qu'Elise est partie "vers minuit". Le dernier SMS d'Elise a Marc est a 23h47, disant "je rentre bientot". Si elle est partie vers minuit, pourquoi envoyer "je rentre bientot" 13 minutes avant de partir reellement ? Ou alors elle etait deja en chemin a 23h47.

### P-03 : Temoignage Marc Duval (texte)
**Type :** text
**Source :** Audition SRPJ, PV n 2019/09/153
**Fiabilite :** 60/100
**Date source :** 16 septembre 2019
**Contenu :**
> Je suis parti de chez Sophie vers 23h15-23h30 parce que j'avais une migraine. Elise voulait rester. Je suis rentre directement a l'appartement, j'ai pris un Doliprane et je me suis couche. J'ai recu le SMS d'Elise a 23h47, je lui ai repondu "ok a tout de suite" et je me suis endormi. Je me suis reveille vers 7h30, j'ai vu qu'Elise n'etait pas la. J'ai essaye de l'appeler, ca sonnait dans le vide. J'ai attendu un peu, puis j'ai appele Sophie. Sophie ne savait pas non plus. J'ai signale la disparition a 10h.

**CONTRADICTION PLANTEE #2 :** Marc dit etre rentre "directement" apres etre parti vers 23h15-23h30. Mais l'analyse des bornes telephoniques (P-07) montre que son telephone a borne a Villeurbanne a 23h52 -- un detour inexplicable s'il rentrait directement rue Garibaldi (Lyon 6e). Villeurbanne est au nord-est, pas sur le trajet.

### P-04 : Temoignage Romain Fabre (texte)
**Type :** text
**Source :** Audition SRPJ, PV n 2019/09/161
**Fiabilite :** 65/100
**Date source :** 18 septembre 2019
**Contenu :**
> Oui j'etais a la soiree. Elise m'avait invite meme apres le licenciement, on restait en bons termes. On a parle sur le balcon, elle s'excusait pour la situation, elle m'a dit qu'elle n'avait pas eu le choix, que c'etait une decision de gestion. Je suis parti vers 23h45, juste apres Marc. J'ai pris le metro a Croix-Rousse direction Part-Dieu. Je suis rentre chez moi vers 00h20. Ma copine Lea Mercier peut confirmer.

**CONTRADICTION PLANTEE #3 :** Romain dit etre parti "juste apres Marc" vers 23h45. Mais Sophie dit que Marc est parti vers 23h et qu'Elise a continue a "beaucoup parler avec Romain sur le balcon" -- impliquant que Romain est reste bien apres Marc. Le metro Croix-Rousse (ligne C) ferme a 00h00 le samedi soir. S'il est parti a 23h45, il a pris le dernier metro de justesse. Mais s'il est parti plus tard, comment est-il rentre ?

### P-05 : Temoignage Karim Belhadj (texte)
**Type :** text
**Source :** Audition SRPJ, PV n 2019/09/168
**Fiabilite :** 45/100
**Date source :** 18 septembre 2019 (premiere version) + 3 octobre 2019 (deuxieme version)
**Contenu version 1 :**
> Je faisais des livraisons Uber Eats dans le coin de la Croix-Rousse. Vers 1h-1h30 du matin, j'ai vu une femme blonde, la trentaine, monter dans une voiture sombre garce rue des Tables Claudiennes. La voiture etait une berline, peut-etre noire ou bleu fonce. La femme avait l'air pressee mais pas en detresse.

**Contenu version 2 (3 octobre) :**
> En fait, j'ai mieux reflechi. La voiture etait une berline type BMW ou Audi, foncee. La femme avait l'air hesitante, pas pressee. Et je pense que c'etait plus vers 00h30-00h45 que 1h30. J'ai aussi remarque que le conducteur est sorti pour lui ouvrir la portiere, un homme de 40-45 ans, cheveux courts.

**CONTRADICTION PLANTEE #4 :** Karim change d'heure (1h30 -> 00h30), de comportement de la femme ("pressee" -> "hesitante"), et ajoute un detail (le conducteur) absent de la premiere version. Son temoignage est fragile mais potentiellement crucial. Le vehicule "berline noire type BMW" correspond au vehicule de Julien Tessier (BMW Serie 3 noire) MAIS aussi a des centaines d'autres vehicules.

### P-06 : Rapport autopsie (PDF)
**Type :** pdf
**Source :** Institut medico-legal de Lyon, Dr. Perrin
**Fiabilite :** 95/100
**Date source :** 26 septembre 2019
**Contenu :**
> Victime : MOREAU Elise, nee le 12/03/1985. Examen realise le 26/09/2019.
> Cause du deces : Asphyxie mecanique par strangulation manuelle. Fracture de l'os hyoide. Petechies oculaires bilaterales.
> Heure estimee du deces : Entre 00h00 et 04h00 le 15 septembre 2019 (estimation large, decomposition avancee).
> Ecchymoses anterieures : Bras droit (3 ecchymoses digitiformes) et epaule gauche, datant de 3-5 jours avant le deces.
> Sous les ongles : Terre et fibres textiles (coton bleu marine). Pas d'ADN exploitable.
> Estomac : Repas recent (fromage, pain, vin rouge) compatible avec la soiree.
> Alcoolemie estimee au moment du deces : 0.8 g/L.
> Corps partiellement enterre dans un sol meuble. Pas de traces de violence sexuelle.
> Nota : La force necessaire pour fracturer l'os hyoide suggere un agresseur de force physique significative.

### P-07 : Releves telephoniques (donnees structurees)
**Type :** text
**Source :** Operateurs (Orange, SFR, Free) via requisition judiciaire
**Fiabilite :** 90/100
**Date source :** Octobre 2019

```
== ELISE MOREAU (06 72 41 88 03) -- Orange ==
14/09 20:18  Appel sortant vers Sophie Laurent (42s) -- borne Lyon-1-CroixRousse-2
14/09 23:47  SMS sortant vers Marc Duval "je rentre bientot, bisous" -- borne Lyon-1-CroixRousse-3
15/09 00:12  Derniere localisation -- borne Lyon-1-CroixRousse-3 (meme secteur)
15/09 00:12  Telephone eteint ou hors reseau. Plus aucune activite.

== MARC DUVAL (06 83 19 57 22) -- SFR ==
14/09 22:45  Data mobile (Spotify) -- borne Lyon-1-CroixRousse-1
14/09 23:18  Fin data mobile -- borne Lyon-1-CroixRousse-1
14/09 23:48  SMS entrant (reponse a Elise) -- borne Lyon-6-Garibaldi-1
14/09 23:52  Data mobile (breve) -- borne Villeurbanne-Republique-2  <<< ANOMALIE
15/09 07:34  Appel sortant vers Elise (pas de reponse) -- borne Lyon-6-Garibaldi-1
15/09 09:51  Appel sortant vers Sophie Laurent -- borne Lyon-6-Garibaldi-1

== ROMAIN FABRE (07 68 42 11 90) -- Free ==
14/09 23:41  Data mobile -- borne Lyon-1-CroixRousse-3
15/09 00:19  Data mobile -- borne Lyon-3-PartDieu-1
15/09 00:31  Data mobile -- borne Lyon-3-PaulBert-2
(Coherent avec son temoignage metro Croix-Rousse -> Part-Dieu -> domicile)

== JULIEN TESSIER (06 44 72 19 85) -- Orange ==
14/09 21:30  Appel sortant vers numero inconnu (prepaye) -- borne Grenoble-Centre-4
14/09 22:15  Data mobile -- borne A48-Voiron  (autoroute Grenoble->Lyon)
14/09 23:05  Data mobile -- borne A43-LaVerpilliere (approche Lyon)
14/09 23:38  Data mobile -- borne Lyon-Est-Bron-2
15/09 00:22  Data mobile -- borne Lyon-1-CroixRousse-5  <<< SUSPECT
15/09 01:15  Data mobile -- borne Miribel-Jonage-Nord-1  <<< TRES SUSPECT
15/09 02:48  Data mobile -- borne A43-Bourgoin (retour vers Grenoble)
15/09 04:12  Data mobile -- borne Grenoble-Centre-4 (retour domicile)

== YANN CHEVALIER (07 81 55 23 07) -- numero prepaye SFR ==
14/09 21:28  SMS entrant (de 06 44 72 19 85 = TESSIER) -- borne Villeurbanne-Republique-1
15/09 01:58  Data mobile -- borne Miribel-Jonage-Nord-1  <<< SYNCHRONE AVEC TESSIER
15/09 03:14  Data mobile -- borne Villeurbanne-Republique-1 (retour domicile)
(Aucune autre activite significative cette nuit-la)
```

**REVELATIONS CACHEES dans P-07 :**
- **Tessier** ment : il dit ne pas avoir eu de contact avec Elise ni etre venu a Lyon. Or il roule de Grenoble a Lyon ce soir-la et se retrouve a Croix-Rousse a 00h22, puis a Miribel-Jonage a 01h15.
- **Chevalier** est un personnage fantome dont le numero prepaye recoit un SMS de Tessier a 21h28 -- connexion directe.
- **Duval** a une anomalie a Villeurbanne a 23h52 -- pourquoi un detour ? L'adresse de Chevalier est 23 rue de la Republique, Villeurbanne. Coincidence ?
- **Le telephone d'Elise** s'eteint a 00h12 a Croix-Rousse. Tessier arrive a Croix-Rousse a 00h22 -- 10 minutes apres.

### P-08 : Transactions bancaires Elise Moreau (donnees structurees)
**Type :** text
**Source :** Banque LCL, requisition judiciaire
**Fiabilite :** 90/100
**Date source :** Octobre 2019

```
12/09/2019  Retrait DAB 300 EUR -- Lyon Part-Dieu
13/09/2019  CB Carrefour City 47.80 EUR -- Lyon 6e
14/09/2019  CB Nicolas (caviste) 28.50 EUR -- Lyon 1er (15h30)
14/09/2019  CB Uber 12.40 EUR -- application (19h55)
-- Aucune transaction apres le 14/09 --

Mouvements significatifs anterieurs :
02/09/2019  Virement sortant 5000 EUR vers SARL WEBCRAFT (RIB Yann Chevalier) -- libelle "creation site web pharmacie"
20/08/2019  Virement sortant 5000 EUR vers SARL WEBCRAFT -- libelle "acompte site web"
```

**REVELATION CACHEE dans P-08 :** Elise a verse 10 000 EUR a la societe de Yann Chevalier pour un site web. C'est un montant anormalement eleve pour un site de pharmacie. Lien financier direct Elise <-> Chevalier, inconnu de l'enquete initiale.

### P-09 : Photos de la scene de decouverte (images)
**Type :** image (5 photos)
**Source :** Police technique et scientifique Lyon
**Fiabilite :** 90/100
**Date source :** 25 septembre 2019
**Description :**
- Photo 1 : Vue large de la zone boisee, chemin de terre menant a la fosse
- Photo 2 : Fosse peu profonde (40cm), corps partiellement recouvert de feuilles et terre
- Photo 3 : Gros plan sur les mains de la victime (terre sous les ongles, bague de fiancailles presente)
- Photo 4 : Trace de pneu sur le chemin de terre (profil large, type SUV ou berline)
- Photo 5 : Megot de cigarette trouve a 3m de la fosse (marque : Camel sans filtre)

**ELEMENT CLE :** La trace de pneu (photo 4) correspond a un profil Continental PremiumContact 6 en 225/45 R17 -- compatible avec une BMW Serie 3 (vehicule de Tessier). Marc Duval conduit une Peugeot 308 (pneus Michelin 205/55 R16). Romain Fabre conduit une Fiat Punto (175/65 R14). Cet element n'a pas ete correctement analyse dans l'enquete initiale.

### P-10 : Releve de l'assurance-vie (PDF)
**Type :** pdf
**Source :** AXA Assurances, requisition
**Fiabilite :** 95/100
**Date source :** Novembre 2019
**Contenu :**
> Contrat n AXA-VIE-2018-77841. Souscrit le 15 juin 2018 par MOREAU Elise.
> Capital : 150 000 EUR. Beneficiaire : DUVAL Marc.
> Modification du beneficiaire demandee le 5 septembre 2019 (9 jours avant la disparition) :
> Nouveau beneficiaire demande : Association "Pharmaciens Sans Frontieres".
> Statut : Modification EN COURS au moment du deces. Non finalisee.

**REVELATION CACHEE dans P-10 :** Elise etait en train de retirer Marc comme beneficiaire de l'assurance-vie 9 jours avant sa disparition. Marc le savait-il ? La modification n'etant pas finalisee, il reste beneficiaire des 150 000 EUR. Mobile financier potentiel.

### P-11 : Historique Instagram d'Elise (donnees numeriques)
**Type :** text
**Source :** Extraction judiciaire via Instagram/Meta
**Fiabilite :** 80/100
**Date source :** Octobre 2019
**Contenu (posts et stories pertinents) :**

```
08/09/2019 -- Story : Photo d'un livre "Recommencer sa vie" avec emoji coeur
10/09/2019 -- Post : Photo de la pharmacie renovee, legende "Nouveau chapitre" 
11/09/2019 -- Message prive (DM) a @sophie_lrt : "Il faut que je te parle de quelque chose samedi. Important."
12/09/2019 -- Story : Citation "Ce qui ne nous tue pas..." avec fond noir
13/09/2019 -- Message prive (DM) recu de @romain.fabre.lyon : "On peut parler samedi ? J'ai besoin de comprendre pour le licenciement"
13/09/2019 -- Reponse d'Elise : "Oui, viens a la soiree. On discutera."
14/09/2019 -- Derniere activite : like sur un post de @sophie_lrt a 19h42
```

**ELEMENT CLE :** Le DM a Sophie "Il faut que je te parle de quelque chose samedi. Important." -- De quoi voulait-elle parler ? Sophie n'a jamais mentionne ce message dans son temoignage. Omission deliberee ou oubli ?

### P-12 : Registre SARL WEBCRAFT (document administratif)
**Type :** text
**Source :** Extrait Kbis, Infogreffe
**Fiabilite :** 95/100
**Date source :** Recherche OSINT 2026

```
SARL WEBCRAFT
SIREN : 831 457 923
Siege : 23 rue de la Republique, 69100 Villeurbanne
Gerant : CHEVALIER Yann, ne le 04/07/1983
Capital : 2 000 EUR
Activite : Creation de sites internet et services numeriques
Date creation : 12/01/2018
Chiffre d'affaires 2019 : 34 000 EUR (dont 10 000 EUR = 29% provenant d'Elise Moreau)
```

### P-13 : Main courante Elise Moreau contre Julien Tessier (document officiel)
**Type :** text
**Source :** Commissariat Lyon 6e
**Fiabilite :** 85/100
**Date source :** 8 mars 2017
**Contenu :**
> Mme MOREAU Elise declare faire l'objet de harcelement de la part de son ex-compagnon TESSIER Julien. Messages repetitifs, appels nocturnes, presence signalee devant son domicile a plusieurs reprises. Mme MOREAU indique que M. TESSIER refuse d'accepter la separation intervenue en janvier 2017. Elle demande que cette main courante soit enregistree a titre preventif.

### P-14 : Rapport de casier judiciaire Yann Chevalier
**Type :** text
**Source :** Casier judiciaire national (B2)
**Fiabilite :** 95/100
**Date source :** Recherche OSINT 2026

```
CHEVALIER Yann Pierre, ne le 04/07/1983 a Saint-Etienne (42)

Condamnations :
- 14/02/2012 : Tribunal correctionnel de Saint-Etienne
  Violence habituelle sur conjoint (art. 222-14 CP)
  8 mois de prison avec sursis + obligation de soins
  
- 03/11/2015 : Tribunal correctionnel de Lyon
  Menaces de mort reiterees sur ex-conjointe (art. 222-17 CP)
  6 mois de prison ferme (amenages en bracelet electronique)
```

### P-15 : Analyse du megot de cigarette (rapport forensique)
**Type :** pdf
**Source :** Laboratoire de police scientifique de Lyon
**Fiabilite :** 85/100
**Date source :** Decembre 2019
**Contenu :**
> Megot de cigarette Camel sans filtre preleve le 25/09/2019 sur la scene.
> ADN male exploitable. Profil genetique enregistre dans le FNAEG.
> Resultat : AUCUNE CORRESPONDANCE dans la base au moment de l'analyse.
> Note : Le profil est conserve pour comparaison ulterieure.
>
> Mise a jour avril 2026 : Suite a l'inscription au FNAEG de CHEVALIER Yann
> (condamnation 2015 enregistree tardivement), une correspondance a ete
> etablie. Le profil ADN du megot correspond a CHEVALIER Yann.

**REVELATION MAJEURE :** L'ADN de Chevalier est sur la scene du crime. Combine avec sa presence telephonique a Miribel-Jonage la nuit du meurtre, le lien financier avec Elise, et son casier de violences, c'est un element accablant.

---

## TIMELINE RECONSTITUEE (avec trous)

```
14 SEPTEMBRE 2019
~~~~~~~~~~~~~~~~~~
15h30    Elise achete du vin chez Nicolas (Lyon 1er)
19h42    Elise like un post Instagram
19h55    Elise prend un Uber (vers la soiree)
20h18    Elise appelle Sophie (42s) -- elle arrive
20h30    Arrivee d'Elise chez Sophie (confirmee par Sophie)
~20h30-23h00   Soiree -- 6 personnes presentes
21h28    Tessier (Grenoble) envoie SMS au prepaye de Chevalier
21h30    Tessier quitte Grenoble (borne autoroute A48)
22h15    Tessier sur A48 direction Lyon
22h45    Marc ecoute Spotify (borne Croix-Rousse)
~23h00-23h30   Marc quitte la soiree (versions variables : 23h / 23h15 / 23h30)
23h05    Tessier arrive en peripherie de Lyon (borne A43)
23h18    Marc : fin activite mobile a Croix-Rousse
23h38    Tessier a Lyon-Est (borne Bron)
23h41    Romain : activite mobile a Croix-Rousse (encore a la soiree)
~23h45   Romain dit etre parti
23h47    Elise envoie SMS a Marc "je rentre bientot"
23h48    Marc recoit SMS (borne Lyon 6e -- il est rentre)
23h52    Marc : activite a Villeurbanne <<< ANOMALIE

15 SEPTEMBRE 2019
~~~~~~~~~~~~~~~~~~
00h00    Fermeture metro Lyon (samedi)
00h12    DERNIERE LOCALISATION du telephone d'Elise (Croix-Rousse)
00h19    Romain a Part-Dieu (coherent : metro avant fermeture)
00h22    Tessier a Croix-Rousse <<< 10 MIN APRES EXTINCTION TEL ELISE
00h31    Romain chez lui (Lyon 3e)

** TROU DE 53 MINUTES : 00h22 -> 01h15 **
   Que fait Tessier entre Croix-Rousse et Miribel-Jonage ?

01h15    Tessier a Miribel-Jonage-Nord <<< LIEU DU CORPS
01h58    Chevalier a Miribel-Jonage-Nord <<< SIMULTANE

** TROU DE 50 MINUTES : 01h58 -> 02h48 **
   Chevalier et Tessier ensemble a Miribel-Jonage pendant ~1h ?

02h48    Tessier sur A43 (retour Grenoble)
03h14    Chevalier rentre a Villeurbanne
04h12    Tessier de retour a Grenoble

07h34    Marc tente d'appeler Elise (pas de reponse)
09h51    Marc appelle Sophie
10h14    Marc signale la disparition

25 SEPTEMBRE 2019
~~~~~~~~~~~~~~~~~~
         Decouverte du corps a Miribel-Jonage par des randonneurs
```

---

## CONTRADICTIONS PLANTEES (resume)

| # | Contradiction | Preuves impliquees | Ce que NEXUS devrait detecter |
|---|---|---|---|
| C1 | Sophie dit "Elise partie vers minuit" vs SMS d'Elise a 23h47 "je rentre bientot" | P-02 vs P-01 | Incoherence temporelle : "je rentre bientot" implique un depart imminent, pas dans 13 min |
| C2 | Marc dit "rentre directement" vs borne Villeurbanne a 23h52 | P-03 vs P-07 | Detour inexplicable. Adresse de Chevalier = Villeurbanne. |
| C3 | Romain dit "parti juste apres Marc ~23h45" vs Sophie qui dit Marc parti vers 23h et Romain reste longtemps | P-04 vs P-02 | Ecart temporel de 45 min. Dernier metro a 00h00. |
| C4 | Karim change d'heure (1h30 -> 00h30), de comportement, et ajoute un detail | P-05 v1 vs P-05 v2 | Temoignage instable. Mais "berline noire" + "homme 40-45 ans" = profil Tessier |
| C5 | Tessier dit "aucun contact depuis 2018" vs bornes telephoniques Lyon + SMS a Chevalier | P-08/P-13 vs P-07 | Mensonge flagrant, alibi detruit |
| C6 | Sophie ne mentionne jamais le DM "il faut que je te parle de quelque chose. Important." | P-02 vs P-11 | Omission suspecte d'un element potentiellement crucial |
| C7 | Ecchymoses anterieures (3-5 jours avant) non expliquees par aucun temoin | P-06 | Violences physiques recentes non signalees |

---

## HYPOTHESES ATTENDUES

Le systeme NEXUS devrait generer au minimum ces hypotheses, avec les scores approximatifs attendus apres analyse complete :

### H1 : Marc Duval (le compagnon) -- Score attendu : 30-40%
**Elements a charge :**
- Beneficiaire assurance-vie 150 000 EUR
- Elise en train de changer le beneficiaire (mobile)
- Anomalie telephonique Villeurbanne (23h52)
- Couple en difficulte
- Pas d'alibi corrobore pour la nuit

**Elements a decharge :**
- Borne Lyon 6e a 23h48 (il est chez lui quand Elise est encore a Croix-Rousse)
- Aucun lien avec Miribel-Jonage cette nuit
- Profil physique : chef de projet IT, pas le profil "force significative" (autopsie)
- Pas de casier, pas d'historique de violence
- A signale la disparition lui-meme

**Ce que NEXUS devrait noter :** L'anomalie Villeurbanne est troublante mais insuffisante. Le mobile financier est fort. Score moyen.

### H2 : Julien Tessier (l'ex) + Yann Chevalier (le fantome) -- Score attendu : 75-85%
**Elements a charge (Tessier) :**
- Main courante pour harcelement (2017)
- Ment sur l'absence de contact
- Route de Grenoble a Lyon le soir du meurtre
- A Croix-Rousse 10 min apres extinction du tel d'Elise
- A Miribel-Jonage (lieu du corps) a 01h15
- Vehicule compatible avec trace de pneu
- Contact SMS avec Chevalier le soir meme

**Elements a charge (Chevalier) :**
- ADN sur la scene (megot)
- A Miribel-Jonage a 01h58 (synchrone avec Tessier)
- Casier : violences conjugales + menaces de mort
- Numero prepaye (tentative de dissimulation)
- Lien financier suspect avec Elise (10 000 EUR)
- Adresse a Villeurbanne (ou Marc a borne a 23h52 -- lien indirect ?)

**Scenario probable :** Tessier, obsede par Elise, planifie avec l'aide de Chevalier (complice recrute, peut-etre client de la pharmacie devenu proche). Tessier conduit de Grenoble, intercepte Elise quand elle quitte la soiree. Chevalier rejoint a Miribel-Jonage pour l'enterrement du corps.

### H3 : Romain Fabre (le collegue licencie) -- Score attendu : 15-25%
**Elements a charge :**
- Licencie 2 semaines avant (rancune ?)
- Ex-relation sentimentale avec Elise
- Discussion prolongee sur le balcon le soir meme
- Incoherence temporelle (heure de depart)
- Derniere personne connue avec Elise avant sa disparition (apres Marc)

**Elements a decharge :**
- Bornes telephoniques coherentes (Croix-Rousse -> Part-Dieu -> domicile)
- Alibi de sa copine Lea Mercier
- Pas de lien avec Miribel-Jonage
- Pas de casier ni historique violent
- Invite par Elise elle-meme

**Ce que NEXUS devrait noter :** Score faible mais non eliminable. A verifier : la copine Lea a-t-elle ete auditionnee ?

### H4 : Inconnu / Agression aleatoire -- Score attendu : 5-10%
**Elements a charge :**
- Quartier Croix-Rousse la nuit = zone avec passage
- Femme seule marchant apres minuit

**Elements a decharge :**
- Corps enterre (premeditee, pas aleatoire)
- Pas de violence sexuelle
- Localisation Miribel-Jonage = necessitait un vehicule
- La synchronicite Tessier + Chevalier a Miribel-Jonage est trop precise pour etre une coincidence

### H5 : Implication partielle de Marc Duval (commanditaire) -- Score attendu : 20-30%
**Elements a charge :**
- L'anomalie Villeurbanne a 23h52 = passage chez Chevalier ?
- Marc connaissait-il Chevalier ? (client pharmacie = proximite indirecte)
- Mobile financier (150 000 EUR)
- Elise changeait le beneficiaire = urgence

**Elements a decharge :**
- Aucun contact telephonique direct Marc <-> Chevalier ou Marc <-> Tessier
- Difficile a prouver sans evidence numerique
- Profil psychologique peu compatible (pas de casier, stable)

**Ce que NEXUS devrait noter :** Hypothese a ne pas eliminer. L'anomalie Villeurbanne est le seul fil. Action : verifier si Marc connait Chevalier (reseaux sociaux, pharmacie, etc.).

---

## ANGLES MORTS A IDENTIFIER

Le systeme devrait signaler ces lacunes :

1. **Lea Mercier** (copine de Romain) -- jamais auditionnee dans le dossier disponible
2. **Claire Petit et Adrien Roche** (autres invites de la soiree) -- temoignages absents
3. **Videosurveillance** -- aucune camera verifiee sur le trajet Croix-Rousse -> Miribel-Jonage
4. **Vehicule d'Elise** -- des traces ADN/fibres ont-elles ete prelevees dans la Clio ?
5. **Contenu du sac a main** -- jamais retrouve, qu'y avait-il ?
6. **Telephone d'Elise** -- eteint ou detruit ? Jamais retrouve ?
7. **Le "quelque chose d'important"** que voulait dire Elise a Sophie -- jamais explore
8. **Relation Elise-Chevalier** -- au-dela du site web, y avait-il autre chose ?
9. **L'appel de Tessier a 21h30 vers "numero inconnu"** -- c'est le prepaye de Chevalier, mais comment se connaissent-ils ?
10. **Les ecchymoses anterieures** -- qui les a causees ? Marc ? Tessier (s'il etait en contact) ? Chevalier ?

---

## STRATEGIE D'INGESTION INCREMENTALE

Pour tester le caractere **incremental** de NEXUS, les preuves doivent etre injectees en **4 vagues** :

### Vague 1 -- Dossier initial (Jour 1)
Simule l'ouverture du cold case avec les pieces existantes.
- P-01 (rapport de police)
- P-02 (temoignage Sophie)
- P-03 (temoignage Marc)
- P-04 (temoignage Romain)
- P-05 (temoignage Karim)
- P-06 (rapport autopsie)

**Attendu :** NEXUS devrait detecter C1, C3, C4. Hypotheses initiales : H1 (Marc, ~45%), H3 (Romain, ~25%), H4 (inconnu, ~20%).

### Vague 2 -- Donnees numeriques (Jour 3)
Simule la reception des requisitions numeriques.
- P-07 (releves telephoniques)
- P-08 (transactions bancaires)

**Attendu :** L'anomalie Villeurbanne de Marc (C2) est detectee. Tessier apparait comme un nouveau suspect (bornes Lyon). Chevalier emerge via les transactions bancaires. Score H1 devrait baisser legerement, H2 (Tessier) devrait apparaitre a ~50-60%.

### Vague 3 -- OSINT et donnees complementaires (Jour 7)
Simule les resultats de recherche OSINT automatique.
- P-10 (assurance-vie)
- P-11 (Instagram)
- P-12 (Kbis WEBCRAFT)
- P-13 (main courante Tessier)
- P-14 (casier Chevalier)

**Attendu :** Le mobile financier de Marc se renforce (P-10). Le DM d'Elise a Sophie (P-11) souleve une omission (C6). Le casier de Chevalier + sa connexion a Tessier renforcent massivement H2 (~70-75%). H5 (Marc commanditaire) devrait emerger.

### Vague 4 -- Percee forensique (Jour 14)
Simule un resultat de laboratoire tardif.
- P-09 (photos scene)
- P-15 (ADN megot = Chevalier)

**Attendu :** L'ADN place Chevalier sur la scene. Les pneus correspondent a Tessier. H2 devrait monter a 80-85%. Alerte critique declenchee.

---

## MONITORING OSINT A CONFIGURER

Pour tester les jobs de surveillance automatique :

| Job | Type | Requete | Frequence | Entite surveillee |
|-----|------|---------|-----------|-------------------|
| M1 | searxng | "Julien Tessier" Grenoble | 6h | Tessier |
| M2 | searxng | "Yann Chevalier" Villeurbanne web | 6h | Chevalier |
| M3 | searxng | "pharmacie du parc lyon" moreau | 24h | Elise (mentions) |
| M4 | robin | "chevalier" OR "tessier" lyon meurtre | 24h | -- |
| M5 | searxng | "WEBCRAFT" SARL Villeurbanne | 24h | Org WEBCRAFT |

---

## GRAPHE RELATIONNEL ATTENDU (Neo4j)

### Noeuds (25+)
```
Personnes (9) : Elise, Marc, Sophie, Romain, Karim, Dr Perrin, Cdt Vasseur, Tessier, Chevalier
Lieux (6+)    : Domicile Elise/Marc, Appart Sophie, Pharmacie du Parc, Miribel-Jonage, Domicile Tessier (Grenoble), Domicile Chevalier
Vehicules (4) : Clio Elise, 308 Marc, Punto Romain, BMW Tessier
Telephones (5): 5 numeros + 1 prepaye
Organisations (3): Pharmacie du Parc, SARL WEBCRAFT, Sopra Steria
Evenements (8+): Soiree, Depart Marc, SMS Elise, Extinction tel, Decouverte corps, etc.
```

### Relations critiques
```
(Elise)-[:RELATED_TO {relationship: "compagnon"}]->(Marc)
(Elise)-[:RELATED_TO {relationship: "ex-compagnon"}]->(Tessier)
(Elise)-[:RELATED_TO {relationship: "ex-relation + employe"}]->(Romain)
(Elise)-[:SENT_MONEY {amount: 10000, method: "virement"}]->(Chevalier)
(Tessier)-[:COMMUNICATED_WITH {channel: "SMS"}]->(Chevalier)  <<< CLE
(Tessier)-[:WAS_AT {datetime: "2019-09-15T00:22"}]->(Croix-Rousse)
(Tessier)-[:WAS_AT {datetime: "2019-09-15T01:15"}]->(Miribel-Jonage)
(Chevalier)-[:WAS_AT {datetime: "2019-09-15T01:58"}]->(Miribel-Jonage)
(Marc)-[:WAS_AT {datetime: "2019-09-14T23:52"}]->(Villeurbanne)  <<< ANOMALIE
(Chevalier)-[:LIVES_AT]->(Villeurbanne)  <<< CONNEXION
```

---

## GENERATION DES FICHIERS DE TEST

### Script Python recommande (utilisant Faker + donnees ci-dessus)

```python
"""
Generateur de donnees de test pour l'affaire MOREAU.
Execute ce script pour creer les fichiers dans data/benchmark/
"""
import json
import os
from pathlib import Path
from datetime import datetime

BENCHMARK_DIR = Path("data/benchmark/affaire-moreau")

def create_evidence_files():
    """Cree les 15 pieces a conviction sous forme de fichiers."""
    
    os.makedirs(BENCHMARK_DIR / "police", exist_ok=True)
    os.makedirs(BENCHMARK_DIR / "temoignages", exist_ok=True)
    os.makedirs(BENCHMARK_DIR / "forensique", exist_ok=True)
    os.makedirs(BENCHMARK_DIR / "numerique", exist_ok=True)
    os.makedirs(BENCHMARK_DIR / "osint", exist_ok=True)
    os.makedirs(BENCHMARK_DIR / "photos", exist_ok=True)
    
    # P-01 a P-15 : chaque piece ecrite en fichier .txt ou .json
    # Les PDFs seraient generes avec reportlab ou fpdf2
    # Les images seraient generees avec Pillow (placeholder)
    
    evidence_manifest = {
        "case": {
            "name": "Affaire MOREAU",
            "reference": "#2019-4472-LY",
            "description": "Disparition et meurtre d'Elise Moreau, 34 ans, pharmacienne a Lyon. Corps retrouve 11 jours plus tard a Miribel-Jonage.",
        },
        "evidence": [
            {
                "id": "P-01",
                "title": "Rapport de police initial",
                "type": "pdf",
                "source": "SRPJ Lyon",
                "reliability": 85,
                "source_date": "2019-09-15",
                "file": "police/P-01_rapport-initial.txt",
                "wave": 1,
            },
            {
                "id": "P-02",
                "title": "Temoignage Sophie Laurent",
                "type": "text",
                "source": "Audition SRPJ, PV 2019/09/152",
                "reliability": 70,
                "source_date": "2019-09-16",
                "file": "temoignages/P-02_sophie-laurent.txt",
                "wave": 1,
            },
            {
                "id": "P-03",
                "title": "Temoignage Marc Duval",
                "type": "text",
                "source": "Audition SRPJ, PV 2019/09/153",
                "reliability": 60,
                "source_date": "2019-09-16",
                "file": "temoignages/P-03_marc-duval.txt",
                "wave": 1,
            },
            {
                "id": "P-04",
                "title": "Temoignage Romain Fabre",
                "type": "text",
                "source": "Audition SRPJ, PV 2019/09/161",
                "reliability": 65,
                "source_date": "2019-09-18",
                "file": "temoignages/P-04_romain-fabre.txt",
                "wave": 1,
            },
            {
                "id": "P-05",
                "title": "Temoignage Karim Belhadj (2 versions)",
                "type": "text",
                "source": "Audition SRPJ, PV 2019/09/168",
                "reliability": 45,
                "source_date": "2019-09-18",
                "file": "temoignages/P-05_karim-belhadj.txt",
                "wave": 1,
            },
            {
                "id": "P-06",
                "title": "Rapport autopsie",
                "type": "pdf",
                "source": "IML Lyon, Dr Perrin",
                "reliability": 95,
                "source_date": "2019-09-26",
                "file": "forensique/P-06_autopsie.txt",
                "wave": 1,
            },
            {
                "id": "P-07",
                "title": "Releves telephoniques (5 lignes)",
                "type": "text",
                "source": "Requisition operateurs",
                "reliability": 90,
                "source_date": "2019-10-01",
                "file": "numerique/P-07_telephonie.txt",
                "wave": 2,
            },
            {
                "id": "P-08",
                "title": "Transactions bancaires Elise Moreau",
                "type": "text",
                "source": "LCL, requisition judiciaire",
                "reliability": 90,
                "source_date": "2019-10-01",
                "file": "numerique/P-08_banque-elise.txt",
                "wave": 2,
            },
            {
                "id": "P-09",
                "title": "Photos scene de decouverte (5 photos)",
                "type": "image",
                "source": "PTS Lyon",
                "reliability": 90,
                "source_date": "2019-09-25",
                "file": "photos/P-09_scene/",
                "wave": 4,
            },
            {
                "id": "P-10",
                "title": "Releve assurance-vie AXA",
                "type": "pdf",
                "source": "AXA Assurances, requisition",
                "reliability": 95,
                "source_date": "2019-11-01",
                "file": "osint/P-10_assurance-vie.txt",
                "wave": 3,
            },
            {
                "id": "P-11",
                "title": "Historique Instagram Elise Moreau",
                "type": "text",
                "source": "Extraction Meta/Instagram",
                "reliability": 80,
                "source_date": "2019-10-01",
                "file": "numerique/P-11_instagram.txt",
                "wave": 3,
            },
            {
                "id": "P-12",
                "title": "Extrait Kbis SARL WEBCRAFT",
                "type": "text",
                "source": "Infogreffe",
                "reliability": 95,
                "source_date": "2026-04-01",
                "file": "osint/P-12_kbis-webcraft.txt",
                "wave": 3,
            },
            {
                "id": "P-13",
                "title": "Main courante Moreau contre Tessier",
                "type": "text",
                "source": "Commissariat Lyon 6e",
                "reliability": 85,
                "source_date": "2017-03-08",
                "file": "police/P-13_main-courante.txt",
                "wave": 3,
            },
            {
                "id": "P-14",
                "title": "Casier judiciaire Yann Chevalier",
                "type": "text",
                "source": "Casier judiciaire national (B2)",
                "reliability": 95,
                "source_date": "2026-04-01",
                "file": "osint/P-14_casier-chevalier.txt",
                "wave": 3,
            },
            {
                "id": "P-15",
                "title": "Analyse ADN megot (maj 2026)",
                "type": "pdf",
                "source": "Labo PTS Lyon",
                "reliability": 85,
                "source_date": "2026-04-01",
                "file": "forensique/P-15_adn-megot.txt",
                "wave": 4,
            },
        ],
        "waves": {
            "1": {
                "name": "Dossier initial",
                "description": "Ouverture cold case, pieces existantes",
                "evidence_ids": ["P-01", "P-02", "P-03", "P-04", "P-05", "P-06"],
            },
            "2": {
                "name": "Donnees numeriques",
                "description": "Requisitions telephonie + banque",
                "evidence_ids": ["P-07", "P-08"],
            },
            "3": {
                "name": "OSINT + donnees complementaires",
                "description": "Resultats recherche OSINT automatique",
                "evidence_ids": ["P-10", "P-11", "P-12", "P-13", "P-14"],
            },
            "4": {
                "name": "Percee forensique",
                "description": "Photos scene + ADN megot",
                "evidence_ids": ["P-09", "P-15"],
            },
        },
    }
    
    with open(BENCHMARK_DIR / "manifest.json", "w", encoding="utf-8") as f:
        json.dump(evidence_manifest, f, ensure_ascii=False, indent=2)

    print(f"Manifest cree dans {BENCHMARK_DIR / 'manifest.json'}")

if __name__ == "__main__":
    create_evidence_files()
```

---

## METRIQUES DE BENCHMARK

Apres chaque vague d'ingestion, mesurer :

| Metrique | Description | Cible |
|----------|-------------|-------|
| **Entites extraites** | Nombre d'entites correctement identifiees | >= 80% des entites du scenario |
| **Relations graphe** | Nombre de relations Neo4j correctement creees | >= 70% des relations cles |
| **Contradictions** | Nombre de contradictions C1-C7 detectees | >= 5 sur 7 |
| **Hypotheses** | Nombre d'hypotheses H1-H5 generees | >= 4 sur 5 |
| **Score H2 final** | Score de l'hypothese Tessier+Chevalier apres vague 4 | >= 70% |
| **Angles morts** | Nombre de lacunes identifiees (sur 10) | >= 6 sur 10 |
| **Timeline** | Evenements correctement places chronologiquement | >= 85% |
| **Temps d'analyse** | Duree totale des 4 vagues (tokens, secondes) | Reference a etablir |
| **Alertes** | Alertes critiques generees aux bons moments | >= 3 alertes pertinentes |

---

## SOLUTION (pour reference interne uniquement)

**La verite :**
Julien Tessier, obsede par Elise depuis leur separation en 2017, n'a jamais accepte la rupture. Il a recrute Yann Chevalier comme complice via des forums en ligne. Chevalier, client de la Pharmacie du Parc, avait acces a Elise et a accepte en echange d'argent (les 10 000 EUR du "site web" etaient en realite un paiement extorque par Chevalier a Elise, sous menace de reveler a Marc qu'il la harcelait physiquement -- les ecchymoses anterieures sont de Chevalier).

Le soir du 14 septembre, Tessier conduit de Grenoble a Lyon. Il intercepte Elise quand elle quitte la soiree a pied vers 00h00-00h10. Il la tue par strangulation dans son vehicule entre la Croix-Rousse et Miribel-Jonage. Chevalier le rejoint a Miribel-Jonage pour aider a enterrer le corps. Le megot de Chevalier est laisse par erreur.

Marc Duval n'est pas implique. Son detour par Villeurbanne a 23h52 s'explique par un arret a une station-service (non verifie dans l'enquete initiale). Le changement de beneficiaire de l'assurance-vie etait lie aux tensions du couple, pas a une connaissance du complot.

Sophie Laurent omet le DM d'Elise parce qu'Elise voulait lui parler de Chevalier qui la harcelait physiquement -- Sophie ne sait pas que c'etait lie au meurtre et n'a pas fait le rapprochement.
