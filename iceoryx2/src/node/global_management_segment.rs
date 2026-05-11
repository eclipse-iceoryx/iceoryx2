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

use crate::{config::Config, service::Service};
use iceoryx2_bb_concurrency::atomic::AtomicU32;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::dynamic_storage::*;
use iceoryx2_cal::named_concept::*;

const GLOBAL_MGMT_NAME: FileName = unsafe { FileName::new_unchecked_const(b"mgmt") };

#[derive(Debug, Default)]
struct State {
    node_counter: AtomicU32,
}

pub(crate) struct GlobalManagementSegment<S: Service> {
    storage: S::DynamicStorage<State>,
}

impl<S: Service> GlobalManagementSegment<S> {
    pub fn open_or_create(global_config: &Config) -> Result<Self, DynamicStorageOpenOrCreateError> {
        let config = Self::dynamic_storage_config(global_config);
        let storage =
            <<S::DynamicStorage<State> as DynamicStorage<State>>::Builder<'_> as NamedConceptBuilder<S::DynamicStorage<State>>>::new(&GLOBAL_MGMT_NAME)
                .has_ownership(false)
                .config(&config)
                .timeout(global_config.global.creation_timeout)
                .open_or_create(State::default()).unwrap();

        Ok(Self { storage })
    }

    pub fn increment_node_counter(&self) -> u32 {
        self.storage
            .get()
            .node_counter
            .fetch_add(1, Ordering::Relaxed)
    }

    fn dynamic_storage_config(
        global_config: &Config,
    ) -> <S::DynamicStorage<State> as NamedConceptMgmt>::Configuration {
        <S::DynamicStorage<State> as NamedConceptMgmt>::Configuration::default()
            .prefix(&global_config.global.prefix)
            .path_hint(global_config.global.root_path())
    }
}
