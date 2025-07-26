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
//! let reader = blackboard.reader_builder().create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::blackboard::PortFactory;
use crate::port::reader::{Reader, ReaderCreateError};
use crate::service;
use core::fmt::Debug;
use core::hash::Hash;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;

/// Factory to create a new [`Reader`] port/endpoint for
/// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryReader<
    'factory,
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    pub(crate) factory: &'factory PortFactory<Service, KeyType>,
}

impl<
        'factory,
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > PortFactoryReader<'factory, Service, KeyType>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, KeyType>) -> Self {
        Self { factory }
    }

    /// Creates a new [`Reader`] or returns a [`ReaderCreateError`] on failure.
    pub fn create(self) -> Result<Reader<Service, KeyType>, ReaderCreateError> {
        let origin = format!("{self:?}");
        Ok(
            fail!(from origin, when Reader::new(self.factory.service.clone()),"Failed to create new Reader port."),
        )
    }
}
