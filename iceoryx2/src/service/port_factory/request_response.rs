// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let req_res = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! println!("name:                             {:?}", req_res.name());
//! println!("service id:                       {:?}", req_res.service_id());
//! println!("request type details:             {:?}", req_res.static_config().request_message_type_details());
//! println!("response type details:            {:?}", req_res.static_config().response_message_type_details());
//! println!("max active requests:              {:?}", req_res.static_config().max_active_requests());
//! println!("max active responses:             {:?}", req_res.static_config().max_active_responses());
//! println!("max borrowed responses:           {:?}", req_res.static_config().max_borrowed_responses());
//! println!("max borrowed requests:            {:?}", req_res.static_config().max_borrowed_requests());
//! println!("max response buffer size:         {:?}", req_res.static_config().max_response_buffer_size());
//! println!("max request buffer size:          {:?}", req_res.static_config().max_request_buffer_size());
//! println!("max servers:                      {:?}", req_res.static_config().max_clients());
//! println!("max clients:                      {:?}", req_res.static_config().max_servers());
//! println!("max nodes:                        {:?}", req_res.static_config().max_nodes());
//! println!("request safe overflow:            {:?}", req_res.static_config().has_safe_overflow_for_requests());
//! println!("response safe overflow:           {:?}", req_res.static_config().has_safe_overflow_for_responses());
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::{
    node::NodeListFailure,
    prelude::AttributeSet,
    service::{
        self, dynamic_config, service_id::ServiceId, service_name::ServiceName, static_config,
    },
};

use super::nodes;

/// The factory for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::client::Client`]
/// or [`crate::port::server::Server`] ports.
#[derive(Debug)]
pub struct PortFactory<Service: service::Service> {
    pub(crate) service: Service,
}

unsafe impl<Service: service::Service> Send for PortFactory<Service> {}
unsafe impl<Service: service::Service> Sync for PortFactory<Service> {}

impl<Service: service::Service> crate::service::port_factory::PortFactory for PortFactory<Service> {
    type Service = Service;
    type StaticConfig = static_config::request_response::StaticConfig;
    type DynamicConfig = dynamic_config::request_response::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.__internal_state().static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.__internal_state().static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.__internal_state().static_config.attributes()
    }

    fn static_config(&self) -> &Self::StaticConfig {
        self.service
            .__internal_state()
            .static_config
            .request_response()
    }

    fn dynamic_config(&self) -> &Self::DynamicConfig {
        self.service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.__internal_state().dynamic_storage.get(),
            self.service.__internal_state().shared_node.config(),
            callback,
        )
    }
}

impl<Service: service::Service> PortFactory<Service> {
    pub(crate) fn new(service: Service) -> Self {
        Self { service }
    }
}
