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

use iceoryx2::service::Service;
use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker};

use crate::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

impl<S: Service> Discovery<S> for Tracker<S> {
    type Error = SyncError;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::Error>,
    >(
        &mut self,
        process_discovery: &mut F,
    ) -> Result<(), Self::Error> {
        let (added, _removed) = self.sync().unwrap();

        for id in added {
            process_discovery(&self.get(&id).unwrap().static_details)?;
        }

        Ok(())
    }
}
