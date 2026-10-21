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

use alloc::format;

use crate::{
    clock::{ClockListenerSchedule, ClockListenerScheduleError, ScheduleOnHandle, Timeout},
    dynamic_storage::DynamicStorage,
    event::{
        ListenerCreateError, ListenerWaitError, NotifierNotifyError, NotifierOpenError,
        common::EventImpl,
        event_state::EventState,
        trigger::{Configuration, HandlerInterface, State, WaiterInterface},
    },
    named_concept::NamedConceptRemoveError,
};
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ptr::NonNull;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_posix::named_pipe::{CreateFiFoError, NamedPipeCreationError, NamedPipeReadError};
use iceoryx2_bb_posix::{
    directory::*,
    file::{CreationMode, File, FileRemoveError},
    file_descriptor::FileDescriptorBased,
    file_descriptor_set::SynchronousMultiplexing,
    named_pipe::{
        NamedPipe, NamedPipeBuilder, NamedPipeOpenError, NamedPipeOpenFdError, NamedPipeWriteError,
    },
    permission::Permission,
};
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::{fail, trace};

const READ_BUFFER_SIZE: u64 = 32;

#[cfg(not(feature = "dev_permissions"))]
const PIPE_PERMISSIONS: Permission = Permission::OWNER_READ_WRITE;
#[cfg(not(feature = "dev_permissions"))]
const DIR_PERMISSIONS: Permission = Permission::OWNER_ALL
    .const_bitor(Permission::GROUP_READ)
    .const_bitor(Permission::GROUP_EXEC);

#[cfg(feature = "dev_permissions")]
const PIPE_PERMISSIONS: Permission = Permission::OWNER_READ_WRITE
    .const_bitor(Permission::GROUP_READ_WRITE)
    .const_bitor(Permission::OTHERS_READ_WRITE);
#[cfg(feature = "dev_permissions")]
const DIR_PERMISSIONS: Permission = Permission::ALL;

#[derive(Debug)]
pub struct NamedPipeHandle<E: EventState, Storage: DynamicStorage<State<E, ()>>> {
    sender: NamedPipe,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> FileDescriptorBased
    for NamedPipeHandle<E, Storage>
{
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        self.sender.file_descriptor()
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> SynchronousMultiplexing
    for NamedPipeHandle<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> Abandonable
    for NamedPipeHandle<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { NamedPipe::abandon_in_place(NonNull::from_mut(&mut this.sender)) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> HandlerInterface<E, (), Storage>
    for NamedPipeHandle<E, Storage>
{
    fn notify(&self) -> Result<(), NotifierNotifyError> {
        let msg = "Unable to send notification";
        match self.sender.write(&[0u8]) {
            Ok(1) => Ok(()),
            Ok(0) => {
                fail!(from self, with NotifierNotifyError::BufferIsFull,
                    "{msg} since data could not be sent through the named pipe.");
            }
            Ok(n) => {
                fail!(from self,
                    with NotifierNotifyError::InternalFailure,
                    "{msg} expected to send 1 byte but it seems that actual {n} bytes were transmitted.");
            }
            Err(NamedPipeWriteError::InterruptedBySignal) => {
                fail!(from self,
                    with NotifierNotifyError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(NamedPipeWriteError::OpenedWithoutWriteAccessMode) => {
                fail!(from self,
                    with NotifierNotifyError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(NamedPipeWriteError::ReceivingPipeEndNotOpened) => {
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
        let origin = "NamedPipeHandle::open()";
        let msg = "Unable to connect to named pipe";
        let full_path = config.path_for(name);
        let sender = match NamedPipeBuilder::new(&full_path)
            .has_ownership(false)
            .open_existing()
        {
            Ok(v) => v,
            Err(NamedPipeOpenError::NamedPipeOpenFdError(
                NamedPipeOpenFdError::InterruptedBySignal,
            )) => {
                fail!(from origin, with NotifierOpenError::Interrupt,
                    "{msg} with {config:?} since an interrupt signal was raised.");
            }
            Err(NamedPipeOpenError::NamedPipeOpenFdError(
                NamedPipeOpenFdError::InsufficientPermissions,
            )) => {
                fail!(from origin, with NotifierOpenError::InsufficientPermissions,
                    "{msg} with {config:?} due to insufficient permissions.");
            }
            Err(NamedPipeOpenError::NamedPipeOpenFdError(
                NamedPipeOpenFdError::FileDoesNotExist,
            )) => {
                fail!(from origin, with NotifierOpenError::DoesNotExist,
                    "{msg} with {config:?} since path to the named pipe does not exist.");
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
pub struct NamedPipeWaiter<E: EventState, Storage: DynamicStorage<State<E, ()>>> {
    name: FileName,
    receiver: NamedPipe,
    clock: Arc<dyn ClockListenerSchedule>,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> FileDescriptorBased
    for NamedPipeWaiter<E, Storage>
{
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        self.receiver.file_descriptor()
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> SynchronousMultiplexing
    for NamedPipeWaiter<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> Abandonable
    for NamedPipeWaiter<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { NamedPipe::abandon_in_place(NonNull::from_mut(&mut this.receiver)) };
        unsafe { core::ptr::drop_in_place(&mut this.clock) };
    }
}

unsafe impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> Send
    for NamedPipeWaiter<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, ()>>> WaiterInterface<E, (), Storage>
    for NamedPipeWaiter<E, Storage>
{
    const IS_FILE_DESCRIPTOR_BASED: bool = true;

    unsafe fn remove(
        name: &FileName,
        config: &Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        let origin = "NamedPipeWaiter::remove()";
        let msg = "Unable to remove named pipe";
        let full_path = config.path_for(name);
        match File::remove(&full_path) {
            Ok(v) => Ok(v),
            Err(FileRemoveError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                    "{msg} due to an internal error. [{e:?}]");
            }
        }
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        crate::named_concept::remove_path_hint(value)
    }

    fn empty_buffer(&self) -> Result<(), ListenerWaitError> {
        let msg = "Unable to empty notification buffer";
        loop {
            let mut buffer = [0u8; READ_BUFFER_SIZE as usize];
            match self.receiver.try_read(&mut buffer) {
                Ok(READ_BUFFER_SIZE) => continue,
                Ok(_) => return Ok(()),
                Err(NamedPipeReadError::InterruptedBySignal) => {
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
        clock: Arc<dyn ClockListenerSchedule>,
    ) -> Result<Self, ListenerCreateError> {
        let origin = "NamedPipeWaiter::create()";
        let msg = "Unable to create named pipe trigger";

        // create root directory before the named Pipe
        let dir_msg = format!("Unable to create root directory \"{}\"", config.path_hint);
        let Ok(root_dir_exist) = Directory::does_exist(&config.path_hint) else {
            fail!(from origin, with ListenerCreateError::RootDirectoryCreationFailure,
                "{} since the system is unable to determine if the directory even exists.", dir_msg);
        };

        if !root_dir_exist {
            match Directory::create(&config.path_hint, DIR_PERMISSIONS) {
                Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
                Err(e) => {
                    fail!(from origin, with ListenerCreateError::RootDirectoryCreationFailure,
                        "{} due to a failure while creating the service root directory ({:?}).", dir_msg, e);
                }
            }
            trace!(from origin, "Created service root directory \"{}\" since it did not exist before.", config.path_hint);
        }

        // create named Pipe
        let full_path = config.path_for(name);
        let receiver = match NamedPipeBuilder::new(&full_path)
            .has_ownership(true)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(PIPE_PERMISSIONS)
            .create()
        {
            Ok(v) => v,
            Err(NamedPipeCreationError::CreateFiFoError(
                CreateFiFoError::InsufficientPermissions,
            )) => {
                fail!(from origin, with ListenerCreateError::InsufficientPermissions,
                    "{msg} with {config:?} due to insufficient permissions.");
            }
            Err(NamedPipeCreationError::CreateFiFoError(CreateFiFoError::AlreadyExist)) => {
                fail!(from origin, with ListenerCreateError::AlreadyExists,
                    "{msg} with {config:?} since it already exists.");
            }
            Err(NamedPipeCreationError::NamedPipeOpenFdError(
                NamedPipeOpenFdError::InterruptedBySignal,
            )) => {
                fail!(from origin, with ListenerCreateError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from origin, with ListenerCreateError::InternalFailure,
                    "{msg} with {config:?} due to an internal error. [{e:?}]");
            }
        };

        Ok(Self {
            name: *name,
            receiver,
            clock,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        let msg = "Unable to try wait for a notification";
        let mut buffer = [0u8; READ_BUFFER_SIZE as usize];
        match self.receiver.try_read(&mut buffer) {
            Ok(READ_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_) => Ok(()),
            Err(NamedPipeReadError::InterruptedBySignal) => {
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
        now: TimePoint,
    ) -> Result<(), ListenerWaitError> {
        let msg = "Unable to wait with a timeout for a notification";
        let mut buffer = [0u8; READ_BUFFER_SIZE as usize];

        match self.clock.schedule_on(Timeout::By(timeout), &self.name) {
            Ok(ScheduleOnHandle::TranslatedTimeout(t)) => {
                let translated_timeout = t.as_duration(now);
                match self.receiver.timed_read(&mut buffer, translated_timeout) {
                    Ok(READ_BUFFER_SIZE) => self.empty_buffer(),
                    Ok(_) => Ok(()),
                    Err(NamedPipeReadError::InterruptedBySignal) => {
                        fail!(from self, with ListenerWaitError::InterruptSignal,
                            "{msg} since an interrupt signal was raised.");
                    }
                    Err(e) => {
                        fail!(from self, with ListenerWaitError::InternalFailure,
                            "{msg} due to an internal failure. [{e:?}]");
                    }
                }
            }
            Ok(ScheduleOnHandle::Guard(_guard)) => self.blocking_wait(),
            Err(ClockListenerScheduleError::ClockScheduleCapacityExceeded) => {
                fail!(from self, with ListenerWaitError::ClockScheduleCapacityExceeded,
                    "{msg} since the clock schedule is full.");
            }
            Err(ClockListenerScheduleError::NotificationFailed) => {
                fail!(from self, with ListenerWaitError::NotificationError,
                    "{msg} since the notification failed.");
            }
            Err(ClockListenerScheduleError::InternalError) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal error.");
            }
        }
    }

    fn blocking_wait(&self) -> Result<(), crate::event::ListenerWaitError> {
        let msg = "Unable to blocking wait for a notification";
        let mut buffer = [0u8; READ_BUFFER_SIZE as usize];
        match self.receiver.blocking_read(&mut buffer) {
            Ok(READ_BUFFER_SIZE) => self.empty_buffer(),
            Ok(_) => Ok(()),
            Err(NamedPipeReadError::InterruptedBySignal) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was raised.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure. [{e:?}]");
            }
        }
    }

    fn clock(&self) -> &dyn crate::clock::ClockNow {
        &*self.clock
    }
}

#[allow(type_alias_bounds)] // they are not enforced, but we keep them to communicate the contract
pub type GenericNamedPipeTrigger<E: EventState, Storage: DynamicStorage<State<E, ()>>> =
    EventImpl<E, (), Storage, NamedPipeHandle<E, Storage>, NamedPipeWaiter<E, Storage>>;
