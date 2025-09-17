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

use core::time::Duration;

use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2_cli::Format;

use crate::cli::ListenOptions;
use crate::command::EventFeedback;
use crate::command::EventType;

pub(crate) fn listen(options: ListenOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service = node
        .service_builder(&ServiceName::new(&options.service)?)
        .event()
        .open_or_create()?;

    let listener = service.listener_builder().create()?;

    for _ in 0..options.repetitions.unwrap_or(u64::MAX) {
        let mut received_notification = false;
        let callback = |event_id: EventId| {
            received_notification = true;
            println!(
                "{}",
                format
                    .as_string(&EventFeedback {
                        event_type: EventType::NotificationReceived,
                        service: options.service.clone(),
                        event_id: Some(event_id.as_value())
                    })
                    .unwrap_or("Failed to format EventFeedback".to_string())
            );
        };

        if options.timeout_in_ms != 0 {
            listener.timed_wait_all(callback, Duration::from_millis(options.timeout_in_ms))?;
        } else {
            listener.blocking_wait_all(callback)?;
        }

        if !received_notification {
            println!(
                "{}",
                format.as_string(&EventFeedback {
                    event_type: EventType::NotificationTimeoutExceeded,
                    service: options.service.clone(),
                    event_id: None
                })?
            );
        }
    }

    Ok(())
}
