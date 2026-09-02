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

use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2::service::{
    Service, local_threadsafe,
    marker::{CustomHeaderMarker, CustomPayloadMarker},
};
use iceoryx2_gateway_backend::{
    traits::{PublishSubscribeRelay, RelayBuilder},
    types::{
        publish_subscribe::{LoanFn, SampleMut},
        service_description::{
            PatternDescription, PublishSubscribeDescription, ServiceDescription,
        },
        wake::WakeHandle,
    },
};
use iceoryx2_log::{fail, trace, warn};
use serde::{Deserialize, Serialize};

use zenoh::{
    Session, Wait,
    pubsub::{Publisher, Subscriber},
    qos::Reliability,
    sample::{Locality, Sample},
};

use crate::descriptor;
use crate::keys;
use crate::relays::bytes::{initialize_sample, payload_bytes, user_header_bytes};
use crate::relays::wake_handler::{WakeAwareChannel, WakeAwareReceiver};

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

/// Frame carried on the publish-subscribe key of a service.
#[derive(Debug, Serialize, Deserialize)]
struct SampleFrame<'a> {
    user_header: &'a [u8],
    payload: &'a [u8],
}

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
    fn put_sample(&self, frame: &SampleFrame<'_>) -> Result<(), zenoh::Error> {
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
        let frame = SampleFrame {
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
                when postcard::from_bytes::<SampleFrame<'_>>(&bytes),
                with ReceiveError::Decode,
                "Failed to decode publish-subscribe frame received from zenoh"
            );

            let port = port_description(&self.description);
            if !validate_frame(&frame, port) {
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

fn validate_frame(frame: &SampleFrame<'_>, port: &PublishSubscribeDescription) -> bool {
    let user_header_matches = frame.user_header.len() == port.user_header.size;
    let payload_matches = match port.payload.variant {
        TypeVariant::FixedSize => frame.payload.len() == port.payload.size,
        TypeVariant::Dynamic => {
            port.payload.size != 0 && frame.payload.len() % port.payload.size == 0
        }
    };
    user_header_matches && payload_matches
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::service_description::{PortSettings, TypeDescription};

    fn type_description(variant: TypeVariant, size: usize) -> TypeDescription {
        TypeDescription {
            variant,
            type_name: "test_type".into(),
            size,
            alignment: 1,
        }
    }

    fn port_description(
        user_header: TypeDescription,
        payload: TypeDescription,
    ) -> PublishSubscribeDescription {
        PublishSubscribeDescription {
            user_header,
            payload,
            settings: PortSettings::LocalDefaults,
        }
    }

    #[test]
    fn accepts_matching_fixed_size_message() {
        let port = port_description(
            type_description(TypeVariant::FixedSize, 4),
            type_description(TypeVariant::FixedSize, 8),
        );
        let frame = SampleFrame {
            user_header: &[0u8; 4],
            payload: &[0u8; 8],
        };

        assert_that!(validate_frame(&frame, &port), eq true);
    }

    #[test]
    fn accepts_dynamic_payload_of_whole_elements() {
        let port = port_description(
            type_description(TypeVariant::FixedSize, 0),
            type_description(TypeVariant::Dynamic, 4),
        );

        for element_count in [0usize, 1, 3] {
            let payload = vec![0u8; element_count * 4];
            let frame = SampleFrame {
                user_header: &[],
                payload: &payload,
            };
            assert_that!(validate_frame(&frame, &port), eq true);
        }
    }

    #[test]
    fn rejects_mismatched_user_header_size() {
        let port = port_description(
            type_description(TypeVariant::FixedSize, 4),
            type_description(TypeVariant::FixedSize, 8),
        );
        let frame = SampleFrame {
            user_header: &[0u8; 8],
            payload: &[0u8; 8],
        };

        assert_that!(validate_frame(&frame, &port), eq false);
    }

    #[test]
    fn rejects_mismatched_fixed_size_payload() {
        let port = port_description(
            type_description(TypeVariant::FixedSize, 0),
            type_description(TypeVariant::FixedSize, 8),
        );
        let frame = SampleFrame {
            user_header: &[],
            payload: &[0u8; 12],
        };

        assert_that!(validate_frame(&frame, &port), eq false);
    }

    #[test]
    fn rejects_dynamic_payload_of_partial_elements() {
        let port = port_description(
            type_description(TypeVariant::FixedSize, 0),
            type_description(TypeVariant::Dynamic, 4),
        );
        let frame = SampleFrame {
            user_header: &[],
            payload: &[0u8; 6],
        };

        assert_that!(validate_frame(&frame, &port), eq false);
    }
}
