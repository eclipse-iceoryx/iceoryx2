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

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    directory::{Directory, DirectoryOpenError, DirectoryReadError},
    file::{File, FileRemoveError},
    file_type::FileType,
    process_state::{
        ProcessCleaner, ProcessCleanerCreateError, ProcessGuard, ProcessGuardCreateError,
        ProcessMonitor, ProcessMonitorCreateError, ProcessMonitorStateError, ProcessState,
    },
    testing::__internal_process_guard_staged_death,
};
use iceoryx2_bb_system_types::{file_name::FileName, path::Path};

use crate::{
    monitoring::{MonitoringCreateCleanerError, MonitoringCreateMonitorError, State},
    named_concept::{
        NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptDoesExistError,
        NamedConceptListError, NamedConceptMgmt, NamedConceptRemoveError,
    },
};

use super::{
    testing::__InternalMonitoringTokenTestable, Monitoring, MonitoringBuilder, MonitoringCleaner,
    MonitoringCreateTokenError, MonitoringMonitor, MonitoringStateError, MonitoringToken,
};

#[derive(Debug)]
pub struct FileLockMonitoring {}

impl NamedConceptMgmt for FileLockMonitoring {
    type Configuration = Configuration;

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        let path = cfg.get_path_hint();
        let origin = "FileLockMonitoring::list_cfg()";
        let msg = format!("Unable to list all FileLockMonitoring instances in \"{path}\"");
        let directory = match Directory::new(path) {
            Ok(directory) => directory,
            Err(DirectoryOpenError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptListError::InsufficientPermissions,
                    "{} due to insufficient permissions to read the directory.", msg);
            }
            Err(DirectoryOpenError::DoesNotExist) => {
                return Ok(vec![]);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptListError::InternalError,
                    "{} due to failure ({:?}) while reading the directory.", msg, v);
            }
        };

        let entries = fail!(from origin,
                            when directory.contents(),
                            map DirectoryReadError::InsufficientPermissions => NamedConceptListError::InsufficientPermissions,
                            unmatched NamedConceptListError::InternalError,
                            "{} due to a failure while reading the directory contents.", msg);

        Ok(entries
            .iter()
            .filter(|entry| {
                let metadata = entry.metadata();
                metadata.file_type() == FileType::File
            })
            .filter_map(|entry| cfg.extract_name_from_file(entry.name()))
            .collect())
    }

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        let process_state_path = cfg.path_for(name);
        let msg =
            format!("Unable to check if the FileLockMonitoring \"{process_state_path}\" exists");
        let origin = "FileLockMonitoring::does_exist_cfg()";

        match File::does_exist(&process_state_path) {
            Ok(v) => Ok(v),
            Err(e) => {
                fail!(from origin, with NamedConceptDoesExistError::InternalError,
                    "{} due to an internal failure ({:?}).", msg, e);
            }
        }
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        let process_state_path = cfg.path_for(name);
        let msg = format!("Unable to remove FileLockMonitoring \"{process_state_path}\"");
        let origin = "FileLockMonitoring::remove_cfg()";
        match File::remove(&process_state_path) {
            Ok(v) => Ok(v),
            Err(FileRemoveError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                        "{} due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        crate::named_concept::remove_path_hint(value)
    }
}

#[derive(Debug)]
pub struct Cleaner {
    cleaner: ProcessCleaner,
    name: FileName,
}

impl NamedConcept for Cleaner {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringCleaner for Cleaner {
    fn abandon(self) {
        self.cleaner.abandon()
    }
}

#[derive(Debug)]
pub struct Token {
    guard: ProcessGuard,
    name: FileName,
}

impl NamedConcept for Token {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringToken for Token {}

impl __InternalMonitoringTokenTestable for Token {
    fn staged_death(self) {
        __internal_process_guard_staged_death(self.guard);
    }
}

#[derive(Debug)]
pub struct Monitor {
    monitor: ProcessMonitor,
    name: FileName,
}

impl NamedConcept for Monitor {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringMonitor for Monitor {
    fn state(&self) -> Result<super::State, MonitoringStateError> {
        let msg = "Unable to acquire monitor state";

        match self.monitor.state() {
            Ok(ProcessState::Alive) => Ok(State::Alive),
            Ok(ProcessState::Dead) => Ok(State::Dead),
            Ok(ProcessState::DoesNotExist)
            | Ok(ProcessState::CleaningUp)
            | Ok(ProcessState::Starting) => Ok(State::DoesNotExist),
            Err(ProcessMonitorStateError::Interrupt) => {
                fail!(from self, with MonitoringStateError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(v) => {
                fail!(from self, with MonitoringStateError::InternalError,
                    "{} since an internal failure occurred ({:?}).", msg, v);
            }
        }
    }
}

#[derive(Debug)]
pub struct Builder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<FileLockMonitoring> for Builder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &<FileLockMonitoring as NamedConceptMgmt>::Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl MonitoringBuilder<FileLockMonitoring> for Builder {
    fn token(
        self,
    ) -> Result<<FileLockMonitoring as super::Monitoring>::Token, super::MonitoringCreateTokenError>
    {
        let msg = "Unable to create FileLockMonitoring token";
        let process_state_path = self.config.path_for(&self.name);
        match ProcessGuard::new(&process_state_path) {
            Ok(guard) => Ok(Token {
                guard,
                name: self.name,
            }),
            Err(ProcessGuardCreateError::InsufficientPermissions) => {
                fail!(from self, with MonitoringCreateTokenError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(ProcessGuardCreateError::AlreadyExists) => {
                fail!(from self, with MonitoringCreateTokenError::AlreadyExists,
                    "{} since it already exists.", msg);
            }
            Err(v) => {
                fail!(from self, with MonitoringCreateTokenError::InternalError,
                    "{} due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn monitor(
        self,
    ) -> Result<
        <FileLockMonitoring as super::Monitoring>::Monitor,
        super::MonitoringCreateMonitorError,
    > {
        let msg = "Unable to acquire monitor";
        let process_state_path = self.config.path_for(&self.name);
        match ProcessMonitor::new(&process_state_path) {
            Ok(monitor) => Ok(Monitor {
                monitor,
                name: self.name,
            }),
            Err(ProcessMonitorCreateError::InsufficientPermissions) => {
                fail!(from self, with MonitoringCreateMonitorError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(ProcessMonitorCreateError::Interrupt) => {
                fail!(from self, with MonitoringCreateMonitorError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(v) => {
                fail!(from self, with MonitoringCreateMonitorError::InternalError,
                    "{} due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn cleaner(
        self,
    ) -> Result<<FileLockMonitoring as Monitoring>::Cleaner, super::MonitoringCreateCleanerError>
    {
        let msg = "Unable to acquire cleaner";
        let process_state_path = self.config.path_for(&self.name);
        match ProcessCleaner::new(&process_state_path) {
            Ok(cleaner) => Ok(Cleaner {
                cleaner,
                name: self.name,
            }),
            Err(ProcessCleanerCreateError::Interrupt) => {
                fail!(from self, with MonitoringCreateCleanerError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(ProcessCleanerCreateError::ProcessIsStillAlive) => {
                fail!(from self, with MonitoringCreateCleanerError::InstanceStillAlive,
                    "{} since the instance is still alive.", msg);
            }
            Err(ProcessCleanerCreateError::OwnedByAnotherProcess) => {
                fail!(from self, with MonitoringCreateCleanerError::AlreadyOwnedByAnotherInstance,
                    "{} since another instance already acquired the cleaner.", msg);
            }
            Err(ProcessCleanerCreateError::DoesNotExist) => {
                fail!(from self, with MonitoringCreateCleanerError::DoesNotExist,
                    "{} since it does not exist.", msg);
            }
            Err(e) => {
                fail!(from self, with MonitoringCreateCleanerError::InternalError,
                    "{} due to an internal failure ({:?}).", msg, e);
            }
        }
    }
}

impl crate::monitoring::Monitoring for FileLockMonitoring {
    type Token = Token;
    type Monitor = Monitor;
    type Builder = Builder;
    type Cleaner = Cleaner;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: FileLockMonitoring::default_suffix(),
            prefix: FileLockMonitoring::default_prefix(),
            path_hint: FileLockMonitoring::default_path_hint(),
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
