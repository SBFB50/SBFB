// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal cross-machine shard driver for LIVE acceptance (NOT a product binary).
//!
//! Runs ONE shard of a layer-block split and pipes the boundary hidden state as
//! raw little-endian f32 on stdin/stdout, so a head shard on machine A can hand
//! off to a tail shard on machine B over a plain SSH pipe — proving the split
//! across two PHYSICAL machines produces the same result as the whole model.
//!
//! Build (per machine): cargo run --release --example shard_node \
//!   --features llm_llama_cpp_cuda   (Windows/5080)  |  llm_llama_cpp_metal (Mac)
//!
//! Modes:
//!   whole <gguf> <prompt>              -> full forward, f32 LE to stdout
//!   head  <gguf> <start> <end> <prompt>-> forward [start,end), boundary to stdout
//!   tail  <gguf> <start> <end>         -> read boundary from stdin, forward to stdout
use nexus_worker_core::llm::shard::ShardBackend;
use std::io::{Read, Write};

fn read_f32_stdin() -> Vec<f32> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).expect("read stdin");
    buf.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn write_f32_stdout(v: &[f32]) {
    let mut out = std::io::stdout().lock();
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for x in v {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    out.write_all(&bytes).expect("write stdout");
    out.flush().ok();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: shard_node whole|head|tail <gguf> ...");
        std::process::exit(2);
    }
    let mode = a[1].as_str();
    let gguf = &a[2];
    // GPU offload count for this shard, via env var. 0 = CPU only (default, keeps the
    // original CPU proof reproducible), a large value (e.g. 999) offloads every layer
    // this shard loaded onto the GPU/Metal backend the binary was built with.
    let n_gpu_layers: u32 = std::env::var("N_GPU_LAYERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    match mode {
        "whole" => {
            let prompt = &a[3];
            let b =
                ShardBackend::load(gguf, 0, 0, true, true, n_gpu_layers, 512).expect("load whole");
            let toks = b.tokenize(prompt).expect("tokenize");
            eprintln!(
                "[shard_node] WHOLE n_embd={} tokens={}",
                b.n_embd(),
                toks.len()
            );
            write_f32_stdout(&b.forward_tokens(&toks).expect("full forward"));
        }
        "head" => {
            let start: u32 = a[3].parse().expect("start");
            let end: u32 = a[4].parse().expect("end");
            let prompt = &a[5];
            let b = ShardBackend::load(gguf, start, end, true, false, n_gpu_layers, 512)
                .expect("load head");
            let toks = b.tokenize(prompt).expect("tokenize");
            eprintln!(
                "[shard_node] HEAD [{},{}) is_first={} n_embd={} tokens={}",
                b.window().start(),
                b.window().end(),
                b.window().is_first(),
                b.n_embd(),
                toks.len()
            );
            write_f32_stdout(&b.forward_tokens(&toks).expect("head forward"));
        }
        "tail" => {
            let start: u32 = a[3].parse().expect("start");
            let end: u32 = a[4].parse().expect("end");
            let b = ShardBackend::load(gguf, start, end, false, true, n_gpu_layers, 512)
                .expect("load tail");
            let boundary = read_f32_stdin();
            eprintln!(
                "[shard_node] TAIL [{},{}) is_last={} n_embd={} boundary_floats={}",
                b.window().start(),
                b.window().end(),
                b.window().is_last(),
                b.n_embd(),
                boundary.len()
            );
            write_f32_stdout(&b.forward_hidden(&boundary).expect("tail forward"));
        }
        other => {
            eprintln!("unknown mode '{other}'");
            std::process::exit(2);
        }
    }
}
