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

use core::fmt::Debug;

use tiny_fn::tiny_fn;
use update_connections::ConnectionFailure;

pub(crate) mod details;
pub use details::data_segment::DataSegmentType;

/// Sends requests to a [`Server`](crate::port::server::Server) and receives responses.
pub mod client;
/// Defines the event id used to identify the source of an event.
pub mod event_id;
/// Receiving endpoint (port) for event based communication
pub mod listener;
/// Sending endpoint (port) for event based communication
pub mod notifier;
/// Sending endpoint (port) for publish-subscribe based communication
pub mod publisher;
/// Reading endpoint (port) for blackboard based communication
pub mod reader;
/// Receives requests from a [`Client`](crate::port::client::Client) port and sends back responses.
pub mod server;
/// Receiving endpoint (port) for publish-subscribe based communication
pub mod subscriber;
/// Interface to perform cyclic updates to the ports. Required to deliver history to new
/// participants or to perform other management tasks.
pub mod update_connections;
/// Producing endpoint (port) for blackboard based communication
pub mod writer;

/// Defines the strategy a sender shall pursue when the buffer of a
/// receiver is full and the service does not overflow.
pub mod unable_to_deliver_strategy;

/// Defines the action that shall be take when an degradation is detected. This can happen when a
/// sample cannot be delivered, or when the system is corrupted and files are modified by
/// non-iceoryx2 instances. Is used as return value of the [`DegradationCallback`] to define a
/// custom behavior.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DegradationAction {
    /// Perform the default action
    Default,
    /// Ignore the degradation completely
    Ignore, // TODO: replace with Discard?
    /// Performs whatever is necessary to discard the degradation
    Discard,
    /// Retries the action that caused the degradation
    Retry,
    /// Blocks until the cause of the degradation disappeared
    Block, // TODO: with an UnableToDeliverStrategy::DeferToHandler, this could be omitted and Retry could achieve the same result
    /// Print out a warning as soon as the degradation is detected
    Warn,
    /// Returns a failure in the function the degradation was detected
    Fail,
}

/// Defines the cause of a degradation and is a parameter of the [`DegradationCallback`].
pub enum DegradationCause {
    /// Connection could not be established
    FailedToEstablishConnection,
    /// Connection is corrupted
    ConnectionCorrupted,
    /// Data could not be delivered
    UnableToDeliverData,
    /// The [`DegradationAction`] used by the [`DegradationCallback`] was invalid for the given [`DegradationCause`].
    /// The function will return with an error after the invocation of the [`DegradationCallback`].
    InvalidDegradationAction,
}

/// The degradation context passed to the [`DegradationCallback`]
#[repr(C)]
pub struct DegradationContext {
    /// The service id, which is involved in the degradation
    pub service_id: u128,
    /// The sender port id, which is involved in the degradation
    pub sender_port_id: u128,
    /// The receiver port id, which is involved in the degradation
    pub receiver_port_id: u128,
}

impl DegradationContext {
    /// Creates a new [`DegradationContext`]
    pub fn new(service_id: u128, sender_port_id: u128, receiver_port_id: u128) -> Self {
        Self {
            service_id,
            sender_port_id,
            receiver_port_id,
        }
    }
}

tiny_fn! {
    /// Defines a custom behavior whenever a port detects a degradation.
    pub struct DegradationCallback = Fn(cause: DegradationCause, context: &DegradationContext) -> DegradationAction;
}

unsafe impl Send for DegradationCallback<'_> {}

impl Debug for DegradationCallback<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

impl Default for DegradationCallback<'_> {
    fn default() -> Self {
        Self::new(|_, _| DegradationAction::Default)
    }
}

/// Defines a failure that can occur in
/// [`Publisher::loan()`](crate::port::publisher::Publisher::loan()) and
/// [`Publisher::loan_uninit()`](crate::port::publisher::Publisher::loan_uninit())
/// or is part of [`SendError`] emitted in
/// [`Publisher::send_copy()`](crate::port::publisher::Publisher::send_copy()).
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum LoanError {
    /// The data segment does not have any more memory left
    OutOfMemory,
    /// The maximum amount of data a user can borrow is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    ExceedsMaxLoans,
    /// The provided slice size exceeds the configured max slice size.
    /// To send data with this size a new port has to be created with as a larger slice size or the
    /// port must be configured with an
    /// [`AllocationStrategy`](iceoryx2_cal::shm_allocator::AllocationStrategy).
    ExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
}

impl core::fmt::Display for LoanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LoanError::{self:?}")
    }
}

impl core::error::Error for LoanError {}

/// Failure that can be emitted when data is sent.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SendError {
    /// Send was called but the corresponding port went already out of scope.
    ConnectionBrokenSinceSenderNoLongerExists,
    /// A connection between two ports has been corrupted.
    ConnectionCorrupted,
    /// A failure occurred while acquiring memory for the payload
    LoanError(LoanError),
    /// A failure occurred while establishing a connection to the ports counterpart port.
    ConnectionError(ConnectionFailure),
    /// The sample could not be delivered
    UnableToDeliver,
    /// An internal mechanisms failed and the data could not be delivered to all receivers.
    InternalError,
}

impl From<LoanError> for SendError {
    fn from(value: LoanError) -> Self {
        SendError::LoanError(value)
    }
}

impl From<ConnectionFailure> for SendError {
    fn from(value: ConnectionFailure) -> Self {
        SendError::ConnectionError(value)
    }
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

/// Defines the failure that can occur when receiving data with
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ReceiveError {
    /// The maximum amount of data a user can borrow with is
    /// defined in [`crate::config::Config`]. When this is exceeded no more data can be received
    /// until the user has released older data.
    ExceedsMaxBorrows,

    /// Occurs when a receiver is unable to connect to a corresponding sender.
    ConnectionFailure(ConnectionFailure),
}

impl From<ConnectionFailure> for ReceiveError {
    fn from(value: ConnectionFailure) -> Self {
        ReceiveError::ConnectionFailure(value)
    }
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}
