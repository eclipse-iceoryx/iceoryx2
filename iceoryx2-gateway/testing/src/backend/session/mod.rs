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

mod registry;
pub mod wire;

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;

use iceoryx2::prelude::SemanticStringError;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::directory::{
    DirectoryAccessError, DirectoryCreateError, DirectoryOpenError,
};
use iceoryx2_bb_posix::file::{
    FileAccessError, FileCreationError, FileRemoveError, FileWriteError, Permission,
};
use iceoryx2_bb_posix::process_state::ProcessGuardCreateError;
use iceoryx2_bb_posix::unique_system_id::{UniqueSystemId, UniqueSystemIdCreationError};
use iceoryx2_bb_posix::unix_datagram_socket::{
    UnixDatagramReceiver, UnixDatagramReceiverBuilder, UnixDatagramReceiverCreationError,
    UnixDatagramSendError, UnixDatagramSender, UnixDatagramSenderBuilder,
};
use iceoryx2_gateway_backend::types::discovery::DiscoveryUpdate;
use iceoryx2_gateway_backend::types::identity::{BackendId, GatewayId};
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

use crate::backend::settings::MAX_DATAGRAM;

use registry::{RegisteredSession, Registration, Registry};
use wire::{Envelope, Kind, Sample, deserialize_envelope, serialize_envelope};

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
    FileExists(FileAccessError),
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
    TooLarge(usize),
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
    updates: Vec<DiscoveryUpdate>,
}

#[derive(Debug)]
pub struct Session {
    /// Unique session ID
    id: SessionId,
    /// The session id in the form peers address over the gateway.
    backend_id: BackendId,
    /// The on-disk registry through which sessions discover each other.
    registry: Registry,
    /// This session's own entry in the registry, removed on drop.
    registration: Registration,
    /// Sending half of the connection to each live peer.
    connections: RefCell<BTreeMap<SessionId, UnixDatagramSender>>,
    /// Hashes of services offered by any live peer at the last `discover()`.
    discovered_services: RefCell<BTreeSet<(GatewayId, ServiceHash)>>,
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
    /// Receiving half of a connection to this session.
    receiver: UnixDatagramReceiver,
}

impl Session {
    /// Create a new session that can announce services to and exchange
    /// samples/events with other live sessions on the same host.
    pub fn create() -> Result<Self, CreationError> {
        let unique_id = UniqueSystemId::new()
            .map_err(CreationError::UniqueIdCreation)?
            .value();
        let id = session_id(unique_id);
        let backend_id = BackendId::new(unique_id.to_le_bytes());

        let registry = Registry::open()?;
        let registration = registry.register(&id)?;

        let receiver = UnixDatagramReceiverBuilder::new(registration.socket_path())
            .creation_mode(CreationMode::PurgeAndCreate)
            .permission(Permission::OWNER_READ_WRITE)
            .create()
            .map_err(CreationError::SocketBind)?;

        Ok(Self {
            id,
            backend_id,
            registry,
            registration,
            connections: RefCell::new(BTreeMap::new()),
            discovered_services: RefCell::new(BTreeSet::new()),
            pending_discoveries: RefCell::new(PendingDiscovery::default()),
            received_events: RefCell::new(BTreeMap::new()),
            received_samples: RefCell::new(BTreeMap::new()),
            recv_buffer: RefCell::new(alloc::vec![0u8; MAX_DATAGRAM]),
            send_buffer: RefCell::new(alloc::vec![0u8; MAX_DATAGRAM]),
            receiver,
        })
    }

    pub fn backend_id(&self) -> BackendId {
        self.backend_id
    }

    /// Make a service offered by this session discoverable to peers.
    pub fn announce_added(
        &self,
        gateway_id: GatewayId,
        description: &ServiceDescription,
    ) -> Result<(), AnnounceError> {
        self.registration.add_service(gateway_id, description)
    }

    /// Withdraw a previously-announced service so peers stop discovering it.
    pub fn announce_removed(&self, hash: &ServiceHash) -> Result<(), AnnounceError> {
        self.registration.remove_service(hash)
    }

    /// Refresh the known set of (gateway, service) pairs; new and dropped
    /// pairs are queued for the next `pending_discoveries()` drain.
    pub fn discover(&self) {
        let sessions = self.refresh_connections();
        let mut pending = self.pending_discoveries.borrow_mut();

        let current = {
            let prev = self.discovered_services.borrow();

            // Mark newly-discovered pairs as added
            let mut current: BTreeSet<(GatewayId, ServiceHash)> = BTreeSet::new();
            for session in &sessions {
                for announced in session.services() {
                    let key = (announced.gateway_id, announced.description.service_hash);
                    if current.insert(key) && !prev.contains(&key) {
                        pending.updates.push(DiscoveryUpdate::Added(
                            announced.gateway_id,
                            announced.description,
                        ));
                    }
                }
            }

            // Mark previously-known pairs that are absent as removed
            for (gateway, hash) in prev.iter() {
                if !current.contains(&(*gateway, *hash)) {
                    pending
                        .updates
                        .push(DiscoveryUpdate::Removed(*gateway, *hash));
                }
            }

            current
        };

        *self.discovered_services.borrow_mut() = current;
    }

    /// Drain the discovery updates accumulated since the last call, in the
    /// order they were observed.
    pub fn pending_discoveries(&self) -> Vec<DiscoveryUpdate> {
        core::mem::take(&mut self.pending_discoveries.borrow_mut().updates)
    }

    /// Send an event id for the given service to all live peers.
    pub fn send_event(&self, service_hash: &ServiceHash, id: u64) -> Result<(), SendError> {
        self.refresh_connections();
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
        self.refresh_connections();
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
        let bytes = serialize_envelope(&envelope, &mut buf)?;

        for sender in self.connections.borrow().values() {
            if let Err(UnixDatagramSendError::MessageTooLarge) = sender.try_send(bytes) {
                return Err(SendError::TooLarge(bytes.len()));
            }
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

            let Some(envelope) = deserialize_envelope(&buf[..n]) else {
                continue; // skip malformed datagrams
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

    /// Refresh the connections to match the sessions currently alive in the
    /// registry.
    fn refresh_connections(&self) -> Vec<RegisteredSession> {
        let mut sessions = self.registry.sessions();
        sessions.retain(|session| session.id != self.id);
        self.reconcile_connections(&sessions);
        sessions
    }

    /// Align the tracked connections with the given live sessions.
    fn reconcile_connections(&self, sessions: &[RegisteredSession]) {
        let mut connections = self.connections.borrow_mut();

        // Drop connections to peers no longer present.
        connections.retain(|id, _| {
            sessions
                .iter()
                .any(|session| session.id.as_str() == id.as_str())
        });

        // Connect to new peers.
        for session in sessions {
            if connections.contains_key(&session.id) {
                continue;
            }
            let Ok(sender) = UnixDatagramSenderBuilder::new(&session.sock_path).create() else {
                continue; // peer may have just exited
            };
            connections.insert(session.id.clone(), sender);
        }
    }
}

fn session_id(unique_id: u128) -> SessionId {
    unique_id.to_b64().to_lowercase()
}
