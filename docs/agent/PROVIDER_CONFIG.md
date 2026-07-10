# Provider Configuration

Configuration du driver LLM (qui code) et du verificateur LLM (qui
review/audit) dans le workflow SBFB.

## Combinaisons supportees

| Driver | Verificateur | Notes |
|--------|-------------|-------|
| Claude | Codex | Actuel. Codex = CLI OpenAI GPT-5.6 Sol (`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max` ; bascule 5.5→5.6 Sol 2026-07-10, tier flagship au reasoning `max`, acces verifie sur ce compte, CLI codex recent requis — cf. README §4.5.2 pour le piege de version PowerShell). |
| Claude | Claude | Fallback si Codex indisponible. Meme session = biais, mais acceptable ponctuellement. |
| Codex/GPT/local | Claude | Driver non-Claude code, Claude review-deep + audit. |
| LLM local | LLM local | Full offline. Moins de profondeur (pas WebSearch/context7), meme workflow. |
| Humain | Claude | Humain code, Claude review. |
| Claude | Humain | Claude code, humain review. |

## Adaptation provider

`sbfb-factory process prompt --provider {claude,codex,gpt,local,human}`
adapte le contenu du prompt selon le provider cible :

| Provider | Adaptation |
|----------|-----------|
| `claude` | Contenu complet. WebSearch, context7, Read 1M tokens. |
| `codex` | Contenu complet. Codex supporte les outils standards. |
| `gpt` | Contenu complet. GPT supporte les outils standards. |
| `local` | Lignes contenant WebSearch, context7, mcp__context7, mcp__claude retirees. Le workflow reste identique, la profondeur OSS est reduite. |
| `human` | Contenu complet en texte. L'humain execute les commandes manuellement. |

## Selection driver/verificateur

Le choix se fait au niveau Operator ou manuellement :

```
# Generer un prompt preflight pour un agent local
sbfb-factory process prompt --kind preflight --provider local --depth deep

# Generer un handoff pour transfert vers GPT
sbfb-factory process prompt --kind handoff --provider gpt --depth deep
```

## Contraintes invariantes (quel que soit le provider)

1. Le workflow (preflight → code → review → Codex → commit) est
   identique pour tous les providers.
2. Le verdict final doit etre exactement `## Verdict: PASS`.
3. Le commit body doit contenir les 9 sections obligatoires.
4. Le Codex review doit etre l'output brut de `codex exec -o`.
5. Les artefacts planning vivent dans `.planning/active/`.
6. Les prompts portables vivent dans `prompts/agent/`.
