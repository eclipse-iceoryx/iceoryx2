// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2::{node::Node, service::Service};

use crate::{Discovery, Transport, Tunnel};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

impl<S: Service, T: Transport> Discovery<S> for Tunnel<S, T> {
    type Handle = Node<S>;
    type Error = Error;

    /// Retrieve discovery information from the transport and then call the provided
    /// handler function to process it.
    fn discover<
        OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::Error>,
    >(
        node: &Self::Handle,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), Self::Error> {
        // Here we should get discovery information from iceoryx2 itself.
        todo!()
    }
}
