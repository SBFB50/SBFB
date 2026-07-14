## Rapport par livrable

1. **PARTIEL — Note Track H**

   La note couvre bien l’inexécutabilité, la 2e occurrence méta, la ré-dérivation, H-1/H-2/H-3 et les homonymes ([sprint81_audit_findings.md:183](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_audit_findings.md:183)). H-2 et H-3 sont défendables. En revanche, « S81-H-1 SOLDÉ » aux lignes 195-199 est faux tant que les drifts du DFD décrits au livrable 2 subsistent.

2. **PARTIEL — THREAT_MODEL, famille coordinator**

   Consent, PA v5/wire v1, `governor` 0.10 et ancres symboliques sont exacts ([THREAT_MODEL.md:227](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:227), [publish.rs:24](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/publish.rs:24), [Cargo.lock:3030](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:3030)).

   Mais le DFD conserve trois erreurs :

   - `Shell daemon :7777`, alors que le défaut est `api_port: 0`, port éphémère écrit dans `running.json` ([THREAT_MODEL.md:112](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:112), [config.rs:223](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/config.rs:223)).
   - Blob-serve reste présenté comme origin/listener séparé `:7000`, alors que `/blob-serve` est fusionné dans le même router/listener daemon ([THREAT_MODEL.md:19](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:19), [THREAT_MODEL.md:102](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:102), [http.rs:255](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:255), [http.rs:531](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:531)).
   - `deploy` est attribué à `nexus-coordinator-rs`, dont les modules n’en contiennent pas ; le handler vit dans le daemon ([THREAT_MODEL.md:18](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:18), [nexus-coordinator-rs/lib.rs:11](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/lib.rs:11), [deploy.rs:65](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/deploy.rs:65)).

3. **CONFIRMÉ — G-D5-1 / VALIDATED_BLUEPRINT**

   Les trois `0.97` ont disparu. Les valeurs correspondent au lock : iroh 1.0.1, iroh-gossip 0.101.0, blake3 1.8.5 et ed25519-dalek direct 2.2.0 ([VALIDATED_BLUEPRINT.md:71](C:/Users/FlowUP/Documents/Code/nexus/docs/security/VALIDATED_BLUEPRINT.md:71), [VALIDATED_BLUEPRINT.md:156](C:/Users/FlowUP/Documents/Code/nexus/docs/security/VALIDATED_BLUEPRINT.md:156), [VALIDATED_BLUEPRINT.md:218](C:/Users/FlowUP/Documents/Code/nexus/docs/security/VALIDATED_BLUEPRINT.md:218), [Cargo.lock:3920](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:3920), [Cargo.lock:4096](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:4096), [Cargo.lock:717](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:717), [Cargo.lock:2102](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:2102)). URL 2.2 et remplacement cargo-deny présents aux lignes 668 et 378. L’overclaim « bloque PR » est traité au livrable 6.

4. **PARTIEL — K-R-7**

   La SPEC est maintenant exacte : branche présente dans l’accept-loop partagé mais déclenchée seulement pour un stage réel, echo byte-identique ([SHARD_PROTOCOL_SPEC.md:270](C:/Users/FlowUP/Documents/Code/nexus/docs/protocol/SHARD_PROTOCOL_SPEC.md:270), [shard.rs:339](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:339)). Le sommaire THREAT et la row `/generate` sont également bornés.

   En revanche, THREAT §16 affirme encore que « l’interception […] vaut pour toute session » ([THREAT_MODEL.md:1604](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1604)), contrairement au `if is_real_stage` ligne 345. La garantie réelle n’est pas affaiblie par le code, mais la documentation reste contradictoire.

5. **CONFIRMÉ — K-2**

   Les quatre surfaces portent la provenance `operator-corroborated`/logs non committés : [sprint81_verification.md:162](C:/Users/FlowUP/Documents/Code/nexus/.planning/archive/v2.1/sprint81_verification.md:162), [CLAUDE.md:190](C:/Users/FlowUP/Documents/Code/nexus/CLAUDE.md:190), [SPRINT_LOG.md:19](C:/Users/FlowUP/Documents/Code/nexus/docs/claude/SPRINT_LOG.md:19), [sprint82_audit_plan.md:69](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_audit_plan.md:69). Le commentaire code est déjà correct et n’a pas été édité ([shard.rs:1050](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:1050)). Aucune prose inventée.

6. **PARTIEL — Claim cargo-audit / subsomption**

   La décision centrale est exacte : S18 D3 retient cargo-deny seul ([sprint18_kickoff.md:238](C:/Users/FlowUP/Documents/Code/nexus/.planning/archive/v1.2/sprint18_kickoff.md:238)); le workflow exécute `cargo-deny check`, avec RustSec et `yanked=deny` ([supply-chain.yml:48](C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/supply-chain.yml:48), [deny.toml:41](C:/Users/FlowUP/Documents/Code/nexus/deny.toml:41)). HARDENING `:181` reste intact, seule l’annotation `:889` change. Aucun ajout « supply-chain green » et `security_posture.md` n’est pas modifié.

   Deux problèmes demeurent :

   - La menace « npm postinstall malveillant » passe sans justification de résiduel H à L, alors qu’audit-ci ne couvre que les advisories connues `critical` ([THREAT_MODEL.md:250](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:250), [audit-ci.json:3](C:/Users/FlowUP/Documents/Code/nexus/web/audit-ci.json:3)). R2 reconnaît lui-même la fenêtre zero-day aux lignes 379-380.
   - `pip-audit` est déclaré livré/bloquant, mais son workflow cible trois packages supprimés et aucun `pyproject.toml` n’est tracké ([supply-chain.yml:77](C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/supply-chain.yml:77)). Le `uv export --package nexus-sdk …` échoue avant d’atteindre pip-audit : exit 2, « No pyproject.toml found ».

7. **GAP — catalog_len=0 seeder**

   Le comportement observé est exact : `own_entries` filtre les entrées directes par `node_id`, donc un pur seeder reste à zéro ([browse.rs:605](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/browse.rs:605), [http.rs:1321](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1321)). Le risque de découvrabilité, la répétition des reports et PO-8 sont consignés.

   Le rationale de provenance et la condition de réouverture sont toutefois faux. Les nouveaux blocs disent qu’un catalogue signé n’atteste que ce que le signataire « publie » et qu’y inclure un seed ferait de lui un rééditeur ([THREAT_MODEL.md:1037](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1037), [PATTERNS.md:3438](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3438)). Le contrat Rust canonique dit l’inverse : catalogue des apps que le nœud « hosts (or seeds) » et signature attestant « I claim to host these hashes », explicitement pas l’auteur ([node_directory.rs:4](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/node_directory.rs:4), [node_directory.rs:44](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/node_directory.rs:44)). Un champ `seeded` distinct exigerait un wire change ; l’inclusion non étiquetée de hashes hébergés est déjà compatible avec le wire actuel.

8. **CONFIRMÉ — Flip présent-vrai PATTERNS**

   Le carry est désormais déclaré fermé en S82 Phase I avec la requalification et la désambiguïsation nécessaires ([PATTERNS.md:849](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:849)). Aucune promesse future ajoutée.

9. **CONFIRMÉ — LOOPBACK**

   `last_validated` est au 2026-07-14 et conserve les trois validations précédentes ([LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3)). La route quarantine est exacte ([LOOPBACK_ENDPOINTS_TRUST_TIERS.md:81](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:81), [http.rs:522](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:522)). Les ancres `auth_required`, `CorsLayer::new`, `SENSITIVE_ACTIONS`, `is_sensitive→sse_gate`, `target.run` et port 3001 existent aux symboles cités.

## Écarts

- **P1 — Baisse supply-chain H→L non fondée.** Un scanner d’advisories critical-only ne borne pas un postinstall volontairement malveillant sans advisory. Fix minimal : restaurer le résiduel H de §5.8 et `M à H selon dépendance` pour R2, ou ajouter une cotation formelle démontrant la baisse.

- **P1 — Clôture catalog_len fondée sur une fausse sémantique de signature.** Fix minimal : conserver PO-8 et le constat `own_entries`, mais dire que le wire atteste l’hébergement, pas l’auteur ; distinguer réouverture code-only pour inclusion non étiquetée et wire bump uniquement pour un champ `seeded` distinct.

- **P2 — DFD courant encore faux.** Fix minimal : remplacer `:7777` par listener loopback éphémère/renseigné via `running.json`, représenter `/blob-serve` comme route publique du même router avec origin iframe opaque, et rattacher `/deploy` au daemon plutôt qu’à la bibliothèque coordinator.

- **P2 — THREAT §16 contredit `is_real_stage`.** Fix minimal : reprendre la formulation de SPEC §5.2 : branche portée par l’accept-loop partagé, interception exécutée uniquement si `is_real_stage`.

- **P2 — `pip-audit` déclaré fonctionnel alors qu’il ne peut pas démarrer.** Fix minimal docs-only : qualifier cette jambe S18 comme historique/inopérante après purge Python et limiter la posture courante à cargo-deny + audit-ci ; router séparément la réparation/suppression du job CI.

- **P2 — « bloque les PR » non prouvé au disque.** [POST_CHATONS.md:411](C:/Users/FlowUP/Documents/Code/nexus/docs/community/POST_CHATONS.md:411) et [VALIDATED_BLUEPRINT.md:378](C:/Users/FlowUP/Documents/Code/nexus/docs/security/VALIDATED_BLUEPRINT.md:378) dépassent la preuve disponible : le repo prouve un job déclenché sur `pull_request`, pas une required check de branch protection. Fix minimal : « câblé sur les PR et fail non-zéro par configuration ».

- **P3 — Périmètre/review live incohérent.** L’état réel est 8 fichiers modifiés + **2** untracked (`preflight` et `phase_i_review`), alors que la review annonce 8+1 ([sprint82_phase_i_review.md:84](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_i_review.md:84)). Elle affirme aussi qu’aucune row S16 n’a été réécrite, alors que la row PA v5 l’a été conformément au contrat. Fix minimal : réconcilier la review et exclure explicitement son propre artefact du payload audité.

Contrôles exécutés : `git diff --check` propre ; `grep 0.97` = 0 ; `check-sharding-docs.sh`, `check-frontier-contracts.sh` et `check-factory-docs.sh` propres ; aucun diff sous `crates/`, `web/`, `Cargo.toml`, `Cargo.lock` ou schemas ; aucune constante wire touchée. Audit strictement read-only.

**Verdict global : GAPS.**



---

# Round 2 (brut, post-corrections — output codex exec -o verbatim)

## Verdict global : GAPS

Les corrections principales sont justes, mais quatre écarts vérifiables empêchent `CLEAN`.

| Écart | Verdict | Preuve |
|---|---|---|
| P1-1 supply-chain H→L | **CORRIGÉ** | Résiduel `H` restauré avec limite des advisories connues dans [THREAT_MODEL.md:252](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:252). R2 porte bien `M à H` et explicite le zero-day dans [THREAT_MODEL.md:371](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:371). `audit-ci` reste critical-only dans [audit-ci.json:3](C:/Users/FlowUP/Documents/Code/nexus/web/audit-ci.json:3). |
| P1-2 `catalog_len` / signature | **CORRIGÉ** | Le contrat atteste bien l’hébergement, pas l’auteur : [node_directory.rs:44](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/node_directory.rs:44). `own_entries` est une politique daemon filtrant `node_id` dans [browse.rs:605](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/browse.rs:605). Les réouvertures code-only/wire/SearchManifest sont correctement séparées dans [THREAT_MODEL.md:1061](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1061) et [PATTERNS.md:3456](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3456). |
| P2 DFD | **ENCORE-OUVERT** | Port éphémère, `running.json`, même listener, deploy daemon et rôles coordinator sont désormais exacts : [config.rs:223](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/config.rs:223), [runtime.rs:403](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:403), [http.rs:255](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:255), [deploy.rs:65](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/deploy.rs:65), [lib.rs:4](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/lib.rs:4). Mais [THREAT_MODEL.md:19](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:19) pointe toujours vers le fichier inexistant `nexus-shell-daemon/src/blob_serve.rs`. Le module réel est [nexus-shell-daemon-core/src/blob_serve.rs:5](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/blob_serve.rs:5), avec le handler dans [http.rs:3338](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:3338). |
| P2 THREAT §16 / `is_real_stage` | **CORRIGÉ** | La doc distingue maintenant accept-loop partagé et interception limitée au stage réel dans [THREAT_MODEL.md:1615](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1615), conformément au gate effectif [shard.rs:339](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard.rs:339). |
| P2 `pip-audit` inopérant | **ENCORE-OUVERT** | La qualification documentaire est corrigée dans [THREAT_MODEL.md:343](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:343) et [VALIDATED_BLUEPRINT.md:378](C:/Users/FlowUP/Documents/Code/nexus/docs/security/VALIDATED_BLUEPRINT.md:378). Le workflow cible toujours trois packages Python purgés dans [supply-chain.yml:79](C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/supply-chain.yml:79). Surtout, aucun ledger actif ne route sa réparation/suppression : l’unique entrée voisine reste `CARGO-AUDIT-CLAIM-HONESTY`, consacrée à cargo-audit/cargo-deny, dans [sprint82_phase_e_ledger_reconciliation.md:127](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_e_ledger_reconciliation.md:127). Seule la review affirme ce routage dans [sprint82_phase_i_review.md:168](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_i_review.md:168). |
| P2 « bloque PR » | **NOUVELLE-ERREUR** | La required-check non prouvée est correctement retirée. Mais [POST_CHATONS.md:413](C:/Users/FlowUP/Documents/Code/nexus/docs/community/POST_CHATONS.md:413) affirme désormais « chaque PR », alors que le workflow ne couvre que les PR ciblant `master` ou `main` : [supply-chain.yml:25](C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/supply-chain.yml:25). La formulation exacte serait « chaque PR vers `main`/`master` ». |
| P3 review | **NOUVELLE-ERREUR** | Le périmètre 8 modifiés + 3 artefacts untracked, exclus du payload, est correctement réconcilié dans [sprint82_phase_i_review.md:84](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_i_review.md:84). Mais la review affirme déjà que le round 2 est concaténé dans `codex_review` à [sprint82_phase_i_review.md:194](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_i_review.md:194), alors que [sprint82_phase_i_codex_review.md:72](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_i_codex_review.md:72) se termine toujours sur le verdict `GAPS` du round 1. |

Non-régressions : **G-D5-1, K-2, flip PATTERNS et LOOPBACK restent intacts**. Les versions correspondent toujours au lock, notamment [Cargo.lock:3920](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:3920), [Cargo.lock:4096](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:4096), [Cargo.lock:717](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:717) et [Cargo.lock:2102](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:2102). Le flip reste présent dans [PATTERNS.md:849](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:849), et les ancres LOOPBACK correspondent toujours aux symboles réels.

Contrôles rejoués : `git diff --check`, absence de diff code/lock, absence de `0.97`, `check-sharding-docs`, `check-frontier-contracts` et `check-factory-docs` — tous propres.

