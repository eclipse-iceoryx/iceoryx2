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
use core::alloc::Layout;
use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error};
use std::ffi::c_void;
use std::rc::Rc;

use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    RMW_RET_OK, rcutils_get_default_allocator, rmw_deserialize, rmw_serialize,
    rmw_serialized_message_t, rosidl_runtime_c__message_initialization as Initialization,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

use crate::rcutils::LoanAllocator;
use crate::typesupport::{self, IntrospectionTypeSupport, LoadError, MessageTypeSupport};

/// The largest alignment a member of a rosidl C struct can have, that of
/// `long double`. The introspection states a struct's size but not its
/// alignment, so every message is aligned to this.
const MEMBER_ALIGNMENT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeserializeError {
    Deserialize,
}

impl core::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DeserializeError::{self:?}")
    }
}

impl core::error::Error for DeserializeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializeError {
    LoanDeclined,
    Serialize,
}

impl core::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SerializeError::{self:?}")
    }
}

impl core::error::Error for SerializeError {}

/// A ROS 2 message as the C struct rosidl generates for its type. The
/// struct is allocated and initialized from the type's introspection.
///
/// rcl publishes and takes topic messages as CDR bytes through
/// `rcl_publish_serialized_message` and `rcl_take_serialized_message`. No
/// equivalent functions exist for services. `rcl_send_request`,
/// `rcl_take_request_with_info` and their response counterparts require the
/// deserialized message struct.
/// Therefore, messages propagated on services must deserialized into or
/// serialize from this struct when crossing the boundary.
///
/// This incurs an additional copy when propagating service messages.
pub struct RosMessage {
    memory: *mut u8,
    layout: Layout,
    members: *const MessageMembers,
    /// The typesupport the rmw serializes and deserializes with.
    type_support: Rc<MessageTypeSupport>,
    /// Keeps the introspection library loaded while `members` is read.
    _introspection: Rc<IntrospectionTypeSupport>,
}

impl core::fmt::Debug for RosMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RosMessage")
            .field("memory", &self.memory)
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}

impl RosMessage {
    /// Allocates and initializes a message of `type_name`.
    ///
    /// Fails when the typesupport of the type cannot be loaded.
    pub(crate) fn new(type_name: &str) -> Result<Self, LoadError> {
        let origin = origin!("RosMessage::new");

        let type_support = fail!(
            from origin,
            when typesupport::get::<typesupport::MessageKind>(type_name),
            "Failed to load the typesupport of '{}'", type_name
        );
        let introspection = fail!(
            from origin,
            when typesupport::get::<typesupport::IntrospectionKind>(type_name),
            "Failed to load the introspection of '{}'", type_name
        );

        let members = introspection.members();
        // SAFETY: `members` points into the loaded library, kept alive by
        // `introspection`.
        let size = unsafe { (*members).size_of_ };
        let layout = Layout::from_size_align(size.max(1), MEMBER_ALIGNMENT)
            .expect("a rosidl struct fits a layout");
        // SAFETY: the layout has a non-zero size.
        let memory = unsafe { alloc_zeroed(layout) };
        if memory.is_null() {
            handle_alloc_error(layout);
        }

        let message = Self {
            memory,
            layout,
            members,
            type_support,
            _introspection: introspection,
        };
        message.init();

        Ok(message)
    }

    /// The message as rcl takes it.
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.memory.cast()
    }

    /// The message as rcl fills it.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut c_void {
        self.memory.cast()
    }

    /// Returns the message to its initial state, releasing whatever a fill
    /// allocated for its members.
    pub(crate) fn reset(&mut self) {
        self.fini();
        // SAFETY: `memory` is `layout.size()` bytes owned by this message.
        unsafe { core::ptr::write_bytes(self.memory, 0, self.layout.size()) };
        self.init();
    }

    /// Deserializes the CDR bytes `cdr` into the message.
    ///
    /// Fails when the rmw rejects the bytes for the type.
    pub(crate) fn deserialize(&mut self, cdr: &[u8]) -> Result<(), DeserializeError> {
        let origin = origin!("RosMessage::deserialize");

        let serialized = rmw_serialized_message_t {
            // The rmw reads the buffer through a const message, the pointer
            // type of the field is the only reason for the cast.
            buffer: cdr.as_ptr() as *mut u8,
            buffer_length: cdr.len(),
            buffer_capacity: cdr.len(),
            // SAFETY: the default allocator has no state.
            allocator: unsafe { rcutils_get_default_allocator() },
        };
        // SAFETY: `memory` is an initialized message of the typesupport's
        // type, and `serialized` is read for the duration of the call.
        let ret =
            unsafe { rmw_deserialize(&serialized, self.type_support.handle(), self.as_mut_ptr()) };
        if ret != RMW_RET_OK as i32 {
            fail!(
                from origin,
                with DeserializeError::Deserialize,
                "The rmw failed to deserialize a message"
            );
        }
        Ok(())
    }

    /// Serializes the message to CDR directly into the buffer `loan`
    /// provides for the serialized size.
    ///
    /// `loan` is called at most once, from within the rmw, and must not
    /// panic. Returns the number of bytes written.
    ///
    /// Fails when the loan is declined or the rmw cannot serialize the
    /// message.
    pub(crate) fn serialize_into<F>(&self, loan: F) -> Result<usize, SerializeError>
    where
        F: FnOnce(usize) -> Option<*mut u8>,
    {
        let origin = origin!("RosMessage::serialize_into");

        let mut loan = LoanAllocator::new(loan);
        let mut serialized = rmw_serialized_message_t {
            buffer: core::ptr::null_mut(),
            buffer_length: 0,
            buffer_capacity: 0,
            allocator: loan.allocator(),
        };
        // SAFETY: `memory` is an initialized message of the typesupport's
        // type, and the serialized message's allocator outlives the call.
        let ret =
            unsafe { rmw_serialize(self.as_ptr(), self.type_support.handle(), &mut serialized) };
        if ret != RMW_RET_OK as i32 {
            if loan.declined() {
                fail!(
                    from origin,
                    with SerializeError::LoanDeclined,
                    "Failed to serialize a message into the loaned buffer"
                );
            }
            fail!(
                from origin,
                with SerializeError::Serialize,
                "The rmw failed to serialize a message"
            );
        }
        Ok(serialized.buffer_length)
    }

    fn init(&self) {
        // SAFETY: `members` is valid while `_introspection` lives, and
        // `memory` is a zeroed allocation of the struct's size.
        if let Some(init) = unsafe { (*self.members).init_function } {
            unsafe {
                init(
                    self.memory.cast(),
                    Initialization::ROSIDL_RUNTIME_C_MSG_INIT_ALL,
                )
            };
        }
    }

    fn fini(&self) {
        // SAFETY: `members` is valid while `_introspection` lives, and
        // `memory` holds an initialized message.
        if let Some(fini) = unsafe { (*self.members).fini_function } {
            unsafe { fini(self.memory.cast()) };
        }
    }
}

impl Drop for RosMessage {
    fn drop(&mut self) {
        self.fini();
        // SAFETY: `memory` was allocated with `layout` in `new`.
        unsafe { dealloc(self.memory, self.layout) };
    }
}
