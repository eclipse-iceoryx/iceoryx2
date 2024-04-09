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

use crate::dynamic_storage::posix_shared_memory::Storage;
use crate::event::common::details::EventImpl;
use crate::event::common::details::Management;
use crate::event::signal_mechanism::semaphore::Semaphore;
use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;

pub type Event =
    EventImpl<RelocatableBitSet, Semaphore, Storage<Management<RelocatableBitSet, Semaphore>>>;
