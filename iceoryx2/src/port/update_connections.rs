// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_cal::{
    shared_memory::SharedMemoryOpenError, zero_copy_connection::ZeroCopyCreationError,
};

enum_gen! { ConnectionFailure
  mapping:
    ZeroCopyCreationError to FailedToEstablishConnection,
    SharedMemoryOpenError to UnableToMapPublishersDataSegment
}

impl std::fmt::Display for ConnectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ConnectionFailure {}

/// Explicitly triggers and update of all connections and performs underlying management work.
pub trait UpdateConnections {
    /// Explicitly updates all connections to the [`crate::port::subscriber::Subscriber`]s. This is
    /// required to be called whenever a new [`crate::port::subscriber::Subscriber`] connected to
    /// the service. It is done implicitly whenever [`crate::sample_mut::SampleMut::send()`] or [`Publisher::send_copy()`]
    /// is called.
    /// When a [`crate::port::subscriber::Subscriber`] is connected that requires a history this
    /// call will deliver it.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// publisher.send_copy(1234)?;
    ///
    /// let subscriber = service.subscriber().create()?;
    ///
    /// // updates all connections and delivers history to new participants
    /// publisher.update_connections();
    ///
    /// println!("history received {:?}", subscriber.receive()?.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    fn update_connections(&self) -> Result<(), ConnectionFailure>;
}
