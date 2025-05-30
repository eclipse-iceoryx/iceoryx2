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

use std::collections::HashSet;

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::listener::Listener as IceoryxListener;
use iceoryx2::port::notifier::Notifier as IceoryxNotifier;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::info;

use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use super::Connection;
use super::PropagationError;
use crate::iox_create_event_service;
use crate::iox_create_listener;
use crate::iox_create_notifier;
use crate::z_announce_service;
use crate::z_create_listener;
use crate::z_create_notifier;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

pub(crate) struct OutboundEventConnection<'a, ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_listener: IceoryxListener<ServiceType>,
    z_notifier: ZenohPublisher<'a>,
}

impl<ServiceType: iceoryx2::service::Service> OutboundEventConnection<'_, ServiceType> {
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_event_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_listener = iox_create_listener(iox_event_service, iox_service_config)
            .map_err(|_e| CreationError::Error)?;
        let z_notifier =
            z_create_notifier(z_session, iox_service_config).map_err(|_e| CreationError::Error)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_listener,
            z_notifier,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for OutboundEventConnection<'_, ServiceType>
{
    fn propagate(&self) -> Result<(), PropagationError> {
        // TODO(correctness): group notifications, only forward one-per-unique-id
        while let Ok(Some(_event_id)) = self.iox_listener.try_wait_one() {
            self.z_notifier.put([]).wait().unwrap();
            info!(
                "PROPAGATED(iceoryx2->zenoh): EVENT {} [{}]",
                self.iox_service_config.service_id().as_str(),
                self.iox_service_config.name()
            );
        }

        Ok(())
    }
}

pub(crate) struct InboundEventConnection<ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_notifier: IceoryxNotifier<ServiceType>,
    z_listener: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> Connection for InboundEventConnection<ServiceType> {
    fn propagate(&self) -> Result<(), PropagationError> {
        let _notified_empty = false;
        let _notified_ids: HashSet<usize> = HashSet::new();

        // TODO(correctness): group notifications, only forward one-per-unqiue-id
        while let Ok(Some(_sample)) = self.z_listener.try_recv() {
            self.iox_notifier.notify().unwrap();
            info!(
                "PROPAGATED(iceoryx2<-zenoh): EVENT {} [{}]",
                self.iox_service_config.service_id().as_str(),
                self.iox_service_config.name()
            );
        }

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service> InboundEventConnection<ServiceType> {
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_event_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_notifier = iox_create_notifier(iox_event_service, iox_service_config)
            .map_err(|_e| CreationError::Error)?;
        let z_listener =
            z_create_listener(z_session, iox_service_config).map_err(|_e| CreationError::Error)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_notifier,
            z_listener,
        })
    }
}

pub(crate) struct BidirectionalEventConnection<'a, ServiceType: iceoryx2::service::Service> {
    outbound_connection: OutboundEventConnection<'a, ServiceType>,
    inbound_connection: InboundEventConnection<ServiceType>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for BidirectionalEventConnection<'_, ServiceType>
{
    fn propagate(&self) -> Result<(), PropagationError> {
        self.outbound_connection.propagate()?;
        self.inbound_connection.propagate()?;

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service> BidirectionalEventConnection<'_, ServiceType> {
    pub fn create(
        iox_node: &IceoryxNode<ServiceType>,
        z_session: &ZenohSession,
        iox_service_config: &IceoryxServiceConfig,
    ) -> Result<Self, CreationError> {
        let iox_event_service =
            iox_create_event_service::<ServiceType>(iox_node, iox_service_config)
                .map_err(|_e| CreationError::Error)?;

        let inbound_connection =
            InboundEventConnection::create(iox_service_config, &iox_event_service, z_session)?;
        let outbound_connection =
            OutboundEventConnection::create(iox_service_config, &iox_event_service, z_session)?;

        z_announce_service(&z_session, iox_service_config).map_err(|_e| CreationError::Error)?;

        Ok(Self {
            outbound_connection,
            inbound_connection,
        })
    }
}
