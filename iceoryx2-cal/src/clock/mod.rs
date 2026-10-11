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

use iceoryx2_bb_si_units::{TimePoint, TimeSpan};

use crate::event::event_state::EventState;

pub enum Timeout {
    By(TimeSpan),
    At(TimePoint),
}

pub enum ScheduleHandle<E: EventState, T: crate::event::Event<E>> {
    TranslatedTimeout(Timeout),
    Trigger(T),
    _Data(PhantomData<E>),
}
