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

//! Decides which endpoint represents a local service on the middleware
//! and which local service represents an endpoint, from their names and
//! settings.
//!
//! Either direction answers `Ok(None)` when the mapping does not cover the
//! input, and `Err` when it covers the input but refuses it, for example
//! an endpoint whose settings differ from what the mapping defines. The
//! error is reported as the reason.
//!
//! ```rust,ignore
//! impl Mapping for MyMapping {
//!     type EndpointSettings = MyEndpointSettings;
//!     type Error = MyRefusal;
//!
//!     fn remote(&self, local: &ServiceSettings) -> Result<Option<MyEndpointSettings>, MyRefusal> {
//!         match self.entry_named(&local.name) {
//!             None => Ok(None),
//!             Some(entry) if entry.settings != local.pattern => Err(MyRefusal::Settings),
//!             Some(entry) => Ok(Some(entry.endpoint.clone())),
//!         }
//!     }
//!
//!     fn local(&self, remote: &MyEndpointSettings) -> Result<Option<ServiceSettings>, MyRefusal> {
//!         // The same the other way round.
//!     }
//! }
//! ```

use core::error::Error;

use iceoryx2_link_backend::service_description::Identified;
use iceoryx2_link_backend::service_description::ServiceSettings;

/// The abstraction over how services and a middleware's endpoints
/// correspond, by name and settings.
pub trait Mapping {
    /// The middleware's settings of an endpoint.
    type EndpointSettings: Clone + PartialEq + Identified + 'static;
    type Error: Error + PartialEq;

    /// The local settings an endpoint maps to.
    fn local(
        &self,
        remote: &Self::EndpointSettings,
    ) -> Result<Option<ServiceSettings>, Self::Error>;

    /// The endpoint settings a local service maps to.
    fn remote(
        &self,
        local: &ServiceSettings,
    ) -> Result<Option<Self::EndpointSettings>, Self::Error>;
}
