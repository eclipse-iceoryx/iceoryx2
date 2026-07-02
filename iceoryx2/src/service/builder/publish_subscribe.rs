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
//!
use core::marker::PhantomData;

use alloc::format;

use iceoryx2_bb_elementary::alignment::Alignment;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_flatbuffers::TypeName;
use iceoryx2_log::{fail, fatal_panic, warn};

use super::ServiceState;
use crate::service::builder::{DynamicConfigCreationArgs, ServiceCreateError, ServiceOpenError};
use crate::service::dynamic_config::publish_subscribe::DynamicConfigSettings;
use crate::service::header::publish_subscribe::Header;
use crate::service::marker::{CustomHeaderMarker, CustomPayloadMarker, Flatbuffer};
use crate::service::port_factory::publish_subscribe;
use crate::service::resource::publish_subscribe::{
    PublishSubscribeResourceConfig, PublishSubscribeResources,
};
use crate::service::static_config::messaging_pattern::MessagingPattern;
use crate::service::*;
use crate::service::{self, dynamic_config::MessagingPatternSettings};

use self::{
    attribute::{AttributeSpecifier, AttributeVerifier},
    message_type_details::{MessageTypeDetails, TypeDetail, TypeVariant},
};

/// Errors that can occur when an existing [`MessagingPattern::PublishSubscribe`] [`Service`] shall be opened.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PublishSubscribeOpenError {
    /// An interrupt signal was received.
    Interrupt,
    /// Service could not be openen since it does not exist
    DoesNotExist,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The [`Service`] has the wrong payload type.
    IncompatibleTypes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has a lower minimum buffer size than requested.
    DoesNotSupportRequestedMinBufferSize,
    /// The [`Service`] has a lower minimum history size than requested.
    DoesNotSupportRequestedMinHistorySize,
    /// The [`Service`] has a lower minimum subscriber borrow size than requested.
    DoesNotSupportRequestedMinSubscriberBorrowedSamples,
    /// The [`Service`] supports less [`Publisher`](crate::port::publisher::Publisher)s than requested.
    DoesNotSupportRequestedAmountOfPublishers,
    /// The [`Service`] supports less [`Subscriber`](crate::port::subscriber::Subscriber)s than requested.
    DoesNotSupportRequestedAmountOfSubscribers,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than requested.
    DoesNotSupportRequestedAmountOfNodes,
    /// The [`Service`] required overflow behavior is not compatible.
    IncompatibleOverflowBehavior,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
    /// The [`Node`](crate::node::Node) service tag could not be created. Required to track resources of dead nodes when cleaning them up.
    UnableToCreateServiceTag,
    /// The iceoryx2 service version does not match the one of the [`Service`].
    VersionMismatch,
}

impl core::fmt::Display for PublishSubscribeOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishSubscribeOpenError::{self:?}")
    }
}

impl core::error::Error for PublishSubscribeOpenError {}

impl From<ServiceState> for PublishSubscribeOpenError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::IncompatiblePayload => PublishSubscribeOpenError::IncompatibleTypes,
            ServiceState::IncompatibleMessagingPattern => {
                PublishSubscribeOpenError::IncompatibleMessagingPattern
            }
            ServiceState::InsufficientPermissions => {
                PublishSubscribeOpenError::InsufficientPermissions
            }
            ServiceState::Interrupt => PublishSubscribeOpenError::Interrupt,
            ServiceState::HangsInCreation => PublishSubscribeOpenError::HangsInCreation,
            ServiceState::Corrupted => PublishSubscribeOpenError::ServiceInCorruptedState,
            ServiceState::InternalFailure => PublishSubscribeOpenError::InternalFailure,
            ServiceState::VersionMismatch => PublishSubscribeOpenError::VersionMismatch,
        }
    }
}

impl From<ServiceOpenError> for PublishSubscribeOpenError {
    fn from(value: ServiceOpenError) -> Self {
        match value {
            ServiceOpenError::DoesNotExist => PublishSubscribeOpenError::DoesNotExist,
            ServiceOpenError::ExceedsMaxNumberOfNodes => {
                PublishSubscribeOpenError::ExceedsMaxNumberOfNodes
            }
            ServiceOpenError::HangsInCreation => PublishSubscribeOpenError::HangsInCreation,
            ServiceOpenError::IncompatibleMessagingPattern => {
                PublishSubscribeOpenError::IncompatibleMessagingPattern
            }
            ServiceOpenError::IncompatiblePayload => PublishSubscribeOpenError::IncompatibleTypes,
            ServiceOpenError::InsufficientPermissions => {
                PublishSubscribeOpenError::InsufficientPermissions
            }
            ServiceOpenError::InternalFailure => PublishSubscribeOpenError::InternalFailure,
            ServiceOpenError::IsMarkedForDestruction => {
                PublishSubscribeOpenError::IsMarkedForDestruction
            }
            ServiceOpenError::ServiceInCorruptedState => {
                PublishSubscribeOpenError::ServiceInCorruptedState
            }
            ServiceOpenError::UnableToCreateServiceTag => {
                PublishSubscribeOpenError::UnableToCreateServiceTag
            }
            ServiceOpenError::VersionMismatch => PublishSubscribeOpenError::VersionMismatch,
            ServiceOpenError::Interrupt => PublishSubscribeOpenError::Interrupt,
        }
    }
}

impl From<PublishSubscribeOpenError> for ServiceOpenError {
    fn from(value: PublishSubscribeOpenError) -> Self {
        match value {
            PublishSubscribeOpenError::DoesNotExist => ServiceOpenError::DoesNotExist,
            PublishSubscribeOpenError::ExceedsMaxNumberOfNodes => {
                ServiceOpenError::ExceedsMaxNumberOfNodes
            }
            PublishSubscribeOpenError::HangsInCreation => ServiceOpenError::HangsInCreation,
            PublishSubscribeOpenError::IncompatibleMessagingPattern => {
                ServiceOpenError::IncompatibleMessagingPattern
            }
            PublishSubscribeOpenError::IncompatibleTypes => ServiceOpenError::IncompatiblePayload,
            PublishSubscribeOpenError::InsufficientPermissions => {
                ServiceOpenError::InsufficientPermissions
            }
            PublishSubscribeOpenError::IsMarkedForDestruction => {
                ServiceOpenError::IsMarkedForDestruction
            }
            PublishSubscribeOpenError::ServiceInCorruptedState => {
                ServiceOpenError::ServiceInCorruptedState
            }
            PublishSubscribeOpenError::UnableToCreateServiceTag => {
                ServiceOpenError::UnableToCreateServiceTag
            }
            PublishSubscribeOpenError::VersionMismatch => ServiceOpenError::VersionMismatch,
            PublishSubscribeOpenError::Interrupt => ServiceOpenError::Interrupt,
            PublishSubscribeOpenError::InternalFailure
            | PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes
            | PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers
            | PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers
            | PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize
            | PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize
            | PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples
            | PublishSubscribeOpenError::IncompatibleAttributes
            | PublishSubscribeOpenError::IncompatibleOverflowBehavior => {
                ServiceOpenError::InternalFailure
            }
        }
    }
}

/// Errors that can occur when a new [`MessagingPattern::PublishSubscribe`] [`Service`] shall be created.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PublishSubscribeCreateError {
    /// An interrupt signal was received.
    Interrupt,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// Invalid [`Service`] configuration provided. The
    /// [`Subscriber`](crate::port::subscriber::Subscriber)s buffer size must be at least the size
    /// of the history. Otherwise, how could it hold the whole history?
    SubscriberBufferMustBeLargerThanHistorySize,
    /// The [`Service`] already exists.
    AlreadyExists,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Node`](crate::node::Node) service tag could not be created. Required to track resources of dead nodes when cleaning them up.
    UnableToCreateServiceTag,
    /// The [`Service`]s config could not be created and written to the static service configuration.
    ServiceConfigCouldNotBeCreated,
}

impl core::fmt::Display for PublishSubscribeCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishSubscribeCreateError::{self:?}")
    }
}

impl core::error::Error for PublishSubscribeCreateError {}

impl From<ServiceCreateError> for PublishSubscribeCreateError {
    fn from(value: ServiceCreateError) -> Self {
        match value {
            ServiceCreateError::AlreadyExists => PublishSubscribeCreateError::AlreadyExists,
            ServiceCreateError::InsufficientPermissions => {
                PublishSubscribeCreateError::InsufficientPermissions
            }
            ServiceCreateError::InternalFailure => PublishSubscribeCreateError::InternalFailure,
            ServiceCreateError::IsBeingCreatedByAnotherInstance => {
                PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance
            }
            ServiceCreateError::ServiceConfigCouldNotBeCreated => {
                PublishSubscribeCreateError::ServiceConfigCouldNotBeCreated
            }
            ServiceCreateError::ServiceInCorruptedState => {
                PublishSubscribeCreateError::ServiceInCorruptedState
            }
            ServiceCreateError::UnableToCreateServiceTag => {
                PublishSubscribeCreateError::UnableToCreateServiceTag
            }
            ServiceCreateError::Interrupt => PublishSubscribeCreateError::Interrupt,
        }
    }
}

impl From<PublishSubscribeCreateError> for ServiceCreateError {
    fn from(value: PublishSubscribeCreateError) -> Self {
        match value {
            PublishSubscribeCreateError::AlreadyExists => ServiceCreateError::AlreadyExists,
            PublishSubscribeCreateError::InsufficientPermissions => {
                ServiceCreateError::InsufficientPermissions
            }
            PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance => {
                ServiceCreateError::IsBeingCreatedByAnotherInstance
            }
            PublishSubscribeCreateError::ServiceConfigCouldNotBeCreated => {
                ServiceCreateError::ServiceConfigCouldNotBeCreated
            }
            PublishSubscribeCreateError::ServiceInCorruptedState => {
                ServiceCreateError::ServiceInCorruptedState
            }
            PublishSubscribeCreateError::UnableToCreateServiceTag => {
                ServiceCreateError::UnableToCreateServiceTag
            }
            PublishSubscribeCreateError::Interrupt => ServiceCreateError::Interrupt,
            PublishSubscribeCreateError::InternalFailure
            | PublishSubscribeCreateError::HangsInCreation
            | PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize => {
                ServiceCreateError::InternalFailure
            }
        }
    }
}

impl From<ServiceState> for PublishSubscribeCreateError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::IncompatiblePayload
            | ServiceState::IncompatibleMessagingPattern
            | ServiceState::VersionMismatch => PublishSubscribeCreateError::AlreadyExists,
            ServiceState::InsufficientPermissions => {
                PublishSubscribeCreateError::InsufficientPermissions
            }
            ServiceState::HangsInCreation => PublishSubscribeCreateError::HangsInCreation,
            ServiceState::Corrupted => PublishSubscribeCreateError::ServiceInCorruptedState,
            ServiceState::InternalFailure => PublishSubscribeCreateError::InternalFailure,
            ServiceState::Interrupt => PublishSubscribeCreateError::Interrupt,
        }
    }
}

/// Errors that can occur when a [`MessagingPattern::PublishSubscribe`] [`Service`] shall be
/// created or opened.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PublishSubscribeOpenOrCreateError {
    /// Failures that can occur when an existing [`Service`] could not be opened.
    PublishSubscribeOpenError(PublishSubscribeOpenError),
    /// Failures that can occur when a [`Service`] could not be created.
    PublishSubscribeCreateError(PublishSubscribeCreateError),
    /// Can occur when another process creates and removes the same [`Service`] repeatedly with a
    /// high frequency.
    SystemInFlux,
}

impl From<ServiceState> for PublishSubscribeOpenOrCreateError {
    fn from(value: ServiceState) -> Self {
        PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(value.into())
    }
}

impl From<PublishSubscribeOpenError> for PublishSubscribeOpenOrCreateError {
    fn from(value: PublishSubscribeOpenError) -> Self {
        Self::PublishSubscribeOpenError(value)
    }
}

impl From<PublishSubscribeCreateError> for PublishSubscribeOpenOrCreateError {
    fn from(value: PublishSubscribeCreateError) -> Self {
        Self::PublishSubscribeCreateError(value)
    }
}

impl core::fmt::Display for PublishSubscribeOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishSubscribeOpenOrCreateError::{self:?}")
    }
}

impl core::error::Error for PublishSubscribeOpenOrCreateError {}

#[derive(Default, Debug, Clone, Copy)]
struct Verify {
    number_of_subscribers: bool,
    number_of_publishers: bool,
    subscriber_max_buffer_size: bool,
    subscriber_max_borrowed_samples: bool,
    publisher_history_size: bool,
    enable_safe_overflow: bool,
    max_nodes: bool,
}

/// Builder to create new [`MessagingPattern::PublishSubscribe`] based [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Builder<
    Payload: Debug + ?Sized + ZeroCopySend,
    UserHeader: Debug + ZeroCopySend,
    ServiceType: service::Service,
> {
    base: builder::BuilderWithServiceType<ServiceType>,
    override_alignment: Option<usize>,
    override_payload_type: Option<TypeDetail>,
    override_user_header_type: Option<TypeDetail>,
    flatbuffer_schema_path: Option<FilePath>,
    verify: Verify,
    _data: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

impl<
    Payload: Debug + ?Sized + ZeroCopySend,
    UserHeader: Debug + ZeroCopySend,
    ServiceType: service::Service,
> Clone for Builder<Payload, UserHeader, ServiceType>
{
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            override_alignment: self.override_alignment,
            override_payload_type: self.override_payload_type,
            override_user_header_type: self.override_user_header_type,
            flatbuffer_schema_path: self.flatbuffer_schema_path,
            verify: self.verify,
            _data: PhantomData,
            _user_header: PhantomData,
        }
    }
}

impl<
    Payload: Debug + ?Sized + ZeroCopySend,
    UserHeader: Debug + ZeroCopySend,
    ServiceType: service::Service,
> Builder<Payload, UserHeader, ServiceType>
{
    fn has_flatbuffer_payload() -> bool {
        unsafe { Payload::type_name() == Flatbuffer::<()>::type_name() }
    }

    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        let mut new_self = Self {
            base,
            verify: Verify::default(),
            override_alignment: None,
            override_payload_type: None,
            override_user_header_type: None,
            flatbuffer_schema_path: None,
            _data: PhantomData,
            _user_header: PhantomData,
        };

        new_self.base.service_config.messaging_pattern = MessagingPattern::PublishSubscribe(
            static_config::publish_subscribe::StaticConfig::new(new_self.base.shared_node.config()),
        );

        new_self
    }

    fn config_details_mut(&mut self) -> &mut static_config::publish_subscribe::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in PublishSubscribe builder!");
            }
        }
    }

    fn config_details(&self) -> &static_config::publish_subscribe::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in PublishSubscribe builder!");
            }
        }
    }

    // triggers the underlying is_service_available method to check whether the service described in base is available.
    fn is_service_available(
        &self,
        error_msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState> {
        let pubsub_service_config = self.config_details();
        match self.base.is_service_available(error_msg) {
            Ok(Some((config, storage))) => {
                if !pubsub_service_config
                    .message_type_details
                    .is_compatible_to(&config.publish_subscribe().message_type_details)
                {
                    fail!(from self, with ServiceState::IncompatiblePayload,
                        "{} since the service offers the type \"{:?}\" which is not compatible to the requested type \"{:?}\".",
                        error_msg, &config.publish_subscribe().message_type_details , pubsub_service_config.message_type_details);
                }

                Ok(Some((config, storage)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Sets the user header type of the [`Service`].
    pub fn user_header<M: Debug + ZeroCopySend>(self) -> Builder<Payload, M, ServiceType> {
        unsafe { core::mem::transmute::<Self, Builder<Payload, M, ServiceType>>(self) }
    }

    /// If the [`Service`] is created, it defines the [`Alignment`] of the payload for the service. If
    /// an existing [`Service`] is opened it requires the service to have at least the defined
    /// [`Alignment`]. If the Payload [`Alignment`] is greater than the provided [`Alignment`]
    /// then the Payload [`Alignment`] is used.
    pub fn payload_alignment(mut self, alignment: Alignment) -> Self {
        self.override_alignment = Some(alignment.value());
        self
    }

    /// If the [`Service`] is created, defines the overflow behavior of the service. If an existing
    /// [`Service`] is opened it requires the service to have the defined overflow behavior.
    pub fn enable_safe_overflow(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow = value;
        self.verify.enable_safe_overflow = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::sample::Sample`] a
    /// [`crate::port::subscriber::Subscriber`] can borrow at most in parallel. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn subscriber_max_borrowed_samples(mut self, value: usize) -> Self {
        self.config_details_mut().subscriber_max_borrowed_samples = value;
        self.verify.subscriber_max_borrowed_samples = true;
        self
    }

    /// If the [`Service`] is created it defines the maximum history size a
    /// [`crate::port::subscriber::Subscriber`] can request on connection. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn history_size(mut self, value: usize) -> Self {
        self.config_details_mut().history_size = value;
        self.verify.publisher_history_size = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::sample::Sample`] a
    /// [`crate::port::subscriber::Subscriber`] can store in its internal buffer. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn subscriber_max_buffer_size(mut self, value: usize) -> Self {
        self.config_details_mut().subscriber_max_buffer_size = value;
        self.verify.subscriber_max_buffer_size = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::port::subscriber::Subscriber`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::subscriber::Subscriber`] must be at least supported.
    pub fn max_subscribers(mut self, value: usize) -> Self {
        self.config_details_mut().max_subscribers = value;
        self.verify.number_of_subscribers = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::port::publisher::Publisher`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::publisher::Publisher`] must be at least supported.
    pub fn max_publishers(mut self, value: usize) -> Self {
        self.config_details_mut().max_publishers = value;
        self.verify.number_of_publishers = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`Node`](crate::node::Node)s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`](crate::node::Node)s must be at least supported.
    pub fn max_nodes(mut self, value: usize) -> Self {
        self.config_details_mut().max_nodes = value;
        self.verify.max_nodes = true;
        self
    }

    /// Validates configuration and overrides the invalid setting with meaningful values.
    fn adjust_configuration_to_meaningful_values(&mut self) {
        let origin = format!("{self:?}");
        let settings = self.base.service_config.publish_subscribe_mut();

        if settings.subscriber_max_borrowed_samples == 0 {
            warn!(from origin,
                "Setting the subscribers max borrowed samples to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.subscriber_max_borrowed_samples = 1;
        }

        if settings.subscriber_max_buffer_size == 0 {
            warn!(from origin,
                "Setting the subscribers buffer size to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.subscriber_max_buffer_size = 1;
        }

        if settings.max_subscribers == 0 {
            warn!(from origin,
                "Setting the maximum amount of subscribers to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_subscribers = 1;
        }

        if settings.max_publishers == 0 {
            warn!(from origin,
                "Setting the maximum amount of publishers to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_publishers = 1;
        }

        if settings.max_nodes == 0 {
            warn!(from origin,
                "Setting the maximum amount of nodes to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_nodes = 1;
        }
    }

    fn verify_service_configuration(
        &self,
        msg: &str,
        existing_service_config: &StaticConfig,
        required_attributes: &AttributeVerifier,
    ) -> Result<(), PublishSubscribeOpenError> {
        let required_service_config = &self.base.service_config;
        let existing_attributes = existing_service_config.attributes();
        if let Err(incompatible_key) = required_attributes.verify_requirements(existing_attributes)
        {
            fail!(from self, with PublishSubscribeOpenError::IncompatibleAttributes,
                "{} due to incompatible service attribute key \"{}\". The following attributes {:?} are required but the service has the attributes {:?}.",
                msg, incompatible_key, required_attributes, existing_attributes);
        }

        let required_settings = required_service_config.publish_subscribe();
        let existing_settings = match &existing_service_config.messaging_pattern {
            MessagingPattern::PublishSubscribe(v) => v,
            p => {
                fail!(from self, with PublishSubscribeOpenError::IncompatibleMessagingPattern,
                "{} since a service with the messaging pattern {:?} exists but MessagingPattern::PublishSubscribe is required.", msg, p);
            }
        };

        if self.verify.number_of_publishers
            && existing_settings.max_publishers < required_settings.max_publishers
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers,
                                "{} since the service supports only {} publishers but a support of {} publishers was requested.",
                                msg, existing_settings.max_publishers, required_settings.max_publishers);
        }

        if self.verify.number_of_subscribers
            && existing_settings.max_subscribers < required_settings.max_subscribers
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers,
                                "{} since the service supports only {} subscribers but a support of {} subscribers was requested.",
                                msg, existing_settings.max_subscribers, required_settings.max_subscribers);
        }

        if self.verify.subscriber_max_buffer_size
            && existing_settings.subscriber_max_buffer_size
                < required_settings.subscriber_max_buffer_size
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize,
                                "{} since the service supports only a subscriber buffer size of {} but a buffer size of {} was requested.",
                                msg, existing_settings.subscriber_max_buffer_size, required_settings.subscriber_max_buffer_size);
        }

        if self.verify.publisher_history_size
            && existing_settings.history_size < required_settings.history_size
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize,
                                "{} since the service supports only a history size of {} but a history size of {} was requested.",
                                msg, existing_settings.history_size, required_settings.history_size);
        }

        if self.verify.subscriber_max_borrowed_samples
            && existing_settings.subscriber_max_borrowed_samples
                < required_settings.subscriber_max_borrowed_samples
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples,
                                "{} since the service supports only {} borrowed subscriber samples but a {} borrowed subscriber samples were requested.",
                                msg, existing_settings.subscriber_max_borrowed_samples, required_settings.subscriber_max_borrowed_samples);
        }

        if self.verify.enable_safe_overflow
            && existing_settings.enable_safe_overflow != required_settings.enable_safe_overflow
        {
            fail!(from self, with PublishSubscribeOpenError::IncompatibleOverflowBehavior,
                                "{} since the service has an incompatible safe overflow behavior.",
                                msg);
        }

        if self.verify.max_nodes && existing_settings.max_nodes < required_settings.max_nodes {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes,
                                "{} since the service supports only {} nodes but {} are required.",
                                msg, existing_settings.max_nodes, required_settings.max_nodes);
        }

        Ok(())
    }

    fn create_impl(
        &self,
        attributes: &AttributeSpecifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeCreateError,
    > {
        let msg = "Unable to create publish subscribe service";
        if !self.config_details().enable_safe_overflow
            && (self.config_details().subscriber_max_buffer_size
                < self.config_details().history_size)
        {
            fail!(from self, with PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize,
                "{} since the history size is greater than the subscriber buffer size. The subscriber buffer size must be always greater or equal to the history size in the non-overflowing setup.", msg);
        }

        let generate_dynamic_config = |service_config: &StaticConfig| {
            let pubsub_config = service_config.publish_subscribe();
            let dynamic_config_setting = DynamicConfigSettings {
                number_of_publishers: pubsub_config.max_publishers,
                number_of_subscribers: pubsub_config.max_subscribers,
            };

            DynamicConfigCreationArgs {
                messaging_pattern_settings: MessagingPatternSettings::PublishSubscribe(
                    dynamic_config_setting,
                ),
                additional_size: dynamic_config::publish_subscribe::DynamicConfig::memory_size(
                    &dynamic_config_setting,
                ),
                max_number_of_nodes: pubsub_config.max_nodes,
            }
        };

        let service_state = self.base.create(
            msg,
            attributes,
            || self.is_service_available(msg),
            |_| Ok(()),
            generate_dynamic_config,
            |service_config| {
                PublishSubscribeResources::create(
                    service_config,
                    &PublishSubscribeResourceConfig::<ServiceType> {
                        use_type_definition: Self::has_flatbuffer_payload(),
                        schema_path: self.flatbuffer_schema_path,
                        shared_node: self.base.shared_node.clone(),
                        type_name: TypeName::new::<Payload>(),
                    },
                )
            },
            |_| {},
        )?;

        Ok(publish_subscribe::PortFactory::new(service_state))
    }

    fn open_impl(
        &self,
        required_attributes: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenError,
    > {
        let msg = "Unable to open publish subscribe service";

        let service_state = self.base.open(
            msg,
            || self.is_service_available(msg),
            |existing_service_config| -> Result<(), PublishSubscribeOpenError> {
                self.verify_service_configuration(msg, existing_service_config, required_attributes)
            },
            |service_config| {
                PublishSubscribeResources::open(
                    service_config,
                    &PublishSubscribeResourceConfig::<ServiceType> {
                        use_type_definition: Self::has_flatbuffer_payload(),
                        schema_path: self.flatbuffer_schema_path,
                        shared_node: self.base.shared_node.clone(),
                        type_name: TypeName::new::<Payload>(),
                    },
                )
            },
        )?;

        Ok(publish_subscribe::PortFactory::new(service_state))
    }

    fn open_or_create_impl(
        self,
        attributes: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenOrCreateError,
    > {
        let msg = "Unable to open or create publish subscribe service";
        self.base.open_or_create(
            msg,
            attributes,
            PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(
                PublishSubscribeOpenError::InternalFailure,
            ),
            PublishSubscribeOpenOrCreateError::SystemInFlux,
            |attributes| self.open_impl(attributes),
            |attributes| self.create_impl(attributes),
        )
    }

    fn adjust_payload_alignment(&mut self) {
        if let Some(alignment) = self.override_alignment {
            self.config_details_mut()
                .message_type_details
                .payload
                .alignment = self
                .config_details()
                .message_type_details
                .payload
                .alignment
                .max(alignment);
        }
    }
}

impl<UserHeader: Debug + ZeroCopySend, ServiceType: service::Service>
    Builder<[CustomPayloadMarker], UserHeader, ServiceType>
{
    #[doc(hidden)]
    pub unsafe fn __internal_set_payload_type_details(mut self, value: &TypeDetail) -> Self {
        self.override_payload_type = Some(*value);
        self
    }
}

impl<Payload: Debug + ?Sized + ZeroCopySend, ServiceType: service::Service>
    Builder<Payload, CustomHeaderMarker, ServiceType>
{
    #[doc(hidden)]
    pub unsafe fn __internal_set_user_header_type_details(mut self, value: &TypeDetail) -> Self {
        self.override_user_header_type = Some(*value);
        self
    }
}

impl<Payload: Debug + ZeroCopySend, UserHeader: Debug + ZeroCopySend, ServiceType: service::Service>
    Builder<Payload, UserHeader, ServiceType>
{
    fn prepare_config_details(&mut self) {
        self.config_details_mut().message_type_details =
            MessageTypeDetails::from::<Header, UserHeader, Payload>(TypeVariant::FixedSize);

        if let Some(details) = &self.override_payload_type {
            self.config_details_mut().message_type_details.payload = *details;
        }

        if let Some(details) = &self.override_user_header_type {
            self.config_details_mut().message_type_details.user_header = *details;
        }

        self.adjust_payload_alignment();
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    pub fn open_or_create(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenOrCreateError,
    > {
        self.open_or_create_with_attributes(&AttributeVerifier::new())
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes.
    ///
    /// If the [`Service`] already exists all attribute requirements must be satisfied,
    /// and service payload type must be the same, otherwise the open process will fail.
    /// If the [`Service`] does not exist the required attributes will be defined in the [`Service`].
    pub fn open_or_create_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenOrCreateError,
    > {
        self.adjust_configuration_to_meaningful_values();
        self.prepare_config_details();
        self.open_or_create_impl(verifier)
    }

    /// Opens an existing [`Service`].
    pub fn open(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenError,
    > {
        self.open_with_attributes(&AttributeVerifier::new())
    }

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeOpenError,
    > {
        self.prepare_config_details();
        self.open_impl(verifier)
    }

    /// Creates a new [`Service`].
    pub fn create(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeCreateError,
    > {
        self.create_with_attributes(&AttributeSpecifier::new())
    }

    /// Creates a new [`Service`] with a set of attributes.
    pub fn create_with_attributes(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, Payload, UserHeader>,
        PublishSubscribeCreateError,
    > {
        self.adjust_configuration_to_meaningful_values();
        self.prepare_config_details();
        self.create_impl(attributes)
    }
}

impl<Payload: Debug, UserHeader: Debug + ZeroCopySend, ServiceType: service::Service>
    Builder<Flatbuffer<Payload>, UserHeader, ServiceType>
{
    /// Sets the path to the flatbuffer schema file. If this is not explicitly defined, iceoryx2
    /// will try to find the best fitting schema file in the configured filebuffer schema paths
    /// defined in the config.
    pub fn flatbuffer_schema_path(mut self, path: &FilePath) -> Self {
        self.flatbuffer_schema_path = Some(*path);
        self
    }
}

impl<Payload: Debug + ZeroCopySend, UserHeader: Debug + ZeroCopySend, ServiceType: service::Service>
    Builder<[Payload], UserHeader, ServiceType>
{
    fn prepare_config_details(&mut self) {
        self.config_details_mut().message_type_details =
            MessageTypeDetails::from::<Header, UserHeader, Payload>(TypeVariant::Dynamic);

        if let Some(details) = &self.override_payload_type {
            self.config_details_mut().message_type_details.payload = *details;
        }

        if let Some(details) = &self.override_user_header_type {
            self.config_details_mut().message_type_details.user_header = *details;
        }

        self.adjust_payload_alignment();
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    pub fn open_or_create(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeOpenOrCreateError,
    > {
        self.open_or_create_with_attributes(&AttributeVerifier::new())
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes. If the [`Service`] already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the [`Service`]
    /// does not exist the required attributes will be defined in the [`Service`].
    pub fn open_or_create_with_attributes(
        mut self,
        attributes: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeOpenOrCreateError,
    > {
        self.prepare_config_details();
        self.open_or_create_impl(attributes)
    }

    /// Opens an existing [`Service`].
    pub fn open(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeOpenError,
    > {
        self.open_with_attributes(&AttributeVerifier::new())
    }

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        mut self,
        attributes: &AttributeVerifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeOpenError,
    > {
        self.prepare_config_details();
        self.open_impl(attributes)
    }

    /// Creates a new [`Service`].
    pub fn create(
        self,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeCreateError,
    > {
        self.create_with_attributes(&AttributeSpecifier::new())
    }

    /// Creates a new [`Service`] with a set of attributes.
    pub fn create_with_attributes(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<
        publish_subscribe::PortFactory<ServiceType, [Payload], UserHeader>,
        PublishSubscribeCreateError,
    > {
        self.prepare_config_details();
        self.create_impl(attributes)
    }
}
