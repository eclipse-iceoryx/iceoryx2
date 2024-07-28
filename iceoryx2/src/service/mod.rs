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
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
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
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let event = node.service_builder(&"MyEventName".try_into()?)
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
//!     .create::<ipc::Service>()?;
//!
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
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
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service_creator = node.service_builder(&"My/Funk/ServiceName".try_into()?)
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
//! let service_open = node.service_builder(&"My/Funk/ServiceName".try_into()?)
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
/// [`MessagingPattern`]s
pub mod header;

/// The messaging patterns with their custom
/// [`StaticConfig`]
pub mod messaging_pattern;

/// After the [`Service`] is created the user owns this factory to create the endpoints of the
/// [`MessagingPattern`], also known as ports.
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
pub mod local;

/// A configuration when communicating between different processes using posix mechanisms.
pub mod ipc;

pub(crate) mod config_scheme;
pub(crate) mod naming_scheme;

use std::fmt::Debug;
use std::sync::Arc;

use crate::config;
use crate::node::{NodeId, NodeListFailure, NodeState, SharedNode};
use crate::service::config_scheme::dynamic_config_storage_config;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::naming_scheme::dynamic_config_storage_name;
use crate::service::static_config::*;
use config_scheme::service_tag_config;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::{debug, fail, trace, warn};
use iceoryx2_cal::dynamic_storage::{
    DynamicStorage, DynamicStorageBuilder, DynamicStorageOpenError,
};
use iceoryx2_cal::event::Event;
use iceoryx2_cal::hash::*;
use iceoryx2_cal::monitoring::Monitoring;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::shared_memory::SharedMemory;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::static_storage::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;
use naming_scheme::service_tag_name;

use self::dynamic_config::DeregisterNodeState;
use self::messaging_pattern::MessagingPattern;
use self::service_name::ServiceName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServiceRemoveNodeError {
    VersionMismatch,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServiceRemoveTagError {
    AlreadyRemoved,
    InternalError,
    InsufficientPermissions,
}

/// Failure that can be reported when the [`ServiceDetails`] are acquired with [`Service::details()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceDetailsError {
    /// The underlying static [`Service`] information could not be opened.
    FailedToOpenStaticServiceInfo,
    /// The underlying static [`Service`] information could not be read.
    FailedToReadStaticServiceInfo,
    /// The underlying static [`Service`] information could not be deserialized. Can be caused by
    /// version mismatch or a corrupted file.
    FailedToDeserializeStaticServiceInfo,
    /// Required [`Service`] resources are not available or corrupted.
    ServiceInInconsistentState,
    /// The [`Service`] was created with a different iceoryx2 version.
    VersionMismatch,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
    /// The [`NodeState`] could not be acquired.
    FailedToAcquireNodeState,
}

impl std::fmt::Display for ServiceDetailsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "ServiceDetailsError::{:?}", self)
    }
}

impl std::error::Error for ServiceDetailsError {}

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

/// Represents all the [`Service`] information that one can acquire with [`Service::list()`]
/// when the [`Service`] is accessible by the current process.
#[derive(Debug)]
pub struct ServiceDynamicDetails<S: Service> {
    /// A list of all [`Node`](crate::node::Node)s that a registered at the [`Service`]
    pub nodes: Vec<NodeState<S>>,
}

/// Represents all the [`Service`] information that one can acquire with [`Service::list()`].
#[derive(Debug)]
pub struct ServiceDetails<S: Service> {
    /// The static configuration of the [`Service`] that never changes during the [`Service`]
    /// lifetime.
    pub static_details: StaticConfig,
    /// The dynamic configuration of the [`Service`] that can conaints runtime informations.
    pub dynamic_details: Option<ServiceDynamicDetails<S>>,
}

/// Represents the [`Service`]s state.
#[derive(Debug)]
pub struct ServiceState<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) shared_node: Arc<SharedNode<S>>,
    pub(crate) dynamic_storage: S::DynamicStorage,
    pub(crate) static_storage: S::StaticStorage,
}

impl<S: Service> ServiceState<S> {
    pub(crate) fn new(
        static_config: StaticConfig,
        shared_node: Arc<SharedNode<S>>,
        dynamic_storage: S::DynamicStorage,
        static_storage: S::StaticStorage,
    ) -> Self {
        let new_self = Self {
            static_config,
            shared_node,
            dynamic_storage,
            static_storage,
        };
        trace!(from "Service::open()", "open service: {} (uuid={:?})",
            new_self.static_config.name(), new_self.static_config.uuid());
        new_self
    }
}

impl<S: Service> Drop for ServiceState<S> {
    fn drop(&mut self) {
        let origin = "ServiceState::drop()";
        let id = self.static_config.uuid();
        self.shared_node.registered_services().remove(id, |handle| {
            if let Err(e) = remove_service_tag::<S>(self.shared_node.id(), id, self.shared_node.config())
            {
                debug!(from origin, "The service tag could not be removed from the node {:?} ({:?}).",
                        self.shared_node.id(), e);
            }

            match self.dynamic_storage.get().deregister_node_id(handle) {
                DeregisterNodeState::HasOwners => {
                    trace!(from origin, "close service: {} (uuid={:?})",
                            self.static_config.name(), id);
                }
                DeregisterNodeState::NoMoreOwners => {
                    self.static_storage.acquire_ownership();
                    self.dynamic_storage.acquire_ownership();
                    trace!(from origin, "close and remove service: {} (uuid={:?})",
                            self.static_config.name(), id);
                }
            }
        });
    }
}

pub(crate) mod internal {
    use config_scheme::static_config_storage_config;
    use dynamic_config::{PortCleanupAction, RemoveDeadNodeResult};
    use naming_scheme::static_config_storage_name;

    use crate::{
        node::NodeId,
        port::{
            listener::remove_connection_of_listener,
            port_identifiers::UniquePortId,
            publisher::{
                remove_data_segment_of_publisher, remove_publisher_from_all_connections,
                remove_subscriber_from_all_connections,
            },
        },
    };

    use super::*;

    pub(crate) trait ServiceInternal<S: Service> {
        fn __internal_from_state(state: ServiceState<S>) -> S;

        fn __internal_state(&self) -> &Arc<ServiceState<S>>;

        fn __internal_remove_node_from_service(
            node_id: &NodeId,
            service_uuid: &FileName,
            config: &config::Config,
        ) -> Result<(), ServiceRemoveNodeError> {
            let origin = format!(
                "Service::remove_node_from_service({:?}, {:?})",
                node_id, service_uuid
            );
            let msg = "Unable to remove node from service";

            let dynamic_config = match open_dynamic_config::<S>(
                config,
                core::str::from_utf8(service_uuid.as_bytes()).unwrap(),
            ) {
                Ok(Some(c)) => c,
                Ok(None) => return Ok(()),
                Err(ServiceDetailsError::VersionMismatch) => {
                    fail!(from origin, with ServiceRemoveNodeError::VersionMismatch,
                        "{} since the service version does not match.", msg);
                }
                Err(e) => {
                    fail!(from origin, with ServiceRemoveNodeError::InternalError,
                        "{} due to an internal failure ({:?}).", msg, e);
                }
            };

            let cleanup_port_resources = |port_id| {
                match port_id {
                    UniquePortId::Publisher(ref id) => {
                        if let Err(e) =
                            unsafe { remove_publisher_from_all_connections::<S>(id, config) }
                        {
                            debug!(from origin, "Failed to remove the publishers ({:?}) from all of its connections ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }

                        if let Err(e) = unsafe { remove_data_segment_of_publisher::<S>(id, config) }
                        {
                            debug!(from origin, "Failed to remove the publishers ({:?}) data segment ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Subscriber(ref id) => {
                        if let Err(e) =
                            unsafe { remove_subscriber_from_all_connections::<S>(id, config) }
                        {
                            debug!(from origin, "Failed to remove the subscriber ({:?}) from all of its connections ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Notifier(_) => (),
                    UniquePortId::Listener(ref id) => {
                        if let Err(e) = unsafe { remove_connection_of_listener::<S>(id, config) } {
                            debug!(from origin, "Failed to remove the listeners ({:?}) connection ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }
                    }
                };

                debug!(from origin, "Remove port {:?} from service.", port_id);
                PortCleanupAction::RemovePort
            };

            let remove_service = match unsafe {
                dynamic_config
                    .get()
                    .remove_dead_node_id(node_id, cleanup_port_resources)
            } {
                Ok(DeregisterNodeState::HasOwners) => false,
                Ok(DeregisterNodeState::NoMoreOwners) => true,
                Err(RemoveDeadNodeResult::NodeNotRegistered) => {
                    dynamic_config.get().is_marked_for_destruction()
                }
            };

            if remove_service {
                match unsafe {
                    <S::StaticStorage as NamedConceptMgmt>::remove_cfg(
                        &static_config_storage_name(
                            core::str::from_utf8(service_uuid.as_bytes()).unwrap(),
                        ),
                        &static_config_storage_config::<S>(config),
                    )
                } {
                    Ok(_) => {
                        debug!(from origin, "Remove unused service.");
                        dynamic_config.acquire_ownership()
                    }
                    Err(e) => {
                        warn!(from origin, "Unable to remove static config of unused service ({:?}).",
                            e);
                    }
                }
            }

            Ok(())
        }
    }
}

/// Represents a service. Used to create or open new services with the
/// [`crate::node::Node::service_builder()`].
/// Contains the building blocks a [`Service`] requires to create the underlying resources and
/// establish communication.
#[allow(private_bounds)]
pub trait Service: Debug + Sized + internal::ServiceInternal<Self> {
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
    /// let does_name_exist =
    ///     ipc::Service::does_exist(
    ///                 &name,
    ///                 Config::global_config(),
    ///                 MessagingPattern::Event)?;
    /// # Ok(())
    /// # }
    /// ```
    fn does_exist(
        service_name: &ServiceName,
        config: &config::Config,
        messaging_pattern: MessagingPattern,
    ) -> Result<bool, ServiceDetailsError> {
        Ok(Self::details(service_name, config, messaging_pattern)?.is_some())
    }

    /// Acquires the [`ServiceDetails`] of a [`Service`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = ServiceName::new("Some/Name")?;
    /// let details =
    ///     ipc::Service::details(
    ///                 &name,
    ///                 Config::global_config(),
    ///                 MessagingPattern::Event)?;
    ///
    /// if let Some(details) = details {
    ///     println!("Service details: {:?}", details);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn details(
        service_name: &ServiceName,
        config: &config::Config,
        messaging_pattern: MessagingPattern,
    ) -> Result<Option<ServiceDetails<Self>>, ServiceDetailsError> {
        let uuid = unsafe {
            FileName::new_unchecked(
                <HashValue as Into<String>>::into(
                    create_uuid::<Self::ServiceNameHasher>(service_name, messaging_pattern).value(),
                )
                .as_bytes(),
            )
        };

        details::<Self>(config, &uuid)
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
    /// ipc::Service::list(Config::global_config(), |service| {
    ///     println!("\n{:#?}", &service);
    ///     CallbackProgression::Continue
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    fn list<F: FnMut(ServiceDetails<Self>) -> CallbackProgression>(
        config: &config::Config,
        mut callback: F,
    ) -> Result<(), ServiceListError> {
        let msg = "Unable to list all services";
        let origin = "Service::list_from_config()";
        let static_storage_config = config_scheme::static_config_storage_config::<Self>(config);

        let service_uuids = fail!(from origin,
                when <Self::StaticStorage as NamedConceptMgmt>::list_cfg(&static_storage_config),
                map NamedConceptListError::InsufficientPermissions => ServiceListError::InsufficientPermissions,
                unmatched ServiceListError::InternalError,
                "{} due to a failure while collecting all active services for config: {:?}", msg, config);

        for uuid in &service_uuids {
            if let Ok(Some(service_details)) = details::<Self>(config, uuid) {
                if callback(service_details) == CallbackProgression::Stop {
                    break;
                }
            }
        }

        Ok(())
    }
}

fn details<S: Service>(
    config: &config::Config,
    uuid: &FileName,
) -> Result<Option<ServiceDetails<S>>, ServiceDetailsError> {
    let msg = "Unable to acquire servic details";
    let origin = "Service::details()";
    let static_storage_config = config_scheme::static_config_storage_config::<S>(config);

    let reader = match <<S::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
        S::StaticStorage,
    >>::new(uuid)
    .config(&static_storage_config.clone())
    .has_ownership(false)
    .open()
    {
        Ok(reader) => reader,
        Err(StaticStorageOpenError::DoesNotExist) => return Ok(None),
        Err(e) => {
            fail!(from origin, with ServiceDetailsError::FailedToOpenStaticServiceInfo,
                        "{} due to a failure while opening the static service info \"{}\" for reading ({:?})",
                        msg, uuid, e);
        }
    };

    let mut content = String::from_utf8(vec![b' '; reader.len() as usize]).unwrap();
    if let Err(e) = reader.read(unsafe { content.as_mut_vec().as_mut_slice() }) {
        fail!(from origin, with ServiceDetailsError::FailedToReadStaticServiceInfo,
                "{} since the static service info \"{}\" could not be read ({:?}).",
                msg, uuid, e );
    }

    let service_config =
        match S::ConfigSerializer::deserialize::<StaticConfig>(unsafe { content.as_mut_vec() }) {
            Ok(service_config) => service_config,
            Err(e) => {
                fail!(from origin, with ServiceDetailsError::FailedToDeserializeStaticServiceInfo,
                    "{} since the static service info \"{}\" could not be deserialized ({:?}).",
                       msg, uuid, e );
            }
        };

    if uuid.as_bytes() != service_config.uuid().as_bytes() {
        fail!(from origin, with ServiceDetailsError::ServiceInInconsistentState,
                "{} since the service {:?} has an inconsistent hash of {} according to config {:?}",
                msg, service_config, uuid, config);
    }

    let dynamic_config = open_dynamic_config::<S>(config, service_config.uuid())?;
    let dynamic_details = if let Some(d) = dynamic_config {
        let mut nodes = vec![];
        d.get().list_node_ids(|node_id| {
            match NodeState::new(node_id, config) {
                Ok(Some(state)) => nodes.push(state),
                Ok(None)
                | Err(NodeListFailure::InsufficientPermissions)
                | Err(NodeListFailure::Interrupt) => (),
                Err(NodeListFailure::InternalError) => {
                    debug!(from origin, "Unable to acquire NodeState for service \"{:?}\"", uuid);
                }
            };
            CallbackProgression::Continue
        });
        Some(ServiceDynamicDetails { nodes })
    } else {
        None
    };

    Ok(Some(ServiceDetails {
        static_details: service_config,
        dynamic_details,
    }))
}

fn open_dynamic_config<S: Service>(
    config: &config::Config,
    service_uuid: &str,
) -> Result<Option<S::DynamicStorage>, ServiceDetailsError> {
    let origin = format!(
        "Service::open_dynamic_details<{}>(service_uuid: {})",
        core::any::type_name::<S>(),
        service_uuid
    );
    let msg = "Unable to open the services dynamic config";
    match
            <<S::DynamicStorage as DynamicStorage<
                    DynamicConfig,
                >>::Builder<'_> as NamedConceptBuilder<
                    S::DynamicStorage,
                >>::new(&dynamic_config_storage_name(service_uuid))
                    .config(&dynamic_config_storage_config::<S>(config))
                .has_ownership(false)
                .open() {
            Ok(storage) => Ok(Some(storage)),
            Err(DynamicStorageOpenError::DoesNotExist) | Err(DynamicStorageOpenError::InitializationNotYetFinalized) => Ok(None),
            Err(DynamicStorageOpenError::VersionMismatch) => {
                fail!(from origin, with ServiceDetailsError::VersionMismatch,
                    "{} since there is a version mismatch. Please use the same iceoryx2 version for the whole system.", msg);
            }
            Err(DynamicStorageOpenError::InternalError) => {
                fail!(from origin, with ServiceDetailsError::InternalError,
                    "{} due to an internal failure while opening the services dynamic config.", msg);
            }
    }
}

pub(crate) fn remove_service_tag<S: Service>(
    node_id: &NodeId,
    service_uuid: &str,
    config: &config::Config,
) -> Result<(), ServiceRemoveTagError> {
    let origin = format!(
        "remove_service_tag<{}>({:?}, service_uuid: {:?})",
        core::any::type_name::<S>(),
        node_id,
        service_uuid
    );

    match unsafe {
        <S::StaticStorage as NamedConceptMgmt>::remove_cfg(
            &service_tag_name(service_uuid),
            &service_tag_config::<S>(config, node_id),
        )
    } {
        Ok(true) => Ok(()),
        Ok(false) => {
            fail!(from origin, with ServiceRemoveTagError::AlreadyRemoved,
                    "The service's tag for the node was already removed. This may indicate a corrupted system!");
        }
        Err(NamedConceptRemoveError::InternalError) => {
            fail!(from origin, with ServiceRemoveTagError::InternalError,
                "Unable to remove the service's tag for the node due to an internal error.");
        }
        Err(NamedConceptRemoveError::InsufficientPermissions) => {
            fail!(from origin, with ServiceRemoveTagError::InsufficientPermissions,
                "Unable to remove the service's tag for the node due to insufficient permissions.");
        }
    }
}
