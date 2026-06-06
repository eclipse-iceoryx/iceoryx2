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

use crate::{
    dynamic_storage,
    event::trigger::{
        State,
        semaphore::{GenericSemaphoreTrigger, SemaphoreMgmt},
        socket_pair::GenericSocketPairTrigger,
        unix_datagram_socket::GenericUnixDatagramSocketTrigger,
    },
};
use iceoryx2_bb_lock_free::mpmc::{
    bit_set::RelocatableBitSet, counting_bit_set::RelocatableCountingBitSet,
};

pub type UnixDatagramShmBitSet = GenericUnixDatagramSocketTrigger<
    RelocatableBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableBitSet, ()>>,
>;

pub type UnixDatagramShmCountingBitSet = GenericUnixDatagramSocketTrigger<
    RelocatableCountingBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableCountingBitSet, ()>>,
>;

pub type SocketPairBitSet = GenericSocketPairTrigger<RelocatableBitSet>;
pub type SocketPairCountingBitSet = GenericSocketPairTrigger<RelocatableCountingBitSet>;

pub type SemaphoreShmBitSet = GenericSemaphoreTrigger<
    RelocatableBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableBitSet, SemaphoreMgmt>>,
>;

pub type SemaphoreShmCountingBitSet = GenericSemaphoreTrigger<
    RelocatableCountingBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableCountingBitSet, SemaphoreMgmt>>,
>;
