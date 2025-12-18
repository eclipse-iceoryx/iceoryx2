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

use iceoryx2_bb_concurrency::cell::RefCell;

use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_log::{error, fail, warn};
use zenoh::{
    handlers::FifoChannelHandler,
    query::{Querier, Reply},
    sample::Locality,
    Session, Wait,
};

use crate::keys;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    QuerierCreation,
    DiscoveryQuery,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    DiscoveryProcessing,
    DiscoveryQuery,
    QueryReplyReceive,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnnouncementError {
    Serialization,
    NotifyingKnownHosts,
    QueryableDeclaration,
}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

#[derive(Debug)]
pub struct Discovery {
    session: Session,
    querier: Querier<'static>,
    replies: RefCell<FifoChannelHandler<Reply>>,
}

impl Discovery {
    pub fn create(session: &Session) -> Result<Self, CreationError> {
        let origin = "Discovery::create()";

        let querier = fail!(
            from origin,
            when session
                    .declare_querier(keys::service_discovery())
                    .allowed_destination(Locality::Remote)
                    .wait(),
            with CreationError::QuerierCreation,
            "Failed to create querier for service discovery"
        );

        // Make query immediately - replies processed in first `discover()` call
        let replies = fail!(
            from origin,
            when querier.get().wait(),
            with CreationError::DiscoveryQuery,
            "Failed to make query for service discovery"
        );

        Ok(Self {
            session: session.clone(),
            querier,
            replies: RefCell::new(replies),
        })
    }
}

impl iceoryx2_tunnel_backend::traits::Discovery for Discovery {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, static_config: &StaticConfig) -> Result<(), Self::AnnouncementError> {
        let key = keys::service_details(static_config.service_id());
        let service_config_serialized = fail!(
            from self,
            when serde_json::to_string(&static_config),
            with AnnouncementError::Serialization,
            "Failed to serialize service config"
        );

        // Notify all current hosts.
        fail!(
            from self,
            when self.session
                .put(key.clone(), service_config_serialized.clone())
                .allowed_destination(Locality::Remote)
                .wait(),
            with AnnouncementError::NotifyingKnownHosts,
            "Failed to notify known hosts of discovery"
        );

        // Set up a queryable to respond to future hosts.
        fail!(
            from self,
            when self.session
                .declare_queryable(key.clone())
                .callback(move |query| {
                    let _ = query
                        .reply(key.clone(), service_config_serialized.clone())
                        .wait()
                        .inspect_err(|e| {
                            error!("Failed to announce service {}: {}", key, e);
                        });
                })
                .allowed_origin(Locality::Remote)
                .background()
                .wait(),
            with AnnouncementError::QueryableDeclaration,
            "Failed to declare queryable for future hosts to discover service"
        );

        Ok(())
    }

    fn discover<E: core::error::Error, F: FnMut(&StaticConfig) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), DiscoveryError> {
        // Drain all replies from previous query
        for reply in self.replies.borrow_mut().drain() {
            match reply.result() {
                Ok(sample) => {
                    match serde_json::from_slice::<StaticConfig>(&sample.payload().to_bytes()) {
                        Ok(static_config) => {
                            fail!(
                                from &self,
                                when process_discovery(&static_config),
                                with DiscoveryError::DiscoveryProcessing,
                                "Failed to process discovery event"
                            )
                        }
                        Err(e) => {
                            warn!(
                                "Skipping discovered service config, unable to deserialize: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => fail!(
                    from self,
                    when Err(e),
                    with DiscoveryError::QueryReplyReceive,
                    "Erroneous reply received from zenoh discovery query"
                ),
            }
        }

        // Make a new query for next `discover()` call
        // NOTE: This results in all service details being resent - not optimal
        // TODO(optimization): A solution to request all quereyables once whilst still retrieving
        //                     querying new quereyables that appear
        let next_query = fail!(
            from &self,
            when self.querier.get().wait(),
            with DiscoveryError::DiscoveryQuery,
            "Failed to query Zenoh for services"
        );
        *self.replies.borrow_mut() = next_query;

        Ok(())
    }
}
