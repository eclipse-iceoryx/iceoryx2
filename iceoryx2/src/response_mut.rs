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

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::service;

/// todo
pub struct ResponseMut<Service: service::Service, ResponsePayload: Debug, ResponseHeader: Debug> {
    _service: PhantomData<Service>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Deref
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    type Target = ResponsePayload;
    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> DerefMut
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
    }
}
