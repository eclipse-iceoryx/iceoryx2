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

use std::ops::Deref;

use crate::service::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::config;

use super::service_name::ServiceName;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct Property {
    key: String,
    value: String,
}

impl Property {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ServiceProperties(Vec<Property>);

impl Deref for ServiceProperties {
    type Target = [Property];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl ServiceProperties {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn add(&mut self, key: &str, value: &str) {
        self.0.push(Property {
            key: key.into(),
            value: value.into(),
        });
        self.0.sort();
    }

    pub fn get(&self, key: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|p| p.key == key)
            .map(|p| p.value.as_str())
            .collect()
    }

    pub(crate) fn is_compatible_to(&self, rhs: &ServiceProperties) -> Result<(), &str> {
        let is_subset = |lhs: Vec<&str>, rhs: Vec<&str>| lhs.iter().all(|v| rhs.contains(v));

        for property in &self.0 {
            let lhs_values = self.get(&property.key);
            let rhs_values = rhs.get(&property.key);

            if !is_subset(lhs_values, rhs_values) {
                return Err(&property.key);
            }
        }

        Ok(())
    }
}

/// Defines a common set of static service configuration details every service shares.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct StaticConfig {
    uuid: String,
    service_name: ServiceName,
    properties: ServiceProperties,
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
        properties: ServiceProperties,
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
        properties: ServiceProperties,
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
    pub fn properties(&self) -> &ServiceProperties {
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
