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
use core::time::Duration;

use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::file_name::FileName;

use crate::event::{NamedConceptBuilder, NamedConceptMgmt, event_state::EventState};

pub enum Timeout {
    By(Duration),
    At(TimePoint),
}

pub enum ScheduleHandle<S: EventState, E: crate::event::Event<S>> {
    TranslatedTimeout(Timeout),
    Trigger(E),
    _Data(PhantomData<S>),
}

pub struct ScheduleGuard<'a, S: EventState, E: crate::event::Event<S>, C: Clock<S, E>> {
    clock: &'a C,
    generation_counter: u64,
    _data_s: PhantomData<S>,
    _data_e: PhantomData<E>,
}

pub trait ClockBuilder<
    S: EventState,
    E: crate::event::Event<S>,
    C: Clock<S, E>,
    T: TimeController<S, E>,
>: NamedConceptBuilder<C> + Sized
{
    fn new(name: &FileName) -> Result<Self, ()>;
    fn acquire(self) -> Result<C, ()>;
    fn time_controller(self) -> Result<T, ()>;
}

pub trait Clock<S: EventState, E: crate::event::Event<S>>: Sized + NamedConceptMgmt {
    fn schedule<'a>(
        &'a self,
        timeout: Timeout,
    ) -> (ScheduleGuard<'a, S, E, Self>, ScheduleHandle<S, E>);
    fn schedule_on<'a>(
        &'a self,
        timeout: Timeout,
        trigger_name: &FileName,
    ) -> (ScheduleGuard<'a, S, E, Self>, Timeout);

    fn blocking_sleep(&self, timeout: Timeout);

    fn now(&self) -> Result<TimePoint, ()>;
}

pub trait TimeControllerBuilder<S: EventState, E: crate::event::Event<S>, T: TimeController<S, E>> {
    fn create(self) -> Result<T, ()>;
    fn open(self) -> Result<T, ()>;
    fn open_or_create(self) -> Result<T, ()>;
}

pub trait TimeController<S: EventState, E: crate::event::Event<S>> {
    fn advance(&self, duration: Duration) -> TimePoint;
}
