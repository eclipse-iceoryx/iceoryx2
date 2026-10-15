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

use iceoryx2::port::event_id::EventId;
use iceoryx2_link_backend::Never;

use iceoryx2_link_backend::wire::request_response::ClientRequestId;

use super::{
    EventEndpoints, PublishSubscribeEndpoints, RequestDestination, RequestResponseEndpoints,
    ResponseDestination, ResponseKind, SampleDestination, TakeOutcome,
};
use crate::SampleBytesRef;

/// The endpoints of a pattern the middleware does not have.
pub struct UnsupportedEndpoints(Never);

impl PublishSubscribeEndpoints for UnsupportedEndpoints {
    type Failure = core::convert::Infallible;

    fn publish(&mut self, _: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
        match self.0 {}
    }

    fn take<'a>(
        &mut self,
        _: impl SampleDestination<'a>,
    ) -> Result<TakeOutcome<()>, Self::Failure> {
        match self.0 {}
    }
}

impl RequestResponseEndpoints for UnsupportedEndpoints {
    type Failure = core::convert::Infallible;
    type ServerRequestId = Never;

    fn send_request(
        &mut self,
        _: ClientRequestId,
        _: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure> {
        match self.0 {}
    }

    fn take_response<'a>(
        &mut self,
        _: impl ResponseDestination<'a>,
    ) -> Result<TakeOutcome<(ClientRequestId, ResponseKind)>, Self::Failure> {
        match self.0 {}
    }

    fn take_request<'a>(
        &mut self,
        _: impl RequestDestination<'a>,
    ) -> Result<TakeOutcome<Never>, Self::Failure> {
        match self.0 {}
    }

    fn send_response(&mut self, _: &Never, _: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
        match self.0 {}
    }

    fn end_response_stream(&mut self, _: Never) -> Result<(), Self::Failure> {
        match self.0 {}
    }
}

impl EventEndpoints for UnsupportedEndpoints {
    type Failure = core::convert::Infallible;

    fn notify(&mut self, _: EventId) -> Result<(), Self::Failure> {
        match self.0 {}
    }

    fn take(&mut self) -> Result<Option<EventId>, Self::Failure> {
        match self.0 {}
    }
}
