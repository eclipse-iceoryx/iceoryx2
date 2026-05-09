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

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

use serde::{Deserialize, Serialize};

use iceoryx2::prelude::SemanticStringError;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::directory::{
    Directory, DirectoryAccessError, DirectoryCreateError, DirectoryOpenError,
};
use iceoryx2_bb_posix::file::{
    AccessMode, File, FileBuilder, FileCreationError, FileRemoveError, FileWriteError, Permission,
};
use iceoryx2_bb_posix::memory_mapping::SemanticString;
use iceoryx2_bb_posix::process_state::{
    ProcessGuard, ProcessGuardBuilder, ProcessGuardCreateError, ProcessMonitor, ProcessState,
};
use iceoryx2_bb_posix::unique_system_id::{UniqueSystemId, UniqueSystemIdCreationError};
use iceoryx2_bb_posix::unix_datagram_socket::{
    UnixDatagramReceiver, UnixDatagramReceiverBuilder, UnixDatagramReceiverCreationError,
    UnixDatagramSender, UnixDatagramSenderBuilder,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;

use crate::backend::settings::{
    LOCKFILE_NAME, MAX_DATAGRAM, ROOT_DIR, SERVICES_DIR_NAME, SESSIONS_DIR_NAME, SOCKET_NAME,
};

#[derive(Debug)]
pub enum CreationError {
    UniqueIdCreation(UniqueSystemIdCreationError),
    Path(SemanticStringError),
    DirectoryPermissions(DirectoryAccessError),
    DirectoryCreation(DirectoryCreateError),
    DirectoryOpen(DirectoryOpenError),
    ProcessGuard(ProcessGuardCreateError),
    SocketBind(UnixDatagramReceiverCreationError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub enum AnnounceError {
    Path(SemanticStringError),
    Encode,
    FileCreate(FileCreationError),
    FileWrite(FileWriteError),
    FileRemove(FileRemoveError),
}

impl core::fmt::Display for AnnounceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnounceError::{self:?}")
    }
}

impl core::error::Error for AnnounceError {}

#[derive(Debug)]
pub enum SendError {
    Encode,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug)]
pub enum ReceiveError {
    Io,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

type SessionId = String;

#[derive(Debug, Default)]
struct PendingDiscovery {
    added: Vec<StaticConfig>,
    removed: Vec<ServiceHash>,
}

#[derive(Debug)]
pub struct Sample {
    pub header: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Envelope {
    from: SessionId,
    kind: Kind,
}

#[derive(Debug, Serialize, Deserialize)]
enum Kind {
    Event {
        service_hash: ServiceHash,
        id: u64,
    },
    Sample {
        service_hash: ServiceHash,
        header: Vec<u8>,
        payload: Vec<u8>,
    },
}

#[derive(Debug)]
pub struct Session {
    /// Unique session ID
    id: SessionId,
    /// Path to directory containing all session files.
    sessions_dir_path: Path,
    /// Directory containing all session files.
    sessions_dir: Directory,
    /// Path to this session's own services directory.
    services_dir_path: Path,
    /// Live peer sessions keyed by id.
    discovered_peers: RefCell<BTreeMap<SessionId, Peer>>,
    /// Hashes of services offered by any live peer at the last `discover()`.
    discovered_services: RefCell<BTreeSet<ServiceHash>>,
    /// Discovery events accumulated by `discover()` and drained by
    /// `discover()`.
    pending_discoveries: RefCell<PendingDiscovery>,
    /// Per-service event id queues populated by `recv_event`'s drain.
    received_events: RefCell<BTreeMap<ServiceHash, VecDeque<u64>>>,
    /// Per-service sample queues populated by `recv_sample`'s drain.
    received_samples: RefCell<BTreeMap<ServiceHash, VecDeque<Sample>>>,
    /// Datagram receive buffer.
    recv_buffer: RefCell<Vec<u8>>,
    /// Datagram serialize buffer.
    send_buffer: RefCell<Vec<u8>>,
    /// Unix soscket abstraction for payloads
    receiver: UnixDatagramReceiver,
    _guard: ProcessGuard,
    _cleanup: SessionCleanup,
}

#[derive(Debug)]
struct Peer {
    sender: UnixDatagramSender,
}

#[derive(Debug)]
struct DiscoveredPeer {
    session_id: SessionId,
    services_dir: Path,
    sock_path: FilePath,
}

impl Session {
    /// Create a new session that can announce services to and exchange
    /// samples/events with other live sessions on the same host.
    pub fn create() -> Result<Self, CreationError> {
        let id = UniqueSystemId::new()
            .map_err(CreationError::UniqueIdCreation)?
            .value()
            .to_b64()
            .to_lowercase();

        // Create or open common sessions directory
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

        // Sweep dirs left behind by aborted prior runs before adding our own.
        sweep_stale_sessions(&sessions_dir, &sessions_dir_path);

        // Create directory for this session and its service files
        let mut session_dir_path = sessions_dir_path.clone();
        add_to_path(&mut session_dir_path, id.as_bytes()).map_err(CreationError::Path)?;

        let mut services_dir_path = session_dir_path.clone();
        add_to_path(&mut services_dir_path, SERVICES_DIR_NAME).map_err(CreationError::Path)?;
        match Directory::create(&services_dir_path, Permission::OWNER_ALL) {
            Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => {}
            Err(e) => return Err(CreationError::DirectoryCreation(e)),
        }

        // Create liveliness lockfile
        let lockfile_path = file_path_in_directory(LOCKFILE_NAME, &session_dir_path)
            .map_err(CreationError::Path)?;

        let guard = ProcessGuardBuilder::new()
            .guard_permissions(Permission::OWNER_ALL)
            .create(&lockfile_path)
            .map_err(CreationError::ProcessGuard)?;

        // Create a UDS receiver
        let sock_path =
            file_path_in_directory(SOCKET_NAME, &session_dir_path).map_err(CreationError::Path)?;
        let receiver = UnixDatagramReceiverBuilder::new(&sock_path)
            .creation_mode(CreationMode::PurgeAndCreate)
            .permission(Permission::OWNER_ALL)
            .create()
            .map_err(CreationError::SocketBind)?;

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
            discovered_services: RefCell::new(BTreeSet::new()),
            pending_discoveries: RefCell::new(PendingDiscovery::default()),
            received_events: RefCell::new(BTreeMap::new()),
            received_samples: RefCell::new(BTreeMap::new()),
            recv_buffer: RefCell::new(alloc::vec![0u8; MAX_DATAGRAM]),
            send_buffer: RefCell::new(alloc::vec![0u8; MAX_DATAGRAM]),
            receiver,
            _guard: guard,
            _cleanup: cleanup,
        })
    }

    /// Make a service offered by this session discoverable to peers.
    pub fn announce_added(&self, static_config: &StaticConfig) -> Result<(), AnnounceError> {
        let bytes = postcard::to_allocvec(static_config).map_err(|_| AnnounceError::Encode)?;
        let path = file_path_in_directory(
            static_config.service_hash().as_str().as_bytes(),
            &self.services_dir_path,
        )
        .map_err(AnnounceError::Path)?;
        let mut file = FileBuilder::new(&path)
            .creation_mode(CreationMode::PurgeAndCreate)
            .permission(Permission::OWNER_ALL)
            .create()
            .map_err(AnnounceError::FileCreate)?;
        file.write(&bytes).map_err(AnnounceError::FileWrite)?;
        Ok(())
    }

    /// Withdraw a previously-announced service so peers stop discovering it.
    pub fn announce_removed(&self, hash: &ServiceHash) -> Result<(), AnnounceError> {
        let path = file_path_in_directory(hash.as_str().as_bytes(), &self.services_dir_path)
            .map_err(AnnounceError::Path)?;
        File::remove(&path).map_err(AnnounceError::FileRemove)?;
        Ok(())
    }

    /// Refresh the known-service set; new and dropped hashes are queued for
    /// the next `pending_discoveries()` drain.
    pub fn discover(&self) {
        let active_peers = self.discover_peers();
        let mut pending = self.pending_discoveries.borrow_mut();

        let current = {
            let prev = self.discovered_services.borrow();

            // Mark newly-discovered hashes as added
            let mut current: BTreeSet<ServiceHash> = BTreeSet::new();
            for peer in &active_peers {
                for (hash, cfg) in Self::discover_peer_services(&peer.services_dir) {
                    if current.insert(hash) && !prev.contains(&hash) {
                        pending.added.push(cfg);
                    }
                }
            }

            // Mark previously-known hashes that are absent as removed
            for hash in prev.iter() {
                if !current.contains(hash) {
                    pending.removed.push(*hash);
                }
            }

            current
        };

        *self.discovered_services.borrow_mut() = current;
    }

    /// Drain (added, removed) service-discovery events accumulated since the
    /// last call.
    pub fn pending_discoveries(&self) -> (Vec<StaticConfig>, Vec<ServiceHash>) {
        let mut pending = self.pending_discoveries.borrow_mut();
        let added = core::mem::take(&mut pending.added);
        let removed = core::mem::take(&mut pending.removed);
        (added, removed)
    }

    /// Send an event id for the given service to all live peers.
    pub fn send_event(&self, service_hash: &ServiceHash, id: u64) -> Result<(), SendError> {
        self.discover_peers();
        self.broadcast(Kind::Event {
            service_hash: *service_hash,
            id,
        })
    }

    /// Send a publish-subscribe sample for the given service to all live peers.
    pub fn send_sample(
        &self,
        service_hash: &ServiceHash,
        header: Vec<u8>,
        payload: Vec<u8>,
    ) -> Result<(), SendError> {
        self.discover_peers();
        self.broadcast(Kind::Sample {
            service_hash: *service_hash,
            header,
            payload,
        })
    }

    /// Return the next event id received for the given service, or `None`.
    pub fn recv_event(&self, service_hash: &ServiceHash) -> Result<Option<u64>, ReceiveError> {
        self.recv()?;
        Ok(self
            .received_events
            .borrow_mut()
            .get_mut(service_hash)
            .and_then(|q| q.pop_front()))
    }

    /// Return the next sample received for the given service, or `None`.
    pub fn recv_sample(&self, service_hash: &ServiceHash) -> Result<Option<Sample>, ReceiveError> {
        self.recv()?;
        Ok(self
            .received_samples
            .borrow_mut()
            .get_mut(service_hash)
            .and_then(|q| q.pop_front()))
    }

    /// Send the given message to every currently-tracked peer.
    fn broadcast(&self, kind: Kind) -> Result<(), SendError> {
        let envelope = Envelope {
            from: self.id.clone(),
            kind,
        };
        let mut buf = self.send_buffer.borrow_mut();
        let bytes = postcard::to_slice(&envelope, &mut buf).map_err(|_| SendError::Encode)?;
        for peer in self.discovered_peers.borrow().values() {
            // `try_send` is non-blocking; ignore EAGAIN/ECONNREFUSED
            // (peer slow or just exited).
            let _ = peer.sender.try_send(bytes);
        }
        Ok(())
    }

    /// Drain all pending datagrams from peers into the per-service queues.
    pub fn recv(&self) -> Result<(), ReceiveError> {
        let mut buf = self.recv_buffer.borrow_mut();
        loop {
            let n = self
                .receiver
                .try_receive(&mut buf)
                .map_err(|_| ReceiveError::Io)? as usize;
            if n == 0 {
                return Ok(());
            }

            let envelope: Envelope = match postcard::from_bytes(&buf[..n]) {
                Ok(e) => e,
                Err(_) => continue, // skip malformed datagrams
            };
            if envelope.from == self.id {
                continue;
            }
            match envelope.kind {
                Kind::Event { service_hash, id } => {
                    self.received_events
                        .borrow_mut()
                        .entry(service_hash)
                        .or_default()
                        .push_back(id);
                }
                Kind::Sample {
                    service_hash,
                    header,
                    payload,
                } => {
                    self.received_samples
                        .borrow_mut()
                        .entry(service_hash)
                        .or_default()
                        .push_back(Sample { header, payload });
                }
            }
        }
    }

    /// Refresh the peer table to match what is currently live on disk.
    fn discover_peers(&self) -> Vec<DiscoveredPeer> {
        let active = self.discover_active_peers();
        self.reconcile_peers(&active);
        active
    }

    /// Return the peers currently alive on disk.
    fn discover_active_peers(&self) -> Vec<DiscoveredPeer> {
        let entries = match self.sessions_dir.contents() {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut active_peers: Vec<DiscoveredPeer> = Vec::new();
        for entry in entries {
            if entry.name().as_bytes() == self.id.as_bytes() {
                continue;
            }
            let mut session_dir_path = self.sessions_dir_path.clone();
            if add_to_path(&mut session_dir_path, entry.name().as_bytes()).is_err() {
                continue;
            }

            match classify_session(&session_dir_path) {
                SessionState::Alive => {
                    let mut services_path = session_dir_path.clone();
                    if add_to_path(&mut services_path, SERVICES_DIR_NAME).is_err() {
                        continue;
                    }
                    let id_str = match core::str::from_utf8(entry.name().as_bytes()) {
                        Ok(s) => s.to_string(),
                        Err(_) => continue,
                    };
                    let sock_path = match file_path_in_directory(SOCKET_NAME, &session_dir_path) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    active_peers.push(DiscoveredPeer {
                        session_id: id_str,
                        services_dir: services_path,
                        sock_path,
                    });
                }
                SessionState::Stale => {
                    let _ = Directory::remove(&session_dir_path);
                }
                SessionState::Indeterminate => continue,
            }
        }

        active_peers
    }

    /// Align the tracked peer set with the given live peers.
    fn reconcile_peers(&self, active_peers: &[DiscoveredPeer]) {
        let mut peers = self.discovered_peers.borrow_mut();

        // Drop peers no longer present.
        peers.retain(|id, _| {
            active_peers
                .iter()
                .any(|p| p.session_id.as_str() == id.as_str())
        });

        // Track new peers.
        for peer in active_peers {
            if peers.contains_key(&peer.session_id) {
                continue;
            }
            let sender = match UnixDatagramSenderBuilder::new(&peer.sock_path).create() {
                Ok(s) => s,
                Err(_) => continue, // peer may have just exited
            };
            peers.insert(peer.session_id.clone(), Peer { sender });
        }
    }

    /// Return the services a peer is currently announcing.
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

enum SessionState {
    Alive,
    Stale,
    Indeterminate,
}

/// Determine whether a session directory belongs to a live process, a
/// crashed/aborted one, or cannot be classified.
fn classify_session(session_dir_path: &Path) -> SessionState {
    let lockfile_path = match file_path_in_directory(LOCKFILE_NAME, session_dir_path) {
        Ok(p) => p,
        Err(_) => return SessionState::Indeterminate,
    };
    let monitor = match ProcessMonitor::new(&lockfile_path) {
        Ok(m) => m,
        Err(_) => return SessionState::Stale,
    };
    match monitor.state() {
        Ok(ProcessState::Alive) | Ok(ProcessState::Starting) => SessionState::Alive,
        Ok(ProcessState::Dead) | Ok(ProcessState::CleaningUp) | Ok(ProcessState::DoesNotExist) => {
            SessionState::Stale
        }
        Err(_) => SessionState::Indeterminate,
    }
}

/// Remove every session directory whose owning process is no longer alive.
fn sweep_stale_sessions(sessions_dir: &Directory, sessions_dir_path: &Path) {
    let entries = match sessions_dir.contents() {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let mut session_dir_path = sessions_dir_path.clone();
        if add_to_path(&mut session_dir_path, entry.name().as_bytes()).is_err() {
            continue;
        }
        if matches!(classify_session(&session_dir_path), SessionState::Stale) {
            let _ = Directory::remove(&session_dir_path);
        }
    }
}
