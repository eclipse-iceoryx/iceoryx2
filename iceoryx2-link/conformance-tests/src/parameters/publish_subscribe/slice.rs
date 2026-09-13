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
use alloc::vec::Vec;
use core::fmt::Debug;

use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe::Builder;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::parameters::payload::slice::SLICE_LEN;
use crate::parameters::{PublishSubscribePayload, SlicePayload};

impl<T> PublishSubscribePayload for SlicePayload<T>
where
    T: ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u8> + 'static,
{
    fn create<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<[T], H, S>,
    ) -> PortFactory<S, [T], H> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<[T], H, S>,
    ) -> PortFactory<S, [T], H> {
        builder.open().expect("service is opened")
    }

    fn publisher<S: Service, H: ZeroCopySend + Debug + Default>(
        service: &PortFactory<S, [T], H>,
    ) -> Publisher<S, [T], H> {
        service
            .publisher_builder()
            .initial_max_slice_len(SLICE_LEN)
            .create()
            .expect("publisher is created")
    }

    fn send<S: Service, H: ZeroCopySend + Debug + Default>(
        publisher: &Publisher<S, [T], H>,
        header: H,
        value: Vec<T>,
    ) {
        let mut sample = publisher
            .loan_slice_uninit(value.len())
            .expect("sample is loaned");
        *sample.user_header_mut() = header;
        sample
            .write_from_slice(&value)
            .send()
            .expect("sample is sent");
    }

    fn receive<S: Service, H: ZeroCopySend + Debug + Copy>(
        subscriber: &Subscriber<S, [T], H>,
    ) -> Option<(H, Vec<T>)> {
        subscriber
            .receive()
            .expect("receive succeeds")
            .map(|sample| (*sample.user_header(), sample.payload().to_vec()))
    }
}
