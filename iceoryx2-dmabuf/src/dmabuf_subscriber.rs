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
    /// Must be dropped after `inner` (declared first = dropped last).
    _factory: DmabufPortFactory<Meta>,
    inner: DmaBufServiceSubscriber<Meta>,
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
            _factory: factory,
            inner,
        })
    }

    /// Non-blocking receive.
    ///
    /// Returns `Ok(None)` if no sample is queued. On success, the received
    /// `OwnedFd` is wrapped via `dma_buf::DmaBuf::from(fd)` (zero syscall).
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
}
