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

use std::{
    any::Any,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub use crate::event::*;
use crate::static_storage::file::NamedConceptConfiguration;
use iceoryx2_bb_container::queue::FixedSizeQueue;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::{
    condition_variable::{ConditionVariable, ConditionVariableBuilder, ConditionVariableData},
    mutex::*,
};
pub use iceoryx2_bb_system_types::file_name::FileName;
pub use iceoryx2_bb_system_types::file_path::FilePath;
use once_cell::sync::Lazy;
use ouroboros::self_referencing;

const DEFAULT_CAPACITY: usize = 2048;

#[self_referencing]
#[derive(Debug)]
struct Management {
    mtx_handle: MutexHandle<ConditionVariableData<FixedSizeQueue<TriggerId, DEFAULT_CAPACITY>>>,
    #[borrows(mtx_handle)]
    #[covariant]
    cvar: ConditionVariable<'this, FixedSizeQueue<TriggerId, DEFAULT_CAPACITY>>,
}

#[derive(Debug)]
struct StorageEntry {
    content: Arc<dyn Any + Send + Sync>,
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            path: EventImpl::default_path_hint(),
            suffix: EventImpl::default_suffix(),
            prefix: EventImpl::default_prefix(),
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
pub struct Duplex {
    name: FileName,
    management: Arc<Management>,
    has_ownership: bool,
    config: Configuration,
}

impl NamedConcept for Duplex {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl Notifier for Duplex {
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError> {
        let msg = "Unable to notify event::process_local::Listener";
        let push_successful = AtomicBool::new(false);

        if self
            .management
            .borrow_cvar()
            .modify_notify_one(|queue| push_successful.store(queue.push(id), Ordering::Relaxed))
            .is_err()
        {
            fail!(from self, with NotifierNotifyError::InternalFailure,
                    "{} due to a failure in the underlying condition variable.", msg);
        }

        if !push_successful.load(Ordering::Relaxed) {
            fail!(from self, with NotifierNotifyError::FailedToDeliverSignal,
                    "{} since the underlying queue is full.", msg);
        }

        Ok(())
    }
}

impl Drop for Duplex {
    fn drop(&mut self) {
        if self.has_ownership {
            fatal_panic!(from self, when unsafe { EventImpl::remove_cfg(&self.name, &self.config) },
                "This should never happen! Unable to remove resources.");
        }
    }
}

impl Listener for Duplex {
    fn try_wait(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.timed_wait(Duration::ZERO)
    }

    fn timed_wait(&self, timeout: Duration) -> Result<Option<TriggerId>, ListenerWaitError> {
        let msg = "Failed to timed_wait";
        match self
            .management
            .as_ref()
            .borrow_cvar()
            .timed_wait_while(timeout)
        {
            Err(v) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                        "{} due to an internal failure in the underlying condition variable ({:?}).", msg, v);
            }
            Ok(None) => Ok(None),
            Ok(Some(mut guard)) => Ok(guard.value.pop()),
        }
    }

    fn blocking_wait(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        let msg = "Failed to blocking_wait";
        match self.management.as_ref().borrow_cvar().blocking_wait_while() {
            Err(v) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                        "{} due to an internal failure in the underlying condition variable ({:?}).", msg, v);
            }
            Ok(mut guard) => Ok(guard.value.pop()),
        }
    }

    fn try_wait_all<F: FnMut(TriggerId)>(&self, callback: F) -> Result<(), ListenerWaitError> {
        self.timed_wait_all(callback, Duration::ZERO)
    }

    fn timed_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        let msg = "Failed to try_wait";
        match self
            .management
            .as_ref()
            .borrow_cvar()
            .timed_wait_while(timeout)
        {
            Err(v) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                        "{} due to an internal failure in the underlying condition variable ({:?}).", msg, v);
            }
            Ok(None) => Ok(()),
            Ok(Some(mut guard)) => {
                while let Some(id) = guard.value.pop() {
                    callback(id)
                }
                Ok(())
            }
        }
    }

    fn blocking_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
    ) -> Result<(), ListenerWaitError> {
        let msg = "Failed to blocking_wait";
        match self.management.as_ref().borrow_cvar().blocking_wait_while() {
            Err(v) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                        "{} due to an internal failure in the underlying condition variable ({:?}).", msg, v);
            }
            Ok(mut guard) => {
                while let Some(id) = guard.value.pop() {
                    callback(id)
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for Builder {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl NotifierBuilder<EventImpl> for Builder {
    fn timeout(self, _timeout: Duration) -> Self {
        self
    }

    fn open(self) -> Result<Duplex, NotifierCreateError> {
        let msg = "Failed to open event";

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with NotifierCreateError::InternalFailure,
            "{} due to a failure while acquiring the lock.", msg
        );
        let full_path = self.config.path_for(&self.name);
        let mut entry = guard.get_mut(&full_path);
        if entry.is_none() {
            fail!(from self, with NotifierCreateError::DoesNotExist,
                "{} since the event does not exist.", msg);
        }

        Ok(Duplex {
            name: self.name,
            management: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<Management>()
                .unwrap(),
            has_ownership: false,
            config: self.config,
        })
    }
}

impl ListenerBuilder<EventImpl> for Builder {
    fn trigger_id_max(self, _id: TriggerId) -> Self {
        self
    }

    fn create(self) -> Result<Duplex, ListenerCreateError> {
        let msg = "Failed to create event";

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with ListenerCreateError::InternalFailure,
            "{} due to a failure while acquiring the lock.", msg
        );

        let full_path = self.config.path_for(&self.name);
        let entry = guard.get_mut(&full_path);
        if entry.is_some() {
            fail!(from self, with ListenerCreateError::AlreadyExists,
                "{} since the event does already exist.", msg);
        }

        let storage_details = Arc::new(
            ManagementBuilder {
                mtx_handle: MutexHandle::new(),
                cvar_builder: |mtx_handle: &MutexHandle<
                    ConditionVariableData<FixedSizeQueue<TriggerId, DEFAULT_CAPACITY>>,
                >| {
                    ConditionVariableBuilder::new()
                        .is_interprocess_capable(false)
                        .create_condition_variable(
                            FixedSizeQueue::new(),
                            |queue| !queue.is_empty(),
                            mtx_handle,
                        )
                        .unwrap()
                },
            }
            .build(),
        );

        guard.insert(
            full_path,
            StorageEntry {
                content: storage_details,
            },
        );

        let mut entry = guard.get_mut(&full_path);
        entry
            .as_mut()
            .unwrap()
            .content
            .clone()
            .downcast::<Management>()
            .unwrap();

        Ok(Duplex {
            name: self.name,
            management: entry
                .as_mut()
                .unwrap()
                .content
                .clone()
                .downcast::<Management>()
                .unwrap(),
            has_ownership: true,
            config: self.config,
        })
    }
}

#[derive(Debug)]
pub struct EventImpl {}

impl crate::event::Event for EventImpl {
    type Notifier = Duplex;
    type Listener = Duplex;
    type NotifierBuilder = Builder;
    type ListenerBuilder = Builder;
}

impl NamedConceptMgmt for EventImpl {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        let msg = "Unable to check if event::process_local exists";
        let origin = "event::process_local::Event::does_exist_cfg()";
        let guard = fatal_panic!(from origin,
                        when PROCESS_LOCAL_STORAGE.lock(),
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&cfg.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        let msg = "Unable to list all event::process_local";
        let origin = "event::process_local::Event::list_cfg()";
        let guard = fatal_panic!(from origin,
                                 when PROCESS_LOCAL_STORAGE.lock(),
                                "{} since the lock could not be acquired.", msg);

        let mut result = vec![];
        for storage_name in guard.keys() {
            if let Some(v) = cfg.extract_name_from_path(storage_name) {
                result.push(v);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);
        let msg = "Unable to remove event::process_local";
        let origin = "event::process_local::Event::remove_cfg()";

        let guard = PROCESS_LOCAL_STORAGE.lock();
        if guard.is_err() {
            fatal_panic!(from origin,
                "{} since the lock could not be acquired.", msg);
        }

        Ok(guard.unwrap().remove(&storage_name).is_some())
    }
}
