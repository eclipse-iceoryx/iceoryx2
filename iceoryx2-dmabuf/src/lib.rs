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

// Unsafe is forbidden at the crate level.  The only exceptions are the
// Linux-specific syscall wrappers in `scm.rs` (marked `#[allow(unsafe_code)]`
// at the function level) and test-only `#[allow(unsafe_code)]` blocks.
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod error;
pub mod path;
pub mod publisher;
pub mod scm;
pub(crate) mod side_channel;
pub mod subscriber;
pub mod token;

pub use error::{DmabufError, Result};
pub use path::uds_path_for_service;
pub use publisher::DmabufPublisher;
pub use subscriber::DmabufSubscriber;
pub use token::DmabufToken;

/// Convenience alias: [`DmabufPublisher`] bound to the IPC service type.
pub type DmabufIpcPublisher<Meta> = DmabufPublisher<iceoryx2::service::ipc::Service, Meta>;

/// Convenience alias: [`DmabufSubscriber`] bound to the IPC service type.
pub type DmabufIpcSubscriber<Meta> = DmabufSubscriber<iceoryx2::service::ipc::Service, Meta>;

/// Build an iceoryx2 node and publish-subscribe port factory for a given
/// service name.
///
/// # Lifetime contract
///
/// The returned `Node<S>` **must** outlive the `PortFactory<S, Meta,
/// DmabufToken>` and any ports derived from it.  Dropping the node before
/// the port causes a use-after-free in iceoryx2's SHM bookkeeping.  Callers
/// must store the node as a struct field (e.g. `_node: Node<S>`) alongside
/// the derived port so that drop order is guaranteed by struct field ordering.
///
/// # Errors
///
/// Returns [`DmabufError::Iceoryx`] if node creation, service name parsing,
/// or `open_or_create` fails.
pub(crate) fn build_node_and_service<S, Meta>(
    service_name: &str,
) -> crate::Result<(
    iceoryx2::node::Node<S>,
    iceoryx2::service::port_factory::publish_subscribe::PortFactory<S, Meta, DmabufToken>,
)>
where
    S: iceoryx2::service::Service,
    Meta: iceoryx2::prelude::ZeroCopySend + core::fmt::Debug,
{
    use crate::error::IceoryxErrorKind;
    use iceoryx2::prelude::NodeBuilder;

    let node = NodeBuilder::new()
        .create::<S>()
        .map_err(|e| DmabufError::Iceoryx {
            kind: IceoryxErrorKind::NodeCreate,
            msg: e.to_string(),
        })?;

    let svc_name: iceoryx2::service::service_name::ServiceName = service_name.try_into().map_err(
        |e: iceoryx2::service::service_name::ServiceNameError| DmabufError::Iceoryx {
            kind: IceoryxErrorKind::Service,
            msg: e.to_string(),
        },
    )?;

    let factory = node
        .service_builder(&svc_name)
        .publish_subscribe::<Meta>()
        .user_header::<DmabufToken>()
        .open_or_create()
        .map_err(|e| DmabufError::Iceoryx {
            kind: IceoryxErrorKind::Service,
            msg: e.to_string(),
        })?;

    Ok((node, factory))
}
