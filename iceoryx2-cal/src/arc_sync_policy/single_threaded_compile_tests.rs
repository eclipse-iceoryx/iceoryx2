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

/// ```compile_fail
/// use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
/// type Policy = iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<u64>;
///
/// fn my_concurrent_function<T: ArcSyncPolicy<u64> + Send>(value: &T) {}
///
/// let my_thing = Policy::new(1234).unwrap();
/// // fails here since this policy does not implement `Send`
/// my_concurrent_function(&my_thing);
/// ```
#[cfg(doctest)]
fn single_threaded_does_not_implement_send() {}

/// ```compile_fail
/// use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
/// type Policy = iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<u64>;
///
/// fn my_concurrent_function<T: ArcSyncPolicy<u64> + Sync>(value: &T) {}
///
/// let my_thing = Policy::new(1234).unwrap();
/// // fails here since this policy does not implement `Sync`
/// my_concurrent_function(&my_thing);
/// ```
#[cfg(doctest)]
fn single_threaded_does_not_implement_sync() {}
