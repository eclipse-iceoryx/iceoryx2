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
use crate::keys;

use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;

use zenoh::handlers::FifoChannelHandler;
use zenoh::query::Querier as ZenohQuerier;
use zenoh::query::Reply;
use zenoh::sample::Locality;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use super::DiscoveryError;

pub enum CreationError {
    FailureToCreateZenohQueriable,
    FailureToQueryZenoh,
}

/// Discovers remote `iceoryx2` services via Zenoh.
///
/// TODO: Explain in detail
pub(crate) struct ZenohDiscovery<'a, ServiceType: iceoryx2::service::Service> {
    z_querier: ZenohQuerier<'a>,
    z_query: FifoChannelHandler<Reply>,
    _phantom: core::marker::PhantomData<ServiceType>,
}

impl<ServiceType: iceoryx2::service::Service> ZenohDiscovery<'_, ServiceType> {
    pub fn create(z_session: &ZenohSession) -> Result<Self, CreationError> {
        let z_querier = z_session
            .declare_querier(keys::discovery())
            .allowed_destination(Locality::Remote)
            .wait()
            .map_err(|_e| CreationError::FailureToCreateZenohQueriable)?;

        // Make query immediately - replies processed in first `discover()` call
        let z_query = z_querier
            .get()
            .wait()
            .map_err(|_e| CreationError::FailureToQueryZenoh)?;

        return Ok(Self {
            z_querier,
            z_query,
            _phantom: core::marker::PhantomData,
        });
    }
}

impl<ServiceType: iceoryx2::service::Service> Discovery<ServiceType>
    for ZenohDiscovery<'_, ServiceType>
{
    fn discover<OnDiscovered: FnMut(&IceoryxServiceConfig)>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), DiscoveryError> {
        // Drain all replies from previous query
        for z_reply in self.z_query.drain() {
            match z_reply.result() {
                Ok(z_sample) => {
                    match serde_json::from_slice::<IceoryxServiceConfig>(
                        &z_sample.payload().to_bytes(),
                    ) {
                        Ok(iox_service_details) => {
                            on_discovered(&iox_service_details);
                        }
                        Err(_e) => todo!(),
                    }
                }
                Err(_e) => { /* Ignore and process other requests */ }
            }
        }

        // Make a new query for next `discover()` call
        self.z_query = self
            .z_querier
            .get()
            .wait()
            .map_err(|_e| DiscoveryError::FailureToMakeQuery)?;

        Ok(())
    }
}
