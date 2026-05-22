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

//! # Example
//!
//! See [`crate::service`]
//!
use core::alloc::Layout;
use core::hash::Hash;
use core::marker::PhantomData;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_container::flatmap::RelocatableFlatMap;
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_container::string::String;
use iceoryx2_bb_container::vector::relocatable_vec::*;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::static_assert::static_assert_eq;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::*;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder};
use iceoryx2_log::{error, fatal_panic};

use crate::constants::{MAX_BLACKBOARD_KEY_ALIGNMENT, MAX_BLACKBOARD_KEY_SIZE};
use crate::service;
use crate::service::builder::{
    CustomKeyMarker, DynamicConfigCreationArgs, ServiceCreateError, ServiceOpenError,
};
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use crate::service::dynamic_config::MessagingPatternSettings;
use crate::service::dynamic_config::blackboard::DynamicConfigSettings;
use crate::service::naming_scheme::blackboard_name;
use crate::service::port_factory::blackboard;
use crate::service::static_config::message_type_details::TypeDetail;
use crate::service::static_config::messaging_pattern::MessagingPattern;
use crate::service::*;

use super::ServiceState;

use self::attribute::{AttributeSpecifier, AttributeVerifier};

/// Errors that can occur when an existing [`MessagingPattern::Blackboard`] [`Service`] shall be opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardOpenError {
    /// The [`Service`] could not be opened since it does not exist
    DoesNotExist,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`] has the wrong key type.
    IncompatibleKeys,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`Service`] supports less [`Reader`](crate::port::reader::Reader)s than requested.
    DoesNotSupportRequestedAmountOfReaders,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than requested.
    DoesNotSupportRequestedAmountOfNodes,
    /// The [`Node`] service tag could not be created. Required to track resources of dead nodes when cleaning them up.
    UnableToCreateServiceTag,
    /// The iceoryx2 service version does not match the one of the [`Service`].
    VersionMismatch,
}

impl core::fmt::Display for BlackboardOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BlackboardOpenError::{self:?}")
    }
}

impl core::error::Error for BlackboardOpenError {}

impl From<ServiceState> for BlackboardOpenError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::IncompatiblePayload => BlackboardOpenError::IncompatibleKeys,
            ServiceState::IncompatibleMessagingPattern => {
                BlackboardOpenError::IncompatibleMessagingPattern
            }
            ServiceState::InsufficientPermissions => BlackboardOpenError::InsufficientPermissions,
            ServiceState::HangsInCreation => BlackboardOpenError::HangsInCreation,
            ServiceState::Corrupted => BlackboardOpenError::ServiceInCorruptedState,
            ServiceState::InternalFailure => BlackboardOpenError::InternalFailure,
        }
    }
}

impl From<ServiceOpenError> for BlackboardOpenError {
    fn from(value: ServiceOpenError) -> Self {
        match value {
            ServiceOpenError::DoesNotExist => BlackboardOpenError::DoesNotExist,
            ServiceOpenError::ExceedsMaxNumberOfNodes => {
                BlackboardOpenError::ExceedsMaxNumberOfNodes
            }
            ServiceOpenError::HangsInCreation => BlackboardOpenError::HangsInCreation,
            ServiceOpenError::IncompatibleMessagingPattern => {
                BlackboardOpenError::IncompatibleMessagingPattern
            }
            ServiceOpenError::IncompatiblePayload => BlackboardOpenError::IncompatibleKeys,
            ServiceOpenError::InsufficientPermissions => {
                BlackboardOpenError::InsufficientPermissions
            }
            ServiceOpenError::InternalFailure => BlackboardOpenError::InternalFailure,
            ServiceOpenError::IsMarkedForDestruction => BlackboardOpenError::IsMarkedForDestruction,
            ServiceOpenError::ServiceInCorruptedState => {
                BlackboardOpenError::ServiceInCorruptedState
            }
            ServiceOpenError::UnableToCreateServiceTag => {
                BlackboardOpenError::UnableToCreateServiceTag
            }
            ServiceOpenError::VersionMismatch => BlackboardOpenError::VersionMismatch,
        }
    }
}

/// Errors that can occur when a new [`MessagingPattern::Blackboard`] [`Service`] shall be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardCreateError {
    /// The [`Service`] already exists.
    AlreadyExists,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// No key-value pairs have been provided. At least one is required.
    NoEntriesProvided,
    /// The [`Node`] service tag could not be created. Required to track resources of dead nodes when cleaning them up.
    UnableToCreateServiceTag,
    /// The [`Service`]s config could not be created and written to the static service configuration.
    ServiceConfigCouldNotBeCreated,
}

impl core::fmt::Display for BlackboardCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BlackboardCreateError::{self:?}")
    }
}

impl core::error::Error for BlackboardCreateError {}

impl From<ServiceState> for BlackboardCreateError {
    fn from(value: ServiceState) -> Self {
        match value {
            ServiceState::IncompatiblePayload | ServiceState::IncompatibleMessagingPattern => {
                BlackboardCreateError::AlreadyExists
            }
            ServiceState::InsufficientPermissions => BlackboardCreateError::InsufficientPermissions,
            ServiceState::HangsInCreation => BlackboardCreateError::HangsInCreation,
            ServiceState::Corrupted => BlackboardCreateError::ServiceInCorruptedState,
            ServiceState::InternalFailure => BlackboardCreateError::InternalFailure,
        }
    }
}

impl From<ServiceCreateError> for BlackboardCreateError {
    fn from(value: ServiceCreateError) -> Self {
        match value {
            ServiceCreateError::AlreadyExists => BlackboardCreateError::AlreadyExists,
            ServiceCreateError::InsufficientPermissions => {
                BlackboardCreateError::InsufficientPermissions
            }
            ServiceCreateError::InternalFailure => BlackboardCreateError::InternalFailure,
            ServiceCreateError::IsBeingCreatedByAnotherInstance => {
                BlackboardCreateError::IsBeingCreatedByAnotherInstance
            }
            ServiceCreateError::ServiceConfigCouldNotBeCreated => {
                BlackboardCreateError::ServiceConfigCouldNotBeCreated
            }
            ServiceCreateError::ServiceInCorruptedState => {
                BlackboardCreateError::ServiceInCorruptedState
            }
            ServiceCreateError::UnableToCreateServiceTag => {
                BlackboardCreateError::UnableToCreateServiceTag
            }
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeyMemoryError {
    ValueTooLarge,
    ValueAlignmentTooLarge,
}

#[doc(hidden)]
#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, ZeroCopySend)]
pub struct KeyMemory<const CAPACITY: usize> {
    pub data: [u8; CAPACITY],
}

impl<const CAPACITY: usize> KeyMemory<CAPACITY> {
    pub fn try_from<T: Copy>(value: &T) -> Result<Self, KeyMemoryError> {
        static_assert_eq::<{ align_of::<KeyMemory<1>>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();

        let origin = "KeyMemory::try_from()";
        let msg = "Unable to create KeyMemory";

        // Replace if block with below compile-time assertion once available for generic parameters
        // static_assert_le::<{ size_of::<T>() }, CAPACITY>();
        if size_of::<T>() > CAPACITY {
            fail!(from origin, with KeyMemoryError::ValueTooLarge,
                "{} since the passed value is too large. Its size must be <= {}.", msg, CAPACITY);
        }
        // Replace if block with below compile-time assertion once available for generic parameters
        // static_assert_le::<{ align_of::<T>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();
        if align_of::<T>() > MAX_BLACKBOARD_KEY_ALIGNMENT {
            fail!(from origin, with KeyMemoryError::ValueAlignmentTooLarge,
                "{} since the alignment of the passed value is too large. The alignment must be <= {}.",
                msg, MAX_BLACKBOARD_KEY_ALIGNMENT);
        }

        let mut new_self = Self {
            data: [0; CAPACITY],
        };
        unsafe { core::ptr::copy_nonoverlapping(value, new_self.data.as_mut_ptr() as *mut T, 1) };
        Ok(new_self)
    }

    /// # Safety
    ///
    ///   * see Safety section of core::ptr::copy_nonoverlapping
    pub unsafe fn try_from_ptr(ptr: *const u8, key_layout: Layout) -> Result<Self, KeyMemoryError> {
        static_assert_eq::<{ align_of::<KeyMemory<1>>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();

        let origin = "KeyMemory::try_from_ptr()";
        let msg = "Unable to create KeyMemory";

        if key_layout.size() > CAPACITY {
            fail!(from origin, with KeyMemoryError::ValueTooLarge,
                "{} since the passed key size is too large. The size must be <= {}.", msg, CAPACITY);
        }
        if key_layout.align() > MAX_BLACKBOARD_KEY_ALIGNMENT {
            fail!(from origin, with KeyMemoryError::ValueAlignmentTooLarge,
                "{} since the alignment of the passed key is too large. The alignment must be <= {}.",
                msg, MAX_BLACKBOARD_KEY_ALIGNMENT);
        }

        let mut new_self = Self {
            data: [0; CAPACITY],
        };
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, new_self.data.as_mut_ptr(), key_layout.size())
        };
        Ok(new_self)
    }

    /// This function compares two KeyMemory<CAPACITY> for equality and is only for blackboard internal
    /// usage. It is passed to functions that require a Fn(*const u8, *const u8) -> bool so
    /// default_key_eq_comparison cannot be marked unsafe. Still, there are safety requirements which are
    /// guaranteed by the by the blackboard implementation:
    ///
    /// # Safety
    ///
    ///   * lhs and rhs must be valid pointers to valid KeyMemory<CAPACITY>
    pub fn default_key_eq_comparison<T: Eq>(lhs: *const u8, rhs: *const u8) -> bool {
        let lhs = unsafe { *(lhs as *const KeyMemory<CAPACITY>) };
        let rhs = unsafe { *(rhs as *const KeyMemory<CAPACITY>) };
        unsafe { *(lhs.data.as_ptr() as *const T) == *(rhs.data.as_ptr() as *const T) }
    }

    /// This function compares two KeyMemory<CAPACITY> for equality using the given compare function.
    /// It is passed to functions that require a Fn(*const u8, *const u8) -> bool so key_eq_comparison
    /// cannot be unsafe. Still, there are safety requirements:
    ///
    /// # Safety
    ///
    ///   * lhs and rhs must be valid pointers to valid KeyMemory<CAPACITY>
    pub fn key_eq_comparison<F: Fn(*const u8, *const u8) -> bool>(
        lhs: *const u8,
        rhs: *const u8,
        eq_func: &F,
    ) -> bool {
        let lhs = unsafe { *(lhs as *const KeyMemory<CAPACITY>) };
        let rhs = unsafe { *(rhs as *const KeyMemory<CAPACITY>) };
        eq_func(lhs.data.as_ptr(), rhs.data.as_ptr())
    }
}

#[doc(hidden)]
pub struct BuilderInternals {
    key: KeyMemory<MAX_BLACKBOARD_KEY_SIZE>,
    value_type_details: TypeDetail,
    value_writer: Box<dyn Fn(*mut u8)>,
    internal_value_size: usize,
    internal_value_alignment: usize,
    internal_value_cleanup_callback: Box<dyn FnMut()>,
}

impl Debug for BuilderInternals {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

impl Drop for BuilderInternals {
    fn drop(&mut self) {
        (self.internal_value_cleanup_callback)();
    }
}

impl BuilderInternals {
    pub fn new(
        key: KeyMemory<MAX_BLACKBOARD_KEY_SIZE>,
        value_type_details: TypeDetail,
        value_writer: Box<dyn Fn(*mut u8)>,
        value_size: usize,
        value_alignment: usize,
        value_cleanup_callback: Box<dyn FnMut()>,
    ) -> Self {
        Self {
            key,
            value_type_details,
            value_writer,
            internal_value_size: value_size,
            internal_value_alignment: value_alignment,
            internal_value_cleanup_callback: value_cleanup_callback,
        }
    }
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub(crate) struct Entry {
    pub(crate) type_details: TypeDetail,
    pub(crate) offset: AtomicU64,
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub(crate) struct Mgmt {
    pub(crate) map: RelocatableFlatMap<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>,
    pub(crate) entries: RelocatableVec<Entry>,
}

pub(crate) struct BlackboardResources<ServiceType: service::Service> {
    pub(crate) mgmt: ServiceType::BlackboardMgmt<Mgmt>,
    pub(crate) data: ServiceType::BlackboardPayload,
    pub(crate) key_eq_func: Arc<dyn Fn(*const u8, *const u8) -> bool + Send + Sync>,
}

impl<ServiceType: service::Service> Abandonable for BlackboardResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            ServiceType::BlackboardMgmt::<Mgmt>::abandon_in_place(NonNull::iox2_from_mut(
                &mut this.mgmt,
            ))
        };
        unsafe {
            ServiceType::BlackboardPayload::abandon_in_place(NonNull::iox2_from_mut(&mut this.data))
        };
    }
}

impl<ServiceType: service::Service> Debug for BlackboardResources<ServiceType> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BlackboardResources {{ mgmt: {:?}, data: {:?} }}",
            self.mgmt, self.data
        )
    }
}

impl<ServiceType: service::Service> ServiceResource for BlackboardResources<ServiceType> {
    fn acquire_ownership(&self) {
        self.data.acquire_ownership();
        self.mgmt.acquire_ownership();
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Verify {
    max_readers: bool,
    max_nodes: bool,
}

struct Builder<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> {
    base: builder::BuilderWithServiceType<ServiceType>,
    verify: Verify,
    internals: Vec<BuilderInternals>,
    override_key_type: Option<TypeDetail>,
    key_eq_func: Arc<dyn Fn(*const u8, *const u8) -> bool + Send + Sync>,
    _key: PhantomData<KeyType>,
}

impl<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> Debug for Builder<KeyType, ServiceType>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Builder<{}, {}> {{ verify_max_readers: {}, verify_max_nodes: {}, internals: {:?} }}",
            core::any::type_name::<KeyType>(),
            core::any::type_name::<ServiceType>(),
            self.verify.max_readers,
            self.verify.max_nodes,
            self.internals
        )
    }
}

impl<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> Builder<KeyType, ServiceType>
{
    fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        let mut new_self = Self {
            base,
            verify: Verify::default(),
            internals: Vec::<BuilderInternals>::new(),
            override_key_type: None,
            key_eq_func: Arc::new(|lhs: *const u8, rhs: *const u8| {
                KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::default_key_eq_comparison::<KeyType>(lhs, rhs)
            }),
            _key: PhantomData,
        };

        new_self.base.service_config.messaging_pattern = MessagingPattern::Blackboard(
            static_config::blackboard::StaticConfig::new(new_self.base.shared_node.config()),
        );

        new_self
    }

    // triggers the underlying is_service_available method to check whether the service described in base is available.
    fn is_service_available(
        &self,
        error_msg: &str,
    ) -> Result<Option<(StaticConfig, ServiceType::StaticStorage)>, ServiceState> {
        let blackboard_service_config = *self.config_details();
        match self.base.is_service_available(error_msg) {
            Ok(Some((config, storage))) => {
                if !(blackboard_service_config.type_details == config.blackboard().type_details) {
                    fail!(from self, with ServiceState::IncompatiblePayload,
                        "{} since the service offers the type \"{:?}\" which is not compatible to the requested type \"{:?}\".",
                        error_msg, &config.blackboard().type_details , blackboard_service_config.type_details);
                }

                Ok(Some((config, storage)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn config_details(&self) -> &static_config::blackboard::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::Blackboard(ref v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in Blackboard builder!");
            }
        }
    }

    fn config_details_mut(&mut self) -> &mut static_config::blackboard::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::Blackboard(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in Blackboard builder!");
            }
        }
    }

    fn prepare_config_details(&mut self) {
        match &self.override_key_type {
            None => {
                self.config_details_mut().type_details =
                    TypeDetail::new::<KeyType>(message_type_details::TypeVariant::FixedSize);
            }
            Some(details) => {
                self.config_details_mut().type_details = *details;
            }
        }
    }

    /// If the [`Service`] is created it defines how many [`Reader`](crate::port::reader::Reader)s
    /// shall be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Reader`](crate::port::reader::Reader)s must be at least supported.
    fn max_readers(&mut self, value: usize) {
        self.config_details_mut().max_readers = value;
        self.verify.max_readers = true;
    }

    /// If the [`Service`] is created it defines how many [`Node`](crate::node::Node)s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`](crate::node::Node)s must be at least supported.
    fn max_nodes(&mut self, value: usize) {
        self.config_details_mut().max_nodes = value;
        self.verify.max_nodes = true;
    }
}

/// Builder to create a new [`MessagingPattern::Blackboard`] based [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Creator<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> {
    builder: Builder<KeyType, ServiceType>,
}

impl<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> Creator<KeyType, ServiceType>
{
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            builder: Builder::new(base),
        }
    }

    /// Defines how many [`Reader`](crate::port::reader::Reader)s shall be supported at most.
    pub fn max_readers(mut self, value: usize) -> Self {
        self.builder.max_readers(value);
        self
    }

    /// Defines how many [`Node`](crate::node::Node)s shall be able to open it in parallel.
    pub fn max_nodes(mut self, value: usize) -> Self {
        self.builder.max_nodes(value);
        self
    }

    /// Adds key-value pairs to the blackboard.
    pub fn add<ValueType: ZeroCopySend + Copy + 'static>(
        mut self,
        key: KeyType,
        value: ValueType,
    ) -> Self {
        let key_mem = match KeyMemory::try_from(&key) {
            Err(_) => {
                fatal_panic!(from self,
                    "This should never happen! Calling add() with a key type that has an invalid layout.")
            }
            Ok(mem) => mem,
        };

        let internals = BuilderInternals {
            key: key_mem,
            value_type_details: TypeDetail::new::<ValueType>(
                message_type_details::TypeVariant::FixedSize,
            ),
            value_writer: Box::new(move |mem: *mut u8| {
                let mem: *mut UnrestrictedAtomic<ValueType> =
                    mem as *mut UnrestrictedAtomic<ValueType>;
                unsafe { mem.write(UnrestrictedAtomic::<ValueType>::new(value)) };
            }),
            internal_value_size: core::mem::size_of::<UnrestrictedAtomic<ValueType>>(),
            internal_value_alignment: core::mem::align_of::<UnrestrictedAtomic<ValueType>>(),
            internal_value_cleanup_callback: Box::new(|| {}),
        };

        self.builder.internals.push(internals);
        self
    }

    /// Adds key-value pairs to the blackboard where value is a default value.
    pub fn add_with_default<ValueType: ZeroCopySend + Copy + 'static + Default>(
        self,
        key: KeyType,
    ) -> Self {
        self.add(key, ValueType::default())
    }

    /// Validates configuration and overrides the invalid setting with meaningful values.
    fn adjust_configuration_to_meaningful_values(&mut self) {
        let origin = format!("{self:?}");
        let settings = self.builder.base.service_config.blackboard_mut();

        if settings.max_readers == 0 {
            warn!(from origin, "Setting the maximum amount of readers to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_readers = 1;
        }

        if settings.max_nodes == 0 {
            warn!(from origin,
                "Setting the maximum amount of nodes to 0 is not supported. Adjust it to 1, the smallest supported value.");
            settings.max_nodes = 1;
        }
    }

    /// Creates a new [`Service`].
    pub fn create(
        mut self,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardCreateError> {
        self.builder.prepare_config_details();
        self.create_impl(&AttributeSpecifier::new())
    }

    /// Creates a new [`Service`] with a set of attributes.
    pub fn create_with_attributes(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardCreateError> {
        self.builder.prepare_config_details();
        self.create_impl(attributes)
    }

    fn create_blackboard_resources(
        &self,
        msg: &str,
        service_config: &StaticConfig,
    ) -> Result<BlackboardResources<ServiceType>, ServiceCreateError> {
        let blackboard_config = *self.builder.config_details();
        let key_eq_func = self.builder.key_eq_func.clone();
        let builder_internals = self.builder.internals.as_slice();
        let shared_node = &self.builder.base.shared_node;
        // create the payload data segment for the writer
        let name = blackboard_name(service_config.unique_service_id());
        let shm_config = blackboard_data_config::<ServiceType>(shared_node.config());
        let mut payload_size = 0;
        for i in builder_internals.iter() {
            payload_size += i.internal_value_size + i.internal_value_alignment - 1;
        }
        let payload_shm = match <<ServiceType::BlackboardPayload as SharedMemory<
            iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
        >>::Builder as NamedConceptBuilder<ServiceType::BlackboardPayload>>::new(
            &name
        )
        .config(&shm_config)
        .has_ownership(false)
        .size(payload_size)
        .create(&iceoryx2_cal::shared_memory::bump_allocator::Config::default())
        {
            Ok(v) => v,
            Err(_) => {
                fail!(from self, with ServiceCreateError::ServiceInCorruptedState,
                    "{} since the blackboard payload data segment could not be created. This could indicate a corrupted system.",
                    msg);
            }
        };

        // create the management segment
        let capacity = builder_internals.len();

        let mut mgmt_config = blackboard_mgmt_config::<ServiceType, Mgmt>(shared_node.config());
        let mgmt_name = blackboard_config.type_details.type_name.as_str();

        // The name is set to be able to remove the concept when a node dies. Safe since the
        // same name is set in ServiceInternal::__internal_remove_node_from_service.
        unsafe {
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage::<
                Mgmt,
            >>::__internal_set_type_name_in_config(&mut mgmt_config, mgmt_name)
        };

        let mgmt_storage = fail!(from self, when
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<Mgmt,
            >>::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(false)
                .supplementary_size(RelocatableFlatMap::<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>::const_memory_size(capacity)+RelocatableVec::<Entry>::const_memory_size(capacity))
                .initializer(|mgmt: &mut Mgmt, allocator: &mut BumpAllocator| {
                    if unsafe {mgmt.map.init(allocator)}.is_err() || unsafe {mgmt.entries.init(allocator).is_err()} {
                        return false
                    }
                    for entry in builder_internals.iter() {
                        // write value passed to add() to payload_shm
                        let mem = match payload_shm.allocate(unsafe { Layout::from_size_align_unchecked(entry.internal_value_size, entry.internal_value_alignment) })
                        {
                            Ok(m) => m,
                            Err(_) => {
                                error!(from self, "Writing the value to the blackboard data segment failed.");
                                return false
                            }
                        };
                        (*entry.value_writer)(mem.data_ptr);
                        // write offset to value in payload_shm to entries vector
                        let res = mgmt.entries.push(Entry{type_details: entry.value_type_details, offset: AtomicU64::new(mem.offset.offset() as u64)});
                        if res.is_err() {
                            error!(from self, "Writing the value offset to the blackboard management segment failed.");
                            return false
                        }
                        // write offset index to map
                        let res = unsafe {mgmt.map.__internal_insert(entry.key, mgmt.entries.len() - 1, &*key_eq_func)};
                        if res.is_err() {
                            error!(from self, "Inserting the key-value pair into the blackboard management segment failed.");
                            return false
                        }
                    }
                    true})
                .create(Mgmt{ map: unsafe { RelocatableFlatMap::<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>::new_uninit(capacity) }, entries: unsafe {RelocatableVec::<Entry>::new_uninit(capacity)}}),
                    with ServiceCreateError::ServiceInCorruptedState, "{} since the blackboard management segment could not be created. This could indicate a corrupted system.",
                    msg);

        Ok(BlackboardResources {
            mgmt: mgmt_storage,
            data: payload_shm,
            key_eq_func,
        })
    }

    fn create_impl(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardCreateError> {
        let origin = format!("{self:?}");
        let msg = "Unable to create blackboard service";

        self.adjust_configuration_to_meaningful_values();
        if self.builder.internals.is_empty() {
            fail!(from origin,  with BlackboardCreateError::NoEntriesProvided,
                "{} without entries. At least one key-value pair is required.", msg);
        }

        let generate_dynamic_config = |service_config: &StaticConfig| {
            let blackboard_config = service_config.blackboard();
            let dynamic_config_setting = DynamicConfigSettings {
                number_of_writers: blackboard_config.max_writers,
                number_of_readers: blackboard_config.max_readers,
            };

            DynamicConfigCreationArgs {
                messaging_pattern_settings: MessagingPatternSettings::Blackboard(
                    dynamic_config_setting,
                ),
                additional_size: dynamic_config::blackboard::DynamicConfig::memory_size(
                    &dynamic_config_setting,
                ),
                max_number_of_nodes: blackboard_config.max_nodes,
            }
        };

        let service_state = self.builder.base.create(
            msg,
            attributes,
            || self.builder.is_service_available(msg),
            |_| Ok(()),
            generate_dynamic_config,
            |service_config| self.create_blackboard_resources(msg, service_config),
        )?;

        Ok(blackboard::PortFactory::new(service_state))
    }
}

impl<ServiceType: service::Service> Creator<CustomKeyMarker, ServiceType> {
    #[doc(hidden)]
    pub unsafe fn __internal_set_key_type_details(mut self, value: &TypeDetail) -> Self {
        self.builder.override_key_type = Some(*value);
        self
    }

    #[doc(hidden)]
    pub unsafe fn __internal_set_key_eq_cmp_func(
        mut self,
        key_eq_func: Box<dyn Fn(*const u8, *const u8) -> bool + Send + Sync>,
    ) -> Self {
        self.builder.key_eq_func = Arc::new(key_eq_func);
        self
    }

    #[doc(hidden)]
    pub unsafe fn __internal_add(
        mut self,
        key: *const u8,
        value: *mut u8,
        value_details: TypeDetail,
        value_cleanup: Box<dyn FnMut()>,
    ) -> Self {
        let key_type_details = match self.builder.override_key_type {
            None => {
                fatal_panic!(from self, "The key type details were not set when __internal_add was called!")
            }
            Some(details) => details,
        };
        let key_layout = match Layout::from_size_align(
            key_type_details.size,
            key_type_details.alignment,
        ) {
            Ok(layout) => layout,
            Err(_) => {
                fatal_panic!(from self, "This should never happen! Key size/alignment is invalid!")
            }
        };
        let key_mem = match unsafe { KeyMemory::try_from_ptr(key, key_layout) } {
            Ok(mem) => mem,
            Err(_) => fatal_panic!(from self, "The key type has the wrong size/alignment!"),
        };

        let value_writer = Box::new(move |raw_memory_ptr: *mut u8| unsafe {
            let ptrs = __internal_calculate_atomic_mgmt_and_payload_ptr(
                raw_memory_ptr,
                value_details.alignment,
            );
            core::ptr::copy_nonoverlapping(value, ptrs.atomic_payload_ptr, value_details.size);
        });
        let value_size = UnrestrictedAtomicMgmt::__internal_get_unrestricted_atomic_size(
            value_details.size,
            value_details.alignment,
        );
        let value_alignment = UnrestrictedAtomicMgmt::__internal_get_unrestricted_atomic_alignment(
            value_details.alignment,
        );

        let internals = BuilderInternals::new(
            key_mem,
            value_details,
            value_writer,
            value_size,
            value_alignment,
            value_cleanup,
        );

        self.builder.internals.push(internals);
        self
    }
}

/// Builder to open a [`MessagingPattern::Blackboard`] based [`Service`]s
///
/// # Example
///
/// See [`crate::service`]
#[derive(Debug)]
pub struct Opener<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> {
    builder: Builder<KeyType, ServiceType>,
}

impl<
    KeyType: Send + Sync + Eq + Clone + Copy + Debug + ZeroCopySend + Hash,
    ServiceType: service::Service,
> Opener<KeyType, ServiceType>
{
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            builder: Builder::new(base),
        }
    }

    /// Defines how many [`Reader`](crate::port::reader::Reader)s must be at least supported.
    pub fn max_readers(mut self, value: usize) -> Self {
        self.builder.max_readers(value);
        self
    }

    /// Defines how many [`Node`](crate::node::Node)s must be at least supported.
    pub fn max_nodes(mut self, value: usize) -> Self {
        self.builder.max_nodes(value);
        self
    }

    fn verify_service_configuration(
        &self,
        msg: &str,
        existing_service_config: &StaticConfig,
        required_attributes: &AttributeVerifier,
    ) -> Result<(), BlackboardOpenError> {
        let required_service_config = &self.builder.base.service_config;
        let existing_attributes = existing_service_config.attributes();
        if let Err(incompatible_key) = required_attributes.verify_requirements(existing_attributes)
        {
            fail!(from self, with BlackboardOpenError::IncompatibleAttributes,
                "{} due to incompatible service attribute key \"{}\". The following attributes {:?} are required but the service has the attributes {:?}.",
                msg, incompatible_key, required_attributes, existing_attributes);
        }

        let required_settings = required_service_config.blackboard();
        let existing_settings = match &existing_service_config.messaging_pattern {
            MessagingPattern::Blackboard(v) => v,
            p => {
                fail!(from self, with BlackboardOpenError::IncompatibleMessagingPattern,
                "{} since a service with the messaging pattern {:?} exists but MessagingPattern::Blackboard is required.", msg, p);
            }
        };

        if self.builder.verify.max_readers
            && existing_settings.max_readers < required_settings.max_readers
        {
            fail!(from self, with BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders,
                                "{} since the service supports only {} readers but a support of {} readers was requested.",
                                msg, existing_settings.max_readers, required_settings.max_readers);
        }

        if self.builder.verify.max_nodes
            && existing_settings.max_nodes < required_settings.max_nodes
        {
            fail!(from self, with BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes,
                                "{} since the service supports only {} nodes but {} are required.",
                                msg, existing_settings.max_nodes, required_settings.max_nodes);
        }

        Ok(())
    }

    /// Opens an existing [`Service`].
    pub fn open(
        self,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardOpenError> {
        self.open_with_attributes(&AttributeVerifier::new())
    }

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardOpenError> {
        self.builder.prepare_config_details();
        self.open_impl(verifier)
    }

    fn open_blackboard_resources(
        &self,
        msg: &str,
        service_config: &StaticConfig,
    ) -> Result<BlackboardResources<ServiceType>, BlackboardOpenError> {
        let shared_node = &self.builder.base.shared_node;
        let blackboard_config = self.builder.config_details();
        let key_eq_func = self.builder.key_eq_func.clone();
        let name = blackboard_name(service_config.unique_service_id());
        let mut mgmt_config = blackboard_mgmt_config::<ServiceType, Mgmt>(shared_node.config());
        let mgmt_name = blackboard_config.type_details.type_name.as_str();
        // The name was set in create_impl to be able to remove the concept when a node
        // dies. Safe since the same name is set in
        // ServiceInternal::__internal_remove_node_from_service.
        unsafe {
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<
                Mgmt,
            >>::__internal_set_type_name_in_config(
                &mut mgmt_config, mgmt_name
            )
        };
        let mgmt_storage = fail!(from self, when
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<Mgmt>
            >::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(AccessMode::ReadWrite),
            with BlackboardOpenError::ServiceInCorruptedState,
            "{} since the blackboard management information could not be opened. This could indicate a corrupted system.", msg);

        let shm_config = blackboard_data_config::<ServiceType>(shared_node.config());
        let payload_shm = match <<ServiceType::BlackboardPayload as SharedMemory<
            iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
        >>::Builder as NamedConceptBuilder<ServiceType::BlackboardPayload>>::new(
            &name
        )
        .config(&shm_config)
        .open(AccessMode::ReadWrite)
        {
            Ok(v) => v,
            Err(_) => {
                fail!(from self, with BlackboardOpenError::ServiceInCorruptedState,
                    "{} since the blackboard payload data segment could not be opened. This could indicate a corrupted system.",
                    msg);
            }
        };

        Ok(BlackboardResources {
            mgmt: mgmt_storage,
            data: payload_shm,
            key_eq_func,
        })
    }

    fn open_impl(
        self,
        required_attributes: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType, KeyType>, BlackboardOpenError> {
        let msg = "Unable to open blackboard service";

        let service_state = self.builder.base.open(
            msg,
            || self.builder.is_service_available(msg),
            |existing_service_config| -> Result<(), BlackboardOpenError> {
                self.verify_service_configuration(msg, existing_service_config, required_attributes)
            },
            |service_config| self.open_blackboard_resources(msg, service_config),
        )?;

        Ok(blackboard::PortFactory::new(service_state))
    }
}

impl<ServiceType: service::Service> Opener<CustomKeyMarker, ServiceType> {
    #[doc(hidden)]
    pub unsafe fn __internal_set_key_type_details(mut self, value: &TypeDetail) -> Self {
        self.builder.override_key_type = Some(*value);
        self
    }
}
