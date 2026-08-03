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

use core::fmt::Debug;
use core::ptr::NonNull;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

use crate::{
    config,
    service::{
        self,
        builder::{ServiceCreateError, ServiceOpenError},
        resource::{blackboard::BlackboardResources, publish_subscribe::PublishSubscribeResources},
        static_config::{StaticConfig, messaging_pattern::MessagingPattern},
    },
};

pub unsafe fn remove_stale_service_resources<ServiceType: service::Service>(
    config: &config::Config,
    static_config: &StaticConfig,
) -> Result<(), RemoveStaleResourcesError> {
    match static_config.messaging_pattern() {
        MessagingPattern::Blackboard(_) => unsafe {
            BlackboardResources::<ServiceType>::remove_stale_resources(config, static_config)
        },
        MessagingPattern::RequestResponse(_) => Ok(()),
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

    fn create(
        static_config: &StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, ServiceCreateError>;

    fn open(
        static_config: &StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, ServiceOpenError>;

    unsafe fn remove_stale_resources(
        config: &config::Config,
        static_config: &StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError>;

    /// Acquires the ownership of the additional resources. When the objects go out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);
}

#[derive(Debug)]
pub struct NoResource;
impl ServiceResource for NoResource {
    type Config = ();

    fn create(
        _static_config: &StaticConfig,
        _resource_config: &Self::Config,
    ) -> Result<Self, ServiceCreateError> {
        Ok(Self {})
    }

    fn open(
        _static_config: &StaticConfig,
        _resource_config: &Self::Config,
    ) -> Result<Self, ServiceOpenError> {
        Ok(Self {})
    }

    fn acquire_ownership(&self) {}

    unsafe fn remove_stale_resources(
        _config: &config::Config,
        _static_config: &StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        Ok(())
    }
}

impl Abandonable for NoResource {
    unsafe fn abandon_in_place(_this: NonNull<Self>) {}
}
