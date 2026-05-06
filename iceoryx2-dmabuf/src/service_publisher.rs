// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`DmaBufServicePublisher`] — publisher port for the `dmabuf::Service` pattern.
//!
//! Composes an iceoryx2 `Publisher<ipc::Service, Meta, ()>` (metadata channel)
//! with a `LinuxPublisher` (fd channel) on Linux, and a non-functional stub on
//! other platforms.
//!
//! ## Ordering invariant
//!
//! [`DmaBufServicePublisher::publish`] sends the fd **first** (so it is queued
//! in the subscriber's socket receive buffer), then sends the iceoryx2 sample.
//! This guarantees that by the time the subscriber dequeues the sample, the fd
//! is already waiting. See [`crate::service`] module documentation for the full
//! SPSC ordering contract.

use core::fmt::Debug;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::ipc;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPortFactory;
use std::os::fd::BorrowedFd;

use crate::service_error::ServiceError;

// ── Platform-specific fd channel ──────────────────────────────────────────────

#[cfg(target_os = "linux")]
use crate::connection::FdPassingConnection as _;
#[cfg(target_os = "linux")]
use crate::connection::linux::LinuxPublisher;

// ── DmaBufServicePublisher ────────────────────────────────────────────────────

/// Publisher port for the `dmabuf::Service` pattern.
///
/// Obtained from [`crate::port_factory::DmabufPortFactory::publisher_builder`].
///
/// On each [`publish`](Self::publish) call:
/// 1. The fd is sent via the fd channel (SCM_RIGHTS over UDS) to all connected
///    subscribers — **before** the iceoryx2 sample.
/// 2. The metadata sample is loaned from the iceoryx2 publisher, written, and sent.
///
/// See [`crate::service`] for the SPSC ordering contract.
///
/// # Platform
///
/// On non-Linux targets, [`publish`](Self::publish) returns
/// [`ServiceError::Connection`] with [`crate::connection::Error::UnsupportedPlatform`].
pub struct DmaBufServicePublisher<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    // On non-Linux targets `publish` is a stub, so the field is not used.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    meta_pub: Publisher<ipc::Service, Meta, ()>,
    #[cfg(target_os = "linux")]
    fd_pub: LinuxPublisher,
    #[cfg(not(target_os = "linux"))]
    _socket_path: String,
    /// Monotonically increasing token counter for auto-token assignment.
    /// All callers of `publish` take `&mut self`, so a plain `u64` suffices
    /// (no shared mutability, no atomics needed).
    token_counter: u64,
}

impl<Meta> DmaBufServicePublisher<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Create a publisher port by binding the fd channel socket and building the
    /// iceoryx2 publisher.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 publisher creation fails.
    /// - [`ServiceError::Connection`] — if binding the UDS socket fails (Linux only).
    pub(crate) fn create(
        meta_factory: &IceoryxPortFactory<ipc::Service, Meta, ()>,
        socket_path: &str,
    ) -> Result<Self, ServiceError> {
        let meta_pub = meta_factory
            .publisher_builder()
            .create()
            .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

        #[cfg(target_os = "linux")]
        let fd_pub = LinuxPublisher::open(socket_path).map_err(ServiceError::Connection)?;

        Ok(Self {
            meta_pub,
            #[cfg(target_os = "linux")]
            fd_pub,
            #[cfg(not(target_os = "linux"))]
            _socket_path: socket_path.to_owned(),
            token_counter: 0,
        })
    }

    /// Publish `meta` alongside `fd` with an associated byte `len`.
    ///
    /// An internal monotonic counter supplies the token automatically. Use
    /// [`publish_with_token`](Self::publish_with_token) to supply a caller-chosen
    /// token for pool-ack correlation.
    ///
    /// Step 1: sends `fd` to all connected subscribers via the fd channel
    /// (SCM_RIGHTS). Step 2: loans an iceoryx2 slot, writes `meta`, and sends
    /// the sample.
    ///
    /// On non-Linux targets this always returns
    /// [`ServiceError::Connection`] wrapping
    /// [`crate::connection::Error::UnsupportedPlatform`].
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Connection`] — if the fd send fails.
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 loan or send fails.
    pub fn publish(
        &mut self,
        meta: Meta,
        fd: BorrowedFd<'_>,
        len: u64,
    ) -> Result<(), ServiceError> {
        // Auto-increment token. Wrapping add is intentional (u64 overflow =
        // restart from 0 after 2^64 frames, which is safe for pool-ack purposes).
        let token = self.token_counter;
        self.token_counter = self.token_counter.wrapping_add(1);
        self.publish_with_token(meta, fd, len, token)
    }

    /// Publish `meta` alongside `fd` with an associated byte `len` and a
    /// caller-supplied `token`.
    ///
    /// The token is embedded in the wire frame and returned by the subscriber's
    /// [`crate::service_subscriber::DmaBufServiceSubscriber::receive_with_token`].
    /// Use this when implementing pool-ack semantics where the caller tracks
    /// buffer lifecycle by token.
    ///
    /// On non-Linux targets this always returns
    /// [`ServiceError::Connection`] wrapping
    /// [`crate::connection::Error::UnsupportedPlatform`].
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Connection`] — if the fd send fails.
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 loan or send fails.
    pub fn publish_with_token(
        &mut self,
        meta: Meta,
        fd: BorrowedFd<'_>,
        len: u64,
        token: u64,
    ) -> Result<(), ServiceError> {
        #[cfg(target_os = "linux")]
        {
            // 1. Send fd first — queued in subscriber socket buffer before sample arrives.
            self.fd_pub
                .send_with_fd(fd, len, token)
                .map_err(ServiceError::Connection)?;

            // 2. Loan iceoryx2 slot, write payload, send.
            let mut sample = self
                .meta_pub
                .loan_uninit()
                .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;
            let sample = sample.write_payload(meta);
            sample
                .send()
                .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = (meta, fd, len, token);
            Err(ServiceError::Connection(
                crate::connection::Error::UnsupportedPlatform,
            ))
        }
    }

    /// Non-blocking drain of one back-channel ack from any subscriber.
    ///
    /// Returns `Ok(None)` if no ack is currently queued.
    /// Returns `Ok(Some(token))` when an ack is drained.
    ///
    /// On non-Linux targets this always returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Connection`] — if the ack receive fails (e.g. bad magic).
    pub fn recv_release_ack(&mut self) -> Result<Option<u64>, ServiceError> {
        #[cfg(target_os = "linux")]
        {
            self.fd_pub
                .recv_release_ack()
                .map_err(ServiceError::Connection)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(None)
        }
    }
}
