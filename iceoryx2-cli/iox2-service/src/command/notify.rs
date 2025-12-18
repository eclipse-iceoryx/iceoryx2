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
use std::io::Write;

use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2_cli::Format;

use crate::cli::NotifyOptions;
use crate::command::EventFeedback;
use crate::command::EventType;

pub(crate) fn notify(options: NotifyOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service = node
        .service_builder(&ServiceName::new(&options.service)?)
        .event()
        .open_or_create()?;

    let notifier = service
        .notifier_builder()
        .default_event_id(EventId::new(options.event_id))
        .create()?;

    let notify_feedback = EventFeedback {
        event_type: EventType::NotificationSent,
        service: options.service,
        event_id: Some(options.event_id),
    };
    let notify = || -> Result<()> {
        notifier.notify()?;
        println!("{}", format.as_string(&notify_feedback)?);
        std::io::stdout().flush()?;
        Ok(())
    };

    for _ in 1..options.num {
        notify()?;
        std::thread::sleep(Duration::from_millis(options.interval_in_ms));
    }

    notify()?;

    Ok(())
}
