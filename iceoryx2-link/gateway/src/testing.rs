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

use alloc::string::String;
use alloc::vec::Vec;

use iceoryx2::service::local;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_link_backend::service_description::{
    PublishSubscribeSettings, ServiceDescription, TypeDescription,
};

use iceoryx2_link_adapter::EndpointDescription;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{NoTranscoder, PublishSubscribeTranslation, Translator};
use iceoryx2_link_backend::service_description::Identified;
use iceoryx2_link_backend::service_description::{
    PatternSettings, PublishSubscribeTypes, ServiceSettings, ServiceTypes,
};

/// The settings of an endpoint, a name and the settings it was created
/// with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StubEndpointSettings {
    pub(crate) name: ServiceName,
    pub(crate) settings: PublishSubscribeSettings,
}

impl Identified for StubEndpointSettings {
    type Id = ServiceName;

    fn id(&self) -> ServiceName {
        self.name
    }
}

pub(crate) type StubEndpointDescription = EndpointDescription<StubEndpointSettings, ServiceTypes>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Refused;

impl core::fmt::Display for Refused {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Refused")
    }
}

impl core::error::Error for Refused {}

/// Maps endpoints to the services of the same name, defining nothing.
pub(crate) struct IdentityMapping;

impl Mapping for IdentityMapping {
    type EndpointSettings = StubEndpointSettings;
    type Error = core::convert::Infallible;

    fn local(&self, remote: &StubEndpointSettings) -> Result<Option<ServiceSettings>, Self::Error> {
        Ok(Some(ServiceSettings::new(
            remote.name,
            PatternSettings::PublishSubscribe(remote.settings.clone()),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<StubEndpointSettings>, Self::Error> {
        let PatternSettings::PublishSubscribe(settings) = &local.pattern else {
            return Ok(None);
        };
        Ok(Some(StubEndpointSettings {
            name: local.id(),
            settings: settings.clone(),
        }))
    }
}

/// Maps like the identity mapping and pins the settings of the services
/// it defines, refusing what differs from them in either direction, as a
/// configured mapping does.
pub(crate) struct DefiningMapping(pub(crate) Vec<ServiceSettings>);

impl DefiningMapping {
    fn defined(&self, name: &ServiceName) -> Option<&ServiceSettings> {
        self.0.iter().find(|defined| defined.id() == *name)
    }
}

impl Mapping for DefiningMapping {
    type EndpointSettings = StubEndpointSettings;
    type Error = Refused;

    fn local(&self, remote: &StubEndpointSettings) -> Result<Option<ServiceSettings>, Refused> {
        if let Some(defined) = self.defined(&remote.name)
            && defined.pattern != PatternSettings::PublishSubscribe(remote.settings.clone())
        {
            return Err(Refused);
        }
        Ok(IdentityMapping.local(remote).expect("never fails"))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<StubEndpointSettings>, Refused> {
        if let Some(defined) = self.defined(&local.id())
            && defined != local
        {
            return Err(Refused);
        }
        Ok(IdentityMapping.remote(local).expect("never fails"))
    }
}

/// Maps like the identity mapping, except that names under `redirected`
/// lead to `redirected/elsewhere` in either direction, so the mapping
/// does not lead back to them.
pub(crate) struct RedirectingMapping;

fn redirected(name: &ServiceName) -> Option<ServiceName> {
    name.as_str()
        .starts_with("redirected")
        .then(|| ServiceName::new("redirected/elsewhere").expect("a valid service name"))
}

impl Mapping for RedirectingMapping {
    type EndpointSettings = StubEndpointSettings;
    type Error = core::convert::Infallible;

    fn local(&self, remote: &StubEndpointSettings) -> Result<Option<ServiceSettings>, Self::Error> {
        let name = redirected(&remote.name).unwrap_or(remote.name);
        Ok(Some(ServiceSettings::new(
            name,
            PatternSettings::PublishSubscribe(remote.settings.clone()),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<StubEndpointSettings>, Self::Error> {
        let PatternSettings::PublishSubscribe(settings) = &local.pattern else {
            return Ok(None);
        };
        Ok(Some(StubEndpointSettings {
            name: redirected(&local.id()).unwrap_or(local.id()),
            settings: settings.clone(),
        }))
    }
}

/// Maps like the identity mapping, except that names under `uncovered`
/// are not covered and names under `unmappable` have no counterpart, in
/// either direction.
pub(crate) struct PartialMapping;

fn uncovered(name: &ServiceName) -> bool {
    name.as_str().starts_with("uncovered")
}

fn unmappable(name: &ServiceName) -> bool {
    name.as_str().starts_with("unmappable")
}

impl Mapping for PartialMapping {
    type EndpointSettings = StubEndpointSettings;
    type Error = Refused;

    fn local(&self, remote: &StubEndpointSettings) -> Result<Option<ServiceSettings>, Refused> {
        if uncovered(&remote.name) {
            return Ok(None);
        }
        if unmappable(&remote.name) {
            return Err(Refused);
        }
        Ok(IdentityMapping.local(remote).expect("never fails"))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<StubEndpointSettings>, Refused> {
        if uncovered(&local.id()) {
            return Ok(None);
        }
        if unmappable(&local.id()) {
            return Err(Refused);
        }
        Ok(IdentityMapping.remote(local).expect("never fails"))
    }
}

/// Passes bytes through but has no types for payloads named `refused`,
/// in either direction.
pub(crate) struct RefusingTranslator;

fn refuses(types: &ServiceTypes) -> bool {
    match types {
        ServiceTypes::PublishSubscribe(types) => types.payload.type_name == "refused",
        ServiceTypes::Event => false,
    }
}

impl Translator for RefusingTranslator {
    type EndpointTypes = ServiceTypes;
    type Error = Refused;
    type Transcoder = NoTranscoder;

    fn local(&self, remote: &ServiceTypes) -> Result<ServiceTypes, Refused> {
        match refuses(remote) {
            true => Err(Refused),
            false => Ok(remote.clone()),
        }
    }

    fn remote(&self, local: &ServiceTypes) -> Result<ServiceTypes, Refused> {
        match refuses(local) {
            true => Err(Refused),
            false => Ok(local.clone()),
        }
    }

    fn publish_subscribe(
        &self,
        _: &ServiceTypes,
        _: &ServiceTypes,
    ) -> Result<PublishSubscribeTranslation<NoTranscoder>, Refused> {
        Ok(PublishSubscribeTranslation::Passthrough)
    }
}

/// An adapter whose endpoints and generation are set by the test.
pub(crate) fn settings(history_size: usize) -> PublishSubscribeSettings {
    let mut settings = PublishSubscribeSettings::from_config(&iceoryx2::config::Config::default());
    settings.history_size = history_size;
    settings
}

pub(crate) fn types_of(payload: &str) -> ServiceTypes {
    let type_description = TypeDescription {
        variant: TypeVariant::FixedSize,
        type_name: String::from(payload),
        size: 8,
        alignment: 8,
    };
    ServiceTypes::PublishSubscribe(PublishSubscribeTypes {
        payload: type_description.clone(),
        user_header: type_description,
    })
}

pub(crate) fn endpoint_of(
    name: &str,
    history_size: usize,
    payload: &str,
) -> StubEndpointDescription {
    StubEndpointDescription {
        settings: StubEndpointSettings {
            name: ServiceName::new(name).expect("valid service name"),
            settings: settings(history_size),
        },
        types: types_of(payload),
    }
}

pub(crate) fn endpoint(name: &str, history_size: usize) -> StubEndpointDescription {
    endpoint_of(name, history_size, "u64")
}

/// The description a local application creating the service would have,
/// the same the gateway composes for the endpoint.
pub(crate) fn description_of(name: &str, history_size: usize, payload: &str) -> ServiceDescription {
    let endpoint = endpoint_of(name, history_size, payload);
    let settings = IdentityMapping
        .local(&endpoint.settings)
        .expect("never fails")
        .expect("covered");
    let PatternSettings::PublishSubscribe(settings) = settings.pattern else {
        panic!("a publish-subscribe endpoint");
    };
    let ServiceTypes::PublishSubscribe(types) = endpoint.types else {
        panic!("a publish-subscribe endpoint");
    };
    ServiceDescription::compose_publish_subscribe::<local::Service>(
        endpoint.settings.name,
        settings,
        types,
    )
}

/// The event service `name` with the default settings.
pub(crate) fn event(name: &str) -> ServiceDescription {
    ServiceDescription::with_default_settings::<local::Service>(
        ServiceName::new(name).expect("valid service name"),
        ServiceTypes::Event,
        &iceoryx2::config::Config::default(),
    )
}

pub(crate) fn description(name: &str, history_size: usize) -> ServiceDescription {
    description_of(name, history_size, "u64")
}
