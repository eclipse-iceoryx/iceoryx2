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

#![allow(non_camel_case_types)]

use core::mem::ManuallyDrop;
use iceoryx2::port::client::Client;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::iox2_service_type_e;
use super::iox2_unable_to_deliver_strategy_e;
use super::AssertNonNullHandle;
use super::HandleToType;
use super::PayloadFfi;
use super::UserHeaderFfi;

// BEGIN types definition
pub(super) union ClientUnion {
    ipc: ManuallyDrop<Client<ipc::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>>,
    local:
        ManuallyDrop<Client<local::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>>,
}

impl ClientUnion {
    pub(super) fn new_ipc(
        client: Client<ipc::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(client),
        }
    }
    pub(super) fn new_local(
        client: Client<local::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(client),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<ClientUnion>
pub struct iox2_client_storage_t {
    internal: [u8; 248], // magic number obtained with size_of::<Option<ClientUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ClientUnion)]
pub struct iox2_client_t {
    service_type: iox2_service_type_e,
    value: iox2_client_storage_t,
    deleter: fn(*mut iox2_client_t),
}

impl iox2_client_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ClientUnion,
        deleter: fn(*mut iox2_client_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_client_h_t;
/// The owning handle for `iox2_client_t`. Passing the handle to an function transfers the ownership.
pub type iox2_client_h = *mut iox2_client_h_t;
/// The non-owning handle for `iox2_client_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_client_h_ref = *const iox2_client_h;

impl AssertNonNullHandle for iox2_client_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_client_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_client_h {
    type Target = *mut iox2_client_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_client_h_ref {
    type Target = *mut iox2_client_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END types definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_client_unable_to_deliver_strategy(
    handle: iox2_client_h_ref,
) -> iox2_unable_to_deliver_strategy_e {
    handle.assert_non_null();

    let client = &mut *handle.as_type();

    match client.service_type {
        iox2_service_type_e::IPC => client
            .value
            .as_mut()
            .ipc
            .unable_to_deliver_strategy()
            .into(),
        iox2_service_type_e::LOCAL => client
            .value
            .as_mut()
            .local
            .unable_to_deliver_strategy()
            .into(),
    }
}
// END C API
