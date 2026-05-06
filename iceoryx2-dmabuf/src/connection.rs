// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Standalone fd-passing connection over Unix-domain sockets with SCM_RIGHTS.
//!
//! NOT a sub-trait of `iceoryx2_cal::zero_copy_connection::ZeroCopyConnection`
//! per Task 0a spike findings (cal trait surface assumes PointerOffset, which
//! cannot losslessly carry RawFd). See `specs/arch-dmabuf-service-variant.adoc`
//! decision D1 (post-spike pivot) and D3 (wire format).
//!
//! # Wire format (v2)
//!
//! ## Forward direction (publisher → subscriber, fd-carrying)
//!
//! ```text
//! [8B payload_len u64 LE][8B token u64 LE][SCM_RIGHTS ancillary: 1 fd]
//! ```
//!
//! ## Back direction (subscriber → publisher, ack, NO ancillary)
//!
//! ```text
//! [8B magic u64 LE  (low-32 = 0x4D4F5346, high-32 = 0)][8B token u64 LE]
//! ```
//!
//! Both directions flow on the **same** `UnixStream` (bidirectional).
//! Disambiguation: a frame with `SCM_RIGHTS` ancillary present is a forward fd
//! frame; a frame with no ancillary is a back-channel ack.
//!
//! `MAGIC = 0x4D4F_5346` is the ASCII bytes `b"MOSF"` interpreted as a
//! little-endian `u32`.

use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(not(target_os = "linux"))]
pub(crate) mod non_linux;

// Re-export concrete types needed by integration tests.
#[cfg(target_os = "linux")]
pub use linux::{Linux, LinuxPublisher, LinuxSubscriber};
#[cfg(not(target_os = "linux"))]
pub use non_linux::NonLinux;

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
    /// Ack frame magic mismatch — received an unexpected magic value.
    BadMagic {
        /// The magic value that was received.
        got: u32,
    },
    /// Ancillary data presence/absence violated protocol expectations.
    ///
    /// Returned when a frame arrives with ancillary where none was expected
    /// (forward fd frame received on a back-channel poll), or vice-versa.
    ProtocolDrift,
    /// Non-blocking operation had no data available (EAGAIN/EWOULDBLOCK).
    ///
    /// Callers typically map this to `Ok(None)`.
    WouldBlock,
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
            Self::BadMagic { got } => {
                write!(f, "fd-passing ack bad magic: got 0x{got:08X}")
            }
            Self::ProtocolDrift => {
                write!(
                    f,
                    "fd-passing protocol drift: unexpected ancillary presence"
                )
            }
            Self::WouldBlock => write!(f, "fd-passing would block (no data)"),
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

/// Standalone trait for fd-passing connections (wire format v2).
///
/// Implemented by `linux::LinuxPublisher` and `linux::LinuxSubscriber`
/// on Linux, and [`non_linux::NonLinux`] elsewhere.
///
/// Each forward message carries one file descriptor via `SCM_RIGHTS` ancillary
/// data together with an 8-byte payload length and an 8-byte caller token.
///
/// Back-channel ack messages carry no ancillary data; they use a 16-byte frame
/// with a magic sentinel and the matching token.
pub trait FdPassingConnection: Sized {
    /// Send a single fd with an associated byte length and caller token to
    /// every connected peer.
    ///
    /// Wire format: `[8B len LE][8B token LE][SCM_RIGHTS fd]`.
    ///
    /// Returns `Ok(())` even if a peer disconnected during the call
    /// (broken-pipe pruning removes the dead peer silently).
    fn send_with_fd(&self, fd: BorrowedFd<'_>, len: u64, token: u64) -> Result<()>;

    /// Non-blocking receive of a forward fd frame.
    ///
    /// Returns `Ok(None)` when no message is currently queued.
    /// Returns `Ok(Some((fd, len, token)))` on success.
    /// Returns `Err(Error::Disconnected)` when the publisher closed.
    /// Returns `Err(Error::ProtocolDrift)` if a frame arrives without ancillary
    /// (subscriber would receive an ack frame instead of a forward frame).
    fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64, u64)>>;

    /// Best-effort: send a "buffer released" ack carrying `token` to the peer.
    ///
    /// Wire format: `[8B magic_u64 LE][8B token LE]` (no ancillary).
    /// The magic value is `0x4D4F_5346` (`b"MOSF"` as little-endian `u32`) in
    /// the low 32 bits; the high 32 bits are zero.
    ///
    /// Returns `Ok(())` even on `EAGAIN` (subscriber send buffer full). Callers
    /// must tolerate occasional dropped acks.
    fn send_release_ack(&self, token: u64) -> Result<()>;

    /// Non-blocking poll for one back-channel ack frame.
    ///
    /// Returns `Ok(None)` if no ack is currently queued.
    /// Returns `Ok(Some(token))` when an ack is drained.
    /// Returns `Err(Error::BadMagic)` if the magic sentinel does not match.
    /// Returns `Err(Error::ProtocolDrift)` if a frame WITH ancillary arrives on
    /// the back-channel (a forward fd frame arrived out-of-order).
    fn recv_release_ack(&self) -> Result<Option<u64>>;
}
