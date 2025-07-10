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

//! **Non inter-process capable** [`CommunicationChannel`] which can be used only in a
//! process-local context.

use crate::communication_channel::*;
use crate::static_storage::file::NamedConceptConfiguration;

use alloc::sync::Arc;
use core::fmt::Debug;

use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Management {
    queue: SafelyOverflowingIndexQueue,
    enable_safe_overflow: bool,
}

impl Management {
    fn new(enable_safe_overflow: bool, capacity: usize) -> Self {
        Self {
            queue: SafelyOverflowingIndexQueue::new(capacity),
            enable_safe_overflow,
        }
    }
}

#[derive(Debug)]
struct StorageEntry {
    content: Arc<Management>,
}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, StorageEntry>>> =
    Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_CHANNELS: Lazy<Mutex<HashMap<FilePath, StorageEntry>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_CHANNELS", "Failed to create process global communication channels");
    }

    result.unwrap()
});

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
            prefix: Channel::default_prefix(),
            path_hint: Channel::default_path_hint(),
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
    name: FileName,
    enable_safe_overflow: bool,
    buffer_size: usize,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Creator {
    fn new(channel_name: &FileName) -> Self {
        Self {
            name: channel_name.clone(),
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

    fn create_receiver(self) -> Result<Duplex, CommunicationChannelCreateError> {
        let msg = "Failed to create receiver";

        let mut guard = fail!(from self, when PROCESS_LOCAL_CHANNELS.lock(),
            with CommunicationChannelCreateError::InternalFailure,
            "{} due to a failure while acquiring the lock.", msg);
        let full_name = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_name);
        if entry.is_some() {
            fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                "{} since the channel with the name \"{}\" already exists.", msg, self.name);
        }

        guard.insert(
            full_name.clone(),
            StorageEntry {
                content: Arc::new(Management::new(self.enable_safe_overflow, self.buffer_size)),
            },
        );

        let entry = guard.get_mut(&full_name).unwrap();

        Ok(Duplex::new_owning(
            self.name,
            entry.content.clone(),
            self.config,
        ))
    }
}

#[derive(Debug)]
pub struct Connector {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Connector {
    fn new(channel_name: &FileName) -> Self {
        Self {
            name: channel_name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl CommunicationChannelConnector<u64, Channel> for Connector {
    fn open_sender(self) -> Result<Duplex, CommunicationChannelOpenError> {
        let msg = "Failed to open sender";
        let origin = format!("{self:?}");
        let name = self.name.clone();
        match self.try_open_sender() {
            Err(CommunicationChannelOpenError::DoesNotExist) => {
                fail!(from origin, with CommunicationChannelOpenError::DoesNotExist,
                                "{} since the channel \"{}\" does not exist.", msg, name);
            }
            Ok(v) => Ok(v),
            Err(v) => {
                fail!(from origin, with v,
                                "{} since an unknown failure occurred ({:?}).", msg, v);
            }
        }
    }

    fn try_open_sender(self) -> Result<Duplex, CommunicationChannelOpenError> {
        let msg = "Failed to open sender";

        let mut guard = fail!(from self, when PROCESS_LOCAL_CHANNELS.lock(),
            with CommunicationChannelOpenError::InternalFailure,
            "{} due to a failure while acquiring the lock.", msg);
        let full_name = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_name);
        if entry.is_none() {
            return Err(CommunicationChannelOpenError::DoesNotExist);
        }

        Ok(Duplex::new_non_owning(
            self.name,
            entry.as_ref().unwrap().content.clone(),
            self.config,
        ))
    }
}

#[derive(Debug)]
pub struct Duplex {
    name: FileName,
    management: Arc<Management>,
    config: Configuration,
    pub(crate) has_ownership: bool,
}

impl Drop for Duplex {
    fn drop(&mut self) {
        if self.has_ownership {
            let msg = "Failed to remove";
            let origin = "communication_channel::process_local::Duplex::remove()";

            let mut guard = fatal_panic!(from origin, when PROCESS_LOCAL_CHANNELS.lock(),
            "{} due to a failure while acquiring the lock.", msg);

            let full_name = self.config.path_for(&self.name);
            if guard.remove(&full_name).is_none() {
                fatal_panic!(from origin,
                "{} since the entry was not existing anymore. This should never happen!", msg);
            }
        }
    }
}

impl Duplex {
    fn new(
        name: FileName,
        management: Arc<Management>,
        has_ownership: bool,
        config: Configuration,
    ) -> Self {
        Self {
            name,
            management,
            has_ownership,
            config,
        }
    }

    pub(crate) fn new_owning(
        name: FileName,
        management: Arc<Management>,
        config: Configuration,
    ) -> Self {
        Self::new(name, management, true, config)
    }

    pub(crate) fn new_non_owning(
        name: FileName,
        management: Arc<Management>,
        config: Configuration,
    ) -> Self {
        Self::new(name, management, false, config)
    }
}

impl NamedConcept for Duplex {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl CommunicationChannelSender<u64> for Duplex {
    fn send(&self, data: &u64) -> Result<Option<u64>, CommunicationChannelSendError> {
        let msg = "Unable to send data";
        match self.try_send(data) {
            Err(CommunicationChannelSendError::ReceiverCacheIsFull) => {
                fail!(from self, with CommunicationChannelSendError::ReceiverCacheIsFull,
                "{} since the receiver cache is full.", msg);
            }
            Err(e) => {
                fail!(from self, with e,
                    "{} due to an unknown failure ({:?}).", msg, e);
            }
            Ok(s) => Ok(s),
        }
    }

    fn try_send(&self, data: &u64) -> Result<Option<u64>, CommunicationChannelSendError> {
        if !self.management.enable_safe_overflow && self.management.queue.is_full() {
            return Err(CommunicationChannelSendError::ReceiverCacheIsFull);
        }

        let result = self
            .management
            .queue
            .acquire_producer()
            .unwrap()
            .push(*data);

        Ok(result)
    }
}

impl CommunicationChannelParticipant for Duplex {
    fn does_enable_safe_overflow(&self) -> bool {
        self.management.enable_safe_overflow
    }
}

impl CommunicationChannelReceiver<u64> for Duplex {
    fn buffer_size(&self) -> usize {
        self.management.queue.capacity()
    }

    fn receive(&self) -> Result<Option<u64>, CommunicationChannelReceiveError> {
        Ok(self.management.queue.acquire_consumer().unwrap().pop())
    }
}

#[derive(Debug)]
pub struct Channel {}

impl NamedConceptMgmt for Channel {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let msg = "Unable to check if communication_channel::process_local exists";
        let origin = "communication_channel::process_local::Channel::does_exist_cfg()";

        let guard = fatal_panic!(from origin,
                        when PROCESS_LOCAL_CHANNELS.lock(),
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&cfg.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let msg = "Unable to list all communication_channel::process_local";
        let origin = "communication_channel::process_local::Channel::list_cfg()";

        let guard = fatal_panic!(from origin,
                                 when PROCESS_LOCAL_CHANNELS.lock(),
                                "{} since the lock could not be acquired.", msg);

        let mut result = vec![];
        for storage_name in guard.keys() {
            if let Some(v) = cfg.extract_name_from_path(storage_name) {
                result.push(v);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);
        let msg = "Unable to remove communication_channel::process_local";
        let origin = "communication_channel::process_local::Channel::remove_cfg()";

        let guard = PROCESS_LOCAL_CHANNELS.lock();
        if guard.is_err() {
            fatal_panic!(from origin,
                "{} since the lock could not be acquired.", msg);
        }

        Ok(guard.unwrap().remove(&storage_name).is_some())
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl CommunicationChannel<u64> for Channel {
    type Sender = Duplex;
    type Connector = Connector;
    type Creator = Creator;
    type Receiver = Duplex;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}
