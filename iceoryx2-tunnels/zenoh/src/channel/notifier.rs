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

use iceoryx2::port::listener::Listener as IceoryxListener;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::info;

use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashSet;

// TODO: More granularity in errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

/// A channel for propagating local `iceoryx2` notifications to remote hosts.
#[derive(Debug)]
pub(crate) struct NotifierChannel<'a, ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_listener: IceoryxListener<ServiceType>,
    z_notifier: ZenohPublisher<'a>,
}

impl<ServiceType: iceoryx2::service::Service> NotifierChannel<'_, ServiceType> {
    // Creates an outbound channel for local notifications on a particular service
    // to remote hosts.
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        info!(
            "CREATE NotifierChannel {} [{}]",
            iox_service_config.service_id().as_str(),
            iox_service_config.name()
        );

        let iox_listener = fail!(
            from "NotifierChannel::create()",
            when middleware::iceoryx::create_listener(iox_service),
            with CreationError::Error,
            "failed to create iceoryx listener for local notifications"
        );
        let z_notifier = fail!(
            from "NotifierChannel::create()",
            when middleware::zenoh::create_notifier(z_session, iox_service_config),
            with CreationError::Error,
            "failed to create zenoh notifier to propagate local notificaitons to remote hosts"
        );

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_listener,
            z_notifier,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Channel for NotifierChannel<'_, ServiceType> {
    /// Propagate local notifications received on the service to remote hosts.
    fn propagate(&self) -> Result<(), PropagationError> {
        // Propagate all notified ids once
        let mut notified_ids: HashSet<usize> = HashSet::new();
        while let Ok(sample) = self.iox_listener.try_wait_one() {
            match sample {
                Some(event_id) => {
                    if !notified_ids.contains(&event_id.as_value()) {
                        fail!(
                            from &self,
                            when self.z_notifier.put(event_id.as_value().to_ne_bytes()).wait(),
                            with PropagationError::OtherPort,
                            "failed to propagate local notification to remote hosts"
                        );
                        info!(
                            "PROPAGATE NotifierChannel(EventId={}) {} [{}]",
                            event_id.as_value(),
                            self.iox_service_config.service_id().as_str(),
                            self.iox_service_config.name()
                        );
                        notified_ids.insert(event_id.as_value());
                    }
                }
                None => break,
            }
        }

        Ok(())
    }
}
