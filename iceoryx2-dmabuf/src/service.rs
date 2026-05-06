// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `dmabuf::Service` — parallel transport service for DMA-BUF file descriptors.
//!
//! This is a **parallel** concept, not an implementation of
//! [`iceoryx2::service::Service`]. It composes:
//!
//! - A `Node<ipc::Service>` for service discovery and lifetime management.
//! - An iceoryx2 `PortFactory<ipc::Service, Meta, ()>` for the **metadata channel**
//!   — `Meta` values travel inside iceoryx2's shared memory as publish payloads.
//!   The user-header is `()` (no internal token; ordering relies on single-publisher FIFO guarantee).
//! - A Unix-domain socket path for the **fd channel** — DMA-BUF file descriptors
//!   travel out-of-band via `SCM_RIGHTS` over a Unix-domain socket.
//!
//! ## Ordering invariant
//!
//! The publisher sends the fd **first** (via the fd channel), then publishes the
//! iceoryx2 metadata sample. The subscriber dequeues the iceoryx2 sample **first**,
//! then drains the fd from the socket. This relies on:
//!
//! 1. The iceoryx2 channel and the UDS socket both maintain FIFO ordering.
//! 2. A single active publisher shares both channels. Multi-publisher is **NOT**
//!    supported in this version — two concurrent publishers would silently
//!    correlate the wrong fd with the wrong metadata sample.
//!
//! Multi-subscriber fanout IS supported: each subscriber has its own UDS
//! connection and receives all fds independently. See `bench_fanout` and
//! `connection_tests::fanout_one_pub_three_sub_100_frames` for examples.
//!
//! Multi-publisher support requires widening the wire format (planned for Task 4b).

use core::fmt::Debug;
use iceoryx2::node::NodeBuilder;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::ipc;

use crate::port_factory::DmabufPortFactory;
use crate::service_error::ServiceError;

/// Namespace struct for creating or opening a `dmabuf` service.
///
/// `Service` itself carries no state; all state lives in the returned
/// [`DmabufPortFactory`]. Use [`Service::open_or_create`] to obtain one.
pub struct Service;

impl Service {
    /// Open or create a `dmabuf` service with the given `name`.
    ///
    /// Idempotent: calling this twice with the same name from the same process
    /// returns two independent [`DmabufPortFactory`] instances that share the
    /// underlying iceoryx2 service.
    ///
    /// The Unix-domain socket path is derived deterministically from `name`
    /// via `crate::path::uds_path_for_service`. The socket parent directory
    /// is created if it does not exist.
    ///
    /// # Type parameters
    ///
    /// `Meta` — application payload type sent alongside every fd.
    ///
    /// # Errors
    ///
    /// - [`ServiceError::Iceoryx`] — if node or service creation fails.
    /// - [`ServiceError::Io`] — if the socket directory cannot be created.
    pub fn open_or_create<Meta>(name: &str) -> Result<DmabufPortFactory<Meta>, ServiceError>
    where
        Meta: ZeroCopySend + Debug + Copy + 'static,
    {
        // Derive socket path and ensure the parent directory exists.
        let socket_path = crate::path::uds_path_for_service(name);
        if let Some(parent) = std::path::Path::new(&socket_path).parent() {
            std::fs::create_dir_all(parent).map_err(ServiceError::Io)?;
        }

        // Build the iceoryx2 node.
        let node = NodeBuilder::new()
            .create::<ipc::Service>()
            .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

        // Construct the service name (iceoryx2 `ServiceName`).
        let service_name = iceoryx2::service::service_name::ServiceName::new(name)
            .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

        // Open or create the iceoryx2 publish-subscribe service for Meta.
        // User-header is `()` — no token; ordering is guaranteed by SPSC contract.
        let meta_factory = node
            .service_builder(&service_name)
            .publish_subscribe::<Meta>()
            .open_or_create()
            .map_err(|e| ServiceError::Iceoryx(e.to_string()))?;

        Ok(DmabufPortFactory::new(node, meta_factory, socket_path))
    }
}
