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

use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::diagnostic::{Findings, Update};
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_log::{error, origin};

use crate::bridge::{BridgeError, OpenError};

/// A failure to open a bridge and the description it failed with.
#[derive(PartialEq)]
pub(crate) struct OpenFailure {
    pub(crate) description: ServiceDescription,
    pub(crate) error: OpenError,
}

/// A failure to propagate a bridge, named for the report.
#[derive(PartialEq)]
pub(crate) struct PropagationFailure {
    pub(crate) name: ServiceName,
    pub(crate) pattern: MessagingPattern,
    pub(crate) error: BridgeError,
}

/// What the link has warned about and not yet seen cleared, so each
/// finding is warned about once.
#[derive(Default)]
pub(crate) struct Diagnostics {
    failed_opens: Findings<ServiceHash, OpenFailure>,
    failed_propagations: Findings<ServiceHash, PropagationFailure>,
}

impl Diagnostics {
    /// Begins an update of the bridges that failed to open, warning about
    /// each recorded that is new or failed with another description.
    pub(crate) fn failed_opens(
        &mut self,
    ) -> Update<
        ServiceHash,
        OpenFailure,
        &mut Findings<ServiceHash, OpenFailure>,
        impl FnMut(&ServiceHash, &OpenFailure),
    > {
        let origin = origin!("Diagnostics::failed_opens");

        let warn_failure = move |_: &ServiceHash, failure: &OpenFailure| {
            error!(
                from origin,
                "Failed to bridge {}: {}", failure.description.name(), failure.error
            );
        };
        self.failed_opens.update(warn_failure)
    }

    /// Begins an update of the bridges that failed to propagate, warning
    /// about each recorded that is new or fails for another reason.
    pub(crate) fn failed_propagations(
        &mut self,
    ) -> Update<
        ServiceHash,
        PropagationFailure,
        &mut Findings<ServiceHash, PropagationFailure>,
        impl FnMut(&ServiceHash, &PropagationFailure),
    > {
        let origin = origin!("Diagnostics::failed_propagations");

        let warn_failure = move |_: &ServiceHash, failure: &PropagationFailure| {
            error!(
                from origin,
                "Failed to propagate the {:?} bridge of \"{}\", {}",
                failure.pattern, failure.name, failure.error
            );
        };
        self.failed_propagations.update(warn_failure)
    }
}
