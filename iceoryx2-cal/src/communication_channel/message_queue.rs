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

//! [CommunicationChannel] based on [`MessageQueueSender`] & [`MessageQueueReceiver`]. Can send and
//! receive data without restrictions.

use std::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::directory::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::{
    message_queue::*,
    shared_memory::{AccessMode, SharedMemory, SharedMemoryBuilder, SharedMemoryCreationError},
};
use iceoryx2_bb_system_types::path::Path;
use std::cell::UnsafeCell;

pub use crate::communication_channel::*;
use crate::static_storage::file::{
    NamedConceptConfiguration, NamedConceptDoesExistError, NamedConceptRemoveError,
};

#[derive(Debug)]
pub struct Channel<T: Copy> {
    _phantom_data: PhantomData<T>,
}

const INIT_PERMISSIONS: Permission = Permission::OWNER_WRITE;

#[cfg(not(feature = "dev_permissions"))]
const FINAL_PERMISSIONS: Permission = Permission::OWNER_ALL;

#[cfg(feature = "dev_permissions")]
const FINAL_PERMISSIONS: Permission = Permission::ALL;

impl<T: Copy + Debug> NamedConceptMgmt for Channel<T> {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let msg = format!(
            "Unable to determine if the communication_channel::message_queue \"{}\" exists",
            name
        );
        let origin = "communication_channel::message_queue::Channel::does_exist_cfg";
        let full_name = cfg.path_for(name).file_name();

        let does_mq_exist = match does_message_queue_exist::<T>(&full_name) {
            Ok(true) => true,
            Ok(false) => false,
            Err(MessageQueueOpenError::PermissionDenied) => {
                fail!(from origin, with NamedConceptDoesExistError::InsufficientPermissions,
                                "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptDoesExistError::InternalError,
                                "{} due to an internal failure ({:?}).", msg, v);
            }
        };

        let does_shm_exist = SharedMemory::does_exist(&full_name);

        if does_shm_exist != does_mq_exist {
            fail!(from origin, with NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp,
                        "{} since the underlying resources seems to be currently set up.", msg);
        }

        Ok(does_shm_exist)
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let entries = SharedMemory::list();

        let mut result = vec![];
        for entry in &entries {
            if let Some(entry_name) = cfg.extract_name_from_file(entry) {
                if does_message_queue_exist::<T>(entry) == Ok(true) {
                    result.push(entry_name);
                }
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let full_name = cfg.path_for(name).file_name();
        let msg = "Unable to remove communication_channel::message_queue";
        let origin = "communication_channel::message_queue::Channel::remove_cfg()";

        match remove_message_queue(&full_name) {
            Ok(()) => (),
            Err(MessageQueueRemoveError::DoesNotExist) => (),
            Err(v) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                            "{} \"{}\" due to an internal failure while removing the message queue ({:?}).", msg, name, v);
            }
        };

        match SharedMemory::remove(&full_name) {
            Ok(v) => Ok(v),
            Err(
                iceoryx2_bb_posix::shared_memory::SharedMemoryRemoveError::InsufficientPermissions,
            ) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                            "{} \"{}\" due to insufficient permissions while accessing the shared memory.", msg, name);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                            "{} \"{}\" due to an internal failure while removing the shared memory ({:?}).", msg, name, v);
            }
        }
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl<T: Copy + Debug> CommunicationChannel<T> for Channel<T> {
    type Sender = Sender<T>;
    type Receiver = Receiver<T>;
    type Creator = Creator<T>;
    type Connector = Connector<T>;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: Channel::<()>::default_suffix(),
            prefix: Channel::<()>::default_prefix(),
            path_hint: Channel::<()>::default_path_hint(),
        }
    }
}

impl NamedConceptConfiguration for Configuration {
    fn prefix(mut self, value: &FileName) -> Self {
        self.prefix = *value;
        self
    }

    fn get_prefix(&self) -> &FileName {
        &self.prefix
    }

    fn suffix(mut self, value: &FileName) -> Self {
        self.suffix = *value;
        self
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.path_hint = *value;
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

#[repr(C)]
struct SharedConfiguration {
    has_safe_overflow: bool,
    buffer_size: usize,
}

#[derive(Debug)]
pub struct Creator<T: Copy + Debug> {
    channel_name: FileName,
    enable_safe_overflow: bool,
    buffer_size: usize,
    config: Configuration,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> NamedConceptBuilder<Channel<T>> for Creator<T> {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: *channel_name,
            enable_safe_overflow: false,
            buffer_size: DEFAULT_RECEIVER_BUFFER_SIZE,
            config: Configuration::default(),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl<T: Copy + Debug> CommunicationChannelCreator<T, Channel<T>> for Creator<T> {
    fn enable_safe_overflow(mut self) -> Self {
        self.enable_safe_overflow = true;
        self
    }

    fn buffer_size(mut self, value: usize) -> Self {
        self.buffer_size = value;
        self
    }

    fn create_receiver(self) -> Result<Receiver<T>, CommunicationChannelCreateError> {
        let msg = "Unable to create receiver";
        let full_name = self.config.path_for(&self.channel_name).file_name();

        // create the message queue channel first to avoid races when open is called and the receiver
        // is created at the same time. The uds is called as second in create_receiver and
        // it cannot exist when not everything is already set up.
        let receiver = MessageQueueBuilder::new(&full_name)
            .capacity(self.buffer_size)
            .create_receiver(CreationMode::CreateExclusive);

        let receiver = match receiver {
            Ok(r) => r,
            Err(MessageQueueCreationError::AlreadyExist) => {
                fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                    "{} since a channel with that name already exists.", msg);
            }
            _ => {
                fail!(from self, with CommunicationChannelCreateError::InternalFailure,
                    "{} since the underlying socket could not be created.", msg);
            }
        };

        let mut _shared_memory = match SharedMemoryBuilder::new(&full_name)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(INIT_PERMISSIONS)
            .size(std::mem::size_of::<SharedConfiguration>())
            .create()
        {
            Ok(s) => s,
            Err(SharedMemoryCreationError::AlreadyExist) => {
                fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                    "{} since the shared management part of the channel with that name already exists.", msg);
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelCreateError::InternalFailure,
                    "{} due to an internal failure while creating the shared management part of the channel ({:?}).", msg, v);
            }
        };

        let shared_config_ptr = _shared_memory.base_address().as_ptr() as *mut SharedConfiguration;
        unsafe {
            shared_config_ptr.write(SharedConfiguration {
                has_safe_overflow: self.enable_safe_overflow,
                buffer_size: self.buffer_size,
            })
        };

        // we are finished with the setup and we open the channel for others to connect
        _shared_memory.set_permission(FINAL_PERMISSIONS).unwrap();

        Ok(Receiver {
            name: self.channel_name,
            receiver: UnsafeCell::new(receiver),
            _shared_memory,
            shared_config_ptr,
        })
    }
}

#[derive(Debug)]
pub struct Connector<T: Copy + Debug> {
    channel_name: FileName,
    config: Configuration,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> NamedConceptBuilder<Channel<T>> for Connector<T> {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: *channel_name,
            config: Configuration::default(),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl<T: Copy + Debug> CommunicationChannelConnector<T, Channel<T>> for Connector<T> {
    fn open_sender(self) -> Result<Sender<T>, CommunicationChannelOpenError> {
        let msg = "Unable to create sender";
        let origin = format!("{:?}", self);
        match self.try_open_sender() {
            Ok(s) => Ok(s),
            Err(CommunicationChannelOpenError::DoesNotExist) => {
                fail!(from origin, with CommunicationChannelOpenError::DoesNotExist,
                    "{} since the receiver does not exist.", msg);
            }
            Err(v) => Err(v),
        }
    }

    fn try_open_sender(self) -> Result<Sender<T>, CommunicationChannelOpenError> {
        let msg = "Unable to create sender";
        let full_name = self.config.path_for(&self.channel_name).file_name();

        let _shared_memory = match SharedMemoryBuilder::new(&full_name)
            .open_existing(AccessMode::Read)
        {
            Ok(s) => s,
            Err(SharedMemoryCreationError::DoesNotExist) => {
                return Err(CommunicationChannelOpenError::DoesNotExist);
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelOpenError::InternalFailure,
                    "{} due to an internal failure while creating the shared management part of the channel ({:?}).", msg, v);
            }
        };

        let shared_config_ptr = _shared_memory.base_address().as_ptr() as *mut SharedConfiguration;

        // open the message queue channel second to avoid race conditions when the channel is
        // opened while it is being created
        let sender = MessageQueueBuilder::new(&full_name)
            .capacity(unsafe { (*shared_config_ptr).buffer_size })
            .open_duplex();

        match &sender {
            Ok(_) => (),
            Err(MessageQueueOpenError::DoesNotExist) => {
                return Err(CommunicationChannelOpenError::DoesNotExist);
            }
            _ => {
                fail!(from self, with CommunicationChannelOpenError::InternalFailure,
                    "{} since a connection to the underlying socket could not be established.", msg);
            }
        };

        Ok(Sender {
            name: self.channel_name,
            sender: UnsafeCell::new(sender.unwrap()),
            _shared_memory,
            shared_config_ptr,
        })
    }
}

#[derive(Debug)]
pub struct Sender<T: Debug> {
    name: FileName,
    sender: UnsafeCell<MessageQueueDuplex<T>>,
    _shared_memory: SharedMemory,
    shared_config_ptr: *mut SharedConfiguration,
}

impl<T: Copy + Debug> Sender<T> {
    fn shared_config(&self) -> &SharedConfiguration {
        unsafe { &*self.shared_config_ptr }
    }

    #[allow(clippy::mut_from_ref)]
    fn sender(&self) -> &mut MessageQueueDuplex<T> {
        unsafe { &mut *self.sender.get() }
    }
}

impl<T: Copy + Debug> CommunicationChannelParticipant for Sender<T> {
    fn does_enable_safe_overflow(&self) -> bool {
        self.shared_config().has_safe_overflow
    }
}

impl<T: Copy + Debug> NamedConcept for Sender<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Copy + Debug> CommunicationChannelSender<T> for Sender<T> {
    fn send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError> {
        let msg = "Unable to send data";
        match self.try_send(data) {
            Err(CommunicationChannelSendError::ReceiverCacheIsFull) => {
                fail!(from self, with CommunicationChannelSendError::ReceiverCacheIsFull,
                    "{} since the receiver cache is full.", msg);
            }
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }

    fn try_send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError> {
        let msg = "Unable to try send data";

        let mut oldest_sample = None;
        loop {
            let result = self.sender().try_send(data);

            match result {
                Ok(true) => {
                    return Ok(oldest_sample);
                }
                Ok(false) | Err(MessageQueueSendError::QueueIsFull) => {
                    if self.shared_config().has_safe_overflow {
                        oldest_sample = Some(self.sender().try_receive().unwrap().unwrap().value);
                    } else {
                        return Err(CommunicationChannelSendError::ReceiverCacheIsFull);
                    }
                }
                Err(v) => {
                    fail!(from self, with CommunicationChannelSendError::InternalFailure,
                    "{} due to an internal failure ({:?}).", msg, v);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Receiver<T: Debug> {
    name: FileName,
    receiver: UnsafeCell<MessageQueueReceiver<T>>,
    _shared_memory: SharedMemory,
    shared_config_ptr: *mut SharedConfiguration,
}

impl<T: Debug> Receiver<T> {
    #[allow(clippy::mut_from_ref)]
    fn receiver(&self) -> &mut MessageQueueReceiver<T> {
        unsafe { &mut *self.receiver.get() }
    }
}

impl<T: Copy + Debug> CommunicationChannelParticipant for Receiver<T> {
    fn does_enable_safe_overflow(&self) -> bool {
        unsafe { (*self.shared_config_ptr).has_safe_overflow }
    }
}

impl<T: Copy + Debug> NamedConcept for Receiver<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Copy + Debug> CommunicationChannelReceiver<T> for Receiver<T> {
    fn buffer_size(&self) -> usize {
        self.receiver().capacity()
    }

    fn receive(&self) -> Result<Option<T>, CommunicationChannelReceiveError> {
        let msg = "Unable to receive data";
        match self.receiver().try_receive() {
            Ok(Some(v)) => Ok(Some(v.value)),
            Ok(None) => Ok(None),
            Err(e) => {
                fail!(from self, with CommunicationChannelReceiveError::InternalFailure,
                    "{} due to some internal failure ({:?}).", msg, e);
            }
        }
    }
}
