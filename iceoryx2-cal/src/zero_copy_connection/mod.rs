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

pub mod common;
pub mod file;
pub mod posix_shared_memory;
pub mod process_local;
pub mod recommended;
pub mod used_chunk_list;

use core::fmt::Debug;
use core::time::Duration;

pub use crate::shared_memory::PointerOffset;
pub use iceoryx2_bb_system_types::file_name::*;
pub use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fail;

use crate::static_storage::file::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZeroCopyPortRemoveError {
    InternalError,
    VersionMismatch,
    InsufficientPermissions,
    DoesNotExist,
}

impl core::fmt::Display for ZeroCopyPortRemoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ZeroCopyPortRemoveError::{self:?}")
    }
}

impl core::error::Error for ZeroCopyPortRemoveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZeroCopyCreationError {
    InternalError,
    IsBeingCleanedUp,
    AnotherInstanceIsAlreadyConnected,
    InsufficientPermissions,
    VersionMismatch,
    ConnectionMaybeCorrupted,
    InvalidSampleSize,
    InitializationNotYetFinalized,
    IncompatibleBufferSize,
    IncompatibleMaxBorrowedSamplesPerChannelSetting,
    IncompatibleOverflowSetting,
    IncompatibleNumberOfSamples,
    IncompatibleNumberOfSegments,
    IncompatibleNumberOfChannels,
}

impl core::fmt::Display for ZeroCopyCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ZeroCopyCreationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroCopySendError {
    ConnectionCorrupted,
    ReceiveBufferFull,
    UnableToDeliver, // NOTE: in order to distinguish between a try_send failure and a user induced send aborting, the UnableToDeliver error is used
    UsedChunkListFull,
    NoConnectedReceiver,
    ChannelIsClosed,
    InternalError,
}

impl core::fmt::Display for ZeroCopySendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ZeroCopySendError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroCopyReceiveError {
    ReceiveWouldExceedMaxBorrowValue,
}

impl core::fmt::Display for ZeroCopyReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ZeroCopyReceiveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroCopyReclaimError {
    ReceiverReturnedCorruptedPointerOffset,
}

impl core::fmt::Display for ZeroCopyReclaimError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ZeroCopyReclaimError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroCopyReleaseError {
    RetrieveBufferFull,
}

impl core::fmt::Display for ZeroCopyReleaseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ZeroCopyReleaseError {}

/// Defines the action that shall be take when a [`PointerOffset`]
/// cannot be delivered.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UnableToDeliverToReceiverAction {
    /// Use an action which is derived from the `UnableToDeliverStrategy`
    FollowUnableToDeliveryStrategy,
    /// Retry to send and invoke the handler again, if sending does not succeed
    Retry,
    /// Discard the [`PointerOffset`] for the [`ZeroCopyReceiver`] which cause the incident
    /// and continue to deliver the [`PointerOffset`] to the remaining [`ZeroCopyReceiver`]s
    DiscardPointerOffset,
    /// Discard the [`PointerOffset`] for the [`ZeroCopyReceiver`] which caused the incident,
    /// continue to deliver the [`PointerOffset`] to the remaining [`ZeroCopyReceiver`]s;
    /// return with an error if the [`PointerOffset`] was not delivered to all [`ZeroCopyReceiver`]s
    DiscardPointerOffsetAndFail,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ZeroCopySend)]
pub struct ChannelId(usize);

impl ChannelId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStateNewError {
    ValueOutOfBounds,
}

impl core::fmt::Display for ChannelStateNewError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", stringify!(Self), self)
    }
}

impl core::error::Error for ChannelStateNewError {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ZeroCopySend)]
pub struct ChannelState(u64);

impl ChannelState {
    pub fn new(value: u64) -> Result<Self, ChannelStateNewError> {
        if value > Self::max_value() {
            fail!(from "ChannelState::new()", with ChannelStateNewError::ValueOutOfBounds,
                "Unable to create new ChannelState since the value must be less or equal than 2^62 and this value is {value},");
        }

        Ok(Self(value))
    }

    pub const fn max_value() -> u64 {
        2u64.pow(62)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

pub const DEFAULT_BUFFER_SIZE: usize = 4;
pub const DEFAULT_ENABLE_SAFE_OVERFLOW: bool = false;
pub const DEFAULT_MAX_BORROWED_SAMPLES_PER_CHANNEL: usize = 4;
pub const DEFAULT_MAX_SUPPORTED_SHARED_MEMORY_SEGMENTS: u8 = 1;
pub const DEFAULT_NUMBER_OF_CHANNELS: usize = 1;
pub const DEFAULT_NUMBER_OF_SAMPLES_PER_SEGMENT: usize = 8;
/// The initial value of the channel state
pub const CHANNEL_STATE_OPEN: ChannelState = ChannelState(0);
/// A channel with an invalid state will never block in [`ZeroCopySender::blocking_send()`];
pub const CHANNEL_STATE_CLOSED: ChannelState = ChannelState(u64::MAX);
/// Hints the channel that the other side intends to disconnect.
const CHANNEL_STATE_DISCONNECT_HINT_BIT: u64 = 1u64 << 63;

pub trait ZeroCopyConnectionBuilder<C: ZeroCopyConnection>: NamedConceptBuilder<C> {
    fn buffer_size(self, value: usize) -> Self;
    fn enable_safe_overflow(self, value: bool) -> Self;
    fn receiver_max_borrowed_samples_per_channel(self, value: usize) -> Self;
    fn max_supported_shared_memory_segments(self, value: u8) -> Self;
    fn number_of_samples_per_segment(self, value: usize) -> Self;
    fn number_of_channels(self, value: usize) -> Self;
    fn initial_channel_state(self, value: ChannelState) -> Self;
    /// The timeout defines how long the [`ZeroCopyConnectionBuilder`] should wait for
    /// concurrent
    /// [`ZeroCopyConnectionBuilder::create_sender()`] or
    /// [`ZeroCopyConnectionBuilder::create_receiver()`] call to finalize its initialization.
    /// By default it is set to [`Duration::ZERO`] for no timeout.
    fn timeout(self, value: Duration) -> Self;

    fn create_sender(self) -> Result<C::Sender, ZeroCopyCreationError>;
    fn create_receiver(self) -> Result<C::Receiver, ZeroCopyCreationError>;
}

pub trait ZeroCopyPortDetails {
    fn number_of_channels(&self) -> usize;
    fn buffer_size(&self) -> usize;
    fn has_enabled_safe_overflow(&self) -> bool;
    fn max_borrowed_samples(&self) -> usize;
    fn max_supported_shared_memory_segments(&self) -> u8;
    fn is_connected(&self) -> bool;
    #[doc(hidden)]
    fn __internal_get_channel_state(&self, channel_id: ChannelId) -> &AtomicU64;

    fn set_channel_state(&self, channel_id: ChannelId, state: ChannelState) -> bool {
        self.__internal_get_channel_state(channel_id)
            .compare_exchange(
                CHANNEL_STATE_CLOSED.0,
                state.0,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    fn set_disconnect_hint(&self, channel_id: ChannelId, expected_state: ChannelState) {
        let disconnect_hint_state = expected_state.0 | CHANNEL_STATE_DISCONNECT_HINT_BIT;

        let _ = self
            .__internal_get_channel_state(channel_id)
            .compare_exchange(
                expected_state.0,
                disconnect_hint_state,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
    }

    fn has_disconnect_hint(&self, channel_id: ChannelId, expected_state: ChannelState) -> bool {
        let disconnect_hint_state = expected_state.0 | CHANNEL_STATE_DISCONNECT_HINT_BIT;
        disconnect_hint_state
            == self
                .__internal_get_channel_state(channel_id)
                .load(Ordering::Relaxed)
    }

    fn has_channel_state(&self, channel_id: ChannelId, expected_state: ChannelState) -> bool {
        let state = self
            .__internal_get_channel_state(channel_id)
            .load(Ordering::Relaxed);
        let state_without_disconnect_hint_bit = state & !(CHANNEL_STATE_DISCONNECT_HINT_BIT);
        expected_state.0 == state_without_disconnect_hint_bit
    }

    fn close_channel(&self, channel_id: ChannelId, expected_state: ChannelState) {
        match self
            .__internal_get_channel_state(channel_id)
            .compare_exchange(
                expected_state.0,
                CHANNEL_STATE_CLOSED.0,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
            Ok(_) => (),
            Err(v) => {
                let graceful_disconnect_state =
                    expected_state.0 | CHANNEL_STATE_DISCONNECT_HINT_BIT;
                if v == graceful_disconnect_state {
                    let _ = self
                        .__internal_get_channel_state(channel_id)
                        .compare_exchange(
                            graceful_disconnect_state,
                            CHANNEL_STATE_CLOSED.0,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        );
                }
            }
        }
    }
}

/// The unable to delivery handler invoked by a send function when a [`PointerOffset`]
/// cannot be delivered to a [`ZeroCopyReceiver`]
///
/// # Arguments
///
/// * u64: is the retry counter for a delivery incident with a specific sender-receiver pair
/// * Duration: is the elapsed time since the incident was detected
///
/// # Returns
///
/// The [`UnableToDeliverToReceiverAction`] to be taken to mitigate the incident
pub trait UnableToDeliverToReceiverFn:
    Fn(u64, Duration) -> UnableToDeliverToReceiverAction
{
}

impl<F: Fn(u64, Duration) -> UnableToDeliverToReceiverAction> UnableToDeliverToReceiverFn for F {}

pub trait ZeroCopySender: Debug + ZeroCopyPortDetails + NamedConcept + Send {
    fn try_send(
        &self,
        ptr: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
    ) -> Result<Option<PointerOffset>, ZeroCopySendError>;

    fn blocking_send<F: UnableToDeliverToReceiverFn>(
        &self,
        ptr: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
        unable_to_deliver_to_receiver_handler: F,
        unable_to_deliver_action_for_strategy: UnableToDeliverToReceiverAction,
    ) -> Result<Option<PointerOffset>, ZeroCopySendError>;

    fn reclaim(&self, channel_id: ChannelId)
    -> Result<Option<PointerOffset>, ZeroCopyReclaimError>;

    /// # Safety
    ///
    /// * must ensure that no receiver is still holding data, otherwise data races may occur on
    ///   receiver side
    /// * must ensure that [`ZeroCopySender::try_send()`] and [`ZeroCopySender::blocking_send()`]
    ///   are not called after using this method
    unsafe fn acquire_used_offsets<F: FnMut(PointerOffset)>(&self, callback: F);
}

pub trait ZeroCopyReceiver: Debug + ZeroCopyPortDetails + NamedConcept + Send {
    fn has_data(&self, channel_id: ChannelId) -> bool;
    fn receive(&self, channel_id: ChannelId)
    -> Result<Option<PointerOffset>, ZeroCopyReceiveError>;
    fn release(
        &self,
        ptr: PointerOffset,
        channel_id: ChannelId,
    ) -> Result<(), ZeroCopyReleaseError>;
    fn borrow_count(&self, channel_id: ChannelId) -> usize;
}

pub trait ZeroCopyConnection: Debug + Sized + NamedConceptMgmt {
    type Sender: ZeroCopySender;
    type Receiver: ZeroCopyReceiver;
    type Builder: ZeroCopyConnectionBuilder<Self>;

    /// Removes the [`ZeroCopySender`] forcefully from the [`ZeroCopyConnection`]. This shall only
    /// be called when the [`ZeroCopySender`] died and the connection shall be cleaned up without
    /// causing any problems on the living [`ZeroCopyReceiver`] side.
    ///
    /// # Safety
    ///
    ///  * must ensure that the [`ZeroCopySender`] died while being connected.
    unsafe fn remove_sender(
        name: &FileName,
        config: &Self::Configuration,
    ) -> Result<(), ZeroCopyPortRemoveError>;

    /// Removes the [`ZeroCopyReceiver`] forcefully from the [`ZeroCopyConnection`]. This shall
    /// only be called when the [`ZeroCopySender`] died and the connection shall be cleaned up
    /// without causing any problems on the living [`ZeroCopySender`] side.
    ///
    /// # Safety
    ///
    ///  * must ensure that the [`ZeroCopyReceiver`] died while being connected.
    unsafe fn remove_receiver(
        name: &FileName,
        config: &Self::Configuration,
    ) -> Result<(), ZeroCopyPortRemoveError>;

    /// Returns true if the connection supports safe overflow
    fn does_support_safe_overflow() -> bool {
        false
    }

    /// Returns true if the buffer size of the connection can be configured
    fn has_configurable_buffer_size() -> bool {
        false
    }

    /// The default suffix of every zero copy connection
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".rx") }
    }
}
