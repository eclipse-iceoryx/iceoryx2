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

//! # Example
//!
//! ## Publish-Subscribe
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//!
//! let service = node.service_builder("My/Funk/ServiceName".try_into()?)
//!     // define the messaging pattern
//!     .publish_subscribe::<u64>()
//!     // various QoS
//!     .enable_safe_overflow(true)
//!     .subscriber_max_borrowed_samples(1)
//!     .history_size(2)
//!     .subscriber_max_buffer_size(3)
//!     .max_subscribers(4)
//!     .max_publishers(5)
//!     // increase the alignment of the payload to 512, interesting for SIMD operations
//!     .payload_alignment(Alignment::new(512).unwrap())
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create()?;
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
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//!
//! let event = node.service_builder("MyEventName".try_into()?)
//!     // define the messaging pattern
//!     .event()
//!     // various QoS
//!     .max_notifiers(12)
//!     .max_listeners(2)
//!     .event_id_max_value(32)
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Service With Custom Configuration
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2_bb_system_types::path::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut custom_config = Config::default();
//! // adjust the global root path under which every file/directory is stored
//! custom_config.global.service.directory = "custom_path".try_into()?;
//!
//! let node = NodeBuilder::new()
//!     .config(&custom_config)
//!     .create::<zero_copy::Service>()?;
//!
//! let service = node.service_builder("My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Publish-Subscribe With Custom Service Attributes
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//!
//! let service_creator = node.service_builder("My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .create_with_attributes(
//!         // all attributes that are defined when creating a new service are stored in the
//!         // static config of the service
//!         &AttributeSpecifier::new()
//!             .define("some attribute key", "some attribute value")
//!             .define("some attribute key", "another attribute value for the same key")
//!             .define("another key", "another value")
//!     )?;
//!
//! let service_open = node.service_builder("My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_with_attributes(
//!         // All attributes that are defined when opening a new service interpreted as
//!         // requirements.
//!         // If a attribute key as either a different value or is not set at all, the service
//!         // cannot be opened. If not specific attributes are required one can skip them completely.
//!         &AttributeVerifier::new()
//!             .require("another key", "another value")
//!             .require_key("some attribute key")
//!     )?;
//!
//! # Ok(())
//! # }
//! ```

/// The builder to create or open [`Service`]s
pub mod builder;

/// The dynamic configuration of a [`Service`]
pub mod dynamic_config;

/// Defines the sample headers for various
/// [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern)s
pub mod header;

/// The messaging patterns with their custom
/// [`StaticConfig`]
pub mod messaging_pattern;

/// After the [`Service`] is created the user owns this factory to create the endpoints of the
/// [`MessagingPattern`](crate::service::messaging_pattern::MessagingPattern), also known as ports.
pub mod port_factory;

/// Represents the name of a [`Service`]
pub mod service_name;

/// Represents the static configuration of a [`Service`]. These are the settings that never change
/// during the runtime of a service, like:
///
///  * name
///  * data type
///  * QoS provided when the service was created
pub mod static_config;

/// Represents static features of a service that can be set when a [`Service`] is created.
pub mod attribute;

/// A configuration when communicating within a single process or single address space.
pub mod process_local;

/// A configuration when communicating between different processes using posix mechanisms.
pub mod zero_copy;

pub(crate) mod config_scheme;
pub(crate) mod naming_scheme;

use std::fmt::Debug;
use std::sync::Arc;

use crate::config;
use crate::node::SharedNode;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::static_config::*;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::{fail, trace, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::Event;
use iceoryx2_cal::hash::Hash;
use iceoryx2_cal::monitoring::Monitoring;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::shared_memory::SharedMemory;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::static_storage::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;

use self::dynamic_config::DecrementReferenceCounterResult;
use self::service_name::ServiceName;

/// Failure that can be reported by [`Service::does_exist()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceDoesExistError {
    /// The process has insufficient permissions to check if a [`Service`] exists.
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
}

impl std::fmt::Display for ServiceDoesExistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "ServiceDoesExistError::{:?}", self)
    }
}

impl std::error::Error for ServiceDoesExistError {}

/// Failure that can be reported by [`Service::list()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceListError {
    /// The process has insufficient permissions to list all [`Service`]s.
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
}

impl std::fmt::Display for ServiceListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "ServiceListError::{:?}", self)
    }
}

impl std::error::Error for ServiceListError {}

/// Represents the [`Service`]s state.
#[derive(Debug)]
pub struct ServiceState<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) shared_node: Arc<SharedNode<S>>,
    pub(crate) dynamic_storage: Arc<S::DynamicStorage>,
    pub(crate) static_storage: S::StaticStorage,
}

impl<S: Service> ServiceState<S> {
    pub(crate) fn new(
        static_config: StaticConfig,
        shared_node: Arc<SharedNode<S>>,
        dynamic_storage: Arc<S::DynamicStorage>,
        static_storage: S::StaticStorage,
    ) -> Self {
        let new_self = Self {
            static_config,
            shared_node,
            dynamic_storage,
            static_storage,
        };
        trace!(from new_self, "open service");
        new_self
    }
}

impl<S: Service> Drop for ServiceState<S> {
    fn drop(&mut self) {
        match self.dynamic_storage.get().decrement_reference_counter() {
            DecrementReferenceCounterResult::HasOwners => {
                trace!(from self, "close service");
            }
            DecrementReferenceCounterResult::NoMoreOwners => {
                self.static_storage.acquire_ownership();
                self.dynamic_storage.acquire_ownership();
                trace!(from self, "close and remove service");
            }
        }
    }
}

/// Represents a service. Used to create or open new services with the
/// [`crate::node::Node::service_builder()`].
/// Contains the building blocks a [`Service`] requires to create the underlying resources and
/// establish communication.
pub trait Service: Debug + Sized {
    /// Every service name will be hashed, to allow arbitrary [`ServiceName`]s with as less
    /// restrictions as possible. The hash of the [`ServiceName`] is the [`Service`]s uuid.
    type ServiceNameHasher: Hash;

    /// Defines the construct that is used to store the [`StaticConfig`] of the [`Service`]
    type StaticStorage: StaticStorage;

    /// Sets the serializer that is used to serialize the [`StaticConfig`] into the [`StaticStorage`]
    type ConfigSerializer: Serialize;

    /// Defines the construct used to store the [`Service`]s dynamic configuration. This
    /// contains for instance all endpoints and other dynamic details.
    type DynamicStorage: DynamicStorage<DynamicConfig>;

    /// The memory used to store the payload.
    type SharedMemory: SharedMemory<PoolAllocator>;

    /// The connection used to exchange pointers to the payload
    type Connection: ZeroCopyConnection;

    /// The mechanism used to signal events between endpoints.
    type Event: Event;

    /// Monitoring mechanism to detect dead processes.
    type Monitoring: Monitoring;

    #[doc(hidden)]
    fn from_state(state: ServiceState<Self>) -> Self;

    #[doc(hidden)]
    fn state(&self) -> &ServiceState<Self>;

    #[doc(hidden)]
    fn state_mut(&mut self) -> &mut ServiceState<Self>;

    /// Checks if a service under a given [`config::Config`] does exist
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = ServiceName::new("Some/Name")?;
    /// let mut custom_config = Config::default();
    /// let does_name_exist = zero_copy::Service::does_exist(&name, &custom_config)?;
    /// # Ok(())
    /// # }
    /// ```
    fn does_exist(
        service_name: &ServiceName,
        config: &config::Config,
    ) -> Result<bool, ServiceDoesExistError> {
        let msg = format!("Unable to verify if \"{}\" exists", service_name);
        let origin = "Service::does_exist_from_config()";
        let static_storage_config = config_scheme::static_config_storage_config::<Self>(config);

        let services = fail!(from origin,
                 when <Self::StaticStorage as NamedConceptMgmt>::list_cfg(&static_storage_config),
                 map NamedConceptListError::InsufficientPermissions => ServiceDoesExistError::InsufficientPermissions,
                 unmatched ServiceDoesExistError::InternalError,
                 "{} due to a failure while collecting all active services for config: {:?}", msg, config);

        for service_storage in services {
            let reader =
                match <<Self::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                    Self::StaticStorage,
                >>::new(&service_storage)
                .config(&static_storage_config.clone())
                .has_ownership(false)
                .open()
                {
                    Ok(reader) => reader,
                    Err(e) => {
                        warn!(from origin, "Unable to open service static info \"{}\" for reading ({:?}). Maybe unable to determin if the service \"{}\" exists.",
                            service_storage, e, service_name);
                        continue;
                    }
                };

            let mut content = String::from_utf8(vec![b' '; reader.len() as usize]).unwrap();
            if let Err(e) = reader.read(unsafe { content.as_mut_vec().as_mut_slice() }) {
                warn!(from origin, "Unable to read service static info \"{}\" - error ({:?}). Maybe unable to determin if the service \"{}\" exists.",
                            service_storage, e, service_name);
            }

            let service_config = match Self::ConfigSerializer::deserialize::<StaticConfig>(unsafe {
                content.as_mut_vec()
            }) {
                Ok(service_config) => service_config,
                Err(e) => {
                    warn!(from origin, "Unable to deserialize service static info \"{}\" - error ({:?}). Maybe unable to determin if the service \"{}\" exists.",
                            service_storage, e, service_name);
                    continue;
                }
            };

            if service_storage.as_bytes() != service_config.uuid().as_bytes() {
                warn!(from origin, "Detected service {:?} with an inconsistent hash of {} when acquiring services according to config {:?}",
                    service_config, service_storage, config);
                continue;
            }

            if service_config.name() == service_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns a list of all services created under a given [`config::Config`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut custom_config = Config::default();
    /// let services = zero_copy::Service::list(&custom_config)?;
    ///
    /// for service in services {
    ///     println!("\n{:#?}", &service);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list(config: &config::Config) -> Result<Vec<StaticConfig>, ServiceListError> {
        let msg = "Unable to list all services";
        let origin = "Service::list_from_config()";
        let static_storage_config = config_scheme::static_config_storage_config::<Self>(config);

        let services = fail!(from origin,
                when <Self::StaticStorage as NamedConceptMgmt>::list_cfg(&static_storage_config),
                map NamedConceptListError::InsufficientPermissions => ServiceListError::InsufficientPermissions,
                unmatched ServiceListError::InternalError,
                "{} due to a failure while collecting all active services for config: {:?}", msg, config);

        let mut service_vec = vec![];
        for service_storage in services {
            let reader =
                match <<Self::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
                    Self::StaticStorage,
                >>::new(&service_storage)
                .config(&static_storage_config.clone())
                .has_ownership(false)
                .open()
                {
                    Ok(reader) => reader,
                    Err(e) => {
                        warn!(from origin, "Unable to acquire a list of all service since the static service info \"{}\" could not be opened for reading ({:?}).",
                           service_storage, e );
                        continue;
                    }
                };

            let mut content = String::from_utf8(vec![b' '; reader.len() as usize]).unwrap();
            if let Err(e) = reader.read(unsafe { content.as_mut_vec().as_mut_slice() }) {
                warn!(from origin, "Unable to acquire a list of all service since the static service info \"{}\" could not be read ({:?}).",
                           service_storage, e );
                continue;
            }

            let service_config = match Self::ConfigSerializer::deserialize::<StaticConfig>(unsafe {
                content.as_mut_vec()
            }) {
                Ok(service_config) => service_config,
                Err(e) => {
                    warn!(from origin, "Unable to acquire a list of all service since the static service info \"{}\" could not be deserialized ({:?}).",
                       service_storage, e );
                    continue;
                }
            };

            if service_storage.as_bytes() != service_config.uuid().as_bytes() {
                warn!(from origin, "Detected service {:?} with an inconsistent hash of {} when acquiring services according to config {:?}",
                    service_config, service_storage, config);
                continue;
            }

            service_vec.push(service_config);
        }

        Ok(service_vec)
    }
}
