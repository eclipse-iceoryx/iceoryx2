// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::string::ToString;
use alloc::vec::Vec;

use iceoryx2::prelude::SemanticStringError;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::directory::{Directory, DirectoryCreateError};
use iceoryx2_bb_posix::file::{AccessMode, File, FileBuilder, Permission};
use iceoryx2_bb_posix::memory_mapping::SemanticString;
use iceoryx2_bb_posix::process_state::{
    ProcessGuard, ProcessGuardBuilder, ProcessMonitor, ProcessState,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_gateway_backend::types::identity::GatewayId;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;
use serde::{Deserialize, Serialize};

use crate::backend::settings::{
    LOCKFILE_NAME, ROOT_DIR, SERVICES_DIR_NAME, SESSIONS_DIR_NAME, SOCKET_NAME,
};

use super::{AnnounceError, CreationError, SessionId};

/// The on-disk registry through which sessions on the same host discover
/// each other. Every session occupies one directory holding its liveliness
/// lockfile, its socket, and its announced services.
#[derive(Debug)]
pub(super) struct Registry {
    sessions_dir_path: Path,
    sessions_dir: Directory,
}

impl Registry {
    /// Open the registry, creating it if it does not exist. Entries of
    /// sessions that are no longer alive are swept.
    pub(super) fn open() -> Result<Self, CreationError> {
        let mut sessions_dir_path = Path::new(ROOT_DIR).unwrap();
        add_to_path(&mut sessions_dir_path, SESSIONS_DIR_NAME).map_err(CreationError::Path)?;
        let sessions_dir = if !Directory::does_exist(&sessions_dir_path)
            .map_err(CreationError::DirectoryPermissions)?
        {
            Directory::create(&sessions_dir_path, Permission::OWNER_ALL)
                .map_err(CreationError::DirectoryCreation)?
        } else {
            Directory::new(&sessions_dir_path).map_err(CreationError::DirectoryOpen)?
        };

        sweep_stale_sessions(&sessions_dir, &sessions_dir_path);

        Ok(Self {
            sessions_dir_path,
            sessions_dir,
        })
    }

    /// Add an entry for the session with the given id to the registry.
    pub(super) fn register(&self, id: &SessionId) -> Result<Registration, CreationError> {
        // Create the directory for this session.
        let mut session_dir_path = self.sessions_dir_path;
        add_to_path(&mut session_dir_path, id.as_bytes()).map_err(CreationError::Path)?;
        match Directory::create(&session_dir_path, Permission::OWNER_ALL) {
            Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => {}
            Err(e) => return Err(CreationError::DirectoryCreation(e)),
        }

        // Create the lockfile to indicate liveliness.
        // Must be created first to ensure other sessions do not detect this
        // session as dead.
        let lockfile_path = file_path_in_directory(LOCKFILE_NAME, &session_dir_path)
            .map_err(CreationError::Path)?;

        let guard = ProcessGuardBuilder::new()
            .guard_permissions(Permission::OWNER_READ_WRITE)
            .create(&lockfile_path)
            .map_err(CreationError::ProcessGuard)?;

        // Create the directory holding this session's service files.
        let mut services_dir_path = session_dir_path;
        add_to_path(&mut services_dir_path, SERVICES_DIR_NAME).map_err(CreationError::Path)?;
        match Directory::create(&services_dir_path, Permission::OWNER_ALL) {
            Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => {}
            Err(e) => return Err(CreationError::DirectoryCreation(e)),
        }

        let sock_path =
            file_path_in_directory(SOCKET_NAME, &session_dir_path).map_err(CreationError::Path)?;

        Ok(Registration {
            services_dir_path,
            sock_path,
            _guard: guard,
            _cleanup: Cleanup {
                session_dir: session_dir_path,
                sessions_dir: self.sessions_dir_path,
            },
        })
    }

    /// Return the sessions currently alive in the registry. Entries of dead
    /// sessions are swept.
    pub(super) fn sessions(&self) -> Vec<RegisteredSession> {
        let Ok(entries) = self.sessions_dir.contents() else {
            return Vec::new();
        };

        let mut sessions: Vec<RegisteredSession> = Vec::new();
        for entry in entries {
            let mut session_dir_path = self.sessions_dir_path;
            if add_to_path(&mut session_dir_path, entry.name().as_bytes()).is_err() {
                continue;
            }

            match classify_session(&session_dir_path) {
                SessionState::Alive => {
                    let mut services_dir = session_dir_path;
                    if add_to_path(&mut services_dir, SERVICES_DIR_NAME).is_err() {
                        continue;
                    }
                    let Ok(id) = core::str::from_utf8(entry.name().as_bytes()) else {
                        continue;
                    };
                    let Ok(sock_path) = file_path_in_directory(SOCKET_NAME, &session_dir_path)
                    else {
                        continue;
                    };

                    sessions.push(RegisteredSession {
                        id: id.to_string(),
                        services_dir,
                        sock_path,
                    });
                }
                SessionState::Stale => {
                    let _ = Directory::remove(&session_dir_path);
                }
                SessionState::Indeterminate => continue,
            }
        }

        sessions
    }
}

/// A session's handle to its own entry in the registry. Dropping it removes the entry.
#[derive(Debug)]
pub(super) struct Registration {
    services_dir_path: Path,
    sock_path: FilePath,
    _guard: ProcessGuard,
    _cleanup: Cleanup,
}

impl Registration {
    /// The path of this session's socket.
    pub(super) fn socket_path(&self) -> &FilePath {
        &self.sock_path
    }

    /// Add a service to this session's registry entry, making it
    /// discoverable by other sessions as offered by the specified gateway.
    pub(super) fn add_service(
        &self,
        gateway_id: GatewayId,
        description: &ServiceDescription,
    ) -> Result<(), AnnounceError> {
        let path = file_path_in_directory(
            description.service_hash.as_str().as_bytes(),
            &self.services_dir_path,
        )
        .map_err(AnnounceError::Path)?;

        if File::does_exist(&path).map_err(AnnounceError::FileExists)? {
            // Already announced
            return Ok(());
        }

        let mut file = FileBuilder::new(&path)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(Permission::OWNER_READ_WRITE)
            .create()
            .map_err(AnnounceError::FileCreate)?;

        let announced = AnnouncedService {
            gateway_id,
            description: description.clone(),
        };
        let bytes = postcard::to_allocvec(&announced).map_err(|_| AnnounceError::Encode)?;
        file.write(&bytes).map_err(AnnounceError::FileWrite)?;

        Ok(())
    }

    /// Remove a previously added service from this session's registry entry.
    pub(super) fn remove_service(&self, hash: &ServiceHash) -> Result<(), AnnounceError> {
        let path = file_path_in_directory(hash.as_str().as_bytes(), &self.services_dir_path)
            .map_err(AnnounceError::Path)?;

        File::remove(&path).map_err(AnnounceError::FileRemove)?;

        Ok(())
    }
}

/// Removes the session's registry entry on drop.
#[derive(Debug)]
struct Cleanup {
    sessions_dir: Path,
    session_dir: Path,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = Directory::remove(&self.session_dir);
        let _ = Directory::remove_empty(&self.sessions_dir);
    }
}

/// A service in a session's registry entry together with the gateway that
/// announced it.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AnnouncedService {
    pub(super) gateway_id: GatewayId,
    pub(super) description: ServiceDescription,
}

/// A foreign session's entry in the registry.
#[derive(Debug)]
pub(super) struct RegisteredSession {
    pub(super) id: SessionId,
    pub(super) sock_path: FilePath,
    services_dir: Path,
}

impl RegisteredSession {
    /// Return the services in this session's registry entry.
    pub(super) fn services(&self) -> Vec<AnnouncedService> {
        let mut services = Vec::new();

        let Ok(services_dir) = Directory::new(&self.services_dir) else {
            return services;
        };

        let Ok(entries) = services_dir.contents() else {
            return services;
        };

        for entry in entries {
            let Ok(path) = FilePath::from_path_and_file(&self.services_dir, entry.name()) else {
                continue;
            };
            let Ok(file) = FileBuilder::new(&path).open_existing(AccessMode::Read) else {
                continue;
            };
            let mut bytes = Vec::new();
            if file.read_to_vector(&mut bytes).is_err() {
                continue;
            }
            let Ok(announced) = postcard::from_bytes::<AnnouncedService>(&bytes) else {
                continue;
            };
            services.push(announced);
        }

        services
    }
}

enum SessionState {
    Alive,
    Stale,
    Indeterminate,
}

/// Determine whether a session directory belongs to a live process, a
/// crashed/aborted one, or cannot be classified.
fn classify_session(session_dir_path: &Path) -> SessionState {
    let Ok(lockfile_path) = file_path_in_directory(LOCKFILE_NAME, session_dir_path) else {
        return SessionState::Indeterminate;
    };
    let Ok(monitor) = ProcessMonitor::new(&lockfile_path) else {
        return SessionState::Indeterminate;
    };
    match monitor.state() {
        Ok(ProcessState::Alive) | Ok(ProcessState::Starting) => SessionState::Alive,
        Ok(ProcessState::Dead) | Ok(ProcessState::CleaningUp) => SessionState::Stale,
        // If the directory exists without the lockfile, the session is being
        // initialized.
        Ok(ProcessState::DoesNotExist) => SessionState::Indeterminate,
        Err(_) => SessionState::Indeterminate,
    }
}

/// Remove every session directory whose owning process is no longer alive.
fn sweep_stale_sessions(sessions_dir: &Directory, sessions_dir_path: &Path) {
    let Ok(entries) = sessions_dir.contents() else {
        return;
    };
    for entry in entries {
        let mut session_dir_path = *sessions_dir_path;
        if add_to_path(&mut session_dir_path, entry.name().as_bytes()).is_err() {
            continue;
        }
        if matches!(classify_session(&session_dir_path), SessionState::Stale) {
            let _ = Directory::remove(&session_dir_path);
        }
    }
}

/// Append a name component to a path.
fn add_to_path(path: &mut Path, name: &[u8]) -> Result<(), SemanticStringError> {
    let entry = Path::new(name)?;
    path.add_path_entry(&entry)
}

/// Build the path of a file inside the given directory.
fn file_path_in_directory(name: &[u8], dir: &Path) -> Result<FilePath, SemanticStringError> {
    let file = FileName::new(name)?;
    FilePath::from_path_and_file(dir, &file)
}
