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

use std::sync::Arc;

use iceoryx2::service::{
    Service, local_threadsafe,
    marker::{CustomHeaderMarker, CustomPayloadMarker},
};
use iceoryx2_gateway_backend::{
    traits::{PublishSubscribeRelay, RelayBuilder},
    types::{
        publish_subscribe::{LoanFn, SampleMut, SampleMutUninit},
        service_description::{
            PatternDescription, PublishSubscribeDescription, ServiceDescription,
        },
        wake::WakeHandle,
    },
};
use iceoryx2_log::{fail, trace, warn};

use zenoh::{
    Session, Wait,
    pubsub::{Publisher, Subscriber},
    qos::Reliability,
    sample::{Locality, Sample},
};

use crate::relays::wake_handler::{WakeAwareChannel, WakeAwareReceiver};
use crate::wire::descriptor;
use crate::wire::keys;
use crate::wire::message::{MessageFrame, payload_bytes, user_header_bytes, validate_frame};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    DescriptionEncoding,
    PublisherDeclaration,
    SubscriberDeclaration,
    ServiceAnouncement,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    PayloadPut,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    SampleReceive,
    Decode,
    IceoryxLoan,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    session: &'a Session,
    description: &'a ServiceDescription,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(
        session: &'a Session,
        description: &'a ServiceDescription,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Builder<'a, S> {
        Builder {
            session,
            description,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = "publish_subscribe::Builder::create";
        let descriptor = fail!(
            from origin,
            when descriptor::describe(self.description),
            with CreationError::DescriptionEncoding,
            "Failed to encode service description"
        );
        let key = keys::publish_subscribe(&descriptor);

        let publisher = fail!(
            from origin,
            when self.session
                .declare_publisher(key.clone())
                .allowed_destination(Locality::Remote)
                .reliability(Reliability::Reliable)
                .wait(),
            with CreationError::PublisherDeclaration,
            "Failed to create zenoh publisher for publish-subscribe payloads"
        );

        // TODO(correctness): Make handler buffer capacity configurable
        let subscriber = fail!(
            from origin,
            when self.session
                .declare_subscriber(key.clone())
                .with(WakeAwareChannel::new(10, self.wake))
                .allowed_origin(Locality::Remote)
                .wait(),
            with CreationError::SubscriberDeclaration,
            "Failed to create zenoh subscriber for publish-subscribe payloads"
        );

        Ok(Relay {
            description: self.description.clone(),
            publisher,
            subscriber,
            _phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    description: ServiceDescription,
    publisher: Publisher<'static>,
    subscriber: Subscriber<WakeAwareReceiver<Sample>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Relay<S> {
    fn put_sample(&self, frame: &MessageFrame<'_>) -> Result<(), zenoh::Error> {
        let bytes = postcard::to_allocvec(frame)?;
        self.publisher.put(bytes).wait()
    }
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(
        &self,
        sample: iceoryx2::sample::Sample<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ) -> Result<(), Self::SendError> {
        trace!(
            from self,
            "Sending {}({})",
            self.description.pattern,
            self.description.name
        );

        let port = port_description(&self.description);
        let frame = MessageFrame {
            user_header: user_header_bytes(sample.user_header(), port.user_header.size),
            payload: payload_bytes(sample.payload()),
        };

        fail!(
            from self,
            when self.put_sample(&frame),
            with SendError::PayloadPut,
            "Failed to propagate publish-subscribe payload to zenoh"
        );

        Ok(())
    }

    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        loop {
            let zenoh_sample = fail!(
                from self,
                when self.subscriber.try_recv(),
                with ReceiveError::SampleReceive,
                "Failed to receive sample from Zenoh"
            );
            let Some(zenoh_sample) = zenoh_sample else {
                return Ok(None);
            };

            trace!(
                from self,
                "Ingesting {}({})",
                self.description.pattern,
                self.description.name
            );

            let bytes = zenoh_sample.payload().to_bytes();
            let frame = fail!(
                from self,
                when postcard::from_bytes::<MessageFrame<'_>>(&bytes),
                with ReceiveError::Decode,
                "Failed to decode publish-subscribe frame received from zenoh"
            );

            let port = port_description(&self.description);
            if !validate_frame(&frame, &port.user_header, &port.payload) {
                warn!(
                    from self,
                    "Discarding sample of {}({}), its message layout does not match \
                    the local service description",
                    self.description.pattern,
                    self.description.name
                );
                continue;
            }

            let iceoryx_sample = fail!(
                from self,
                when loan(frame.payload.len()),
                with ReceiveError::IceoryxLoan,
                "Failed to loan sample from iceoryx"
            );

            let sample =
                unsafe { initialize_sample(frame.user_header, frame.payload, iceoryx_sample) };

            return Ok(Some(sample));
        }
    }
}

fn port_description(description: &ServiceDescription) -> &PublishSubscribeDescription {
    let PatternDescription::PublishSubscribe(description) = &description.pattern else {
        unreachable!("relay is only built for publish-subscribe descriptions")
    };

    description
}

/// Initializes a loaned sample with received user header and payload
/// bytes.
///
/// # Safety
///
/// `user_header` must be exactly the size of the service's user header
/// type and `payload` must be exactly the size of the loan.
unsafe fn initialize_sample<S: Service>(
    user_header: &[u8],
    payload: &[u8],
    mut sample: SampleMutUninit<S>,
) -> SampleMut<S> {
    debug_assert!(
        sample.payload_mut().len() == payload.len(),
        "Loaned payload size ({}) does not match received payload ({})",
        sample.payload_mut().len(),
        payload.len()
    );

    unsafe {
        core::ptr::copy_nonoverlapping(
            user_header.as_ptr(),
            sample.user_header_mut() as *mut CustomHeaderMarker as *mut u8,
            user_header.len(),
        );
        core::ptr::copy_nonoverlapping(
            payload.as_ptr(),
            sample.payload_mut().as_mut_ptr().cast::<u8>(),
            payload.len(),
        );
        sample.assume_init()
    }
}
