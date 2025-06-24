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

//! The [`ArcSyncPolicy`] can make types threadsafe in a configurable manner. Depending on what
//! implementation is used, the contained type can become threadsafe and implement [`Send`] and
//! [`Sync`] or when the implementation does not support it the resulting type will implement
//! neither [`Send`] nor [`Sync`].
//!
//! It can be used like a mutex and is not inter-process capable.
//!
//! # Example
//!
//! ## Generic Case
//!
//! ```
//! use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
//!
//! fn example<Policy: ArcSyncPolicy<u64>>() {
//!     let my_thing = Policy::new(1234).unwrap();
//!     assert!(*my_thing.lock() == 1234);
//! }
//! ```
//!
//! ## Mutex-Protected Version, implement [`Send`] and [`Sync`]
//!
//! ```
//! use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
//! type Policy = iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<u64>;
//!
//! fn my_concurrent_function<T: ArcSyncPolicy<u64> + Send + Sync>(value: &T) {}
//!
//! let my_thing = Policy::new(1234).unwrap();
//! my_concurrent_function(&my_thing);
//! ```
//!
//! ## SingleThreaded Version, does not implement [`Send`] and [`Sync`]
//!
//! ```compile_fail
//! use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
//! type Policy = iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<u64>;
//!
//! fn my_concurrent_function<T: ArcSyncPolicy<u64> + Send + Sync>(value: &T) {}
//!
//! let my_thing = Policy::new(1234).unwrap();
//! // fails here since this policy does not implement `Send` or `Sync`
//! my_concurrent_function(&my_thing);
//! ```
pub mod mutex_protected;
pub mod single_threaded;

#[cfg(doctest)]
mod single_threaded_compile_tests;

use core::{fmt::Debug, ops::Deref};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ArcSyncPolicyCreationError {
    InsufficientResources,
    InternalFailure,
}

/// The [`LockGuard`] provides access to the underlying object.
pub trait LockGuard<'parent, T: Send>: Deref<Target = T> {}

/// The actual [`ArcSyncPolicy`] concept trait.
pub trait ArcSyncPolicy<T: Send>: Sized + Clone + Debug {
    type LockGuard<'parent>: LockGuard<'parent, T>
    where
        Self: 'parent,
        T: 'parent;

    /// Creates a new [`ArcSyncPolicy`] and moves the provided value into the newly created object.
    /// On failure it returns an [`ArcSyncPolicyCreationError`].
    fn new(value: T) -> Result<Self, ArcSyncPolicyCreationError>;

    /// Lock-operation that returns a [`LockGuard`] on success to gain access to the underlying
    /// value.
    fn lock(&self) -> Self::LockGuard<'_>;
}
