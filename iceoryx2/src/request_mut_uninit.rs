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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! // write payload and acquire an initialized request that can be sent
//! let request = request.write_payload(55712);
//!
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::{port::client::ClientSharedState, request_mut::RequestMut, service};
use core::{fmt::Debug, mem::MaybeUninit};

/// A version of the [`RequestMut`] where the payload is not initialized which allows
/// true zero copy usage. To send a [`RequestMutUninit`] it must be first initialized
/// and converted into [`RequestMut`] with [`RequestMutUninit::assume_init()`].
#[repr(transparent)]
pub struct RequestMutUninit<
    Service: crate::service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) request:
        RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
}

unsafe impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Debug
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RequestMutUninit {{  request: {:?} }}", self.request)
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
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
        RequestPayload: Debug + ZeroCopySend + Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
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
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized.
    /// Sending the content when it is not fully initialized causes immediate undefined behavior.
    pub unsafe fn assume_init(
        self,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        let initialized_request = core::mem::transmute_copy(&self.request);
        core::mem::forget(self);
        initialized_request
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    >
    RequestMutUninit<
        Service,
        [MaybeUninit<RequestPayload>],
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// When the payload is manually populated by using
    /// [`RequestMutUninit::payload_mut()`], then this function can be used
    /// to convert it into the initialized [`RequestMut`] version.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// for element in request.payload_mut() {
    ///     element.write(1234);
    /// }
    /// // we have written the payload, initialize the request
    /// let request = unsafe { request.assume_init() };
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized.
    /// Sending the content when it is not fully initialized causes immediate undefined behavior.
    pub unsafe fn assume_init(
        self,
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        let initialized_request = core::mem::transmute_copy(&self.request);
        core::mem::forget(self);
        initialized_request
    }

    /// Writes the payload to the [`RequestMutUninit`] and labels the [`RequestMutUninit`] as
    /// initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[usize], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// let request = request.write_from_fn(|index| index + 123);
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> RequestPayload>(
        mut self,
        mut initializer: F,
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + Copy + ZeroCopySend,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    >
    RequestMutUninit<
        Service,
        [MaybeUninit<RequestPayload>],
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Writes the payload by mem copying the provided slice into the [`RequestMutUninit`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 3;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// let request = request.write_from_slice(&vec![1,2,3]);
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_slice(
        mut self,
        value: &[RequestPayload],
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        self.payload_mut().copy_from_slice(unsafe {
            core::mem::transmute::<&[RequestPayload], &[MaybeUninit<RequestPayload>]>(value)
        });
        unsafe { self.assume_init() }
    }
}
