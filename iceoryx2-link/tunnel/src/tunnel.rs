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

use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_backend::{Announcement, Backend, OnRemote, Reactive, WakeHandle};
use iceoryx2_log::{origin, trace};

use crate::OfferId;
use crate::relay::{self, Factory};
use crate::resolver::Resolver;
use iceoryx2_link_carrier::{self as carrier, Carrier};

/// A backend connecting `iceoryx2` systems over a carrier.
pub struct Tunnel<C: Carrier> {
    carrier: C,
    resolver: Resolver,
}

impl<C: Carrier> Tunnel<C> {
    /// Creates a tunnel over `carrier` for the local system configured by
    /// `config`.
    pub fn new(carrier: C, config: &Config) -> Self {
        Self {
            carrier,
            resolver: Resolver::new(config),
        }
    }
}

impl<S: Service, C: Carrier> Backend<S> for Tunnel<C> {
    type ListError = C::ListError;
    type AnnouncementError = C::AnnouncementError;
    type RemoteId = OfferId;
    type RemoteDescription = ServiceDescriptor;
    type Refusal = crate::resolver::Refusal;
    type Resolver<'a>
        = &'a Resolver
    where
        Self: 'a;
    type PublishSubscribeRelay = relay::publish_subscribe::Relay<S, C::SampleChannel>;
    type EventRelay = relay::event::Relay<S, C::EventChannel>;
    type RelayFactory<'a>
        = Factory<'a, S, C>
    where
        Self: 'a;

    fn generation(&self) -> Generation {
        self.carrier.generation()
    }

    fn list(&self, on_remote: &mut OnRemote<'_, S, Self>) -> Result<(), C::ListError> {
        self.carrier
            .offers(&mut |offer| on_remote(&OfferId::from(&offer), &offer.descriptor))
    }

    fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    fn announce(&mut self, announcement: Announcement<'_>) -> Result<(), C::AnnouncementError> {
        let origin = origin!("Tunnel::announce");
        let announcement = match announcement {
            Announcement::Offered(description) => {
                trace!(from origin, "Offering {}", description.name());
                carrier::Announcement::Offered {
                    descriptor: ServiceDescriptor::from(description),
                }
            }
            Announcement::Withdrawn(hash) => {
                trace!(from origin, "Withdrawing the service {}", hash.as_str());
                carrier::Announcement::Withdrawn { hash }
            }
        };
        self.carrier.announce(announcement)
    }

    fn relay_factory(&mut self) -> Self::RelayFactory<'_> {
        Factory::new(&mut self.carrier)
    }
}

impl<C: Carrier + Reactive> Reactive for Tunnel<C> {
    fn attach(&mut self, wake: WakeHandle) {
        self.carrier.attach(wake);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use iceoryx2::service::local;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;

    use crate::testing::{description, descriptor, peer};
    use iceoryx2::port::event_id::EventId;
    use iceoryx2_link_backend::wire::sample::LoanableSample;
    use iceoryx2_link_carrier::{
        EventChannel, EventReceiveError, Offer, SampleChannel, SampleReceiveError,
    };

    const FIRST_PEER: u8 = 1;
    const SECOND_PEER: u8 = 2;
    const PAYLOAD: &str = "u64";
    const GENERATION: u64 = 3;

    /// A carrier whose peers' offers are set directly by the test and
    /// which logs what is announced to it.
    #[derive(Default)]
    struct FakeCarrier {
        offers: Vec<Offer>,
        announced: Vec<carrier::Announcement>,
        generation: u64,
    }

    struct NoChannel;

    impl SampleChannel for NoChannel {
        type Error = core::fmt::Error;
        fn send(&mut self, _: &[&[u8]]) -> Result<(), Self::Error> {
            Ok(())
        }
        fn receive<L: LoanableSample>(
            &mut self,
            _: L,
        ) -> Result<Option<L::Sample>, SampleReceiveError<Self::Error>> {
            Ok(None)
        }
    }

    impl EventChannel for NoChannel {
        type Error = core::fmt::Error;
        fn send(&mut self, _: EventId) -> Result<(), Self::Error> {
            Ok(())
        }
        fn receive(&mut self) -> Result<Option<EventId>, EventReceiveError<Self::Error>> {
            Ok(None)
        }
    }

    impl Carrier for FakeCarrier {
        type AnnouncementError = core::fmt::Error;
        type ListError = core::fmt::Error;
        type ChannelError = core::fmt::Error;
        type SampleChannel = NoChannel;
        type EventChannel = NoChannel;

        fn announce(
            &mut self,
            announcement: carrier::Announcement,
        ) -> Result<(), Self::AnnouncementError> {
            self.announced.push(announcement);
            Ok(())
        }

        fn generation(&self) -> Generation {
            Generation::At(self.generation)
        }

        fn offers(&self, callback: &mut dyn FnMut(Offer)) -> Result<(), Self::ListError> {
            for offer in &self.offers {
                callback(offer.clone());
            }
            Ok(())
        }

        fn open_sample_channel(
            &mut self,
            _: &ServiceDescriptor,
        ) -> Result<Self::SampleChannel, Self::ChannelError> {
            Ok(NoChannel)
        }

        fn open_event_channel(
            &mut self,
            _: &ServiceDescriptor,
        ) -> Result<Self::EventChannel, Self::ChannelError> {
            Ok(NoChannel)
        }
    }

    fn offer(peer_discriminator: u8, name: &str) -> Offer {
        Offer {
            peer: peer(peer_discriminator),
            descriptor: descriptor(name, PAYLOAD),
        }
    }

    fn list(sut: &Tunnel<FakeCarrier>) -> Vec<(OfferId, ServiceDescriptor)> {
        let mut listed = Vec::new();
        <Tunnel<FakeCarrier> as Backend<local::Service>>::list(sut, &mut |id, descriptor| {
            listed.push((*id, descriptor.clone()));
        })
        .expect("carrier never fails");
        listed
    }

    fn announce(sut: &mut Tunnel<FakeCarrier>, announcement: Announcement<'_>) {
        <Tunnel<FakeCarrier> as Backend<local::Service>>::announce(sut, announcement)
            .expect("carrier never fails");
    }

    #[test]
    fn the_peers_offers_are_listed_by_offer_id_and_descriptor() {
        const SERVICE: &str = "tunnel/list";

        let carrier = FakeCarrier {
            offers: alloc::vec![offer(FIRST_PEER, SERVICE), offer(SECOND_PEER, SERVICE)],
            ..FakeCarrier::default()
        };
        let sut = Tunnel::new(carrier, &Config::default());

        let listed = list(&sut);

        let expected = alloc::vec![
            (
                OfferId::from(&offer(FIRST_PEER, SERVICE)),
                descriptor(SERVICE, PAYLOAD)
            ),
            (
                OfferId::from(&offer(SECOND_PEER, SERVICE)),
                descriptor(SERVICE, PAYLOAD)
            ),
        ];
        assert_that!(listed, eq expected);
    }

    #[test]
    fn the_carriers_generation_is_the_tunnels() {
        let carrier = FakeCarrier {
            generation: GENERATION,
            ..FakeCarrier::default()
        };
        let sut = Tunnel::new(carrier, &Config::default());

        let generation = <Tunnel<FakeCarrier> as Backend<local::Service>>::generation(&sut);

        assert_that!(generation, eq Generation::At(GENERATION));
    }

    #[test]
    fn an_offered_service_is_announced_by_its_descriptor() {
        const SERVICE: &str = "tunnel/announce/offered";

        let mut sut = Tunnel::new(FakeCarrier::default(), &Config::default());
        let service = description(SERVICE, PAYLOAD);
        announce(&mut sut, Announcement::Offered(&service));

        let expected = carrier::Announcement::Offered {
            descriptor: descriptor(SERVICE, PAYLOAD),
        };
        assert_that!(sut.carrier.announced, eq alloc::vec![expected]);
    }

    #[test]
    fn a_withdrawn_service_is_announced_by_its_hash() {
        const SERVICE: &str = "tunnel/announce/withdrawn";

        let mut sut = Tunnel::new(FakeCarrier::default(), &Config::default());
        let hash: ServiceHash = description(SERVICE, PAYLOAD).hash();
        announce(&mut sut, Announcement::Withdrawn(hash));

        let expected = carrier::Announcement::Withdrawn { hash };
        assert_that!(sut.carrier.announced, eq alloc::vec![expected]);
    }
}
