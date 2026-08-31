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

//! Generates a system-wide [`UniqueId`]. For a detailed documentation, see [`UniqueSystemId`].

pub use iceoryx2_bb_posix::unique_system_id::*;

use crate::node::global_management_segment::GlobalManagementSegment;
pub use crate::unique_id_generator::*;

impl From<UniqueSystemIdCreationError> for UniqueIdGeneratorError {
    fn from(_: UniqueSystemIdCreationError) -> Self {
        UniqueIdGeneratorError::GenerationError
    }
}

impl UniqueIdGenerator for UniqueSystemId {
    /// Generates a system-wide unique ID by using the process ID and the incremented static
    /// atomic counter from the global management segment.
    fn generate<Service: service::Service>(
        entity: Entity,
        config: &Config,
    ) -> Result<UniqueId, UniqueIdGeneratorError> {
        let id = match entity {
            Entity::Node(_) => {
                let node_counter = match GlobalManagementSegment::<Service>::open_or_create(config)
                {
                    Ok(mgmt) => mgmt.increment_node_counter(),
                    Err(e) => {
                        fail!(from "UniqueIdGenerator::generate()",
                        with UniqueIdGeneratorError::GenerationError,
                        "Unable to generate unique id since the global management segment could not be opened. {e:?}");
                    }
                };
                UniqueSystemId::from_counter(node_counter)?
            }
            _ => UniqueSystemId::new()?,
        };
        Ok(unsafe { UniqueId::from_raw_id(id.value()) })
    }

    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        Ok(self.pid())
    }

    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        Ok(self.creation_time())
    }
}

impl From<UniqueId> for UniqueSystemId {
    fn from(value: UniqueId) -> Self {
        Self::from(value.value())
    }
}
