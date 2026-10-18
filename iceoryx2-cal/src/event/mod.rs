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

pub mod common;
pub mod event_state;
pub mod implementations;
pub mod recommended;
pub mod trigger;

use crate::clock::ClockListenerSchedule;
pub use crate::named_concept::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
pub use iceoryx2_bb_system_types::file_name::*;
pub use iceoryx2_bb_system_types::path::Path;

pub use implementations::*;

use crate::event::event_state::{EventActivation, EventState, EventStateActivateError};
use alloc::sync::Arc;
use core::{fmt::Debug, time::Duration};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

pub const DEFAULT_MAX_EVENT_ID: EventId = EventId::new(u8::MAX as usize);

#[derive(ZeroCopySend, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(C)]
pub struct EventId(usize);

impl EventId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn as_value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotifierNotifyError {
    Interrupt,
    BufferIsFull,
    EventIdOutOfBounds,
    InsufficientPermissions,
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
pub enum NotifierOpenError {
    Interrupt,
    DoesNotExist,
    InsufficientPermissions,
    VersionMismatch,
    InitializationNotYetFinalized,
    InternalFailure,
}

impl core::fmt::Display for NotifierOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for NotifierOpenError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListenerWaitError {
    ContractViolation,
    InternalFailure,
    ClockScheduleCapacityExceeded,
    InterruptSignal,
    NotificationError,
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
    RootDirectoryCreationFailure,
    InsufficientPermissions,
    Interrupt,
    InternalFailure,
}

impl core::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ListenerCreateError {}

impl From<EventStateActivateError> for NotifierNotifyError {
    fn from(value: EventStateActivateError) -> Self {
        match value {
            EventStateActivateError::EventIdOutOfBounds => NotifierNotifyError::EventIdOutOfBounds,
        }
    }
}

pub trait ListenerWaitInterface {
    fn try_wait(&self, callback: &mut dyn FnMut(EventActivation))
    -> Result<u64, ListenerWaitError>;
    fn timed_wait(
        &self,
        callback: &mut dyn FnMut(EventActivation),
        timeout: Duration,
    ) -> Result<u64, ListenerWaitError>;
    fn blocking_wait(
        &self,
        callback: &mut dyn FnMut(EventActivation),
    ) -> Result<u64, ListenerWaitError>;
}

pub trait Listener<E: EventState>:
    NamedConcept + Debug + Abandonable + Send + ListenerWaitInterface
{
    const IS_FILE_DESCRIPTOR_BASED: bool = false;

    fn max_event_count(&self) -> u64;
}

pub trait Notifier<E: EventState>: NamedConcept + Debug + Abandonable + Send {
    fn max_event_count(&self) -> u64;
    fn event_id_max(&self) -> EventId;
    fn notify(&self, event_id: EventId) -> Result<(), NotifierNotifyError>;
}

pub trait NotifierBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
    fn timeout(self, timeout: Duration) -> Self;
    fn fail_when_buffer_is_full(self, value: bool) -> Self;
    fn open(self) -> Result<T::Notifier, NotifierOpenError>;
}

pub trait ListenerBuilder<T: Event>: NamedConceptBuilder<T> + Debug {
    fn event_id_max(self, id: EventId) -> Self;
    fn create(
        self,
        clock: Arc<dyn ClockListenerSchedule>,
    ) -> Result<T::Listener, ListenerCreateError>;
}

pub trait Event: Sized + NamedConceptMgmt + Debug {
    type State: EventState;
    type Listener: Listener<Self::State>;
    type ListenerBuilder: ListenerBuilder<Self>;
    type Notifier: Notifier<Self::State>;
    type NotifierBuilder: NotifierBuilder<Self>;

    fn does_support_persistency() -> bool;

    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".trigger") }
    }
}
