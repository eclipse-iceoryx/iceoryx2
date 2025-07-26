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
//! use core::sync::atomic::{AtomicI64, Ordering};
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

use alloc::sync::Arc;
use core::alloc::Layout;
use core::any::Any;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;

use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use once_cell::sync::Lazy;
use std::collections::HashMap;

pub use crate::dynamic_storage::*;
use crate::named_concept::{
    NamedConceptDoesExistError, NamedConceptListError, NamedConceptRemoveError,
};
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
    call_drop_on_destruction: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Configuration<T: Send + Sync + Debug> {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
    _data: PhantomData<T>,
    type_name: String,
}

impl<T: Send + Sync + Debug> Clone for Configuration<T> {
    fn clone(&self) -> Self {
        Self {
            suffix: self.suffix.clone(),
            prefix: self.prefix.clone(),
            path_hint: self.path_hint.clone(),
            _data: PhantomData,
            type_name: self.type_name.clone(),
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
        self.path_hint = value.clone();
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }

    fn path_for(&self, value: &FileName) -> iceoryx2_bb_system_types::file_path::FilePath {
        self.path_for_with_type(value)
    }

    fn extract_name_from_file(&self, value: &FileName) -> Option<FileName> {
        self.extract_name_from_file_with_type(value)
    }
}

impl<T> StorageDetails<T> {
    fn new(value: T, additional_size: u64, call_drop_on_destruction: bool) -> Self {
        let size = core::mem::size_of::<T>() + additional_size as usize;
        let align = core::mem::align_of::<T>();
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        let new_self = Self {
            data_ptr: fatal_panic!(from "StorageDetails::new", when HeapAllocator::new()
                .allocate(layout), "Failed to allocate {} bytes for dynamic global storage.", size)
            .as_ptr() as *mut T,
            layout,
            call_drop_on_destruction,
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
    has_ownership: IoxAtomicBool,
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
    ) -> Result<bool, NamedConceptDoesExistError> {
        let msg = "Unable to check if dynamic_storage::process_local exists";
        let origin = "dynamic_storage::process_local::Storage::does_exist_cfg()";

        let guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                        with NamedConceptDoesExistError::InternalError,
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&config.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn list_cfg(config: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let msg = "Unable to list all dynamic_storage::process_local";
        let origin = "dynamic_storage::process_local::Storage::list_cfg()";

        let guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                                with NamedConceptListError::InternalError,
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

        let msg = "Unable to remove dynamic_storage::process_local";
        let origin = "dynamic_storage::process_local::Storage::remove_cfg()";

        let mut guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                                with NamedConceptRemoveError::InternalError,
                                "{} since the lock could not be acquired.", msg);

        let mut entry = guard.get_mut(&storage_name);
        if entry.is_none() {
            return Ok(false);
        }

        let details = entry
            .as_mut()
            .unwrap()
            .content
            .clone()
            .downcast::<StorageDetails<T>>()
            .unwrap();

        if details.call_drop_on_destruction {
            core::ptr::drop_in_place(details.data_ptr);
        }

        Ok(guard.remove(&storage_name).is_some())
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
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

    unsafe fn __internal_set_type_name_in_config(
        config: &mut Self::Configuration,
        type_name: &str,
    ) {
        config.type_name = type_name.to_string()
    }
}

impl<T: Send + Sync + Debug + 'static> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.has_ownership() {
            match unsafe { Self::remove_cfg(&self.name, &self.config) } {
                Ok(false) => {
                    fatal_panic!(from self, "This should never happen! Unable to remove dynamic storage since it does not exist.");
                }
                Err(e) => {
                    fatal_panic!(from self, "This should never happen! Unable to remove dynamic storage ({:?}).", e);
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
    call_drop_on_destruction: bool,
    config: Configuration<T>,
    initializer: Initializer<'builder, T>,
    _phantom_data: PhantomData<T>,
}

impl<T: Send + Sync + Debug + 'static> NamedConceptBuilder<Storage<T>> for Builder<'_, T> {
    fn new(storage_name: &FileName) -> Self {
        Self {
            name: storage_name.clone(),
            has_ownership: true,
            call_drop_on_destruction: true,
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

impl<T: Send + Sync + Debug + 'static> Builder<'_, T> {
    fn open_impl(
        &self,
        guard: &mut MutexGuard<'static, HashMap<FilePath, StorageEntry>>,
    ) -> Result<Storage<T>, DynamicStorageOpenError> {
        let msg = "Failed to open dynamic storage";

        let full_path = self.config.path_for(&self.name);
        let mut entry = guard.get_mut(&full_path);
        if entry.is_none() {
            fail!(from self, with DynamicStorageOpenError::DoesNotExist,
                "{} since the storage does not exist.", msg);
        }

        Ok(Storage::<T> {
            name: self.name.clone(),
            data: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<StorageDetails<T>>()
                .unwrap(),
            has_ownership: IoxAtomicBool::new(false),
            config: self.config.clone(),
        })
    }

    fn create_impl(
        &mut self,
        guard: &mut MutexGuard<'static, HashMap<FilePath, StorageEntry>>,
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
            self.call_drop_on_destruction,
        ));

        let value = storage_details.data_ptr;
        let supplementary_start = (value as usize + core::mem::size_of::<T>()) as *mut u8;

        let mut allocator = BumpAllocator::new(
            unsafe { NonNull::new_unchecked(supplementary_start) },
            self.supplementary_size,
        );

        let origin = format!("{self:?}");
        if !self
            .initializer
            .call(unsafe { &mut *value }, &mut allocator)
        {
            fail!(from origin, with DynamicStorageCreateError::InitializationFailed,
                "{} since the initialization of the underlying construct failed.", msg);
        }

        guard.insert(
            full_path.clone(),
            StorageEntry {
                content: storage_details,
            },
        );

        let mut entry = guard.get_mut(&full_path);
        Ok(Storage::<T> {
            name: self.name.clone(),
            data: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<StorageDetails<T>>()
                .unwrap(),
            has_ownership: IoxAtomicBool::new(self.has_ownership),
            config: self.config.clone(),
        })
    }
}

impl<'builder, T: Send + Sync + Debug + 'static> DynamicStorageBuilder<'builder, T, Storage<T>>
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
