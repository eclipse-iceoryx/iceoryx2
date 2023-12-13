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

//! A building block to handle resources that need to be explicitly cleaned up.
//!
//! Useful when building higher level abstractions of low level hardware/OS resources.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_elementary::scope_guard::*;
//!
//! fn acquire_resource() -> u64 { 123 }
//! fn release_resource(value: u64) { println!("release resource {}", value); }
//! fn do_stuff_with_resource(value: u64) { println!("do stuff with resource {}", value); }
//!
//! pub enum ResourceCreationFailure {
//!     OutOfResources
//! }
//!
//! pub fn do_things_with_resources() -> Result<(), ResourceCreationFailure> {
//!     let resource = ScopeGuardBuilder::new(acquire_resource())
//!                     .on_init(|resource| {
//!                         if *resource == 0 {
//!                             return Err(ResourceCreationFailure::OutOfResources);
//!                         }
//!
//!                         println!("acquired resource: {}", resource);
//!                         Ok(())
//!                     })
//!                     .on_drop(|resource| {
//!                         release_resource(*resource);
//!                     }).create()?;
//!
//!     do_stuff_with_resource(*resource.get());
//!
//!     // resource goes out of scope and `release_resource` is called automatically
//!     Ok(())
//! }
//! ```

/// The builder to create a [`ScopeGuard`].
pub struct ScopeGuardBuilder<T, E, Finit: FnOnce(&mut T) -> Result<(), E>, Fdrop: FnOnce(&mut T)> {
    value: T,
    on_drop: Option<Fdrop>,
    on_init: Option<Finit>,
}

impl<T, E, Finit: FnOnce(&mut T) -> Result<(), E>, Fdrop: FnOnce(&mut T)>
    ScopeGuardBuilder<T, E, Finit, Fdrop>
{
    /// Creates a new scope guard be providing the initial value of the guarded object
    pub fn new(value: T) -> Self {
        ScopeGuardBuilder {
            value,
            on_drop: None,
            on_init: None,
        }
    }

    /// Callback which is called directly after create. Can be used to verify if the resource
    /// was acquired correctly. Must return a [`Result`] of type `Result<(), E>`
    pub fn on_init(mut self, on_init: Finit) -> Self {
        self.on_init = Some(on_init);
        self
    }

    /// Callback which is called when the [`ScopeGuard`] goes out of scope. It shall be used to
    /// clean up the acquired resource.
    pub fn on_drop(mut self, on_drop: Fdrop) -> Self {
        self.on_drop = Some(on_drop);
        self
    }

    /// Creates a new scope guard. If the provided callback in [`ScopeGuardBuilder::on_init()`]
    /// succeeds it returns the [`ScopeGuard`] otherwise the error value which was returned by the
    /// init function.
    pub fn create(mut self) -> Result<ScopeGuard<T, Fdrop>, E> {
        if self.on_init.is_some() {
            self.on_init.unwrap()(&mut self.value)?;
        }

        Ok(ScopeGuard {
            value: self.value,
            on_drop: self.on_drop,
        })
    }
}

/// A guard which calls a callback to cleanup resources when it goes out of scope. Can be created
/// with the [`ScopeGuardBuilder`].
pub struct ScopeGuard<T, F: FnOnce(&mut T)> {
    value: T,
    on_drop: Option<F>,
}

impl<T, F: FnOnce(&mut T)> ScopeGuard<T, F> {
    /// Returns a mutable reference to the underlying object
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Returns a reference to the underlying object
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T, F: FnOnce(&mut T)> Drop for ScopeGuard<T, F> {
    fn drop(&mut self) {
        (self.on_drop.take().unwrap())(&mut self.value);
    }
}
