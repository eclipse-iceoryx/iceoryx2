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

use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct SpinLock<T>(internal::SpinLock<T>);

impl<T: Default> PlacementDefault for SpinLock<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(SpinLock::default());
    }
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self(internal::SpinLock::new(value))
    }

    pub fn lock(&self) -> LockResult<SpinLockGuard<'_, T>> {
        self.0.lock()
    }

    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        self.0.try_lock()
    }

    pub fn blocking_lock(&self) -> SpinLockGuard<'_, T> {
        self.0.blocking_lock()
    }
}

pub use internal::LockResult;
pub use internal::SpinLockGuard;

mod internal {
    pub use iceoryx2_pal_concurrency_sync::spin_lock::LockResult;
    pub use iceoryx2_pal_concurrency_sync::spin_lock::SpinLock;
    pub use iceoryx2_pal_concurrency_sync::spin_lock::SpinLockGuard;
}
