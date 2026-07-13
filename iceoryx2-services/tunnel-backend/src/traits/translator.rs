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

use core::alloc::Layout;
use core::error::Error;
use core::fmt::Debug;
use core::marker::PhantomData;

use crate::traits::ResizableBuffer;
use crate::types::service_description::ServiceDescription;

/// Strategy for translating payload bytes between the wire format and the
/// iceoryx2 payload.
///
/// [`Default`] must never resolve to silently wrong behavior: a strategy
/// whose default cannot translate must fail [`Translator::resolve`] rather
/// than fall back to [`TranslationMode::Passthrough`].
pub trait Translator: Default + Debug + Send + 'static {
    /// Error type returned by the strategy's methods.
    type Error: Error;

    /// The backend-side description of a tunneled service's endpoints.
    /// Must match the [`Mapping::EndpointDescription`](crate::traits::Mapping::EndpointDescription)
    /// of the backend the strategy is instantiated for.
    type EndpointDescription;

    /// Decides, once at relay creation, how payloads of the described
    /// service cross the backend. An error aborts relay creation.
    fn resolve(
        &self,
        service: &ServiceDescription,
        endpoint: &Self::EndpointDescription,
    ) -> Result<TranslationMode, Self::Error>;

    /// Translates an iceoryx2 payload into its wire representation, written
    /// into `wire`. Returns the number of bytes written.
    fn to_wire(
        &self,
        service: &ServiceDescription,
        endpoint: &Self::EndpointDescription,
        payload: &[u8],
        wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error>;

    /// Translates a wire payload into its iceoryx2 representation, written
    /// into `payload`. Returns the number of bytes written.
    fn from_wire(
        &self,
        service: &ServiceDescription,
        endpoint: &Self::EndpointDescription,
        wire: &[u8],
        payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error>;
}

// TODO: Consider if supporting both passthrough and translation modes
//       in one translator is really needed...
/// How a service's payloads cross the backend.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TranslationMode {
    /// Payload bytes cross unmodified.
    Passthrough,
    /// Payloads are transcoded via [`Translator::to_wire`] and
    /// [`Translator::from_wire`].
    Translate {
        /// Layout of the iceoryx2-side payload. Applied when the tunnel
        /// creates the local service for a remotely discovered one, where
        /// only the translator knows the native layout.
        native_layout: Layout,
    },
}

/// The identity [`Translator`]: resolves every service to
/// [`TranslationMode::Passthrough`], so payloads cross unmodified in both
/// directions.
pub struct Passthrough<E> {
    _endpoint: PhantomData<fn() -> E>,
}

impl<E> Passthrough<E> {
    pub fn new() -> Self {
        Self {
            _endpoint: PhantomData,
        }
    }
}

impl<E> Default for Passthrough<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> Debug for Passthrough<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Passthrough")
    }
}

impl<E> Clone for Passthrough<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for Passthrough<E> {}

impl<E: 'static> Translator for Passthrough<E> {
    type Error = core::convert::Infallible;
    type EndpointDescription = E;

    fn resolve(
        &self,
        _service: &ServiceDescription,
        _endpoint: &Self::EndpointDescription,
    ) -> Result<TranslationMode, Self::Error> {
        Ok(TranslationMode::Passthrough)
    }

    fn to_wire(
        &self,
        _service: &ServiceDescription,
        _endpoint: &Self::EndpointDescription,
        _payload: &[u8],
        _wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        unreachable!("Passthrough resolves every service to TranslationMode::Passthrough")
    }

    fn from_wire(
        &self,
        _service: &ServiceDescription,
        _endpoint: &Self::EndpointDescription,
        _wire: &[u8],
        _payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        unreachable!("Passthrough resolves every service to TranslationMode::Passthrough")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use iceoryx2::service::local::Service;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2_bb_testing::assert_that;

    use crate::types::service_description::{
        PatternDescription, PortSettings, PublishSubscribeDescription, TypeDescription,
    };

    fn service_description() -> ServiceDescription {
        ServiceDescription::new::<Service>(
            "translator-tests".try_into().expect("valid service name"),
            PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
                payload: TypeDescription::from(&TypeDetail::new::<u64>(TypeVariant::FixedSize)),
                settings: PortSettings::LocalDefaults,
            }),
        )
    }

    /// Wire format: every payload byte appears twice.
    #[derive(Debug, Default)]
    struct DuplicatingTranslator;

    impl Translator for DuplicatingTranslator {
        type Error = core::convert::Infallible;
        type EndpointDescription = String;

        fn resolve(
            &self,
            _service: &ServiceDescription,
            _endpoint: &String,
        ) -> Result<TranslationMode, Self::Error> {
            Ok(TranslationMode::Translate {
                native_layout: Layout::new::<u8>(),
            })
        }

        fn to_wire(
            &self,
            _service: &ServiceDescription,
            _endpoint: &String,
            payload: &[u8],
            wire: &mut impl ResizableBuffer,
        ) -> Result<usize, Self::Error> {
            let region = wire.resize(payload.len() * 2);
            for (index, byte) in payload.iter().enumerate() {
                region[index * 2] = *byte;
                region[index * 2 + 1] = *byte;
            }
            Ok(payload.len() * 2)
        }

        fn from_wire(
            &self,
            _service: &ServiceDescription,
            _endpoint: &String,
            wire: &[u8],
            payload: &mut impl ResizableBuffer,
        ) -> Result<usize, Self::Error> {
            let region = payload.resize(wire.len() / 2);
            for (index, byte) in region.iter_mut().enumerate().take(wire.len() / 2) {
                *byte = wire[index * 2];
            }
            Ok(wire.len() / 2)
        }
    }

    #[test]
    fn passthrough_resolves_every_service_to_passthrough() {
        let translator = Passthrough::<String>::new();

        let mode = translator
            .resolve(&service_description(), &String::from("endpoint"))
            .expect("passthrough resolution is infallible");

        assert_that!(mode, eq TranslationMode::Passthrough);
    }

    #[test]
    fn translated_payload_round_trips_through_translation_buffers() {
        const PAYLOAD: [u8; 4] = [1, 2, 3, 4];

        let translator = DuplicatingTranslator;
        let service = service_description();
        let endpoint = String::from("endpoint");

        let mut wire = Vec::<u8>::new();
        let wire_len = translator
            .to_wire(&service, &endpoint, &PAYLOAD, &mut wire)
            .expect("translation to the wire is infallible");
        assert_that!(wire[..wire_len], eq vec![1, 1, 2, 2, 3, 3, 4, 4]);

        let mut payload = Vec::<u8>::new();
        let payload_len = translator
            .from_wire(&service, &endpoint, &wire[..wire_len], &mut payload)
            .expect("translation from the wire is infallible");
        assert_that!(payload[..payload_len], eq PAYLOAD);
    }
}
