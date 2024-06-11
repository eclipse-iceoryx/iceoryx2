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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ServiceName::new("Service/With/Properties")?;

    // the opening of the service will fail since the
    // `camera_resolution` attribute is `1920x1080` and not `3840x2160`
    let _incompatible_service = zero_copy::Service::new(&service_name)
        .add_attribute("camera_resolution", "3840x2160")
        .publish_subscribe::<u64>()
        .open()?;

    Ok(())
}
