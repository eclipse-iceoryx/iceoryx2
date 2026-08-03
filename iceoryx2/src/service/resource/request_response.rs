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

use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::{event::NamedConceptMgmt, static_storage::StaticStorage};
use iceoryx2_log::warn;

use crate::{
    node::SharedNode,
    service::{
        self,
        resource::{ServiceResource, type_definition::TypeDefinition},
    },
};

pub struct RequestResponseResourceConfig<ServiceType: service::Service> {
    pub(crate) request: TypeDefinition,
    pub(crate) response: TypeDefinition,
    pub(crate) shared_node: SharedNode<ServiceType>,
}

#[derive(Debug)]
pub struct RequestResponseResources<ServiceType: service::Service> {
    request_type_definition: Option<ServiceType::StaticStorage>,
    response_type_definition: Option<ServiceType::StaticStorage>,
    path_hint: Option<Path>,
    has_ownership: AtomicBool,
}

impl<ServiceType: service::Service> Abandonable for RequestResponseResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        this.has_ownership.store(false, Ordering::Relaxed);

        if let Some(td) = this.request_type_definition.as_mut() {
            unsafe {
                ServiceType::StaticStorage::abandon_in_place(core::ptr::NonNull::from_mut(td))
            };
        }

        if let Some(td) = this.response_type_definition.as_mut() {
            unsafe {
                ServiceType::StaticStorage::abandon_in_place(core::ptr::NonNull::from_mut(td))
            };
        }
    }
}

impl<ServiceType: service::Service> Drop for RequestResponseResources<ServiceType> {
    fn drop(&mut self) {
        if let Some(path_hint) = &self.path_hint {
            drop(self.request_type_definition.take());
            drop(self.response_type_definition.take());

            if self.has_ownership.load(Ordering::Relaxed)
                && let Err(e) =
                    <ServiceType::StaticStorage as NamedConceptMgmt>::remove_path_hint(path_hint)
            {
                warn!(from self,
                    "Failed to remove resource directory: \"{path_hint}\". [{e:?}]")
            }
        }
    }
}

impl<ServiceType: service::Service> ServiceResource for RequestResponseResources<ServiceType> {
    type Config = RequestResponseResourceConfig<ServiceType>;

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
        if let Some(s) = &self.request_type_definition {
            s.acquire_ownership();
        }

        if let Some(s) = &self.response_type_definition {
            s.acquire_ownership();
        }
    }

    fn create(
        static_config: &service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, service::builder::ServiceCreateError> {
        todo!()
    }

    fn open(
        static_config: &service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, service::builder::ServiceOpenError> {
        todo!()
    }

    unsafe fn remove_stale_resources(
        config: &crate::config::Config,
        static_config: &service::static_config::StaticConfig,
    ) -> Result<(), super::RemoveStaleResourcesError> {
        todo!()
    }
}
