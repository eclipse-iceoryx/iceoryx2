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

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_backend::description::ServiceDescription;
use iceoryx2_link_backend::diagnostic::{Findings, Update};
use iceoryx2_link_backend::origin;
use iceoryx2_log::error;

use crate::bridge::OpenError;

/// A failure to open a bridge and the description it failed with.
#[derive(PartialEq)]
pub(crate) struct Failure {
    pub(crate) description: ServiceDescription,
    pub(crate) error: OpenError,
}

/// What the link has warned about and not yet seen cleared, so each
/// finding is warned about once.
#[derive(Default)]
pub(crate) struct Diagnostics {
    failed: Findings<ServiceHash, Failure>,
}

impl Diagnostics {
    /// Begins an update of the bridges that failed to open, warning about
    /// each recorded that is new or failed with another description.
    pub(crate) fn failed_bridges(
        &mut self,
    ) -> Update<
        ServiceHash,
        Failure,
        &mut Findings<ServiceHash, Failure>,
        impl FnMut(&ServiceHash, &Failure),
    > {
        let origin = origin!("Diagnostics::failed_bridges");

        let warn_failure = move |_: &ServiceHash, failure: &Failure| {
            error!(
                from origin,
                "Failed to bridge {}: {}", failure.description.name(), failure.error
            );
        };
        self.failed.update(warn_failure)
    }
}
