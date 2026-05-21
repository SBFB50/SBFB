// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 68 Phase B — `sbfb-factory preview` subcommand.
//!
//! Zips the project directory, uploads it to the daemon's ephemeral
//! preview store, and prints the blob-serve URL so the developer
//! can test their app in the browser before publishing.

use std::io::Write;
use std::path::Path;

use crate::daemon_client::DaemonConnection;

pub fn run(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(path).canonicalize()?;
    if !project_dir.join("index.html").exists() {
        return Err("project directory must contain an index.html".into());
    }

    let zip_bytes = zip_directory(&project_dir)?;
    let conn = DaemonConnection::discover()?;
    let url = format!("{}/api/v1/preview/load", conn.base_url);

    let resp = conn
        .client()
        .post(&url)
        .header("X-SBFB-Token", &conn.token)
        .header("Host", "127.0.0.1")
        .header("Content-Type", "application/octet-stream")
        .body(zip_bytes)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("daemon returned {status}: {body}").into());
    }

    let json: serde_json::Value = resp.json()?;
    let hash = json["hash"]
        .as_str()
        .ok_or("daemon response missing 'hash' field")?;

    eprintln!("preview loaded: {hash}");
    eprintln!("open: {}/blob-serve/{hash}/index.html", conn.base_url);
    Ok(())
}

fn zip_directory(dir: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut archive = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            let relative = entry_path
                .strip_prefix(dir)
                .unwrap_or(entry_path)
                .to_string_lossy()
                .replace('\\', "/");

            if relative.is_empty() || relative.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                archive.add_directory(&relative, options)?;
            } else {
                archive.start_file(&relative, options)?;
                let content = std::fs::read(entry_path)?;
                archive.write_all(&content)?;
            }
        }
        archive.finish()?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_directory_creates_valid_archive() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("index.html"), "<html></html>").unwrap();
        std::fs::create_dir(tmp.path().join("css")).unwrap();
        std::fs::write(tmp.path().join("css/style.css"), "body {}").unwrap();

        let bytes = zip_directory(tmp.path()).unwrap();
        assert!(!bytes.is_empty());

        let cursor = std::io::Cursor::new(bytes);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        assert!(names.contains(&"index.html".to_string()));
        assert!(names.contains(&"css/style.css".to_string()));
    }

    #[test]
    fn run_rejects_missing_index_html() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("app.js"), "// code").unwrap();
        let err = run(tmp.path().to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("index.html"));
    }
}
