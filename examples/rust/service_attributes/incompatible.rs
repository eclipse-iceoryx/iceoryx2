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
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let _incompatible_service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .open_with_attributes(
            // the opening of the service will fail since the
            // `camera_resolution` attribute is `1920x1080` and not `3840x2160`
            &AttributeVerifier::new().require("camera_resolution", "3840x2160"),
        );

    let _incompatible_service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .open_with_attributes(
            // the opening of the service will fail since the key is not defined.
            &AttributeVerifier::new().require_key("camera_type"),
        );

    Ok(())
}
