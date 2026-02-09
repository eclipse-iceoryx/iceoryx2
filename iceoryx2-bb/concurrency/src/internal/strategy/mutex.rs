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
pub struct Mutex(internal::Mutex);

impl Mutex {
    pub fn new() -> Self {
        Self(internal::Mutex::new())
    }
}

impl Default for Mutex {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for Mutex {
    type Target = internal::Mutex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Mutex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod internal {
    pub use iceoryx2_pal_concurrency_sync::strategy::mutex::Mutex;
}
