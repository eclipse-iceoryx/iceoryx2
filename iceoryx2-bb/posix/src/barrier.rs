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

//! Inter-process capable [`Barrier`] which blocks a previously defined number of waiters
//! untilthe all the waiters reached [`Barrier::wait()`]
//!
//! # Examples
//!
//! ```
//! use iceoryx2_bb_posix::barrier::*;
//! use std::thread;
//!
//! let number_of_waiters = 2;
//! let handle = BarrierHandle::new();
//! let barrier = BarrierBuilder::new(number_of_waiters)
//!                                    .is_interprocess_capable(false)
//!                                    .create(&handle).unwrap();
//! thread::scope(|s| {
//!   s.spawn(|| {
//!     println!("Thread: waiting ...");
//!     barrier.wait();
//!     println!("Thread: lets start!");
//!   });
//!
//!   println!("main: waiting ...");
//!   barrier.wait();
//!   println!("all systems ready!");
//! });
//! ```
pub use crate::ipc_capable::{Handle, IpcCapable};

use iceoryx2_bb_elementary::scope_guard::ScopeGuardBuilder;
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;

use crate::handle_errno;
use crate::ipc_capable::internal::{Capability, HandleStorage, IpcConstructible};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum BarrierCreationError {
    InsufficientMemory,
    SystemWideBarrierLimitReached,
    UnknownError(i32),
}

/// Builder for the [`Barrier`]. The default values for number_of_waiters is 1 and it is
/// interprocess capable unless it is configured otherwise.
#[derive(Debug)]
pub struct BarrierBuilder {
    number_of_waiters: u32,
    is_interprocess_capable: bool,
}

impl BarrierBuilder {
    /// Creates a new [`BarrierBuilder`] for a [`Barrier`] which is waiting for the provided number of waiters
    pub fn new(number_of_waiters: u32) -> BarrierBuilder {
        BarrierBuilder {
            number_of_waiters,
            is_interprocess_capable: false,
        }
    }

    /// Defines if the [`Barrier`] is inter-process capable or not.
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self
    }

    fn initialize_barrier(
        &self,
        barrier: *mut posix::pthread_barrier_t,
    ) -> Result<Capability, BarrierCreationError> {
        let msg = "Unable to create barrier";

        let mut attr =
            ScopeGuardBuilder::new( posix::pthread_barrierattr_t::new_zeroed() )
                .on_init(|attr| {
                    let msg = "Unable to create barrier attributes";
                    handle_errno!(BarrierCreationError, from self,
                        errno_source unsafe { posix::pthread_barrierattr_init(attr).into() },
                        success Errno::ESUCCES => (),
                        Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                        v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
                    );})
                    .on_drop(|attr| {
                        if unsafe{ posix::pthread_barrierattr_destroy(attr)} != 0 {
                            fatal_panic!(from self, "This should never happen! Unable to cleanup barrier attribute.");
                        }})
                .create()?;

        match unsafe {
            posix::pthread_barrierattr_setpshared(
                attr.get_mut(),
                if self.is_interprocess_capable {
                    posix::PTHREAD_PROCESS_SHARED
                } else {
                    posix::PTHREAD_PROCESS_PRIVATE
                },
            )
        } {
            0 => (),
            v => {
                fatal_panic!(from self, "This should never happen! Unable to set pshared attribute ({}).",v);
            }
        }

        match unsafe { posix::pthread_barrier_init(barrier, attr.get(), self.number_of_waiters) }
            .into()
        {
            Errno::ESUCCES => (),
            Errno::ENOMEM => {
                fail!(from self, with BarrierCreationError::InsufficientMemory, "{} due to insufficient memory.", msg);
            }
            Errno::EAGAIN => {
                fail!(from self, with BarrierCreationError::SystemWideBarrierLimitReached,
                    "{} since system-wide maximum of barriers is reached.", msg
                );
            }
            v => {
                fail!(from self, with BarrierCreationError::UnknownError(v as i32),
                    "{} since an unknown error occurred ({}).", msg, v
                );
            }
        };

        match self.is_interprocess_capable {
            true => Ok(Capability::InterProcess),
            false => Ok(Capability::ProcessLocal),
        }
    }

    /// Creates a new [`Barrier`]
    pub fn create(self, handle: &BarrierHandle) -> Result<Barrier<'_>, BarrierCreationError> {
        unsafe {
            handle
                .handle
                .initialize(|barrier| self.initialize_barrier(barrier))?;
        }

        Ok(Barrier::new(handle))
    }
}

#[derive(Debug)]
pub struct BarrierHandle {
    handle: HandleStorage<posix::pthread_barrier_t>,
}

unsafe impl Send for BarrierHandle {}
unsafe impl Sync for BarrierHandle {}

impl Handle for BarrierHandle {
    fn new() -> Self {
        Self {
            handle: HandleStorage::new(posix::pthread_barrier_t::new_zeroed()),
        }
    }

    fn is_inter_process_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }

    fn is_initialized(&self) -> bool {
        self.handle.is_initialized()
    }
}

impl Drop for BarrierHandle {
    fn drop(&mut self) {
        if self.handle.is_initialized() {
            unsafe {
                self.handle.cleanup(|barrier| {
                if posix::pthread_barrier_destroy(barrier) != 0 {
                    warn!(from self,
                        "Unable to destroy barrier. Was it already destroyed by another instance in another process?");
                }
            });
            };
        }
    }
}

/// Barrier which blocks a previously defined number of waiters until all the waiters
/// reached [`Barrier::wait()`]
#[derive(Debug)]
pub struct Barrier<'a> {
    handle: &'a BarrierHandle,
}

unsafe impl Sync for Barrier<'_> {}
unsafe impl Send for Barrier<'_> {}

impl Barrier<'_> {
    pub fn wait(&self) {
        match unsafe { posix::pthread_barrier_wait(self.handle.handle.get()) } {
            0 | posix::PTHREAD_BARRIER_SERIAL_THREAD => (),
            v => {
                fatal_panic!(from self, "This should never happen! Unable to wait on barrier ({}).", v);
            }
        }
    }
}

impl<'a> IpcConstructible<'a, BarrierHandle> for Barrier<'a> {
    fn new(handle: &BarrierHandle) -> Barrier<'_> {
        Barrier { handle }
    }
}

impl<'a> IpcCapable<'a, BarrierHandle> for Barrier<'a> {
    fn is_interprocess_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}
