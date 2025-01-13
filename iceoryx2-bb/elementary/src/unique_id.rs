// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Contains two building blocks to generate unique ids. Useful for parallized unit test to
//! generate names which point to OS resources or to enumerate constructs uniquely.
//!
//! # Examples
//!
//! ## [`UniqueId`]
//!
//! The [`UniqueId`] is globally unique.
//!
//! ```
//! use iceoryx2_bb_elementary::unique_id::UniqueId;
//!
//! struct MyThing {
//!     unique_id: UniqueId,
//! }
//!
//! impl MyThing {
//!     fn new() -> Self {
//!         Self {
//!             unique_id: UniqueId::new()
//!         }
//!     }
//!
//!     fn id(&self) -> u64 {
//!         self.unique_id.value()
//!     }
//! }
//! ```
//!
//! ## [`TypedUniqueId`]
//!
//! The [`TypedUniqueId`] is unique for a given type.
//!
//! ```
//! use iceoryx2_bb_elementary::unique_id::TypedUniqueId;
//!
//! struct MyThing {
//!     unique_id: TypedUniqueId<MyThing>,
//! }
//!
//! impl MyThing {
//!     fn new() -> Self {
//!         Self {
//!             unique_id: TypedUniqueId::new()
//!         }
//!     }
//!
//!     fn id(&self) -> u64 {
//!         self.unique_id.value()
//!     }
//! }
//! ```

use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

use core::{marker::PhantomData, sync::atomic::Ordering};

/// A building block to generate global unique ids
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct UniqueId {
    value: u64,
}

impl Default for UniqueId {
    fn default() -> Self {
        static COUNTER: IoxAtomicU64 = IoxAtomicU64::new(0);

        UniqueId {
            value: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl UniqueId {
    /// Creates a new unique id
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the underlying integer value of the unique id
    pub fn value(&self) -> u64 {
        self.value
    }
}

/// A building block to generate per type global unique ids. It is allowed that different types
/// have the same id but never the same type.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct TypedUniqueId<T> {
    value: u64,
    _phantom: PhantomData<T>,
}

impl<T> Default for TypedUniqueId<T> {
    fn default() -> Self {
        static COUNTER: IoxAtomicU64 = IoxAtomicU64::new(0);

        Self {
            value: COUNTER.fetch_add(1, Ordering::Relaxed),
            _phantom: PhantomData,
        }
    }
}

impl<T> TypedUniqueId<T> {
    /// Creates a new unique id
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the underlying integer value of the unique id
    pub fn value(&self) -> u64 {
        self.value
    }
}
