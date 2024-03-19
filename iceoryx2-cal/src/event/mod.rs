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

pub mod id_tracker;
pub mod process_local;
pub mod signal;
pub mod unix_datagram_socket;

use std::{fmt::Debug, time::Duration};

pub use crate::named_concept::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
pub use iceoryx2_bb_system_types::file_name::*;
pub use iceoryx2_bb_system_types::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierNotifyError {
    FailedToDeliverSignal,
    TriggerIdOutOfBounds,
    InternalFailure,
}

impl std::fmt::Display for NotifierNotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for NotifierNotifyError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierCreateError {
    DoesNotExist,
    InsufficientPermissions,
    InternalFailure,
}

impl std::fmt::Display for NotifierCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for NotifierCreateError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListenerWaitError {
    ContractViolation,
    InternalFailure,
}

impl std::fmt::Display for ListenerWaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ListenerWaitError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListenerCreateError {
    AlreadyExists,
    InsufficientPermissions,
    InternalFailure,
}

impl std::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ListenerCreateError {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TriggerId(u64);

impl TriggerId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

pub trait Notifier: NamedConcept + Debug {
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError>;
}

pub trait NotifierBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
    fn open(self) -> Result<T::Notifier, NotifierCreateError>;
}

pub trait Listener: NamedConcept + Debug {
    fn try_wait(&self) -> Result<Option<TriggerId>, ListenerWaitError>;
    fn timed_wait(&self, timeout: Duration) -> Result<Option<TriggerId>, ListenerWaitError>;
    fn blocking_wait(&self) -> Result<Option<TriggerId>, ListenerWaitError>;
}

pub trait ListenerBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
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
}
