// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Non-Linux stub for [`FdPassingConnection`].
//!
//! Every method returns [`Error::UnsupportedPlatform`].
#![cfg(not(target_os = "linux"))]

use std::os::fd::{BorrowedFd, OwnedFd};

use super::{Error, FdPassingConnection, Result};

/// Stub connection type for non-Linux platforms.
///
/// All methods return [`Error::UnsupportedPlatform`].
#[allow(dead_code)]
pub struct NonLinux;

impl FdPassingConnection for NonLinux {
    fn send_with_fd(&self, _fd: BorrowedFd<'_>, _len: u64, _token: u64) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64, u64)>> {
        Err(Error::UnsupportedPlatform)
    }

    fn send_release_ack(&self, _token: u64) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    fn recv_release_ack(&self) -> Result<Option<u64>> {
        Err(Error::UnsupportedPlatform)
    }
}
