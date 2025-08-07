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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! println!("name:                             {:?}", pubsub.name());
//! println!("service id:                       {:?}", pubsub.service_id());
//! println!("type details:                     {:?}", pubsub.static_config().message_type_details());
//! println!("max publishers:                   {:?}", pubsub.static_config().max_publishers());
//! println!("max subscribers:                  {:?}", pubsub.static_config().max_subscribers());
//! println!("subscriber buffer size:           {:?}", pubsub.static_config().subscriber_max_buffer_size());
//! println!("history size:                     {:?}", pubsub.static_config().history_size());
//! println!("subscriber max borrowed samples:  {:?}", pubsub.static_config().subscriber_max_borrowed_samples());
//! println!("safe overflow:                    {:?}", pubsub.static_config().has_safe_overflow());
//! println!("number of active publishers:      {:?}", pubsub.dynamic_config().number_of_publishers());
//! println!("number of active subscribers:     {:?}", pubsub.dynamic_config().number_of_subscribers());
//!
//! let publisher = pubsub.publisher_builder().create()?;
//! let subscriber = pubsub.subscriber_builder().create()?;
//!
//! # Ok(())
//! # }
//! ```
extern crate alloc;

use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::node::NodeListFailure;
use crate::service::attribute::AttributeSet;
use crate::service::service_id::ServiceId;
use crate::service::service_name::ServiceName;
use crate::service::{self, dynamic_config, static_config, NoResource, ServiceState};
use alloc::sync::Arc;

use super::nodes;
use super::{publisher::PortFactoryPublisher, subscriber::PortFactorySubscriber};

/// The factory for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::publisher::Publisher`]
/// or [`crate::port::subscriber::Subscriber`] ports.
#[derive(Debug)]
pub struct PortFactory<
    Service: service::Service,
    Payload: Debug + ZeroCopySend + ?Sized,
    UserHeader: Debug + ZeroCopySend,
> {
    pub(crate) service: Arc<ServiceState<Service, NoResource>>,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

unsafe impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Send for PortFactory<Service, Payload, UserHeader>
{
}
unsafe impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Sync for PortFactory<Service, Payload, UserHeader>
{
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > crate::service::port_factory::PortFactory for PortFactory<Service, Payload, UserHeader>
{
    type Service = Service;
    type StaticConfig = static_config::publish_subscribe::StaticConfig;
    type DynamicConfig = dynamic_config::publish_subscribe::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.static_config.attributes()
    }

    fn static_config(&self) -> &static_config::publish_subscribe::StaticConfig {
        self.service.static_config.publish_subscribe()
    }

    fn dynamic_config(&self) -> &dynamic_config::publish_subscribe::DynamicConfig {
        self.service.dynamic_storage.get().publish_subscribe()
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
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > PortFactory<Service, Payload, UserHeader>
{
    pub(crate) fn new(service: ServiceState<Service, NoResource>) -> Self {
        Self {
            service: Arc::new(service),
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }

    /// Returns a [`PortFactorySubscriber`] to create a new
    /// [`crate::port::subscriber::Subscriber`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .publish_subscribe::<u64>()
    ///     .open_or_create()?;
    ///
    /// let subscriber = pubsub.subscriber_builder().create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscriber_builder(&self) -> PortFactorySubscriber<'_, Service, Payload, UserHeader> {
        PortFactorySubscriber::new(self)
    }

    /// Returns a [`PortFactoryPublisher`] to create a new
    /// [`crate::port::publisher::Publisher`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .publish_subscribe::<u64>()
    ///     .open_or_create()?;
    ///
    /// let publisher = pubsub.publisher_builder()
    ///                     .max_loaned_samples(6)
    ///                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
    ///                     .create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn publisher_builder(&self) -> PortFactoryPublisher<'_, Service, Payload, UserHeader> {
        PortFactoryPublisher::new(self)
    }
}
