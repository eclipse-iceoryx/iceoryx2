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
/// [`PublishSubscribeRelay`] enables bi-directional transmission of [`Sample`]s
/// between local iceoryx2 [`Service`]s and remote [`Service`]s via the
/// [`Backend`](crate::traits::Backend) communication mechanism.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 [`Service`] type
///
/// # Memory Management
///
/// Received [`Sample`]s are ingested into iceoryx2 shared memory using a loan
/// function, which allocates memory from the local shared
/// memory pool. This enables efficient zero-copy delivery to local
/// participants.
///
/// # Examples
///
/// Sending a [`Sample`] over the [`Backend`](crate::traits::Backend):
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
/// Receiving remote [`Sample`]s into loaned memory from the [`Backend`](crate::traits::Backend):
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
/// Implementing a custom [`PublishSubscribeRelay`]:
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

    /// Sends a [`Sample`] via the backend communication mechanism.
    ///
    /// Transmits the [`Sample`]'s payload and header to remote endpoints. The
    /// [`Sample`] is consumed by this operation.
    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError>;

    /// Attempts to receive a [`Sample`] via the backend communication mechanism.
    ///
    /// Checks for incoming [`Sample`]s without blocking. If a [`Sample`] is available,
    /// it allocates shared memory via the provided loan function and
    /// deserializes the [`Sample`] data into that memory.
    ///
    /// The loan function must allocate enough memory to hold the incoming
    /// [`Sample`]'s payload. The relay should initialize this memory with the
    /// received data.
    ///
    /// # Parameters
    ///
    /// * `loan` - Function to allocate shared memory of the requested size.
    ///   Returns a [`SampleMutUninit`](iceoryx2::sample_mut_uninit::SampleMutUninit)
    ///   for the relay to initialize.
    ///
    /// # Type Parameters
    ///
    /// * `LoanError` - Error type that the loan function may return if memory
    ///   allocation fails. This error will be wrapped in `Self::ReceiveError`
    ///   if the loan fails.
    ///
    /// # Returns
    ///
    /// * [`SampleMut`] - A [`Sample`] was successfully received and initialized
    /// * [`None`] when no [`Sample`]s to be received
    ///
    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError>;
}
