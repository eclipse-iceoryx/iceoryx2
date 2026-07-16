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

use core::alloc::Layout;

use iceoryx2::service::{Service, local_threadsafe};
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{
    Mapping, PayloadLayout, PublishSubscribeRelay, RelayBuilder, ResizableBuffer, Transcoder,
    Translation, Translator,
};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{
    LoanFn, Sample, SampleMut, SampleMutUninit,
};
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, ServiceDescription, TypeDescription,
};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::mapping::TopicDescription;
use crate::payload;
use crate::rcl::{
    RclNode, RclPublisher, RclPublisherBuilder, RclSubscription, RclSubscriptionBuilder, TopicName,
    subscription::TakeError,
};
use crate::ros_header::RosHeader;
use crate::typesupport::TypeSupportRegistry;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CreationError {
    Mapping,
    Translator,
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
    Translation,
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
    Translation,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// The path a relayed service's payloads take across the boundary.
#[derive(Debug)]
enum PayloadPath<C> {
    /// Bytes cross unmodified, on the relay's zero-copy path.
    Passthrough,
    /// Bytes are translated through `transcoder`, reusing scratch buffers.
    Transcode {
        transcoder: C,
        payload_layout: PayloadLayout,
        /// Reused wire-byte scratch; grows to the largest message seen.
        wire_scratch: RefCell<Vec<u8>>,
        /// Reused payload-byte scratch for dynamic layouts, where the loan
        /// can only be sized after translation; grows to the largest message
        /// seen.
        payload_scratch: RefCell<Vec<u8>>,
    },
}

// TODO: Consider moving somewhere else?
/// [`ResizableBuffer`] over a fixed-size loaned payload; refuses to resize
/// past the loan.
struct LoanBuffer<'a> {
    bytes: &'a mut [u8],
}

impl ResizableBuffer for LoanBuffer<'_> {
    fn resize(&mut self, min_capacity: usize) -> &mut [u8] {
        assert!(
            min_capacity <= self.bytes.len(),
            "translated payload ({min_capacity} bytes) exceeds the loaned capacity ({} bytes)",
            self.bytes.len()
        );
        self.bytes
    }
}

/// Relays publish-subscribe payloads between iceoryx2 and a ROS 2 topic.
#[derive(Debug)]
pub struct Relay<S: Service, T: Translator<EndpointDescription = TopicDescription>> {
    publisher: RclPublisher,
    subscription: RclSubscription,
    /// Whether the service's user-header type is [`RosHeader`], i.e. the
    /// relay may write the remote origin into received samples.
    write_ros_header: bool,
    payload_path: PayloadPath<T::Transcoder>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service, T: Translator<EndpointDescription = TopicDescription>> Relay<S, T> {
    /// The direct path: publishes the payload bytes as they are.
    fn send_passthrough(&self, payload: &[u8]) -> Result<(), SendError> {
        let origin = "publish_subscribe::Relay::send";

        fail!(from origin,
            when self.publisher.publish(payload),
            with SendError::Publish,
            "Failed to relay sample to ROS 2"
        );

        Ok(())
    }

    /// The buffered path: translates the payload into the scratch, then
    /// publishes the wire bytes.
    fn send_translated(
        &self,
        transcoder: &T::Transcoder,
        wire_scratch: &RefCell<Vec<u8>>,
        payload: &[u8],
    ) -> Result<(), SendError> {
        let origin = "publish_subscribe::Relay::send";

        let mut wire_scratch = wire_scratch.borrow_mut();
        let written = fail!(from origin,
            when transcoder.to_wire(payload, &mut *wire_scratch),
            with SendError::Translation,
            "Failed to translate payload to the wire"
        );
        fail!(from origin,
            when self.publisher.publish(&wire_scratch[..written]),
            with SendError::Publish,
            "Failed to relay translated sample to ROS 2"
        );

        Ok(())
    }

    /// The direct path: takes the wire bytes straight into a loaned payload.
    fn receive_passthrough<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, ReceiveError> {
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

    /// The buffered path: takes the wire bytes into the wire scratch, then
    /// translates them into a loaned payload of the resolved layout.
    fn receive_translated<LoanError>(
        &self,
        transcoder: &T::Transcoder,
        payload_layout: PayloadLayout,
        wire_scratch: &RefCell<Vec<u8>>,
        payload_scratch: &RefCell<Vec<u8>>,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, ReceiveError> {
        // The wire scratch is rcl's take destination here, the transcoder reads
        // it and writes the translation towards the loan.
        let mut wire_scratch = wire_scratch.borrow_mut();
        let result = self.subscription.take_into(|size| {
            if wire_scratch.len() < size {
                wire_scratch.resize(size, 0);
            }
            Some(wire_scratch.as_mut_ptr())
        });

        match result {
            Ok(Some((size, message_info))) => {
                let mut sample = match payload_layout {
                    PayloadLayout::FixedSize(layout) => {
                        self.translate_into_loan(transcoder, &wire_scratch[..size], layout, loan)?
                    }
                    PayloadLayout::Dynamic { element } => self.translate_via_scratch(
                        transcoder,
                        payload_scratch,
                        &wire_scratch[..size],
                        element,
                        loan,
                    )?,
                };

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

    /// Translates wire bytes directly into a loaned payload of the fixed
    /// layout, which the transcoder must fill exactly.
    fn translate_into_loan<LoanError>(
        &self,
        transcoder: &T::Transcoder,
        wire: &[u8],
        layout: Layout,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<SampleMutUninit<S>, ReceiveError> {
        let origin = "publish_subscribe::Relay::receive";

        let mut sample = match loan(layout.size()) {
            Ok(sample) => sample,
            Err(_) => return Err(ReceiveError::Loan),
        };

        let mut loan_buffer = LoanBuffer {
            bytes: payload::zeroed_bytes(sample.payload_mut()),
        };
        let written = fail!(from origin,
            when transcoder.from_wire(wire, &mut loan_buffer),
            with ReceiveError::Translation,
            "Failed to translate received payload from the wire"
        );
        if written != layout.size() {
            fail!(from origin,
                with ReceiveError::Translation,
                "Translated payload ({} bytes) does not fill the fixed payload layout ({} bytes)",
                written,
                layout.size()
            );
        }

        Ok(sample)
    }

    /// Translates wire bytes into the payload scratch, then loans a payload
    /// sized to the translated length and copies the translation into it.
    fn translate_via_scratch<LoanError>(
        &self,
        transcoder: &T::Transcoder,
        payload_scratch: &RefCell<Vec<u8>>,
        wire: &[u8],
        element: Layout,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<SampleMutUninit<S>, ReceiveError> {
        let origin = "publish_subscribe::Relay::receive";

        let mut payload_scratch = payload_scratch.borrow_mut();
        let written = fail!(from origin,
            when transcoder.from_wire(wire, &mut *payload_scratch),
            with ReceiveError::Translation,
            "Failed to translate received payload from the wire"
        );
        if !written.is_multiple_of(element.size()) {
            fail!(from origin,
                with ReceiveError::Translation,
                "Translated payload ({} bytes) is not a whole number of {}-byte elements",
                written,
                element.size()
            );
        }

        let mut sample = match loan(written) {
            Ok(sample) => sample,
            Err(_) => return Err(ReceiveError::Loan),
        };
        payload::copy_into_uninit(sample.payload_mut(), &payload_scratch[..written]);

        Ok(sample)
    }
}

impl<S: Service, T: Translator<EndpointDescription = TopicDescription>> PublishSubscribeRelay<S>
    for Relay<S, T>
{
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError> {
        let payload = payload::as_bytes(sample.payload());
        match &self.payload_path {
            PayloadPath::Passthrough => self.send_passthrough(payload),
            PayloadPath::Transcode {
                transcoder, wire_scratch, ..
            } => self.send_translated(transcoder, wire_scratch, payload),
        }
    }

    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        match &self.payload_path {
            PayloadPath::Passthrough => self.receive_passthrough(loan),
            PayloadPath::Transcode {
                transcoder,
                payload_layout,
                wire_scratch,
                payload_scratch,
            } => self.receive_translated(
                transcoder,
                *payload_layout,
                wire_scratch,
                payload_scratch,
                loan,
            ),
        }
    }
}

/// Builder for publish-subscribe [`Relay`]s.
#[derive(Debug)]
pub struct Builder<
    'a,
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> {
    node: Rc<RclNode>,
    type_registry: &'a TypeSupportRegistry,
    mapping: &'a M,
    translator: Rc<T>,
    service_description: &'a ServiceDescription,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    'a,
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> Builder<'a, S, M, T>
{
    pub fn new(
        service_description: &'a ServiceDescription,
        node: Rc<RclNode>,
        type_registry: &'a TypeSupportRegistry,
        mapping: &'a M,
        translator: Rc<T>,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Self {
            node,
            type_registry,
            mapping,
            translator,
            service_description,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> RelayBuilder for Builder<'_, S, M, T>
{
    type CreationError = CreationError;
    type Relay = Relay<S, T>;

    // Endpoint creation simultaneously announces over the DDS SEDP.
    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = "publish_subscribe::Relay::create";

        let PatternDescription::PublishSubscribe(pattern_description) =
            &self.service_description.pattern
        else {
            unreachable!("relay is only built for publish-subscribe descriptions")
        };

        let Some(topic_description) = self.mapping.remote(self.service_description) else {
            fail!(from origin, with CreationError::Mapping,
                "Mapping does not map service '{}' to a ROS 2 topic",
                self.service_description.name
            );
        };

        let translation = fail!(from origin,
            when self.translator.create(self.service_description, &topic_description),
            with CreationError::Translator,
            "Translator failed to create translation for service '{}'",
            self.service_description.name
        );

        let type_name = topic_description.type_name.as_str();
        let type_support = fail!(from origin,
            when self.type_registry.load(type_name),
            with CreationError::TypeSupport,
            "Failed to load typesupport for '{}'",
            type_name
        );

        let topic_name = TopicName::from(&topic_description.topic);
        let qos = &topic_description.qos;
        let publisher = fail!(from origin,
            when RclPublisherBuilder::new(Rc::clone(&self.node), &topic_name, Rc::clone(&type_support))
                .qos(qos.clone())
                .create(),
            with CreationError::Publisher,
            "Failed to create ROS 2 publisher for topic '{}'",
            topic_name.as_str()
        );
        let mut subscription = fail!(from origin,
            when RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic_name, type_support)
                .qos(qos.clone())
                .create(),
            with CreationError::Subscription,
            "Failed to create ROS 2 subscription for topic '{}'",
            topic_name.as_str()
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

        let payload_path = match translation {
            Translation::Passthrough => PayloadPath::Passthrough,
            Translation::Transcode {
                payload_layout,
                transcoder,
            } => {
                if let PayloadLayout::Dynamic { element } = payload_layout
                    && element.size() == 0
                {
                    fail!(from origin, with CreationError::Translator,
                        "Translator produced a dynamic payload with a zero-sized element for service '{}'",
                        self.service_description.name
                    );
                }
                PayloadPath::Transcode {
                    transcoder,
                    payload_layout,
                    wire_scratch: RefCell::new(Vec::new()),
                    payload_scratch: RefCell::new(Vec::new()),
                }
            }
        };

        Ok(Relay {
            publisher,
            subscription,
            write_ros_header,
            payload_path,
            _phantom: core::marker::PhantomData,
        })
    }
}
