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

#[doc(hidden)]
pub mod blackboard;

use core::ptr::NonNull;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

/// Represents resources a service could use and have to be cleaned up when no owners
/// are left
pub trait ServiceResource: Abandonable {
    /// Acquires the ownership of the additional resources. When the objects go out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);
}

#[derive(Debug)]
pub(crate) struct NoResource;
impl ServiceResource for NoResource {
    fn acquire_ownership(&self) {}
}

impl Abandonable for NoResource {
    unsafe fn abandon_in_place(_this: NonNull<Self>) {}
}
