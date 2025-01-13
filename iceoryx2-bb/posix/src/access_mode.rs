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

//! The [`AccessMode`] describes the mode in which resources like [`crate::file::File`],
//! [`crate::shared_memory::SharedMemory`] or others should be opened.

use core::fmt::Display;
use iceoryx2_pal_posix::*;

/// Describes the mode in which resources like [`crate::file::File`],
/// [`crate::shared_memory::SharedMemory`] or others should be opened.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Default)]
pub enum AccessMode {
    /// Do not open in any mode
    None,
    #[default]
    /// Default value, open for reading only
    Read,
    /// Open for writing only
    Write,
    /// Open for reading and writing
    ReadWrite,
}

impl AccessMode {
    /// Converts itself into the C [`posix::PROT_NONE`], [`posix::PROT_READ`] or/and
    /// [`posix::PROT_WRITE`] flag pendant
    /// # Examples
    /// ```
    /// use iceoryx2_bb_posix::access_mode::*;
    /// let pflags = AccessMode::ReadWrite.as_protflag();
    /// ```
    pub fn as_protflag(&self) -> posix::int {
        match self {
            AccessMode::None => posix::PROT_NONE,
            AccessMode::Read => posix::PROT_READ,
            AccessMode::Write => posix::PROT_WRITE,
            AccessMode::ReadWrite => posix::PROT_READ | posix::PROT_WRITE,
        }
    }

    /// Converts itself into the C [`posix::O_RDONLY`], [`posix::O_WRONLY`] or
    /// [`posix::O_RDWR`] flag pendant
    /// # Examples
    /// ```
    /// use iceoryx2_bb_posix::access_mode::*;
    /// let oflags = AccessMode::ReadWrite.as_oflag();
    /// ```
    pub fn as_oflag(&self) -> posix::int {
        match self {
            AccessMode::None => 0,
            AccessMode::Read => posix::O_RDONLY,
            AccessMode::Write => posix::O_WRONLY,
            AccessMode::ReadWrite => posix::O_RDWR,
        }
    }
}

impl Display for AccessMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "AccessMode::{}",
            match self {
                AccessMode::None => "None",
                AccessMode::Read => "Read",
                AccessMode::Write => "Write",
                AccessMode::ReadWrite => "ReadWrite",
            }
        )
    }
}
