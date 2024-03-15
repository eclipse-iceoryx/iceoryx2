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
use std::mem::MaybeUninit;
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

pub mod details {
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

    use super::*;

    fn get_payload_start_address<
        Allocator: ShmAllocator + Debug,
        Storage: DynamicStorage<AllocatorDetails<Allocator>>,
    >(
        storage: &Storage,
    ) -> usize {
        (storage.get() as *const AllocatorDetails<Allocator>) as usize
            + storage.get().payload_start_offset
            + unsafe {
                storage
                    .get()
                    .allocator
                    .assume_init_ref()
                    .relative_start_address()
            }
    }

    #[derive(Debug)]
    pub struct Configuration<
        Allocator: ShmAllocator + Debug,
        Storage: DynamicStorage<AllocatorDetails<Allocator>>,
    > {
        pub zero_memory: bool,
        path: Path,
        suffix: FileName,
        prefix: FileName,
        _phantom: PhantomData<Allocator>,
        _phantom_storage: PhantomData<Storage>,
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        Configuration<Allocator, Storage>
    {
        fn convert(&self) -> Storage::Configuration {
            Storage::Configuration::default()
                .prefix(self.prefix)
                .suffix(self.suffix)
                .path_hint(self.path)
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        Default for Configuration<Allocator, Storage>
    {
        fn default() -> Self {
            Self {
                zero_memory: true,
                path: Memory::<Allocator, Storage>::default_path_hint(),
                suffix: Memory::<Allocator, Storage>::default_suffix(),
                prefix: Memory::<Allocator, Storage>::default_prefix(),
                _phantom: PhantomData,
                _phantom_storage: PhantomData,
            }
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        Clone for Configuration<Allocator, Storage>
    {
        fn clone(&self) -> Self {
            Self {
                zero_memory: self.zero_memory,
                path: self.path,
                suffix: self.suffix,
                prefix: self.prefix,
                _phantom: PhantomData,
                _phantom_storage: PhantomData,
            }
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        NamedConceptConfiguration for Configuration<Allocator, Storage>
    {
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
    pub struct Builder<
        Allocator: ShmAllocator + Debug,
        Storage: DynamicStorage<AllocatorDetails<Allocator>>,
    > {
        name: FileName,
        size: usize,
        config: Configuration<Allocator, Storage>,
        timeout: Duration,
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        NamedConceptBuilder<Memory<Allocator, Storage>> for Builder<Allocator, Storage>
    {
        fn new(name: &FileName) -> Self {
            Self {
                name: *name,
                config: Configuration::default(),
                size: 0,
                timeout: Duration::ZERO,
            }
        }

        fn config(mut self, config: &Configuration<Allocator, Storage>) -> Self {
            self.config = config.clone();
            self
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        Builder<Allocator, Storage>
    {
        fn initialize(
            &self,
            allocator_config: &Allocator::Configuration,
            details: &mut AllocatorDetails<Allocator>,
            init_allocator: &mut BumpAllocator,
        ) -> bool {
            let msg = "Unable to initialize shared memory";
            let res =
                init_allocator.allocate(unsafe { Layout::from_size_align_unchecked(self.size, 1) });
            let memory = match res {
                Ok(m) => m,
                Err(e) => {
                    debug!(from self, "{} since the payload memory could not be acquired ({:?}).", msg, e);
                    return false;
                }
            };

            details.payload_start_offset = (memory.as_ptr() as *const u8) as usize
                - (details as *const AllocatorDetails<Allocator>) as usize;

            details.allocator.write(unsafe {
                Allocator::new_uninit(SystemInfo::PageSize.value(), memory, allocator_config)
            });

            if let Err(e) = unsafe { details.allocator.assume_init_ref().init(init_allocator) } {
                debug!(from self, "{} since the management memory for the allocator could not be initialized ({:?}).", msg, e);
                false
            } else {
                true
            }
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        crate::shared_memory::SharedMemoryBuilder<Allocator, Memory<Allocator, Storage>>
        for Builder<Allocator, Storage>
    {
        fn size(mut self, value: usize) -> Self {
            self.size = value;
            self
        }

        fn timeout(mut self, value: Duration) -> Self {
            self.timeout = value;
            self
        }

        fn create(
            self,
            allocator_config: &Allocator::Configuration,
        ) -> Result<Memory<Allocator, Storage>, SharedMemoryCreateError> {
            let msg = "Unable to create shared memory";

            if self.size == 0 {
                fail!(from self, with SharedMemoryCreateError::SizeIsZero,
                    "{} since the size is zero.", msg);
            }

            let allocator_mgmt_size = Allocator::management_size(self.size, allocator_config);

            let storage = match Storage::Builder::new(&self.name)
                .config(&self.config.convert())
                .supplementary_size(self.size + allocator_mgmt_size)
                .has_ownership(true)
                .create_and_initialize(
                    AllocatorDetails {
                        allocator_id: Allocator::unique_id(),
                        allocator: MaybeUninit::uninit(),
                        mgmt_size: allocator_mgmt_size,
                        payload_size: self.size,
                        payload_start_offset: 0,
                    },
                    |details, init_allocator| -> bool {
                        self.initialize(allocator_config, details, init_allocator)
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

            Ok(Memory::<Allocator, Storage> {
                payload_start_address: get_payload_start_address(&storage),
                storage,
                name: self.name,
                _phantom: PhantomData,
            })
        }

        fn open(self) -> Result<Memory<Allocator, Storage>, SharedMemoryOpenError> {
            let msg = "Unable to open shared memory";

            let storage = match Storage::Builder::new(&self.name)
                .config(&self.config.convert())
                .has_ownership(false)
                .timeout(self.timeout)
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

            Ok(Memory::<Allocator, Storage> {
                payload_start_address: get_payload_start_address(&storage),
                name: self.name,
                storage,
                _phantom: PhantomData,
            })
        }
    }

    #[derive(Debug)]
    pub struct Memory<Allocator: ShmAllocator, Storage: DynamicStorage<AllocatorDetails<Allocator>>> {
        storage: Storage,
        name: FileName,
        payload_start_address: usize,
        _phantom: PhantomData<Allocator>,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct AllocatorDetails<Allocator: ShmAllocator> {
        allocator_id: u8,
        allocator: MaybeUninit<Allocator>,
        mgmt_size: usize,
        payload_size: usize,
        payload_start_offset: usize,
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        NamedConcept for Memory<Allocator, Storage>
    {
        fn name(&self) -> &FileName {
            &self.name
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        NamedConceptMgmt for Memory<Allocator, Storage>
    {
        type Configuration = Configuration<Allocator, Storage>;

        fn does_exist_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
            Ok(fail!(from "shared_memory::posix::does_exist_cfg()",
            when Storage::does_exist_cfg(name, &cfg.convert()),
            "Unable to remove shared memory concept \"{}\".", name))
        }

        fn list_cfg(
            cfg: &Self::Configuration,
        ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
            Ok(fail!(from "shared_memory::posix::list_cfg()",
            when Storage::list_cfg(&cfg.convert()),
            "Unable to list shared memory concepts."))
        }

        unsafe fn remove_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
            Ok(fail!(from "shared_memory::posix::remove_cfg()",
            when Storage::remove_cfg(name, &cfg.convert()),
            "Unable to remove shared memory concept \"{}\".", name))
        }
    }

    impl<Allocator: ShmAllocator + Debug, Storage: DynamicStorage<AllocatorDetails<Allocator>>>
        crate::shared_memory::SharedMemory<Allocator> for Memory<Allocator, Storage>
    {
        type Builder = Builder<Allocator, Storage>;

        fn size(&self) -> usize {
            self.storage.get().payload_size
        }

        fn max_alignment(&self) -> usize {
            unsafe { self.storage.get().allocator.assume_init_ref() }.max_alignment()
        }

        fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
            let offset = fail!(from self, when unsafe { self.storage.get().allocator.assume_init_ref().allocate(layout) },
            "Failed to allocate shared memory due to an internal allocator failure.");

            Ok(ShmPointer {
                offset,
                data_ptr: (offset.value() + self.payload_start_address) as *mut u8,
            })
        }

        unsafe fn deallocate(&self, offset: PointerOffset, layout: std::alloc::Layout) {
            self.storage
                .get()
                .allocator
                .assume_init_ref()
                .deallocate(offset, layout);
        }

        fn release_ownership(&mut self) {
            self.storage.release_ownership()
        }

        fn payload_start_address(&self) -> usize {
            self.payload_start_address
        }
    }
}
