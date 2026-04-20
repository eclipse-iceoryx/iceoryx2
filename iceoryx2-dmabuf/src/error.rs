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

use iceoryx2::port::{LoanError, ReceiveError, SendError};

/// Discriminant for [`FdSidecarError::Iceoryx`] indicating which iceoryx2
/// operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceoryxErrorKind {
    /// Node creation via `NodeBuilder::create` failed.
    NodeCreate,
    /// Service open/create (`open_or_create`) failed.
    Service,
    /// Port builder (`publisher_builder` / `subscriber_builder`) failed.
    PortBuilder,
}

/// All errors that can be returned by the iceoryx2-dmabuf side-channel.
#[derive(Debug)]
#[non_exhaustive]
pub enum FdSidecarError {
    /// A Unix-domain socket or ancillary-data operation failed.
    SideChannelIo(std::io::Error),
    /// The connecting peer's effective UID does not match the publisher's UID.
    PeerUidMismatch {
        /// UID reported by `SO_PEERCRED` on the accepted socket.
        peer_uid: u32,
        /// UID of the publisher process.
        expected_uid: u32,
    },
    /// The `recvmsg` call returned no file descriptor in the ancillary data.
    NoFdInMessage,
    /// The token in the iceoryx2 sample's user-header does not match the token
    /// received over the side-channel.
    TokenMismatch {
        /// Token extracted from the iceoryx2 sample user-header.
        expected: u64,
        /// Token delivered over the Unix-domain socket.
        got: u64,
    },
    /// The 64-bit sequence counter wrapped to zero — the service must be
    /// restarted to reset the counter.
    TokenExhausted,
    /// This build target does not support `SCM_RIGHTS` fd passing (non-Linux).
    UnsupportedPlatform,
    /// An iceoryx2 loan (slot allocation) operation failed.
    IceoryxLoan(LoanError),
    /// An iceoryx2 publish operation failed.
    IceoryxPublish(SendError),
    /// An iceoryx2 receive operation failed.
    IceoryxReceive(ReceiveError),
    /// Iceoryx2 node/service/port error encountered during `create()`.
    Iceoryx {
        /// Which iceoryx2 operation failed.
        kind: IceoryxErrorKind,
        /// Error message from the iceoryx2 error type.
        msg: String,
    },
    /// A DMA-BUF CPU-access ioctl failed.
    ///
    /// This variant is reserved for future helpers that invoke
    /// `MappedDmaBuf::read/write`; today the user calls those directly
    /// and receives `dma_buf::BufferError` from the upstream crate.
    #[cfg(all(target_os = "linux", feature = "dma-buf"))]
    DmaBuf(dma_buf::BufferError),
}

impl core::fmt::Display for FdSidecarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SideChannelIo(e) => write!(f, "side-channel I/O error: {e}"),
            Self::PeerUidMismatch {
                peer_uid,
                expected_uid,
            } => write!(
                f,
                "peer UID mismatch: got {peer_uid}, expected {expected_uid}"
            ),
            Self::NoFdInMessage => write!(f, "no file descriptor carried with message"),
            Self::TokenMismatch { expected, got } => {
                write!(f, "token mismatch: expected {expected}, got {got}")
            }
            Self::TokenExhausted => {
                write!(f, "64-bit token counter exhausted; restart the service")
            }
            Self::UnsupportedPlatform => {
                write!(f, "SCM_RIGHTS fd passing is not supported on this platform")
            }
            Self::IceoryxLoan(e) => write!(f, "iceoryx2 loan error: {e}"),
            Self::IceoryxPublish(e) => write!(f, "iceoryx2 publish error: {e}"),
            Self::IceoryxReceive(e) => write!(f, "iceoryx2 receive error: {e}"),
            Self::Iceoryx { kind, msg } => write!(f, "iceoryx2 {kind:?} error: {msg}"),
            #[cfg(all(target_os = "linux", feature = "dma-buf"))]
            Self::DmaBuf(e) => write!(f, "DMA-BUF ioctl error: {e}"),
        }
    }
}

impl core::error::Error for FdSidecarError {}

/// Convenience alias for `Result<T, FdSidecarError>`.
pub type Result<T> = core::result::Result<T, FdSidecarError>;
