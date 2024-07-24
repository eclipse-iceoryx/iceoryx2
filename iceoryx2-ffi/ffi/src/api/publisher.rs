// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use crate::api::{iox2_service_type_e, HandleToType, NoUserHeaderFfi, PayloadFfi};

use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union PublisherUnion {
    ipc: ManuallyDrop<Publisher<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<Publisher<process_local::Service, PayloadFfi, NoUserHeaderFfi>>,
}

impl PublisherUnion {
    pub(super) fn new_ipc(
        publisher: Publisher<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(publisher),
        }
    }
    pub(super) fn new_local(
        publisher: Publisher<process_local::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(publisher),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PublisherUnion>
pub struct iox2_publisher_storage_t {
    internal: [u8; 40], // magic number obtained with size_of::<Option<PublisherUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PublisherUnion)]
pub struct iox2_publisher_t {
    service_type: iox2_service_type_e,
    value: iox2_publisher_storage_t,
    deleter: fn(*mut iox2_publisher_t),
}

impl iox2_publisher_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PublisherUnion,
        deleter: fn(*mut iox2_publisher_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_publisher_h_t;
/// The owning handle for `iox2_publisher_t`. Passing the handle to an function transfers the ownership.
pub type iox2_publisher_h = *mut iox2_publisher_h_t;

pub struct iox2_publisher_ref_h_t;
/// The non-owning handle for `iox2_publisher_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_publisher_ref_h = *mut iox2_publisher_ref_h_t;

impl HandleToType for iox2_publisher_h {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_publisher_ref_h {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function needs to be called to destroy the publisher!
///
/// # Arguments
///
/// * `publisher_handle` - A valid [`iox2_publisher_h`]
///
/// # Safety
///
/// * The `publisher_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_publisher_t`] can be re-used with a call to
///   [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_drop(publisher_handle: iox2_publisher_h) {
    debug_assert!(!publisher_handle.is_null());

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut publisher.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut publisher.value.as_mut().local);
        }
    }
    (publisher.deleter)(publisher);
}

// END C API

use core::time::Duration;
use iceoryx2_bb_log::set_log_level;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[no_mangle]
pub extern "C" fn run_publisher(seconds: u32) -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let service_name = ServiceName::new("Hello/from/C");
    let node = NodeBuilder::new().create::<zero_copy::Service>();

    if service_name.is_err() || node.is_err() {
        return -1;
    }

    let service_name = service_name.unwrap();
    let node = node.unwrap();

    let service = node
        .service_builder(&service_name)
        .publish_subscribe::<u64>()
        .open_or_create();

    if service.is_err() {
        return -1;
    }

    let service = service.unwrap();

    let publisher = service.publisher_builder().create();

    if publisher.is_err() {
        return -1;
    }

    let publisher = publisher.unwrap();

    let mut counter: u64 = 0;

    let mut remaining_seconds = seconds;

    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        counter += 1;
        let sample = publisher.loan_uninit();

        if sample.is_err() {
            return -1;
        }

        let sample = sample.unwrap();

        let sample = sample.write_payload(counter);

        if sample.send().is_err() {
            return -1;
        }

        println!("Send sample {} ...", counter);

        remaining_seconds = remaining_seconds.saturating_sub(1);
        if remaining_seconds == 0 {
            break;
        }
    }

    println!("exit");

    0
}
