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

//! Process local implementation of [`StaticStorage`]. Cannot be used in an
//! inter-process context.
//!
//! # Example
//!
//! ```
//! use iceoryx2_cal::static_storage::process_local::*;
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let mut content = "look over there!".to_string();
//!
//! let storage_name = FileName::new(b"someInternalStorage").unwrap();
//! let owner = Builder::new(&storage_name)
//!                 .create(content.as_bytes()).unwrap();
//!
//! // at some other place in the local process, can be another thread
//! let initialization_timeout = core::time::Duration::from_millis(100);
//! let reader = Builder::new(&storage_name)
//!                 .open(initialization_timeout).unwrap();
//!
//! let content_length = reader.len();
//! let mut content = String::from_utf8(vec![b' '; content_length as usize]).unwrap();
//! reader.read(unsafe { content.as_mut_vec() }.as_mut_slice()).unwrap();
//!
//! println!("Storage {} content: {}", reader.name(), content);
//! ```

pub use crate::named_concept::*;
pub use crate::static_storage::*;

use alloc::sync::Arc;
use core::sync::atomic::Ordering;

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug)]
struct StorageContent {
    is_locked: bool,
    value: Vec<u8>,
}

#[derive(Debug)]
struct StorageEntry {
    content: Arc<StorageContent>,
}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, StorageEntry>>> =
    Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashMap<FilePath, StorageEntry>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global static storage");
    }

    result.unwrap()
});

#[derive(Clone, Debug)]
pub struct Configuration {
    path: Path,
    suffix: FileName,
    prefix: FileName,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            path: Storage::default_path_hint(),
            suffix: Storage::default_suffix(),
            prefix: Storage::default_prefix(),
        }
    }
}

impl NamedConceptConfiguration for Configuration {
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
}

impl StaticStorageConfiguration for Configuration {}

#[derive(Debug)]
pub struct Locked {
    storage: Storage,
}

impl NamedConcept for Locked {
    fn name(&self) -> &FileName {
        self.storage.name()
    }
}

impl StaticStorageLocked<Storage> for Locked {
    fn unlock(self, contents: &[u8]) -> Result<Storage, StaticStorageUnlockError> {
        let msg = "Failed to unlock storage";
        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
                with StaticStorageUnlockError::InternalError,
                "{} due to a failure while acquiring the lock.", msg);

        let name = self.storage.config.path_for(&self.storage.name);
        let entry = guard.get(&name);
        if entry.is_none() {
            fatal_panic!(from self,
                "{} since a storage with the name \"{}\" does not exists. The internal data structure seems to be corrupted.",
                msg, self.storage.name());
        }

        guard.insert(
            name,
            StorageEntry {
                content: Arc::new(StorageContent {
                    is_locked: false,
                    value: Vec::from(contents),
                }),
            },
        );

        Ok(self.storage)
    }
}

#[derive(Debug)]
pub struct Storage {
    name: FileName,
    has_ownership: IoxAtomicBool,
    config: Configuration,
    content: Arc<StorageContent>,
}

impl Drop for Storage {
    fn drop(&mut self) {
        if self.has_ownership.load(Ordering::Relaxed) {
            if let Err(v) = unsafe { Self::remove_cfg(&self.name, &self.config) } {
                fatal_panic!(from self, "This should never happen! Failed to remove underlying storage ({:?})", v);
            }
        }
    }
}

impl NamedConceptMgmt for Storage {
    type Configuration = Configuration;

    unsafe fn remove_cfg(
        storage_name: &FileName,
        config: &Self::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        let msg = "Unable to release static storage";
        let guard = PROCESS_LOCAL_STORAGE.lock();
        if guard.is_err() {
            fatal_panic!(from "static_storage::process_local::Storage::remove_cfg",
                "{} \"{}\" since the lock could not be acquired.", msg, storage_name);
        }

        Ok(guard
            .unwrap()
            .remove(&config.path_for(storage_name))
            .is_some())
    }

    fn list_cfg(config: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let msg = "Unable to list all static storages";
        let guard = fatal_panic!(from "static_storage::process_local::Storage::list_cfg",
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

    fn does_exist_cfg(
        storage_name: &FileName,
        config: &Self::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        let msg = "Unable to check if storage exists";
        let guard = fatal_panic!(from "static_storage::process_local::Storage::does_exist_cfg",
                        when PROCESS_LOCAL_STORAGE.lock(), "{} since the lock could not be acquired.", msg);

        match guard.get(&config.path_for(storage_name)) {
            Some(v) => match v.content.is_locked {
                true => Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp),
                false => Ok(true),
            },
            None => Ok(false),
        }
    }

    fn remove_path_hint(_value: &Path) -> Result<(), NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl NamedConcept for Storage {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl StaticStorage for Storage {
    type Builder = Builder;
    type Locked = Locked;

    fn len(&self) -> u64 {
        self.content.value.len() as u64
    }

    fn is_empty(&self) -> bool {
        self.content.value.is_empty()
    }

    fn read(&self, content: &mut [u8]) -> Result<(), StaticStorageReadError> {
        let msg = "Failed to read from storage";
        if self.content.value.len() > content.len() {
            fail!(from self, with StaticStorageReadError::BufferTooSmall,
                    "{} since the provided buffer with a size of {} bytes is too small. Require at least a size of {} bytes.",
                    msg, content.len(), self.content.value.len() );
        }

        content.clone_from_slice(self.content.value.as_slice());

        Ok(())
    }

    fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed);
    }

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    has_ownership: bool,
    config: Configuration,
}

impl NamedConceptBuilder<Storage> for Builder {
    fn new(storage_name: &FileName) -> Self {
        Self {
            has_ownership: true,
            name: storage_name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl StaticStorageBuilder<Storage> for Builder {
    fn has_ownership(mut self, value: bool) -> Self {
        self.has_ownership = value;
        self
    }

    fn open(self, timeout: Duration) -> Result<Storage, StaticStorageOpenError> {
        let msg = "Failed to open static storage";
        let mut wait_for_read_access = fail!(from self,
            when AdaptiveWaitBuilder::new().create(),
            with StaticStorageOpenError::InternalError,
            "{} since the AdaptiveWait could not be initialized.", msg);

        let mut elapsed_time = Duration::ZERO;

        loop {
            let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
                with StaticStorageOpenError::InternalError,
                "{} due to a failure while acquiring the lock.", msg);

            let entry = guard.get_mut(&self.config.path_for(&self.name));
            if entry.is_none() {
                fail!(from self, with StaticStorageOpenError::DoesNotExist,
                                "{} since the storage does not exist.", msg);
            }

            let entry = entry.unwrap();
            if entry.content.is_locked {
                if elapsed_time > timeout {
                    fail!(from self, with StaticStorageOpenError::InitializationNotYetFinalized,
                        "{} since the static storage is still being created (in locked state), try later.", msg);
                }

                elapsed_time = fail!(from self,
                    when wait_for_read_access.wait(),
                    with StaticStorageOpenError::InternalError,
                    "{} since the adaptive wait call failed.", msg);
            } else {
                return Ok(Storage {
                    name: self.name,
                    has_ownership: IoxAtomicBool::new(self.has_ownership),
                    config: self.config,
                    content: entry.content.clone(),
                });
            }
        }
    }

    fn create_locked(self) -> Result<<Storage as StaticStorage>::Locked, StaticStorageCreateError> {
        let msg = "Failed to create storage";

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
                with StaticStorageCreateError::InternalError,
                "{} due to a failure while acquiring the lock.", msg);

        let name = self.config.path_for(&self.name);
        let entry = guard.get(&name);
        if entry.is_some() {
            fail!(from self, with StaticStorageCreateError::AlreadyExists,
                "{} since a storage with the name \"{}\" does already exist.", msg, self.name);
        }

        let content = Arc::new(StorageContent {
            is_locked: true,
            value: vec![],
        });

        guard.insert(
            name,
            StorageEntry {
                content: content.clone(),
            },
        );

        Ok(Locked {
            storage: Storage {
                name: self.name,
                has_ownership: IoxAtomicBool::new(self.has_ownership),
                config: self.config,
                content,
            },
        })
    }
}
