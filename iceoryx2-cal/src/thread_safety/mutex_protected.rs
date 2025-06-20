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
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::sync::Arc;

use iceoryx2_bb_posix::mutex::{Handle, Mutex, MutexBuilder, MutexGuard, MutexHandle};

use crate::thread_safety::{LockGuard, ThreadSafety};

pub struct Guard<'handle, T: Debug> {
    guard: MutexGuard<'handle, T>,
}

impl<T: Debug> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: Debug> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<T: Debug> LockGuard<T> for Guard<'_, T> {}

pub struct MutexProtected<'a, T: Debug> {
    handle: Arc<MutexHandle<T>>,
    _data: PhantomData<&'a ()>,
}

impl<'a, T: Debug + 'a> ThreadSafety<T> for MutexProtected<'a, T> {
    type LockGuard = Guard<'a, T>;

    fn new(value: T) -> Self {
        let handle = Arc::new(MutexHandle::new());
        MutexBuilder::new()
            .is_interprocess_capable(true)
            .create(value, &handle)
            .unwrap();

        Self {
            handle,
            _data: PhantomData,
        }
    }

    fn lock(&self) -> Self::LockGuard {
        Guard {
            guard: Mutex::from_handle(&self.handle).lock().unwrap(),
        }
    }
}
