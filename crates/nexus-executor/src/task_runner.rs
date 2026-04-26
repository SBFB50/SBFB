// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ipc::{TaskExecuteParams, TaskExecuteResult};

// Stub: returns an empty result. The real Ollama/llama.cpp dispatch
// is gated on the executor IPC stabilisation (carry S31). This stub
// cannot execute arbitrary code — it touches no prompt, no model,
// no subprocess, no network.
pub fn execute_task(params: &TaskExecuteParams) -> TaskExecuteResult {
    TaskExecuteResult {
        task_id: params.task_id.clone(),
        output: String::new(),
        output_token_ids: Vec::new(),
        model_used: params.model.clone(),
        duration_ms: 0,
        gpu_vram_peak_mb: 0,
    }
}
