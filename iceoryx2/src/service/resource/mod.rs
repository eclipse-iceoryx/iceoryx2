// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

extern crate alloc;

pub mod blackboard;
pub mod publish_subscribe;
pub mod request_response;
pub(crate) mod type_definition;

use core::ptr::NonNull;
use core::{fmt::Debug, marker::PhantomData};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

use crate::{
    config,
    service::{
        self,
        builder::{ServiceCreateError, ServiceOpenError},
        resource::{
            blackboard::BlackboardResources, publish_subscribe::PublishSubscribeResources,
            request_response::RequestResponseResources,
        },
        static_config::{StaticConfig, messaging_pattern::MessagingPattern},
    },
};

pub unsafe fn remove_stale_service_resources<ServiceType: service::Service>(
    config: &config::Config,
    static_config: &StaticConfig<ServiceType>,
) -> Result<(), RemoveStaleResourcesError> {
    match static_config.messaging_pattern() {
        MessagingPattern::Blackboard(_) => unsafe {
            BlackboardResources::<ServiceType>::remove_stale_resources(config, static_config)
        },
        MessagingPattern::RequestResponse(_) => unsafe {
            RequestResponseResources::<ServiceType>::remove_stale_resources(config, static_config)
        },
        MessagingPattern::Event(_) => Ok(()),
        MessagingPattern::PublishSubscribe(_) => unsafe {
            PublishSubscribeResources::<ServiceType>::remove_stale_resources(config, static_config)
        },
    }
}

enum_gen! {
    RemoveStaleResourcesError
  entry:
    InsufficientPermissions,
    InterruptedBySignal,
    InternalFailure
}

/// Represents resources a service could use and have to be cleaned up when no owners
/// are left
pub trait ServiceResource: Abandonable + Debug + Send {
    type Config;
    type ServiceType: service::Service;

    fn create(
        static_config: &StaticConfig<Self::ServiceType>,
        resource_config: &Self::Config,
    ) -> Result<Self, ServiceCreateError>;

    fn open(
        static_config: &StaticConfig<Self::ServiceType>,
        resource_config: &Self::Config,
    ) -> Result<Self, ServiceOpenError>;

    unsafe fn remove_stale_resources(
        config: &config::Config,
        static_config: &StaticConfig<Self::ServiceType>,
    ) -> Result<(), RemoveStaleResourcesError>;

    /// Acquires the ownership of the additional resources. When the objects go out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);
}

#[derive(Debug)]
pub struct NoResource<ServiceType: service::Service> {
    _service: PhantomData<ServiceType>,
}
impl<Service: service::Service> ServiceResource for NoResource<Service> {
    type Config = ();
    type ServiceType = Service;

    fn create(
        _static_config: &StaticConfig<Service>,
        _resource_config: &Self::Config,
    ) -> Result<Self, ServiceCreateError> {
        Ok(Self {
            _service: PhantomData,
        })
    }

    fn open(
        _static_config: &StaticConfig<Service>,
        _resource_config: &Self::Config,
    ) -> Result<Self, ServiceOpenError> {
        Ok(Self {
            _service: PhantomData,
        })
    }

    fn acquire_ownership(&self) {}

    unsafe fn remove_stale_resources(
        _config: &config::Config,
        _static_config: &StaticConfig<Service>,
    ) -> Result<(), RemoveStaleResourcesError> {
        Ok(())
    }
}

impl<Service: service::Service> Abandonable for NoResource<Service> {
    unsafe fn abandon_in_place(_this: NonNull<Self>) {}
}

impl<Service: service::Service> Default for NoResource<Service> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Service: service::Service> NoResource<Service> {
    pub fn new() -> Self {
        Self {
            _service: PhantomData,
        }
    }
}
