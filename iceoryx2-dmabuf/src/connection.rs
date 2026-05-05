// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Standalone fd-passing connection over Unix-domain sockets with SCM_RIGHTS.
//!
//! NOT a sub-trait of `iceoryx2_cal::zero_copy_connection::ZeroCopyConnection`
//! per Task 0a spike findings (cal trait surface assumes PointerOffset, which
//! cannot losslessly carry RawFd). See `specs/arch-dmabuf-service-variant.adoc`
//! decision D1 (post-spike pivot) and D3 (wire format).
//!
//! # Wire format (v1)
//!
//! ```text
//! [8B payload_len u64 LE][8B reserved u64 LE][SCM_RIGHTS ancillary: 1 fd]
//! ```
//!
//! Per `sendmsg(2)` call. The caller hands `(fd, len)`; the subscriber gets
//! `(OwnedFd, len)`.
//!
//! Task 4b widens this for back-channel + token + Meta-inline.

use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(not(target_os = "linux"))]
pub mod non_linux;

/// Errors returned by [`FdPassingConnection`] operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Underlying I/O failure.
    Io(io::Error),
    /// Remote peer closed the connection.
    Disconnected,
    /// Header was shorter than expected.
    Truncated {
        /// Bytes actually received.
        got: usize,
        /// Bytes expected.
        want: usize,
    },
    /// The message carried no `SCM_RIGHTS` fd in ancillary data.
    NoFdInMessage,
    /// Called on a platform without UDS fd-passing support.
    UnsupportedPlatform,
    /// Internal mutex was poisoned.
    LockPoisoned,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "fd-passing I/O: {e}"),
            Self::Disconnected => write!(f, "fd-passing peer disconnected"),
            Self::Truncated { got, want } => {
                write!(f, "fd-passing truncated: got {got} bytes, want {want}")
            }
            Self::NoFdInMessage => write!(f, "fd-passing message had no SCM_RIGHTS fd"),
            Self::UnsupportedPlatform => write!(f, "fd-passing unsupported on this platform"),
            Self::LockPoisoned => write!(f, "fd-passing internal lock poisoned"),
        }
    }
}

impl core::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Convenience `Result` alias for this module.
pub type Result<T> = core::result::Result<T, Error>;

/// Standalone trait for fd-passing connections.
///
/// Implemented by [`linux::LinuxPublisher`] and [`linux::LinuxSubscriber`]
/// on Linux, and [`non_linux::NonLinux`] elsewhere.
///
/// Each message carries one file descriptor via `SCM_RIGHTS` ancillary data
/// together with an 8-byte payload length.
pub trait FdPassingConnection: Sized {
    /// Send a single fd with an associated byte length to every connected peer.
    ///
    /// Returns `Ok(())` even if a peer disconnected during the call
    /// (broken-pipe pruning removes the dead peer silently).
    fn send_with_fd(&self, fd: BorrowedFd<'_>, len: u64) -> Result<()>;

    /// Non-blocking receive.
    ///
    /// Returns `Ok(None)` when no message is currently queued.
    /// Returns `Ok(Some((fd, len)))` on success.
    /// Returns `Err(Error::Disconnected)` when the publisher closed.
    fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64)>>;
}
