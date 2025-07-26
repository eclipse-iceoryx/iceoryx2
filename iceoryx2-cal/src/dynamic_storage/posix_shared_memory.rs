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

//! [`SharedMemory`] based implementation of a [`DynamicStorage`].
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_cal::dynamic_storage::posix_shared_memory::*;
//! use iceoryx2_cal::named_concept::*;
//! use core::sync::atomic::{AtomicI64, Ordering};
//!
//! let additional_size: usize = 1024;
//! let storage_name = FileName::new(b"myStorageName").unwrap();
//! let owner = Builder::new(&storage_name)
//!                 .supplementary_size(additional_size)
//!                 // we always have to use a thread-safe object since multiple processes can
//!                 // access this concurrently
//!                 .create(AtomicI64::new(0)).unwrap();
//! owner.get().store(123, Ordering::Relaxed);
//!
//! // usually a different process
//! let storage = Builder::<AtomicI64>::new(&storage_name)
//!                 .open().unwrap();
//!
//! println!("Initial value: {}", storage.get().load(Ordering::Relaxed));
//! // returns a reference to the underlying atomic
//! storage.get().store(456, Ordering::Relaxed);
//!
//! ```
pub use crate::dynamic_storage::*;
use crate::static_storage::file::NamedConceptConfiguration;
use crate::static_storage::file::NamedConceptRemoveError;
use core::fmt::Debug;
use core::marker::PhantomData;
pub use core::ops::Deref;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::warn;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::directory::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::shared_memory::*;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

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
            suffix: self.suffix.clone(),
            prefix: self.prefix.clone(),
            path: self.path.clone(),
            _data: PhantomData,
            type_name: self.type_name.clone(),
        }
    }
}

#[repr(C)]
struct Data<T: Send + Sync + Debug> {
    version: IoxAtomicU64,
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
        self.path = value.clone();
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
            storage_name: storage_name.clone(),
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
        let msg = "Failed to open posix_shared_memory::DynamicStorage";

        let full_name = self.config.path_for(&self.storage_name).file_name();
        let mut wait_for_read_write_access = fail!(from self, when AdaptiveWaitBuilder::new().create(),
                                    with DynamicStorageOpenError::InternalError,
                                    "{} since the AdaptiveWait could not be initialized.", msg);

        let mut elapsed_time = Duration::ZERO;
        let shm = loop {
            match SharedMemoryBuilder::new(&full_name).open_existing(AccessMode::ReadWrite) {
                Ok(v) => break v,
                Err(SharedMemoryCreationError::DoesNotExist) => {
                    fail!(from self, with DynamicStorageOpenError::DoesNotExist,
                    "{} since a shared memory with that name does not exists.", msg);
                }
                Err(SharedMemoryCreationError::InsufficientPermissions) => {
                    if elapsed_time >= self.timeout {
                        fail!(from self, with DynamicStorageOpenError::InitializationNotYetFinalized,
                        "{} since it is not readable - (it is not initialized after {:?}).",
                        msg, self.timeout);
                    }
                }
                Err(_) => {
                    fail!(from self, with DynamicStorageOpenError::InternalError, "{} since the underlying shared memory could not be opened.", msg);
                }
            };

            elapsed_time = fail!(from self, when wait_for_read_write_access.wait(),
                                    with DynamicStorageOpenError::InternalError,
                                    "{} since the adaptive wait call failed.", msg);
        };

        let init_state = shm.base_address().as_ptr() as *const Data<T>;

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
            shm,
            name: self.storage_name.clone(),
            _phantom_data: PhantomData,
        })
    }

    fn create_impl(&mut self) -> Result<SharedMemory, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic_storage::PosixSharedMemory";

        let full_name = self.config.path_for(&self.storage_name).file_name();
        let shm = match SharedMemoryBuilder::new(&full_name)
            .creation_mode(CreationMode::CreateExclusive)
            // posix shared memory is always aligned to the greatest possible value (PAGE_SIZE)
            // therefore we do not have to add additional alignment space for T
            .size(core::mem::size_of::<Data<T>>() + self.supplementary_size)
            .permission(INIT_PERMISSIONS)
            .zero_memory(false)
            .has_ownership(self.has_ownership)
            .create()
        {
            Ok(v) => v,
            Err(SharedMemoryCreationError::AlreadyExist) => {
                fail!(from self, with DynamicStorageCreateError::AlreadyExists,
                    "{} since a shared memory with the name already exists.", msg);
            }
            Err(SharedMemoryCreationError::InsufficientPermissions) => {
                fail!(from self, with DynamicStorageCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(_) => {
                fail!(from self, with DynamicStorageCreateError::InternalError,
                    "{} since the underlying shared memory could not be created.", msg);
            }
        };

        Ok(shm)
    }

    fn init_impl(
        &mut self,
        mut shm: SharedMemory,
        initial_value: T,
    ) -> Result<Storage<T>, DynamicStorageCreateError> {
        let msg = "Failed to init dynamic_storage::PosixSharedMemory";
        let value = shm.base_address().as_ptr() as *mut Data<T>;
        let version_ptr = unsafe { core::ptr::addr_of_mut!((*value).version) };
        unsafe { version_ptr.write(IoxAtomicU64::new(0)) };

        unsafe { core::ptr::addr_of_mut!((*value).data).write(initial_value) };
        unsafe {
            core::ptr::addr_of_mut!((*value).call_drop_on_destruction)
                .write(self.call_drop_on_destruction)
        };

        let supplementary_start =
            (shm.base_address().as_ptr() as usize + core::mem::size_of::<Data<T>>()) as *mut u8;
        let supplementary_len = shm.size() - core::mem::size_of::<Data<T>>();

        let mut allocator = BumpAllocator::new(
            unsafe { NonNull::new_unchecked(supplementary_start) },
            supplementary_len,
        );

        let origin = format!("{self:?}");
        if !self
            .initializer
            .call(unsafe { &mut (*value).data }, &mut allocator)
        {
            unsafe { core::ptr::drop_in_place(value) };
            shm.acquire_ownership();
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

        if let Err(e) = shm.set_permission(FINAL_PERMISSIONS) {
            unsafe { core::ptr::drop_in_place(value) };
            shm.acquire_ownership();
            fail!(from origin, with DynamicStorageCreateError::InternalError,
                "{} since the final permissions could not be applied to the underlying shared memory ({:?}).",
                msg, e);
        }

        Ok(Storage {
            shm,
            name: self.storage_name.clone(),
            _phantom_data: PhantomData,
        })
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

/// Implements [`DynamicStorage`] for POSIX shared memory. It is built by
/// [`Builder`].
#[derive(Debug)]
pub struct Storage<T: Debug + Send + Sync> {
    shm: SharedMemory,
    name: FileName,
    _phantom_data: PhantomData<T>,
}

unsafe impl<T: Debug + Send + Sync> Send for Storage<T> {}
unsafe impl<T: Debug + Send + Sync> Sync for Storage<T> {}

impl<T: Debug + Send + Sync> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.shm.has_ownership() {
            let data = unsafe { &mut (*(self.shm.base_address().as_ptr() as *mut Data<T>)) };
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
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let full_name = cfg.path_for(name).file_name();

        Ok(iceoryx2_bb_posix::shared_memory::SharedMemory::does_exist(
            &full_name,
        ))
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
        let full_name = cfg.path_for(name).file_name();
        let msg = "Unable to remove dynamic_storage::posix_shared_memory";
        let origin = "dynamic_storage::posix_shared_memory::Storage::remove_cfg()";

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
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl<T: Send + Sync + Debug> DynamicStorage<T> for Storage<T> {
    type Builder<'builder> = Builder<'builder, T>;

    fn does_support_persistency() -> bool {
        SharedMemory::does_support_persistency()
    }

    fn acquire_ownership(&self) {
        self.shm.acquire_ownership()
    }

    fn get(&self) -> &T {
        unsafe { &(*(self.shm.base_address().as_ptr() as *const Data<T>)).data }
    }

    fn has_ownership(&self) -> bool {
        self.shm.has_ownership()
    }

    fn release_ownership(&self) {
        self.shm.release_ownership()
    }

    unsafe fn __internal_set_type_name_in_config(
        config: &mut Self::Configuration,
        type_name: &str,
    ) {
        config.type_name = type_name.to_string()
    }
}
