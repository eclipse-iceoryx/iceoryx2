// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

//! Allows one process to monitor the state of another process. Can detect if the process is
//! [`State::Alive`], [`State::Dead`] or the existance with [`State::DoesNotExist`]. To activate
//! monitoring the process that shall be monitored must instantiate a [`MonitoringToken`]. As long
//! as the [`MonitoringToken`] is in scope the [`MonitoringMonitor`] will detect the process as
//! [`State::Alive`]. When the process crashes it will be detected as [`State::Dead`]. If the
//! process does not yet have instantiated a [`MonitoringMonitor`] the process is identified as
//! [`State::DoesNotExist`].
//!
//! # Example
//!
//! ```
//! use iceoryx2_cal::monitoring::*;
//!
//! fn monitored_process<M: Monitoring>() {
//!     let token =
//!     M::Builder::new(&FileName::new(b"unique_process_identifier").unwrap()).
//!                         token().unwrap();
//!
//!     // keep the token in scope and do what a process shall do
//!
//!     // process can no longer be monitored
//!     drop(token);
//! }
//!
//! fn watching_process<M: Monitoring>() {
//!     let monitor = M::Builder::new(&FileName::new(b"unique_process_identifier").unwrap()).
//!                         monitor().unwrap();
//!
//!     match monitor.state().unwrap() {
//!         State::Alive => println!("process is alive"),
//!         State::Dead => println!("process is dead"),
//!         State::DoesNotExist => println!("process does not exist"),
//!     }
//! }
//!
//! fn cleaning_process<M: Monitoring>() {
//!     let cleaner = match M::Builder::new(&FileName::new(b"unique_process_identifier")
//!                         .unwrap()).cleaner() {
//!         Ok(cleaner) => cleaner,
//!         Err(MonitoringCreateCleanerError::AlreadyOwnedByAnotherInstance) => {
//!             // someone is already cleaning up for us - perfect :)
//!             return;
//!         }
//!         Err(MonitoringCreateCleanerError::InstanceStillAlive) => {
//!             // whoopsie, the monitored instance is not dead
//!             return;
//!         }
//!         Err(e) => {
//!             // usual error handling
//!             return;
//!         }
//!     };
//!
//!     // cleanup all stale resources of the dead process
//!     drop(cleaner);
//! }

//! ```

use core::fmt::Debug;

pub use iceoryx2_bb_container::semantic_string::SemanticString;
pub use iceoryx2_bb_system_types::file_name::FileName;

pub use crate::{
    named_concept::NamedConcept, named_concept::NamedConceptBuilder,
    named_concept::NamedConceptMgmt,
};

pub mod file_lock;
pub mod process_local;
pub mod recommended;
#[doc(hidden)]
pub mod testing;

/// Represents the state of a monitored process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Alive,
    Dead,
    DoesNotExist,
}

/// Represents the possible errors that can occur when a new [`MonitoringToken`] is created with
/// [`MonitoringBuilder::token()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringCreateTokenError {
    InsufficientPermissions,
    AlreadyExists,
    InternalError,
}

/// Represents the possible errors that can occur when a new [`MonitoringCleaner`] is created with
/// [`MonitoringBuilder::cleaner()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringCreateCleanerError {
    Interrupt,
    InstanceStillAlive,
    AlreadyOwnedByAnotherInstance,
    DoesNotExist,
    InternalError,
}

/// Represents the possible errors that can occur when a new [`MonitoringMonitor`] is created with
/// [`MonitoringBuilder::monitor()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringCreateMonitorError {
    InsufficientPermissions,
    Interrupt,
    InternalError,
}

/// Represents the possible errors that can occur when the [`State`] is acquired via
/// [`MonitoringMonitor::state()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringStateError {
    Interrupt,
    InternalError,
}

/// The token enables a process to be monitored by another process.
pub trait MonitoringToken: NamedConcept {}

/// The cleaner owns the remains of a dead process and is the only one that is allowed to clean up
/// those resources.
pub trait MonitoringCleaner: NamedConcept {
    /// Abandons the [`MonitoringCleaner`] without removing the underlying [`Monitoring`] concept. This is useful
    /// when another process tried to cleanup the stale resources of the dead process but is unable
    /// to due to insufficient permissions.
    fn abandon(self);
}

/// The monitor allows to monitor another process that has instantiated a [`MonitoringToken`]
pub trait MonitoringMonitor: NamedConcept {
    /// Returns the current [`State`] of the monitored process. On failure it returns
    /// [`MonitoringStateError`].
    fn state(&self) -> Result<State, MonitoringStateError>;
}

/// Creates either a [`MonitoringToken`] or instantiates a [`MonitoringMonitor`] that can monitor
/// the state of a token.
pub trait MonitoringBuilder<T: Monitoring>: NamedConceptBuilder<T> {
    /// Creates a new [`MonitoringToken`] on success or returns a [`MonitoringCreateTokenError`]
    /// on failure.
    fn token(self) -> Result<T::Token, MonitoringCreateTokenError>;

    /// Instantiates a [`MonitoringMonitor`] to monitor a [`MonitoringToken`]
    fn monitor(self) -> Result<T::Monitor, MonitoringCreateMonitorError>;

    /// Instantiates a [`MonitoringCleaner`]. If it could be instantiated successfully the owner is
    /// allowed to remove all stale resources from the former dead process.
    fn cleaner(self) -> Result<T::Cleaner, MonitoringCreateCleanerError>;
}

/// Concept that allows to monitor a process from within another process. The process must hereby
/// instantiate a [`MonitoringToken`] with [`MonitoringBuilder`] so that it can be monitored with
/// the [`MonitoringMonitor`].
pub trait Monitoring: NamedConceptMgmt + Sized {
    type Token: MonitoringToken;
    type Monitor: MonitoringMonitor;
    type Cleaner: MonitoringCleaner;
    type Builder: MonitoringBuilder<Self>;

    /// Returns the default suffix that shall be used for every [`MonitoringToken`].
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".monitor") }
    }
}
