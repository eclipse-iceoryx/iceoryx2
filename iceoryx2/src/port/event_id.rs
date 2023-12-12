// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let event_name = ServiceName::new("MyEventName")?;
//! # let event = zero_copy::Service::new(&event_name)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listener = event.listener().create()?;
//! let mut notifier = event.notifier()
//!     .default_event_id(EventId::new(123))
//!     .create()?;
//!
//! // notify the listener with default event id 123
//! notifier.notify()?;
//!
//! notifier.notify_with_custom_event_id(EventId::new(456));
//!
//! for event_id in listener.try_wait()? {
//!     println!("event was triggered with id: {:?}", event_id);
//! }
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_cal::event::TriggerId;

/// Id to identify the source in event based communication.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventId(u64);

impl EventId {
    /// Creates a new [`EventId`] from a given integer value.
    pub fn new(value: u64) -> Self {
        EventId(value)
    }

    /// Returns the underlying integer value of the [`EventId`].
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new(0)
    }
}

impl TriggerId for EventId {}
