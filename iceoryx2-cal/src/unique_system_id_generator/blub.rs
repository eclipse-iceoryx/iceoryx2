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

pub use iceoryx2_bb_posix::unique_system_id::*;

pub use crate::unique_system_id_generator::*;

// TODO: better error handling
impl From<UniqueSystemIdCreationError> for UniqueSystemIdGeneratorError {
    fn from(_: UniqueSystemIdCreationError) -> Self {
        UniqueSystemIdGeneratorError::GenerationError
    }
}

impl UniqueSystemIdGenerator for UniqueSystemId {
    fn generate(_entity: &Entity) -> Result<UniqueId, UniqueSystemIdGeneratorError> {
        let id = UniqueSystemId::new()?;
        Ok(unsafe { UniqueId::from_value(id.value()) })
    }
}

// impl From<UniqueId> for UniqueSystemId {
//     fn from(value: UniqueId) -> Self {
//         Self::from(value.value())
//     }
// }
