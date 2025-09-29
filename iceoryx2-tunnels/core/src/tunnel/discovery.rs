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

use iceoryx2::port::ReceiveError;
use iceoryx2::{port::subscriber::Subscriber, service::Service};
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryUpdate;
use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker};

use crate::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

impl<S: Service> Discovery<S> for Tracker<S> {
    type Handle = Self;
    type Error = SyncError;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::Error>,
    >(
        handle: &mut Self::Handle,
        process_discovery: &mut F,
    ) -> Result<(), Self::Error> {
        let tracker = handle;
        let (added, _removed) = tracker.sync().unwrap();

        for id in added {
            process_discovery(&tracker.get(&id).unwrap().static_details)?;
        }

        Ok(())
    }
}

pub struct DiscoverySubscriber<S: iceoryx2::service::Service>(
    pub Subscriber<S, DiscoveryUpdate, ()>,
);

impl<S: Service> Discovery<S> for DiscoverySubscriber<S> {
    type Handle = Self;
    type Error = ReceiveError;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::Error>,
    >(
        handle: &mut Self::Handle,
        process_discovery: &mut F,
    ) -> Result<(), Self::Error> {
        let subscriber = &handle.0;
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    if let DiscoveryUpdate::Added(static_config) = sample.payload() {
                        process_discovery(static_config)?;
                    }
                }
                Ok(None) => break Ok(()),
                Err(_) => todo!(),
            }
        }
    }
}
