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

use std::marker::PhantomData;
use std::{alloc::Layout, fmt::Debug};

use crate::dynamic_storage::*;
pub use crate::shared_memory::*;
use iceoryx2_bb_elementary::allocator::BaseAllocator;
use iceoryx2_bb_log::{debug, fail};
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;

use crate::static_storage::file::{
    NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt,
};

type StorageType<T> = crate::dynamic_storage::posix_shared_memory::Storage<AllocatorDetails<T>>;

#[derive(Debug)]
pub struct Configuration<Allocator: ShmAllocator + Debug> {
    pub zero_memory: bool,
    path: Path,
    suffix: FileName,
    prefix: FileName,
    _phantom: PhantomData<Allocator>,
}

impl<T: ShmAllocator + Debug> From<&Configuration<T>>
    for <StorageType<T> as NamedConceptMgmt>::Configuration
{
    #[allow(private_interfaces)]
    fn from(value: &Configuration<T>) -> Self {
        <StorageType<T> as NamedConceptMgmt>::Configuration::default()
            .prefix(value.prefix)
            .suffix(value.suffix)
            .path_hint(value.path)
    }
}

impl<Allocator: ShmAllocator + Debug> Default for Configuration<Allocator> {
    fn default() -> Self {
        Self {
            zero_memory: true,
            path: Memory::<Allocator>::default_path_hint(),
            suffix: Memory::<Allocator>::default_suffix(),
            prefix: Memory::<Allocator>::default_prefix(),
            _phantom: PhantomData,
        }
    }
}

impl<Allocator: ShmAllocator + Debug> Clone for Configuration<Allocator> {
    fn clone(&self) -> Self {
        Self {
            zero_memory: self.zero_memory,
            path: self.path,
            suffix: self.suffix,
            prefix: self.prefix,
            _phantom: self._phantom,
        }
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConceptConfiguration for Configuration<Allocator> {
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
    config: Configuration<Allocator>,
    _phantom_allocator: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator + Debug> NamedConceptBuilder<Memory<Allocator>>
    for Builder<Allocator>
{
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: Configuration::default(),
            size: 0,
            _phantom_allocator: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration<Allocator>) -> Self {
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

        let allocator_mgmt_size = Allocator::management_size(self.size, allocator_config);

        let storage = match <<StorageType<Allocator> as DynamicStorage<
            AllocatorDetails<Allocator>,
        >>::Builder as NamedConceptBuilder<StorageType<Allocator>>>::new(
            &self.name
        )
        .config(&(&self.config).into())
        .supplementary_size(self.size + allocator_mgmt_size)
        .has_ownership(true)
        .create_and_initialize(
            AllocatorDetails {
                allocator_id: Allocator::unique_id(),
                allocator: None,
                mgmt_size: allocator_mgmt_size,
                payload_size: self.size,
                payload_start_offset: 0,
            },
            |details, init_allocator| -> bool {
                let memory = match init_allocator.allocate(unsafe { Layout::from_size_align_unchecked(self.size, 1) }) {
                    Ok(m) => m,
                    Err(e) => {
                        debug!(from self, "{} since the payload memory could not be acquired ({:?}).", msg, e);
                        return false;
                    }
                };

                details.payload_start_offset = (memory.as_ptr() as *const u8) as usize - (details as *const AllocatorDetails<Allocator>) as usize;

                details.allocator = Some(unsafe {
                    Allocator::new_uninit(
                        SystemInfo::PageSize.value(),
                        memory,
                        allocator_config,
                    )
                });

                if let Err(e) = unsafe { details.allocator.as_ref().unwrap_unchecked().init(init_allocator) } {
                    debug!(from self, "{} since the management memory for the allocator could not be initialized ({:?}).", msg, e);
                    false
                } else {
                    true
                }
            },
        ) {
            Ok(s) => s,
            Err(DynamicStorageCreateError::AlreadyExists) => {
                fail!(from self, with SharedMemoryCreateError::AlreadyExists,
                        "{} since a shared memory with that name already exists.", msg);
                }
            Err(DynamicStorageCreateError::InsufficientPermissions) => {
                fail!(from self, with SharedMemoryCreateError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
                }
            Err(DynamicStorageCreateError::InitializationFailed) => {
                fail!(from self, with SharedMemoryCreateError::InternalError,
                        "{} since the initialization failed.", msg);
                }
            Err(DynamicStorageCreateError::InternalError) => {
                fail!(from self, with SharedMemoryCreateError::InternalError,
                        "{} since an unknown error has occurred.", msg);
                }
        };

        Ok(Memory::<Allocator> {
            storage,
            name: self.name,
        })
    }

    fn open(self) -> Result<Memory<Allocator>, SharedMemoryOpenError> {
        let msg = "Unable to open shared memory";

        let storage = match <<StorageType<Allocator> as DynamicStorage<
            AllocatorDetails<Allocator>,
        >>::Builder as NamedConceptBuilder<StorageType<Allocator>>>::new(
            &self.name
        )
        .config(&(&self.config).into())
        .has_ownership(false)
        .open()
        {
            Ok(s) => s,
            Err(DynamicStorageOpenError::DoesNotExist) => {
                fail!(from self, with SharedMemoryOpenError::DoesNotExist,
                        "{} since a shared memory with that name does not exist.", msg);
            }
            Err(DynamicStorageOpenError::InitializationNotYetFinalized) => {
                fail!(from self, with SharedMemoryOpenError::InitializationNotYetFinalized,
                        "{} since the underlying shared memory is not yet initialized.", msg);
            }
            Err(DynamicStorageOpenError::VersionMismatch) => {
                fail!(from self, with SharedMemoryOpenError::VersionMismatch,
                        "{} since the version number of the construct does not match.", msg);
            }
            Err(DynamicStorageOpenError::InternalError) => {
                fail!(from self, with SharedMemoryOpenError::InternalError,
                        "{} since an unknown error has occurred.", msg);
            }
        };

        if storage.get().allocator_id != Allocator::unique_id() {
            fail!(from self, with SharedMemoryOpenError::WrongAllocatorSelected,
                "{} since the shared memory contains an allocator with unique id {} but the selected allocator has the unique id {}.",
                msg, storage.get().allocator_id, Allocator::unique_id());
        }

        let payload_size = storage.get().payload_size;
        if payload_size < self.size {
            fail!(from self, with SharedMemoryOpenError::SizeDoesNotFit,
                    "{} since a memory size of {} was requested but only {} is available.",
                    msg, self.size, payload_size);
        }

        Ok(Memory::<Allocator> {
            storage,
            name: self.name,
        })
    }
}

#[derive(Debug)]
pub struct Memory<Allocator: ShmAllocator> {
    storage: StorageType<Allocator>,
    name: FileName,
}

#[derive(Debug)]
#[repr(C)]
struct AllocatorDetails<Allocator: ShmAllocator> {
    allocator_id: u8,
    mgmt_size: usize,
    payload_size: usize,
    allocator: Option<Allocator>,
    payload_start_offset: usize,
}

impl<Allocator: ShmAllocator + Debug> NamedConcept for Memory<Allocator> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConceptMgmt for Memory<Allocator> {
    type Configuration = Configuration<Allocator>;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        Ok(fail!(from "shared_memory::posix::does_exist_cfg()",
            when StorageType::<Allocator>::does_exist_cfg(name, &cfg.into()),
            "Unable to remove shared memory concept \"{}\".", name))
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        Ok(fail!(from "shared_memory::posix::list_cfg()",
            when StorageType::<Allocator>::list_cfg(&cfg.into()),
            "Unable to list shared memory concepts."))
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        Ok(fail!(from "shared_memory::posix::remove_cfg()",
            when StorageType::<Allocator>::remove_cfg(name, &cfg.into()),
            "Unable to remove shared memory concept \"{}\".", name))
    }
}

impl<Allocator: ShmAllocator + Debug> crate::shared_memory::SharedMemory<Allocator>
    for Memory<Allocator>
{
    type Builder = Builder<Allocator>;

    fn size(&self) -> usize {
        self.storage.get().payload_size
    }

    fn max_alignment(&self) -> usize {
        unsafe { self.storage.get().allocator.as_ref().unwrap_unchecked() }.max_alignment()
    }

    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
        let offset = fail!(from self, when unsafe { self.storage.get().allocator.as_ref().unwrap_unchecked().allocate(layout) },
            "Failed to allocate shared memory due to an internal allocator failure.");

        Ok(ShmPointer {
            offset,
            data_ptr: (offset.value() + self.payload_start_address()) as *mut u8,
        })
    }

    unsafe fn deallocate(&self, offset: PointerOffset, layout: std::alloc::Layout) {
        self.storage
            .get()
            .allocator
            .as_ref()
            .unwrap_unchecked()
            .deallocate(offset, layout);
    }

    fn release_ownership(&mut self) {
        self.storage.release_ownership()
    }

    fn payload_start_address(&self) -> usize {
        (self.storage.get() as *const AllocatorDetails<Allocator>) as usize
            + self.storage.get().payload_start_offset
            + unsafe {
                self.storage
                    .get()
                    .allocator
                    .as_ref()
                    .unwrap_unchecked()
                    .relative_start_address()
            }
    }
}
