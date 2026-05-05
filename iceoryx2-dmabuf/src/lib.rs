// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # iceoryx2-dmabuf
//!
//! Typed DMA-BUF transport via a parallel `dmabuf::Service` variant.
//!
//! This crate provides standalone primitives for fd-backed shared memory
//! that are deliberately **not** coupled to `iceoryx2-cal`'s `SharedMemory`
//! trait (which is built around `PointerOffset` and pool allocators).
//! See `iceoryx2-dmabuf/specs/arch-dmabuf-service-variant.adoc` decision D1.
//!
//! ## Platform support
//!
//! | Platform                    | Status                          |
//! |-----------------------------|---------------------------------|
//! | x86_64-unknown-linux-gnu    | Full support                    |
//! | aarch64-unknown-linux-gnu   | Full support                    |
//! | aarch64-apple-darwin        | Compiles via non-Linux stub     |

// Unsafe is forbidden at the crate level.
// The sole exceptions are the Linux-specific syscall wrappers in
// `shm/linux.rs`, marked `#[allow(unsafe_code)]` at the block level.
#![deny(unsafe_code)]

pub mod external_buffer;
pub mod shm;
