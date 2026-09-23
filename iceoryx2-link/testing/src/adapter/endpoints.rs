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
use alloc::vec::Vec;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::{
    PublishSubscribeEndpoints, SampleBytesRef, SampleLengths, TakeDestination, TakeOutcome,
};

use crate::adapter::Error;
use crate::adapter::middleware::FakeMiddleware;

/// A publisher and subscription on a [`FakeMiddleware`], the
/// gateway's or an application's.
pub struct FakeEndpoints {
    middleware: FakeMiddleware,
    name: ServiceName,
    id: u64,
    header_size: usize,
}

impl FakeEndpoints {
    pub(super) fn new(
        middleware: FakeMiddleware,
        name: ServiceName,
        id: u64,
        header_size: usize,
    ) -> Self {
        Self {
            middleware,
            name,
            id,
            header_size,
        }
    }
}

impl FakeEndpoints {
    /// Sends a message to every other endpoint under the description.
    pub fn send(&self, message: &[u8]) {
        self.middleware.publish(&self.name, self.id, message);
    }

    /// The next pending message, if any.
    pub fn receive(&self) -> Option<Vec<u8>> {
        self.middleware.receive(&self.name, self.id)
    }
}

impl PublishSubscribeEndpoints for FakeEndpoints {
    type Failure = Error;

    fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
        self.middleware.publish(
            &self.name,
            self.id,
            &[sample.header, sample.payload].concat(),
        );

        Ok(())
    }

    fn take<'a>(
        &mut self,
        destination: impl TakeDestination<'a>,
    ) -> Result<TakeOutcome, Self::Failure> {
        let Some(message) = self.middleware.receive(&self.name, self.id) else {
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

        Ok(TakeOutcome::Taken)
    }
}

impl Drop for FakeEndpoints {
    fn drop(&mut self) {
        self.middleware.leave(&self.name, self.id);
    }
}
