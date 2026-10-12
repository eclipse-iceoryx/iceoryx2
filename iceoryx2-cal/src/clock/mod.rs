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

pub mod posix_monotonic;

use core::time::Duration;

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::file_name::FileName;

use crate::event::{NamedConceptBuilder, NamedConceptMgmt};

enum_gen! {
    ClockBlockingSleepError
  entry:
    InterruptedBySignal,
    DurationOutOfRange,
    InternalError
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Timeout {
    By(Duration),
    At(TimePoint),
}

pub enum ScheduleHandle<E: crate::event::Event> {
    TranslatedTimeout(Timeout),
    Trigger(E),
}

pub trait ScheduleGuard {}

pub trait ClockBuilder<C: Clock, T: TimeController>: Sized {
    fn new(name: &FileName) -> Result<Self, ()>;
    fn acquire(self) -> Result<C, ()>;
    fn time_controller(self) -> Result<T, ()>;
}

pub trait Clock: Sized {
    type E: crate::event::Event;
    type Guard: ScheduleGuard;

    fn schedule<'a>(&'a self, timeout: Timeout) -> (Self::Guard, ScheduleHandle<Self::E>);
    fn schedule_on<'a>(
        &'a self,
        timeout: Timeout,
        trigger_name: &FileName,
    ) -> (Self::Guard, Timeout);

    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError>;

    fn now(&self) -> Result<TimePoint, ()>;
}

pub trait TimeControllerBuilder<T: TimeController>: NamedConceptBuilder<T> {
    fn create(self) -> Result<T, ()>;
    fn open(self) -> Result<T, ()>;
    fn open_or_create(self) -> Result<T, ()>;
}

pub trait TimeController: NamedConceptMgmt {
    type E: crate::event::Event;

    fn advance(&self, duration: Duration) -> TimePoint;
}
