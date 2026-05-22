# RRV sprint intake - S70/S69.5 decision packet

**Date:** 2026-05-22
**Status:** intake packet for next kickoff, not an active sprint plan
**Purpose:** make the RRV research corpus consumable by the Nexus sprint process
**Owner:** next sprint kickoff / S69 Phase E wrap-up

## 1. Why this file exists

The sprint process does not infer a clean plan from many research documents by
itself. It consumes a small set of explicit planning artifacts:

- `.planning/active/sprint{N}_kickoff.md`;
- `.planning/active/sprint{N}_plan.md`;
- `.planning/active/sprint{N}_design_review.md`;
- phase preflights and reviews;
- `sprint{N}_verification.md`;
- `sprint{N+1}_audit_plan.md`.

Research files are inputs, not executable plans. To make RRV serious and
logical, the next sprint must convert the research into:

- D1..D5 kickoff decisions;
- explicit scope cuts;
- phase order;
- acceptance tests;
- audit tracks;
- source list with status `canon/live/candidate/fossil`.

This file is the bridge between the RRV research notes and the sprint process.

## 2. Current process fact pattern

Repo process facts:

- `.planning/README.md` says `.planning/research/` is cross-sprint input, while
  `.planning/active/` is the only sprint execution surface.
- `docs/agent/PROCESS.md` says sprint start opens or updates kickoff and plan,
  and requires `Research consulte` before freezing D-decisions.
- `prompts/agent/phase-auditor.md` has a `SPRINT_START` mode that produces
  kickoff, design review and plan when a new sprint begins.
- `prompts/agent/preflight.md` says the sprint plan is a starting point, not
  blind authority; if research proves drift, the phase writes a pivot proposal.
- Current S69 plan Phase E is the correct official place to write
  `sprint69_verification.md` and `sprint70_audit_plan.md`.

Implication: do not expect RRV research to affect execution unless S69 Phase E
or S70 kickoff explicitly imports it.

## 3. Canonical RRV inputs

Use these first:

| File | Status | Use |
| --- | --- | --- |
| `.planning/research/rrv_llm_runtime_and_app_boundary.md` | live intake | LLM local/central, `/rrv` vs app installee, provider/privacy boundary |
| `.planning/research/rrv_app_protocol_best_features.md` | live intake | product shape, scopes, features, access model |
| `.planning/research/SYNTHESIS_factory_rrv_protocol.md` | canon but partly superseded | architecture roles, Factory/RRV/Babel boundary, tests |
| `.planning/roadmap_v4_neutral_protocol_factory_rrv.md` | canon roadmap | current official sequencing and Gate 1 criteria |
| `.planning/research/rrv_scoped_search_compute_groups.md` | live/candidate | scope UX and compute groups |
| `.planning/research/s70_s72_rrv_research.md` | candidate/fossil mixed | older RRV implementation research; verify before importing |
| `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md` | vision/fossil mixed | useful vision, not direct plan |
| `docs/apps/CHAT_IA_RESEAU.md` | vision/fossil mixed | source-access vision; rewrite proof-first before using |
| `docs/apps/GENERATION_COMPOSEE.md` | post-RRV vision | use after `@dev` source index and Factory patterns exist |

## 4. Product correction to freeze

Factory is not the source of verification.

An app can be:

- created by Factory and verified;
- coded manually and verified;
- imported from OSS and later packaged as a verified SBFB app;
- indexed as OSS source-only without becoming a verified SBFB app.

Verification comes from the evidence pack:

```text
repo/source pack
+ pinned commit
+ SBFB.json
+ build/deploy provenance
+ archive hash
+ provenance_hash
+ feed signature
+ ProofCard
= SBFB verifiable app
```

Factory is only one producer of such evidence. RRV must display this clearly.

## 5. Strategic option now

There are two coherent routes. The next kickoff must choose one.

### Option A - strict roadmap

Keep S69 and S70 as currently framed:

1. finish S69 Factory/Babel/Gate 1;
2. S70 consolidates Gate 1, refactors seams, fixes debt;
3. RRV `@dev` and OSS seed start after consolidation.

Pros:

- least disruption;
- aligns with roadmap v4 D17;
- protects current Factory dogfood.

Cons:

- RRV remains less concrete until after Factory/Babel;
- `@dev` learning starts later.

### Option B - RRV-first correction

Use S69 Phase E to route a pivot, then make the next sprint:

```text
RRV Core + OSS Seed Corpus + source-only contract
```

Factory continues after RRV has a real `@dev` corpus and product shape.

Pros:

- RRV becomes serious before Factory UI grows;
- 10 OSS projects give `@dev` real material;
- Factory later consumes proven patterns instead of guessing.

Cons:

- changes roadmap v4 D17;
- delays some Factory hardening;
- requires explicit trust labels to avoid calling OSS repos verified apps.

## 6. Recommended route

Recommended: Option B, but bounded.

Do not build "RRV total" first. Build:

```text
RRV Core
+ OSS Seed Corpus
+ @dev source-only index
+ proof-first UI/spec
+ process-aware index
```

Then resume Factory/Babel with better inputs.

This is a product correction, not a rejection of Factory. Factory becomes more
valuable once RRV can find, cite, compare and score reusable patterns.

## 7. Draft kickoff D-decisions

These are not active decisions until copied into a kickoff/design review.

### D1 - RRV is the next product surface to make serious

RRV is the app that lets users navigate, question, prove and act on the
protocol. The next sprint should create a usable RRV Core, not only more
research.

RRV is layered, not binary: the daemon owns RRV Core facts/proofs/privacy, the
shell exposes a first-class `/rrv` bootstrap route, and an installable
`sbfb-search` app dogfoods the same public contracts.

### D2 - OSS seed corpus is source-only, not SBFB verified

Ingest 10 curated OSS projects as `External OSS source index`. They may later
be packaged as SBFB apps, but indexing alone does not make them verified.

### D3 - Verification is evidence-based, not Factory-based

Apps created outside Factory can become verified if they publish the same SBFB
evidence pack. RRV must make origin and evidence separate fields.

### D4 - `@dev` starts local/source-only

`@dev` indexes files, lines, symbols, manifests, schemas, tests, process docs,
licence and risk metadata. It does not read private app data.

`@dev` defaults to `LocalOnly`. A central LLM can only receive a redacted
EvidenceBundle after explicit egress consent.

### D5 - Process files are part of the RRV corpus

RRV must index and answer on `.planning/`, `docs/agent/`, `prompts/agent/`,
hooks and verification artifacts. This is how Factory-generated projects stay
auditable.

LLM providers are answer composers only. They cannot create proof labels,
upgrade OSS source indexes into verified apps, or replace ProofCard/provenance.

## 8. Draft scope cuts

Must not be included in the first RRV sprint:

- no network SearchManifest;
- no private group compute;
- no real-time distributed inference;
- no automatic remote LLM egress;
- no claim that OSS seed repos are SBFB verified apps;
- no private DB rows;
- no broad web crawling by default;
- no Factory UI rewrite;
- no Babel translation engine.

## 9. Draft phase shape

This is a kickoff input, not an executable plan yet.

### Phase A - RRV product/spec canon

Write or update:

- `docs/product/RRV_PRODUCT.md`;
- `docs/protocol/RRV_SEARCH.md`;
- `docs/security/THREAT_MODEL.md` RRV section;
- provider/egress policy;
- `EvidenceBundle` / `PromptBundle` / `LlmProvider` contract;
- trust label matrix;
- source-only evidence contract.

Acceptance:

- scopes defined;
- labels defined;
- answer format defined;
- app origin vs verification separated.
- `/rrv` shell surface vs `sbfb-search` app boundary defined.
- LLM declared as composer, not proof authority.

### Phase B - OSS seed manifest and ingestion contract

Write:

- `configs/rrv_oss_seed.sample.json`;
- seed project selection criteria;
- pinned commit/licence/source metadata schema;
- local cache policy;
- no-private-data policy.

Acceptance:

- 10 candidate slots with required fields;
- label always `External OSS source index`;
- invalid licence/commit missing rejected by validation.

### Phase C - `@dev LocalOnly` index MVP

Implement or prototype:

- file walker for approved source packs;
- line/hash citations;
- basic symbol/capability extraction;
- manifest/process file detection;
- no embeddings required.

Acceptance:

- query returns file, line, commit/hash;
- secrets ignored or flagged;
- private paths excluded.

### Phase D - RRV Core UI/API integration

Build minimal product surface:

- shell route `/rrv`;
- scope chips;
- question/search input;
- result list with labels;
- proof/source panes;
- actions: open, cite, verify, inspect source;
- mode selector: search-only first, local/central provider later.

Acceptance:

- `@protocole` uses existing `/api/daemon/search`;
- `@dev` uses local/source-only index;
- result trust labels never merge.
- `/rrv` works without any LLM provider.

### Phase D2 - installable `sbfb-search` dogfood

Build or update the installable app surface that consumes the same daemon/bridge
contracts as `/rrv`.

Acceptance:

- no hidden privileged API only available to shell UI;
- equivalent query returns equivalent evidence sources;
- manifest capabilities are explicit;
- sandbox assumptions remain true.

### Phase D3 - LLM provider router minimal

Implement or spec the runtime modes:

- `none/search_only`;
- `local`;
- `central`;
- `agent_process`;
- `network_task` later/explicit only.

Acceptance:

- local provider performs no external egress;
- central provider blocked without consent;
- remote prompts receive redacted EvidenceBundle only;
- provider/model/evidence hashes logged;
- LLM output without citations is degraded to hypothesis/non verified.

### Phase E - verification and next audit plan

Write:

- `sprint{N}_verification.md`;
- `sprint{N+1}_audit_plan.md`;
- carry items for Factory/Babel resume.

Acceptance:

- each RRV feature has a verification row;
- unresolved P2/P3 routed;
- next sprint decision between Factory resume and RRV hardening is explicit.

## 10. Audit tracks for the next sprint

The next audit plan should include:

| Track | Question |
| --- | --- |
| A - Trust labels | Did any OSS source result appear as SBFB verified? |
| B - Privacy | Did `@dev` index private data, secrets or DB rows? |
| C - Citations | Do search answers cite file/line/hash or feed/provenance refs? |
| D - Protocol bridge | Does `@protocole` still use existing daemon/bridge primitives? |
| E - Process | Did kickoff cite the RRV research corpus explicitly? |
| F - Factory boundary | Did Factory remain a client/tool, not verification authority? |
| G - Scope cuts | Were SearchManifest, private compute and web crawl kept out? |
| H - Provider egress | Did any central LLM receive local/private data without explicit consent? |
| I - Shell/app parity | Can `sbfb-search` perform the same proof/search operation as `/rrv` via public contracts? |

## 11. What S69 Phase E should do

If S69 continues as planned, Phase E should add these items to
`sprint70_audit_plan.md`:

1. audit whether RRV research is now coherent enough for a product sprint;
2. decide Option A strict roadmap vs Option B RRV-first correction;
3. require the S70 kickoff to cite this file;
4. require D-decisions to separate Factory origin from verified evidence;
5. require any OSS seed to be source-only with labels;
6. require the kickoff to import `rrv_llm_runtime_and_app_boundary.md`;
7. require a decision on local/default vs central/opt-in provider policy.

This is the minimal change that lets the process "know how to get out" of the
research pile.

## 12. One-line recommendation

Make the next kickoff consume this packet and choose:

```text
S70 = RRV Core + OSS Seed Corpus + @dev source-only
```

unless Gate 1/S69 exposes blocking P0/P1 that force a consolidation sprint
first.
