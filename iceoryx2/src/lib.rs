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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]
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
//! - Request-Response
//! - Blackboard (planned)
//! - Pipeline (planned)
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! while node.wait(CYCLE_TIME).is_ok() {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! while node.wait(CYCLE_TIME).is_ok() {
//!     let sample = publisher.loan_uninit()?;
//!     let sample = sample.write_payload(1234);
//!     sample.send()?;
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Request-Response
//!
//! This is a simple request-response example where a client sends a request, and the server
//! responds with multiple replies until the processes are gracefully terminated by the user
//! with `CTRL+C`
//!
//! **Client (Process 1)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let client = service.client_builder().create()?;
//!
//! // sending first request by using slower, inefficient copy API
//! let mut pending_response = client.send_copy(1234)?;
//!
//! while node.wait(CYCLE_TIME).is_ok() {
//!     // acquire all responses to our request from our buffer that were sent by the servers
//!     while let Some(response) = pending_response.receive()? {
//!         println!("  received response: {:?}", *response);
//!     }
//!
//!     // send all other requests by using zero copy API
//!     let request = client.loan_uninit()?;
//!     let request = request.write_payload(5678);
//!
//!     pending_response = request.send()?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! **Server (Process 2)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! const CYCLE_TIME: Duration = Duration::from_millis(100);
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let server = service.server_builder().create()?;
//!
//! while node.wait(CYCLE_TIME).is_ok() {
//!     while let Some(active_request) = server.receive()? {
//!         println!("received request: {:?}", *active_request);
//!
//!         // use zero copy API, send out some responses to demonstrate the streaming API
//!         for n in 0..4 {
//!             let response = active_request.loan_uninit()?;
//!             let response = response.write_payload(n as _);
//!             println!("  send response: {:?}", *response);
//!             response.send()?;
//!         }
//!     }
//! }
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! while node.wait(Duration::ZERO).is_ok() {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! while node.wait(CYCLE_TIME).is_ok() {
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
//! ## Blackboard
//!
//! Explore a simple blackboard setup with one key-value pair which is continuously updated by the
//! writer and read by the reader until the processes are gracefully terminated with `CTRL+C`.
//!
//! **Reader (Process 1)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! // create our port factory by creating the service
//! type KeyType = u32;
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<KeyType>()
//!     .add::<u64>(0, 0)
//!     .create()?;
//!
//! let reader = service.reader_builder().create()?;
//! let entry_handle = reader.entry::<u64>(&0)?;
//!
//! while node.wait(CYCLE_TIME).is_ok() {
//!     println!("read: {}", entry_handle.get());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! **Writer (Process 2)**
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! // create our port factory by opening the service
//! type KeyType = u32;
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_opener::<KeyType>()
//!     .open()?;
//!
//! let writer = service.writer_builder().create()?;
//! let entry_handle_mut = writer.entry::<u64>(&0)?;
//!
//! let mut counter = 0;
//! while node.wait(CYCLE_TIME).is_ok() {
//!     counter += 1;
//!     entry_handle_mut.update_with_copy(counter);
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
//! For a detailed documentation see the
//! [`publish_subscribe::Builder`](crate::service::builder::publish_subscribe::Builder)
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"PubSubQos".try_into()?)
//!     .publish_subscribe::<u64>()
//!     // when the subscriber buffer is full the oldest data is overridden with the newest
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
//! ## Request-Response
//!
//! For a detailed documentation see the
//! [`request_response::Builder`](crate::service::builder::request_response::Builder)
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"ReqResQos".try_into()?)
//!     .request_response::<u64, u64>()
//!     // overrides the alignment of the request payload
//!     .request_payload_alignment(Alignment::new(128).unwrap())
//!     // overrides the alignment of the response payload
//!     .response_payload_alignment(Alignment::new(128).unwrap())
//!     // when the server buffer is full the oldest data is overridden with the newest
//!     .enable_safe_overflow_for_requests(true)
//!     // when the client buffer is full the oldest data is overridden with the newest
//!     .enable_safe_overflow_for_responses(true)
//!     // allows to send requests without expecting an answer
//!     .enable_fire_and_forget_requests(true)
//!     // how many requests can a client send in parallel
//!     .max_active_requests_per_client(2)
//!     // how many request payload objects can be loaned in parallel
//!     .max_loaned_requests(1)
//!     // the max buffer size for incoming responses per request
//!     .max_response_buffer_size(4)
//!     // the max number of servers
//!     .max_servers(2)
//!     // the max number of clients
//!     .max_clients(10)
//!     .create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Event
//!
//! For a detailed documentation see the
//! [`event::Builder`](crate::service::builder::event::Builder)
//!
//! ```
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//!     // optional event id that is emitted when a new notifier was created
//!     .notifier_created_event(EventId::new(999))
//!     // optional event id that is emitted when a notifier is dropped
//!     .notifier_dropped_event(EventId::new(0))
//!     // optional event id that is emitted when a notifier is identified as dead
//!     .notifier_dead_event(EventId::new(2000))
//!     // the deadline of the service defines how long a listener has to wait at most until
//!     // a signal will be received
//!     .deadline(Duration::from_secs(1))
//!     .create()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Blackboard
//!
//! For a detailed documentation see the
//! [`blackboard::Creator`](crate::service::builder::blackboard::Creator)
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! type KeyType = u64;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<KeyType>()
//!     // the maximum amount of readers of this service
//!     .max_readers(4)
//!     // the maximum amount of nodes that are able to open this service
//!     .max_nodes(5)
//!     .add::<u64>(0, 0)
//!     .create()?;
//!
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
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//!  * `dev_permissions` - The permissions of all resources will be set to read, write, execute
//!    for everyone. This shall not be used in production and is meant to be enabled in a docker
//!    environment with inconsistent user configuration.
//!  * `logger_log` - Uses the [log crate](https://crates.io/crates/log) as default log backend
//!  * `logger_tracing` - Uses the [tracing crate](https://crates.io/crates/tracing) as default log
//!    backend
//!  * `libc_platform` - Uses the [libc crate](https://crates.io/crates/libc) for the platform
//!    abstraction to simplify cross compilation. Works currently only for Linux based targets.
//!
//! # Custom Configuration
//!
//! iceoryx2 offers the flexibility to configure default quality of service settings, paths, and
//! file suffixes through a custom configuration file.
//!
//! For in-depth details and examples, please visit the
//! [GitHub config folder](https://github.com/eclipse-iceoryx/iceoryx2/tree/main/config).

extern crate alloc;

pub(crate) mod constants;

/// Handles iceoryx2s global configuration
pub mod config;

/// Central instance that owns all service entities and can handle incoming event in an event loop
pub mod node;

/// The ports or communication endpoints of iceoryx2
pub mod port;

pub(crate) mod raw_sample;

/// Represents a "connection" to a [`Client`](crate::port::client::Client) that corresponds to a
/// previously received [`RequestMut`](crate::request_mut::RequestMut).
pub mod active_request;

/// Represents a "connection" to a [`Server`](crate::port::server::Server) that corresponds to a
/// previously sent [`RequestMut`](crate::request_mut::RequestMut).
pub mod pending_response;

/// The payload that is sent by a [`Client`](crate::port::client::Client) to a
/// [`Server`](crate::port::server::Server).
pub mod request_mut;

/// The uninitialized payload that is sent by a [`Client`](crate::port::client::Client) to a
/// [`Server`](crate::port::server::Server).
pub mod request_mut_uninit;

/// The answer a [`Client`](crate::port::client::Client) receives from a
/// [`Server`](crate::port::server::Server) on a [`RequestMut`](crate::request_mut::RequestMut).
pub mod response;

/// The answer a [`Server`](crate::port::server::Server) allocates to respond to
/// a received [`RequestMut`](crate::request_mut::RequestMut) from a
/// [`Client`](crate::port::client::Client)
pub mod response_mut;

/// The uninitialized answer a [`Server`](crate::port::server::Server) allocates to respond to
/// a received [`RequestMut`](crate::request_mut::RequestMut) from a
/// [`Client`](crate::port::client::Client)
pub mod response_mut_uninit;

/// The payload that is received by a [`Subscriber`](crate::port::subscriber::Subscriber).
pub mod sample;

/// The payload that is sent by a [`Publisher`](crate::port::publisher::Publisher).
pub mod sample_mut;

/// The uninitialized payload that is sent by a [`Publisher`](crate::port::publisher::Publisher).
pub mod sample_mut_uninit;

/// The foundation of communication the service with its
/// [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)
pub mod service;

/// Defines how constructs like the [`Node`](crate::node::Node) or the
/// [`WaitSet`](crate::waitset::WaitSet) shall handle system signals.
pub mod signal_handling_mode;

/// Loads a meaninful subset to cover 90% of the iceoryx2 communication use cases.
pub mod prelude;

#[doc(hidden)]
pub mod testing;

/// Event handling mechanism to wait on multiple [`Listener`](crate::port::listener::Listener)s
/// in one call, realizing the reactor pattern. (Event multiplexer)
pub mod waitset;
