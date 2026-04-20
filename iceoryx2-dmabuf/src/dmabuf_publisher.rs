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

//! [`DmaBufPublisher`] — typed wrapper over [`FdSidecarPublisher`] for
//! publishing `dma_buf::DmaBuf` values.
//!
//! Gated on `cfg(all(target_os = "linux", feature = "dma-buf"))`.

#![cfg(all(target_os = "linux", feature = "dma-buf"))]

use core::fmt::Debug;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;
use std::os::fd::AsFd as _;

use crate::error::{FdSidecarError, Result};
use crate::publisher::FdSidecarPublisher;

// `DmaBufPublisher` in module `dmabuf_publisher` would trigger
// clippy::module_name_repetitions.  The module name carries the necessary
// disambiguation; renaming it adds churn without correctness benefit.
#[allow(clippy::module_name_repetitions)]
/// Typed DMA-BUF publisher.
///
/// A newtype over [`FdSidecarPublisher`] that accepts `dma_buf::DmaBuf`
/// values instead of raw `OwnedFd`. On each [`send`](DmaBufPublisher::send)
/// call, the fd is borrowed and duplicated via `fcntl(F_DUPFD_CLOEXEC)` —
/// one syscall per frame — so that the upstream `DmaBuf` value remains
/// valid in the caller's scope after `send` returns.
///
/// # Platform
///
/// Only available on Linux with the `dma-buf` Cargo feature enabled.
pub struct DmaBufPublisher<S: Service, Meta: ZeroCopySend + Debug + 'static> {
    inner: FdSidecarPublisher<S, Meta>,
}

impl<S: Service, Meta: ZeroCopySend + Debug + Copy + 'static> DmaBufPublisher<S, Meta> {
    /// Create a new `DmaBufPublisher` for `service_name`.
    ///
    /// Delegates to [`FdSidecarPublisher::create`].
    ///
    /// # Errors
    ///
    /// Returns the same error set as [`FdSidecarPublisher::create`].
    pub fn create(service_name: &str) -> Result<Self> {
        Ok(Self {
            inner: FdSidecarPublisher::create(service_name)?,
        })
    }

    /// Publish `meta` alongside a reference to `buf`.
    ///
    /// The `buf` fd is duplicated via `fcntl(F_DUPFD_CLOEXEC)` (one syscall
    /// per frame) so that `buf` remains valid in the caller after this
    /// function returns. The duplicated fd is delivered to all connected
    /// subscribers via SCM_RIGHTS.
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::SideChannelIo`] — if `fcntl` or socket I/O fails.
    /// - All errors from [`FdSidecarPublisher::send`].
    pub fn send(&mut self, meta: Meta, buf: &dma_buf::DmaBuf) -> Result<()> {
        let fd = buf
            .as_fd()
            .try_clone_to_owned()
            .map_err(FdSidecarError::SideChannelIo)?;
        self.inner.send(meta, fd)
    }

    /// Forward `inject_raw_for_test` from the inner publisher.
    ///
    /// Enabled by the `test-utils` Cargo feature — not part of the stable
    /// public interface.
    #[cfg(feature = "test-utils")]
    pub fn inject_raw_for_test(&self, token: u64, fd: std::os::fd::BorrowedFd<'_>) -> Result<()> {
        self.inner.inject_raw_for_test(token, fd)
    }

    /// Forward `send_metadata_only_for_test` from the inner publisher.
    ///
    /// Enabled by the `test-utils` Cargo feature — not part of the stable
    /// public interface.
    #[cfg(feature = "test-utils")]
    pub fn send_metadata_only_for_test(&mut self, meta: Meta) -> Result<()> {
        self.inner.send_metadata_only_for_test(meta)
    }
}
