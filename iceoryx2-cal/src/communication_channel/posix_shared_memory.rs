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

//! [`CommunicationChannel`] which is able to send and receive only [`usize`] values
//! (**except** [`usize::MAX`]).
//!
//! It uses internally a [`DynamicStorage`] and [`SafelyOverflowingIndexQueue`].
pub use crate::communication_channel::*;

use crate::dynamic_storage::{
    self, DynamicStorage, DynamicStorageBuilder, DynamicStorageCreateError, DynamicStorageOpenError,
};
use crate::named_concept::*;
use iceoryx2_bb_elementary_traits::relocatable_container::*;
use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
use iceoryx2_bb_log::fail;

type SharedMemory = dynamic_storage::posix_shared_memory::Storage<Management>;
type SharedMemoryBuilder<'builder> =
    <SharedMemory as DynamicStorage<Management>>::Builder<'builder>;

#[derive(Debug)]
pub struct Channel {}

impl NamedConceptMgmt for Channel {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        SharedMemory::does_exist_cfg(name, &(cfg.clone()).into())
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        SharedMemory::list_cfg(&(cfg.clone()).into())
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        SharedMemory::remove_cfg(name, &(cfg.clone()).into())
    }

    fn remove_path_hint(_value: &Path) -> Result<(), NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl CommunicationChannel<u64> for Channel {
    type Sender = Sender;
    type Receiver = Receiver;
    type Creator = Creator;
    type Connector = Connector;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: Channel::default_suffix(),
            path_hint: Channel::default_path_hint(),
            prefix: Channel::default_prefix(),
        }
    }
}

impl From<Configuration> for dynamic_storage::posix_shared_memory::Configuration<Management> {
    fn from(value: Configuration) -> Self {
        Self::default()
            .prefix(&value.prefix)
            .suffix(&value.suffix)
            .path_hint(&value.path_hint)
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
        self.path_hint = value.clone();
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

#[derive(Debug)]
pub struct Creator {
    pub(crate) channel_name: FileName,
    enable_safe_overflow: bool,
    buffer_size: usize,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Creator {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: channel_name.clone(),
            enable_safe_overflow: false,
            buffer_size: DEFAULT_RECEIVER_BUFFER_SIZE,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl CommunicationChannelCreator<u64, Channel> for Creator {
    fn enable_safe_overflow(mut self) -> Self {
        self.enable_safe_overflow = true;
        self
    }

    fn buffer_size(mut self, value: usize) -> Self {
        self.buffer_size = value;
        self
    }

    fn create_receiver(self) -> Result<Receiver, CommunicationChannelCreateError> {
        let msg = "Unable to create communication channel";
        let shared_memory = match SharedMemoryBuilder::new(&self.channel_name)
            .config(&self.config.clone().into())
            .supplementary_size(SafelyOverflowingIndexQueue::const_memory_size(
                self.buffer_size,
            ))
            .initializer(|mgmt, allocator| unsafe { mgmt.index_queue.init(allocator).is_ok() })
            .create(Management {
                enable_safe_overflow: self.enable_safe_overflow,
                index_queue: unsafe {
                    RelocatableSafelyOverflowingIndexQueue::new_uninit(self.buffer_size)
                },
            }) {
            Ok(s) => s,
            Err(DynamicStorageCreateError::AlreadyExists) => {
                fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                    "{} since a channel with that name already exists.", msg);
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelCreateError::InternalFailure,
                    "{} due to an internal failure ({:?})", msg, v);
            }
        };

        Ok(Receiver { shared_memory })
    }
}

#[derive(Debug)]
pub struct Connector {
    pub(crate) channel_name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Connector {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: channel_name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl CommunicationChannelConnector<u64, Channel> for Connector {
    fn try_open_sender(self) -> Result<Sender, CommunicationChannelOpenError> {
        let msg = "Unable to try open communication channel";

        match SharedMemoryBuilder::new(&self.channel_name)
            .config(&self.config.clone().into())
            .open()
        {
            Ok(shared_memory) => Ok(Sender { shared_memory }),
            Err(DynamicStorageOpenError::DoesNotExist)
            | Err(DynamicStorageOpenError::InitializationNotYetFinalized) => {
                Err(CommunicationChannelOpenError::DoesNotExist)
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelOpenError::InternalFailure,
                    "{} since an internal failure occurred ({:?}).", msg, v);
            }
        }
    }

    fn open_sender(self) -> Result<Sender, CommunicationChannelOpenError> {
        let msg = "Unable to open communication channel";
        let origin = format!("{self:?}");
        match self.try_open_sender() {
            Ok(s) => Ok(s),
            Err(CommunicationChannelOpenError::DoesNotExist) => {
                fail!(from origin, with CommunicationChannelOpenError::DoesNotExist,
                    "{} since the channel does not exist.", msg);
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Management {
    index_queue: RelocatableSafelyOverflowingIndexQueue,
    enable_safe_overflow: bool,
}

#[derive(Debug)]
pub struct Receiver {
    shared_memory: SharedMemory,
}

impl NamedConcept for Receiver {
    fn name(&self) -> &FileName {
        self.shared_memory.name()
    }
}

impl Receiver {
    fn management(&self) -> &Management {
        self.shared_memory.get()
    }
}

impl CommunicationChannelParticipant for Receiver {
    fn does_enable_safe_overflow(&self) -> bool {
        self.management().enable_safe_overflow
    }
}

impl CommunicationChannelReceiver<u64> for Receiver {
    fn buffer_size(&self) -> usize {
        self.management().index_queue.capacity()
    }

    fn receive(&self) -> Result<Option<u64>, CommunicationChannelReceiveError> {
        Ok(unsafe { self.management().index_queue.pop() })
    }
}

#[derive(Debug)]
pub struct Sender {
    shared_memory: SharedMemory,
}

impl Sender {
    fn management(&self) -> &Management {
        self.shared_memory.get()
    }
}

impl CommunicationChannelParticipant for Sender {
    fn does_enable_safe_overflow(&self) -> bool {
        self.management().enable_safe_overflow
    }
}

impl NamedConcept for Sender {
    fn name(&self) -> &FileName {
        self.shared_memory.name()
    }
}

impl CommunicationChannelSender<u64> for Sender {
    fn send(&self, value: &u64) -> Result<Option<u64>, CommunicationChannelSendError> {
        match self.try_send(value) {
            Err(CommunicationChannelSendError::ReceiverCacheIsFull) => {
                fail!(from self, with CommunicationChannelSendError::ReceiverCacheIsFull,
                    "Unable to send data since the corresponding receiver cache is full.");
            }
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }

    fn try_send(&self, value: &u64) -> Result<Option<u64>, CommunicationChannelSendError> {
        if !self.management().enable_safe_overflow && self.management().index_queue.is_full() {
            return Err(CommunicationChannelSendError::ReceiverCacheIsFull);
        }

        Ok(unsafe { self.management().index_queue.push(*value) })
    }
}
