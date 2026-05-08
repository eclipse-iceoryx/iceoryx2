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

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

use iceoryx2::prelude::SemanticStringError;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::directory::{
    Directory, DirectoryAccessError, DirectoryCreateError, DirectoryOpenError,
};
use iceoryx2_bb_posix::file::{AccessMode, File, FileBuilder, Permission};
use iceoryx2_bb_posix::memory_mapping::SemanticString;
use iceoryx2_bb_posix::process_state::{
    ProcessGuard, ProcessGuardBuilder, ProcessGuardCreateError, ProcessMonitor, ProcessState,
};
use iceoryx2_bb_posix::unique_system_id::{UniqueSystemId, UniqueSystemIdCreationError};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;

use crate::backend::settings::{LOCKFILE_NAME, SESSIONS_DIR};

#[derive(Debug)]
pub enum CreationError {
    UniqueIdCreation(UniqueSystemIdCreationError),
    Path(SemanticStringError),
    DirectoryPermissions(DirectoryAccessError),
    DirectoryCreation(DirectoryCreateError),
    DirectoryOpen(DirectoryOpenError),
    ProcessGuard(ProcessGuardCreateError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub enum DiscoveryError {}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug)]
pub enum AnnounceError {
    Io,
    Encode,
}

impl core::fmt::Display for AnnounceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnounceError::{self:?}")
    }
}

impl core::error::Error for AnnounceError {}

type SessionId = String;

#[derive(Debug, Default)]
struct PendingDiscovery {
    added: Vec<StaticConfig>,
    removed: Vec<ServiceHash>,
}

#[derive(Debug)]
pub struct Session {
    /// Unique ID for this session.
    id: UniqueSystemId,
    /// Path to directory containing all session files.
    sessions_dir_path: Path,
    /// Directory containing all session files.
    sessions_dir: Directory,
    /// Path to this session's own services directory.
    services_dir_path: Path,
    /// Live peer sessions keyed by id.
    discovered_peers: RefCell<BTreeMap<SessionId, Peer>>,
    /// Aggregated set of services offered by any live peer. Updated on
    /// each `discover()`; the diff against the previous snapshot is
    /// surfaced through `pending_discoveries`.
    discovered_services: RefCell<BTreeMap<ServiceHash, StaticConfig>>,
    /// Discovery events accumulated by `discover()` and drained by
    /// `discover()`.
    pending_discoveries: RefCell<PendingDiscovery>,

    _guard: ProcessGuard,
    _cleanup: SessionCleanup,
}

#[derive(Debug)]
struct Peer {
    // The lockfile representing the liveliness of the peer.
    lockfile_path: FilePath,
}

#[derive(Debug)]
struct DiscoveredPeer {
    session_id: SessionId,
    services_dir: Path,
    liveliness_lockfile: FilePath,
}

impl Session {
    pub fn create() -> Result<Self, CreationError> {
        let id = UniqueSystemId::new().map_err(CreationError::UniqueIdCreation)?;

        // Create or open common sessions directory
        let sessions_dir_path = Path::new(SESSIONS_DIR).unwrap();
        let sessions_dir = if !Directory::does_exist(&sessions_dir_path)
            .map_err(CreationError::DirectoryPermissions)?
        {
            Directory::create(&sessions_dir_path, Permission::OWNER_ALL)
                .map_err(CreationError::DirectoryCreation)?
        } else {
            Directory::new(&sessions_dir_path).map_err(CreationError::DirectoryOpen)?
        };

        // Create directory for this session and its service files
        let mut session_dir_path = sessions_dir_path.clone();
        let id_b64 = id.value().to_b64().to_lowercase();
        add_to_path(&mut session_dir_path, id_b64.as_bytes())?;

        let mut services_dir_path = session_dir_path.clone();
        add_to_path(&mut services_dir_path, b"services")?;
        match Directory::create(&services_dir_path, Permission::OWNER_ALL) {
            Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => {}
            Err(e) => return Err(CreationError::DirectoryCreation(e)),
        }

        // Create liveliness lockfile
        let lockfile_path = file_path_in_directory(LOCKFILE_NAME, &session_dir_path)?;

        let guard = ProcessGuardBuilder::new()
            .guard_permissions(Permission::OWNER_ALL)
            .create(&lockfile_path)
            .map_err(CreationError::ProcessGuard)?;

        let cleanup = SessionCleanup {
            session_dir: session_dir_path,
            sessions_dir: sessions_dir_path,
        };

        Ok(Self {
            id,
            sessions_dir_path,
            sessions_dir,
            services_dir_path,
            discovered_peers: RefCell::new(BTreeMap::new()),
            discovered_services: RefCell::new(BTreeMap::new()),
            pending_discoveries: RefCell::new(PendingDiscovery::default()),
            _guard: guard,
            _cleanup: cleanup,
        })
    }

    pub fn announce_added(&self, static_config: &StaticConfig) -> Result<(), AnnounceError> {
        let bytes = postcard::to_allocvec(static_config).map_err(|_| AnnounceError::Encode)?;
        let path = file_path_in_directory(
            static_config.service_hash().as_str().as_bytes(),
            &self.services_dir_path,
        )
        .map_err(|_| AnnounceError::Io)?;
        let mut file = FileBuilder::new(&path)
            .creation_mode(CreationMode::PurgeAndCreate)
            .permission(Permission::OWNER_ALL)
            .create()
            .map_err(|_| AnnounceError::Io)?;
        file.write(&bytes).map_err(|_| AnnounceError::Io)?;
        Ok(())
    }

    pub fn announce_removed(&self, hash: &ServiceHash) -> Result<(), AnnounceError> {
        let path = file_path_in_directory(hash.as_str().as_bytes(), &self.services_dir_path)
            .map_err(|_| AnnounceError::Io)?;
        File::remove(&path).map_err(|_| AnnounceError::Io)?;
        Ok(())
    }

    /// Scan active peers, refresh the aggregated service set, and queue
    /// added/removed events into `pending_discoveries` for the next
    /// `discover()` call.
    pub fn discover(&self) -> Result<(), DiscoveryError> {
        let active_peers = self.discover_active_peers()?;

        // Build the new aggregated set: union of all active peers' services.
        let mut new_aggregated: BTreeMap<ServiceHash, StaticConfig> = BTreeMap::new();
        for peer in &active_peers {
            for (hash, cfg) in Self::discover_peer_services(&peer.services_dir) {
                new_aggregated.entry(hash).or_insert(cfg);
            }
        }

        // Diff against the previous aggregated set.
        {
            let mut pending = self.pending_discoveries.borrow_mut();
            let prev = self.discovered_services.borrow();
            for (hash, cfg) in &new_aggregated {
                if !prev.contains_key(hash) {
                    pending.added.push(cfg.clone());
                }
            }
            for hash in prev.keys() {
                if !new_aggregated.contains_key(hash) {
                    pending.removed.push(*hash);
                }
            }
        }

        self.reconcile_peers(active_peers);
        *self.discovered_services.borrow_mut() = new_aggregated;

        Ok(())
    }

    /// Drain discovery events accumulated since the last call.
    pub fn pending_discoveries(&self) -> (Vec<StaticConfig>, Vec<ServiceHash>) {
        let mut pending = self.pending_discoveries.borrow_mut();

        // TODO: Is take correct?
        let added = core::mem::take(&mut pending.added);
        let removed = core::mem::take(&mut pending.removed);

        (added, removed)
    }

    fn discover_active_peers(&self) -> Result<Vec<DiscoveredPeer>, DiscoveryError> {
        let entries = self.sessions_dir.contents().unwrap();

        let mut active_peers: Vec<DiscoveredPeer> = Vec::new();
        for entry in entries {
            if entry.name().as_bytes() == self.id.value().to_b64().to_lowercase().as_bytes() {
                continue;
            }

            let mut session_dir_path = self.sessions_dir_path.clone();
            if add_to_path(&mut session_dir_path, entry.name().as_bytes()).is_err() {
                continue;
            }

            // Verify peer liveliness
            let lockfile_path = match file_path_in_directory(LOCKFILE_NAME, &session_dir_path) {
                Ok(path) => path,
                Err(_) => continue,
            };

            let monitor = match ProcessMonitor::new(&lockfile_path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            match monitor.state() {
                Ok(ProcessState::Alive) | Ok(ProcessState::Starting) => {
                    let mut services_path = session_dir_path.clone();
                    // TODO: Make services leaf a const
                    if add_to_path(&mut services_path, b"services").is_err() {
                        continue;
                    }
                    let id_str = match core::str::from_utf8(entry.name().as_bytes()) {
                        Ok(s) => s.to_string(),
                        Err(_) => continue,
                    };

                    active_peers.push(DiscoveredPeer {
                        session_id: id_str,
                        services_dir: services_path,
                        liveliness_lockfile: lockfile_path,
                    });
                }
                Ok(ProcessState::Dead) | Ok(ProcessState::CleaningUp) => {
                    // Crash cleanup: tear down the stale directory so we
                    // stop re-detecting it.
                    let _ = Directory::remove(&session_dir_path);
                }
                Ok(ProcessState::DoesNotExist) => {
                    // Graceful exit already removed the lock file. Treat
                    // any leftover sibling files as stale.
                    let _ = Directory::remove(&session_dir_path);
                }
                Err(_) => continue,
            }
        }

        Ok(active_peers)
    }

    fn reconcile_peers<I>(&self, active_peers: I)
    where
        I: IntoIterator<Item = DiscoveredPeer>,
    {
        let active_peers: Vec<DiscoveredPeer> = active_peers.into_iter().collect();

        // Get IDs of active peers.
        let active_ids: alloc::collections::BTreeSet<SessionId> = active_peers
            .iter()
            .map(|peer| peer.session_id.clone())
            .collect();

        // Remove peers that are no longer present.
        let stale: Vec<SessionId> = self
            .discovered_peers
            .borrow()
            .keys()
            .filter(|session_id| !active_ids.contains(*session_id))
            .cloned()
            .collect();

        for id in stale {
            self.discovered_peers.borrow_mut().remove(&id);
        }

        // Track to new peers.
        for peer in active_peers {
            if self
                .discovered_peers
                .borrow_mut()
                .contains_key(&peer.session_id)
            {
                continue;
            }

            self.discovered_peers.borrow_mut().insert(
                peer.session_id,
                Peer {
                    lockfile_path: peer.liveliness_lockfile,
                },
            );
        }
    }

    fn discover_peer_services(services_dir_path: &Path) -> BTreeMap<ServiceHash, StaticConfig> {
        let mut discovered_services = BTreeMap::new();

        let services_dir = match Directory::new(services_dir_path) {
            Ok(d) => d,
            Err(_) => return discovered_services,
        };

        let entries = match services_dir.contents() {
            Ok(e) => e,
            Err(_) => return discovered_services,
        };

        for entry in entries {
            let path = match FilePath::from_path_and_file(services_dir_path, entry.name()) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let file = match FileBuilder::new(&path).open_existing(AccessMode::Read) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let mut bytes = Vec::new();
            if file.read_to_vector(&mut bytes).is_err() {
                continue;
            }
            let static_config: StaticConfig = match postcard::from_bytes(&bytes) {
                Ok(c) => c,
                Err(_) => continue,
            };
            discovered_services.insert(*static_config.service_hash(), static_config);
        }

        discovered_services
    }
}

#[derive(Debug)]
struct SessionCleanup {
    sessions_dir: Path,
    session_dir: Path,
}

impl Drop for SessionCleanup {
    fn drop(&mut self) {
        let _ = Directory::remove(&self.session_dir);
        let _ = Directory::remove_empty(&self.sessions_dir);
    }
}

fn add_to_path(path: &mut Path, name: &[u8]) -> Result<(), CreationError> {
    let entry = Path::new(name).map_err(CreationError::Path)?;
    path.add_path_entry(&entry).map_err(CreationError::Path)
}

fn file_path_in_directory(name: &[u8], dir: &Path) -> Result<FilePath, CreationError> {
    let file = FileName::new(name).map_err(CreationError::Path)?;
    FilePath::from_path_and_file(dir, &file).map_err(CreationError::Path)
}
