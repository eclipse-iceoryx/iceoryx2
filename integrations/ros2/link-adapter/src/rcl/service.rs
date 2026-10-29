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
    RCL_RET_OK, RCL_RET_SERVICE_TAKE_FAILED, rcl_get_zero_initialized_service, rcl_ret_t,
    rcl_send_response, rcl_service_fini, rcl_service_get_default_options, rcl_service_init,
    rcl_service_set_on_new_request_callback, rcl_service_t, rcl_take_request_with_info,
    rmw_request_id_t, rmw_service_info_t,
};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::{fail, origin};

use crate::qos::QosProfile;
use crate::rcl::node::RclNode;
use crate::rcl::{ParticipantPrefix, RclError, RequestId, ServiceInfo, ServiceName, qos};
use crate::ros_message::RosMessage;
use crate::typesupport::ServiceTypeSupport;

/// Invoked by the rmw with the number of newly arrived requests.
pub type NewRequestCallback = Box<dyn Fn(usize) + Send>;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    ServiceInit,
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
pub enum TakeError {
    Take,
}

impl core::fmt::Display for TakeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TakeError::{self:?}")
    }
}

impl core::error::Error for TakeError {}

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

/// Builder for [`RclService`].
#[derive(Debug)]
pub struct RclServiceBuilder<'a> {
    node: Rc<RclNode>,
    service: &'a ServiceName,
    type_support: Rc<ServiceTypeSupport>,
    qos: QosProfile,
}

impl<'a> RclServiceBuilder<'a> {
    /// Begins building a service server on `node` for the given service
    /// and typesupport.
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

    /// Sets the QoS profile of the service.
    pub fn qos(mut self, qos: QosProfile) -> Self {
        self.qos = qos;
        self
    }

    pub fn create(self) -> Result<RclService, CreationError> {
        let origin = origin!("RclServiceBuilder::create");

        unsafe {
            // The service with the builder's QoS over the default options.
            let service = Box::new(UnsafeCell::new(rcl_get_zero_initialized_service()));
            let mut options = rcl_service_get_default_options();
            qos::apply(&self.qos, &mut options.qos);

            let ret = rcl_service_init(
                service.get(),
                self.node.handle(),
                self.type_support.handle(),
                self.service.as_c_str().as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::ServiceInit,
                    "Failed to initialize service: {}",
                    RclError::from(ret)
                );
            }

            Ok(RclService {
                service,
                callback: None,
                node: self.node,
                _type_support: self.type_support,
            })
        }
    }
}

/// Takes requests from and sends responses to the clients of a ROS 2
/// service.
pub struct RclService {
    service: Box<UnsafeCell<rcl_service_t>>,
    /// The registered new-request callback, behind a box so that rcl holds a
    /// thin pointer to it as `user_data`.
    callback: Option<Box<NewRequestCallback>>,
    node: Rc<RclNode>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: Rc<ServiceTypeSupport>,
}

impl core::fmt::Debug for RclService {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RclService")
            .field("service", &self.service.get())
            .field("callback", &self.callback.is_some())
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl RclService {
    /// The participant of the node the server belongs to.
    pub fn participant_prefix(&self) -> &ParticipantPrefix {
        self.node.participant_prefix()
    }

    /// Registers `callback` for new requests arriving at the service.
    ///
    /// The rmw invokes it from a middleware thread with the number of new
    /// requests, so it must not panic. It replaces the callback registered
    /// before.
    pub fn on_new_request(&mut self, callback: NewRequestCallback) -> Result<(), CallbackError> {
        let origin = origin!("RclService::on_new_request");

        let callback = Box::new(callback);
        let user_data: *const NewRequestCallback = &*callback;

        let ret = unsafe {
            rcl_service_set_on_new_request_callback(
                self.service.get(),
                Some(on_new_request_trampoline),
                user_data.cast(),
            )
        };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(from origin,
                with CallbackError::Set,
                "Failed to set on-new-request callback: {}",
                RclError::from(ret)
            );
        }

        self.callback = Some(callback);

        Ok(())
    }

    /// Takes the next request into `request`.
    ///
    /// `request` must be of the service's request type, which rcl does not
    /// check. Returns the request's info, whose id addresses the response,
    /// or [`None`] when no request is pending.
    pub fn take_request(&self, request: &mut RosMessage) -> Result<Option<ServiceInfo>, TakeError> {
        let origin = origin!("RclService::take_request");

        // SAFETY: every field of the info is an integer, so zeroes are valid.
        let mut info: rmw_service_info_t = unsafe { core::mem::zeroed() };
        let ret = unsafe {
            rcl_take_request_with_info(self.service.get(), &mut info, request.as_mut_ptr())
        };
        if ret == RCL_RET_SERVICE_TAKE_FAILED as rcl_ret_t {
            return Ok(None);
        }
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with TakeError::Take,
                "Failed to take request: {}",
                RclError::from(ret)
            );
        }

        Ok(Some(ServiceInfo::from(&info)))
    }

    /// Sends `response` to the client that sent the request `request_id`.
    ///
    /// `response` must be of the service's response type, which rcl does not
    /// check.
    pub fn send_response(
        &self,
        request_id: &RequestId,
        response: &RosMessage,
    ) -> Result<(), SendError> {
        let origin = origin!("RclService::send_response");

        let mut header = rmw_request_id_t::from(request_id);
        // rcl reads the response through a mutable pointer. The pointer type
        // of the parameter is the only reason for the cast.
        let ret = unsafe {
            rcl_send_response(
                self.service.get(),
                &mut header,
                response.as_ptr() as *mut c_void,
            )
        };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with SendError::Send,
                "Failed to send response: {}",
                RclError::from(ret)
            );
        }

        Ok(())
    }
}

impl Drop for RclService {
    fn drop(&mut self) {
        unsafe {
            // Clear the callback before the closure is dropped. A middleware
            // thread must not invoke a freed closure.
            if self.callback.is_some() {
                let _ = rcl_service_set_on_new_request_callback(
                    self.service.get(),
                    None,
                    core::ptr::null(),
                );
            }

            let _ = rcl_service_fini(self.service.get(), self.node.handle());
        }
    }
}

/// The C entry point rcl invokes when requests arrive.
///
/// Recovers the [`NewRequestCallback`] from the `user_data` pointer registered in
/// [`RclService::on_new_request`] and calls it.
unsafe extern "C" fn on_new_request_trampoline(user_data: *const c_void, number_of_events: usize) {
    let callback = unsafe { &*(user_data as *const NewRequestCallback) };
    callback(number_of_events);
}
