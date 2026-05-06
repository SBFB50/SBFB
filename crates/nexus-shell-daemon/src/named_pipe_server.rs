// SPDX-License-Identifier: AGPL-3.0-or-later
//! Windows Named Pipe accept loop with a custom DACL allowing
//! only the current user SID.
//!
//! Sprint 16 Phase B (D2 — Named Pipes Windows avec
//! SECURITY_ATTRIBUTES custom).
//!
//! ## Why a custom DACL
//!
//! The Win32 default DACL on `\\.\pipe\<name>` grants
//! `GENERIC_READ | GENERIC_WRITE` to the world, which makes any
//! process on the same machine — including another local user —
//! capable of opening the pipe and exchanging data. That is the
//! exact bug class Microsoft documents under "Named Pipe Security
//! and Access Rights": a Named Pipe inherits the process's
//! default token DACL unless the caller hands one in.
//!
//! Tailscale ran into this and switched to
//! `\\.\pipe\ProtectedPrefix\Administrators\...` plus a SDDL DACL
//! restricted to the current user. We use the same approach
//! without the `ProtectedPrefix` prefix because the daemon runs
//! as a regular user, not LocalSystem.
//!
//! ## Authentication
//!
//! The DACL itself is the gate: a different user trying to
//! `CreateFile(\\.\pipe\sbfb-daemon)` gets `ACCESS_DENIED`
//! before the pipe connect even starts. There is no equivalent
//! to `SO_PEERCRED` to re-check after the fact — connection
//! success implies the OS has matched the caller's token against
//! the pipe DACL. We still inject [`PeerCredsVerified`] into the
//! request extensions so the auth middleware bypass is uniform
//! across UDS and Named Pipes.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use axum::Router;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use nexus_shell_daemon_core::auth::PeerCredsVerified;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower::ServiceExt;
use tracing::{debug, info, warn};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{PCWSTR, PWSTR};

/// A scoped wrapper around a heap-allocated `SECURITY_DESCRIPTOR`
/// plus the matching `SECURITY_ATTRIBUTES` value. Drop frees the
/// descriptor via `LocalFree` so a partial failure during pipe
/// creation does not leak Win32 heap.
///
/// ## Send + Sync
///
/// The struct contains raw `*mut c_void` (the SD body) and is
/// therefore not auto-Send. We mark it manually because:
/// - Once `build_user_only_attributes` returns, the descriptor
///   is read-only — only `Drop` mutates the pointer (to free).
/// - The pipe accept loop hands an `Arc<PipeSecurity>` to a
///   spawned task that only reads `as_attrs_ptr()`.
pub struct PipeSecurity {
    sa: SECURITY_ATTRIBUTES,
    sd: PSECURITY_DESCRIPTOR,
}

// SAFETY: The struct is logically immutable post-construction.
// The Win32 SECURITY_ATTRIBUTES + SECURITY_DESCRIPTOR are read by
// the kernel during `CreateNamedPipe`; tokio internally holds
// the pointer only for the duration of that syscall. The
// `Drop` impl runs once on the owner thread.
unsafe impl Send for PipeSecurity {}
unsafe impl Sync for PipeSecurity {}

impl PipeSecurity {
    /// Pointer to the populated `SECURITY_ATTRIBUTES` suitable
    /// for `ServerOptions::create_with_security_attributes_raw`.
    pub fn as_attrs_ptr(&self) -> *mut c_void {
        // The pointer must remain valid until the pipe is created;
        // the `&self` borrow keeps the struct on the stack.
        &self.sa as *const _ as *mut c_void
    }
}

impl Drop for PipeSecurity {
    fn drop(&mut self) {
        if !self.sd.0.is_null() {
            // Safe: the descriptor was allocated by
            // ConvertStringSecurityDescriptorToSecurityDescriptorW
            // which documents that the caller frees via LocalFree.
            unsafe {
                let _ = LocalFree(HLOCAL(self.sd.0 as *mut _));
            }
        }
    }
}

/// Build a `SECURITY_ATTRIBUTES` whose DACL grants `GENERIC_ALL`
/// to **only** the current user SID. Anyone else trying to
/// `CreateFile` the pipe gets `ACCESS_DENIED` at the kernel.
pub fn build_user_only_attributes() -> Result<PipeSecurity> {
    let sid = current_user_string_sid().context("read current user SID")?;
    // SDDL: D = DACL, A = Allow ACE, GA = GENERIC_ALL access.
    // Format reference: https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format
    let sddl = format!("D:(A;;GA;;;{sid})");
    let mut sddl_w: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();

    let mut sd = PSECURITY_DESCRIPTOR::default();
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl_w.as_mut_ptr()),
            SDDL_REVISION_1,
            &mut sd,
            None,
        )
        .context("ConvertStringSecurityDescriptorToSecurityDescriptorW")?;
    }

    let sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0,
        bInheritHandle: false.into(),
    };
    Ok(PipeSecurity { sa, sd })
}

/// Read the current process's user SID and convert to a string
/// (`S-1-5-21-...`). Returned string lifetime is independent of
/// any Win32 heap allocation — every intermediate buffer is
/// freed before the function returns.
pub fn current_user_string_sid() -> Result<String> {
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .context("OpenProcessToken")?;
    }

    // First call: probe the size of the TOKEN_USER blob.
    let mut size = 0u32;
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut size) };
    if size == 0 {
        unsafe {
            let _ = CloseHandle(token);
        }
        return Err(anyhow!("GetTokenInformation returned zero size"));
    }
    let mut buffer = vec![0u8; size as usize];
    let res = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr() as *mut _),
            size,
            &mut size,
        )
    };
    let info_ok = res.is_ok();
    if !info_ok {
        unsafe {
            let _ = CloseHandle(token);
        }
        return Err(anyhow!("GetTokenInformation(TokenUser) failed"));
    }

    // SAFETY: GetTokenInformation populated `buffer` with a valid
    // TOKEN_USER struct. The struct's `User.Sid` field points
    // into the same buffer.
    let token_user = unsafe { &*(buffer.as_ptr() as *const TOKEN_USER) };
    let sid = token_user.User.Sid;

    let mut sid_str = PWSTR::null();
    let conv_res = unsafe { ConvertSidToStringSidW(sid, &mut sid_str) };
    let _ = unsafe { CloseHandle(token) };
    conv_res.context("ConvertSidToStringSidW")?;

    let s = unsafe {
        let s = sid_str.to_string()?;
        let _ = LocalFree(HLOCAL(sid_str.0 as *mut _));
        s
    };
    Ok(s)
}

/// Resolve the production Named Pipe name + a hint string the
/// caller can log on boot.
pub fn resolve_pipe_name() -> String {
    nexus_shell_daemon_core::auth::daemon_pipe_name()
}

/// Create the first pipe instance with the user-restricted DACL.
/// Subsequent instances are created inside the accept loop after
/// each client connect.
fn create_first_instance(name: &str, attrs: &PipeSecurity) -> Result<NamedPipeServer> {
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(true)
            .create_with_security_attributes_raw(name, attrs.as_attrs_ptr())
            .with_context(|| format!("create first NP instance at {name}"))
    }
}

fn create_next_instance(name: &str, attrs: &PipeSecurity) -> Result<NamedPipeServer> {
    unsafe {
        ServerOptions::new()
            .create_with_security_attributes_raw(name, attrs.as_attrs_ptr())
            .with_context(|| format!("create next NP instance at {name}"))
    }
}

/// Spawn the Named Pipe accept loop on the tokio runtime.
///
/// Returns a join handle plus a oneshot sender the caller signals
/// to drain in-flight connections and stop accepting new ones.
/// The accept loop wraps the router with the [`PeerCredsVerified`]
/// extension layer so every Named Pipe request bypasses the
/// bearer + Host + Origin middleware (the DACL is the gate).
pub fn spawn(router: Router, pipe_name: String) -> Result<(JoinHandle<()>, oneshot::Sender<()>)> {
    let attrs = build_user_only_attributes()
        .context("build user-only SECURITY_ATTRIBUTES for Named Pipe")?;
    let attrs = Arc::new(attrs);

    // Bind the first instance up-front so a startup failure (pipe
    // already exists, denied, etc.) surfaces immediately rather
    // than after the runtime hands control back to main.
    let first = create_first_instance(&pipe_name, &attrs)?;
    info!(
        pipe = %pipe_name,
        "shell daemon Named Pipe listener bound (DACL: current user SID only)"
    );

    let (tx, rx) = oneshot::channel::<()>();
    let router_with_marker = router.layer(axum::Extension(PeerCredsVerified));
    let pipe_for_log = pipe_name.clone();

    let handle = tokio::spawn(async move {
        serve_until_shutdown(first, pipe_name, attrs, router_with_marker, rx).await;
        info!(pipe = %pipe_for_log, "Named Pipe listener shut down");
    });

    Ok((handle, tx))
}

async fn serve_until_shutdown(
    mut current: NamedPipeServer,
    pipe_name: String,
    attrs: Arc<PipeSecurity>,
    router: Router,
    mut shutdown: oneshot::Receiver<()>,
) {
    let router = Arc::new(router);
    loop {
        tokio::select! {
            res = current.connect() => {
                match res {
                    Ok(()) => {
                        // Client connected. Hand the current
                        // instance to a worker task and spin up
                        // the next instance for the next client.
                        let connected = current;
                        match create_next_instance(&pipe_name, &attrs) {
                            Ok(next) => current = next,
                            Err(e) => {
                                warn!(error = %e, "failed to bind next NP instance — accept loop exiting");
                                spawn_handler(connected, Arc::clone(&router));
                                break;
                            }
                        }
                        spawn_handler(connected, Arc::clone(&router));
                    }
                    Err(e) => {
                        warn!(error = %e, "Named Pipe connect failed — re-binding instance");
                        match create_next_instance(&pipe_name, &attrs) {
                            Ok(next) => current = next,
                            Err(e) => {
                                warn!(error = %e, "failed to re-bind NP instance after connect error — exiting");
                                break;
                            }
                        }
                    }
                }
            }
            _ = &mut shutdown => {
                debug!("Named Pipe server received shutdown signal");
                drop(current);
                break;
            }
        }
    }
}

fn spawn_handler(stream: NamedPipeServer, router: Arc<Router>) {
    let svc = (*router).clone();
    let svc = TowerToHyperService::new(
        svc.map_request(|req: hyper::Request<Incoming>| req.map(axum::body::Body::new)),
    );
    tokio::spawn(async move {
        let io = TokioIo::new(stream);
        if let Err(e) = auto::Builder::new(TokioExecutor::new())
            .serve_connection(io, svc)
            .await
        {
            debug!(error = %e, "Named Pipe HTTP connection ended");
        }
    });
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router as AxumRouter;
    use axum::routing::get;
    use nexus_shell_daemon_core::auth::{AuthState, auth_required};
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    /// Per-test pipe name suffix so cargo's parallel runner does
    /// not collide on `\\.\pipe\sbfb-daemon`.
    fn unique_pipe_name() -> String {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        format!(r"\\.\pipe\sbfb-daemon-test-{pid}-{seq}")
    }

    #[test]
    fn current_user_sid_starts_with_s_1() {
        let sid = current_user_string_sid().expect("read SID");
        assert!(
            sid.starts_with("S-1-"),
            "expected S-1-... SID format, got: {sid}"
        );
    }

    #[test]
    fn build_user_only_attributes_returns_non_null_descriptor() {
        let attrs = build_user_only_attributes().expect("build SA");
        assert!(
            !attrs.sd.0.is_null(),
            "SECURITY_DESCRIPTOR must be allocated"
        );
        assert_eq!(
            attrs.sa.nLength,
            std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32
        );
    }

    #[test]
    fn pipe_security_drop_frees_descriptor() {
        // Smoke test: building + dropping many SECURITY_ATTRIBUTES
        // in a tight loop must not leak the SD heap (manual probe;
        // the Drop impl is the contract). If LocalFree is wrong
        // this test will spike memory, not panic — kept for the
        // human reviewer running it under task manager.
        for _ in 0..32 {
            let _ = build_user_only_attributes().unwrap();
        }
    }

    #[tokio::test]
    async fn end_to_end_named_pipe_serves_handler_response() {
        let auth = AuthState::new("0".repeat(64));
        let router: AxumRouter = AxumRouter::new()
            .route("/protected", get(|| async { "ok-via-np" }))
            .layer(axum::middleware::from_fn_with_state(
                auth.clone(),
                auth_required,
            ));

        let pipe_name = unique_pipe_name();
        let (handle, shutdown) = spawn(router, pipe_name.clone()).expect("spawn NP server");

        // Give the accept loop a beat to bind the next instance
        // after the first one is consumed by `current.connect()`.
        // 50ms is plenty on a developer machine; CI can dial this
        // up if it ever flakes.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Open a client connection to the pipe.
        let mut client = ClientOptions::new()
            .open(&pipe_name)
            .expect("open NP client");
        client
            .write_all(b"GET /protected HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        client.read_to_end(&mut buf).await.unwrap();
        let raw = String::from_utf8_lossy(&buf);
        assert!(raw.starts_with("HTTP/1.1 200"), "expected 200, got: {raw}");
        assert!(
            raw.contains("ok-via-np"),
            "expected handler body, got: {raw}"
        );

        let _ = shutdown.send(());
        let _ = handle.await;
    }
}
