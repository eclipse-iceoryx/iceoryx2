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

use crate::{
    dynamic_storage::{self, DynamicStorage},
    event::{
        ListenerCreateError, ListenerWaitError, NotifierNotifyError, NotifierOpenError,
        common::EventImpl,
        event_state::EventState,
        trigger::{HandlerInterface, State, WaiterInterface},
    },
};
use core::marker::PhantomData;
use core::ptr::NonNull;
use iceoryx2_bb_elementary_traits::{non_null::NonNullCompat, testing::abandonable::Abandonable};
use iceoryx2_bb_lock_free::mpmc::{
    bit_set::RelocatableBitSet, counting_bit_set::RelocatableCountingBitSet,
};
use iceoryx2_bb_posix::{
    file::CreationMode,
    unix_datagram_socket::{
        UnixDatagramReceiveError, UnixDatagramReceiver, UnixDatagramReceiverBuilder,
        UnixDatagramReceiverCreationError, UnixDatagramSendError, UnixDatagramSender,
        UnixDatagramSenderBuilder, UnixDatagramSenderCreationError,
    },
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_log::fail;

const RECEIVE_BUFFER_SIZE: u64 = 32;

#[derive(Debug)]
pub struct UnixDatagramHandle<E: EventState, Storage: DynamicStorage<State<E, ()>>> {
    sender: UnixDatagramSender,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> Abandonable
    for UnixDatagramHandle<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { UnixDatagramSender::abandon_in_place(NonNull::iox2_from_mut(&mut this.sender)) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> HandlerInterface<E, (), Storage>
    for UnixDatagramHandle<E, Storage>
{
    fn notify(&self) -> Result<(), NotifierNotifyError> {
        let msg = "Unable to send notification";
        match self.sender.try_send(&[0u8]) {
            Ok(true) => Ok(()),
            Ok(false) => {
                fail!(from self, with NotifierNotifyError::BufferIsFull,
                    "{msg} since data could not be sent through the socket.");
            }
            Err(UnixDatagramSendError::Interrupt) => {
                fail!(from self,
                    with NotifierNotifyError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(UnixDatagramSendError::InsufficientPermissions) => {
                fail!(from self,
                    with NotifierNotifyError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(UnixDatagramSendError::NotConnected)
            | Err(UnixDatagramSendError::ConnectionReset)
            | Err(UnixDatagramSendError::ConnectionRefused) => {
                fail!(from self,
                    with NotifierNotifyError::Disconnected,
                    "{msg} since the other side is disconnected.");
            }
            Err(e) => {
                fail!(from self,
                    with NotifierNotifyError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }

    fn open(
        name: &FileName,
        config: &super::Configuration,
        _mgmt: &(),
    ) -> Result<Self, NotifierOpenError> {
        let origin = "UnixDatagramHandler::open()";
        let msg = "Unable to connect to unix datagram socket";
        let full_path = config.path_for(name);
        let sender = match UnixDatagramSenderBuilder::new(&full_path).create() {
            Ok(v) => v,
            Err(UnixDatagramSenderCreationError::Interrupt) => {
                fail!(from origin, with NotifierOpenError::Interrupt,
                    "{msg} with {config:?} since an interrupt signal was raised.");
            }
            Err(UnixDatagramSenderCreationError::InsufficientPermissions) => {
                fail!(from origin, with NotifierOpenError::InsufficientPermissions,
                    "{msg} with {config:?} due to insufficient permissions.");
            }
            Err(UnixDatagramSenderCreationError::DoesNotExist) => {
                fail!(from origin, with NotifierOpenError::DoesNotExist,
                    "{msg} with {config:?} since the socket does not exist.");
            }
            Err(e) => {
                fail!(from origin, with NotifierOpenError::InternalFailure,
                    "{msg} with {config:?} due to an internal failure. [{e:?}]");
            }
        };

        Ok(Self {
            sender,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct UnixDatagramWaiter<E: EventState, Storage: DynamicStorage<State<E, ()>>> {
    receiver: UnixDatagramReceiver,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> Abandonable
    for UnixDatagramWaiter<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            UnixDatagramReceiver::abandon_in_place(NonNull::iox2_from_mut(&mut this.receiver))
        };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> WaiterInterface<E, (), Storage>
    for UnixDatagramWaiter<E, Storage>
{
    fn empty_buffer(&self) -> Result<(), ListenerWaitError> {
        let msg = "Unable to empty notification buffer";
        loop {
            let mut buffer = [0u8; RECEIVE_BUFFER_SIZE as usize];
            match self.receiver.try_receive(&mut buffer) {
                Ok(RECEIVE_BUFFER_SIZE) => continue,
                Ok(_)
                | Err(UnixDatagramReceiveError::NotConnected)
                | Err(UnixDatagramReceiveError::ConnectionReset) => return Ok(()),
                Err(UnixDatagramReceiveError::Interrupt) => {
                    fail!(from self, with ListenerWaitError::InterruptSignal,
                        "{msg} since an interrupt signal was raised.");
                }
                Err(e) => {
                    fail!(from self, with ListenerWaitError::InternalFailure,
                        "{msg} due to an internal failure. [{e:?}]");
                }
            }
        }
    }

    fn create(
        name: &FileName,
        config: &super::Configuration,
        _mgmt: &mut core::mem::MaybeUninit<()>,
    ) -> Result<Self, ListenerCreateError> {
        let origin = "UnixDatagramWaiter::create()";
        let msg = "Unable to create unix datagram socket trigger";
        let full_path = config.path_for(name);
        let receiver = match UnixDatagramReceiverBuilder::new(&full_path)
            .creation_mode(CreationMode::CreateExclusive)
            .create()
        {
            Ok(v) => v,
            Err(UnixDatagramReceiverCreationError::InsufficientPermissions) => {
                fail!(from origin, with ListenerCreateError::InsufficientPermissions,
                    "{msg} with {config:?} due to insufficient permissions.");
            }
            Err(UnixDatagramReceiverCreationError::AddressAlreadyInUse)
            | Err(UnixDatagramReceiverCreationError::SocketFileAlreadyExists) => {
                fail!(from origin, with ListenerCreateError::AlreadyExists,
                    "{msg} with {config:?} since it already exists.");
            }
            Err(e) => {
                fail!(from origin, with ListenerCreateError::InternalFailure,
                    "{msg} with {config:?} due to an internal error. [{e:?}]");
            }
        };

        Ok(Self {
            receiver,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        let msg = "Unable to try wait for a notification";
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE as usize];
        match self.receiver.try_receive(&mut buffer) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_)
            | Err(UnixDatagramReceiveError::NotConnected)
            | Err(UnixDatagramReceiveError::ConnectionReset) => Ok(()),
            Err(UnixDatagramReceiveError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }

    fn timed_wait(&self, timeout: core::time::Duration) -> Result<(), ListenerWaitError> {
        let msg = "Unable to wait with a timeout for a notification";
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE as usize];
        match self.receiver.timed_receive(&mut buffer, timeout) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_)
            | Err(UnixDatagramReceiveError::NotConnected)
            | Err(UnixDatagramReceiveError::ConnectionReset) => Ok(()),
            Err(UnixDatagramReceiveError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }

    fn blocking_wait(&self) -> Result<(), crate::event::ListenerWaitError> {
        let msg = "Unable to blocking wait for a notification";
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE as usize];
        match self.receiver.blocking_receive(&mut buffer) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_)
            | Err(UnixDatagramReceiveError::NotConnected)
            | Err(UnixDatagramReceiveError::ConnectionReset) => Ok(()),
            Err(UnixDatagramReceiveError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }
}

#[allow(type_alias_bounds)] // they are not enforced, but we keep them to communicate the contract
pub type GenericUnixDatagramSocketTrigger<E: EventState, Storage: DynamicStorage<State<E, ()>>> =
    EventImpl<E, (), Storage, UnixDatagramHandle<E, Storage>, UnixDatagramWaiter<E, Storage>>;

pub type UnixDatagramShmBitSet = GenericUnixDatagramSocketTrigger<
    RelocatableBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableBitSet, ()>>,
>;

pub type UnixDatagramShmCountingBitSet = GenericUnixDatagramSocketTrigger<
    RelocatableCountingBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableCountingBitSet, ()>>,
>;
