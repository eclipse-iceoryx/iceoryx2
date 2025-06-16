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

use self::attribute::{AttributeSpecifier, AttributeVerifier};
use crate::service;
use crate::service::port_factory::blackboard;
use crate::service::static_config::messaging_pattern::MessagingPattern;
use crate::service::*;
use core::marker::PhantomData;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardOpenError {
    DoesNotExist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackboardCreateError {
    AlreadyExists,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlackboardOpenOrCreateError {
    SystemInFlux,
}

impl core::fmt::Display for BlackboardOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "BlackboardOpenOrCreateError::{:?}", self)
    }
}

impl core::error::Error for BlackboardOpenOrCreateError {}

#[derive(Debug)]
pub struct Builder<KeyType: ZeroCopySend + Debug, ServiceType: service::Service> {
    base: builder::BuilderWithServiceType<ServiceType>,
    verify_max_readers: bool,
    _key: PhantomData<KeyType>,
}

impl<KeyType: ZeroCopySend + Debug, ServiceType: service::Service> Builder<KeyType, ServiceType> {
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        let mut new_self = Self {
            base,
            verify_max_readers: false,
            _key: PhantomData,
        };

        new_self.base.service_config.messaging_pattern = MessagingPattern::Blackboard(
            static_config::blackboard::StaticConfig::new(new_self.base.shared_node.config()),
        );

        new_self
    }
    fn config_details_mut(&mut self) -> &mut static_config::blackboard::StaticConfig {
        match self.base.service_config.messaging_pattern {
            MessagingPattern::Blackboard(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in Blackboard builder!");
            }
        }
    }

    pub fn max_readers(mut self, value: usize) -> Self {
        self.config_details_mut().max_readers = value;
        self.verify_max_readers = true;
        self
    }

    pub fn add<ValueType: ZeroCopySend>(mut self, key: KeyType, value: ValueType) -> Self {
        self
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    pub fn open_or_create(
        self,
    ) -> Result<blackboard::PortFactory<ServiceType>, BlackboardOpenOrCreateError> {
        self.open_or_create_with_attributes(&AttributeVerifier::new())
    }

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes. If the [`Service`] already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the [`Service`]
    /// does not exist the required attributes will be defined in the [`Service`].
    pub fn open_or_create_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType>, BlackboardOpenOrCreateError> {
        Err(BlackboardOpenOrCreateError::SystemInFlux)
    }

    /// Opens an existing [`Service`].
    pub fn open(self) -> Result<blackboard::PortFactory<ServiceType>, BlackboardOpenError> {
        self.open_with_attributes(&AttributeVerifier::new())
    }

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        mut self,
        verifier: &AttributeVerifier,
    ) -> Result<blackboard::PortFactory<ServiceType>, BlackboardOpenError> {
        Err(BlackboardOpenError::DoesNotExist)
    }

    /// Creates a new [`Service`].
    pub fn create(mut self) -> Result<blackboard::PortFactory<ServiceType>, BlackboardCreateError> {
        self.create_impl(&AttributeSpecifier::new())
    }

    /// Creates a new [`Service`] with a set of attributes.
    pub fn create_with_attributes(
        mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType>, BlackboardCreateError> {
        self.create_impl(attributes)
    }

    fn create_impl(
        &mut self,
        attributes: &AttributeSpecifier,
    ) -> Result<blackboard::PortFactory<ServiceType>, BlackboardCreateError> {
        Err(BlackboardCreateError::AlreadyExists)
    }
}
