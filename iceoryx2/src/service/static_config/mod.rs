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
pub mod type_details;

use crate::service::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::config;

use super::{attribute::AttributeSet, service_name::ServiceName};

/// Defines a common set of static service configuration details every service shares.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct StaticConfig {
    uuid: String,
    service_name: ServiceName,
    properties: AttributeSet,
    pub(crate) messaging_pattern: MessagingPattern,
}

fn create_uuid<Hasher: Hash>(
    service_name: &ServiceName,
    messaging_pattern: &MessagingPattern,
) -> Hasher {
    let pattern_and_service = (<MessagingPattern as Into<u32>>::into(messaging_pattern.clone()))
        .to_string()
        + service_name.as_str();
    Hasher::new(pattern_and_service.as_bytes())
}

impl StaticConfig {
    pub(crate) fn new_event<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
        properties: AttributeSet,
    ) -> Self {
        let messaging_pattern = MessagingPattern::Event(event::StaticConfig::new(config));
        Self {
            uuid: create_uuid::<Hasher>(service_name, &messaging_pattern)
                .value()
                .into(),
            service_name: service_name.clone(),
            messaging_pattern,
            properties,
        }
    }

    pub(crate) fn new_publish_subscribe<Hasher: Hash>(
        service_name: &ServiceName,
        config: &config::Config,
        properties: AttributeSet,
    ) -> Self {
        let messaging_pattern =
            MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(config));
        Self {
            uuid: create_uuid::<Hasher>(service_name, &messaging_pattern)
                .value()
                .into(),
            service_name: service_name.clone(),
            messaging_pattern,
            properties,
        }
    }

    /// Returns the properties of the [`crate::service::Service`]
    pub fn properties(&self) -> &AttributeSet {
        &self.properties
    }

    /// Returns the uuid of the [`crate::service::Service`]
    pub fn uuid(&self) -> &str {
        &self.uuid
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

    pub(crate) fn event(&self) -> &event::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::Event(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen. Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn event_mut(&mut self) -> &mut event::StaticConfig {
        let origin = format!("{:?}", self);
        match &mut self.messaging_pattern {
            MessagingPattern::Event(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen. Trying to access event::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn publish_subscribe(&self) -> &publish_subscribe::StaticConfig {
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen. Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }

    pub(crate) fn publish_subscribe_mut(&mut self) -> &mut publish_subscribe::StaticConfig {
        let origin = format!("{:?}", self);
        match &mut self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref mut v) => v,
            m => {
                fatal_panic!(from origin, "This should never happen. Trying to access publish_subscribe::StaticConfig when the messaging pattern is actually {:?}!", m)
            }
        }
    }
}
