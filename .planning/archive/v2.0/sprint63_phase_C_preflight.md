# Sprint 63 Phase C — preflight G8

Date : 2026-05-15 | HEAD : `51aff78` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, OSS prior art obligatoire, research before code
- feedback_context7_systematic.md : context7 avant tout code touchant lib/API — applique (shadcn Dialog query)

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (npm provenance UI, Sigstore/cosign, C2PA Verify Tool), APPROACH-ALIGNED — le pattern progressive disclosure (badge → modal detail → verify button) est identique a npm provenance (badge npmjs.com → provenance page) et C2PA Verify Tool (web utility → detail view). Le bridge postMessage est SBFB-specifique mais suit le pattern standard iframe communication. Clean.
- S1b deps : 0 nouvelle dep Phase C. shadcn Dialog deja installe (dialog.tsx present, utilise dans 4 composants). context7 shadcn/ui query confirme API stable (Dialog/DialogContent/DialogHeader/DialogTitle/DialogDescription/DialogTrigger). Clean.
- S2 historiques : 4 fichiers cibles scannes (sbfb-bridge.js, useBridge.ts, http.rs, BrowsedProject.tsx), git log historique + archive v1.2 S56 preflight (memes fichiers, clean). Memory feedback : aucune contrainte bridge/provenance/modal. Clean.
- S3 threat model : fast-path verified. Phase C ajoute 3 methodes bridge read-only (provenance_get, provenance_verify, feed_cursor_get). Pas de nouveau composant de securite, pas de wire format. Sandbox model existant couvre le bridge postMessage. Pas d'entree S63 dans HARDENING_ROADMAP. Clean.
- S4 wire format : fast-path verified. Phase C ne touche pas canonical.rs ni schemas/. Bridge messages = transport-level hors wire format. VERSION=1 preserve, Day 0 preservees. Clean.

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~2m / 3 projets OSS consultes (npm provenance, Sigstore, C2PA) / finding : APPROACH-ALIGNED
- S1b : ~30s / 1 lib scannee (shadcn Dialog) / finding : clean
- S2 : ~30s / 4 fichiers + archive scan / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase C.
