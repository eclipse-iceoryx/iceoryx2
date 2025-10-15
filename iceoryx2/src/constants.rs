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

/// Defines the maximum length of a [`Service`](crate::service::Service)
pub const MAX_SERVICE_NAME_LENGTH: usize = 255;

/// Defines how many [`Attribute`](crate::service::attribute::Attribute)s a
/// [`Service`](crate::service::Service) can have at most
pub const MAX_ATTRIBUTES: usize = 8;

/// Defines the maximum length of an [`AttributeKey`](crate::service::attribute::AttributeKey)
pub const MAX_ATTRIBUTE_KEY_LENGTH: usize = 64;

/// Defines the maximum length of an [`AttributeValue`](crate::service::attribute::AttributeValue)
pub const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 256;

/// Defines the maximum length of a [`NodeName`](crate::node::node_name::NodeName)
pub const MAX_NODE_NAME_LENGTH: usize = 128;

/// Defines the maximum length of a [`TypeName`](crate::service::static_config::message_type_details::TypeName)
pub const MAX_TYPE_NAME_LENGTH: usize = 256;

/// The maximum size the [`MessagingPattern::Blackboard`](crate::service::static_config::messaging_pattern::MessagingPattern::Blackboard)
/// supports for the keytype.
pub const MAX_BLACKBOARD_KEY_SIZE: usize = 64;

/// The maximum alignment the [`MessagingPattern::Blackboard`](crate::service::static_config::messaging_pattern::MessagingPattern::Blackboard)
/// supports for the keytype.
pub const MAX_BLACKBOARD_KEY_ALIGNMENT: usize = 8;
