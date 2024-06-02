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

use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level;

mod publisher;
mod subscriber;

pub use publisher::*;
pub use subscriber::*;

#[no_mangle]
pub extern "C" fn zero_copy_service_list() -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let services = zero_copy::Service::list();

    if services.is_err() {
        return -1;
    }

    let services = services.unwrap();

    for service in services {
        println!("\n{:#?}", &service);
    }

    0
}
