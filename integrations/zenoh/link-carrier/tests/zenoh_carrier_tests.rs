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
use iceoryx2_integrations_zenoh_link_carrier::ZenohCarrier;
use iceoryx2_link_conformance_tests::fixture::{CarrierFixture, TunnelFixture, TunnelLinkFixture};
use iceoryx2_link_conformance_tests::parameters::{
    AnyName, Event, FixedSizePayload, PublishSubscribe,
};
use zenoh::key_expr::OwnedKeyExpr;
use zenoh::sample::Locality;
use zenoh::session::ZenohId;
use zenoh::{Session, Wait};

/// The scouting delay of every session, shortened from zenoh's default
/// so peers find each other within a scenario.
const SCOUTING_DELAY_MS: u64 = 50;
/// The pause between two looks at the sessions' peers and matching status.
const POLL_PERIOD: Duration = Duration::from_millis(10);

/// The key a session's probe subscriber listens on while syncing.
fn probe_key(hash: &ServiceHash, session: &Session) -> OwnedKeyExpr {
    OwnedKeyExpr::try_from(format!(
        "iox2/test/probe/{}/{}",
        hash.as_str(),
        session.zid()
    ))
    .expect("the probe key is well formed")
}

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

    /// Waits until every session knows every other one's subscription to
    /// the channel of the service `hash`.
    ///
    /// Zenoh delivers a frame only to the subscribers a session already
    /// knows of, and a new subscription takes a moment to reach the other
    /// sessions. A scenario should open its channels first and then call
    /// this, which ensures the channels are discovered by all sessions
    /// in the scenario.
    ///
    /// The check uses probes. Every session declares a subscriber on a key
    /// of its own, later than the channel subscriber it already has, and
    /// every other session declares a publisher on that key. Zenoh passes
    /// subscriptions on in the order they were made, so once every probe
    /// publisher matches, the channel subscribers can be assumed to have
    /// matched as well.
    fn sync(&self, hash: &ServiceHash, timeout: Duration) -> bool {
        let started = Instant::now();

        // One probe subscriber per session for `hash`.
        let _subscribers: Vec<_> = self
            .sessions
            .iter()
            .map(|session| {
                session
                    .declare_subscriber(probe_key(hash, session))
                    .allowed_origin(Locality::Remote)
                    .wait()
                    .expect("the probe subscriber is declared")
            })
            .collect();

        // And one probe publisher per session for `hash`.
        let publishers: Vec<_> = self
            .sessions
            .iter()
            .flat_map(|from| {
                self.sessions
                    .iter()
                    .filter(move |to| to.zid() != from.zid())
                    .map(move |to| {
                        from.declare_publisher(probe_key(hash, to))
                            .allowed_destination(Locality::Remote)
                            .wait()
                            .expect("the probe publisher is declared")
                    })
            })
            .collect();

        // Wait until every probe publisher is matched.
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
