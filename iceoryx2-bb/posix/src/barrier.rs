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
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use iceoryx2_bb_elementary::scope_guard::ScopeGuardBuilder;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
use iceoryx2_pal_posix::*;

use crate::handle_errno;
use crate::unmovable_ipc_handle::internal::UnmovableIpcHandle;
use crate::unmovable_ipc_handle::IpcHandleState;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum BarrierCreationError {
    InsufficientMemory,
    HandleAlreadyInitialized,
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

    /// Creates a new [`Barrier`]
    pub fn create(self, handle: &BarrierHandle) -> Result<Barrier, BarrierCreationError> {
        let msg = "Unable to create barrier";

        if handle
            .reference_counter
            .compare_exchange(
                IpcHandleState::Uninitialized as _,
                IpcHandleState::PerformingInitialization as _,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_err()
        {
            fail!(from self, with BarrierCreationError::HandleAlreadyInitialized,
                "{} since the handle is already initialized with another barrier.", msg);
        }

        let mut attr =
            ScopeGuardBuilder::new( posix::pthread_barrierattr_t::new() )
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

        handle
            .is_interprocess_capable
            .store(self.is_interprocess_capable, Ordering::Relaxed);

        match unsafe {
            posix::pthread_barrier_init(handle.handle_ptr(), attr.get(), self.number_of_waiters)
        }
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
        }

        handle
            .reference_counter
            .store(IpcHandleState::Initialized as _, Ordering::Release);

        Ok(Barrier::new(handle))
    }
}

#[derive(Debug)]
pub struct BarrierHandle {
    handle: UnsafeCell<posix::pthread_barrier_t>,
    reference_counter: AtomicI64,
    is_interprocess_capable: AtomicBool,
}

unsafe impl Send for BarrierHandle {}
unsafe impl Sync for BarrierHandle {}

impl UnmovableIpcHandle for BarrierHandle {
    fn reference_counter(&self) -> &std::sync::atomic::AtomicI64 {
        &self.reference_counter
    }

    fn is_interprocess_capable(&self) -> bool {
        self.is_interprocess_capable
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for BarrierHandle {
    fn default() -> Self {
        Self {
            handle: UnsafeCell::new(posix::pthread_barrier_t::new()),
            reference_counter: AtomicI64::new(IpcHandleState::Uninitialized as _),
            is_interprocess_capable: AtomicBool::new(false),
        }
    }
}

impl BarrierHandle {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_ptr(&self) -> *mut posix::pthread_barrier_t {
        self.handle.get()
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

impl Drop for Barrier<'_> {
    fn drop(&mut self) {
        if self.handle.reference_counter.fetch_sub(1, Ordering::AcqRel) == 1 {
            if unsafe { posix::pthread_barrier_destroy(self.handle.handle_ptr()) } != 0 {
                fatal_panic!(from self, "This should never happen! Unable to remove resources.");
            }
            self.handle.reference_counter.store(-1, Ordering::Release);
        }
    }
}

impl<'a> Barrier<'a> {
    fn new(handle: &'a BarrierHandle) -> Barrier {
        Barrier { handle }
    }

    pub fn wait(&self) {
        match unsafe { posix::pthread_barrier_wait(self.handle.handle_ptr().as_mut().unwrap()) } {
            0 | posix::PTHREAD_BARRIER_SERIAL_THREAD => (),
            v => {
                fatal_panic!(from self, "This should never happen! Unable to wait on barrier ({}).", v);
            }
        }
    }
}
