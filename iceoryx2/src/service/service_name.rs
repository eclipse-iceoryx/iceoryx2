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
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! # Ok(())
//! # }
//! ```

use crate::constants::MAX_SERVICE_NAME_LENGTH;
use iceoryx2_bb_container::byte_string::{
    FixedSizeByteString, FixedSizeByteStringModificationError,
};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use iceoryx2_bb_log::fatal_panic;
use serde::{de::Visitor, Deserialize, Serialize};

/// Prefix used to identify internal iceoryx2 services.
///
/// This prefix is used to distinguish between user-defined services and internal
/// iceoryx2 services. Services with this prefix are considered internal and are
/// managed by the iceoryx2 system.
pub const INTERNAL_SERVICE_PREFIX: &str = "iox2://";

/// Errors that can occur when creating a [`ServiceName`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServiceNameError {
    /// The service name has invalid content (e.g., empty string).
    InvalidContent,
    /// The service name exceeds the maximum allowed length.
    ExceedsMaximumLength,
}

impl core::fmt::Display for ServiceNameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServiceNameError::{self:?}")
    }
}

impl core::error::Error for ServiceNameError {}

impl From<FixedSizeByteStringModificationError> for ServiceNameError {
    fn from(error: FixedSizeByteStringModificationError) -> Self {
        match error {
            FixedSizeByteStringModificationError::InsertWouldExceedCapacity => {
                ServiceNameError::ExceedsMaximumLength
            }
        }
    }
}

type ServiceNameString = FixedSizeByteString<MAX_SERVICE_NAME_LENGTH>;

/// The name of a [`Service`](crate::service::Service).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopySend)]
#[repr(C)]
pub struct ServiceName {
    value: ServiceNameString,
}

impl ServiceName {
    /// Creates a new [`ServiceName`].
    ///
    /// The name is not allowed to be empty nor be prefixed with "iox2://".
    pub fn new(name: &str) -> Result<Self, ServiceNameError> {
        if Self::has_iox2_prefix(name) {
            return Err(ServiceNameError::InvalidContent);
        }

        Self::__internal_new(name)
    }

    #[doc(hidden)]
    pub fn __internal_new_prefixed(name: &str) -> Result<Self, ServiceNameError> {
        Self::__internal_new(&(INTERNAL_SERVICE_PREFIX.to_owned() + name))
    }

    #[doc(hidden)]
    pub fn __internal_new(name: &str) -> Result<Self, ServiceNameError> {
        if name.is_empty() {
            return Err(ServiceNameError::InvalidContent);
        }

        let value = ServiceNameString::try_from(name).map_err(ServiceNameError::from)?;

        Ok(Self { value })
    }

    /// Returns a str reference to the [`ServiceName`]
    pub fn as_str(&self) -> &str {
        fatal_panic!(from self,
             when self.value.as_str(),
             "This should never happen! The underlying service name does not contain a valid UTF-8 string.")
    }

    /// Checks if a service is an internal iceoryx2 service.
    pub fn has_iox2_prefix(name: &str) -> bool {
        name.starts_with(INTERNAL_SERVICE_PREFIX)
    }

    /// Returns the maximum length of a [`ServiceName`].
    pub fn max_len() -> usize {
        ServiceNameString::capacity()
    }
}

impl core::fmt::Display for ServiceName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryInto<ServiceName> for &str {
    type Error = ServiceNameError;

    fn try_into(self) -> Result<ServiceName, Self::Error> {
        ServiceName::__internal_new(self)
    }
}

impl PartialEq<&str> for ServiceName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl PartialEq<&str> for &ServiceName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl core::ops::Deref for ServiceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

struct ServiceNameVisitor;

impl Visitor<'_> for ServiceNameVisitor {
    type Value = ServiceName;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing the service name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match ServiceName::__internal_new(v) {
            Ok(v) => Ok(v),
            Err(v) => Err(E::custom(format!("invalid service name provided {v:?}."))),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ServiceNameVisitor)
    }
}

impl Serialize for ServiceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
