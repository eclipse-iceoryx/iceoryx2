// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`DmaBufServiceSubscriber`] — subscriber port for the `dmabuf::Service` pattern.
//!
//! Composes an iceoryx2 `Subscriber<ipc::Service, Meta, ()>` (metadata channel)
//! with a `LinuxSubscriber` (fd channel) on Linux.
//!
//! ## Ordering invariant
//!
//! [`DmaBufServiceSubscriber::receive`] dequeues the iceoryx2 sample **first**,
//! then drains the fd from the socket. This is safe because the publisher sends
//! the fd before the iceoryx2 sample (see [`crate::service_publisher`] and the
//! SPSC contract in [`crate::service`]).
//!
//! If `receive()` finds an iceoryx2 sample but the fd socket is empty, it
//! returns `Err(ServiceError::Connection(Error::NoFdInMessage))`.

use core::fmt::Debug;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::ipc;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPortFactory;
use std::os::fd::OwnedFd;

use crate::service_error::ServiceError;

// ── Platform-specific fd channel ──────────────────────────────────────────────

#[cfg(target_os = "linux")]
use crate::connection::FdPassingConnection as _;
#[cfg(target_os = "linux")]
use crate::connection::linux::LinuxSubscriber;

// ── DmaBufServiceSubscriber ───────────────────────────────────────────────────

/// Subscriber port for the `dmabuf::Service` pattern.
///
/// Obtained from [`crate::port_factory::DmabufPortFactory::subscriber_builder`].
///
/// [`receive`](Self::receive) returns `Ok(None)` when no iceoryx2 sample is
/// ready. On success it returns `(meta, fd, len)`.
///
/// # Platform
///
/// On non-Linux targets, [`receive`](Self::receive) returns
/// [`ServiceError::Connection`] with [`crate::connection::Error::UnsupportedPlatform`].
pub struct DmaBufServiceSubscriber<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    // On non-Linux targets `receive` is a stub, so the field is not used.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    meta_sub: Subscriber<ipc::Service, Meta, ()>,
    #[cfg(target_os = "linux")]
    fd_sub: LinuxSubscriber,
    #[cfg(not(target_os = "linux"))]
    _socket_path: String,
}

impl<Meta> DmaBufServiceSubscriber<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Create a subscriber port by connecting to the fd channel socket and
    /// building the iceoryx2 subscriber.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 subscriber creation fails.
    /// - [`ServiceError::Connection`] — if connecting to the UDS socket fails (Linux only).
    pub(crate) fn create(
        meta_factory: &IceoryxPortFactory<ipc::Service, Meta, ()>,
        socket_path: &str,
    ) -> Result<Self, ServiceError> {
        let meta_sub = meta_factory
            .subscriber_builder()
            .create()
            .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

        #[cfg(target_os = "linux")]
        let fd_sub = LinuxSubscriber::open(socket_path).map_err(ServiceError::Connection)?;

        Ok(Self {
            meta_sub,
            #[cfg(target_os = "linux")]
            fd_sub,
            #[cfg(not(target_os = "linux"))]
            _socket_path: socket_path.to_owned(),
        })
    }

    /// Non-blocking receive.
    ///
    /// Returns `Ok(None)` if no iceoryx2 sample is currently queued.
    ///
    /// On success:
    /// 1. Dequeues the iceoryx2 metadata sample.
    /// 2. Drains the fd from the socket (non-blocking).
    /// 3. Returns `(meta, fd, len)`.
    ///
    /// Returns `Err(ServiceError::Connection(Error::NoFdInMessage))` if the
    /// iceoryx2 sample arrived but the fd socket is empty (publisher crashed
    /// between the two sends, violating the ordering contract).
    ///
    /// On non-Linux targets this always returns
    /// [`ServiceError::Connection(Error::UnsupportedPlatform)`].
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 receive fails.
    /// - [`ServiceError::Connection`] — if the fd receive fails.
    pub fn receive(&mut self) -> Result<Option<(Meta, OwnedFd, u64)>, ServiceError> {
        #[cfg(target_os = "linux")]
        {
            // 1. Check iceoryx2 metadata channel first.
            let Some(sample) = self
                .meta_sub
                .receive()
                .map_err(|e| ServiceError::Iceoryx(e.to_string()))?
            else {
                return Ok(None);
            };

            // Copy meta before the sample slot is returned to the pool.
            let meta = *sample.payload();
            drop(sample);

            // 2. Drain the fd from the socket. Publisher sent it before the sample,
            //    so it must be present. If not, the publisher violated the ordering
            //    contract (e.g. crash mid-send).
            let Some((fd, len)) = self
                .fd_sub
                .recv_with_fd()
                .map_err(ServiceError::Connection)?
            else {
                return Err(ServiceError::Connection(
                    crate::connection::Error::NoFdInMessage,
                ));
            };

            Ok(Some((meta, fd, len)))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(ServiceError::Connection(
                crate::connection::Error::UnsupportedPlatform,
            ))
        }
    }
}
