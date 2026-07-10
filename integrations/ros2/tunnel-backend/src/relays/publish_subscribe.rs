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

use std::rc::Rc;
use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe};
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{
    LoanFn, Sample, SampleMut, SampleMutUninit,
};
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, ServiceDescription, TypeDescription,
};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::rcl::{
    RclNode, RclPublisher, RclPublisherBuilder, RclSubscription, RclSubscriptionBuilder, TopicName,
    subscription::TakeError,
};
use crate::ros_header::RosHeader;
use crate::typesupport::TypeSupportRegistry;
use crate::{mapping, payload};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CreationError {
    InvalidServiceName,
    InvalidTopic,
    TypeSupport,
    Publisher,
    Subscription,
    WakeCallback,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Publish,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    Loan,
    Take,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// Relays publish-subscribe payloads between iceoryx2 and a ROS 2 topic.
#[derive(Debug)]
pub struct Relay<S: Service> {
    publisher: RclPublisher,
    subscription: RclSubscription,
    /// Whether the service's user-header type is [`RosHeader`], i.e. the
    /// relay may write the remote origin into received samples.
    write_ros_header: bool,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError> {
        let origin = "publish_subscribe::Relay::send";

        fail!(from origin,
            when self.publisher.publish(payload::as_bytes(sample.payload())),
            with SendError::Publish,
            "Failed to relay sample to ROS 2"
        );

        Ok(())
    }

    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        let mut loaned: Option<SampleMutUninit<S>> = None;
        let result = self.subscription.take_into(|size| match loan(size) {
            Ok(mut sample) => {
                let buffer = payload::uninit_bytes_ptr(sample.payload_mut());
                loaned = Some(sample);
                Some(buffer)
            }
            Err(_) => None,
        });

        match result {
            Ok(Some((size, message_info))) => {
                let Some(mut sample) = loaned.take() else {
                    return Err(ReceiveError::Loan);
                };
                debug_assert!(
                    sample.payload().len() == size,
                    "Loaned payload size ({}) does not match the taken message size ({})",
                    sample.payload().len(),
                    size
                );

                if self.write_ros_header {
                    payload::write_user_header(
                        sample.user_header_mut(),
                        RosHeader::from(message_info),
                    );
                }

                Ok(Some(payload::assume_init(sample)))
            }
            Ok(None) => Ok(None),
            Err(TakeError::LoanDeclined) => Err(ReceiveError::Loan),
            Err(TakeError::Take) => Err(ReceiveError::Take),
        }
    }
}

/// Builder for publish-subscribe [`Relay`]s.
#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    node: Rc<RclNode>,
    type_registry: &'a TypeSupportRegistry,
    description: &'a ServiceDescription,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(
        node: Rc<RclNode>,
        type_registry: &'a TypeSupportRegistry,
        description: &'a ServiceDescription,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Self {
            node,
            type_registry,
            description,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    // Endpoint creation simultaneously announces over the DDS SEDP.
    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = "publish_subscribe::Relay::create";

        let PatternDescription::PublishSubscribe(pattern_description) = &self.description.pattern
        else {
            unreachable!("relay is only built for publish-subscribe descriptions")
        };

        let topic = fail!(from origin,
            when mapping::topic(self.description.name.as_str()).ok_or(CreationError::InvalidServiceName),
            "Failed to map service name to a ROS 2 topic"
        );

        // The payload type name carries the ROS 2 type name.
        let type_name = pattern_description.payload.type_name.as_str();

        let type_support = fail!(from origin,
            when self.type_registry.load(type_name),
            with CreationError::TypeSupport,
            "Failed to load typesupport for '{}'",
            type_name
        );
        let topic_name = fail!(from origin,
            when TopicName::new(topic),
            with CreationError::InvalidTopic,
            "Invalid ROS 2 topic name '{}'",
            topic
        );
        let publisher = fail!(from origin,
            when RclPublisherBuilder::new(Rc::clone(&self.node), &topic_name, Rc::clone(&type_support)).create(),
            with CreationError::Publisher,
            "Failed to create ROS 2 publisher for topic '{}'",
            topic
        );
        let mut subscription = fail!(from origin,
            when RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic_name, type_support).create(),
            with CreationError::Subscription,
            "Failed to create ROS 2 subscription for topic '{}'",
            topic
        );

        // Reactive mode: incoming ROS 2 data wakes the tunnel.
        if let Some(wake) = &self.wake {
            let wake = wake.clone();
            fail!(from origin,
                when subscription.on_new_message(Box::new(move |_number_of_events| wake.signal())),
                with CreationError::WakeCallback,
                "Failed to register wake callback on ROS 2 subscription"
            );
        }

        // Only services declaring the RosHeader user header receive the
        // remote origin; anything else (e.g. a header-less local service)
        // must not be written to.
        let write_ros_header =
            pattern_description.user_header == TypeDescription::from(&RosHeader::type_detail());

        Ok(Relay {
            publisher,
            subscription,
            write_ros_header,
            _phantom: core::marker::PhantomData,
        })
    }
}
