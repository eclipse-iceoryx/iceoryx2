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

use core::cell::UnsafeCell;
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, rcl_context_fini, rcl_context_t, rcl_get_zero_initialized_context,
    rcl_get_zero_initialized_init_options, rcl_get_zero_initialized_node, rcl_init,
    rcl_init_options_fini, rcl_init_options_init, rcl_node_fini, rcl_node_get_default_options,
    rcl_node_init, rcl_node_t, rcl_ret_t, rcl_shutdown, rcutils_get_default_allocator,
};

use iceoryx2_log::{fail, warn};

use crate::rcl::{NodeName, NodeNamespace, RclError, TopicName, publisher};
use crate::typesupport::TypeSupport;

/// rcl is initialized without forwarding any command-line arguments.
const NO_ARGS: core::ffi::c_int = 0;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InitOptionsInit(RclError),
    ContextInit(RclError),
    NodeInit(RclError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

/// An rcl node together with the context it belongs to. The tunnel is a single
/// node, so it can be coupled to the context.
#[derive(Debug)]
pub(crate) struct NodeInner {
    node: Box<UnsafeCell<rcl_node_t>>,
    context: Box<UnsafeCell<rcl_context_t>>,
}

impl NodeInner {
    pub(crate) fn handle(&self) -> *mut rcl_node_t {
        self.node.get()
    }
}

impl Drop for NodeInner {
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

/// A handle to an rcl node.
///
/// The node and its context stay alive until the last handle is dropped.
pub struct Node {
    inner: Rc<NodeInner>,
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Node")
            .field("node", &self.inner.node.get())
            .field("context", &self.inner.context.get())
            .finish()
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
                    with CreationError::InitOptionsInit(ret.into()),
                    "Failed to initialize init options"
                );
            }

            let context = Box::new(UnsafeCell::new(rcl_get_zero_initialized_context()));
            let ret = rcl_init(NO_ARGS, core::ptr::null(), &init_options, context.get());
            let _ = rcl_init_options_fini(&mut init_options);
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::ContextInit(ret.into()),
                    "Failed to initialize context"
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
                    with CreationError::NodeInit(ret.into()),
                    "Failed to initialize node"
                );
            }

            Ok(Node {
                inner: Rc::new(NodeInner { node, context }),
            })
        }
    }
}

impl Node {
    /// Begins building a node with the given name. The namespace defaults to
    /// the root namespace unless set via [`Builder::namespace`].
    pub fn new(name: NodeName) -> Builder {
        Builder::new(name)
    }

    /// Build a publisher on this node for the given topic and typesupport.
    pub fn publisher_builder(
        &self,
        topic: TopicName,
        type_support: TypeSupport,
    ) -> publisher::Builder {
        publisher::Builder::new(Rc::clone(&self.inner), topic, type_support)
    }

    pub(crate) fn handle(&self) -> *mut rcl_node_t {
        self.inner.handle()
    }
}
