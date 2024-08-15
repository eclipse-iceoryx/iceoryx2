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

use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;

macro_rules! generate_id {
    { $(#[$documentation:meta])*
        $id_name:ident } => {
        $(#[$documentation])*
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $id_name(pub(crate) UniqueSystemId);

        impl Default for $id_name {
            fn default() -> Self {
                Self(
                    fatal_panic!(from format!("{}::new()", stringify!($id_name)), when UniqueSystemId::new(),
                        "Unable to generate required {}!", stringify!($id_name)),
                )
            }
        }

        impl $id_name {
            /// Creates a new instance
            pub fn new() -> Self {
                Self::default()
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
}
