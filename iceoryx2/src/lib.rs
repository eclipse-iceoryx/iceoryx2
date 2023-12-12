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

//! # Iceoryx2
//!
//! Iceoryx2 is a cutting-edge service-oriented zero-copy lock-free inter-process communication
//! middleware. Designed to support various
//! [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)s
//! Iceoryx2 empowers developers with
//! the flexibility of:
//!
//! - Publish-Subscribe
//! - Events
//! - Request-Response (planned)
//! - Pipeline (planned)
//! - Blackboard (planned)
//!
//! For a comprehensive list of all planned features, please refer to the
//! [GitHub Roadmap](https://github.com/iceoryx2/iceoryx2/ROADMAP.md).
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
//! Iceoryx2 traces its lineage back to the
//! [eclipse iceoryx](https://github.com/eclipse-iceoryx/iceoryx) project, addressing a major
//! drawback – the central daemon. Iceoryx2 embraces a fully decentralized architecture,
//! eliminating the need for a central daemon entirely.
//!
//! # Examples
//!
//! Each service is uniquely identified by a [`ServiceName`](crate::service::service_name::ServiceName).
//! Initiating communication requires the creation of a service, which serves as a port factory.
//! With this factory, endpoints for the service can be created, enabling seamless communication.
//!
//! For more detailed examples, explore the
//! [GitHub example folder](https://github.com/iceoryx2/iceoryx2/tree/main/examples).
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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! // create our port factory by creating or opening the service
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let subscriber = service.subscriber().create()?;
//!
//! while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
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
//! let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//!
//! // create our port factory by creating or opening the service
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service.publisher().create()?;
//!
//! while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
//!     let sample = publisher.loan_uninit()?;
//!     let sample = sample.write_payload(1234);
//!     publisher.send(sample)?;
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
//! let event_name = ServiceName::new("MyEventName")?;
//!
//! let event = zero_copy::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener().create()?;
//!
//! while let Iox2Event::Tick = Iox2::wait(Duration::ZERO) {
//!     if let Ok(events) = listener.timed_wait(CYCLE_TIME) {
//!         for event_id in events {
//!             println!("event was triggered with id: {:?}", event_id);
//!         }
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
//! let event_name = ServiceName::new("MyEventName")?;
//!
//! let event = zero_copy::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let notifier = event.notifier().create()?;
//!
//! let mut counter: u64 = 0;
//! while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
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
//! let service_name = ServiceName::new("PubSubQos")?;
//!
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
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
//!     .create::<u64>()?;
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
//! let event_name = ServiceName::new("EventQos")?;
//!
//! let event = zero_copy::Service::new(&event_name)
//!     .event()
//!     // the maximum amount of notifiers of this service
//!     .max_notifiers(2)
//!     // the maximum amount of listeners of this service
//!     .max_listeners(2)
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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .enable_safe_overflow(false)
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service.publisher()
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
//!
//! # Custom Configuration
//!
//! Iceoryx2 offers the flexibility to configure default quality of service settings, paths, and
//! file suffixes through a custom configuration file.
//!
//! For in-depth details and examples, please visit the
//! [GitHub config folder](https://github.com/iceoryx2/iceoryx2/tree/main/config).

#[cfg(doctest)]
mod compiletests;

/// Handles iceoryx2s global configuration
pub mod config;

/// Central instance that handles all incoming events, the event loop
pub mod iox2;

pub(crate) mod message;

/// The ports or communication endpoints of iceoryx2
pub mod port;

pub(crate) mod raw_sample;

/// The payload that is received by a [`crate::port::subscriber::Subscriber`].
pub mod sample;

/// The payload that is sent by a [`crate::port::publisher::Publisher`].
pub mod sample_mut;

/// The foundation of communication the service with its
/// [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)
pub mod service;

/// Loads a meaninful subset to cover 90% of the iceoryx2 communication use cases.
pub mod prelude;
