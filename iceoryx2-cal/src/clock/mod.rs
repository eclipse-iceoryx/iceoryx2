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

//! Clock concept
//!
//! The clock concept abstracts the source of time for every time-dependent
//! iceoryx2 operation: retrieving the current time, sleeping, timed waits, and
//! the timeout scheduling used by nodes, wait sets, reactors and events.
//!
//! Two kinds of clocks are possible:
//!
//! * a [`FreeRunningClock`], and
//! * a [`SteppedClock`] whose time advances only when the single
//!   [`TimeController`] calls [`TimeController::advance()`].

pub mod posix;
pub mod simulation_clock;

use crate::{
    event::{Event, ListenerBuilder, ListenerWaitInterface, NamedConceptBuilder, NamedConceptMgmt},
    named_concept::NamedConceptConfiguration,
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::time::Duration;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::{file_name::FileName, path::SemanticString};
use iceoryx2_log::{fail, origin};

enum_gen! {
/// Error returned by [`Clock::blocking_sleep()`]
    ClockBlockingSleepError
  entry:
    InterruptedBySignal,
    DurationOutOfRange,
    InternalError,
    ScheduleTimeoutFailed
}

enum_gen! {
/// Error returned by [`Clock::now()`]
    ClockNowError
  entry:
    InternalError
}

enum_gen! {
/// Error returned by [`ClockBuilder::open_clock()`]
    ClockBuilderOpenClockError
  entry:
    InternalError
}

enum_gen! {
/// Error returned by [`ClockListenerSchedule::schedule_on()`]
    ClockListenerScheduleError
  entry:
    ClockScheduleCapacityExceeded,
    NotificationFailed,
    InternalError
}

enum_gen! {
/// Error returned by [`ClockReactorSchedule::schedule()`]
    ClockReactorScheduleError
  entry:
    ClockScheduleCapacityExceeded,
    ListenerAttachmentFailed,
    InternalError
}

enum_gen! {
/// Error returned by [`TimeControllerBuilder::create()`]
    TimeControllerBuilderCreateError
  entry:
    AlreadyCreatedByAnotherInstance,
    InternalError
}

enum_gen! {
/// Error returned by [`TimeControllerBuilder::open()`]
    TimeControllerBuilderOpenError
  entry:
    AlreadyOpenedByAnotherInstance,
    TimeControllerDoesNotExist,
    InternalError
}

enum_gen! {
/// Error returned by [`TimeControllerBuilder::open_or_create()`]
    TimeControllerBuilderOpenOrCreateError
  entry:
    AlreadyOpenedByAnotherInstance,
    InternalError
}

enum_gen! {
/// Error returned by [`TimeController::advance()`]
    TimeControllerAdvanceError
  entry:
    NotificationFailed
}

/// The point in time at which a scheduled operation shall expire.
///
/// * [`Timeout::By`]: relative to the current time, after the given [`Duration`] elapsed.
/// * [`Timeout::At`]: at the given absolute [`TimePoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Timeout {
    /// Expire relative to the current time, after `duration` has elapsed.
    By(Duration),
    /// Expire at the absolute `time` point.
    At(TimePoint),
}

impl Timeout {
    /// Returns either the [`Duration`] stored in [`Timeout::By`] or converts the [`TimePoint`]
    /// stored in [`Timeout::At`] into a [`Duration`] based on the provided now [`TimePoint`].
    pub fn as_duration(&self, now: TimePoint) -> Duration {
        match self {
            Timeout::By(duration) => *duration,
            Timeout::At(time_point) => (*time_point - now).into(),
        }
    }
}

/// The handle returned by [`ClockReactorSchedule::schedule()`], describing how a scheduled [`Timeout`] is
/// exposed to other concepts (typically the reactor).
///
/// * [`ScheduleHandle::TranslatedTimeout`]: the clock is a [`FreeRunningClock`] and the translated timeout
///   can be forwarded unchanged to the underlying mechanism.
/// * [`ScheduleHandle::GuardAndListener`]: the clock is a [`SteppedClock`] and the
///   [`Event::Listener`](crate::event::Listener) is activated when the scheduled time is reached.
///   If the provided [`ScheduleGuard`] goes out of scope, registered timout is removed from the [`Clock`]
///   and the [`Event::Listener`](crate::event::Listener) will no longer be notified.
pub enum ScheduleHandle<'a> {
    /// A translated timeout that can be passed directly to the underlying implementation.
    TranslatedTimeout(Timeout),
    /// A trigger that is activated once the scheduled time has been reached.
    GuardAndListener((Box<dyn ScheduleGuard<'a>>, Box<dyn SynchronousMultiplexing>)),
}

/// Marker trait implemented by the guard returned from [`ClockReactorSchedule::schedule()`] and
/// [`ClockListenerSchedule::schedule_on()`].
///
/// The guard keeps a scheduled timeout valid for as long as it lives.
pub trait ScheduleGuard<'a>: 'a {}

/// Marker trait that can be implemented by [`Clock`] to identify a [`Clock`] that runs on its own,
/// continuously, uncontrolled and one can only read it.
pub trait FreeRunningClock {}

/// Marker trait that can be implemented by [`Clock`] to identify a [`Clock`] that is tied to the
/// [`TimeController`] and must be explicitly advanced.
pub trait SteppedClock {}

/// Builder that acquires a [`Clock`] and, where the implementation supports it, its [`TimeController`].
///
/// The [`Clock`] is identified by name so that the same [`Clock`] can be acquired by multiple processes.
///
/// [`FreeRunningClock`] implementations do not support a [`TimeController`].
pub trait ClockBuilder<E: Event, C: Clock<E>>: Sized {
    /// Defines the name of the [`Clock`] that shall be used.
    fn new(name: &FileName) -> Self;
    /// Sets the custom configuration of the [`Clock`].
    fn config(self, config: &C::Configuration) -> Self;
    /// Sets the custom event configuration.
    fn event_config(self, config: &E::Configuration) -> Self;
    /// Acquires the [`Clock`] with the name this builder was created for.
    fn open_clock(self) -> Result<C, ClockBuilderOpenClockError>;
}

/// Builder that creates or opens a [`TimeController`].
///
/// [`FreeRunningClock`] implementations do not provide a [`TimeController`].
pub trait TimeControllerBuilder {
    type TimeController: TimeController;

    /// Creates a new [`TimeController`].
    fn create_time_controller(
        self,
    ) -> Result<Self::TimeController, TimeControllerBuilderCreateError>;
    /// Opens an existing [`TimeController`].
    fn open_time_controller(self) -> Result<Self::TimeController, TimeControllerBuilderOpenError>;
    /// Opens the [`TimeController`] if it exists, otherwise creates it.
    fn open_or_create_time_controller(
        self,
    ) -> Result<Self::TimeController, TimeControllerBuilderOpenOrCreateError>;
}

/// The handle is returned by [`ClockListenerSchedule::schedule_on()`], describing how a scheduled
/// [`Timeout`] is exposed to other concepts (typically the [`Event::Listener`](crate::event::Listener))
///
/// * [`ScheduleOnHandle::TranslatedTimeout`]: When the clock is a [`FreeRunningClock`], the translated timeout
///   can be forwarded unchanged to the underlying mechanism. When the clock is a [`SteppedClock`], the timeout
///   will be zero which means that the timeout has already expired while scheduling and the listener was notified.
/// * [`ScheduleOnHandle::Guard`]: the clock is a [`SteppedClock`] and the
///   [`Event::Listener`](crate::event::Listener) is activated when the scheduled time is reached.
///   If the provided [`ScheduleGuard`] goes out of scope, registered timout is removed from the [`Clock`]
///   and the [`Event::Listener`](crate::event::Listener) will no longer be notified.
pub enum ScheduleOnHandle<'a> {
    /// A translated timeout that can be passed directly to the underlying implementation.
    TranslatedTimeout(Timeout),
    /// The user provided [`Event::Listener`](crate::event::Listener) is when the [`Timeout`] has
    /// passed and the [`ScheduleGuard`] is still alive.
    Guard(Box<dyn ScheduleGuard<'a>>),
}

/// Trait that defines an interface to acquire the current time.
pub trait ClockNow {
    /// Returns the current time according to this clock.
    fn now(&self) -> Result<TimePoint, ClockNowError>;
}

/// Trait that defines an interface to schedule a notification on a defined
/// [`Event::Listener`](crate::event::Listener) name.
pub trait ClockListenerSchedule: ClockNow + Send + Sync + Debug {
    /// Schedules an existing trigger, identified by `listener_name`, to be
    /// activated once the given [`Timeout`] expires.
    ///
    /// The [`Clock`] is responsible for ensuring the corresponding trigger fires at
    /// the requested point in time. The returned guard keeps the schedule valid
    /// for its lifetime.
    fn schedule_on<'a>(
        &'a self,
        timeout: Timeout,
        listener_name: &FileName,
    ) -> Result<ScheduleOnHandle<'a>, ClockListenerScheduleError>;
}

/// Trait that defines an interface to schedule a notification on a returned
/// [`Event::Listener`](crate::event::Listener).
pub trait ClockReactorSchedule: ClockNow + Send + Sync + Debug {
    /// Schedules a timeout.
    ///
    /// Depending on the [`Clock`] implementation this returns either a [`Timeout`] that
    /// can be forwarded to the underlying mechanism
    /// ([`ScheduleHandle::TranslatedTimeout`]) or a trigger that is activated
    /// when the timeout expires ([`ScheduleHandle::GuardAndListener`]). The returned [`ScheduleGuard`]
    /// keeps the schedule valid for its lifetime.
    fn schedule<'a>(
        &'a self,
        timeout: Timeout,
    ) -> Result<ScheduleHandle<'a>, ClockReactorScheduleError>;
}

/// The time-consumer interface of a [`Clock`].
///
/// A [`Clock`] provides all time-related operations required by the remaining concepts: retrieving the
/// current time, sleeping, and scheduling the timeouts. Higher layers use this interface uniformly and do
/// not need to know whether [`FreeRunningClock`] or a [`SteppedClock`] is in use.
pub trait Clock<E: Event>:
    Sized + Abandonable + ClockListenerSchedule + ClockReactorSchedule + Debug
{
    /// The configuration of the [`Clock`].
    type Configuration: NamedConceptConfiguration;
    /// The builder that acquires this [`Clock`] (and its time controller, if any).
    type Builder: ClockBuilder<E, Self>;
    /// Whether this clock supports a [`TimeController`]. `true` for [`SteppedClock`]s, `false` for
    /// [`FreeRunningClock`]s.
    const HAS_TIME_CONTROLLER_SUPPORT: bool;

    /// Blocks the calling thread until the given [`Timeout`] has expired.
    ///
    /// For a [`SteppedClock`] the thread is released according to time stepping
    /// rather than physical time.
    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError>;

    /// The default suffix of every [`Clock`]
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".clock") }
    }
}

/// The time-source interface, available only for [`SteppedClock`] implementations that
/// support explicit time advancement (i.e. simulation clocks).
///
/// Exactly one [`TimeController`] exists for a [`SteppedClock`] and is
/// responsible for advancing its time. It can run in a dedicated thread or
/// process. All [`SteppedClock`] instances associated with the same [`Clock`]
/// observe this advancement and wake scheduled operations whose timeouts have
/// expired.
pub trait TimeController: NamedConceptMgmt + Abandonable {
    /// The event type produced by triggers scheduled through the clock this controller drives.
    type E: crate::event::Event;

    /// Advances time by `duration` and activates all scheduled triggers whose deadlines have been
    /// reached.
    /// Returns the time before the advance.
    fn advance(&self, duration: Duration) -> Result<TimePoint, TimeControllerAdvanceError>;
}

#[doc(hidden)]
pub mod internal {
    use iceoryx2_log::debug;

    use crate::event::{NotifierBuilder, NotifierOpenError};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CreateTriggerError {
        FailedToOpenStubClock,
        FailedToCreateListener,
    }

    fn create_listener<E: Event + 'static>(
        name: &FileName,
        config: &E::Configuration,
        origin: &str,
        msg: &str,
    ) -> Result<Box<E::Listener>, CreateTriggerError> {
        let clock = fail!(from origin,
                    when crate::testing::create_stub_clock::<crate::clock::posix::PosixClock<crate::event::InternalStubEvent>>(),
                    with CreateTriggerError::FailedToOpenStubClock,
                    "{msg} {name} since the stub clock could not be opened.");

        let listener = fail!(from origin,
        when <E::ListenerBuilder as NamedConceptBuilder<E>>::new(name).config(config).create(Arc::new(clock)),
        with CreateTriggerError::FailedToCreateListener,
        "{msg} {name} since the listener could not be created.");

        Ok(Box::new(listener))
    }

    pub fn create_reactor_attachment<E: Event + 'static>(
        name: &FileName,
        config: &E::Configuration,
    ) -> Result<Box<dyn SynchronousMultiplexing>, CreateTriggerError>
    where
        E::Listener: SynchronousMultiplexing,
    {
        Ok(create_listener::<E>(
            name,
            config,
            origin!("create_reactor_attachment()"),
            "Failed to create reactor attachment",
        )?)
    }

    pub fn create_notifier<E: Event>(
        name: &FileName,
        config: &E::Configuration,
        origin: &impl Debug,
        msg: &str,
    ) -> Result<E::Notifier, NotifierOpenError> {
        E::NotifierBuilder::new(name)
            .config(config)
            .open()
            .inspect_err(
                |e| debug!(from origin, "{msg} since the notifier could not be opened. [{e:?}]"),
            )
    }

    pub fn create_waiter<E: Event + 'static>(
        name: &FileName,
        config: &E::Configuration,
    ) -> Result<Box<dyn ListenerWaitInterface>, CreateTriggerError> {
        Ok(create_listener::<E>(
            name,
            config,
            origin!("create_waiter()"),
            "Failed to create reactor attachment",
        )?)
    }
}
