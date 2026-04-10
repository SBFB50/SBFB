-- nexus-coordinator persistent state schema, version 1.
--
-- Applied by nexus_coordinator.db.migrations.init_db on every
-- coordinator boot. Fields are deliberately simple: every timestamp
-- is a Unix integer (seconds), every cryptographic blob is stored
-- as raw BLOB.

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

INSERT OR IGNORE INTO schema_version(version) VALUES (1);

-- task_state: dispatcher tracking for every TaskEntry submitted to
-- the project doc.
CREATE TABLE IF NOT EXISTS task_state (
    task_id          TEXT PRIMARY KEY,
    state            TEXT NOT NULL CHECK (state IN (
        'pending', 'claimed', 'completed', 'failed', 'timed_out'
    )),
    task_json        TEXT NOT NULL,
    task_type        TEXT NOT NULL,
    model            TEXT NOT NULL,
    priority         INTEGER NOT NULL,
    submitted_at     INTEGER NOT NULL,
    claimed_by_pubkey BLOB,
    claimed_at       INTEGER,
    completed_at     INTEGER,
    result_hash      BLOB,
    last_error       TEXT
);

CREATE INDEX IF NOT EXISTS task_state_by_state
    ON task_state(state, priority, submitted_at);

-- kudos_ledger: append-only hash-chain of kudos credits.
--
-- For row id = N:
--   prev_hash  = kudos_ledger[N-1].entry_hash, or 32 zero bytes for N=1
--   canonical  = jcs.canonicalize({
--                   worker_pubkey_hex, task_id, tokens,
--                   quality_factor, trust_multiplier, amount,
--                   awarded_at
--                })
--   entry_hash = sha256(prev_hash || DOMAIN_KUDOS_V1 || 0x00 || canonical)
--   entry_sig  = Ed25519.sign(coord_secret, entry_hash)
CREATE TABLE IF NOT EXISTS kudos_ledger (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    worker_pubkey    BLOB NOT NULL,
    task_id          TEXT NOT NULL,
    tokens           INTEGER NOT NULL,
    quality_factor   REAL NOT NULL,
    trust_multiplier REAL NOT NULL,
    amount           REAL NOT NULL,
    awarded_at       INTEGER NOT NULL,
    prev_hash        BLOB NOT NULL,
    entry_hash       BLOB NOT NULL,
    entry_sig        BLOB NOT NULL
);

CREATE INDEX IF NOT EXISTS kudos_by_worker
    ON kudos_ledger(worker_pubkey);
