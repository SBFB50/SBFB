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

use nexus_core_rs::{
    blobs::BlobsClient as RsBlobsClient,
    create_node as rs_create_node, create_node_with_config,
    crypto::{KeyPair, SECRET_KEY_BYTES},
    discovery::DiscoveryClient as RsDiscoveryClient,
    docs::{DocHandle as RsDocHandle, DocsClient as RsDocsClient},
    gossip::{
        GossipClient as RsGossipClient, GossipEvent as RsGossipEvent, TopicHandle as RsTopicHandle,
    },
    task::{ResultEntry, ResultPayload, Task, TaskEntry},
    Node, NodeConfig, VerificationReport, Verifier as RsVerifier,
};

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
fn create_node_with_secret<'py>(
    py: Python<'py>,
    secret: &Bound<'_, PyBytes>,
) -> PyResult<Bound<'py, PyAny>> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let cfg = NodeConfig::default().with_secret_key(sk);
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

// ======================================================================
// Module entry point
// ======================================================================

#[pymodule]
fn nexus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<PyNode>()?;
    m.add_class::<PyDoc>()?;
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

    Ok(())
}
