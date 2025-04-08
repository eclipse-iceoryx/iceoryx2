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

use core::time::Duration;
use examples_common::PubSubEvent;
use iceoryx2::{node::NodeView, prelude::*};

const CYCLE_TIME: Duration = Duration::from_millis(100);
const DEADLINE_SERVICE_1: Duration = Duration::from_millis(1500);
const DEADLINE_SERVICE_2: Duration = Duration::from_millis(2000);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let service_name_1 = ServiceName::new("service_1")?;
    let service_name_2 = ServiceName::new("service_2")?;

    let node = NodeBuilder::new()
        .name(&"central daemon".try_into()?)
        .create::<ipc::Service>()?;

    // The central daemon is responsible to create all services before hand and the other processes
    // just open the communication resources and start communicating.
    let _service_pubsub_1 = node
        .service_builder(&service_name_1)
        .publish_subscribe::<u64>()
        // We use here open_or_create so that, in case of a crash of the central daemon, it can
        // be restarted.
        .open_or_create()?;

    let _service_event_1 = node
        .service_builder(&service_name_1)
        .event()
        // Defines the maximum timespan between two notifications for this service. The user of a
        // notifier that send a notification after the deadline was already reached, receives an
        // MISSED_DEADLINE error after the notification was delivered.
        .deadline(DEADLINE_SERVICE_1)
        // Whenever a new notifier is created the PublisherConnected event is emitted. this makes
        // sense since in this example a notifier is always created after a new publisher was
        // created.
        // The task of the notifier/event is it to inform and wake up other processes when certain
        // system event have happened.
        .notifier_created_event(PubSubEvent::PublisherConnected.into())
        .notifier_dropped_event(PubSubEvent::PublisherDisconnected.into())
        // This event is emitted when either the central daemon or a decentralized process detects
        // a dead node and cleaned up all of its stale resources succesfully.
        .notifier_dead_event(PubSubEvent::ProcessDied.into())
        .open_or_create()?;

    let _service_pubsub_2 = node
        .service_builder(&service_name_2)
        .publish_subscribe::<u64>()
        .open_or_create()?;

    let _service_event_2 = node
        .service_builder(&service_name_2)
        .event()
        .deadline(DEADLINE_SERVICE_2)
        .notifier_created_event(PubSubEvent::PublisherConnected.into())
        .notifier_dropped_event(PubSubEvent::PublisherDisconnected.into())
        .notifier_dead_event(PubSubEvent::ProcessDied.into())
        .open_or_create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let _cycle_guard = waitset.attach_interval(CYCLE_TIME);

    println!("Central daemon up and running.");
    waitset.wait_and_process(|_| {
        // The only task of our central daemon is it to monitor all running nodes and cleanup their
        // resources if a process has died.
        //
        // Since we added the notifier_dead_event to the service, all listeners, that are waiting
        // on a service where one participant has died, will be woken up and they receive
        // the PubSubEvent::ProcessDied
        find_and_cleanup_dead_nodes();
        CallbackProgression::Continue
    })?;

    Ok(())
}

fn find_and_cleanup_dead_nodes() {
    Node::<ipc::Service>::list(Config::global_config(), |node_state| {
        if let NodeState::Dead(state) = node_state {
            println!(
                "detected dead node: {:?}",
                state.details().as_ref().map(|v| v.name())
            );
            state.remove_stale_resources().expect("");
        }

        CallbackProgression::Continue
    })
    .expect("");
}
