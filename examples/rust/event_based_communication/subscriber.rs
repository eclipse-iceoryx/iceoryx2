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

use examples_common::{PubSubEvent, TransmissionData};
use iceoryx2::{
    port::{listener::Listener, notifier::Notifier, subscriber::Subscriber},
    prelude::*,
    sample::Sample,
};

const HISTORY_SIZE: usize = 20;
const DEADLINE: Duration = Duration::from_secs(2);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let subscriber = CustomSubscriber::new(&node, &"My/Funk/ServiceName".try_into()?)?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    // The subscriber is attached as a deadline, meaning that we expect some activity
    // latest after the deadline has passed.
    let subscriber_guard = waitset.attach_deadline(&subscriber, DEADLINE)?;

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        // If we have received a new event on the subscriber we handle it.
        if attachment_id.has_event_from(&subscriber_guard) {
            subscriber.handle_event().unwrap();
            // If the subscriber did not receive an event until DEADLINE has
            // passed, we print out a warning.
        } else if attachment_id.has_missed_deadline(&subscriber_guard) {
            println!(
                "Contract violation! The subscriber did not receive a message for {DEADLINE:?}."
            );
        }

        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

    println!("exit");

    Ok(())
}

#[derive(Debug)]
struct CustomSubscriber {
    subscriber: Subscriber<ipc::Service, TransmissionData, ()>,
    notifier: Notifier<ipc::Service>,
    listener: Listener<ipc::Service>,
}

impl FileDescriptorBased for CustomSubscriber {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.listener.file_descriptor()
    }
}

impl SynchronousMultiplexing for CustomSubscriber {}

// High-level subscriber class that contains besides a subscriber also a notifier
// and a listener. The notifier is used to send events like
// `PubSubEvent::ReceivedSample` or to notify the publisher that a new subscriber
// connected.
// The listener waits for events originating from the publisher like
// `PubSubEvent::SentSample`.
impl CustomSubscriber {
    fn new(
        node: &Node<ipc::Service>,
        service_name: &ServiceName,
    ) -> Result<Self, Box<dyn core::error::Error>> {
        let pubsub_service = node
            .service_builder(service_name)
            .publish_subscribe::<TransmissionData>()
            .history_size(HISTORY_SIZE)
            .subscriber_max_buffer_size(HISTORY_SIZE)
            .open_or_create()?;
        let event_service = node
            .service_builder(service_name)
            .event()
            .open_or_create()?;

        let listener = event_service.listener_builder().create()?;
        let notifier = event_service.notifier_builder().create()?;
        let subscriber = pubsub_service.subscriber_builder().create()?;

        notifier.notify_with_custom_event_id(PubSubEvent::SubscriberConnected.into())?;

        Ok(Self {
            subscriber,
            listener,
            notifier,
        })
    }

    fn handle_event(&self) -> Result<(), Box<dyn core::error::Error>> {
        while let Some(event) = self.listener.try_wait_one()? {
            let event: PubSubEvent = event.into();
            match event {
                PubSubEvent::SentHistory => {
                    println!("History delivered");
                    while let Ok(Some(sample)) = self.receive() {
                        println!("  history: {:?}", sample.x);
                    }
                }
                PubSubEvent::SentSample => {
                    while let Ok(Some(sample)) = self.receive() {
                        println!("received: {:?}", sample.x);
                    }
                }
                PubSubEvent::PublisherConnected => println!("new publisher connected"),
                PubSubEvent::PublisherDisconnected => println!("publisher disconnected"),
                _ => (),
            }
        }

        Ok(())
    }

    fn receive(
        &self,
    ) -> Result<Option<Sample<ipc::Service, TransmissionData, ()>>, Box<dyn core::error::Error>>
    {
        match self.subscriber.receive()? {
            Some(sample) => {
                self.notifier
                    .notify_with_custom_event_id(PubSubEvent::ReceivedSample.into())?;
                Ok(Some(sample))
            }
            None => Ok(None),
        }
    }
}

impl Drop for CustomSubscriber {
    fn drop(&mut self) {
        self.notifier
            .notify_with_custom_event_id(PubSubEvent::SubscriberDisconnected.into())
            .unwrap();
    }
}
