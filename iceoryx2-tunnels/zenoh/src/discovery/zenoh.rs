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

use crate::discovery::Discovery;
use crate::discovery::DiscoveryError;
use crate::keys;

use iceoryx2::service::static_config::StaticConfig as ServiceConfig;

use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::warn;
use zenoh::handlers::FifoChannelHandler;
use zenoh::query::Querier as ZenohQuerier;
use zenoh::query::Reply;
use zenoh::sample::Locality;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

// TODO: More granularity in errors
pub enum CreationError {
    Error,
}

/// Discovers remote `iceoryx2` services via Zenoh.
#[derive(Debug)]
pub(crate) struct ZenohDiscovery<'a, ServiceType: iceoryx2::service::Service> {
    querier: ZenohQuerier<'a>,
    replies: FifoChannelHandler<Reply>,
    _phantom: core::marker::PhantomData<ServiceType>,
}

impl<ServiceType: iceoryx2::service::Service> ZenohDiscovery<'_, ServiceType> {
    pub fn create(z_session: &ZenohSession) -> Result<Self, CreationError> {
        let querier = fail!(
            from "ZenohDiscovery::create()",
            when z_session
                    .declare_querier(keys::service_discovery())
                    .allowed_destination(Locality::Remote)
                    .wait(),
            with CreationError::Error,
            "failed to create Zenoh querier for service details on remote hosts"
        );

        // Make query immediately - replies processed in first `discover()` call
        let replies = fail!(
            from "ZenohDiscovery::create()",
            when querier.get().wait(),
            with CreationError::Error,
            "failed to query Zenoh for service details on remote hosts"
        );

        Ok(Self {
            querier,
            replies,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Discovery<ServiceType>
    for ZenohDiscovery<'_, ServiceType>
{
    fn discover<OnDiscovered: FnMut(&ServiceConfig) -> Result<(), DiscoveryError>>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), DiscoveryError> {
        // Drain all replies from previous query
        for reply in self.replies.drain() {
            match reply.result() {
                Ok(sample) => {
                    match serde_json::from_slice::<ServiceConfig>(&sample.payload().to_bytes()) {
                        Ok(service_details) => {
                            fail!(
                                from &self,
                                when on_discovered(&service_details),
                                "failed to process remote service discovered via Zenoh"
                            )
                        }
                        Err(e) => {
                            warn!(
                                "skipping discovered service config, unable to deserialize: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => fail!(
                    from "discovery_via_subscriber()",
                    when Err(e),
                    with DiscoveryError::UpdateFromRemotePort,
                    "errorneous reply received from zenoh discovery query"
                ),
            }
        }

        // Make a new query for next `discover()` call
        // NOTE: This results in all service details being resent - not optimal
        // TODO(optimization): A solution to request all quereyables once whilst still retrieving
        //                     querying new quereyables that appear
        self.replies = fail!(
            from &self,
            when self.querier.get().wait(),
            with DiscoveryError::UpdateFromRemotePort,
            "failed to query Zenoh for service details on remote hosts"
        );

        Ok(())
    }
}
