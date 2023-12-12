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

use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::Duration;

use crate::named_concept::*;
pub use crate::zero_copy_connection::*;
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::spsc::{
    index_queue::RelocatableIndexQueue,
    safely_overflowing_index_queue::RelocatableSafelyOverflowingIndexQueue,
};
use iceoryx2_bb_log::{error, fail, fatal_panic};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::permission::Permission;
use iceoryx2_bb_posix::shared_memory::{SharedMemory, SharedMemoryBuilder};

const MAX_CREATION_DURATION: Duration = Duration::from_millis(10);
const IS_INITIALIZED_STATE_VALUE: u64 = 0xbeefaffedeadbeef;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum State {
    None = 0b00000000,
    Sender = 0b00000001,
    Receiver = 0b00000010,
    MarkedForDestruction = 0b10000000,
}

impl State {
    fn value(&self) -> u8 {
        *self as u8
    }
}

fn cleanup_shared_memory<T: Debug>(
    origin: &T,
    shared_memory: &SharedMemory,
    state_to_remove: State,
) {
    let mgmt_ref =
        unsafe { &*(shared_memory.base_address().as_ptr() as *const SharedManagementData) };

    let mut current_state = mgmt_ref.state.load(Ordering::Relaxed);
    loop {
        let new_state = if current_state == state_to_remove.value() {
            State::MarkedForDestruction.value()
        } else {
            current_state & !state_to_remove.value()
        };

        match mgmt_ref.state.compare_exchange(
            current_state,
            new_state,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                current_state = new_state;
                break;
            }
            Err(s) => {
                current_state = s;
            }
        }
    }

    if current_state == State::MarkedForDestruction.value()
        && SharedMemory::remove(shared_memory.name()).is_err()
    {
        error!(from origin, "Failed to remove shared memory when cleaning up the connection.");
    }
}

#[repr(C)]
struct SharedManagementData {
    receive_channel: RelocatableSafelyOverflowingIndexQueue,
    retrieve_channel: RelocatableIndexQueue,
    max_borrowed_samples: usize,
    state: AtomicU8,
    init_state: AtomicU64,
    enable_safe_overflow: bool,
}

impl SharedManagementData {
    fn new(
        receive_channel_buffer_size: usize,
        retrieve_channel_buffer_size: usize,
        enable_safe_overflow: bool,
        max_borrowed_samples: usize,
    ) -> Self {
        Self {
            receive_channel: unsafe {
                RelocatableSafelyOverflowingIndexQueue::new_uninit(receive_channel_buffer_size)
            },
            retrieve_channel: unsafe {
                RelocatableIndexQueue::new_uninit(retrieve_channel_buffer_size)
            },
            state: AtomicU8::new(State::None.value()),
            init_state: AtomicU64::new(0),
            enable_safe_overflow,
            max_borrowed_samples,
        }
    }

    const fn const_memory_size(
        receive_channel_buffer_size: usize,
        retrieve_channel_buffer_size: usize,
    ) -> usize {
        // we do not have to consider the alignment of Self since posix shared memory is always
        // page size aligned
        std::mem::size_of::<Self>()
            + RelocatableIndexQueue::const_memory_size(retrieve_channel_buffer_size)
            + RelocatableSafelyOverflowingIndexQueue::const_memory_size(receive_channel_buffer_size)
    }
}

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    buffer_size: usize,
    enable_safe_overflow: bool,
    max_borrowed_samples: usize,
    config: Configuration,
}

impl Builder {
    fn receive_channel_size(&self) -> usize {
        self.buffer_size
    }

    fn retrieve_channel_size(&self) -> usize {
        self.buffer_size + self.max_borrowed_samples + 1
    }

    fn create_or_open_shm(&self) -> Result<SharedMemory, ZeroCopyCreationError> {
        let shm_size = SharedManagementData::const_memory_size(
            self.receive_channel_size(),
            self.retrieve_channel_size(),
        );

        let msg = "Failed to acquire underlying shared memory";
        let full_name =
            unsafe { FileName::new_unchecked(self.config.path_for(&self.name).file_name()) };
        let mut shm = fail!(from self, when SharedMemoryBuilder::new(&full_name)
                                                .creation_mode(CreationMode::OpenOrCreate)
                                                .size(shm_size)
                                                .permission(Permission::OWNER_ALL)
                                                .create(),
                                       with ZeroCopyCreationError::InternalError,
                            "{} since it could not be opened/created. This can be caused by incompatible builder settings.", msg);

        let mgmt_ptr = shm.base_address().as_ptr() as *mut SharedManagementData;
        match shm.has_ownership() {
            true => {
                let msg = "Failed to set up newly created connection";
                unsafe {
                    mgmt_ptr.write(SharedManagementData::new(
                        self.receive_channel_size(),
                        self.retrieve_channel_size(),
                        self.enable_safe_overflow,
                        self.max_borrowed_samples,
                    ))
                };

                let supplementary_ptr =
                    (mgmt_ptr as usize + std::mem::size_of::<SharedManagementData>()) as *mut u8;
                let supplementary_len = shm_size - std::mem::size_of::<SharedManagementData>();

                let allocator = BumpAllocator::new(
                    unsafe { NonNull::new_unchecked(supplementary_ptr) },
                    supplementary_len,
                );

                fatal_panic!(from self, when unsafe { (*mgmt_ptr).receive_channel.init(&allocator) },
                            "{} since the receive channel allocation failed. - This is an implementation bug!", msg);
                fatal_panic!(from self, when unsafe { (*mgmt_ptr).retrieve_channel.init(&allocator) },
                            "{} since the retrieve channel allocation failed. - This is an implementation bug!", msg);

                unsafe {
                    (*mgmt_ptr)
                        .init_state
                        .store(IS_INITIALIZED_STATE_VALUE, Ordering::Relaxed)
                };
                shm.release_ownership();
            }
            false => {
                let msg = "Failed to open existing connection";

                let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new().create(),
                                            with ZeroCopyCreationError::InternalError, "{} since the adaptive wait could not be created.", msg);

                let mgmt_ref = unsafe { &mut *mgmt_ptr };
                while mgmt_ref.init_state.load(Ordering::Relaxed) != IS_INITIALIZED_STATE_VALUE {
                    if fail!(from self, when adaptive_wait.wait(), with ZeroCopyCreationError::InternalError,
                            "{} since a failure while waiting for creation finalization occurred.", msg)
                        < MAX_CREATION_DURATION
                    {
                        break;
                    }
                }

                if mgmt_ref.receive_channel.capacity() != self.receive_channel_size() {
                    fail!(from self, with ZeroCopyCreationError::IncompatibleBufferSize,
                        "{} since the connection has a buffer size of {} but a buffer size of {} is required.",
                        msg, mgmt_ref.receive_channel.capacity(), self.receive_channel_size());
                }

                if mgmt_ref.retrieve_channel.capacity() != self.retrieve_channel_size() {
                    fail!(from self, with ZeroCopyCreationError::IncompatibleMaxBorrowedSampleSetting,
                        "{} since the max borrowed sample setting is set to {} but a value of {} is required.",
                        msg, mgmt_ref.retrieve_channel.capacity() - mgmt_ref.receive_channel.capacity(), self.max_borrowed_samples);
                }

                if mgmt_ref.enable_safe_overflow != self.enable_safe_overflow {
                    fail!(from self, with ZeroCopyCreationError::IncompatibleOverflowSetting,
                        "{} since the safe overflow is set to {} but should be set to {}.",
                        msg, mgmt_ref.enable_safe_overflow, self.enable_safe_overflow);
                }
            }
        }

        Ok(shm)
    }

    fn reserve_port(
        &self,
        mgmt_ref: &mut SharedManagementData,
        new_state: u8,
        msg: &str,
    ) -> Result<(), ZeroCopyCreationError> {
        let mut current_state = State::None.value();

        loop {
            match mgmt_ref.state.compare_exchange(
                current_state,
                current_state | new_state,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => {
                    current_state = v;
                    if current_state & new_state != 0 {
                        fail!(from self, with ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected,
                            "{} since an instance is already connected.", msg);
                    } else if current_state & State::MarkedForDestruction.value() != 0 {
                        fail!(from self, with ZeroCopyCreationError::InternalError,
                            "{} since the connection is currently being cleaned up.", msg);
                    }
                }
            }
        }

        Ok(())
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
        let shm = fail!(from self, when self.create_or_open_shm(),
            "{} since the corresponding connection could not be created or opened", msg);

        let mgmt_ref = unsafe { &mut *(shm.base_address().as_ptr() as *mut SharedManagementData) };
        self.reserve_port(mgmt_ref, State::Sender.value(), msg)?;

        Ok(Sender {
            shared_memory: shm,
            name: self.name,
        })
    }

    fn create_receiver(
        self,
    ) -> Result<<Connection as ZeroCopyConnection>::Receiver, ZeroCopyCreationError> {
        let msg = "Unable to create receiver";
        let shm = fail!(from self, when self.create_or_open_shm(),
            "{} since the corresponding connection could not be created or opened", msg);

        let mgmt_ref = unsafe { &mut *(shm.base_address().as_ptr() as *mut SharedManagementData) };
        self.reserve_port(mgmt_ref, State::Receiver.value(), msg)?;

        Ok(Receiver {
            shared_memory: shm,
            borrow_counter: UnsafeCell::new(0),
            name: self.name,
        })
    }
}

#[derive(Debug)]
pub struct Sender {
    shared_memory: SharedMemory,
    name: FileName,
}

impl Drop for Sender {
    fn drop(&mut self) {
        cleanup_shared_memory(self, &self.shared_memory, State::Sender);
    }
}

impl Sender {
    fn mgmt(&self) -> &SharedManagementData {
        unsafe { &*(self.shared_memory.base_address().as_ptr() as *const SharedManagementData) }
    }
}

impl NamedConcept for Sender {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl ZeroCopyPortDetails for Sender {
    fn buffer_size(&self) -> usize {
        self.mgmt().receive_channel.capacity()
    }

    fn max_borrowed_samples(&self) -> usize {
        self.mgmt().max_borrowed_samples
    }

    fn has_enabled_safe_overflow(&self) -> bool {
        self.mgmt().enable_safe_overflow
    }

    fn is_connected(&self) -> bool {
        self.mgmt().state.load(Ordering::Relaxed) == State::Sender.value() | State::Receiver.value()
    }
}

impl ZeroCopySender for Sender {
    fn try_send(&self, ptr: PointerOffset) -> Result<Option<PointerOffset>, ZeroCopySendError> {
        let msg = "Unable to send sample";
        let space_in_retrieve_channel =
            self.mgmt().retrieve_channel.capacity() - self.mgmt().retrieve_channel.len();

        if space_in_retrieve_channel
            <= self.mgmt().max_borrowed_samples + self.mgmt().receive_channel.len()
        {
            fail!(from self, with ZeroCopySendError::ClearRetrieveChannelBeforeSend,
                "{} since sufficient space for every sample in the retrieve channel cannot be guaranteed. Samples have to be retrieved before a new sample can be send.", msg);
        }

        if !self.mgmt().enable_safe_overflow && self.mgmt().receive_channel.is_full() {
            fail!(from self, with ZeroCopySendError::ReceiveBufferFull,
                             "{} since the receive buffer is full.", msg);
        }

        match unsafe { self.mgmt().receive_channel.push(ptr.value()) } {
            Some(v) => Ok(Some(PointerOffset::new(v))),
            None => Ok(None),
        }
    }

    fn blocking_send(
        &self,
        ptr: PointerOffset,
    ) -> Result<Option<PointerOffset>, ZeroCopySendError> {
        if !self.mgmt().enable_safe_overflow {
            AdaptiveWaitBuilder::new()
                .create()
                .unwrap()
                .wait_while(|| self.mgmt().receive_channel.is_full())
                .unwrap();
        }

        self.try_send(ptr)
    }

    fn reclaim(&self) -> Result<Option<PointerOffset>, ZeroCopyReclaimError> {
        match unsafe { self.mgmt().retrieve_channel.pop() } {
            None => Ok(None),
            Some(v) => Ok(Some(PointerOffset::new(v))),
        }
    }
}

#[derive(Debug)]
pub struct Receiver {
    shared_memory: SharedMemory,
    borrow_counter: UnsafeCell<usize>,
    name: FileName,
}

impl Drop for Receiver {
    fn drop(&mut self) {
        cleanup_shared_memory(self, &self.shared_memory, State::Receiver);
    }
}

impl Receiver {
    fn mgmt(&self) -> &SharedManagementData {
        unsafe { &*(self.shared_memory.base_address().as_ptr() as *const SharedManagementData) }
    }

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
        &self.name
    }
}

impl ZeroCopyPortDetails for Receiver {
    fn buffer_size(&self) -> usize {
        self.mgmt().receive_channel.capacity()
    }

    fn max_borrowed_samples(&self) -> usize {
        self.mgmt().max_borrowed_samples
    }

    fn has_enabled_safe_overflow(&self) -> bool {
        self.mgmt().enable_safe_overflow
    }

    fn is_connected(&self) -> bool {
        self.mgmt().state.load(Ordering::Relaxed) == State::Sender.value() | State::Receiver.value()
    }
}

impl ZeroCopyReceiver for Receiver {
    fn receive(&self) -> Result<Option<PointerOffset>, ZeroCopyReceiveError> {
        if *self.borrow_counter() >= self.mgmt().max_borrowed_samples {
            fail!(from self, with ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue,
                "Unable to receive another sample since already {} samples were borrowed and this would exceed the max borrow value of {}.",
                    self.borrow_counter(), self.max_borrowed_samples());
        }

        match unsafe { self.mgmt().receive_channel.pop() } {
            None => Ok(None),
            Some(v) => {
                *self.borrow_counter() += 1;
                Ok(Some(PointerOffset::new(v)))
            }
        }
    }

    fn release(&self, ptr: PointerOffset) -> Result<(), ZeroCopyReleaseError> {
        match unsafe { self.mgmt().retrieve_channel.push(ptr.value()) } {
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
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        Ok(SharedMemory::does_exist(unsafe {
            &FileName::new_unchecked(cfg.path_for(name).file_name())
        }))
    }

    fn list_cfg(
        config: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let entries = SharedMemory::list();

        let mut result = vec![];
        for entry in &entries {
            if let Some(entry_name) = config.extract_name_from_file(entry) {
                result.push(entry_name);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let full_name = unsafe { FileName::new_unchecked(cfg.path_for(name).file_name()) };
        let msg = "Unable to remove zero_copy_connection::posix_shared_memory";
        let origin = "zero_copy_connection::posix_shared_memory::Connection::remove_cfg()";

        match iceoryx2_bb_posix::shared_memory::SharedMemory::remove(&full_name) {
            Ok(v) => Ok(v),
            Err(
                iceoryx2_bb_posix::shared_memory::SharedMemoryRemoveError::InsufficientPermissions,
            ) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                            "{} \"{}\" due to insufficient permissions.", msg, name);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                        "{} \"{}\" due to an internal failure ({:?}).", msg, name, v);
            }
        }
    }
}

impl ZeroCopyConnection for Connection {
    type Sender = Sender;
    type Builder = Builder;
    type Receiver = Receiver;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}
