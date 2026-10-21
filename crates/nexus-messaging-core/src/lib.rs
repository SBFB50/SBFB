// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB chat v1 core (plan 2026-10-21): end-to-end encryption for 1:1 and
//! small groups, run by the local daemon, transported as opaque bytes over
//! one iroh-docs document per conversation.
//!
//! Crypto is vodozemac (Olm for pairwise key exchange, Megolm for group
//! sessions). This crate does no I/O.
