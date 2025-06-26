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
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .request_response::<u64, u64>()
//! #     .open_or_create()?;
//! #
//! # let client = service.client_builder().create()?;
//! # let server = service.server_builder().create()?;
//! # let pending_response = client.send_copy(0)?;
//! # let active_request = server.receive()?.unwrap();
//!
//! let mut response = active_request.loan_uninit()?;
//! // write 1234 into sample
//! response.payload_mut().write(1234);
//! // overwrite contents with 456 because its fun
//! let response = response.write_payload(456);
//!
//! println!("server id: {:?}", response.header().server_id());
//! response.send()?;
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::{port::server::SharedServerState, response_mut::ResponseMut, service};
use core::{fmt::Debug, mem::MaybeUninit};

/// Acquired by a [`ActiveRequest`](crate::active_request::ActiveRequest) with
///  * [`ActiveRequest::loan_uninit()`](crate::active_request::ActiveRequest::loan_uninit())
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) of the
/// [`Client`](crate::port::client::Client).
///
/// If the [`ResponseMutUninit`] is not sent it will reelase the loaned memory when going out of
/// scope.
///
/// The generic parameter `Payload` is actually [`core::mem::MaybeUninit<Payload>`].
pub struct ResponseMutUninit<
    Service: service::Service,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) response: ResponseMut<Service, ResponsePayload, ResponseHeader>,
}

unsafe impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Debug for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResponseMut {{ response: {:?} }}", self.response)
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let response = active_request.loan_uninit()?;
    ///
    /// println!("server id: {:?}", response.header().server_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.response.header()
    }

    /// Returns a reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever2".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_user_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// // initializes the user header with default, therefore it is okay to access
    /// // it without assigning something first
    /// let mut response = active_request.loan_uninit()?;
    /// println!("user header {}", response.user_header());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header(&self) -> &ResponseHeader {
        self.response.user_header()
    }

    /// Returns a mutable reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_user_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// *response.user_header_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header_mut(&mut self) -> &mut ResponseHeader {
        self.response.user_header_mut()
    }

    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever3".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(123);
    /// println!("payload: {:?}", *response.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &ResponsePayload {
        self.response.payload()
    }

    /// Returns a mutable reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever4".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(123);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        self.response.payload_mut()
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    > ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>
{
    /// Writes the provided payload into the [`ResponseMutUninit`] and returns an initialized
    /// [`ResponseMut`] that is ready to be sent.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever5".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// let response = response.write_payload(123);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_payload(
        mut self,
        value: ResponsePayload,
    ) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// Converts the [`ResponseMutUninit`] into [`ResponseMut`]. This shall be done after the
    /// payload was written into the [`ResponseMutUninit`].
    ///
    /// # Safety
    ///
    ///  * Must ensure that the payload was properly initialized.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(789);
    /// // this is fine since the payload was initialized to 789
    /// let response = unsafe { response.assume_init() };
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        let initialized_response = core::mem::transmute_copy(&self.response);
        core::mem::forget(self);
        initialized_response
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    > ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>
{
    /// Converts the [`ResponseMutUninit`] into [`ResponseMut`]. This shall be done after the
    /// payload was written into the [`ResponseMutUninit`].
    ///
    /// # Safety
    ///
    ///  * Must ensure that the payload was properly initialized.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// for element in response.payload_mut() {
    ///     element.write(1234);
    /// }
    /// // this is fine since the payload was initialized to 789
    /// let response = unsafe { response.assume_init() };
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        let initialized_response = core::mem::transmute_copy(&self.response);
        core::mem::forget(self);
        initialized_response
    }

    /// Writes the payload to the [`ResponseMutUninit`] and labels the [`ResponseMutUninit`] as
    /// initialized
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// let response = response.write_from_fn(|index| index * 2 + 3);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> ResponsePayload>(
        mut self,
        mut initializer: F,
    ) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + Copy + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    > ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>
{
    /// Writes the payload by mem copying the provided slice into the [`ResponseMutUninit`].
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 4;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// let response = response.write_from_slice(&vec![1, 2, 3, 4]);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_slice(
        mut self,
        value: &[ResponsePayload],
    ) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        self.payload_mut().copy_from_slice(unsafe {
            core::mem::transmute::<&[ResponsePayload], &[MaybeUninit<ResponsePayload>]>(value)
        });
        unsafe { self.assume_init() }
    }
}
