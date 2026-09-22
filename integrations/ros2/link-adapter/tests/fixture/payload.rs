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

use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::{AllocationStrategy, ZeroCopySend};
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe::Builder;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_link_conformance_tests::parameters::{PayloadShape, PublishSubscribePayload};
use ros_env::std_msgs::msg::rmw;

use super::serialization;

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
        let bytes: Vec<StringByte> = serialization::serialize(&value)
            .into_iter()
            .map(StringByte)
            .collect();
        let mut sample = publisher
            .loan_slice_uninit(bytes.len())
            .expect("sample is loaned");
        *sample.user_header_mut() = header;
        sample
            .write_from_slice(&bytes)
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
                let bytes: Vec<u8> = sample.payload().iter().map(|byte| byte.0).collect();
                (*sample.user_header(), serialization::deserialize(&bytes))
            })
    }
}
