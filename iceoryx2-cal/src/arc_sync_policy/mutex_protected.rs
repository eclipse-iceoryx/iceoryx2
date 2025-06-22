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

use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use std::sync::Arc;

use iceoryx2_bb_posix::mutex::{Handle, Mutex, MutexBuilder, MutexGuard, MutexHandle};

use crate::arc_sync_policy::{ArcSyncPolicy, LockGuard};

pub struct Guard<'parent, T: Send + Debug> {
    guard: MutexGuard<'parent, T>,
}

impl<T: Send + Debug> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: Send + Debug> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<'parent, T: Send + Debug> LockGuard<'parent, T> for Guard<'parent, T> {}

pub struct MutexProtected<T: Send + Debug> {
    handle: Arc<MutexHandle<T>>,
}

unsafe impl<T: Send + Debug> Send for MutexProtected<T> {}
unsafe impl<T: Send + Debug> Sync for MutexProtected<T> {}

impl<T: Send + Debug> ArcSyncPolicy<T> for MutexProtected<T> {
    type LockGuard<'parent>
        = Guard<'parent, T>
    where
        Self: 'parent,
        T: 'parent;

    fn new(value: T) -> Self {
        let handle = Arc::new(MutexHandle::new());
        MutexBuilder::new()
            .is_interprocess_capable(false)
            .create(value, &handle)
            .unwrap();

        Self { handle }
    }

    fn lock<'this>(&'this self) -> Self::LockGuard<'this> {
        Guard {
            guard: unsafe {
                core::mem::transmute(Mutex::from_handle(&self.handle).lock().unwrap())
            },
        }
    }
}
