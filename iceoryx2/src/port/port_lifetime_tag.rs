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

use core::ptr::NonNull;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_log::fail;

use crate::{node::SharedNode, service};

#[derive(Debug)]
pub struct PortLifetimeTag<Service: service::Service> {
    port_tag: Service::StaticStorage,
    // The last element that is being destroyed must be the shared node to ensure the destruction
    // order of port -> service -> node.
    shared_node: SharedNode<Service>,
}

impl<Service: service::Service> PortLifetimeTag<Service> {
    pub fn new<E: core::error::Error>(
        origin: &str,
        msg: &str,
        id_value: u128,
        shared_node: &SharedNode<Service>,
        error: E,
    ) -> Result<Self, E> {
        let port_tag = match shared_node.create_port_tag(origin, msg, id_value) {
            Ok(port_tag) => port_tag,
            Err(e) => {
                fail!(from origin, with error,
                        "{msg} since the port tag, that is required for cleanup, could not be created. [{e:?}]");
            }
        };

        Ok(Self {
            port_tag,
            shared_node: shared_node.clone(),
        })
    }
}

impl<Service: service::Service> Abandonable for PortLifetimeTag<Service> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { Service::StaticStorage::abandon_in_place(NonNull::from_mut(&mut this.port_tag)) };
        unsafe {
            SharedNode::<Service>::abandon_in_place(NonNull::from_mut(&mut this.shared_node))
        };
    }
}
