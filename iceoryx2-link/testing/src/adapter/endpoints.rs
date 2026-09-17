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
use iceoryx2_link_adapter::{Destination, PublishSubscribeEndpoints, ResizeError, TakeError};
use iceoryx2_log::{fail, origin};

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

    fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Failure> {
        // The middleware's wire form, the header followed by the payload.
        self.middleware
            .publish(&self.name, self.id, &[header, payload].concat());
        Ok(())
    }

    fn take<D: Destination>(&mut self, into: &mut D) -> Result<bool, TakeError<Self::Failure>> {
        let origin = origin!("FakeEndpoints::take");

        let Some(message) = self.middleware.receive(&self.name, self.id) else {
            return Ok(false);
        };
        let (header, payload) = message.split_at(self.header_size.min(message.len()));
        fail!(
            from origin,
            when self.write(into.payload(payload.len()), payload),
            "Failed to take a message on {}", self.name
        );
        fail!(
            from origin,
            when self.write(into.header(header.len()), header),
            "Failed to take a message on {}", self.name
        );
        Ok(true)
    }
}

impl FakeEndpoints {
    fn write(
        &self,
        region: Result<&mut [u8], ResizeError>,
        bytes: &[u8],
    ) -> Result<(), TakeError<Error>> {
        let origin = origin!("FakeEndpoints::take");

        match region {
            Ok(into) => {
                into.copy_from_slice(bytes);
                Ok(())
            }
            Err(refusal) => {
                fail!(
                    from origin,
                    with TakeError::Rejected(refusal),
                    "Dropped a message of {} bytes on {}", bytes.len(), self.name
                );
            }
        }
    }
}

impl Drop for FakeEndpoints {
    fn drop(&mut self) {
        self.middleware.leave(&self.name, self.id);
    }
}
