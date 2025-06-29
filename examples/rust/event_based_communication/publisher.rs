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
    port::{
        listener::Listener, notifier::Notifier, publisher::Publisher,
        update_connections::UpdateConnections,
    },
    prelude::*,
};

const CYCLE_TIME: Duration = Duration::from_secs(1);
const HISTORY_SIZE: usize = 20;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let publisher = CustomPublisher::new(&node, &"My/Funk/ServiceName".try_into()?)?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    // Whenever our publisher receives an event we get notified.
    let publisher_guard = waitset.attach_notification(&publisher)?;
    // Attach an interval so that we wake up and can publish a new message
    let cyclic_trigger_guard = waitset.attach_interval(CYCLE_TIME)?;

    let mut counter: u64 = 0;

    // Event callback that is called whenever the WaitSet received an event.
    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        // when the cyclic trigger guard gets notified we send out a new message
        if attachment_id.has_event_from(&cyclic_trigger_guard) {
            println!("send message: {counter}");
            publisher.send(counter).unwrap();
            counter += 1;
            // when something else happens on the publisher we handle the events
        } else if attachment_id.has_event_from(&publisher_guard) {
            publisher.handle_event().unwrap();
        }
        CallbackProgression::Continue
    };

    // Start the event loop. It will run until `CallbackProgression::Stop` is returned by the
    // event callback or an interrupt/termination signal was received.
    waitset.wait_and_process(on_event)?;

    println!("exit");

    Ok(())
}

/// High-level publisher class that contains besides a publisher also a notifier and a listener.
/// The notifier is used to send events like `PubSubEvent::SentSample` or `PubSubEvent::SentHistory`
/// and the listener to wait for new subscribers.
#[derive(Debug)]
struct CustomPublisher {
    publisher: Publisher<ipc::Service, TransmissionData, ()>,
    listener: Listener<ipc::Service>,
    notifier: Notifier<ipc::Service>,
}

impl FileDescriptorBased for CustomPublisher {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.listener.file_descriptor()
    }
}

impl SynchronousMultiplexing for CustomPublisher {}

impl CustomPublisher {
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
        let publisher = pubsub_service.publisher_builder().create()?;

        notifier.notify_with_custom_event_id(PubSubEvent::PublisherConnected.into())?;

        Ok(Self {
            publisher,
            listener,
            notifier,
        })
    }

    fn handle_event(&self) -> Result<(), Box<dyn core::error::Error>> {
        while let Some(event) = self.listener.try_wait_one()? {
            let event: PubSubEvent = event.into();
            match event {
                PubSubEvent::SubscriberConnected => {
                    println!("new subscriber connected - delivering history");
                    self.publisher.update_connections().unwrap();
                    self.notifier
                        .notify_with_custom_event_id(PubSubEvent::SentHistory.into())
                        .unwrap();
                }
                PubSubEvent::SubscriberDisconnected => {
                    println!("subscriber disconnected");
                }
                PubSubEvent::ReceivedSample => {
                    println!("subscriber has consumed sample");
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn send(&self, counter: u64) -> Result<(), Box<dyn core::error::Error>> {
        let sample = self.publisher.loan_uninit()?;

        let sample = sample.write_payload(TransmissionData {
            x: counter as i32,
            y: counter as i32 * 3,
            funky: counter as f64 * 812.12,
        });

        sample.send()?;
        self.notifier
            .notify_with_custom_event_id(PubSubEvent::SentSample.into())?;

        Ok(())
    }
}

impl Drop for CustomPublisher {
    fn drop(&mut self) {
        let _ = self
            .notifier
            .notify_with_custom_event_id(PubSubEvent::PublisherDisconnected.into());
    }
}
