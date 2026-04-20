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

use core::num::NonZeroU64;
use iceoryx2::prelude::ZeroCopySend;

/// Monotonic sequence number used to correlate an iceoryx2 sample with the
/// file descriptor delivered over the side-channel socket.
///
/// Stored in the sample's user-header so that the subscriber can verify it
/// received the fd matching the sample it pulled from the ring buffer.
///
/// # Safety (ZeroCopySend)
///
/// `FdSidecarToken` is `#[repr(C)]` with a single `u64` field — a plain integer
/// with no pointers, references, or OS handles.  It is safe to copy across
/// process address spaces via the iceoryx2 SHM pool.
///
/// The raw `u64` value is always non-zero at the application level (enforced
/// by [`crate::publisher::FdSidecarPublisher::send`]); zero is reserved as a
/// sentinel (see [`crate::FdSidecarError::TokenExhausted`]).
#[derive(Debug, Clone, Copy, Default, ZeroCopySend)]
#[repr(C)]
pub struct FdSidecarToken {
    /// The non-zero sequence counter stored as a `u64` for SHM compatibility.
    /// `pub(crate)` to prevent external struct-literal construction;
    /// use [`FdSidecarToken::from_nonzero`] to construct a valid token.
    pub(crate) token: u64,
}

impl FdSidecarToken {
    /// Construct from a non-zero value.
    ///
    /// The caller guarantees that `v` is a valid, non-zero sequence counter.
    pub fn from_nonzero(v: NonZeroU64) -> Self {
        Self { token: v.get() }
    }

    /// Return the inner non-zero value, or `None` if the token is zero.
    ///
    /// A zero value indicates a corrupted or uninitialized SHM header.
    /// The subscriber maps `None` to [`crate::FdSidecarError::NoFdInMessage`].
    pub fn as_nonzero(self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.token)
    }
}
