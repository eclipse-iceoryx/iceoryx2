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

// `FdSidecarSubscriber` in module `subscriber` triggers module_name_repetitions.
// Renaming the module adds churn without correctness benefit — allowed at module level per spec §NFR Clippy.
#![allow(clippy::module_name_repetitions)]

//! [`FdSidecarSubscriber`] — composes an iceoryx2 subscriber with the SCM_RIGHTS
//! side-channel to receive DMA-BUF file descriptors alongside metadata
//! samples.

use core::fmt::Debug;
use core::time::Duration;
use iceoryx2::node::Node;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use std::os::fd::OwnedFd;

use crate::error::{FdSidecarError, IceoryxErrorKind, Result};
use crate::scm::ScmRightsSubscriber;
use crate::token::FdSidecarToken;

/// Default timeout for [`FdSidecarSubscriber::recv`] when waiting for the fd on
/// the side-channel socket.  50 ms matches the spec §Fault model.
const DEFAULT_RECV_TIMEOUT: Duration = Duration::from_millis(50);

/// SCM_RIGHTS fd-sidecar subscriber.
///
/// Composes an iceoryx2 `Subscriber<S, Meta, FdSidecarToken>` with a
/// [`ScmRightsSubscriber`] that receives file descriptors out-of-band via a
/// Unix-domain socket using `SCM_RIGHTS`.
///
/// # Type parameters
///
/// - `S` — iceoryx2 service type (e.g. [`iceoryx2::service::ipc::Service`]).
///   Use `FdSidecarIpcSubscriber` for the common IPC case.
/// - `Meta` — application payload type; must be `ZeroCopySend + Debug`.
pub struct FdSidecarSubscriber<S: Service, Meta: ZeroCopySend + Debug + 'static> {
    /// Node MUST be declared before `inner` and `_port_factory` so it is
    /// dropped last (Rust drops fields in declaration order).  See
    /// `crate::build_node_and_service` for the Node lifetime contract.
    _node: Node<S>,
    inner: Subscriber<S, Meta, FdSidecarToken>,
    side: ScmRightsSubscriber,
    service_name: String,
    // Keep the port factory alive so the iceoryx2 service is not dropped.
    _port_factory: PortFactory<S, Meta, FdSidecarToken>,
}

impl<S: Service, Meta: ZeroCopySend + Debug + Copy + 'static> FdSidecarSubscriber<S, Meta> {
    /// Create a new `FdSidecarSubscriber` for `service_name`.
    ///
    /// Opens (or creates) an iceoryx2 service of type `S` with the given name,
    /// configures `FdSidecarToken` as the user-header type, and connects a
    /// Unix-domain socket side-channel for fd reception.
    ///
    /// `_node` is stored to guarantee it outlives the port.
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::UnsupportedPlatform`] — on non-Linux targets.
    /// - [`FdSidecarError::SideChannelIo`] — if the UDS socket cannot connect.
    pub fn create(service_name: &str) -> Result<Self> {
        use iceoryx2::port::side_channel::Role;

        let (_node, port_factory) = crate::build_node_and_service::<S, Meta>(service_name)?;

        let subscriber =
            port_factory
                .subscriber_builder()
                .create()
                .map_err(|e| FdSidecarError::Iceoryx {
                    kind: IceoryxErrorKind::PortBuilder,
                    msg: e.to_string(),
                })?;

        // Connect the SCM_RIGHTS side-channel subscriber.
        let side = ScmRightsSubscriber::open(service_name, Role::Subscriber)?;

        Ok(Self {
            _node,
            inner: subscriber,
            side,
            service_name: service_name.to_owned(),
            _port_factory: port_factory,
        })
    }

    /// Non-blocking receive.
    ///
    /// Returns `None` if no sample is ready in the iceoryx2 queue.
    ///
    /// On success:
    /// 1. Receives the next iceoryx2 sample.
    /// 2. Extracts the correlation `token` from the sample's `user_header`.
    /// 3. Calls [`ScmRightsSubscriber::recv_fd_matching_impl`] with a
    ///    50 ms timeout to drain the side-channel until the matching fd arrives.
    /// 4. Copies the `Meta` payload out of the SHM slot (which is then returned
    ///    to the pool).
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::NoFdInMessage`] — side-channel timed out (producer
    ///   may have crashed between the sidecar send and the iceoryx2 send).
    /// - [`FdSidecarError::TokenMismatch`] — out-of-order fd delivery.
    /// - [`FdSidecarError::IceoryxReceive`] — the iceoryx2 receive failed.
    pub fn recv(&mut self) -> Result<Option<(Meta, OwnedFd)>> {
        let Some(sample) = self
            .inner
            .receive()
            .map_err(FdSidecarError::IceoryxReceive)?
        else {
            return Ok(None);
        };

        // Extract the expected token from the user-header; zero means corrupted header.
        let expected = sample
            .user_header()
            .as_nonzero()
            .ok_or(FdSidecarError::NoFdInMessage)?;

        // Drain the sidecar until we find the fd matching `expected`.
        let fd = self
            .side
            .recv_fd_matching_impl(expected, DEFAULT_RECV_TIMEOUT)?;

        // Copy the meta before the sample is dropped (which returns the SHM
        // slot to the pool).
        let meta = *sample.payload();

        Ok(Some((meta, fd)))
    }

    /// Reconnect the side-channel subscriber.
    ///
    /// Call this after a [`FdSidecarError::SideChannelIo`] error to re-establish
    /// the Unix-domain socket connection without recreating the iceoryx2 port.
    ///
    /// # Errors
    ///
    /// - [`FdSidecarError::UnsupportedPlatform`] — on non-Linux targets.
    /// - [`FdSidecarError::SideChannelIo`] — if the UDS socket cannot connect.
    pub fn reconnect(&mut self) -> Result<()> {
        use iceoryx2::port::side_channel::Role;
        self.side = ScmRightsSubscriber::open(&self.service_name, Role::Subscriber)?;
        Ok(())
    }
}
