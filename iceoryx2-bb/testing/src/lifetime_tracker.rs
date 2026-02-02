// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

#[cfg(feature = "std")]
use std::sync::MutexGuard;

#[cfg(not(feature = "std"))]
use iceoryx2_bb_concurrency::spin_lock::SpinLockGuard as MutexGuard;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LifetimeTracker(internal::LifetimeTracker);

impl core::ops::Deref for LifetimeTracker {
    type Target = internal::LifetimeTracker;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for LifetimeTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LifetimeTracker {
    pub fn new() -> Self {
        Self(internal::LifetimeTracker::new())
    }

    pub fn new_with_value(value: usize) -> Self {
        Self(internal::LifetimeTracker::new_with_value(value))
    }

    pub fn start_tracking() -> MutexGuard<'static, internal::LifetimeTrackingState> {
        internal::LifetimeTracker::start_tracking()
    }
}

unsafe impl ZeroCopySend for LifetimeTracker {}

mod internal {
    pub use iceoryx2_pal_testing::lifetime_tracker::LifetimeTracker;
    pub use iceoryx2_pal_testing::lifetime_tracker::LifetimeTrackingState;
}
