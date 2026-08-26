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

// TODO: documentation, tests

// contract: unique_value member is unique within on process
// system-wide unique when UniqueIdGenerator implementation guarantees this, e.g. iceoryx2_bb_posix::UniqueSystemId
// that increments a static atomic counter and the pid
#[repr(C)]
#[derive(
    Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, ZeroCopySend,
)]
pub struct UniqueId {
    payload_value: u64,
    unique_value: u64,
}

impl UniqueId {
    // safety: see contract
    pub unsafe fn from_value(value: u128) -> Self {
        Self {
            payload_value: (value >> 64) as u64,
            unique_value: value as u64,
        }
    }

    pub fn value(&self) -> u128 {
        (self.payload_value as u128) << 64 | (self.unique_value as u128)
    }

    pub fn payload_value(&self) -> u64 {
        self.payload_value
    }

    pub fn unique_value(&self) -> u64 {
        self.unique_value
    }
}

#[repr(C)]
#[derive(ZeroCopySend)]
pub struct Entity {
    pub name: StaticString<255>, // equivalent to MAX_SERVICE_NAME_LENGTH; TODO: own type?
    pub id: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniqueIdGeneratorError {
    GenerationError,
    NotImplemented,
}

impl core::fmt::Display for UniqueIdGeneratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UniqueIdGeneratorError::{self:?}")
    }
}

impl core::error::Error for UniqueIdGeneratorError {}

// TODO: separate types for entity_name and entity_id?
pub struct UniqueIdBuilder<'a, T: UniqueIdGenerator> {
    entity_name: StaticString<255>,
    entity_id: u128,
    id_storage: Option<&'a T::IdStorage>,
    id_hint: Option<T::IdStorage>,
}

impl<'a, T: UniqueIdGenerator> UniqueIdBuilder<'a, T> {
    pub fn new(name: &StaticString<255>) -> Self {
        Self {
            entity_name: *name,
            entity_id: 0,
            id_storage: None,
            id_hint: None,
        }
    }

    pub fn id_storage(mut self, id_storage: &'a T::IdStorage) -> Self {
        self.id_storage = Some(id_storage);
        self
    }

    pub fn id_hint(mut self, id_hint: T::IdStorage) -> Self {
        self.id_hint = Some(id_hint);
        self
    }

    pub fn create(mut self, entity_id: u128) -> Result<UniqueId, UniqueIdGeneratorError> {
        self.entity_id = entity_id;
        T::generate(self)
    }
}

pub trait UniqueIdGenerator: From<UniqueId> {
    type IdStorage: ZeroCopySend + Send + Sync + Debug + 'static; // TODO: better name

    fn generate(builder: UniqueIdBuilder<Self>) -> Result<UniqueId, UniqueIdGeneratorError>;

    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::pid()", with UniqueIdGeneratorError::NotImplemented,
            "pid() is not implemented");
    }

    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::creation_time()",
            with UniqueIdGeneratorError::NotImplemented, "creation_time() not implemented");
    }

    fn adapt_id_storage(_: &Self::IdStorage) -> Result<Self::IdStorage, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::adapt_id_storage()",
            with UniqueIdGeneratorError::NotImplemented, "adapt_id_storage() not implemented");
    }
}
