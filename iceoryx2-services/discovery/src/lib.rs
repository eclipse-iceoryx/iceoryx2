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

#![warn(missing_docs)]

//! # iceoryx2-discovery
//!
//! A library for service discovery and monitoring in iceoryx2 applications.
//!
//! This crate provides utilities for discovering, tracking, and monitoring services
//! in an iceoryx2 system. It enables applications to detect when services become
//! available or unavailable, and to react to these changes.
//!
//! ## Features
//!
//! - **Service Tracking**: Track the availability of services in the system
//! - **Service Monitoring**: Monitor services and receive notifications when they appear or disappear
//!
//! ## Usage Examples
//!
//! ### Tracking Services
//!
//! ```rust
//! use iceoryx2::config::Config;
//! use iceoryx2_discovery::service::Tracker;
//! use iceoryx2::service::YourServiceType;
//!
//! // Create a new tracker for your service type
//! let mut tracker = Tracker::<YourServiceType>::new();
//!
//! // Sync with the system to discover services
//! let (added_services, removed_services) = tracker.sync(&Config::global_config());
//!
//! // Process newly discovered services
//! for service_id in added_services {
//!     if let Some(service_details) = tracker.get(&service_id) {
//!         println!("Discovered service: {:?}", service_details.static_details.name());
//!     }
//! }
//! ```
//!
//! ### Monitoring Services
//!
//! ```rust
//! use iceoryx2_discovery::service::Monitor;
//! use iceoryx2::service::YourServiceType;
//!
//! // Create a new service monitor
//! let mut monitor = Monitor::<YourServiceType>::new();
//!
//! // Periodically call spin() to detect and publish service changes
//! loop {
//!     monitor.spin();
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//! }
//! ```
//!
//! ### Subscribing to Service Discovery Events
//!
//! ```rust
//! use iceoryx2::config::Config;
//! use iceoryx2::node::{Node, NodeBuilder};
//! use iceoryx2::port::subscriber::Subscriber;
//! use iceoryx2::prelude::ServiceName;
//! use iceoryx2_discovery::service::DiscoveryEvent;
//! use iceoryx2::service::YourServiceType;
//!
//! // Create a node
//! let node = NodeBuilder::new()
//!     .config(Config::global_config())
//!     .create::<YourServiceType>()
//!     .expect("Failed to create node");
//!
//! // Create a subscriber for discovery events
//! let service_name = ServiceName::new("iox2://monitor/services")
//!     .expect("Failed to create service name");
//!
//! let publish_subscribe = node
//!     .service_builder(&service_name)
//!     .publish_subscribe::<DiscoveryEvent>()
//!     .create()
//!     .expect("Failed to create publish-subscribe service");
//!
//! let subscriber = publish_subscribe
//!     .subscriber_builder()
//!     .create()
//!     .expect("Failed to create subscriber");
//!
//! // Process discovery events
//! while let Some(event) = subscriber.receive() {
//!     match event.payload() {
//!         DiscoveryEvent::Added(config) => println!("Service added: {:?}", config.name()),
//!         DiscoveryEvent::Removed(config) => println!("Service removed: {:?}", config.name()),
//!     }
//! }
//! ```
//!
//! ## Integration with iceoryx2
//!
//! This crate is designed to work seamlessly with the core iceoryx2 library and provides
//! a higher-level API for service discovery. It builds upon the service management
//! capabilities of iceoryx2 to provide a more convenient interface for tracking and
//! monitoring services.

/// Utilities for service discovery and monitoring
pub mod service;
