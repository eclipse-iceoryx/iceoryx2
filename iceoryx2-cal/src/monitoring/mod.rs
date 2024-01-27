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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_system_types::file_name::FileName;

use crate::{
    named_concept::NamedConcept, named_concept::NamedConceptBuilder,
    named_concept::NamedConceptMgmt,
};

pub mod file_lock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Alive,
    Dead,
    DoesNotExist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringCreateTokenError {
    InsufficientPermissions,
    AlreadyExists,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringCreateMonitorError {
    InsufficientPermissions,
    Interrupt,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringStateError {
    Interrupt,
    InternalError,
}

pub trait MonitoringToken: NamedConcept {}

pub trait MonitoringMonitor: NamedConcept {
    fn state(&self) -> Result<State, MonitoringStateError>;
}

pub trait MonitoringBuilder<T: Monitoring>: NamedConceptBuilder<T> {
    fn create(self) -> Result<T::Token, MonitoringCreateTokenError>;
    fn monitor(self) -> Result<T::Monitor, MonitoringCreateMonitorError>;
}

pub trait Monitoring: NamedConceptMgmt + Sized {
    type Token: MonitoringToken;
    type Monitor: MonitoringMonitor;
    type Builder: MonitoringBuilder<Self>;

    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".monitor") }
    }
}
