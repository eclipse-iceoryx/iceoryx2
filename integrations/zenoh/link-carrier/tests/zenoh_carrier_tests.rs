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
use core::time::Duration;
use std::time::Instant;

use iceoryx2::service::ipc::Service as Ipc;
use iceoryx2::service::local::Service as Local;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_testing::instantiate_conformance_tests;
use iceoryx2_integrations_zenoh_link_carrier::{ZenohCarrier, channels_of};
use iceoryx2_link_conformance_tests::fixture::{CarrierFixture, TunnelFixture, TunnelLinkFixture};
use iceoryx2_link_conformance_tests::parameters::{
    AnyName, Event, FixedSizePayload, PublishSubscribe,
};
use zenoh::sample::Locality;
use zenoh::session::ZenohId;
use zenoh::{Session, Wait};

/// The scouting delay of every session, shortened from zenoh's default
/// so peers find each other within a scenario.
const SCOUTING_DELAY_MS: u64 = 50;
/// The pause between two looks at the sessions' peers and matching status.
const POLL_PERIOD: Duration = Duration::from_millis(10);

fn config() -> zenoh::Config {
    let mut config = zenoh::Config::default();
    config
        .scouting
        .set_delay(Some(SCOUTING_DELAY_MS))
        .expect("the scouting delay is set");
    config
}

/// A zenoh mesh on this host and the sessions of the carriers on it.
struct ZenohFixture {
    sessions: Vec<Session>,
}

impl ZenohFixture {
    /// Whether every carrier's session lists every other one as a peer.
    fn is_meshed(&self) -> bool {
        self.sessions.iter().all(|session| {
            let peers: Vec<ZenohId> = session.info().peers_zid().wait().collect();
            self.sessions
                .iter()
                .filter(|other| other.zid() != session.zid())
                .all(|other| peers.contains(&other.zid()))
        })
    }
}

impl CarrierFixture for ZenohFixture {
    type Carrier = ZenohCarrier;

    fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    fn carrier(&mut self) -> Self::Carrier {
        let session = zenoh::open(config())
            .wait()
            .expect("the carrier's session opens");
        self.sessions.push(session.clone());
        ZenohCarrier::open(session).expect("the carrier is created")
    }

    /// Every carrier's session is connected to every other one, and sees a
    /// remote subscriber on the service's channels from its own side.
    fn sync(&self, hash: &ServiceHash, timeout: Duration) -> bool {
        let started = Instant::now();
        let publishers: Vec<_> = self
            .sessions
            .iter()
            .map(|session| {
                session
                    .declare_publisher(channels_of(hash))
                    .allowed_destination(Locality::Remote)
                    .wait()
                    .expect("the probe publisher is declared")
            })
            .collect();
        loop {
            let matched = publishers.iter().all(|publisher| {
                publisher
                    .matching_status()
                    .wait()
                    .expect("the matching status is read")
                    .matching()
            });
            if self.is_meshed() && matched {
                return true;
            }
            if started.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(POLL_PERIOD);
        }
    }
}

impl TunnelFixture for ZenohFixture {}

instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::carrier_discovery,
    ZenohFixture
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::carrier_propagation,
    ZenohFixture
);
instantiate_conformance_tests!(iceoryx2_link_conformance_tests::carrier_wake, ZenohFixture);

mod ipc {
    use super::*;

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<ZenohFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<ZenohFixture>
        );
    }

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Ipc,
            Event<AnyName>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_event,
            Ipc,
            Event<AnyName>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Ipc,
            Event<AnyName>,
            TunnelLinkFixture<ZenohFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Ipc,
            Event<AnyName>,
            TunnelLinkFixture<ZenohFixture>
        );
    }
}

mod local {
    use super::*;

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<ZenohFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<ZenohFixture>
        );
    }

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Local,
            Event<AnyName>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_event,
            Local,
            Event<AnyName>,
            ZenohFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Local,
            Event<AnyName>,
            TunnelLinkFixture<ZenohFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Local,
            Event<AnyName>,
            TunnelLinkFixture<ZenohFixture>
        );
    }
}
