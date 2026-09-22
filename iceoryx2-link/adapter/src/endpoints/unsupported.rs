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
use iceoryx2_link_backend::relay::ReceiveOutcome;

use super::{EventEndpoints, PublishSubscribeEndpoints, TakeError};
use crate::LoanableSample;

/// The endpoints of a pattern the middleware does not have.
pub struct UnsupportedEndpoints(Never);

impl PublishSubscribeEndpoints for UnsupportedEndpoints {
    type Failure = core::convert::Infallible;

    fn publish(&mut self, _: &[u8], _: &[u8]) -> Result<(), Self::Failure> {
        match self.0 {}
    }

    fn take<L: LoanableSample>(
        &mut self,
        _: L,
    ) -> Result<ReceiveOutcome<L::Sample>, TakeError<Self::Failure>> {
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
