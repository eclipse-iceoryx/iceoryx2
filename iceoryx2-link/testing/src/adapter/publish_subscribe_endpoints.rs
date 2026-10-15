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

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::{
    EndpointTypes, PublishSubscribeEndpoints, SampleBytesRef, SampleDestination, SampleLengths,
    TakeOutcome,
};

use crate::adapter::middleware::{EndpointId, FakeMiddleware, Message, MessageRef};
use crate::adapter::{Error, FakeEndpointDescription};

/// A publisher and subscription on a [`FakeMiddleware`], the
/// gateway's or an application's.
pub struct FakePublishSubscribeEndpoints {
    middleware: FakeMiddleware,
    name: ServiceName,
    id: EndpointId,
    header_size: usize,
}

impl FakePublishSubscribeEndpoints {
    pub(super) fn new(
        middleware: FakeMiddleware,
        description: &FakeEndpointDescription,
        id: EndpointId,
    ) -> Self {
        let header_size = match &description.types {
            EndpointTypes::PublishSubscribe(types) => types.user_header.size,
            EndpointTypes::Event => 0,
            EndpointTypes::RequestResponse { .. } => {
                panic!(
                    "the endpoints of {} are not publish-subscribe",
                    description.settings.name
                )
            }
        };
        Self {
            middleware,
            name: description.settings.name,
            id,
            header_size,
        }
    }
}

impl FakePublishSubscribeEndpoints {
    /// Sends a message to every other endpoint with the description.
    pub fn send(&self, message: MessageRef<'_>) {
        self.middleware.send_sample(&self.name, self.id, message);
    }

    /// The next pending message, if any.
    pub fn receive(&self) -> Option<Message> {
        self.middleware.receive_sample(&self.name, self.id)
    }
}

impl PublishSubscribeEndpoints for FakePublishSubscribeEndpoints {
    type Failure = Error;

    fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
        self.middleware.send_sample(
            &self.name,
            self.id,
            &[sample.header, sample.payload].concat(),
        );

        Ok(())
    }

    fn take<'a>(
        &mut self,
        destination: impl SampleDestination<'a>,
    ) -> Result<TakeOutcome<()>, Self::Failure> {
        let Some(message) = self.middleware.receive_sample(&self.name, self.id) else {
            return Ok(TakeOutcome::Empty);
        };
        let (header, payload) = message.split_at(self.header_size.min(message.len()));
        let Some(locations) = destination.for_lengths(SampleLengths {
            header: header.len(),
            payload: payload.len(),
        }) else {
            return Ok(TakeOutcome::Declined);
        };
        locations.header.copy_from_slice(header);
        locations.payload.copy_from_slice(payload);

        Ok(TakeOutcome::Taken(()))
    }
}

impl Drop for FakePublishSubscribeEndpoints {
    fn drop(&mut self) {
        self.middleware.leave(&self.name, self.id);
    }
}
