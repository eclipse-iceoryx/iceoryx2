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

use core::error::Error;

use iceoryx2::{prelude::EventId, service::Service};

/// Relay for tunneling iceoryx2 events through a [`Backend`](crate::traits::Backend).
///
/// [`EventRelay`] provides bidirectional transmission of event notifications
/// between local iceoryx2 [`Service`]s and remote [`Service`]s via the
/// [`Backend`](crate::traits::Backend) communication mechanism.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 [`Service`] type
///
/// # Examples
///
/// Sending local event over the [`Backend`](crate::traits::Backend):
///
/// ```no_run
/// # use iceoryx2::prelude::EventId;
/// # use iceoryx2_tunnel_backend::traits::EventRelay;
/// # use iceoryx2::service::ipc::Service;
/// # fn example<R: EventRelay<Service>>(relay: &R) -> Result<(), R::SendError> {
/// let event_id = EventId::new(42);
/// relay.send(event_id)?;
/// # Ok(())
/// # }
/// ```
///
/// Receiving remote events from the [`Backend`](crate::traits::Backend):
///
/// ```no_run
/// # use iceoryx2_tunnel_backend::traits::EventRelay;
/// # use iceoryx2::service::ipc::Service;
/// # fn example<R: EventRelay<Service>>(relay: &R) -> Result<(), R::ReceiveError> {
/// loop {
///     match relay.receive()? {
///         Some(event_id) => {
///             println!("Received event: {:?}", event_id);
///             // Handle event
///         }
///         None => {
///             // No events available
///             break;
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// Implementing an [`EventRelay`]:
///
/// ```no_run
/// use iceoryx2::prelude::EventId;
/// use iceoryx2::service::ipc::Service;
/// use iceoryx2_tunnel_backend::traits::EventRelay;
///
/// struct MyEventRelay {
///     // Backend state
/// }
///
/// #[derive(Debug)]
/// struct MySendError;
/// impl core::fmt::Display for MySendError {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         write!(f, "send failed")
///     }
/// }
/// impl core::error::Error for MySendError {}
///
/// #[derive(Debug)]
/// struct MyReceiveError;
/// impl core::fmt::Display for MyReceiveError {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         write!(f, "receive failed")
///     }
/// }
/// impl core::error::Error for MyReceiveError {}
///
/// impl EventRelay<Service> for MyEventRelay {
///     type SendError = MySendError;
///     type ReceiveError = MyReceiveError;
///
///     fn send(&self, event_id: EventId) -> Result<(), Self::SendError> {
///         // Serialize and transmit event_id via backend
///         Ok(())
///     }
///
///     fn receive(&self) -> Result<Option<EventId>, Self::ReceiveError> {
///         // Check backend for incoming events
///         // Return Some(event_id) if available, None otherwise
///         Ok(None)
///     }
/// }
/// ```
pub trait EventRelay<S: Service> {
    /// Error type returned when sending an event fails.
    type SendError: Error;

    /// Error type returned when receiving an event fails.
    type ReceiveError: Error;

    /// Sends an event notification through the backend.
    ///
    /// Transmits the specified [`EventId`] to remote endpoints via the
    /// [`Backend`](crate::traits::Backend) communication mechanism.
    /// The send operation should be non-blocking.
    fn send(&self, event_id: EventId) -> Result<(), Self::SendError>;

    /// Attempts to receive an event notification from the backend.
    ///
    /// Checks for incoming event notifications without blocking. Returns
    /// [`EventId`] if an event is available, or [`None`] if no events
    /// are pending.
    ///
    /// # Returns
    ///
    /// * [`EventId`] received via the backend communication mechanism
    /// * [`None`] when no [`EventId`]s to be received
    fn receive(&self) -> Result<Option<EventId>, Self::ReceiveError>;
}
