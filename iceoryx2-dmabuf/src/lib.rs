// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
//! Zero-copy DMA-BUF file-descriptor transport layered on top of iceoryx2.
//!
//! ## Motivation
//!
//! iceoryx2's typed SHM pool model is excellent for value-type payloads but
//! cannot represent kernel-owned DMA-BUF allocations produced by V4L2 ISP,
//! DRM scanout, or Vulkan external-memory exports.  Those frames are identified
//! by a file descriptor whose numeric value is meaningless outside the
//! producing process; cross-process transfer requires `SCM_RIGHTS` over a
//! Unix domain socket.
//!
//! ## Two-channel architecture
//!
//! Every [`FdSidecarPublisher::send`] call performs two coordinated actions:
//!
//! 1. **iceoryx2 metadata channel** — carries the `Meta` payload plus a
//!    [`FdSidecarToken`] user-header that uniquely identifies the frame.
//! 2. **SCM_RIGHTS sidecar** — a Unix domain socket delivers the raw fd
//!    alongside the same token for correlation.
//!
//! [`FdSidecarSubscriber::recv`] dequeues one iceoryx2 sample, extracts its
//! token, and drains the sidecar until a matching fd arrives (50 ms timeout).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use iceoryx2::service::ipc;
//! use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
//!
//! #[derive(Debug, Clone, Copy)]
//! #[repr(C)]
//! struct FrameMeta { width: u32, height: u32 }
//! # // Safety: FrameMeta is repr(C) with no padding of undefined value.
//! unsafe impl iceoryx2::prelude::ZeroCopySend for FrameMeta {}
//!
//! fn example() -> iceoryx2_dmabuf::Result<()> {
//!     let svc = "mos4/frame-plane/video/0";
//!     let mut publisher = FdSidecarPublisher::<ipc::Service, FrameMeta>::create(svc)?;
//!     let mut subscriber = FdSidecarSubscriber::<ipc::Service, FrameMeta>::create(svc)?;
//!     // publisher.send(meta, fd.into())?;
//!     // let (meta, fd) = subscriber.recv()?.unwrap();
//!     Ok(())
//! }
//! ```
//!
//! ## Platform support
//!
//! | Platform | Status |
//! |---|---|
//! | x86_64-unknown-linux-gnu | Full support |
//! | aarch64-unknown-linux-gnu | Full support |
//! | aarch64-apple-darwin | Compiles; `UnsupportedPlatform` at runtime |

// Unsafe is forbidden at the crate level.  The only exceptions are the
// Linux-specific syscall wrappers in `scm.rs` (marked `#[allow(unsafe_code)]`
// at the function level) and test-only `#[allow(unsafe_code)]` blocks.
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod error;
pub mod path;
pub mod publisher;
pub mod scm;
pub mod side_channel;
pub mod subscriber;
pub mod token;

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub mod dmabuf_publisher;
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub mod dmabuf_subscriber;

pub use error::{FdSidecarError, Result};
pub use path::uds_path_for_service;
pub use publisher::FdSidecarPublisher;
pub use subscriber::FdSidecarSubscriber;
pub use token::FdSidecarToken;

/// Convenience alias: [`FdSidecarPublisher`] bound to the IPC service type.
pub type FdSidecarIpcPublisher<Meta> = FdSidecarPublisher<iceoryx2::service::ipc::Service, Meta>;

/// Convenience alias: [`FdSidecarSubscriber`] bound to the IPC service type.
pub type FdSidecarIpcSubscriber<Meta> = FdSidecarSubscriber<iceoryx2::service::ipc::Service, Meta>;

// ── DMA-BUF typed layer (feature = "dma-buf", Linux only) ───────────────────
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dmabuf_publisher::DmaBufPublisher;
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dmabuf_subscriber::DmaBufSubscriber;

/// Re-export of [`dma_buf::DmaBuf`] for callers who don't depend on `dma-buf` directly.
#[cfg(all(target_os = "linux", feature = "dma-buf"))]
pub use dma_buf::{DmaBuf, MappedDmaBuf};

/// Build an iceoryx2 node and publish-subscribe port factory for a given
/// service name.
///
/// # Lifetime contract
///
/// The returned `Node<S>` **must** outlive the `PortFactory<S, Meta,
/// FdSidecarToken>` and any ports derived from it.  Dropping the node before
/// the port causes a use-after-free in iceoryx2's SHM bookkeeping.  Callers
/// must store the node as a struct field (e.g. `_node: Node<S>`) alongside
/// the derived port so that drop order is guaranteed by struct field ordering.
///
/// # Errors
///
/// Returns [`FdSidecarError::Iceoryx`] if node creation, service name parsing,
/// or `open_or_create` fails.
pub(crate) fn build_node_and_service<S, Meta>(
    service_name: &str,
) -> crate::Result<(
    iceoryx2::node::Node<S>,
    iceoryx2::service::port_factory::publish_subscribe::PortFactory<S, Meta, FdSidecarToken>,
)>
where
    S: iceoryx2::service::Service,
    Meta: iceoryx2::prelude::ZeroCopySend + core::fmt::Debug,
{
    use crate::error::IceoryxErrorKind;
    use iceoryx2::prelude::NodeBuilder;

    let node = NodeBuilder::new()
        .create::<S>()
        .map_err(|e| FdSidecarError::Iceoryx {
            kind: IceoryxErrorKind::NodeCreate,
            msg: e.to_string(),
        })?;

    let svc_name: iceoryx2::service::service_name::ServiceName = service_name.try_into().map_err(
        |e: iceoryx2::service::service_name::ServiceNameError| FdSidecarError::Iceoryx {
            kind: IceoryxErrorKind::Service,
            msg: e.to_string(),
        },
    )?;

    let factory = node
        .service_builder(&svc_name)
        .publish_subscribe::<Meta>()
        .user_header::<FdSidecarToken>()
        .open_or_create()
        .map_err(|e| FdSidecarError::Iceoryx {
            kind: IceoryxErrorKind::Service,
            msg: e.to_string(),
        })?;

    Ok((node, factory))
}
