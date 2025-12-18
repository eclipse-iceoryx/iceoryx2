// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

pub use crate::dynamic_storage::*;
use crate::named_concept::NamedConceptDoesExistError;
use crate::named_concept::NamedConceptListError;
pub use core::ops::Deref;

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ptr::NonNull;
use iceoryx2_bb_concurrency::atomic::Ordering;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::directory::*;
use iceoryx2_bb_posix::file::File;
use iceoryx2_bb_posix::file::FileAccessError;
use iceoryx2_bb_posix::file::FileBuilder;
use iceoryx2_bb_posix::file::FileCreationError;
use iceoryx2_bb_posix::file::FileOpenError;
use iceoryx2_bb_posix::file::FileRemoveError;
use iceoryx2_bb_posix::file_descriptor::FileDescriptor;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::memory_mapping::MappingBehavior;
use iceoryx2_bb_posix::memory_mapping::MappingPermission;
use iceoryx2_bb_posix::memory_mapping::MemoryMapping;
use iceoryx2_bb_posix::memory_mapping::MemoryMappingBuilder;
use iceoryx2_bb_posix::shared_memory::*;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fail;
use iceoryx2_log::warn;

use crate::static_storage::file::NamedConceptConfiguration;
use crate::static_storage::file::NamedConceptRemoveError;

use self::dynamic_storage_configuration::DynamicStorageConfiguration;

const INIT_PERMISSIONS: Permission = Permission::OWNER_WRITE;

#[cfg(not(feature = "dev_permissions"))]
const FINAL_PERMISSIONS: Permission = Permission::OWNER_ALL;

#[cfg(feature = "dev_permissions")]
const FINAL_PERMISSIONS: Permission = Permission::ALL;

/// The builder of [`Storage`].
#[derive(Debug)]
pub struct Builder<'builder, T: Send + Sync + Debug> {
    storage_name: FileName,
    call_drop_on_destruction: bool,
    supplementary_size: usize,
    has_ownership: bool,
    config: Configuration<T>,
    timeout: Duration,
    initializer: Initializer<'builder, T>,
    _phantom_data: PhantomData<T>,
}

#[derive(Debug)]
pub struct Configuration<T: Send + Sync + Debug> {
    suffix: FileName,
    prefix: FileName,
    path: Path,
    _data: PhantomData<T>,
    type_name: String,
}

impl<T: Send + Sync + Debug> Clone for Configuration<T> {
    fn clone(&self) -> Self {
        Self {
            suffix: self.suffix,
            prefix: self.prefix,
            path: self.path,
            _data: PhantomData,
            type_name: self.type_name.clone(),
        }
    }
}

#[repr(C)]
struct Data<T: Send + Sync + Debug> {
    version: AtomicU64,
    call_drop_on_destruction: bool,
    data: T,
}

impl<T: Send + Sync + Debug> Default for Configuration<T> {
    fn default() -> Self {
        Self {
            path: Storage::<()>::default_path_hint(),
            suffix: Storage::<()>::default_suffix(),
            prefix: Storage::<()>::default_prefix(),
            _data: PhantomData,
            type_name: core::any::type_name::<T>().to_string(),
        }
    }
}

impl<T: Send + Sync + Debug> DynamicStorageConfiguration for Configuration<T> {
    fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl<T: Send + Sync + Debug> NamedConceptConfiguration for Configuration<T> {
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
        self.path = *value;
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path
    }

    fn path_for(&self, value: &FileName) -> iceoryx2_bb_system_types::file_path::FilePath {
        self.path_for_with_type(value)
    }

    fn extract_name_from_file(&self, value: &FileName) -> Option<FileName> {
        self.extract_name_from_file_with_type(value)
    }
}

impl<T: Send + Sync + Debug> NamedConceptBuilder<Storage<T>> for Builder<'_, T> {
    fn new(storage_name: &FileName) -> Self {
        Self {
            call_drop_on_destruction: true,
            has_ownership: true,
            storage_name: *storage_name,
            supplementary_size: 0,
            config: Configuration::default(),
            timeout: Duration::ZERO,
            initializer: Initializer::new(|_, _| true),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration<T>) -> Self {
        self.config = config.clone();
        self
    }
}

impl<T: Send + Sync + Debug> Builder<'_, T> {
    fn open_impl(&self) -> Result<Storage<T>, DynamicStorageOpenError> {
        let msg = "Failed to open file::DynamicStorage";

        let full_path = self.config.path_for(&self.storage_name);
        let mut wait_for_read_write_access = fail!(from self, when AdaptiveWaitBuilder::new().create(),
                                    with DynamicStorageOpenError::InternalError,
                                    "{} since the AdaptiveWait could not be initialized.", msg);

        let mut elapsed_time = Duration::ZERO;
        let file = loop {
            match FileBuilder::new(&full_path).open_existing(AccessMode::ReadWrite) {
                Ok(v) => break v,
                Err(FileOpenError::FileDoesNotExist) => {
                    fail!(from self, with DynamicStorageOpenError::DoesNotExist,
                    "{} since a file with that name does not exists.", msg);
                }
                Err(FileOpenError::InsufficientPermissions) => {
                    if elapsed_time >= self.timeout {
                        fail!(from self, with DynamicStorageOpenError::InitializationNotYetFinalized,
                        "{} since it is not readable - (it is not initialized after {:?}).",
                        msg, self.timeout);
                    }
                }
                Err(_) => {
                    fail!(from self, with DynamicStorageOpenError::InternalError, "{} since the underlying file could not be opened.", msg);
                }
            };

            elapsed_time = fail!(from self, when wait_for_read_write_access.wait(),
                                    with DynamicStorageOpenError::InternalError,
                                    "{} since the adaptive wait call failed.", msg);
        };

        let raw_fd = unsafe { file.file_descriptor().native_handle() };
        let fd = unsafe { FileDescriptor::non_owning_new_unchecked(raw_fd) };

        let file_size = match file.metadata() {
            Ok(m) => m.size(),
            Err(e) => {
                fail!(from self, with DynamicStorageOpenError::InternalError,
                    "{msg} since the file size could not be acquired ({e:?}).");
            }
        };

        let memory_mapping = match MemoryMappingBuilder::from_file_descriptor(fd)
            .mapping_behavior(MappingBehavior::Shared)
            .initial_mapping_permission(MappingPermission::ReadWrite)
            .size(file_size as usize)
            .create()
        {
            Ok(v) => v,
            Err(e) => {
                fail!(from self, with DynamicStorageOpenError::InternalError,
                        "{msg} since the memory could not be mapped into the process ({e:?}).");
            }
        };

        let init_state = memory_mapping.base_address() as *const Data<T>;

        loop {
            // The mem-sync is actually not required since an uninitialized dynamic storage has
            // only write permissions and can be therefore not consumed.
            // This is only for the case that this strategy fails on an obscure POSIX platform.
            //
            //////////////////////////////////////////
            // SYNC POINT: read Data<T>::data
            //////////////////////////////////////////
            let package_version = unsafe { &(*init_state) }
                .version
                .load(core::sync::atomic::Ordering::SeqCst);

            let package_version = PackageVersion::from_u64(package_version);
            if package_version.to_u64() == 0 {
                if elapsed_time >= self.timeout {
                    fail!(from self, with DynamicStorageOpenError::InitializationNotYetFinalized,
                        "{} since the version number was not set - (it is not initialized after {:?}).",
                        msg, self.timeout);
                }
            } else if package_version != PackageVersion::get() {
                fail!(from self, with DynamicStorageOpenError::VersionMismatch,
                       "{} since the dynamic storage was created with version {} but this process requires version {}.",
                        msg, package_version, PackageVersion::get());
            } else {
                break;
            }

            elapsed_time = fail!(from self, when wait_for_read_write_access.wait(),
                                    with DynamicStorageOpenError::InternalError,
                                    "{} since the adaptive wait call failed.", msg);
        }

        Ok(Storage {
            file,
            memory_mapping,
            name: self.storage_name,
            _data: PhantomData,
        })
    }

    fn create_impl(&mut self) -> Result<Storage<T>, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic_storage::file::DynamicStorage";

        let full_name = self.config.path_for(&self.storage_name);
        let mut file = match FileBuilder::new(&full_name)
            .has_ownership(self.has_ownership)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(INIT_PERMISSIONS)
            .create()
        {
            Ok(v) => v,
            Err(FileCreationError::FileAlreadyExists) => {
                fail!(from self, with DynamicStorageCreateError::AlreadyExists,
                    "{} since a file with the name already exists.", msg);
            }
            Err(FileCreationError::InsufficientPermissions) => {
                fail!(from self, with DynamicStorageCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(_) => {
                fail!(from self, with DynamicStorageCreateError::InternalError,
                    "{} since the underlying file could not be created.", msg);
            }
        };

        let file_size = core::mem::size_of::<Data<T>>() + self.supplementary_size;

        if let Err(e) = file.truncate(file_size) {
            fail!(from self, with DynamicStorageCreateError::InternalError,
                "{msg} since the file could not be resized to {file_size} ({e:?}).");
        }

        let raw_fd = unsafe { file.file_descriptor().native_handle() };
        let fd = unsafe { FileDescriptor::non_owning_new_unchecked(raw_fd) };

        let memory_mapping = match MemoryMappingBuilder::from_file_descriptor(fd)
            .mapping_behavior(MappingBehavior::Shared)
            .initial_mapping_permission(MappingPermission::ReadWrite)
            .size(file_size)
            .create()
        {
            Ok(m) => m,
            Err(e) => {
                fail!(from self, with DynamicStorageCreateError::InternalError,
                        "{msg} since the file could not be mapped into the process space ({e:?}).");
            }
        };

        Ok(Storage {
            file,
            memory_mapping,
            name: self.storage_name,
            _data: PhantomData,
        })
    }

    fn init_impl(
        &mut self,
        mut storage: Storage<T>,
        initial_value: T,
    ) -> Result<Storage<T>, DynamicStorageCreateError> {
        let msg = "Failed to init dynamic_storage::file::DynamicStorage";
        let value = storage.memory_mapping.base_address_mut() as *mut Data<T>;
        let version_ptr = unsafe { core::ptr::addr_of_mut!((*value).version) };
        unsafe { version_ptr.write(AtomicU64::new(0)) };

        unsafe { core::ptr::addr_of_mut!((*value).data).write(initial_value) };
        unsafe {
            core::ptr::addr_of_mut!((*value).call_drop_on_destruction)
                .write(self.call_drop_on_destruction)
        };

        let supplementary_start = (storage.memory_mapping.base_address() as usize
            + core::mem::size_of::<Data<T>>()) as *mut u8;
        let supplementary_len = storage.memory_mapping.size() - core::mem::size_of::<Data<T>>();

        let mut allocator = BumpAllocator::new(
            unsafe { NonNull::new_unchecked(supplementary_start) },
            supplementary_len,
        );

        let origin = format!("{self:?}");
        if !self
            .initializer
            .call(unsafe { &mut (*value).data }, &mut allocator)
        {
            storage.file.acquire_ownership();
            fail!(from origin, with DynamicStorageCreateError::InitializationFailed,
                "{} since the initialization of the underlying construct failed.", msg);
        }

        // The mem-sync is actually not required since an uninitialized dynamic storage has
        // only write permissions and can be therefore not consumed.
        // This is only for the case that this strategy fails on an obscure POSIX platform.
        //
        //////////////////////////////////////////
        // SYNC POINT: write Data<T>::data
        //////////////////////////////////////////
        unsafe { (*version_ptr).store(PackageVersion::get().to_u64(), Ordering::SeqCst) };

        if let Err(e) = storage.file.set_permission(FINAL_PERMISSIONS) {
            storage.file.acquire_ownership();
            fail!(from origin, with DynamicStorageCreateError::InternalError,
                "{} since the final permissions could not be applied to the underlying file ({:?}).",
                msg, e);
        }

        Ok(storage)
    }
}

impl<'builder, T: Send + Sync + Debug> DynamicStorageBuilder<'builder, T, Storage<T>>
    for Builder<'builder, T>
{
    fn call_drop_on_destruction(mut self, value: bool) -> Self {
        self.call_drop_on_destruction = value;
        self
    }

    fn has_ownership(mut self, value: bool) -> Self {
        self.has_ownership = value;
        self
    }

    fn initializer<F: FnMut(&mut T, &mut BumpAllocator) -> bool + 'builder>(
        mut self,
        value: F,
    ) -> Self {
        self.initializer = Initializer::new(value);
        self
    }

    fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    fn supplementary_size(mut self, value: usize) -> Self {
        self.supplementary_size = value;
        self
    }

    fn create(mut self, initial_value: T) -> Result<Storage<T>, DynamicStorageCreateError> {
        let shm = self.create_impl()?;
        self.init_impl(shm, initial_value)
    }

    fn open(self) -> Result<Storage<T>, DynamicStorageOpenError> {
        self.open_impl()
    }

    fn open_or_create(
        mut self,
        initial_value: T,
    ) -> Result<Storage<T>, DynamicStorageOpenOrCreateError> {
        loop {
            match self.open_impl() {
                Ok(storage) => return Ok(storage),
                Err(DynamicStorageOpenError::DoesNotExist) => match self.create_impl() {
                    Ok(shm) => {
                        return Ok(self.init_impl(shm, initial_value)?);
                    }
                    Err(DynamicStorageCreateError::AlreadyExists) => continue,
                    Err(e) => return Err(e.into()),
                },
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// Implements [`DynamicStorage`] based on a [`File`]. It is built by
/// [`Builder`].
#[derive(Debug)]
pub struct Storage<T: Debug + Send + Sync> {
    file: File,
    memory_mapping: MemoryMapping,
    name: FileName,
    _data: PhantomData<T>,
}

unsafe impl<T: Debug + Send + Sync> Send for Storage<T> {}
unsafe impl<T: Debug + Send + Sync> Sync for Storage<T> {}

impl<T: Debug + Send + Sync> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.file.has_ownership() {
            let data = unsafe { &mut (*(self.memory_mapping.base_address_mut() as *mut Data<T>)) };
            if data.call_drop_on_destruction {
                let user_type = &mut data.data;
                unsafe { core::ptr::drop_in_place(user_type) };
            }
        }
    }
}

impl<T: Send + Sync + Debug> NamedConcept for Storage<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Send + Sync + Debug> NamedConceptMgmt for Storage<T> {
    type Configuration = Configuration<T>;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        let origin = "dynamic_storage::File::does_exist_cfg()";
        let msg = "Unable to determine if a dynamic storage exists";
        let full_name = cfg.path_for(name);
        match File::does_exist(&full_name) {
            Ok(v) => Ok(v),
            Err(FileAccessError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptDoesExistError::InsufficientPermissions,
                    "{msg} with the name {name} due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from origin, with NamedConceptDoesExistError::InternalError,
                    "{msg} with the name {name} due to an internal error ({e:?}).");
            }
        }
    }

    fn list_cfg(cfg: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let origin = "dynamic_storage::File::list_cfg()";
        let msg = "Unable to list all dynamic storages";
        let directory = match Directory::new(&cfg.path) {
            Ok(d) => d,
            Err(DirectoryOpenError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptListError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from origin, with NamedConceptListError::InternalError,
                    "{msg} due to an internal error ({e:?}).");
            }
        };

        let mut result = vec![];
        let contents = match directory.contents() {
            Ok(c) => c,
            Err(DirectoryReadError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptListError::InsufficientPermissions,
                    "{msg} since the directory content of {} could not be listed due to insufficient permissions.", cfg.path);
            }
            Err(e) => {
                fail!(from origin, with NamedConceptListError::InternalError,
                    "{msg} since the directory content of {} could not be listed due to an internal error ({e:?}).", cfg.path);
            }
        };

        for entry in contents {
            if let Some(entry_name) = cfg.extract_name_from_file(entry.name()) {
                result.push(entry_name);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let full_path = cfg.path_for(name);
        let msg = "Unable to remove dynamic_storage::file::Storage";
        let origin = "dynamic_storage::file::Storage::remove_cfg()";

        match Builder::<T>::new(name).config(cfg).open() {
            Ok(s) => {
                s.acquire_ownership();
                Ok(true)
            }
            Err(DynamicStorageOpenError::DoesNotExist) => Ok(false),
            Err(e) => {
                warn!(from origin,
                    "Removing DynamicStorage in broken state ({:?}) will not call drop of the underlying data type {:?}.",
                    e, core::any::type_name::<T>());

                match File::remove(&full_path) {
                    Ok(v) => Ok(v),
                    Err(FileRemoveError::InsufficientPermissions) => {
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
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        crate::named_concept::remove_path_hint(value)
    }
}

impl<T: Send + Sync + Debug> DynamicStorage<T> for Storage<T> {
    type Builder<'builder> = Builder<'builder, T>;

    fn does_support_persistency() -> bool {
        SharedMemory::does_support_persistency()
    }

    fn acquire_ownership(&self) {
        self.file.acquire_ownership()
    }

    fn get(&self) -> &T {
        unsafe { &(*(self.memory_mapping.base_address() as *const Data<T>)).data }
    }

    fn has_ownership(&self) -> bool {
        self.file.has_ownership()
    }

    fn release_ownership(&self) {
        self.file.release_ownership()
    }

    unsafe fn __internal_set_type_name_in_config(
        config: &mut Self::Configuration,
        type_name: &str,
    ) {
        config.type_name = type_name.to_string()
    }
}
