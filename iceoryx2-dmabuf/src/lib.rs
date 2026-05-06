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
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod connection;
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub mod dmabuf_publisher;
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub mod dmabuf_subscriber;
pub mod external_buffer;
pub(crate) mod path;
pub mod port_factory;
pub mod service;
pub mod service_error;
pub mod service_publisher;
pub mod service_subscriber;
pub mod shm;

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dmabuf_publisher::{DmaBufError, DmaBufPublisher};
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dmabuf_subscriber::DmaBufSubscriber;

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dma_buf::{DmaBuf, MappedDmaBuf};

pub use external_buffer::ExternalFdBuffer;
pub use service::Service as DmaBufService;
