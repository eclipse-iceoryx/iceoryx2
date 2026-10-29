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

use std::ffi::c_void;
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_CLIENT_TAKE_FAILED, RCL_RET_OK, rcl_client_fini, rcl_client_get_default_options,
    rcl_client_init, rcl_client_set_on_new_response_callback, rcl_client_t,
    rcl_get_zero_initialized_client, rcl_ret_t, rcl_send_request, rcl_take_response_with_info,
    rmw_service_info_t,
};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::{fail, origin};

use crate::qos::QosProfile;
use crate::rcl::node::RclNode;
use crate::rcl::{RclError, ServiceInfo, ServiceName, qos};
use crate::ros_message::RosMessage;
use crate::typesupport::ServiceTypeSupport;

/// Invoked by the rmw with the number of newly arrived responses.
pub type NewResponseCallback = Box<dyn Fn(usize) + Send>;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    ClientInit,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CallbackError {
    Set,
}

impl core::fmt::Display for CallbackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CallbackError::{self:?}")
    }
}

impl core::error::Error for CallbackError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Send,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TakeError {
    Take,
}

impl core::fmt::Display for TakeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TakeError::{self:?}")
    }
}

impl core::error::Error for TakeError {}

/// Builder for [`RclClient`].
#[derive(Debug)]
pub struct RclClientBuilder<'a> {
    node: Rc<RclNode>,
    service: &'a ServiceName,
    type_support: Rc<ServiceTypeSupport>,
    qos: QosProfile,
}

impl<'a> RclClientBuilder<'a> {
    /// Begins building a client on `node` for the given service and
    /// typesupport.
    pub fn new(
        node: Rc<RclNode>,
        service: &'a ServiceName,
        type_support: Rc<ServiceTypeSupport>,
    ) -> Self {
        Self {
            node,
            service,
            type_support,
            qos: QosProfile::default(),
        }
    }

    /// Sets the QoS profile of the client.
    pub fn qos(mut self, qos: QosProfile) -> Self {
        self.qos = qos;
        self
    }

    pub fn create(self) -> Result<RclClient, CreationError> {
        let origin = origin!("RclClientBuilder::create");

        unsafe {
            // The client with the builder's QoS over the default options.
            let client = Box::new(UnsafeCell::new(rcl_get_zero_initialized_client()));
            let mut options = rcl_client_get_default_options();
            qos::apply(&self.qos, &mut options.qos);

            let ret = rcl_client_init(
                client.get(),
                self.node.handle(),
                self.type_support.handle(),
                self.service.as_c_str().as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::ClientInit,
                    "Failed to initialize client: {}",
                    RclError::from(ret)
                );
            }

            Ok(RclClient {
                client,
                callback: None,
                node: self.node,
                _type_support: self.type_support,
            })
        }
    }
}

/// Sends requests to and takes responses from a ROS 2 service.
pub struct RclClient {
    client: Box<UnsafeCell<rcl_client_t>>,
    /// The registered new-response callback, behind a box so that rcl holds a
    /// thin pointer to it as `user_data`.
    callback: Option<Box<NewResponseCallback>>,
    node: Rc<RclNode>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: Rc<ServiceTypeSupport>,
}

impl core::fmt::Debug for RclClient {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RclClient")
            .field("client", &self.client.get())
            .field("callback", &self.callback.is_some())
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl RclClient {
    /// Registers `callback` for new responses arriving at the client.
    ///
    /// The rmw invokes it from a middleware thread with the number of new
    /// responses, so it must not panic. It replaces the callback registered
    /// before.
    pub fn on_new_response(&mut self, callback: NewResponseCallback) -> Result<(), CallbackError> {
        let origin = origin!("RclClient::on_new_response");

        let callback = Box::new(callback);
        let user_data: *const NewResponseCallback = &*callback;

        let ret = unsafe {
            rcl_client_set_on_new_response_callback(
                self.client.get(),
                Some(on_new_response_trampoline),
                user_data.cast(),
            )
        };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(from origin,
                with CallbackError::Set,
                "Failed to set on-new-response callback: {}",
                RclError::from(ret)
            );
        }

        self.callback = Some(callback);
        Ok(())
    }

    /// Sends `request` to the service.
    ///
    /// `request` must be of the service's request type, which rcl does not
    /// check. Returns the sequence number rcl assigned to the request, which
    /// the response carries back in its [`ServiceInfo`].
    pub fn send_request(&self, request: &RosMessage) -> Result<i64, SendError> {
        let origin = origin!("RclClient::send_request");

        let mut sequence_number = 0;
        let ret =
            unsafe { rcl_send_request(self.client.get(), request.as_ptr(), &mut sequence_number) };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with SendError::Send,
                "Failed to send request: {}",
                RclError::from(ret)
            );
        }

        Ok(sequence_number)
    }

    /// Takes the next response into `response`.
    ///
    /// `response` must be of the service's response type, which rcl does not
    /// check. Returns the response's info, or [`None`] when no response is
    /// pending.
    pub fn take_response(
        &self,
        response: &mut RosMessage,
    ) -> Result<Option<ServiceInfo>, TakeError> {
        let origin = origin!("RclClient::take_response");

        // SAFETY: every field of the info is an integer, so zeroes are valid.
        let mut info: rmw_service_info_t = unsafe { core::mem::zeroed() };
        let ret = unsafe {
            rcl_take_response_with_info(self.client.get(), &mut info, response.as_mut_ptr())
        };
        if ret == RCL_RET_CLIENT_TAKE_FAILED as rcl_ret_t {
            return Ok(None);
        }
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with TakeError::Take,
                "Failed to take response: {}",
                RclError::from(ret)
            );
        }

        Ok(Some(ServiceInfo::from(&info)))
    }
}

impl Drop for RclClient {
    fn drop(&mut self) {
        unsafe {
            // Clear the callback before the closure is dropped. A middleware
            // thread must not invoke a freed closure.
            if self.callback.is_some() {
                let _ = rcl_client_set_on_new_response_callback(
                    self.client.get(),
                    None,
                    core::ptr::null(),
                );
            }

            let _ = rcl_client_fini(self.client.get(), self.node.handle());
        }
    }
}

/// The C entry point rcl invokes when responses arrive.
///
/// Recovers the [`NewResponseCallback`] from the `user_data` pointer registered in
/// [`RclClient::on_new_response`] and calls it.
unsafe extern "C" fn on_new_response_trampoline(user_data: *const c_void, number_of_events: usize) {
    let callback = unsafe { &*(user_data as *const NewResponseCallback) };
    callback(number_of_events);
}
