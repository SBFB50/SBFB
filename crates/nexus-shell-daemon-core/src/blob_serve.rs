// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 12 Phase A — blob-serve cache for archived web apps.
//!
//! The daemon serves web app archives (zip files stored as iroh
//! blobs) to sandboxed iframes via `GET /blob-serve/{hash}/{path}`.
//! This module handles zip decompression, path validation, content
//! type detection, and an LRU-style in-memory cache of decompressed
//! archives.
//!
//! ## Security model
//!
//! Archives come from untrusted publishers on the P2P network.
//! Defence layers:
//!
//! - **Path traversal rejection** — `validate_zip_path` refuses
//!   `..`, absolute paths, and backslash escapes.
//! - **Decompressed size limit** — `load` rejects archives whose
//!   total decompressed size exceeds `max_decompressed_bytes`
//!   (default 100 MB), mitigating zip bombs.
//! - **CSP headers** — the HTTP handler (in the binary crate)
//!   injects `Content-Security-Policy: connect-src 'none'` on
//!   every response so scripts inside the iframe cannot make
//!   outbound network requests.

use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

use dashmap::DashMap;

/// Default maximum decompressed archive size (100 MB).
pub const DEFAULT_MAX_DECOMPRESSED_BYTES: usize = 100 * 1024 * 1024;

/// Default maximum number of cached archives.
pub const DEFAULT_MAX_CACHE_ENTRIES: usize = 32;

// =================================================================
// Cache
// =================================================================

/// In-memory cache of decompressed zip archives.
///
/// Each entry maps a blob hash (hex) to the full set of extracted
/// files (path → bytes). The cache evicts the oldest entry when
/// `max_entries` is exceeded — "oldest" is approximated by
/// insertion order tracked via a separate `DashMap<hash, instant>`.
#[derive(Debug)]
pub struct BlobServeCache {
    /// Decompressed file maps, keyed by blob hash hex.
    entries: DashMap<String, Arc<HashMap<String, Vec<u8>>>>,
    /// Insertion timestamps for LRU eviction.
    insertion_order: DashMap<String, std::time::Instant>,
    max_entries: usize,
}

/// Errors returned by [`BlobServeCache::load`].
#[derive(Debug, thiserror::Error)]
pub enum BlobServeError {
    #[error("invalid zip archive: {0}")]
    InvalidZip(#[from] zip::result::ZipError),
    #[error("decompressed size {actual} exceeds limit {limit}")]
    TooLarge { actual: usize, limit: usize },
    #[error("unsafe path in archive: {0}")]
    UnsafePath(String),
    #[error("I/O error reading archive entry: {0}")]
    Io(#[from] std::io::Error),
}

impl BlobServeCache {
    /// Create a new cache with the given maximum entry count.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: DashMap::new(),
            insertion_order: DashMap::new(),
            max_entries,
        }
    }

    /// Returns `true` if the cache already holds the given hash.
    pub fn has(&self, hash: &str) -> bool {
        self.entries.contains_key(hash)
    }

    /// Decompress a zip archive and cache the result.
    ///
    /// Returns `Err` if the bytes are not a valid zip, if any
    /// entry path fails validation, or if the total decompressed
    /// size exceeds `max_decompressed_bytes`.
    pub fn load(
        &self,
        hash: &str,
        zip_bytes: &[u8],
        max_decompressed_bytes: usize,
    ) -> Result<(), BlobServeError> {
        let cursor = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;

        let mut files: HashMap<String, Vec<u8>> = HashMap::new();
        let mut total_size: usize = 0;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();

            // Skip directories.
            if entry.is_dir() {
                continue;
            }

            if !validate_zip_path(&name) {
                return Err(BlobServeError::UnsafePath(name));
            }

            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;

            total_size += buf.len();
            if total_size > max_decompressed_bytes {
                return Err(BlobServeError::TooLarge {
                    actual: total_size,
                    limit: max_decompressed_bytes,
                });
            }

            files.insert(name, buf);
        }

        self.evict_if_needed();
        self.entries.insert(hash.to_string(), Arc::new(files));
        self.insertion_order
            .insert(hash.to_string(), std::time::Instant::now());
        Ok(())
    }

    /// Retrieve a single file from a cached archive.
    ///
    /// Returns `None` if the hash is not cached or the path does
    /// not exist within the archive.
    pub fn get_file(&self, hash: &str, path: &str) -> Option<Vec<u8>> {
        let entry = self.entries.get(hash)?;
        entry.get(path).cloned()
    }

    /// Evict the oldest entry if the cache is at capacity.
    fn evict_if_needed(&self) {
        while self.entries.len() >= self.max_entries {
            // Find the oldest entry by insertion time.
            let oldest = self
                .insertion_order
                .iter()
                .min_by_key(|e| *e.value())
                .map(|e| e.key().clone());
            if let Some(key) = oldest {
                self.entries.remove(&key);
                self.insertion_order.remove(&key);
            } else {
                break;
            }
        }
    }
}

// =================================================================
// Path validation
// =================================================================

/// Validate that a path extracted from a zip archive is safe.
///
/// Rejects:
/// - Paths containing `..` (directory traversal)
/// - Absolute paths (starting with `/`)
/// - Backslash paths (Windows traversal via `\`)
/// - Empty paths
pub fn validate_zip_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    if path.contains("..") {
        return false;
    }
    if path.contains('\\') {
        return false;
    }
    true
}

// =================================================================
// Content-type detection
// =================================================================

/// Detect the HTTP `Content-Type` for a file based on its name
/// extension and (optionally) magic bytes.
///
/// Returns a `&'static str` suitable for the `Content-Type` header.
/// Falls back to `application/octet-stream` for unknown types.
pub fn detect_content_type(filename: &str, data: &[u8]) -> &'static str {
    // Extension-based detection first.
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_ascii_lowercase().as_str() {
            "html" | "htm" => return "text/html; charset=utf-8",
            "js" | "mjs" => return "text/javascript; charset=utf-8",
            "css" => return "text/css; charset=utf-8",
            "json" => return "application/json; charset=utf-8",
            "svg" => return "image/svg+xml",
            "png" => return "image/png",
            "jpg" | "jpeg" => return "image/jpeg",
            "gif" => return "image/gif",
            "webp" => return "image/webp",
            "ico" => return "image/x-icon",
            "woff" => return "font/woff",
            "woff2" => return "font/woff2",
            "ttf" => return "font/ttf",
            "otf" => return "font/otf",
            "wasm" => return "application/wasm",
            "xml" => return "application/xml",
            "txt" => return "text/plain; charset=utf-8",
            "map" => return "application/json",
            _ => {}
        }
    }

    // Magic bytes fallback.
    if data.len() >= 8 {
        if data.starts_with(&[0x89, b'P', b'N', b'G']) {
            return "image/png";
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "image/jpeg";
        }
        if data.starts_with(b"GIF8") {
            return "image/gif";
        }
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
            return "image/webp";
        }
        if data.starts_with(b"\x00asm") {
            return "application/wasm";
        }
    }

    // Check for HTML-like content.
    let header = &data[..data.len().min(50)];
    let trimmed = std::str::from_utf8(header)
        .unwrap_or("")
        .trim_start()
        .to_ascii_lowercase();
    if trimmed.starts_with("<!doctype") || trimmed.starts_with("<html") {
        return "text/html; charset=utf-8";
    }

    "application/octet-stream"
}

// =================================================================
// CSP header value
// =================================================================

/// The Content-Security-Policy header injected on every blob-serve
/// response. Blocks all outbound network requests from scripts
/// running inside the sandboxed iframe.
pub const BLOB_SERVE_CSP: &str =
    "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none'; frame-ancestors *";

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal zip archive in memory.
    fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    // ---------------------------------------------------------
    // Path validation
    // ---------------------------------------------------------

    #[test]
    fn validate_rejects_path_traversal() {
        assert!(!validate_zip_path("../etc/passwd"));
        assert!(!validate_zip_path("foo/../../bar"));
        assert!(!validate_zip_path(".."));
    }

    #[test]
    fn validate_rejects_absolute_path() {
        assert!(!validate_zip_path("/etc/passwd"));
    }

    #[test]
    fn validate_rejects_backslash() {
        assert!(!validate_zip_path("foo\\bar.txt"));
        assert!(!validate_zip_path("\\start"));
    }

    #[test]
    fn validate_rejects_empty() {
        assert!(!validate_zip_path(""));
    }

    #[test]
    fn validate_accepts_normal_paths() {
        assert!(validate_zip_path("index.html"));
        assert!(validate_zip_path("assets/main.js"));
        assert!(validate_zip_path("deep/nested/file.css"));
    }

    // ---------------------------------------------------------
    // Content-type detection
    // ---------------------------------------------------------

    #[test]
    fn detect_html_by_extension() {
        assert_eq!(
            detect_content_type("index.html", b""),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            detect_content_type("page.htm", b""),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn detect_js_by_extension() {
        assert_eq!(
            detect_content_type("main.js", b""),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            detect_content_type("module.mjs", b""),
            "text/javascript; charset=utf-8"
        );
    }

    #[test]
    fn detect_png_by_magic_bytes() {
        let png_header = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_content_type("unknown", &png_header), "image/png");
    }

    #[test]
    fn detect_wasm_by_extension() {
        assert_eq!(detect_content_type("module.wasm", b""), "application/wasm");
    }

    #[test]
    fn detect_fallback_octet_stream() {
        assert_eq!(
            detect_content_type("mystery.xyz", b"\x00\x01\x02"),
            "application/octet-stream"
        );
    }

    // ---------------------------------------------------------
    // Cache: load + get
    // ---------------------------------------------------------

    #[test]
    fn load_valid_zip_and_retrieve_file() {
        let cache = BlobServeCache::new(8);
        let zip_bytes = make_zip(&[
            ("index.html", b"<h1>Hello</h1>"),
            ("assets/main.js", b"console.log('ok')"),
        ]);
        cache
            .load("abc123", &zip_bytes, DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap();

        assert!(cache.has("abc123"));
        assert_eq!(
            cache.get_file("abc123", "index.html").unwrap(),
            b"<h1>Hello</h1>"
        );
        assert_eq!(
            cache.get_file("abc123", "assets/main.js").unwrap(),
            b"console.log('ok')"
        );
        assert!(cache.get_file("abc123", "nonexistent").is_none());
    }

    #[test]
    fn load_rejects_invalid_zip() {
        let cache = BlobServeCache::new(8);
        let err = cache
            .load("bad", b"not a zip", DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap_err();
        assert!(matches!(err, BlobServeError::InvalidZip(_)));
    }

    #[test]
    fn load_rejects_oversized_archive() {
        let cache = BlobServeCache::new(8);
        // Create a zip with content exceeding a tiny limit.
        let zip_bytes = make_zip(&[("big.txt", &[0x41; 1024])]);
        let err = cache.load("big", &zip_bytes, 512).unwrap_err();
        assert!(matches!(err, BlobServeError::TooLarge { .. }));
    }

    #[test]
    fn load_rejects_path_traversal_in_zip() {
        // Manually construct a zip with a traversal path.
        use std::io::Write;
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file("../escape.txt", options).unwrap();
        writer.write_all(b"pwned").unwrap();
        let zip_bytes = writer.finish().unwrap().into_inner();

        let cache = BlobServeCache::new(8);
        let err = cache
            .load("traversal", &zip_bytes, DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap_err();
        assert!(matches!(err, BlobServeError::UnsafePath(_)));
    }

    #[test]
    fn get_file_returns_none_for_unknown_hash() {
        let cache = BlobServeCache::new(8);
        assert!(cache.get_file("unknown", "index.html").is_none());
    }

    // ---------------------------------------------------------
    // Cache: eviction
    // ---------------------------------------------------------

    #[test]
    fn cache_evicts_oldest_when_full() {
        let cache = BlobServeCache::new(2);
        let zip1 = make_zip(&[("a.txt", b"aaa")]);
        let zip2 = make_zip(&[("b.txt", b"bbb")]);
        let zip3 = make_zip(&[("c.txt", b"ccc")]);

        cache
            .load("h1", &zip1, DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap();
        cache
            .load("h2", &zip2, DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap();
        // h1 should be evicted when h3 is loaded.
        cache
            .load("h3", &zip3, DEFAULT_MAX_DECOMPRESSED_BYTES)
            .unwrap();

        assert!(!cache.has("h1"), "oldest entry should be evicted");
        assert!(cache.has("h2"));
        assert!(cache.has("h3"));
    }
}
