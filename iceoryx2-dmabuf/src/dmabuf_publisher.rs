// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Typed DMA-BUF convenience publisher.
//!
//! Newtype over [`crate::service_publisher::DmaBufServicePublisher`] that
//! accepts `&dma_buf::DmaBuf` instead of `BorrowedFd + len`. Gated on the
//! `dma-buf` Cargo feature (Linux only, pulls upstream `dma-buf` 0.5).
//!
//! Per-send overhead vs the lower-level service publisher: one
//! `fcntl(F_DUPFD_CLOEXEC)` to clone the borrowed DmaBuf fd before sending,
//! so the caller's `dma_buf::DmaBuf` value remains usable after the call.
//! Additionally, one `fstat` syscall per send to retrieve the buffer length
//! from the kernel (DmaBuf exposes no `.size()` accessor).

use core::fmt::Debug;
use std::os::fd::AsFd as _;

use iceoryx2::prelude::ZeroCopySend;

use crate::port_factory::DmabufPortFactory;
use crate::service::Service;
use crate::service_error::ServiceError;
use crate::service_publisher::DmaBufServicePublisher;

// ── DmaBufError ───────────────────────────────────────────────────────────────

/// Errors returned by [`DmaBufPublisher`] and [`crate::dmabuf_subscriber::DmaBufSubscriber`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DmaBufError {
    /// An underlying [`ServiceError`] from service creation or port operations.
    Service(ServiceError),
    /// `fcntl(F_DUPFD_CLOEXEC)`, `fstat`, or type-conversion on the DMA-BUF fd failed.
    FdDup(std::io::Error),
}

impl core::fmt::Display for DmaBufError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Service(e) => write!(f, "service error: {e}"),
            Self::FdDup(e) => write!(f, "fd dup/stat failed: {e}"),
        }
    }
}

impl core::error::Error for DmaBufError {}

impl From<ServiceError> for DmaBufError {
    fn from(e: ServiceError) -> Self {
        Self::Service(e)
    }
}

// ── DmaBufPublisher ───────────────────────────────────────────────────────────

/// Typed DMA-BUF publisher.
///
/// Wraps a [`DmaBufServicePublisher`] with `&dma_buf::DmaBuf` ergonomics.
/// Owns the [`DmabufPortFactory`] to keep the iceoryx2 node alive.
pub struct DmaBufPublisher<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Must be dropped after `inner` (declared first = dropped last).
    _factory: DmabufPortFactory<Meta>,
    inner: DmaBufServicePublisher<Meta>,
}

impl<Meta> DmaBufPublisher<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Open or create the named DMA-BUF service and build a typed publisher port.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] if [`Service::open_or_create`] or
    /// [`crate::port_factory::PublisherBuilder::create`] fails.
    pub fn create(service_name: &str) -> Result<Self, DmaBufError> {
        let factory = Service::open_or_create::<Meta>(service_name)?;
        let inner = factory.publisher_builder().create()?;
        Ok(Self {
            _factory: factory,
            inner,
        })
    }

    /// Publish `meta` alongside the DMA-BUF buffer.
    ///
    /// `buf.as_fd()` is duplicated via `fcntl(F_DUPFD_CLOEXEC)` (one syscall
    /// per frame) so the caller's `DmaBuf` remains valid after this call. The
    /// buffer length is retrieved with `fstat` (one syscall per frame) because
    /// `dma_buf::DmaBuf` exposes no `.size()` accessor.
    ///
    /// An internal monotonic counter supplies the token automatically. Use
    /// [`publish_with_token`](Self::publish_with_token) to supply a caller-chosen
    /// token for pool-ack correlation.
    ///
    /// # Errors
    ///
    /// - [`DmaBufError::FdDup`] if the fd duplication, `fstat`, or size
    ///   conversion fails.
    /// - [`DmaBufError::Service`] for any underlying publish failure.
    pub fn publish(&mut self, meta: Meta, buf: &dma_buf::DmaBuf) -> Result<(), DmaBufError> {
        let (cloned, len) = dup_and_stat(buf)?;
        self.inner
            .publish(meta, cloned.as_fd(), len)
            .map_err(DmaBufError::Service)
    }

    /// Publish `meta` alongside the DMA-BUF buffer with a caller-supplied `token`.
    ///
    /// Same fd-dup + fstat overhead as [`publish`](Self::publish). The token is
    /// embedded in the wire frame and returned by the subscriber's
    /// [`crate::dmabuf_subscriber::DmaBufSubscriber::receive_with_token`] call.
    /// Use this when implementing pool-ack semantics.
    ///
    /// # Errors
    ///
    /// - [`DmaBufError::FdDup`] if the fd duplication, `fstat`, or size
    ///   conversion fails.
    /// - [`DmaBufError::Service`] for any underlying publish failure.
    pub fn publish_with_token(
        &mut self,
        meta: Meta,
        buf: &dma_buf::DmaBuf,
        token: u64,
    ) -> Result<(), DmaBufError> {
        let (cloned, len) = dup_and_stat(buf)?;
        self.inner
            .publish_with_token(meta, cloned.as_fd(), len, token)
            .map_err(DmaBufError::Service)
    }

    /// Non-blocking drain of one back-channel ack from any subscriber.
    ///
    /// Returns `Ok(None)` if no ack is currently queued.
    /// Returns `Ok(Some(token))` when an ack is drained.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] on connection-level errors (e.g. bad magic).
    pub fn recv_release_ack(&mut self) -> Result<Option<u64>, DmaBufError> {
        self.inner.recv_release_ack().map_err(DmaBufError::Service)
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Duplicate the DmaBuf fd and retrieve the buffer length via `fstat`.
fn dup_and_stat(buf: &dma_buf::DmaBuf) -> Result<(std::os::fd::OwnedFd, u64), DmaBufError> {
    let cloned = buf
        .as_fd()
        .try_clone_to_owned()
        .map_err(DmaBufError::FdDup)?;

    let stat = rustix::fs::fstat(&cloned)
        .map_err(|e| DmaBufError::FdDup(std::io::Error::from_raw_os_error(e.raw_os_error())))?;

    // st_size is i64; a valid DMA-BUF must have a non-negative size.
    let len = u64::try_from(stat.st_size).map_err(|_| {
        DmaBufError::FdDup(std::io::Error::other(
            "DMA-BUF fstat returned negative st_size",
        ))
    })?;

    Ok((cloned, len))
}
