// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix Domain Socket accept loop with `SO_PEERCRED` validation.
//!
//! Sprint 16 Phase B (D2 — UDS durcis avec SO_PEERCRED).
//!
//! ## Why a separate listener
//!
//! The daemon's TCP serve task (`runtime::start`) already binds
//! `127.0.0.1:<ephemeral>` so the React shell can talk in via the
//! coordinator proxy. The UDS path is added beside it for two
//! reasons:
//!
//! 1. **Defense in depth** — the file mode `0600` on
//!    `~/.sbfb/run/daemon.sock` and the `0700` mode on the parent
//!    directory mean only the current user can open the socket
//!    in the first place. We then verify `SO_PEERCRED` returns
//!    the same uid as `geteuid()` before serving the connection,
//!    so even a hypothetical filesystem-permission bypass cannot
//!    impersonate a different local user.
//! 2. **Bearer-free CLI path** — the future `sbfb` CLI on Unix
//!    can talk to the daemon without reading the bearer token
//!    file. The accept loop injects the
//!    [`PeerCredsVerified`] marker into the request extensions
//!    so [`auth_required`] bypasses the bearer + Host + Origin
//!    checks for kernel-authenticated peers (mirrors Tailscale
//!    `safesocket.PlatformUsesPeerCreds`).
//!
//! ## Cross-platform shape
//!
//! The whole module is gated on `cfg(unix)`. Windows uses
//! [`crate::named_pipe_server`] instead, which provides the same
//! `PeerCredsVerified` injection through a DACL-restricted Named
//! Pipe.

#![cfg(unix)]

use std::io;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use nexus_shell_daemon_core::auth::PeerCredsVerified;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower::ServiceExt;
use tracing::{debug, info, warn};

/// Resolve the on-disk UDS path used by `serve` from the
/// `SBFB_HOME` env / home dir.
///
/// Exposed (rather than inlined into `serve`) so the daemon
/// runtime can log the path it is about to bind, and so tests
/// can assert that `SBFB_HOME` overrides the production path.
pub fn resolve_socket_path() -> Result<PathBuf> {
    nexus_shell_daemon_core::auth::daemon_socket_path()
        .context("could not resolve ~/.sbfb/run/daemon.sock for this platform")
}

/// Construct the parent run dir at mode `0700`, remove any
/// stale socket file at `path`, bind a fresh `UnixListener`, and
/// chmod the socket to `0600`.
///
/// Caller is responsible for removing the socket file when the
/// listener is dropped — `serve_until_shutdown` handles this on
/// the happy path.
fn bind_socket(path: &Path) -> io::Result<UnixListener> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        set_mode(parent, 0o700)?;
    }
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let listener = UnixListener::bind(path)?;
    set_mode(path, 0o600)?;
    Ok(listener)
}

fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, perms)
}

/// Read the peer uid of a connected `UnixStream` via
/// `getsockopt(SOL_SOCKET, SO_PEERCRED)` (Linux) or `getpeereid`
/// (macOS, *BSD).
///
/// The `unsafe` block is unavoidable: every libc surface for OS
/// peer credentials goes through raw FFI. We minimize the unsafe
/// scope to the single syscall and immediately return a typed
/// `io::Result`.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    let fd = stream.as_raw_fd();
    #[cfg(target_os = "linux")]
    {
        use std::mem;
        let mut cred: libc::ucred = unsafe { mem::zeroed() };
        let mut len = mem::size_of::<libc::ucred>() as libc::socklen_t;
        let ret = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &mut cred as *mut _ as *mut libc::c_void,
                &mut len,
            )
        };
        if ret == 0 {
            Ok(cred.uid)
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    ))]
    {
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        let ret = unsafe { libc::getpeereid(fd, &mut uid, &mut gid) };
        if ret == 0 {
            Ok(uid)
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    )))]
    {
        let _ = fd;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SO_PEERCRED unavailable on this Unix variant",
        ))
    }
}

/// Return the effective uid of the current process via
/// `geteuid`. Wrapped here so callers that mix peer + self uid
/// reads see the same `u32` shape on both sides.
pub fn current_uid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Spawn the UDS accept loop on the tokio runtime and return a
/// join handle + a shutdown sender.
///
/// The router handed in is consumed; clone it before calling if
/// the caller still needs to use the original router elsewhere
/// (e.g. for the TCP serve task). The accept loop wraps the
/// router with the [`PeerCredsVerified`] extension layer so
/// every request that comes in over UDS bypasses the bearer +
/// Host + Origin middleware.
pub fn spawn(
    router: Router,
    socket_path: PathBuf,
) -> Result<(JoinHandle<()>, oneshot::Sender<()>)> {
    let listener = bind_socket(&socket_path)
        .with_context(|| format!("bind UDS at {}", socket_path.display()))?;
    info!(
        path = %socket_path.display(),
        "shell daemon UDS listener bound (mode 0600, parent 0700)"
    );

    let (tx, rx) = oneshot::channel::<()>();
    let router_with_marker = router.layer(axum::Extension(PeerCredsVerified));
    let path_for_cleanup = socket_path.clone();

    let handle = tokio::spawn(async move {
        serve_until_shutdown(listener, router_with_marker, rx).await;
        // Best-effort cleanup of the on-disk socket file. Leaving
        // it behind is harmless on the next boot — `bind_socket`
        // unlinks any pre-existing path before binding — but a
        // tidy shutdown leaves the run dir empty.
        let _ = std::fs::remove_file(&path_for_cleanup);
        info!(path = %path_for_cleanup.display(), "UDS listener shut down");
    });

    Ok((handle, tx))
}

async fn serve_until_shutdown(
    listener: UnixListener,
    router: Router,
    mut shutdown: oneshot::Receiver<()>,
) {
    let our_uid = current_uid();
    // Cloning is cheap — Router is internally Arc-backed.
    let router = Arc::new(router);
    loop {
        tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, _addr)) => handle_connection(stream, Arc::clone(&router), our_uid).await,
                    Err(e) => {
                        warn!(error = %e, "UDS accept failed; continuing");
                    }
                }
            }
            _ = &mut shutdown => {
                debug!("UDS server received shutdown signal");
                break;
            }
        }
    }
}

async fn handle_connection(stream: UnixStream, router: Arc<Router>, our_uid: u32) {
    match peer_uid(&stream) {
        Ok(uid) if uid == our_uid => {
            debug!(peer_uid = uid, "UDS peer creds verified");
        }
        Ok(uid) => {
            warn!(
                peer_uid = uid,
                our_uid, "UDS peer uid mismatch — rejecting connection"
            );
            // Dropping the stream tears down the connection.
            return;
        }
        Err(e) => {
            warn!(error = %e, "SO_PEERCRED failed — rejecting UDS connection");
            return;
        }
    }

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
            debug!(error = %e, "UDS HTTP connection ended");
        }
    });
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_uid_matches_geteuid_syscall() {
        let direct = unsafe { libc::geteuid() };
        assert_eq!(current_uid(), direct);
    }

    #[tokio::test]
    async fn bind_socket_sets_mode_0600_and_parent_0700() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("nested").join("daemon.sock");
        let _listener = bind_socket(&sock).unwrap();
        let mode = std::fs::metadata(&sock).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "socket file must be mode 0600");
        let parent_mode = std::fs::metadata(sock.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(parent_mode, 0o700, "parent dir must be mode 0700");
        // Cleanup so tempdir drop succeeds quietly.
        std::fs::remove_file(&sock).unwrap();
    }

    #[tokio::test]
    async fn bind_socket_replaces_stale_file() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("daemon.sock");
        std::fs::write(&sock, b"stale").unwrap();
        let _listener = bind_socket(&sock).unwrap();
        // The file now is the bound socket, not our stale 5 bytes.
        let meta = std::fs::metadata(&sock).unwrap();
        // Sockets report file_type().is_socket() on Unix.
        use std::os::unix::fs::FileTypeExt;
        assert!(meta.file_type().is_socket());
        std::fs::remove_file(&sock).unwrap();
    }

    #[tokio::test]
    async fn peer_uid_matches_self_for_local_connection() {
        // Bind a temporary UDS, connect to it from the same process,
        // and read the peer uid on the server side. Must equal our
        // own uid.
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let listener = bind_socket(&sock_path).unwrap();

        let client_handle =
            tokio::spawn(async move { UnixStream::connect(&sock_path).await.unwrap() });

        let (server_stream, _) = listener.accept().await.unwrap();
        let _client_stream = client_handle.await.unwrap();

        let uid = peer_uid(&server_stream).unwrap();
        assert_eq!(uid, current_uid());
    }

    #[tokio::test]
    async fn end_to_end_serve_returns_handler_response() {
        // Spin up a tiny axum router on a UDS, send a raw HTTP/1
        // GET, and check the body. Proves the hyper-util glue is
        // wired correctly and that the PeerCredsVerified marker
        // makes the auth middleware skip the bearer check.
        use axum::routing::get;
        use axum::Router as AxumRouter;
        use nexus_shell_daemon_core::auth::{auth_required, AuthState};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let auth = AuthState::new("0".repeat(64));
        let router: AxumRouter = AxumRouter::new()
            .route("/protected", get(|| async { "ok-via-uds" }))
            .layer(axum::middleware::from_fn_with_state(
                auth.clone(),
                auth_required,
            ));

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let (handle, shutdown) = spawn(router, sock_path.clone()).unwrap();

        // Drive a hand-rolled HTTP/1 GET — no token header at all.
        let mut client = UnixStream::connect(&sock_path).await.unwrap();
        client
            .write_all(b"GET /protected HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        client.read_to_end(&mut buf).await.unwrap();
        let raw = String::from_utf8_lossy(&buf);
        assert!(raw.starts_with("HTTP/1.1 200"), "expected 200, got: {raw}");
        assert!(
            raw.contains("ok-via-uds"),
            "expected handler body, got: {raw}"
        );

        let _ = shutdown.send(());
        let _ = handle.await;
    }
}
