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

pub mod publish_subscribe;

use core::error::Error;
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_link_backend::relay::{RelayFactory, UnsupportedRelay, UnsupportedRelayBuilder};
use iceoryx2_link_backend::service_description::{EventDescription, PublishSubscribeDescription};

use iceoryx2_link_adapter::{Adapter, EndpointDescription};
use iceoryx2_link_adapter::{Mapping, UnsupportedLength, WriteError};
use iceoryx2_link_adapter::{TranscodeError, Translator};

pub struct Factory<'a, S, A, M: Mapping, T: Translator> {
    pub(crate) adapter: &'a mut A,
    pub(crate) translator: &'a T,
    pub(crate) _service: PhantomData<(S, M)>,
}

impl<S, A, M, T> RelayFactory<S> for Factory<'_, S, A, M, T>
where
    S: Service,
    M: Mapping,
    T: Translator,
    A: Adapter<EndpointSettings = M::EndpointSettings, EndpointTypes = T::EndpointTypes>,
{
    type RemoteDescription = EndpointDescription<M::EndpointSettings, T::EndpointTypes>;
    type PublishSubscribeRelay =
        publish_subscribe::Relay<S, A::PublishSubscribeEndpoints, T::Transcoder>;
    type PublishSubscribeBuilder<'b>
        = publish_subscribe::Builder<'b, S, A, M, T>
    where
        Self: 'b;
    type EventRelay = UnsupportedRelay<S>;
    type EventBuilder<'b>
        = UnsupportedRelayBuilder<S>
    where
        Self: 'b;

    fn publish_subscribe<'b>(
        &'b mut self,
        publish_subscribe_description: PublishSubscribeDescription<'b>,
        endpoint_description: &'b Self::RemoteDescription,
    ) -> Self::PublishSubscribeBuilder<'b>
    where
        Self: 'b,
    {
        publish_subscribe::Builder {
            adapter: self.adapter,
            translator: self.translator,
            publish_subscribe_description,
            endpoint_description,
            _service: PhantomData,
        }
    }

    /// The gateway does not bridge events.
    fn event<'b>(
        &'b mut self,
        _: EventDescription<'b>,
        _: &'b Self::RemoteDescription,
    ) -> Self::EventBuilder<'b>
    where
        Self: 'b,
    {
        UnsupportedRelayBuilder::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationError {
    /// The translator has no translation for the service.
    PublishSubscribeTranslation,
    /// The adapter could not open the endpoints.
    Endpoints,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl Error for CreationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    /// The transcoder could not encode the sample.
    Transcode,
    /// The endpoints could not publish the message.
    Endpoints,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl Error for SendError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveError {
    /// The endpoints could not take the message.
    Endpoints,
    /// The transcoder could not decode the message.
    Transcode,
    /// The message does not fit the service's description.
    Malformed,
    /// The local publisher could not provide a sample.
    Loan,
}

impl From<WriteError> for ReceiveError {
    fn from(refusal: WriteError) -> Self {
        match refusal {
            WriteError::Malformed => ReceiveError::Malformed,
            WriteError::Exhausted => ReceiveError::Loan,
        }
    }
}

impl From<UnsupportedLength> for ReceiveError {
    fn from(refusal: UnsupportedLength) -> Self {
        WriteError::from(refusal).into()
    }
}

impl<Failure> From<TranscodeError<Failure>> for ReceiveError {
    fn from(error: TranscodeError<Failure>) -> Self {
        match error {
            TranscodeError::Rejected(refusal) => ReceiveError::from(refusal),
            TranscodeError::Failed(_) => ReceiveError::Transcode,
        }
    }
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl Error for ReceiveError {}
