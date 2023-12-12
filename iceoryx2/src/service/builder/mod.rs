// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! See [`crate::service`]

/// Builder for [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
pub mod event;

/// Builder for [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
pub mod publish_subscribe;

use crate::config;
use crate::service;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::static_config::*;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::dynamic_storage::DynamicStorageCreateError;
use iceoryx2_cal::dynamic_storage::DynamicStorageOpenError;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::named_concept::NamedConceptDoesExistError;
use iceoryx2_cal::named_concept::NamedConceptMgmt;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::static_storage::*;
use std::marker::PhantomData;

use super::config_scheme::dynamic_config_storage_config;
use super::config_scheme::static_config_storage_config;
use super::naming_scheme::dynamic_config_storage_name;
use super::naming_scheme::static_config_storage_name;
use super::service_name::ServiceName;
use super::Service;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceState {
    IsBeingCreatedByAnotherInstance,
    IncompatibleMessagingPattern,
    PermissionDenied,
    Corrupted,
}

enum_gen! {
#[doc(hidden)]
    OpenDynamicStorageFailure
  entry:
    IsMarkedForDestruction
  mapping:
    DynamicStorageOpenError
}

impl std::fmt::Display for OpenDynamicStorageFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for OpenDynamicStorageFailure {}

enum_gen! {
#[doc(hidden)]
    ReadStaticStorageFailure
  mapping:
    StaticStorageOpenError,
    StaticStorageReadError
}

impl std::fmt::Display for ReadStaticStorageFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ReadStaticStorageFailure {}

/// Builder to create or open [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Builder<S: Service> {
    name: ServiceName,
    _phantom_s: PhantomData<S>,
}

impl<S: Service> Builder<S> {
    pub(crate) fn new(name: &ServiceName) -> Self {
        Self {
            name: *name,
            _phantom_s: PhantomData,
        }
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) [`Service`].
    pub fn publish_subscribe<'config>(
        self,
    ) -> publish_subscribe::Builder<'config, S::Type<'config>> {
        self.publish_subscribe_with_custom_config(config::Config::get_global_config())
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) [`Service`].
    /// with a custom [`config::Config`]
    pub fn publish_subscribe_with_custom_config(
        self,
        config: &config::Config,
    ) -> publish_subscribe::Builder<'_, S::Type<'_>> {
        BuilderWithServiceType::new(
            StaticConfig::new_publish_subscribe::<
                <<S as service::Service>::Type<'_> as service::Details<'_>>::ServiceNameHasher,
            >(&self.name, config),
            config,
        )
        .publish_subscribe()
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) [`Service`].
    pub fn event<'config>(self) -> event::Builder<'config, S::Type<'config>> {
        self.event_with_custom_config(config::Config::get_global_config())
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) [`Service`].
    /// with a custom [`config::Config`]
    pub fn event_with_custom_config(
        self,
        config: &config::Config,
    ) -> event::Builder<'_, S::Type<'_>> {
        BuilderWithServiceType::new(
            StaticConfig::new_event::<
                <<S as service::Service>::Type<'_> as service::Details<'_>>::ServiceNameHasher,
            >(&self.name, config),
            config,
        )
        .event()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BuilderWithServiceType<'config, ServiceType: service::Details<'config>> {
    service_config: StaticConfig,
    global_config: &'config config::Config,
    _phantom_data: PhantomData<ServiceType>,
    _phantom_lifetime_b: PhantomData<&'config ()>,
}

impl<'config, ServiceType: service::Details<'config>> BuilderWithServiceType<'config, ServiceType> {
    fn new(service_config: StaticConfig, global_config: &'config config::Config) -> Self {
        Self {
            service_config,
            global_config,
            _phantom_data: PhantomData,
            _phantom_lifetime_b: PhantomData,
        }
    }

    fn publish_subscribe(self) -> publish_subscribe::Builder<'config, ServiceType> {
        publish_subscribe::Builder::new(self)
    }

    fn event(self) -> event::Builder<'config, ServiceType> {
        event::Builder::new(self)
    }

    fn is_service_available(
        &self,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState> {
        let msg = "Unable to check if the service is available";
        let static_storage_config = static_config_storage_config::<ServiceType>(self.global_config);
        let file_name_uuid = fatal_panic!(from self,
                        when FileName::new(self.service_config.uuid().as_bytes()),
                        "This should never happen! The uuid should be always a valid file name.");

        match <ServiceType::StaticStorage as NamedConceptMgmt>::does_exist_cfg(
            &file_name_uuid,
            &static_storage_config,
        ) {
            Ok(false) => Ok(None),
            Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp) => {
                fail!(from self, with ServiceState::IsBeingCreatedByAnotherInstance,
                        "{} since it is currently being created.", msg);
            }
            Ok(true) => {
                let storage = if let Ok(v) = <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                                       <ServiceType as service::Details>::StaticStorage>>
                                       ::new(&file_name_uuid)
                                        .has_ownership(false)
                                        .config(&static_storage_config)
                                        .open() { v }
                else {
                    fail!(from self, with ServiceState::PermissionDenied,
                            "{} since it is not possible to open the services underlying static details. Is the service accessible?", msg);
                };

                let mut read_content =
                    String::from_utf8(vec![b' '; storage.len() as usize]).expect("");
                if storage
                    .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
                    .is_err()
                {
                    fail!(from self, with ServiceState::PermissionDenied,
                            "{} since it is not possible to read the services underlying static details. Is the service accessible?", msg);
                }

                let service_config = fail!(from self, when ServiceType::ConfigSerializer::deserialize::<StaticConfig>(unsafe {
                                            read_content.as_mut_vec() }),
                                     with ServiceState::Corrupted, "Unable to deserialize the service config. Is the service corrupted?");

                if service_config.uuid() != self.service_config.uuid() {
                    fail!(from self, with ServiceState::Corrupted,
                        "{} a service with that name exist but different uuid.", msg);
                }

                let msg = "Service exist but is not compatible";
                if !service_config.has_same_messaging_pattern(&self.service_config) {
                    fail!(from self, with ServiceState::IncompatibleMessagingPattern,
                        "{} since the messaging pattern \"{:?}\" does not fit the requested pattern \"{:?}\".",
                        msg, service_config.messaging_pattern(), self.service_config.messaging_pattern());
                }

                Ok(Some((service_config, storage)))
            }
            Err(v) => {
                fail!(from self, with ServiceState::Corrupted,
                    "{} since the service seems to be in a corrupted/inaccessible state ({:?}).", msg, v);
            }
        }
    }

    fn create_dynamic_config_storage(
        &self,
        messaging_pattern: super::dynamic_config::MessagingPattern,
        additional_size: usize,
    ) -> Result<ServiceType::DynamicStorage, DynamicStorageCreateError> {
        match <<ServiceType::DynamicStorage as DynamicStorage<
            DynamicConfig,
        >>::Builder as NamedConceptBuilder<
            ServiceType::DynamicStorage,
        >>::new(&dynamic_config_storage_name(&self.service_config))
            .config(&dynamic_config_storage_config::<ServiceType>(self.global_config))
            .supplementary_size(additional_size)
            .has_ownership(false)
            .create_and_initialize(DynamicConfig::new_uninit(messaging_pattern),
                |config, allocator| {
                    unsafe { config.init(allocator) };
                    true
                }
                ) {
                Ok(dynamic_storage) => Ok(dynamic_storage),
                Err(e) => {
                    fail!(from self, with e, "Failed to create dynamic storage for service.");
                }
            }
    }

    fn open_dynamic_config_storage(
        &self,
    ) -> Result<ServiceType::DynamicStorage, OpenDynamicStorageFailure> {
        let msg = "Failed to open dynamic service information";
        let storage = fail!(from self, when
            <<ServiceType::DynamicStorage as DynamicStorage<
                    DynamicConfig,
                >>::Builder as NamedConceptBuilder<
                    ServiceType::DynamicStorage,
                >>::new(&dynamic_config_storage_name(&self.service_config))
                    .config(&dynamic_config_storage_config::<ServiceType>(self.global_config))
                .has_ownership(false)
                .open(),
            "{} since the dynamic storage could not be opened.", msg);

        fail!(from self, when storage.get().increment_reference_counter(),
                with OpenDynamicStorageFailure::IsMarkedForDestruction,
                "{} since the dynamic storage is marked for destruction.", msg);

        Ok(storage)
    }

    fn create_static_config_storage(
        &self,
    ) -> Result<
        <<ServiceType as service::Details<'config>>::StaticStorage as StaticStorage>::Locked,
        StaticStorageCreateError,
    > {
        Ok(
            fail!(from self, when <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                        <ServiceType as service::Details>::StaticStorage,
                    >>::new(&static_config_storage_name(self.service_config.uuid()))
                    .config(&static_config_storage_config::<ServiceType>(
                        self.global_config,
                    ))
                    .has_ownership(false)
                    .create_locked(),
                    "Failed to create static service information since the underlying static storage could not be created."),
        )
    }
}
