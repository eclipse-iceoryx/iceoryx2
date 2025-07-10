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

pub mod common;
pub mod id_tracker;
pub mod process_local_socketpair;
pub mod recommended;
pub mod sem_bitset_posix_shared_memory;
pub mod sem_bitset_process_local;
pub mod signal_mechanism;
pub mod unix_datagram_socket;

use core::{fmt::Debug, time::Duration};

pub use crate::named_concept::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
pub use iceoryx2_bb_system_types::file_name::*;
pub use iceoryx2_bb_system_types::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierNotifyError {
    Interrupt,
    FailedToDeliverSignal,
    TriggerIdOutOfBounds,
    Disconnected,
    InternalFailure,
}

impl core::fmt::Display for NotifierNotifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for NotifierNotifyError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierCreateError {
    Interrupt,
    DoesNotExist,
    InsufficientPermissions,
    VersionMismatch,
    InitializationNotYetFinalized,
    InternalFailure,
}

impl core::fmt::Display for NotifierCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for NotifierCreateError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListenerWaitError {
    ContractViolation,
    InternalFailure,
    InterruptSignal,
}

impl core::fmt::Display for ListenerWaitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ListenerWaitError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListenerCreateError {
    AlreadyExists,
    InsufficientPermissions,
    InternalFailure,
}

impl core::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ListenerCreateError {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TriggerId(usize);

impl TriggerId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn as_value(&self) -> usize {
        self.0
    }
}

pub trait Notifier: NamedConcept + Debug + Send {
    fn trigger_id_max(&self) -> TriggerId {
        TriggerId::new(usize::MAX)
    }
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError>;
}

pub trait NotifierBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
    fn timeout(self, timeout: Duration) -> Self;
    fn open(self) -> Result<T::Notifier, NotifierCreateError>;
}

pub trait Listener: NamedConcept + Debug + Send {
    const IS_FILE_DESCRIPTOR_BASED: bool = false;

    fn try_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError>;
    fn timed_wait_one(&self, timeout: Duration) -> Result<Option<TriggerId>, ListenerWaitError>;
    fn blocking_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError>;

    fn try_wait_all<F: FnMut(TriggerId)>(&self, callback: F) -> Result<(), ListenerWaitError>;
    fn timed_wait_all<F: FnMut(TriggerId)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError>;
    fn blocking_wait_all<F: FnMut(TriggerId)>(&self, callback: F) -> Result<(), ListenerWaitError>;
}

pub trait ListenerBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
    fn trigger_id_max(self, id: TriggerId) -> Self;
    fn create(self) -> Result<T::Listener, ListenerCreateError>;
}

pub trait Event: Sized + NamedConceptMgmt + Debug {
    type Notifier: Notifier;
    type NotifierBuilder: NotifierBuilder<Self>;
    type Listener: Listener;
    type ListenerBuilder: ListenerBuilder<Self>;

    /// The default suffix of every event
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".event") }
    }

    fn has_trigger_id_limit() -> bool {
        false
    }
}
