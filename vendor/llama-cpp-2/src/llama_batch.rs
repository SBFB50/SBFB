//! Safe wrapper around `llama_batch`.

use crate::token::LlamaToken;
use llama_cpp_sys_2::{llama_batch, llama_batch_free, llama_batch_init, llama_pos, llama_seq_id};
use std::marker::PhantomData;

/// A safe wrapper around `llama_batch`.
#[derive(Debug)]
pub struct LlamaBatch<'a> {
    /// The number of tokens the batch was allocated with. they are safe to write to - but not necessarily read from as they are not necessarily initialized
    allocated: usize,
    /// The logits that are initialized. Used by [`LlamaContext`] to ensure that only initialized logits are accessed.
    pub(crate) initialized_logits: Vec<i32>,
    #[allow(clippy::doc_markdown)]
    /// The llama_cpp batch. always initialize by `llama_cpp_sys_2::llama_batch_init(allocated, <unknown>, <unknown>)`
    pub(crate) llama_batch: llama_batch,
    /// SBFB S77 fork: the per-row embedding width the `embd` buffer was allocated with by
    /// [`LlamaBatch::new_embeddings`] (0 for token batches). [`LlamaBatch::add_embedding`]
    /// requires it to be non-zero (the batch carries an `embd` buffer, not a `token` one) and
    /// checks every row against it so the unsafe strided write can never go out of bounds.
    embd_n: usize,
    /// SBFB S77 fork: the max sequences per token the batch was allocated for (the `seq_id`
    /// buffer is `n_seq_max` wide per row). [`LlamaBatch::add_embedding`] bounds `seq_ids`
    /// against it so the unsafe per-sequence write can never overflow that row.
    n_seq_max: usize,
    phantom: PhantomData<&'a [LlamaToken]>,
}

/// Errors that can occur when adding a token to a batch.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum BatchAddError {
    /// There was not enough space in the batch to add the token.
    #[error("Insufficient Space of {0}")]
    InsufficientSpace(usize),
    /// Empty buffer is provided for [`LlamaBatch::get_one`]
    #[error("Empty buffer")]
    EmptyBuffer,
    /// SBFB S77 fork: an embedding row of the wrong width was passed to
    /// [`LlamaBatch::add_embedding`] — it must equal the `n_embd` the batch was created with.
    #[error("Embedding width mismatch: batch allocated for {expected}, got {got}")]
    WrongEmbeddingWidth {
        /// The `n_embd` the batch was allocated with.
        expected: usize,
        /// The width of the row that was passed.
        got: usize,
    },
    /// SBFB S77 fork: [`LlamaBatch::add_embedding`] was called on a token batch (one created by
    /// [`LlamaBatch::new`] / [`LlamaBatch::get_one`], whose `embd` buffer is null) instead of one
    /// from [`LlamaBatch::new_embeddings`].
    #[error("add_embedding called on a token batch (use new_embeddings)")]
    NotAnEmbeddingBatch,
    /// SBFB S77 fork: more sequence ids were passed to a batch row than the `n_seq_max` the batch
    /// was allocated for.
    #[error("Too many sequence ids for one row: max {max}, got {got}")]
    TooManySequences {
        /// The `n_seq_max` the batch was allocated for.
        max: usize,
        /// The number of sequence ids passed.
        got: usize,
    },
}

impl<'a> LlamaBatch<'a> {
    /// Clear the batch. This does not free the memory associated with the batch, but it does reset
    /// the number of tokens to 0.
    pub fn clear(&mut self) {
        self.llama_batch.n_tokens = 0;
        self.initialized_logits.clear();
    }

    /// add a token to the batch for sequences `seq_ids` at position `pos`. If `logits` is true, the
    /// token will be initialized and can be read from after the next decode.
    ///
    /// # Panics
    ///
    /// - [`self.llama_batch.n_tokens`] does not fit into a usize
    /// - [`seq_ids.len()`] does not fit into a [`llama_seq_id`]
    ///
    /// # Errors
    ///
    /// returns a error if there is insufficient space in the buffer
    pub fn add(
        &mut self,
        LlamaToken(id): LlamaToken,
        pos: llama_pos,
        seq_ids: &[i32],
        logits: bool,
    ) -> Result<(), BatchAddError> {
        if self.allocated
            < usize::try_from(self.n_tokens() + 1).expect("cannot fit n_tokens into a usize")
        {
            return Err(BatchAddError::InsufficientSpace(self.allocated));
        }
        let offset = self.llama_batch.n_tokens;
        let offset_usize = usize::try_from(offset).expect("cannot fit n_tokens into a usize");
        unsafe {
            // batch.token   [batch.n_tokens] = id;
            self.llama_batch.token.add(offset_usize).write(id);
            // batch.pos     [batch.n_tokens] = pos,
            self.llama_batch.pos.add(offset_usize).write(pos);
            // batch.n_seq_id[batch.n_tokens] = seq_ids.size();
            self.llama_batch.n_seq_id.add(offset_usize).write(
                llama_seq_id::try_from(seq_ids.len())
                    .expect("cannot fit seq_ids.len() into a llama_seq_id"),
            );
            // for (size_t i = 0; i < seq_ids.size(); ++i) {
            //     batch.seq_id[batch.n_tokens][i] = seq_ids[i];
            // }
            for (i, seq_id) in seq_ids.iter().enumerate() {
                let tmp = *self.llama_batch.seq_id.add(offset_usize);
                tmp.add(i).write(*seq_id);
            }
            // batch.logits  [batch.n_tokens] = logits;
            self.llama_batch
                .logits
                .add(offset_usize)
                .write(i8::from(logits));
        }

        if logits {
            self.initialized_logits.push(offset);
        } else {
            self.initialized_logits.retain(|l| l != &offset);
        }

        // batch.n_tokens++;
        self.llama_batch.n_tokens += 1;

        Ok(())
    }

    /// Add a sequence of tokens to the batch for the given sequence id. If `logits_all` is true, the
    /// tokens will be initialized and can be read from after the next decode.
    ///
    /// Either way the last token in the sequence will have its logits set to `true`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is insufficient space in the buffer
    ///
    /// # Panics
    ///
    /// - [`self.llama_batch.n_tokens`] does not fit into a [`usize`]
    /// - [`n_tokens - 1`] does not fit into a [`llama_pos`]
    pub fn add_sequence(
        &mut self,
        tokens: &[LlamaToken],
        seq_id: i32,
        logits_all: bool,
    ) -> Result<(), BatchAddError> {
        let n_tokens_0 =
            usize::try_from(self.llama_batch.n_tokens).expect("cannot fit n_tokens into a usize");
        let n_tokens = tokens.len();

        if self.allocated < n_tokens_0 + n_tokens {
            return Err(BatchAddError::InsufficientSpace(self.allocated));
        }

        let last_index = llama_pos::try_from(n_tokens.saturating_sub(1))
            .expect("cannot fit n_tokens into a llama_pos");
        for (i, token) in (0..).zip(tokens.iter()) {
            self.add(*token, i, &[seq_id], logits_all || i == last_index)?;
        }

        Ok(())
    }

    /// Create a new `LlamaBatch` that can contain up to `n_tokens` tokens.
    ///
    /// # Arguments
    ///
    /// - `n_tokens`: the maximum number of tokens that can be added to the batch
    /// - `n_seq_max`: the maximum number of sequences that can be added to the batch (generally 1 unless you know what you are doing)
    ///
    /// # Panics
    ///
    /// Panics if `n_tokens` is greater than `i32::MAX`.
    #[must_use]
    pub fn new(n_tokens: usize, n_seq_max: i32) -> Self {
        let n_tokens_i32 = i32::try_from(n_tokens).expect("cannot fit n_tokens into a i32");
        let batch = unsafe { llama_batch_init(n_tokens_i32, 0, n_seq_max) };

        LlamaBatch {
            allocated: n_tokens,
            initialized_logits: vec![],
            llama_batch: batch,
            embd_n: 0,
            n_seq_max: n_seq_max.max(0) as usize,
            phantom: PhantomData,
        }
    }

    /// Create a new `LlamaBatch` for injecting raw input embeddings (Sprint 77 SBFB fork
    /// pipeline-split shard path) instead of token ids. The batch allocates `n_tokens`
    /// rows of `n_embd` floats in `llama_batch.embd`; `llama_batch.token` stays null, so
    /// such a batch must be filled with [`LlamaBatch::add_embedding`], never
    /// [`LlamaBatch::add`]. The downstream shard resumes the pipeline from the previous
    /// shard's boundary residual by injecting it here.
    ///
    /// # Panics
    ///
    /// Panics if `n_tokens` or `n_embd` is greater than `i32::MAX`.
    #[must_use]
    pub fn new_embeddings(n_tokens: usize, n_embd: usize, n_seq_max: i32) -> Self {
        let n_tokens_i32 = i32::try_from(n_tokens).expect("cannot fit n_tokens into a i32");
        let n_embd_i32 = i32::try_from(n_embd).expect("cannot fit n_embd into a i32");
        let batch = unsafe { llama_batch_init(n_tokens_i32, n_embd_i32, n_seq_max) };

        LlamaBatch {
            allocated: n_tokens,
            initialized_logits: vec![],
            llama_batch: batch,
            embd_n: n_embd,
            n_seq_max: n_seq_max.max(0) as usize,
            phantom: PhantomData,
        }
    }

    /// Add one row of raw input embeddings (length `n_embd`) at position `pos` for the
    /// given `seq_ids` (Sprint 77 SBFB fork pipeline-split shard path). The batch MUST have
    /// been created with [`LlamaBatch::new_embeddings`] using the same `n_embd` as
    /// `embd.len()`. If `logits` is true the row's hidden state is read back after the next
    /// decode via [`crate::context::LlamaContext::embeddings_ith`].
    ///
    /// # Panics
    ///
    /// - [`self.llama_batch.n_tokens`] does not fit into a usize
    /// - [`seq_ids.len()`] does not fit into a [`llama_seq_id`]
    ///
    /// # Errors
    ///
    /// Returns [`BatchAddError::NotAnEmbeddingBatch`] if the batch is a token batch,
    /// [`BatchAddError::InsufficientSpace`] if the buffer is full,
    /// [`BatchAddError::WrongEmbeddingWidth`] if `embd.len()` does not equal the `n_embd` the
    /// batch was created with via [`LlamaBatch::new_embeddings`], or
    /// [`BatchAddError::TooManySequences`] if `seq_ids` is wider than the batch's `n_seq_max`.
    pub fn add_embedding(
        &mut self,
        embd: &[f32],
        pos: llama_pos,
        seq_ids: &[i32],
        logits: bool,
    ) -> Result<(), BatchAddError> {
        if self.allocated
            < usize::try_from(self.n_tokens() + 1).expect("cannot fit n_tokens into a usize")
        {
            return Err(BatchAddError::InsufficientSpace(self.allocated));
        }
        // SBFB S77 fork: this method is only valid on a batch from `new_embeddings`. A token
        // batch (`new`/`get_one`) has `embd_n == 0` and a null `embd` buffer, so an empty slice
        // would otherwise slip past the width check below and write through the null pointer.
        if self.embd_n == 0 {
            return Err(BatchAddError::NotAnEmbeddingBatch);
        }
        // The strided unsafe write below uses the allocation width as the row stride, so a row of
        // any other width would overflow or mis-stride the `embd` buffer. Reject it instead of
        // trusting the caller's slice length.
        if embd.len() != self.embd_n {
            return Err(BatchAddError::WrongEmbeddingWidth {
                expected: self.embd_n,
                got: embd.len(),
            });
        }
        // The `seq_id` buffer is `n_seq_max` wide per row; bound the per-sequence writes below.
        if seq_ids.len() > self.n_seq_max {
            return Err(BatchAddError::TooManySequences {
                max: self.n_seq_max,
                got: seq_ids.len(),
            });
        }
        let offset = self.llama_batch.n_tokens;
        let offset_usize = usize::try_from(offset).expect("cannot fit n_tokens into a usize");
        let n_embd = self.embd_n;
        unsafe {
            // batch.embd[batch.n_tokens * n_embd ..][.. n_embd] = embd;
            let dst = self.llama_batch.embd.add(offset_usize * n_embd);
            std::ptr::copy_nonoverlapping(embd.as_ptr(), dst, n_embd);
            self.llama_batch.pos.add(offset_usize).write(pos);
            self.llama_batch.n_seq_id.add(offset_usize).write(
                llama_seq_id::try_from(seq_ids.len())
                    .expect("cannot fit seq_ids.len() into a llama_seq_id"),
            );
            for (i, seq_id) in seq_ids.iter().enumerate() {
                let tmp = *self.llama_batch.seq_id.add(offset_usize);
                tmp.add(i).write(*seq_id);
            }
            self.llama_batch
                .logits
                .add(offset_usize)
                .write(i8::from(logits));
        }

        if logits {
            self.initialized_logits.push(offset);
        } else {
            self.initialized_logits.retain(|l| l != &offset);
        }

        self.llama_batch.n_tokens += 1;

        Ok(())
    }

    /// ``llama_batch_get_one``
    /// Return batch for single sequence of tokens
    ///
    /// NOTE: this is a helper function to facilitate transition to the new batch API
    ///
    /// # Errors
    /// If the provided token buffer is empty.
    ///
    /// # Panics
    /// If the number of tokens in ``tokens`` exceeds [`i32::MAX`].
    pub fn get_one(tokens: &'a [LlamaToken]) -> Result<Self, BatchAddError> {
        if tokens.is_empty() {
            return Err(BatchAddError::EmptyBuffer);
        }
        let batch = unsafe {
            let ptr = tokens.as_ptr() as *mut i32;
            llama_cpp_sys_2::llama_batch_get_one(
                ptr,
                tokens
                    .len()
                    .try_into()
                    .expect("number of tokens exceeds i32::MAX"),
            )
        };
        let batch = Self {
            allocated: 0,
            initialized_logits: vec![(tokens.len() - 1)
                .try_into()
                .expect("number of tokens exceeds i32::MAX + 1")],
            llama_batch: batch,
            embd_n: 0,
            // get_one wraps a single sequence; add_embedding is rejected on this token batch
            // (embd_n == 0) before the seq_ids bound is consulted.
            n_seq_max: 1,
            phantom: PhantomData,
        };
        Ok(batch)
    }

    /// Returns the number of tokens in the batch.
    #[must_use]
    pub fn n_tokens(&self) -> i32 {
        self.llama_batch.n_tokens
    }
}

impl<'a> Drop for LlamaBatch<'a> {
    /// Drops the `LlamaBatch`.
    ///
    /// ```
    /// # use llama_cpp_2::llama_batch::LlamaBatch;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let batch = LlamaBatch::new(512, 1);
    /// // frees the memory associated with the batch. (allocated by llama.cpp)
    /// drop(batch);
    /// # Ok(())
    /// # }
    fn drop(&mut self) {
        unsafe {
            if self.allocated > 0 {
                llama_batch_free(self.llama_batch);
            }
        }
    }
}
