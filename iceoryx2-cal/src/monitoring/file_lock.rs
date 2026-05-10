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

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_posix::file::Permission;
use iceoryx2_bb_posix::process_state::ProcessGuardBuilder;
use iceoryx2_bb_posix::process_state::ProcessMonitorOpenError;
use iceoryx2_bb_posix::{
    directory::{Directory, DirectoryOpenError, DirectoryReadError},
    file::{File, FileRemoveError},
    file_type::FileType,
    process_state::{
        ProcessCleaner, ProcessCleanerCreateError, ProcessGuard, ProcessGuardCreateError,
        ProcessMonitor, ProcessMonitorCreateError, ProcessMonitorStateError, ProcessState,
    },
};
use iceoryx2_bb_system_types::{file_name::FileName, path::Path};
use iceoryx2_bb_testing::abandonable::Abandonable;
use iceoryx2_bb_testing::abandonable::NonNullFromRef;
use iceoryx2_log::fail;

use crate::{
    monitoring::{MonitoringCreateCleanerError, MonitoringCreateMonitorError, State},
    named_concept::{
        NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptDoesExistError,
        NamedConceptListError, NamedConceptMgmt, NamedConceptRemoveError,
    },
};

use super::{
    Monitoring, MonitoringBuilder, MonitoringCleaner, MonitoringCreateTokenError,
    MonitoringMonitor, MonitoringStateError, MonitoringToken,
};

#[cfg(not(feature = "dev_permissions"))]
const GUARD_PERMISSIONS: Permission = Permission::OWNER_ALL;

#[cfg(not(feature = "dev_permissions"))]
const DIR_PERMISSIONS: Permission = Permission::OWNER_ALL
    .const_bitor(Permission::GROUP_READ)
    .const_bitor(Permission::GROUP_EXEC)
    .const_bitor(Permission::OTHERS_READ)
    .const_bitor(Permission::OTHERS_EXEC);

#[cfg(feature = "dev_permissions")]
const GUARD_PERMISSIONS: Permission = Permission::ALL;

#[cfg(feature = "dev_permissions")]
const DIR_PERMISSIONS: Permission = Permission::ALL;

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
        match unsafe { ProcessGuard::remove(&process_state_path) } {
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

impl Abandonable for Cleaner {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            ProcessCleaner::abandon_in_place(core::ptr::NonNull::iox2_from_mut(&mut this.cleaner))
        };
    }
}

impl NamedConcept for Cleaner {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringCleaner for Cleaner {
    fn relinquish(self) {
        self.cleaner.abandon()
    }
}

#[derive(Debug)]
pub struct Token {
    guard: ProcessGuard,
    name: FileName,
}

impl Abandonable for Token {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            ProcessGuard::abandon_in_place(core::ptr::NonNull::iox2_from_mut(&mut this.guard))
        };
    }
}

impl NamedConcept for Token {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl MonitoringToken for Token {}

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
            Err(ProcessMonitorStateError::Interrupt)
            | Err(ProcessMonitorStateError::ProcessMonitorOpenError(
                ProcessMonitorOpenError::Interrupt,
            )) => {
                fail!(from self, with MonitoringStateError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(ProcessMonitorStateError::ProcessMonitorOpenError(
                ProcessMonitorOpenError::InsufficientPermissions,
            )) => {
                fail!(from self, with MonitoringStateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
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
            name: *name,
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
        match ProcessGuardBuilder::new()
            .guard_permissions(GUARD_PERMISSIONS)
            .directory_permissions(DIR_PERMISSIONS)
            .create(&process_state_path)
        {
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
            Err(ProcessGuardCreateError::SystemCorrupted) => {
                fail!(from self, with MonitoringCreateTokenError::SystemCorrupted,
                    "{} since another process corrupted the process guard while it was created.", msg);
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
            Err(ProcessMonitorCreateError::InvalidCleanerPathName) => {
                fail!(from self, with MonitoringCreateMonitorError::ConceptNameNotSupportedOnPlatform,
                    "{} since the concept name \"{}\" results in a concept name that is not supported on this platform.", msg, self.name);
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
            Err(ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup) => {
                fail!(from self, with MonitoringCreateCleanerError::IsBeingCleanedUpOrAnotherCleanerCrashedDuringCleanup,
                    "{} since the process is currently being cleaned up (or crashed during cleanup).", msg);
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

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.path_hint = *value;
        self
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}
