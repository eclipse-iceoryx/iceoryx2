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

use crate::named_concept::*;
pub use crate::zero_copy_connection::*;
use iceoryx2_bb_lock_free::spsc::{
    index_queue::IndexQueue, safely_overflowing_index_queue::SafelyOverflowingIndexQueue,
};
use iceoryx2_bb_log::{error, fail, fatal_panic};
use iceoryx2_bb_posix::{
    adaptive_wait::AdaptiveWaitBuilder,
    mutex::{Mutex, MutexBuilder, MutexHandle},
};
use once_cell::sync::Lazy;
use std::{
    cell::UnsafeCell,
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum State {
    Sender = 0b00000001,
    Receiver = 0b00000010,
}

#[derive(Debug)]
struct Management {
    name: FileName,
    receive_channel: SafelyOverflowingIndexQueue,
    retrieve_channel: IndexQueue,
    enable_safe_overflow: bool,
    max_borrowed_samples: usize,
    state: AtomicU8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: DEFAULT_SUFFIX,
            prefix: DEFAULT_PREFIX,
            path_hint: DEFAULT_PATH_HINT,
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
        self.path_hint = value;
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, Arc<Management>>>> =
    Lazy::new(MutexHandle::<HashMap<FilePath, Arc<Management>>>::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashMap<FilePath, Arc<Management>>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global dynamic zero copy connection");
    }

    result.unwrap()
});

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    buffer_size: usize,
    enable_safe_overflow: bool,
    max_borrowed_samples: usize,
    config: Configuration,
}

impl Builder {
    fn check_compatibility(
        &self,
        entry: &Management,
        msg: &str,
    ) -> Result<(), ZeroCopyCreationError> {
        if entry.enable_safe_overflow != self.enable_safe_overflow {
            fail!(from self, with ZeroCopyCreationError::IncompatibleOverflowSetting,
                        "{} since the overflow setting is not compatible.", msg);
        }

        if entry.max_borrowed_samples != self.max_borrowed_samples {
            fail!(from self, with ZeroCopyCreationError::IncompatibleMaxBorrowedSampleSetting,
                        "{} since the max borrow setting is not compatible.", msg);
        }

        if entry.receive_channel.capacity() != self.buffer_size {
            fail!(from self, with ZeroCopyCreationError::IncompatibleBufferSize,
                        "{} since the buffer size is not compatible.", msg);
        }

        Ok(())
    }

    fn retrieve_channel_size(&self) -> usize {
        self.buffer_size + self.max_borrowed_samples + 1
    }
}

impl NamedConceptBuilder<Connection> for Builder {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            buffer_size: DEFAULT_BUFFER_SIZE,
            enable_safe_overflow: DEFAULT_ENABLE_SAFE_OVERFLOW,
            max_borrowed_samples: DEFAULT_MAX_BORROWED_SAMPLES,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl ZeroCopyConnectionBuilder<Connection> for Builder {
    fn buffer_size(mut self, value: usize) -> Self {
        self.buffer_size = value;
        self
    }

    fn enable_safe_overflow(mut self, value: bool) -> Self {
        self.enable_safe_overflow = value;
        self
    }

    fn receiver_max_borrowed_samples(mut self, value: usize) -> Self {
        self.max_borrowed_samples = value;
        self
    }

    fn create_sender(self) -> Result<Sender, ZeroCopyCreationError> {
        let msg = "Unable to create sender";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with ZeroCopyCreationError::InternalError,
            "{} due to a failure while acquiring the lock to the global zero copy connections.", msg);

        let full_path = self.config.path_for(&self.name);
        match guard.get_mut(&full_path) {
            Some(entry) => {
                let state = entry.state.load(Ordering::Relaxed);
                if state & State::Sender as u8 != 0 {
                    fail!(from self, with ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected,
                        "{} since a sender is already connected.", msg);
                }

                self.check_compatibility(entry, msg)?;

                entry
                    .state
                    .store(state | State::Sender as u8, Ordering::Relaxed);

                Ok(Sender {
                    mgmt: entry.clone(),
                    config: self.config,
                })
            }
            None => {
                let entry = Arc::new(Management {
                    name: self.name,
                    receive_channel: SafelyOverflowingIndexQueue::new(self.buffer_size),
                    retrieve_channel: IndexQueue::new(self.retrieve_channel_size()),
                    enable_safe_overflow: self.enable_safe_overflow,
                    max_borrowed_samples: self.max_borrowed_samples,
                    state: AtomicU8::new(State::Sender as u8),
                });
                guard.insert(full_path, entry.clone());

                Ok(Sender {
                    mgmt: entry,
                    config: self.config,
                })
            }
        }
    }

    fn create_receiver(self) -> Result<Receiver, ZeroCopyCreationError> {
        let msg = "Unable to create receiver";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with ZeroCopyCreationError::InternalError,
            "{} due to a failure while acquiring the lock to the global zero copy connections.", msg);

        let full_path = self.config.path_for(&self.name);
        match guard.get_mut(&full_path) {
            Some(entry) => {
                let state = entry.state.load(Ordering::Relaxed);
                if state & State::Receiver as u8 != 0 {
                    fail!(from self, with ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected,
                        "{} since a receiver is already connected.", msg);
                }

                self.check_compatibility(entry, msg)?;

                entry
                    .state
                    .store(state | State::Receiver as u8, Ordering::Relaxed);

                Ok(Receiver {
                    mgmt: entry.clone(),
                    borrow_counter: UnsafeCell::new(0),
                    config: self.config,
                })
            }
            None => {
                let entry = Arc::new(Management {
                    name: self.name,
                    receive_channel: SafelyOverflowingIndexQueue::new(self.buffer_size),
                    retrieve_channel: IndexQueue::new(self.retrieve_channel_size()),
                    enable_safe_overflow: self.enable_safe_overflow,
                    max_borrowed_samples: self.max_borrowed_samples,
                    state: AtomicU8::new(State::Receiver as u8),
                });
                guard.insert(full_path, entry.clone());

                Ok(Receiver {
                    mgmt: entry,
                    borrow_counter: UnsafeCell::new(0),
                    config: self.config,
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct Sender {
    mgmt: Arc<Management>,
    config: Configuration,
}

impl Drop for Sender {
    fn drop(&mut self) {
        cleanup_connection(self, &self.mgmt.name, &self.config, State::Sender);
    }
}

impl NamedConcept for Sender {
    fn name(&self) -> &FileName {
        &self.mgmt.name
    }
}

impl ZeroCopyPortDetails for Sender {
    fn buffer_size(&self) -> usize {
        self.mgmt.receive_channel.capacity()
    }

    fn max_borrowed_samples(&self) -> usize {
        self.mgmt.max_borrowed_samples
    }

    fn has_enabled_safe_overflow(&self) -> bool {
        self.mgmt.enable_safe_overflow
    }

    fn is_connected(&self) -> bool {
        let state = self.mgmt.state.load(Ordering::Relaxed);
        (state & State::Sender as u8) != 0 && (state & State::Receiver as u8) != 0
    }
}

impl ZeroCopySender for Sender {
    fn try_send(&self, ptr: PointerOffset) -> Result<Option<PointerOffset>, ZeroCopySendError> {
        let msg = "Unable to send sample";
        let space_in_retrieve_channel =
            self.mgmt.retrieve_channel.capacity() - self.mgmt.retrieve_channel.len();

        if space_in_retrieve_channel
            <= self.mgmt.max_borrowed_samples + self.mgmt.receive_channel.len()
        {
            fail!(from self, with ZeroCopySendError::ClearRetrieveChannelBeforeSend,
                "{} since sufficient space for every sample in the retrieve channel cannot be guaranteed. Samples have to be retrieved before a new sample can be send.", msg);
        }

        if !self.mgmt.enable_safe_overflow && self.mgmt.receive_channel.is_full() {
            fail!(from self, with ZeroCopySendError::ReceiveBufferFull,
                        "{} since the receive buffer is full.", msg);
        }

        match unsafe { self.mgmt.receive_channel.push(ptr.value()) } {
            Some(v) => Ok(Some(PointerOffset::new(v))),
            None => Ok(None),
        }
    }

    fn blocking_send(
        &self,
        ptr: PointerOffset,
    ) -> Result<Option<PointerOffset>, ZeroCopySendError> {
        if !self.mgmt.enable_safe_overflow {
            while self.mgmt.receive_channel.is_full() {
                AdaptiveWaitBuilder::new()
                    .create()
                    .unwrap()
                    .wait_while(|| self.mgmt.receive_channel.is_full())
                    .unwrap();
            }
        }

        self.try_send(ptr)
    }

    fn reclaim(&self) -> Result<Option<PointerOffset>, ZeroCopyReclaimError> {
        match unsafe { self.mgmt.retrieve_channel.pop() } {
            None => Ok(None),
            Some(v) => Ok(Some(PointerOffset::new(v))),
        }
    }
}

#[derive(Debug)]
pub struct Receiver {
    mgmt: Arc<Management>,
    borrow_counter: UnsafeCell<usize>,
    config: Configuration,
}

fn cleanup_connection<T: Debug>(origin: &T, name: &FileName, config: &Configuration, state: State) {
    let msg = "Unable to cleanup port";

    let mut guard = match PROCESS_LOCAL_STORAGE.lock() {
        Ok(g) => g,
        Err(_) => {
            error!(from origin, "{} due to a failure while acquiring the lock to the global zero copy connections.", msg);
            return;
        }
    };

    let full_path = config.path_for(name);
    match guard.get(&full_path) {
        Some(v) => {
            let current_state = v.state.load(Ordering::Relaxed);
            v.state
                .store(current_state & !(state as u8), Ordering::Relaxed);

            if current_state == state as u8 {
                guard.remove(&full_path);
            }
        }
        None => {
            fatal_panic!(from origin, "This should never happen! Connection was removed when it was still active.");
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        cleanup_connection(self, &self.mgmt.name, &self.config, State::Receiver);
    }
}

impl Receiver {
    #[allow(clippy::mut_from_ref)]
    // convenience to access internal mutable object
    fn borrow_counter(&self) -> &mut usize {
        #[deny(clippy::mut_from_ref)]
        unsafe {
            &mut *self.borrow_counter.get()
        }
    }
}

impl NamedConcept for Receiver {
    fn name(&self) -> &FileName {
        &self.mgmt.name
    }
}

impl ZeroCopyPortDetails for Receiver {
    fn buffer_size(&self) -> usize {
        self.mgmt.receive_channel.capacity()
    }

    fn max_borrowed_samples(&self) -> usize {
        self.mgmt.max_borrowed_samples
    }

    fn has_enabled_safe_overflow(&self) -> bool {
        self.mgmt.enable_safe_overflow
    }

    fn is_connected(&self) -> bool {
        let state = self.mgmt.state.load(Ordering::Relaxed);
        (state & State::Sender as u8) != 0 && (state & State::Receiver as u8) != 0
    }
}

impl ZeroCopyReceiver for Receiver {
    fn receive(
        &self,
    ) -> Result<Option<crate::shared_memory::PointerOffset>, super::ZeroCopyReceiveError> {
        if *self.borrow_counter() >= self.mgmt.max_borrowed_samples {
            fail!(from self, with ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue,
                "Unable to receive another sample since this would exceed the max borrow value.");
        }

        match unsafe { self.mgmt.receive_channel.pop() } {
            None => Ok(None),
            Some(v) => {
                *self.borrow_counter() += 1;
                Ok(Some(PointerOffset::new(v)))
            }
        }
    }

    fn release(
        &self,
        ptr: crate::shared_memory::PointerOffset,
    ) -> Result<(), super::ZeroCopyReleaseError> {
        match unsafe { self.mgmt.retrieve_channel.push(ptr.value()) } {
            true => {
                *self.borrow_counter() -= 1;
                Ok(())
            }
            false => {
                fail!(from self, with ZeroCopyReleaseError::RetrieveBufferFull,
                    "Unable to release pointer since the retrieve buffer is full.");
            }
        }
    }
}

pub struct Connection {}

impl NamedConceptMgmt for Connection {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        let msg = "Unable to determine if connection exists";
        let origin = "ZeroCopyConnection::does_exist()";
        let guard = match PROCESS_LOCAL_STORAGE.lock() {
            Ok(g) => g,
            Err(_) => {
                error!(from origin, "{} due to a failure while acquiring the lock to the global zero copy connections.", msg);
                return Err(NamedConceptDoesExistError::InternalError);
            }
        };

        Ok(guard.get(&cfg.path_for(name)).is_some())
    }

    fn list_cfg(config: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let msg = "Unable to list all zero_copy_connection::process_local";
        let origin = "zero_copy_connection::process_local::Connection::list_cfg()";

        let guard = fatal_panic!(from origin,
                                 when PROCESS_LOCAL_STORAGE.lock(),
                                "{} since the lock could not be acquired.", msg);

        let mut result = vec![];
        for storage_name in guard.keys() {
            if let Some(v) = config.extract_name_from_path(storage_name) {
                result.push(v);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);

        let msg = "Unable to remove shared memory";
        let guard = PROCESS_LOCAL_STORAGE.lock();
        if guard.is_err() {
            fatal_panic!(from "zero_copy_connection::process_local::Connection::remove_cfg()",
                    "{} since the lock could not be acquired.", msg);
        }

        Ok(guard.unwrap().remove(&storage_name).is_some())
    }
}

impl ZeroCopyConnection for Connection {
    type Sender = Sender;
    type Receiver = Receiver;
    type Builder = Builder;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}
