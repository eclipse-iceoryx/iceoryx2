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
use core::fmt::Display;

use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;

use crate::service_description::ServiceDescription;

/// What a service resolves to, decided from both of its sides.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // the mirror description is carried by value, resolutions are rare and short-lived
pub enum Resolution<E, F> {
    /// The local service is bridged, under this description on the
    /// opposing side.
    Exported(E),
    /// A mirror is created from this description and bridged, under
    /// this description on the opposing side.
    Imported(ServiceDescription, E),
    /// Nothing is bridged, and why.
    Refused(F),
    /// Nothing is bridged, the backend does not cover the service. Not a
    /// refusal, there is nothing to warn about.
    OutOfScope,
}

/// Resolves each service from its local description and the remote
/// descriptions keyed to it.
pub trait Resolver<S: Service> {
    /// What identifies a description in the opposing side's listing.
    type RemoteId: Ord + Clone + Display;
    /// What the opposing side lists, and what a bridged service is
    /// opened as there.
    type RemoteDescription: Clone + PartialEq + 'static;
    /// Why a service is refused.
    type Refusal: Display + PartialEq;

    /// The service a remote description belongs to, hashed with the
    /// name hasher of `S`. None if it is out of scope, a refusal if it
    /// is in scope but belongs to no service.
    fn service_hash(
        &self,
        remote: &Self::RemoteDescription,
    ) -> Result<Option<ServiceHash>, Self::Refusal>;

    /// The resolution of one service.
    fn resolve<'a>(
        &self,
        local: Option<&ServiceDescription>,
        remotes: impl Iterator<Item = &'a Self::RemoteDescription> + Clone,
    ) -> Resolution<Self::RemoteDescription, Self::Refusal>;
}

/// A reference to a resolver is a resolver.
impl<S: Service, R: Resolver<S>> Resolver<S> for &R {
    type RemoteId = R::RemoteId;
    type RemoteDescription = R::RemoteDescription;
    type Refusal = R::Refusal;

    fn service_hash(
        &self,
        remote: &Self::RemoteDescription,
    ) -> Result<Option<ServiceHash>, Self::Refusal> {
        (**self).service_hash(remote)
    }

    fn resolve<'a>(
        &self,
        local: Option<&ServiceDescription>,
        remotes: impl Iterator<Item = &'a Self::RemoteDescription> + Clone,
    ) -> Resolution<Self::RemoteDescription, Self::Refusal> {
        (**self).resolve(local, remotes)
    }
}
