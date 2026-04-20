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

//! [`DmaBufSubscriber`] — typed wrapper over [`FdSidecarSubscriber`] for
//! receiving `dma_buf::DmaBuf` values.
//!
//! Gated on `cfg(all(target_os = "linux", feature = "dma-buf"))`.

#![cfg(all(target_os = "linux", feature = "dma-buf"))]

use core::fmt::Debug;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;

use crate::error::Result;
use crate::subscriber::FdSidecarSubscriber;

#[allow(clippy::module_name_repetitions)]
/// Typed DMA-BUF subscriber.
///
/// A newtype over [`FdSidecarSubscriber`] that yields `dma_buf::DmaBuf`
/// values instead of raw `OwnedFd`. The conversion `DmaBuf::from(OwnedFd)`
/// is zero-syscall.
///
/// Subscribers obtain CPU access via `dma_buf::MappedDmaBuf::read`,
/// `write`, or `readwrite`, which wrap `DMA_BUF_IOCTL_SYNC` start/end
/// pairing on cache-incoherent SoCs.
///
/// # Platform
///
/// Only available on Linux with the `dma-buf` Cargo feature enabled.
pub struct DmaBufSubscriber<S: Service, Meta: ZeroCopySend + Debug + 'static> {
    inner: FdSidecarSubscriber<S, Meta>,
}

impl<S: Service, Meta: ZeroCopySend + Debug + Copy + 'static> DmaBufSubscriber<S, Meta> {
    /// Create a new `DmaBufSubscriber` for `service_name`.
    ///
    /// Delegates to [`FdSidecarSubscriber::create`].
    ///
    /// # Errors
    ///
    /// Returns the same error set as [`FdSidecarSubscriber::create`].
    pub fn create(service_name: &str) -> Result<Self> {
        Ok(Self {
            inner: FdSidecarSubscriber::create(service_name)?,
        })
    }

    /// Non-blocking receive.
    ///
    /// Returns `None` if no sample is ready. On success, the received
    /// `OwnedFd` is converted to `dma_buf::DmaBuf` via `DmaBuf::from(fd)`
    /// (zero syscall).
    ///
    /// # Errors
    ///
    /// All errors from [`FdSidecarSubscriber::recv`].
    pub fn recv(&mut self) -> Result<Option<(Meta, dma_buf::DmaBuf)>> {
        let Some((meta, fd)) = self.inner.recv()? else {
            return Ok(None);
        };
        Ok(Some((meta, dma_buf::DmaBuf::from(fd))))
    }

    /// Reconnect the side-channel.
    ///
    /// Forwards to [`FdSidecarSubscriber::reconnect`]. The DmaBuf layer
    /// adds no additional reconnect semantics.
    ///
    /// # Errors
    ///
    /// All errors from [`FdSidecarSubscriber::reconnect`].
    pub fn reconnect(&mut self) -> Result<()> {
        self.inner.reconnect()
    }
}
