# Babel — Bibliothèque universelle multilingue P2P

**Status :** Idée post-v1.0. Pas un sprint actif. Doc de référence pour future décision roadmap.
**Date :** 2026-04-27
**Auteur :** session ideation @ alexandria-globe
**Trigger** : ne PAS démarrer avant tag SBFB v1.0 (Day 0 / scope cuts respect).

## Pitch en une phrase

Traduire l'intégralité de Project Gutenberg (~75 000 livres) dans toutes les langues du monde
(~8 000), via NLLB-200 + bootstrap biblique, distribué sur le réseau SBFB par des volontaires GPU,
avec provenance signée Ed25519 et stockage iroh-blobs immuable.

## Le gap vérifié (recherche du 2026-04-27)

**Personne n'a publié un corpus Project Gutenberg traduit IA dans plus de 16 langues.**

Vérifié exhaustivement :
- **Google** : Translate API payante, pas de corpus public
- **Meta NLLB-200** : modèle libre, démo "Stories Told Through Translation" sur ~5 livres,
  démo morte depuis avril 2025, archives non publiées (décision active de ne pas publier)
- **Microsoft / OpenAI / Anthropic** : aucun
- **Cohere Aya** : instructions multilingues, pas de littérature
- **MADLAD-400 (Google)** : 419 langues monolingues, pas de traduction
- **FineTranslations (HF, jan 2026)** : 1T tokens, 500 langues mais X→anglais sur web,
  pas anglais→X sur littérature
- **OPUS Books** : 16 langues, ~50 livres, traduit humain pré-IA
- **eBible / JHU Bible Corpus** : 1500+ langues mais Bible uniquement, missionnaire humain

→ **Le trou est réel et de plus de 70 ans de littérature numérique.**

Pourquoi personne ne le fait :
- Cannibalise les API payantes (Google Translate, DeepL)
- Risque légal sur œuvres dérivées dans juridictions où traductions modernes sont copyrightées
- Coût compute (~$280k cloud) pour zéro ROI business
- Risque qualité publié = embarrassant pour produit commercial
- Pas dans le business model des géants

Pour SBFB : aucune contrainte business, distribué, communautaire, opt-out par langue possible.
Le gap est exactement la forme du réseau.

## Architecture haut niveau

```
Project Gutenberg (~75k livres)
    ↓ scraper one-shot → blobs iroh signés
    ↓
work queue (gossip)
    ↓
[volontaires GPU exécutant NLLB-200 ou silnlp pipeline]
    ↓ chunks traduits, 3-worker consensus + BLEU
    ↓
blob signé Ed25519 + provenance SLSA-L1
    ↓
catalogue distribué (par langue / par auteur / par sujet)
    ↓
reader app (front-end : Alexandria globe + library Interstellar)
```

## Ce que SBFB a déjà — réutilisable tel quel

- iroh-blobs : stockage immuable, dédup, distribution
- gossip : coordination work queue
- nexus-worker-core + LlmBackend : pattern Ollama existant (Sprint 31)
  → calque pour NLLB-200
- coordinator FastAPI + dispatcher : task scheduling
- Kudos : reputation par contributeur GPU et par valideur natif
- Signature Ed25519 + SLSA-L1 (Sprint 14 verified deploy)
- App SDK + bridge postMessage : front-end iframe

## Ce qu'il faut construire (sprints estimés à rythme actuel)

| Sprint | Livrable | Durée |
|---|---|---|
| A | Backend NLLB-200 dans nexus-worker-core (calque LlmBackend Ollama) | 1 |
| B | Wire format `TranslationTask` + coordinator dispatch | 1 |
| C | Scraper Gutenberg + chunking livre + reassembly | 1 |
| D | Pipeline e2e : 1 livre → 1 langue → blob signé | 1 |
| E | Consensus 3-workers + BLEU score validation | 1 |
| F | nexus-app-babel (lecteur + browse) | 2 |
| G | Dashboard couverture (réutilise globe Alexandria) | 1 |
| H | Bible bootstrap (eBible ingestion + fine-tune par langue) | 2 |
| I | Validation native-speaker UI + Kudos integration | 1 |

**v1.0 publique (200 langues NLLB auto) : ~7 sprints**
**v2.0 (Bible bootstrap, ~1500 langues) : +3 sprints**
**v3.0 (toutes langues + communautés) : multi-année, scale humain**

## Calculs de scaling — temps total

**Volume :** ~75k livres × ~110k tokens/livre = **~8 G tokens source**
Output ratio ~1.05 → **~8 G tokens à générer par langue cible**
Pour 100 langues : **800 G tokens**
Pour 200 langues : **1.6 T tokens**
Pour 8000 langues (sans exception) : **64 T tokens**

**Throughput volontaire moyen :** ~300 tok/s × 8h/jour = ~8.6 M tok/jour

| Volontaires | 100 langues | 200 langues | 8000 langues |
|---|---|---|---|
| 100 | 2.5 ans | 5 ans | 200 ans |
| 1 000 | 3 mois | 6 mois | 20 ans |
| 10 000 | 9 jours | 18 jours | 2 ans |
| 100 000 (Folding@home) | 23h | 46h | 74 jours |

**Sweet spot pour v1.0 : 1000 volontaires actifs × 3 mois = 100 langues complètes.**

## Bootstrap pour langues à faibles ressources

Pour les langues hors NLLB-200 (~6800 langues), technique du **pivot biblique** :
- Bible existe en ~3500 langues (eBible, JHU Bible Corpus)
- Alignement par numéro de verset = corpus parallèle automatique
- Fine-tune NLLB sur 31k versets parallèles → modèle MT v0.1 pour la langue
- Bootstrap silencieux : le contenu traduit ensuite est Tolstoï/Cervantès, pas la Bible
- User voit "Don Quichotte en Wolof", pas "Don Quichotte traduit via la Bible"

**Composants existants réutilisables :**
- `silnlp` (SIL International) : pipeline production NLLB + Bible, AGPL
- `BibleNLP/zero-draft-translation` : zero-draft pour nouvelle langue
- `BibleNLP/ebible` : 1000 langues Bible alignées
- `Helsinki-NLP/opus-mt` : 1900 modèles MarianMT

→ **On wrappe, on ne ré-invente pas la NMT.**

## Positionnement défendable

Babel n'est pas un projet de recherche NMT. C'est :

1. **Couche de calcul P2P** au-dessus de pipelines existants
2. **Couche de provenance** (Ed25519, SLSA-L1)
3. **Couche de distribution** (iroh-blobs, app store SBFB)

Les géants font la science (modèles, données). Babel fait l'**infrastructure ouverte
de production et distribution**. Sans Babel, on dépend d'Azure et HuggingFace pour exécuter.
Avec Babel, le réseau est l'infrastructure.

## Risques identifiés

1. **Qualité poésie** : NLLB faible. → v1 prose seulement, poésie attend modèles meilleurs.
2. **Stockage** : 75k × 100 = 7.5M fichiers ~1.1 TB. → iroh-blobs déduplique, réplication 5x.
3. **Validation** : qui dit qu'une trad est bonne ? → BLEU round-trip + signalement utilisateur
   + kudos négatifs. Validation native-speaker post-v2.
4. **Énergie** : compute distribué = transparence wattheures par contribution.
5. **Légal** : domaine public Gutenberg strict. Stockage P2P sans entité légale centrale.
   Opt-out par langue/communauté respecté.
6. **Pression diplomatique** : possible que Meta ait tué sa démo NLLB pour cette raison.
   → Forkable et opt-out par construction.
7. **Biais culturel Bible-pivot** : utilisation silencieuse en bootstrap, pas exposé à
   l'utilisateur final.

## Pourquoi POST-v1.0 SBFB et pas avant

- v1.0 SBFB n'est pas tagué (zones rouges P0 R-iroh-audit / R-wasmtime-cve restent)
- Diviser l'attention = retarder les deux
- Babel devient bien plus **frappant** comme première grande app post-v1.0 que comme
  projet parallèle qui se traîne
- Cohérent avec Day 0 : pas de scope creep avant v1.0

## Chronologie projetée

- **Q2 2026 (en cours)** : Sprints S31-S40, finalisation v1.0 SBFB
- **Q3 2026** : tag v1.0, audit publique, premiers utilisateurs
- **Q4 2026** : Sprint Babel A-G (v1.0 publique 200 langues)
- **2027** : Sprint Babel H-I (Bible bootstrap, 1500+ langues)
- **2028+** : phase communautaire, partenariats Masakhane / AmericasNLP / SIL

## Trigger de réveil de ce doc

Relire ce document quand :
- Tag v1.0 SBFB est posé (donc lecture obligatoire au moment de planifier post-v1.0)
- Quelqu'un dans la communauté SBFB demande "quelle est la première grande app à construire ?"
- Annonce d'un acteur tiers qui lancerait un projet équivalent (vérifier que le gap existe encore)

## Liens / références

- Project Gutenberg : `gutenberg.org`
- NLLB-200 : `https://ai.meta.com/research/no-language-left-behind/`
- silnlp : `https://github.com/sillsdev/silnlp`
- BibleNLP/ebible : `https://github.com/BibleNLP/ebible`
- BibleNLP/zero-draft-translation : `https://github.com/BibleNLP/zero-draft-translation`
- Masakhane : `https://github.com/masakhane-io`
- AmericasNLP : `https://turing.iimas.unam.mx/americasnlp/`
- AI4Bharat IndicTrans2 : `https://github.com/AI4Bharat/IndicTrans2`
- JHU Bible Corpus : `https://christos-c.com/bible/`
- OPUS-MT : `https://github.com/Helsinki-NLP/Opus-MT`

## Décision attendue

Aucune décision à prendre maintenant. Doc de référence en attente du tag v1.0.
À ce moment-là, candidat fort pour première application phare du réseau SBFB.
