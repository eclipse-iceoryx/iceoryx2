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

use crate::DataStream;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::prelude::*;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashMap;

///
/// # Design
///
/// ## Discovery
///
/// - For all (new) iceoryx2 services discovered
///     - Create an iceoryx2 subscriber
///         - Avoid connecting to own !
///     - Create a Zenoh publisher
///         - Remote locality
/// - For all zenoh (new) keys discovered
///     - Create an iceoryx2 publisher
///
/// # Propagation
///
/// - Periodically propagate ... maybe make reactive later
/// - For all iceoryx2 subscribers
///     - Pull all new samples
///     - Publish on corresponding Zenoh key
/// - For all zenoh keys
///     - Pull all data from Zenoh
///     - Publish on corresponding iceoryx2 service
///
/// # Open Questions
///
/// - How to prevent loopback ?
///     - Is it possible to ignore "own host" on Zenoh ?
///
pub struct ZenohTunnel<'a> {
    z_session: ZenohSession,
    iox_config: IceoryxConfig,
    iox_node: IceoryxNode<ipc::Service>,
    iox_tracker: IceoryxServiceTracker<ipc::Service>,
    streams: HashMap<IceoryxServiceId, DataStream<'a>>,
}

impl<'a> ZenohTunnel<'a> {
    pub fn new(iox_config: IceoryxConfig) -> Self {
        let mut z_config = zenoh::config::Config::default();
        z_config.insert_json5("adminspace/enabled", "true").unwrap(); // this is mandatory
        let z_session = zenoh::open(z_config).wait().unwrap();

        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<ipc::Service>()
            .unwrap();
        let iox_tracker = IceoryxServiceTracker::new();

        let streams: HashMap<IceoryxServiceId, DataStream<'a>> = HashMap::new();

        Self {
            z_session,
            iox_config,
            iox_node,
            iox_tracker,
            streams,
        }
    }

    // TODO: enforce setup before spinning?
    pub fn initialize(&mut self) {
        info!("Zenoh Tunnel UP");
    }

    pub fn discover(&mut self) {
        self.iceoryx_discovery();
    }

    pub fn propagate(&self) {
        for (_id, stream) in &self.streams {
            stream.propagate();
        }
    }

    pub fn shutdown(&mut self) {
        info!("Zenoh Tunnel DOWN");
    }

    fn iceoryx_discovery(&mut self) {
        let (added, _removed) = self.iox_tracker.sync(&self.iox_config).unwrap();

        for id in added {
            let iox_service = self.iox_tracker.get(&id).unwrap();

            if let MessagingPattern::PublishSubscribe(_) =
                iox_service.static_details.messaging_pattern()
            {
                info!(
                    "DISCOVERY (iceoryx2): {}",
                    iox_service.static_details.name().as_str()
                );

                if !self
                    .streams
                    .contains_key(&iox_service.static_details.service_id())
                {
                    self.streams.insert(
                        iox_service.static_details.service_id().clone(),
                        DataStream::new_outbound(&self.z_session, &self.iox_node, iox_service),
                    );
                } else {
                    // TODO
                }
            }
        }
    }
}
