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

// `FdSidecarPublisher` in module `publisher` triggers module_name_repetitions.
// Renaming the module adds churn without correctness benefit — allowed at module level per spec §NFR Clippy.
#![allow(clippy::module_name_repetitions)]

//! [`FdSidecarPublisher`] — composes an iceoryx2 publisher with the SCM_RIGHTS
//! side-channel to deliver DMA-BUF file descriptors alongside metadata
//! samples.

use core::fmt::Debug;
use core::num::NonZeroU64;
use iceoryx2::node::Node;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use std::os::fd::{AsFd as _, OwnedFd};

use crate::error::{FdSidecarError, IceoryxErrorKind, Result};
use crate::scm::ScmRightsPublisher;
use crate::token::FdSidecarToken;

/// SCM_RIGHTS fd-sidecar publisher.
///
/// Composes an iceoryx2 `Publisher<S, Meta, FdSidecarToken>` with a
/// [`ScmRightsPublisher`] that transports the file descriptor out-of-band via
/// a Unix-domain socket using `SCM_RIGHTS`.
///
/// The fd is sent *before* the iceoryx2 sample so that by the time the
/// subscriber dequeues the sample, the fd is already waiting in its socket
/// receive queue.
///
/// # Type parameters
///
/// - `S` — iceoryx2 service type (e.g. [`iceoryx2::service::ipc::Service`]).
///   Use `FdSidecarIpcPublisher` for the common IPC case.
/// - `Meta` — application payload type; must be `ZeroCopySend + Debug`.
///
/// # Token monotonicity
///
/// `next_token` starts at 1 so that the first emitted token is always
/// non-zero; zero is reserved as a sentinel for
/// [`FdSidecarError::TokenExhausted`].
///
/// `send` requires `&mut self`, so exclusive mutable access is guaranteed by
/// the borrow checker — no atomic is needed.
pub struct FdSidecarPublisher<S: Service, Meta: ZeroCopySend + Debug + 'static> {
    /// Node MUST be declared before `inner` and `_port_factory` so it is
    /// dropped last (Rust drops fields in declaration order).  See
    /// `crate::build_node_and_service` for the Node lifetime contract.
    _node: Node<S>,
    inner: Publisher<S, Meta, FdSidecarToken>,
    side: ScmRightsPublisher,
    /// Monotonically increasing token counter.  `send` takes `&mut self`, so
    /// this field is always accessed under exclusive ownership.
    next_token: u64,
    // Keep the port factory alive so the iceoryx2 service is not dropped.
    _port_factory: PortFactory<S, Meta, FdSidecarToken>,
}

impl<S: Service, Meta: ZeroCopySend + Debug + Copy + 'static> FdSidecarPublisher<S, Meta> {
    /// Create a new `FdSidecarPublisher` for `service_name`.
    ///
    /// Opens (or creates) an iceoryx2 service of type `S` with the given name,
    /// configures `FdSidecarToken` as the user-header type, and binds a
    /// Unix-domain socket side-channel for fd delivery.
    ///
    /// `_node` is stored to guarantee it outlives the port.
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::UnsupportedPlatform`] — on non-Linux targets.
    /// - [`FdSidecarError::SideChannelIo`] — if the UDS socket cannot be created.
    /// - [`FdSidecarError::Iceoryx`] — if node or service creation fails.
    pub fn create(service_name: &str) -> Result<Self> {
        use iceoryx2::port::side_channel::Role;

        let (_node, port_factory) = crate::build_node_and_service::<S, Meta>(service_name)?;

        let publisher =
            port_factory
                .publisher_builder()
                .create()
                .map_err(|e| FdSidecarError::Iceoryx {
                    kind: IceoryxErrorKind::PortBuilder,
                    msg: e.to_string(),
                })?;

        // Open the SCM_RIGHTS side-channel publisher.
        let side = ScmRightsPublisher::open(service_name, Role::Publisher)?;

        Ok(Self {
            _node,
            inner: publisher,
            side,
            next_token: 1,
            _port_factory: port_factory,
        })
    }

    /// Publish `meta` alongside `fd`.
    ///
    /// 1. Allocates the next correlation token.
    /// 2. Sends `fd` to all connected subscribers via `SCM_RIGHTS` **first**,
    ///    so the fd is in the subscriber's socket receive queue before the
    ///    iceoryx2 sample arrives.
    /// 3. Loans a slot from the iceoryx2 publisher, writes `token` into the
    ///    user-header and `meta` into the payload, then sends the sample.
    ///
    /// `fd` is consumed: the publisher takes temporary ownership for the
    /// duration of the `sendmsg` call.  The kernel duplicates the fd into
    /// every connected subscriber's fd table before this function returns.
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::TokenExhausted`] — the 64-bit token space wrapped to 0.
    /// - [`FdSidecarError::SideChannelIo`] — a socket operation failed.
    /// - [`FdSidecarError::IceoryxPublish`] — the iceoryx2 loan or send failed.
    pub fn send(&mut self, meta: Meta, fd: OwnedFd) -> Result<()> {
        let raw = self.next_token;
        self.next_token = self.next_token.wrapping_add(1);
        let token = NonZeroU64::new(raw).ok_or(FdSidecarError::TokenExhausted)?;

        // 1. Send fd on the sidecar FIRST (spec §Fault model).
        self.side.send_fd_impl(token, fd.as_fd())?;

        // Test-only pause hook: when `DMABUF_CRASH_PHASE=mid-iceoryx2` the
        // process sends SIGSTOP to itself so that a test can observe the
        // subscriber receiving `NoFdInMessage`.  This call is compiled out in
        // non-test builds.
        #[cfg(test)]
        Self::pause_hook_if_requested();

        // 2. Publish the iceoryx2 sample.
        let mut sample = self
            .inner
            .loan_uninit()
            .map_err(FdSidecarError::IceoryxLoan)?;

        // Store the token as a raw u64 in the user-header.
        sample.user_header_mut().token = token.get();
        let sample = sample.write_payload(meta);
        sample.send().map_err(FdSidecarError::IceoryxPublish)?;

        Ok(())
    }

    /// Test-only pause hook.
    ///
    /// When the environment variable `DMABUF_CRASH_PHASE` is set to
    /// `mid-iceoryx2`, the calling process sends `SIGSTOP` to itself.  This
    /// freezes the publisher between the sidecar `send_fd` and the iceoryx2
    /// publish so that a test can assert the subscriber sees
    /// [`FdSidecarError::NoFdInMessage`].
    ///
    /// Compiled out in non-test builds (`#[cfg(test)]`).
    ///
    /// # Safety rationale (approved in primary plan)
    ///
    /// `libc::raise(SIGSTOP)` is a standard POSIX signal send to self.
    /// Only used inside `#[cfg(test)]` when the env var is set.
    #[cfg(test)]
    fn pause_hook_if_requested() {
        if std::env::var("DMABUF_CRASH_PHASE").as_deref() == Ok("mid-iceoryx2") {
            #[cfg(target_os = "linux")]
            // SAFETY: raise(SIGSTOP) is a well-defined POSIX signal operation on self;
            // test-only, controlled by env var. Approved in primary plan.
            #[allow(unsafe_code)]
            unsafe {
                libc::raise(libc::SIGSTOP);
            }
        }
    }

    /// Inject a raw (token, fd) into all connected subscriber streams.
    ///
    /// For use in tests that need to forge out-of-order tokens to exercise
    /// the TokenMismatch / NoFdInMessage error paths.
    ///
    /// Enabled by the `test-utils` feature — not part of the stable public interface.
    #[cfg(all(feature = "test-utils", target_os = "linux"))]
    pub fn inject_raw_for_test(
        &self,
        token: u64,
        fd: std::os::fd::BorrowedFd<'_>,
    ) -> crate::Result<()> {
        self.side.inject_raw_for_test(token, fd)
    }

    /// Publish the iceoryx2 metadata sample **without** sending the fd on the
    /// sidecar.
    ///
    /// This is the test-only complement to `inject_raw_for_test`.  Use it to
    /// drive the `NoFdInMessage` error path: the subscriber's iceoryx2 queue
    /// receives a sample with a valid token, but no matching fd is ever
    /// delivered on the Unix-domain socket, so `recv_fd_matching_impl` times
    /// out and returns `NoFdInMessage`.
    ///
    /// The token counter is advanced exactly as in `send`, so subsequent real
    /// `send` calls remain consistent.
    ///
    /// # Invariant
    ///
    /// Callers must not send a real fd (via `inject_raw_for_test` or `send`)
    /// with the same token after calling this function, as that would violate
    /// the monotonicity contract.
    ///
    /// Enabled by the `test-utils` feature — not part of the stable public interface.
    #[cfg(all(feature = "test-utils", target_os = "linux"))]
    pub fn send_metadata_only_for_test(&mut self, meta: Meta) -> crate::Result<()> {
        let raw = self.next_token;
        self.next_token = self.next_token.wrapping_add(1);
        let token = NonZeroU64::new(raw).ok_or(FdSidecarError::TokenExhausted)?;

        // Skip side.send_fd_impl — intentionally no fd is delivered.
        let mut sample = self
            .inner
            .loan_uninit()
            .map_err(FdSidecarError::IceoryxLoan)?;
        sample.user_header_mut().token = token.get();
        let sample = sample.write_payload(meta);
        sample.send().map_err(FdSidecarError::IceoryxPublish)?;

        Ok(())
    }
}
