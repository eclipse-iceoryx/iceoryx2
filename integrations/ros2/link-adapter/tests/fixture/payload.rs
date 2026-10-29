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

use core::fmt::Debug;

use iceoryx2::active_request::ActiveRequest;
use iceoryx2::pending_response::PendingResponse;
use iceoryx2::port::client::Client;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::server::Server;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::{AllocationStrategy, IceoryxSend, ZeroCopySend};
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe::Builder;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::port_factory::request_response::PortFactory as RequestResponsePortFactory;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_link_conformance_tests::parameters::{
    PayloadShape, PublishSubscribePayload, RequestPayload, ResponsePayload, UserHeader,
};
use ros_env::std_msgs::msg::rmw;

use super::serialization;

/// The C struct of `example_interfaces/srv/AddTwoInts_Request`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("example_interfaces/srv/AddTwoInts_Request")]
#[repr(C)]
pub struct AddTwoIntsRequest {
    pub a: i64,
    pub b: i64,
}

impl From<u64> for AddTwoIntsRequest {
    fn from(a: u64) -> Self {
        Self { a: a as i64, b: 1 }
    }
}

impl From<AddTwoIntsRequest> for ros_env::example_interfaces::srv::rmw::AddTwoInts_Request {
    fn from(value: AddTwoIntsRequest) -> Self {
        Self {
            a: value.a,
            b: value.b,
        }
    }
}

impl From<ros_env::example_interfaces::srv::rmw::AddTwoInts_Request> for AddTwoIntsRequest {
    fn from(message: ros_env::example_interfaces::srv::rmw::AddTwoInts_Request) -> Self {
        Self {
            a: message.a,
            b: message.b,
        }
    }
}

/// The C struct of `example_interfaces/srv/AddTwoInts_Response`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("example_interfaces/srv/AddTwoInts_Response")]
#[repr(C)]
pub struct AddTwoIntsResponse {
    pub sum: i64,
}

impl From<u64> for AddTwoIntsResponse {
    fn from(sum: u64) -> Self {
        Self { sum: sum as i64 }
    }
}

impl From<AddTwoIntsResponse> for ros_env::example_interfaces::srv::rmw::AddTwoInts_Response {
    fn from(value: AddTwoIntsResponse) -> Self {
        Self { sum: value.sum }
    }
}

impl From<ros_env::example_interfaces::srv::rmw::AddTwoInts_Response> for AddTwoIntsResponse {
    fn from(message: ros_env::example_interfaces::srv::rmw::AddTwoInts_Response) -> Self {
        Self { sum: message.sum }
    }
}

/// The C struct of `std_msgs/msg/UInt64`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/UInt64")]
#[repr(C)]
pub struct UInt64 {
    pub data: u64,
}

impl From<u64> for UInt64 {
    fn from(data: u64) -> Self {
        Self { data }
    }
}

impl From<UInt64> for rmw::UInt64 {
    fn from(value: UInt64) -> Self {
        Self { data: value.data }
    }
}

impl From<rmw::UInt64> for UInt64 {
    fn from(message: rmw::UInt64) -> Self {
        Self { data: message.data }
    }
}

/// One byte of a serialized `std_msgs/msg/String`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/String")]
#[repr(C)]
pub struct StringByte(u8);

/// The payload carrying `bytes`.
fn string_payload(bytes: Vec<u8>) -> Vec<StringByte> {
    bytes.into_iter().map(StringByte).collect()
}

/// The bytes `payload` carries.
fn string_bytes(payload: &[StringByte]) -> Vec<u8> {
    payload.iter().map(|byte| byte.0).collect()
}

/// A payload carrying a serialized `std_msgs/msg/String`.
pub struct SerializedString;

impl PayloadShape for SerializedString {
    type Type = [StringByte];
    type Value = rmw::String;

    fn value(n: u64) -> rmw::String {
        rmw::String {
            data: format!("message {n}").as_str().into(),
        }
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<StringByte>(TypeVariant::Dynamic)
    }
}

impl PublishSubscribePayload for SerializedString {
    fn create<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<[StringByte], H, S>,
    ) -> PortFactory<S, [StringByte], H> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<[StringByte], H, S>,
    ) -> PortFactory<S, [StringByte], H> {
        builder.open().expect("service is opened")
    }

    /// A publisher growing to the messages it sends.
    fn publisher<S: Service, H: ZeroCopySend + Debug + Default>(
        service: &PortFactory<S, [StringByte], H>,
    ) -> Publisher<S, [StringByte], H> {
        service
            .publisher_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .expect("publisher is created")
    }

    fn send<S: Service, H: ZeroCopySend + Debug + Default>(
        publisher: &Publisher<S, [StringByte], H>,
        header: H,
        value: rmw::String,
    ) {
        let bytes = serialization::serialize(&value);
        let payload = string_payload(bytes);
        let mut sample = publisher
            .loan_slice_uninit(payload.len())
            .expect("sample is loaned");
        *sample.user_header_mut() = header;
        sample
            .write_from_slice(&payload)
            .send()
            .expect("sample is sent");
    }

    fn receive<S: Service, H: ZeroCopySend + Debug + Copy>(
        subscriber: &Subscriber<S, [StringByte], H>,
    ) -> Option<(H, rmw::String)> {
        subscriber
            .receive()
            .expect("receive succeeds")
            .map(|sample| {
                let bytes = string_bytes(sample.payload());
                let value = serialization::deserialize(&bytes);
                (*sample.user_header(), value)
            })
    }
}

/// One byte of a serialized `example_interfaces/srv/AddTwoInts_Request`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("example_interfaces/srv/AddTwoInts_Request")]
#[repr(C)]
pub struct RequestByte(u8);

/// The payload carrying `bytes`.
fn request_payload(bytes: Vec<u8>) -> Vec<RequestByte> {
    bytes.into_iter().map(RequestByte).collect()
}

/// The bytes `payload` carries.
fn request_bytes(payload: &[RequestByte]) -> Vec<u8> {
    payload.iter().map(|byte| byte.0).collect()
}

/// A payload carrying a serialized `example_interfaces/srv/AddTwoInts_Request`.
pub struct SerializedAddTwoIntsRequest;

impl PayloadShape for SerializedAddTwoIntsRequest {
    type Type = [RequestByte];
    type Value = ros_env::example_interfaces::srv::rmw::AddTwoInts_Request;

    fn value(n: u64) -> ros_env::example_interfaces::srv::rmw::AddTwoInts_Request {
        AddTwoIntsRequest::from(n).into()
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<RequestByte>(TypeVariant::Dynamic)
    }
}

impl RequestPayload for SerializedAddTwoIntsRequest {
    fn client<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        service: &RequestResponsePortFactory<S, [RequestByte], QH, R, RH>,
    ) -> Client<S, [RequestByte], QH, R, RH> {
        service
            .client_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .expect("client is created")
    }

    fn send_request<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        client: &Client<S, [RequestByte], QH, R, RH>,
        header: QH,
        value: ros_env::example_interfaces::srv::rmw::AddTwoInts_Request,
    ) -> PendingResponse<S, [RequestByte], QH, R, RH> {
        let bytes = serialization::serialize(&value);
        let payload = request_payload(bytes);
        let mut request = client
            .loan_slice_uninit(payload.len())
            .expect("request is loaned");
        *request.user_header_mut() = header;
        request
            .write_from_slice(&payload)
            .send()
            .expect("request is sent")
    }

    fn receive_request<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        server: &Server<S, [RequestByte], QH, R, RH>,
    ) -> Option<ActiveRequest<S, [RequestByte], QH, R, RH>> {
        server.receive().expect("receive succeeds")
    }

    fn request_header<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, [RequestByte], QH, R, RH>,
    ) -> QH {
        *request.user_header()
    }

    fn request_value<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, [RequestByte], QH, R, RH>,
    ) -> ros_env::example_interfaces::srv::rmw::AddTwoInts_Request {
        let bytes = request_bytes(request.payload());
        serialization::deserialize(&bytes)
    }
}

/// One byte of a serialized `example_interfaces/srv/AddTwoInts_Response`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("example_interfaces/srv/AddTwoInts_Response")]
#[repr(C)]
pub struct ResponseByte(u8);

/// The payload carrying `bytes`.
fn response_payload(bytes: Vec<u8>) -> Vec<ResponseByte> {
    bytes.into_iter().map(ResponseByte).collect()
}

/// The bytes `payload` carries.
fn response_bytes(payload: &[ResponseByte]) -> Vec<u8> {
    payload.iter().map(|byte| byte.0).collect()
}

/// A payload carrying a serialized `example_interfaces/srv/AddTwoInts_Response`.
pub struct SerializedAddTwoIntsResponse;

impl PayloadShape for SerializedAddTwoIntsResponse {
    type Type = [ResponseByte];
    type Value = ros_env::example_interfaces::srv::rmw::AddTwoInts_Response;

    fn value(n: u64) -> ros_env::example_interfaces::srv::rmw::AddTwoInts_Response {
        AddTwoIntsResponse::from(n).into()
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<ResponseByte>(TypeVariant::Dynamic)
    }
}

impl ResponsePayload for SerializedAddTwoIntsResponse {
    fn server<S: Service, Q: IceoryxSend + Debug + ?Sized, QH: UserHeader, RH: UserHeader>(
        service: &RequestResponsePortFactory<S, Q, QH, [ResponseByte], RH>,
    ) -> Server<S, Q, QH, [ResponseByte], RH> {
        service
            .server_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .expect("server is created")
    }

    fn send_response<
        S: Service,
        Q: IceoryxSend + Debug + ?Sized,
        QH: UserHeader,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, Q, QH, [ResponseByte], RH>,
        header: RH,
        value: ros_env::example_interfaces::srv::rmw::AddTwoInts_Response,
    ) {
        let bytes = serialization::serialize(&value);
        let payload = response_payload(bytes);
        let mut response = request
            .loan_slice_uninit(payload.len())
            .expect("response is loaned");
        *response.user_header_mut() = header;
        response
            .write_from_slice(&payload)
            .send()
            .expect("response is sent");
    }

    fn receive_response<
        S: Service,
        Q: IceoryxSend + Debug + ?Sized,
        QH: UserHeader,
        RH: UserHeader,
    >(
        pending: &PendingResponse<S, Q, QH, [ResponseByte], RH>,
    ) -> Option<(
        RH,
        ros_env::example_interfaces::srv::rmw::AddTwoInts_Response,
    )> {
        pending
            .receive()
            .expect("receive succeeds")
            .map(|response| {
                let bytes = response_bytes(response.payload());
                let value = serialization::deserialize(&bytes);
                (*response.user_header(), value)
            })
    }
}
