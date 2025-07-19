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

use core::ptr::copy_nonoverlapping;
use core::time::Duration;
use std::io::Write;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::{Context, Error, Result};
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::{
    TypeDetail, TypeNameString, TypeVariant,
};
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::output::ServiceDescription;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;
use iceoryx2_services_discovery::service_discovery::Config as DiscoveryConfig;
use iceoryx2_services_discovery::service_discovery::Discovery;
use iceoryx2_services_discovery::service_discovery::Service as DiscoveryService;
use serde::Serialize;

use crate::cli::{
    CliTypeVariant, DataRepresentation, ListenOptions, NotifyOptions, OutputFilter, PublishOptions,
    SubscribeOptions,
};

#[allow(clippy::enum_variant_names)] // explicitly allow same prefix Notification since it shall
// be human readable on command line
#[derive(Serialize)]
enum EventType {
    NotificationSent,
    NotificationReceived,
    NotificationTimeoutExceeded,
}

#[derive(Serialize)]
struct EventFeedback {
    event_type: EventType,
    service: String,
    event_id: Option<usize>,
}

fn raw_data_to_hex_string(data: &[u8]) -> String {
    let mut ret_val = String::with_capacity(2 * data.len());
    for byte in data {
        ret_val.push_str(&format!("{0:0>2x} ", byte));
    }

    ret_val
}

pub fn listen(options: ListenOptions, format: Format) -> Result<()> {
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

pub fn notify(options: NotifyOptions, format: Format) -> Result<()> {
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

pub fn subscribe(options: SubscribeOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let config = iceoryx2::config::Config::global_config();
    let service_name = ServiceName::new(&options.service)?;
    let service_details =
        match ipc::Service::details(&service_name, config, MessagingPattern::PublishSubscribe)? {
            Some(v) => v,
            None => {
                return Err(anyhow!(
                    "unable to access service \"{}\", does it exist?",
                    options.service
                ))
            }
        };

    let mut file = match options.output_file {
        Some(v) => Some(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(v)?,
        ),
        None => None,
    };

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(
                &service_details
                    .static_details
                    .messaging_pattern()
                    .publish_subscribe()
                    .message_type_details()
                    .payload,
            )
            .__internal_set_user_header_type_details(
                &service_details
                    .static_details
                    .messaging_pattern()
                    .publish_subscribe()
                    .message_type_details()
                    .user_header,
            )
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let cycle_time = Duration::from_millis(10);

    let start = Instant::now();
    let mut msg_counter = 0u64;
    'node_loop: while node.wait(cycle_time).is_ok() {
        while let Some(sample) = unsafe { subscriber.receive_custom_payload()? } {
            let content = match options.data_representation {
                DataRepresentation::Hex => Some(raw_data_to_hex_string(unsafe {
                    core::slice::from_raw_parts(
                        sample.payload().as_ptr().cast(),
                        sample.payload().len(),
                    )
                })),
                DataRepresentation::Text => match str::from_utf8(unsafe {
                    core::slice::from_raw_parts(
                        sample.payload().as_ptr().cast(),
                        sample.payload().len(),
                    )
                }) {
                    Ok(content) => Some(content.to_string()),
                    Err(e) => {
                        eprintln!("received data contains invalid UTF-8 symbols ({e:?}).");
                        None
                    }
                },
            };

            if let Some(file) = &mut file {
                if let Some(content) = content {
                    file.write(content.as_bytes())?;
                }
            }

            msg_counter += 1;
            if let Some(max_messages) = options.max_messages {
                if msg_counter >= max_messages {
                    break 'node_loop;
                }
            }

            if let Some(timeout) = options.timeout {
                if start.elapsed().as_millis() >= timeout as _ {
                    break 'node_loop;
                }
            }
        }
    }

    Ok(())
}

pub fn publish(options: PublishOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service = unsafe {
        node.service_builder(&ServiceName::new(&options.service)?)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&TypeDetail {
                variant: match options.type_variant {
                    CliTypeVariant::Dynamic => TypeVariant::Dynamic,
                    CliTypeVariant::FixedSize => TypeVariant::FixedSize,
                },
                type_name: TypeNameString::try_from(options.type_name.as_str())?,
                size: options.type_size,
                alignment: options.type_alignment,
            })
            .__internal_set_user_header_type_details(&TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: TypeNameString::try_from(options.header_type_name.as_str())?,
                size: options.header_type_size,
                alignment: options.header_type_alignment,
            })
            .open_or_create()?
    };

    let publisher = match options.type_variant {
        CliTypeVariant::FixedSize => service.publisher_builder().create()?,
        CliTypeVariant::Dynamic => service
            .publisher_builder()
            .initial_max_slice_len(4096)
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()?,
    };

    for message in options.message {
        let mut sample = unsafe { publisher.loan_custom_payload(message.len())? };
        unsafe {
            copy_nonoverlapping(
                message.as_ptr(),
                sample.payload_mut().as_mut_ptr().cast(),
                message.len(),
            )
        };
        let sample = unsafe { sample.assume_init() };
        sample.send()?;
        std::thread::sleep(Duration::from_millis(options.time_between_messages as _));
    }

    Ok(())
}

pub fn list(filter: OutputFilter, format: Format) -> Result<()> {
    let mut services = Vec::<ServiceDescriptor>::new();

    ipc::Service::list(Config::global_config(), |service| {
        if filter.matches(&service) {
            services.push(ServiceDescriptor::from(&service));
        }
        CallbackProgression::Continue
    })
    .context("failed to retrieve services")?;

    services.sort_by_key(|pattern| match pattern {
        ServiceDescriptor::PublishSubscribe(name) => (name.clone(), 0),
        ServiceDescriptor::Event(name) => (name.clone(), 1),
        ServiceDescriptor::RequestResponse(name) => (name.clone(), 2),
        ServiceDescriptor::Undefined(name) => (name.to_string(), 3),
    });

    print!("{}", format.as_string(&services)?);

    Ok(())
}

pub fn details(service_name: String, filter: OutputFilter, format: Format) -> Result<()> {
    let mut error: Option<Error> = None;

    ipc::Service::list(Config::global_config(), |service| {
        if service_name == service.static_details.name().to_string() && filter.matches(&service) {
            match format.as_string(&ServiceDescription::from(&service)) {
                Ok(output) => {
                    print!("{output}");
                    CallbackProgression::Continue
                }
                Err(e) => {
                    error = Some(e);
                    CallbackProgression::Stop
                }
            }
        } else {
            CallbackProgression::Continue
        }
    })?;

    if let Some(err) = error {
        return Err(err);
    }
    Ok(())
}

pub fn discovery(
    rate: u64,
    publish_events: bool,
    max_subscribers: usize,
    send_notifications: bool,
    max_listeners: usize,
    format: Format,
) -> Result<()> {
    let discovery_config = DiscoveryConfig {
        publish_events,
        max_subscribers,
        send_notifications,
        max_listeners,
        include_internal: false,
        ..Default::default()
    };

    let mut service =
        DiscoveryService::<ipc::Service>::create(&discovery_config, Config::global_config())
            .map_err(|e| anyhow::anyhow!("failed to create service: {:?}", e))?;

    println!("=== Service Started (rate: {rate}ms) ===");

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let guard = waitset
        .attach_interval(core::time::Duration::from_millis(rate))
        .map_err(|e| anyhow!("failed to attach interval to waitset: {:?}", e))?;
    let attachment = WaitSetAttachmentId::from_guard(&guard);

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if attachment_id == attachment {
            let on_added = |service: &ServiceDetails<ipc::Service>| {
                println!(
                    "{}",
                    format
                        .as_string(&Discovery::Added(service.static_details.clone()))
                        .unwrap_or_default()
                )
            };
            let on_removed = |service: &ServiceDetails<ipc::Service>| {
                println!(
                    "{}",
                    format
                        .as_string(&Discovery::Removed(service.static_details.clone()))
                        .unwrap_or_default()
                )
            };
            if let Err(e) = service.spin(on_added, on_removed) {
                eprintln!("error while spinning service: {e:?}");
            }
        }

        CallbackProgression::Continue
    };

    waitset
        .wait_and_process(on_event)
        .map_err(|e| anyhow!("error waiting on waitset: {:?}", e))?;

    Ok(())
}
