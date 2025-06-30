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

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::config;

use self::messaging_pattern::MessagingPattern;

use super::{attribute::AttributeSet, service_id::ServiceId, service_name::ServiceName};

/// Defines a common set of static service configuration details every service shares.
#[derive(Debug, Eq, PartialEq, Clone, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    service_id: ServiceId,
    service_name: ServiceName,
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
            service_id: ServiceId::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::RequestResponse,
            ),
            service_name: service_name.clone(),
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
            service_id: ServiceId::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::Event,
            ),
            service_name: service_name.clone(),
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
            service_id: ServiceId::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::PublishSubscribe,
            ),
            service_name: service_name.clone(),
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    pub(crate) fn new_blackboard<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Self {
        let messaging_pattern = MessagingPattern::Blackboard(blackboard::StaticConfig::new(config));
        Self {
            service_id: ServiceId::new::<Hasher>(
                service_name,
                crate::service::messaging_pattern::MessagingPattern::Blackboard,
            ),
            service_name: service_name.clone(),
            messaging_pattern,
            attributes: AttributeSet::new(),
        }
    }

    /// Returns the attributes of the [`crate::service::Service`]
    pub fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the uuid of the [`crate::service::Service`]
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
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
            MessagingPattern::RequestResponse(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access request_response::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn request_response_mut(&mut self) -> &mut request_response::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::RequestResponse(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access request_response::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the Event static configuration.
    pub fn event(&self) -> &event::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::Event(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn event_mut(&mut self) -> &mut event::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::Event(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the PublishSubscribe static configuration.
    pub fn publish_subscribe(&self) -> &publish_subscribe::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn publish_subscribe_mut(&mut self) -> &mut publish_subscribe::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    /// Unwrap the Blackboard static configuration.
    pub fn blackboard(&self) -> &blackboard::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::Blackboard(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access blackboard::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn blackboard_mut(&mut self) -> &mut blackboard::StaticConfig {
        let origin = format!("{self:?}");
        match &mut self.messaging_pattern {
            MessagingPattern::Blackboard(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen! Trying to access blackboard::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }
}
