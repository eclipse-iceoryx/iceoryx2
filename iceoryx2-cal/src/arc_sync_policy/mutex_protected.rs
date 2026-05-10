// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::{fmt::Debug, ops::Deref};

use alloc::format;
use alloc::sync::Arc;

use iceoryx2_bb_posix::mutex::{
    Handle, Mutex, MutexBuilder, MutexCreationError, MutexGuard, MutexHandle, MutexType,
};
use iceoryx2_bb_testing::abandonable::{Abandonable, NonNullFromRef};
use iceoryx2_log::{fail, fatal_panic};

use crate::arc_sync_policy::{ArcSyncPolicy, ArcSyncPolicyCreationError, LockGuard};

pub struct Guard<'parent, T: Send + Debug> {
    guard: MutexGuard<'parent, T>,
}

impl<T: Send + Debug> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'parent, T: Send + Debug + Abandonable> LockGuard<'parent, T> for Guard<'parent, T> {}

#[derive(Debug)]
pub struct MutexProtected<T: Send + Debug + Abandonable> {
    handle: Arc<MutexHandle<T>>,
}

impl<T: Send + Debug + Abandonable> Clone for MutexProtected<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

unsafe impl<T: Send + Debug + Abandonable> Send for MutexProtected<T> {}
unsafe impl<T: Send + Debug + Abandonable> Sync for MutexProtected<T> {}

impl<T: Send + Debug + Abandonable> Abandonable for MutexProtected<T> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        let origin = format!("{this:?}");
        if let Some(v) = Arc::get_mut(&mut this.handle) {
            match unsafe { Mutex::from_handle(v) }.lock() {
                Ok(mut guard) => unsafe {
                    T::abandon_in_place(core::ptr::NonNull::iox2_from_mut(&mut *guard))
                },
                Err(e) => {
                    fatal_panic!(from origin,
                        "This should never happen! Failed to lock the underlying mutex ({e:?}).")
                }
            }
        } else {
            unsafe { core::ptr::drop_in_place(&mut this.handle) };
        }
    }
}

impl<T: Send + Debug + Abandonable> ArcSyncPolicy<T> for MutexProtected<T> {
    type LockGuard<'parent>
        = Guard<'parent, T>
    where
        Self: 'parent,
        T: 'parent;

    fn new(value: T) -> Result<Self, ArcSyncPolicyCreationError> {
        let msg = "Unable to create new arc_sync_policy::MutexProtected";
        let handle = Arc::new(MutexHandle::new());
        match MutexBuilder::new()
            .is_interprocess_capable(false)
            .mutex_type(MutexType::Recursive)
            .create(value, &handle)
        {
            Ok(_) => (),
            Err(MutexCreationError::InsufficientMemory)
            | Err(MutexCreationError::InsufficientResources) => {
                fail!(from format!("MutexProtected::<{}>::new()", core::any::type_name::<T>()),
                        with ArcSyncPolicyCreationError::InsufficientResources,
                        "{msg} due to insufficient resources");
            }
            Err(e) => {
                fail!(from format!("MutexProtected::<{}>::new()", core::any::type_name::<T>()),
                        with ArcSyncPolicyCreationError::InternalFailure,
                        "{msg} due to an internal failure during creation ({e:?})");
            }
        };

        Ok(Self { handle })
    }

    fn lock(&self) -> Self::LockGuard<'_> {
        Guard {
            guard:
                // handle was successfully initialized in `new()`
                match unsafe {Mutex::from_handle(&self.handle)}.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        fatal_panic!(from self,
                            "This should never happen! Failed to lock the underlying mutex ({e:?}).")
                    }
                }
        }
    }
}
