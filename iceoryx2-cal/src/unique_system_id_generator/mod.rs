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

// TODO: better name
pub mod blub;

#[repr(C)]
#[derive(
    Debug,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    ZeroCopySend,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UniqueId {
    id: u128,
}

impl UniqueId {
    pub unsafe fn from_value(value: u128) -> Self {
        Self { id: value }
    }

    pub fn value(&self) -> u128 {
        self.id
    }
}

#[repr(C)]
#[derive(ZeroCopySend)]
pub struct Entity {
    pub name: StaticString<255>, // equivalent to MAX_SERVICE_NAME_LENGTH
    pub id: u128,                // smaller? needed?
                                 // additional parent_id? no, port names have to be unique
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniqueSystemIdGeneratorError {
    GenerationError,
}

impl core::fmt::Display for UniqueSystemIdGeneratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UniqueSystemIdGeneratorError::{self:?}")
    }
}

impl core::error::Error for UniqueSystemIdGeneratorError {}

pub trait UniqueSystemIdGenerator /*ZeroCopySend + From<UniqueId>*/ {
    fn generate(entity: &Entity) -> Result<UniqueId, UniqueSystemIdGeneratorError>;
}
