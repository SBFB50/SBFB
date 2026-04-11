//! # nexus-core-py
//!
//! PyO3 bindings for [`nexus_core_rs`]. Exposes five Python
//! classes (`Node`, `Doc`, `Gossip`, `Blobs`, `Verifier`) and the
//! handful of free functions Sprint 4 needs to drive them from
//! the coordinator and the SDK.
//!
//! Matches the Sprint 2 scope in the SBFB plan section
//! "nexus-core-py | PyO3 extension | ~300 LOC".
//!
//! ## Python usage
//!
//! ```python
//! import asyncio, nexus_core
//!
//! async def main():
//!     node = await nexus_core.create_node()
//!     doc = await node.docs_create()
//!     author = await node.docs_author_create()
//!     await doc.set(author, b"k", b"v")
//!     ticket = await doc.share_write()
//!     await node.shutdown()
//!
//! asyncio.run(main())
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyModule};
use tokio::sync::Mutex;

use futures_lite::StreamExt;
use iroh_docs::engine::LiveEvent;
use iroh_docs::NamespaceId;
use nexus_core_rs::{
    blobs::BlobsClient as RsBlobsClient,
    create_node as rs_create_node, create_node_with_config,
    crypto::{KeyPair, SECRET_KEY_BYTES},
    curator::{CuratorList, CuratorListEntry},
    discovery::DiscoveryClient as RsDiscoveryClient,
    docs::{DocHandle as RsDocHandle, DocsClient as RsDocsClient},
    gossip::{
        GossipClient as RsGossipClient, GossipEvent as RsGossipEvent, TopicHandle as RsTopicHandle,
    },
    task::{Claim, ClaimEntry, ResultEntry, ResultPayload, Task, TaskEntry},
    Node, NodeConfig, VerificationReport, Verifier as RsVerifier,
};
use nexus_worker_core::invite::{
    current_unix_secs, Invite as RsInvite, InviteError as RsInviteError,
    InviteScope as RsInviteScope,
};
use std::str::FromStr;

fn py_err<E: std::fmt::Display>(label: &str, e: E) -> PyErr {
    PyRuntimeError::new_err(format!("{label}: {e}"))
}

fn array32(b: &Bound<'_, PyBytes>, label: &str) -> PyResult<[u8; 32]> {
    let s = b.as_bytes();
    if s.len() != 32 {
        return Err(PyValueError::new_err(format!(
            "{label}: want 32 bytes, got {}",
            s.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(s);
    Ok(out)
}

// ======================================================================
// Node
// ======================================================================

#[pyclass(name = "Node", module = "nexus_core")]
pub struct PyNode {
    inner: Arc<Mutex<Option<Node>>>,
    cached_node_id: String,
}

#[pymethods]
impl PyNode {
    #[getter]
    fn node_id(&self) -> &str {
        &self.cached_node_id
    }

    fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let node = inner
                .lock()
                .await
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("already shut down"))?;
            node.shutdown().await.map_err(|e| py_err("shutdown", e))
        })
    }

    fn docs_author_create<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let id = RsDocsClient::new(n.docs())
                .author_create()
                .await
                .map_err(|e| py_err("author_create", e))?;
            Ok(id.to_string())
        })
    }

    fn docs_create<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let doc = RsDocsClient::new(n.docs())
                .create_doc()
                .await
                .map_err(|e| py_err("create_doc", e))?;
            Ok(PyDoc {
                inner: Arc::new(Mutex::new(Some(doc))),
            })
        })
    }

    fn docs_import<'py>(&self, py: Python<'py>, ticket: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let parsed: iroh_docs::DocTicket = ticket
                .parse()
                .map_err(|e| PyValueError::new_err(format!("bad ticket: {e}")))?;
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let doc = RsDocsClient::new(n.docs())
                .import_ticket(parsed)
                .await
                .map_err(|e| py_err("import_ticket", e))?;
            Ok(PyDoc {
                inner: Arc::new(Mutex::new(Some(doc))),
            })
        })
    }

    /// Re-open a locally-stored document by its namespace id.
    ///
    /// Used by the Sprint 4 coordinator on every boot after the
    /// first: the project's doc is created once via `docs_create`,
    /// its namespace id is persisted to `coordinator.toml`, and
    /// subsequent boots call `docs_open` to re-attach without
    /// producing a new namespace.
    ///
    /// Returns `None` if the document is not present on this node.
    fn docs_open<'py>(&self, py: Python<'py>, namespace_id: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let id = NamespaceId::from_str(&namespace_id)
                .map_err(|e| PyValueError::new_err(format!("bad namespace id: {e}")))?;
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let maybe = RsDocsClient::new(n.docs())
                .open_doc(id)
                .await
                .map_err(|e| py_err("open_doc", e))?;
            Ok(maybe.map(|doc| PyDoc {
                inner: Arc::new(Mutex::new(Some(doc))),
            }))
        })
    }

    fn gossip_join<'py>(
        &self,
        py: Python<'py>,
        topic_bytes: &Bound<'_, PyBytes>,
        bootstrap: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let topic = array32(topic_bytes, "topic_bytes")?;
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let handle = RsGossipClient::new(n.gossip())
                .join_topic(topic, bootstrap)
                .await
                .map_err(|e| py_err("join_topic", e))?;
            Ok(PyGossip {
                inner: Arc::new(Mutex::new(Some(handle))),
            })
        })
    }

    fn blobs(&self) -> PyBlobs {
        PyBlobs {
            node: Arc::clone(&self.inner),
        }
    }

    fn addr<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let info = RsDiscoveryClient::new(n.endpoint())
                .my_addr()
                .await
                .map_err(|e| py_err("my_addr", e))?;
            Ok(NodeAddrDict {
                node_id: info.node_id,
                relay_url: info.relay_url,
                direct_addresses: info.direct_addresses,
            })
        })
    }
}

/// Owned helper that converts to a Python dict via IntoPyObject.
/// Used as the return of `Node.addr()` so the future can yield
/// an owned Send value.
struct NodeAddrDict {
    node_id: String,
    relay_url: Option<String>,
    direct_addresses: Vec<String>,
}

impl<'py> IntoPyObject<'py> for NodeAddrDict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let d = PyDict::new(py);
        d.set_item("node_id", self.node_id)?;
        d.set_item("relay_url", self.relay_url)?;
        d.set_item("direct_addresses", self.direct_addresses)?;
        Ok(d)
    }
}

#[pyfunction]
fn create_node(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let node = rs_create_node()
            .await
            .map_err(|e| py_err("create_node", e))?;
        let cached_node_id = node.node_id();
        Ok(PyNode {
            inner: Arc::new(Mutex::new(Some(node))),
            cached_node_id,
        })
    })
}

#[pyfunction]
#[pyo3(signature = (secret, data_dir=None))]
fn create_node_with_secret<'py>(
    py: Python<'py>,
    secret: &Bound<'_, PyBytes>,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let mut cfg = NodeConfig::default().with_secret_key(sk);
        if let Some(path) = data_dir {
            cfg = cfg.with_data_dir(std::path::PathBuf::from(path));
        }
        let node = create_node_with_config(cfg)
            .await
            .map_err(|e| py_err("create_node_with_secret", e))?;
        let cached_node_id = node.node_id();
        Ok(PyNode {
            inner: Arc::new(Mutex::new(Some(node))),
            cached_node_id,
        })
    })
}

/// Generate a fresh Ed25519 keypair and return (secret_bytes, public_bytes).
#[pyfunction]
fn generate_secret<'py>(py: Python<'py>) -> Bound<'py, PyDict> {
    let kp = KeyPair::generate();
    let d = PyDict::new(py);
    d.set_item("secret", PyBytes::new(py, &kp.secret_bytes()))
        .ok();
    d.set_item("public", PyBytes::new(py, &kp.public_bytes()))
        .ok();
    d
}

/// Load or generate a persistent Ed25519 keypair at `path`.
/// Returns (secret_bytes, public_bytes).
#[pyfunction]
fn load_or_generate_secret<'py>(py: Python<'py>, path: String) -> PyResult<Bound<'py, PyDict>> {
    let kp = KeyPair::load_or_generate(&path).map_err(|e| py_err("load_or_generate", e))?;
    let d = PyDict::new(py);
    d.set_item("secret", PyBytes::new(py, &kp.secret_bytes()))?;
    d.set_item("public", PyBytes::new(py, &kp.public_bytes()))?;
    Ok(d)
}

// ======================================================================
// Doc
// ======================================================================

#[pyclass(name = "Doc", module = "nexus_core")]
pub struct PyDoc {
    inner: Arc<Mutex<Option<RsDocHandle>>>,
}

#[pymethods]
impl PyDoc {
    fn id<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            Ok(doc.id().to_string())
        })
    }

    fn set<'py>(
        &self,
        py: Python<'py>,
        author_hex: String,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let author: iroh_docs::AuthorId = author_hex
                .parse()
                .map_err(|e| PyValueError::new_err(format!("bad author id: {e}")))?;
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let hash = doc
                .set(author, key, value)
                .await
                .map_err(|e| py_err("set", e))?;
            Ok(ByteVec(hash.to_vec()))
        })
    }

    fn share_write<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let t = doc
                .share_write()
                .await
                .map_err(|e| py_err("share_write", e))?;
            Ok(t.to_string())
        })
    }

    fn share_read<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let t = doc
                .share_read()
                .await
                .map_err(|e| py_err("share_read", e))?;
            Ok(t.to_string())
        })
    }

    /// Return every entry whose key starts with `prefix`, from
    /// any author, as a list of dicts with keys
    /// `{author, key, hash, content_len, timestamp}`.
    ///
    /// Used by the coordinator dispatcher to scan `task:*` /
    /// `claim:*` / `result:*` entries in the project doc. The
    /// content bytes themselves are fetched via
    /// [`PyDoc::content_bytes`] keyed on the returned `hash`.
    fn get_many_by_prefix<'py>(
        &self,
        py: Python<'py>,
        prefix: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let entries = doc
                .get_many_by_prefix(&prefix)
                .await
                .map_err(|e| py_err("get_many_by_prefix", e))?;
            let out: Vec<EntryDict> = entries.into_iter().map(EntryDict::from).collect();
            Ok(out)
        })
    }

    /// Read a single entry by `(author_hex, key)`.
    ///
    /// Returns `None` if the entry does not exist, otherwise a
    /// dict matching the layout produced by
    /// [`PyDoc::get_many_by_prefix`].
    fn get_exact<'py>(
        &self,
        py: Python<'py>,
        author_hex: String,
        key: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let author: iroh_docs::AuthorId = author_hex
                .parse()
                .map_err(|e| PyValueError::new_err(format!("bad author id: {e}")))?;
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let entry = doc
                .get_exact(author, key)
                .await
                .map_err(|e| py_err("get_exact", e))?;
            Ok(entry.map(EntryDict::from))
        })
    }

    /// Subscribe to LiveEvents on this document.
    ///
    /// Returns a [`PyDocSubscription`] that the coordinator
    /// validator polls in a loop via `await sub.next_event()`
    /// to observe `InsertRemote` / `InsertLocal` / `SyncFinished`
    /// events. The subscription holds a `Box<dyn Stream>` pinned
    /// under a `Mutex<Option<...>>` so it can outlive the
    /// originating `PyDoc`.
    fn subscribe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = inner.lock().await;
            let doc = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("doc closed"))?;
            let stream = doc.subscribe().await.map_err(|e| py_err("subscribe", e))?;
            let boxed: LiveEventStream = Box::pin(stream);
            Ok(PyDocSubscription {
                inner: Arc::new(Mutex::new(Some(boxed))),
            })
        })
    }
}

/// Dict-shaped view of an iroh-docs Entry. Used as the return of
/// [`PyDoc::get_many_by_prefix`] and [`PyDoc::get_exact`].
struct EntryDict {
    author: String,
    key: Vec<u8>,
    hash: [u8; 32],
    content_len: u64,
    timestamp: u64,
}

impl From<iroh_docs::Entry> for EntryDict {
    fn from(e: iroh_docs::Entry) -> Self {
        EntryDict {
            author: e.author().to_string(),
            key: e.key().to_vec(),
            hash: *e.content_hash().as_bytes(),
            content_len: e.content_len(),
            timestamp: e.timestamp(),
        }
    }
}

impl<'py> IntoPyObject<'py> for EntryDict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let d = PyDict::new(py);
        d.set_item("author", self.author)?;
        d.set_item("key", PyBytes::new(py, &self.key))?;
        d.set_item("hash", PyBytes::new(py, &self.hash))?;
        d.set_item("content_len", self.content_len)?;
        d.set_item("timestamp", self.timestamp)?;
        Ok(d)
    }
}

/// Type alias for the boxed LiveEvent stream used by
/// [`PyDocSubscription`]. The alias keeps the long generic bound
/// out of the field declaration.
type LiveEventStream = std::pin::Pin<
    Box<dyn futures_lite::Stream<Item = nexus_core_rs::Result<LiveEvent>> + Send + Unpin>,
>;

/// Live subscription to a document, created by
/// [`PyDoc::subscribe`]. The coordinator validator polls
/// `next_event` in a loop to observe new results arriving from
/// workers.
#[pyclass(name = "DocSubscription", module = "nexus_core")]
pub struct PyDocSubscription {
    inner: Arc<Mutex<Option<LiveEventStream>>>,
}

#[pymethods]
impl PyDocSubscription {
    /// Pull the next [`LiveEvent`] off the stream.
    ///
    /// Returns:
    /// - A dict with `kind="insert_local"` / `"insert_remote"` /
    ///   `"content_ready"` / `"pending_content_ready"` /
    ///   `"neighbor_up"` / `"neighbor_down"` / `"sync_finished"`
    ///   on a new event.
    /// - `None` when the stream has ended (the doc was dropped or
    ///   the underlying sync engine shut down).
    fn next_event<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = inner.lock().await;
            let stream = guard
                .as_mut()
                .ok_or_else(|| PyRuntimeError::new_err("subscription closed"))?;
            let next = stream.next().await;
            match next {
                None => Ok(None::<LiveEventDict>),
                Some(Ok(ev)) => Ok(Some(LiveEventDict(ev))),
                Some(Err(e)) => Err(py_err("live event stream error", e)),
            }
        })
    }

    /// Close the subscription, dropping the underlying stream.
    /// Subsequent calls to `next_event` raise `RuntimeError`.
    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.lock().await.take();
            Ok(())
        })
    }
}

/// Dict converter for [`LiveEvent`]. Used as the return type of
/// [`PyDocSubscription::next_event`].
struct LiveEventDict(LiveEvent);

impl<'py> IntoPyObject<'py> for LiveEventDict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let d = PyDict::new(py);
        match self.0 {
            LiveEvent::InsertLocal { entry } => {
                d.set_item("kind", "insert_local")?;
                d.set_item("entry", EntryDict::from(entry))?;
            }
            LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => {
                d.set_item("kind", "insert_remote")?;
                d.set_item("from", from.to_string())?;
                d.set_item("entry", EntryDict::from(entry))?;
                d.set_item("content_status", format!("{content_status:?}"))?;
            }
            LiveEvent::ContentReady { hash } => {
                d.set_item("kind", "content_ready")?;
                d.set_item("hash", PyBytes::new(py, hash.as_bytes()))?;
            }
            LiveEvent::PendingContentReady => {
                d.set_item("kind", "pending_content_ready")?;
            }
            LiveEvent::NeighborUp(node_id) => {
                d.set_item("kind", "neighbor_up")?;
                d.set_item("node_id", node_id.to_string())?;
            }
            LiveEvent::NeighborDown(node_id) => {
                d.set_item("kind", "neighbor_down")?;
                d.set_item("node_id", node_id.to_string())?;
            }
            LiveEvent::SyncFinished(ev) => {
                d.set_item("kind", "sync_finished")?;
                d.set_item("summary", format!("{ev:?}"))?;
            }
        }
        Ok(d)
    }
}

/// Newtype that converts to Python `bytes` via IntoPyObject.
/// Used by Doc.set() and Blobs.add_bytes() so the async return
/// is a single owned Send value.
struct ByteVec(Vec<u8>);

impl<'py> IntoPyObject<'py> for ByteVec {
    type Target = PyBytes;
    type Output = Bound<'py, PyBytes>;
    type Error = std::convert::Infallible;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self.0))
    }
}

// ======================================================================
// Gossip (combined topic handle)
// ======================================================================

#[pyclass(name = "Gossip", module = "nexus_core")]
pub struct PyGossip {
    inner: Arc<Mutex<Option<RsTopicHandle>>>,
}

#[pymethods]
impl PyGossip {
    fn broadcast<'py>(&self, py: Python<'py>, message: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = inner.lock().await;
            let topic = guard
                .as_mut()
                .ok_or_else(|| PyRuntimeError::new_err("topic closed"))?;
            topic
                .broadcast(message)
                .await
                .map_err(|e| py_err("broadcast", e))
        })
    }

    fn next_event<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = inner.lock().await;
            let topic = guard
                .as_mut()
                .ok_or_else(|| PyRuntimeError::new_err("topic closed"))?;
            let ev = topic
                .next_event()
                .await
                .map_err(|e| py_err("next_event", e))?;
            Ok(ev.map(GossipEventDict::from))
        })
    }
}

struct GossipEventDict(RsGossipEvent);

impl From<RsGossipEvent> for GossipEventDict {
    fn from(e: RsGossipEvent) -> Self {
        GossipEventDict(e)
    }
}

impl<'py> IntoPyObject<'py> for GossipEventDict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let d = PyDict::new(py);
        match self.0 {
            RsGossipEvent::Message {
                content,
                delivered_from,
            } => {
                d.set_item("kind", "message")?;
                d.set_item("content", PyBytes::new(py, &content))?;
                d.set_item("delivered_from", delivered_from)?;
            }
            RsGossipEvent::NeighborUp { node_id } => {
                d.set_item("kind", "neighbor_up")?;
                d.set_item("node_id", node_id)?;
            }
            RsGossipEvent::NeighborDown { node_id } => {
                d.set_item("kind", "neighbor_down")?;
                d.set_item("node_id", node_id)?;
            }
            RsGossipEvent::Lagged => {
                d.set_item("kind", "lagged")?;
            }
        }
        Ok(d)
    }
}

// ======================================================================
// Blobs
// ======================================================================

#[pyclass(name = "Blobs", module = "nexus_core")]
pub struct PyBlobs {
    node: Arc<Mutex<Option<Node>>>,
}

#[pymethods]
impl PyBlobs {
    fn add_bytes<'py>(&self, py: Python<'py>, data: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let node = Arc::clone(&self.node);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = node.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let hash = RsBlobsClient::new(n.blobs_store())
                .add_bytes(data)
                .await
                .map_err(|e| py_err("add_bytes", e))?;
            Ok(ByteVec(hash.to_vec()))
        })
    }

    fn get_bytes<'py>(
        &self,
        py: Python<'py>,
        hash: &Bound<'_, PyBytes>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let h = array32(hash, "hash")?;
        let node = Arc::clone(&self.node);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = node.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            let data = RsBlobsClient::new(n.blobs_store())
                .get_bytes(h)
                .await
                .map_err(|e| py_err("get_bytes", e))?;
            Ok(ByteVec(data))
        })
    }

    fn has<'py>(&self, py: Python<'py>, hash: &Bound<'_, PyBytes>) -> PyResult<Bound<'py, PyAny>> {
        let h = array32(hash, "hash")?;
        let node = Arc::clone(&self.node);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let guard = node.lock().await;
            let n = guard
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("shut down"))?;
            RsBlobsClient::new(n.blobs_store())
                .has(h)
                .await
                .map_err(|e| py_err("has", e))
        })
    }
}

// ======================================================================
// Verifier
// ======================================================================

#[pyclass(name = "Verifier", module = "nexus_core")]
pub struct PyVerifier {
    inner: RsVerifier,
}

#[pymethods]
impl PyVerifier {
    #[new]
    fn new() -> Self {
        PyVerifier {
            inner: RsVerifier::new(),
        }
    }

    fn register_digest(&mut self, model: String, digest: &Bound<'_, PyBytes>) -> PyResult<()> {
        self.inner
            .register_digest(model, array32(digest, "digest")?);
        Ok(())
    }

    fn verify_entries<'py>(
        &self,
        py: Python<'py>,
        task_entry_json: &str,
        result_entry_json: &str,
        calibration_prompt_id: &str,
    ) -> PyResult<Bound<'py, PyDict>> {
        let task: TaskEntry = serde_json::from_str(task_entry_json)
            .map_err(|e| PyValueError::new_err(format!("bad task entry: {e}")))?;
        let result: ResultEntry = serde_json::from_str(result_entry_json)
            .map_err(|e| PyValueError::new_err(format!("bad result entry: {e}")))?;
        let report = self.inner.verify(&task, &result, calibration_prompt_id);
        report_to_dict(py, &report)
    }
}

fn report_to_dict<'py>(py: Python<'py>, r: &VerificationReport) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("passed", r.passed)?;
    d.set_item("trust_delta", r.trust_delta)?;
    d.set_item("ban", r.ban)?;
    d.set_item(
        "signature",
        format!("{:?}:{}", r.signature.status, r.signature.reason),
    )?;
    d.set_item(
        "digest",
        format!("{:?}:{}", r.digest.status, r.digest.reason),
    )?;
    d.set_item(
        "logprobs",
        format!("{:?}:{}", r.logprobs.status, r.logprobs.reason),
    )?;
    Ok(d)
}

// ======================================================================
// Task/Result sign/verify (JSON in/out)
// ======================================================================

#[pyfunction]
fn sign_task(task_json: &str, secret: &Bound<'_, PyBytes>) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let task: Task = serde_json::from_str(task_json)
        .map_err(|e| PyValueError::new_err(format!("bad task json: {e}")))?;
    let entry = TaskEntry::sign(task, &kp).map_err(|e| py_err("sign_task", e))?;
    serde_json::to_string(&entry).map_err(|e| py_err("serialize", e))
}

#[pyfunction]
fn verify_task_entry(entry_json: &str) -> PyResult<()> {
    let entry: TaskEntry = serde_json::from_str(entry_json)
        .map_err(|e| PyValueError::new_err(format!("bad entry json: {e}")))?;
    entry
        .verify_signature()
        .map_err(|e| py_err("verify_task_entry", e))
}

#[pyfunction]
fn sign_result(result_json: &str, secret: &Bound<'_, PyBytes>) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let payload: ResultPayload = serde_json::from_str(result_json)
        .map_err(|e| PyValueError::new_err(format!("bad result json: {e}")))?;
    let entry = ResultEntry::sign(payload, &kp).map_err(|e| py_err("sign_result", e))?;
    serde_json::to_string(&entry).map_err(|e| py_err("serialize", e))
}

#[pyfunction]
fn verify_result_entry(entry_json: &str) -> PyResult<()> {
    let entry: ResultEntry = serde_json::from_str(entry_json)
        .map_err(|e| PyValueError::new_err(format!("bad entry json: {e}")))?;
    entry
        .verify_signature()
        .map_err(|e| py_err("verify_result_entry", e))
}

/// Sign a Claim with the worker's keypair.
///
/// Takes the claim as a JSON string (the Python side already has
/// `jcs.canonicalize` for canonical bytes, but for ergonomic
/// signing we accept a plain JSON that we'll deserialize here).
/// Returns the signed [`ClaimEntry`] serialized as JSON for
/// write-back into the tasks doc under `claim:<task_id>`.
#[pyfunction]
fn sign_claim(claim_json: &str, secret: &Bound<'_, PyBytes>) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let claim: Claim = serde_json::from_str(claim_json)
        .map_err(|e| PyValueError::new_err(format!("bad claim json: {e}")))?;
    let entry = ClaimEntry::sign(claim, &kp).map_err(|e| py_err("sign_claim", e))?;
    serde_json::to_string(&entry).map_err(|e| py_err("serialize", e))
}

/// Verify a [`ClaimEntry`] JSON blob as produced by
/// [`sign_claim`]. Raises `RuntimeError` on any failure
/// (attribution mismatch, tampered claim, wrong signer, bad
/// bytes).
#[pyfunction]
fn verify_claim_entry(entry_json: &str) -> PyResult<()> {
    let entry: ClaimEntry = serde_json::from_str(entry_json)
        .map_err(|e| PyValueError::new_err(format!("bad entry json: {e}")))?;
    entry
        .verify_signature()
        .map_err(|e| py_err("verify_claim_entry", e))
}

/// Sign a curator list JSON blob and return the signed
/// [`CuratorListEntry`] as JSON.
///
/// Sprint 7 Phase B. Consumed by the Python SDK so coordinators
/// and curators can mint lists without touching Rust, and
/// consumed by the shell-daemon Phase C pipeline to verify lists
/// that arrive over gossip.
///
/// Takes the list as a JSON string (the canonical bytes are
/// recomputed Rust-side via RFC 8785 JCS; callers do not need to
/// pre-canonicalize). Returns the signed entry as JSON. Raises
/// `ValueError` on bad input or `RuntimeError` on a signing
/// failure (mismatched pubkey in payload, oversized entries).
#[pyfunction]
fn sign_curator_list(list_json: &str, secret: &Bound<'_, PyBytes>) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let list: CuratorList = serde_json::from_str(list_json)
        .map_err(|e| PyValueError::new_err(format!("bad curator list json: {e}")))?;
    let entry = CuratorListEntry::sign(list, &kp).map_err(|e| py_err("sign_curator_list", e))?;
    serde_json::to_string(&entry).map_err(|e| py_err("serialize", e))
}

/// Verify a [`CuratorListEntry`] JSON blob as produced by
/// [`sign_curator_list`] or by a Rust signer. Raises
/// `RuntimeError` on any failure (version mismatch, oversized
/// entries, attribution split-brain, tampered payload, wrong
/// signer, bad bytes).
#[pyfunction]
fn verify_curator_list_entry(entry_json: &str) -> PyResult<()> {
    let entry: CuratorListEntry = serde_json::from_str(entry_json)
        .map_err(|e| PyValueError::new_err(format!("bad entry json: {e}")))?;
    entry
        .verify_signature()
        .map_err(|e| py_err("verify_curator_list_entry", e))
}

// ======================================================================
// Invite v2 mint/decode
// ======================================================================

fn scope_from_str(s: &str) -> PyResult<RsInviteScope> {
    match s {
        "worker" => Ok(RsInviteScope::Worker),
        "observer" => Ok(RsInviteScope::Observer),
        other => Err(PyValueError::new_err(format!(
            "invite scope must be 'worker' or 'observer', got {other:?}"
        ))),
    }
}

fn scope_to_str(s: RsInviteScope) -> &'static str {
    match s {
        RsInviteScope::Worker => "worker",
        RsInviteScope::Observer => "observer",
    }
}

/// Mint a new Sprint 4 Phase C invite (version 2) and return its
/// `nx1...` wire form.
///
/// Parameters:
///
/// - `coord_secret`: 32-byte Ed25519 secret of the coordinator.
/// - `project_id`: opaque project identifier (the iroh-docs
///   NamespaceId as a hex string, or any stable token the
///   coordinator uses).
/// - `project_name`: human-readable project name for display.
/// - `coordinator_addr`: optional compact EndpointAddr string
///   that seeds the worker's memory_lookup so first contact
///   bypasses pkarr discovery.
/// - `tasks_doc_ticket`: serialized iroh-docs write ticket. MUST
///   be provided when `scope == "worker"` — the mint function
///   raises `ValueError` otherwise.
/// - `scope`: `"worker"` or `"observer"`.
/// - `expires_at_unix`: token expiry as a Unix timestamp.
#[pyfunction]
#[pyo3(signature = (
    coord_secret,
    project_id,
    project_name,
    coordinator_addr,
    tasks_doc_ticket,
    scope,
    expires_at_unix,
))]
#[allow(clippy::too_many_arguments)]
fn mint_invite(
    coord_secret: &Bound<'_, PyBytes>,
    project_id: String,
    project_name: String,
    coordinator_addr: Option<String>,
    tasks_doc_ticket: Option<String>,
    scope: String,
    expires_at_unix: u64,
) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(coord_secret, "coord_secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let rs_scope = scope_from_str(&scope)?;
    let invite = RsInvite::mint(
        &kp,
        project_id,
        project_name,
        coordinator_addr,
        tasks_doc_ticket,
        rs_scope,
        expires_at_unix,
    )
    .map_err(|e| match e {
        RsInviteError::MissingTasksDocTicket => {
            PyValueError::new_err("tasks_doc_ticket is required when scope == 'worker'")
        }
        other => PyRuntimeError::new_err(format!("mint_invite: {other}")),
    })?;
    Ok(invite.encode())
}

/// Decode an `nx1...` invite, verify the signature, check the
/// expiry (using ``now_unix`` if provided, else the system
/// clock), and return its fields as a dict.
///
/// Raises `ValueError` on malformed / expired / signature-invalid
/// invites so the Python caller can translate to a
/// ``HTTPException`` at the API boundary.
#[pyfunction]
#[pyo3(signature = (wire, now_unix=None))]
fn decode_invite<'py>(
    py: Python<'py>,
    wire: &str,
    now_unix: Option<u64>,
) -> PyResult<Bound<'py, PyDict>> {
    let invite =
        RsInvite::decode(wire).map_err(|e| PyValueError::new_err(format!("decode_invite: {e}")))?;
    let now = now_unix.unwrap_or_else(current_unix_secs);
    invite
        .ensure_not_expired(now)
        .map_err(|e| PyValueError::new_err(format!("decode_invite: {e}")))?;

    let d = PyDict::new(py);
    d.set_item("version", invite.payload.version)?;
    d.set_item("project_id", invite.payload.project_id.clone())?;
    d.set_item("project_name", invite.payload.project_name.clone())?;
    d.set_item(
        "coordinator_pubkey",
        PyBytes::new(py, &invite.payload.coordinator_pubkey),
    )?;
    d.set_item("coordinator_addr", invite.payload.coordinator_addr.clone())?;
    d.set_item("tasks_doc_ticket", invite.payload.tasks_doc_ticket.clone())?;
    d.set_item("scope", scope_to_str(invite.payload.scope))?;
    d.set_item("expires_at_unix", invite.payload.expires_at_unix)?;
    Ok(d)
}

// ======================================================================
// Module entry point
// ======================================================================

#[pymodule]
fn nexus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<PyNode>()?;
    m.add_class::<PyDoc>()?;
    m.add_class::<PyDocSubscription>()?;
    m.add_class::<PyGossip>()?;
    m.add_class::<PyBlobs>()?;
    m.add_class::<PyVerifier>()?;

    m.add_function(wrap_pyfunction!(create_node, m)?)?;
    m.add_function(wrap_pyfunction!(create_node_with_secret, m)?)?;
    m.add_function(wrap_pyfunction!(generate_secret, m)?)?;
    m.add_function(wrap_pyfunction!(load_or_generate_secret, m)?)?;
    m.add_function(wrap_pyfunction!(sign_task, m)?)?;
    m.add_function(wrap_pyfunction!(verify_task_entry, m)?)?;
    m.add_function(wrap_pyfunction!(sign_result, m)?)?;
    m.add_function(wrap_pyfunction!(verify_result_entry, m)?)?;
    m.add_function(wrap_pyfunction!(sign_claim, m)?)?;
    m.add_function(wrap_pyfunction!(verify_claim_entry, m)?)?;
    m.add_function(wrap_pyfunction!(sign_curator_list, m)?)?;
    m.add_function(wrap_pyfunction!(verify_curator_list_entry, m)?)?;
    m.add_function(wrap_pyfunction!(mint_invite, m)?)?;
    m.add_function(wrap_pyfunction!(decode_invite, m)?)?;

    Ok(())
}
