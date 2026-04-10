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

use alloc::format;

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_log::fatal_panic;

macro_rules! generate_id {
    { $(#[$documentation:meta])*
        $id_name:ident } => {
        $(#[$documentation])*
        #[repr(C)]
        #[derive(
            Debug,
            Eq,
            Hash,
            PartialEq,
            Clone,
            Copy,
            PartialOrd,
            Ord,
            ZeroCopySend,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $id_name(pub(crate) UniqueSystemId);

        impl core::fmt::Display for $id_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "0x{:032x}", self.0.value())
            }
        }

        impl $id_name {
            pub(crate) fn new() -> Self {
                Self(
                    fatal_panic!(from format!("{}::new()", stringify!($id_name)), when UniqueSystemId::new(),
                        "Unable to generate required {}!", stringify!($id_name)),
                )
            }

            /// Returns the underlying raw value of the ID
            pub fn value(&self) -> u128 {
                self.0.value()
            }

            /// Returns the [`ProcessId`](iceoryx2_bb_posix::process::ProcessId) of the process that created the id.
            pub fn pid(&self) -> iceoryx2_bb_posix::process::ProcessId {
                self.0.pid()
            }

            /// Returns the [`Time`](iceoryx2_bb_posix::time::Time) the [`Node`] was created.
            pub fn creation_time(&self) -> iceoryx2_bb_posix::clock::Time {
                self.0.creation_time()
            }
        }
    };
}

generate_id! {
    /// The system-wide unique id of a [`Publisher`](crate::port::publisher::Publisher).
    UniquePublisherId
}
generate_id! {
    /// The system-wide unique id of a [`Subscriber`](crate::port::subscriber::Subscriber).
    UniqueSubscriberId
}
generate_id! {
    /// The system-wide unique id of a [`Notifier`](crate::port::notifier::Notifier).
    UniqueNotifierId
}
generate_id! {
    /// The system-wide unique id of a [`Listener`](crate::port::listener::Listener).
    UniqueListenerId
}
generate_id! {
    /// The system-wide unique id of a [`Client`](crate::port::client::Client).
    UniqueClientId
}
generate_id! {
    /// The system-wide unique id of a [`Server`](crate::port::server::Server).
    UniqueServerId
}
generate_id! {
    /// The system-wide unique id of a [`Reader`](crate::port::reader::Reader).
    UniqueReaderId
}
generate_id! {
    /// The system-wide unique id of a [`Writer`](crate::port::writer::Writer).
    UniqueWriterId
}

generate_id! {
    /// The system-wide unique id of a [`Service`](crate::service::Service).
    UniqueServiceId
}

generate_id! {
    /// The system-wide unique id of a [`Node`](crate::node::Node).
    UniqueNodeId
}

/// Enum that contains the unique port id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniquePortId {
    /// The system-wide unique id of a [`Publisher`](crate::port::publisher::Publisher).
    Publisher(UniquePublisherId),
    /// The system-wide unique id of a [`Subscriber`](crate::port::subscriber::Subscriber).
    Subscriber(UniqueSubscriberId),
    /// The system-wide unique id of a [`Notifier`](crate::port::notifier::Notifier).
    Notifier(UniqueNotifierId),
    /// The system-wide unique id of a [`Listener`](crate::port::listener::Listener).
    Listener(UniqueListenerId),
    /// The system-wide unique id of a [`Client`](crate::port::client::Client).
    Client(UniqueClientId),
    /// The system-wide unique id of a [`Server`](crate::port::server::Server).
    Server(UniqueServerId),
    /// The system-wide unique id of a [`Reader`](crate::port::reader::Reader).
    Reader(UniqueReaderId),
    /// The system-wide unique id of a [`Writer`](crate::port::writer::Writer).
    Writer(UniqueWriterId),
}
