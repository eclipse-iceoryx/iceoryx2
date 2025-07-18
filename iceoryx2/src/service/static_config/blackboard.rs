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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! type KeyType = u64;
//! let blackboard = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<KeyType>()
//!     .add::<i32>(0,0)
//!     .create()?;
//!
//! println!("type details: {:?}", blackboard.static_config().type_details());
//! println!("max readers: {:?}", blackboard.static_config().max_readers());
//!
//! # Ok(())
//! # }
//! ```

use crate::config;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

use super::message_type_details::TypeDetail;

/// The static configuration of an [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    pub(crate) max_readers: usize,
    pub(crate) max_writers: usize,
    pub(crate) max_nodes: usize,
    pub(crate) type_details: TypeDetail,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_readers: config.defaults.blackboard.max_readers,
            max_writers: 1,
            max_nodes: config.defaults.blackboard.max_nodes,
            type_details: TypeDetail::default(),
        }
    }

    /// Returns the maximum supported amount of [`Node`](crate::node::Node)s that can open the
    /// [`Service`](crate::service::Service) in parallel.
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum supported amount of [`crate::port::reader::Reader`] ports
    pub fn max_readers(&self) -> usize {
        self.max_readers
    }

    /// Returns the type details of the [`crate::service::Service`].
    pub fn type_details(&self) -> &TypeDetail {
        &self.type_details
    }
}
