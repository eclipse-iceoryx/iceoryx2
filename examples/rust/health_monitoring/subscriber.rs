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
use examples_common::{open_service, PubSubEvent};
use iceoryx2::{
    node::NodeView,
    port::{listener::Listener, subscriber::Subscriber},
    prelude::*,
};

const REACTION_BUFFER_MS: u64 = 500;
const CYCLE_TIME_1: Duration = Duration::from_millis(1000 + REACTION_BUFFER_MS);
const CYCLE_TIME_2: Duration = Duration::from_millis(1500 + REACTION_BUFFER_MS);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let service_name_1 = ServiceName::new("service_1")?;
    let service_name_2 = ServiceName::new("service_2")?;

    let node = NodeBuilder::new()
        .name(&"subscriber".try_into()?)
        .create::<ipc::Service>()?;

    // open a pubsub and an event service with the same name
    let (service_event_1, service_pubsub_1) = open_service(&node, &service_name_1)?;
    let (service_event_2, service_pubsub_2) = open_service(&node, &service_name_2)?;

    let subscriber_1 = service_pubsub_1.subscriber_builder().create()?;
    let subscriber_2 = service_pubsub_2.subscriber_builder().create()?;
    let listener_1 = service_event_1.listener_builder().create()?;
    let listener_2 = service_event_2.listener_builder().create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    // If the service has defined a deadline we will use it, otherwise
    // we expect that the listener receive a message sent event after at most CYCLE_TIME_X
    let deadline_1 = listener_1.deadline().unwrap_or(CYCLE_TIME_1);
    let deadline_2 = listener_2.deadline().unwrap_or(CYCLE_TIME_2);
    let listener_1_guard = waitset.attach_deadline(&listener_1, deadline_1)?;
    let listener_2_guard = waitset.attach_deadline(&listener_2, deadline_2)?;

    let missed_deadline = |service_name, cycle_time| {
        println!(
            "{service_name}: violated contract and did not send a message after {cycle_time:?}."
        );
    };

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if attachment_id.has_missed_deadline(&listener_1_guard) {
            missed_deadline(&service_name_1, deadline_1);
            // one cause of a deadline it can be a dead node. usually our "central_daemon" would
            // take care of monitoring but when the node and the central daemon crashed we take
            // over here and check for dead nodes
            find_and_cleanup_dead_nodes();
        }

        if attachment_id.has_missed_deadline(&listener_2_guard) {
            missed_deadline(&service_name_2, deadline_2);
            find_and_cleanup_dead_nodes();
        }

        if attachment_id.has_event_from(&listener_1_guard) {
            // in this function we either print out the received sample or the event that has
            // occurred like, publisher connected/disconnected or a process was identified as dead
            handle_incoming_event(&listener_1, &subscriber_1, &service_name_1);
        }

        if attachment_id.has_event_from(&listener_2_guard) {
            handle_incoming_event(&listener_2, &subscriber_2, &service_name_2);
        }

        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

    println!("exit");

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

fn handle_incoming_event(
    listener: &Listener<ipc::Service>,
    subscriber: &Subscriber<ipc::Service, u64, ()>,
    service_name: &ServiceName,
) {
    listener
        .try_wait_all(|event_id| {
            if event_id == PubSubEvent::ProcessDied.into() {
                println!("{service_name}: process died!");
            } else if event_id == PubSubEvent::PublisherConnected.into() {
                println!("{service_name}: publisher connected!");
            } else if event_id == PubSubEvent::PublisherDisconnected.into() {
                println!("{service_name}: publisher disconnected!");
            } else if event_id == PubSubEvent::SentSample.into() {
                if let Some(sample) = subscriber.receive().expect("") {
                    println!("{}: Received sample {} ...", service_name, *sample)
                }
            }
        })
        .expect("");
}
