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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let event = node.service_builder(&"MyEventName".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
//! let mut notifier = event.notifier_builder()
//!     .default_event_id(EventId::new(12))
//!     .create()?;
//!
//! // notify the listener with default event id 12
//! notifier.notify()?;
//!
//! notifier.notify_with_custom_event_id(EventId::new(5));
//!
//! while let Some(event_id) = listener.try_wait_one()? {
//!     println!("event was triggered with id: {:?}", event_id);
//! }
//!
//! # Ok(())
//! # }
//! ```

/// Type that allows to identify an event uniquely.
pub type EventId = iceoryx2_cal::event::TriggerId;
