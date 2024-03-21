// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StorageState {
    Initialized,
    Uninitialized,
}

/// Represents a handle that is in general inter-process capable.
pub trait Handle: Send + Sync {
    fn is_inter_process_capable(&self) -> bool;
}

/// Stores an OS handle to some resource that is also inter-process capable, like a unnamed
/// semaphore or mutex handle.
pub struct HandleStorage<T: Handle> {
    handle: UnsafeCell<T>,
    state: AtomicBool,
}

impl<T: Handle> HandleStorage<T> {
    /// Creates a new HandleStorage with a predefined handle that must be not initialized.
    pub fn new(handle: T) -> Self {
        Self {
            handle: UnsafeCell::new(handle),
            state: AtomicBool::new(false),
        }
    }

    /// Returns the current state of the underlying handle
    pub fn state(&self) -> StorageState {
        match self.state.load(Ordering::Relaxed) {
            false => StorageState::Uninitialized,
            true => StorageState::Initialized,
        }
    }

    /// Initializes the handle via a provided initializer callback. If the initializer returns
    /// true the underlying handle is marked as initialized otherwise it is still uninitialized.
    ///
    /// # Safety
    ///  * The handle must be uninitialized
    ///
    pub unsafe fn initialize<F: FnOnce(*mut T) -> bool>(&self, initializer: F) {
        debug_assert!(
            self.state.load(Ordering::Relaxed) == false,
            "The handle must be uninitialized before it can be initialized."
        );

        if initializer(self.handle.get()) {
            self.state.store(true, Ordering::Relaxed);
        }
    }

    /// Releases the underlying resources of the handle via the cleanup callback.
    ///
    /// # Safety
    ///  * The handle must be initialized
    ///
    pub unsafe fn cleanup<F: FnOnce(&mut T)>(&self, cleanup: F) {
        debug_assert!(
            self.state.load(Ordering::Relaxed) == true,
            "The handle must be initialized before it can be cleaned up."
        );

        cleanup(self.get());
        self.state.store(false, Ordering::Relaxed);
    }

    /// Returns a mutable reference to the underlying handle.
    ///
    /// # Safety
    ///  * The handle must be initialized
    ///
    pub unsafe fn get(&self) -> &mut T {
        debug_assert!(
            self.state.load(Ordering::Relaxed) == true,
            "The handle must be initialized before it can be acquired."
        );

        unsafe { &mut *self.handle.get() }
    }
}

pub(crate) mod internal {
    use super::*;

    pub trait IpcConstructible<'a, T: Handle> {
        fn new(handle: &'a HandleStorage<T>) -> Self;
    }
}

/// Every struct that implements this trait is inter-process capable without any restriction,
/// meaning there is no configuration/variation that is not inter-process capable.
pub trait PocIpcCapable<'a, T: Handle>: internal::IpcConstructible<'a, T> + Sized {
    /// Returns the ownership state of construct
    fn has_ownership(&self) -> bool;

    /// Acquires the ownership. When the object goes out of scope the resources stored inside
    /// the handle will be cleaned up.
    fn acquire_ownership(&self);

    /// Releases the ownership. No resources stored inside the handle will be released.
    fn release_ownership(&self);

    /// # Safety
    ///   * The handle must be initialized
    ///   * The handle must be ipc capable, see [`Handle::is_inter_process_capable()`].
    ///   * The handle will not be cleaned up while it is used by the object
    unsafe fn from_ipc_handle(handle: &'a HandleStorage<T>) -> Self {
        debug_assert!(
            handle.state() == StorageState::Initialized,
            "The handle must be initialized before it can be used to construct an object."
        );

        debug_assert!(
            handle.get().is_inter_process_capable(),
            "The handle must be interprocess capable to be used for constructing an ipc capable object."
        );

        Self::new(handle)
    }
}
