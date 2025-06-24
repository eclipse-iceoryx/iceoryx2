// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
//!
use self::attribute::{AttributeSpecifier, AttributeVerifier};
use super::{OpenDynamicStorageFailure, ServiceState};
use crate::service;
use crate::service::config_scheme::blackboard_mgmt_data_segment_config;
use crate::service::dynamic_config::blackboard::DynamicConfigSettings;
use crate::service::naming_scheme::blackboard_mgmt_data_segment_name;
use crate::service::port_factory::blackboard;
use crate::service::static_config::message_type_details::TypeDetail;
use crate::service::static_config::messaging_pattern::MessagingPattern;
use crate::service::*;
use builder::RETRY_LIMIT;
use core::marker::PhantomData;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::dynamic_storage::DynamicStorageCreateError;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceAvailabilityState {
    ServiceState(ServiceState),
    IncompatibleKeys,
}

/// Errors that can occur when an existing [`MessagingPattern::Blackboard`] [`Service`] shall be opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardOpenError {
    /// Service could not be openen since it does not exist
    DoesNotExist,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`] has the wrong key type.
    IncompatibleKeys,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`Service`] supports less [`Reader`](crate::port::reader::Reader)s than requested.
    DoesNotSupportRequestedAmountOfReaders,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than requested.
    DoesNotSupportRequestedAmountOfNodes,
}

impl core::fmt::Display for BlackboardOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BlackboardOpenError::{self:?}")
    }
}

impl core::error::Error for BlackboardOpenError {}

impl From<ServiceAvailabilityState> for BlackboardOpenError {
    fn from(value: ServiceAvailabilityState) -> Self {
        match value {
            ServiceAvailabilityState::IncompatibleKeys => BlackboardOpenError::IncompatibleKeys,
            ServiceAvailabilityState::ServiceState(ServiceState::IncompatibleMessagingPattern) => {
                BlackboardOpenError::IncompatibleMessagingPattern
            }
            ServiceAvailabilityState::ServiceState(ServiceState::InsufficientPermissions) => {
                BlackboardOpenError::InsufficientPermissions
            }
            ServiceAvailabilityState::ServiceState(ServiceState::HangsInCreation) => {
                BlackboardOpenError::HangsInCreation
            }
            ServiceAvailabilityState::ServiceState(ServiceState::Corrupted) => {
                BlackboardOpenError::ServiceInCorruptedState
            }
        }
    }
}

/// Errors that can occur when a new [`MessagingPattern::Blackboard`] [`Service`] shall be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardCreateError {
    /// The [`Service`] already exists.
    AlreadyExists,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
}

impl core::fmt::Display for BlackboardCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BlackboardCreateError::{self:?}")
    }
}

impl core::error::Error for BlackboardCreateError {}

impl From<ServiceAvailabilityState> for BlackboardCreateError {
    fn from(value: ServiceAvailabilityState) -> Self {
        match value {
            ServiceAvailabilityState::IncompatibleKeys
            | ServiceAvailabilityState::ServiceState(ServiceState::IncompatibleMessagingPattern) => {
                BlackboardCreateError::AlreadyExists
            }
            ServiceAvailabilityState::ServiceState(ServiceState::InsufficientPermissions) => {
                BlackboardCreateError::InsufficientPermissions
            }
            ServiceAvailabilityState::ServiceState(ServiceState::HangsInCreation) => {
                BlackboardCreateError::HangsInCreation
            }
            ServiceAvailabilityState::ServiceState(ServiceState::Corrupted) => {
                BlackboardCreateError::ServiceInCorruptedState
            }
        }
    }
}

/// Errors that can occur when a [`MessagingPattern::Blackboard`] [`Service`] shall be
/// created or opened.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlackboardOpenOrCreateError {
    /// Failures that can occur when an existing [`Service`] could not be opened.
    BlackboardOpenError(BlackboardOpenError),
    /// Failures that can occur when a [`Service`] could not be created.
    BlackboardCreateError(BlackboardCreateError),
    /// Can occur when another process creates and removes the same [`Service`] repeatedly with a
    /// high frequency.
    SystemInFlux,
}

impl core::fmt::Display for BlackboardOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BlackboardOpenOrCreateError::{self:?}")
    }
}

impl core::error::Error for BlackboardOpenOrCreateError {}

impl From<ServiceAvailabilityState> for BlackboardOpenOrCreateError {
    fn from(value: ServiceAvailabilityState) -> Self {
        Self::BlackboardOpenError(value.into())
    }
}

impl From<BlackboardOpenError> for BlackboardOpenOrCreateError {
    fn from(value: BlackboardOpenError) -> Self {
        Self::BlackboardOpenError(value)
    }
}

impl From<BlackboardCreateError> for BlackboardOpenOrCreateError {
    fn from(value: BlackboardCreateError) -> Self {
        Self::BlackboardCreateError(value)
    }
}

#[derive(Debug)]
pub(crate) struct BlackboardResources<ServiceType: service::Service> {
    pub(crate) mgmt: ServiceType::BlackboardMgmt<AtomicU32>,
}

impl<ServiceType: service::Service> ServiceResource for BlackboardResources<ServiceType> {
    fn acquire_ownership() {
    }
}

/// Builder to create new [`MessagingPattern::Blackboard`] based [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Builder<KeyType: ZeroCopySend + Debug, ServiceType: service::Service> {
    base: builder::BuilderWithServiceType<ServiceType>,
    verify_max_readers: bool,
    verify_max_nodes: bool,
    _key: PhantomData<KeyType>,
}

impl<KeyType: ZeroCopySend + Debug, ServiceType: service::Service> Builder<KeyType, ServiceType> {
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        let mut new_self = Self {
            base,
            verify_max_readers: false,
            verify_max_nodes: false,
            _key: PhantomData,
        };

        new_self.base.service_config.messaging_pattern = MessagingPattern::Blackboard(
            static_config::blackboard::StaticConfig::new(new_self.base.shared_node.config()),
        );

        new_self
    }

    // triggers the underlying is_service_available method to check whether the service described in base is available.
    fn is_service_available(
        &mut self,
        error_msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceAvailabilityState> {
        match self.base.is_service_available(error_msg) {
            Ok(Some((config, storage))) => {
                if !(self.config_details().type_details == config.blackboard().type_details) {
                    fail!(from self, with ServiceAvailabilityState::IncompatibleKeys,
                        "{} since the service offers the type \"{:?}\" which is not compatible to the requested type \"{:?}\".",
                        error_msg, &config.blackboard().type_details , self.config_details().type_details);
                }

                Ok(Some((config, storage)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ServiceAvailabilityState::ServiceState(e)),
        }
    }

    fn config_details(&self) -> &static_config::blackboard::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::Blackboard(ref v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in Blackboard builder!");
            }
        }
    }

    fn config_details_mut(&mut self) -> &mut static_config::blackboard::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::Blackboard(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in Blackboard builder!");
            }
        }
    }

    /// If the [`Service`] is created it defines how many [`Reader`](crate::port::reader::Reader)s
    /// shall be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Reader`](crate::port::reader::Reader)s must be at least supported.
    pub fn max_readers(mut self, value: usize) -> Self {
        self.config_details_mut().max_readers = value;
        self.verify_max_readers = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`Node`](crate::node::Node)s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`](crate::node::Node)s must be at least supported.
    pub fn max_nodes(mut self, value: usize) -> Self {
        self.config_details_mut().max_nodes = value;
        self.verify_max_nodes = true;
        self
    }

    #[doc(hidden)]
    #[allow(unused_mut)]
    pub fn add<ValueType: ZeroCopySend>(mut self, _key: KeyType, _value: ValueType) -> Self {
        //todo!()
        self
    }

    fn prepare_config_details(&mut self) {
        self.config_details_mut().type_details =
            TypeDetail::__internal_new::<KeyType>(message_type_details::TypeVariant::FixedSize);
    }

    fn verify_service_configuration(
        &self,
        existing_settings: &static_config::StaticConfig,
        verifier: &AttributeVerifier,
    ) -> Result<static_config::blackboard::StaticConfig, BlackboardOpenError> {
        let msg = "Unable to open blackboard service";

        let existing_attributes = existing_settings.attributes();
        if let Err(incompatible_key) = verifier.verify_requirements(existing_attributes) {
            fail!(from self, with BlackboardOpenError::IncompatibleAttributes,
                "{} due to incompatible service attribute key \"{}\". The following attributes {:?} are required but the service has the attributes {:?}.",
                msg, incompatible_key, verifier, existing_attributes);
        }

        let required_settings = self.base.service_config.blackboard();
        let existing_settings = match &existing_settings.messaging_pattern {
            MessagingPattern::Blackboard(ref v) => v,
            p => {
                fail!(from self, with BlackboardOpenError::IncompatibleMessagingPattern,
                "{} since a service with the messaging pattern {:?} exists but MessagingPattern::Blackboard is required.", msg, p);
            }
        };

        if self.verify_max_readers && existing_settings.max_readers < required_settings.max_readers
        {
            fail!(from self, with BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders,
                                "{} since the service supports only {} readers but a support of {} readers was requested.",
                                msg, existing_settings.max_readers, required_settings.max_readers);
        }

        if self.verify_max_nodes && existing_settings.max_nodes < required_settings.max_nodes {
            fail!(from self, with BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes,
                                "{} since the service supports only {} nodes but {} are required.",
                                msg, existing_settings.max_nodes, required_settings.max_nodes);
        }

        Ok(existing_settings.clone())
    }

    /// Validates configuration and overrides the invalid setting with meaningful values.
    fn adjust_configuration_to_meaningful_values(&mut self) {
        let origin = format!("{self:?}");
        let settings = self.base.service_config.blackboard_mut();

        if settings.max_readers == 0 {
            warn!(from origin, "Setting the maximum amount of readers to 0 is not supported, Adjust it to 1, the smallest supported value.");
            settings.max_readers = 1;
        }

        if settings.max_nodes == 0 {
            warn!(from origin,
                "Setting the maximum amount of nodes to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_nodes = 1;
        }
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    pub fn open_or_create(
        self,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardOpenOrCreateError> {
        self.open_or_create_with_attributes(&AttributeVerifier::new())
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes. If the [`Service`] already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the [`Service`]
    /// does not exist the required attributes will be defined in the [`Service`].
    pub fn open_or_create_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardOpenOrCreateError> {
        self.prepare_config_details();

        let msg = "Unable to open or create blackboard service";

        let mut retry_count = 0;
        loop {
            if RETRY_LIMIT < retry_count {
                fail!(from self, with BlackboardOpenOrCreateError::SystemInFlux, "{} since an instance is creating and removing the same service repeatedly.", msg);
            }
            retry_count += 1;

            match self.is_service_available(msg)? {
                Some(_) => match self.open_impl(verifier) {
                    Ok(factory) => return Ok(factory),
                    Err(BlackboardOpenError::DoesNotExist) => continue,
                    Err(e) => return Err(e.into()),
                },
                None => {
                    match self
                        .create_impl(&AttributeSpecifier(verifier.required_attributes().clone()))
                    {
                        Ok(factory) => {
                            return Ok(factory);
                        }
                        Err(BlackboardCreateError::AlreadyExists)
                        | Err(BlackboardCreateError::IsBeingCreatedByAnotherInstance) => {
                            continue;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            }
        }
    }

    /// Opens an existing [`Service`].
    pub fn open(self) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardOpenError> {
        self.open_with_attributes(&AttributeVerifier::new())
    }

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardOpenError> {
        self.prepare_config_details();
        self.open_impl(verifier)
    }

    fn open_impl(
        &mut self,
        attributes: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardOpenError> {
        let msg = "Unable to open blackboard service";

        let mut service_open_retry_count = 0;
        loop {
            match self.is_service_available(msg)? {
                None => {
                    fail!(from self, with BlackboardOpenError::DoesNotExist, "{} since the service does not exist.", msg);
                }
                Some((static_config, static_storage)) => {
                    let blackboard_static_config =
                        self.verify_service_configuration(&static_config, attributes)?;

                    let service_tag = self
                        .base
                        .create_node_service_tag(msg, BlackboardOpenError::InternalFailure)?;

                    let dynamic_config = match self.base.open_dynamic_config_storage() {
                        Ok(v) => v,
                        Err(OpenDynamicStorageFailure::IsMarkedForDestruction) => {
                            fail!(from self, with BlackboardOpenError::IsMarkedForDestruction,
                                "{} since the service is marked for destruction.", msg);
                        }
                        Err(OpenDynamicStorageFailure::ExceedsMaxNumberOfNodes) => {
                            fail!(from self, with BlackboardOpenError::ExceedsMaxNumberOfNodes,
                                "{} since it would exceed the maximum number of supported nodes.", msg);
                        }
                        Err(OpenDynamicStorageFailure::DynamicStorageOpenError(
                            DynamicStorageOpenError::DoesNotExist,
                        )) => {
                            fail!(from self, with BlackboardOpenError::ServiceInCorruptedState,
                                "{} since the dynamic segment of the service is missing.", msg);
                        }
                        Err(e) => {
                            if self.is_service_available(msg)?.is_none() {
                                fail!(from self, with BlackboardOpenError::DoesNotExist, "{}, since the service does not exist.", msg);
                            }

                            service_open_retry_count += 1;

                            if RETRY_LIMIT < service_open_retry_count {
                                fail!(from self, with BlackboardOpenError::ServiceInCorruptedState,
                                    "{} since the dynamic service information could not be opened ({:?}). This could indicate a corrupted system or a misconfigured system where services are created/removed with a high frequency.",
                                    msg, e);
                            }

                            continue;
                        }
                    };

                    self.base.service_config.messaging_pattern =
                        MessagingPattern::Blackboard(blackboard_static_config.clone());

                    let storage_name = blackboard_mgmt_data_segment_name(
                        self.base.service_config.service_id().as_str(),
                    );
                    let storage_config = blackboard_mgmt_data_segment_config::<ServiceType, AtomicU32>(
                        self.base.shared_node.config(),
                    );
                    // TODO: error type and message
                    let storage = fail!(from self, 
                        when <ServiceType::BlackboardMgmt<AtomicU32> as iceoryx2_cal::dynamic_storage::DynamicStorage<AtomicU32>>::Builder::new(&storage_name).config(&storage_config).has_ownership(false).open(), with BlackboardOpenError::ServiceInCorruptedState,
                        "{} blub", msg);

                    if let Some(mut service_tag) = service_tag {
                        service_tag.release_ownership();
                    }

                    return Ok(blackboard::PortFactory::<ServiceType, AtomicU32>::new(
                        service::ServiceState::new(
                            static_config,
                            self.base.shared_node.clone(),
                            dynamic_config,
                            static_storage,
                            BlackboardResources { mgmt: storage },
                        ),
                    ));
                }
            }
        }
    }

    /// Creates a new [`Service`].
    pub fn create(mut self) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardCreateError> {
        self.prepare_config_details();
        self.create_impl(&AttributeSpecifier::new())
    }

    /// Creates a new [`Service`] with a set of attributes.
    pub fn create_with_attributes(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardCreateError> {
        self.prepare_config_details();
        self.create_impl(attributes)
    }

    fn create_impl(
        &mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType, AtomicU32>, BlackboardCreateError> {
        self.adjust_configuration_to_meaningful_values();

        let msg = "Unable to create blackboard service";

        match self.is_service_available(msg)? {
            Some(_) => {
                fail!(from self, with BlackboardCreateError::AlreadyExists, "{} since the service already exists.", msg);
            }
            None => {
                let service_tag = self
                    .base
                    .create_node_service_tag(msg, BlackboardCreateError::InternalFailure)?;

                // create static config
                let static_config = match self.base.create_static_config_storage() {
                    Ok(c) => c,
                    Err(StaticStorageCreateError::AlreadyExists) => {
                        fail!(from self, with BlackboardCreateError::AlreadyExists,
                           "{} since the service already exists.", msg);
                    }
                    Err(StaticStorageCreateError::Creation) => {
                        fail!(from self, with BlackboardCreateError::IsBeingCreatedByAnotherInstance,
                            "{} since the service is being created by another instance.", msg);
                    }
                    Err(StaticStorageCreateError::InsufficientPermissions) => {
                        fail!(from self, with BlackboardCreateError::InsufficientPermissions,
                            "{} since the static service information could not be created due to insufficient permissions.", msg);
                    }
                    Err(e) => {
                        fail!(from self, with BlackboardCreateError::InternalFailure,
                            "{} since the static service information could not be created due to an internal failure ({:?}).", msg, e);
                    }
                };

                let blackboard_config = self.base.service_config.blackboard();

                // create dynamic config
                let dynamic_config_setting = DynamicConfigSettings {
                    number_of_readers: blackboard_config.max_readers,
                };

                let dynamic_config = match self.base.create_dynamic_config_storage(
                    dynamic_config::MessagingPattern::Blackboard(
                        dynamic_config::blackboard::DynamicConfig::new(&dynamic_config_setting),
                    ),
                    dynamic_config::blackboard::DynamicConfig::memory_size(&dynamic_config_setting),
                    blackboard_config.max_nodes,
                ) {
                    Ok(dynamic_config) => dynamic_config,
                    Err(DynamicStorageCreateError::AlreadyExists) => {
                        fail!(from self, with BlackboardCreateError::ServiceInCorruptedState,
                            "{} since the dynamic config of a previous instance of the service still exists.", msg);
                    }
                    Err(e) => {
                        fail!(from self, with BlackboardCreateError::InternalFailure,
                            "{} since the dynamic service segment could not be created ({:?}).", msg, e);
                    }
                };

                self.base.service_config.attributes = attributes.0.clone();
                let service_config = fail!(from self,
                            when ServiceType::ConfigSerializer::serialize(&self.base.service_config),
                            with BlackboardCreateError::ServiceInCorruptedState,
                            "{} since the configuration could not be serialized.", msg);

                // create the management data segment including the flatmap; dynamic storage
                // additional size can be acquired by flatmap::memory_size()?
                // create naming scheme: iox2 + service id + blackboard_mgmt suffix
                // dynamic storage returns bump allocator?
                let storage_name = blackboard_mgmt_data_segment_name(
                    self.base.service_config.service_id().as_str(),
                );
                let storage_config = blackboard_mgmt_data_segment_config::<ServiceType, AtomicU32>(
                    self.base.shared_node.config(),
                );
                // TODO: error type and message
                let storage = fail!(from self, when
                    <ServiceType::BlackboardMgmt<AtomicU32> as iceoryx2_cal::dynamic_storage::DynamicStorage<
                        AtomicU32,
                    >>::Builder::new(&storage_name)
                        .config(&storage_config)
                        .has_ownership(false)
                    .create(AtomicU32::new(0)), with BlackboardCreateError::ServiceInCorruptedState, "{} blub", msg);

                // create the payload data segment for the writer; shm concept with allocator

                // only unlock the static details when the service is successfully created
                let unlocked_static_details = fail!(from self, when static_config.unlock(service_config.as_slice()),
                            with BlackboardCreateError::ServiceInCorruptedState,
                            "{} since the configuration could not be written to the static storage.", msg);

                unlocked_static_details.release_ownership();
                if let Some(service_tag) = service_tag {
                    service_tag.release_ownership();
                }

                Ok(blackboard::PortFactory::<ServiceType, AtomicU32>::new(
                    service::ServiceState::new(
                        self.base.service_config.clone(),
                        self.base.shared_node.clone(),
                        dynamic_config,
                        unlocked_static_details,
                        BlackboardResources { mgmt: storage }
                    ),
                ))
            }
        }
    }
}
