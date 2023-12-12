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
    { $id_name:ident } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

generate_id! { UniquePublisherId }
generate_id! { UniqueSubscriberId }
generate_id! { UniqueNotifierId }
generate_id! { UniqueListenerId }
