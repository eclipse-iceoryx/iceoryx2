// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::ops::Deref;

use crate::service_description::{
    EventSettings, PatternSettings, PublishSubscribeSettings, PublishSubscribeTypes,
    ServiceDescription, ServiceDescriptor, ServiceTypes,
};

/// A description seen as the pattern its halves are of, each arm giving
/// the halves typed.
#[derive(Clone, Copy)]
pub enum MessagingPattern<'a> {
    PublishSubscribe(PublishSubscribeDescription<'a>),
    Event(EventDescription<'a>),
}

impl<'a> MessagingPattern<'a> {
    pub(super) fn of(description: &'a ServiceDescription) -> Self {
        match (&description.settings.pattern, &description.types) {
            (
                PatternSettings::PublishSubscribe(settings),
                ServiceTypes::PublishSubscribe(types),
            ) => Self::PublishSubscribe(PublishSubscribeDescription {
                description,
                settings,
                types,
            }),
            (PatternSettings::Event(settings), ServiceTypes::Event) => {
                Self::Event(EventDescription {
                    description,
                    settings,
                })
            }
            _ => unreachable!("a description is composed of settings and types of one pattern"),
        }
    }
}

/// A description known to be of a publish-subscribe service.
#[derive(Clone, Copy)]
pub struct PublishSubscribeDescription<'a> {
    description: &'a ServiceDescription,
    settings: &'a PublishSubscribeSettings,
    types: &'a PublishSubscribeTypes,
}

impl<'a> PublishSubscribeDescription<'a> {
    pub fn settings(&self) -> &'a PublishSubscribeSettings {
        self.settings
    }

    pub fn types(&self) -> &'a PublishSubscribeTypes {
        self.types
    }

    pub fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::from(self.description)
    }
}

impl Deref for PublishSubscribeDescription<'_> {
    type Target = ServiceDescription;

    fn deref(&self) -> &Self::Target {
        self.description
    }
}

/// A description known to be of an event service.
#[derive(Clone, Copy)]
pub struct EventDescription<'a> {
    description: &'a ServiceDescription,
    settings: &'a EventSettings,
}

impl<'a> EventDescription<'a> {
    pub fn settings(&self) -> &'a EventSettings {
        self.settings
    }

    pub fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::from(self.description)
    }
}

impl Deref for EventDescription<'_> {
    type Target = ServiceDescription;

    fn deref(&self) -> &Self::Target {
        self.description
    }
}
