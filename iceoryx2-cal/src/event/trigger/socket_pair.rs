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
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_bb_posix::socket_pair::{
    StreamingSocket, StreamingSocketDuplicateError, StreamingSocketPairCreationError,
    StreamingSocketPairReceiveError, StreamingSocketPairSendError,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_log::fail;

const RECEIVE_BUFFER_SIZE: usize = 32;

#[derive(Debug)]
#[repr(C)]
pub struct SocketPairMgmt {
    handler: StreamingSocket,
}

#[derive(Debug)]
pub struct SocketPairHandle<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> {
    sender: StreamingSocket,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> Abandonable
    for SocketPairHandle<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.sender) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>>
    HandlerInterface<E, SocketPairMgmt, Storage> for SocketPairHandle<E, Storage>
{
    fn open(
        _name: &FileName,
        _config: &super::Configuration,
        mgmt: &SocketPairMgmt,
    ) -> Result<Self, NotifierOpenError> {
        let origin = "SocketPairHandle::open()";
        let msg = "Unable to open socket pair handle";

        let sender = match mgmt.handler.duplicate() {
            Ok(v) => v,
            Err(StreamingSocketDuplicateError::Interrupt) => {
                fail!(from origin, with NotifierOpenError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(StreamingSocketDuplicateError::FileDescriptorBroken) => {
                fail!(from origin, with NotifierOpenError::DoesNotExist,
                    "{msg} since the other side disconnected and closed the handler socket.");
            }
            Err(e) => {
                fail!(from origin, with NotifierOpenError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        };

        Ok(Self {
            sender,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }

    fn notify(&self) -> Result<(), NotifierNotifyError> {
        let msg = "Unable to send notification";
        match self.sender.try_send(&[0u8]) {
            Ok(0) => {
                fail!(from self, with NotifierNotifyError::FailedToDeliverSignal,
                    "{msg} since data could not be sent through the socket.");
            }
            Ok(_) => Ok(()),
            Err(StreamingSocketPairSendError::Interrupt) => {
                fail!(from self,
                    with NotifierNotifyError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(StreamingSocketPairSendError::Disconnected)
            | Err(StreamingSocketPairSendError::ConnectionReset) => {
                fail!(from self, with NotifierNotifyError::Disconnected,
                    "{msg} since the other side is disconnected.");
            }
            Err(e) => {
                fail!(from self, with NotifierNotifyError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }
}

#[derive(Debug)]
pub struct SocketPairWaiter<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> {
    mgmt: *mut SocketPairMgmt,
    receiver: StreamingSocket,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> Drop
    for SocketPairWaiter<E, Storage>
{
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.mgmt);
        }
    }
}

unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> Send
    for SocketPairWaiter<E, Storage>
{
}

unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> Sync
    for SocketPairWaiter<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>> Abandonable
    for SocketPairWaiter<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.receiver) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>>
    SocketPairWaiter<E, Storage>
{
    fn empty_buffer(&self) -> Result<(), ListenerWaitError> {
        loop {
            let msg = "Unable to empty notification buffer";
            let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
            match self.receiver.try_receive(&mut buffer) {
                Ok(RECEIVE_BUFFER_SIZE) => continue,
                Ok(_) | Err(StreamingSocketPairReceiveError::ConnectionReset) => return Ok(()),
                Err(StreamingSocketPairReceiveError::Interrupt) => {
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
}

impl<E: EventState, Storage: DynamicStorage<State<E, SocketPairMgmt>>>
    WaiterInterface<E, SocketPairMgmt, Storage> for SocketPairWaiter<E, Storage>
{
    fn create(
        _name: &FileName,
        _config: &super::Configuration,
        mgmt: &mut core::mem::MaybeUninit<SocketPairMgmt>,
    ) -> Result<Self, ListenerCreateError> {
        let origin = "SocketPairWaiter::create()";
        let msg = "Unable to create socket pair trigger";

        let (handler, waiter) = match StreamingSocket::create_pair() {
            Ok((handler, waiter)) => (handler, waiter),
            Err(StreamingSocketPairCreationError::Interrupt) => {
                fail!(from origin,
                      with ListenerCreateError::Interrupt,
                      "{msg} since an interrupt signal was raised.");
            }
            Err(StreamingSocketPairCreationError::InsufficientPermissions) => {
                fail!(from origin,
                      with ListenerCreateError::InsufficientPermissions,
                      "{msg} due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from origin,
                      with ListenerCreateError::InternalFailure,
                      "{msg} due to an internal failure. [{e:?}]");
            }
        };

        mgmt.write(SocketPairMgmt { handler });

        Ok(Self {
            mgmt: mgmt.as_mut_ptr(),
            receiver: waiter,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        let msg = "Unable to try wait for a notification";
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
        match self.receiver.try_receive(&mut buffer) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_) | Err(StreamingSocketPairReceiveError::ConnectionReset) => Ok(()),
            Err(StreamingSocketPairReceiveError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }

    fn timed_wait(
        &self,
        timeout: core::time::Duration,
    ) -> Result<(), crate::event::ListenerWaitError> {
        let msg = "Unable to wait with timeout for a notification";
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
        match self.receiver.timed_receive(&mut buffer, timeout) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_) | Err(StreamingSocketPairReceiveError::ConnectionReset) => Ok(()),
            Err(StreamingSocketPairReceiveError::Interrupt) => {
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
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
        match self.receiver.blocking_receive(&mut buffer) {
            Ok(RECEIVE_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_) | Err(StreamingSocketPairReceiveError::ConnectionReset) => Ok(()),
            Err(StreamingSocketPairReceiveError::Interrupt) => {
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

unsafe impl ZeroCopySend for SocketPairMgmt {}

#[allow(type_alias_bounds)] // they are not enforced, but we keep them to communicate the contract
pub type GenericSocketPairTrigger<E: EventState> = EventImpl<
    E,
    SocketPairMgmt,
    dynamic_storage::process_local::Storage<State<RelocatableBitSet, SocketPairMgmt>>,
    SocketPairHandle<
        E,
        dynamic_storage::process_local::Storage<State<RelocatableBitSet, SocketPairMgmt>>,
    >,
    SocketPairWaiter<
        E,
        dynamic_storage::process_local::Storage<State<RelocatableBitSet, SocketPairMgmt>>,
    >,
>;

pub type SocketPairBitSet = GenericSocketPairTrigger<RelocatableBitSet>;
