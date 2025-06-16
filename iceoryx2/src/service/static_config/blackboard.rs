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

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

use crate::config;

/// The static configuration of an [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    pub(crate) max_readers: usize,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_readers: config.defaults.blackboard.max_readers,
        }
    }

    pub fn max_readers(&self) -> usize {
        self.max_readers
    }
}
