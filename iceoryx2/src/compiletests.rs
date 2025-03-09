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

/// ```compile_fail
/// use iceoryx2::prelude::*;
/// fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
///
/// let service = zero_copy::Service::new(&service_name)
///     .publish_subscribe()
///     .open_or_create::<u64>()?;
///
/// let publisher = service.publisher().create()?;
///
/// let mut sample = publisher.loan_uninit()?;
/// sample.payload_mut().write(1234);
///
/// publisher.send(sample)?; // should fail to compile since sample contains a 'MaybeUninit<T>' instead of a 'T'
///
/// Ok(())
/// }
/// ```
#[cfg(doctest)]
fn sending_uninitialized_sample_fails_to_compile() {}

/// ```compile_fail
/// use iceoryx2::prelude::*;
///
/// struct Wrapper(u64);
///
/// fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
///
/// let service = zero_copy::Service::new(&service_name)
///     .publish_subscribe()
///     .open_or_create::<Wrapper>()?;
///
/// let publisher = service.publisher().create()?;
///
/// let sample = publisher.loan()?; // should fail to compile since 'Wrapper' does not implement 'Default'
///
/// Ok(())
/// }
/// ```
#[cfg(doctest)]
fn loan_with_type_not_implementing_default_fails_to_compile() {}
