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

//! Safe wrappers around the `r2r_rcl` bindings, covering what the gateway
//! needs from `rcl`.

#[allow(dead_code)] // the request-response endpoints follow
pub(crate) mod client;
pub(crate) mod error;
pub(crate) mod gid;
pub(crate) mod graph_listener;
pub(crate) mod message_info;
pub(crate) mod names;
pub(crate) mod node;
pub(crate) mod publisher;
pub(crate) mod qos;
#[allow(dead_code)] // the request-response endpoints follow
pub(crate) mod request_id;
#[allow(dead_code)] // the request-response endpoints follow
pub(crate) mod service;
#[allow(dead_code)] // the request-response endpoints follow
pub(crate) mod service_info;
pub(crate) mod subscription;

#[allow(unused_imports)] // the request-response endpoints follow
pub(crate) use client::{RclClient, RclClientBuilder};
pub(crate) use error::RclError;
pub(crate) use gid::{Gid, ParticipantPrefix};
pub(crate) use graph_listener::GraphListener;
pub(crate) use message_info::MessageInfo;
pub(crate) use names::*;
pub(crate) use node::{EndpointInfo, RclNode, RclNodeBuilder};
pub(crate) use publisher::{RclPublisher, RclPublisherBuilder};
pub(crate) use request_id::RequestId;
#[allow(unused_imports)] // the request-response endpoints follow
pub(crate) use service::{RclService, RclServiceBuilder};
pub(crate) use service_info::ServiceInfo;
pub(crate) use subscription::{RclSubscription, RclSubscriptionBuilder};
