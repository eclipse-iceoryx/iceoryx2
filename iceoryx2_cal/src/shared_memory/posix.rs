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

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

pub use crate::shared_memory::*;
use iceoryx2_bb_elementary::allocator::DeallocationError;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::shared_memory::{AccessMode, Permission};
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::unix_datagram_socket::CreationMode;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;

use crate::static_storage::file::{
    NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt,
    NamedConceptRemoveError,
};

const IS_INITIALIZED_STATE_VALUE: u64 = 0xbeefaffedeadbeef;

#[derive(Clone, Debug)]
pub struct Configuration {
    pub is_memory_locked: bool,
    pub permission: Permission,
    pub zero_memory: bool,
    path: Path,
    suffix: FileName,
    prefix: FileName,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            is_memory_locked: false,
            permission: Permission::OWNER_ALL,
            zero_memory: true,
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

impl<Allocator: ShmAllocator + Debug> Builder<Allocator> {
    fn allocator_details_size() -> usize {
        std::mem::size_of::<AllocatorDetails<Allocator>>()
            + std::mem::align_of::<AllocatorDetails<Allocator>>()
            - 1
    }

    fn allocator_size(&self, allocator_config: &Allocator::Configuration) -> usize {
        Self::allocator_details_size() + Allocator::management_size(self.size, allocator_config)
    }
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

        let allocator_mgmt_size = self.allocator_size(allocator_config);

        let shm = match iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder::new(unsafe {
            &FileName::new_unchecked(self.config.path_for(&self.name).file_name())
        })
        .is_memory_locked(self.config.is_memory_locked)
        .creation_mode(CreationMode::CreateExclusive)
        .size(self.size + allocator_mgmt_size)
        .permission(self.config.permission)
        .zero_memory(self.config.zero_memory)
        .create()
        {
            Ok(s) => s,
            Err(iceoryx2_bb_posix::shared_memory::SharedMemoryCreationError::AlreadyExist) => {
                fail!(from self, with SharedMemoryCreateError::AlreadyExists,
                        "{} since a shared memory with that name already exists.", msg);
            }
            Err(
                iceoryx2_bb_posix::shared_memory::SharedMemoryCreationError::InsufficientPermissions,
            ) => {
                fail!(from self, with SharedMemoryCreateError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from self, with SharedMemoryCreateError::InternalError,
                        "{} since an unknown error has occurred ({:?})", msg, v);
            }
        };

        let allocator_addr = shm.base_address().as_ptr() as *mut AllocatorDetails<Allocator>;
        let slice = unsafe {
            std::slice::from_raw_parts_mut(
                (allocator_addr as usize + allocator_mgmt_size) as *mut u8,
                self.size,
            )
        };

        unsafe {
            allocator_addr.write(AllocatorDetails {
                state: AtomicU64::new(0),
                allocator_id: Allocator::unique_id(),
                allocator: Allocator::new_uninit(
                    SystemInfo::PageSize.value(),
                    NonNull::new_unchecked(slice),
                    allocator_config,
                ),
                mgmt_size: allocator_mgmt_size,
            })
        };

        let mgmt_addr = unsafe {
            NonNull::new_unchecked(
                (shm.base_address().as_ptr() as usize + Self::allocator_details_size()) as *mut u8,
            )
        };
        let bump_allocator = BumpAllocator::new(
            mgmt_addr,
            Allocator::management_size(self.size, allocator_config),
        );

        fail!(from self, when unsafe { (*allocator_addr).allocator.init(&bump_allocator) },
                with SharedMemoryCreateError::InternalError,
                "{} since the management memory for the allocator could not be initialized.", msg);

        unsafe {
            (*allocator_addr)
                .state
                .store(IS_INITIALIZED_STATE_VALUE, Ordering::Relaxed)
        };

        Ok(Memory::<Allocator> {
            shared_memory: shm,
            name: self.name,
            allocator: unsafe { NonNull::new_unchecked(allocator_addr) },
        })
    }

    fn open(self) -> Result<Memory<Allocator>, SharedMemoryOpenError> {
        let msg = "Unable to open shared memory";

        let shm = match iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder::new(unsafe {
            &FileName::new_unchecked(self.config.path_for(&self.name).file_name())
        })
        .is_memory_locked(self.config.is_memory_locked)
        .open_existing(AccessMode::ReadWrite)
        {
            Ok(s) => s,
            Err(iceoryx2_bb_posix::shared_memory::SharedMemoryCreationError::DoesNotExist) => {
                fail!(from self, with SharedMemoryOpenError::DoesNotExist,
                        "{} since a shared memory with that name does not exist.", msg);
            }
            Err(iceoryx2_bb_posix::shared_memory::SharedMemoryCreationError::SizeDoesNotFit) => {
                fail!(from self, with SharedMemoryOpenError::SizeDoesNotFit,
                        "{} since the requested size is not equal the actual size of the shared memory.", msg);
            }
            Err(
                iceoryx2_bb_posix::shared_memory::SharedMemoryCreationError::InsufficientPermissions,
            ) => {
                fail!(from self, with SharedMemoryOpenError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from self, with SharedMemoryOpenError::InternalError,
                        "{} since an unknown error has occurred ({:?}).", msg, v);
            }
        };

        let allocator_addr = shm.base_address().as_ptr() as *mut AllocatorDetails<Allocator>;

        if unsafe { &*allocator_addr }.state.load(Ordering::Relaxed) != IS_INITIALIZED_STATE_VALUE {
            fail!(from self, with SharedMemoryOpenError::InternalError,
                    "{} since the creation of the shared memory is not yet finished.", msg);
        }

        const SPACE_FOR_ALLOCATOR_ID: usize = 1;

        if shm.size() <= SPACE_FOR_ALLOCATOR_ID {
            fail!(from self, with SharedMemoryOpenError::SizeDoesNotFit,
                "{} since the shared memories size {} is smaller than the minimum required size of {}.",
                msg, shm.size(), SPACE_FOR_ALLOCATOR_ID);
        }

        if unsafe { &*allocator_addr }.allocator_id != Allocator::unique_id() {
            fail!(from self, with SharedMemoryOpenError::WrongAllocatorSelected,
                "{} since the shared memory contains an allocator with unique id {} but the selected allocator has the unique id {}.",
                msg, unsafe{&*allocator_addr}.allocator_id, Allocator::unique_id());
        }

        Ok(Memory::<Allocator> {
            shared_memory: shm,
            name: self.name,
            allocator: unsafe { NonNull::new_unchecked(allocator_addr) },
        })
    }
}

#[derive(Debug)]
pub struct Memory<Allocator: ShmAllocator> {
    shared_memory: iceoryx2_bb_posix::shared_memory::SharedMemory,
    name: FileName,
    allocator: NonNull<AllocatorDetails<Allocator>>,
}

#[repr(C)]
struct AllocatorDetails<Allocator: ShmAllocator> {
    state: AtomicU64,
    allocator_id: u8,
    allocator: Allocator,
    mgmt_size: usize,
}

impl<Allocator: ShmAllocator + Debug> Memory<Allocator> {
    fn allocator(&self) -> &AllocatorDetails<Allocator> {
        unsafe { self.allocator.as_ref() }
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConcept for Memory<Allocator> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<Allocator: ShmAllocator + Debug> NamedConceptMgmt for Memory<Allocator> {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let full_name = unsafe { FileName::new_unchecked(cfg.path_for(name).file_name()) };

        Ok(iceoryx2_bb_posix::shared_memory::SharedMemory::does_exist(
            &full_name,
        ))
    }

    fn list_cfg(
        config: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let entries = iceoryx2_bb_posix::shared_memory::SharedMemory::list();

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
        let msg = "Unable to remove shared_memory::posix";
        let origin = "shared_memory::Posix::remove_cfg()";

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

impl<Allocator: ShmAllocator + Debug> crate::shared_memory::SharedMemory<Allocator>
    for Memory<Allocator>
{
    type Builder = Builder<Allocator>;

    fn size(&self) -> usize {
        self.shared_memory.size() - self.allocator().mgmt_size
    }

    fn max_alignment(&self) -> usize {
        self.allocator().allocator.max_alignment()
    }

    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
        let offset = fail!(from self, when unsafe { self.allocator().allocator.allocate(layout) },
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
        fail!(from self, when self.allocator().allocator.deallocate(offset, layout),
            "Failed to deallocate shared memory chunk due to an internal allocator failure.");
        Ok(())
    }

    fn release_ownership(&mut self) {
        self.shared_memory.release_ownership()
    }

    fn allocator_data_start_address(&self) -> usize {
        (self.shared_memory.base_address().as_ptr() as *const u8) as usize
            + self.allocator().mgmt_size
    }
}
