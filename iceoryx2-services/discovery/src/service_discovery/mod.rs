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

//! # Service Discovery
//!
//! This module provides functionality for discovering and tracking services in an iceoryx2 system.
//!
//! The service discovery system consists of two main components:
//!
//! 1. **Service**: A service that tracks and publishes information about other services in the system.
//!    It detects when services are added or removed and notifies interested parties about these changes.
//!
//! 2. **Tracker**: A component that keeps track of services in the system. It maintains a list of
//!    currently available services and can detect changes in the service landscape.
//!
//! ## Usage
//!
//! To use the service discovery system, you typically create a `Service` instance with appropriate
//! configuration, and then periodically call its `spin` method to process service changes and emit
//! events/notifications.
//!
//! ```rust,no_run
//! use iceoryx2_services_discovery::service_discovery::Service;
//! use iceoryx2_services_discovery::service_discovery::Config as DiscoveryConfig;
//! use iceoryx2::prelude::*;
//!
//! // Create a service discovery service
//! let config = DiscoveryConfig::default();
//! let mut service = Service::<ipc::Service>::create(&config, &Config::global_config()).expect("Failed to create service");
//!
//! // Periodically process service changes
//! loop {
//!     match service.spin() {
//!         Ok((added, removed)) => {
//!             println!("Added {} services, removed {} services", added.len(), removed.len());
//!         }
//!         Err(e) => {
//!             eprintln!("Error spinning service: {:?}", e);
//!             break;
//!         }
//!     }
//!     // Sleep or do other work...
//! }

/// A service discovery service that tracks and publishes information about services in the system.
mod service;

/// A tracker for services that maintains a list of currently available services.
mod tracker;

pub use service::*;
pub use tracker::*;
