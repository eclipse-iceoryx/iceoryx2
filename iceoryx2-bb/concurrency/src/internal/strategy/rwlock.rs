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

#[derive(Debug)]
#[repr(transparent)]
pub struct RwLockReaderPreference(internal::RwLockReaderPreference);

impl RwLockReaderPreference {
    pub fn new() -> Self {
        Self(internal::RwLockReaderPreference::new())
    }
}

impl Default for RwLockReaderPreference {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for RwLockReaderPreference {
    type Target = internal::RwLockReaderPreference;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RwLockReaderPreference {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RwLockWriterPreference(internal::RwLockWriterPreference);

impl RwLockWriterPreference {
    pub fn new() -> Self {
        Self(internal::RwLockWriterPreference::new())
    }
}

impl Default for RwLockWriterPreference {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for RwLockWriterPreference {
    type Target = internal::RwLockWriterPreference;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RwLockWriterPreference {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod internal {
    pub use iceoryx2_pal_concurrency_sync::strategy::rwlock::RwLockReaderPreference;
    pub use iceoryx2_pal_concurrency_sync::strategy::rwlock::RwLockWriterPreference;
}
