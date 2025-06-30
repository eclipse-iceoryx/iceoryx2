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

//! Provides a POSIX [`SignalHandler`]. It provides an interface to register custom callbacks for
//! signals, to perform a blocking wait until a certain signal arrived (for instance like CTRL+c) and
//! tracks signals which were received by the process.
//!
//! # Examples
//!
//! ## Callbacks for signals
//!
//! ```
//! use iceoryx2_bb_posix::signal::*;
//!
//! fn some_call_which_may_emits_sigabrt() {}
//!
//! fn my_signal_callback(signal: FetchableSignal) {
//!     println!("Signal {:?} received.", signal);
//! }
//!
//! {
//!     let _guard = SignalHandler::register(FetchableSignal::Abort, &my_signal_callback);
//!     some_call_which_may_emits_sigabrt();
//! }
//! ```
//!
//! ## Perform tasks until CTRL+c was pressed.
//!
//! ```no_run
//! use iceoryx2_bb_posix::signal::*;
//!
//! fn some_task() {}
//!
//! while SignalHandler::last_signal() != Some(NonFatalFetchableSignal::Terminate) {
//!     some_task();
//! }
//! ```
//!
//! ## Wait until CTRL+c was pressed.
//!
//! ```no_run
//! use iceoryx2_bb_posix::signal::*;
//!
//! SignalHandler::wait_for_signal(NonFatalFetchableSignal::Terminate);
//! ```
use core::{
    fmt::{Debug, Display},
    time::Duration,
};

use crate::{
    adaptive_wait::*,
    clock::{ClockType, NanosleepError, Time, TimeError},
    mutex::*,
};
use core::sync::atomic::Ordering;
use enum_iterator::{all, Sequence};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;
use iceoryx2_pal_posix::posix::{Errno, MemZeroedStruct};
use iceoryx2_pal_posix::*;
use lazy_static::lazy_static;

macro_rules! define_signals {
    {fetchable: $($entry:ident = $nn:ident::$value:ident),*
     fatal_fetchable: $($fatal_entry:ident = $fatal_nn:ident::$fatal_value:ident),*
     unknown_translates_to: $unknown_entry:ident = $unknown_nn:ident::$unknown_value:ident
     unfetchable: $($uentry:ident = $unn:ident::$uvalue:ident),*
    } => {
        /// Represents signals which cannot be fetched by a signal handler.
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
        #[repr(i32)]
        pub enum UnfetchableSignal {
          $($uentry = $unn::$uvalue),*,
        }

        enum_gen! {
            /// Represents signals that can be fetched and are non-fatal, meaning that they do not
            /// indicate that the process is somehow corrupted. For example: `SIGUSR1` or `SIGINT`.
            #[derive(Sequence)]
            NonFatalFetchableSignal
          unknown_translates_to:
            $unknown_entry = $unknown_nn::$unknown_value
          entry:
            $($entry = $nn::$value),*
        }

        /// Represents signals that can be fetched and are fatal, meaning that they
        /// indicate that the process is somehow corrupted and should terminate.
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
        #[repr(i32)]
        pub enum FatalFetchableSignal {
          $($fatal_entry = $fatal_nn::$fatal_value),*,
        }

        enum_gen! {
            /// Represents all signals which can be fetched by a signal handler.
            #[derive(Sequence)]
            FetchableSignal
          unknown_translates_to:
            $unknown_entry = $unknown_nn::$unknown_value
          entry:
            $($entry = $nn::$value),*,
            $($fatal_entry = $fatal_nn::$fatal_value),*
        }

        /// Represents all known POSIX signals
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
        #[repr(i32)]
        pub enum Signal {
            $($entry = $nn::$value),*,
            $($uentry = $unn::$uvalue),*,
            $($fatal_entry = $fatal_nn::$fatal_value),*,
            $unknown_entry = $unknown_nn::$unknown_value
        }

        impl From<UnfetchableSignal> for Signal {
            fn from(v: UnfetchableSignal) -> Self {
                match v {
                    $(UnfetchableSignal::$uentry => Signal::$uentry),*,
                }
            }
        }

        impl From<FatalFetchableSignal> for FetchableSignal {
            fn from(v: FatalFetchableSignal) -> Self {
                match v {
                    $(FatalFetchableSignal::$fatal_entry => FetchableSignal::$fatal_entry),*
                }
            }
        }

        impl From<FatalFetchableSignal> for Signal {
            fn from(v: FatalFetchableSignal) -> Self {
                match v {
                    $(FatalFetchableSignal::$fatal_entry => Signal::$fatal_entry),*
                }
            }
        }

        impl From<NonFatalFetchableSignal> for FetchableSignal {
            fn from(v: NonFatalFetchableSignal) -> Self {
                match v {
                    $(NonFatalFetchableSignal::$entry => FetchableSignal::$entry),*,
                    NonFatalFetchableSignal::$unknown_entry => FetchableSignal::$unknown_entry,
                }
            }
        }

        impl From<NonFatalFetchableSignal> for Signal {
            fn from(v: NonFatalFetchableSignal) -> Self {
                match v {
                    $(NonFatalFetchableSignal::$entry => Signal::$entry),*,
                    NonFatalFetchableSignal::$unknown_entry => Signal::$unknown_entry,
                }
            }
        }

        impl From<FetchableSignal> for Signal {
            fn from(v: FetchableSignal) -> Self {
                match v {
                    $(FetchableSignal::$entry => Signal::$entry),*,
                    $(FetchableSignal::$fatal_entry => Signal::$fatal_entry),*,
                    FetchableSignal::$unknown_entry => Signal::$unknown_entry,
                }
            }
        }
    };
}

define_signals! {
  fetchable:
    //TODO: https://github.com/eclipse-iceoryx/iceoryx2/issues/81
    //  comment in Continue again when the bindgen bug is fixed
    //Continue = posix::SIGCONT,
    Abort = posix::SIGABRT,
    Interrupt = posix::SIGINT,
    TerminalQuit = posix::SIGQUIT,
    Terminate = posix::SIGTERM,
    TerminalStop = posix::SIGTSTP,
    BackgroundProcessReadAttempt = posix::SIGTTIN,
    BackgroundProcessWriteAttempt = posix::SIGTTOU,
    UserDefined1 = posix::SIGUSR1,
    ProfilingTimerExpired = posix::SIGPROF,
    UrgentDataAvailableAtSocket = posix::SIGURG,
    VirtualTimerExpired = posix::SIGVTALRM
  fatal_fetchable:
    Alarm = posix::SIGALRM,
    BadSystemCall = posix::SIGSYS,
    BrokenPipe = posix::SIGPIPE,
    Bus = posix::SIGBUS,
    Child = posix::SIGCHLD,
    CpuTimeLimitExceeded = posix::SIGXCPU,
    FileSizeLimitExceeded = posix::SIGXFSZ,
    FloatingPointError = posix::SIGFPE,
    Hangup = posix::SIGHUP,
    IllegalInstruction = posix::SIGILL,
    SegmentationFault = posix::SIGSEGV,
    TraceTrap = posix::SIGTRAP
  unknown_translates_to:
    UserDefined2 = posix::SIGUSR2
  unfetchable:
    Kill = posix::SIGKILL,
    StopExecution = posix::SIGSTOP
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignalRegisterError {
    AlreadyRegistered,
}

enum_gen! { SignalWaitError
  mapping:
    NanosleepError,
    AdaptiveWaitError,
    TimeError
}

enum_gen! {
    /// The SignalError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward SignalError as more generic return value when a method
    /// returns a Signal***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    SignalError
  generalization:
    FailedToRegister <= SignalRegisterError,
    FailedToWait <= SignalWaitError
}

#[derive(Debug)]
struct SignalDetail {
    signal: FetchableSignal,
    state: posix::sigaction_t,
}

impl SignalDetail {
    pub fn new(signal: FetchableSignal, state: posix::sigaction_t) -> Self {
        Self { signal, state }
    }
}

/// Guards a registered signal and can be acquired from the [`SignalHandler`] with [`SignalHandler::register()`]
/// or [`SignalHandler::register_multiple_signals()`]
/// When it goes out of scope it deregisters the signal. It is not allowed to register a signal
/// more than once.
pub struct SignalGuard {
    signals: Option<Vec<SignalDetail>>,
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        for signal in self.signals.take().unwrap() {
            SignalHandler::instance().restore_previous_state(signal);
        }
    }
}

static LAST_SIGNAL: IoxAtomicUsize = IoxAtomicUsize::new(posix::MAX_SIGNAL_VALUE);

/// Manages POSIX signal handling. It provides an interface to register custom callbacks for
/// signals, to perform a blocking wait until a certain signal arrived (for instance like CTRL+c) and
/// tracks signals which were received by the process.
///
/// This class must be a singleton class otherwise one could register multiple signal handlers
/// for the same signal.
pub struct SignalHandler {
    registered_signals: [Option<&'static dyn Fn(FetchableSignal)>; posix::MAX_SIGNAL_VALUE],
    do_repeat_eintr_call: bool,
}
unsafe impl Send for SignalHandler {}

impl Debug for SignalHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for SignalHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut registered_signals: Vec<FetchableSignal> = vec![];
        for i in 0..posix::MAX_SIGNAL_VALUE {
            if self.registered_signals[i].is_some() {
                registered_signals.push((i as i32).into());
            }
        }
        write!(
            f,
            "SignalHandler {{ registered_signals: {registered_signals:?} }}"
        )
    }
}

extern "C" fn handler(signal: posix::int) {
    capture_signal(signal);
    if let Some(callback) = SignalHandler::instance().get_callback_for_signal(signal) {
        callback(signal.into());
    }
}

extern "C" fn capture_signal(signal: posix::int) {
    LAST_SIGNAL.store(signal as usize, Ordering::Relaxed);
}

impl Drop for SignalHandler {
    fn drop(&mut self) {
        for signal in all::<FetchableSignal>().collect::<Vec<_>>() {
            self.register_raw_signal(signal, posix::SIG_DFL as posix::sighandler_t);
        }
    }
}

impl SignalHandler {
    /// Registers a callback for a specified signal and returns a [`SignalGuard`]. When the
    /// signal guard goes out of scope the callback is unregistered.
    ///
    /// ```
    /// use iceoryx2_bb_posix::signal::*;
    ///
    /// fn custom_callback(signal: FetchableSignal) {
    ///     println!("Received: {:?}", signal);
    /// }
    ///
    /// {
    ///   let _guard = SignalHandler::register(FetchableSignal::Interrupt, &custom_callback);
    ///   // do stuff which maybe emits signals
    /// }
    /// // the _guard is out of scope again and the custom_callback is unregistered for
    /// // FetchableSignal::Interrupt
    /// ```
    pub fn register<F: Fn(FetchableSignal)>(
        signal: FetchableSignal,
        callback: &'static F,
    ) -> Result<SignalGuard, SignalRegisterError> {
        let s = vec![signal];
        Self::register_multiple_signals(&s, callback)
    }

    /// Registers a callback for multiple signals.
    pub fn register_multiple_signals<F: Fn(FetchableSignal)>(
        signals: &Vec<FetchableSignal>,
        callback: &'static F,
    ) -> Result<SignalGuard, SignalRegisterError> {
        let mut previous_state = vec![];

        {
            let mut sighandle = Self::instance();
            for signal in signals {
                previous_state.push(SignalDetail {
                    signal: *signal,
                    state: sighandle.register_signal(*signal, callback)?,
                });
            }
        }

        Ok(SignalGuard {
            signals: Some(previous_state),
        })
    }

    /// Calls a provided callable and fetches possible signals which where raised indirectly by
    /// the call. This is helpful for instance when a low level C call can fail by emitting a
    /// signal. On example is `memset` when it writes on a preallocated chunk but the actual
    /// memory of the operating system does not suffice then it emits `SIGABRT`
    pub fn call_and_fetch<F: FnOnce()>(call: F) -> Option<NonFatalFetchableSignal> {
        {
            let _sighandle = Self::instance();
            LAST_SIGNAL.store(posix::MAX_SIGNAL_VALUE, Ordering::Relaxed);

            call();
        }

        match LAST_SIGNAL.load(Ordering::Relaxed) {
            posix::MAX_SIGNAL_VALUE => None,
            v => Some((v as i32).into()),
        }
    }

    /// Returns the last signal which was raised. After the call the last signal is reset and would
    /// return [`None`] on the next call when no new signal was raised again.
    pub fn last_signal() -> Option<NonFatalFetchableSignal> {
        Self::instance();
        match LAST_SIGNAL.swap(posix::MAX_SIGNAL_VALUE, Ordering::Relaxed) {
            posix::MAX_SIGNAL_VALUE => None,
            v => Some((v as i32).into()),
        }
    }

    /// Returns true if ([`NonFatalFetchableSignal::Interrupt`] or
    /// [`NonFatalFetchableSignal::Terminate`]) was emitted
    /// for instance by pressing CTRL+c, otherwise false
    pub fn termination_requested() -> bool {
        let last_signal = Self::last_signal();
        last_signal == Some(NonFatalFetchableSignal::Interrupt)
            || last_signal == Some(NonFatalFetchableSignal::Terminate)
    }

    /// Blocks until the provided signal was raised or an error occurred.
    /// ```no_run
    /// use iceoryx2_bb_posix::signal::*;
    ///
    /// SignalHandler::wait_for_signal(NonFatalFetchableSignal::Terminate);
    /// ```
    pub fn wait_for_signal(signal: NonFatalFetchableSignal) -> Result<(), SignalWaitError> {
        let s = vec![signal];
        Self::wait_for_multiple_signals(&s)
    }

    /// Blocks until the provided vector signals was raised or an error occurred.
    pub fn wait_for_multiple_signals(
        signals: &[NonFatalFetchableSignal],
    ) -> Result<(), SignalWaitError> {
        Self::instance();

        let origin = "Signal::wait_for_multiple_signals";
        let msg = "Unable to wait for multiple signals";

        let mut wait = fail!(from origin, when AdaptiveWaitBuilder::new()
            .clock_type(ClockType::Monotonic)
            .create(), "{} since the underlying adaptive wait could not be created.", msg);

        LAST_SIGNAL.store(posix::MAX_SIGNAL_VALUE, Ordering::Relaxed);
        fail!(from "Signal::wait_for_multiple_signals", when wait.wait_while(|| {
            !signals
                .iter().any(|e| *e as i32 == LAST_SIGNAL.load(Ordering::Relaxed) as i32)
        }), "Failed to wait for signal in SignalHandler.");

        Ok(())
    }

    /// Blocks until the provided signal was raised or the timeout was reached. If the signal was
    /// raised it returns true otherwise false.
    /// ```ignore
    /// use iceoryx2_bb_posix::signal::*;
    /// use core::time::Duration;
    ///
    /// let result = SignalHandler::timed_wait_for_signal(
    ///                     FetchableSignal::Terminate, Duration::from_millis(10)).unwrap();
    ///
    /// match result {
    ///   true => println!("signal was raised"),
    ///   false => println!("timeout was hit")
    /// }
    /// ```
    pub fn timed_wait_for_signal(
        signal: NonFatalFetchableSignal,
        timeout: Duration,
    ) -> Result<bool, SignalWaitError> {
        let signals = vec![signal];
        Ok(Self::timed_wait_for_multiple_signals(&signals, timeout)?.is_some())
    }

    /// Blocks until one of the provided signals was raised or the timeout was reached. If one of the
    /// signals raised it returns its value, otherwise [`None`].
    pub fn timed_wait_for_multiple_signals(
        signals: &Vec<NonFatalFetchableSignal>,
        timeout: Duration,
    ) -> Result<Option<NonFatalFetchableSignal>, SignalWaitError> {
        Self::instance();

        let origin = "Signal::timed_wait_for_multiple_signals";
        let msg = "Failed to wait with the timeout ".to_string()
            + &timeout.as_secs_f64().to_string()
            + "s on multiple signals";

        let mut wait = fail!(from origin, when AdaptiveWaitBuilder::new()
            .clock_type(ClockType::Monotonic)
            .create(), "{} since the adaptive wait could not be created.", msg);

        let start = fail!(from "Signal::timed_wait_for_multiple_signals", when Time::now_with_clock(ClockType::Monotonic),
                    "Failed to acquire current time.");

        LAST_SIGNAL.store(posix::MAX_SIGNAL_VALUE, Ordering::Relaxed);
        loop {
            for signal in signals {
                if *signal as i32 == LAST_SIGNAL.load(Ordering::Relaxed) as i32 {
                    return Ok(Some(*signal));
                }
            }
            if fail!(from "Signal::timed_wait_for_multiple_signals", when start.elapsed(),
                        "Failed to acquire elapsed time in SignalHandler::timed_wait")
                >= timeout
            {
                return Ok(None);
            }

            fail!(from "Signal::timed_wait_for_multiple_signals", when wait.wait(),
                        "Failed to wait for signal in SignalHandler.");
        }
    }

    /// Certain POSIX calls like for instance `close` can be interrupted by the signal
    /// [`FetchableSignal::Interrupt`]. One approach to handle this signal is to repeat the call.
    /// This method calls the underlying c function `sigaction` with the `SA_RESTART` flag.
    /// Function which can emit [`FetchableSignal::Interrupt`] are repeated when this signal was raised and no
    /// longer fail with [`FetchableSignal::Interrupt`]
    pub fn set_auto_repeat_call_for_interrupt_signals() {
        let mut sighandle = SignalHandler::instance();
        sighandle.do_repeat_eintr_call = true;

        let is_signal_registered = sighandle.is_signal_registered(FetchableSignal::Interrupt);

        sighandle.register_raw_signal(
            FetchableSignal::Interrupt,
            match is_signal_registered {
                true => handler as posix::sighandler_t,
                false => capture_signal as posix::sighandler_t,
            },
        );
    }

    fn new() -> Self {
        let mut sighandle = SignalHandler {
            registered_signals: core::array::from_fn(|_| None),
            do_repeat_eintr_call: false,
        };

        for signal in all::<NonFatalFetchableSignal>().collect::<Vec<_>>() {
            sighandle.register_raw_signal(signal.into(), capture_signal as posix::sighandler_t);
        }

        sighandle
    }

    fn instance() -> MutexGuard<'static, Self> {
        lazy_static! {
            static ref HANDLE: MutexHandle<SignalHandler> = MutexHandle::new();
            static ref MTX: Mutex<'static, 'static, SignalHandler> = fatal_panic!(from "SignalHandler::instance",
                when MutexBuilder::new().create(SignalHandler::new(), &HANDLE),
                "Unable to create global signal handler");
        }

        fatal_panic!(from "SignalHandler::instance", when MTX.lock(),
            "Unable to acquire global SignalHandler")
    }

    fn get_callback_for_signal(
        &self,
        signal: posix::int,
    ) -> &Option<&'static dyn Fn(FetchableSignal)> {
        if signal as usize >= posix::MAX_SIGNAL_VALUE {
            return &None;
        }

        &self.registered_signals[signal as usize]
    }

    fn register_signal_from_state(&mut self, details: SignalDetail) -> posix::sigaction_t {
        let mut adjusted_state = details.state;
        if self.do_repeat_eintr_call {
            adjusted_state.set_flags(adjusted_state.flags() | posix::SA_RESTART);
        }

        let mut previous_action = posix::sigaction_t::new_zeroed();

        let sigaction_return = unsafe {
            posix::sigaction(details.signal as i32, &adjusted_state, &mut previous_action)
        };

        if sigaction_return != 0 {
            fatal_panic!(from self, "This should never happen! posix::sigaction returned {}. Unable to register raw signal since sigaction was called with invalid parameters: {:?} which lead to error: {:?}.",
                                     sigaction_return, details, Errno::get());
        }

        previous_action
    }

    fn register_raw_signal(
        &mut self,
        signal: FetchableSignal,
        callback: posix::sighandler_t,
    ) -> posix::sigaction_t {
        let mut action = posix::sigaction_t::new_zeroed();
        action.set_handler(callback);
        self.register_signal_from_state(SignalDetail::new(signal, action))
    }

    fn register_signal<F: Fn(FetchableSignal)>(
        &mut self,
        signal: FetchableSignal,
        callback: &'static F,
    ) -> Result<posix::sigaction_t, SignalRegisterError> {
        if self.is_signal_registered(signal) {
            fail!(from self, with SignalRegisterError::AlreadyRegistered, "The Signal::{:?} is already registered.", signal);
        }

        let previous_action = self.register_raw_signal(signal, handler as posix::sighandler_t);
        self.registered_signals[signal as usize] = Some(callback);

        Ok(previous_action)
    }

    fn is_signal_registered(&self, signal: FetchableSignal) -> bool {
        self.registered_signals[signal as usize].is_some()
    }

    fn restore_previous_state(&mut self, detail: SignalDetail) {
        if !self.is_signal_registered(detail.signal) {
            fatal_panic!(from self, "This should never happen! Restoring a signal which was never registered.");
        }

        self.registered_signals[detail.signal as usize] = None;
        self.register_signal_from_state(detail);
    }
}
