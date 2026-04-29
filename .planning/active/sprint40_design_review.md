# Sprint 40 — Design Review Board (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Date** : 2026-04-29.

## Scoring

| Decision | Score | Finding |
|---|---|---|
| D1 | ✅ | `strsim 0.11.1` stable, API `normalized_levenshtein()` confirmee. Guardrail trait Pass/Tripwire uniquement (pas Mutation). Port Tripwire coherent S39. |
| D2 | ⚠️ | SHA-256 et BLAKE3 dans workspace, 0 collision. Justification "parite wire format Python" non documentee dans kickoff — confirmer redundancy.py utilise SHA-256. |
| D3 | ⚠️ | 5 items existent aux locations citees. Substring min_len : kickoff dit "skip < 8" mais code a default 40 — clarifier. Chain Arc singleton pas encore impl — Phase A. |
| D4 | ✅ | P3-grammar et P3-watermark a 3/3+ confirmes audit S39. Map Phase C correct. |
| D5 | ✅ | 12 scope cuts justifies, 0 conflit roadmap. |

## Details

### D2 ⚠️ — SHA-256 parite wire

Le kickoff cite "parite wire format Python" pour SHA-256 dans
redundancy.rs mais ne documente pas la source. Confirmer que
`redundancy.py` utilise bien `hashlib.sha256` (pas md5/sha1).

### D3 ⚠️ — Substring + chain

(b) `output_filter.rs:90` a `DEFAULT_SUBSTRING_MIN_LEN=40`.
Le kickoff D3 dit "skip < 8 chars" — incoherent avec le code.
Clarifier si le fix est le min_len (deja fait) ou l'early exit
(pas encore fait).

(c) `http.rs:1300,1383` appelle `default_input_chain()` /
`default_output_chain()` par requete. Chain a stocker dans
DaemonHttpState.

## Verdict

EXECUTE — G1 rigor satisfait (3 ✅ + 2 ⚠️ sur 5). Angles
morts non-bloquants, addressables Phase A/C.
