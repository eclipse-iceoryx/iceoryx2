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

use iceoryx2::service::Service;

use crate::types::publish_subscribe::LoanFn;
use crate::types::publish_subscribe::Sample;
use crate::types::publish_subscribe::SampleMut;

/// Relay for tunneling iceoryx2 publish-subscribe samples through a backend.
///
/// `PublishSubscribeRelay` enables bi-directional transmission of samples
/// between local iceoryx2 services and remote services via the
/// backend's communication mechanism.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 service type
///
/// # Memory Management
///
/// Received samples are ingested into iceoryx2 shared memory using a loan
/// function, which allocates memory from the local shared
/// memory pool. This enables efficient zero-copy delivery to local
/// participants.
///
/// # Examples
///
/// Sending a sample over the backend:
///
/// ```no_run
/// # use iceoryx2_tunnel_backend::traits::PublishSubscribeRelay;
/// # use iceoryx2_tunnel_backend::types::publish_subscribe::Sample;
/// # use iceoryx2::service::ipc::Service;
/// # fn example<R: PublishSubscribeRelay<Service>>(relay: &R, sample: Sample<Service>)
/// #     -> Result<(), R::SendError> {
/// relay.send(sample)?;
/// # Ok(())
/// # }
/// ```
///
/// Receiving remote samples into loaned memory from the backend:
///
/// ```no_run
/// # use iceoryx2_tunnel_backend::traits::PublishSubscribeRelay;
/// # use iceoryx2::service::ipc::Service;
/// # fn example<R: PublishSubscribeRelay<Service>, LoanError>(relay: &R)
/// #     -> Result<(), R::ReceiveError> {
/// let mut loan_fn = |size: usize| {
///     // Loan an uninitialized sample from iceoryx2 and
///     // return it to the relay to be initialized
/// #    unimplemented!()
/// };
///
/// if let Some(sample) = relay.receive::<LoanError>(&mut loan_fn)? {
///     println!("Received {} bytes", sample.payload().len());
///     // println!("Received {} bytes", sample.payload().len());
///     // Payload byte are now in shared memory, ready to be delivered
/// }
/// # Ok(())
/// # }
/// ```
///
/// Implementing a custom PublishSubscribeRelay:
///
/// ```no_run
/// use iceoryx2::service::ipc::Service;
/// use iceoryx2_tunnel_backend::traits::PublishSubscribeRelay;
/// use iceoryx2_tunnel_backend::types::publish_subscribe::{
///     Sample, SampleMut, LoanFn
/// };
///
/// struct MyPublishSubscribeRelay {
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
/// impl PublishSubscribeRelay<Service> for MyPublishSubscribeRelay {
///     type SendError = MySendError;
///     type ReceiveError = MyReceiveError;
///
///     fn send(&self, sample: Sample<Service>) -> Result<(), Self::SendError> {
///         // Serialize and transmit sample payload over the backend
///         Ok(())
///     }
///
///     fn receive<LoanError>(
///         &self,
///         loan: &mut LoanFn<'_, Service, LoanError>,
///     ) -> Result<Option<SampleMut<Service>>, Self::ReceiveError> {
///         // Check for incoming samples
///         // If available:
///         //   1. Determine required size
///         //   2. Call loan(size) to allocate shared memory
///         //   3. Deserialize into the loaned memory
///         //   4. Return Some(initialized_sample), or None if none available
///         Ok(None)
///     }
/// }
/// ```
pub trait PublishSubscribeRelay<S: Service> {
    /// Error type returned when sending fails.
    type SendError: Error;

    /// Error type returned when receiving fails.
    type ReceiveError: Error;

    /// Sends a sample through the backend.
    ///
    /// Transmits the sample's payload and header to remote endpoints. The
    /// sample is consumed by this operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend connection is unavailable,
    /// serialization fails, the transmission buffer is full, or network
    /// communication fails.
    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError>;

    /// Attempts to receive a sample from the backend.
    ///
    /// Checks for incoming samples without blocking. If a sample is available,
    /// it allocates shared memory via the provided loan function and
    /// deserializes the sample data into that memory.
    ///
    /// The loan function must allocate enough memory to hold the incoming
    /// sample's payload. The relay should initialize this memory with the
    /// received data.
    ///
    /// # Parameters
    ///
    /// * `loan` - Function to allocate shared memory of the requested size.
    ///   Returns `SampleMutUninit` for the relay to initialize.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SampleMut))` - A sample was received and ingested
    /// * `Ok(None)` - No samples are currently available
    ///
    /// # Errors
    ///
    /// Returns an error if the backend connection fails, deserialization
    /// fails, the loan function returns an error, or the allocated memory is
    /// too small for the incoming sample.
    ///
    /// Returns an error that should describe the failure reason, for example,
    /// the backend connection is unavailable, network transmission fails,
    /// deserialization fails, the loan function returns an error, the
    /// allocated memory is too small, etc.
    ///
    /// # Examples
    ///
    /// Receiving with error handling:
    ///
    /// ```no_run
    /// # use iceoryx2_tunnel_backend::traits::PublishSubscribeRelay;
    /// # use iceoryx2::service::ipc::Service;
    /// # fn example<R: PublishSubscribeRelay<Service>, LoanError>(relay: &R)
    /// #     -> Result<(), R::ReceiveError> {
    /// let mut loan_fn = |size: usize| {
    ///     // Loan an uninitialized sample from iceoryx2 and
    ///     // return it to the relay to be initialized
    /// #   unimplemented!()
    /// };
    ///
    /// match relay.receive::<LoanError>(&mut loan_fn) {
    ///     Ok(Some(sample)) => {
    ///         // Deliver the initialized sample
    ///         println!("Received: {:?}", sample.payload());
    ///     }
    ///     Ok(None) => {
    ///         // No data available
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Receive error: {}", e);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError>;
}
