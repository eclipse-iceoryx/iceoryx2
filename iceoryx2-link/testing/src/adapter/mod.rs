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

mod endpoint_description;
mod mapping;
mod middleware;
mod publish_subscribe_endpoints;
mod request_response_endpoints;
mod translator;

pub use endpoint_description::{
    FakeEndpointDescription, FakeEndpointSettings, FakeSampleTypes, endpoint_of,
};
pub use mapping::FakeMapping;
pub use middleware::{
    FakeMiddleware, FakeRequestId, FakeRequestMessage, FakeResponseMessage, Message, MessageRef,
};
pub use publish_subscribe_endpoints::FakePublishSubscribeEndpoints;
pub use request_response_endpoints::FakeRequestResponseEndpoints;
pub use translator::{FakeSwapTranslator, SwapHeader, SwapPayload, swapped_bytes};

use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_adapter::{Adapter, EndpointTypes, UnsupportedEndpoints};
use iceoryx2_link_backend::{Reactive, WakeHandle};
use iceoryx2_log::{fail, origin};

use crate::adapter::middleware::Origin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The middleware has no endpoints of the pattern.
    UnsupportedPattern,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error::{self:?}")
    }
}

impl core::error::Error for Error {}

/// An adapter over a [`FakeMiddleware`].
pub struct FakeAdapter {
    middleware: FakeMiddleware,
}

impl FakeAdapter {
    pub(super) fn new(middleware: FakeMiddleware) -> Self {
        Self { middleware }
    }
}

impl Adapter for FakeAdapter {
    type ListError = Error;
    type OpenError = Error;
    type EndpointSettings = FakeEndpointSettings;
    type EndpointTypes = EndpointTypes<FakeSampleTypes>;
    type PublishSubscribeEndpoints = FakePublishSubscribeEndpoints;
    type RequestResponseEndpoints = FakeRequestResponseEndpoints;
    type EventEndpoints = UnsupportedEndpoints;

    fn generation(&self) -> Generation {
        Generation::At(self.middleware.state().generation)
    }

    fn endpoints(
        &self,
        callback: &mut dyn FnMut(&FakeEndpointDescription),
    ) -> Result<(), Self::ListError> {
        let state = self.middleware.state();
        for topic in state.topics.values() {
            let remote = topic
                .endpoints
                .values()
                .any(|remote| remote.origin == Origin::Remote);
            if remote {
                callback(&topic.description);
            }
        }
        Ok(())
    }

    fn publish_subscribe(
        &mut self,
        description: &FakeEndpointDescription,
    ) -> Result<Self::PublishSubscribeEndpoints, Self::OpenError> {
        let id = self.middleware.join(description, Origin::Gateway);
        Ok(FakePublishSubscribeEndpoints::new(
            self.middleware.clone(),
            description,
            id,
        ))
    }

    fn request_response(
        &mut self,
        description: &FakeEndpointDescription,
    ) -> Result<Self::RequestResponseEndpoints, Self::OpenError> {
        let id = self.middleware.join(description, Origin::Gateway);
        Ok(FakeRequestResponseEndpoints::new(
            self.middleware.clone(),
            description,
            id,
        ))
    }

    fn event(
        &mut self,
        description: &FakeEndpointDescription,
    ) -> Result<Self::EventEndpoints, Self::OpenError> {
        let origin = origin!("FakeAdapter::event");

        fail!(
            from origin,
            with Error::UnsupportedPattern,
            "The fake middleware has no events, {} cannot be opened as one", description.settings.name
        );
    }
}

impl Reactive for FakeAdapter {
    fn attach(&mut self, wake: WakeHandle) {
        self.middleware.attach(wake);
    }
}
