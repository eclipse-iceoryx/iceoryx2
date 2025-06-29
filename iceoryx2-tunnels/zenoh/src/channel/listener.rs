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

use crate::middleware;
use crate::Channel;
use crate::PropagationError;

use iceoryx2::port::notifier::Notifier as IceoryxNotifier;
use iceoryx2::prelude::EventId;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::info;

use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;

use std::collections::HashSet;

// TODO: More granularity in errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

/// A channel for propagating remote `iceoryx2` notifications from remote hosts
/// to local listeners.
pub(crate) struct ListenerChannel<ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_notifier: IceoryxNotifier<ServiceType>,
    z_listener: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> ListenerChannel<ServiceType> {
    // Creates an inbound channel for notifications from remote hosts for a
    // particular service.
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_event_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_notifier =
            middleware::iceoryx::create_notifier(iox_event_service, iox_service_config)
                .map_err(|_e| CreationError::Error)?;
        let z_listener = middleware::zenoh::create_listener(z_session, iox_service_config)
            .map_err(|_e| CreationError::Error)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_notifier,
            z_listener,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Channel for ListenerChannel<ServiceType> {
    /// Propagate remote notifications for a particular service to local listeners.
    fn propagate(&self) -> Result<(), PropagationError> {
        // Collect all notified ids
        let mut received_ids: HashSet<usize> = HashSet::new();
        while let Ok(Some(sample)) = self.z_listener.try_recv() {
            let payload = sample.payload();
            if payload.len() == std::mem::size_of::<usize>() {
                let id: usize =
                    unsafe { payload.to_bytes().as_ptr().cast::<usize>().read_unaligned() };
                received_ids.insert(id);
            } else {
                // Error, invalid event id. Skip.
            }
        }

        // Propagate notifications received - once per event id
        for id in received_ids {
            self.iox_notifier
                .__internal_notify(EventId::new(id), true)
                .map_err(|_| PropagationError::IceoryxPort)?;
            info!(
                "PROPAGATED(iceoryx<-zenoh): Event({}) {} [{}]",
                id,
                self.iox_service_config.service_id().as_str(),
                self.iox_service_config.name()
            );
        }

        Ok(())
    }
}
