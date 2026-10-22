// SPDX-License-Identifier: AGPL-3.0-or-later
//! Chat v1, daemon side (ADR `docs/architecture/ADR_CHAT_V1.md`, plan Phase C).
//!
//! The daemon owns every key: one messaging [`Identity`] per app, sealed on
//! disk under a 32-byte key file (`chat.key`, 0600 — the same at-rest level
//! as `node_key`; the PIN-protected host vault is not wired to any UI yet).
//! Each conversation is one iroh-docs document holding only signed
//! ciphertext envelopes. A per-app loop, woken by document events and by a
//! safety tick, feeds new entries to [`Conversation::ingest`] and writes back
//! whatever the protocol asks for.
//!
//! Plaintext is never stored: message history is rebuilt from the document
//! on demand ([`Conversation::read_message`]) and pushed live over SSE.
//! Conversation metadata (members, titles) sits in the sealed-state files in
//! clear JSON next to vodozemac-encrypted pickles — protected by file
//! permissions only, like `node_key`.
//!
//! Under duress the whole surface answers 503: a decoy identity must not
//! publish or join anything.
//!
//! Offline mailbox (plan Phase E): per app, OFF until its user turns it on
//! (`POST /app/{name}/chat/mailbox`). While on, the pump hands each
//! conversation's READ ticket to every `[mailbox] servers` node (at start-up,
//! then once a day to keep the registration alive) and syncs with those
//! servers right after, then every [`MAILBOX_RESYNC_SECS`] — which is how a
//! member coming back online catches up when no other member is reachable,
//! and how a failed sync is retried. Turning it off withdraws every
//! registration; a document rotation does not (the old document holds the
//! `Moved` entry offline members need), it only stops refreshing it.

use std::collections::{HashMap, HashSet};
use std::path::{Path as FsPath, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_lite::StreamExt;
use iroh::{EndpointAddr, EndpointId};
use nexus_core_rs::docs::{DocsAuthorId, DocsNamespaceId, DocsTicket};
use nexus_core_rs::{BlobsClient, DocHandle, DocsClient, Node};
use nexus_messaging_core::mailbox::{MailboxRequest, MailboxVerdict};
use nexus_messaging_core::{
    ChatEvent, Conversation, Envelope, Identity, Invitation, MemberProfile,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify, broadcast};
use tracing::{debug, info, warn};

use crate::http::DaemonHttpState;

pub const CHAT_KEY_FILE: &str = "chat.key";
const INVITE_TTL_DEFAULT_SECS: u64 = 3 * 24 * 3600;
const INVITE_TTL_MAX_SECS: u64 = 30 * 24 * 3600;
const PUMP_TICK: Duration = Duration::from_secs(5);
const MAX_TITLE_CHARS: usize = 120;
const MAX_NAME_CHARS: usize = 64;
const EVENT_BUFFER: usize = 256;
/// Re-deposit period: far inside the server TTL (21 days by default), so
/// a daemon that runs at least once every few weeks never lapses.
const MAILBOX_REFRESH_SECS: u64 = 24 * 3600;
/// Delay before retrying a deposit no server kept.
const MAILBOX_RETRY_SECS: u64 = 10 * 60;
/// Period of the pull from the servers that keep a conversation: retries a
/// sync that failed (network, load) without waiting for the daily refresh.
const MAILBOX_RESYNC_SECS: u64 = 10 * 60;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Process-wide chat service, one per daemon.
pub struct ChatService {
    node: Arc<Node>,
    root: PathBuf,
    key: [u8; 32],
    enabled: bool,
    mailbox_servers: Vec<EndpointId>,
    apps: Mutex<HashMap<String, Arc<AppChat>>>,
}

impl std::fmt::Debug for ChatService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatService")
            .field("root", &self.root)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

struct AppChat {
    dir: PathBuf,
    state: Mutex<AppState>,
    events: broadcast::Sender<OutEvent>,
    wake: Notify,
    /// Mailbox bookkeeping per conversation id. In memory only: after a
    /// restart every conversation is due again, which also re-syncs it
    /// with the servers.
    mailbox: std::sync::Mutex<HashMap<String, MailboxSlot>>,
}

struct AppState {
    identity: Identity,
    name: String,
    author: DocsAuthorId,
    convs: HashMap<String, Conv>,
    mailbox: bool,
}

#[derive(Debug, Clone, Default)]
struct MailboxSlot {
    /// Namespace the slot is about: a deposit result only lands on the
    /// slot of the document it deposited.
    ns: String,
    due: u64,
    status: MailboxStatus,
    expires_at: u64,
    /// Last sync started with the servers.
    synced_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MailboxAction {
    Deposit,
    Resync,
    Nothing,
}

/// What the pump does for one conversation's slot at `now`: deposit when
/// the slot is about another document or its deposit is due; otherwise,
/// for a kept deposit, pull from the servers once the last sync is stale.
fn mailbox_action(slot: &MailboxSlot, ns: &str, now: u64) -> MailboxAction {
    if slot.ns != ns || slot.due <= now {
        MailboxAction::Deposit
    } else if slot.status == MailboxStatus::Kept
        && now >= slot.synced_at.saturating_add(MAILBOX_RESYNC_SECS)
    {
        MailboxAction::Resync
    } else {
        MailboxAction::Nothing
    }
}

/// Delivery state of one conversation's deposit, as the app sees it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailboxStatus {
    #[default]
    Pending,
    Kept,
    /// No server kept it; retried after [`MAILBOX_RETRY_SECS`].
    Failed,
}

struct Conv {
    title: String,
    doc: DocHandle,
    conv: Conversation,
    processed: HashSet<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct AppFile {
    name: String,
    author: String,
    identity: String,
    /// Offline mailbox opt-in; absent in files written before Phase E.
    #[serde(default)]
    mailbox: bool,
}

#[derive(Serialize, Deserialize)]
struct ConvFile {
    id: String,
    doc: String,
    title: String,
    state: String,
}

/// Events pushed to the app over SSE.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutEvent {
    Message {
        conversation: String,
        from: String,
        from_name: String,
        body: String,
        sent_at: u64,
        mine: bool,
    },
    Members {
        conversation: String,
        members: Vec<MemberView>,
    },
    Removed {
        conversation: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberView {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
    pub admin: bool,
}

#[derive(Debug)]
pub enum ChatApiError {
    Unavailable,
    NotFound,
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ChatApiError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "chat unavailable".to_string(),
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "conversation not found".to_string()),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            Self::Internal(m) => {
                warn!(error = %m, "chat internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "chat error".to_string())
            }
        };
        (code, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

fn internal<E: std::fmt::Display>(e: E) -> ChatApiError {
    ChatApiError::Internal(e.to_string())
}

fn bad<E: std::fmt::Display>(e: E) -> ChatApiError {
    ChatApiError::BadRequest(e.to_string())
}

/// Load or create the 32-byte sealing key (0600 on unix).
pub fn load_or_create_chat_key(dir: &FsPath) -> std::io::Result<[u8; 32]> {
    let path = dir.join(CHAT_KEY_FILE);
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(key) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return Ok(key);
    }
    std::fs::create_dir_all(dir)?;
    let mut key = [0u8; 32];
    getrandom_fill(&mut key);
    std::fs::write(&path, key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(key)
}

fn getrandom_fill(buf: &mut [u8]) {
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(buf);
}

impl ChatService {
    pub fn new(
        node: Arc<Node>,
        root: PathBuf,
        key: [u8; 32],
        enabled: bool,
        mailbox_servers: Vec<EndpointId>,
    ) -> Self {
        Self {
            node,
            root,
            key,
            enabled,
            mailbox_servers,
            apps: Mutex::new(HashMap::new()),
        }
    }

    fn app_dir(&self, app: &str) -> PathBuf {
        let h = nexus_core_rs::blake3_hash(app.as_bytes());
        self.root.join(
            h.iter()
                .take(16)
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        )
    }

    /// Load (or create) an app's chat state and start its pump loop.
    async fn app(self: &Arc<Self>, app: &str) -> Result<Arc<AppChat>, ChatApiError> {
        if !self.enabled {
            return Err(ChatApiError::Unavailable);
        }
        let mut apps = self.apps.lock().await;
        if let Some(a) = apps.get(app) {
            return Ok(Arc::clone(a));
        }
        let dir = self.app_dir(app);
        std::fs::create_dir_all(dir.join("conv")).map_err(internal)?;
        let docs = DocsClient::for_node(&self.node);

        let app_file = dir.join("app.json");
        let state = if let Ok(raw) = std::fs::read_to_string(&app_file) {
            let f: AppFile = serde_json::from_str(&raw).map_err(internal)?;
            let identity = Identity::unseal(&f.identity, &self.key).map_err(internal)?;
            let author = DocsAuthorId::from_str(&f.author).map_err(internal)?;
            let mut convs = HashMap::new();
            for entry in std::fs::read_dir(dir.join("conv"))
                .map_err(internal)?
                .flatten()
            {
                match self.load_conv(&docs, &entry.path()).await {
                    Ok((id, conv)) => {
                        convs.insert(id, conv);
                    }
                    Err(e) => {
                        warn!(path = %entry.path().display(), error = ?e, "skipping unreadable conversation")
                    }
                }
            }
            AppState {
                identity,
                name: f.name,
                author,
                convs,
                mailbox: f.mailbox,
            }
        } else {
            let st = AppState {
                identity: Identity::new(),
                name: String::new(),
                author: docs.author_create().await.map_err(internal)?,
                convs: HashMap::new(),
                mailbox: false,
            };
            write_app_file(&app_file, &st, &self.key)?;
            st
        };

        let (tx, _) = broadcast::channel(EVENT_BUFFER);
        let chat = Arc::new(AppChat {
            dir,
            state: Mutex::new(state),
            events: tx,
            wake: Notify::new(),
            mailbox: std::sync::Mutex::new(HashMap::new()),
        });
        {
            let st = chat.state.lock().await;
            for conv in st.convs.values() {
                spawn_doc_watch(Arc::clone(&chat), conv.doc.clone());
            }
        }
        spawn_pump(Arc::clone(self), Arc::clone(&chat));
        apps.insert(app.to_owned(), Arc::clone(&chat));
        info!(app = %app, "chat app state ready");
        Ok(chat)
    }

    async fn load_conv(
        &self,
        docs: &DocsClient,
        path: &FsPath,
    ) -> Result<(String, Conv), ChatApiError> {
        let f: ConvFile = serde_json::from_str(&std::fs::read_to_string(path).map_err(internal)?)
            .map_err(internal)?;
        let ns = DocsNamespaceId::from_str(&f.doc).map_err(internal)?;
        let doc = docs
            .open_doc(ns)
            .await
            .map_err(internal)?
            .ok_or_else(|| ChatApiError::Internal("conversation document missing".into()))?;
        // Re-dial the peers the docs store remembers for this namespace.
        if let Err(e) = doc.start_sync(Vec::new()).await {
            debug!(error = %e, "start_sync on reopened conversation");
        }
        let conv = Conversation::unseal_state(&f.state, &self.key).map_err(internal)?;
        Ok((
            f.id,
            Conv {
                title: f.title,
                doc,
                conv,
                processed: HashSet::new(),
            },
        ))
    }

    fn persist_conv(&self, app: &AppChat, id: &str, c: &Conv) -> Result<(), ChatApiError> {
        let f = ConvFile {
            id: id.to_owned(),
            doc: c.doc.id().to_string(),
            title: c.title.clone(),
            state: c.conv.seal_state(&self.key),
        };
        atomic_write(
            &app.dir.join("conv").join(format!("{id}.json")),
            &serde_json::to_vec(&f).map_err(internal)?,
        )
    }

    /// Ingest every new document entry of every conversation of `app`.
    async fn pump(self: &Arc<Self>, app: &Arc<AppChat>) {
        let blobs = BlobsClient::new(self.node.blobs_store());
        let docs = DocsClient::for_node(&self.node);
        let mut guard = app.state.lock().await;
        let AppState {
            identity,
            author,
            convs,
            mailbox,
            ..
        } = &mut *guard;
        let mailbox_on = *mailbox && !self.mailbox_servers.is_empty();
        let ids: Vec<String> = convs.keys().cloned().collect();
        for id in ids {
            let Some(c) = convs.get_mut(&id) else {
                continue;
            };
            let mut entries = match c.doc.get_many_by_prefix(b"m/").await {
                Ok(e) => e,
                Err(e) => {
                    debug!(error = %e, "chat doc read failed");
                    continue;
                }
            };
            entries.sort_by_key(|e| e.timestamp());
            let mut changed = false;
            let mut moved_to = None;
            for entry in entries {
                if c.processed.contains(entry.key()) {
                    continue;
                }
                let Ok(bytes) = blobs.get_bytes(*entry.content_hash().as_bytes()).await else {
                    continue; // content not downloaded yet: retry next pump
                };
                c.processed.insert(entry.key().to_vec());
                let Ok(env) = Envelope::open(&bytes) else {
                    debug!("dropping unverifiable chat entry");
                    continue;
                };
                let ingested = match c.conv.ingest(identity, &env, now()) {
                    Ok(i) => i,
                    Err(e) => {
                        debug!(error = %e, "chat entry rejected");
                        continue;
                    }
                };
                changed = true;
                for out in ingested.outgoing {
                    write_envelope(&c.doc, *author, &out).await;
                }
                for ev in ingested.events {
                    match ev {
                        ChatEvent::Moved { ticket, .. } => moved_to = Some(ticket),
                        other => emit(app, &id, &c.conv, identity, other),
                    }
                }
            }
            if let Some(ticket) = moved_to {
                match import_doc(&docs, &ticket).await {
                    Ok(doc) => {
                        c.doc = doc.clone();
                        c.processed.clear();
                        spawn_doc_watch(Arc::clone(app), doc);
                        if mailbox_on {
                            ChatService::rotate_mailbox(app, &id);
                        }
                        info!(conversation = %id, "conversation moved to a fresh document");
                    }
                    Err(e) => warn!(error = ?e, "could not follow conversation move"),
                }
            }
            if changed && let Err(e) = self.persist_conv(app, &id, c) {
                warn!(error = ?e, "persisting conversation failed");
            }
            if mailbox_on && !c.conv.is_removed() {
                self.deposit_if_due(app, &id, &c.doc).await;
            }
        }
    }

    /// Deposit `doc` when its slot is due, otherwise re-sync with the
    /// servers that keep it when the last sync is stale.
    async fn deposit_if_due(self: &Arc<Self>, app: &Arc<AppChat>, id: &str, doc: &DocHandle) {
        let ns = doc.id().to_string();
        let action = {
            let at = now();
            let mut slots = app.mailbox.lock().unwrap_or_else(|p| p.into_inner());
            let slot = slots.entry(id.to_owned()).or_default();
            let action = mailbox_action(slot, &ns, at);
            match action {
                // Claimed until the task reports back.
                MailboxAction::Deposit => {
                    slot.ns = ns.clone();
                    slot.due = at + MAILBOX_RETRY_SECS;
                }
                MailboxAction::Resync => slot.synced_at = at,
                MailboxAction::Nothing => {}
            }
            action
        };
        match action {
            MailboxAction::Nothing => {}
            MailboxAction::Resync => self.sync_with_servers(doc.clone()),
            MailboxAction::Deposit => self.deposit(app, id, doc, ns).await,
        }
    }

    /// Pull from — and push to — the mailbox servers. The client dials: the
    /// server is the always-on side, so this is the direction that works
    /// from behind any NAT, and it is what brings a returning member up to
    /// date. Fire and forget; the next resync retries.
    fn sync_with_servers(self: &Arc<Self>, doc: DocHandle) {
        let peers: Vec<EndpointAddr> = self
            .mailbox_servers
            .iter()
            .map(|s| EndpointAddr::new(*s))
            .collect();
        tokio::spawn(async move {
            if let Err(e) = doc.start_sync(peers).await {
                debug!(error = %e, "start_sync with mailbox servers");
            }
        });
    }

    async fn deposit(self: &Arc<Self>, app: &Arc<AppChat>, id: &str, doc: &DocHandle, ns: String) {
        let ticket = match doc.share_read().await {
            Ok(t) => t.to_string(),
            Err(e) => {
                debug!(error = %e, "share_read for mailbox");
                return;
            }
        };
        let svc = Arc::clone(self);
        let app = Arc::clone(app);
        let id = id.to_owned();
        let doc = doc.clone();
        tokio::spawn(async move {
            let req = MailboxRequest::deposit(ticket);
            let mut kept = 0usize;
            let mut expires_at = 0;
            for server in &svc.mailbox_servers {
                match crate::mailbox_protocol::mailbox_call(svc.node.endpoint(), *server, &req)
                    .await
                {
                    Ok(r) if r.verdict == MailboxVerdict::Kept => {
                        kept += 1;
                        expires_at = expires_at.max(r.expires_at);
                    }
                    Ok(r) => debug!(verdict = ?r.verdict, "mailbox deposit refused"),
                    Err(e) => debug!(error = %e, "mailbox deposit failed"),
                }
            }
            let ok = kept > 0;
            let now = now();
            let current = {
                let mut slots = app.mailbox.lock().unwrap_or_else(|p| p.into_inner());
                match slots.get_mut(&id) {
                    Some(slot) if slot.ns == ns => {
                        if ok {
                            slot.due = now + MAILBOX_REFRESH_SECS;
                            slot.status = MailboxStatus::Kept;
                            slot.expires_at = expires_at;
                            slot.synced_at = now;
                        } else {
                            slot.due = now + MAILBOX_RETRY_SECS;
                            slot.status = MailboxStatus::Failed;
                        }
                        true
                    }
                    _ => false,
                }
            };
            // An opt-out landed while we were dialing, and its withdrawal may
            // have reached the server before this deposit: release what we
            // just registered. (A rotation also clears the slot, but keeps
            // the old document on the servers on purpose.)
            if !current && ok && !app.state.lock().await.mailbox {
                svc.withdraw_all(vec![ns]);
            } else if current && ok {
                svc.sync_with_servers(doc);
            }
        });
    }

    /// The conversation left its document for a fresh one: make the new one
    /// due now. The old registration is NOT withdrawn — the old document
    /// carries the `Moved` entry that members still offline need to find the
    /// new one, possibly through the mailbox alone; it lapses at the
    /// server's TTL once nobody refreshes it.
    fn rotate_mailbox(app: &AppChat, id: &str) {
        app.mailbox
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(id);
    }

    /// Release `namespaces` on every mailbox server (fire and forget: an
    /// unreached server lets the registration lapse at its TTL).
    fn withdraw_all(self: &Arc<Self>, namespaces: Vec<String>) {
        if self.mailbox_servers.is_empty() || namespaces.is_empty() {
            return;
        }
        let svc = Arc::clone(self);
        tokio::spawn(async move {
            for ns in namespaces {
                let req = MailboxRequest::withdraw(ns);
                for server in &svc.mailbox_servers {
                    if let Err(e) =
                        crate::mailbox_protocol::mailbox_call(svc.node.endpoint(), *server, &req)
                            .await
                    {
                        debug!(error = %e, "mailbox withdraw failed");
                    }
                }
            }
        });
    }
}

fn emit(app: &AppChat, id: &str, conv: &Conversation, me: &Identity, ev: ChatEvent) {
    let out = match ev {
        ChatEvent::Text {
            from,
            body,
            sent_at,
        } => OutEvent::Message {
            conversation: id.to_owned(),
            from_name: member_name(conv, &from),
            mine: from == me.public().ed,
            from,
            body,
            sent_at,
        },
        ChatEvent::MembersChanged { .. } => OutEvent::Members {
            conversation: id.to_owned(),
            members: member_views(conv),
        },
        ChatEvent::Removed => OutEvent::Removed {
            conversation: id.to_owned(),
        },
        ChatEvent::Moved { .. } => return,
    };
    let _ = app.events.send(out);
}

fn member_name(conv: &Conversation, ed: &str) -> String {
    conv.members()
        .into_iter()
        .find(|m| m.identity.ed == ed)
        .map(|m| m.name)
        .unwrap_or_default()
}

fn member_views(conv: &Conversation) -> Vec<MemberView> {
    conv.members()
        .into_iter()
        .map(|m| MemberView {
            admin: m.identity.ed == conv.admin(),
            fingerprint: m.identity.fingerprint(),
            id: m.identity.ed,
            name: m.name,
        })
        .collect()
}

async fn write_envelope(doc: &DocHandle, author: DocsAuthorId, env: &Envelope) {
    let key = match env.doc_key() {
        Ok(k) => k,
        Err(e) => {
            warn!(error = %e, "envelope key");
            return;
        }
    };
    if let Err(e) = doc.set(author, key.into_bytes(), env.to_bytes()).await {
        warn!(error = %e, "writing chat envelope failed");
    }
}

async fn import_doc(docs: &DocsClient, ticket: &str) -> Result<DocHandle, ChatApiError> {
    let t = DocsTicket::from_str(ticket.trim()).map_err(bad)?;
    docs.import_ticket(t).await.map_err(internal)
}

fn spawn_doc_watch(app: Arc<AppChat>, doc: DocHandle) {
    tokio::spawn(async move {
        let mut stream = match doc.subscribe().await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "chat doc subscribe failed");
                return;
            }
        };
        while let Some(ev) = stream.next().await {
            if ev.is_err() {
                break;
            }
            app.wake.notify_one();
        }
    });
}

fn spawn_pump(svc: Arc<ChatService>, app: Arc<AppChat>) {
    tokio::spawn(async move {
        loop {
            svc.pump(&app).await;
            tokio::select! {
                _ = app.wake.notified() => {}
                _ = tokio::time::sleep(PUMP_TICK) => {}
            }
        }
    });
}

fn atomic_write(path: &FsPath, bytes: &[u8]) -> Result<(), ChatApiError> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).map_err(internal)?;
    std::fs::rename(&tmp, path).map_err(internal)
}

fn write_app_file(path: &FsPath, st: &AppState, key: &[u8; 32]) -> Result<(), ChatApiError> {
    let f = AppFile {
        name: st.name.clone(),
        author: st.author.to_string(),
        identity: st.identity.seal(key),
        mailbox: st.mailbox,
    };
    atomic_write(path, &serde_json::to_vec(&f).map_err(internal)?)
}

fn my_profile(st: &AppState, node_id: &str) -> MemberProfile {
    MemberProfile {
        identity: st.identity.public(),
        fallback_key: st.identity.fallback_key(),
        node_id: node_id.to_owned(),
        name: st.name.clone(),
    }
}

fn clean_text(s: &str, max: usize, what: &str) -> Result<String, ChatApiError> {
    let t = s.trim();
    if t.chars().count() > max {
        return Err(ChatApiError::BadRequest(format!("{what} too long")));
    }
    Ok(t.to_owned())
}

// ---------------------------------------------------------------------------
// HTTP handlers — mounted under /app/{name}/chat/ (authed loopback router)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct IdentityView {
    id: String,
    name: String,
    fingerprint: String,
}

pub async fn chat_identity(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
) -> Result<Json<IdentityView>, ChatApiError> {
    let a = state.chat.app(&app).await?;
    let st = a.state.lock().await;
    let p = st.identity.public();
    Ok(Json(IdentityView {
        fingerprint: p.fingerprint(),
        id: p.ed,
        name: st.name.clone(),
    }))
}

#[derive(Deserialize)]
pub struct ProfileBody {
    name: String,
}

pub async fn chat_set_profile(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
    Json(body): Json<ProfileBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let mut guard = a.state.lock().await;
    guard.name = clean_text(&body.name, MAX_NAME_CHARS, "name")?;
    write_app_file(&a.dir.join("app.json"), &guard, &svc.key)?;
    // Re-announce the profile in every conversation.
    let profile = my_profile(&guard, &state.node_id);
    let AppState {
        identity,
        author,
        convs,
        ..
    } = &mut *guard;
    for c in convs.values_mut() {
        if let Ok(env) = c.conv.announce_profile(identity, profile.clone()) {
            write_envelope(&c.doc, *author, &env).await;
        }
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct ConversationView {
    id: String,
    title: String,
    is_admin: bool,
    removed: bool,
    members: Vec<MemberView>,
}

pub async fn chat_list(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
) -> Result<Json<Vec<ConversationView>>, ChatApiError> {
    let a = state.chat.app(&app).await?;
    let st = a.state.lock().await;
    let mut out: Vec<ConversationView> = st
        .convs
        .iter()
        .map(|(id, c)| ConversationView {
            id: id.clone(),
            title: c.title.clone(),
            is_admin: c.conv.is_admin(),
            removed: c.conv.is_removed(),
            members: member_views(&c.conv),
        })
        .collect();
    out.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct CreateBody {
    title: String,
}

pub async fn chat_create(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
    Json(body): Json<CreateBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let title = clean_text(&body.title, MAX_TITLE_CHARS, "title")?;
    let docs = DocsClient::for_node(&svc.node);
    let doc = docs.create_doc().await.map_err(internal)?;
    let id = doc.id().to_string();
    let mut st = a.state.lock().await;
    let profile = my_profile(&st, &state.node_id);
    let (conv, out) = Conversation::create(&st.identity, profile, now()).map_err(internal)?;
    for env in &out {
        write_envelope(&doc, st.author, env).await;
    }
    let c = Conv {
        title,
        doc: doc.clone(),
        conv,
        processed: HashSet::new(),
    };
    svc.persist_conv(&a, &id, &c)?;
    st.convs.insert(id.clone(), c);
    drop(st);
    spawn_doc_watch(Arc::clone(&a), doc);
    a.wake.notify_one();
    Ok(Json(serde_json::json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct InviteBody {
    #[serde(default)]
    ttl_secs: Option<u64>,
}

pub async fn chat_invite(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app, id)): Path<(String, String)>,
    Json(body): Json<InviteBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let mut guard = a.state.lock().await;
    let AppState {
        identity, convs, ..
    } = &mut *guard;
    let c = convs.get_mut(&id).ok_or(ChatApiError::NotFound)?;
    let ticket = c.doc.share_write().await.map_err(internal)?.to_string();
    let ttl = body
        .ttl_secs
        .unwrap_or(INVITE_TTL_DEFAULT_SECS)
        .clamp(60, INVITE_TTL_MAX_SECS);
    let inv = c
        .conv
        .invite(identity, ticket, c.title.clone(), ttl, now())
        .map_err(bad)?;
    svc.persist_conv(&a, &id, c)?;
    Ok(Json(
        serde_json::json!({ "invite": inv.encode(), "expires_at": inv.body.expires_at }),
    ))
}

#[derive(Deserialize)]
pub struct AcceptBody {
    invite: String,
}

pub async fn chat_accept(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
    Json(body): Json<AcceptBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let inv = Invitation::decode(&body.invite, now()).map_err(bad)?;
    let docs = DocsClient::for_node(&svc.node);
    let doc = import_doc(&docs, &inv.body.doc_ticket).await?;
    let id = doc.id().to_string();
    let mut st = a.state.lock().await;
    if st.convs.contains_key(&id) {
        return Err(ChatApiError::BadRequest("already a member".into()));
    }
    let profile = my_profile(&st, &state.node_id);
    let (conv, out) = Conversation::accept(&mut st.identity, profile, &inv, now()).map_err(bad)?;
    for env in &out {
        write_envelope(&doc, st.author, env).await;
    }
    let c = Conv {
        title: inv.body.title.clone(),
        doc: doc.clone(),
        conv,
        processed: HashSet::new(),
    };
    svc.persist_conv(&a, &id, &c)?;
    write_app_file(&a.dir.join("app.json"), &st, &svc.key)?;
    st.convs.insert(id.clone(), c);
    drop(st);
    spawn_doc_watch(Arc::clone(&a), doc);
    a.wake.notify_one();
    Ok(Json(serde_json::json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct SendBody {
    text: String,
}

pub async fn chat_send(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app, id)): Path<(String, String)>,
    Json(body): Json<SendBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let mut guard = a.state.lock().await;
    let AppState {
        identity,
        author,
        convs,
        ..
    } = &mut *guard;
    let c = convs.get_mut(&id).ok_or(ChatApiError::NotFound)?;
    let out = c.conv.send_text(identity, &body.text, now()).map_err(bad)?;
    for env in &out {
        write_envelope(&c.doc, *author, env).await;
    }
    svc.persist_conv(&a, &id, c)?;
    drop(guard);
    a.wake.notify_one();
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct MessageView {
    from: String,
    from_name: String,
    body: String,
    sent_at: u64,
    mine: bool,
}

pub async fn chat_messages(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app, id)): Path<(String, String)>,
) -> Result<Json<Vec<MessageView>>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let blobs = BlobsClient::new(svc.node.blobs_store());
    let mut guard = a.state.lock().await;
    let me = guard.identity.public().ed;
    let c = guard.convs.get_mut(&id).ok_or(ChatApiError::NotFound)?;
    let entries = c.doc.get_many_by_prefix(b"m/").await.map_err(internal)?;
    let mut out = Vec::new();
    for entry in entries {
        let Ok(bytes) = blobs.get_bytes(*entry.content_hash().as_bytes()).await else {
            continue;
        };
        let Ok(env) = Envelope::open(&bytes) else {
            continue;
        };
        if let Some(ChatEvent::Text {
            from,
            body,
            sent_at,
        }) = c.conv.read_message(&env)
        {
            out.push(MessageView {
                from_name: member_name(&c.conv, &from),
                mine: from == me,
                from,
                body,
                sent_at,
            });
        }
    }
    out.sort_by_key(|m| m.sent_at);
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct RemoveBody {
    member: String,
}

pub async fn chat_remove(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app, id)): Path<(String, String)>,
    Json(body): Json<RemoveBody>,
) -> Result<Json<serde_json::Value>, ChatApiError> {
    let svc = &state.chat;
    let a = svc.app(&app).await?;
    let docs = DocsClient::for_node(&svc.node);
    let mut guard = a.state.lock().await;
    let AppState {
        identity,
        author,
        convs,
        mailbox,
        ..
    } = &mut *guard;
    let c = convs.get_mut(&id).ok_or(ChatApiError::NotFound)?;
    if !c.conv.is_admin() {
        return Err(ChatApiError::BadRequest("only the admin can remove".into()));
    }
    let new_doc = docs.create_doc().await.map_err(internal)?;
    let ticket = new_doc.share_write().await.map_err(internal)?.to_string();
    let out = c
        .conv
        .remove_member(identity, &body.member, ticket, now())
        .map_err(bad)?;
    for env in &out {
        write_envelope(&c.doc, *author, env).await;
    }
    c.doc = new_doc.clone();
    c.processed.clear();
    svc.persist_conv(&a, &id, c)?;
    let members = member_views(&c.conv);
    if *mailbox {
        ChatService::rotate_mailbox(&a, &id);
    }
    drop(guard);
    spawn_doc_watch(Arc::clone(&a), new_doc);
    a.wake.notify_one();
    let _ = a.events.send(OutEvent::Members {
        conversation: id,
        members,
    });
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct MailboxView {
    enabled: bool,
    /// Endpoint ids the conversations are deposited to (`[mailbox] servers`).
    servers: Vec<String>,
    conversations: Vec<MailboxConvView>,
}

#[derive(Serialize)]
pub struct MailboxConvView {
    id: String,
    status: MailboxStatus,
    /// Unix seconds the furthest server keeps it until; `0` if not kept.
    expires_at: u64,
}

async fn mailbox_view(svc: &ChatService, a: &AppChat) -> MailboxView {
    let st = a.state.lock().await;
    let slots = a.mailbox.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let mut conversations: Vec<MailboxConvView> = if st.mailbox {
        st.convs
            .iter()
            .filter(|(_, c)| !c.conv.is_removed())
            .map(|(id, c)| {
                let ns = c.doc.id().to_string();
                let slot = slots.get(id).filter(|s| s.ns == ns);
                MailboxConvView {
                    id: id.clone(),
                    status: slot.map(|s| s.status).unwrap_or_default(),
                    expires_at: slot.map(|s| s.expires_at).unwrap_or(0),
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    conversations.sort_by(|x, y| x.id.cmp(&y.id));
    MailboxView {
        enabled: st.mailbox,
        servers: svc.mailbox_servers.iter().map(|s| s.to_string()).collect(),
        conversations,
    }
}

pub async fn chat_mailbox(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
) -> Result<Json<MailboxView>, ChatApiError> {
    let a = state.chat.app(&app).await?;
    Ok(Json(mailbox_view(&state.chat, &a).await))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailboxBody {
    enabled: bool,
}

/// Turn the offline mailbox on or off for one app (the user's consent,
/// collected by the app). Off withdraws every registration.
pub async fn chat_set_mailbox(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
    Json(body): Json<MailboxBody>,
) -> Result<Json<MailboxView>, ChatApiError> {
    let svc = &state.chat;
    if body.enabled && svc.mailbox_servers.is_empty() {
        return Err(ChatApiError::BadRequest(
            "no mailbox server configured".into(),
        ));
    }
    let a = svc.app(&app).await?;
    let mut st = a.state.lock().await;
    if st.mailbox != body.enabled {
        st.mailbox = body.enabled;
        write_app_file(&a.dir.join("app.json"), &st, &svc.key)?;
        // Flag first, then slots: an in-flight deposit that finds its slot
        // gone reads the flag to decide whether to release (see the task).
        a.mailbox.lock().unwrap_or_else(|p| p.into_inner()).clear();
        if !body.enabled {
            let all = st.convs.values().map(|c| c.doc.id().to_string()).collect();
            svc.withdraw_all(all);
        }
    }
    drop(st);
    a.wake.notify_one();
    Ok(Json(mailbox_view(svc, &a).await))
}

pub async fn chat_stream(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app): Path<String>,
) -> Result<impl IntoResponse, ChatApiError> {
    let a = state.chat.app(&app).await?;
    let rx = a.events.subscribe();
    let stream = futures_lite::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(ev) => {
                    let data = serde_json::to_string(&ev).unwrap_or_default();
                    return Some((
                        Ok::<_, std::convert::Infallible>(Event::default().data(data)),
                        rx,
                    ));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{build_test_router, mk_state, mk_state_with_mode};
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    async fn call(
        state: &Arc<DaemonHttpState>,
        method: Method,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let resp = build_test_router(Arc::clone(state))
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }

    async fn texts(state: &Arc<DaemonHttpState>, conv: &str) -> Vec<String> {
        let (s, v) = call(
            state,
            Method::GET,
            &format!("/app/chat-test/chat/conversations/{conv}/messages"),
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        v.as_array()
            .unwrap()
            .iter()
            .map(|m| m["body"].as_str().unwrap().to_owned())
            .collect()
    }

    async fn wait_texts(state: &Arc<DaemonHttpState>, conv: &str, want: &[&str]) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
        loop {
            let got = texts(state, conv).await;
            if want.iter().all(|w| got.iter().any(|g| g == w)) {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "timed out waiting for {want:?}, have {got:?}"
            );
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    /// Wait until the ADMIN `admin` lists `n` members of `conv`, i.e. has
    /// ingested every join. The invitee's own view is no signal: it lists the
    /// inviter and itself from the invitation alone, before the admin knows
    /// it — and a message the admin sends before that point is encrypted for
    /// nobody but its author, unreadable to the newcomer forever.
    async fn wait_admitted(admin: &Arc<DaemonHttpState>, conv: &str, n: usize) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
        loop {
            let (_, v) = call(
                admin,
                Method::GET,
                "/app/chat-test/chat/conversations",
                serde_json::Value::Null,
            )
            .await;
            let listed = v
                .as_array()
                .and_then(|a| a.iter().find(|c| c["id"] == conv))
                .and_then(|c| c["members"].as_array())
                .map(Vec::len);
            if listed == Some(n) {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "the admin never admitted {n} members: {v}"
            );
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn two_daemons_chat_end_to_end_over_iroh_docs() {
        let alice = mk_state().await;
        let bob = mk_state().await;
        let app = "/app/chat-test/chat";

        let (s, _) = call(
            &alice,
            Method::POST,
            &format!("{app}/profile"),
            serde_json::json!({"name": "Alice"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        let (s, v) = call(
            &alice,
            Method::POST,
            &format!("{app}/conversations"),
            serde_json::json!({"title": "Projet"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        let conv = v["id"].as_str().unwrap().to_owned();
        let (s, v) = call(
            &alice,
            Method::POST,
            &format!("{app}/conversations/{conv}/invite"),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        let invite = v["invite"].as_str().unwrap().to_owned();
        assert!(invite.starts_with("sbfb-chat:"));

        let (s, v) = call(
            &bob,
            Method::POST,
            &format!("{app}/accept"),
            serde_json::json!({"invite": invite}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        assert_eq!(
            v["id"].as_str().unwrap(),
            conv,
            "same document id on both sides"
        );
        let (s, v) = call(
            &bob,
            Method::POST,
            &format!("{app}/accept"),
            serde_json::json!({"invite": invite}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{v}");

        // Wait until Alice's roster includes Bob, then talk both ways.
        wait_admitted(&alice, &conv, 2).await;
        let (s, _) = call(
            &alice,
            Method::POST,
            &format!("{app}/conversations/{conv}/send"),
            serde_json::json!({"text": "bonjour bob secret-1"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = call(
            &bob,
            Method::POST,
            &format!("{app}/conversations/{conv}/send"),
            serde_json::json!({"text": "salut alice secret-2"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        wait_texts(
            &bob,
            &conv,
            &["bonjour bob secret-1", "salut alice secret-2"],
        )
        .await;
        wait_texts(
            &alice,
            &conv,
            &["bonjour bob secret-1", "salut alice secret-2"],
        )
        .await;

        // No plaintext at rest in either daemon's chat directory.
        for st in [&alice, &bob] {
            for entry in walk(&st.chat.root) {
                let bytes = std::fs::read(&entry).unwrap();
                let hay = String::from_utf8_lossy(&bytes);
                assert!(
                    !hay.contains("secret-1") && !hay.contains("secret-2"),
                    "plaintext in {}",
                    entry.display()
                );
            }
        }

        // Another app on the same daemon sees nothing of this conversation.
        let (s, v) = call(
            &bob,
            Method::GET,
            "/app/other-app/chat/conversations",
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    /// Alice creates a conversation, Bob joins, both have spoken: every
    /// session key is exchanged. Returns the conversation id.
    async fn paired_conversation(
        alice: &Arc<DaemonHttpState>,
        bob: &Arc<DaemonHttpState>,
    ) -> String {
        let app = "/app/chat-test/chat";
        let (_, v) = call(
            alice,
            Method::POST,
            &format!("{app}/conversations"),
            serde_json::json!({"title": "Boite"}),
        )
        .await;
        let conv = v["id"].as_str().unwrap().to_owned();
        let (_, v) = call(
            alice,
            Method::POST,
            &format!("{app}/conversations/{conv}/invite"),
            serde_json::json!({}),
        )
        .await;
        let invite = v["invite"].as_str().unwrap().to_owned();
        let (s, v) = call(
            bob,
            Method::POST,
            &format!("{app}/accept"),
            serde_json::json!({"invite": invite}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        wait_admitted(alice, &conv, 2).await;
        for (who, text) in [(alice, "hello-1"), (bob, "hello-2")] {
            call(
                who,
                Method::POST,
                &format!("{app}/conversations/{conv}/send"),
                serde_json::json!({ "text": text }),
            )
            .await;
        }
        wait_texts(alice, &conv, &["hello-1", "hello-2"]).await;
        wait_texts(bob, &conv, &["hello-1", "hello-2"]).await;
        conv
    }

    async fn set_mailbox(st: &Arc<DaemonHttpState>, on: bool) -> (StatusCode, serde_json::Value) {
        call(
            st,
            Method::POST,
            "/app/chat-test/chat/mailbox",
            serde_json::json!({ "enabled": on }),
        )
        .await
    }

    async fn wait_kept(st: &Arc<DaemonHttpState>, conv: &str) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
        loop {
            let (_, v) = call(
                st,
                Method::GET,
                "/app/chat-test/chat/mailbox",
                serde_json::Value::Null,
            )
            .await;
            let kept = v["conversations"].as_array().unwrap().iter().any(|c| {
                c["id"] == conv && c["status"] == "kept" && c["expires_at"].as_u64() > Some(0)
            });
            if kept {
                return;
            }
            assert!(tokio::time::Instant::now() < deadline, "never kept: {v}");
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    async fn app_chat(st: &Arc<DaemonHttpState>) -> Arc<AppChat> {
        st.chat.app("chat-test").await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn returning_member_catches_up_through_the_mailbox_alone() {
        use crate::mailbox_protocol::tests::{LIMITS, anchor, wait_entries};
        let anchor = anchor(LIMITS).await;
        let alice = crate::test_support::mk_state_with_mailbox(vec![anchor.id()]).await;
        let bob = crate::test_support::mk_state_with_mailbox(vec![anchor.id()]).await;
        for st in [&alice, &bob] {
            st.node
                .memory_lookup()
                .add_endpoint_info(anchor.addr.clone());
        }
        let conv = paired_conversation(&alice, &bob).await;

        // Off by default: the server lists nothing.
        let (s, v) = call(
            &alice,
            Method::GET,
            "/app/chat-test/chat/mailbox",
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(v["enabled"], false);
        assert_eq!(v["servers"][0], anchor.id().to_string());
        assert!(anchor.server.held().await.is_empty());

        // Bob drops off this conversation BEFORE the mailbox ever existed
        // for it: his store knows Alice, not the anchor.
        let bob_doc = {
            let a = app_chat(&bob).await;
            let st = a.state.lock().await;
            st.convs[&conv].doc.clone()
        };
        bob_doc.inner().leave().await.unwrap();

        let (s, v) = set_mailbox(&alice, true).await;
        assert_eq!(s, StatusCode::OK, "{v}");
        assert_eq!(v["enabled"], true);
        wait_kept(&alice, &conv).await;
        assert_eq!(anchor.server.held().await, vec![conv.clone()]);

        // Alice writes; only the mailbox gets it.
        let ns = DocsNamespaceId::from_str(&conv).unwrap();
        let before = wait_entries(&anchor.node, ns, 1).await.len();
        let (s, _) = call(
            &alice,
            Method::POST,
            &format!("/app/chat-test/chat/conversations/{conv}/send"),
            serde_json::json!({"text": "while-you-were-out secret-3"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        let held = wait_entries(&anchor.node, ns, before + 1).await;
        for bytes in &held {
            assert!(
                !String::from_utf8_lossy(bytes).contains("secret-3"),
                "the mailbox holds ciphertext only"
            );
        }
        assert!(
            !texts(&bob, &conv)
                .await
                .iter()
                .any(|t| t.contains("secret-3")),
            "bob must not have it before coming back"
        );

        // Alice goes away for good. Bob comes back and opts in: his deposit
        // lands on a document the anchor already holds, and the anchor
        // syncs with him — the only peer left that has the message.
        alice.node.endpoint().close().await;
        let (s, v) = set_mailbox(&bob, true).await;
        assert_eq!(s, StatusCode::OK, "{v}");
        wait_kept(&bob, &conv).await;
        wait_texts(&bob, &conv, &["while-you-were-out secret-3"]).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn mailbox_opt_out_withdraws_and_opt_in_needs_a_server() {
        use crate::mailbox_protocol::tests::{LIMITS, anchor};
        // No server configured: the opt-in is refused, not silently inert.
        let lone = mk_state().await;
        let (s, _) = set_mailbox(&lone, true).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (s, _) = call(
            &lone,
            Method::POST,
            "/app/chat-test/chat/mailbox",
            serde_json::json!({"enabled": true, "server": "x"}),
        )
        .await;
        assert_eq!(s.as_u16() / 100, 4, "unknown fields are refused");

        let anchor = anchor(LIMITS).await;
        let alice = crate::test_support::mk_state_with_mailbox(vec![anchor.id()]).await;
        alice
            .node
            .memory_lookup()
            .add_endpoint_info(anchor.addr.clone());
        let (_, v) = call(
            &alice,
            Method::POST,
            "/app/chat-test/chat/conversations",
            serde_json::json!({"title": "Solo"}),
        )
        .await;
        let conv = v["id"].as_str().unwrap().to_owned();
        set_mailbox(&alice, true).await;
        wait_kept(&alice, &conv).await;
        assert_eq!(anchor.server.held().await, vec![conv.clone()]);
        // The choice survives a restart.
        let a = app_chat(&alice).await;
        let file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(a.dir.join("app.json")).unwrap()).unwrap();
        assert_eq!(file["mailbox"], true);

        let (s, v) = set_mailbox(&alice, false).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(v["enabled"], false);
        assert_eq!(v["conversations"].as_array().unwrap().len(), 0);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while !anchor.server.held().await.is_empty() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "opt-out never withdrew"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let ns = DocsNamespaceId::from_str(&conv).unwrap();
        assert!(!anchor.docs().list_docs().await.unwrap().contains(&ns));
    }

    fn walk(dir: &FsPath) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.extend(walk(&p));
                } else {
                    out.push(p);
                }
            }
        }
        out
    }

    #[tokio::test]
    async fn chat_is_off_under_duress() {
        // The runtime builds the service with `enabled = (mode == Normal)`;
        // a disabled service refuses every app before touching disk.
        let st = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let svc = ChatService::new(
            Arc::clone(&st.node),
            std::env::temp_dir().join("chat-duress"),
            [1; 32],
            false,
            Vec::new(),
        );
        assert!(matches!(
            Arc::new(svc).app("x").await,
            Err(ChatApiError::Unavailable)
        ));
    }

    /// Chat plan Phase I: an invitation travels as a link or a QR code to
    /// whoever the user hands it to; it names the inviter's relay and
    /// private addresses, never a public IP the node knows of itself.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn chat_invite_ticket_has_no_global_ip() {
        const PUBLIC: &str = "198.51.100.7:4433";
        let alice = mk_state().await;
        alice
            .node
            .endpoint()
            .add_external_addr(PUBLIC.parse().unwrap())
            .await;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while !alice
            .node
            .endpoint()
            .addr()
            .ip_addrs()
            .any(|a| a.to_string() == PUBLIC)
        {
            assert!(
                tokio::time::Instant::now() < deadline,
                "external address never advertised"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let app = "/app/chat-test/chat";
        let (_, v) = call(
            &alice,
            Method::POST,
            &format!("{app}/conversations"),
            serde_json::json!({"title": "Lien"}),
        )
        .await;
        let conv = v["id"].as_str().unwrap().to_owned();
        let (s, v) = call(
            &alice,
            Method::POST,
            &format!("{app}/conversations/{conv}/invite"),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{v}");
        let inv = Invitation::decode(v["invite"].as_str().unwrap(), now()).unwrap();
        let ticket = DocsTicket::from_str(&inv.body.doc_ticket).unwrap();
        assert!(!ticket.nodes.is_empty(), "the inviter is named");
        for node in &ticket.nodes {
            assert!(
                !node.ip_addrs().any(|a| a.to_string() == PUBLIC),
                "public IP in the invitation: {node:?}"
            );
        }
    }

    #[tokio::test]
    async fn invalid_inputs_are_400_not_500() {
        let st = mk_state().await;
        let app = "/app/chat-test/chat";
        let (s, _) = call(
            &st,
            Method::POST,
            &format!("{app}/accept"),
            serde_json::json!({"invite": "sbfb-chat:garbage"}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (s, _) = call(
            &st,
            Method::POST,
            &format!("{app}/conversations"),
            serde_json::json!({"title": "x".repeat(500)}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (s, _) = call(
            &st,
            Method::POST,
            &format!("{app}/conversations/nope/send"),
            serde_json::json!({"text": "hi"}),
        )
        .await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[test]
    fn mailbox_pump_deposits_when_due_and_resyncs_when_stale() {
        let kept = |due, synced_at| MailboxSlot {
            ns: "d1".into(),
            due,
            status: MailboxStatus::Kept,
            expires_at: 9,
            synced_at,
        };
        // A fresh slot, or one about another document, deposits.
        assert_eq!(
            mailbox_action(&MailboxSlot::default(), "d1", 0),
            MailboxAction::Deposit
        );
        assert_eq!(
            mailbox_action(&kept(1_000, 0), "d2", 0),
            MailboxAction::Deposit
        );
        // A due deposit wins over everything.
        assert_eq!(
            mailbox_action(&kept(100, 100), "d1", 100),
            MailboxAction::Deposit
        );
        // Kept and fresh: nothing; kept and stale: pull from the servers.
        let s = kept(100_000, 1_000);
        assert_eq!(
            mailbox_action(&s, "d1", 1_000 + MAILBOX_RESYNC_SECS - 1),
            MailboxAction::Nothing
        );
        assert_eq!(
            mailbox_action(&s, "d1", 1_000 + MAILBOX_RESYNC_SECS),
            MailboxAction::Resync
        );
        // A failed or pending deposit is retried by the deposit path, not
        // resynced (nothing is kept to sync with).
        let failed = MailboxSlot {
            status: MailboxStatus::Failed,
            ..kept(100_000, 0)
        };
        assert_eq!(
            mailbox_action(&failed, "d1", 50_000),
            MailboxAction::Nothing
        );
    }

    #[test]
    fn chat_key_is_created_once_and_reused() {
        let dir = tempfile::tempdir().unwrap();
        let a = load_or_create_chat_key(dir.path()).unwrap();
        let b = load_or_create_chat_key(dir.path()).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, [0u8; 32]);
    }
}
