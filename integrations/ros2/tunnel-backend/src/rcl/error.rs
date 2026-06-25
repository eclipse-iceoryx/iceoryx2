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

//! Named representation of the `rcl_ret_t` error codes.

use r2r_rcl::{
    RCL_RET_ALREADY_INIT, RCL_RET_ALREADY_SHUTDOWN, RCL_RET_BAD_ALLOC, RCL_RET_CLIENT_INVALID,
    RCL_RET_CLIENT_TAKE_FAILED, RCL_RET_ERROR, RCL_RET_EVENT_INVALID, RCL_RET_EVENT_TAKE_FAILED,
    RCL_RET_INVALID_ARGUMENT, RCL_RET_INVALID_LOG_LEVEL_RULE, RCL_RET_INVALID_PARAM_RULE,
    RCL_RET_INVALID_REMAP_RULE, RCL_RET_INVALID_ROS_ARGS, RCL_RET_LIFECYCLE_STATE_NOT_REGISTERED,
    RCL_RET_LIFECYCLE_STATE_REGISTERED, RCL_RET_MISMATCHED_RMW_ID, RCL_RET_NODE_INVALID,
    RCL_RET_NODE_INVALID_NAME, RCL_RET_NODE_INVALID_NAMESPACE, RCL_RET_NODE_NAME_NON_EXISTENT,
    RCL_RET_NOT_FOUND, RCL_RET_NOT_INIT, RCL_RET_PUBLISHER_INVALID, RCL_RET_SERVICE_INVALID,
    RCL_RET_SERVICE_NAME_INVALID, RCL_RET_SERVICE_TAKE_FAILED, RCL_RET_SUBSCRIPTION_INVALID,
    RCL_RET_SUBSCRIPTION_TAKE_FAILED, RCL_RET_TIMEOUT, RCL_RET_TIMER_CANCELED,
    RCL_RET_TIMER_INVALID, RCL_RET_TOPIC_NAME_INVALID, RCL_RET_UNKNOWN_SUBSTITUTION,
    RCL_RET_UNSUPPORTED, RCL_RET_WAIT_SET_EMPTY, RCL_RET_WAIT_SET_FULL, RCL_RET_WAIT_SET_INVALID,
    RCL_RET_WRONG_LEXEME, rcl_ret_t,
};

/// A non-`OK` `rcl_ret_t`, decoded into its named meaning.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum RclError {
    Error,
    Timeout,
    Unsupported,
    BadAlloc,
    InvalidArgument,
    AlreadyInit,
    NotInit,
    MismatchedRmwId,
    TopicNameInvalid,
    ServiceNameInvalid,
    UnknownSubstitution,
    AlreadyShutdown,
    NotFound,
    NodeInvalid,
    NodeInvalidName,
    NodeInvalidNamespace,
    NodeNameNonExistent,
    PublisherInvalid,
    SubscriptionInvalid,
    SubscriptionTakeFailed,
    ClientInvalid,
    ClientTakeFailed,
    ServiceInvalid,
    ServiceTakeFailed,
    TimerInvalid,
    TimerCanceled,
    WaitSetInvalid,
    WaitSetEmpty,
    WaitSetFull,
    InvalidRemapRule,
    WrongLexeme,
    InvalidRosArgs,
    InvalidParamRule,
    InvalidLogLevelRule,
    EventInvalid,
    EventTakeFailed,
    LifecycleStateRegistered,
    LifecycleStateNotRegistered,
    /// A code this build does not know (e.g. introduced by a newer distro).
    Unknown(rcl_ret_t),
}

impl From<rcl_ret_t> for RclError {
    fn from(ret: rcl_ret_t) -> Self {
        match ret as u32 {
            RCL_RET_ERROR => Self::Error,
            RCL_RET_TIMEOUT => Self::Timeout,
            RCL_RET_UNSUPPORTED => Self::Unsupported,
            RCL_RET_BAD_ALLOC => Self::BadAlloc,
            RCL_RET_INVALID_ARGUMENT => Self::InvalidArgument,
            RCL_RET_ALREADY_INIT => Self::AlreadyInit,
            RCL_RET_NOT_INIT => Self::NotInit,
            RCL_RET_MISMATCHED_RMW_ID => Self::MismatchedRmwId,
            RCL_RET_TOPIC_NAME_INVALID => Self::TopicNameInvalid,
            RCL_RET_SERVICE_NAME_INVALID => Self::ServiceNameInvalid,
            RCL_RET_UNKNOWN_SUBSTITUTION => Self::UnknownSubstitution,
            RCL_RET_ALREADY_SHUTDOWN => Self::AlreadyShutdown,
            RCL_RET_NOT_FOUND => Self::NotFound,
            RCL_RET_NODE_INVALID => Self::NodeInvalid,
            RCL_RET_NODE_INVALID_NAME => Self::NodeInvalidName,
            RCL_RET_NODE_INVALID_NAMESPACE => Self::NodeInvalidNamespace,
            RCL_RET_NODE_NAME_NON_EXISTENT => Self::NodeNameNonExistent,
            RCL_RET_PUBLISHER_INVALID => Self::PublisherInvalid,
            RCL_RET_SUBSCRIPTION_INVALID => Self::SubscriptionInvalid,
            RCL_RET_SUBSCRIPTION_TAKE_FAILED => Self::SubscriptionTakeFailed,
            RCL_RET_CLIENT_INVALID => Self::ClientInvalid,
            RCL_RET_CLIENT_TAKE_FAILED => Self::ClientTakeFailed,
            RCL_RET_SERVICE_INVALID => Self::ServiceInvalid,
            RCL_RET_SERVICE_TAKE_FAILED => Self::ServiceTakeFailed,
            RCL_RET_TIMER_INVALID => Self::TimerInvalid,
            RCL_RET_TIMER_CANCELED => Self::TimerCanceled,
            RCL_RET_WAIT_SET_INVALID => Self::WaitSetInvalid,
            RCL_RET_WAIT_SET_EMPTY => Self::WaitSetEmpty,
            RCL_RET_WAIT_SET_FULL => Self::WaitSetFull,
            RCL_RET_INVALID_REMAP_RULE => Self::InvalidRemapRule,
            RCL_RET_WRONG_LEXEME => Self::WrongLexeme,
            RCL_RET_INVALID_ROS_ARGS => Self::InvalidRosArgs,
            RCL_RET_INVALID_PARAM_RULE => Self::InvalidParamRule,
            RCL_RET_INVALID_LOG_LEVEL_RULE => Self::InvalidLogLevelRule,
            RCL_RET_EVENT_INVALID => Self::EventInvalid,
            RCL_RET_EVENT_TAKE_FAILED => Self::EventTakeFailed,
            RCL_RET_LIFECYCLE_STATE_REGISTERED => Self::LifecycleStateRegistered,
            RCL_RET_LIFECYCLE_STATE_NOT_REGISTERED => Self::LifecycleStateNotRegistered,
            _ => Self::Unknown(ret),
        }
    }
}

impl core::fmt::Display for RclError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RclError::{self:?}")
    }
}

impl core::error::Error for RclError {}
