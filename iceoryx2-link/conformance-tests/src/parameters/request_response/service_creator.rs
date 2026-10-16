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

use core::fmt::Debug;
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2::service::builder::request_response::Builder;
use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::parameters::UserHeader;

/// The creator of request-response services of the request payload type
/// `Q` and the response payload type `R`, each `T` or `[T]`.
pub struct ServiceCreator<Q: ?Sized, R: ?Sized>(PhantomData<Q>, PhantomData<R>);

/// Creates and opens a request-response service of a pair of payload
/// types. iceoryx2 builds the service differently for each combination
/// of fixed-size and slice payloads, so the pair decides.
pub trait CreatesRequestResponseService {
    type Request: IceoryxSend + Debug + ?Sized;
    type Response: IceoryxSend + Debug + ?Sized;

    /// Creates the service `builder` describes.
    fn create<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Self::Request, QH, Self::Response, RH, S>,
    ) -> PortFactory<S, Self::Request, QH, Self::Response, RH>;

    /// Opens the service `builder` describes.
    fn open<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Self::Request, QH, Self::Response, RH, S>,
    ) -> PortFactory<S, Self::Request, QH, Self::Response, RH>;
}

impl<Q: IceoryxSend + Debug, R: IceoryxSend + Debug> CreatesRequestResponseService
    for ServiceCreator<Q, R>
{
    type Request = Q;
    type Response = R;

    fn create<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Q, QH, R, RH, S>,
    ) -> PortFactory<S, Q, QH, R, RH> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Q, QH, R, RH, S>,
    ) -> PortFactory<S, Q, QH, R, RH> {
        builder.open().expect("service is opened")
    }
}

impl<Q: ZeroCopySend + Debug, R: IceoryxSend + Debug> CreatesRequestResponseService
    for ServiceCreator<[Q], R>
{
    type Request = [Q];
    type Response = R;

    fn create<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<[Q], QH, R, RH, S>,
    ) -> PortFactory<S, [Q], QH, R, RH> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<[Q], QH, R, RH, S>,
    ) -> PortFactory<S, [Q], QH, R, RH> {
        builder.open().expect("service is opened")
    }
}

impl<Q: IceoryxSend + Debug, R: ZeroCopySend + Debug> CreatesRequestResponseService
    for ServiceCreator<Q, [R]>
{
    type Request = Q;
    type Response = [R];

    fn create<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Q, QH, [R], RH, S>,
    ) -> PortFactory<S, Q, QH, [R], RH> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<Q, QH, [R], RH, S>,
    ) -> PortFactory<S, Q, QH, [R], RH> {
        builder.open().expect("service is opened")
    }
}

impl<Q: ZeroCopySend + Debug, R: ZeroCopySend + Debug> CreatesRequestResponseService
    for ServiceCreator<[Q], [R]>
{
    type Request = [Q];
    type Response = [R];

    fn create<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<[Q], QH, [R], RH, S>,
    ) -> PortFactory<S, [Q], QH, [R], RH> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, QH: UserHeader, RH: UserHeader>(
        builder: Builder<[Q], QH, [R], RH, S>,
    ) -> PortFactory<S, [Q], QH, [R], RH> {
        builder.open().expect("service is opened")
    }
}
