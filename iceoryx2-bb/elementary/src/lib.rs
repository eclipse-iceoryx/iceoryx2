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

//! Contains basic constructs which do not have any kind of dependency.

#[macro_use]
pub mod enum_gen;

/// A strong type that represents the alignment part of [`std::alloc::Layout`]
pub mod alignment;
pub mod allocator;

#[cfg(feature = "testing")]
pub mod bump_allocator;
pub mod lazy_singleton;
pub mod math;
pub mod owning_pointer;
pub mod package_version;
pub mod placement_default;
pub mod pointer_trait;
pub mod relocatable_container;
pub mod relocatable_ptr;
pub mod scope_guard;
pub mod static_assert;
pub mod unique_id;

/// Defines how a callback based iteration shall progress after the calling the callback. Either
/// stop the iteration with [`CallbackProgression::Stop`] or continue with
/// [`CallbackProgression::Continue`].
///
/// ```rust
/// use iceoryx2_bb_elementary::CallbackProgression;
///
/// fn iterate_over_something<F: FnMut(usize) -> CallbackProgression>(mut callback: F) {
///     for i in 0..123 {
///         match callback(i) {
///             CallbackProgression::Stop => break,
///             CallbackProgression::Continue => continue
///         }
///     }
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CallbackProgression {
    /// Stops the iteration
    Stop,
    /// Continues the iteration
    Continue,
}
