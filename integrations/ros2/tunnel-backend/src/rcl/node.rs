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

use std::ffi::CString;

use r2r_rcl::{
    RCL_RET_OK, rcl_context_fini, rcl_context_t, rcl_get_zero_initialized_context,
    rcl_get_zero_initialized_init_options, rcl_get_zero_initialized_node, rcl_init,
    rcl_init_options_fini, rcl_init_options_init, rcl_node_fini, rcl_node_get_default_options,
    rcl_node_init, rcl_node_t, rcl_shutdown, rcutils_get_default_allocator,
};

/// rcl is initialized without forwarding any command-line arguments.
const NO_ARGS: core::ffi::c_int = 0;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InvalidName,
    InitOptionsInit(i32),
    ContextInit(i32),
    NodeInit(i32),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

/// An rcl node together with the context it lives in.
#[derive(Debug)]
pub struct Node {
    node: *mut rcl_node_t,
    context: *mut rcl_context_t,
}

impl Node {
    pub fn create(name: &str, namespace: &str) -> Result<Self, CreationError> {
        let name = CString::new(name).map_err(|_| CreationError::InvalidName)?;
        let namespace = CString::new(namespace).map_err(|_| CreationError::InvalidName)?;

        unsafe {
            let mut init_options = rcl_get_zero_initialized_init_options();
            let ret = rcl_init_options_init(&mut init_options, rcutils_get_default_allocator());
            if ret != RCL_RET_OK as i32 {
                return Err(CreationError::InitOptionsInit(ret));
            }

            let mut context = Box::new(rcl_get_zero_initialized_context());
            let ret = rcl_init(NO_ARGS, core::ptr::null(), &init_options, context.as_mut());
            let _ = rcl_init_options_fini(&mut init_options);
            if ret != RCL_RET_OK as i32 {
                return Err(CreationError::ContextInit(ret));
            }

            let mut node = Box::new(rcl_get_zero_initialized_node());
            let node_options = rcl_node_get_default_options();
            let ret = rcl_node_init(
                node.as_mut(),
                name.as_ptr(),
                namespace.as_ptr(),
                context.as_mut(),
                &node_options,
            );
            if ret != RCL_RET_OK as i32 {
                let _ = rcl_shutdown(context.as_mut());
                let _ = rcl_context_fini(context.as_mut());
                return Err(CreationError::NodeInit(ret));
            }

            Ok(Self {
                node: Box::into_raw(node),
                context: Box::into_raw(context),
            })
        }
    }

    pub(crate) fn handle(&self) -> *mut rcl_node_t {
        self.node
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            let mut node = Box::from_raw(self.node);
            let _ = rcl_node_fini(node.as_mut());

            let mut context = Box::from_raw(self.context);
            let _ = rcl_shutdown(context.as_mut());
            let _ = rcl_context_fini(context.as_mut());
        }
    }
}
