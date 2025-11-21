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

use iceoryx2::service::builder::{CustomHeaderMarker, CustomKeyMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::service_builder_blackboard::{
    ServiceBuilderBlackboardCreatorType, ServiceBuilderBlackboardOpenerType,
};
use crate::service_builder_request_response::ServiceBuilderRequestResponseType;
use crate::{
    service_builder_blackboard::{ServiceBuilderBlackboardCreator, ServiceBuilderBlackboardOpener},
    service_builder_event::{ServiceBuilderEvent, ServiceBuilderEventType},
    service_builder_publish_subscribe::{
        ServiceBuilderPublishSubscribe, ServiceBuilderPublishSubscribeType,
    },
    service_builder_request_response::ServiceBuilderRequestResponse,
};

pub(crate) enum ServiceBuilderType {
    Ipc(iceoryx2::service::builder::Builder<crate::IpcService>),
    Local(iceoryx2::service::builder::Builder<crate::LocalService>),
}

#[pyclass]
/// Builder to create or open `Service`s
pub struct ServiceBuilder(pub(crate) ServiceBuilderType);

#[pymethods]
impl ServiceBuilder {
    /// Create a new builder to create a `MessagingPattern::Event` `Service`.
    pub fn event(&self) -> ServiceBuilderEvent {
        match &self.0 {
            ServiceBuilderType::Ipc(v) => {
                let this = v.clone();
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this.event()))
            }
            ServiceBuilderType::Local(v) => {
                let this = v.clone();
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this.event()))
            }
        }
    }

    /// Create a new builder to create a `MessagingPattern::PublishSubscribe` `Service`.
    pub fn __publish_subscribe(&self) -> ServiceBuilderPublishSubscribe {
        match &self.0 {
            ServiceBuilderType::Ipc(v) => {
                let this = v.clone();
                ServiceBuilderPublishSubscribe::new(ServiceBuilderPublishSubscribeType::Ipc(
                    this.publish_subscribe::<[CustomPayloadMarker]>()
                        .user_header::<CustomHeaderMarker>(),
                ))
            }
            ServiceBuilderType::Local(v) => {
                let this = v.clone();
                ServiceBuilderPublishSubscribe::new(ServiceBuilderPublishSubscribeType::Local(
                    this.publish_subscribe::<[CustomPayloadMarker]>()
                        .user_header::<CustomHeaderMarker>(),
                ))
            }
        }
    }

    /// Create a new builder to create a `MessagingPattern::RequestResponse` `Service`.
    pub fn __request_response(&self) -> ServiceBuilderRequestResponse {
        match &self.0 {
            ServiceBuilderType::Ipc(v) => {
                let this = v.clone();
                ServiceBuilderRequestResponse::new(ServiceBuilderRequestResponseType::Ipc(
                    this.request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                        .request_user_header::<CustomHeaderMarker>()
                        .response_user_header::<CustomHeaderMarker>(),
                ))
            }
            ServiceBuilderType::Local(v) => {
                let this = v.clone();
                ServiceBuilderRequestResponse::new(ServiceBuilderRequestResponseType::Local(
                    this.request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                        .request_user_header::<CustomHeaderMarker>()
                        .response_user_header::<CustomHeaderMarker>(),
                ))
            }
        }
    }

    /// Create a new builder to create a `MessagingPattern::Blackboard` `Service`.
    pub fn __blackboard_creator(&self) -> ServiceBuilderBlackboardCreator {
        match &self.0 {
            ServiceBuilderType::Ipc(v) => {
                let this = v.clone();
                ServiceBuilderBlackboardCreator::new(ServiceBuilderBlackboardCreatorType::Ipc(
                    Some(this.blackboard_creator::<CustomKeyMarker>()),
                ))
            }
            ServiceBuilderType::Local(v) => {
                let this = v.clone();
                ServiceBuilderBlackboardCreator::new(ServiceBuilderBlackboardCreatorType::Local(
                    Some(this.blackboard_creator::<CustomKeyMarker>()),
                ))
            }
        }
    }

    /// Create a new builder to open a `MessagingPattern::Blackboard` `Service`.
    pub fn __blackboard_opener(&self) -> ServiceBuilderBlackboardOpener {
        match &self.0 {
            ServiceBuilderType::Ipc(v) => {
                let this = v.clone();
                ServiceBuilderBlackboardOpener::new(ServiceBuilderBlackboardOpenerType::Ipc(Some(
                    this.blackboard_opener::<CustomKeyMarker>(),
                )))
            }
            ServiceBuilderType::Local(v) => {
                let this = v.clone();
                ServiceBuilderBlackboardOpener::new(ServiceBuilderBlackboardOpenerType::Local(
                    Some(this.blackboard_opener::<CustomKeyMarker>()),
                ))
            }
        }
    }
}
