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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::rc::Rc;
use alloc::vec::Vec;
use iceoryx2::service::marker::CustomHeaderMarker;
use iceoryx2::service::{Service, static_config::StaticConfig};
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};

use crate::backend::session::{self, Session};

#[derive(Debug)]
pub enum CreationError {}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub enum SendError {
    SendSample(session::SendError),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug)]
pub enum ReceiveError {
    ReceiveSample(session::ReceiveError),
    LoanSample,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    session: Rc<Session>,
    static_config: &'a StaticConfig,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(session: Rc<Session>, static_config: &'a StaticConfig) -> Self {
        Self {
            session,
            static_config,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        Ok(Relay {
            session: self.session,
            static_config: self.static_config.clone(),
            _phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    session: Rc<Session>,
    static_config: StaticConfig,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;

    type ReceiveError = ReceiveError;

    fn send(
        &self,
        sample: iceoryx2_services_tunnel_backend::types::publish_subscribe::Sample<S>,
    ) -> Result<(), Self::SendError> {
        let user_header = sample.user_header();
        let payload = sample.payload();

        let header_bytes: Vec<u8> = unsafe {
            core::slice::from_raw_parts(
                user_header as *const CustomHeaderMarker as *const u8,
                user_header_size(&self.static_config),
            )
        }
        .to_vec();
        let payload_bytes: Vec<u8> =
            unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
                .to_vec();

        self.session
            .send_sample(
                self.static_config.service_hash(),
                header_bytes,
                payload_bytes,
            )
            .map_err(SendError::SendSample)
    }

    fn receive<LoanError>(
        &self,
        loan: &mut iceoryx2_services_tunnel_backend::types::publish_subscribe::LoanFn<
            '_,
            S,
            LoanError,
        >,
    ) -> Result<
        Option<iceoryx2_services_tunnel_backend::types::publish_subscribe::SampleMut<S>>,
        Self::ReceiveError,
    > {
        let received = match self
            .session
            .recv_sample(self.static_config.service_hash())
            .map_err(ReceiveError::ReceiveSample)?
        {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut sample = loan(received.payload.len()).map_err(|_| ReceiveError::LoanSample)?;

        let header_size = user_header_size(&self.static_config);
        debug_assert_eq!(received.header.len(), header_size);
        debug_assert!(sample.payload_mut().len() >= received.payload.len());

        unsafe {
            core::ptr::copy_nonoverlapping(
                received.header.as_ptr(),
                sample.user_header_mut() as *mut CustomHeaderMarker as *mut u8,
                header_size,
            );
            core::ptr::copy_nonoverlapping(
                received.payload.as_ptr(),
                sample.payload_mut().as_mut_ptr().cast::<u8>(),
                received.payload.len(),
            );
        }
        Ok(Some(unsafe { sample.assume_init() }))
    }
}

fn user_header_size(static_config: &StaticConfig) -> usize {
    static_config
        .publish_subscribe()
        .message_type_details()
        .user_header
        .size()
}
