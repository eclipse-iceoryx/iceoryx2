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

use core::{fmt::Debug, hash::Hash};

use iceoryx2_bb_container::string::StaticString;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;
use serde::{Deserialize, Serialize};

// TODO: better name
pub mod blub;
pub mod recommended;

#[repr(C)]
#[repr(align(8))]
#[derive(
    Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, ZeroCopySend,
)]
pub struct UniqueId {
    high_bits: u64,
    low_bits: u64,
}

impl UniqueId {
    // TODO: unsafe?
    pub unsafe fn from_value(value: u128) -> Self {
        // TODO: UB? if so, then also wrong in bb posix unique system id
        // value and Self have same size so it should be fine
        // unsafe { core::mem::transmute(value) }
        Self {
            high_bits: (value >> 64) as u64,
            low_bits: value as u64,
        }
    }

    pub fn value(&self) -> u128 {
        (self.high_bits as u128) << 64 | (self.low_bits as u128)
    }
}

#[repr(C)]
#[derive(ZeroCopySend)]
pub struct Entity {
    pub name: StaticString<255>, // equivalent to MAX_SERVICE_NAME_LENGTH; own type?
    pub id: u128,                // smaller?
}

// TODO: rename errors + better error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniqueIdGeneratorError {
    GenerationError,
}

impl core::fmt::Display for UniqueIdGeneratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UniqueIdGeneratorError::{self:?}")
    }
}

impl core::error::Error for UniqueIdGeneratorError {}

pub struct UniqueIdBuilder {
    entity_name: StaticString<255>,
    entity_id: u128,
    counter_hint: Option<u32>,
}

impl UniqueIdBuilder {
    pub fn new(name: &StaticString<255>) -> Self {
        Self {
            entity_name: *name,
            entity_id: 0,
            counter_hint: None,
        }
    }

    pub fn counter_hint(mut self, counter: u32) -> Self {
        self.counter_hint = Some(counter);
        self
    }

    pub fn create<T: UniqueIdGenerator>(
        mut self,
        entity_id: u128,
    ) -> Result<UniqueId, UniqueIdGeneratorError> {
        self.entity_id = entity_id;
        T::generate(self)
    }
}

pub trait UniqueIdGenerator: From<UniqueId> {
    fn generate(builder: UniqueIdBuilder) -> Result<UniqueId, UniqueIdGeneratorError>;

    // TODO: better error + handling
    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::pid()", with UniqueIdGeneratorError::GenerationError, "pid() is not implemented");
    }

    // TODO: better error + handling
    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::creation_time()", with UniqueIdGeneratorError::GenerationError, "creation_time() not implemented");
    }
}
