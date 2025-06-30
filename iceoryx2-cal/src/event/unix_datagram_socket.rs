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

use core::mem::MaybeUninit;

pub use crate::event::*;
use crate::static_storage::file::NamedConceptConfiguration;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptorBased, file_descriptor_set::SynchronousMultiplexing,
    unix_datagram_socket::*,
};
pub use iceoryx2_bb_system_types::file_name::FileName;

const MAX_BATCH_SIZE: usize = 512;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            path: EventImpl::default_path_hint(),
            suffix: EventImpl::default_suffix(),
            prefix: EventImpl::default_prefix(),
        }
    }
}

impl NamedConceptConfiguration for Configuration {
    fn prefix(mut self, value: &FileName) -> Self {
        self.prefix = value.clone();
        self
    }

    fn get_prefix(&self) -> &FileName {
        &self.prefix
    }

    fn suffix(mut self, value: &FileName) -> Self {
        self.suffix = value.clone();
        self
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.path = value.clone();
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
        Self::default()
            .prefix(&value.prefix)
            .suffix(&value.suffix)
            .path_hint(&value.path)
    }
}

#[derive(Debug)]
pub struct EventImpl {}

impl NamedConceptMgmt for EventImpl {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        crate::communication_channel::unix_datagram::Channel::<TriggerId>::does_exist_cfg(
            name,
            &(cfg.clone()).into(),
        )
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        crate::communication_channel::unix_datagram::Channel::<TriggerId>::list_cfg(
            &(cfg.clone()).into(),
        )
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        crate::communication_channel::unix_datagram::Channel::<TriggerId>::remove_cfg(
            name,
            &(cfg.clone()).into(),
        )
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        crate::named_concept::remove_path_hint(value)
    }
}

impl crate::event::Event for EventImpl {
    type Notifier = Notifier;
    type Listener = Listener;
    type NotifierBuilder = NotifierBuilder;
    type ListenerBuilder = ListenerBuilder;
}

#[derive(Debug)]
pub struct Notifier {
    sender: UnixDatagramSender,
    name: FileName,
}

impl NamedConcept for Notifier {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl crate::event::Notifier for Notifier {
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError> {
        let msg = "Failed to notify event::unix_datagram_socket::Listener";
        match self.sender.try_send(unsafe {
            core::slice::from_raw_parts(
                (&id as *const TriggerId).cast(),
                core::mem::size_of::<TriggerId>(),
            )
        }) {
            Ok(true) => Ok(()),
            Ok(false) | Err(UnixDatagramSendError::MessagePartiallySend(_)) => {
                fail!(from self, with NotifierNotifyError::FailedToDeliverSignal,
                        "{} since the signal could not be delivered.", msg);
            }
            Err(
                UnixDatagramSendError::ConnectionReset | UnixDatagramSendError::ConnectionRefused,
            ) => {
                fail!(from self, with NotifierNotifyError::Disconnected,
                        "{} since the notifier is no longer connected to the listener.", msg);
            }
            Err(v) => {
                fail!(from self, with NotifierNotifyError::InternalFailure,
                        "{} due to an unknown failure ({:?}).", msg, v);
            }
        }
    }
}

#[derive(Debug)]
pub struct NotifierBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for NotifierBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl crate::event::NotifierBuilder<EventImpl> for NotifierBuilder {
    fn timeout(self, _timeout: Duration) -> Self {
        self
    }

    fn open(self) -> Result<Notifier, NotifierCreateError> {
        let msg = "Failed to open event::unix_datagram_socket::Notifier";

        let full_name = self.config.path_for(&self.name);
        match UnixDatagramSenderBuilder::new(&full_name).create() {
            Ok(sender) => Ok(Notifier {
                sender,
                name: self.name,
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
pub struct Listener {
    receiver: UnixDatagramReceiver,
    name: FileName,
}

impl FileDescriptorBased for Listener {
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        self.receiver.file_descriptor()
    }
}

impl SynchronousMultiplexing for Listener {}

impl NamedConcept for Listener {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl Listener {
    fn wait<F: FnMut(&Self, &mut [u8]) -> Result<u64, UnixDatagramReceiveError>>(
        &self,
        error_msg: &str,
        mut wait_call: F,
    ) -> Result<Option<TriggerId>, ListenerWaitError> {
        let mut id_buffer = MaybeUninit::uninit();
        match wait_call(self, unsafe {
            core::slice::from_raw_parts_mut(
                id_buffer.as_mut_ptr() as *mut u8,
                core::mem::size_of::<TriggerId>(),
            )
        }) {
            Ok(v) => {
                if v == 0 {
                    return Ok(None);
                }

                if v as usize != core::mem::size_of::<TriggerId>() {
                    fail!(from self, with ListenerWaitError::ContractViolation,
                        "{} since the expected amount of received bytes {} does not match the expected amount of bytes {}.",
                        error_msg, v, core::mem::size_of::<TriggerId>());
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

impl crate::event::Listener for Listener {
    const IS_FILE_DESCRIPTOR_BASED: bool = true;

    fn try_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait(
            "Unable to try wait for signal on event::unix_datagram_socket::Listener",
            |this, buffer| this.receiver.try_receive(buffer),
        )
    }

    fn timed_wait_one(
        &self,
        timeout: core::time::Duration,
    ) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait(
           &format!("Unable to wait for signal with timeout {timeout:?} on event::unix_datagram_socket::Listener"),
            |this, buffer| this.receiver.timed_receive(buffer, timeout),
        )
    }

    fn blocking_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait(
            "Unable to blocking wait for signal on event::unix_datagram_socket::Listener",
            |this, buffer| this.receiver.blocking_receive(buffer),
        )
    }

    fn try_wait_all<F: FnMut(TriggerId)>(&self, mut callback: F) -> Result<(), ListenerWaitError> {
        let mut counter = 0;
        while let Some(id) = self.try_wait_one()? {
            callback(id);

            counter += 1;
            if counter == MAX_BATCH_SIZE {
                break;
            }
        }

        Ok(())
    }

    fn timed_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        if let Some(id) = self.timed_wait_one(timeout)? {
            callback(id);
        }
        self.try_wait_all(callback)
    }

    fn blocking_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
    ) -> Result<(), ListenerWaitError> {
        if let Some(id) = self.blocking_wait_one()? {
            callback(id);
        }
        self.try_wait_all(callback)
    }
}

#[derive(Debug)]
pub struct ListenerBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for ListenerBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl crate::event::ListenerBuilder<EventImpl> for ListenerBuilder {
    fn trigger_id_max(self, _id: TriggerId) -> Self {
        self
    }

    fn create(self) -> Result<Listener, ListenerCreateError> {
        let msg = "Failed to create event::unix_datagram_socket::Listener";
        let full_name = self.config.path_for(&self.name);
        match UnixDatagramReceiverBuilder::new(&full_name)
            .creation_mode(CreationMode::CreateExclusive)
            .create()
        {
            Ok(r) => Ok(Listener {
                receiver: r,
                name: self.name,
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
