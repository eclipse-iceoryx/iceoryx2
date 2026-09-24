// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

mod rcl;

use core::ffi::CStr;
use core::time::Duration;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::thread::ThreadBuilder;
use r2r_rcl::RCUTILS_LOG_SEVERITY::{RCUTILS_LOG_SEVERITY_DEBUG, RCUTILS_LOG_SEVERITY_ERROR};
use r2r_rcl::{
    RCUTILS_LOG_SEVERITY, rcl_publisher_get_default_options, rcutils_logging_initialize,
    rcutils_logging_set_default_logger_level, rmw_get_implementation_identifier,
    rmw_qos_durability_policy_e, rmw_qos_history_policy_e, rmw_qos_profile_t,
    rmw_qos_reliability_policy_e,
};

use rcl::{
    RclContext, RclError, RclNode, RclPublisher, RclSubscription, RclWaitSet, UInt8MultiArray,
};

const ITERATIONS: u64 = 100000;
const NAMESPACE: &CStr = c"";
const SPIN_TIMEOUT: Duration = Duration::from_millis(100);
const TIMEOUT: Duration = Duration::from_secs(10);

const SUPPORTED_RMW: &str = "rmw_fastrtps_cpp";

const PROFILES: &str = include_str!("fastdds_profiles.xml");
const DELIVERY_ENV: &str = "IOX2_BENCHMARK_INTRAPROCESS_DELIVERY";

/// The `intraprocess_delivery` setup of Fast DDS that the benchmark runs in.
/// See https://fast-dds.docs.eprosima.com/en/stable/fastdds/transport/intraprocess.html
#[derive(Debug, Clone, Copy, ValueEnum)]
enum IntraprocessDelivery {
    /// Both user data and discovery metadata using intra-process delivery.
    Full,
    /// Discovery metadata keeps using ordinary transport.
    UserDataOnly,
    /// The feature is disabled.
    Off,
}

impl IntraprocessDelivery {
    fn xml_value(&self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::UserDataOnly => "USER_DATA_ONLY",
            Self::Off => "OFF",
        }
    }
}

impl core::fmt::Display for IntraprocessDelivery {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "INTRAPROCESS_{}", self.xml_value())
    }
}

fn write_profiles() -> Result<PathBuf, Box<dyn core::error::Error>> {
    let path = std::env::temp_dir().join("iox2_benchmark_fastdds_profiles.xml");
    std::fs::write(&path, PROFILES)?;

    Ok(path)
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum History {
    KeepLast,
    KeepAll,
}

impl From<History> for rmw_qos_history_policy_e {
    fn from(history: History) -> Self {
        match history {
            History::KeepLast => Self::RMW_QOS_POLICY_HISTORY_KEEP_LAST,
            History::KeepAll => Self::RMW_QOS_POLICY_HISTORY_KEEP_ALL,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Reliability {
    Reliable,
    BestEffort,
}

impl From<Reliability> for rmw_qos_reliability_policy_e {
    fn from(reliability: Reliability) -> Self {
        match reliability {
            Reliability::Reliable => Self::RMW_QOS_POLICY_RELIABILITY_RELIABLE,
            Reliability::BestEffort => Self::RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Durability {
    Volatile,
    TransientLocal,
}

impl From<Durability> for rmw_qos_durability_policy_e {
    fn from(durability: Durability) -> Self {
        match durability {
            Durability::Volatile => Self::RMW_QOS_POLICY_DURABILITY_VOLATILE,
            Durability::TransientLocal => Self::RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL,
        }
    }
}

fn qos(args: &Args) -> rmw_qos_profile_t {
    rmw_qos_profile_t {
        history: args.history.into(),
        depth: args.depth,
        reliability: args.reliability.into(),
        durability: args.durability.into(),
        ..unsafe { rcl_publisher_get_default_options() }.qos
    }
}

fn set_log_level(level: RCUTILS_LOG_SEVERITY) {
    unsafe {
        rcutils_logging_initialize();
        rcutils_logging_set_default_logger_level(level as i32);
    }
}

fn rmw_implementation() -> &'static str {
    let identifier = unsafe { rmw_get_implementation_identifier() };
    assert!(!identifier.is_null(), "no rmw implementation is loaded");

    unsafe { CStr::from_ptr(identifier) }
        .to_str()
        .expect("rmw identifiers are valid utf-8")
}

struct Participant {
    publisher: RclPublisher,
    subscription: RclSubscription,
    wait_set: RclWaitSet,
    message: UInt8MultiArray,
}

impl Participant {
    fn create(
        context: &Arc<RclContext>,
        name: &CStr,
        publish_topic: &CStr,
        subscribe_topic: &CStr,
        qos: rmw_qos_profile_t,
        payload_size: usize,
    ) -> Result<Self, RclError> {
        let node = RclNode::create(context, name, NAMESPACE)?;

        Ok(Self {
            publisher: RclPublisher::create(&node, publish_topic, qos)?,
            subscription: RclSubscription::create(&node, subscribe_topic, qos)?,
            wait_set: RclWaitSet::create(context)?,
            message: UInt8MultiArray::with_size(payload_size)?,
        })
    }

    fn send(&self) -> Result<(), RclError> {
        self.publisher.publish(&self.message)
    }

    fn receive(&self) -> Result<(), RclError> {
        let mut message = UInt8MultiArray::new()?;
        let start = Time::now().expect("failed to acquire time");

        loop {
            if self.wait_set.wait(&self.subscription, SPIN_TIMEOUT)?
                && self.subscription.take(&mut message)?
            {
                return Ok(());
            }

            if start.elapsed().expect("failed to measure time") > TIMEOUT {
                panic!("no message within {TIMEOUT:?}");
            }
        }
    }

    /// Wait until the publisher has matched a subscription.
    fn await_subscriber(&self) {
        let matched = AdaptiveWaitBuilder::new()
            .create()
            .expect("failed to create the wait")
            .timed_wait_while(
                || self.publisher.subscription_count().map(|count| count == 0),
                TIMEOUT,
            )
            .expect("failed to wait for a subscription");

        assert!(matched, "no subscription matched within {TIMEOUT:?}");
    }
}

fn perform_benchmark(args: &Args) -> Result<(), Box<dyn core::error::Error>> {
    let topic_name_a2b = c"a2b";
    let topic_name_b2a = c"b2a";
    let profiles = write_profiles()?;

    // SAFETY: the benchmark is still single threaded at this point.
    unsafe {
        std::env::set_var("FASTRTPS_DEFAULT_PROFILES_FILE", &profiles);
        std::env::set_var(DELIVERY_ENV, args.intraprocess_delivery.xml_value());
        std::env::set_var("ROS_AUTOMATIC_DISCOVERY_RANGE", "LOCALHOST");
    };

    let context = RclContext::create()?;
    let qos = qos(args);

    let mut additional_publishers = Vec::new();
    let mut additional_subscribers = Vec::new();

    if args.number_of_additional_publishers > 0 || args.number_of_additional_subscribers > 0 {
        let node = RclNode::create(&context, c"benchmark_additional", NAMESPACE)?;

        for topic in [topic_name_a2b, topic_name_b2a] {
            for _ in 0..args.number_of_additional_publishers {
                additional_publishers.push(RclPublisher::create(&node, topic, qos)?);
            }

            for _ in 0..args.number_of_additional_subscribers {
                additional_subscribers.push(RclSubscription::create(&node, topic, qos)?);
            }
        }
    }

    let start_benchmark_barrier_handle = BarrierHandle::new();
    let startup_barrier_handle = BarrierHandle::new();
    let startup_barrier = BarrierBuilder::new(3)
        .create(&startup_barrier_handle)
        .unwrap();
    let start_benchmark_barrier = BarrierBuilder::new(3)
        .create(&start_benchmark_barrier_handle)
        .unwrap();

    let t1 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_1])
        .priority(255)
        .spawn(|| {
            let participant = Participant::create(
                &context,
                c"benchmark_a",
                topic_name_a2b,
                topic_name_b2a,
                qos,
                args.payload_size,
            )
            .unwrap();

            participant.await_subscriber();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                participant.send().unwrap();
                participant.receive().unwrap();
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            let participant = Participant::create(
                &context,
                c"benchmark_b",
                topic_name_b2a,
                topic_name_a2b,
                qos,
                args.payload_size,
            )
            .unwrap();

            participant.await_subscriber();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                participant.receive().unwrap();
                participant.send().unwrap();
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ({}) ::: Iterations: {}, Time: {} s, Latency: {} ns, Sample Size: {}",
        rmw_implementation(),
        args.intraprocess_delivery,
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2),
        args.payload_size
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: u64,
    /// The Fast DDS intra-process delivery setup the benchmark runs in
    #[clap(long, value_enum, default_value_t = IntraprocessDelivery::Off)]
    intraprocess_delivery: IntraprocessDelivery,
    /// Activate full log output
    #[clap(short, long)]
    debug_mode: bool,
    /// The cpu core that shall be used by participant 1
    #[clap(long, default_value_t = 0)]
    cpu_core_participant_1: usize,
    /// The cpu core that shall be used by participant 2
    #[clap(long, default_value_t = 1)]
    cpu_core_participant_2: usize,
    /// The size in bytes of the payload that shall be used
    #[clap(short, long, default_value_t = 8192)]
    payload_size: usize,
    /// The history policy of every publisher and subscription
    #[clap(long, value_enum, default_value_t = History::KeepLast)]
    history: History,
    /// The queue depth.
    #[clap(long, default_value_t = 1)]
    depth: usize,
    /// The reliability policy of every publisher and subscription.
    #[clap(long, value_enum, default_value_t = Reliability::Reliable)]
    reliability: Reliability,
    /// The durability policy of every publisher and subscription.
    #[clap(long, value_enum, default_value_t = Durability::Volatile)]
    durability: Durability,
    /// The number of additional publishers per topic in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_publishers: usize,
    /// The number of additional subscribers per topic in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_subscribers: usize,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(RCUTILS_LOG_SEVERITY_DEBUG);
    } else {
        set_log_level(RCUTILS_LOG_SEVERITY_ERROR);
    }

    if rmw_implementation() != SUPPORTED_RMW {
        return Err(format!(
            "{} is not supported, the benchmark requires {SUPPORTED_RMW}",
            rmw_implementation()
        )
        .into());
    }

    perform_benchmark(&args)
}
