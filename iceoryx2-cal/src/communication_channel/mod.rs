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

//! A [`CommunicationChannel`] is identified by a name and allows two processes to communicate
//! with each other (inter-process communication).
//!
//! It consists of exactly one [`CommunicationChannelReceiver`] which can be created with the
//! [`CommunicationChannelCreator`] and exactly one
//! [`CommunicationChannelSender`] which can be created with the [`CommunicationChannelConnector`].
//!
//! A [`CommunicationChannel`] has to fulfill the following contract:
//!  * zero sized names are not valid
//!  * **unique:** multiple [`CommunicationChannel`]s with the same name cannot be created
//!  * the receiver always creates the [`CommunicationChannel`]
//!  * the sender always opens the [`CommunicationChannel`]
//!  * non-existing [`CommunicationChannel`]s cannot be opened
//!  * the communication must have fifo behavior
//!  * the default receiver buffer size must be at least [`DEFAULT_RECEIVER_BUFFER_SIZE`]
//!  * must be able to transmit at least u64 values (larger more complex values are allowed as well)
//!  * The [`CommunicationChannelSender`] must be able to handle a [`CommunicationChannelReceiver`]
//!    which removes the underlying channel.
//!
//! The contract is verified by the corresponding unit tests. Every [`CommunicationChannel`] must
//! pass the test.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_cal::communication_channel::*;
//! use iceoryx2_cal::named_concept::*;
//!
//! // the following two functions can be implemented in different processes
//! fn process_one<Channel: CommunicationChannel<u64>>() {
//!     let channel_name = FileName::new(b"myChannelName").unwrap();
//!     let mut receiver = Channel::Creator::new(&channel_name).create_receiver().unwrap();
//!
//!     println!("Create channel {}", receiver.name());
//!     match receiver.receive().unwrap() {
//!         Some(data) => println!("Received {:?}", data),
//!         None => println!("Received nothing")
//!     }
//! }
//!
//! fn process_two<Channel: CommunicationChannel<u64>>() {
//!     let channel_name = FileName::new(b"myChannelName").unwrap();
//!     let mut sender = Channel::Connector::new(&channel_name).open_sender().unwrap();
//!     let data: u64 = 1238712390;
//!
//!     println!("Open channel {}", sender.name());
//!     match sender.send(&data).is_ok() {
//!         true => println!("Sent {}", data),
//!         false => println!("Unable to send data. Receiver queue full?"),
//!     }
//! }
//! ```

pub mod posix_shared_memory;
pub mod process_local;
pub mod recommended;
pub mod unix_datagram;

use core::fmt::Debug;

use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_system_types::path::Path;

use crate::named_concept::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};

/// The buffer size which the receiver has at least by default
pub const DEFAULT_RECEIVER_BUFFER_SIZE: usize = 8;

/// Describes failures when sending data
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CommunicationChannelSendError {
    ConnectionBroken,
    MessageTooLarge,
    ReceiverCacheIsFull,
    InternalFailure,
}

/// Describes failures when receiving data
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CommunicationChannelReceiveError {
    ConnectionBroken,
    MessageCorrupt,
    InternalFailure,
}

/// Describes failures when creating the [`CommunicationChannel`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CommunicationChannelCreateError {
    AlreadyExists,
    SafeOverflowNotSupported,
    CustomBufferSizeNotSupported,
    InternalFailure,
}

/// Describes failures when opening the [`CommunicationChannel`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CommunicationChannelOpenError {
    InternalFailure,
    AnotherInstanceIsAlreadyConnected,
    DoesNotExist,
}

/// Creates a [`CommunicationChannel`].
pub trait CommunicationChannelCreator<T, C: CommunicationChannel<T> + Sized>:
    NamedConceptBuilder<C>
{
    /// Activates safe overflow for the channel. If the receiver buffer is full the oldest data
    /// is returned to the sender and replaced with the newest data.
    fn enable_safe_overflow(self) -> Self;

    /// Sets the internal receive buffer of the channel. Defines the buffer the channel has at
    /// least but it is allowed to be larger.
    fn buffer_size(self, value: usize) -> Self;

    /// Creates a new [`CommunicationChannel`] and returns a [`CommunicationChannelReceiver`]
    fn create_receiver(self) -> Result<C::Receiver, CommunicationChannelCreateError>;
}

/// Connects to a [`CommunicationChannel`].
pub trait CommunicationChannelConnector<T, C: CommunicationChannel<T> + Sized>:
    NamedConceptBuilder<C>
{
    /// Opens an existing [`CommunicationChannel`] and returns a [`CommunicationChannelSender`]
    fn open_sender(self) -> Result<C::Sender, CommunicationChannelOpenError>;

    /// Opens an existing [`CommunicationChannel`] and returns a [`CommunicationChannelSender`].
    /// In contrast to the counterpart [`CommunicationChannelConnector::open_sender()`] it does
    /// not print an error message when the channel does not exist.
    fn try_open_sender(self) -> Result<C::Sender, CommunicationChannelOpenError>;
}

pub trait CommunicationChannelParticipant {
    /// Returns true when the channel returns and exchanges the oldest data with the
    /// newest data when it is full (safe overflow).
    fn does_enable_safe_overflow(&self) -> bool;
}

/// Sends data to the corresponding [`CommunicationChannelReceiver`].
pub trait CommunicationChannelSender<T>:
    Debug + CommunicationChannelParticipant + NamedConcept
{
    /// If the corresponding receiver is able to receive it sends the data and returns true,
    /// otherwise false. If the channel is configured to be safely overflowing it returns the
    /// oldest sample when receiver buffer was full and overrides it with the newest data.
    fn send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError>;

    /// If the corresponding receiver is able to receive it sends the data and returns true,
    /// otherwise false. If the channel is configured to be safely overflowing it returns the
    /// oldest sample when receiver buffer was full and overrides it with the newest data.
    /// In contrast to the counterpart [`CommunicationChannelSender::send()`]
    /// it does not print an error message when the receiver buffer is full.
    fn try_send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError>;
}

/// Receives data from a corresponding [`CommunicationChannelSender`].
pub trait CommunicationChannelReceiver<T>:
    Debug + CommunicationChannelParticipant + NamedConcept
{
    /// Returns the maximum amount of message the [`CommunicationChannelReceiver`] can receive
    /// without acquiring them. If underlying buffer is full the [`CommunicationChannelSender`]
    /// is no longer able to send messages or, when safe overflow is enabled, it returns the oldest
    /// data to the sender and replaces it with the newest data.
    fn buffer_size(&self) -> usize;

    /// Tries to receive data. If no data is present it returns [`None`] otherwise the data.
    fn receive(&self) -> Result<Option<T>, CommunicationChannelReceiveError>;
}

/// Bundles all corresponding [`CommunicationChannelSender`], [`CommunicationChannelReceiver`]
/// [`CommunicationChannelConnector`] and [`CommunicationChannelCreator`] together in one object.
pub trait CommunicationChannel<T>: Sized + Debug + NamedConceptMgmt {
    type Sender: CommunicationChannelSender<T>;
    type Receiver: CommunicationChannelReceiver<T>;
    type Creator: CommunicationChannelCreator<T, Self>;
    type Connector: CommunicationChannelConnector<T, Self>;

    /// Returns true if the channel supports safe overflow
    fn does_support_safe_overflow() -> bool {
        false
    }

    /// Returns true if the buffer size of the channel can be configured
    fn has_configurable_buffer_size() -> bool {
        false
    }

    /// The default suffix of every communication channel
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".com") }
    }
}
