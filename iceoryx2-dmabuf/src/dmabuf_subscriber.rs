// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Typed DMA-BUF convenience subscriber.
//!
//! Newtype over [`crate::service_subscriber::DmaBufServiceSubscriber`] that
//! returns `(Meta, dma_buf::DmaBuf)` instead of `(Meta, OwnedFd, u64)`.
//! Gated on `all(target_os = "linux", feature = "dma-buf")` via the `pub mod`
//! declaration in `lib.rs`; this file contains no inner cfg attribute.

use core::fmt::Debug;

use iceoryx2::prelude::ZeroCopySend;

use crate::dmabuf_publisher::DmaBufError;
use crate::port_factory::DmabufPortFactory;
use crate::service::Service;
use crate::service_subscriber::DmaBufServiceSubscriber;

/// Typed DMA-BUF subscriber.
///
/// Wraps a [`DmaBufServiceSubscriber`] and converts received `OwnedFd` values
/// to `dma_buf::DmaBuf` via `From<OwnedFd>` (zero syscall).
/// Owns the [`DmabufPortFactory`] to keep the iceoryx2 node alive.
pub struct DmaBufSubscriber<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    inner: DmaBufServiceSubscriber<Meta>,
    /// Declared LAST — dropped last (Rust drops fields in declaration order;
    /// last declared = last dropped). `inner` must drop before `_factory`
    /// so the iceoryx2 node outlives any port it created.
    _factory: DmabufPortFactory<Meta>,
}

impl<Meta> DmaBufSubscriber<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Open or create the named DMA-BUF service and build a typed subscriber port.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] if [`Service::open_or_create`] or
    /// [`crate::port_factory::SubscriberBuilder::create`] fails.
    pub fn create(service_name: &str) -> Result<Self, DmaBufError> {
        let factory = Service::open_or_create::<Meta>(service_name)?;
        let inner = factory.subscriber_builder().create()?;
        Ok(Self {
            inner,
            _factory: factory,
        })
    }

    /// Non-blocking receive.
    ///
    /// Returns `Ok(None)` if no sample is queued. On success, the received
    /// `OwnedFd` is wrapped via `dma_buf::DmaBuf::from(fd)` (zero syscall).
    ///
    /// The token embedded in the wire frame is discarded. Use
    /// [`receive_with_token`](Self::receive_with_token) to retrieve it.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] on any underlying receive failure.
    pub fn receive(&mut self) -> Result<Option<(Meta, dma_buf::DmaBuf)>, DmaBufError> {
        let Some((meta, fd, _len)) = self.inner.receive()? else {
            return Ok(None);
        };
        Ok(Some((meta, dma_buf::DmaBuf::from(fd))))
    }

    /// Non-blocking receive with token.
    ///
    /// Returns `Ok(None)` if no sample is queued. On success returns
    /// `(meta, buf, token)`.
    ///
    /// The `token` matches the one passed to the publisher's
    /// [`crate::dmabuf_publisher::DmaBufPublisher::publish_with_token`] call
    /// (or the internal counter token from [`crate::dmabuf_publisher::DmaBufPublisher::publish`]).
    /// Use [`release`](Self::release) to send an ack back.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] on any underlying receive failure.
    pub fn receive_with_token(
        &mut self,
    ) -> Result<Option<(Meta, dma_buf::DmaBuf, u64)>, DmaBufError> {
        let Some((meta, fd, _len, token)) = self.inner.receive_with_token()? else {
            return Ok(None);
        };
        Ok(Some((meta, dma_buf::DmaBuf::from(fd), token)))
    }

    /// Send a "buffer released" ack back to the publisher carrying `token`.
    ///
    /// Best-effort: dropped silently if the subscriber's send buffer is full.
    /// Callers must tolerate occasional missed acks.
    ///
    /// # Errors
    ///
    /// Returns [`DmaBufError::Service`] on connection-level send errors.
    pub fn release(&mut self, token: u64) -> Result<(), DmaBufError> {
        self.inner.release(token).map_err(DmaBufError::Service)
    }
}
