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

use iceoryx2::{prelude::EventId, service::Service};

use crate::types::event::NotifyFn;

pub trait EventRelay<S: Service> {
    type SendError: Debug;
    type ReceiveError: Debug;

    fn send(&self, event_id: EventId) -> Result<(), Self::SendError>;
    fn receive(&self, notify: &mut NotifyFn<'_>) -> Result<(), Self::ReceiveError>;
}
