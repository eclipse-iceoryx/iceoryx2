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

pub use crate::shared_memory::*;
use crate::static_storage::file::NamedConceptConfiguration;
use iceoryx2_bb_elementary::allocator::BaseAllocator;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_system_types::{file_path::FilePath, path::Path};
use once_cell::sync::Lazy;
use std::alloc::Layout;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

#[derive(Debug)]
struct SharedMemoryEntry {
    memory: NonNull<u8>,
    mgmt_memory: NonNull<u8>,
    layout: Layout,
    size: usize,
    allocator: Option<Box<dyn Any + Send + Sync>>,
}

impl Drop for SharedMemoryEntry {
    fn drop(&mut self) {
        self.allocator = None;

        unsafe {
            fatal_panic!(from self, when HeapAllocator::new()
                .deallocate(self.memory, self.layout),
                "This should never happen! Failed to release shared memory.");
            fatal_panic!(from self, when HeapAllocator::new().deallocate(self.mgmt_memory, self.layout),
                "This should never happen! Failed to release shared memory allocator management memory.");
        };
    }
}

unsafe impl Send for SharedMemoryEntry {}
unsafe impl Sync for SharedMemoryEntry {}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, Arc<SharedMemoryEntry>>>> =
    Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashMap<FilePath, Arc<SharedMemoryEntry>>>> =
    Lazy::new(|| {
        let result = MutexBuilder::new()
            .is_interprocess_capable(false)
            .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

        if result.is_err() {
            fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global dynamic storage");
        }

        result.unwrap()
    });

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub struct Builder<Allocator: ShmAllocator + Debug> {
    name: FileName,
    size: usize,
    config: Configuration,
    _phantom_allocator: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator + Debug> NamedConceptBuilder<Memory<Allocator>>
    for Builder<Allocator>
{
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            size: 0,
            config: Configuration::default(),
            _phantom_allocator: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl<Allocator: ShmAllocator + Debug>
    crate::shared_memory::SharedMemoryBuilder<Allocator, Memory<Allocator>> for Builder<Allocator>
{
    fn size(mut self, value: usize) -> Self {
        self.size = value;
        self
    }

    fn create(
        self,
        allocator_config: &Allocator::Configuration,
    ) -> Result<Memory<Allocator>, SharedMemoryCreateError> {
        let msg = "Unable to create shared memory";

        if self.size == 0 {
            fail!(from self, with SharedMemoryCreateError::SizeIsZero,
                    "{} since the size is zero.", msg);
        }

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with SharedMemoryCreateError::InternalError,
            "{} due to a failure while acquiring the lock.", msg);

        let full_path = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_path);
        if entry.is_some() {
            fail!(from self, with SharedMemoryCreateError::AlreadyExists,
                "{} since the shared memory does already exist.", msg);
        }

        let layout = Layout::from_size_align(self.size, SystemInfo::PageSize.value()).unwrap();
        let memory = fail!(from self, when  HeapAllocator::new().allocate(layout),
                                        with SharedMemoryCreateError::InternalError,
                                        "{} since the memory could not be allocated.", msg);

        let mgmt_layout = unsafe {
            Layout::from_size_align_unchecked(
                Allocator::management_size(self.size, allocator_config),
                1,
            )
        };

        let mgmt_memory = match HeapAllocator::new().allocate(mgmt_layout) {
            Ok(m) => m,
            Err(_) => {
                fatal_panic!(from self,
                    when unsafe { HeapAllocator::new().deallocate(NonNull::new_unchecked(memory.as_ptr() as *mut u8), mgmt_layout) },
                    "This should never happen! {} since the deallocation of the shared memory failed in the error path.", msg);
                fail!(from self, with SharedMemoryCreateError::InternalError,
                    "{} since the allocators management memory could not be initialized.", msg);
            }
        };

        let mgmt_memory = unsafe { NonNull::new_unchecked(mgmt_memory.as_ptr() as *mut u8) };

        guard.insert(
            full_path,
            Arc::new(SharedMemoryEntry {
                memory: unsafe { NonNull::new_unchecked(memory.as_ptr() as *mut u8) },
                mgmt_memory,
                size: self.size,
                layout,
                allocator: Some(Box::new(unsafe {
                    Allocator::new_uninit(SystemInfo::PageSize.value(), memory, allocator_config)
                })),
            }),
        );

        let new_entry = guard.get_mut(&full_path).unwrap();
        let bump_allocator = BumpAllocator::new(mgmt_memory, mgmt_layout.size());
        fail!(from self, when unsafe {
                                new_entry
                                    .allocator
                                    .as_ref()
                                    .unwrap()
                                    .downcast_ref::<Allocator>()
                                    .unwrap()
                                    .init(&bump_allocator)
                            },
                with SharedMemoryCreateError::InternalError,
                "{} since the initialization of the allocators management section failed.", msg);

        Ok(Memory::<Allocator> {
            name: self.name,
            shm: new_entry.clone(),
            has_ownership: true,
            config: self.config,
            _phantom_data: PhantomData,
        })
    }

    fn open(self) -> Result<Memory<Allocator>, SharedMemoryOpenError> {
        let msg = "Unable to open shared memory";

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with SharedMemoryOpenError::InternalError, "{} due to a failure while acquiring the lock.", msg);

        let full_path = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_path);
        if entry.is_none() {
            fail!(from self, with SharedMemoryOpenError::DoesNotExist,
                "{} since the shared memory does not exist.", msg);
        }

        Ok(Memory::<Allocator> {
            name: self.name,
            shm: entry.unwrap().clone(),
            has_ownership: false,
            config: self.config,
            _phantom_data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Memory<Allocator: ShmAllocator + Debug> {
    name: FileName,
    shm: Arc<SharedMemoryEntry>,
    has_ownership: bool,
    config: Configuration,
    _phantom_data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator + Debug> Drop for Memory<Allocator> {
    fn drop(&mut self) {
        if self.has_ownership {
            let msg = "Unable to release shared memory";
            let guard = PROCESS_LOCAL_STORAGE.lock();
            if guard.is_err() {
                fatal_panic!(from self, "{} since the lock could not be acquired.", msg);
            }

            guard
                .unwrap()
                .remove(&self.config.path_for(&self.name))
                .unwrap();
        }
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConcept for Memory<Allocator> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConceptMgmt for Memory<Allocator> {
    type Configuration = Configuration;

    fn list_cfg(
        config: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let msg = "Unable to list all shared memories";
        let guard = fatal_panic!(from "shared_memory::process_local::Storage::list_cfg",
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
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);

        let msg = "Unable to remove shared memory";
        let guard = PROCESS_LOCAL_STORAGE.lock();
        if guard.is_err() {
            fatal_panic!(from "shared_memory::ProcessLocal::remove", "{} since the lock could not be acquired.", msg);
        }

        Ok(guard.unwrap().remove(&storage_name).is_some())
    }

    fn does_exist_cfg(
        name: &FileName,
        config: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let msg = "Unable to check if shared memory exists";
        let guard = fatal_panic!(from "shared_memory::process_local::Storage::does_exist_cfg",
                        when PROCESS_LOCAL_STORAGE.lock(), "{} since the lock could not be acquired.", msg);

        match guard.get(&config.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

impl<Allocator: ShmAllocator + Debug> Memory<Allocator> {
    fn allocator(&self) -> &Allocator {
        self.shm
            .allocator
            .as_ref()
            .unwrap()
            .downcast_ref::<Allocator>()
            .unwrap()
    }
}

impl<Allocator: ShmAllocator + Debug> crate::shared_memory::SharedMemory<Allocator>
    for Memory<Allocator>
{
    type Builder = Builder<Allocator>;

    fn size(&self) -> usize {
        self.shm.size
    }

    fn max_alignment(&self) -> usize {
        self.allocator().max_alignment()
    }

    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
        let offset = fail!(from self, when unsafe { self.allocator().allocate(layout) },
            "Failed to allocate shared memory due to an internal allocator failure.");

        Ok(ShmPointer {
            offset,
            data_ptr: (offset.value() + self.allocator_data_start_address()) as *mut u8,
        })
    }

    unsafe fn deallocate(
        &self,
        offset: PointerOffset,
        layout: std::alloc::Layout,
    ) -> Result<(), DeallocationError> {
        fail!(from self, when self.allocator().deallocate(offset, layout),
            "Failed to deallocate shared memory chunk due to an internal allocator failure.");
        Ok(())
    }

    fn release_ownership(&mut self) {
        self.has_ownership = false;
    }

    fn allocator_data_start_address(&self) -> usize {
        self.shm.memory.as_ptr() as usize
    }
}
