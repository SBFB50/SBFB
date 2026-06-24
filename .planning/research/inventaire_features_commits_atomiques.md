# Inventaire des features — registre des commits atomiques (Sprint 0-77)

> **Statut** : recherche hors-sprint / découverte (2026-06-24). Figé sur demande PO.
> **Source** : workflow multi-agent Opus 4.8 1M `atomic-commit-feature-inventory`
> (run `wf_24e612af-729`) — 8 finders (7 bandes de sprints + sweep `fix`) minant les bodies de
> commits, + synthèse. Recensement de référence vérifié : `git log` HEAD = **1228 commits**,
> **324 `feat`**, **152 `fix`**, Sprint 0 → 77.
> **But** : ne plus jamais sous-compter la masse réelle de features (les analyses stratégiques
> raisonnaient sur ~8 briques). Carte ancrée par hash, réutilisable pour kickoff S78/S79.

## Chiffres durs

- **1228 commits** (HEAD) · **324 `feat`** + **152 `fix`** atomiques · Sprint 0-77.
- **~2 650 sous-features** minées (bodies décomposés) · **439 entrées-feature** agrégées.
- **17 domaines** · ~6 sous-features par commit · 6 domaines > 200 sous-features.
- **~34 % du travail (515 sous-features) = sweep `fix`** : durcissement par audit-gate, pas déclaré.

## Inventaire par domaine (trié par profondeur)

```
securite-hardening      ████████████████████████████████████████ 316 sous  (55 feat)
p2p-transport           ████████████████████████████████████ 286            (38)
frontend-ux             ███████████████████████████████████ 273             (48)
verification-inference  ████████████████████████████████ 256                (35)
ci-tooling              ███████████████████████████ 214                     (48)
factory-authoring       ██████████████████████████ 205                      (31)
worker-engine           ███████████████████████ 185                         (20)  + profond/feat
hosting-blobserve       ██████████████████████ 174                          (31)
coordinator-dispatch    ████████████████████ 159                            (23)
provenance-supply       ████████████████ 125                                (22)
crypto-wire             ████████████████ 124                                (18)  socle critique
gouvernance-curation    ███████████████ 120                                 (19)
docs-process            ██████████████ 114                                  (29)
rrv-search              ██████████ 81                                       (10)
compute-sharding        ███████ 53                                          (7)   PHARE, PROVISIONAL
p2p-resilience          ██ 17                                               (4)
persistence-durability  █ 9                                                 (1)
```

| Domaine | feat | sous | Note de profondeur (ancrée hash) |
|---|--:|--:|---|
| **securite-hardening** | 55 | 316 | Le plus dense. Défense en profondeur multi-couche : loopback bearer+Host+Origin mitige CVE-2025-49596 (`d7c265a`), UDS SO_PEERCRED + Named Pipes DACL (`1cfde89`), TokenRotator 24h (`677556f`), git option-injection fix (`f19ed83`). >½ vient du sweep `fix` = durci par audit, pas théâtre. |
| **frontend-ux** | 48 | 273 | Shell React mature : TabView schema-driven Pydantic→Zod 11 kinds (`667ae6b`), Factory Viewer+Operator i18n (`c12aadb`), ExecutionChat SSE (`95cae05`), Browse node-centrique (`4f52bea`). Polish UX honnête (intentions vs jargon). |
| **ci-tooling** | 48 | 214 | GHA 18-step (`ef28d75`), Woodpecker self-hosted VPS — LT-7 (`b5ec810`), Docker SHA256-pinné, test-harness multi-node. Domaine « méta » : part notable = plomberie de discipline. |
| **p2p-transport** | 38 | 286 | Cœur protocolaire : gossip+fetch_ticket 9-checks (`818429d`), NodeDirectory + pivot PULL (`f6637d3`), fetch_hash_multi ancre-d'abord (`0010450`), ALPN `sbfb/shard/1` (`81d667c`). iroh 0.97→0.98. |
| **verification-inference** | 35 | 256 | Cœur anti-triche : watermark SynthID E2E (`7bb656b`+`c5f35f7`), output-filter 3-couches stégano Unicode (`0862a9d`), N0 TOPLOC (`ce2f6a7`), N1 VRF (`fdc65a2`), N2/N3 clique+commit-reveal (`99ba7b8`). **Primitives NON câblées in-vivo (carry S78).** |
| **factory-authoring** | 31 | 205 | Arc 2-2.5 : bridge postMessage (`c32d9c7`), crate sbfb-factory CLI (`49d6bcd`), gates FG4-FG7 (`a201b3e`), prompt portability 8 kinds (`c68e989`), Babel dogfood (`faf4952`). Orienté mono-auteur/dogfood. |
| **worker-engine** | 20 | 185 | Profondeur/feat la plus élevée. worker-core 12 vagues (`accb7a3..9476be8`), consent 4 niveaux GDPR (`3247e88`), fork llama.cpp partial-decode CUDA sm_120+Metal (`14fa313`), claim shard fail-closed (`a93d8bb`), auto-spawn (`dc9a478`). |
| **hosting-blobserve** | 31 | 174 | Promesse produit la plus aboutie/éprouvée LIVE : blob-serve zip→iframe+CSP (`32a1dca`), fetch 3-tiers (`f5c2575`), AppStorage iroh-docs (`41e9e1f`), pin keep_online M18 (`4c1acc5`). Durabilité gagnée par debug live. |
| **coordinator-dispatch** | 23 | 159 | Migration Python→Rust (`a9cfb45`), alignement clé dispatch `task:` P0 dormant depuis S49 (`2f9238d`), absorption dans daemon (`63875d9`), routing DAG+churn Petals (`8ab8f97`). |
| **provenance-supply** | 22 | 125 | deploy-from-repo Keyoxide SLSA L1 multi-forge (`407af60`), warrant canary Ed25519 (`04c9621`), endpoint provenance M12 (`e362092`), TRUST_TAXONOMY (`ace05b0`). Honnête : auto-attestation L1, pas de reproductible tiers. |
| **crypto-wire** | 18 | 124 | Socle : JCS RFC 8785 + `DOMAIN_*_V1` (`1c1fcfb`), FROST-ed25519 K-of-N (`6a3f199`), FROST DKG air-gapped (`387b6b9`), shard wire+RunProof (`ebe6779`). Petit en count, structurellement critique. |
| **gouvernance-curation** | 19 | 120 | curator-list Ed25519+rollback (`f4ae22d`), Sybil multi-forge (`d52ce89`), Kudos-v2 fairness anti-Matthew (`e194329`). Le pan « gov 19 tabs » a été supprimé au pivot S50-S51 ; le protocole de curation survit. |
| **docs-process** | 29 | 114 | AGENT_SYSTEM.md (`92a4d19`), PUBLIC_FEED_SPEC (`ff5e349`), SHARD_PROTOCOL_SPEC drift-gated (`744f84a`), hub Diataxis (`91be0e4`). Process/spec, « feature produit » faible. |
| **rrv-search** | 10 | 81 | FTS5 @protocole bm25 (`f46bc66`), reindex à chaud (`47c9ff7`), provenance UNINDEXED M17 (`0f86e5a`), barre shell (`9472085`). @dev/@web index-code = DEFER. |
| **compute-sharding** | 7 | 53 | **LA PHARE.** Placement Parallax (`81c8f64`), routing DAG+churn (`8ab8f97`), compute cross-machine B-3 LIVE (`1cc28e7`), quorum dedup (`d75ae77`), harness T2 (`0f597cf`). **T2 = RIG-ABSENT, PROVISIONAL, carry P1 S78.** |
| **p2p-resilience** | 4 | 17 | freshness probe (`376bfe2`), retry InsertRemote (`587016f`), feed_join backfill (`73b0f7b`). Résilience gagnée empiriquement (convergence live). |
| **persistence-durability** | 1 | 9 | orphan recovery + RevocationCache M14 + key rotation (`141f3ff`). Quasi-fusionnable avec hosting. |

## Features réelles sous-estimées par les analyses haut-niveau

Vérification graduée N0-N3 reconstituée de la SOTA · output-filter 3-couches (stégano Unicode
invisible) · watermark SynthID E2E · anti-Sybil multi-forge offline (GPG/SSH sans KYC ni token) ·
FROST-ed25519 + DKG air-gapped · défense duress local-only · worker auto-spawn on-demand ·
loopback hardening OS-level · quarantine + capability store + upload-queue jitter · migration
coordinator Python→Rust (−103 000 LOC) + CI Woodpecker self-hosted · Tor transport phase 1 (dormant).

## Discipline transversale (signaux de cohérence)

`DOMAIN_*_V1` + JCS RFC 8785 (18+ domaines, byte-identique cross-langage) · **no-float dans les
canonical bytes** (basis-points u128, de S2 à S77) · named constants no-magic-numbers · **fail-closed
partout** · 0-bump-wire via raw-op (pré-launch) · dual-platform Win+Docker fmt byte-identique · **gate
Codex bloquante** review→commit · **audit gate Phase 0 permanent** (depuis S7) · cap-DoS-avant-crypto ·
invariant cardinal **héberger ≠ publier, seeder ≠ auteur**.

## Réévaluation du verdict

- **Puissance / breadth : 4/5 mérité, et profondeur par domaine SOUS-ESTIMÉE.** 439 entrées sur 17
  domaines, worker-engine 185 sous/20 feat, vérification 256 sous incluant 4 étages N0-N3 + watermark
  + output-filter + Sybil. Ce n'est pas un side-project : une plateforme P2P de compute complète,
  écrite solo, avec audit-gate qui repasse chaque feature plusieurs fois (34 % = sweep `fix`).
- **Novelty vs OSS : 3/5 tenu, mais la masse RENFORCE la nouveauté réelle** sur des points précis
  (N0-N3, watermark E2E, Sybil multi-forge, FROST DKG, fork llama.cpp partial-decode) — non assemblés
  dans un seul OSS. 3/5 reste juste car beaucoup sont des primitives hermétiques non câblées in-vivo
  et la phare est PROVISIONAL.

## Caveats durs (features ≠ valeur livrée)

1. **Phare sharding PROVISIONAL** : T2 = RIG-ABSENT, carry P1 S78 — 53 sous-features de code testé, pas
   une démo qui tourne.
2. **Vérification N0-N3 (256 sous) = primitives non câblées in-vivo** : recompute réel carry S78.
3. **~34 % des entrées = `fix`**, pas des features neuves — la masse `feat` réelle ≈ 324, pas 439.
4. **Pré-launch intégral** : aucun nœud tiers en prod, wire encore éditable, tag v1.0 non poussé.
5. **Pans supprimés** (gov 19 tabs, coordinator Python) : une partie des sous-features ne survit pas
   dans HEAD.
6. **Bugs P0/P1 longtemps dormants** (clé dispatch `task:` jamais vue par un vrai worker avant S71 ;
   quorum cross-machine jamais formé avant fix dedup S76) : existence du code ≠ fonctionnement e2e.

**En une ligne** : la breadth est authentique et disciplinée — celle d'un système **construit-et-durci**,
pas encore **déployé-et-éprouvé**.
