//! # nexus-core-py
//!
//! PyO3 bindings that expose [`nexus_core_rs`] as a Python extension
//! module named `nexus_core`. Shipped as a wheel via maturin.
//!
//! ## Sprint 1 scope
//!
//! Exposes exactly one function — `create_node()` — as an awaitable
//! Python coroutine that returns a [`PyNode`] handle. Sprint 2 will
//! broaden the surface to cover `Doc`, `Gossip`, `Blobs`, discovery
//! and verification.
//!
//! ## Example (Python)
//!
//! ```python
//! import asyncio
//! import nexus_core
//!
//! async def main():
//!     node = await nexus_core.create_node()
//!     print("node id:", node.node_id)
//!     await node.shutdown()
//!
//! asyncio.run(main())
//! ```
//!
//! ## Async bridge
//!
//! The function uses `pyo3-asyncio` with the tokio runtime to turn
//! the `async fn` in `nexus-core-rs` into a Python awaitable. The
//! tokio runtime is initialized lazily the first time a coroutine is
//! awaited.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use tokio::sync::Mutex;

use nexus_core_rs::{create_node as rs_create_node, Node};

/// Python handle to a running iroh node.
///
/// Wraps [`nexus_core_rs::Node`] behind an `Arc<Mutex<Option<Node>>>`
/// so that `shutdown()` can move the inner node out while still
/// letting Python hold the handle across await points.
#[pyclass(name = "Node", module = "nexus_core")]
pub struct PyNode {
    inner: Arc<Mutex<Option<Node>>>,
    cached_node_id: String,
}

#[pymethods]
impl PyNode {
    /// Stable textual form of this node's Ed25519 public key.
    ///
    /// Read-only property: the key is assigned once at creation and
    /// never mutates for the life of the node.
    #[getter]
    fn node_id(&self) -> &str {
        &self.cached_node_id
    }

    /// Graceful shutdown. Returns a Python awaitable.
    ///
    /// After `await node.shutdown()`, the node handle is consumed.
    /// Further method calls on the same `PyNode` will raise
    /// `RuntimeError("node already shut down")`.
    fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut guard = inner.lock().await;
            let node = guard
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("node already shut down"))?;
            node.shutdown()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("shutdown failed: {e}")))?;
            Ok(())
        })
    }

    fn __repr__(&self) -> String {
        format!("<nexus_core.Node id={}>", &self.cached_node_id)
    }
}

/// Boot a fresh anonymous iroh node.
///
/// Returns an awaitable that resolves to a [`PyNode`] handle.
/// Raises `RuntimeError` if the underlying iroh endpoint fails to
/// bind (e.g. because the machine is offline or the UDP socket is
/// blocked).
#[pyfunction]
fn create_node(py: Python<'_>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let node = rs_create_node()
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("create_node failed: {e}")))?;
        let cached_node_id = node.node_id();
        Ok(PyNode {
            inner: Arc::new(Mutex::new(Some(node))),
            cached_node_id,
        })
    })
}

/// Python module entry point.
///
/// maturin calls this during wheel initialization. Adds every
/// public-facing function and class to the `nexus_core` module.
#[pymodule]
fn nexus_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_node, m)?)?;
    m.add_class::<PyNode>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
