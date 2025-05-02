// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use std::collections::HashSet;

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath, path::Path};
use once_cell::sync::Lazy;

use crate::{
    monitoring::{MonitoringCreateCleanerError, MonitoringCreateTokenError, MonitoringStateError},
    named_concept::NamedConceptConfiguration,
};

use super::{
    Monitoring, MonitoringBuilder, MonitoringCleaner, MonitoringMonitor, MonitoringToken,
    NamedConcept, NamedConceptBuilder, NamedConceptMgmt, State,
};

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashSet<FilePath>>> = Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashSet<FilePath>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashSet::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global monitoring storage");
    }

    result.unwrap()
});

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: ProcessLocalMonitoring::default_suffix(),
            prefix: ProcessLocalMonitoring::default_prefix(),
            path_hint: ProcessLocalMonitoring::default_path_hint(),
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

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.path_hint = value.clone();
        self
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

#[derive(Debug)]
pub struct ProcessLocalMonitoring {}

impl NamedConceptMgmt for ProcessLocalMonitoring {
    type Configuration = Configuration;

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        let msg = "Unable to list all monitoring::process_local";
        let origin = "monitoring::process_local::ProcessLocalMonitoring::list_cfg()";

        let guard = fatal_panic!(from origin,
                                 when PROCESS_LOCAL_STORAGE.lock(),
                                "{} since the lock could not be acquired.", msg);

        let mut result = vec![];
        for storage_name in guard.iter() {
            if let Some(v) = cfg.extract_name_from_path(storage_name) {
                result.push(v);
            }
        }

        Ok(result)
    }

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        let msg = "Unable to check if monitoring::process_local exists";
        let origin = "monitoring::process_local::ProcessLocalMonitoring::does_exist_cfg()";

        let guard = fatal_panic!(from origin,
                        when PROCESS_LOCAL_STORAGE.lock(),
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&cfg.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);
        let msg = "Unable to remove monitoring::process_local";
        let origin = "monitoring::process_local::ProcessLocalMonitoring::remove_cfg()";

        let guard = PROCESS_LOCAL_STORAGE.lock();
        if guard.is_err() {
            fatal_panic!(from origin,
                "{} since the lock could not be acquired.", msg);
        }

        Ok(guard.unwrap().remove(&storage_name))
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl Monitoring for ProcessLocalMonitoring {
    type Token = Token;
    type Monitor = Monitor;
    type Cleaner = Cleaner;
    type Builder = Builder;
}

#[derive(Debug)]
pub struct Cleaner {
    name: FileName,
}

impl NamedConcept for Cleaner {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringCleaner for Cleaner {
    fn abandon(self) {}
}

#[derive(Debug)]
pub struct Token {
    name: FileName,
    config: Configuration,
}

impl NamedConcept for Token {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringToken for Token {}

impl Drop for Token {
    fn drop(&mut self) {
        let msg = "Failed to remove";

        let mut guard = fatal_panic!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            "{} due to a failure while acquiring the lock.", msg);

        let full_name = self.config.path_for(&self.name);
        if !guard.remove(&full_name) {
            fatal_panic!(from self,
                "{} since the entry was not existing anymore. This should never happen!", msg);
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    name: FileName,
    full_name: FilePath,
}

impl NamedConcept for Monitor {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringMonitor for Monitor {
    fn state(&self) -> Result<super::State, super::MonitoringStateError> {
        let msg = "Failed to acquire state of monitor";

        let guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with MonitoringStateError::InternalError,
            "{} due to a failure while acquiring the lock.", msg);

        match guard.get(&self.full_name) {
            Some(_) => Ok(State::Alive),
            None => Ok(State::DoesNotExist),
        }
    }
}

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<ProcessLocalMonitoring> for Builder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(
        mut self,
        config: &<ProcessLocalMonitoring as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.config = config.clone();
        self
    }
}

impl MonitoringBuilder<ProcessLocalMonitoring> for Builder {
    fn token(
        self,
    ) -> Result<<ProcessLocalMonitoring as Monitoring>::Token, super::MonitoringCreateTokenError>
    {
        let msg = "Failed to create monitoring token";

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with MonitoringCreateTokenError::InternalError,
            "{} due to a failure while acquiring the lock.", msg);

        let full_name = self.config.path_for(&self.name);
        if guard.contains(&full_name) {
            fail!(from self, with MonitoringCreateTokenError::AlreadyExists,
                "{} since the token already exists.", msg);
        }

        guard.insert(full_name);

        Ok(Token {
            name: self.name,
            config: self.config,
        })
    }

    fn monitor(
        self,
    ) -> Result<<ProcessLocalMonitoring as Monitoring>::Monitor, super::MonitoringCreateMonitorError>
    {
        let full_name = self.config.path_for(&self.name);

        Ok(Monitor {
            name: self.name,
            full_name,
        })
    }

    fn cleaner(
        self,
    ) -> Result<<ProcessLocalMonitoring as Monitoring>::Cleaner, super::MonitoringCreateCleanerError>
    {
        let msg = "Failed to create monitoring cleaner";

        let guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with MonitoringCreateCleanerError::InternalError,
            "{} due to a failure while acquiring the lock.", msg);

        let full_name = self.config.path_for(&self.name);

        match guard.get(&full_name) {
            Some(_) => {
                fail!(from self, with MonitoringCreateCleanerError::InstanceStillAlive,
                    "{} since the instance is still alive.", msg);
            }
            None => {
                fail!(from self, with MonitoringCreateCleanerError::DoesNotExist,
                    "{} since the instance does not exist.", msg);
            }
        }
    }
}
