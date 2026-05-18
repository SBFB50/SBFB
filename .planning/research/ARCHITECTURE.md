# Architecture Patterns

**Domain:** P2P app factory, broker sandbox, domain-specific app generation
**Researched:** 2026-05-18

## Recommended Architecture

### Overview

```
User
  |
  v
Shell React (/factory page)          CLI (sbfb create)
  |                                     |
  | HTTP /api/v1/factory/*              | Direct Rust calls
  |                                     |
  v                                     v
Factory Broker (nexus-shell-daemon-core)
  |
  +-- Template Engine
  |     - Parse template.json
  |     - Substitute variables
  |     - Copy bridge SDK
  |     - Generate SBFB.json v2
  |     - Write factory.template.lock
  |     - Write factory.provenance.json
  |
  +-- Diff Generator
  |     - Compare workspace vs proposed changes
  |     - JSON structured diff (not unified text)
  |     - Require user confirmation before apply
  |
  +-- Preview Manager
  |     - Zip workspace
  |     - Serve via blob-serve (existing)
  |     - Same sandbox as production deploy
  |
  +-- Publish Gate
  |     - Checklist: index.html, SBFB.json v2, bridge methods exist
  |     - Secret scan (regex)
  |     - Build check (if build_command in SBFB.json)
  |     - Provenance generation
  |
  +-- Audit Log
        - JSONL file (factory.audit.jsonl)
        - Every action: timestamp + type + user_confirmed + hashes
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| Factory UI (React) | User interaction, diff display, approve/reject | Broker via HTTP |
| Factory Broker | Authorization, execution, path validation | Template Engine, Diff, Preview, Publish |
| Template Engine | File generation from templates | Filesystem (bounded workspace) |
| Diff Generator | Compute changes between states | Workspace filesystem |
| Preview Manager | Serve preview app | Blob-serve cache (existing) |
| Publish Gate | Validate before publish | Deploy pipeline (existing deploy.rs) |
| Audit Log | Tracability | JSONL file |
| CLI (sbfb create) | Non-interactive scaffolding | Template Engine directly |

### Data Flow

**Scaffolding (S73):**
```
User -> CLI sbfb create -> Template Engine -> Filesystem -> Git init
```

**Brokered creation (S74):**
```
User -> Shell /factory -> HTTP POST /api/v1/factory/create
  -> Broker validates path + template
  -> Template Engine generates files in memory
  -> Diff Generator computes diff vs empty workspace
  -> HTTP response: JSON diff
User reviews diff -> Shell /factory -> HTTP POST /api/v1/factory/apply
  -> Broker writes files to workspace
  -> Audit log entry
  -> HTTP response: success
```

**Preview (S74):**
```
User -> Shell /factory -> HTTP POST /api/v1/factory/preview
  -> Broker zips workspace
  -> blob-serve loads zip into cache
  -> HTTP response: { preview_url: "/blob-serve/{hash}/index.html" }
User sees app in iframe sandbox (same CSP as production)
```

**Publish (S74):**
```
User -> Shell /factory -> HTTP POST /api/v1/factory/publish-check
  -> Publish Gate runs checklist
  -> HTTP response: { checks: [{name, status, details}] }
If all pass:
User -> Shell /factory -> HTTP POST /api/v1/deploy-from-repo (existing)
  -> Standard deploy pipeline
  -> Provenance generated
  -> Feed entry ReleasePublished
```

## Patterns to Follow

### Pattern 1: Flatpak Portal Broker
**What:** Every privileged operation (filesystem write, git, shell command)
passes through the broker which mediates access and requires user confirmation.
**When:** Any Factory action that modifies the workspace or interacts with
external systems.
**Example:**
```rust
pub struct FactoryAction {
    pub kind: ActionKind,
    pub workspace_path: PathBuf,
    pub details: serde_json::Value,
    pub user_confirmed: bool,
}

pub enum ActionKind {
    TemplateGenerate,
    FileWrite,
    GitInit,
    GitCommit,
    BuildRun,
    PreviewServe,
    PublishCheck,
}

impl FactoryBroker {
    pub fn execute(&self, action: FactoryAction) -> Result<ActionResult> {
        self.validate_workspace_path(&action.workspace_path)?;
        if !action.user_confirmed {
            return Err(FactoryError::ConfirmationRequired);
        }
        // Execute + audit log
        self.audit_log.append(&action)?;
        Ok(result)
    }
}
```

### Pattern 2: SBFB.json v2 Schema Versioning
**What:** Schema version field allows coexistence of old and new manifests.
**When:** Any manifest parsing in deploy.rs or template generation.
**Example:**
```rust
pub fn parse_sbfb_manifest(json: &str) -> Result<SbfbManifest> {
    let raw: serde_json::Value = serde_json::from_str(json)?;
    let schema_version = raw.get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    match schema_version {
        1 => parse_v1(raw),  // node_id, name, version only
        2 => parse_v2(raw),  // full manifest with bridge, tech, requirements
        _ => Err(ManifestError::UnsupportedSchemaVersion(schema_version)),
    }
}
```

### Pattern 3: Preview = Production Path
**What:** Preview uses exactly the same pipeline as production deploy
(zip -> blob-serve -> iframe sandbox), ensuring WYSIWYG deploy.
**When:** Any preview action in the Factory.
**Example:**
```rust
impl PreviewManager {
    pub fn serve_preview(&self, workspace: &Path) -> Result<String> {
        let zip_bytes = zip_directory(workspace)?;
        let hash = blake3::hash(&zip_bytes);
        self.blob_cache.load(&hex::encode(hash), &zip_bytes, MAX_DECOMPRESSED)?;
        Ok(format!("/blob-serve/{}/index.html", hex::encode(hash)))
    }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Factory as Protocol Business Logic
**What:** Adding factory-specific methods to the bridge protocol or coordinator.
**Why bad:** Protocol must remain domain-neutral. Factory in the protocol = tight coupling.
**Instead:** Factory uses daemon HTTP routes, not bridge methods.

### Anti-Pattern 2: AI Generation Without Human Gate
**What:** Factory generates code and publishes automatically without user review.
**Why bad:** Destroys trust model. Only 33% of devs trust AI code (2025 research).
**Instead:** Every modification shows a diff, requires explicit approval.

### Anti-Pattern 3: Iframe-Based Factory
**What:** Building Factory as a sandboxed iframe app via bridge.
**Why bad:** Factory needs FS, git, shell — impossible from sandboxed iframe.
**Instead:** Factory UI = React page in shell, broker = daemon module.

### Anti-Pattern 4: WebContainers for Preview
**What:** Embedding StackBlitz WebContainers for Node.js preview.
**Why bad:** 20+ MB WASM, complex setup. blob-serve already does this.
**Instead:** blob-serve serves zipped workspace as preview.

## Scalability Considerations

| Concern | At 5 templates | At 50 templates | At 500+ templates |
|---------|----------------|-----------------|-------------------|
| Template engine | Rust native sufficient | Consider giget for Git-based | Copier with registry |
| Storage | Embedded in binary | Templates in Git repo | Template registry |
| Diff perf | Instant (<100ms) | Instant (<100ms) | Cache diffs, lazy compute |
| Preview | blob-serve LRU (32) | Increase LRU capacity | Eviction policy review |

## Sources

- Flatpak portal architecture: https://docs.flatpak.org/en/latest/sandbox-permissions.html
- Factory.ai trust levels: https://sidbharath.com/blog/factory-ai-guide/
- Internal: `.planning/research/sbfb_project_factory_rrv_oss_research.md`
- Internal: `crates/nexus-shell-daemon/src/deploy.rs`
- Internal: `crates/nexus-shell-daemon-core/src/blob_serve.rs`
