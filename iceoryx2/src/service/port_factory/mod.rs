// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

/// Factory to create the endpoints of
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) based
/// communication and to acquire static and dynamic service information
pub mod event;

/// Factory to create a [`crate::port::listener::Listener`]
pub mod listener;

/// Factory to create a [`crate::port::notifier::Notifier`]
pub mod notifier;

/// Factory to create the endpoints of
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication and to acquire static and dynamic service information
pub mod publish_subscribe;

/// Factory to create a [`crate::port::publisher::Publisher`]
pub mod publisher;

/// Factory to create a [`crate::port::subscriber::Subscriber`]
pub mod subscriber;
