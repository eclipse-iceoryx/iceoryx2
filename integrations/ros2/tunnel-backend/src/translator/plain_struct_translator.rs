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

//! Translation between rosidl C structs and their CDR wire representation,
//! driven by the type's introspection data loaded at runtime.

use std::rc::Rc;

use core::alloc::Layout;
use core::ffi::c_void;

use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{
    PayloadLayout, ResizableBuffer, Transcoder, Translation, Translator,
};
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, ServiceDescription,
};
use r2r_rcl::{
    RMW_RET_OK, rcutils_allocator_t, rcutils_get_default_allocator, rmw_deserialize, rmw_serialize,
    rmw_serialized_message_t, rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
    rosidl_typesupport_introspection_c_field_types as FieldType,
};

use crate::mapping::TopicDescription;
use crate::typesupport::{self, TypeSupport};

const FLOAT: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_FLOAT as u8;
const DOUBLE: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_DOUBLE as u8;
const CHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_CHAR as u8;
const WCHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_WCHAR as u8;
const BOOLEAN: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_BOOLEAN as u8;
const OCTET: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_OCTET as u8;
const UINT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT8 as u8;
const INT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT8 as u8;
const UINT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT16 as u8;
const INT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT16 as u8;
const UINT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT32 as u8;
const INT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT32 as u8;
const UINT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT64 as u8;
const INT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT64 as u8;
const MESSAGE: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_MESSAGE as u8;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TranslationError {
    /// The introspection library of the type's package failed to load.
    FailedToLoadIntrospectionLibrary,
    /// The typesupport library of the type's package failed to load.
    FailedToLoadTypeSupportLibrary,
    /// The type has pointer-backed (strings, sequences) or
    /// platform-defined (long double) members and cannot be translated.
    UnsupportedType,
    /// The service's declared fixed-size payload contradicts the layout
    /// introspected from the ROS 2 type.
    LayoutMismatch,
    /// The payload size does not match the resolved layout.
    UnexpectedPayloadSize,
    /// The destination buffer cannot hold the translated bytes.
    InsufficientCapacity,
    /// Failed to serialize the payload.
    Serialize,
    /// Failed to deserialize the wire bytes.
    Deserialize,
}

impl core::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranslationError::{self:?}")
    }
}

impl core::error::Error for TranslationError {}

/// A [`Translator`] for fixed-size ROS 2 message types.
///
/// The payload in shared memory is the type's rosidl C struct, which is
/// converted to and from CDR using the typesupport (de)serializer when
/// crossing wire boundaries.
///
/// Types introspected to have dynamically sized members (strings, sequences)
/// are rejected.
#[derive(Debug, Default)]
pub struct PlainStructTranslator;

impl Translator for PlainStructTranslator {
    type Error = TranslationError;
    type EndpointDescription = TopicDescription;
    type Transcoder = CdrTranscoder;

    fn create(
        &self,
        service_description: &ServiceDescription,
        topic_description: &TopicDescription,
    ) -> Result<Translation<Self::Transcoder>, Self::Error> {
        let origin = "PlainStructTranslator::create";
        let type_name = topic_description.type_name.as_str();

        let introspection = fail!(from origin,
            when typesupport::load_introspection(type_name),
            with TranslationError::FailedToLoadIntrospectionLibrary,
            "Failed to load introspection for type '{}'",
            type_name
        );

        let members = unsafe { (*introspection.handle()).data }.cast::<MessageMembers>();
        let layout = match unsafe { introspected_layout(members) } {
            IntrospectedLayout::FixedSize(layout) => layout,
            IntrospectedLayout::Unsupported => {
                fail!(from origin,
                    with TranslationError::UnsupportedType,
                    "ROS 2 type '{}' has dynamically sized or platform-defined members and cannot be translated",
                    type_name
                );
            }
        };

        // The type-name contract requires that a fixed-size local payload
        // claiming this ROS 2 type has the type's introspected layout.
        if let PatternDescription::PublishSubscribe(pattern_description) =
            &service_description.pattern
            && pattern_description.payload.variant == TypeVariant::FixedSize
            && (pattern_description.payload.size != layout.size()
                || pattern_description.payload.alignment != layout.align())
        {
            fail!(from origin,
                with TranslationError::LayoutMismatch,
                "Payload of service '{}' ({} bytes, align {}) does not match ROS 2 type '{}' ({} bytes, align {})",
                service_description.name,
                pattern_description.payload.size,
                pattern_description.payload.alignment,
                type_name,
                layout.size(),
                layout.align()
            );
        }

        let type_support = fail!(from origin,
            when typesupport::load(type_name),
            with TranslationError::FailedToLoadTypeSupportLibrary,
            "Failed to load typesupport for type '{}'",
            type_name
        );

        Ok(Translation::Transcode {
            payload_layout: PayloadLayout::FixedSize(layout),
            transcoder: CdrTranscoder {
                type_name: type_name.to_string(),
                type_support,
                layout,
            },
        })
    }
}

/// The [`Transcoder`] for a resolved fixed-size ROS 2 message type.
///
/// Transcodes the type's rosidl C struct to and from its CDR wire
/// representation using the typesupports (de)serializer function pointers.
#[derive(Debug)]
pub struct CdrTranscoder {
    /// ROS 2 type name, for diagnostics.
    type_name: String,
    /// Typesupport handle driving `rmw_serialize`/`rmw_deserialize`.
    type_support: Rc<TypeSupport>,
    /// Layout of the type's rosidl C struct.
    layout: Layout,
}

impl Transcoder for CdrTranscoder {
    type Error = TranslationError;

    fn to_wire(
        &self,
        payload: &[u8],
        wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        let origin = "CdrTranscoder::to_wire";

        if payload.len() != self.layout.size() {
            fail!(from origin,
                with TranslationError::UnexpectedPayloadSize,
                "Payload ({} bytes) does not match the resolved layout ({} bytes)",
                payload.len(),
                self.layout.size()
            );
        }
        debug_assert!(
            (payload.as_ptr() as usize).is_multiple_of(self.layout.align()),
            "the payload must be aligned to the resolved layout"
        );

        let mut serialized = rmw_serialized_message_t {
            buffer: core::ptr::null_mut(),
            buffer_length: 0,
            buffer_capacity: 0,
            allocator: buffer_allocator(wire),
        };
        let ret = unsafe {
            rmw_serialize(
                payload.as_ptr().cast::<c_void>(),
                self.type_support.handle(),
                &mut serialized,
            )
        };
        if ret != RMW_RET_OK as i32 {
            fail!(from origin,
                with TranslationError::Serialize,
                "Middleware failed to serialize payload of type '{}'",
                self.type_name
            );
        }

        Ok(serialized.buffer_length)
    }

    fn from_wire(
        &self,
        wire: &[u8],
        payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        let origin = "CdrTranscoder::from_wire";

        let destination = fail!(from origin,
            when payload.resize(self.layout.size()),
            with TranslationError::InsufficientCapacity,
            "Payload buffer cannot hold the deserialized type '{}' ({} bytes)",
            self.type_name,
            self.layout.size()
        );
        debug_assert!(
            (destination.as_ptr() as usize).is_multiple_of(self.layout.align()),
            "the payload buffer must be aligned to the resolved layout"
        );

        let serialized = rmw_serialized_message_t {
            buffer: wire.as_ptr().cast_mut(),
            buffer_length: wire.len(),
            buffer_capacity: wire.len(),
            allocator: unsafe { rcutils_get_default_allocator() },
        };
        let ret = unsafe {
            rmw_deserialize(
                &serialized,
                self.type_support.handle(),
                destination.as_mut_ptr().cast::<c_void>(),
            )
        };
        if ret != RMW_RET_OK as i32 {
            fail!(from origin,
                with TranslationError::Deserialize,
                "Middleware failed to deserialize wire bytes of type '{}'",
                self.type_name
            );
        }

        Ok(self.layout.size())
    }
}

/// The outcome of introspecting a message type's C struct layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum IntrospectedLayout {
    /// Every member is stored inline.
    FixedSize(Layout),
    /// A member is pointer-backed (strings, sequences) or has a
    /// platform-defined layout (long double) therefore not translatable.
    Unsupported,
}

/// Computes the layout of the rosidl C struct described by `members`.
///
/// # Safety
///
/// `members` must point to a valid introspection member table whose
/// library stays loaded for the duration of the call.
unsafe fn introspected_layout(members: *const MessageMembers) -> IntrospectedLayout {
    let table = unsafe { &*members };

    let mut alignment = 1;
    for index in 0..table.member_count_ as usize {
        let member = unsafe { &*table.members_.add(index) };

        if member.is_array_ && (member.array_size_ == 0 || member.is_upper_bound_) {
            return IntrospectedLayout::Unsupported;
        }

        let member_alignment = if member.type_id_ == MESSAGE {
            let nested = unsafe { (*member.members_).data }.cast::<MessageMembers>();
            match unsafe { introspected_layout(nested) } {
                IntrospectedLayout::FixedSize(layout) => layout.align(),
                IntrospectedLayout::Unsupported => return IntrospectedLayout::Unsupported,
            }
        } else {
            match basic_member_alignment(member.type_id_) {
                Some(member_alignment) => member_alignment,
                None => return IntrospectedLayout::Unsupported,
            }
        };
        alignment = alignment.max(member_alignment);
    }

    IntrospectedLayout::FixedSize(
        Layout::from_size_align(table.size_of_, alignment)
            .expect("introspected size and alignment form a valid layout"),
    )
}

/// Alignment of a basic (non-message) member type inside the rosidl C
/// struct.
///
/// `None` for dynamically sized (strings) or unsupported (long
/// double, with platform-defined alignment) member types.
fn basic_member_alignment(type_id: u8) -> Option<usize> {
    match type_id {
        BOOLEAN | OCTET | CHAR | UINT8 | INT8 => Some(1),
        WCHAR | UINT16 | INT16 => Some(2),
        FLOAT | UINT32 | INT32 => Some(4),
        DOUBLE | UINT64 | INT64 => Some(8),
        _ => None,
    }
}

/// An `rcutils` allocator over a [`ResizableBuffer`], letting the
/// middleware serialize directly into it. The buffer stays owned by the
/// caller; deallocation is a no-op. A buffer that cannot provide the
/// requested capacity surfaces as a null allocation.
fn buffer_allocator<B: ResizableBuffer>(buffer: &mut B) -> rcutils_allocator_t {
    unsafe extern "C" fn allocate<B: ResizableBuffer>(
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        let buffer = unsafe { &mut *(state as *mut B) };
        match buffer.resize(size) {
            Ok(region) => region.as_mut_ptr().cast::<c_void>(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe extern "C" fn reallocate<B: ResizableBuffer>(
        _pointer: *mut c_void,
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        // Resizing preserves written bytes (realloc semantics), regardless
        // of the pointer handed back.
        unsafe { allocate::<B>(size, state) }
    }

    unsafe extern "C" fn deallocate(_pointer: *mut c_void, _state: *mut c_void) {}

    unsafe extern "C" fn zero_allocate<B: ResizableBuffer>(
        number_of_elements: usize,
        size_of_element: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        let Some(size) = number_of_elements.checked_mul(size_of_element) else {
            return core::ptr::null_mut();
        };
        let buffer = unsafe { &mut *(state as *mut B) };
        let Ok(region) = buffer.resize(size) else {
            return core::ptr::null_mut();
        };
        let region = &mut region[..size];
        region.fill(0);
        region.as_mut_ptr().cast::<c_void>()
    }

    rcutils_allocator_t {
        allocate: Some(allocate::<B>),
        deallocate: Some(deallocate),
        reallocate: Some(reallocate::<B>),
        zero_allocate: Some(zero_allocate::<B>),
        state: (buffer as *mut B).cast::<c_void>(),
    }
}
