// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#![warn(missing_docs)]

//! # iceoryx2
//!
//! iceoryx2 is a cutting-edge service-oriented zero-copy lock-free inter-process communication
//! middleware. Designed to support various
//! [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)s
//! iceoryx2 empowers developers with
//! the flexibility of:
//!
//! - Publish-Subscribe
//! - Events
//! - Request-Response (planned)
//! - Pipeline (planned)
//! - Blackboard (planned)
//!
//! For a comprehensive list of all planned features, please refer to the
//! [GitHub Roadmap](https://github.com/eclipse-iceoryx/iceoryx2/blob/main/ROADMAP.md).
//!
//! Services are uniquely identified by name and
//! [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern). They can be instantiated with
//! diverse quality-of-service settings and are envisioned to be deployable in a `no_std` and
//! safety-critical environment in the future.
//!
//! Moreover, iceoryx2 offers configuration options that enable multiple service setups to coexist
//! on the same machine or even within the same process without interference. This versatility
//! allows iceoryx2 to seamlessly integrate with other frameworks simultaneously.
//!
//! iceoryx2 traces its lineage back to the
//! [eclipse iceoryx](https://github.com/eclipse-iceoryx/iceoryx) project, addressing a major
//! drawback – the central daemon. iceoryx2 embraces a fully decentralized architecture,
//! eliminating the need for a central daemon entirely.
//!
//! # Examples
//!
//! Each service is uniquely identified by a [`ServiceName`](crate::service::service_name::ServiceName).
//! Initiating communication requires the creation of a service, which serves as a port factory.
//! With this factory, endpoints for the service can be created, enabling seamless communication.
//!
//! For more detailed examples, explore the
//! [GitHub example folder](https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples).
//!
//! ## Publish-Subscribe
//!
//! Explore a simple publish-subscribe setup where the subscriber continuously receives data from
//! the publisher until the processes are gracefully terminated by the user with `CTRL+C`.
//!
//! **Subscriber (Process 1)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! // create our port factory by creating or opening the service
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let subscriber = service.subscriber_builder().create()?;
//!
//! while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
//!     while let Some(sample) = subscriber.receive()? {
//!         println!("received: {:?}", *sample);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! **Publisher (Process 2)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! // create our port factory by creating or opening the service
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let publisher = service.publisher_builder().create()?;
//!
//! while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
//!     let sample = publisher.loan_uninit()?;
//!     let sample = sample.write_payload(1234);
//!     sample.send()?;
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Events
//!
//! Explore a straightforward event setup, where the listener patiently awaits events from the
//! notifier. This continuous event listening continues until the user gracefully terminates
//! the processes by pressing `CTRL+C`.
//!
//! **Listener (Process 1)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
//!
//! while let NodeEvent::Tick = node.wait(Duration::ZERO) {
//!     if let Ok(Some(event_id)) = listener.timed_wait_one(CYCLE_TIME) {
//!         println!("event was triggered with id: {:?}", event_id);
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! **Notifier (Process 2)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let notifier = event.notifier_builder().create()?;
//!
//! let mut counter: usize = 0;
//! while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
//!     counter += 1;
//!     notifier.notify_with_custom_event_id(EventId::new(counter))?;
//!
//!     println!("Trigger event with id {} ...", counter);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Quality Of Services
//!
//! Quality of service settings, or service settings, play a crucial role in determining memory
//! allocation in a worst-case scenario. These settings can be configured during the creation of
//! a service, immediately after defining the
//! [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern). In cases where the service
//! already exists, these settings are interpreted as minimum requirements, ensuring a flexible
//! and dynamic approach to memory management.
//!
//! ## Publish-Subscribe
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"PubSubQos".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .enable_safe_overflow(true)
//!     // how many samples a subscriber can borrow in parallel
//!     .subscriber_max_borrowed_samples(2)
//!     // the maximum history size a subscriber can request
//!     .history_size(3)
//!     // the maximum buffer size of a subscriber
//!     .subscriber_max_buffer_size(4)
//!     // the maximum amount of subscribers of this service
//!     .max_subscribers(5)
//!     // the maximum amount of publishers of this service
//!     .max_publishers(2)
//!     .create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Event
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let event = node.service_builder(&"EventQos".try_into()?)
//!     .event()
//!     // the maximum amount of notifiers of this service
//!     .max_notifiers(2)
//!     // the maximum amount of listeners of this service
//!     .max_listeners(2)
//!     // defines the maximum supported event id value
//!     // WARNING: an increased value can have a significant performance impact on some
//!     //          configurations that use a bitset as event tracking mechanism
//!     .event_id_max_value(256)
//!     .create()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Port Behavior
//!
//! Certain ports in iceoryx2 provide users with the flexibility to define custom behaviors in
//! specific situations.
//! Custom port behaviors can be specified during the creation of a port,
//! utilizing the port factory or service, immediately following the specification of the port
//! type. This feature enhances the adaptability of iceoryx2 to diverse use cases and scenarios.
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::service::port_factory::publisher::UnableToDeliverStrategy;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .enable_safe_overflow(false)
//!     .open_or_create()?;
//!
//! let publisher = service.publisher_builder()
//!     // the maximum amount of samples this publisher can loan in parallel
//!     .max_loaned_samples(2)
//!     // defines the behavior when a sample could not be delivered when the subscriber buffer is
//!     // full, only useful in an non-overflow scenario
//!     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!     .create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//!  * `logger_log` - Uses the [log crate](https://crates.io/crates/log) as default log backend
//!  * `logger_tracing` - Uses the [tracing crate](https://crates.io/crates/tracing) as default log
//!     backend
//!  * `enforce_32bit_rwlock_atomic` - Enforces the 32-bit atomic also on 64-bit platforms. Enables
//!     32-bit and 64-bit applications to communicate but at the expense of the lock-free
//!     guarantee. Enabling the feature can cause a deadlock of the whole system when one
//!     application crashes at the wrong time.
//!
//! # Custom Configuration
//!
//! iceoryx2 offers the flexibility to configure default quality of service settings, paths, and
//! file suffixes through a custom configuration file.
//!
//! For in-depth details and examples, please visit the
//! [GitHub config folder](https://github.com/eclipse-iceoryx/iceoryx2/tree/main/config).

#[cfg(doctest)]
mod compiletests;

/// Handles iceoryx2s global configuration
pub mod config;

/// Central instance that owns all service entities and can handle incoming event in an event loop
pub mod node;

/// The ports or communication endpoints of iceoryx2
pub mod port;

pub(crate) mod raw_sample;

/// The payload that is received by a [`Subscriber`](crate::port::subscriber::Subscriber).
pub mod sample;

/// The payload that is sent by a [`Publisher`](crate::port::publisher::Publisher).
pub mod sample_mut;

/// The foundation of communication the service with its
/// [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)
pub mod service;

/// Loads a meaninful subset to cover 90% of the iceoryx2 communication use cases.
pub mod prelude;
