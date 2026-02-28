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

use crate::cli::HzOptions;
use crate::command::get_pubsub_service_types;
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_cli::Format;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(serde::Serialize)]
struct HzStats {
    rate_hz: f64,
    avg_s: f64,
    min_s: f64,
    max_s: f64,
    std_dev_s: f64,
    window: usize,
}

pub(crate) fn hz(options: HzOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service_name = ServiceName::new(&options.service)?;
    let cycle_time = Duration::from_micros(100);
    let start = Instant::now();

    while !ipc::Service::does_exist(
        &service_name,
        node.config(),
        MessagingPattern::PublishSubscribe,
    )? {
        if reached_timeout(start, options.timeout) {
            return Ok(());
        }

        if node.wait(cycle_time).is_err() {
            return Ok(());
        }
    }

    let service_types = get_pubsub_service_types(&service_name, &node)?;

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&service_types.payload)
            .__internal_set_user_header_type_details(&service_types.user_header)
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let mut intervals: VecDeque<u128> = VecDeque::new();
    let mut last_msg_time: Option<Instant> = None;
    let mut last_print = Instant::now();
    let mut last_printed_msg_time: Option<Instant> = None;

    while node.wait(cycle_time).is_ok() {
        let mut timeout_reached = reached_timeout(start, options.timeout);

        while let Some(_sample) = unsafe { subscriber.receive_custom_payload()? } {
            let now = Instant::now();
            if let Some(prev) = last_msg_time {
                let interval_ns = now.duration_since(prev).as_nanos();
                intervals.push_back(interval_ns);
                if intervals.len() > options.window {
                    intervals.pop_front();
                }
            }
            last_msg_time = Some(now);

            if reached_timeout(start, options.timeout) {
                timeout_reached = true;
                break;
            }
        }

        if last_print.elapsed() >= Duration::from_secs(1) {
            last_print = Instant::now();
            if last_msg_time == last_printed_msg_time {
                continue;
            }
            last_printed_msg_time = last_msg_time;
            print_stats(&intervals, format)?;
        }

        if timeout_reached || reached_timeout(start, options.timeout) {
            if last_msg_time != last_printed_msg_time {
                print_stats(&intervals, format)?;
            }
            break;
        }
    }

    Ok(())
}

fn reached_timeout(start: Instant, timeout_s: Option<u64>) -> bool {
    timeout_s
        .map(|timeout| start.elapsed() >= Duration::from_secs(timeout))
        .unwrap_or(false)
}

fn print_stats(intervals: &VecDeque<u128>, format: Format) -> Result<()> {
    let n = intervals.len();
    if n == 0 {
        return Ok(());
    }
    let mean_ns = intervals.iter().sum::<u128>() as f64 / n as f64;
    let rate_hz = if mean_ns > 0.0 { 1e9 / mean_ns } else { 0.0 };
    let min_ns = *intervals.iter().min().unwrap() as f64;
    let max_ns = *intervals.iter().max().unwrap() as f64;
    let variance = intervals
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean_ns;
            diff * diff
        })
        .sum::<f64>()
        / n as f64;

    let stats = HzStats {
        rate_hz,
        avg_s: mean_ns * 1e-9,
        min_s: min_ns * 1e-9,
        max_s: max_ns * 1e-9,
        std_dev_s: variance.sqrt() * 1e-9,
        window: n,
    };

    println!("{}", format.as_string(&stats)?);
    Ok(())
}
