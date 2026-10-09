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

use iceoryx2::active_request::RequestId;
use iceoryx2::identifiers::UniqueClientId;
use iceoryx2::service::Service;
use iceoryx2::service::header::request_response::RequestHeader;
use iceoryx2_log::{fail, origin};
use serde::{Deserialize, Serialize};

use crate::wire::region::ResizeError;
use crate::wire::sample::{
    FlatbufferMemory, Header, LoanableSample, Payload, PayloadUninit, UninitializedSample,
};

pub type Client<S> = iceoryx2::port::client::Client<S, Payload, Header, Payload, Header>;
pub type Server<S> = iceoryx2::port::server::Server<S, Payload, Header, Payload, Header>;

pub type RequestMut<S> = iceoryx2::request_mut::RequestMut<S, Payload, Header, Payload, Header>;
pub type RequestMutUninit<S> =
    iceoryx2::request_mut_uninit::RequestMutUninit<S, PayloadUninit, Header, Payload, Header>;
pub type PendingResponse<S> =
    iceoryx2::pending_response::PendingResponse<S, Payload, Header, Payload, Header>;

pub type ActiveRequest<S> =
    iceoryx2::active_request::ActiveRequest<S, Payload, Header, Payload, Header>;
pub type Response<S> = iceoryx2::response::Response<S, Payload, Header>;
pub type ResponseMut<S> = iceoryx2::response_mut::ResponseMut<S, Payload, Header>;
pub type ResponseMutUninit<S> =
    iceoryx2::response_mut_uninit::ResponseMutUninit<S, PayloadUninit, Header>;

/// Identifies a request by the client that issued it on both sides of the
/// boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientRequestId {
    client: UniqueClientId,
    request: RequestId,
}

impl ClientRequestId {
    pub fn of(header: &RequestHeader) -> Self {
        Self {
            client: header.client_id(),
            request: header.request_id(),
        }
    }
}

impl core::fmt::Display for ClientRequestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.client, self.request.value())
    }
}

/// A loanable request for the local servers.
pub trait LoanableRequest<S: Service>: LoanableSample<InitializedSample = RequestMut<S>> {}

impl<S: Service, L: LoanableSample<InitializedSample = RequestMut<S>>> LoanableRequest<S> for L {}

/// The loanable response to any held request.
pub trait LoanableResponse<S: Service> {
    type Loanable: LoanableSample<InitializedSample = ResponseMut<S>>;

    /// The loanable response to the request `id`.
    ///
    /// Returns `None` if the request is no longer held.
    fn of(self, id: ClientRequestId) -> Option<Self::Loanable>;
}

impl<S, L, F> LoanableResponse<S> for F
where
    S: Service,
    L: LoanableSample<InitializedSample = ResponseMut<S>>,
    F: FnOnce(ClientRequestId) -> Option<L>,
{
    type Loanable = L;

    fn of(self, id: ClientRequestId) -> Option<L> {
        self(id)
    }
}

/// View of the initialized bytes of a request's payload.
pub fn request_payload_bytes<S: Service>(request: &ActiveRequest<S>) -> &[u8] {
    let payload = request.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    let bytes =
        unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) };
    &bytes[request.header().payload_offset() as usize..]
}

/// View of the initialized bytes of a response's payload.
pub fn response_payload_bytes<S: Service>(response: &Response<S>) -> &[u8] {
    let payload = response.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    let bytes =
        unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) };
    &bytes[response.header().payload_offset() as usize..]
}

impl<S: Service> UninitializedSample for RequestMutUninit<S> {
    type InitializedSample = RequestMut<S>;
    type FlatbufferMemory = iceoryx2::request_mut_uninit::FlatbufferMemory<S>;

    fn user_header(&self) -> &Header {
        RequestMutUninit::user_header(self)
    }

    fn user_header_mut(&mut self) -> &mut Header {
        RequestMutUninit::user_header_mut(self)
    }

    fn payload(&self) -> &PayloadUninit {
        RequestMutUninit::payload(self)
    }

    fn payload_mut(&mut self) -> &mut PayloadUninit {
        RequestMutUninit::payload_mut(self)
    }

    fn flatbuffer_memory(&self) -> Self::FlatbufferMemory {
        self.__internal_create_resizable_memory()
    }

    fn finish_serialized(&mut self, payload: *const u8) {
        self.__internal_finish_serialized(payload)
    }

    unsafe fn assume_init(self) -> RequestMut<S> {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { RequestMutUninit::assume_init(self) }
    }
}

impl<S: Service> FlatbufferMemory for iceoryx2::request_mut_uninit::FlatbufferMemory<S> {
    fn grow(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("FlatbufferMemory::grow");

        fail!(
            from origin,
            when self.grow_downwards_with_size(len, 0),
            with ResizeError::Exhausted,
            "Failed to grow the memory to {} bytes", len
        );
        Ok(())
    }
}

impl<S: Service> UninitializedSample for ResponseMutUninit<S> {
    type InitializedSample = ResponseMut<S>;
    type FlatbufferMemory = iceoryx2::response_mut_uninit::FlatbufferMemory<S>;

    fn user_header(&self) -> &Header {
        ResponseMutUninit::user_header(self)
    }

    fn user_header_mut(&mut self) -> &mut Header {
        ResponseMutUninit::user_header_mut(self)
    }

    fn payload(&self) -> &PayloadUninit {
        ResponseMutUninit::payload(self)
    }

    fn payload_mut(&mut self) -> &mut PayloadUninit {
        ResponseMutUninit::payload_mut(self)
    }

    fn flatbuffer_memory(&self) -> Self::FlatbufferMemory {
        self.__internal_create_resizable_memory()
    }

    fn finish_serialized(&mut self, payload: *const u8) {
        self.__internal_finish_serialized(payload)
    }

    unsafe fn assume_init(self) -> ResponseMut<S> {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { ResponseMutUninit::assume_init(self) }
    }
}

impl<S: Service> FlatbufferMemory for iceoryx2::response_mut_uninit::FlatbufferMemory<S> {
    fn grow(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("FlatbufferMemory::grow");

        fail!(
            from origin,
            when self.grow_downwards_with_size(len, 0),
            with ResizeError::Exhausted,
            "Failed to grow the memory to {} bytes", len
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::{Node, NodeBuilder};
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::AllocationStrategy;
    use iceoryx2::service::local;
    use iceoryx2::service::marker::Flatbuffer;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;

    use crate::service_description::{SampleTypes, Schema, TypeIdentifier};
    use crate::wire::sample::{LoanableSample, UnloanedSample, WritableSample};

    type Factory = PortFactory<local::Service, Payload, Header, Payload, Header>;

    fn no_user_header() -> TypeDetail {
        TypeDetail::new::<()>(TypeVariant::FixedSize)
    }

    fn sample_types(payload: &TypeDetail) -> SampleTypes {
        SampleTypes {
            payload: payload.into(),
            user_header: (&no_user_header()).into(),
        }
    }

    fn flatbuffer_sample_types(schema: &[u8]) -> SampleTypes {
        let detail = TypeDetail::new::<Flatbuffer<u64>>(TypeVariant::FixedSize);
        let mut types = sample_types(&detail);
        types.payload.identifier = TypeIdentifier::Flatbuffer(Schema::new(schema.to_vec()));
        types
    }

    /// The untyped request-response service with `u64` requests and `u32`
    /// responses.
    fn plain_service(node: &Node<local::Service>) -> Factory {
        // SAFETY: the type details describe the markers' real types.
        unsafe {
            node.service_builder(&generate_service_name())
                .request_response::<Payload, Payload>()
                .request_user_header::<Header>()
                .response_user_header::<Header>()
                .__internal_set_request_payload_type_details(&TypeDetail::new::<u64>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_request_header_type_details(&no_user_header())
                .__internal_set_response_payload_type_details(&TypeDetail::new::<u32>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_response_header_type_details(&no_user_header())
        }
        .create()
        .expect("service is created")
    }

    /// The untyped request-response service with flatbuffer requests and
    /// responses under the given schemas.
    fn flatbuffer_service(
        node: &Node<local::Service>,
        request_schema: &[u8],
        response_schema: &[u8],
    ) -> Factory {
        let flatbuffer = TypeDetail::new::<Flatbuffer<u64>>(TypeVariant::FixedSize);
        // SAFETY: the type details and schemas describe the markers' real
        // types.
        unsafe {
            node.service_builder(&generate_service_name())
                .request_response::<Payload, Payload>()
                .request_user_header::<Header>()
                .response_user_header::<Header>()
                .__internal_set_request_payload_type_details(&flatbuffer)
                .__internal_set_request_header_type_details(&no_user_header())
                .__internal_set_response_payload_type_details(&flatbuffer)
                .__internal_set_response_header_type_details(&no_user_header())
                .__internal_request_flatbuffer_schema(request_schema)
                .__internal_response_flatbuffer_schema(response_schema)
        }
        .create()
        .expect("service is created")
    }

    #[test]
    fn requests_and_responses_round_trip_through_forward_loans() {
        const REQUEST: u64 = 42;
        const RESPONSE: u32 = 7;

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service = plain_service(&node);
        let client = service
            .client_builder()
            .create()
            .expect("client is created");
        let server = service
            .server_builder()
            .create()
            .expect("server is created");
        let request_types = sample_types(&TypeDetail::new::<u64>(TypeVariant::FixedSize));
        let response_types = sample_types(&TypeDetail::new::<u32>(TypeVariant::FixedSize));

        // The request is loaned from the client and written from its front.
        let mut loan = |len: usize| -> Result<RequestMutUninit<local::Service>, LoanError> {
            // SAFETY: the client was created for the untyped markers.
            unsafe { client.loan_custom_payload(len / core::mem::size_of::<u64>()) }
        };
        let mut request = UnloanedSample::new(&request_types, &mut loan)
            .loan_forward(core::mem::size_of::<u64>())
            .expect("one u64 fits");
        request
            .as_mut()
            .payload
            .copy_from_slice(&REQUEST.to_ne_bytes());
        // SAFETY: the payload was written above and the header is zero sized.
        let pending = unsafe { request.assume_init() }
            .send()
            .expect("request is sent");
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the request arrived");
        let request_bytes = REQUEST.to_ne_bytes();
        assert_that!(request_payload_bytes(&active), eq request_bytes.as_slice());
        assert_that!(ClientRequestId::of(active.header()), eq ClientRequestId::of(pending.header()));

        // The response is loaned from the active request and written from
        // its front.
        let mut loan = |len: usize| -> Result<ResponseMutUninit<local::Service>, LoanError> {
            // SAFETY: the server was created for the untyped markers.
            unsafe { active.loan_custom_payload(len / core::mem::size_of::<u32>()) }
        };
        let mut response = UnloanedSample::new(&response_types, &mut loan)
            .loan_forward(core::mem::size_of::<u32>())
            .expect("one u32 fits");
        response
            .as_mut()
            .payload
            .copy_from_slice(&RESPONSE.to_ne_bytes());
        // SAFETY: the payload was written above and the header is zero sized.
        unsafe { response.assume_init() }
            .send()
            .expect("response is sent");
        // SAFETY: the client was created for the untyped markers.
        let received = unsafe { pending.receive_custom_payload() }
            .expect("receive succeeds")
            .expect("the response arrived");
        let response_bytes = RESPONSE.to_ne_bytes();
        assert_that!(response_payload_bytes(&received), eq response_bytes.as_slice());
    }

    #[test]
    fn flatbuffer_requests_and_responses_round_trip_through_backward_loans() {
        const REQUEST_SCHEMA: &[u8] = b"the request schema";
        const RESPONSE_SCHEMA: &[u8] = b"the response schema";
        const REQUEST: &[u8] = b"a serialized request";
        const RESPONSE: &[u8] = b"a longer serialized response";

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service = flatbuffer_service(&node, REQUEST_SCHEMA, RESPONSE_SCHEMA);
        let client = service
            .client_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .expect("client is created");
        let server = service
            .server_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .expect("server is created");
        let request_types = flatbuffer_sample_types(REQUEST_SCHEMA);
        let response_types = flatbuffer_sample_types(RESPONSE_SCHEMA);

        // The request is loaned from the client and grown from its end.
        let mut loan = |_: usize| -> Result<RequestMutUninit<local::Service>, LoanError> {
            // SAFETY: the client was created for the untyped markers.
            unsafe { client.loan_custom_payload(1) }
        };
        let mut request = UnloanedSample::new(&request_types, &mut loan)
            .loan_backward(REQUEST.len())
            .expect("the bytes fit");
        request.as_mut().payload.copy_from_slice(REQUEST);
        // SAFETY: the payload was written above and the header is zero sized.
        let pending = unsafe { request.assume_init() }
            .send()
            .expect("request is sent");
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the request arrived");
        assert_that!(request_payload_bytes(&active), eq REQUEST);

        // The response is loaned from the active request and grown from its
        // end.
        let mut loan = |_: usize| -> Result<ResponseMutUninit<local::Service>, LoanError> {
            // SAFETY: the server was created for the untyped markers.
            unsafe { active.loan_custom_payload(1) }
        };
        let mut response = UnloanedSample::new(&response_types, &mut loan)
            .loan_backward(RESPONSE.len())
            .expect("the bytes fit");
        response.as_mut().payload.copy_from_slice(RESPONSE);
        // SAFETY: the payload was written above and the header is zero sized.
        unsafe { response.assume_init() }
            .send()
            .expect("response is sent");
        // SAFETY: the client was created for the untyped markers.
        let received = unsafe { pending.receive_custom_payload() }
            .expect("receive succeeds")
            .expect("the response arrived");
        assert_that!(response_payload_bytes(&received), eq RESPONSE);
    }
}
