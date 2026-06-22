//! Safe wrapper around `gguf_context` for reading GGUF file metadata.
//!
//! Provides metadata-only access to GGUF files without loading tensor data.
//! Useful for inspecting model architecture parameters before loading a model.

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::NonNull;

/// A safe wrapper around `gguf_context`.
///
/// Opens a GGUF file and parses only the metadata header; tensor weights are
/// never loaded into memory (`no_alloc = true`).
#[derive(Debug)]
pub struct GgufContext {
    ctx: NonNull<llama_cpp_sys_2::gguf_context>,
}

impl GgufContext {
    /// Open a GGUF file and parse its metadata header.
    ///
    /// Returns `None` if the path contains a null byte, the file does not
    /// exist, or the file is not a valid GGUF file.
    pub fn from_file(path: &Path) -> Option<Self> {
        let c_path = CString::new(path.to_str()?).ok()?;
        let params = llama_cpp_sys_2::gguf_init_params {
            no_alloc: true,
            ctx: std::ptr::null_mut(),
        };
        let ptr = unsafe { llama_cpp_sys_2::gguf_init_from_file(c_path.as_ptr(), params) };
        Some(Self {
            ctx: NonNull::new(ptr)?,
        })
    }

    /// Total number of key-value pairs in the metadata.
    pub fn n_kv(&self) -> i64 {
        unsafe { llama_cpp_sys_2::gguf_get_n_kv(self.ctx.as_ptr()) }
    }

    /// Find the index of a key by name. Returns `-1` if not found.
    pub fn find_key(&self, key: &str) -> i64 {
        let Ok(c_key) = CString::new(key) else {
            return -1;
        };
        unsafe { llama_cpp_sys_2::gguf_find_key(self.ctx.as_ptr(), c_key.as_ptr()) }
    }

    /// Return the key name at the given index, or `None` if out of range.
    pub fn key_at(&self, idx: i64) -> Option<&str> {
        let ptr = unsafe { llama_cpp_sys_2::gguf_get_key(self.ctx.as_ptr(), idx) };
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Return the value type of the KV pair at `idx`.
    pub fn kv_type(&self, idx: i64) -> llama_cpp_sys_2::gguf_type {
        unsafe { llama_cpp_sys_2::gguf_get_kv_type(self.ctx.as_ptr(), idx) }
    }

    /// Read a `uint32` value. Panics (inside llama.cpp) if the stored type is
    /// not `GGUF_TYPE_UINT32` — check `kv_type` first if unsure.
    pub fn val_u32(&self, idx: i64) -> u32 {
        unsafe { llama_cpp_sys_2::gguf_get_val_u32(self.ctx.as_ptr(), idx) }
    }

    /// Read an `int32` value.
    pub fn val_i32(&self, idx: i64) -> i32 {
        unsafe { llama_cpp_sys_2::gguf_get_val_i32(self.ctx.as_ptr(), idx) }
    }

    /// Read a `uint64` value.
    pub fn val_u64(&self, idx: i64) -> u64 {
        unsafe { llama_cpp_sys_2::gguf_get_val_u64(self.ctx.as_ptr(), idx) }
    }

    /// Read a string value. Returns `None` if the pointer is null or not
    /// valid UTF-8.
    pub fn val_str(&self, idx: i64) -> Option<&str> {
        let ptr = unsafe { llama_cpp_sys_2::gguf_get_val_str(self.ctx.as_ptr(), idx) };
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Total number of tensors described in the file.
    pub fn n_tensors(&self) -> i64 {
        unsafe { llama_cpp_sys_2::gguf_get_n_tensors(self.ctx.as_ptr()) }
    }

    /// Name of the tensor at `tensor_id`, or `None` if the pointer is null or
    /// not valid UTF-8.
    ///
    /// # Aborts
    ///
    /// `tensor_id` MUST be in `0..n_tensors()`. Unlike the `Option`-returning
    /// KV getters, the underlying ggml `gguf_get_tensor_*` functions
    /// `GGML_ASSERT` an in-range id and **abort the process** on an
    /// out-of-range one — they cannot signal the error in-band. Always iterate
    /// `0..n_tensors()` (or otherwise pre-bound the id); never pass an
    /// unvalidated id. This `# Aborts` contract applies equally to
    /// [`Self::tensor_size`] and [`Self::tensor_type`].
    pub fn tensor_name(&self, tensor_id: i64) -> Option<&str> {
        let ptr = unsafe { llama_cpp_sys_2::gguf_get_tensor_name(self.ctx.as_ptr(), tensor_id) };
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// On-disk byte size of the tensor at `tensor_id`, exactly as computed by
    /// ggml (block-quantization aware — Q4_0/Q4_K/Q6_K/F16/... all handled by
    /// the same `ggml_type` traits the runtime uses). This is the authoritative
    /// per-tensor size for a header-only VRAM estimate, so a caller never has
    /// to re-derive a `block_size`/`type_size` table that would drift from the
    /// ggml actually compiled into this build.
    ///
    /// # Aborts
    ///
    /// Out-of-range `tensor_id` aborts the process — see [`Self::tensor_name`].
    pub fn tensor_size(&self, tensor_id: i64) -> u64 {
        let sz = unsafe { llama_cpp_sys_2::gguf_get_tensor_size(self.ctx.as_ptr(), tensor_id) };
        sz as u64
    }

    /// `ggml_type` of the tensor at `tensor_id`.
    ///
    /// # Aborts
    ///
    /// Out-of-range `tensor_id` aborts the process — see [`Self::tensor_name`].
    pub fn tensor_type(&self, tensor_id: i64) -> llama_cpp_sys_2::ggml_type {
        unsafe { llama_cpp_sys_2::gguf_get_tensor_type(self.ctx.as_ptr(), tensor_id) }
    }

    /// Read a string metadata value by key, or `None` if the key is absent or
    /// not stored as a GGUF string. Type-checked before the read so a
    /// wrong-typed value never triggers the in-llama.cpp `gguf_get_val_*` panic.
    /// Lets callers in crates that do not depend on `llama-cpp-sys-2` read typed
    /// metadata without touching the `GGUF_TYPE_*` discriminants directly.
    pub fn meta_str(&self, key: &str) -> Option<String> {
        let idx = self.find_key(key);
        if idx < 0 {
            return None;
        }
        if self.kv_type(idx) != llama_cpp_sys_2::GGUF_TYPE_STRING {
            return None;
        }
        self.val_str(idx).map(str::to_string)
    }

    /// Read an unsigned-integer metadata value by key as `u32`, tolerating the
    /// `UINT32` / `INT32` / `UINT64` encodings a GGUF writer may use, or `None`
    /// if the key is absent, not an integer, or out of `u32` range. Type-checked
    /// before the read (no panic on a wrong-typed value).
    pub fn meta_u32(&self, key: &str) -> Option<u32> {
        let idx = self.find_key(key);
        if idx < 0 {
            return None;
        }
        let ty = self.kv_type(idx);
        if ty == llama_cpp_sys_2::GGUF_TYPE_UINT32 {
            Some(self.val_u32(idx))
        } else if ty == llama_cpp_sys_2::GGUF_TYPE_INT32 {
            u32::try_from(self.val_i32(idx)).ok()
        } else if ty == llama_cpp_sys_2::GGUF_TYPE_UINT64 {
            u32::try_from(self.val_u64(idx)).ok()
        } else {
            None
        }
    }
}

impl Drop for GgufContext {
    fn drop(&mut self) {
        unsafe { llama_cpp_sys_2::gguf_free(self.ctx.as_ptr()) }
    }
}

#[cfg(test)]
mod tests;
