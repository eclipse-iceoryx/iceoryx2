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

use crate::node::SharedNode;
use crate::service;
use crate::service::resource::type_definition::TypeDefinition;
use crate::service::resource::{RemoveStaleResourcesError, ServiceResource};
use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::{
    NamedConceptConfiguration, NamedConceptPathHintRemoveError, NamedConceptRemoveError,
};
use iceoryx2_cal::static_storage::StaticStorage;
use iceoryx2_log::{fail, warn};

const PAYLOAD_TYPE_DEFINITION: FileName = unsafe { FileName::new_unchecked_const(b"payload") };

pub struct PublishSubscribeResourceConfig<ServiceType: service::Service> {
    pub(crate) type_definition: TypeDefinition,
    pub(crate) shared_node: SharedNode<ServiceType>,
}

#[derive(Debug)]
pub struct PublishSubscribeResources<ServiceType: service::Service> {
    type_definition_storage: Option<ServiceType::StaticStorage>,
    path_hint: Option<Path>,
    has_ownership: AtomicBool,
}

impl<ServiceType: service::Service> Abandonable for PublishSubscribeResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        this.has_ownership.store(false, Ordering::Relaxed);

        if let Some(td) = this.type_definition_storage.as_mut() {
            unsafe {
                ServiceType::StaticStorage::abandon_in_place(core::ptr::NonNull::from_mut(td))
            };
        }
    }
}

impl<ServiceType: service::Service> Drop for PublishSubscribeResources<ServiceType> {
    fn drop(&mut self) {
        if let Some(path_hint) = &self.path_hint {
            drop(self.type_definition_storage.take());
            if self.has_ownership.load(Ordering::Relaxed)
                && let Err(e) =
                    <ServiceType::StaticStorage as NamedConceptMgmt>::remove_path_hint(path_hint)
            {
                warn!(from self,
                        "Failed to remove the resource directory: \"{path_hint}\". [{e:?}]");
            }
        }
    }
}

impl<ServiceType: service::Service> ServiceResource for PublishSubscribeResources<ServiceType> {
    type Config = PublishSubscribeResourceConfig<ServiceType>;

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
        if let Some(s) = &self.type_definition_storage {
            s.acquire_ownership();
        }
    }

    fn create(
        static_config: &crate::service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, crate::service::builder::ServiceCreateError> {
        match resource_config
            .type_definition
            .create_storage::<ServiceType>(
                &PAYLOAD_TYPE_DEFINITION,
                resource_config.shared_node.config(),
                static_config,
            ) {
            Ok(Some(v)) => Ok(Self {
                type_definition_storage: Some(v.storage),
                path_hint: Some(v.path_hint),
                has_ownership: AtomicBool::new(false),
            }),
            Ok(None) => Ok(Self {
                type_definition_storage: None,
                path_hint: None,
                has_ownership: AtomicBool::new(false),
            }),
            Err(e) => {
                fail!(from "PublishSubscribeResources::create()",
                    with e,
                    "Unable to create publish subscribe resources since the type definition storage could not be created. [{e:?}]");
            }
        }
    }

    fn open(
        static_config: &crate::service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, crate::service::builder::ServiceOpenError> {
        match resource_config.type_definition.open_storage::<ServiceType>(
            &PAYLOAD_TYPE_DEFINITION,
            resource_config.shared_node.config(),
            static_config,
        ) {
            Ok(Some(v)) => Ok(Self {
                type_definition_storage: Some(v.storage),
                path_hint: Some(v.path_hint),
                has_ownership: AtomicBool::new(false),
            }),
            Ok(None) => Ok(Self {
                type_definition_storage: None,
                path_hint: None,
                has_ownership: AtomicBool::new(false),
            }),
            Err(e) => {
                fail!(from "PublishSubscribeResources::create()",
                     with e,
                     "Unable to open publish subscribe resources since the type definition storage could not be opened. [{e:?}]");
            }
        }
    }

    unsafe fn remove_stale_resources(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        let origin = "PublishSubscribeResources::remove_stale_resources()";
        let msg = "Unable to remove stale publish subscribe resources";
        let storage_config = Self::type_definition_static_storage_config(config, static_config);
        match unsafe {
            <ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::remove_cfg(
            &PAYLOAD_TYPE_DEFINITION, &storage_config,
        )
        } {
            Ok(_) => (),
            Err(NamedConceptRemoveError::Interrupt) => {
                fail!(from origin, with RemoveStaleResourcesError::InterruptedBySignal,
                    "{msg} since it was interrupted by a signal.");
            }
            Err(NamedConceptRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(NamedConceptRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} due to an internal failure.");
            }
        }

        let dir = Self::service_resource_directory(config, static_config);
        match <ServiceType::StaticStorage as NamedConceptMgmt>::remove_path_hint(&dir) {
            Ok(()) => Ok(()),
            Err(NamedConceptPathHintRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} since the resource directory could not be removed due to insufficient permissions.");
            }
            Err(NamedConceptPathHintRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} since the resource directory could not be removed due to an internal failure.");
            }
        }
    }
}

impl<ServiceType: service::Service> PublishSubscribeResources<ServiceType> {
    pub fn type_definition(&self) -> Option<&ServiceType::StaticStorage> {
        self.type_definition_storage.as_ref()
    }

    fn type_definition_static_storage_config(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> <ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration
    {
        let dir = Self::service_resource_directory(config, static_config);
        (<<ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration as Default>::default())
            .path_hint(&dir)
            .prefix(&config.global.prefix)
            .suffix(&config.global.service.type_definition_suffix)
    }
}
