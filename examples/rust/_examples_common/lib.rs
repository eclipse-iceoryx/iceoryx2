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

mod cross_language_complex_type;
mod custom_header;
mod pubsub_event;
mod transmission_data;

pub use cross_language_complex_type::*;
pub use custom_header::*;
pub use pubsub_event::*;
pub use transmission_data::*;

use iceoryx2::{
    prelude::*,
    service::port_factory::{event, publish_subscribe},
};

pub type ServiceTuple = (
    event::PortFactory<ipc::Service>,
    publish_subscribe::PortFactory<ipc::Service, u64, ()>,
);

pub fn open_service(
    node: &Node<ipc::Service>,
    service_name: &ServiceName,
) -> Result<ServiceTuple, Box<dyn core::error::Error>> {
    let service_pubsub = node
        .service_builder(service_name)
        .publish_subscribe::<u64>()
        .open()?;

    let service_event = node.service_builder(service_name).event().open()?;

    Ok((service_event, service_pubsub))
}
