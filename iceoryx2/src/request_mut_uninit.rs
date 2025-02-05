// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node
//! #   .service_builder(&"My/Funk/ServiceName".try_into()?)
//! #   .request_response::<u64, u64>()
//! #   .open_or_create()?;
//! #
//! # let client = service.client_builder().create()?;
//! #
//!
//! // acquire uninitialized request
//! let request = client.loan_uninit()?;
//! // write payload and acquire an initialized request that can be send
//! let request = request.write_payload(counter);
//!
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{request_mut::RequestMut, service};
use core::{fmt::Debug, mem::MaybeUninit};

/// A version of the [`RequestMut`] where the payload is not initialized which allows
/// true zero copy usage. To send a [`RequestMutUninit`] it must be first initialized
/// and converted into [`RequestMut`] with [`RequestMutUninit::assume_init()`].
#[repr(transparent)]
pub struct RequestMutUninit<
    Service: crate::service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) request:
        RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > Debug
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RequestMutUninit<{}, {}, {}, {}, {}> {{ }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>()
        )
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`]
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.request.header()
    }

    /// Returns a reference to the user defined request header.
    pub fn user_header(&self) -> &RequestHeader {
        self.request.user_header()
    }

    /// Returns a mutable reference to the user defined request header.
    pub fn user_header_mut(&mut self) -> &mut RequestHeader {
        self.request.user_header_mut()
    }

    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &RequestPayload {
        self.request.payload()
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut RequestPayload {
        self.request.payload_mut()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    >
    RequestMutUninit<
        Service,
        MaybeUninit<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    pub fn write_payload(
        mut self,
        value: RequestPayload,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// When the payload is manually populated by using
    /// [`RequestMutUninit::payload_mut()`], then this function can be used
    /// to convert it into the initialized [`RequestMut`] version.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// let mut request = client.loan_uninit()?;
    /// // use the MaybeUninit API to initialize the payload
    /// request.payload_mut().write(8283);
    /// // we have written the payload, initialize the request
    /// let request = unsafe { request.assume_init() };
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(
        self,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        core::mem::transmute(self.request)
    }
}
