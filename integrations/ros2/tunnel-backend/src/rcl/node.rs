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

use std::ffi::CStr;

use r2r_rcl::{
    RCL_RET_OK, rcl_context_fini, rcl_context_t, rcl_get_topic_names_and_types,
    rcl_get_zero_initialized_context, rcl_get_zero_initialized_init_options,
    rcl_get_zero_initialized_node, rcl_init, rcl_init_options_fini, rcl_init_options_init,
    rcl_names_and_types_fini, rcl_names_and_types_t, rcl_node_fini, rcl_node_get_default_options,
    rcl_node_init, rcl_node_t, rcl_ret_t, rcl_shutdown, rcutils_get_default_allocator,
    rcutils_string_array_t,
};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::{fail, warn};

use crate::rcl::{NodeName, NodeNamespace, RclError, TopicName, TypeName};

/// rcl is initialized without forwarding any command-line arguments.
const NO_ARGS: core::ffi::c_int = 0;

/// Topic and type names are demangled into their ROS form rather than left as
/// the underlying middleware names.
const NO_DEMANGLE: bool = false;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InitOptions,
    Context,
    Node,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum GraphError {
    Query,
}

impl core::fmt::Display for GraphError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GraphError::{self:?}")
    }
}

impl core::error::Error for GraphError {}

/// An rcl node together with the context it belongs to. The tunnel is a single
/// node, so it can be coupled to the context.
///
/// The node is shared: everything that needs it to stay alive - endpoints,
/// discovery, the backend itself - holds a share of one `Rc<Node>`.
#[derive(Debug)]
pub struct Node {
    node: Box<UnsafeCell<rcl_node_t>>,
    context: Box<UnsafeCell<rcl_context_t>>,
}

impl Node {
    /// Begins building a node with the given name. The namespace defaults to
    /// the root namespace unless set via [`Builder::namespace`].
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: NodeName) -> Builder {
        Builder::new(name)
    }

    pub(crate) fn handle(&self) -> *mut rcl_node_t {
        self.node.get()
    }

    /// Query the ROS graph for all topics visible to this node and their type names.
    pub fn topic_names_and_types(&self) -> Result<Vec<(TopicName, Vec<TypeName>)>, GraphError> {
        let origin = "Node::topic_names_and_types";

        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut rcl_names_and_types: rcl_names_and_types_t = core::mem::zeroed();
            let ret = rcl_get_topic_names_and_types(
                self.node.get(),
                &mut allocator,
                NO_DEMANGLE,
                &mut rcl_names_and_types,
            );
            if ret != RCL_RET_OK as i32 {
                fail!(
                    from origin,
                    with GraphError::Query,
                    "Failed to query topic names and types: {}",
                    RclError::from(ret)
                );
            }

            let mut names_and_types = Vec::with_capacity(rcl_names_and_types.names.size);
            for i in 0..rcl_names_and_types.names.size {
                let topic = TopicName::from_c_str_unchecked(cstr_at(&rcl_names_and_types.names, i));
                let types = collect_cstrs(&*rcl_names_and_types.types.add(i))
                    .into_iter()
                    .map(|type_name| TypeName::from_c_str_unchecked(type_name))
                    .collect();
                names_and_types.push((topic, types));
            }

            let _ = rcl_names_and_types_fini(&mut rcl_names_and_types);

            Ok(names_and_types)
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            let ret = rcl_node_fini(self.node.get());
            if ret != RCL_RET_OK as rcl_ret_t {
                warn!("Failed to finalize node: {}", RclError::from(ret));
            }

            let ret = rcl_shutdown(self.context.get());
            if ret != RCL_RET_OK as rcl_ret_t {
                warn!("Failed to shut down context: {}", RclError::from(ret));
            }

            let ret = rcl_context_fini(self.context.get());
            if ret != RCL_RET_OK as rcl_ret_t {
                warn!("Failed to finalize context: {}", RclError::from(ret));
            }
        }
    }
}

/// Builder for [`Node`].
#[derive(Debug)]
pub struct Builder {
    name: NodeName,
    namespace: NodeNamespace,
}

impl Builder {
    fn new(name: NodeName) -> Self {
        Self {
            name,
            namespace: NodeNamespace::root(),
        }
    }

    /// Sets the node's namespace. Defaults to the root namespace.
    pub fn namespace(mut self, namespace: NodeNamespace) -> Self {
        self.namespace = namespace;
        self
    }

    pub fn create(self) -> Result<Node, CreationError> {
        let origin = "Node::create";

        unsafe {
            let mut init_options = rcl_get_zero_initialized_init_options();
            let ret = rcl_init_options_init(&mut init_options, rcutils_get_default_allocator());
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::InitOptions,
                    "Failed to initialize init options: {}",
                    RclError::from(ret)
                );
            }

            let context = Box::new(UnsafeCell::new(rcl_get_zero_initialized_context()));
            let ret = rcl_init(NO_ARGS, core::ptr::null(), &init_options, context.get());
            let _ = rcl_init_options_fini(&mut init_options);
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::Context,
                    "Failed to initialize context: {}",
                    RclError::from(ret)
                );
            }

            let node = Box::new(UnsafeCell::new(rcl_get_zero_initialized_node()));
            let node_options = rcl_node_get_default_options();
            let ret = rcl_node_init(
                node.get(),
                self.name.as_c_str().as_ptr(),
                self.namespace.as_c_str().as_ptr(),
                context.get(),
                &node_options,
            );
            if ret != RCL_RET_OK as rcl_ret_t {
                let _ = rcl_shutdown(context.get());
                let _ = rcl_context_fini(context.get());
                fail!(
                    from origin,
                    with CreationError::Node,
                    "Failed to initialize node: {}",
                    RclError::from(ret)
                );
            }

            Ok(Node { node, context })
        }
    }
}

/// Borrows the string at `index` out of an rcutils string array.
///
/// # Safety
///
/// `index` must be within bounds and the array's entries must be valid
/// null-terminated strings (guaranteed by rcl for graph query results). The
/// returned reference is valid until the array is finalized.
unsafe fn cstr_at(array: &rcutils_string_array_t, index: usize) -> &CStr {
    unsafe { CStr::from_ptr(*array.data.add(index)) }
}

/// Borrows every string out of an rcutils string array into a Vec.
///
/// # Safety
///
/// The array's entries must be valid null-terminated strings (guaranteed by
/// rcl for graph query results). The returned references are valid until the
/// array is finalized.
unsafe fn collect_cstrs(array: &rcutils_string_array_t) -> Vec<&CStr> {
    (0..array.size)
        .map(|i| unsafe { cstr_at(array, i) })
        .collect()
}
