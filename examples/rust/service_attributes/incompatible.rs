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

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let _incompatible_service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .open_with_attributes(
            // the opening of the service will fail since the
            // `camera_resolution` attribute is `1920x1080` and not `3840x2160`
            &AttributeVerifier::new()
                .require(&"camera_resolution".try_into()?, &"3840x2160".try_into()?),
        )
        .map_err(|e| println!("camera_resolution: 3840x2160 -> {:?}", e));

    let _incompatible_service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .open_with_attributes(
            // the opening of the service will fail since the key is not defined.
            &AttributeVerifier::new().require_key(&"camera_type".try_into()?),
        )
        .map_err(|e| println!("camera_type -> {:?}", e));

    Ok(())
}
