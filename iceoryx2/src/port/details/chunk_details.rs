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

extern crate alloc;

use alloc::sync::Arc;
use iceoryx2_cal::shm_allocator::PointerOffset;

#[derive(Debug)]
pub(crate) struct ChunkDetails<Service: crate::service::Service> {
    pub(crate) connection: Arc<super::receiver::Connection<Service>>,
    pub(crate) offset: PointerOffset,
    pub(crate) origin: u128,
}
