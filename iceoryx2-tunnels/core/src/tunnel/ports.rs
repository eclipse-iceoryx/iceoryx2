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

use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::Service;

pub(crate) enum Ports<S: Service> {
    PublishSubscribe(
        Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
        Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ),
    Event(Notifier<S>, Listener<S>),
}
