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

//! A [`MemoryLock`] excludes a specific range in the memory from paging, e.g. makes it non-swapable.
//! This may increase the runtime and reduce jitter in a realtime applications since the specific
//! region inside the memory is not moved into the swap space.

use crate::handle_errno;
use crate::system_configuration::SystemInfo;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MemoryLockCreationError {
    InvalidAddressRange,
    UnableToLock,
    AddressNotAMultipleOfThePageSize,
    InsufficientPermissions,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MemoryLockAllError {
    UnableToLock,
    WouldExceedMainMemory,
    InsufficientPermissions,
    UnknownError(i32),
}

enum_gen! {
    /// The MemoryLockError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward MemoryLockError as more generic return value when a method
    /// returns a MemoryLock***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    MemoryLockError
  generalization:
    LockFailed <= MemoryLockCreationError; MemoryLockAllError
}

/// Required by [`MemoryLock::lock_all()`]. Defines what memory should be locked, either the
/// current memory or all memory that will become mapped in the future.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LockMode {
    LockAllPagesCurrentlyMapped = posix::MCL_CURRENT,
    LockAllPagesThatBecomeMapped = posix::MCL_FUTURE,
}

/// A MemoryLock excludes a specific range in the memory from paging, e.g. makes it non-swapable.
/// This may increase the runtime and reduce jitter in a realtime applications since the specific
/// region inside the memory is not moved into the swap space.
#[derive(Debug)]
pub struct MemoryLock {
    address: *const posix::void,
    len: usize,
}

impl MemoryLock {
    /// Locks a provided memory region. As soon as the memory lock goes out of scope the memory
    /// region is unlocked again.
    ///
    /// # Safety
    ///   * the memory range [address, len] be a valid address during the lifetime of [`MemoryLock`]
    ///
    pub unsafe fn new(
        address: *const posix::void,
        len: usize,
    ) -> Result<MemoryLock, MemoryLockCreationError> {
        if unsafe { posix::mlock(address, len) } == 0 {
            return Ok(MemoryLock { address, len });
        }

        let msg = "Unable to lock memory";
        handle_errno!(MemoryLockCreationError, from "MemoryLock::new",
            Errno::ENOMEM => (InvalidAddressRange, "{} since the specified range beginning from {:#16X} with a length of {} is not contained in the valid mapped pages in the address spaces of the current process.", msg, address as usize, len),
            Errno::EAGAIN => (UnableToLock, "{} since some or all memory could not be locked.", msg),
            Errno::EINVAL => (AddressNotAMultipleOfThePageSize, "{} since the address {:#16X} is not a multiple of the page-size {}.", msg, address as usize, SystemInfo::PageSize.value()),
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Locks all pages with defined [`LockMode`]
    pub fn lock_all(mode: LockMode) -> Result<(), MemoryLockAllError> {
        if unsafe { posix::mlockall(mode as i32) } != -1 {
            return Ok(());
        }

        let msg = "Unable to lock all memory with the mode";
        handle_errno!(MemoryLockAllError, from "MemoryLock::lock_all",
            Errno::EAGAIN => (UnableToLock, "{} {:?} since the memory could not be locked.", msg, mode),
            Errno::ENOMEM => (WouldExceedMainMemory, "{} {:?} due to insufficient main memory.", msg, mode),
            Errno::EPERM => (InsufficientPermissions, "{} {:?} due to insufficient permissions.", msg, mode),
            v => (UnknownError(v as i32), "{} {:?} since an unknown error occurred ({}).", msg, mode, v)
        );
    }

    /// Unlocks all pages.
    pub fn unlock_all() {
        unsafe { posix::munlockall() };
    }
}

impl Drop for MemoryLock {
    fn drop(&mut self) {
        if unsafe { posix::munlock(self.address, self.len) } != 0 {
            fatal_panic!(from self, "This should never happen! Unable to unlock memory region.");
        }
    }
}
