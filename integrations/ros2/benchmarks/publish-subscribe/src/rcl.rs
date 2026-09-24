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

//! RAII wrappers around the `r2r_rcl` bindings exposing a node,
//! a publisher and a subscription of `std_msgs/msg/UInt8MultiArray`,
//! and a wait set to block on the subscription.

use core::ffi::{CStr, c_void};
use core::time::Duration;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use r2r_rcl::{
    RCL_RET_BAD_ALLOC, RCL_RET_OK, RCL_RET_SUBSCRIPTION_TAKE_FAILED, RCL_RET_TIMEOUT,
    rcl_context_fini, rcl_context_t, rcl_get_zero_initialized_context,
    rcl_get_zero_initialized_init_options, rcl_get_zero_initialized_node,
    rcl_get_zero_initialized_publisher, rcl_get_zero_initialized_subscription,
    rcl_get_zero_initialized_wait_set, rcl_init, rcl_init_options_fini, rcl_init_options_init,
    rcl_node_fini, rcl_node_get_default_options, rcl_node_init, rcl_node_t, rcl_publish,
    rcl_publisher_fini, rcl_publisher_get_default_options, rcl_publisher_get_subscription_count,
    rcl_publisher_init, rcl_publisher_t, rcl_ret_t, rcl_shutdown, rcl_subscription_fini,
    rcl_subscription_get_default_options, rcl_subscription_init, rcl_subscription_t, rcl_take,
    rcl_wait, rcl_wait_set_add_subscription, rcl_wait_set_clear, rcl_wait_set_fini,
    rcl_wait_set_init, rcl_wait_set_t, rcutils_get_default_allocator, rcutils_get_error_string,
    rcutils_reset_error, rmw_message_info_t, rmw_qos_profile_t, rosidl_message_type_support_t,
    rosidl_runtime_c__String, rosidl_runtime_c__uint8__Sequence,
    rosidl_runtime_c__uint8__Sequence__init,
};

/// rcl is initialized without forwarding any command-line arguments.
const NO_ARGS: core::ffi::c_int = 0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RclError {
    call: &'static str,
    ret: rcl_ret_t,
    message: String,
}

impl core::fmt::Display for RclError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} failed with rcl_ret_t {}: {}",
            self.call, self.ret, self.message
        )
    }
}

impl core::error::Error for RclError {}

fn check(call: &'static str, ret: rcl_ret_t) -> Result<(), RclError> {
    if ret != RCL_RET_OK as rcl_ret_t {
        let message = unsafe {
            let error = rcutils_get_error_string();
            rcutils_reset_error();
            CStr::from_ptr(error.str_.as_ptr())
                .to_string_lossy()
                .into_owned()
        };

        return Err(RclError { call, ret, message });
    }

    Ok(())
}

/// Maps the `bool` result of a rosidl message function onto an [`RclError`].
fn check_alloc(call: &'static str, ok: bool) -> Result<(), RclError> {
    if !ok {
        return Err(RclError {
            call,
            ret: RCL_RET_BAD_ALLOC as rcl_ret_t,
            message: "failed to allocate memory".into(),
        });
    }

    Ok(())
}

/// `std_msgs__msg__MultiArrayDimension`.
#[repr(C)]
struct MultiArrayDimension {
    label: rosidl_runtime_c__String,
    size: u32,
    stride: u32,
}

/// `std_msgs__msg__MultiArrayDimension__Sequence`.
#[repr(C)]
struct MultiArrayDimensionSequence {
    data: *mut MultiArrayDimension,
    size: usize,
    capacity: usize,
}

/// `std_msgs__msg__MultiArrayLayout`.
#[repr(C)]
struct MultiArrayLayout {
    dim: MultiArrayDimensionSequence,
    data_offset: u32,
}

/// `std_msgs__msg__UInt8MultiArray`, the rosidl C struct of the message.
#[repr(C)]
struct UInt8MultiArrayStruct {
    layout: MultiArrayLayout,
    data: rosidl_runtime_c__uint8__Sequence,
}

// Typesupport and generated message functions.
#[link(name = "std_msgs__rosidl_typesupport_c")]
unsafe extern "C" {
    fn rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__UInt8MultiArray()
    -> *const rosidl_message_type_support_t;
}

#[link(name = "std_msgs__rosidl_generator_c")]
unsafe extern "C" {
    fn std_msgs__msg__UInt8MultiArray__init(message: *mut UInt8MultiArrayStruct) -> bool;
    fn std_msgs__msg__UInt8MultiArray__fini(message: *mut UInt8MultiArrayStruct);
}

fn type_support() -> *const rosidl_message_type_support_t {
    unsafe {
        rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__UInt8MultiArray()
    }
}

/// An initialized `std_msgs/msg/UInt8MultiArray` message.
pub struct UInt8MultiArray {
    message: UInt8MultiArrayStruct,
}

impl UInt8MultiArray {
    /// Creates an empty message.
    pub fn new() -> Result<Self, RclError> {
        let mut message: UInt8MultiArrayStruct = unsafe { core::mem::zeroed() };
        check_alloc("std_msgs__msg__UInt8MultiArray__init", unsafe {
            std_msgs__msg__UInt8MultiArray__init(&mut message)
        })?;

        Ok(Self { message })
    }

    /// Creates a message with `size` bytes of data.
    pub fn with_size(size: usize) -> Result<Self, RclError> {
        let mut message = Self::new()?;
        check_alloc("rosidl_runtime_c__uint8__Sequence__init", unsafe {
            rosidl_runtime_c__uint8__Sequence__init(&mut message.message.data, size)
        })?;

        Ok(message)
    }

    fn as_ptr(&self) -> *const c_void {
        (&self.message as *const UInt8MultiArrayStruct).cast()
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        (&mut self.message as *mut UInt8MultiArrayStruct).cast()
    }
}

impl Drop for UInt8MultiArray {
    fn drop(&mut self) {
        unsafe { std_msgs__msg__UInt8MultiArray__fini(&mut self.message) };
    }
}

/// An initialized rcl context. It is shared by all nodes of the benchmark.
#[derive(Debug)]
pub struct RclContext {
    context: Box<UnsafeCell<rcl_context_t>>,
    lock: Mutex<()>,
}

// SAFETY: every rcl call taking the context goes through `RclContext::locked`,
// except `rcl_shutdown` and `rcl_context_fini` on drop, which have exclusive
// access.
unsafe impl Send for RclContext {}
unsafe impl Sync for RclContext {}

impl RclContext {
    pub fn create() -> Result<Arc<Self>, RclError> {
        unsafe {
            let mut init_options = rcl_get_zero_initialized_init_options();
            check(
                "rcl_init_options_init",
                rcl_init_options_init(&mut init_options, rcutils_get_default_allocator()),
            )?;

            let context = Box::new(UnsafeCell::new(rcl_get_zero_initialized_context()));
            let ret = rcl_init(NO_ARGS, core::ptr::null(), &init_options, context.get());
            let _ = rcl_init_options_fini(&mut init_options);
            check("rcl_init", ret)?;

            Ok(Arc::new(Self {
                context,
                lock: Mutex::new(()),
            }))
        }
    }

    /// Calls `f` with the context while holding its lock.
    fn locked<R>(&self, f: impl FnOnce(*mut rcl_context_t) -> R) -> R {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        f(self.context.get())
    }
}

impl Drop for RclContext {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_shutdown(self.context.get());
            let _ = rcl_context_fini(self.context.get());
        }
    }
}

/// An rcl node in a shared [`RclContext`].
#[derive(Debug)]
pub struct RclNode {
    node: Box<UnsafeCell<rcl_node_t>>,
    context: Arc<RclContext>,
}

impl RclNode {
    pub fn create(
        context: &Arc<RclContext>,
        name: &CStr,
        namespace: &CStr,
    ) -> Result<Rc<Self>, RclError> {
        unsafe {
            let node = Box::new(UnsafeCell::new(rcl_get_zero_initialized_node()));
            let node_options = rcl_node_get_default_options();
            check(
                "rcl_node_init",
                context.locked(|context| {
                    rcl_node_init(
                        node.get(),
                        name.as_ptr(),
                        namespace.as_ptr(),
                        context,
                        &node_options,
                    )
                }),
            )?;

            Ok(Rc::new(Self {
                node,
                context: context.clone(),
            }))
        }
    }

    fn handle(&self) -> *mut rcl_node_t {
        self.node.get()
    }
}

impl Drop for RclNode {
    fn drop(&mut self) {
        self.context.locked(|_| unsafe {
            let _ = rcl_node_fini(self.node.get());
        });
    }
}

/// Publishes `std_msgs/msg/UInt8MultiArray` messages on a ROS 2 topic.
#[derive(Debug)]
pub struct RclPublisher {
    node: Rc<RclNode>,
    publisher: Box<UnsafeCell<rcl_publisher_t>>,
}

impl RclPublisher {
    pub fn create(
        node: &Rc<RclNode>,
        topic: &CStr,
        qos: rmw_qos_profile_t,
    ) -> Result<Self, RclError> {
        unsafe {
            let publisher = Box::new(UnsafeCell::new(rcl_get_zero_initialized_publisher()));
            let mut options = rcl_publisher_get_default_options();
            options.qos = qos;

            check(
                "rcl_publisher_init",
                rcl_publisher_init(
                    publisher.get(),
                    node.handle(),
                    type_support(),
                    topic.as_ptr(),
                    &options,
                ),
            )?;

            Ok(Self {
                node: node.clone(),
                publisher,
            })
        }
    }

    pub fn publish(&self, message: &UInt8MultiArray) -> Result<(), RclError> {
        check("rcl_publish", unsafe {
            rcl_publish(
                self.publisher.get(),
                message.as_ptr(),
                core::ptr::null_mut(),
            )
        })
    }

    /// The number of subscriptions matched with the publisher.
    pub fn subscription_count(&self) -> Result<usize, RclError> {
        let mut count = 0;
        check("rcl_publisher_get_subscription_count", unsafe {
            rcl_publisher_get_subscription_count(self.publisher.get(), &mut count)
        })?;

        Ok(count)
    }
}

impl Drop for RclPublisher {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_publisher_fini(self.publisher.get(), self.node.handle());
        }
    }
}

/// Receives `std_msgs/msg/UInt8MultiArray` messages from a ROS 2 topic.
#[derive(Debug)]
pub struct RclSubscription {
    node: Rc<RclNode>,
    subscription: Box<UnsafeCell<rcl_subscription_t>>,
}

impl RclSubscription {
    pub fn create(
        node: &Rc<RclNode>,
        topic: &CStr,
        qos: rmw_qos_profile_t,
    ) -> Result<Self, RclError> {
        unsafe {
            let subscription = Box::new(UnsafeCell::new(rcl_get_zero_initialized_subscription()));
            let mut options = rcl_subscription_get_default_options();
            options.qos = qos;

            check(
                "rcl_subscription_init",
                rcl_subscription_init(
                    subscription.get(),
                    node.handle(),
                    type_support(),
                    topic.as_ptr(),
                    &options,
                ),
            )?;

            Ok(Self {
                node: node.clone(),
                subscription,
            })
        }
    }

    /// Takes the next message into `message`. Returns `false` when the queue
    /// is empty.
    pub fn take(&self, message: &mut UInt8MultiArray) -> Result<bool, RclError> {
        let mut message_info = rmw_message_info_t::default();
        let ret = unsafe {
            rcl_take(
                self.subscription.get(),
                message.as_mut_ptr(),
                &mut message_info,
                core::ptr::null_mut(),
            )
        };
        if ret == RCL_RET_SUBSCRIPTION_TAKE_FAILED as rcl_ret_t {
            return Ok(false);
        }
        check("rcl_take", ret)?;

        Ok(true)
    }
}

impl Drop for RclSubscription {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_subscription_fini(self.subscription.get(), self.node.handle());
        }
    }
}

/// A wait set holding a single subscription, created once and reused for
/// every wait.
#[derive(Debug)]
pub struct RclWaitSet {
    wait_set: Box<UnsafeCell<rcl_wait_set_t>>,
    context: Arc<RclContext>,
}

impl RclWaitSet {
    pub fn create(context: &Arc<RclContext>) -> Result<Self, RclError> {
        unsafe {
            let wait_set = Box::new(UnsafeCell::new(rcl_get_zero_initialized_wait_set()));
            check(
                "rcl_wait_set_init",
                context.locked(|context| {
                    rcl_wait_set_init(
                        wait_set.get(),
                        1,
                        0,
                        0,
                        0,
                        0,
                        0,
                        context,
                        rcutils_get_default_allocator(),
                    )
                }),
            )?;

            Ok(Self {
                wait_set,
                context: context.clone(),
            })
        }
    }

    /// Blocks until `subscription` has data or `timeout` passed. Returns
    /// `false` on timeout.
    pub fn wait(
        &self,
        subscription: &RclSubscription,
        timeout: Duration,
    ) -> Result<bool, RclError> {
        unsafe {
            check(
                "rcl_wait_set_clear",
                rcl_wait_set_clear(self.wait_set.get()),
            )?;
            check(
                "rcl_wait_set_add_subscription",
                rcl_wait_set_add_subscription(
                    self.wait_set.get(),
                    subscription.subscription.get(),
                    core::ptr::null_mut(),
                ),
            )?;

            let ret = rcl_wait(self.wait_set.get(), timeout.as_nanos() as i64);
            if ret == RCL_RET_TIMEOUT as rcl_ret_t {
                return Ok(false);
            }
            check("rcl_wait", ret)?;

            Ok(true)
        }
    }
}

impl Drop for RclWaitSet {
    fn drop(&mut self) {
        self.context.locked(|_| unsafe {
            let _ = rcl_wait_set_fini(self.wait_set.get());
        });
    }
}
