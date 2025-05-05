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

#[doc(hidden)]
pub mod details {
    use core::{fmt::Debug, marker::PhantomData, sync::atomic::Ordering, time::Duration};
    use iceoryx2_bb_log::{debug, fail};
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_system_types::{file_name::FileName, path::Path};
    use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicUsize};

    use crate::{
        dynamic_storage::{
            DynamicStorage, DynamicStorageBuilder, DynamicStorageCreateError,
            DynamicStorageOpenError,
        },
        event::{
            id_tracker::IdTracker, signal_mechanism::SignalMechanism, Event, ListenerCreateError,
            NotifierCreateError, NotifierNotifyError, TriggerId,
        },
        named_concept::{
            NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt,
        },
    };

    const TRIGGER_ID_DEFAULT_MAX: TriggerId = TriggerId::new(u16::MAX as _);

    #[derive(Debug)]
    #[repr(C)]
    pub struct Management<Tracker: IdTracker, WaitMechanism: SignalMechanism> {
        id_tracker: Tracker,
        signal_mechanism: WaitMechanism,
        reference_counter: IoxAtomicUsize,
        has_listener: IoxAtomicBool,
    }

    #[derive(PartialEq, Eq)]
    pub struct Configuration<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        suffix: FileName,
        prefix: FileName,
        path: Path,
        _tracker: PhantomData<Tracker>,
        _wait_mechanism: PhantomData<WaitMechanism>,
        _storage: PhantomData<Storage>,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Default for Configuration<Tracker, WaitMechanism, Storage>
    {
        fn default() -> Self {
            Self {
                path: EventImpl::<Tracker, WaitMechanism, Storage>::default_path_hint(),
                suffix: EventImpl::<Tracker, WaitMechanism, Storage>::default_suffix(),
                prefix: EventImpl::<Tracker, WaitMechanism, Storage>::default_prefix(),
                _tracker: PhantomData,
                _wait_mechanism: PhantomData,
                _storage: PhantomData,
            }
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Configuration<Tracker, WaitMechanism, Storage>
    {
        fn convert(&self) -> <Storage as NamedConceptMgmt>::Configuration {
            <Storage as NamedConceptMgmt>::Configuration::default()
                .prefix(&self.prefix)
                .suffix(&self.suffix)
                .path_hint(&self.path)
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Debug for Configuration<Tracker, WaitMechanism, Storage>
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "Configuration<{}, {}, {}> {{ suffix: {}, prefix: {}, path: {} }}",
                core::any::type_name::<Tracker>(),
                core::any::type_name::<WaitMechanism>(),
                core::any::type_name::<Storage>(),
                self.suffix,
                self.prefix,
                self.path
            )
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Clone for Configuration<Tracker, WaitMechanism, Storage>
    {
        fn clone(&self) -> Self {
            Self {
                suffix: self.suffix.clone(),
                prefix: self.prefix.clone(),
                path: self.path.clone(),
                _tracker: PhantomData,
                _wait_mechanism: PhantomData,
                _storage: PhantomData,
            }
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConceptConfiguration for Configuration<Tracker, WaitMechanism, Storage>
    {
        fn prefix(mut self, value: &FileName) -> Self {
            self.prefix = value.clone();
            self
        }

        fn get_prefix(&self) -> &FileName {
            &self.prefix
        }

        fn suffix(mut self, value: &FileName) -> Self {
            self.suffix = value.clone();
            self
        }

        fn path_hint(mut self, value: &Path) -> Self {
            self.path = value.clone();
            self
        }

        fn get_suffix(&self) -> &FileName {
            &self.suffix
        }

        fn get_path_hint(&self) -> &Path {
            &self.path
        }
    }

    #[derive(Debug)]
    pub struct EventImpl<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        _tracker: PhantomData<Tracker>,
        _wait_mechanism: PhantomData<WaitMechanism>,
        _storage: PhantomData<Storage>,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConceptMgmt for EventImpl<Tracker, WaitMechanism, Storage>
    {
        type Configuration = Configuration<Tracker, WaitMechanism, Storage>;

        fn does_exist_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
            Ok(fail!(from "Event::does_exist_cfg()",
                    when Storage::does_exist_cfg(name, &cfg.convert()),
                    "Failed to check if Event \"{}\" exists.",
                    name))
        }

        fn list_cfg(
            cfg: &Self::Configuration,
        ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
            Ok(fail!(from "Event::list_cfg()",
                    when Storage::list_cfg(&cfg.convert()),
                    "Failed to list all Events."))
        }

        unsafe fn remove_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
            Ok(fail!(from "Event::remove_cfg()",
                    when Storage::remove_cfg(name, &cfg.convert()),
                    "Failed to remove Event \"{}\".", name))
        }

        fn remove_path_hint(
            _value: &Path,
        ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
            Ok(())
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Event for EventImpl<Tracker, WaitMechanism, Storage>
    {
        type Notifier = Notifier<Tracker, WaitMechanism, Storage>;
        type NotifierBuilder = NotifierBuilder<Tracker, WaitMechanism, Storage>;
        type Listener = Listener<Tracker, WaitMechanism, Storage>;
        type ListenerBuilder = ListenerBuilder<Tracker, WaitMechanism, Storage>;

        fn has_trigger_id_limit() -> bool {
            true
        }
    }

    #[derive(Debug)]
    pub struct Notifier<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        storage: Storage,
        _tracker: PhantomData<Tracker>,
        _wait_mechanism: PhantomData<WaitMechanism>,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Drop for Notifier<Tracker, WaitMechanism, Storage>
    {
        fn drop(&mut self) {
            if self
                .storage
                .get()
                .reference_counter
                .fetch_sub(1, Ordering::Relaxed)
                == 1
            {
                self.storage.acquire_ownership();
            }
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConcept for Notifier<Tracker, WaitMechanism, Storage>
    {
        fn name(&self) -> &FileName {
            self.storage.name()
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > crate::event::Notifier for Notifier<Tracker, WaitMechanism, Storage>
    {
        fn trigger_id_max(&self) -> TriggerId {
            self.storage.get().id_tracker.trigger_id_max()
        }

        fn notify(&self, id: crate::event::TriggerId) -> Result<(), NotifierNotifyError> {
            let msg = "Failed to notify listener";
            if !self.storage.get().has_listener.load(Ordering::Relaxed) {
                fail!(from self, with NotifierNotifyError::Disconnected,
                    "{} since the listener is no longer connected.", msg);
            }

            if self.storage.get().id_tracker.trigger_id_max() < id {
                fail!(from self, with NotifierNotifyError::TriggerIdOutOfBounds,
                    "{} since the TriggerId {:?} is greater than the max supported TriggerId {:?}.",
                    msg, id, self.storage.get().id_tracker.trigger_id_max());
            }

            unsafe { self.storage.get().id_tracker.add(id)? };
            unsafe { self.storage.get().signal_mechanism.notify()? };
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct NotifierBuilder<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        name: FileName,
        config: Configuration<Tracker, WaitMechanism, Storage>,
        creation_timeout: Duration,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConceptBuilder<EventImpl<Tracker, WaitMechanism, Storage>>
        for NotifierBuilder<Tracker, WaitMechanism, Storage>
    {
        fn new(name: &FileName) -> Self {
            Self {
                name: name.clone(),
                creation_timeout: Duration::ZERO,
                config: Configuration::default(),
            }
        }

        fn config(
            mut self,
            config: &<EventImpl<Tracker, WaitMechanism, Storage> as NamedConceptMgmt>::Configuration,
        ) -> Self {
            self.config = config.clone();
            self
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > crate::event::NotifierBuilder<EventImpl<Tracker, WaitMechanism, Storage>>
        for NotifierBuilder<Tracker, WaitMechanism, Storage>
    {
        fn timeout(mut self, timeout: Duration) -> Self {
            self.creation_timeout = timeout;
            self
        }

        fn open(
            self,
        ) -> Result<
            <EventImpl<Tracker, WaitMechanism, Storage> as crate::event::Event>::Notifier,
            crate::event::NotifierCreateError,
        > {
            let msg = "Failed to open Notifier";

            match Storage::Builder::new(&self.name)
                .config(&self.config.convert())
                .timeout(self.creation_timeout)
                .open()
            {
                Ok(storage) => {
                    let mut ref_count = storage.get().reference_counter.load(Ordering::Relaxed);

                    loop {
                        if !storage.get().has_listener.load(Ordering::Relaxed) || ref_count == 0 {
                            fail!(from self, with NotifierCreateError::DoesNotExist,
                            "{} since it has no listener and will no longer exist.", msg);
                        }

                        match storage.get().reference_counter.compare_exchange(
                            ref_count,
                            ref_count + 1,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(v) => ref_count = v,
                        };
                    }

                    Ok(Notifier {
                        storage,
                        _tracker: PhantomData,
                        _wait_mechanism: PhantomData,
                    })
                }
                Err(DynamicStorageOpenError::DoesNotExist) => {
                    fail!(from self, with NotifierCreateError::DoesNotExist,
                        "{} since it does not exist.", msg);
                }
                Err(DynamicStorageOpenError::VersionMismatch) => {
                    fail!(from self, with NotifierCreateError::VersionMismatch,
                        "{} since the version of the existing construct does not match.", msg);
                }
                Err(DynamicStorageOpenError::InitializationNotYetFinalized) => {
                    fail!(from self, with NotifierCreateError::InitializationNotYetFinalized,
                        "{} since the initialization is after a timeout of {:?} still not finalized..",
                        msg, self.creation_timeout);
                }
                Err(e) => {
                    fail!(from self, with NotifierCreateError::InternalFailure,
                        "{} due to an internal failure ({:?}).", msg, e);
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct Listener<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        storage: Storage,
        _tracker: PhantomData<Tracker>,
        _wait_mechanism: PhantomData<WaitMechanism>,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > Drop for Listener<Tracker, WaitMechanism, Storage>
    {
        fn drop(&mut self) {
            self.storage
                .get()
                .has_listener
                .store(false, Ordering::Relaxed);

            if self
                .storage
                .get()
                .reference_counter
                .fetch_sub(1, Ordering::Relaxed)
                == 1
            {
                self.storage.acquire_ownership();
            }
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConcept for Listener<Tracker, WaitMechanism, Storage>
    {
        fn name(&self) -> &FileName {
            self.storage.name()
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > crate::event::Listener for Listener<Tracker, WaitMechanism, Storage>
    {
        fn try_wait_one(
            &self,
        ) -> Result<Option<crate::event::TriggerId>, crate::event::ListenerWaitError> {
            // collect all notifications until no more are available, otherwise it is possible
            // that blocking_wait and timed_wait are becoming non-blocking when the same id is
            // triggered multiple times in combination with a bitset id_tracker since the bit is
            // set only once but the signal is triggered whenever the user sets the bit.
            //
            // of course one could check if the id_tracker has already set the bit, but this is
            // not possible on every implemenation. a bitset can check it a mpmc::queue maybe not.
            while unsafe { self.storage.get().signal_mechanism.try_wait()? } {}
            Ok(unsafe { self.storage.get().id_tracker.acquire() })
        }

        fn timed_wait_one(
            &self,
            timeout: Duration,
        ) -> Result<Option<crate::event::TriggerId>, crate::event::ListenerWaitError> {
            if let Some(id) = self.try_wait_one()? {
                return Ok(Some(id));
            }

            Ok(unsafe {
                self.storage
                    .get()
                    .signal_mechanism
                    .timed_wait(timeout)?
                    .then_some(self.storage.get().id_tracker.acquire())
                    .flatten()
            })
        }

        fn blocking_wait_one(
            &self,
        ) -> Result<Option<crate::event::TriggerId>, crate::event::ListenerWaitError> {
            if let Some(id) = self.try_wait_one()? {
                return Ok(Some(id));
            }

            unsafe { self.storage.get().signal_mechanism.blocking_wait()? };
            Ok(unsafe { self.storage.get().id_tracker.acquire() })
        }

        fn try_wait_all<F: FnMut(TriggerId)>(
            &self,
            callback: F,
        ) -> Result<(), crate::event::ListenerWaitError> {
            // We have to collect all signals first since we collect
            // all trigger notifications afterwards. It is also important that
            // the signals are collected first so that timed or blocking wait
            // do not miss signal notifications despite a signal was already
            // delivered.
            // But this may lead to spurious wakeups.
            while unsafe { self.storage.get().signal_mechanism.try_wait()? } {}
            unsafe { self.storage.get().id_tracker.acquire_all(callback) };
            Ok(())
        }

        fn timed_wait_all<F: FnMut(TriggerId)>(
            &self,
            callback: F,
            timeout: Duration,
        ) -> Result<(), crate::event::ListenerWaitError> {
            unsafe { self.storage.get().signal_mechanism.timed_wait(timeout)? };
            self.try_wait_all(callback)
        }

        fn blocking_wait_all<F: FnMut(TriggerId)>(
            &self,
            callback: F,
        ) -> Result<(), crate::event::ListenerWaitError> {
            unsafe { self.storage.get().signal_mechanism.blocking_wait()? };
            self.try_wait_all(callback)
        }
    }

    #[derive(Debug)]
    pub struct ListenerBuilder<
        Tracker: IdTracker,
        WaitMechanism: SignalMechanism,
        Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
    > {
        name: FileName,
        config: Configuration<Tracker, WaitMechanism, Storage>,
        trigger_id_max: TriggerId,
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > NamedConceptBuilder<EventImpl<Tracker, WaitMechanism, Storage>>
        for ListenerBuilder<Tracker, WaitMechanism, Storage>
    {
        fn new(name: &FileName) -> Self {
            Self {
                name: name.clone(),
                config: Configuration::default(),
                trigger_id_max: TRIGGER_ID_DEFAULT_MAX,
            }
        }

        fn config(
            mut self,
            config: &<EventImpl<Tracker, WaitMechanism, Storage> as NamedConceptMgmt>::Configuration,
        ) -> Self {
            self.config = config.clone();
            self
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > ListenerBuilder<Tracker, WaitMechanism, Storage>
    {
        fn init(
            mgmt: &mut Management<Tracker, WaitMechanism>,
            allocator: &mut BumpAllocator,
        ) -> bool {
            let origin = "init()";
            if unsafe { mgmt.id_tracker.init(allocator).is_err() } {
                debug!(from origin, "Unable to initialize IdTracker.");
                return false;
            }
            if unsafe { mgmt.signal_mechanism.init().is_err() } {
                debug!(from origin, "Unable to initialize SignalMechanism.");
                return false;
            }

            true
        }
    }

    impl<
            Tracker: IdTracker,
            WaitMechanism: SignalMechanism,
            Storage: DynamicStorage<Management<Tracker, WaitMechanism>>,
        > crate::event::ListenerBuilder<EventImpl<Tracker, WaitMechanism, Storage>>
        for ListenerBuilder<Tracker, WaitMechanism, Storage>
    {
        fn trigger_id_max(mut self, id: crate::event::TriggerId) -> Self {
            self.trigger_id_max = id;
            self
        }

        fn create(
            self,
        ) -> Result<
            <EventImpl<Tracker, WaitMechanism, Storage> as crate::event::Event>::Listener,
            ListenerCreateError,
        > {
            let msg = "Failed to create Listener";
            let id_tracker_capacity = self.trigger_id_max.as_value() + 1;

            match Storage::Builder::new(&self.name)
                .config(&self.config.convert())
                .supplementary_size(Tracker::memory_size(id_tracker_capacity))
                .initializer(Self::init)
                .has_ownership(false)
                .create(Management {
                    id_tracker: unsafe { Tracker::new_uninit(id_tracker_capacity) },
                    signal_mechanism: WaitMechanism::new(),
                    reference_counter: IoxAtomicUsize::new(1),
                    has_listener: IoxAtomicBool::new(true),
                }) {
                Ok(storage) => Ok(Listener {
                    storage,
                    _tracker: PhantomData,
                    _wait_mechanism: PhantomData,
                }),
                Err(DynamicStorageCreateError::AlreadyExists) => {
                    fail!(from self, with ListenerCreateError::AlreadyExists,
                        "{} since it already exists.", msg);
                }
                Err(DynamicStorageCreateError::InsufficientPermissions) => {
                    fail!(from self, with ListenerCreateError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
                }
                Err(e) => {
                    fail!(from self, with ListenerCreateError::InternalFailure,
                        "{} due to an internal failure ({:?}).", msg, e);
                }
            }
        }
    }
}
