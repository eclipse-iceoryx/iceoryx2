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

extern crate alloc;

use crate::{config::Config, service::Service};
use alloc::format;
use iceoryx2_bb_concurrency::atomic::AtomicU32;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::dynamic_storage::*;
use iceoryx2_cal::named_concept::*;
use iceoryx2_log::fail;
use iceoryx2_log::fatal_panic;

const GLOBAL_MGMT_NAME: FileName = unsafe { FileName::new_unchecked_const(b"node") };

#[derive(Debug, Default)]
struct State {
    node_counter: AtomicU32,
}

pub(crate) struct GlobalManagementSegment<S: Service> {
    storage: S::PersistentDynamicStorage<State>,
}

impl<S: Service> GlobalManagementSegment<S> {
    pub fn open_or_create(global_config: &Config) -> Result<Self, DynamicStorageOpenOrCreateError> {
        let origin = "GlobalManagementSegment::open_or_create()";
        let msg = "Unable to open or create the management segment";
        let config = Self::dynamic_storage_config(global_config);
        let storage = match <<S::PersistentDynamicStorage<State> as DynamicStorage<State>>::Builder<
            '_,
        > as NamedConceptBuilder<S::PersistentDynamicStorage<State>>>::new(
            &GLOBAL_MGMT_NAME
        )
        .has_ownership(false)
        .config(&config)
        .timeout(global_config.global.creation_timeout)
        .open_or_create(State::default())
        {
            Ok(storage) => storage,
            Err(e) => {
                fail!(from origin, with e,
                    "{msg} since the underlying persistent dynamic storage could not be opened. [{e:?}]");
            }
        };

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
    ) -> <S::PersistentDynamicStorage<State> as NamedConceptMgmt>::Configuration {
        let mut suffix = global_config.global.node.global_mgmt_suffix;
        let ver = PackageVersion::get();
        fatal_panic!(
            from "dynamic_storage_config()",
            when suffix.insert_bytes(0, format!(".{}_{}_{}", ver.major(), ver.minor(), ver.patch()).as_bytes()),
            "This should never happen! Failed to added package version suffix to global management segment.");

        <S::PersistentDynamicStorage<State> as NamedConceptMgmt>::Configuration::default()
            .prefix(&global_config.global.prefix)
            .suffix(&suffix)
            .path_hint(global_config.global.root_path())
    }
}
