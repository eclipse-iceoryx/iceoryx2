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

use core::convert::Infallible;
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::Identified;
use iceoryx2_link_backend::{Announcement, Backend, OnRemote, Reactive, WakeHandle};

use crate::relay::{self, Factory};
use crate::resolver::Resolver;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::Translator;
use iceoryx2_link_adapter::{Adapter, EndpointDescription, EndpointTypes};
use iceoryx2_link_backend::relay::UnsupportedRelay;

/// A backend connecting `iceoryx2` to a middleware through an adapter,
/// a mapping and a translator.
pub struct Gateway<A, M: Mapping, T: Translator> {
    adapter: A,
    resolver: Resolver<M, T>,
}

impl<A, M: Mapping, T: Translator> Gateway<A, M, T> {
    /// Creates a gateway over `adapter`, mapping with `mapping` and
    /// translating with `translator`.
    pub fn new(adapter: A, mapping: M, translator: T) -> Self {
        Self {
            adapter,
            resolver: Resolver {
                mapping,
                translator,
            },
        }
    }
}

impl<S, A, M, T> Backend<S> for Gateway<A, M, T>
where
    S: Service,
    M: Mapping,
    T: Translator,
    A: Adapter<
            EndpointSettings = M::EndpointSettings,
            EndpointTypes = EndpointTypes<T::RemoteTypes>,
        >,
{
    type ListError = A::ListError;
    type AnnouncementError = Infallible;
    type RemoteId = <M::EndpointSettings as Identified>::Id;
    type RemoteDescription =
        EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>;
    type Refusal = crate::resolver::Refusal<M::Error>;
    type Resolver<'a>
        = &'a Resolver<M, T>
    where
        Self: 'a;
    type PublishSubscribeRelay =
        relay::publish_subscribe::Relay<S, A::PublishSubscribeEndpoints, T>;
    type EventRelay = UnsupportedRelay<S>;
    type RelayFactory<'a>
        = Factory<'a, S, A, M, T>
    where
        Self: 'a;

    fn generation(&self) -> Generation {
        self.adapter.generation()
    }

    fn list(&self, on_remote: &mut OnRemote<'_, S, Self>) -> Result<(), A::ListError> {
        self.adapter
            .endpoints(&mut |endpoint| on_remote(&endpoint.settings.id(), endpoint))
    }

    fn resolver(&self) -> &Resolver<M, T> {
        &self.resolver
    }

    /// Creating a relay already made the service known on the middleware,
    /// there is nothing further to announce.
    fn announce(&mut self, _: Announcement<'_>) -> Result<(), Infallible> {
        Ok(())
    }

    fn relay_factory(&mut self) -> Self::RelayFactory<'_> {
        Factory {
            adapter: &mut self.adapter,
            translator: &self.resolver.translator,
            _service: PhantomData,
        }
    }
}

impl<A: Adapter + Reactive, M: Mapping, T: Translator> Reactive for Gateway<A, M, T> {
    fn attach(&mut self, wake: WakeHandle) {
        self.adapter.attach(wake);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use iceoryx2::service::local;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;

    use crate::testing::{IdentityMapping, StubAdapter, StubEndpointDescription, endpoint};
    use iceoryx2_link_adapter::Passthrough;

    type Sut = Gateway<StubAdapter, IdentityMapping, Passthrough>;

    const HISTORY_SIZE: usize = 1;
    const GENERATION: u64 = 3;

    fn list(sut: &Sut) -> Vec<(ServiceName, StubEndpointDescription)> {
        let mut listed = Vec::new();
        <Sut as Backend<local::Service>>::list(sut, &mut |name, endpoint| {
            listed.push((*name, endpoint.clone()));
        })
        .expect("adapter never fails");
        listed
    }

    #[test]
    fn the_middlewares_endpoints_are_listed_by_name_and_description() {
        const ENDPOINT: &str = "gateway/list";

        let adapter = StubAdapter {
            endpoints: alloc::vec![endpoint(ENDPOINT, HISTORY_SIZE)],
            ..StubAdapter::default()
        };
        let sut = Gateway::new(adapter, IdentityMapping, Passthrough);
        let listed = list(&sut);

        let name = ServiceName::new(ENDPOINT).expect("valid service name");
        assert_that!(listed, eq alloc::vec![(name, endpoint(ENDPOINT, HISTORY_SIZE))]);
    }

    #[test]
    fn the_adapters_generation_is_the_gateways() {
        let adapter = StubAdapter {
            generation: GENERATION,
            ..StubAdapter::default()
        };
        let sut = Gateway::new(adapter, IdentityMapping, Passthrough);
        let generation = <Sut as Backend<local::Service>>::generation(&sut);

        assert_that!(generation, eq Generation::At(GENERATION));
    }
}
