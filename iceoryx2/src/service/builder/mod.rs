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

use crate::node::SharedNode;
use crate::service;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::dynamic_config::RegisterNodeResult;
use crate::service::static_config::*;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::dynamic_storage::DynamicStorageCreateError;
use iceoryx2_cal::dynamic_storage::DynamicStorageOpenError;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::named_concept::NamedConceptDoesExistError;
use iceoryx2_cal::named_concept::NamedConceptMgmt;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::static_storage::*;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use super::config_scheme::dynamic_config_storage_config;
use super::config_scheme::service_tag_config;
use super::config_scheme::static_config_storage_config;
use super::naming_scheme::dynamic_config_storage_name;
use super::naming_scheme::service_tag_name;
use super::naming_scheme::static_config_storage_name;
use super::service_name::ServiceName;
use super::Service;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceState {
    IncompatibleMessagingPattern,
    InsufficientPermissions,
    HangsInCreation,
    Corrupted,
    InternalFailure,
}

enum_gen! {
#[doc(hidden)]
    OpenDynamicStorageFailure
  entry:
    IsMarkedForDestruction,
    ExceedsMaxNumberOfNodes
  mapping:
    DynamicStorageOpenError
}

impl std::fmt::Display for OpenDynamicStorageFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "OpenDynamicStorageFailure::{:?}", self)
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
        std::write!(f, "ReadStaticStorageFailure::{:?}", self)
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
    shared_node: Arc<SharedNode<S>>,
    _phantom_s: PhantomData<S>,
}

impl<S: Service> Builder<S> {
    pub(crate) fn new(name: ServiceName, shared_node: Arc<SharedNode<S>>) -> Self {
        Self {
            name,
            shared_node,
            _phantom_s: PhantomData,
        }
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) [`Service`].
    pub fn publish_subscribe<PayloadType: Debug + ?Sized>(
        self,
    ) -> publish_subscribe::Builder<PayloadType, (), S> {
        BuilderWithServiceType::new(
            StaticConfig::new_publish_subscribe::<S::ServiceNameHasher>(
                &self.name,
                self.shared_node.config(),
            ),
            self.shared_node,
        )
        .publish_subscribe()
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) [`Service`].
    pub fn event(self) -> event::Builder<S> {
        BuilderWithServiceType::new(
            StaticConfig::new_event::<S::ServiceNameHasher>(&self.name, self.shared_node.config()),
            self.shared_node,
        )
        .event()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BuilderWithServiceType<ServiceType: service::Service> {
    service_config: StaticConfig,
    shared_node: Arc<SharedNode<ServiceType>>,
    _phantom_data: PhantomData<ServiceType>,
}

impl<ServiceType: service::Service> BuilderWithServiceType<ServiceType> {
    fn new(service_config: StaticConfig, shared_node: Arc<SharedNode<ServiceType>>) -> Self {
        Self {
            service_config,
            shared_node,
            _phantom_data: PhantomData,
        }
    }

    fn publish_subscribe<PayloadType: Debug + ?Sized>(
        self,
    ) -> publish_subscribe::Builder<PayloadType, (), ServiceType> {
        publish_subscribe::Builder::new(self)
    }

    fn event(self) -> event::Builder<ServiceType> {
        event::Builder::new(self)
    }

    fn is_service_available(
        &self,
        msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState> {
        let static_storage_config =
            static_config_storage_config::<ServiceType>(self.shared_node.config());
        let file_name_uuid = fatal_panic!(from self,
                        when FileName::new(self.service_config.uuid().as_bytes()),
                        "This should never happen! The uuid should be always a valid file name.");
        let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new().create(),
                                        with ServiceState::InternalFailure,
                                        "{} since the adaptive wait could not be created.", msg);

        loop {
            match <ServiceType::StaticStorage as NamedConceptMgmt>::does_exist_cfg(
                &file_name_uuid,
                &static_storage_config,
            ) {
                Ok(false) => return Ok(None),
                Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp) => {
                    let timeout = fail!(from self, when adaptive_wait.wait(),
                                        with ServiceState::InternalFailure,
                                        "{} since the adaptive wait failed.", msg);

                    if timeout > self.shared_node.config().global.service.creation_timeout {
                        fail!(from self, with ServiceState::HangsInCreation,
                            "{} since the service hangs while being created, max timeout for service creation of {:?} exceeded. Waited for {:?} but the state did not change.",
                            msg, self.shared_node.config().global.service.creation_timeout, timeout);
                    }
                }
                Ok(true) => {
                    let storage = match <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                                       ServiceType::StaticStorage>>
                                       ::new(&file_name_uuid)
                                        .has_ownership(false)
                                        .config(&static_storage_config)
                                        .open() {
                        Ok(storage) => storage,
                        Err(StaticStorageOpenError::DoesNotExist) => return Ok(None),
                        Err(StaticStorageOpenError::IsLocked) => {
                            let timeout = fail!(from self, when adaptive_wait.wait(),
                                                with ServiceState::InternalFailure,
                                                "{} since the adaptive wait failed.", msg);

                            if timeout > self.shared_node.config().global.service.creation_timeout {
                                fail!(from self, with ServiceState::HangsInCreation,
                                    "{} since the service hangs while being created, max timeout for service creation of {:?} exceeded. Waited for {:?} but the state did not change.",
                                    msg, self.shared_node.config().global.service.creation_timeout, timeout);
                            }

                            continue
                        },
                        Err(e) =>
                        {
                            fail!(from self, with ServiceState::InsufficientPermissions,
                                    "{} since it is not possible to open the services underlying static details ({:?}). Is the service accessible?",
                                    msg, e);
                        }
                    };

                    let mut read_content =
                        String::from_utf8(vec![b' '; storage.len() as usize]).expect("");
                    if storage
                        .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
                        .is_err()
                    {
                        fail!(from self, with ServiceState::InsufficientPermissions,
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

                    return Ok(Some((service_config, storage)));
                }
                Err(v) => {
                    fail!(from self, with ServiceState::Corrupted,
                    "{} since the service seems to be in a corrupted/inaccessible state ({:?}).", msg, v);
                }
            }
        }
    }

    fn config_init_call(config: &mut DynamicConfig, allocator: &mut BumpAllocator) -> bool {
        unsafe { config.init(allocator) };
        true
    }

    fn create_dynamic_config_storage(
        &self,
        messaging_pattern: super::dynamic_config::MessagingPattern,
        additional_size: usize,
        max_number_of_nodes: usize,
    ) -> Result<ServiceType::DynamicStorage, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic storage for service";
        let required_memory_size = DynamicConfig::memory_size(max_number_of_nodes);
        match <<ServiceType::DynamicStorage as DynamicStorage<
            DynamicConfig,
        >>::Builder<'_> as NamedConceptBuilder<
            ServiceType::DynamicStorage,
        >>::new(&dynamic_config_storage_name(&self.service_config))
            .config(&dynamic_config_storage_config::<ServiceType>(self.shared_node.config()))
            .supplementary_size(additional_size + required_memory_size)
            .has_ownership(false)
            .initializer(Self::config_init_call)
            .create(DynamicConfig::new_uninit(messaging_pattern, max_number_of_nodes) ) {
                Ok(dynamic_storage) => {
                    let node_id = self.shared_node.id();
                    let node_handle = fatal_panic!(from self,
                            when dynamic_storage.get().register_node_id(*node_id),
                            "{} since event the first NodeId could not be registered.", msg);
                    self.shared_node.registered_services.add(self.service_config.uuid(), node_handle);
                    Ok(dynamic_storage)
                },
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
                >>::Builder<'_> as NamedConceptBuilder<
                    ServiceType::DynamicStorage,
                >>::new(&dynamic_config_storage_name(&self.service_config))
                    .config(&dynamic_config_storage_config::<ServiceType>(self.shared_node.config()))
                .has_ownership(false)
                .open(),
            "{} since the dynamic storage could not be opened.", msg);

        self.shared_node
            .registered_services
            .add_or(self.service_config.uuid(), || {
                let node_id = self.shared_node.id();
                match storage.get().register_node_id(*node_id) {
                    Ok(handle) => Ok(handle),
                    Err(RegisterNodeResult::MarkedForDestruction) => {
                        fail!(from self, with OpenDynamicStorageFailure::IsMarkedForDestruction,
                            "{} since the dynamic storage is marked for destruction.", msg);
                    }
                    Err(RegisterNodeResult::ExceedsMaxNumberOfNodes) => {
                        fail!(from self, with OpenDynamicStorageFailure::ExceedsMaxNumberOfNodes,
                            "{} since it would exceed the maxium supported number of nodes.", msg);
                    }
                }
            })?;

        Ok(storage)
    }

    fn create_node_service_tag<ErrorType>(
        &self,
        error_msg: &str,
        error_value: ErrorType,
    ) -> Result<Option<ServiceType::StaticStorage>, ErrorType> {
        match <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
            ServiceType::StaticStorage,
        >>::new(&service_tag_name(self.service_config.uuid()))
        .config(&service_tag_config::<ServiceType>(
            self.shared_node.config(),
            self.shared_node.id(),
        ))
        .has_ownership(true)
        .create(&[])
        {
            Ok(static_storage) => Ok(Some(static_storage)),
            Err(StaticStorageCreateError::AlreadyExists) => Ok(None),
            Err(e) => {
                fail!(from self, with error_value,
                    "{} since the nodes service tag could not be created ({:?}).", error_msg, e);
            }
        }
    }

    fn create_static_config_storage(
        &self,
    ) -> Result<<ServiceType::StaticStorage as StaticStorage>::Locked, StaticStorageCreateError>
    {
        Ok(
            fail!(from self, when <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                        ServiceType::StaticStorage,
                    >>::new(&static_config_storage_name(self.service_config.uuid()))
                    .config(&static_config_storage_config::<ServiceType>(
                        self.shared_node.config(),
                    ))
                    .has_ownership(true)
                    .create_locked(),
                    "Failed to create static service information since the underlying static storage could not be created."),
        )
    }
}
