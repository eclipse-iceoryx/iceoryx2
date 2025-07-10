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

use alloc::sync::Arc;
use core::{fmt::Debug, ops::Deref};

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::mutex::{
    Handle, Mutex, MutexBuilder, MutexCreationError, MutexGuard, MutexHandle, MutexType,
};

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

impl<'parent, T: Send + Debug> LockGuard<'parent, T> for Guard<'parent, T> {}

#[derive(Debug)]
pub struct MutexProtected<T: Send + Debug> {
    handle: Arc<MutexHandle<T>>,
}

impl<T: Send + Debug> Clone for MutexProtected<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

unsafe impl<T: Send + Debug> Send for MutexProtected<T> {}
unsafe impl<T: Send + Debug> Sync for MutexProtected<T> {}

impl<T: Send + Debug> ArcSyncPolicy<T> for MutexProtected<T> {
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
                            when unsafe { Mutex::from_handle(&self.handle) }.lock(),
                            "This should never happen! Failed to lock the underlying mutex ({e:?}).")
                    }
                }
        }
    }
}
