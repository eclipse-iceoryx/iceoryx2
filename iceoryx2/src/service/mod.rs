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
//!             .define(&"some attribute key".try_into()?, &"some attribute value".try_into()?)?
//!             .define(&"some attribute key".try_into()?, &"another attribute value for the same key".try_into()?)?
//!             .define(&"another key".try_into()?, &"another value".try_into()?)?
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
//!             .require(&"another key".try_into()?, &"another value".try_into()?)?
//!             .require_key(&"some attribute key".try_into()?)?
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
//! use iceoryx2_bb_container::string::*;
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
//!     .add::<StaticString<8>>(17, "Nalalala".try_into().unwrap())
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

/// Represents the unique hash of a [`Service`]
pub mod service_hash;

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

use core::fmt::Debug;
use core::ptr::NonNull;
use core::time::Duration;

use alloc::format;
use alloc::string::String as CoreString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::counting_bit_set::RelocatableCountingBitSet;
use iceoryx2_bb_posix::file::AccessMode;

use crate::config;
use crate::identifiers::UniqueServiceId;
use crate::node::{NodeListFailure, NodeState, SharedNode};
use crate::service::config_scheme::dynamic_config_storage_config;
use crate::service::dynamic_config::DynamicConfig;
use crate::service::naming_scheme::dynamic_config_name;
use crate::service::naming_scheme::static_config_name;
use crate::service::stale_resource_cleanup::{
    remove_additional_blackboard_resources,
    remove_sender_and_receiver_connections_and_data_segment,
    remove_sender_connection_and_data_segment,
};
use crate::service::stale_resource_cleanup::{remove_service_tag, remove_static_service_config};
use crate::service::static_config::*;
use crate::{
    identifiers::{UniqueNodeId, UniquePortId},
    node::NodeBuilder,
    port::{listener::remove_connection_of_listener, notifier::Notifier},
    prelude::EventId,
    service::stale_resource_cleanup::{remove_port_tag, remove_receiver_port_from_all_connections},
};
use builder::event::EventOpenError;
use dynamic_config::PortCleanupAction;
use iceoryx2_bb_elementary::CallbackProgression;
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
use iceoryx2_cal::shm_allocator::shm_bump_allocator::BumpAllocator;
use iceoryx2_cal::static_storage::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;
use iceoryx2_log::error;
use iceoryx2_log::{debug, fail, trace, warn};
use port_factory::{PortFactory, notifier::NotifierConfig};
use service_hash::ServiceHash;

use self::dynamic_config::DeregisterNodeState;
use self::messaging_pattern::MessagingPattern;
use self::service_name::ServiceName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Error that can be reported when removing a [`Node`](crate::node::Node).
pub enum ServiceRemoveNodeError {
    /// An interrupt signal was received.
    Interrupt,
    /// The iceoryx2 version that created the [`Node`](crate::node::Node) does
    /// not match this iceoryx2 version.
    VersionMismatch,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
    /// The process does not have the permissions to remove the service after last node was removed
    InsufficientPermissions,
}

impl core::fmt::Display for ServiceRemoveNodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServiceRemoveNodeError::{self:?}")
    }
}

impl core::error::Error for ServiceRemoveNodeError {}

impl From<ServiceRemoveError> for ServiceRemoveNodeError {
    fn from(value: ServiceRemoveError) -> Self {
        match value {
            ServiceRemoveError::Interrupt => ServiceRemoveNodeError::Interrupt,
            ServiceRemoveError::InsufficientPermissions => {
                ServiceRemoveNodeError::InsufficientPermissions
            }
            ServiceRemoveError::VersionMismatch => ServiceRemoveNodeError::VersionMismatch,
            ServiceRemoveError::InternalError => ServiceRemoveNodeError::InternalError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Error that can be reported when removing a [`Node`](crate::node::Node).
pub enum ServiceRemoveError {
    /// An interrupt signal was received
    Interrupt,
    /// The iceoryx2 version that created the [`Node`](crate::node::Node) does
    /// not match this iceoryx2 version.
    VersionMismatch,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalError,
    /// The process does not have the permissions to remove the service
    InsufficientPermissions,
}

impl core::fmt::Display for ServiceRemoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServiceRemoveError::{self:?}")
    }
}

impl core::error::Error for ServiceRemoveError {}

/// Failure that can be reported when the [`ServiceDetails`] are acquired with [`Service::details()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceDetailsError {
    /// An interrupt signal was raised.
    Interrupt,
    /// The process does not have the permissions to acquire the service details
    InsufficientPermissions,
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
    pub(crate) dynamic_storage: S::DynamicStorage<DynamicConfig>,
    pub(crate) additional_resource: R,
    pub(crate) static_config: StaticConfig,
    pub(crate) shared_node: SharedNode<S>,

    // IMPORTANT: The static service config must be removed last since it contains the details about all
    // other resources that also have to be removed. If this is removed earlier, those resources are leaked.
    pub(crate) static_storage: S::StaticStorage,
}

impl<S: Service, R: ServiceResource> Abandonable for ServiceState<S, R> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };

        unsafe {
            S::DynamicStorage::abandon_in_place(NonNull::iox2_from_mut(&mut this.dynamic_storage))
        };
        unsafe { R::abandon_in_place(NonNull::iox2_from_mut(&mut this.additional_resource)) };
        unsafe { SharedNode::<S>::abandon_in_place(NonNull::iox2_from_mut(&mut this.shared_node)) };
        unsafe {
            S::StaticStorage::abandon_in_place(NonNull::iox2_from_mut(&mut this.static_storage))
        };
    }
}

#[derive(Debug)]
pub(crate) struct SharedServiceState<S: Service, R: ServiceResource> {
    state: Arc<ServiceState<S, R>>,
}

impl<S: Service, R: ServiceResource> Clone for SharedServiceState<S, R> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<S: Service, R: ServiceResource> Abandonable for SharedServiceState<S, R> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        if let Some(state) = Arc::get_mut(&mut this.state) {
            unsafe { ServiceState::abandon_in_place(NonNull::iox2_from_mut(state)) };
        } else {
            unsafe { core::ptr::drop_in_place(&mut this.state) };
        }
    }
}

impl<S: Service, R: ServiceResource> SharedServiceState<S, R> {
    pub(crate) fn static_config(&self) -> &StaticConfig {
        &self.state.static_config
    }

    pub(crate) fn dynamic_storage(&self) -> &S::DynamicStorage<DynamicConfig> {
        &self.state.dynamic_storage
    }

    pub(crate) fn shared_node(&self) -> &SharedNode<S> {
        &self.state.shared_node
    }

    pub(crate) fn additional_resource(&self) -> &R {
        &self.state.additional_resource
    }
}

impl<S: Service, R: ServiceResource> ServiceState<S, R> {
    pub(crate) fn new(
        static_config: StaticConfig,
        shared_node: SharedNode<S>,
        dynamic_storage: S::DynamicStorage<DynamicConfig>,
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
            new_self.static_config.name(), new_self.static_config.service_hash());
        new_self
    }
}

impl<S: Service, R: ServiceResource> Drop for ServiceState<S, R> {
    fn drop(&mut self) {
        let origin = "ServiceState::drop()";
        let hash = self.static_config.service_hash();
        self.shared_node.registered_services().remove(hash, |handle| {
            if let Err(e) = remove_service_tag::<S>(self.shared_node.id(), hash, self.shared_node.config())
            {
                debug!(from origin, "The service tag could not be removed from the node {:?} ({:?}).",
                        self.shared_node.id(), e);
            }

            match self.dynamic_storage.get().deregister_node_id(handle) {
                Ok(DeregisterNodeState::HasOwners) => {
                    trace!(from origin, "close service: {} ({:?})",
                            self.static_config.name(), hash);
                }
                Ok(DeregisterNodeState::NoMoreOwners) => {
                    self.static_storage.acquire_ownership();
                    self.dynamic_storage.acquire_ownership();
                    self.additional_resource.acquire_ownership();
                    trace!(from origin, "close and remove service: {} ({:?})",
                            self.static_config.name(), hash);
                }
                Err(e) => {
                    error!(from origin,
                        "Unable to deregister node {} from service. This could indicate a corrupted system! [{e:?}]", self.shared_node.id())
                }
            }
        });
    }
}

#[doc(hidden)]
pub mod internal {
    use crate::{
        port::port_name::PortName, service::stale_resource_cleanup::ServiceRemoveTagError,
    };

    use super::*;
    fn send_dead_node_signal<S: Service>(service_hash: &ServiceHash, config: &config::Config) {
        let origin = "send_dead_node_signal()";

        let service_details = match __internal_details::<S>(config, service_hash) {
            Ok(Some(service_details)) => service_details,
            Ok(None) => return,
            Err(e) => {
                warn!(from origin,
                    "Unable to acquire service details to emit dead node signal to waiting listeners for the service id {:?} due to ({:?})",
                    service_hash, e);
                return;
            }
        };

        let service_name = service_details.static_details.name();

        let mut config = config.clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        config.global.service.cleanup_dead_nodes_on_open = false;

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

        let event_id = match service.static_config().notifier_dead_event.as_option_ref() {
            Some(event_id) => *event_id,
            None => return,
        };

        let notifier = match Notifier::new_without_auto_event_emission(
            service.service,
            NotifierConfig {
                default_event_id: EventId::new(0),
                port_name: PortName::new_empty(),
            },
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

    pub trait ServiceInternal<S: Service> {
        /// # Safety
        ///
        /// * No other process shall using the service currently.
        ///
        #[doc(hidden)]
        unsafe fn __internal_force_remove_service(
            service_name: &ServiceName,
            config: &config::Config,
            messaging_pattern: MessagingPattern,
        ) -> Result<bool, ServiceRemoveError> {
            let origin = "Service::remove()";
            let msg = format!("Unable to remove all resources of the service \"{service_name}\"");
            let service_hash =
                ServiceHash::new::<S::ServiceNameHasher>(service_name, messaging_pattern);
            match read_static_service_config::<S>(config, &service_hash) {
                Ok(Some(static_config)) => {
                    unsafe {
                        S::__internal_remove_service(
                            &service_hash,
                            static_config.unique_service_id(),
                            config,
                        )?
                    };

                    Ok(true)
                }
                Ok(None) => Ok(false),
                Err(e) => {
                    warn!(from origin, "{msg} since the static config could not be read. [{e:?}]");

                    match unsafe { remove_static_service_config::<S>(config, &service_hash) } {
                        Ok(v) => Ok(v),
                        Err(NamedConceptRemoveError::InsufficientPermissions) => {
                            fail!(from origin, with ServiceRemoveError::InsufficientPermissions,
                                "{msg} due to insufficient permissions.");
                        }
                        Err(NamedConceptRemoveError::Interrupt) => {
                            fail!(from origin, with ServiceRemoveError::Interrupt,
                                "{msg} since an interrupt signal was received.");
                        }
                        Err(NamedConceptRemoveError::InternalError) => {
                            fail!(from origin, with ServiceRemoveError::InternalError,
                                "{msg} due to an internal error.");
                        }
                    }
                }
            }
        }

        /// # Safety
        ///
        /// * No other process shall using the service currently.
        ///
        #[doc(hidden)]
        unsafe fn __internal_remove_service(
            service_hash: &ServiceHash,
            unique_service_id: UniqueServiceId,
            config: &config::Config,
        ) -> Result<(), ServiceRemoveError> {
            let origin = "Service::remove()";
            let msg = "Unable to remove all service resources";

            // check if service was a blackboard service to remove its additional resources
            let blackboard_name = crate::service::naming_scheme::blackboard_name(unique_service_id);
            let blackboard_payload_config =
                crate::service::config_scheme::blackboard_data_config::<S>(config);
            let blackboard_payload = <S::BlackboardPayload as NamedConceptMgmt>::does_exist_cfg(
                &blackboard_name,
                &blackboard_payload_config,
            );
            if let Ok(true) = blackboard_payload {
                match __internal_details::<S>(config, service_hash) {
                    Ok(Some(details)) => {
                        let blackboard_mgmt_name =
                            details.static_details.blackboard().type_details.type_name;

                        remove_additional_blackboard_resources::<S>(
                            config,
                            unique_service_id,
                            &blackboard_mgmt_name,
                            origin,
                            msg,
                        );
                    }
                    _ => {
                        warn!(from origin,
                            "{} for {service_hash} due to a failure while acquiring the service details", msg);
                    }
                };
            }

            let segment_name = dynamic_config_name(unique_service_id);
            match unsafe {
                <S::DynamicStorage<DynamicConfig> as NamedConceptMgmt>::remove_cfg(
                    &segment_name,
                    &dynamic_config_storage_config::<S>(config),
                )
            } {
                Ok(_) => (),
                Err(NamedConceptRemoveError::InsufficientPermissions) => {
                    fail!(from origin, with ServiceRemoveError::InsufficientPermissions,
                        "{msg} for {service_hash} since the dynamic config could not be removed due to insufficient permissions.");
                }
                Err(NamedConceptRemoveError::Interrupt) => {
                    fail!(from origin, with ServiceRemoveError::Interrupt,
                        "{msg} for {service_hash} since an interrupt signal was received.");
                }
                Err(NamedConceptRemoveError::InternalError) => {
                    fail!(from origin, with ServiceRemoveError::InsufficientPermissions,
                        "{msg} for {service_hash} since the dynamic config could not be removed due to an internal error.");
                }
            }

            match unsafe {
                // IMPORTANT: The static service config must be removed last.
                remove_static_service_config::<S>(config, service_hash)
            } {
                Ok(_) => {
                    trace!(from origin, "Remove unused service.");
                }
                Err(NamedConceptRemoveError::InsufficientPermissions) => {
                    fail!(from origin, with ServiceRemoveError::InsufficientPermissions,
                        "{msg} for {service_hash} due to insufficient permissions.");
                }
                Err(NamedConceptRemoveError::InternalError) => {
                    fail!(from origin, with ServiceRemoveError::InternalError,
                        "{msg} for {service_hash} due to an internal error.");
                }
                Err(NamedConceptRemoveError::Interrupt) => {
                    fail!(from origin, with ServiceRemoveError::Interrupt,
                        "{msg} for {service_hash} since an interrupt signal was received.");
                }
            }

            Ok(())
        }

        #[doc(hidden)]
        fn __internal_remove_node_from_service(
            node_id: &UniqueNodeId,
            service_hash: &ServiceHash,
            config: &config::Config,
        ) -> Result<(), ServiceRemoveNodeError> {
            let origin =
                format!("Service::remove_node_from_service({node_id:?}, {service_hash:?})");
            let msg = "Unable to remove node from service";

            let remove_service_tag = || match remove_service_tag::<S>(node_id, service_hash, config)
            {
                Ok(()) | Err(ServiceRemoveTagError::AlreadyRemoved) => Ok(()),
                Err(ServiceRemoveTagError::InsufficientPermissions) => {
                    fail!(from origin, with ServiceRemoveNodeError::InsufficientPermissions,
                        "{msg} since the service tag could not be removed due to insufficient permissions.");
                }
                Err(ServiceRemoveTagError::Interrupt) => {
                    fail!(from origin, with ServiceRemoveNodeError::Interrupt,
                        "{msg} since the service tag could not be removed since an interrupt signal was raised.");
                }
                Err(ServiceRemoveTagError::InternalError) => {
                    fail!(from origin, with ServiceRemoveNodeError::InternalError,
                        "{msg} since the service tag could not be removed due to an internal error.");
                }
            };

            let static_config = match read_static_service_config::<S>(config, service_hash) {
                Ok(Some(config)) => config,
                Ok(None) => {
                    warn!(from origin, "Trying to remove node {} from non-existing service {}",
                        node_id, service_hash);
                    return remove_service_tag();
                }
                Err(e) => {
                    fail!(from origin, with ServiceRemoveNodeError::InternalError,
                        "{msg} since the static service config could not be read. [{e:?}]");
                }
            };

            let dynamic_config = match open_dynamic_config::<S>(
                config,
                static_config.unique_service_id(),
            ) {
                Ok(Some(c)) => c,
                Ok(None) => {
                    warn!(from origin,
                        "Found corrupted service {service_hash}. Trying to remove it completely");
                    match unsafe {
                        Self::__internal_remove_service(
                            service_hash,
                            static_config.unique_service_id(),
                            config,
                        )
                    } {
                        Ok(()) => {
                            return remove_service_tag();
                        }
                        Err(e) => {
                            fail!(from origin, with e.into(),
                                    "{msg} since the corrupted service could not be removed. [{e:?}]");
                        }
                    }
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

                if let Err(e) = remove_port_tag::<S>(node_id, port_id.value(), config) {
                    debug!(from origin,  "Failed to remove the port tag for port {:?}. [{e:?}]", port_id);
                    return PortCleanupAction::SkipPort;
                }
                trace!(from origin, "Remove port {:?} from service.", port_id);
                PortCleanupAction::RemovePort
            };

            let remove_service = match unsafe {
                dynamic_config
                    .get()
                    .remove_dead_node_id(node_id, cleanup_port_resources)
            } {
                DeregisterNodeState::HasOwners => false,
                DeregisterNodeState::NoMoreOwners => true,
            };

            if remove_service {
                unsafe {
                    Self::__internal_remove_service(
                        service_hash,
                        static_config.unique_service_id(),
                        config,
                    )?
                };
            } else if number_of_dead_node_notifications != 0 {
                send_dead_node_signal::<S>(service_hash, config);
            }

            remove_service_tag()
        }
    }
}

/// Represents additional resources a service could use and have to be cleaned up when no owners
/// are left
pub trait ServiceResource: Abandonable {
    /// Acquires the ownership of the additional resources. When the objects go out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);
}

#[derive(Debug)]
pub(crate) struct NoResource;
impl ServiceResource for NoResource {
    fn acquire_ownership(&self) {}
}

impl Abandonable for NoResource {
    unsafe fn abandon_in_place(_this: NonNull<Self>) {}
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

    /// Defines the construct used to store the data that can be changed at runtime but
    /// persist after a process crashed.
    type PersistentDynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static>: DynamicStorage<
        T,
    >;

    /// Defines the construct used to store the [`Service`]s dynamic configuration. This
    /// contains for instance all endpoints and other dynamic details.
    type DynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static>: DynamicStorage<T>;

    /// The memory used to store the payload.
    type SharedMemory: SharedMemoryForPoolAllocator;

    /// The dynamic memory used to store dynamic payload
    type ResizableSharedMemory: ResizableSharedMemoryForPoolAllocator<Self::SharedMemory>;

    /// The connection used to exchange pointers to the payload
    type Connection: ZeroCopyConnection;

    /// The mechanism used to signal events between endpoints.
    type Event: Event<RelocatableCountingBitSet>;

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
    type ArcThreadSafetyPolicy<T: Send + Debug + Abandonable>: ArcSyncPolicy<T>;

    /// Defines the construct used to store the management data of the blackboard service.
    type BlackboardMgmt<T: Send + Sync + Debug + ZeroCopySend + 'static>: DynamicStorage<T>;

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
        let service_hash =
            ServiceHash::new::<Self::ServiceNameHasher>(service_name, messaging_pattern);
        __internal_details::<Self>(config, &service_hash)
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
            let hash = match ServiceHash::from_bytes(uuid) {
                Ok(hash) => hash,
                Err(e) => {
                    warn!(from origin,
                        "{msg} since there static configs of service where the name ({uuid}) violates the naming rule. [{e:?}]");
                    continue;
                }
            };
            if let Ok(Some(service_details)) = __internal_details::<Self>(config, &hash) {
                if callback(service_details) == CallbackProgression::Stop {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn __internal_details<S: Service>(
    config: &config::Config,
    service_hash: &ServiceHash,
) -> Result<Option<ServiceDetails<S>>, ServiceDetailsError> {
    let msg = "Unable to acquire service details";
    let origin = "Service::details()";
    let service_config = match read_static_service_config::<S>(config, service_hash) {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(None),
        Err(e) => {
            fail!(from origin, with e,
                "{msg} since the static service config could not be read. [{e:?}]");
        }
    };

    let dynamic_config = open_dynamic_config::<S>(config, service_config.unique_service_id())?;
    let dynamic_details = if let Some(d) = dynamic_config {
        let mut nodes = vec![];
        d.get().list_node_ids(|node_id| {
            match NodeState::new(node_id, config) {
                Ok(Some(state)) => nodes.push(state),
                Ok(None)
                | Err(NodeListFailure::InsufficientPermissions)
                | Err(NodeListFailure::Interrupt) => (),
                Err(NodeListFailure::InternalError) => {
                    debug!(from origin, "Unable to acquire NodeState for service \"{:?}\"", service_hash);
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

fn read_static_service_config<S: Service>(
    config: &config::Config,
    service_hash: &ServiceHash,
) -> Result<Option<StaticConfig>, ServiceDetailsError> {
    let msg = "Unable to acquire service details";
    let origin = "Service::details()";
    let static_storage_config = config_scheme::static_config_storage_config::<S>(config);
    let name = static_config_name(service_hash);
    let reader = match <<S::StaticStorage as StaticStorage>::Builder as NamedConceptBuilder<
        S::StaticStorage,
    >>::new(&name)
    .config(&static_storage_config.clone())
    .has_ownership(false)
    .open(Duration::ZERO)
    {
        Ok(reader) => reader,
        Err(StaticStorageOpenError::DoesNotExist)
        | Err(StaticStorageOpenError::InitializationNotYetFinalized) => return Ok(None),
        Err(StaticStorageOpenError::Interrupt) => {
            fail!(from origin,
                  with ServiceDetailsError::Interrupt,
                  "{} since an interrupt signal was raised while opening the static service info \"{}\" for reading.",
                  msg, name);
        }
        Err(StaticStorageOpenError::InsufficientPermissions) => {
            fail!(from origin,
                with ServiceDetailsError::InsufficientPermissions,
                "{} since the process does not have the permission to acquire the service details.", msg);
        }
        Err(e) => {
            fail!(from origin,
                  with ServiceDetailsError::FailedToOpenStaticServiceInfo,
                  "{} due to a failure while opening the static service info \"{}\" for reading ({:?})",
                  msg, name, e);
        }
    };

    let mut content = CoreString::from_utf8(vec![b' '; reader.len() as usize]).unwrap();
    match reader.read(unsafe { content.as_mut_vec().as_mut_slice() }) {
        Ok(_) => (),
        Err(StaticStorageReadError::Interrupt) => {
            fail!(from origin, with ServiceDetailsError::Interrupt,
                    "{} since an interrupt signal was raised while reading the static service info \"{}\".",
                    msg, name);
        }
        Err(e) => {
            fail!(from origin, with ServiceDetailsError::FailedToReadStaticServiceInfo,
                    "{} since the static service info \"{}\" could not be read ({:?}).",
                    msg, name, e );
        }
    }

    let service_config =
        match S::ConfigSerializer::deserialize::<StaticConfig>(unsafe { content.as_mut_vec() }) {
            Ok(service_config) => service_config,
            Err(e) => {
                fail!(from origin, with ServiceDetailsError::FailedToDeserializeStaticServiceInfo,
                    "{} since the static service info \"{}\" could not be deserialized ({:?}).",
                       msg, name, e );
            }
        };

    if service_hash != service_config.service_hash() {
        fail!(from origin, with ServiceDetailsError::ServiceInInconsistentState,
                "{} since the service {:?} has an inconsistent hash of {} according to config {:?}",
                msg, service_config, service_hash, config);
    }

    if service_config.iceoryx2_version() != PackageVersion::get() {
        fail!(from origin, with ServiceDetailsError::VersionMismatch,
            "{} since the service was created with iceoryx2 version {} but this process expects iceoryx2 version {}.", msg, service_config.iceoryx2_version(), PackageVersion::get());
    }

    Ok(Some(service_config))
}

fn open_dynamic_config<S: Service>(
    config: &config::Config,
    service_id: UniqueServiceId,
) -> Result<Option<S::DynamicStorage<DynamicConfig>>, ServiceDetailsError> {
    let origin = format!(
        "Service::open_dynamic_details<{}>({:?})",
        core::any::type_name::<S>(),
        service_id
    );
    let msg = "Unable to open the services dynamic config";
    let segment_name = dynamic_config_name(service_id);
    match
            <<S::DynamicStorage<DynamicConfig> as DynamicStorage<
                    DynamicConfig,
                >>::Builder<'_> as NamedConceptBuilder<
                    S::DynamicStorage<DynamicConfig>,
                >>::new(&segment_name)
                    .config(&dynamic_config_storage_config::<S>(config))
                .has_ownership(false)
                .open(AccessMode::ReadWrite) {
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
