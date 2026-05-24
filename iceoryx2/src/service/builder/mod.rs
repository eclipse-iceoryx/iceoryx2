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

/// Builder for [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
pub mod request_response;

/// Builder for [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
pub mod blackboard;

use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;

use alloc::format;
use alloc::string::String;
use alloc::vec;

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_cal::dynamic_storage::DynamicStorageCreateError;
use iceoryx2_cal::dynamic_storage::DynamicStorageOpenError;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::named_concept::NamedConceptDoesExistError;
use iceoryx2_cal::named_concept::NamedConceptMgmt;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::static_storage::*;
use iceoryx2_log::fail;
use iceoryx2_log::fatal_panic;
use iceoryx2_log::warn;

use crate::config::IO_TICK_TIME;
use crate::identifiers::UniqueServiceId;
use crate::node::NodeState;
use crate::node::SharedNode;
use crate::prelude::AttributeSpecifier;
use crate::prelude::AttributeVerifier;
use crate::service;
use crate::service::__internal_details;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::dynamic_config::MessagingPatternSettings;
use crate::service::dynamic_config::RegisterNodeResult;
use crate::service::naming_scheme::dynamic_config_name;
use crate::service::naming_scheme::static_config_name;
use crate::service::static_config::*;

use super::Service;
use super::config_scheme::dynamic_config_storage_config;
use super::config_scheme::service_tag_config;
use super::config_scheme::static_config_storage_config;
use super::service_name::ServiceName;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceState {
    IncompatibleMessagingPattern,
    InternalFailure,
    Interrupt,
    InsufficientPermissions,
    HangsInCreation,
    Corrupted,
    IncompatiblePayload,
}

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone, Default)]
#[doc(hidden)]
pub struct CustomHeaderMarker {}

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone)]
#[doc(hidden)]
pub struct CustomPayloadMarker(u8);

#[repr(C)]
#[derive(ZeroCopySend, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct CustomKeyMarker(u8);

enum_gen! {
#[doc(hidden)]
    OpenDynamicStorageFailure
  entry:
    IsMarkedForDestruction,
    ExceedsMaxNumberOfNodes
  mapping:
    DynamicStorageOpenError
}

enum_gen! {
#[doc(hidden)]
    ReadStaticStorageFailure
  mapping:
    StaticStorageOpenError,
    StaticStorageReadError
}

/// Builder to create or open [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug, Clone)]
pub struct Builder<S: Service> {
    name: ServiceName,
    shared_node: SharedNode<S>,
    _phantom_s: PhantomData<S>,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceCreateError {
    InternalFailure,
    AlreadyExists,
    IsBeingCreatedByAnotherInstance,
    InsufficientPermissions,
    ServiceInCorruptedState,
    UnableToCreateServiceTag,
    ServiceConfigCouldNotBeCreated,
    Interrupt,
}

impl From<ServiceState> for ServiceCreateError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::IncompatibleMessagingPattern
            | ServiceState::HangsInCreation
            | ServiceState::IncompatiblePayload => ServiceCreateError::AlreadyExists,
            ServiceState::InsufficientPermissions => ServiceCreateError::InsufficientPermissions,
            ServiceState::Corrupted => ServiceCreateError::ServiceInCorruptedState,
            ServiceState::InternalFailure => ServiceCreateError::InternalFailure,
            ServiceState::Interrupt => ServiceCreateError::Interrupt,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceOpenError {
    InternalFailure,
    DoesNotExist,
    UnableToCreateServiceTag,
    IsMarkedForDestruction,
    ExceedsMaxNumberOfNodes,
    ServiceInCorruptedState,
    HangsInCreation,
    Interrupt,
    IncompatibleMessagingPattern,
    IncompatiblePayload,
    InsufficientPermissions,
    VersionMismatch,
}

impl From<ServiceState> for ServiceOpenError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::Corrupted => ServiceOpenError::ServiceInCorruptedState,
            ServiceState::HangsInCreation => ServiceOpenError::HangsInCreation,
            ServiceState::IncompatibleMessagingPattern => {
                ServiceOpenError::IncompatibleMessagingPattern
            }
            ServiceState::IncompatiblePayload => ServiceOpenError::IncompatiblePayload,
            ServiceState::InsufficientPermissions => ServiceOpenError::InsufficientPermissions,
            ServiceState::InternalFailure => ServiceOpenError::InternalFailure,
            ServiceState::Interrupt => ServiceOpenError::Interrupt,
        }
    }
}

struct DynamicConfigCreationArgs {
    messaging_pattern_settings: MessagingPatternSettings,
    additional_size: usize,
    max_number_of_nodes: usize,
}

impl<S: Service> Builder<S> {
    pub(crate) fn new(name: &ServiceName, shared_node: SharedNode<S>) -> Self {
        Self {
            name: *name,
            shared_node,
            _phantom_s: PhantomData,
        }
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse) [`Service`].
    pub fn request_response<
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
    >(
        self,
    ) -> request_response::Builder<RequestPayload, (), ResponsePayload, (), S> {
        BuilderWithServiceType::new(
            StaticConfig::new_request_response::<S::ServiceNameHasher>(
                &self.name,
                self.shared_node.config(),
            ),
            self.shared_node,
        )
        .request_response::<RequestPayload, ResponsePayload>()
    }

    /// Create a new builder to create a
    /// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) [`Service`].
    pub fn publish_subscribe<PayloadType: Debug + ?Sized + ZeroCopySend>(
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

    /// Create a new builder to create a
    /// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard) [`Service`].
    pub fn blackboard_creator<
        KeyType: Send + Sync + Eq + Clone + Copy + Debug + 'static + ZeroCopySend + Hash,
    >(
        self,
    ) -> blackboard::Creator<KeyType, S> {
        BuilderWithServiceType::new(
            StaticConfig::new_blackboard::<S::ServiceNameHasher>(
                &self.name,
                self.shared_node.config(),
            ),
            self.shared_node,
        )
        .blackboard_creator()
    }

    /// Create a new builder to open a
    /// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard) [`Service`].
    pub fn blackboard_opener<
        KeyType: Send + Sync + Eq + Clone + Copy + Debug + 'static + ZeroCopySend + Hash,
    >(
        self,
    ) -> blackboard::Opener<KeyType, S> {
        BuilderWithServiceType::new(
            StaticConfig::new_blackboard::<S::ServiceNameHasher>(
                &self.name,
                self.shared_node.config(),
            ),
            self.shared_node,
        )
        .blackboard_opener()
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct BuilderWithServiceType<ServiceType: service::Service> {
    service_config: StaticConfig,
    shared_node: SharedNode<ServiceType>,
    _phantom_data: PhantomData<ServiceType>,
}

impl<ServiceType: service::Service> BuilderWithServiceType<ServiceType> {
    fn new(service_config: StaticConfig, shared_node: SharedNode<ServiceType>) -> Self {
        Self {
            service_config,
            shared_node,
            _phantom_data: PhantomData,
        }
    }

    fn request_response<
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
    >(
        self,
    ) -> request_response::Builder<RequestPayload, (), ResponsePayload, (), ServiceType> {
        request_response::Builder::new(self)
    }

    fn publish_subscribe<PayloadType: Debug + ?Sized + ZeroCopySend>(
        self,
    ) -> publish_subscribe::Builder<PayloadType, (), ServiceType> {
        publish_subscribe::Builder::new(self)
    }

    fn event(self) -> event::Builder<ServiceType> {
        event::Builder::new(self)
    }

    fn blackboard_creator<
        KeyType: Send + Sync + Eq + Clone + Copy + Debug + 'static + ZeroCopySend + Hash,
    >(
        self,
    ) -> blackboard::Creator<KeyType, ServiceType> {
        blackboard::Creator::new(self)
    }

    fn blackboard_opener<
        KeyType: Send + Sync + Eq + Clone + Copy + Debug + 'static + ZeroCopySend + Hash,
    >(
        self,
    ) -> blackboard::Opener<KeyType, ServiceType> {
        blackboard::Opener::new(self)
    }

    fn open_or_create<
        ErrorTypeOpen: Into<ServiceOpenError> + Copy,
        ErrorTypeCreate: Into<ServiceCreateError> + Copy,
        ErrorTypeOpenOrCreate: From<ErrorTypeOpen> + From<ErrorTypeCreate> + Debug,
        PortFactory,
        FOpen: FnMut(&AttributeVerifier) -> Result<PortFactory, ErrorTypeOpen>,
        FCreate: FnMut(&AttributeSpecifier) -> Result<PortFactory, ErrorTypeCreate>,
    >(
        &self,
        msg: &str,
        attributes: &AttributeVerifier,
        internal_error_type: ErrorTypeOpenOrCreate,
        system_in_flux_error_type: ErrorTypeOpenOrCreate,
        mut open_call: FOpen,
        mut create_call: FCreate,
    ) -> Result<PortFactory, ErrorTypeOpenOrCreate> {
        const ERROR_TRACKING_LIMIT: usize = 5;
        let mut adaptive_wait = fail!(from self,
              when AdaptiveWaitBuilder::new().strategy(iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitStrategy::FixedTicks(IO_TICK_TIME)).create(),
              with internal_error_type,
              "{msg} since the adaptive wait could not be created.");
        let start = fail!(from self,
              when Time::now(),
              with internal_error_type,
              "{msg} since the current time could not be acquired.");
        let creation_timeout = self.shared_node.config().global.creation_timeout;
        let attribute_specifier = AttributeSpecifier(attributes.required_attributes().clone());

        let mut flux_counter = 0;
        let mut last_errors = vec![];
        loop {
            let mut try_create_service = true;
            match open_call(attributes) {
                Ok(service) => return Ok(service),
                Err(e) => {
                    last_errors.insert(0, Into::<ErrorTypeOpenOrCreate>::into(e));
                    match e.into() {
                        ServiceOpenError::DoesNotExist => (),
                        ServiceOpenError::HangsInCreation
                        | ServiceOpenError::IsMarkedForDestruction => {
                            try_create_service = false;
                        }
                        ServiceOpenError::InsufficientPermissions
                        | ServiceOpenError::ServiceInCorruptedState
                        | ServiceOpenError::ExceedsMaxNumberOfNodes
                        | ServiceOpenError::InternalFailure
                        | ServiceOpenError::IncompatibleMessagingPattern
                        | ServiceOpenError::IncompatiblePayload
                        | ServiceOpenError::UnableToCreateServiceTag
                        | ServiceOpenError::Interrupt
                        | ServiceOpenError::VersionMismatch => {
                            return Err(Into::<ErrorTypeOpenOrCreate>::into(e));
                        }
                    }
                }
            }

            if try_create_service {
                flux_counter += 1;
                match create_call(&attribute_specifier) {
                    Ok(service) => return Ok(service),
                    Err(e) => {
                        last_errors.insert(0, Into::<ErrorTypeOpenOrCreate>::into(e));
                        match e.into() {
                            ServiceCreateError::AlreadyExists
                            | ServiceCreateError::IsBeingCreatedByAnotherInstance => (),
                            ServiceCreateError::InsufficientPermissions
                            | ServiceCreateError::InternalFailure
                            | ServiceCreateError::ServiceInCorruptedState
                            | ServiceCreateError::UnableToCreateServiceTag
                            | ServiceCreateError::Interrupt
                            | ServiceCreateError::ServiceConfigCouldNotBeCreated => {
                                return Err(Into::<ErrorTypeOpenOrCreate>::into(e));
                            }
                        }
                    }
                }
            }

            let elapsed = fail!(from self,
                                when start.elapsed(),
                                with internal_error_type,
                                "{msg} since the elapsed time could not be acquired.");

            if elapsed >= creation_timeout {
                if flux_counter > 1 {
                    fail!(from self, with system_in_flux_error_type,
                        "{msg} tried to open and create the service repeatedly ({flux_counter} times) but another instance seems to create and remove the service continuously. Last errors: [{last_errors:?}]");
                } else {
                    fail!(from self, with last_errors.pop().unwrap_or(internal_error_type),
                        "{msg} since the service is being created and removed repeatedly.");
                }
            }

            while last_errors.len() > ERROR_TRACKING_LIMIT {
                last_errors.pop();
            }

            fail!(from self,
                  when adaptive_wait.wait(),
                  with internal_error_type,
                  "{msg} since the adaptive wait failed.");
        }
    }

    fn open<
        ErrorType: From<ServiceOpenError> + From<ServiceState>,
        R: service::ServiceResource,
        FA: FnMut() -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState>,
        F1: FnMut(&StaticConfig) -> Result<(), ErrorType>,
        F2: FnMut(&StaticConfig) -> Result<R, ErrorType>,
    >(
        &self,
        msg: &str,
        mut is_service_available: FA,
        mut verify_service_configuration: F1,
        mut open_service_resource: F2,
    ) -> Result<service::ServiceState<ServiceType, R>, ErrorType> {
        let origin = format!("{self:?}");
        let config = self.shared_node.config();

        if config.global.service.cleanup_dead_nodes_on_open {
            match __internal_details::<ServiceType>(config, self.service_config.service_hash()) {
                Ok(Some(service)) => {
                    if let Some(dynamic_details) = service.dynamic_details {
                        for node in dynamic_details.nodes {
                            let node_id = *node.node_id();
                            if let NodeState::Dead(node) = node {
                                if let Err(e) = node
                                    .blocking_remove_stale_resources(config.global.creation_timeout)
                                {
                                    warn!(from origin,
                                        "Detected dead node ({}) in service {} but failed to cleanup the resources. This might cause problems when stale port resources block the creation of new ones. [{e:?}]", node_id, self.service_config.service_hash())
                                }
                            }
                        }
                    }
                }
                Ok(None) => (),
                Err(e) => {
                    warn!(from origin,
                        "Failed to check if the service {} contains dead nodes. [{e:?}]",
                        self.service_config.service_hash())
                }
            };
        }

        let mut adaptive_wait = fail!(from origin,
              when AdaptiveWaitBuilder::new().strategy(iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitStrategy::FixedTicks(IO_TICK_TIME)).create(),
              with ServiceOpenError::InternalFailure.into(),
              "{msg} since the adaptive wait could not be created.");
        let start = fail!(from origin,
              when Time::now(),
              with ServiceOpenError::InternalFailure.into(),
              "{msg} since the current time could not be acquired.");
        let creation_timeout = self.shared_node.config().global.creation_timeout;

        loop {
            match is_service_available() {
                Err(ServiceState::HangsInCreation) => {
                    let elapsed = fail!(from self,
                        when start.elapsed(),
                        with ServiceOpenError::InternalFailure.into(),
                        "{} since the elapsed time could not be acquired.", msg);
                    if elapsed > creation_timeout {
                        fail!(from origin, with ServiceOpenError::HangsInCreation.into(),
                            "{} since the service hangs in creation", msg);
                    } else {
                        if let Err(e) = adaptive_wait.wait() {
                            fail!(from origin,
                            with ServiceOpenError::InternalFailure.into(),
                            "{} since the adaptive wait failed. [{e:?}]", msg);
                        }
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(None) => {
                    fail!(from self, with ServiceOpenError::DoesNotExist.into(),
                        "{} since the service does not exist.", msg);
                }
                Ok(Some((existing_static_config, static_storage))) => {
                    verify_service_configuration(&existing_static_config)?;

                    let service_tag = self
                        .create_node_service_tag(msg, ServiceOpenError::UnableToCreateServiceTag)?;

                    let dynamic_config = match self
                        .open_dynamic_config_storage(existing_static_config.unique_service_id())
                    {
                        Ok(v) => v,
                        Err(OpenDynamicStorageFailure::IsMarkedForDestruction) => {
                            fail!(from self, with ServiceOpenError::IsMarkedForDestruction.into(),
                                "{} since the service is marked for destruction.", msg);
                        }
                        Err(OpenDynamicStorageFailure::ExceedsMaxNumberOfNodes) => {
                            fail!(from self, with ServiceOpenError::ExceedsMaxNumberOfNodes.into(),
                                "{} since it would exceed the maximum number of supported nodes.", msg);
                        }
                        Err(OpenDynamicStorageFailure::DynamicStorageOpenError(
                            DynamicStorageOpenError::DoesNotExist,
                        )) => {
                            fail!(from self, with ServiceOpenError::IsMarkedForDestruction.into(),
                                "{} since the dynamic segment of the service is missing.", msg);
                        }
                        Err(OpenDynamicStorageFailure::DynamicStorageOpenError(
                            DynamicStorageOpenError::InitializationNotYetFinalized,
                        )) => {
                            fail!(from self, with ServiceOpenError::ServiceInCorruptedState.into(),
                                "This should never happen! {} since the corresponding dynamic config is not yet finalized.", msg);
                        }
                        Err(OpenDynamicStorageFailure::DynamicStorageOpenError(
                            DynamicStorageOpenError::VersionMismatch,
                        )) => {
                            fail!(from self,
                                with ServiceOpenError::VersionMismatch.into(),
                                "{msg} since the service seems to be created with a different iceoryx2 version.");
                        }
                        Err(e) => {
                            fail!(from self, with ServiceOpenError::InternalFailure.into(),
                                "{msg} since the dynamic service information could not be opened. [{e:?}]");
                        }
                    };

                    if let Some(service_tag) = service_tag {
                        service_tag.release_ownership();
                    }

                    let resource = open_service_resource(&existing_static_config)?;

                    return Ok(service::ServiceState::new(
                        existing_static_config,
                        self.shared_node.clone(),
                        dynamic_config,
                        static_storage,
                        resource,
                    ));
                }
            }
        }
    }

    fn create<
        R: service::ServiceResource,
        FA: FnMut() -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState>,
        F1: FnMut(&mut StaticConfig) -> Result<(), ServiceCreateError>,
        F2: FnMut(&StaticConfig) -> DynamicConfigCreationArgs,
        F3: FnMut(&StaticConfig) -> Result<R, ServiceCreateError>,
    >(
        &self,
        msg: &str,
        attributes: &AttributeSpecifier,
        mut is_service_available: FA,
        mut prepare_service_config: F1,
        mut generate_dynamic_config: F2,
        mut create_service_resource: F3,
    ) -> Result<service::ServiceState<ServiceType, R>, ServiceCreateError> {
        let mut service_config = self.service_config.clone();
        match is_service_available()? {
            None => {
                let service_tag = fail!(from self,
                    when self.create_node_service_tag(msg, ServiceCreateError::InternalFailure),
                    with ServiceCreateError::UnableToCreateServiceTag,
                    "{msg} since the service tag could not be created."
                );

                prepare_service_config(&mut service_config)?;

                let dyn_conf_creation_args = generate_dynamic_config(&service_config);

                let dynamic_config = match self
                    .create_dynamic_config_storage(dyn_conf_creation_args)
                {
                    Ok(dynamic_config) => dynamic_config,
                    Err(DynamicStorageCreateError::AlreadyExists) => {
                        fail!(from self, with ServiceCreateError::ServiceInCorruptedState,
                            "This should never happen! {} since the unique dynamic service management segment already exists.", msg);
                    }
                    Err(DynamicStorageCreateError::InsufficientPermissions) => {
                        fail!(from self, with ServiceCreateError::InsufficientPermissions,
                            "{msg} since the dynamic service config could not be created due to insufficient permissions.");
                    }
                    Err(DynamicStorageCreateError::InitializationFailed) => {
                        fail!(from self, with ServiceCreateError::InternalFailure,
                            "{msg} since the dynamic service config initialization failed.");
                    }
                    Err(DynamicStorageCreateError::InternalError) => {
                        fail!(from self, with ServiceCreateError::InternalFailure,
                            "{} since the dynamic service segment could not be created due to an internal failure.", msg);
                    }
                };

                let resource = create_service_resource(&service_config)?;

                service_config.attributes = attributes.0.clone();

                let serialized_service_config = fail!(from self, when ServiceType::ConfigSerializer::serialize(&service_config),
                                            with ServiceCreateError::ServiceConfigCouldNotBeCreated,
                                            "{} since the configuration could not be serialized.", msg);

                let static_config = match self.create_static_config_storage() {
                    Ok(c) => c,
                    Err(StaticStorageCreateError::Interrupt) => {
                        fail!(from self, with ServiceCreateError::Interrupt,
                            "{} since an interrupt signal was received.", msg);
                    }
                    Err(StaticStorageCreateError::AlreadyExists) => {
                        fail!(from self, with ServiceCreateError::AlreadyExists,
                           "{} since the service already exists.", msg);
                    }
                    Err(StaticStorageCreateError::Creation) => {
                        fail!(from self, with ServiceCreateError::IsBeingCreatedByAnotherInstance,
                            "{} since the service is being created by another instance.", msg);
                    }
                    Err(StaticStorageCreateError::InsufficientPermissions) => {
                        fail!(from self, with ServiceCreateError::InsufficientPermissions,
                            "{} since the static service information could not be created due to insufficient permissions.", msg);
                    }
                    Err(StaticStorageCreateError::Write) => {
                        fail!(from self, with ServiceCreateError::InsufficientPermissions,
                            "{} since the static service information could not be written.", msg);
                    }
                    Err(StaticStorageCreateError::InternalError) => {
                        fail!(from self, with ServiceCreateError::InternalFailure,
                            "{} since the static service information could not be created due to an internal failure.", msg);
                    }
                };

                let node_id = self.shared_node.id();
                let node_handle = fatal_panic!(from self,
                        when dynamic_config.get().register_node_id(*node_id),
                        "{} since event the first NodeId could not be registered.", msg);
                self.shared_node
                    .registered_services()
                    .add(service_config.service_hash(), node_handle);

                // only unlock the static details when the service is successfully created
                let unlocked_static_details = fail!(from self, when static_config.unlock(serialized_service_config.as_slice()),
                            with ServiceCreateError::ServiceConfigCouldNotBeCreated,
                            "{} since the configuration could not be written to the static storage.", msg);

                unlocked_static_details.release_ownership();
                if let Some(service_tag) = service_tag {
                    service_tag.release_ownership();
                }

                Ok(service::ServiceState::new(
                    service_config,
                    self.shared_node.clone(),
                    dynamic_config,
                    unlocked_static_details,
                    resource,
                ))
            }
            Some(_) => {
                fail!(from self, with ServiceCreateError::AlreadyExists,
                    "{} since the service already exists.", msg);
            }
        }
    }

    fn is_service_available(
        &self,
        msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState> {
        let expected_service_config = &self.service_config;
        let static_storage_config =
            static_config_storage_config::<ServiceType>(self.shared_node.config());
        let name = static_config_name(expected_service_config.service_hash());
        let creation_timeout = self.shared_node.config().global.creation_timeout;

        match <ServiceType::StaticStorage as NamedConceptMgmt>::does_exist_cfg(
            &name,
            &static_storage_config,
        ) {
            Ok(false) => Ok(None),
            Ok(true) => {
                let storage = match <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                                       ServiceType::StaticStorage>>
                                       ::new(&name)
                                        .has_ownership(false)
                                        .config(&static_storage_config)
                                        .open(creation_timeout) {
                        Ok(storage) => storage,
                        Err(StaticStorageOpenError::DoesNotExist) => return Ok(None),
                        Err(StaticStorageOpenError::InsufficientPermissions) => {
                            fail!(from self, with ServiceState::InsufficientPermissions,
                                "{} due to insufficient permissions.", msg);
                        }
                        Err(StaticStorageOpenError::Interrupt) => {
                            fail!(from self, with ServiceState::Interrupt,
                                "{} since an interrupt signal was received.", msg);
                        }
                        Err(StaticStorageOpenError::InitializationNotYetFinalized) => {
                            fail!(from self, with ServiceState::HangsInCreation,
                                "{} since the service hangs while being created, max timeout for service creation of {:?} exceeded.",
                                msg, creation_timeout);
                        },
                        Err(StaticStorageOpenError::Read) =>
                        {
                            fail!(from self, with ServiceState::InsufficientPermissions,
                                    "{} since it is not possible to read the services underlying static details. Is the service accessible?",
                                    msg);
                        }
                        Err(StaticStorageOpenError::InternalError) =>
                        {
                            fail!(from self, with ServiceState::InternalFailure,
                                    "{} since it is not possible to open the services underlying static details due to an internal failure.",
                                    msg);
                        }
                    };

                let mut read_content =
                    String::from_utf8(vec![b' '; storage.len() as usize]).expect("");
                if let Err(e) = storage.read(unsafe { read_content.as_mut_vec() }.as_mut_slice()) {
                    fail!(from self, with ServiceState::InsufficientPermissions,
                            "{} since it is not possible to read the services underlying static details. Is the service accessible? [{e:?}]", msg);
                }

                let service_config = fail!(from self, when ServiceType::ConfigSerializer::deserialize::<StaticConfig>(unsafe {
                                            read_content.as_mut_vec() }),
                                     with ServiceState::Corrupted, "Unable to deserialize the service config. Is the service corrupted?");

                if service_config.service_hash() != service_config.service_hash() {
                    fail!(from self, with ServiceState::Corrupted,
                        "{} a service with that name exist but different ServiceHash.", msg);
                }

                let msg = "Service exist but is not compatible";
                if !service_config.has_same_messaging_pattern(expected_service_config) {
                    fail!(from self, with ServiceState::IncompatibleMessagingPattern,
                        "{} since the messaging pattern \"{:?}\" does not fit the requested pattern \"{:?}\".",
                        msg, service_config.messaging_pattern(), service_config.messaging_pattern());
                }

                Ok(Some((service_config, storage)))
            }
            Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp) => {
                fail!(from self, with ServiceState::HangsInCreation,
                    "{} since the service is currently being set up.", msg);
            }
            Err(NamedConceptDoesExistError::InsufficientPermissions) => {
                fail!(from self, with ServiceState::InsufficientPermissions,
                    "{} since the service cannot be accessed due to insufficient permissions.", msg);
            }
            Err(NamedConceptDoesExistError::UnderlyingResourcesCorrupted) => {
                fail!(from self, with ServiceState::Corrupted,
                    "{} since the the underlying static service config seems to be corrupted.",
                    msg);
            }
            Err(NamedConceptDoesExistError::Interrupt) => {
                fail!(from self, with ServiceState::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(NamedConceptDoesExistError::InternalError) => {
                fail!(from self, with ServiceState::InternalFailure,
                    "{} since an internal error has occurred.", msg);
            }
        }
    }

    fn config_init_call(config: &mut DynamicConfig, allocator: &mut BumpAllocator) -> bool {
        unsafe { config.init(allocator) };
        true
    }

    fn create_dynamic_config_storage_resource(
        &self,
        args: DynamicConfigCreationArgs,
    ) -> Result<ServiceType::DynamicStorage<DynamicConfig>, DynamicStorageCreateError> {
        let required_memory_size = DynamicConfig::memory_size(args.max_number_of_nodes);
        let segment_name = dynamic_config_name(self.service_config.unique_service_id());
        match <<ServiceType::DynamicStorage<DynamicConfig> as DynamicStorage<
            DynamicConfig,
        >>::Builder<'_> as NamedConceptBuilder<
            ServiceType::DynamicStorage<DynamicConfig>,
        >>::new(&segment_name)
            .config(&dynamic_config_storage_config::<ServiceType>(self.shared_node.config()))
            .supplementary_size(args.additional_size + required_memory_size)
            .has_ownership(false)
            .initializer(Self::config_init_call)
            .create(DynamicConfig::new_uninit(super::dynamic_config::MessagingPattern::new(&args.messaging_pattern_settings), args.max_number_of_nodes) ) {
                Ok(dynamic_storage) => {
                   Ok(dynamic_storage)
                },
                Err(e) => {
                    fail!(from self, with e, "Failed to create dynamic storage for service.");
                }
            }
    }

    fn create_dynamic_config_storage(
        &self,
        args: DynamicConfigCreationArgs,
    ) -> Result<ServiceType::DynamicStorage<DynamicConfig>, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic storage for service";
        match self.create_dynamic_config_storage_resource(args) {
            Ok(storage) => Ok(storage),
            Err(DynamicStorageCreateError::AlreadyExists) => {
                fail!(from self, with DynamicStorageCreateError::AlreadyExists,
                    "{msg} since the dynamic config already exists. This should never happen!");
            }
            Err(e) => {
                fail!(from self, with e,
                    "{msg} since the dynamic config could not be created. [{e:?}]");
            }
        }
    }

    fn open_dynamic_config_storage(
        &self,
        unique_service_id: UniqueServiceId,
    ) -> Result<ServiceType::DynamicStorage<DynamicConfig>, OpenDynamicStorageFailure> {
        let msg = "Failed to open dynamic service information";
        let segment_name = dynamic_config_name(unique_service_id);
        let storage = fail!(from self, when
            <<ServiceType::DynamicStorage<DynamicConfig> as DynamicStorage<
                    DynamicConfig,
                >>::Builder<'_> as NamedConceptBuilder<
                    ServiceType::DynamicStorage<DynamicConfig>,
                >>::new(&segment_name)
                    .timeout(self.shared_node.config().global.creation_timeout)
                    .config(&dynamic_config_storage_config::<ServiceType>(self.shared_node.config()))
                .has_ownership(false)
                .open(AccessMode::ReadWrite),
            "{} since the dynamic storage could not be opened.", msg);

        self.shared_node.registered_services().add_or(
            self.service_config.service_hash(),
            || {
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
            },
        )?;

        Ok(storage)
    }

    fn create_node_service_tag<ErrorType>(
        &self,
        error_msg: &str,
        error_value: ErrorType,
    ) -> Result<Option<ServiceType::StaticStorage>, ErrorType> {
        match <<ServiceType::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
            ServiceType::StaticStorage,
        >>::new(&self.service_config.service_hash().0.into())
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
                    >>::new(&self.service_config.service_hash().0.into())
                    .config(&static_config_storage_config::<ServiceType>(
                        self.shared_node.config(),
                    ))
                    .has_ownership(true)
                    .create_locked(),
                    "Failed to create static service information since the underlying static storage could not be created."),
        )
    }
}
