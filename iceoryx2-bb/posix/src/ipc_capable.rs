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

use core::{cell::UnsafeCell, sync::atomic::Ordering};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

pub(crate) mod internal {
    use core::fmt::Debug;

    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum Capability {
        InterProcess,
        ProcessLocal,
    }

    /// Stores an OS handle to some resource that is also inter-process capable, like a unnamed
    /// semaphore or mutex handle.
    pub struct HandleStorage<T> {
        handle: UnsafeCell<T>,
        is_inter_process_capable: IoxAtomicBool,
        is_initialized: IoxAtomicBool,
    }

    unsafe impl<T> Send for HandleStorage<T> {}
    unsafe impl<T> Sync for HandleStorage<T> {}

    impl<T> Debug for HandleStorage<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "HandleStorage<{}> {{ is_interprocess_capable: {}, is_initialized: {} }}",
                core::any::type_name::<T>(),
                self.is_inter_process_capable.load(Ordering::Relaxed),
                self.is_initialized.load(Ordering::Relaxed),
            )
        }
    }

    impl<T> Drop for HandleStorage<T> {
        fn drop(&mut self) {
            self.is_initialized.store(false, Ordering::Relaxed);
        }
    }

    impl<T> HandleStorage<T> {
        /// Creates a new HandleStorage with a predefined handle that must be not initialized.
        pub fn new(handle: T) -> Self {
            Self {
                handle: UnsafeCell::new(handle),
                is_initialized: IoxAtomicBool::new(false),
                is_inter_process_capable: IoxAtomicBool::new(false),
            }
        }

        /// Returns true if the [`Handle`] is initialized, otherwise false.
        pub fn is_initialized(&self) -> bool {
            self.is_initialized.load(Ordering::Relaxed)
        }

        /// Initializes the handle via a provided initializer callback. If the initializer returns
        /// true the underlying handle is marked as initialized otherwise it is still uninitialized.
        ///
        /// # Safety
        ///  * The handle must be uninitialized
        ///  * Must not be shared with other threads before calling [`IpcCapable::initialize()`]
        ///
        pub unsafe fn initialize<E, F: FnOnce(*mut T) -> Result<Capability, E>>(
            &self,
            initializer: F,
        ) -> Result<(), E> {
            debug_assert!(
                !self.is_initialized.load(Ordering::Relaxed),
                "The handle must be uninitialized before it can be initialized."
            );

            self.is_inter_process_capable.store(
                initializer(self.handle.get())? == Capability::InterProcess,
                Ordering::Relaxed,
            );

            // does not need to sync any memory since the construct is not allowed to
            // be shared with any other thread before it is initialized
            // -> Ordering::Relaxed
            self.is_initialized.store(true, Ordering::Relaxed);

            Ok(())
        }

        pub fn is_inter_process_capable(&self) -> bool {
            self.is_inter_process_capable.load(Ordering::Relaxed)
        }

        /// Releases the underlying resources of the handle via the cleanup callback.
        ///
        /// # Safety
        ///  * The handle must be initialized
        ///  * Must not be used concurrently. Only one thread - the one that calls
        ///    [`IpcCapable::cleanup()`] - is allowed to operate on the [`IpcCapable`].
        ///
        pub unsafe fn cleanup<F: FnOnce(&mut T)>(&self, cleanup: F) {
            debug_assert!(
                self.is_initialized.load(Ordering::Relaxed),
                "The handle must be initialized before it can be cleaned up."
            );

            cleanup(self.get());

            // does not need to sync any memory since the construct is not allowed to
            // be shared with any other thread before it is initialized
            // -> Ordering::Relaxed
            self.is_initialized.store(false, Ordering::Relaxed);
        }

        /// Returns a mutable reference to the underlying handle.
        ///
        /// # Safety
        ///  * The handle must be initialized
        ///
        #[allow(clippy::mut_from_ref)]
        pub unsafe fn get(&self) -> &mut T {
            debug_assert!(
                self.is_initialized.load(Ordering::Relaxed),
                "The handle must be initialized before it can be acquired."
            );

            unsafe { &mut *self.handle.get() }
        }
    }

    pub trait IpcConstructible<'a, T: Handle> {
        fn new(handle: &'a T) -> Self;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HandleState {
    Initialized,
    Uninitialized,
}

/// Represents a handle that is in general inter-process capable.
pub trait Handle: Send + Sync {
    fn new() -> Self;
    fn is_inter_process_capable(&self) -> bool;
    fn is_initialized(&self) -> bool;
}

/// Represents struct that can be configured for inter-process use.
pub trait IpcCapable<'a, T: Handle>: internal::IpcConstructible<'a, T> + Sized {
    /// Returns true if the object is interprocess capable, otherwise false
    fn is_interprocess_capable(&self) -> bool;

    /// Creates an IPC Capable object from its handle.
    ///
    /// # Safety
    ///   * The handle must be initialized
    ///   * The handle must be ipc capable, see [`Handle::is_inter_process_capable()`].
    ///   * The handle must not be cleaned up while it is used by the object
    ///
    unsafe fn from_ipc_handle(handle: &'a T) -> Self {
        debug_assert!(
            handle.is_initialized(),
            "The handle must be initialized before it can be used to construct an object."
        );

        debug_assert!(
            handle.is_inter_process_capable(),
            "The handle must be interprocess capable to be used for constructing an ipc capable object."
        );

        Self::new(handle)
    }
}
