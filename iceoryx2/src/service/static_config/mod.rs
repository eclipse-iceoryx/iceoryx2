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

/// The static service configuration of an
/// [`MessagingPattern::Event`]
/// based service.
pub mod event;

/// The static service configuration of an
/// [`MessagingPattern::PublishSubscribe`]
/// based service.
pub mod publish_subscribe;

/// Contains the size, alignment and name of the header and payload type
/// and the type variant
pub mod message_type_details;

pub mod request_response;

pub mod messaging_pattern;

pub mod blackboard;

use alloc::format;

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::hash::Hash;
use iceoryx2_log::fatal_panic;

use serde::{Deserialize, Serialize};

use crate::{config, identifiers::UniqueServiceId, service::service_hash::ServiceHash};

use self::messaging_pattern::MessagingPattern;

use super::{attribute::AttributeSet, service_name::ServiceName};

/// Defines a common set of static service configuration details every service shares.
#[derive(Debug, Eq, PartialEq, Clone, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    iceoryx2_version: PackageVersion,
    service_hash: ServiceHash,
    service_name: ServiceName,
    unique_service_id: UniqueServiceId,
    pub(crate) attributes: AttributeSet,
    pub(crate) messaging_pattern: MessagingPattern,
}

impl StaticConfig {
    pub(crate) fn new_request_response<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Self {
        let messaging_pattern =
            MessagingPattern::RequestResponse(request_response::StaticConfig::new(config));
        Self {
            iceoryx2_version: PackageVersion::get(),
            service_hash: ServiceHash::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::RequestResponse,
            ),
            unique_service_id: UniqueServiceId::new(),
            service_name: *service_name,
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    pub(crate) fn new_event<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Self {
        let messaging_pattern = MessagingPattern::Event(event::StaticConfig::new(config));
        Self {
            iceoryx2_version: PackageVersion::get(),
            service_hash: ServiceHash::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::Event,
            ),
            unique_service_id: UniqueServiceId::new(),
            service_name: *service_name,
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    pub(crate) fn new_publish_subscribe<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Self {
        let messaging_pattern =
            MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(config));
        Self {
            iceoryx2_version: PackageVersion::get(),
            service_hash: ServiceHash::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::PublishSubscribe,
            ),
            unique_service_id: UniqueServiceId::new(),
            service_name: *service_name,
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    /// Creates the [`StaticConfig`] of a publish-subscribe
    /// [`Service`](crate::service::Service) with the provided message type
    /// details; all other values are taken from the defaults in `config`,
    /// but this might need to be revised to be more precise.
    ///
    /// Intended for infrastructure that describes services it does not
    /// create itself, e.g. tunnels. Not part of the stable public API.
    #[doc(hidden)]
    pub fn __internal_new_publish_subscribe_with_details<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
        payload: message_type_details::TypeDetail,
        user_header: message_type_details::TypeDetail,
    ) -> Self {
        let mut new_self = Self::new_publish_subscribe::<Hasher>(service_name, config);
        match &mut new_self.messaging_pattern {
            MessagingPattern::PublishSubscribe(pattern_config) => {
                pattern_config.message_type_details.user_header = user_header;
                pattern_config.message_type_details.payload = payload;
            }
            // new_publish_subscribe always creates this pattern
            _ => unreachable!(),
        }
        new_self
    }

    pub(crate) fn new_blackboard<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Self {
        let messaging_pattern = MessagingPattern::Blackboard(blackboard::StaticConfig::new(config));
        Self {
            iceoryx2_version: PackageVersion::get(),
            service_hash: ServiceHash::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::Blackboard,
            ),
            unique_service_id: UniqueServiceId::new(),
            service_name: *service_name,
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    /// Returns the iceoryx2 version of the [`Service`](crate::service::Service)
    pub fn iceoryx2_version(&self) -> PackageVersion {
        self.iceoryx2_version
    }

    /// Returns the attributes of the [`crate::service::Service`]
    pub fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the hash of the [`crate::service::Service`]
    pub fn service_hash(&self) -> &ServiceHash {
        &self.service_hash
    }

    /// Returns the id of the [`crate::service::Service`]
    pub fn unique_service_id(&self) -> UniqueServiceId {
        self.unique_service_id
    }

    /// Returns the [`ServiceName`] of the [`crate::service::Service`]
    pub fn name(&self) -> &ServiceName {
        &self.service_name
    }

    /// Returns the [`MessagingPattern`] of the [`crate::service::Service`]
    pub fn messaging_pattern(&self) -> &MessagingPattern {
        &self.messaging_pattern
    }

    pub(crate) fn has_same_messaging_pattern(&self, rhs: &StaticConfig) -> bool {
        self.messaging_pattern
            .is_same_pattern(&rhs.messaging_pattern)
    }

    /// Unwrap the RequestResponse static configuration.
    pub fn request_response(&self) -> &request_response::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::RequestResponse(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access request_response::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn request_response_mut(&mut self) -> &mut request_response::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::RequestResponse(v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access request_response::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the Event static configuration.
    pub fn event(&self) -> &event::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::Event(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn event_mut(&mut self) -> &mut event::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::Event(v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the PublishSubscribe static configuration.
    pub fn publish_subscribe(&self) -> &publish_subscribe::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn publish_subscribe_mut(&mut self) -> &mut publish_subscribe::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::PublishSubscribe(v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the Blackboard static configuration.
    pub fn blackboard(&self) -> &blackboard::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::Blackboard(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access blackboard::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn blackboard_mut(&mut self) -> &mut blackboard::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::Blackboard(v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access blackboard::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }
}
