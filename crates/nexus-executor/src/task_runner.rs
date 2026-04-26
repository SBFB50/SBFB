// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ipc::{TaskExecuteParams, TaskExecuteResult};

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
