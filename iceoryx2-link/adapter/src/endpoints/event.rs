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

use core::error::Error;

use iceoryx2::port::event_id::EventId;

/// The gateway's notifier and listener on the middleware.
pub trait EventEndpoints {
    type Failure: Error;

    /// Notifies every other endpoint under the same description.
    fn notify(&mut self, id: EventId) -> Result<(), Self::Failure>;

    /// The pending notification, if any.
    fn take(&mut self) -> Result<Option<EventId>, Self::Failure>;
}
