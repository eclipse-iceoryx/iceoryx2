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
//! let response = active_request.loan_uninit()?;
//! // write 1234 into sample
//! let mut response = response.write_payload(1234);
//! // override contents with 456 because its fun
//! *response.payload_mut() = 456;
//!
//! println!("server port id: {:?}", response.header().server_id());
//! response.send()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{response_mut::ResponseMut, service};
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
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) response: ResponseMut<Service, ResponsePayload, ResponseHeader>,
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Debug
    for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResponseMut {{ response: {:?} }}", self.response)
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the header of the response.
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
    /// println!("server port id: {:?}", response.header().server_id());
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
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// // loan_uninit initializes the user header with default therefore it is okay to access the
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
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
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
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// *response.payload_mut() = 123;
    /// println!("default payload {}", *response.payload());
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
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// *response.payload_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        self.response.payload_mut()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>
{
    /// Writes the provided payload into the [`ResponseMutUninit`] and returns and initialized
    /// [`ResponseMut`] that is ready to be sent.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
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
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// *response.payload_mut() = 789;
    /// // this is fine since the payload was initialized to 789
    /// let response = unsafe { response.assume_init() };
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        core::mem::transmute(self.response)
    }
}
