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

use crate::service::service_name::ServiceName;

/// Role a side-channel participant takes in a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Sending side — binds the side-channel socket.
    Publisher,
    /// Receiving side — connects to the publisher socket.
    Subscriber,
}

/// Extension point for out-of-band transport channels alongside iceoryx2 pub/sub.
///
/// ## Motivation
///
/// iceoryx2's typed SHM pool delivers value-type payloads efficiently but
/// cannot transfer kernel-owned resources such as file descriptors.
/// `SideChannel` is a cross-platform, zero-assumption role marker that
/// downstream crates implement to add a complementary transport.  It is
/// deliberately free of Linux syscall surface so that it compiles everywhere.
///
/// The canonical downstream implementation is `iceoryx2-dmabuf`, which
/// implements `SideChannel` (and its Linux-specific `FdSideChannel` extension)
/// via a Unix domain socket with `SCM_RIGHTS` ancillary data.
///
/// ## Send + Sync contract
///
/// Implementations that are shared across threads (e.g., wrapped in
/// `Arc<Mutex<T>>`) MUST be `Send + Sync`.  Single-threaded use only requires
/// `Send`.
///
/// ## Implementing `SideChannel`
///
/// ```ignore
/// use iceoryx2::port::side_channel::{Role, SideChannel};
/// use iceoryx2::service::service_name::ServiceName;
///
/// struct MyChannel {
///     // ... transport state ...
/// }
///
/// impl SideChannel for MyChannel {
///     type Error = std::io::Error;
///     type Transport = ();   // replace with your transport type
///
///     fn open(service_name: &ServiceName, role: Role) -> Result<Self, Self::Error> {
///         // Bind (Publisher) or connect (Subscriber) based on `role`.
///         let _ = (service_name, role);
///         Ok(MyChannel { /* ... */ })
///     }
///
///     fn transport(&mut self) -> &mut Self::Transport {
///         // Return a mutable reference to the underlying transport.
///         todo!()
///     }
/// }
/// ```
///
/// For a complete Linux DMA-BUF implementation using `SCM_RIGHTS`, see the
/// `iceoryx2-dmabuf` crate.
pub trait SideChannel: Sized {
    /// Error type returned by [`SideChannel::open`].
    type Error: core::error::Error;
    /// The underlying transport (e.g. a Unix-domain socket).
    type Transport;
    /// Open a side channel for the given service in the given role.
    fn open(service_name: &ServiceName, role: Role) -> Result<Self, Self::Error>;
    /// Access the underlying transport for sending or receiving.
    fn transport(&mut self) -> &mut Self::Transport;
}
