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

pub mod process_local;
pub mod unix_datagram_socket;

use std::{fmt::Debug, time::Duration};

pub use crate::named_concept::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
use iceoryx2_bb_posix::config::TEMP_DIRECTORY;
pub use iceoryx2_bb_system_types::file_name::FileName;
pub use iceoryx2_bb_system_types::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierNotifyError {
    FailedToDeliverSignal,
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

/// The default suffix of every event
pub const DEFAULT_SUFFIX: FileName = unsafe { FileName::new_unchecked(b".event") };

/// The default prefix of every event
pub const DEFAULT_PREFIX: FileName = unsafe { FileName::new_unchecked(b"iox2_") };

/// The default path hint for every event
pub const DEFAULT_PATH_HINT: Path = TEMP_DIRECTORY;

pub trait TriggerId: Debug + Copy {}

impl TriggerId for u64 {}
impl TriggerId for u32 {}
impl TriggerId for u16 {}
impl TriggerId for u8 {}

pub trait Notifier<Id: TriggerId>: NamedConcept + Debug {
    fn notify(&self, id: Id) -> Result<(), NotifierNotifyError>;
}

pub trait NotifierBuilder<Id: TriggerId, T: Event<Id>>: NamedConceptBuilder<T> + Debug {
    fn open(self) -> Result<T::Notifier, NotifierCreateError>;
}

pub trait Listener<Id: TriggerId>: NamedConcept + Debug {
    fn try_wait(&self) -> Result<Option<Id>, ListenerWaitError>;
    fn timed_wait(&self, timeout: Duration) -> Result<Option<Id>, ListenerWaitError>;
    fn blocking_wait(&self) -> Result<Option<Id>, ListenerWaitError>;
}

pub trait ListenerBuilder<Id: TriggerId, T: Event<Id>>: NamedConceptBuilder<T> + Debug {
    fn create(self) -> Result<T::Listener, ListenerCreateError>;
}

pub trait Event<Id: TriggerId>: Sized + NamedConceptMgmt + Debug {
    type Notifier: Notifier<Id>;
    type NotifierBuilder: NotifierBuilder<Id, Self>;
    type Listener: Listener<Id>;
    type ListenerBuilder: ListenerBuilder<Id, Self>;
}
