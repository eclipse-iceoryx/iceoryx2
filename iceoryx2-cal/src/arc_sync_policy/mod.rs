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

pub mod mutex_protected;
pub mod single_threaded;

use core::ops::Deref;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ArcSyncPolicyCreationError {
    InsufficientResources,
    InternalFailure,
}

pub trait LockGuard<'parent, T: Send>: Deref<Target = T> {}

pub trait ArcSyncPolicy<T: Send>: Sized + Clone {
    type LockGuard<'parent>: LockGuard<'parent, T>
    where
        Self: 'parent,
        T: 'parent;

    fn new(value: T) -> Result<Self, ArcSyncPolicyCreationError>;
    fn lock(&self) -> Self::LockGuard<'_>;
}
