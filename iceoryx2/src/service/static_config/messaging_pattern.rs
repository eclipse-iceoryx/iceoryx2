// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Stores the [`Service`](crate::service::Service) messaging pattern specific static configuration.
use core::fmt::Display;

use crate::service::static_config::blackboard;
use crate::service::static_config::event;
use crate::service::static_config::publish_subscribe;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use serde::{Deserialize, Serialize};

use super::request_response;

/// Contains the static config of the corresponding
/// [`service::MessagingPattern`](crate::service::messaging_pattern::MessagingPattern).
///
/// # Safety
///
/// This is a large struct (>1KB). Be cautious with where it is placed and how it is passed around.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
#[allow(clippy::large_enum_variant)]
pub enum MessagingPattern {
    /// Stores the static config of the
    /// [`service::MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
    RequestResponse(request_response::StaticConfig),

    /// Stores the static config of the
    /// [`service::MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
    PublishSubscribe(publish_subscribe::StaticConfig),

    /// Stores the static config of the
    /// [`service::MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
    Event(event::StaticConfig),

    /// Stores the static config of the
    /// [`service::MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
    Blackboard(blackboard::StaticConfig),
}

impl Display for MessagingPattern {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MessagingPattern::RequestResponse(_) => write!(f, "RequestResponse"),
            MessagingPattern::Event(_) => write!(f, "Event"),
            MessagingPattern::PublishSubscribe(_) => write!(f, "PublishSubscribe"),
            MessagingPattern::Blackboard(_) => write!(f, "Blackboard"),
        }
    }
}

impl MessagingPattern {
    /// checks whether the 2 MessagingPatterns are the same regardless the values inside them.
    pub(crate) fn is_same_pattern(&self, rhs: &MessagingPattern) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(rhs)
    }

    /// # Safety
    ///
    ///  * User must ensure that publish subscribe is stored inside
    pub unsafe fn publish_subscribe(&self) -> &publish_subscribe::StaticConfig {
        if let MessagingPattern::PublishSubscribe(v) = self {
            v
        } else {
            fatal_panic!(from self,
                "This should never happen! Trying to access publish subscribe messaging pattern that is not stored.");
        }
    }

    /// # Safety
    ///
    ///  * User must ensure that event is stored inside
    pub unsafe fn request_response(&self) -> &request_response::StaticConfig {
        if let MessagingPattern::RequestResponse(v) = self {
            v
        } else {
            fatal_panic!(from self,
                "This should never happen! Trying to access request response messaging pattern that is not stored.");
        }
    }
}

#[cfg(test)]
mod tests {
    use iceoryx2_bb_testing::assert_that;

    use super::*;
    use crate::service::config;
    use crate::service::static_config::event;
    use crate::service::static_config::publish_subscribe;

    #[test]
    fn test_is_same_pattern() {
        let cfg = config::Config::default();
        let p1 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg));
        let p2 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg));
        assert_that!(p1.is_same_pattern(&p2), eq true);
        assert_that!(p2.is_same_pattern(&p1), eq true);

        let e1 = MessagingPattern::Event(event::StaticConfig::new(&cfg));
        let e2 = MessagingPattern::Event(event::StaticConfig::new(&cfg));
        assert_that!(e1.is_same_pattern(&e2), eq true);
        assert_that!(e2.is_same_pattern(&e1), eq true);

        let r1 = MessagingPattern::RequestResponse(request_response::StaticConfig::new(&cfg));
        let r2 = MessagingPattern::RequestResponse(request_response::StaticConfig::new(&cfg));
        assert_that!(r1.is_same_pattern(&r2), eq true);
        assert_that!(r2.is_same_pattern(&r1), eq true);

        let b1 = MessagingPattern::Blackboard(blackboard::StaticConfig::new(&cfg));
        let b2 = MessagingPattern::Blackboard(blackboard::StaticConfig::new(&cfg));
        assert_that!(b1.is_same_pattern(&b2), eq true);
        assert_that!(b2.is_same_pattern(&b1), eq true);

        let mut new_defaults = config::Defaults {
            request_response: cfg.defaults.request_response.clone(),
            publish_subscribe: cfg.defaults.publish_subscribe.clone(),
            event: cfg.defaults.event.clone(),
            blackboard: cfg.defaults.blackboard.clone(),
        };
        new_defaults.event.event_id_max_value -= 1;
        new_defaults.publish_subscribe.max_nodes -= 1;
        new_defaults.request_response.max_nodes -= 1;
        new_defaults.blackboard.max_nodes -= 1;

        let cfg2 = config::Config {
            defaults: new_defaults,
            global: cfg.global.clone(),
        };

        // ensure the cfg and cfg2 are not equal
        assert_that!(cfg, ne cfg2);

        let p3 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg2));
        assert_that!(p1.is_same_pattern(&p3), eq true);
        assert_that!(p3.is_same_pattern(&p1), eq true);

        let e3 = MessagingPattern::Event(event::StaticConfig::new(&cfg2));
        assert_that!(e1.is_same_pattern(&e3), eq true);
        assert_that!(e2.is_same_pattern(&e3), eq true);

        let r3 = MessagingPattern::RequestResponse(request_response::StaticConfig::new(&cfg));
        assert_that!(r1.is_same_pattern(&r3), eq true);
        assert_that!(r2.is_same_pattern(&r3), eq true);

        let b3 = MessagingPattern::Blackboard(blackboard::StaticConfig::new(&cfg));
        assert_that!(b1.is_same_pattern(&b3), eq true);
        assert_that!(b2.is_same_pattern(&b3), eq true);

        assert_that!(p1.is_same_pattern(&e1), eq false);
        assert_that!(p3.is_same_pattern(&e3), eq false);
        assert_that!(p1.is_same_pattern(&r1), eq false);
        assert_that!(p3.is_same_pattern(&r3), eq false);
        assert_that!(p1.is_same_pattern(&b1), eq false);
        assert_that!(p3.is_same_pattern(&b3), eq false);
        assert_that!(e1.is_same_pattern(&r1), eq false);
        assert_that!(e3.is_same_pattern(&r3), eq false);
        assert_that!(e1.is_same_pattern(&b1), eq false);
        assert_that!(e3.is_same_pattern(&b3), eq false);
        assert_that!(r1.is_same_pattern(&b1), eq false);
        assert_that!(r3.is_same_pattern(&b3), eq false);
    }
}
