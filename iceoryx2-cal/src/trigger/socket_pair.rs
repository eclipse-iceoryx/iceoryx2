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

//! [`Trigger`] implementation based on a [`StreamingSocket`] pair. An activation sends one
//! byte, a reset receives all of them. When the socket buffer is full, the trigger is
//! already activated and the activation succeeds without sending.

use core::ptr::NonNull;

use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_posix::file_descriptor::{FileDescriptor, FileDescriptorBased};
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;
use iceoryx2_bb_posix::socket_pair::{
    StreamingSocket, StreamingSocketPairCreationError, StreamingSocketPairReceiveError,
    StreamingSocketPairSendError,
};
use iceoryx2_log::fail;

use crate::trigger::{
    Activator, Observer, Trigger, TriggerActivateError, TriggerBuilder, TriggerCreateError,
    TriggerResetError,
};

const RECEIVE_BUFFER_SIZE: usize = 64;

/// The [`Trigger`] based on a [`StreamingSocket`] pair.
#[derive(Debug)]
pub struct SocketPair {}

impl Trigger for SocketPair {
    type Activator = SocketPairActivator;
    type Observer = SocketPairObserver;
    type Builder = SocketPairBuilder;
}

/// The [`TriggerBuilder`] of the [`SocketPair`] trigger.
#[derive(Debug)]
pub struct SocketPairBuilder {}

impl TriggerBuilder<SocketPair> for SocketPairBuilder {
    fn new() -> Self {
        Self {}
    }

    fn create(self) -> Result<(SocketPairActivator, SocketPairObserver), TriggerCreateError> {
        let msg = "Unable to create socket pair trigger";
        match StreamingSocket::create_pair() {
            Ok((activator, observer)) => Ok((
                SocketPairActivator { socket: activator },
                SocketPairObserver { socket: observer },
            )),
            Err(StreamingSocketPairCreationError::PerProcessFileHandleLimitReached)
            | Err(StreamingSocketPairCreationError::SystemWideFileHandleLimitReached)
            | Err(StreamingSocketPairCreationError::InsufficientResources)
            | Err(StreamingSocketPairCreationError::InsufficientMemory) => {
                fail!(from self, with TriggerCreateError::InsufficientResources,
                    "{msg} due to insufficient resources.");
            }
            Err(e) => {
                fail!(from self, with TriggerCreateError::InternalError,
                    "{msg} due to an internal failure ({e:?}).");
            }
        }
    }
}

/// The [`Activator`] of the [`SocketPair`] trigger.
#[derive(Debug)]
pub struct SocketPairActivator {
    socket: StreamingSocket,
}

impl Abandonable for SocketPairActivator {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.socket) };
    }
}

impl Activator for SocketPairActivator {
    fn activate(&self) -> Result<(), TriggerActivateError> {
        let msg = "Unable to activate the trigger";
        match self.socket.try_send(&[1u8]) {
            Ok(_) | Err(StreamingSocketPairSendError::InsufficientResources) => Ok(()),
            Err(StreamingSocketPairSendError::Disconnected)
            | Err(StreamingSocketPairSendError::ConnectionReset) => {
                fail!(from self, with TriggerActivateError::Disconnected,
                    "{msg} since the observer no longer exists.");
            }
            Err(e) => {
                fail!(from self, with TriggerActivateError::InternalError,
                    "{msg} due to an internal failure ({e:?}).");
            }
        }
    }
}

/// The [`Observer`] of the [`SocketPair`] trigger.
#[derive(Debug)]
pub struct SocketPairObserver {
    socket: StreamingSocket,
}

impl Abandonable for SocketPairObserver {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.socket) };
    }
}

impl FileDescriptorBased for SocketPairObserver {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.socket.file_descriptor()
    }
}

impl SynchronousMultiplexing for SocketPairObserver {}

impl Observer for SocketPairObserver {
    fn reset(&self) -> Result<bool, TriggerResetError> {
        let msg = "Unable to reset the trigger";
        let mut was_activated = false;
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];
        loop {
            match self.socket.try_receive(&mut buffer) {
                Ok(RECEIVE_BUFFER_SIZE) => was_activated = true,
                Ok(0) | Err(StreamingSocketPairReceiveError::ConnectionReset) => {
                    return Ok(was_activated);
                }
                Ok(_) => return Ok(true),
                Err(e) => {
                    fail!(from self, with TriggerResetError::InternalError,
                        "{msg} due to an internal failure ({e:?}).");
                }
            }
        }
    }
}
