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
use std::thread::JoinHandle;

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::{fail, origin, warn};
use r2r_rcl::{
    RCL_RET_OK, RCL_RET_TIMEOUT, rcl_get_zero_initialized_guard_condition,
    rcl_get_zero_initialized_wait_set, rcl_guard_condition_fini,
    rcl_guard_condition_get_default_options, rcl_guard_condition_init, rcl_guard_condition_t,
    rcl_ret_t, rcl_trigger_guard_condition, rcl_wait, rcl_wait_set_add_guard_condition,
    rcl_wait_set_clear, rcl_wait_set_fini, rcl_wait_set_init, rcl_wait_set_t,
    rcutils_get_default_allocator,
};

use crate::rcl::RclError;
use crate::rcl::node::RclNode;

/// The wait set's capacity per kind of waitable.
const GUARD_CONDITIONS: usize = 2;
const SUBSCRIPTIONS: usize = 0;
const TIMERS: usize = 0;
const CLIENTS: usize = 0;
const SERVICES: usize = 0;
const EVENTS: usize = 0;
/// Blocks until a guard condition triggers.
const NO_TIMEOUT: i64 = -1;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ListenError {
    GuardCondition,
    WaitSet,
    Thread,
}

impl core::fmt::Display for ListenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListenError::{self:?}")
    }
}

impl core::error::Error for ListenError {}

/// Listens for notifications on a node's graph guard condition to wake the
/// link.
pub struct GraphListener {
    /// Triggered to end the thread, finalized once it has joined.
    stop: Box<UnsafeCell<rcl_guard_condition_t>>,
    thread: Option<JoinHandle<()>>,
}

impl core::fmt::Debug for GraphListener {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GraphListener")
            .field("running", &self.thread.is_some())
            .finish_non_exhaustive()
    }
}

impl GraphListener {
    /// Spawns the graph listener on a separate thread to listen for graph changes.
    pub fn spawn(node: &RclNode, on_change: Box<dyn Fn() + Send>) -> Result<Self, ListenError> {
        let origin = origin!("GraphListener::spawn");

        let stop = fail!(
            from origin,
            when stop_guard_condition(node),
            "Failed to create the stop guard condition"
        );
        let guard_conditions = match GuardConditions::new(node, stop.get()) {
            Ok(guard_conditions) => guard_conditions,
            Err(error) => {
                unsafe {
                    let _ = rcl_guard_condition_fini(stop.get());
                }
                return Err(error);
            }
        };

        let thread = std::thread::Builder::new()
            .name("iox2-ros2-graph".into())
            .spawn(move || wait_for_changes(guard_conditions, on_change));
        let Ok(thread) = thread else {
            unsafe {
                let _ = rcl_guard_condition_fini(stop.get());
            }
            fail!(
                from origin,
                with ListenError::Thread,
                "Failed to spawn the graph listener thread"
            );
        };

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for GraphListener {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_trigger_guard_condition(self.stop.get());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        unsafe {
            let _ = rcl_guard_condition_fini(self.stop.get());
        }
    }
}

/// Which guard condition triggered a wait.
enum Triggered {
    Graph,
    Stop,
    Neither,
}

/// What the listener's thread waits on.
struct GuardConditions {
    waitset: rcl_wait_set_t,
    graph: *const rcl_guard_condition_t,
    stop: *const rcl_guard_condition_t,
}

// SAFETY: the thread alone uses the pointers, and both guard conditions
// outlive it, the node's owner drops the listener first and the listener
// finalizes the stop guard condition after the thread joined.
unsafe impl Send for GuardConditions {}

impl GuardConditions {
    /// Initializes a wait set with the `node`'s graph guard condition and
    /// the `stop` condition.
    fn new(node: &RclNode, stop: *const rcl_guard_condition_t) -> Result<Self, ListenError> {
        let origin = origin!("GuardConditions::new");

        let mut waitset = unsafe { rcl_get_zero_initialized_wait_set() };
        let ret = unsafe {
            rcl_wait_set_init(
                &mut waitset,
                SUBSCRIPTIONS,
                GUARD_CONDITIONS,
                TIMERS,
                CLIENTS,
                SERVICES,
                EVENTS,
                node.context(),
                rcutils_get_default_allocator(),
            )
        };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with ListenError::WaitSet,
                "Failed to initialize the wait set: {}",
                RclError::from(ret)
            );
        }
        Ok(Self {
            waitset,
            graph: node.graph_guard_condition(),
            stop,
        })
    }

    /// Waits for the next trigger.
    fn wait(&mut self) -> Result<Triggered, RclError> {
        let mut graph_index = 0;
        let mut stop_index = 0;
        let ret = unsafe {
            let _ = rcl_wait_set_clear(&mut self.waitset);
            let _ =
                rcl_wait_set_add_guard_condition(&mut self.waitset, self.graph, &mut graph_index);
            let _ = rcl_wait_set_add_guard_condition(&mut self.waitset, self.stop, &mut stop_index);
            rcl_wait(&mut self.waitset, NO_TIMEOUT)
        };
        if ret == RCL_RET_TIMEOUT as rcl_ret_t {
            return Ok(Triggered::Neither);
        }
        if ret != RCL_RET_OK as rcl_ret_t {
            return Err(RclError::from(ret));
        }
        if self.triggered(stop_index) {
            return Ok(Triggered::Stop);
        }
        if self.triggered(graph_index) {
            return Ok(Triggered::Graph);
        }
        Ok(Triggered::Neither)
    }

    /// Whether the guard condition at `index` triggered on the last wait.
    fn triggered(&self, index: usize) -> bool {
        unsafe { !(*self.waitset.guard_conditions.add(index)).is_null() }
    }
}

impl Drop for GuardConditions {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_wait_set_fini(&mut self.waitset);
        }
    }
}

/// Guard condition to stop the graph listener thread.
fn stop_guard_condition(
    node: &RclNode,
) -> Result<Box<UnsafeCell<rcl_guard_condition_t>>, ListenError> {
    let origin = origin!("GraphListener::stop_guard_condition");

    let stop = Box::new(UnsafeCell::new(unsafe {
        rcl_get_zero_initialized_guard_condition()
    }));
    let ret = unsafe {
        rcl_guard_condition_init(
            stop.get(),
            node.context(),
            rcl_guard_condition_get_default_options(),
        )
    };
    if ret != RCL_RET_OK as rcl_ret_t {
        fail!(
            from origin,
            with ListenError::GuardCondition,
            "Failed to initialize the stop guard condition: {}",
            RclError::from(ret)
        );
    }
    Ok(stop)
}

/// Listener thread loop.
fn wait_for_changes(mut guard_conditions: GuardConditions, on_change: Box<dyn Fn() + Send>) {
    let origin = origin!("GraphListener::wait_for_changes");

    loop {
        match guard_conditions.wait() {
            Ok(Triggered::Graph) => on_change(),
            Ok(Triggered::Neither) => {}
            Ok(Triggered::Stop) => break,
            Err(error) => {
                warn!(
                    from origin,
                    "Failed to wait on the graph, the listener ends: {}", error
                );
                break;
            }
        }
    }
}
