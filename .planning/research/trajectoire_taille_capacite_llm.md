# Trajectoire taille / capacité des LLM dans le temps — et quand le local remplace le cloud

> **Statut** : recherche hors-sprint / découverte (2026-06-24). Figé sur demande PO.
> **Source** : workflow multi-agent Opus 4.8 1M `llm-size-capability-trajectory`
> (run `wf_dc1ba024-10c`) — 7 lentilles web-grounded (tendance densité + écosystèmes
> US/Chine/France/autres + matériel-coding + sceptique adversarial) + synthèse datée.
> **Honnêteté de méthode** : la *tendance* et l'*état mi-2026* sont solidement datés/sourcés. Les
> *horizons* (+1/+2-3/+3-5 ans) sont de la **prospective**. Plusieurs chiffres frontier 2026
> (Opus 4.8, GPT-5.5, DeepSeek-V4…) sont **au-delà du cutoff fiable ~jan 2026** → à re-vérifier sur
> sources primaires avant de s'y appuyer. **Confiance globale : moyenne.**
> **Voir aussi** : `puissance_noeud_enabler_4sur5.md`, `doctrine_contrat_pour_llm.md`,
> `sharding_design_addendum_sota_2026-05-30.md`.

## Question posée

Quand un modèle « assez puissant » tiendra-t-il sur le **PC d'un utilisateur ordinaire** (réf.
RTX 5080 16 Go VRAM, 4-bit) — et, pour le code, assez bon pour **remplacer Claude Code / Codex**
dans le process Factory ? Avec le rôle du **partage GPU SBFB comme pont** entretemps (projets
humanistes type Babel + modèles trop gros pour un seul PC).

## En une phrase

Mi-2026, un PC 16 Go fait **déjà** tourner du **généraliste GPT-4-class** (Gemma 3 27B, Mistral
Small 24B) et un assistant de code utile — mais le **coding agentique fiable type Claude Code reste
~30-38 pts SWE-bench au-dessus** du meilleur modèle local grand-public. Bascule locale crédible
pour le coding sérieux : **~fin 2027 (utile) → 2028-2029 (remplacement) → 2029-2031 (parité
frontier-fermé)**. D'où le sharding GPU SBFB comme pont.

## Le moteur quantitatif — la « Densing Law »

> Densité de capacité (capacité **par paramètre**) qui **double tous les ~3,3-3,5 mois** (Xiao et
> al., arXiv:2412.04315, déc 2024 ; Nature Machine Intelligence nov 2025 ; ~3,2 mois post-ChatGPT
> sur 51 modèles fév 2023→avril 2025). À iso-capacité, **params /2 tous les ~3,5 mois** ; coût
> d'inférence « niveau GPT-3.5 » /266,7 en 20 mois.

Motif « petit-rattrape-grand-d'il-y-a-N-mois » vérifié : MiniCPM-2.4B (fév 2024) = Mistral-7B
(sep 2023) en 35 % des params · **Qwen3-4B ≈ Qwen2.5-72B** (~18× en ~8 mois) · Phi-4 14B = Llama
3.3 70B (distillation o3-mini) · OLMo 2 32B bat GPT-4o-mini · **Gemma 3 27B bat Llama 3 405B et
DeepSeek-V3 671B** sur Chatbot Arena.

**3 leviers cumulatifs** : (1) **distillation** (DeepSeek-R1 → denses 1.5-70B ; Sakana
TinySwallow-1.5B sur iPhone) ; (2) **quantization 4-bit** (Q4_K_M garde ~92-99 % de FP16, +0,5-2 %
perplexité, VRAM /4 ≈ 0,56 Go/1B ; reco « Q5_K_M+ pour le code ») ; (3) **MoE sparse**.

**Verrou clé** : le **MoE réduit le calcul, pas la VRAM résidente** — un 80B/3B-actifs reste
**80B en VRAM**. C'est pourquoi les open-weights qui frôlent le frontier (DeepSeek-V4, GLM, Kimi :
MoE 0,4-1,6T) **ne tiennent pas** sur un PC, malgré peu d'actifs.

> Extrapolation (optimiste, bornée par un plancher de connaissances) : pour égaler un frontier
> d'**aujourd'hui**, ~1/8 params dans 1 an, ~1/64 dans 2 ans, ~1/512 dans 3 ans.

## Par écosystème (poids OUVERTS vs FERMÉS)

| | Posture | Force on-device / leaders | Pertinence directe |
|---|---|---|---|
| **Chine** | **Ouverts**, **mène l'open-weight efficace** (MoE sparse ; Apache/MIT) ; rattrape le frontier fermé à ~14-18 pts SWE-bench en ~12 mois | **Qwen3-14B** sweet-spot 16 Go (~94 tok/s) ; **Qwen3-Coder-30B-A3B** meilleur code directement runnable (~17-20 Go, tendu) ; DeepSeek V3.1, GLM-4.6, Kimi K2, MiniMax M2 | Réservoir n°1 de modèles **ouverts** à héberger/mutualiser |
| **USA** | **Double jeu** : frontier 100 % **fermé/cloud** (Claude, GPT, Gemini) + ouvert on-device de qualité | **Gemma 3 27B = GPT-4-class sur 16 Go** ; gpt-oss-20b ; Phi-4-mini (~o1-mini maths, 4 Go) ; Nemotron-Nano-9B ; OLMo 2 32B (full-open) ; Apple ~3B | Claude/Codex = **baseline frontier** à égaler |
| **France / EU** | **Ouverts souverains**, fer de lance on-device EU (repose quasi tout sur Mistral) | **Mistral Small 3.1 24B** (~13,4 Go, tient juste) ; **Devstral Small 24B = meilleur candidat code local** (68 % SWE-bench, ~13 pts sous frontier) ; Codestral (95,3 % FIM, faible multi-fichiers) ; Ministral, SmolLM3 ; Teuken/Pharia (DE) | Voie **souveraine** ; Devstral = candidat local n°1 pour le coding |
| **Autres** | Ouverts majoritaires (licences hétérogènes) | Falcon-H1R-7B (Mamba2, contexte 256K peu coûteux) ; EXAONE, Solar Pro 2, Sarvam, GigaChat ; **Cohere Command A Translate (SOTA 23 langues)** ; **Prime Intellect INTELLECT-2** (32B RL décentralisé, stack TOPLOC/SHARDCAST) | **Command A Translate → Babel** ; **INTELLECT-2 valide directement le modèle SBFB** de mutualisation GPU |

## Timeline PC-user (réf. RTX 5080 16 Go, 4-bit)

```
 MAINTENANT (mi-2026)          +1 an (~2027)         +2-3 ans (~2028-29)      +3-5 ans (~2029-31)
 ───────────────────          ─────────────         ──────────────────       ──────────────────
 GÉNÉRALISTE GPT-4-class      14-24B ≈ frontier      coder local ~70-75%      coder local ≈ frontier
 DÉJÀ local (Gemma 27B,       ouvert d'il y a 12mo   SWE-bench (= meilleur    FERMÉ de ~2026 (~88%)
 Mistral Small 24B)          + MoE 30B/3B mûrs       OUVERT de 2026)          SI Densing Law tient
                              + hybrides SSM (KV↓)
 CODE local ~50% SWE-bench    ~60-65% SWE-bench      remplace Claude/Codex    remplacement Factory
 vs ~88% frontier (−38pts)    refactor multi-fich.   sur large part Factory   local quasi-complet
 = autocomplete/édition OK,   supervisé OK ; verdict  ; agentique repo-scale  MAIS la cible frontier
 PAS l'agentique autonome     reste cloud            fiabilité = incertain    aura encore avancé
 confiance: ÉLEVÉE            confiance: MOYENNE     confiance: FAIBLE-MOY    confiance: FAIBLE
```

## Le déclencheur concret pour SBFB

Ce n'est **pas un seuil unique** mais un **ordre de bascule par type de tâche** (du plus tôt au
plus tard) :

```
autocomplete / édition ciblée    →  DÉJÀ local
Q&A code, mono-fichier, tests     →  local maintenant / +1 an
refactor multi-fichiers supervisé →  ~+1 an
── seuil Factory "utile en local" : ≤24 Go, ~70%+ SWE-bench SUR CODE FRAIS (non contaminé) ──
agentique autonome repo-scale     →  EN DERNIER (~2028-2031) ← terrain de Claude Code/Codex
```

**Décision SBFB défendable** : garder le coding Factory sur **Claude Code/Codex** comme
orchestrateur/verdict tant que ce seuil n'est pas franchi (~fin 2027 « utile », ~2028-2029
« remplacer vraiment », parité frontier-fermé-courant ~2029-2031).

**Le pont SBFB sert deux usages à deux vitesses :**
- **Inférence d'app humaniste (Babel/traduction)** = tolérante à la latence, batch, non-interactif
  → **passe local/mutualisé dès maintenant**. Le sharding cross-machine permet **aujourd'hui** de
  faire tourner les **gros modèles ouverts non-portables** (Mistral Large 2 123B, DeepSeek-V3,
  **Cohere Command A Translate 23 langues**, GLM/Kimi) sur des projets soutenus par qui le veut.
  **Cas d'usage immédiat et le mieux justifié.**
- **Coding agentique sérieux** = sensible à la latence (boucles itératives, dizaines d'appels
  outils) → le sharding WAN naïf à 1-2 tok/s est **inutilisable en interactif** (Petals ~1 step/s ;
  améliorable par speculative decoding, débit-borné au lieu de latence-borné). SBFB sert ici à
  **attendre** que la Densing Law fasse descendre les ~70 % SWE-bench dans 16-24 Go.

> Validation directe du modèle SBFB : les collectifs décentralisés (Prime Intellect **INTELLECT-2**,
> 32B RL sur essaim hétérogène, stack PRIME-RL/**TOPLOC**/SHARDCAST — exactement les primitives
> S77) montrent un compute décentralisé qui croît ~+20×/an vs ~+5×/an pour le frontier.

## Le contre-point sceptique (solide à ~60-70 %)

1. **La cible bouge** : le frontier a cessé de rétrécir et **re-grossit** (Epoch AI : 80 % que la
   prochaine gen dépasse GPT-4 en taille ; RL/raisonnement long/agents tirent taille ET compute
   d'inférence vers le haut). La Densing Law dit qu'on atteint une capacité *donnée* avec moins de
   params — pas que la capacité *maximale* descend en VRAM grand-public.
2. **« open-weight rattrape » ≠ « local »** : les ouverts proches du frontier sont des MoE géants
   (~800 Go+ en VRAM) ; les « 49B actifs » trompent.
3. **Le vrai local plafonne** : Qwen3-Coder-30B-A3B ~52 % (vs ~70 % son grand frère cloud) ;
   Gemma 3 27B ~6,6 % sur tau2-bench/SWE-bench (la capacité ne se lit pas à la taille).
4. **KV-cache ignoré** : « le modèle tient en 16 Go » oublie le cache du long contexte agentique
   (Llama 3.1 8B @128k → ~16 Go de cache → ~21,5 Go total, déjà hors 5080 ; crossover cache=poids
   ~32k tokens, pile la zone du tool-use). **Mur le plus fragile du sceptique** : un saut archi
   SSM/Mamba2 (Falcon-H1, hybrid-attention) pourrait le casser et **avancer le calendrier de
   plusieurs années**.
5. **Inférence locale lente** tue l'agentique itératif (32B seulement en Q2 dégradé ; KV-cache
   paginé « slows to a crawl »).
6. **Contamination des benchmarks** : « The SWE-Bench Illusion » (arXiv:2506.12286) — 32,67 % des
   patches réussis fuitent la solution, 76 % de rappel de file-paths du training. → le « good
   enough local » mesuré est **au-dessus du réel sur code frais**. **C'est exactement pourquoi la
   doctrine T2 (acceptance JSON anti-contamination sur code frais) est le bon arbitre.**
7. **Q4 pas gratuit pour le code** : +0,5-2 % perplexité = taux d'échec non-modeste sur chaînes de
   20+ étapes exactes (un token faux = appel d'outil malformé).

> Probabilité honnête que le coding agentique autonome repo-scale à fiabilité frontier **doive
> rester cloud** plus longtemps que l'optimisme ne suppose : **~60-70 % sur 2-4 ans.**

## Deux signaux-déclencheurs à surveiller (veille)

1. **Un coder ≤24 Go (4-bit) qui passe ~70 %+ SWE-bench Verified _sur code frais non-contaminé_**
   ET tient le KV-cache du contexte agentique → le coding Factory peut commencer à déléguer en
   local.
2. **Un saut archi (hybride SSM/Mamba2) qui tue le coût KV-cache** du long contexte → avance le
   calendrier de plusieurs années.

## Verdict

La trajectoire taille↓/capacité↑ est **réelle et bien documentée** (Densing Law). Pour le
**généraliste et l'inférence humaniste tolérante (Babel)**, la bascule local est **déjà franchie
ou imminente** : utiliser le partage GPU SBFB **dès maintenant** pour mutualiser les gros modèles
ouverts non-portables. Pour le **coding agentique fiable** remplaçant Claude Code/Codex, l'écart
est encore large (~30-38 pts SWE-bench) et la bascule sera **tardive et graduelle** : Claude
Code/Codex restent le bon défaut, le sharding est le pont, et la Densing Law travaille en
arrière-plan. **La stratégie PO est correcte et bien calibrée.**
