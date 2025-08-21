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

use iceoryx2::prelude::*;
use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int, c_void};

mod active_request;
mod attribute;
mod attribute_set;
mod attribute_specifier;
mod attribute_verifier;
mod client;
mod client_details;
mod config;
mod event_id;
mod file_descriptor;
mod iceoryx2_settings;
mod listener;
mod listener_details;
mod log;
mod message_type_details;
mod node;
mod node_builder;
mod node_id;
mod node_name;
mod notifier;
mod notifier_details;
mod pending_response;
mod port_factory_client_builder;
mod port_factory_event;
mod port_factory_listener_builder;
mod port_factory_notifier_builder;
mod port_factory_pub_sub;
mod port_factory_publisher_builder;
mod port_factory_request_response;
mod port_factory_server_builder;
mod port_factory_subscriber_builder;
mod publish_subscribe_header;
mod publisher;
mod publisher_details;
mod quirks_correction;
mod request_header;
mod request_mut;
mod response;
mod response_header;
mod response_mut;
mod sample;
mod sample_mut;
mod server;
mod server_details;
mod service;
mod service_builder;
mod service_builder_event;
mod service_builder_pub_sub;
mod service_builder_request_response;
mod service_name;
mod signal_handling_mode;
mod static_config;
mod static_config_event;
mod static_config_publish_subscribe;
mod static_config_request_response;
mod subscriber;
mod subscriber_details;
mod unique_client_id;
mod unique_listener_id;
mod unique_notifier_id;
mod unique_publisher_id;
mod unique_server_id;
mod unique_subscriber_id;
mod waitset;
mod waitset_attachment_id;
mod waitset_builder;
mod waitset_guard;

pub use active_request::*;
pub use attribute::*;
pub use attribute_set::*;
pub use attribute_specifier::*;
pub use attribute_verifier::*;
pub use client::*;
pub use client_details::*;
pub use config::*;
pub use event_id::*;
pub use file_descriptor::*;
pub use iceoryx2_settings::*;
pub use listener::*;
pub use listener_details::*;
pub use message_type_details::*;
pub use node::*;
pub use node_builder::*;
pub use node_id::*;
pub use node_name::*;
pub use notifier::*;
pub use notifier_details::*;
pub use pending_response::*;
pub use port_factory_client_builder::*;
pub use port_factory_event::*;
pub use port_factory_listener_builder::*;
pub use port_factory_notifier_builder::*;
pub use port_factory_pub_sub::*;
pub use port_factory_publisher_builder::*;
pub use port_factory_request_response::*;
pub use port_factory_server_builder::*;
pub use port_factory_subscriber_builder::*;
pub use publish_subscribe_header::*;
pub use publisher::*;
pub use publisher_details::*;
pub use quirks_correction::*;
pub use request_header::*;
pub use request_mut::*;
pub use response::*;
pub use response_header::*;
pub use response_mut::*;
pub use sample::*;
pub use sample_mut::*;
pub use server::*;
pub use server_details::*;
pub use service::*;
pub use service_builder::*;
pub use service_builder_event::*;
pub use service_builder_pub_sub::*;
pub use service_builder_request_response::*;
pub use service_name::*;
pub use signal_handling_mode::*;
pub use static_config::*;
pub use static_config_event::*;
pub use static_config_publish_subscribe::*;
pub use static_config_request_response::*;
pub use subscriber::*;
pub use subscriber_details::*;
pub use unique_client_id::*;
pub use unique_listener_id::*;
pub use unique_notifier_id::*;
pub use unique_publisher_id::*;
pub use unique_server_id::*;
pub use unique_subscriber_id::*;
pub use waitset::*;
pub use waitset_attachment_id::*;
pub use waitset_builder::*;
pub use waitset_guard::*;

/// This constant signals an successful function call
pub const IOX2_OK: c_int = 0;

/// An alias to a `void *` which can be used to pass arbitrary data to the callback
pub type iox2_callback_context = *mut c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_callback_progression_e {
    STOP = 0,
    CONTINUE,
}

impl From<iox2_callback_progression_e> for CallbackProgression {
    fn from(value: iox2_callback_progression_e) -> Self {
        match value {
            iox2_callback_progression_e::STOP => CallbackProgression::Stop,
            iox2_callback_progression_e::CONTINUE => CallbackProgression::Continue,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_semantic_string_error_e {
    INVALID_CONTENT = IOX2_OK as isize + 1,
    EXCEEDS_MAXIMUM_LENGTH,
}

impl IntoCInt for SemanticStringError {
    fn into_c_int(self) -> c_int {
        (match self {
            SemanticStringError::InvalidContent => iox2_semantic_string_error_e::INVALID_CONTENT,
            SemanticStringError::ExceedsMaximumLength => {
                iox2_semantic_string_error_e::EXCEEDS_MAXIMUM_LENGTH
            }
        }) as c_int
    }
}

/// This is a trait to convert a Rust error enum into the corresponding C error enum and then to a c_int in one go
///
/// # Example
///
/// ```no_run
/// use core::ffi::c_int;
/// use iceoryx2_ffi_c::IOX2_OK;
///
/// trait IntoCInt {
///     fn into_c_int(self) -> c_int;
/// }
///
/// enum FooError {
///     BAR,
///     BAZ
/// }
///
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// pub enum iox2_foo_error_e {
///     BAR = IOX2_OK as isize + 1, // start `IOX2_OK + 1` to prevent ambiguous values
///     BAZ,
/// }
///
/// impl IntoCInt for FooError {
///     fn into_c_int(self) -> c_int {
///         (match self {
///             FooError::BAR => iox2_foo_error_e::BAR,
///             FooError::BAZ => iox2_foo_error_e::BAZ,
///         }) as c_int
///     }
/// }
/// ```
trait IntoCInt {
    fn into_c_int(self) -> c_int;
}

trait HandleToType {
    type Target;

    // NOTE in this case, the handle `self` is already a `*mut`. Passing by value means a copy
    // of the pointer; passing by reference make the implementation more error prone since one
    // has to remember to de-reference `self` in order to get the `*mut`
    #[allow(clippy::wrong_self_convention)]
    fn as_type(self) -> Self::Target;
}

trait AssertNonNullHandle {
    fn assert_non_null(self);
}

/// Returns a string literal describing the provided [`iox2_semantic_string_error_e`].
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[no_mangle]
pub unsafe extern "C" fn iox2_semantic_string_error_string(
    error: iox2_semantic_string_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}
