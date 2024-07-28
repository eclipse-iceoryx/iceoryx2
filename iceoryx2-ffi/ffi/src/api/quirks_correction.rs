// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#![allow(non_camel_case_types)]

// TODO: c_size_t is currently only available in nightly and defined like:
pub type c_size_t = usize;

use crate::{
    iox2_event_open_or_create_error_e, iox2_listener_create_error_e, iox2_listener_wait_error_e,
    iox2_node_creation_failure_e, iox2_node_event_e, iox2_node_list_failure_e,
    iox2_notifier_create_error_e, iox2_notifier_notify_error_e,
    iox2_pub_sub_open_or_create_error_e, iox2_publisher_create_error_e,
    iox2_publisher_loan_error_e, iox2_publisher_send_error_e, iox2_semantic_string_error_e,
    iox2_service_details_error_e, iox2_service_list_error_e, iox2_subscriber_create_error_e,
    iox2_subscriber_receive_error_e, iox2_type_detail_error_e,
};

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_semantic_string_error_stub() -> iox2_semantic_string_error_e
{
    iox2_semantic_string_error_e::INVALID_CONTENT
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_node_creation_failure_stub() -> iox2_node_creation_failure_e
{
    iox2_node_creation_failure_e::INTERNAL_ERROR
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_node_list_failure_stub() -> iox2_node_list_failure_e {
    iox2_node_list_failure_e::INTERNAL_ERROR
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_node_event_stub() -> iox2_node_event_e {
    iox2_node_event_e::TICK
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_service_details_error_stub() -> iox2_service_details_error_e
{
    iox2_service_details_error_e::INTERNAL_ERROR
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_event_open_or_create_error_stub(
) -> iox2_event_open_or_create_error_e {
    iox2_event_open_or_create_error_e::O_INTERNAL_FAILURE
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_pub_sub_open_or_create_error_stub(
) -> iox2_pub_sub_open_or_create_error_e {
    iox2_pub_sub_open_or_create_error_e::O_INTERNAL_FAILURE
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_notifier_create_error_stub() -> iox2_notifier_create_error_e
{
    iox2_notifier_create_error_e::EXCEEDS_MAX_SUPPORTED_NOTIFIERS
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_type_detail_error_stub() -> iox2_type_detail_error_e {
    iox2_type_detail_error_e::INVALID_TYPE_NAME
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_listener_create_error_stub() -> iox2_listener_create_error_e
{
    iox2_listener_create_error_e::EXCEEDS_MAX_SUPPORTED_LISTENERS
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_notifier_notify_error_stub() -> iox2_notifier_notify_error_e
{
    iox2_notifier_notify_error_e::EVENT_ID_OUT_OF_BOUNDS
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_listener_wait_error_stub() -> iox2_listener_wait_error_e {
    iox2_listener_wait_error_e::INTERNAL_FAILURE
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_publisher_create_error_stub(
) -> iox2_publisher_create_error_e {
    iox2_publisher_create_error_e::EXCEEDS_MAX_SUPPORTED_PUBLISHERS
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_subscriber_create_error_stub(
) -> iox2_subscriber_create_error_e {
    iox2_subscriber_create_error_e::EXCEEDS_MAX_SUPPORTED_SUBSCRIBERS
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_publisher_send_error_stub() -> iox2_publisher_send_error_e
{
    iox2_publisher_send_error_e::CONNECTION_ERROR
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_publisher_loan_error_stub() -> iox2_publisher_loan_error_e
{
    iox2_publisher_loan_error_e::INTERNAL_FAILURE
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_subscriber_receive_error_stub(
) -> iox2_subscriber_receive_error_e {
    iox2_subscriber_receive_error_e::CONNECTION_FAILURE
}

#[doc(hidden)]
#[no_mangle]
// TODO: enums are only exported when they are actually used by some function
pub unsafe extern "C" fn __iox2_internal_service_list_error_stub() -> iox2_service_list_error_e {
    iox2_service_list_error_e::INTERNAL_ERROR
}
