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
pub trait Translator: Default + Debug + Send + 'static {
    /// Error type returned by the translator's methods.
    type Error: Error;

    /// Type describing the remote endpoints to translate to/from.
    type EndpointDescription;

    /// The transcoder used to translate the bytes.
    type Transcoder: Transcoder<Error = Self::Error> + Debug;

    /// Creates a [`Translation`] instance for the specified iceoryx2 service
    /// and remote endpoint pair.
    fn create(
        &self,
        service: &ServiceDescription,
        endpoint: &Self::EndpointDescription,
    ) -> Result<Translation<Self::Transcoder>, Self::Error>;
}

/// The logic to convert payload bytes between wire format and shared memory
/// format.
pub trait Transcoder {
    /// Error type returned by the transcoder's methods.
    type Error: Error;

    /// Translates an iceoryx2 payload into its wire representation, written
    /// into `wire`.
    ///
    /// Returns the number of bytes written.
    fn to_wire(
        &self,
        payload: &[u8],
        wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error>;

    /// Translates a wire payload into its shared memory representation,
    /// written into `payload`.
    ///
    /// Returns the number of bytes written.
    #[allow(clippy::wrong_self_convention)]
    fn from_wire(
        &self,
        wire: &[u8],
        payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error>;
}

/// An instantiation of the [`Translator`] for a specific service/endpoint
/// pair.
#[derive(Debug)]
pub enum Translation<T> {
    /// Payload bytes cross unmodified. Bytes are moved directly to the
    /// destination with no intermediate buffering or transcoding.
    Passthrough,
    /// Payload bytes are translated into the target representation by `transcoder`
    /// in an intermediate buffer before being written to the destination.
    Transcode {
        /// Layout of the payload in shared memory.
        payload_layout: PayloadLayout,
        /// Transcoder converting the payload bytes in both directions.
        transcoder: T,
    },
}

/// Layout of the payload in shared memory.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PayloadLayout {
    /// A single value of a static size.
    FixedSize(Layout),
    /// A slice of elements whose length varies per message.
    Dynamic { element: Layout },
}

/// The identity [`Translator`]: payloads cross unmodified in both directions.
pub struct Passthrough<EndpointDescription> {
    _endpoint: PhantomData<fn() -> EndpointDescription>,
}

impl<EndpointDescription> Passthrough<EndpointDescription> {
    pub fn new() -> Self {
        Self {
            _endpoint: PhantomData,
        }
    }
}

impl<EndpointDescription> Default for Passthrough<EndpointDescription> {
    fn default() -> Self {
        Self::new()
    }
}

impl<EndpointDescription> Debug for Passthrough<EndpointDescription> {
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

impl<EndpointDescription: 'static> Translator for Passthrough<EndpointDescription> {
    type Error = core::convert::Infallible;
    type EndpointDescription = EndpointDescription;
    type Transcoder = Never;

    fn create(
        &self,
        _service: &ServiceDescription,
        _endpoint: &Self::EndpointDescription,
    ) -> Result<Translation<Self::Transcoder>, Self::Error> {
        Ok(Translation::Passthrough)
    }
}

/// A [`Transcoder`] that cannot be instantiated for [`Translator`]s that
/// should never do any transcoding.
#[derive(Debug)]
pub enum Never {}

impl Transcoder for Never {
    type Error = core::convert::Infallible;

    fn to_wire(
        &self,
        _payload: &[u8],
        _wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        match *self {}
    }

    fn from_wire(
        &self,
        _wire: &[u8],
        _payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        match *self {}
    }
}
