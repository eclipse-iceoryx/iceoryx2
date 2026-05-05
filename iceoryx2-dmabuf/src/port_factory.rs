// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`DmabufPortFactory`] — factory for `dmabuf` service publishers and subscribers.
//!
//! Returned by [`crate::service::Service::open_or_create`]. Holds the
//! iceoryx2 `Node` and metadata `PortFactory`; defers fd-channel construction
//! until a port is requested so that the factory itself is role-agnostic.

use core::fmt::Debug;
use iceoryx2::node::Node;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::ipc;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPortFactory;

use crate::service_error::ServiceError;
use crate::service_publisher::DmaBufServicePublisher;
use crate::service_subscriber::DmaBufServiceSubscriber;

/// Factory for [`DmaBufServicePublisher`] and [`DmaBufServiceSubscriber`] ports.
///
/// Constructed via [`crate::service::Service::open_or_create`].
///
/// ## Lifetime contract
///
/// `_node` is declared **first** so it is dropped **last** (Rust drops struct
/// fields in declaration order). All iceoryx2 ports created from this factory
/// must not outlive the factory itself — which in turn must not outlive the node.
/// Enforce this by keeping the factory alive (e.g. in the same scope) as any
/// publisher or subscriber built from it.
pub struct DmabufPortFactory<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Node MUST be declared first — dropped last. See struct-level doc.
    _node: Node<ipc::Service>,
    meta_factory: IceoryxPortFactory<ipc::Service, Meta, ()>,
    socket_path: String,
}

impl<Meta> DmabufPortFactory<Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Construct a new factory. Called only from [`crate::service::Service::open_or_create`].
    pub(crate) fn new(
        node: Node<ipc::Service>,
        meta_factory: IceoryxPortFactory<ipc::Service, Meta, ()>,
        socket_path: String,
    ) -> Self {
        Self {
            _node: node,
            meta_factory,
            socket_path,
        }
    }

    /// Return a builder for a [`DmaBufServicePublisher`].
    ///
    /// The publisher will bind the fd channel (UDS server socket) and create an
    /// iceoryx2 publisher port for the metadata channel.
    pub fn publisher_builder(&self) -> PublisherBuilder<'_, Meta> {
        PublisherBuilder {
            meta_factory: &self.meta_factory,
            socket_path: &self.socket_path,
        }
    }

    /// Return a builder for a [`DmaBufServiceSubscriber`].
    ///
    /// The subscriber will connect to the fd channel and create an iceoryx2
    /// subscriber port for the metadata channel.
    pub fn subscriber_builder(&self) -> SubscriberBuilder<'_, Meta> {
        SubscriberBuilder {
            meta_factory: &self.meta_factory,
            socket_path: &self.socket_path,
        }
    }
}

// ── PublisherBuilder ──────────────────────────────────────────────────────────

/// Builder for a [`DmaBufServicePublisher`].
///
/// Obtained from [`DmabufPortFactory::publisher_builder`].
pub struct PublisherBuilder<'a, Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    pub(crate) meta_factory: &'a IceoryxPortFactory<ipc::Service, Meta, ()>,
    pub(crate) socket_path: &'a str,
}

impl<Meta> PublisherBuilder<'_, Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Create a [`DmaBufServicePublisher`] by binding the fd channel and creating
    /// the iceoryx2 publisher port.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 port creation fails.
    /// - [`ServiceError::Connection`] — if binding the UDS socket fails.
    pub fn create(self) -> Result<DmaBufServicePublisher<Meta>, ServiceError> {
        DmaBufServicePublisher::create(self.meta_factory, self.socket_path)
    }
}

// ── SubscriberBuilder ─────────────────────────────────────────────────────────

/// Builder for a [`DmaBufServiceSubscriber`].
///
/// Obtained from [`DmabufPortFactory::subscriber_builder`].
pub struct SubscriberBuilder<'a, Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    pub(crate) meta_factory: &'a IceoryxPortFactory<ipc::Service, Meta, ()>,
    pub(crate) socket_path: &'a str,
}

impl<Meta> SubscriberBuilder<'_, Meta>
where
    Meta: ZeroCopySend + Debug + Copy + 'static,
{
    /// Create a [`DmaBufServiceSubscriber`] by connecting to the fd channel and
    /// creating the iceoryx2 subscriber port.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if the iceoryx2 port creation fails.
    /// - [`ServiceError::Connection`] — if connecting to the UDS socket fails.
    pub fn create(self) -> Result<DmaBufServiceSubscriber<Meta>, ServiceError> {
        DmaBufServiceSubscriber::create(self.meta_factory, self.socket_path)
    }
}
