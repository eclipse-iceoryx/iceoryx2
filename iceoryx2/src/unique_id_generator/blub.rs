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

use iceoryx2_bb_concurrency::atomic::{AtomicU32, Ordering};
pub use iceoryx2_bb_posix::unique_system_id::*;

pub use crate::unique_id_generator::*;

impl From<UniqueSystemIdCreationError> for UniqueIdGeneratorError {
    fn from(_: UniqueSystemIdCreationError) -> Self {
        UniqueIdGeneratorError::GenerationError
    }
}

impl UniqueIdGenerator for UniqueSystemId {
    type IdStorage = AtomicU32;

    fn generate(
        builder: UniqueIdBuilder<UniqueSystemId>,
    ) -> Result<UniqueId, UniqueIdGeneratorError> {
        let id = match builder.id_hint {
            Some(counter) => UniqueSystemId::from_counter(counter.load(Ordering::Relaxed)),
            None => UniqueSystemId::new(),
        }?;
        Ok(unsafe { UniqueId::from_value(id.value()) })
    }

    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        Ok(self.pid())
    }

    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        Ok(self.creation_time())
    }

    fn adapt_id_storage(
        id_storage: &Self::IdStorage,
    ) -> Result<Self::IdStorage, UniqueIdGeneratorError> {
        Ok(AtomicU32::new(id_storage.fetch_add(1, Ordering::Relaxed)))
    }
}

impl From<UniqueId> for UniqueSystemId {
    fn from(value: UniqueId) -> Self {
        Self::from(value.value())
    }
}
