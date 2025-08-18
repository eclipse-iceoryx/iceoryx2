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
//! For a detailed documentation see the
//! [`publish_subscribe::Builder`](crate::service::builder::publish_subscribe::Builder)
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//!     // various QoS
//!     .request_payload_alignment(Alignment::new(128).unwrap())
//!     .response_payload_alignment(Alignment::new(128).unwrap())
//!     .enable_safe_overflow_for_requests(true)
//!     .enable_safe_overflow_for_responses(true)
//!     .enable_fire_and_forget_requests(true)
//!     .max_active_requests_per_client(2)
//!     .max_loaned_requests(1)
//!     .max_response_buffer_size(4)
//!     .max_servers(2)
//!     .max_clients(10)
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create()?;
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
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     // define the messaging pattern
//!     .event()
//!     // various QoS
//!     .max_notifiers(12)
//!     .max_listeners(2)
//!     .event_id_max_value(32)
//!     .notifier_created_event(EventId::new(999))
//!     .notifier_dropped_event(EventId::new(0))
//!     .notifier_dead_event(EventId::new(2000))
//!     // if the service already exists, open it, otherwise create it
//!     .open_or_create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Service With Custom Configuration
//!
//! An individual [`Config`](crate::config::Config) can be attached when the
//! [`Node`](crate::node::Node) is created and it will be used for every construct created using
//! this [`Node`](crate::node::Node).
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2_bb_system_types::path::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! ## Service With Custom Service Attributes
//!
//! Every [`Service`](crate::service::Service) can be created with a set of attributes.
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service_creator = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .create_with_attributes(
//!         // all attributes that are defined when creating a new service are stored in the
//!         // static config of the service
//!         &AttributeSpecifier::new()
//!             .define(&"some attribute key".try_into()?, &"some attribute value".try_into()?)
//!             .define(&"some attribute key".try_into()?, &"another attribute value for the same key".try_into()?)
//!             .define(&"another key".try_into()?, &"another value".try_into()?)
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
//!             .require(&"another key".try_into()?, &"another value".try_into()?)
//!             .require_key(&"some attribute key".try_into()?)
//!     )?;
//!
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
//! use iceoryx2_bb_container::byte_string::FixedSizeByteString;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! type KeyType = u64;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     // define the messaging pattern
//!     .blackboard_creator::<KeyType>()
//!     // QoS
//!     .max_readers(4)
//!     .max_nodes(5)
//!     // add key-value pairs
//!     .add::<i32>(0, -9)
//!     .add::<bool>(5, true)
//!     .add::<FixedSizeByteString<8>>(17, "Nalalala".try_into().unwrap())
//!     .add_with_default::<u32>(2)
//!     // create the service
//!     .create()?;
//!
//! # Ok(())
//! # }
//! ```

pub(crate) mod stale_resource_cleanup;

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

/// Represents the unique id of a [`Service`]
pub mod service_id;

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

/// A threadsafe configuration when communicating within a single process or single address space.
/// All [`Service`] ports implement [`Send`] and [`Sync`], the payload constructs will implement
/// [`Send`] but at the cost of an additional internal mutex.
pub mod local_threadsafe;

/// A configuration when communicating between different processes using posix mechanisms.
pub mod ipc;

/// A threadsafe configuration when communicating between different processes using posix mechanisms.
/// All [`Service`] ports implement [`Send`] and [`Sync`], the payload constructs will implement
/// [`Send`] but at the cost of an additional internal mutex.
pub mod ipc_threadsafe;

pub(crate) mod config_scheme;
pub(crate) mod naming_scheme;

use alloc::sync::Arc;
use core::fmt::Debug;
use core::time::Duration;

use crate::config;
use crate::constants::MAX_TYPE_NAME_LENGTH;
use crate::node::{NodeId, NodeListFailure, NodeState, SharedNode};
use crate::service::config_scheme::dynamic_config_storage_config;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::static_config::*;
use config_scheme::service_tag_config;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::{debug, fail, trace, warn};
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::dynamic_storage::{
    DynamicStorage, DynamicStorageBuilder, DynamicStorageOpenError,
};
use iceoryx2_cal::event::Event;
use iceoryx2_cal::hash::*;
use iceoryx2_cal::monitoring::Monitoring;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::reactor::Reactor;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemoryForPoolAllocator;
use iceoryx2_cal::serialize::Serialize;
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryForPoolAllocator};
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;
use iceoryx2_cal::static_storage::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;
use service_id::ServiceId;

use self::dynamic_config::DeregisterNodeState;
use self::messaging_pattern::MessagingPattern;
use self::service_name::ServiceName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Error that can be reported when removing a [`Node`](crate::node::Node).
pub enum ServiceRemoveNodeError {
    /// The iceoryx2 version that created the [`Node`](crate::node::Node) does
    /// not match this iceoryx2 version.
    VersionMismatch,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
    /// The [`Node`](crate::node::Node) has opened a [`Service`] that is in a
    /// corrupted state and therefore it cannot be remove from it.
    ServiceInCorruptedState,
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

impl core::fmt::Display for ServiceDetailsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServiceDetailsError::{self:?}")
    }
}

impl core::error::Error for ServiceDetailsError {}

/// Failure that can be reported by [`Service::list()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceListError {
    /// The process has insufficient permissions to list all [`Service`]s.
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
}

impl core::fmt::Display for ServiceListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServiceListError::{self:?}")
    }
}

impl core::error::Error for ServiceListError {}

/// Represents all the [`Service`] information that one can acquire with [`Service::list()`]
/// when the [`Service`] is accessible by the current process.
#[derive(Debug, Clone)]
pub struct ServiceDynamicDetails<S: Service> {
    /// A list of all [`Node`](crate::node::Node)s that are registered at the [`Service`]
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
pub struct ServiceState<S: Service, R: ServiceResource> {
    // For this struct it is important to know that Rust drops fields of a struct in declaration
    // order - not in memory order!

    // must be destructed first, to prevent services to open it
    pub(crate) dynamic_storage: S::DynamicStorage,
    // must be destructed after the dynamic resources
    pub(crate) additional_resource: R,
    pub(crate) static_config: StaticConfig,
    pub(crate) shared_node: Arc<SharedNode<S>>,
    // must be destructed last, otherwise other processes might create a new service with the same
    // name and their resources are then removed by another process while they are creating them
    // which would end up in a completely corrupted service
    pub(crate) static_storage: S::StaticStorage,
}

impl<S: Service, R: ServiceResource> ServiceState<S, R> {
    pub(crate) fn new(
        static_config: StaticConfig,
        shared_node: Arc<SharedNode<S>>,
        dynamic_storage: S::DynamicStorage,
        static_storage: S::StaticStorage,
        additional_resource: R,
    ) -> Self {
        let new_self = Self {
            static_config,
            shared_node,
            dynamic_storage,
            static_storage,
            additional_resource,
        };
        trace!(from "Service::open()", "open service: {} ({:?})",
            new_self.static_config.name(), new_self.static_config.service_id());
        new_self
    }
}

impl<S: Service, R: ServiceResource> Drop for ServiceState<S, R> {
    fn drop(&mut self) {
        let origin = "ServiceState::drop()";
        let id = self.static_config.service_id();
        self.shared_node.registered_services().remove(id, |handle| {
            if let Err(e) = remove_service_tag::<S>(self.shared_node.id(), id, self.shared_node.config())
            {
                debug!(from origin, "The service tag could not be removed from the node {:?} ({:?}).",
                        self.shared_node.id(), e);
            }

            match self.dynamic_storage.get().deregister_node_id(handle) {
                DeregisterNodeState::HasOwners => {
                    trace!(from origin, "close service: {} ({:?})",
                            self.static_config.name(), id);
                }
                DeregisterNodeState::NoMoreOwners => {
                    self.static_storage.acquire_ownership();
                    self.dynamic_storage.acquire_ownership();
                    self.additional_resource.acquire_ownership();
                    trace!(from origin, "close and remove service: {} ({:?})",
                            self.static_config.name(), id);
                }
            }
        });
    }
}

#[doc(hidden)]
pub mod internal {
    use builder::event::EventOpenError;
    use dynamic_config::{PortCleanupAction, RemoveDeadNodeResult};
    use iceoryx2_bb_container::byte_string::FixedSizeByteString;
    use iceoryx2_bb_log::error;
    use port_factory::PortFactory;

    use crate::{
        node::{NodeBuilder, NodeId},
        port::{
            listener::remove_connection_of_listener, notifier::Notifier,
            port_identifiers::UniquePortId,
        },
        prelude::EventId,
        service::stale_resource_cleanup::{
            remove_data_segment_of_port, remove_receiver_port_from_all_connections,
            remove_sender_port_from_all_connections,
        },
    };

    use super::*;

    #[derive(Debug)]
    struct CleanupFailure;

    fn send_dead_node_signal<S: Service>(service_id: &ServiceId, config: &config::Config) {
        let origin = "send_dead_node_signal()";

        let service_details = match details::<S>(config, &service_id.0.clone().into()) {
            Ok(Some(service_details)) => service_details,
            Ok(None) => return,
            Err(e) => {
                warn!(from origin,
                    "Unable to acquire service details to emit dead node signal to waiting listeners for the service id {:?} due to ({:?})",
                    service_id, e);
                return;
            }
        };

        let service_name = service_details.static_details.name();

        let mut config = config.clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;

        let node = match NodeBuilder::new().config(&config).create::<S>() {
            Ok(node) => node,
            Err(e) => {
                warn!(from origin,
                                "Unable to create node to emit dead node signal to waiting listeners on the service {} due to ({:?}).",
                                service_name, e);
                return;
            }
        };

        let service = match node.service_builder(service_name).event().open() {
            Ok(service) => service,
            Err(EventOpenError::DoesNotExist) => return,
            Err(e) => {
                warn!(from origin,
                                "Unable to open event service to emit dead node signal to waiting listeners on the service {} due to ({:?}).",
                                service_name, e);
                return;
            }
        };

        if service.dynamic_config().number_of_listeners() == 0 {
            return;
        }

        let event_id = match service.static_config().notifier_dead_event {
            Some(event_id) => event_id,
            None => return,
        };

        let notifier = match Notifier::new_without_auto_event_emission(
            service.service,
            EventId::new(0),
        ) {
            Ok(notifier) => notifier,
            Err(e) => {
                warn!(from origin,
                                "Unable to create notifier to send dead node signal to waiting listeners on the service {} due to ({:?})",
                                service_name, e);
                return;
            }
        };

        if let Err(e) = notifier.notify_with_custom_event_id(EventId::new(event_id)) {
            warn!(from origin,
                            "Unable to send dead node signal to waiting listeners on service {} due to ({:?})",
                            service_name, e);
        }

        trace!(from origin, "Send dead node signal on service {}.", service_name);
    }

    fn remove_sender_connection_and_data_segment<S: Service>(
        id: u128,
        config: &config::Config,
        origin: &str,
        port_name: &str,
    ) -> Result<(), CleanupFailure> {
        unsafe { remove_sender_port_from_all_connections::<S>(id, config) }.map_err(|e| {
            debug!(from origin,
                "Failed to remove the {} ({:?}) from all of its connections ({:?}).",
                port_name, id, e);
            CleanupFailure
        })?;

        unsafe { remove_data_segment_of_port::<S>(id, config) }.map_err(|e| {
            debug!(from origin,
                "Failed to remove the {} ({:?}) data segment ({:?}).",
                port_name, id, e);
            CleanupFailure
        })?;

        Ok(())
    }

    fn remove_sender_and_receiver_connections_and_data_segment<S: Service>(
        id: u128,
        config: &config::Config,
        origin: &str,
        port_name: &str,
    ) -> Result<(), CleanupFailure> {
        remove_sender_connection_and_data_segment::<S>(id, config, origin, port_name)?;
        unsafe { remove_receiver_port_from_all_connections::<S>(id, config) }.map_err(|e| {
            debug!(from origin,
                    "Failed to remove the {} ({:?}) from all of its incoming connections ({:?}).",
                    port_name, id, e);
            CleanupFailure
        })?;

        Ok(())
    }

    fn remove_additional_blackboard_resources<S: Service>(
        config: &config::Config,
        blackboard_name: &FileName,
        blackboard_payload_config: &<S::BlackboardPayload as NamedConceptMgmt>::Configuration,
        blackboard_mgmt_name: &FixedSizeByteString<MAX_TYPE_NAME_LENGTH>,
        origin: &str,
        msg: &str,
    ) {
        match unsafe {
            <S::BlackboardPayload as NamedConceptMgmt>::remove_cfg(
                blackboard_name,
                blackboard_payload_config,
            )
        } {
            Ok(true) => {
                trace!(from origin, "Remove blackboard payload segment.");
            }
            _ => {
                error!(from origin,
                                  "{} since the blackboard payload segment cannot be removed - service seems to be in a corrupted state.", msg);
            }
        }

        match blackboard_mgmt_name.as_str() {
            Ok(s) => {
                // u64 is just a placeholder needed for the DynamicStorageConfiguration; it is
                // overwritten right below
                let mut blackboard_mgmt_config =
                    crate::service::config_scheme::blackboard_mgmt_config::<S, u64>(config);
                // Safe since the same type name is set when creating the BlackboardMgmt in
                // Creator::create_impl so we can safely remove the concept.
                unsafe {
                    <S::BlackboardMgmt<u64> as DynamicStorage::<u64>>::__internal_set_type_name_in_config(
                                            &mut blackboard_mgmt_config,
                                    s
                                        )
                };
                match unsafe {
                    <S::BlackboardMgmt<u64> as NamedConceptMgmt>::remove_cfg(
                        blackboard_name,
                        &blackboard_mgmt_config,
                    )
                } {
                    Ok(true) => {
                        trace!(from origin, "Remove blackboard mgmt segment.");
                    }
                    _ => {
                        error!(from origin,
                                            "{} since the blackboard mgmt segment cannot be removed - service seems to be in a corrupted state.", msg);
                    }
                }
            }
            Err(_) => {
                error!(from origin, "{} since the blackboard mgmt segment name cannot be acquired.", msg);
            }
        }
    }

    pub trait ServiceInternal<S: Service> {
        fn __internal_remove_node_from_service(
            node_id: &NodeId,
            service_id: &ServiceId,
            config: &config::Config,
        ) -> Result<(), ServiceRemoveNodeError> {
            let origin = format!("Service::remove_node_from_service({node_id:?}, {service_id:?})");
            let msg = "Unable to remove node from service";

            let dynamic_config = match open_dynamic_config::<S>(config, service_id) {
                Ok(Some(c)) => c,
                Ok(None) => {
                    fail!(from origin,
                          with ServiceRemoveNodeError::ServiceInCorruptedState,
                          "{} since the dynamic service segment is missing - service seems to be in a corrupted state.", msg);
                }
                Err(ServiceDetailsError::VersionMismatch) => {
                    fail!(from origin, with ServiceRemoveNodeError::VersionMismatch,
                        "{} since the service version does not match.", msg);
                }
                Err(e) => {
                    fail!(from origin, with ServiceRemoveNodeError::InternalError,
                        "{} due to an internal failure ({:?}).", msg, e);
                }
            };

            let mut number_of_dead_node_notifications = 0;
            let cleanup_port_resources = |port_id| {
                match port_id {
                    UniquePortId::Publisher(ref id) => {
                        if remove_sender_connection_and_data_segment::<S>(
                            id.value(),
                            config,
                            &origin,
                            "publisher",
                        )
                        .is_err()
                        {
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Subscriber(ref id) => {
                        if let Err(e) = unsafe {
                            remove_receiver_port_from_all_connections::<S>(id.value(), config)
                        } {
                            debug!(from origin, "Failed to remove the subscriber ({:?}) from all of its connections ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Notifier(_) => {
                        number_of_dead_node_notifications += 1;
                    }
                    UniquePortId::Listener(ref id) => {
                        if let Err(e) = unsafe { remove_connection_of_listener::<S>(id, config) } {
                            debug!(from origin, "Failed to remove the listeners ({:?}) connection ({:?}).", id, e);
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Client(ref id) => {
                        if remove_sender_and_receiver_connections_and_data_segment::<S>(
                            id.value(),
                            config,
                            &origin,
                            "client",
                        )
                        .is_err()
                        {
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Server(ref id) => {
                        if remove_sender_and_receiver_connections_and_data_segment::<S>(
                            id.value(),
                            config,
                            &origin,
                            "server",
                        )
                        .is_err()
                        {
                            return PortCleanupAction::SkipPort;
                        }
                    }
                    UniquePortId::Reader(ref _id) => {}
                    UniquePortId::Writer(ref _id) => {}
                };

                trace!(from origin, "Remove port {:?} from service.", port_id);
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
                // check if service was a blackboard service to remove its additional resources
                let blackboard_name =
                    crate::service::naming_scheme::blackboard_name(service_id.as_str());
                let blackboard_payload_config =
                    crate::service::config_scheme::blackboard_data_config::<S>(config);
                let blackboard_payload = <S::BlackboardPayload as NamedConceptMgmt>::does_exist_cfg(
                    &blackboard_name,
                    &blackboard_payload_config,
                );
                let mut is_blackboard = false;
                let mut blackboard_mgmt_name = FixedSizeByteString::<MAX_TYPE_NAME_LENGTH>::new();
                if let Ok(true) = blackboard_payload {
                    is_blackboard = true;

                    let details = match details::<S>(config, &service_id.0.clone().into()) {
                        Ok(Some(d)) => d,
                        _ => {
                            fail!(from origin,
                                  with ServiceRemoveNodeError::ServiceInCorruptedState,
                                  "{} due to a failure while acquiring the service details.", msg);
                        }
                    };
                    blackboard_mgmt_name =
                        details.static_details.blackboard().type_details.type_name;
                }

                match unsafe {
                    // IMPORTANT: The static service config must be removed first. If it cannot be
                    // removed, the process may lack sufficient permissions and should not remove
                    // any other resources.
                    remove_static_service_config::<S>(config, &service_id.0.clone().into())
                } {
                    Ok(_) => {
                        trace!(from origin, "Remove unused service.");

                        // remove additional blackboard resources
                        if is_blackboard {
                            remove_additional_blackboard_resources::<S>(
                                config,
                                &blackboard_name,
                                &blackboard_payload_config,
                                &blackboard_mgmt_name,
                                &origin,
                                msg,
                            );
                        }

                        dynamic_config.acquire_ownership()
                    }
                    Err(e) => {
                        error!(from origin, "Unable to remove static config of unused service ({:?}).",
                            e);
                    }
                }
            } else if number_of_dead_node_notifications != 0 {
                send_dead_node_signal::<S>(service_id, config);
            }

            Ok(())
        }
    }
}

/// Represents additional resources a service could use and have to be cleaned up when no owners
/// are left
pub trait ServiceResource {
    /// Acquires the ownership of the additional resources. When the objects go out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);
}

#[derive(Debug)]
pub(crate) struct NoResource;
impl ServiceResource for NoResource {
    fn acquire_ownership(&self) {}
}

/// Represents a service. Used to create or open new services with the
/// [`crate::node::Node::service_builder()`].
/// Contains the building blocks a [`Service`] requires to create the underlying resources and
/// establish communication.
#[allow(private_bounds)]
pub trait Service: Debug + Sized + internal::ServiceInternal<Self> + Clone {
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
    type SharedMemory: SharedMemoryForPoolAllocator;

    /// The dynamic memory used to store dynamic payload
    type ResizableSharedMemory: ResizableSharedMemoryForPoolAllocator<Self::SharedMemory>;

    /// The connection used to exchange pointers to the payload
    type Connection: ZeroCopyConnection;

    /// The mechanism used to signal events between endpoints.
    type Event: Event;

    /// Monitoring mechanism to detect dead processes.
    type Monitoring: Monitoring;

    /// Event multiplexing mechanisms to wait on multiple events.
    type Reactor: Reactor;

    /// Defines the thread-safety policy of the service. If it is defined as
    /// [`MutexProtected`](iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected), the
    /// [`Service`]s ports are threadsafe and the payload can be moved into threads. If it is set
    /// to [`SingleThreaded`](iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded),
    /// the [`Service`]s ports and payload cannot be shared ([`Sync`]) between threads or moved
    /// ([`Send`]) into other threads.
    type ArcThreadSafetyPolicy<T: Send + Debug>: ArcSyncPolicy<T>;

    /// Defines the construct used to store the management data of the blackboard service.
    type BlackboardMgmt<T: Send + Sync + Debug + 'static>: DynamicStorage<T>;

    /// Defines the construct used to store the payload data of the blackboard service.
    type BlackboardPayload: SharedMemory<BumpAllocator>;

    /// Checks if a service under a given [`config::Config`] does exist
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
        let service_id = ServiceId::new::<Self::ServiceNameHasher>(service_name, messaging_pattern);
        details::<Self>(config, &service_id.0.into())
    }

    /// Returns a list of all services created under a given [`config::Config`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
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

pub(crate) unsafe fn remove_static_service_config<S: Service>(
    config: &config::Config,
    uuid: &FileName,
) -> Result<bool, NamedConceptRemoveError> {
    let msg = "Unable to remove static service config";
    let origin = "Service::remove_static_service_config()";
    let static_storage_config = config_scheme::static_config_storage_config::<S>(config);

    match <S::StaticStorage as NamedConceptMgmt>::remove_cfg(uuid, &static_storage_config) {
        Ok(v) => Ok(v),
        Err(e) => {
            fail!(from origin, with e, "{msg} due to ({:?}).", e);
        }
    }
}

fn details<S: Service>(
    config: &config::Config,
    uuid: &FileName,
) -> Result<Option<ServiceDetails<S>>, ServiceDetailsError> {
    let msg = "Unable to acquire service details";
    let origin = "Service::details()";
    let static_storage_config = config_scheme::static_config_storage_config::<S>(config);

    let reader = match <<S::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
        S::StaticStorage,
    >>::new(uuid)
    .config(&static_storage_config.clone())
    .has_ownership(false)
    .open(Duration::ZERO)
    {
        Ok(reader) => reader,
        Err(StaticStorageOpenError::DoesNotExist)
        | Err(StaticStorageOpenError::InitializationNotYetFinalized) => return Ok(None),
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

    if uuid.as_bytes() != service_config.service_id().0.as_bytes() {
        fail!(from origin, with ServiceDetailsError::ServiceInInconsistentState,
                "{} since the service {:?} has an inconsistent hash of {} according to config {:?}",
                msg, service_config, uuid, config);
    }

    let dynamic_config = open_dynamic_config::<S>(config, service_config.service_id())?;
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
    service_id: &ServiceId,
) -> Result<Option<S::DynamicStorage>, ServiceDetailsError> {
    let origin = format!(
        "Service::open_dynamic_details<{}>({:?})",
        core::any::type_name::<S>(),
        service_id
    );
    let msg = "Unable to open the services dynamic config";
    match
            <<S::DynamicStorage as DynamicStorage<
                    DynamicConfig,
                >>::Builder<'_> as NamedConceptBuilder<
                    S::DynamicStorage,
                >>::new(&service_id.0.clone().into())
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
    service_id: &ServiceId,
    config: &config::Config,
) -> Result<(), ServiceRemoveTagError> {
    let origin = format!(
        "remove_service_tag<{}>({:?}, service_id: {:?})",
        core::any::type_name::<S>(),
        node_id,
        service_id
    );

    match unsafe {
        <S::StaticStorage as NamedConceptMgmt>::remove_cfg(
            &service_id.0.clone().into(),
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
