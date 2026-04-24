# Sprint 26 Phase C — preflight G8

Date : 2026-04-24 | HEAD : `9623f3e` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option, research before code, OSS prior art obligatory (G10)
- feedback_context7_systematic.md : context7 MCP avant code touchant lib/API — vérifié tracing-etw + chrono
- Tensions plan vs memory : aucune

## Adaptations plan mineures (non-bloquantes)

1. **Edition** : plan dit `edition = "2024"`, workspace utilise `edition = "2021"` — utiliser `edition.workspace = true`
2. **Deps** : utiliser workspace versions (serde 1.0, serde_json 1.0, chrono 0.4, tracing 0.1, tempfile 3.13) + chrono features serde ajoutée
3. **consent.rs** : plan dit `ConsentWatcher::handle_change` — la méthode n'existe pas, insertion dans `ConsentConfig::save_atomic` (point de mutation)
4. **auth.rs** : plan dit "rotation handler" dans auth.rs — le handler est `key_rotation_handler.rs` (fichier séparé)
5. **capability_store.py** : plan dit `enable()`/`disable()` — vérifier méthodes actuelles, adapter insertion

## Scans (all clean)

- S1a OSS prior art : 3 projets/écosystèmes recherchés (tracing ecosystem, audit-logging crate, rust-secure-logger), APPROACH-ALIGNED — l'approche typed SecurityEvent enum + EventWriter trait + platform-specific writers est le pattern standard des systèmes d'audit. Pas de lib mature couvrant exactement le domaine (audit événements sécurité P2P avec ETW/journald/oslog). Pas de APPROACH-NAIVE ni LIB-EXISTS.
- S1b deps : 5 deps scannées (serde 1.0, chrono 0.4, tracing 0.1, tracing-etw, tempfile 3.13), 0 CVE, 0 breaking change — clean
- S2 historiques : 6 fichiers cibles scannés, 2 commits historiques trouvés (S18 E2 threat-model canary auto-publish + S25 B key rotation) — aucun ne rejette l'approche audit events, aucun conflit — clean
- S3 threat model : **FULL scan** (nouveau composant sécurité). SecurityEvent est une couche de mitigation (audit trail), pas une surface d'attaque. Fichier JSONL dans `~/.sbfb/` même trust boundary que consent.json/auth_token (AD2 user-mode malware même risque). ETW events vont dans le log système (plus dur à tamper). Log rotation non couverte Phase C (acceptable, carry S27+ si besoin). Aucune régression T0-T5. HARDENING_ROADMAP §3 S26 mentionne A3 OS audit dans le scope. CAPABILITY_TOGGLES.md §6 référence déjà l'intégration nexus-events-core — clean
- S4 wire format : fast-path verified — nouveau crate sans wire format existant touché, pas de canonical.rs modifié, pas de `*_VERSION` bumped, Day 0 préservées — clean

## Télémétrie preflight
- Durée totale : ~4m
- S1a : ~2m / 3 projets OSS consultés / finding : APPROACH-ALIGNED
- S1b : ~1m / 5 libs scannées / finding : clean
- S2 : ~30s / 6 fichiers, 4 commits scannés / finding : clean
- S3 : FULL / ~1m
- S4 : fast-path / ~15s

## Action
Procéder code phase C.
