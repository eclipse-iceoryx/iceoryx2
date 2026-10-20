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

//! A [`Trigger`] is a process-local wake-up mechanism. The [`TriggerBuilder`] creates a
//! connected pair of an [`Activator`] and an [`Observer`]. Every thread can activate the
//! trigger with [`Activator::activate()`]. The [`Observer`] can be attached to a
//! [`Reactor`](crate::reactor::Reactor), which wakes up as soon as the trigger was
//! activated, and [`Observer::reset()`] consumes the activations.
//!
//! Activations are not counted. Any number of activations before the next
//! [`Observer::reset()`] are reported as one.
//!
//! # Example
//!
//! ```
//! use iceoryx2_cal::trigger::*;
//! use iceoryx2_cal::trigger::socket_pair::SocketPair;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let (activator, observer) = <SocketPair as Trigger>::Builder::new().create()?;
//!
//! activator.activate()?;
//! activator.activate()?;
//!
//! assert!(observer.reset()?);
//! assert!(!observer.reset()?);
//! # Ok(())
//! # }
//! ```

pub mod socket_pair;

use core::fmt::Debug;

use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;

/// Defines the failures that can occur when a [`Trigger`] is created with
/// [`TriggerBuilder::create()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerCreateError {
    /// The process or the system ran out of the resources a trigger requires.
    InsufficientResources,
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for TriggerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerCreateError::{self:?}")
    }
}

impl core::error::Error for TriggerCreateError {}

/// Defines the failures that can occur in [`Activator::activate()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerActivateError {
    /// The [`Observer`] of the trigger no longer exists.
    Disconnected,
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for TriggerActivateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerActivateError::{self:?}")
    }
}

impl core::error::Error for TriggerActivateError {}

/// Defines the failures that can occur in [`Observer::reset()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerResetError {
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for TriggerResetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerResetError::{self:?}")
    }
}

impl core::error::Error for TriggerResetError {}

/// The activating side of a [`Trigger`]. It can be shared between threads, so that every
/// thread can activate the trigger.
pub trait Activator: Debug + Send + Sync + Abandonable {
    /// Activates the trigger. The [`Observer`] wakes up every
    /// [`Reactor`](crate::reactor::Reactor) it is attached to, until it is reset with
    /// [`Observer::reset()`]. Activating an already activated trigger succeeds.
    fn activate(&self) -> Result<(), TriggerActivateError>;
}

/// The observing side of a [`Trigger`]. It can be attached to a
/// [`Reactor`](crate::reactor::Reactor) and shared between threads.
pub trait Observer: Debug + Send + Sync + Abandonable + SynchronousMultiplexing {
    /// Consumes all activations of the trigger. Returns `true` when the trigger was
    /// activated since the last reset, otherwise `false`.
    fn reset(&self) -> Result<bool, TriggerResetError>;
}

/// Creates a connected pair of an [`Activator`] and an [`Observer`] of a [`Trigger`].
pub trait TriggerBuilder<T: Trigger>: Debug {
    /// Creates a new builder.
    fn new() -> Self;

    /// Creates the [`Activator`] and the [`Observer`] of a new trigger.
    fn create(self) -> Result<(T::Activator, T::Observer), TriggerCreateError>;
}

/// A process-local wake-up mechanism that consists of an [`Activator`] and an
/// [`Observer`].
pub trait Trigger: Debug + Sized {
    /// The activating side of the trigger.
    type Activator: Activator;
    /// The observing side of the trigger.
    type Observer: Observer;
    /// The builder that creates the trigger.
    type Builder: TriggerBuilder<Self>;
}
