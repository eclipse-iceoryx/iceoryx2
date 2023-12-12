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
use crate::service;
use crate::service::dynamic_config::publish_subscribe::DynamicConfigSettings;
use crate::service::messaging_pattern::MessagingPattern;
use crate::service::port_factory::publish_subscribe;
use crate::service::*;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::static_storage::StaticStorageLocked;

use super::ServiceState;

/// Errors that can occur when an existing [`MessagingPattern::PublishSubscribe`] [`Service`] shall be opened.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PublishSubscribeOpenError {
    DoesNotExist,
    InternalFailure,
    IncompatibleTypes,
    IncompatibleMessagingPattern,
    DoesNotSupportRequestedMinBufferSize,
    DoesNotSupportRequestedMinHistorySize,
    DoesNotSupportRequestedMinSubscriberBorrowedSamples,
    DoesNotSupportRequestedAmountOfPublishers,
    DoesNotSupportRequestedAmountOfSubscribers,
    IncompatibleOverflowBehavior,
    Inaccessible,
    PermissionDenied,
    ServiceInCorruptedState,
    HangsInCreation,
    UnableToOpenDynamicServiceInformation,
}

impl std::fmt::Display for PublishSubscribeOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublishSubscribeOpenError {}

/// Errors that can occur when a new [`MessagingPattern::PublishSubscribe`] [`Service`] shall be created.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PublishSubscribeCreateError {
    Corrupted,
    SubscriberBufferMustBeLargerThanHistorySize,
    AlreadyExists,
    PermissionDenied,
    InternalFailure,
    IsBeingCreatedByAnotherInstance,
    UnableToCreateStaticServiceInformation,
}

impl std::fmt::Display for PublishSubscribeCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublishSubscribeCreateError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceAvailabilityState {
    ServiceState(ServiceState),
    IncompatibleTypes,
}

enum_gen! {
    /// Errors that can occur when a [`MessagingPattern::PublishSubscribe`] [`Service`] shall be
    /// created or opened.
    PublishSubscribeOpenOrCreateError
  mapping:
    PublishSubscribeOpenError,
    PublishSubscribeCreateError
}

impl std::fmt::Display for PublishSubscribeOpenOrCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublishSubscribeOpenOrCreateError {}

/// Builder to create new [`MessagingPattern::PublishSubscribe`] based [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Builder<'config, ServiceType: service::Details<'config>> {
    base: builder::BuilderWithServiceType<'config, ServiceType>,
    verify_number_of_subscribers: bool,
    verify_number_of_publishers: bool,
    verify_subscriber_max_buffer_size: bool,
    verify_subscriber_max_borrowed_samples: bool,
    verify_publisher_history_size: bool,
    verify_enable_safe_overflow: bool,
}

impl<'config, ServiceType: service::Details<'config>> Builder<'config, ServiceType> {
    pub(crate) fn new(base: builder::BuilderWithServiceType<'config, ServiceType>) -> Self {
        let mut new_self = Self {
            base,
            verify_number_of_publishers: false,
            verify_number_of_subscribers: false,
            verify_subscriber_max_buffer_size: false,
            verify_publisher_history_size: false,
            verify_subscriber_max_borrowed_samples: false,
            verify_enable_safe_overflow: false,
        };

        new_self.base.service_config.messaging_pattern = MessagingPattern::PublishSubscribe(
            static_config::publish_subscribe::StaticConfig::new(new_self.base.global_config),
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

    /// If the [`Service`] is created, defines the overflow behavior of the service. If an existing
    /// [`Service`] is opened it requires the service to have the defined overflow behavior.
    pub fn enable_safe_overflow(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow = value;
        self.verify_enable_safe_overflow = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::sample::Sample`] a
    /// [`crate::port::subscriber::Subscriber`] can borrow at most in parallel. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn subscriber_max_borrowed_samples(mut self, value: usize) -> Self {
        self.config_details_mut().subscriber_max_borrowed_samples = std::cmp::max(value, 1);
        self.verify_subscriber_max_borrowed_samples = true;
        self
    }

    /// If the [`Service`] is created it defines the maximum history size a
    /// [`crate::port::subscriber::Subscriber`] can request on connection. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn history_size(mut self, value: usize) -> Self {
        self.config_details_mut().history_size = value;
        self.verify_publisher_history_size = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::sample::Sample`] a
    /// [`crate::port::subscriber::Subscriber`] can store in its internal buffer. If an existing
    /// [`Service`] is opened it defines the minimum required.
    pub fn subscriber_max_buffer_size(mut self, value: usize) -> Self {
        self.config_details_mut().subscriber_max_buffer_size = value;
        self.verify_subscriber_max_buffer_size = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::port::subscriber::Subscriber`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::subscriber::Subscriber`] must be at least supported.
    pub fn max_subscribers(mut self, value: usize) -> Self {
        self.config_details_mut().max_subscribers = value;
        self.verify_number_of_subscribers = true;
        self
    }

    /// If the [`Service`] is created it defines how many [`crate::port::publisher::Publisher`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::publisher::Publisher`] must be at least supported.
    pub fn max_publishers(mut self, value: usize) -> Self {
        self.config_details_mut().max_publishers = value;
        self.verify_number_of_publishers = true;
        self
    }

    fn is_service_available(
        &mut self,
        error_msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceAvailabilityState> {
        match self.base.is_service_available() {
            Ok(Some((config, storage))) => {
                if config.publish_subscribe().type_name != self.config_details().type_name {
                    fail!(from self, with ServiceAvailabilityState::IncompatibleTypes,
                        "{} since the service offers the type \"{}\" but the requested type is \"{}\".",
                        error_msg, &config.publish_subscribe().type_name , self.config_details().type_name);
                }

                Ok(Some((config, storage)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ServiceAvailabilityState::ServiceState(e)),
        }
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    pub fn open_or_create<MessageType: Debug>(
        mut self,
    ) -> Result<
        publish_subscribe::PortFactory<'config, ServiceType, MessageType>,
        PublishSubscribeOpenOrCreateError,
    > {
        let msg = "Unable to open or create publish subscribe service";
        self.config_details_mut().type_name = std::any::type_name::<MessageType>().to_string();

        match self.is_service_available(msg) {
            Ok(Some(_)) => Ok(self.open::<MessageType>()?),
            Ok(None) => Ok(self.create::<MessageType>()?),
            Err(ServiceAvailabilityState::ServiceState(
                ServiceState::IsBeingCreatedByAnotherInstance,
            )) => Ok(self.open::<MessageType>()?),
            Err(ServiceAvailabilityState::IncompatibleTypes) => {
                fail!(from self, with PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(PublishSubscribeOpenError::IncompatibleTypes),
                    "{} since the service is not type compatible.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(
                ServiceState::IncompatibleMessagingPattern,
            )) => {
                fail!(from self, with PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(PublishSubscribeOpenError::IncompatibleMessagingPattern),
                    "{} since the services messaging pattern does not match.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(ServiceState::Corrupted)) => {
                fail!(from self, with PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(PublishSubscribeOpenError::ServiceInCorruptedState),
                    "{} since the service is in a corrupted state.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(ServiceState::PermissionDenied)) => {
                fail!(from self, with PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(PublishSubscribeOpenError::PermissionDenied),
                    "{} due to insufficient permissions to access the service.", msg);
            }
        }
    }

    /// Opens an existing [`Service`].
    pub fn open<MessageType: Debug>(
        mut self,
    ) -> Result<
        publish_subscribe::PortFactory<'config, ServiceType, MessageType>,
        PublishSubscribeOpenError,
    > {
        let msg = "Unable to open publish subscribe service";
        self.config_details_mut().type_name = std::any::type_name::<MessageType>().to_string();

        let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new().create(),
                                        with PublishSubscribeOpenError::InternalFailure,
                                        "{} since the adaptive wait could not be created.", msg);

        loop {
            match self.is_service_available(msg) {
                Ok(None) => {
                    fail!(from self, with PublishSubscribeOpenError::DoesNotExist,
                        "{} since the service does not exist.", msg);
                }
                Ok(Some((static_config, static_storage))) => {
                    let static_config = self.verify_service_properties(&static_config)?;

                    let dynamic_config = fail!(from self, when self.base.open_dynamic_config_storage(),
                            with PublishSubscribeOpenError::UnableToOpenDynamicServiceInformation,
                            "{} since the dynamic service information could not be opened.", msg);

                    self.base.service_config.messaging_pattern =
                        MessagingPattern::PublishSubscribe(static_config.clone());

                    return Ok(publish_subscribe::PortFactory::new(
                        ServiceType::from_state(service::ServiceState::new(
                            self.base.service_config,
                            self.base.global_config,
                            dynamic_config,
                            static_storage,
                        )),
                    ));
                }
                Err(ServiceAvailabilityState::ServiceState(
                    ServiceState::IsBeingCreatedByAnotherInstance,
                )) => {
                    let timeout = fail!(from self, when adaptive_wait.wait(),
                                        with PublishSubscribeOpenError::InternalFailure,
                                        "{} since the adaptive wait failed.", msg);

                    if timeout > self.base.global_config.global.service.creation_timeout {
                        fail!(from self, with PublishSubscribeOpenError::HangsInCreation,
                            "{} since the service hangs while being created, max timeout for service creation of {:?} exceeded. Waited for {:?} but the state did not change.",
                            msg, self.base.global_config.global.service.creation_timeout, timeout);
                    }
                }
                Err(ServiceAvailabilityState::IncompatibleTypes) => {
                    fail!(from self, with PublishSubscribeOpenError::IncompatibleTypes,
                    "{} since the service is not type compatible.", msg);
                }
                Err(ServiceAvailabilityState::ServiceState(
                    ServiceState::IncompatibleMessagingPattern,
                )) => {
                    fail!(from self, with PublishSubscribeOpenError::IncompatibleMessagingPattern,
                    "{} since the services messaging pattern does not match.", msg);
                }
                Err(ServiceAvailabilityState::ServiceState(ServiceState::Corrupted)) => {
                    fail!(from self, with PublishSubscribeOpenError::ServiceInCorruptedState,
                    "{} since the service is in a corrupted state.", msg);
                }
                Err(ServiceAvailabilityState::ServiceState(ServiceState::PermissionDenied)) => {
                    fail!(from self, with PublishSubscribeOpenError::PermissionDenied,
                    "{} due to insufficient permissions to access the service.", msg);
                }
            }
        }
    }

    /// Creates a new [`Service`].
    pub fn create<MessageType: Debug>(
        mut self,
    ) -> Result<
        publish_subscribe::PortFactory<'config, ServiceType, MessageType>,
        PublishSubscribeCreateError,
    > {
        self.adjust_properties_to_meaningful_values();

        let msg = "Unable to create publish subscribe service";
        self.config_details_mut().type_name = std::any::type_name::<MessageType>().to_string();

        if !self.config_details().enable_safe_overflow
            && (self.config_details().subscriber_max_buffer_size
                < self.config_details().history_size)
        {
            fail!(from self, with PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize,
                "{} since the history size is greater than the subscriber buffer size. The subscriber buffer size must be always greater or equal to the history size in the non-overflowing setup.", msg);
        }

        match self.is_service_available(msg) {
            Ok(None) => {
                // create static config
                let static_config = fail!(from self, when self.base.create_static_config_storage(),
                    with PublishSubscribeCreateError::UnableToCreateStaticServiceInformation,
                    "{} since the static service information could not be created.", msg);

                let pubsub_config = self.base.service_config.publish_subscribe();

                // create dynamic config
                let dynamic_config_setting = DynamicConfigSettings {
                    number_of_publishers: pubsub_config.max_publishers,
                    number_of_subscribers: pubsub_config.max_subscribers,
                };

                let dynamic_config = self.base.create_dynamic_config_storage(
                    dynamic_config::MessagingPattern::PublishSubscribe(
                        dynamic_config::publish_subscribe::DynamicConfig::new(
                            &dynamic_config_setting,
                        ),
                    ),
                    dynamic_config::publish_subscribe::DynamicConfig::memory_size(
                        &dynamic_config_setting,
                    ),
                );
                let dynamic_config = fail!(from self, when dynamic_config,
                    with PublishSubscribeCreateError::InternalFailure,
                    "{} since the dynamic service segment could not be created.", msg);

                let service_config = fail!(from self, when ServiceType::ConfigSerializer::serialize(&self.base.service_config),
                            with PublishSubscribeCreateError::Corrupted,
                            "{} since the configuration could not be serialized.", msg);

                // only unlock the static details when the service is successfully created
                let unlocked_static_details = fail!(from self, when static_config.unlock(service_config.as_slice()),
                            with PublishSubscribeCreateError::Corrupted,
                            "{} since the configuration could not be written to the static storage.", msg);

                return Ok(publish_subscribe::PortFactory::new(
                    ServiceType::from_state(service::ServiceState::new(
                        self.base.service_config.clone(),
                        self.base.global_config,
                        dynamic_config,
                        unlocked_static_details,
                    )),
                ));
            }
            Ok(Some(_))
            | Err(ServiceAvailabilityState::IncompatibleTypes)
            | Err(ServiceAvailabilityState::ServiceState(
                ServiceState::IncompatibleMessagingPattern,
            )) => {
                fail!(from self, with PublishSubscribeCreateError::AlreadyExists,
                    "{} since the service already exists.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(ServiceState::PermissionDenied)) => {
                fail!(from self, with PublishSubscribeCreateError::PermissionDenied,
                    "{} due to possible insufficient permissions to access the underlying service details.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(ServiceState::Corrupted)) => {
                fail!(from self, with PublishSubscribeCreateError::Corrupted,
                    "{} since a service in a corrupted state already exists. A cleanup of the service constructs may help.", msg);
            }
            Err(ServiceAvailabilityState::ServiceState(
                ServiceState::IsBeingCreatedByAnotherInstance,
            )) => {
                fail!(from self, with PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance,
                    "{} since the service is being created by another instance.", msg);
            }
        }
    }

    fn adjust_properties_to_meaningful_values(&mut self) {
        let origin = format!("{:?}", self);
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
    }

    fn verify_service_properties(
        &self,
        existing_settings: &static_config::StaticConfig,
    ) -> Result<static_config::publish_subscribe::StaticConfig, PublishSubscribeOpenError> {
        let msg = "Unable to open publish subscribe service";

        let required_settings = self.base.service_config.publish_subscribe();
        let existing_settings = match &existing_settings.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v,
            p => {
                fail!(from self, with PublishSubscribeOpenError::IncompatibleMessagingPattern,
                "{} since a service with the messaging pattern {:?} exists but MessagingPattern::PublishSubscribe is required.", msg, p);
            }
        };

        if self.verify_number_of_publishers
            && existing_settings.max_publishers < required_settings.max_publishers
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers,
                                "{} since the service supports only {} publishers but a support of {} publishers was requested.",
                                msg, existing_settings.max_publishers, required_settings.max_publishers);
        }

        if self.verify_number_of_subscribers
            && existing_settings.max_subscribers < required_settings.max_subscribers
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers,
                                "{} since the service supports only {} subscribers but a support of {} subscribers was requested.",
                                msg, existing_settings.max_subscribers, required_settings.max_subscribers);
        }

        if self.verify_subscriber_max_buffer_size
            && existing_settings.subscriber_max_buffer_size
                < required_settings.subscriber_max_buffer_size
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize,
                                "{} since the service supports only a subscriber buffer size of {} but a buffer size of {} was requested.",
                                msg, existing_settings.subscriber_max_buffer_size, required_settings.subscriber_max_buffer_size);
        }

        if self.verify_publisher_history_size
            && existing_settings.history_size < required_settings.history_size
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize,
                                "{} since the service supports only a history size of {} but a history size of {} was requested.",
                                msg, existing_settings.history_size, required_settings.history_size);
        }

        if self.verify_subscriber_max_borrowed_samples
            && existing_settings.subscriber_max_borrowed_samples
                < required_settings.subscriber_max_borrowed_samples
        {
            fail!(from self, with PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples,
                                "{} since the service supports only {} borrowed subscriber samples but a {} borrowed subscriber samples were requested.",
                                msg, existing_settings.subscriber_max_borrowed_samples, required_settings.subscriber_max_borrowed_samples);
        }

        if self.verify_enable_safe_overflow
            && existing_settings.enable_safe_overflow != required_settings.enable_safe_overflow
        {
            fail!(from self, with PublishSubscribeOpenError::IncompatibleOverflowBehavior,
                                "{} since the service has an incompatible safe overflow behavior.",
                                msg);
        }

        Ok(existing_settings.clone())
    }
}
