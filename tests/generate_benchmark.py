"""
NEXUS -- Generateur de donnees de benchmark : Affaire MOREAU.

Cree les 15 pieces a conviction sous forme de fichiers texte dans
data/benchmark/affaire-moreau/, organisees par type et par vague
d'ingestion.

Usage:
    python -m tests.generate_benchmark

Prerequis: aucun (pas de dependance externe).
"""

from __future__ import annotations

import json
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
BENCHMARK_DIR = BASE_DIR / "data" / "benchmark" / "affaire-moreau"

# ===================================================================
# Evidence content -- each piece as a plain text string
# ===================================================================

EVIDENCE: dict[str, str] = {}

EVIDENCE["P-01"] = """\
REPUBLIQUE FRANCAISE
MINISTERE DE L'INTERIEUR
SERVICE REGIONAL DE POLICE JUDICIAIRE DE LYON
---
RAPPORT DE DISPARITION INQUIETANTE
PV n 2019/09/148 -- 15 septembre 2019

Le 15 septembre 2019 a 10h14, M. DUVAL Marc, ne le 22/06/1982, domicilie \
17 rue Garibaldi, 69006 Lyon, se presente au commissariat du 6e arrondissement \
pour signaler la disparition de sa compagne, Mme MOREAU Elise, nee le 12/03/1985, \
domiciliee a la meme adresse.

CIRCONSTANCES :
M. DUVAL declare que Mme MOREAU n'est pas rentree au domicile conjugal apres \
une soiree chez Mme LAURENT Sophie, 42 rue Imbert Colomes, 69001 Lyon \
(quartier Croix-Rousse), le samedi 14 septembre 2019.

DERNIER CONTACT :
SMS envoye par la victime a M. DUVAL le 14/09/2019 a 23h47 : \
"je rentre bientot, bisous"
M. DUVAL a repondu : "ok a tout de suite"

VEHICULE :
Le vehicule de Mme MOREAU, une Renault Clio IV grise immatriculee EF-429-GH, \
a ete retrouve stationne rue des Pierres Plantees, 69001 Lyon, a environ 200 \
metres du domicile de Mme LAURENT. La cle etait dans le contact, la portiere \
conducteur non verrouillee. Le sac a main de Mme MOREAU est absent du vehicule.

TELEPHONE :
Le telephone de Mme MOREAU (06 72 41 88 03, operateur Orange) a ete localise \
pour la derniere fois le 15/09/2019 a 00h12 sur l'antenne-relais Croix-Rousse \
secteur 3. Depuis, aucune activite n'a ete enregistree.

SIGNALEMENT :
Femme caucasienne, 1m67, 58 kg, cheveux blonds mi-longs, yeux bleus. Portait \
lors de la soiree une robe noire et un blouson en cuir marron. Bague de \
fiancailles or blanc a la main gauche.

MESURES PRISES :
- Avis de recherche diffuse
- Requisition operateur telephonique en cours
- Auditions du compagnon et des convives programmees

Le Commandant Michel VASSEUR
SRPJ Lyon
"""

EVIDENCE["P-02"] = """\
PROCES-VERBAL D'AUDITION
PV n 2019/09/152 -- 16 septembre 2019
Temoin : LAURENT Sophie, nee le 07/04/1986
Profession : Avocate, cabinet Fidal Lyon
Domicile : 42 rue Imbert Colomes, 69001 Lyon
---

Q : Racontez-nous la soiree du 14 septembre.

R : J'ai organise une soiree chez moi. On etait six : moi, Elise Moreau, Marc \
Duval, Romain Fabre, Claire Petit et Adrien Roche. Elise est arrivee vers \
20h30. L'ambiance etait bonne, on a mange, bu du vin.

Q : Comment etait Elise ce soir-la ?

R : Elle semblait preoccupee. Pas deprimee, mais distante par moments. Elle a \
beaucoup parle avec Romain Fabre sur le balcon, ce que j'ai trouve un peu \
bizarre vu qu'elle venait de le licencier de la pharmacie deux semaines avant.

Q : A quelle heure Marc Duval est-il parti ?

R : Marc est parti vers 23h, il disait avoir une migraine. Elise a dit qu'elle \
restait encore un peu.

Q : A quelle heure Elise est-elle partie ?

R : Vers minuit, Elise m'a dit qu'elle allait rentrer a pied. C'est a environ \
quinze minutes de chez moi. Elle est partie seule. Je l'ai accompagnee jusqu'a \
la porte. Je ne l'ai plus revue.

Q : Quelqu'un d'autre est-il parti en meme temps ou juste apres ?

R : Non, Romain etait deja parti avant, et Claire et Adrien sont restes encore \
un peu apres Elise. Ils sont partis vers 00h15.

Q : Avez-vous remarque quelque chose d'inhabituel ?

R : Non, rien de special. C'etait une soiree normale entre amis.

Fin d'audition a 11h45.
"""

EVIDENCE["P-03"] = """\
PROCES-VERBAL D'AUDITION
PV n 2019/09/153 -- 16 septembre 2019
Temoin : DUVAL Marc, ne le 22/06/1982
Profession : Chef de projet informatique, Sopra Steria Lyon
Domicile : 17 rue Garibaldi, 69006 Lyon
---

Q : Racontez-nous la soiree du 14 septembre et les heures qui ont suivi.

R : On est alles chez Sophie Laurent pour une soiree entre amis. Je suis arrive \
avec Elise vers 20h30. Soiree classique, repas, discussions. Vers 23h15-23h30, \
j'ai eu une grosse migraine, donc j'ai decide de rentrer. Elise voulait rester \
encore un peu.

Q : Comment etes-vous rentre ?

R : Je suis rentre directement a l'appartement. A pied jusqu'a ma voiture garee \
rue Burdeau, puis en voiture jusqu'a la rue Garibaldi. Ca m'a pris environ \
vingt minutes en tout.

Q : Qu'avez-vous fait en arrivant ?

R : J'ai pris un Doliprane et je me suis couche. J'ai recu le SMS d'Elise a \
23h47 : "je rentre bientot, bisous". Je lui ai repondu "ok a tout de suite". \
Apres ca je me suis endormi.

Q : Le lendemain matin ?

R : Je me suis reveille vers 7h30. J'ai vu qu'Elise n'etait pas a cote de moi. \
Son cote du lit n'etait pas defait. J'ai essaye de l'appeler, ca sonnait dans \
le vide. J'ai attendu un peu en pensant qu'elle etait peut-etre restee dormir \
chez Sophie. J'ai appele Sophie vers 9h50, elle m'a dit qu'Elise etait partie \
vers minuit. La j'ai commence a paniquer. J'ai signale la disparition a 10h.

Q : Comment etait votre relation avec Elise ces derniers temps ?

R : Correcte. On avait des hauts et des bas comme tout couple, mais rien de \
dramatique. On parlait de se marier l'annee prochaine.

Q : Etiez-vous au courant de tensions professionnelles ?

R : Elise etait stressee par la pharmacie. Elle avait du licencier Romain, son \
preparateur, et ca l'avait affectee. Sinon, la pharmacie marchait bien.

Fin d'audition a 14h30.
"""

EVIDENCE["P-04"] = """\
PROCES-VERBAL D'AUDITION
PV n 2019/09/161 -- 18 septembre 2019
Temoin : FABRE Romain, ne le 14/11/1988
Profession : Preparateur en pharmacie (sans emploi actuellement)
Domicile : 8 rue Paul Bert, 69003 Lyon
---

Q : Quelle etait votre relation avec Elise Moreau ?

R : J'etais son preparateur en pharmacie depuis 2018. On se connaissait depuis \
plus longtemps, on avait eu une breve relation en 2015, avant qu'elle ne \
rencontre Marc. C'etait reste amical.

Q : Pourquoi avez-vous ete licencie ?

R : Des erreurs de stock, des retards. Elise m'a dit que c'etait une decision \
de gestion, qu'elle n'avait pas le choix. J'etais decu mais je comprenais.

Q : Vous etiez a la soiree du 14 septembre ?

R : Oui, Elise m'avait invite malgre le licenciement. On restait en bons termes. \
A la soiree, on a parle sur le balcon. Elle s'excusait pour la situation, elle \
m'a dit qu'elle n'avait pas eu le choix. C'etait une conversation honnete.

Q : A quelle heure etes-vous parti ?

R : Vers 23h45, juste apres Marc. J'ai pris le metro a Croix-Rousse, la \
ligne C direction Part-Dieu. Je suis rentre chez moi vers 00h20. Ma copine \
Lea Mercier peut confirmer l'heure de mon arrivee.

Q : Avez-vous revu Elise apres la soiree ?

R : Non. La derniere fois que je l'ai vue, c'etait dans l'appartement de Sophie, \
quand je suis parti.

Fin d'audition a 16h15.
"""

EVIDENCE["P-05"] = """\
PROCES-VERBAL D'AUDITION
PV n 2019/09/168 -- 18 septembre 2019
Temoin : BELHADJ Karim, ne le 30/05/1991
Profession : Livreur Uber Eats
Domicile : 15 avenue Berthelot, 69007 Lyon
---

VERSION 1 (18 septembre 2019) :

Q : Que faisiez-vous dans la nuit du 14 au 15 septembre ?

R : Je faisais des livraisons Uber Eats dans le coin de la Croix-Rousse. Vers \
1h-1h30 du matin, j'ai vu une femme blonde, la trentaine, monter dans une \
voiture sombre garee rue des Tables Claudiennes. La voiture etait une berline, \
peut-etre noire ou bleu fonce. La femme avait l'air pressee mais pas en \
detresse. Je n'ai pas vu le conducteur.

---

VERSION 2 (3 octobre 2019, reconvoque) :

Q : Nous vous avons reconvoque car des elements nouveaux necessitent des \
precisions. Pouvez-vous revenir sur ce que vous avez vu ?

R : En fait, j'ai mieux reflechi depuis. La voiture etait une berline type BMW \
ou Audi, definitivement foncee, noire je dirais. Et la femme, en fait elle \
avait l'air plutot hesitante, pas pressee. Comme si elle connaissait la \
personne mais qu'elle hesitait a monter.

Q : A quelle heure etait-ce exactement ?

R : Je pense que c'etait plus tot que ce que j'ai dit. Plutot vers 00h30-00h45 \
que 1h30. J'ai verifie mes courses Uber et ma derniere livraison dans le \
quartier etait a 00h20.

Q : Avez-vous vu le conducteur ?

R : Oui, en fait. Le conducteur est sorti pour lui ouvrir la portiere. Un homme \
de 40-45 ans, cheveux courts, de corpulence moyenne. Je ne l'ai vu que de \
dos puis de profil. Il faisait sombre.

Fin d'audition a 10h30.
"""

EVIDENCE["P-06"] = """\
INSTITUT MEDICO-LEGAL DE LYON
RAPPORT D'AUTOPSIE
---
Date de l'examen : 26 septembre 2019
Medecin legiste : Dr. Nathalie PERRIN

VICTIME : MOREAU Elise, nee le 12/03/1985 a Lyon
Corps decouvert le 25/09/2019 dans une zone boisee du parc de Miribel-Jonage, \
partiellement enterre dans une fosse peu profonde (environ 40 cm).

EXAMEN EXTERNE :
- Decomposition avancee (11 jours d'exposition, temperature moyenne de la \
  periode : 18-22 C)
- Strangulation manuelle evidente : marques de pression bilaterales sur le cou
- Petechies oculaires bilaterales
- Fracture de l'os hyoide (confirmee par scanner post-mortem)
- Ecchymoses anterieures au deces :
  * Bras droit : 3 ecchymoses digitiformes (compatibles avec une prise ferme)
  * Epaule gauche : ecchymose diffuse
  * Datation : 3 a 5 jours avant le deces (soit entre le 10 et le 12 septembre)
- Bague de fiancailles or blanc toujours en place (main gauche)
- Ongles : terre et fibres textiles (coton bleu marine) sous les ongles des \
  deux mains

EXAMEN INTERNE :
- Cause du deces : Asphyxie mecanique par strangulation manuelle
- La force exercee etait significative (fracture hyoidienne nette)
- Estomac : repas recent (fromage, pain, vin rouge) compatible avec un repas \
  pris dans la soiree du 14 septembre
- Alcoolemie estimee au moment du deces : environ 0.8 g/L

PRELEVEMENTS :
- Sous les ongles : terre et fibres analysees. Pas d'ADN etranger exploitable.
- Aucune trace de violence sexuelle
- Ecouvillonnage cutane : pas de resultat significatif (decomposition)

ESTIMATION DE L'HEURE DU DECES :
Entre 00h00 et 04h00 le 15 septembre 2019 (estimation large en raison de la \
decomposition avancee au moment de la decouverte)

NOTE :
La force necessaire pour fracturer l'os hyoide de cette maniere suggere un \
agresseur disposant d'une force physique significative. L'absence de traces \
defensives sous les ongles (pas d'ADN) pourrait indiquer une attaque soudaine \
ou un etat de sideration de la victime.

Dr. Nathalie PERRIN
Medecin legiste
"""

EVIDENCE["P-07"] = """\
RELEVES TELEPHONIQUES -- REQUISITION JUDICIAIRE
Operateurs : Orange, SFR, Free
Periode : 14 septembre 2019 18h00 -- 15 septembre 2019 12h00
---

== LIGNE 1 : MOREAU Elise -- 06 72 41 88 03 (Orange) ==
14/09 20:18  APPEL SORTANT vers 06 91 33 74 16 (LAURENT Sophie) -- duree 42s
             Borne : Lyon-1-CroixRousse-2
14/09 23:47  SMS SORTANT vers 06 83 19 57 22 (DUVAL Marc)
             Contenu : "je rentre bientot, bisous"
             Borne : Lyon-1-CroixRousse-3
15/09 00:12  DERNIERE LOCALISATION
             Borne : Lyon-1-CroixRousse-3 (meme secteur que precedent)
             Telephone eteint ou hors reseau. Plus aucune activite.

== LIGNE 2 : DUVAL Marc -- 06 83 19 57 22 (SFR) ==
14/09 22:45  DATA mobile (application Spotify, streaming)
             Borne : Lyon-1-CroixRousse-1
14/09 23:18  FIN data mobile
             Borne : Lyon-1-CroixRousse-1
14/09 23:48  SMS ENTRANT de 06 72 41 88 03 (MOREAU Elise)
             Reponse envoyee : "ok a tout de suite"
             Borne : Lyon-6-Garibaldi-1  [Domicile -- coherent]
14/09 23:52  DATA mobile (breve, 4 secondes)
             Borne : Villeurbanne-Republique-2  [ANOMALIE -- pas sur le trajet domicile]
15/09 07:34  APPEL SORTANT vers 06 72 41 88 03 (MOREAU Elise) -- pas de reponse
             Borne : Lyon-6-Garibaldi-1
15/09 09:51  APPEL SORTANT vers 06 91 33 74 16 (LAURENT Sophie) -- duree 3m12s
             Borne : Lyon-6-Garibaldi-1

== LIGNE 3 : FABRE Romain -- 07 68 42 11 90 (Free) ==
14/09 23:41  DATA mobile
             Borne : Lyon-1-CroixRousse-3
15/09 00:19  DATA mobile
             Borne : Lyon-3-PartDieu-1
15/09 00:31  DATA mobile
             Borne : Lyon-3-PaulBert-2  [Domicile -- coherent]

== LIGNE 4 : TESSIER Julien -- 06 44 72 19 85 (Orange) ==
14/09 21:30  APPEL SORTANT vers 07 81 55 23 07 (numero prepaye -- identite inconnue)
             Duree : 2m47s
             Borne : Grenoble-Centre-4
14/09 22:15  DATA mobile
             Borne : A48-Voiron  [Autoroute Grenoble -> Lyon]
14/09 23:05  DATA mobile
             Borne : A43-LaVerpilliere  [Approche Lyon par l'est]
14/09 23:38  DATA mobile
             Borne : Lyon-Est-Bron-2  [Peripherie est de Lyon]
15/09 00:22  DATA mobile
             Borne : Lyon-1-CroixRousse-5  [SUSPECT -- meme quartier qu'Elise]
15/09 01:15  DATA mobile
             Borne : Miribel-Jonage-Nord-1  [TRES SUSPECT -- lieu de decouverte du corps]
15/09 02:48  DATA mobile
             Borne : A43-Bourgoin  [Autoroute retour vers Grenoble]
15/09 04:12  DATA mobile
             Borne : Grenoble-Centre-4  [Retour domicile]

== LIGNE 5 : NUMERO PREPAYE -- 07 81 55 23 07 (SFR prepaye) ==
Titulaire : Carte prepaye, identite non verifiee
14/09 21:28  SMS ENTRANT de 06 44 72 19 85 (TESSIER Julien)
             Borne : Villeurbanne-Republique-1
15/09 01:58  DATA mobile
             Borne : Miribel-Jonage-Nord-1  [SYNCHRONE avec TESSIER -- meme lieu]
15/09 03:14  DATA mobile
             Borne : Villeurbanne-Republique-1  [Retour]

NOTE ANALYSTE : Le numero prepaye 07 81 55 23 07 n'a pas ete identifie dans \
l'enquete initiale. L'appareil borne systematiquement a Villeurbanne-Republique, \
suggerant un domicile dans ce secteur.
"""

EVIDENCE["P-08"] = """\
RELEVE DE COMPTE BANCAIRE -- REQUISITION JUDICIAIRE
Banque : LCL -- Agence Lyon Part-Dieu
Titulaire : MOREAU Elise
Compte : FR76 3000 4000 xx (courant)
Periode : 1er aout -- 30 septembre 2019
---

MOUVEMENTS SIGNIFICATIFS :

20/08/2019  VIREMENT SORTANT     -5 000.00 EUR
            Beneficiaire : SARL WEBCRAFT -- IBAN FR76 xxx
            Libelle : "acompte site web pharmacie"

02/09/2019  VIREMENT SORTANT     -5 000.00 EUR
            Beneficiaire : SARL WEBCRAFT -- IBAN FR76 xxx
            Libelle : "creation site web pharmacie"

MOUVEMENTS DERNIERS JOURS :

12/09/2019  RETRAIT DAB           -300.00 EUR
            Distributeur : LCL Lyon Part-Dieu

13/09/2019  PAIEMENT CB            -47.80 EUR
            Carrefour City -- Lyon 6e

14/09/2019  PAIEMENT CB            -28.50 EUR
            Nicolas Caviste -- Lyon 1er (15h30)

14/09/2019  PAIEMENT CB            -12.40 EUR
            Uber BV -- application (19h55)

--- AUCUNE TRANSACTION APRES LE 14/09/2019 ---

SOLDE AU 14/09/2019 :     8 742.30 EUR
SOLDE AU 30/09/2019 :     8 742.30 EUR (inchange -- confirme absence d'activite)

NOTE : Les deux virements vers SARL WEBCRAFT totalisent 10 000 EUR. \
Ce montant represente un cout anormalement eleve pour la creation d'un site \
internet de pharmacie (tarif marche : 2 000-4 000 EUR).
"""

EVIDENCE["P-09_descriptions"] = """\
PHOTOS DE LA SCENE DE DECOUVERTE
Police Technique et Scientifique -- Lyon
Date : 25 septembre 2019
Lieu : Zone boisee, parc de Miribel-Jonage, acces par chemin de terre
---

PHOTO 1 (P-09-01.jpg) :
Vue large de la zone boisee. Chemin de terre visible menant a un sous-bois \
dense. Ruban de police en place. Le sol est meuble (terre + feuilles mortes). \
Plusieurs traces de passage recentes visibles sur le chemin.

PHOTO 2 (P-09-02.jpg) :
Fosse peu profonde, environ 40 cm de profondeur, 180 cm de longueur. Corps \
partiellement recouvert de feuilles mortes et de terre. La fosse semble creusee \
a la hate, bords irreguliers. Pas d'outil retrouve sur place.

PHOTO 3 (P-09-03.jpg) :
Gros plan sur les mains de la victime. Terre visible sous les ongles des deux \
mains. Bague de fiancailles or blanc toujours en place (main gauche, annulaire). \
Pas de blessures defensives visibles sur les mains.

PHOTO 4 (P-09-04.jpg) :
Trace de pneu sur le chemin de terre, a environ 15 metres de la fosse. Profil \
large et bien defini. Expertise pneumatique ulterieure : compatible avec un pneu \
Continental PremiumContact 6 en dimension 225/45 R17.
Note : Cette dimension est standard sur les BMW Serie 3 (F30/G20).
Vehicules exclus par cette dimension : Peugeot 308 (205/55 R16), \
Fiat Punto (175/65 R14), Renault Clio IV (185/65 R15).

PHOTO 5 (P-09-05.jpg) :
Megot de cigarette preleve a 3 metres de la fosse, sur le chemin. Marque : \
Camel sans filtre. Megot partiellement humide (pluie des jours precedents). \
Preleve pour analyse ADN. Reference scelle : SC-2019-4472-MEG-01.
"""

EVIDENCE["P-10"] = """\
AXA ASSURANCES -- RELEVE DE CONTRAT
Requisition judiciaire n 2019/RJ/4472-08
---

Contrat : AXA-VIE-2018-77841
Type : Assurance vie
Date de souscription : 15 juin 2018

Souscripteur : MOREAU Elise, nee le 12/03/1985
Capital garanti : 150 000 EUR

Beneficiaire enregistre : DUVAL Marc, ne le 22/06/1982
Relation declaree : Compagnon

MODIFICATION EN COURS :
Date de la demande : 5 septembre 2019
Objet : Changement de beneficiaire
Nouveau beneficiaire demande : Association "Pharmaciens Sans Frontieres"
                               (SIREN : 302 413 987)
Statut de la modification : EN COURS -- formulaire recu, en attente de \
                            validation du service juridique
Date estimee de finalisation : fin septembre 2019

NOTE : La modification n'a pas ete finalisee au moment du deces de la \
souscriptrice (15 septembre 2019). En consequence, le beneficiaire au moment \
du deces reste M. DUVAL Marc.

Capital verse a M. DUVAL : EN ATTENTE (procedure judiciaire en cours)
"""

EVIDENCE["P-11"] = """\
EXTRACTION DONNEES INSTAGRAM
Requisition judiciaire a Meta Platforms Ireland Ltd
Compte : @elise_pharma_lyon (MOREAU Elise)
Periode extraite : 1er septembre -- 15 septembre 2019
---

=== POSTS PUBLICS ===

10/09/2019 14:22
Type : Photo
Contenu : Photo de la pharmacie avec nouvelle devanture renovee
Legende : "Nouveau chapitre pour la Pharmacie du Parc ! Fiere de cette renovation."
Likes : 127
Commentaires : 8 (felicitations diverses)

=== STORIES (expirees, extraites du serveur) ===

08/09/2019 21:14
Type : Story photo
Contenu : Photo d'un livre titre "Recommencer sa vie" de Marc Levy, \
pose sur une table avec une tasse de the
Emoji : coeur rouge
Vues : 43

12/09/2019 23:01
Type : Story texte
Contenu : Citation sur fond noir : "Ce qui ne nous tue pas nous rend plus fort"
Vues : 38

=== MESSAGES PRIVES (DM) ===

11/09/2019 19:47
DE : @elise_pharma_lyon
A : @sophie_lrt (LAURENT Sophie)
Message : "Il faut que je te parle de quelque chose samedi. Important."

11/09/2019 19:52
DE : @sophie_lrt
A : @elise_pharma_lyon
Message : "OK pas de souci, on en parle samedi alors ! Ca va ?"

11/09/2019 19:55
DE : @elise_pharma_lyon
A : @sophie_lrt
Message : "Oui oui ca va. C'est juste... je t'expliquerai de vive voix."

13/09/2019 09:14
DE : @romain.fabre.lyon (FABRE Romain)
A : @elise_pharma_lyon
Message : "On peut parler samedi ? J'ai besoin de comprendre pour le licenciement."

13/09/2019 09:31
DE : @elise_pharma_lyon
A : @romain.fabre.lyon
Message : "Oui, viens a la soiree chez Sophie. On discutera."

=== DERNIERE ACTIVITE ===
14/09/2019 19:42 -- Like sur un post de @sophie_lrt
"""

EVIDENCE["P-12"] = """\
EXTRAIT KBIS -- INFOGREFFE
Date de l'extrait : 1er avril 2026
---

Denomination : SARL WEBCRAFT
SIREN : 831 457 923
SIRET (siege) : 831 457 923 00014
Forme juridique : Societe a responsabilite limitee (SARL)
Capital social : 2 000 EUR

Siege social : 23 rue de la Republique, 69100 Villeurbanne

Gerant : CHEVALIER Yann Pierre
Ne le : 04/07/1983 a Saint-Etienne (42)
Nationalite : Francaise

Activite principale declaree : Creation de sites internet, hebergement, \
services numeriques et conseil en communication digitale (NAF 6201Z)

Date d'immatriculation : 12 janvier 2018
Greffe : Tribunal de commerce de Lyon

Chiffre d'affaires declare :
  2018 :  12 400 EUR
  2019 :  34 000 EUR
  2020 :  8 200 EUR
  2021 :  3 100 EUR (quasi-inactif)
  2022-2025 : Non communique (presumee en sommeil)

Observations : Aucune procedure collective. Pas de mention de dissolution.
"""

EVIDENCE["P-13"] = """\
COMMISSARIAT DE POLICE -- LYON 6e ARRONDISSEMENT
MAIN COURANTE
Reference : MC-2017-6e-0847
Date : 8 mars 2017
---

DECLARANTE : MOREAU Elise, nee le 12/03/1985
Domicile : 17 rue Garibaldi, 69006 Lyon
Profession : Pharmacienne

OBJET : Harcelement de la part d'un ex-compagnon

Mme MOREAU Elise se presente ce jour et declare etre victime de harcelement \
de la part de son ancien compagnon, M. TESSIER Julien, ne le 03/09/1983, \
domicilie a Grenoble (38).

FAITS DECLARES :
- Depuis la separation intervenue en janvier 2017, M. TESSIER envoie des \
  messages repetitifs (SMS et emails) -- parfois 10 a 15 par jour
- Appels nocturnes entre 23h et 3h du matin
- Presence signalee devant le domicile de Mme MOREAU a au moins 3 reprises \
  (dernier episode : 2 mars 2017)
- M. TESSIER refuse d'accepter la rupture et alterne entre messages \
  d'excuses et menaces voilees

Mme MOREAU declare ne pas souhaiter deposer plainte a ce stade mais demande \
que cette main courante soit enregistree a titre preventif.

CONSEILS DONNES :
- Conservation des messages et captures d'ecran
- Signaler immediatement tout escalade
- Possibilite de deposer plainte a tout moment

Fait et enregistre le 8 mars 2017.
Brigadier MORIN
"""

EVIDENCE["P-14"] = """\
EXTRAIT DU CASIER JUDICIAIRE -- BULLETIN N 2 (B2)
---
NOM : CHEVALIER
Prenom : Yann Pierre
Ne le : 04/07/1983 a Saint-Etienne (Loire, 42)
Nationalite : Francaise
Filiation : Fils de CHEVALIER Michel et de DURAND Sylvie

CONDAMNATIONS :

1) Tribunal correctionnel de Saint-Etienne
   Audience du : 14 fevrier 2012
   Infraction : Violence habituelle sur conjoint (art. 222-14 du Code Penal)
   Victime : RENARD Aurelie (ex-epouse)
   Peine : 8 mois d'emprisonnement avec sursis
           Obligation de soins (suivi psychologique pendant 2 ans)
   Mention : Premiere condamnation

2) Tribunal correctionnel de Lyon
   Audience du : 3 novembre 2015
   Infraction : Menaces de mort reiterees sur ex-conjointe (art. 222-17 CP)
   Victime : GARCIA Maria (ex-compagne)
   Peine : 6 mois d'emprisonnement ferme
           Amenes en placement sous surveillance electronique (bracelet)
           Interdiction de contact avec la victime pendant 3 ans
   Mention : Recidive legale

NOTE : Inscription au FNAEG (Fichier National Automatise des Empreintes \
Genetiques) suite a la condamnation de 2015. Toutefois, le prelevement ADN \
n'a ete effectivement realise et enregistre qu'en janvier 2026 suite a un \
controle administratif.
"""

EVIDENCE["P-15"] = """\
LABORATOIRE DE POLICE SCIENTIFIQUE DE LYON
RAPPORT D'ANALYSE GENETIQUE
---
Reference : LPS-2019-4472-ADN-03
Scelle d'origine : SC-2019-4472-MEG-01 (megot de cigarette)
Preleve le : 25/09/2019 sur la scene de decouverte (Miribel-Jonage)
Marque : Camel sans filtre

ANALYSE INITIALE (decembre 2019) :
- ADN male exploitable extrait de la salive residuelle sur le megot
- Profil genetique complet obtenu (16 loci STR)
- Recherche dans le FNAEG : AUCUNE CORRESPONDANCE a cette date
- Profil conserve dans la base d'attente pour comparaison ulterieure

MISE A JOUR (avril 2026) :
Suite a l'enregistrement tardif du profil genetique de CHEVALIER Yann Pierre \
(ne le 04/07/1983) dans le FNAEG en janvier 2026, une recherche automatique \
a produit une correspondance positive.

RESULTAT :
Le profil ADN extrait du megot SC-2019-4472-MEG-01 CORRESPOND au profil \
genetique de CHEVALIER Yann Pierre.
Probabilite de correspondance : > 1 sur 10 milliards (certitude statistique)

INTERPRETATION :
La presence de ce megot a proximite immediate de la fosse ou le corps de \
Mme MOREAU a ete retrouve etablit que M. CHEVALIER s'est trouve physiquement \
sur les lieux. Ce resultat est a mettre en perspective avec les autres \
elements de l'enquete.

Signe : Chef de laboratoire Dr. MARTIN Francois
Date : 3 avril 2026
"""


def create_directory_structure() -> None:
    """Create the benchmark data directory tree."""
    dirs = [
        BENCHMARK_DIR / "police",
        BENCHMARK_DIR / "temoignages",
        BENCHMARK_DIR / "forensique",
        BENCHMARK_DIR / "numerique",
        BENCHMARK_DIR / "osint",
        BENCHMARK_DIR / "photos",
    ]
    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)

    print(f"[OK] Repertoires crees dans {BENCHMARK_DIR}")


FILE_MAP: dict[str, str] = {
    "P-01": "police/P-01_rapport-initial.txt",
    "P-02": "temoignages/P-02_sophie-laurent.txt",
    "P-03": "temoignages/P-03_marc-duval.txt",
    "P-04": "temoignages/P-04_romain-fabre.txt",
    "P-05": "temoignages/P-05_karim-belhadj.txt",
    "P-06": "forensique/P-06_autopsie.txt",
    "P-07": "numerique/P-07_telephonie.txt",
    "P-08": "numerique/P-08_banque-elise.txt",
    "P-09_descriptions": "photos/P-09_scene-descriptions.txt",
    "P-10": "osint/P-10_assurance-vie.txt",
    "P-11": "numerique/P-11_instagram.txt",
    "P-12": "osint/P-12_kbis-webcraft.txt",
    "P-13": "police/P-13_main-courante.txt",
    "P-14": "osint/P-14_casier-chevalier.txt",
    "P-15": "forensique/P-15_adn-megot.txt",
}


def write_evidence_files() -> None:
    """Write each piece of evidence to its file."""
    for evidence_id, relative_path in FILE_MAP.items():
        filepath = BENCHMARK_DIR / relative_path
        content = EVIDENCE[evidence_id]
        filepath.write_text(content, encoding="utf-8")
        print(f"[OK] {evidence_id} -> {filepath.name}")


def write_manifest() -> None:
    """Write the JSON manifest describing all evidence and waves."""

    manifest = {
        "case": {
            "name": "Affaire MOREAU",
            "reference": "#2019-4472-LY",
            "description": (
                "Disparition et meurtre d'Elise Moreau, 34 ans, "
                "pharmacienne a Lyon. Corps retrouve 11 jours plus tard "
                "a Miribel-Jonage. Strangulation manuelle. Affaire classee "
                "en mars 2021, reinvestigation ouverte en avril 2026."
            ),
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
                "source": "Requisition Orange/SFR/Free",
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
                "title": "Photos scene de decouverte (descriptions)",
                "type": "image",
                "source": "PTS Lyon",
                "reliability": 90,
                "source_date": "2019-09-25",
                "file": "photos/P-09_scene-descriptions.txt",
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
                "title": "Analyse ADN megot (correspondance 2026)",
                "type": "pdf",
                "source": "Labo PTS Lyon",
                "reliability": 85,
                "source_date": "2026-04-03",
                "file": "forensique/P-15_adn-megot.txt",
                "wave": 4,
            },
        ],
        "waves": {
            "1": {
                "name": "Dossier initial",
                "description": "Ouverture cold case -- pieces de l'enquete initiale",
                "evidence_ids": ["P-01", "P-02", "P-03", "P-04", "P-05", "P-06"],
            },
            "2": {
                "name": "Donnees numeriques",
                "description": "Requisitions telephonie + banque",
                "evidence_ids": ["P-07", "P-08"],
            },
            "3": {
                "name": "OSINT + complementaires",
                "description": "Resultats recherche OSINT automatique + pieces administratives",
                "evidence_ids": ["P-10", "P-11", "P-12", "P-13", "P-14"],
            },
            "4": {
                "name": "Percee forensique",
                "description": "Photos scene + correspondance ADN tardive",
                "evidence_ids": ["P-09", "P-15"],
            },
        },
        "expected_contradictions": [
            {
                "id": "C1",
                "description": "Sophie dit Elise partie 'vers minuit' vs SMS 'je rentre bientot' a 23h47",
                "evidence": ["P-02", "P-01"],
            },
            {
                "id": "C2",
                "description": "Marc dit 'rentre directement' vs borne Villeurbanne a 23h52",
                "evidence": ["P-03", "P-07"],
            },
            {
                "id": "C3",
                "description": "Romain dit 'parti juste apres Marc ~23h45' vs Sophie implique decalage > 45 min",
                "evidence": ["P-04", "P-02"],
            },
            {
                "id": "C4",
                "description": "Karim change d'heure, de comportement observe, et ajoute un detail (conducteur)",
                "evidence": ["P-05"],
            },
            {
                "id": "C5",
                "description": "Tessier dit 'aucun contact depuis 2018' vs bornes telephoniques a Lyon + SMS a Chevalier",
                "evidence": ["P-13", "P-07"],
            },
            {
                "id": "C6",
                "description": "Sophie omet le DM 'il faut que je te parle de quelque chose. Important.'",
                "evidence": ["P-02", "P-11"],
            },
            {
                "id": "C7",
                "description": "Ecchymoses anterieures (3-5 jours avant deces) non expliquees par aucun temoin",
                "evidence": ["P-06"],
            },
        ],
        "expected_hypotheses": [
            {
                "id": "H1",
                "title": "Marc Duval (compagnon) -- auteur",
                "expected_final_score_range": [30, 40],
            },
            {
                "id": "H2",
                "title": "Julien Tessier + Yann Chevalier -- auteurs (duo)",
                "expected_final_score_range": [75, 85],
            },
            {
                "id": "H3",
                "title": "Romain Fabre (collegue licencie) -- auteur",
                "expected_final_score_range": [15, 25],
            },
            {
                "id": "H4",
                "title": "Inconnu / agression aleatoire",
                "expected_final_score_range": [5, 10],
            },
            {
                "id": "H5",
                "title": "Marc Duval -- commanditaire via Chevalier",
                "expected_final_score_range": [20, 30],
            },
        ],
        "monitoring_jobs": [
            {
                "type": "searxng",
                "query": "\"Julien Tessier\" Grenoble",
                "interval_hours": 6,
                "entity": "Tessier",
            },
            {
                "type": "searxng",
                "query": "\"Yann Chevalier\" Villeurbanne",
                "interval_hours": 6,
                "entity": "Chevalier",
            },
            {
                "type": "searxng",
                "query": "\"pharmacie du parc\" lyon moreau",
                "interval_hours": 24,
                "entity": "Elise",
            },
            {
                "type": "robin",
                "query": "chevalier OR tessier lyon meurtre",
                "interval_hours": 24,
                "entity": None,
            },
            {
                "type": "searxng",
                "query": "WEBCRAFT SARL Villeurbanne",
                "interval_hours": 24,
                "entity": "WEBCRAFT",
            },
        ],
    }

    manifest_path = BENCHMARK_DIR / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    print(f"[OK] Manifest -> {manifest_path}")


def main() -> None:
    print("=" * 60)
    print("NEXUS -- Generation du benchmark Affaire MOREAU")
    print("=" * 60)

    create_directory_structure()
    write_evidence_files()
    write_manifest()

    print()
    print(f"Benchmark complet genere dans : {BENCHMARK_DIR}")
    print(f"  - {len(FILE_MAP)} fichiers de preuves")
    print(f"  - 1 manifest.json")
    print()
    print("Pour ingerer dans NEXUS, utiliser les vagues definies dans le manifest.")
    print("Vague 1: 6 preuves (dossier initial)")
    print("Vague 2: 2 preuves (numerique)")
    print("Vague 3: 5 preuves (OSINT)")
    print("Vague 4: 2 preuves (forensique)")


if __name__ == "__main__":
    main()
