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

use std::{marker::PhantomData, mem::MaybeUninit};

pub use crate::event::*;
use crate::static_storage::file::NamedConceptConfiguration;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptorBased, file_descriptor_set::SynchronousMultiplexing,
    unix_datagram_socket::*,
};
pub use iceoryx2_bb_system_types::file_name::FileName;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            path: DEFAULT_PATH_HINT,
            suffix: DEFAULT_SUFFIX,
            prefix: DEFAULT_PREFIX,
        }
    }
}

impl NamedConceptConfiguration for Configuration {
    fn prefix(mut self, value: FileName) -> Self {
        self.prefix = value;
        self
    }

    fn get_prefix(&self) -> &FileName {
        &self.prefix
    }

    fn suffix(mut self, value: FileName) -> Self {
        self.suffix = value;
        self
    }

    fn path_hint(mut self, value: Path) -> Self {
        self.path = value;
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path
    }
}

impl From<Configuration> for crate::communication_channel::unix_datagram::Configuration {
    fn from(value: Configuration) -> Self {
        Self::default().suffix(value.suffix).path_hint(value.path)
    }
}

#[derive(Debug)]
pub struct Event<Id: crate::event::TriggerId> {
    _data: PhantomData<Id>,
}

impl<Id: crate::event::TriggerId + Copy> NamedConceptMgmt for Event<Id> {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        crate::communication_channel::unix_datagram::Channel::<Id>::does_exist_cfg(
            name,
            &(*cfg).into(),
        )
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        crate::communication_channel::unix_datagram::Channel::<Id>::list_cfg(&(*cfg).into())
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        crate::communication_channel::unix_datagram::Channel::<Id>::remove_cfg(name, &(*cfg).into())
    }
}

impl<Id: crate::event::TriggerId + Copy> crate::event::Event<Id> for Event<Id> {
    type Notifier = Notifier<Id>;
    type Listener = Listener<Id>;
    type NotifierBuilder = NotifierBuilder<Id>;
    type ListenerBuilder = ListenerBuilder<Id>;
}

#[derive(Debug)]
pub struct Notifier<Id: crate::event::TriggerId + Copy> {
    sender: UnixDatagramSender,
    name: FileName,
    _data: PhantomData<Id>,
}

impl<Id: crate::event::TriggerId + Copy> NamedConcept for Notifier<Id> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<Id: crate::event::TriggerId + Copy> crate::event::Notifier<Id> for Notifier<Id> {
    fn notify(&self, id: Id) -> Result<(), NotifierNotifyError> {
        let msg = "Failed to notify event::unix_datagram_socket::Listener";
        match self.sender.try_send(unsafe {
            core::slice::from_raw_parts((&id as *const Id).cast(), core::mem::size_of::<Id>())
        }) {
            Ok(true) => Ok(()),
            Ok(false) | Err(UnixDatagramSendError::MessagePartiallySend(_)) => {
                fail!(from self, with NotifierNotifyError::FailedToDeliverSignal,
                        "{} since the signal could not be delivered", msg);
            }
            Err(v) => {
                fail!(from self, with NotifierNotifyError::InternalFailure,
                        "{} due to an unknown failure ({:?}).", msg, v);
            }
        }
    }
}

#[derive(Debug)]
pub struct NotifierBuilder<Id: crate::event::TriggerId> {
    name: FileName,
    config: Configuration,
    _data: PhantomData<Id>,
}

impl<Id: crate::event::TriggerId + Copy> NamedConceptBuilder<Event<Id>> for NotifierBuilder<Id> {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: Configuration::default(),
            _data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl<Id: crate::event::TriggerId + Copy> crate::event::NotifierBuilder<Id, Event<Id>>
    for NotifierBuilder<Id>
{
    fn open(self) -> Result<Notifier<Id>, NotifierCreateError> {
        let msg = "Failed to create event::unix_datagram_socket::Notifier";

        let full_name = self.config.path_for(&self.name);
        match UnixDatagramSenderBuilder::new(&full_name).create() {
            Ok(sender) => Ok(Notifier {
                sender,
                name: self.name,
                _data: PhantomData,
            }),
            Err(UnixDatagramSenderCreationError::DoesNotExist) => {
                fail!(from self, with NotifierCreateError::DoesNotExist,
                    "{} since the corresponding listener does not exist.", msg);
            }
            Err(UnixDatagramSenderCreationError::InsufficientPermissions) => {
                fail!(from self, with NotifierCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from self, with NotifierCreateError::InternalFailure,
                    "{} due to an unknown failure ({:?}).", msg, v);
            }
        }
    }
}

#[derive(Debug)]
pub struct Listener<Id: crate::event::TriggerId + Copy> {
    receiver: UnixDatagramReceiver,
    name: FileName,
    _data: PhantomData<Id>,
}

impl<Id: crate::event::TriggerId + Copy> FileDescriptorBased for Listener<Id> {
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        self.receiver.file_descriptor()
    }
}

impl<Id: crate::event::TriggerId + Copy> SynchronousMultiplexing for Listener<Id> {}

impl<Id: crate::event::TriggerId + Copy> NamedConcept for Listener<Id> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<Id: crate::event::TriggerId + Copy> Listener<Id> {
    fn wait<F: FnMut(&Self, &mut [u8]) -> Result<u64, UnixDatagramReceiveError>>(
        &self,
        error_msg: &str,
        mut wait_call: F,
    ) -> Result<Option<Id>, ListenerWaitError> {
        let mut id_buffer = MaybeUninit::uninit();
        match wait_call(self, unsafe {
            core::slice::from_raw_parts_mut(
                id_buffer.as_mut_ptr() as *mut u8,
                core::mem::size_of::<Id>(),
            )
        }) {
            Ok(v) => {
                if v == 0 {
                    return Ok(None);
                }

                if v as usize != core::mem::size_of::<Id>() {
                    fail!(from self, with ListenerWaitError::ContractViolation,
                        "{} since the expected amount of received bytes {} does not match the expected amount of bytes {}.",
                        error_msg, v, core::mem::size_of::<Id>());
                }
                Ok(Some(unsafe { id_buffer.assume_init() }))
            }
            Err(v) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{} due to an unknown failure ({:?}).", error_msg ,v);
            }
        }
    }
}

impl<Id: crate::event::TriggerId + Copy> crate::event::Listener<Id> for Listener<Id> {
    fn try_wait(&self) -> Result<Option<Id>, ListenerWaitError> {
        self.wait(
            "Unable to try wait for signal on event::unix_datagram_socket::Listener",
            |this, buffer| this.receiver.try_receive(buffer),
        )
    }

    fn timed_wait(&self, timeout: std::time::Duration) -> Result<Option<Id>, ListenerWaitError> {
        self.wait(
           &format!("Unable to wait for signal with timeout {:?} on event::unix_datagram_socket::Listener", timeout),
            |this, buffer| this.receiver.timed_receive(buffer, timeout),
        )
    }

    fn blocking_wait(&self) -> Result<Option<Id>, ListenerWaitError> {
        self.wait(
            "Unable to blocking wait for signal on event::unix_datagram_socket::Listener",
            |this, buffer| this.receiver.blocking_receive(buffer),
        )
    }
}

#[derive(Debug)]
pub struct ListenerBuilder<Id: crate::event::TriggerId> {
    name: FileName,
    config: Configuration,
    _data: PhantomData<Id>,
}

impl<Id: crate::event::TriggerId + Copy> NamedConceptBuilder<Event<Id>> for ListenerBuilder<Id> {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: Configuration::default(),
            _data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl<Id: crate::event::TriggerId + Copy> crate::event::ListenerBuilder<Id, Event<Id>>
    for ListenerBuilder<Id>
{
    fn create(self) -> Result<Listener<Id>, ListenerCreateError> {
        let msg = "Failed to create event::unix_datagram_socket::Listener";
        let full_name = self.config.path_for(&self.name);
        match UnixDatagramReceiverBuilder::new(&full_name)
            .creation_mode(CreationMode::CreateExclusive)
            .create()
        {
            Ok(r) => Ok(Listener {
                receiver: r,
                name: self.name,
                _data: PhantomData,
            }),
            Err(UnixDatagramReceiverCreationError::SocketFileAlreadyExists) => {
                fail!(from self, with ListenerCreateError::AlreadyExists,
                            "{} since the underlying socket does not exist.", msg);
            }
            Err(UnixDatagramReceiverCreationError::UnixDatagramCreationError(
                UnixDatagramCreationError::InsufficientPermissions,
            )) => {
                fail!(from self, with ListenerCreateError::InsufficientPermissions,
                            "{} due insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from self, with ListenerCreateError::InternalFailure,
                            "{} due to an unknown failure ({:?}).", msg, v);
            }
        }
    }
}
