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
//! println!("name:                             {:?}", blackboard.name());
//! println!("service id:                       {:?}", blackboard.service_id());
//! println!("type details:                     {:?}", blackboard.static_config().type_details());
//! println!("max nodes:                        {:?}", blackboard.static_config().max_nodes());
//! println!("max readers:                      {:?}", blackboard.static_config().max_readers());
//! println!("number of active readers:         {:?}", blackboard.dynamic_config().number_of_readers());
//!
//! let writer = blackboard.writer_builder().create()?;
//! let reader = blackboard.reader_builder().create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::nodes;
use super::reader::PortFactoryReader;
use super::writer::PortFactoryWriter;
use crate::node::NodeListFailure;
use crate::service::attribute::AttributeSet;
use crate::service::builder::blackboard::BlackboardResources;
use crate::service::service_id::ServiceId;
use crate::service::service_name::ServiceName;
use crate::service::{self, dynamic_config, static_config, ServiceState};
use core::fmt::Debug;
use core::hash::Hash;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

extern crate alloc;
use alloc::sync::Arc;

/// The factory for
/// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::reader::Reader`] or [`crate::port::writer::Writer`] ports.
#[derive(Debug)]
pub struct PortFactory<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    pub(crate) service: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > crate::service::port_factory::PortFactory for PortFactory<Service, KeyType>
{
    type Service = Service;
    type StaticConfig = static_config::blackboard::StaticConfig;
    type DynamicConfig = dynamic_config::blackboard::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.static_config.attributes()
    }

    fn static_config(&self) -> &static_config::blackboard::StaticConfig {
        self.service.static_config.blackboard()
    }

    fn dynamic_config(&self) -> &dynamic_config::blackboard::DynamicConfig {
        self.service.dynamic_storage.get().blackboard()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.dynamic_storage.get(),
            self.service.shared_node.config(),
            callback,
        )
    }
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > PortFactory<Service, KeyType>
{
    pub(crate) fn new(
        service: ServiceState<Service, BlackboardResources<Service, KeyType>>,
    ) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Returns a [`PortFactoryWriter`] to create a new [`crate::port::writer::Writer`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// type KeyType = u64;
    /// let blackboard = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .blackboard_creator::<KeyType>()
    ///     .add::<i32>(0,0)
    ///     .create()?;
    ///
    /// let writer = blackboard.writer_builder().create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn writer_builder(&self) -> PortFactoryWriter<'_, Service, KeyType> {
        PortFactoryWriter::new(self)
    }

    /// Returns a [`PortFactoryReader`] to create a new [`crate::port::reader::Reader`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// type KeyType = u64;
    /// let blackboard = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .blackboard_creator::<KeyType>()
    ///     .add::<i32>(0,0)
    ///     .create()?;
    ///
    /// let reader = blackboard.reader_builder().create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn reader_builder(&self) -> PortFactoryReader<'_, Service, KeyType> {
        PortFactoryReader::new(self)
    }
}
