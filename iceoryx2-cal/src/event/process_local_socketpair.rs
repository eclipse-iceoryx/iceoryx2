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

use std::{collections::HashMap, time::Duration};

pub use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptorBased,
    file_descriptor_set::SynchronousMultiplexing,
    mutex::{Handle, Mutex, MutexBuilder, MutexHandle},
};
pub use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath, path::Path};
use once_cell::sync::Lazy;

use crate::named_concept::NamedConceptConfiguration;

use super::{
    ListenerCreateError, ListenerWaitError, NamedConcept, NamedConceptBuilder, NamedConceptMgmt,
    NotifierCreateError, NotifierNotifyError, TriggerId,
};

#[derive(Debug)]
struct StorageEntry {}

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
}

#[derive(Debug)]
struct EventImpl {}

impl NamedConceptMgmt for EventImpl {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        todo!()
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        todo!()
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        todo!()
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        todo!()
    }
}

impl crate::event::Event for EventImpl {
    type Notifier = Notifier;
    type Listener = Listener;
    type NotifierBuilder = NotifierBuilder;
    type ListenerBuilder = ListenerBuilder;
}

impl EventImpl {
    fn default_path_hint() -> Path {
        Path::new(b"").unwrap()
    }

    fn default_prefix() -> FileName {
        FileName::new(b"iox2").unwrap()
    }

    fn default_suffix() -> FileName {
        FileName::new(b".event").unwrap()
    }
}

#[derive(Debug)]
pub struct Notifier {
    name: FileName,
}

impl NamedConcept for Notifier {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl crate::event::Notifier for Notifier {
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NotifierBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for NotifierBuilder {
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

impl crate::event::NotifierBuilder<EventImpl> for NotifierBuilder {
    fn timeout(self, _timeout: Duration) -> Self {
        todo!()
    }

    fn open(self) -> Result<Notifier, NotifierCreateError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Listener {
    name: FileName,
}

impl FileDescriptorBased for Listener {
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        todo!()
    }
}

impl SynchronousMultiplexing for Listener {}

impl NamedConcept for Listener {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl crate::event::Listener for Listener {
    fn try_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        todo!()
    }

    fn timed_wait_one(
        &self,
        timeout: core::time::Duration,
    ) -> Result<Option<TriggerId>, ListenerWaitError> {
        todo!()
    }

    fn blocking_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        todo!()
    }

    fn try_wait_all<F: FnMut(TriggerId)>(&self, mut callback: F) -> Result<(), ListenerWaitError> {
        todo!()
    }

    fn timed_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        todo!()
    }

    fn blocking_wait_all<F: FnMut(TriggerId)>(
        &self,
        mut callback: F,
    ) -> Result<(), ListenerWaitError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct ListenerBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for ListenerBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &<EventImpl as super::NamedConceptMgmt>::Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl crate::event::ListenerBuilder<EventImpl> for ListenerBuilder {
    fn trigger_id_max(self, _id: TriggerId) -> Self {
        self
    }

    fn create(self) -> Result<Listener, ListenerCreateError> {
        todo!()
    }
}
