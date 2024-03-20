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

//! Process local implementation of [`DynamicStorage`]. **Cannot be used in an inter-process.**
//! context.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_cal::dynamic_storage::process_local::*;
//! use iceoryx2_cal::named_concept::*;
//! use std::sync::atomic::{AtomicI64, Ordering};
//!
//! let additional_size: usize = 1024;
//! let storage_name = FileName::new(b"myDynStorage").unwrap();
//! let storage = Builder::new(&storage_name)
//!                 .supplementary_size(additional_size)
//!                 .create(AtomicI64::new(444)).unwrap();
//!
//! // at some other place in the local process, can be another thread
//! let reader = Builder::<AtomicI64>::new(&storage_name)
//!                                 .open().unwrap();
//!
//! println!("Old value: {}", reader.get().load(Ordering::Relaxed));
//! reader.get().store(456, Ordering::Relaxed);
//! println!("New value: {}", reader.get().load(Ordering::Relaxed));
//! ```

use iceoryx2_bb_elementary::allocator::BaseAllocator;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;
use iceoryx2_bb_posix::mutex::{Mutex, MutexBuilder, MutexGuard, MutexHandle};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use once_cell::sync::Lazy;
use std::alloc::Layout;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use crate::dynamic_storage::*;
use crate::static_storage::file::NamedConceptConfiguration;

use self::dynamic_storage_configuration::DynamicStorageConfiguration;

#[derive(Debug)]
struct StorageEntry {
    content: Arc<dyn Any + Send + Sync>,
}

#[derive(Debug)]
struct StorageDetails<T> {
    data_ptr: *mut T,
    layout: Layout,
}

#[derive(PartialEq, Eq, Copy, Debug)]
pub struct Configuration<T: Send + Sync + Debug> {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
    _data: PhantomData<T>,
}

impl<T: Send + Sync + Debug> Clone for Configuration<T> {
    fn clone(&self) -> Self {
        Self {
            suffix: self.suffix,
            prefix: self.prefix,
            path_hint: self.path_hint,
            _data: PhantomData,
        }
    }
}

impl<T: Send + Sync + Debug> Default for Configuration<T> {
    fn default() -> Self {
        Self {
            suffix: Storage::<()>::default_suffix(),
            prefix: Storage::<()>::default_prefix(),
            path_hint: Storage::<()>::default_path_hint(),
            _data: PhantomData,
        }
    }
}

impl<T: Send + Sync + Debug> DynamicStorageConfiguration<T> for Configuration<T> {}

impl<T: Send + Sync + Debug> NamedConceptConfiguration for Configuration<T> {
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

impl<T> StorageDetails<T> {
    fn new(value: T, additional_size: u64) -> Self {
        let size = std::mem::size_of::<T>() + additional_size as usize;
        let align = std::mem::align_of::<T>();
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        let new_self = Self {
            data_ptr: fatal_panic!(from "StorageDetails::new", when HeapAllocator::new()
                .allocate(layout), "Failed to allocate {} bytes for dynamic global storage.", size)
            .as_ptr() as *mut T,
            layout,
        };
        unsafe { new_self.data_ptr.write(value) };
        new_self
    }
}

impl<T> Drop for StorageDetails<T> {
    fn drop(&mut self) {
        unsafe {
            HeapAllocator::new().deallocate(
                NonNull::new_unchecked(self.data_ptr as *mut u8),
                self.layout,
            );
        };
    }
}

unsafe impl<T> Send for StorageDetails<T> {}
unsafe impl<T> Sync for StorageDetails<T> {}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, StorageEntry>>> =
    Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashMap<FilePath, StorageEntry>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global dynamic storage");
    }

    result.unwrap()
});

#[derive(Debug)]
pub struct Storage<T: Send + Sync + Debug + 'static> {
    name: FileName,
    data: Arc<StorageDetails<T>>,
    has_ownership: AtomicBool,
    config: Configuration<T>,
}

impl<T: Send + Sync + Debug + 'static> NamedConcept for Storage<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Send + Sync + Debug + 'static> NamedConceptMgmt for Storage<T> {
    type Configuration = Configuration<T>;

    fn does_exist_cfg(
        name: &FileName,
        config: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let msg = "Unable to check if dynamic_storage::process_local exists";
        let origin = "dynamic_storage::process_local::Storage::does_exist_cfg()";

        let guard = fatal_panic!(from origin,
                        when PROCESS_LOCAL_STORAGE.lock(),
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&config.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn list_cfg(
        config: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let msg = "Unable to list all dynamic_storage::process_local";
        let origin = "dynamic_storage::process_local::Storage::list_cfg()";

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
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);

        let msg = "Unable to remove dynamic_storage::process_local";
        let origin = "dynamic_storage::process_local::Storage::remove_cfg()";

        let mut guard = fatal_panic!(from origin, when PROCESS_LOCAL_STORAGE.lock()
                                , "{} since the lock could not be acquired.", msg);

        let mut entry = guard.get_mut(&storage_name);
        if entry.is_none() {
            return Ok(false);
        }

        std::ptr::drop_in_place(
            entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<StorageDetails<T>>()
                .unwrap()
                .data_ptr,
        );

        Ok(guard.remove(&storage_name).is_some())
    }
}

impl<T: Send + Sync + Debug + 'static> DynamicStorage<T> for Storage<T> {
    type Builder<'builder> = Builder<'builder, T>;

    fn does_support_persistency() -> bool {
        true
    }

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }

    fn get(&self) -> &T {
        unsafe { &*self.data.data_ptr }
    }

    fn has_ownership(&self) -> bool {
        self.has_ownership.load(Ordering::Relaxed)
    }

    fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed)
    }
}

impl<T: Send + Sync + Debug + 'static> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.has_ownership() {
            match unsafe { Self::remove_cfg(&self.name, &self.config) } {
                Ok(false) | Err(_) => {
                    fatal_panic!(from self, "This should never happen! Unable to remove dynamic storage");
                }
                Ok(_) => (),
            }
        }
    }
}

#[derive(Debug)]
pub struct Builder<'builder, T: Send + Sync + Debug> {
    name: FileName,
    supplementary_size: usize,
    has_ownership: bool,
    config: Configuration<T>,
    initializer: Initializer<'builder, T>,
    _phantom_data: PhantomData<T>,
}

impl<'builder, T: Send + Sync + Debug + 'static> NamedConceptBuilder<Storage<T>>
    for Builder<'builder, T>
{
    fn new(storage_name: &FileName) -> Self {
        Self {
            name: *storage_name,
            has_ownership: true,
            supplementary_size: 0,
            config: Configuration::default(),
            initializer: Initializer::new(|_, _| true),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration<T>) -> Self {
        self.config = config.clone();
        self
    }
}

impl<'builder, T: Send + Sync + Debug + 'static> Builder<'builder, T> {
    fn open_impl(
        &self,
        guard: &mut MutexGuard<'static, 'static, HashMap<FilePath, StorageEntry>>,
    ) -> Result<Storage<T>, DynamicStorageOpenError> {
        let msg = "Failed to open dynamic storage";

        let full_path = self.config.path_for(&self.name);
        let mut entry = guard.get_mut(&full_path);
        if entry.is_none() {
            fail!(from self, with DynamicStorageOpenError::DoesNotExist,
                "{} since the storage does not exist.", msg);
        }

        Ok(Storage::<T> {
            name: self.name,
            data: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<StorageDetails<T>>()
                .unwrap(),
            has_ownership: AtomicBool::new(false),
            config: self.config.clone(),
        })
    }

    fn create_impl(
        &mut self,
        guard: &mut MutexGuard<'static, 'static, HashMap<FilePath, StorageEntry>>,
        initial_value: T,
    ) -> Result<Storage<T>, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic storage";

        let full_path = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_path);
        if entry.is_some() {
            fail!(from self, with DynamicStorageCreateError::AlreadyExists,
                "{} since the storage does already exist.", msg);
        }

        let storage_details = Arc::new(StorageDetails::new(
            initial_value,
            self.supplementary_size as u64,
        ));

        let value = storage_details.data_ptr;
        let supplementary_start = (value as usize + std::mem::size_of::<T>()) as *mut u8;

        let mut allocator = BumpAllocator::new(
            unsafe { NonNull::new_unchecked(supplementary_start) },
            self.supplementary_size,
        );

        let origin = format!("{:?}", self);
        if !self
            .initializer
            .call(unsafe { &mut *value }, &mut allocator)
        {
            fail!(from origin, with DynamicStorageCreateError::InitializationFailed,
                "{} since the initialization of the underlying construct failed.", msg);
        }

        guard.insert(
            full_path,
            StorageEntry {
                content: storage_details,
            },
        );

        let mut entry = guard.get_mut(&full_path);
        Ok(Storage::<T> {
            name: self.name,
            data: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<StorageDetails<T>>()
                .unwrap(),
            has_ownership: AtomicBool::new(self.has_ownership),
            config: self.config.clone(),
        })
    }
}

impl<'builder, T: Send + Sync + Debug + 'static> DynamicStorageBuilder<'builder, T, Storage<T>>
    for Builder<'builder, T>
{
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

    fn timeout(self, _value: Duration) -> Self {
        self
    }

    fn supplementary_size(mut self, value: usize) -> Self {
        self.supplementary_size = value;
        self
    }

    fn open(self) -> Result<Storage<T>, DynamicStorageOpenError> {
        let msg = "Failed to open dynamic storage";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with DynamicStorageOpenError::InternalError,
            "{} due to a failure while acquiring the lock.", msg
        );

        self.open_impl(&mut guard)
    }

    fn create(mut self, initial_value: T) -> Result<Storage<T>, DynamicStorageCreateError> {
        let msg = "Failed to create dynamic storage";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with DynamicStorageCreateError::InternalError,
            "{} due to a failure while acquiring the lock.", msg
        );

        self.create_impl(&mut guard, initial_value)
    }

    fn open_or_create(
        mut self,
        initial_value: T,
    ) -> Result<Storage<T>, DynamicStorageOpenOrCreateError> {
        let msg = "Failed to open or create dynamic storage";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with DynamicStorageOpenOrCreateError::DynamicStorageOpenError(DynamicStorageOpenError::InternalError),
            "{} due to a failure while acquiring the lock.", msg
        );

        match self.open_impl(&mut guard) {
            Ok(storage) => Ok(storage),
            Err(DynamicStorageOpenError::DoesNotExist) => {
                match self.create_impl(&mut guard, initial_value) {
                    Ok(storage) => Ok(storage),
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
