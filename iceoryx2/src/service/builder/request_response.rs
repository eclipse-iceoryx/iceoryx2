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

use std::fmt::Debug;
use std::marker::PhantomData;

use iceoryx2_bb_elementary::alignment::Alignment;
use iceoryx2_bb_log::{fail, fatal_panic, warn};

use crate::prelude::{AttributeSpecifier, AttributeVerifier};
use crate::service::builder;
use crate::service::{self, static_config};

use super::ServiceState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestResponseOpenError {
    IncompatibleAttributes,
    IncompatibleMessagingPattern,
    IncompatibleOverflowBehaviorForRequests,
    IncompatibleOverflowBehaviorForResponses,
    DoesNotSupportRequestedAmountOfActiveRequests,
    DoesNotSupportRequestedAmountOfBorrowedResponses,
    DoesNotSupportRequestedResponseBufferSize,
    DoesNotSupportRequestedAmountOfServers,
    DoesNotSupportRequestedAmountOfClients,
    DoesNotSupportRequestedAmountOfNodes,
}

impl core::fmt::Display for RequestResponseOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseOpenError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseOpenError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestResponseCreateError {}

impl core::fmt::Display for RequestResponseCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseCreateError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseCreateError {}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RequestResponseOpenOrCreateError {
    RequestResponseOpenError(RequestResponseOpenError),
    RequestResponseCreateError(RequestResponseCreateError),
}

impl From<RequestResponseOpenError> for RequestResponseOpenOrCreateError {
    fn from(value: RequestResponseOpenError) -> Self {
        RequestResponseOpenOrCreateError::RequestResponseOpenError(value)
    }
}

impl From<RequestResponseCreateError> for RequestResponseOpenOrCreateError {
    fn from(value: RequestResponseCreateError) -> Self {
        RequestResponseOpenOrCreateError::RequestResponseCreateError(value)
    }
}

impl core::fmt::Display for RequestResponseOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseOpenOrCreateError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseOpenOrCreateError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum ServiceAvailabilityState {
    ServiceState(ServiceState),
    IncompatibleRequestType,
    IncompatibleResponseType,
}

#[derive(Debug)]
pub struct BuilderRequest<RequestPayload: Debug, ServiceType: service::Service> {
    base: builder::BuilderWithServiceType<ServiceType>,
    _request_payload: PhantomData<RequestPayload>,
}

impl<RequestPayload: Debug, ServiceType: service::Service>
    BuilderRequest<RequestPayload, ServiceType>
{
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            base,
            _request_payload: PhantomData,
        }
    }

    pub fn response<ResponsePayload: Debug>(
        self,
    ) -> Builder<RequestPayload, (), ResponsePayload, (), ServiceType> {
        Builder::new(self.base)
    }
}

#[derive(Debug)]
pub struct Builder<
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
    ServiceType: service::Service,
> {
    base: builder::BuilderWithServiceType<ServiceType>,
    override_request_alignment: Option<usize>,
    override_response_alignment: Option<usize>,
    verify_enable_safe_overflow_for_requests: bool,
    verify_enable_safe_overflow_for_responses: bool,
    verify_max_active_requests: bool,
    verify_max_borrowed_responses: bool,
    verify_max_response_buffer_size: bool,
    verify_max_servers: bool,
    verify_max_clients: bool,
    verify_max_nodes: bool,

    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

impl<
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
        ServiceType: service::Service,
    > Builder<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, ServiceType>
{
    fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            base,
            override_request_alignment: None,
            override_response_alignment: None,
            verify_enable_safe_overflow_for_requests: false,
            verify_enable_safe_overflow_for_responses: false,
            verify_max_active_requests: false,
            verify_max_borrowed_responses: false,
            verify_max_response_buffer_size: false,
            verify_max_servers: false,
            verify_max_clients: false,
            verify_max_nodes: false,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    fn config_details_mut(&mut self) -> &mut static_config::request_response::StaticConfig {
        match self.base.service_config.messaging_pattern {
            static_config::messaging_pattern::MessagingPattern::RequestResponse(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in RequestResponse builder!");
            }
        }
    }

    fn config_details(&self) -> &static_config::request_response::StaticConfig {
        match self.base.service_config.messaging_pattern {
            static_config::messaging_pattern::MessagingPattern::RequestResponse(ref v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in RequestResponse builder!");
            }
        }
    }

    pub fn request_header<M: Debug>(
        self,
    ) -> Builder<RequestPayload, M, ResponsePayload, ResponseHeader, ServiceType> {
        unsafe {
            core::mem::transmute::<
                Self,
                Builder<RequestPayload, M, ResponsePayload, ResponseHeader, ServiceType>,
            >(self)
        }
    }

    pub fn response_header<M: Debug>(
        self,
    ) -> Builder<RequestPayload, RequestHeader, ResponsePayload, M, ServiceType> {
        unsafe {
            core::mem::transmute::<
                Self,
                Builder<RequestPayload, RequestHeader, ResponsePayload, M, ServiceType>,
            >(self)
        }
    }

    pub fn request_payload_alignment(mut self, alignment: Alignment) -> Self {
        self.override_request_alignment = Some(alignment.value());
        self
    }

    pub fn response_payload_alignment(mut self, alignment: Alignment) -> Self {
        self.override_response_alignment = Some(alignment.value());
        self
    }

    pub fn enable_safe_overflow_for_requests(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow_for_requests = value;
        self.verify_enable_safe_overflow_for_requests = true;
        self
    }

    pub fn enable_safe_overflow_for_responses(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow_for_responses = value;
        self.verify_enable_safe_overflow_for_responses = true;
        self
    }

    pub fn max_active_requests(mut self, value: usize) -> Self {
        self.config_details_mut().max_active_requests = value;
        self.verify_max_active_requests = true;
        self
    }

    pub fn max_borrowed_responses(mut self, value: usize) -> Self {
        self.config_details_mut().max_borrowed_responses = value;
        self.verify_max_borrowed_responses = true;
        self
    }

    pub fn max_response_buffer_size(mut self, value: usize) -> Self {
        self.config_details_mut().max_response_buffer_size = value;
        self.verify_max_response_buffer_size = true;
        self
    }

    pub fn max_servers(mut self, value: usize) -> Self {
        self.config_details_mut().max_servers = value;
        self.verify_max_servers = true;
        self
    }

    pub fn max_clients(mut self, value: usize) -> Self {
        self.config_details_mut().max_clients = value;
        self.verify_max_clients = true;
        self
    }

    pub fn max_nodes(mut self, value: usize) -> Self {
        self.config_details_mut().max_nodes = value;
        self.verify_max_nodes = true;
        self
    }

    fn adjust_configuration_to_meaningful_values(&mut self) {
        let origin = format!("{:?}", self);
        let settings = self.base.service_config.request_response_mut();

        if settings.max_active_requests == 0 {
            warn!(from origin,
                "Setting the maximum number of active requests to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_active_requests = 1;
        }

        if settings.max_borrowed_responses == 0 {
            warn!(from origin,
                "Setting the maximum number of borrowed responses to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_borrowed_responses = 1;
        }

        if settings.max_servers == 0 {
            warn!(from origin,
                "Setting the maximum number of servers to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_servers = 1;
        }

        if settings.max_clients == 0 {
            warn!(from origin,
                "Setting the maximum number of clients to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_clients = 1;
        }

        if settings.max_nodes == 0 {
            warn!(from origin,
                "Setting the maximum number of nodes to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_nodes = 1;
        }
    }

    fn verify_service_configuration(
        &self,
        existing_settings: &static_config::StaticConfig,
        required_attributes: &AttributeVerifier,
    ) -> Result<static_config::request_response::StaticConfig, RequestResponseOpenError> {
        let msg = "Unable to open request response service";

        let existing_attributes = existing_settings.attributes();
        if let Err(incompatible_key) = required_attributes.verify_requirements(existing_attributes)
        {
            fail!(from self, with RequestResponseOpenError::IncompatibleAttributes,
                "{} due to incompatible service attribute key \"{}\". The following attributes {:?} are required but the service has the attributes {:?}.",
                msg, incompatible_key, required_attributes, existing_attributes);
        }

        let required_configuration = self.base.service_config.request_response();
        let existing_configuration = match &existing_settings.messaging_pattern {
            static_config::messaging_pattern::MessagingPattern::RequestResponse(ref v) => v,
            p => {
                fail!(from self, with RequestResponseOpenError::IncompatibleMessagingPattern,
                    "{} since a service with the messaging pattern {:?} exists but MessagingPattern::RequestResponse is required.",
                    msg, p);
            }
        };

        if self.verify_enable_safe_overflow_for_requests
            && existing_configuration.enable_safe_overflow_for_requests
                != required_configuration.enable_safe_overflow_for_requests
        {
            fail!(from self, with RequestResponseOpenError::IncompatibleOverflowBehaviorForRequests,
                "{} since the service has an incompatible safe overflow behavior for requests.",
                msg);
        }

        if self.verify_enable_safe_overflow_for_responses
            && existing_configuration.enable_safe_overflow_for_responses
                != required_configuration.enable_safe_overflow_for_responses
        {
            fail!(from self, with RequestResponseOpenError::IncompatibleOverflowBehaviorForResponses,
                "{} since the service has an incompatible safe overflow behavior for responses.",
                msg);
        }

        if self.verify_max_active_requests
            && existing_configuration.max_active_requests
                < required_configuration.max_active_requests
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedAmountOfActiveRequests,
                "{} since the service supports only {} active requests but {} are required.",
                msg, existing_configuration.max_active_requests, required_configuration.max_active_requests);
        }

        if self.verify_max_borrowed_responses
            && existing_configuration.max_borrowed_responses
                < required_configuration.max_borrowed_responses
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedAmountOfBorrowedResponses,
                "{} since the service supports only {} borrowed responses but {} are required.",
                msg, existing_configuration.max_borrowed_responses, required_configuration.max_borrowed_responses);
        }

        if self.verify_max_response_buffer_size
            && existing_configuration.max_response_buffer_size
                < required_configuration.max_response_buffer_size
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedResponseBufferSize,
                "{} since the service supports a maximum response buffer size of {} but a size of {} is required.",
                msg, existing_configuration.max_response_buffer_size, required_configuration.max_response_buffer_size);
        }

        if self.verify_max_servers
            && existing_configuration.max_servers < required_configuration.max_servers
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers,
                "{} since the service supports at most {} servers but {} are required.",
                msg, existing_configuration.max_servers, required_configuration.max_servers);
        }

        if self.verify_max_clients
            && existing_configuration.max_clients < required_configuration.max_clients
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients,
                "{} since the service supports at most {} clients but {} are required.",
                msg, existing_configuration.max_clients, required_configuration.max_clients);
        }

        if self.verify_max_nodes
            && existing_configuration.max_nodes < required_configuration.max_nodes
        {
            fail!(from self, with RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients,
                "{} since the service supports at most {} nodes but {} are required.",
                msg, existing_configuration.max_nodes, required_configuration.max_nodes);
        }

        Ok(existing_configuration.clone())
    }

    fn is_service_available(
        &mut self,
        error_msg: &str,
    ) -> Result<
        Option<(static_config::StaticConfig, ServiceType::StaticStorage)>,
        ServiceAvailabilityState,
    > {
        match self.base.is_service_available(error_msg) {
            Ok(Some((config, storage))) => {
                if !self
                    .config_details()
                    .request_message_type_details
                    .is_compatible_to(&config.request_response().request_message_type_details)
                {
                    fail!(from self, with ServiceAvailabilityState::IncompatibleRequestType,
                        "{} since the services uses the request type \"{:?}\" which is not compatible to the requested type \"{:?}\".",
                        error_msg, &config.request_response().request_message_type_details,
                        self.config_details().request_message_type_details);
                }

                if !self
                    .config_details()
                    .response_message_type_details
                    .is_compatible_to(&config.request_response().response_message_type_details)
                {
                    fail!(from self, with ServiceAvailabilityState::IncompatibleResponseType,
                        "{} since the services uses the response type \"{:?}\" which is not compatible to the requested type \"{:?}\".",
                        error_msg, &config.request_response().response_message_type_details,
                        self.config_details().response_message_type_details);
                }

                Ok(Some((config, storage)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ServiceAvailabilityState::ServiceState(e)),
        }
    }
}
