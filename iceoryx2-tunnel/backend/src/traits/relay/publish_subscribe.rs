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

use core::fmt::Debug;

use iceoryx2::service::Service;

use crate::types::publish_subscribe::LoanFn;
use crate::types::publish_subscribe::Sample;
use crate::types::publish_subscribe::SampleMut;

/// Relay for exchanging publish-subscribe samples between iceoryx and the
/// backend.
pub trait PublishSubscribeRelay<S: Service> {
    /// Error type returned when sending fails
    type SendError: Debug;
    /// Error type returned when receiving fails
    type ReceiveError: Debug;

    /// Send a sample via the backend communication mechanism.
    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError>;

    /// Receive a sample via the backend communication mechanism and ingest it
    /// into a memory loan obtained from the provided loan function.
    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError>;
}
