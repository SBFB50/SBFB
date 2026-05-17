# SBFB cross-domain use cases research

Date: 2026-05-17
Status: research / ideation
Scope: practical use cases outside the first Babel/ecology framing, using existing open-source or open-data projects as reusable reference points.

Related local research:

- `chat_ia_reseau_recherche_reseau_rnd.md` - RRV as verifiable search, not generic web search.
- `rrv_scoped_search_compute_groups.md` - scoped search, compute groups, `@Babel`, `@network`, `@web`, `@dev`.
- `sbfb_project_factory_rrv_oss_research.md` - Project Factory + RRV local-first + OSS reuse.
- `babel_translation_protocol.md` - Babel as P2P multilingual library and translation workflow.
- `iroh_no_internet_babel_anti_censure.md` - offline, local network and out-of-band distribution.

## 0. Verdict

The most interesting future for SBFB is not only a P2P library or a coding
factory. The deeper product direction is a **portable infrastructure for
commons**:

- find useful open-source/open-data bricks;
- verify their provenance;
- translate them;
- adapt them into local apps;
- run them offline or with weak connectivity;
- replicate them without a central platform;
- keep human governance over validation;
- use shared compute only for bounded, verifiable work.

The domains with the strongest fit are not necessarily the most technical.
They are domains where the failure of a central platform causes real social
damage: crisis response, education, health logistics, public records,
repair, local manufacturing, cultural archives, energy resilience, science,
and supply-chain transparency.

This file is speculative product research. It does **not** claim these
features already exist in the repo. It defines possible future directions for
SBFB, RRV and Project Factory.

## 1. Selection criteria

A domain is a strong SBFB candidate when it has at least four of these traits:

1. **Knowledge must survive**: documents, protocols, maps, rules or datasets
   must remain usable if a server, company, state service or internet access
   disappears.
2. **Translation matters**: the value increases when materials can move
   across languages without losing human review.
3. **Provenance matters**: users need to know where a fact, rule, plan,
   dataset, model, app or result came from.
4. **Local adaptation matters**: a community needs to fork or adapt the tool,
   not only consume a remote SaaS.
5. **Connectivity is weak or contested**: LAN, offline sync, USB, local
   hotspots or delayed sync are useful.
6. **Compute can be decomposed**: OCR, translation, embeddings, simulation,
   classification, build/test, document analysis or dataset cleaning can be
   distributed as tasks.
7. **Human validation stays central**: AI helps, but final validation is made
   by domain users, translators, maintainers or local communities.

## 2. What SBFB adds over existing projects

Most existing open-source projects already solve a specific domain problem.
SBFB should not replace them. The unique contribution is a generic layer:

| Layer | Existing tools usually provide | SBFB/RRV/Factory could add |
| --- | --- | --- |
| Application | domain UI and workflow | app packaging, local runtime, iframe sandbox, bridge |
| Data | files, APIs, databases | content-addressed blobs, provenance, signed feeds |
| Search | keyword or platform search | proof-ranked RRV across local, network and web sources |
| Translation | external service or manual files | Babel-style AI draft + human validation lineage |
| Offline | sometimes partial | LAN/USB/out-of-band sync as first-class distribution |
| Compute | local/server jobs | opt-in task pools with consent, caps and provenance |
| Governance | app-specific roles | explicit review, source, license and validation metadata |
| Creation | manual setup | Code Factory templates, tests, manifests, app skeletons |

The product wedge is therefore:

> Take a useful open project, wrap it as a verifiable local-first SBFB app,
> translate/adapt it with humans in the loop, and let RRV discover the best
> bricks, proofs and maintenance actions.

## 3. Reusable open-source and open-data reference projects

These are not dependencies to vendor blindly. They are reference projects,
formats or ecosystems that can inspire app templates and RRV search sources.

| Domain | Reference projects | Why they matter |
| --- | --- | --- |
| Crisis response | Sahana, Ushahidi, Humanitarian OpenStreetMap Team | disaster coordination, local reports, humanitarian maps |
| Health systems | OpenMRS, DHIS2 | low-resource health records and health information systems |
| Offline education | Kolibri / Learning Equality | offline-first educational content distribution |
| Public rules | OpenFisca | rules-as-code for tax/benefit simulations |
| Public procurement | Open Contracting Data Standard | structured public contracting data |
| Investigative documents | DocumentCloud, MuckRock | document analysis, publication, public records |
| Local democracy | Decidim | participatory democracy and civic governance |
| Energy systems | LF Energy, OpenEMS, GridLAB-D, OpenInfraMap | energy data, grid simulation, EMS, infrastructure maps |
| Repair | Open Repair Alliance, Open Know-How | repair data and open documentation for physical objects |
| Open hardware | Open Source Ecology, OSHWA, FarmBot | machine plans, open hardware, local manufacturing |
| Cultural governance | Mukurtu, Local Contexts | indigenous/community archive governance and labels |
| Heritage | OpenHeritage3D, Wikimedia Commons, OpenHistoricalMap | 3D heritage, media archives, historical maps |
| Science | Galaxy Project, OpenAlex | reproducible workflows and scholarly graph |
| Supply chains | Open Supply Hub, Open Food Facts | supply-chain facilities and product facts |

## 4. Practical use cases

### 4.1 Autonomous crisis cell

Existing inspiration: Sahana, Ushahidi, Humanitarian OpenStreetMap Team.

Practical scenario:

- a town, NGO, camp, school or local group keeps maps, shelters, water points,
  missing needs, medical posts, road status and stock requests;
- the system works on a local network first;
- updates sync when any connection is available;
- critical documents can move by USB, QR, hotspot or local relay.

What SBFB adds:

- resilient distribution instead of one central incident server;
- provenance for reports, maps and imported datasets;
- local translation for multi-language response;
- RRV search such as `@crisis show verified water points updated today`;
- Code Factory templates for "crisis board", "needs registry", "map pack",
  "local report intake".

First realistic app:

- read-only offline crisis pack with maps, contacts and verified procedures;
- later: local report intake and sync.

Risk gate:

- do not treat unverified reports as truth;
- label freshness, source and validator;
- protect sensitive location data.

### 4.2 Low-connectivity clinic knowledge and logistics

Existing inspiration: OpenMRS, DHIS2.

Practical scenario:

- a clinic stores protocols, stock levels, vaccination schedules, aggregate
  public-health indicators and translated training materials;
- the system keeps working when connectivity fails;
- human staff remain responsible for medical decisions.

What SBFB adds:

- provenance for protocols and public-health datasets;
- translated medical documentation with human validation;
- offline training packs;
- task-based compute for OCR, translation and document classification.

First realistic app:

- not a medical diagnosis tool;
- start with "clinic knowledge pack": protocols, stock documents, translated
  manuals, versioned source citations.

Risk gate:

- no autonomous medical advice;
- privacy and PII redaction before any network sync;
- strict separation between public knowledge and patient data.

### 4.3 Portable school / offline learning mesh

Existing inspiration: Kolibri / Learning Equality.

Practical scenario:

- a school, family network or community center shares courses, books,
  exercises, videos, quizzes and local translations;
- teachers review and adapt content;
- students use it without permanent internet.

What SBFB adds:

- Babel as translation and validation layer;
- RRV to find the best content by age, language, topic, source and proof;
- Code Factory creates small learning apps from templates;
- compute pool handles OCR, subtitle generation, translation drafts and
  embeddings.

First realistic app:

- "Babel School Pack": public-domain books + exercises + teacher review notes.

Risk gate:

- content moderation and age labels;
- language quality review;
- source/license visibility.

### 4.4 Public rules as executable, verifiable knowledge

Existing inspiration: OpenFisca.

Practical scenario:

- social benefits, taxes, local subsidies, public procedures or eligibility
  rules are encoded as executable calculators;
- citizens and associations can inspect the source rule and date;
- translations explain the rule in plain language.

What SBFB adds:

- every rule links to source text, jurisdiction, date and maintainer;
- RRV can answer: "which calculator implements this rule and with what
  proof?";
- Code Factory can generate a local simulator app from a verified rule pack.

First realistic app:

- "public aid simulator template" for one narrow, non-sensitive rule set.

Risk gate:

- never present an unofficial calculator as legal authority;
- visible source date and confidence label;
- human review required for rule changes.

### 4.5 Anti-corruption public document memory

Existing inspiration: Open Contracting Data Standard, DocumentCloud, MuckRock.

Practical scenario:

- public contracts, tenders, amendments, invoices, permits and reports are
  archived, OCRed, translated and compared;
- journalists or citizens annotate anomalies;
- documents stay available even if an original page disappears.

What SBFB adds:

- content-addressed documents and provenance;
- RRV for cross-document search by supplier, date, authority, amount, clause;
- distributed OCR and translation tasks;
- human review queue for document claims.

First realistic app:

- "public document ledger": import PDFs, hash them, OCR them, cite source URL,
  generate searchable local index.

Risk gate:

- distinguish document facts from allegations;
- retain original files and extraction confidence;
- defamation and safety risks need human moderation.

### 4.6 Local democracy without platform lock-in

Existing inspiration: Decidim.

Practical scenario:

- a city, cooperative, association or citizens' group runs proposals,
  deliberation, budget ideas and decision archives;
- if a SaaS goes away, the public memory remains portable.

What SBFB adds:

- local archives with provenance;
- offline meeting packs and sync;
- multilingual deliberation;
- RRV finds prior decisions, arguments, documents and implementation status.

First realistic app:

- "assembly memory": proposals, minutes, decisions, votes, source documents.

Risk gate:

- identity, vote integrity and coercion are hard;
- start with transparent archives before binding elections.

### 4.7 Neighborhood energy resilience

Existing inspiration: LF Energy, OpenEMS, GridLAB-D, OpenInfraMap.

Practical scenario:

- a neighborhood or cooperative models solar, batteries, EV charging,
  consumption, backup priorities and maintenance plans;
- simulations help plan resilience without sending everything to a cloud.

What SBFB adds:

- local data packs and reproducible simulations;
- RRV finds open models, datasets, hardware specs and previous runs;
- compute pool runs batch simulations or forecasts;
- Code Factory creates dashboards around a specific local energy dataset.

First realistic app:

- read-only energy planning notebook with simulations and provenance.

Risk gate:

- do not control critical infrastructure at first;
- use advisory/simulation mode before operational control;
- privacy of consumption data.

### 4.8 Repair network and object memory

Existing inspiration: Open Repair Alliance, Open Know-How.

Practical scenario:

- repair cafes, workshops and users record product failures, repair steps,
  parts, tools, time, success rate and photos;
- manuals and fixes are translated and shared offline.

What SBFB adds:

- verifiable repair reports by model/version;
- local-first repair manuals;
- RRV search: "show successful repairs for this device model";
- Code Factory generates a repair-log app for a workshop.

First realistic app:

- "repair notebook": object model, issue, fix, parts, photos, outcome,
  license/provenance.

Risk gate:

- electrical/safety warnings;
- distinguish official manual, community fix and dangerous workaround.

### 4.9 Micro-factories and open hardware kits

Existing inspiration: Open Source Ecology, OSHWA, FarmBot, Open Know-How.

Practical scenario:

- an atelier stores plans, BOMs, CAD files, CNC/3D print files, build logs,
  calibration notes and local material substitutions;
- the same object can be built, forked and improved in different regions.

What SBFB adds:

- build provenance: which version of the plan produced which result;
- RRV finds components, machine capabilities, safety notes and compatible
  forks;
- compute tasks can run slicing, simulation, translation, OCR of manuals or
  test generation for firmware;
- Code Factory creates project pages and build-log apps.

First realistic app:

- "open hardware build ledger": plan import, BOM, build photos, result,
  issue tracker, provenance.

Risk gate:

- safety, liability, quality assurance;
- do not imply a build is safe because the files are open.

### 4.10 Cultural archives under local governance

Existing inspiration: Mukurtu, Local Contexts.

Practical scenario:

- communities archive stories, images, language resources, maps, oral history
  and documents;
- some knowledge is public, some is restricted, some requires community
  context before use.

What SBFB adds:

- local control and offline access;
- governance metadata travels with the content;
- Babel helps translate without stripping cultural review;
- RRV can respect access labels and show provenance/context before content.

First realistic app:

- "community archive pack": local catalog, labels, language files, review
  workflow, export/import.

Risk gate:

- not all knowledge should be open;
- avoid extractive "open data" framing;
- access rules and community authority must be first-class.

### 4.11 Heritage and city memory after loss

Existing inspiration: OpenHeritage3D, Wikimedia Commons, OpenHistoricalMap.

Practical scenario:

- scans, photos, old maps, testimonies, architectural plans and cultural
  descriptions are replicated before war, disaster, neglect or censorship
  erases them;
- local groups can reconstruct memory even when official archives fail.

What SBFB adds:

- distributed archive packs;
- provenance and file hashes for media;
- RRV search across maps, time periods, media types and validators;
- compute for photogrammetry, OCR, translation and metadata extraction.

First realistic app:

- "city memory pack": media catalog, source proofs, timeline, offline viewer.

Risk gate:

- sensitive sites may need redaction;
- personal testimonies need consent and takedown rules.

### 4.12 Reproducible science network

Existing inspiration: Galaxy Project, OpenAlex.

Practical scenario:

- workflows, papers, datasets, scripts, models and results are linked;
- a lab or citizen-science group can reproduce a result locally or via a
  compute group;
- RRV finds not only a paper but the runnable workflow and lineage.

What SBFB adds:

- proof-aware scientific search;
- task-based distributed execution for reproducible runs;
- result provenance tied to dataset, code version, parameters and worker;
- Code Factory creates small reproducibility apps.

First realistic app:

- "workflow proof card": import a public workflow, record dataset/code hash,
  run status, result hash, notes.

Risk gate:

- compute results are not truth without validation;
- dataset licensing and privacy;
- deterministic/reproducible containers where possible.

### 4.13 Citizen-readable supply chains and product facts

Existing inspiration: Open Supply Hub, Open Food Facts.

Practical scenario:

- users, journalists, NGOs or cooperatives inspect product ingredients,
  facilities, origins, labels, recalls and sustainability claims;
- product facts remain queryable even if a central API is unavailable.

What SBFB adds:

- local product/facility data packs;
- source-ranked contradiction tracking;
- RRV queries across product, factory, claim, country, document and date;
- translation and offline access for consumers and field investigators.

First realistic app:

- "product proof notebook": barcode/product record, source documents,
  contradictions, photos, local annotations.

Risk gate:

- claims can be politically or commercially sensitive;
- separate verified public data from community allegations.

## 5. Prioritization for SBFB

### 5.1 Best early domains

The best early domains are those with high usefulness and lower safety risk:

1. **Repair network**: concrete, local, not too regulated, strong knowledge
   sharing and translation value.
2. **Offline education**: strong Babel fit, clear public-good narrative,
   offline distribution is directly useful.
3. **Public document memory**: strong RRV/provenance fit, good for showing
   verifiable search.
4. **Open hardware build ledger**: strong Project Factory fit, practical
   provenance, useful for communities.
5. **Cultural archive pack**: powerful if governance is handled carefully.

These can be built as SBFB apps without controlling critical infrastructure or
handling high-risk personal data by default.

### 5.2 Domains to stage later

These are high-impact but need stronger governance/security before productizing:

1. **Health systems**: privacy and harm risks.
2. **Energy systems**: critical infrastructure risk.
3. **Binding democratic decisions**: identity, coercion and vote integrity.
4. **Supply-chain allegations**: safety, defamation and retaliation risk.
5. **Scientific compute at scale**: reproducibility and worker integrity.

Start them as read-only packs, notebooks, simulations or proof cards before
any operational workflow.

## 6. How RRV should interact with these domains

RRV should not be a generic answer engine. In these domains, it should return
proof-bearing objects:

- source documents;
- datasets;
- app templates;
- open-source repos;
- workflows;
- build logs;
- repair reports;
- translation status;
- validators;
- worker task results;
- stale or conflicting claims.

Example queries:

```text
@repair show verified fixes for this device model
@education find French public-domain lessons for age 10-12 with human review
@documents compare this procurement PDF with prior tenders from the same buyer
@energy find open models for neighborhood battery simulation
@hardware show forks of this machine plan with successful build logs
@culture show public items only, with Local Contexts labels visible
@science find runnable workflows linked to this paper
@supply show product facts with source conflict labels
```

The key product rule:

> Ranking can merge sources, but confidence cannot. A web result, a local
> document, a signed SBFB feed item and a reproduced compute result must keep
> different proof labels.

## 7. How Code Factory should interact with these domains

Code Factory should not try to generate giant apps from scratch. It should
start from small domain templates:

| Template | First useful output |
| --- | --- |
| `repair-notebook` | workshop repair logs and manual links |
| `offline-school-pack` | lessons, books, exercises, translations |
| `document-ledger` | imported PDFs, hashes, OCR, citations |
| `open-hardware-ledger` | plan, BOM, build log, photos, result |
| `community-archive-pack` | catalog, access labels, review workflow |
| `workflow-proof-card` | dataset/code/result provenance |
| `crisis-readonly-pack` | maps, contacts, procedures, local viewer |

For each template, the generated repo should include:

- `SBFB.json` app metadata;
- app storage namespace;
- provenance schema;
- import/export format;
- fixtures;
- tests;
- bridge usage limited to generic protocol methods;
- a small RRV local index;
- documentation for offline handoff.

## 8. Shared GPU and compute opportunities

The most credible compute work is batch/decomposable:

- OCR pages and scanned documents;
- translate chunks;
- generate subtitles;
- classify documents;
- extract tables;
- create embeddings;
- run deterministic tests;
- build artifacts;
- run simulations;
- validate checksums and provenance;
- compare datasets;
- generate metadata from images or scans.

Avoid promising WAN model-parallel giant AI early. The practical path is:

1. define a task;
2. define input hashes;
3. define expected output type;
4. run on consenting workers;
5. record worker, resource caps and result hash;
6. optionally ask a quorum or human reviewer;
7. publish provenance.

## 9. Most unique long-term scenarios

These are the cases that become genuinely different if SBFB, RRV, Babel,
Project Factory and compute groups mature together.

### 9.1 A city that can rebuild its own memory

Before a crisis, a city keeps cultural archives, maps, repair knowledge,
school packs, emergency procedures and public documents in local SBFB packs.
After internet loss or institutional failure, the local network still carries
the knowledge.

Why this is unique:

- not one app;
- a mesh of domain packs;
- all searchable through RRV;
- all exportable/importable;
- all provenance-aware.

### 9.2 A repair cafe becomes a local manufacturing node

The same group starts by logging repairs. Then it imports open hardware plans,
tracks parts, translates manuals, builds small tools, and shares results back.

Why this is unique:

- repair knowledge becomes manufacturing knowledge;
- Code Factory creates local apps for each workshop;
- RRV finds plans, fixes, compatible parts and safety notes.

### 9.3 A school becomes a translation and preservation node

Students and teachers translate public-domain books, local history, technical
manuals and emergency guides. Their review work improves Babel and creates a
local knowledge commons.

Why this is unique:

- education is not only consumption;
- it becomes contribution to a durable public corpus;
- humans remain the validators.

### 9.4 A public document archive becomes an anti-censorship memory

Citizens archive public documents, hash them, translate them, annotate them,
and keep them available across nodes.

Why this is unique:

- deletion of the original URL does not erase the record;
- claims are separated from source documents;
- RRV can surface contradictions without pretending to be a judge.

### 9.5 A science workflow becomes a public proof object

A result is no longer only a PDF. It is a paper + code + dataset + workflow +
parameters + run record + result hash + review notes.

Why this is unique:

- RRV finds runnable knowledge;
- compute groups can reproduce bounded tasks;
- provenance becomes product UI, not hidden metadata.

## 10. Recommended near-term research artifact

Before coding one of these apps, create a short domain pack spec:

```text
domain:
  name:
  user:
  offline_value:
  source_types:
  private_data:
  provenance_required:
  human_review_required:
  import_format:
  export_format:
  rrv_objects:
  compute_tasks:
  first_template:
  non_goals:
```

This prevents each app from becoming a custom island and forces a common SBFB
shape across domains.

## 11. Sources

Official or project sources used for this research direction:

- Sahana Foundation: https://sahanafoundation.org/
- Ushahidi documentation: https://docs.ushahidi.com/platform-user-manual/about-ushahidi
- Humanitarian OpenStreetMap Team: https://www.hotosm.org/
- OpenMRS: https://openmrs.org/
- DHIS2: https://dhis2.org/about-2/
- Learning Equality / Kolibri: https://learningequality.org/
- Decidim: https://decidim.org/
- OpenFisca: https://openfisca.org/en/about/
- Open Contracting Data Standard: https://standard.open-contracting.org/
- DocumentCloud: https://www.documentcloud.org/
- MuckRock: https://www.muckrock.com/
- LF Energy: https://lfenergy.org/
- OpenEMS: https://openems.io/
- GridLAB-D: https://www.gridlabd.org/
- OpenInfraMap: https://openinframap.org/
- Open Repair Alliance: https://openrepair.org/open-data/open-standard/
- Open Know-How: https://www.internetofproduction.org/openknowhow
- Open Source Ecology GVCS: https://wiki.opensourceecology.org/wiki/GVCS
- Open Source Hardware Association: https://www.oshwa.org/
- FarmBot: https://farm.bot/pages/open-source
- Mukurtu: https://mukurtu.org/about/
- Local Contexts: https://localcontexts.org/
- OpenHeritage3D: https://openheritage3d.org/about
- Wikimedia Commons: https://commons.wikimedia.org/
- OpenHistoricalMap: https://www.openhistoricalmap.org/
- Galaxy Project: https://galaxyproject.org/
- OpenAlex: https://developers.openalex.org/
- Open Supply Hub: https://info.opensupplyhub.org/
- Open Food Facts: https://world.openfoodfacts.org/

## 12. Bottom line

SBFB becomes most powerful when it is not pitched as one app. The stronger
future is:

```text
Babel + RRV + Project Factory + provenance + offline sync + voluntary compute
= a portable commons infrastructure.
```

The first product should stay narrow. But the architecture should assume that
many future apps are not "web apps". They are local knowledge systems that can
survive disconnection, censorship, institutional failure, language barriers
and platform death.
