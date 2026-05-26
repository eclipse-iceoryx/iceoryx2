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

use core::marker::PhantomData;

use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_posix::semaphore::{UnnamedSemaphore, UnnamedSemaphoreHandle};

use crate::{dynamic_storage::DynamicStorage, event::event_state::EventState};

struct State<E: EventState, Mgmt> {
    event: E,
    handle: Mgmt,
}

pub struct Semaphore<E: EventState, Storage: DynamicStorage<State<E, UnnamedSemaphoreHandle>>> {
    notification_counter: AtomicU64,
    semaphore: UnnamedSemaphore<'static>,
    storage: Storage,
    _data: PhantomData<E>,
}
