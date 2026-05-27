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

pub mod semaphore;

use crate::event::event_state::{EventActivation, EventId, EventStateActivateError};
use crate::event::{NamedConcept, NamedConceptBuilder};
use crate::event::{NamedConceptMgmt, event_state::EventState};
use core::fmt::Debug;
use core::time::Duration;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_system_types::file_name::FileName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerCreateError {}

impl core::fmt::Display for TriggerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerCreateError::{self:?}")
    }
}

impl core::error::Error for TriggerCreateError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerOpenError {}

impl core::fmt::Display for TriggerOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerOpenError::{self:?}")
    }
}

impl core::error::Error for TriggerOpenError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerWaitError {}

impl core::fmt::Display for TriggerWaitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerWaitError::{self:?}")
    }
}

impl core::error::Error for TriggerWaitError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerNotifyError {}

impl core::fmt::Display for TriggerNotifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TriggerNotifyError::{self:?}")
    }
}

impl core::error::Error for TriggerNotifyError {}

impl From<EventStateActivateError> for TriggerNotifyError {
    fn from(value: EventStateActivateError) -> Self {
        todo!()
    }
}

pub trait TriggerWaiter<E: EventState>: NamedConcept + Debug + Abandonable + Send {
    const IS_FILE_DESCRIPTOR_BASED: bool = false;

    fn try_wait<F: FnMut(EventActivation)>(&self, callback: F) -> Result<u64, TriggerWaitError>;
    fn timed_wait<F: FnMut(EventActivation)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<u64, TriggerWaitError>;
    fn blocking_wait<F: FnMut(EventActivation)>(
        &self,
        callback: F,
    ) -> Result<u64, TriggerWaitError>;
}

pub trait TriggerHandle<E: EventState>: NamedConcept + Debug + Abandonable + Send {
    fn event_id_max(&self) -> EventId;
    fn notify(&self, event_id: EventId) -> Result<(), TriggerNotifyError>;
}

pub trait TriggerWaiterBuilder<E: EventState, T: Trigger<E>>:
    NamedConceptBuilder<T> + Debug
{
    fn timeout(self, timeout: Duration) -> Self;
    fn open(self) -> Result<T::Handle, TriggerOpenError>;
}

pub trait TriggerHandleBuilder<E: EventState, T: Trigger<E>>:
    NamedConceptBuilder<T> + Debug
{
    fn event_id_max(self, id: EventId) -> Self;
    fn create(self) -> Result<T::Waiter, TriggerCreateError>;
}

pub trait Trigger<E: EventState>: Sized + NamedConceptMgmt + Debug {
    type Waiter: TriggerWaiter<E>;
    type WaiterBuiler;
    type Handle: TriggerHandle<E>;
    type HandleBuilder;

    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".trigger") }
    }
}
