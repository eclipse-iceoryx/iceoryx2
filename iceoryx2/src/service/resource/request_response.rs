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
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::{event::NamedConceptMgmt, static_storage::StaticStorage};
use iceoryx2_log::{fail, warn};

use crate::{
    node::SharedNode,
    service::{
        self,
        builder::{ServiceCreateError, ServiceOpenError},
        resource::{
            ServiceResource,
            type_definition::{TypeDefinition, TypeDefinitionStorage},
        },
    },
};

const REQUEST_TYPE_DEFINITION: FileName = unsafe { FileName::new_unchecked_const(b"request") };
const RESPONSE_TYPE_DEFINITION: FileName = unsafe { FileName::new_unchecked_const(b"response") };

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
        let request_storage = Self::create_type_storage(
            &resource_config.request,
            &REQUEST_TYPE_DEFINITION,
            resource_config.shared_node.config(),
            static_config,
        )?;
        let response_storage = Self::create_type_storage(
            &resource_config.response,
            &RESPONSE_TYPE_DEFINITION,
            resource_config.shared_node.config(),
            static_config,
        )?;

        let mut path_hint = request_storage.as_ref().map(|v| v.path_hint);
        if path_hint.is_none() {
            path_hint = response_storage.as_ref().map(|v| v.path_hint);
        }

        Ok(Self {
            request_type_definition: request_storage.map(|v| v.storage),
            response_type_definition: response_storage.map(|v| v.storage),
            path_hint,
            has_ownership: AtomicBool::new(false),
        })
    }

    fn open(
        static_config: &service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, service::builder::ServiceOpenError> {
        let request_storage = Self::open_type_storage(
            &resource_config.request,
            &REQUEST_TYPE_DEFINITION,
            resource_config.shared_node.config(),
            static_config,
        )?;
        let response_storage = Self::open_type_storage(
            &resource_config.response,
            &RESPONSE_TYPE_DEFINITION,
            resource_config.shared_node.config(),
            static_config,
        )?;

        let mut path_hint = request_storage.as_ref().map(|v| v.path_hint);
        if path_hint.is_none() {
            path_hint = response_storage.as_ref().map(|v| v.path_hint);
        }

        Ok(Self {
            request_type_definition: request_storage.map(|v| v.storage),
            response_type_definition: response_storage.map(|v| v.storage),
            path_hint,
            has_ownership: AtomicBool::new(false),
        })
    }

    unsafe fn remove_stale_resources(
        config: &crate::config::Config,
        static_config: &service::static_config::StaticConfig,
    ) -> Result<(), super::RemoveStaleResourcesError> {
        let origin = "RequestResponseResources::remove_stale_resources()";
        let msg = "Failed to remove the stale request response resource";
        if let Err(e) = TypeDefinition::remove_stale_storage::<ServiceType>(
            &REQUEST_TYPE_DEFINITION,
            config,
            static_config,
        ) {
            fail!(from origin,
                with e,
                "{msg} since the request type definition storage could not be removed. [{e:?}]");
        }

        if let Err(e) = TypeDefinition::remove_stale_storage::<ServiceType>(
            &RESPONSE_TYPE_DEFINITION,
            config,
            static_config,
        ) {
            fail!(from origin,
                with e,
                "{msg} since the response type definition storage could not be removed. [{e:?}]");
        }

        if let Err(e) =
            TypeDefinition::remove_resource_directory::<ServiceType>(config, static_config)
        {
            fail!(from origin,
                with e,
                "{msg} since the resource directory could not be removed. [{e:?}]");
        }

        Ok(())
    }
}

impl<ServiceType: service::Service> RequestResponseResources<ServiceType> {
    fn create_type_storage(
        type_definition: &TypeDefinition,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<TypeDefinitionStorage<ServiceType>>, ServiceCreateError> {
        match type_definition.create_storage::<ServiceType>(name, config, static_config) {
            Ok(v) => Ok(v),
            Err(e) => {
                fail!(from "RequestResponseResources::create_type_storage()",
                    with e,
                    "Unable to create request response resources since the type definition storage {name} could not be created. [{e:?}]");
            }
        }
    }

    fn open_type_storage(
        type_definition: &TypeDefinition,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<TypeDefinitionStorage<ServiceType>>, ServiceOpenError> {
        match type_definition.open_storage::<ServiceType>(name, config, static_config) {
            Ok(v) => Ok(v),
            Err(e) => {
                fail!(from "RequestResponseResources::open_type_storage()",
                    with e,
                    "Unable to open request response resources since the type definition storage {name} could not be opened. [{e:?}]");
            }
        }
    }

    pub fn request_type_definition(&self) -> Option<&ServiceType::StaticStorage> {
        self.request_type_definition.as_ref()
    }

    pub fn response_type_definition(&self) -> Option<&ServiceType::StaticStorage> {
        self.response_type_definition.as_ref()
    }
}
