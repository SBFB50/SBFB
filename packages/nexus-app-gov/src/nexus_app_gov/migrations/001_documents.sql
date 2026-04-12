-- Sprint 9 Phase D — gov documents table for file metadata.
--
-- This table lives in the per-app writable SQLite
-- (projects/<p>/apps/gov/app.sqlite), NOT in the legacy
-- govdata.db which remains read-only. The migration runner
-- applies this automatically on coordinator boot.

CREATE TABLE IF NOT EXISTS gov_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    filename TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT '',
    size_bytes INTEGER NOT NULL DEFAULT 0,
    uploaded_by TEXT NOT NULL DEFAULT '',
    uploaded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_gov_documents_sha256
    ON gov_documents (sha256);
