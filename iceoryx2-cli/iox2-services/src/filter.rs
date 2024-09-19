// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use crate::cli::DetailsFilter;
use crate::cli::MessagingPatternFilter;
use crate::output::ServiceDescription;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_cli_utils::Filter;

impl Filter<ServiceDescription> for MessagingPatternFilter {
    fn matches(&self, description: &ServiceDescription) -> bool {
        matches!(
            (self, &description.pattern),
            (
                MessagingPatternFilter::PublishSubscribe,
                MessagingPattern::PublishSubscribe(_)
            ) | (MessagingPatternFilter::Event, MessagingPattern::Event(_))
                | (MessagingPatternFilter::All, _)
        )
    }
}

impl Filter<ServiceDescription> for DetailsFilter {
    fn matches(&self, description: &ServiceDescription) -> bool {
        self.pattern.matches(description)
    }
}
