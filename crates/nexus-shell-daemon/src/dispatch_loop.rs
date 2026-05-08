// SPDX-License-Identifier: AGPL-3.0-or-later
//! MPSC dispatch loop — writes TaskEntry values to the project iroh doc.
//!
//! Sprint 49 Phase A (G1 D2 ack): the dispatch loop is the **sole writer**
//! to the project doc. HTTP endpoints submit signed TaskEntry values into
//! the MPSC channel; this loop drains them and writes sequentially.

use std::sync::Arc;

use nexus_core_rs::docs::{DocHandle, DocsAuthorId};
use nexus_core_rs::task::TaskEntry;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};

pub type TaskEntrySender = mpsc::Sender<TaskEntry>;

const CHANNEL_CAPACITY: usize = 64;

pub fn create_dispatch_channel() -> (TaskEntrySender, mpsc::Receiver<TaskEntry>) {
    mpsc::channel(CHANNEL_CAPACITY)
}

pub async fn run(
    mut rx: mpsc::Receiver<TaskEntry>,
    doc: Arc<DocHandle>,
    author: DocsAuthorId,
    shutdown: oneshot::Receiver<()>,
) {
    info!("dispatch_loop started");
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            entry = rx.recv() => {
                let Some(entry) = entry else { break };
                let key = format!("tasks/{}", entry.task.task_id);
                let value = match serde_json::to_vec(&entry) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(task_id = %entry.task.task_id, error = %e, "failed to serialize task entry");
                        continue;
                    }
                };
                if let Err(e) = doc.set(author, key.as_bytes().to_vec(), value).await {
                    warn!(task_id = %entry.task.task_id, error = %e, "failed to write task entry to project doc");
                }
            }
            _ = &mut shutdown => {
                info!("dispatch_loop received shutdown signal");
                break;
            }
        }
    }
    info!("dispatch_loop exiting");
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::task::{TASK_FORMAT_VERSION, Task};

    fn make_test_entry() -> TaskEntry {
        let task = Task {
            version: TASK_FORMAT_VERSION,
            task_id: "test-dispatch-001".into(),
            task_type: "analysis".into(),
            prompt: "test prompt".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            created_at: 1714300000,
            parent_task_id: String::new(),
            metadata: std::collections::BTreeMap::new(),
            is_open_source: false,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
            watermark_seed: Vec::new(),
        };
        let kp = KeyPair::generate();
        TaskEntry::sign(task, &kp).expect("sign")
    }

    #[tokio::test]
    async fn dispatch_loop_writes_to_doc() {
        let node = nexus_core_rs::create_node().await.expect("boot");
        let docs = nexus_core_rs::docs::DocsClient::new(node.docs());
        let doc = docs.create_doc().await.expect("create doc");
        let author = docs.author_default().await.expect("author");
        let doc = Arc::new(doc);

        let (tx, rx) = create_dispatch_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let doc_clone = Arc::clone(&doc);
        let handle = tokio::spawn(run(rx, doc_clone, author, shutdown_rx));

        let entry = make_test_entry();
        let task_id = entry.task.task_id.clone();
        tx.send(entry).await.expect("send");

        // Yield to let the spawned task process the buffered message,
        // then signal shutdown. The mpsc message is already in the
        // channel buffer so recv() will return it before shutdown fires.
        tokio::task::yield_now().await;
        drop(tx);
        let _ = shutdown_tx.send(());
        handle.await.expect("dispatch loop joins");

        let entries = doc
            .get_many_by_prefix(b"tasks/")
            .await
            .expect("get entries");
        assert_eq!(
            entries.len(),
            1,
            "dispatch loop must write exactly one entry"
        );

        let stored_key = std::str::from_utf8(entries[0].key()).unwrap();
        assert_eq!(stored_key, format!("tasks/{task_id}"));
    }
}
