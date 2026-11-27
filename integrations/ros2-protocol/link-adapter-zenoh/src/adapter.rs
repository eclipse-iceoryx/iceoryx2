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

use core::convert::Infallible;

use iceoryx2_link_adapter::{Adapter, EndpointTypes, UnsupportedEndpoints};

use crate::endpoint_description::{EndpointDescription, EndpointSettings, MessageType};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OpenError {
    UnsupportedPattern,
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpenError::{self:?}")
    }
}

impl core::error::Error for OpenError {}

/// An [`Adapter`] to a ROS 2 graph on `rmw_zenoh_cpp`.
#[derive(Debug, Default)]
pub struct RmwZenohAdapter {}

impl RmwZenohAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Adapter for RmwZenohAdapter {
    type ListError = Infallible;
    type OpenError = OpenError;
    type EndpointSettings = EndpointSettings;
    type EndpointTypes = EndpointTypes<MessageType>;
    type PublishSubscribeEndpoints = UnsupportedEndpoints;
    type RequestResponseEndpoints = UnsupportedEndpoints;
    type EventEndpoints = UnsupportedEndpoints;

    fn endpoints(
        &self,
        _callback: &mut dyn FnMut(&EndpointDescription),
    ) -> Result<(), Self::ListError> {
        Ok(())
    }

    fn publish_subscribe(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::PublishSubscribeEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }

    fn request_response(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::RequestResponseEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }

    fn event(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::EventEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }
}
