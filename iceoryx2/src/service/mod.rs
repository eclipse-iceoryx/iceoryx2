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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! let service = zero_copy::Service::new(&service_name)
//!     // define the messaging pattern
//!     .publish_subscribe()
//!     // various QoS
//!     .enable_safe_overflow(true)
//!     .subscriber_max_borrowed_samples(1)
//!     .history_size(2)
//!     .subscriber_max_buffer_size(3)
//!     .max_subscribers(4)
//!     .max_publishers(5)
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create::<u64>()?;
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
//! let event_name = ServiceName::new("MyEventName")?;
//!
//! let event = zero_copy::Service::new(&event_name)
//!     // define the messaging pattern
//!     .event()
//!     // various QoS
//!     .max_notifiers(12)
//!     .max_listeners(2)
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Publish-Subscribe With Custom Configuration
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! let mut custom_config = Config::default();
//! // adjust the global root path under which every file/directory is stored
//! custom_config.global.service.directory = "custom_path".to_string();
//!
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe_with_custom_config(&custom_config)
//!     .open_or_create::<u64>()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Event With Custom Configuration
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! let mut custom_config = Config::default();
//! // adjust the global service path under which service related files are stored
//! custom_config.global.service.directory = "custom_services".to_string();
//!
//! let service = zero_copy::Service::new(&service_name)
//!     .event_with_custom_config(&custom_config)
//!     .open_or_create()?;
//!
//! # Ok(())
//! # }
//! ```

/// The builder to create or open [`Service`]s
pub mod builder;

/// The dynamic configuration of a [`Service`]
pub mod dynamic_config;

/// Defines the message headers for various
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

/// A configuration when communicating within a single process or single address space.
pub mod process_local;

/// A configuration when communicating between different processes using posix mechanisms.
pub mod zero_copy;

pub(crate) mod config_scheme;
pub(crate) mod naming_scheme;

use std::fmt::Debug;

use crate::config;
use crate::port::event_id::EventId;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::static_config::*;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::{fail, trace, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::Event;
use iceoryx2_cal::hash::Hash;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::shared_memory::SharedMemory;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::static_storage::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;

use self::builder::Builder;
use self::dynamic_config::DecrementReferenceCounterResult;
use self::service_name::ServiceName;

/// Failure that can be reported by [`Details::does_exist()`] or
/// [`Details::does_exist_with_custom_config()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceDoesExistError {
    InsufficientPermissions,
    InternalError,
}

impl std::fmt::Display for ServiceDoesExistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ServiceDoesExistError {}

/// Failure that can be reported by [`Details::list()`] or
/// [`Details::list_with_custom_config()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceListError {
    InsufficientPermissions,
    InternalError,
}

impl std::fmt::Display for ServiceListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ServiceListError {}

/// Represents the [`Service`]s state.
#[derive(Debug)]
pub struct ServiceState<'config, Static: StaticStorage, Dynamic: DynamicStorage<DynamicConfig>> {
    pub(crate) static_config: StaticConfig,
    pub(crate) global_config: &'config config::Config,
    pub(crate) dynamic_storage: Dynamic,
    pub(crate) static_storage: Static,
}

impl<'config, Static: StaticStorage, Dynamic: DynamicStorage<DynamicConfig>>
    ServiceState<'config, Static, Dynamic>
{
    pub(crate) fn new(
        static_config: StaticConfig,
        global_config: &'config config::Config,
        dynamic_storage: Dynamic,
        static_storage: Static,
    ) -> Self {
        let new_self = Self {
            static_config,
            global_config,
            dynamic_storage,
            static_storage,
        };
        trace!(from new_self, "open service");
        new_self
    }
}

impl<'config, Static: StaticStorage, Dynamic: DynamicStorage<DynamicConfig>> Drop
    for ServiceState<'config, Static, Dynamic>
{
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

/// Represents a service. Used to create or open new services with the [`Builder`].
pub trait Service: Sized {
    type Type<'a>: Details<'a>;

    /// Creates a new [`Builder`] for a given service name
    fn new(name: &ServiceName) -> Builder<Self> {
        Builder::new(name)
    }
}

/// Contains the building blocks a [`Service`] requires to create the underlying resources and
/// establish communication.
pub trait Details<'config>: Debug + Sized {
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
    type Event: Event<EventId>;

    #[doc(hidden)]
    fn from_state(state: ServiceState<'config, Self::StaticStorage, Self::DynamicStorage>) -> Self;

    #[doc(hidden)]
    fn state(&self) -> &ServiceState<'config, Self::StaticStorage, Self::DynamicStorage>;

    #[doc(hidden)]
    fn state_mut(
        &mut self,
    ) -> &mut ServiceState<'config, Self::StaticStorage, Self::DynamicStorage>;

    /// Checks if a service with the name exists.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = ServiceName::new("Some/Name")?;
    /// let does_name_exist = zero_copy::Service::does_exist(&name)?;
    /// # Ok(())
    /// # }
    /// ```
    fn does_exist(service_name: &ServiceName) -> Result<bool, ServiceDoesExistError> {
        Self::does_exist_with_custom_config(service_name, config::Config::get_global_config())
    }

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
    /// let does_name_exist = zero_copy::Service::does_exist_with_custom_config(&name, &custom_config)?;
    /// # Ok(())
    /// # }
    /// ```
    fn does_exist_with_custom_config(
        service_name: &ServiceName,
        config: &'config config::Config,
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

            if service_config.service_name() == service_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns a list of all created services in the system.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let services = zero_copy::Service::list()?;
    ///
    /// for service in services {
    ///     println!("\n{:#?}", &service);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list() -> Result<Vec<StaticConfig>, ServiceListError> {
        Self::list_with_custom_config(config::Config::get_global_config())
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
    /// let services = zero_copy::Service::list_with_custom_config(&custom_config)?;
    ///
    /// for service in services {
    ///     println!("\n{:#?}", &service);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list_with_custom_config(
        config: &'config config::Config,
    ) -> Result<Vec<StaticConfig>, ServiceListError> {
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
