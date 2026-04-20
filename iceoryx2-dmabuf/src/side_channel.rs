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

//! Downstream extension of the upstream SideChannel role trait.
//!
//! The upstream `SideChannel` is a cross-platform role marker; fd semantics are
//! Linux-specific. `FdSideChannel` extends it with the two fd-transfer primitives
//! required for DMA-BUF delivery via `SCM_RIGHTS`.

use core::num::NonZeroU64;
use core::time::Duration;
use std::os::fd::{BorrowedFd, OwnedFd};

use crate::FdSidecarError;

/// Linux-specific extension for side channels capable of transferring file descriptors.
///
/// Implementors MUST also implement `iceoryx2::port::side_channel::SideChannel`.
/// The canonical implementation is `crate::scm::ScmRightsPublisher` /
/// `crate::scm::ScmRightsSubscriber`.
///
/// This trait is crate-internal: it documents the contract between `scm.rs`
/// and the publisher/subscriber layers but is not part of the public API.
/// Call sites use the inherent `_impl` methods directly for monomorphic dispatch;
/// the trait is retained as a structural contract.
#[allow(dead_code)]
pub(crate) trait FdSideChannel: iceoryx2::port::side_channel::SideChannel {
    /// Send `fd` to all connected peers, tagged with `token` for correlation.
    fn send_fd(&mut self, token: NonZeroU64, fd: BorrowedFd<'_>) -> Result<(), FdSidecarError>;

    /// Block until an fd tagged with `expected` arrives, or `timeout` elapses.
    ///
    /// Returns `Err(FdSidecarError::TokenMismatch { .. })` if the received token
    /// does not match `expected`. Returns `Err(FdSidecarError::NoFdInMessage)` if
    /// the sidecar times out before any fd arrives.
    fn recv_fd_matching(
        &mut self,
        expected: NonZeroU64,
        timeout: Duration,
    ) -> Result<OwnedFd, FdSidecarError>;
}
