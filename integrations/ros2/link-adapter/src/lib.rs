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

#![deny(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod endpoint_description;
#[allow(unsafe_code)]
pub mod endpoints;
pub mod mapping;
pub mod qos;
pub mod ros_header;
#[allow(unsafe_code)]
pub mod translator;

#[allow(unsafe_code)]
pub(crate) mod rcl;
#[allow(unsafe_code)]
pub(crate) mod typesupport;

pub use adapter::Ros2Adapter;
pub use config::{Config, TopicName, TypeName};
pub use endpoint_description::{TopicDescription, TopicSettings, TopicTypes};
pub use endpoints::PublishSubscribeEndpoints;
pub use iceoryx2_link_backend::AllowList;
pub use mapping::{PrefixMapping, StaticMapping};
pub use qos::QosProfile;
pub use translator::{CdrTranscoder, PassthroughTranslator, PlainStructTranslator};

/// The name of the ROS 2 node representing the gateway.
#[allow(unsafe_code)]
const NODE_NAME: rcl::NodeName =
    unsafe { rcl::NodeName::from_c_str_static_unchecked(c"iceoryx2_gateway") };

/// The reason a string failed ROS 2 name validation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NameError {
    /// The name is empty where a name is required.
    Empty,
    /// A token is empty or contains characters outside `[A-Za-z0-9_]`, or
    /// starts with a digit.
    InvalidToken,
    /// A namespace does not start with `/`.
    NoLeadingSlash,
}

impl core::fmt::Display for NameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NameError::{self:?}")
    }
}

impl core::error::Error for NameError {}
