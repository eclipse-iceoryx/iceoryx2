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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::rc::Rc;
use iceoryx2_services_common::{DiscoveryEvent, DiscoveryEventRef};

use crate::backend::session::Session;

#[derive(Debug)]
pub enum DiscoveryError {
    Processing,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug)]
pub enum AnnouncementError {
    AnnounceAdded(crate::backend::session::AnnounceError),
    AnnounceRemoved(crate::backend::session::AnnounceError),
}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

#[derive(Debug)]
pub struct Discovery {
    session: Rc<Session>,
}

impl Discovery {
    pub fn new(session: Rc<Session>) -> Self {
        Self { session }
    }
}

impl iceoryx2_services_tunnel_backend::traits::Discovery for Discovery {
    type DiscoveryError = DiscoveryError;

    type AnnouncementError = AnnouncementError;

    fn announce(&self, event: DiscoveryEventRef<'_>) -> Result<(), Self::AnnouncementError> {
        match event {
            DiscoveryEventRef::Added(static_config) => self
                .session
                .announce_added(static_config)
                .map_err(AnnouncementError::AnnounceAdded),
            DiscoveryEventRef::Removed(service_hash) => self
                .session
                .announce_removed(service_hash)
                .map_err(AnnouncementError::AnnounceRemoved),
        }
    }

    fn discover<E: core::error::Error, F: FnMut(DiscoveryEvent) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        self.session.discover();
        let (added, removed) = self.session.pending_discoveries();

        for static_config in added {
            process_discovery(DiscoveryEvent::Added(static_config))
                .map_err(|_| DiscoveryError::Processing)?;
        }
        for service_hash in removed {
            process_discovery(DiscoveryEvent::Removed(service_hash))
                .map_err(|_| DiscoveryError::Processing)?;
        }

        Ok(())
    }
}
