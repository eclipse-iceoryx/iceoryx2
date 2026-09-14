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

use iceoryx2::config::Config;
use iceoryx2::service::local;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_link_backend::description::{
    PublishSubscribeSettings, PublishSubscribeTypes, ServiceDescription, ServiceDescriptor,
    TypeDescription,
};

use iceoryx2_link_carrier::PeerId;

pub(crate) fn peer(discriminator: u8) -> PeerId {
    PeerId::new([discriminator; PeerId::LENGTH])
}

/// The publish-subscribe service `name` with `payload` as its payload
/// type and the default settings.
pub(crate) fn description(name: &str, payload: &str) -> ServiceDescription {
    let type_description = TypeDescription {
        variant: TypeVariant::FixedSize,
        type_name: String::from(payload),
        size: 8,
        alignment: 8,
    };
    ServiceDescription::compose_publish_subscribe::<local::Service>(
        ServiceName::new(name).expect("valid service name"),
        PublishSubscribeSettings::from_config(&Config::default()),
        PublishSubscribeTypes {
            payload: type_description.clone(),
            user_header: type_description,
        },
    )
}

pub(crate) fn descriptor(name: &str, payload: &str) -> ServiceDescriptor {
    ServiceDescriptor::from(&description(name, payload))
}
