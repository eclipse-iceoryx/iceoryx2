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

//! Abstraction for the posix message queue. The [`MessageQueueReceiver`] can receive only, the
//! [`MessageQueueSender`] can send only and the [`MessageQueueDuplex`] can send and receive.
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::message_queue::*;
//!
//! let mq_name = FileName::new(b"myMqName").unwrap();
//! let mut receiver = MessageQueueBuilder::new(&mq_name)
//!                     .capacity(8)
//!                     .clock_type(ClockType::Monotonic)
//!                     .create_receiver::<u64>(CreationMode::PurgeAndCreate)
//!                     .expect("Failed to create message queue");
//!
//! let mut sender = MessageQueueBuilder::new(&mq_name)
//!                     // a mq with a capacity of at least 5 is required
//!                     .capacity(5)
//!                     .clock_type(ClockType::Monotonic)
//!                     .open_sender::<u64>()
//!                     .expect("Failed to open message queue");
//!
//! sender.try_send(&1234);
//! sender.try_send_with_prio(&5678, 2);
//!
//! let data = receiver.try_receive().unwrap();
//! assert!(data.is_some());
//! assert_eq!(data.as_ref().unwrap().value, 5678);
//! assert_eq!(data.as_ref().unwrap().priority, 2);
//! ```

use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::time::Duration;

pub use crate::clock::ClockType;
pub use crate::creation_mode::CreationMode;
pub use crate::permission::Permission;
pub use iceoryx2_bb_container::semantic_string::SemanticString;
pub use iceoryx2_bb_system_types::file_name::FileName;

use crate::adaptive_wait::*;
use crate::clock::{AsTimespec, Time};
use crate::clock::{NanosleepError, TimeError};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{error, fail, fatal_panic};
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
use iceoryx2_pal_posix::*;

enum_gen! {
    /// Describes failures when creating a message queue with [`MessageQueueBuilder::create_sender()`],
    /// [`MessageQueueBuilder::create_receiver()`] or [`MessageQueueBuilder::create_duplex()`].
    MessageQueueCreationError
  entry:
    AlreadyExist,
    PermissionDenied,
    Interrupt,
    InvalidMessageSizeOrNumberOfMessages,
    PerProcessFileHandleLimitReached,
    SystemMessageQueueLimitReached,
    OutOfResources,
    UnknownError(i32)
  mapping:
    MessageQueueRemoveError,
    MessageQueueOpenError
}

enum_gen! {
    /// Failures when receiving messages with [`MessageQueueReceiverInterface::timed_receive()`].
    MessageQueueTimedReceiveError
  mapping:
    MessageQueueReceiveError,
    NanosleepError,
    AdaptiveWaitError,
    TimeError
}

enum_gen! {
    /// Failures when sending messages with [`MessageQueueSenderInterface::timed_send()`] or
    /// [`MessageQueueSenderInterface::timed_send_with_prio()`].
    MessageQueueTimedSendError
  mapping:
    MessageQueueSendError,
    NanosleepError,
    AdaptiveWaitError,
    TimeError
}

/// Failures when removing an existing message queue with [`remove_message_queue()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MessageQueueRemoveError {
    PermissionDenied,
    DoesNotExist,
    Interrupt,
    UnknownError(i32),
}

/// Describes failures when opening a message queue with [`MessageQueueBuilder::open_sender()`],
/// [`MessageQueueBuilder::open_receiver()`] or [`MessageQueueBuilder::open_duplex()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MessageQueueOpenError {
    DoesNotExist,
    Interrupt,
    PermissionDenied,
    PerProcessFileHandleLimitReached,
    SystemMessageQueueLimitReached,
    CapacitySmallerThanRequired,
    MessageSizeDoesNotFit,
    UnknownError(i32),
}

/// Failures when sending messages with [`MessageQueueSenderInterface::try_send()`],
/// [`MessageQueueSenderInterface::try_send_with_prio()`],
/// [`MessageQueueSenderInterface::blocking_send()`] or
/// [`MessageQueueSenderInterface::blocking_send_with_prio()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MessageQueueSendError {
    QueueIsFull,
    Interrupt,
    PriorityIsInvalid,
    UnknownError(i32),
}

/// Failures when receiving messages with [`MessageQueueReceiverInterface::try_receive()`] or
/// [`MessageQueueReceiverInterface::blocking_receive()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MessageQueueReceiveError {
    CorruptedMessageReceived,
    MessageSizeInconsistencies,
    Interrupt,
    UnknownError(i32),
}

#[derive(Debug, Clone)]
#[repr(i32)]
enum TransmissionMode {
    Sending = posix::O_WRONLY,
    Receiving = posix::O_RDONLY,
    SendingAndReceiving = posix::O_RDWR,
}

/// A message received through a [`MessageQueueReceiver`] or [`MessageQueueDuplex`]. It contains
/// the value of the message and the priority.
#[derive(Debug, PartialEq, Eq)]
pub struct Message<T> {
    pub value: T,
    pub priority: u32,
}

/// If the capacity is not explicitly set with [`MessageQueueBuilder::capacity()`] this value
/// will be used as default for the queue capacity.
pub const DEFAULT_MESSAGE_QUEUE_CAPACITY: usize = 8;

/// The default permissions of the message queue, can be overridden with
/// [`MessageQueueBuilder::permissions()`].
pub const DEFAULT_MESSAGE_QUEUE_PERMISSIONS: Permission = Permission::OWNER_ALL;

/// Creates or opens a [`MessageQueueSender`], [`MessageQueueReceiver`] or [`MessageQueueDuplex`].
#[derive(Debug)]
pub struct MessageQueueBuilder {
    name: FileName,
    capacity: usize,
    permissions: Permission,
    clock_type: ClockType,

    max_message_size: usize,
    transmission_mode: TransmissionMode,
}

impl MessageQueueBuilder {
    /// Creates a new builder and sets the name of the message queue which shall be created.
    pub fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            capacity: DEFAULT_MESSAGE_QUEUE_CAPACITY,
            max_message_size: 0,
            permissions: DEFAULT_MESSAGE_QUEUE_PERMISSIONS,
            transmission_mode: TransmissionMode::Sending,
            clock_type: ClockType::default(),
        }
    }

    /// Sets the capacity of the queue. If this is not set the value
    /// [`DEFAULT_MESSAGE_QUEUE_CAPACITY`] will be used.
    pub fn capacity(mut self, value: usize) -> Self {
        self.capacity = value;
        self
    }

    /// Defines the [`ClockType`] which is required in [`MessageQueueSenderInterface::timed_send()`]
    /// or [`MessageQueueReceiverInterface::timed_receive()`].
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    /// Sets the permissions of the message queue.
    pub fn permissions(mut self, value: Permission) -> Self {
        self.permissions = value;
        self
    }

    /// Creates a new [`MessageQueueSender`].
    pub fn create_sender<T>(
        mut self,
        mode: CreationMode,
    ) -> Result<MessageQueueSender<T>, MessageQueueCreationError> {
        self.transmission_mode = TransmissionMode::Sending;
        MessageQueueSender::create(self, mode)
    }

    /// Creates a new [`MessageQueueReceiver`].
    pub fn create_receiver<T>(
        mut self,
        mode: CreationMode,
    ) -> Result<MessageQueueReceiver<T>, MessageQueueCreationError> {
        self.transmission_mode = TransmissionMode::Receiving;
        MessageQueueReceiver::create(self, mode)
    }

    /// Creates a new [`MessageQueueDuplex`].
    pub fn create_duplex<T>(
        mut self,
        mode: CreationMode,
    ) -> Result<MessageQueueDuplex<T>, MessageQueueCreationError> {
        self.transmission_mode = TransmissionMode::SendingAndReceiving;
        MessageQueueDuplex::create(self, mode)
    }

    /// Opens a new [`MessageQueueSender`].
    pub fn open_sender<T>(mut self) -> Result<MessageQueueSender<T>, MessageQueueOpenError> {
        self.transmission_mode = TransmissionMode::Sending;
        MessageQueueSender::open(self)
    }

    /// Opens a new [`MessageQueueReceiver`].
    pub fn open_receiver<T>(mut self) -> Result<MessageQueueReceiver<T>, MessageQueueOpenError> {
        self.transmission_mode = TransmissionMode::Receiving;
        MessageQueueReceiver::open(self)
    }

    /// Opens a new [`MessageQueueDuplex`].
    pub fn open_duplex<T>(mut self) -> Result<MessageQueueDuplex<T>, MessageQueueOpenError> {
        self.transmission_mode = TransmissionMode::SendingAndReceiving;
        MessageQueueDuplex::open(self)
    }
}

/// Removes an existing message queue in the system. Be aware, when the message queue is currently
/// owned by another instance it will lead to an error message when the object goes out of scope.
/// It should be only used to remove deserted message queues.
///
/// # Safety
///
///  * Only use when the [`MessageQueueSender`] or [`MessageQueueReceiver`] or [`MessageQueueDuplex`]
///    is not owned by any object in any process on the system.
///
pub unsafe fn remove_message_queue(name: &FileName) -> Result<(), MessageQueueRemoveError> {
    internal::MessageQueue::mq_unlink(name)
}

/// Returns true if the message queue exists, otherwise false
pub fn does_message_queue_exist<T>(name: &FileName) -> Result<bool, MessageQueueOpenError> {
    internal::MessageQueue::does_exist::<T>(name)
}

mod internal {
    use super::*;

    #[derive(Debug)]
    pub struct MessageQueue {
        pub(super) mqdes: posix::mqd_t,
        pub(super) name: FileName,
        pub(super) capacity: usize,
        pub(super) has_ownership: bool,
        is_non_blocking: bool,
        pub(super) clock_type: ClockType,
    }

    impl Drop for MessageQueue {
        fn drop(&mut self) {
            if self.has_ownership && Self::mq_unlink(&self.name).is_err() {
                error!(from self, "The was already removed by a different instance. This should not happen!");
            }

            if unsafe { posix::mq_close(self.mqdes) } == -1 {
                fatal_panic!(from self, "This should never happen! Unable to close message queue descriptor.");
            }
        }
    }

    impl MessageQueue {
        fn add_slash(name: &FileName) -> FilePath {
            let mut path = FilePath::new(name.as_string().as_bytes()).unwrap();
            path.insert(0, b'/').expect("");
            path
        }

        fn mq_create(
            config: &MessageQueueBuilder,
        ) -> Result<posix::mqd_t, MessageQueueCreationError> {
            let mut attributes = Self::create_attributes(config);

            let mqdes = unsafe {
                posix::mq_open4(
                    Self::add_slash(&config.name).as_string().as_c_str(),
                    posix::O_CREAT | posix::O_EXCL | config.transmission_mode.clone() as i32,
                    config.permissions.as_mode(),
                    &mut attributes,
                )
            };

            if mqdes != posix::MQ_INVALID {
                return Ok(mqdes);
            }

            let msg = format!(
                "Unable to create message queue for {:?}",
                config.transmission_mode
            );
            handle_errno!(MessageQueueCreationError, from config,
                Errno::EEXIST => (AlreadyExist, "{} since the queue already exists", msg),
                Errno::EACCES => (PermissionDenied, "{} due to insufficient permissions.", msg ),
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                Errno::EINVAL => (InvalidMessageSizeOrNumberOfMessages, "{} since the provided message size or max number of messages value was invalid.", msg),
                Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process limit of file handles was reached.", msg),
                Errno::ENFILE => (SystemMessageQueueLimitReached, "{} since the system limit of message queue was reached.", msg),
                Errno::ENOSPC => (OutOfResources, "{} since there are no resources left to create a new message queue.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        fn mq_open(
            config: &mut MessageQueueBuilder,
        ) -> Result<posix::mqd_t, MessageQueueOpenError> {
            let attributes = Self::create_attributes(config);

            let mqdes = unsafe {
                posix::mq_open2(
                    Self::add_slash(&config.name).as_string().as_c_str(),
                    config.transmission_mode.clone() as i32,
                )
            };

            let msg = format!(
                "Unable to open message queue for {:?}",
                config.transmission_mode
            );
            if mqdes != posix::MQ_INVALID {
                let existing_attr = Self::mq_getattr(mqdes);

                if existing_attr.mq_maxmsg < attributes.mq_maxmsg {
                    fail!(from config, with MessageQueueOpenError::CapacitySmallerThanRequired,
                    "{} since the max amount of messages {} is smaller than the required max amount of messages {}.",
                        msg, existing_attr.mq_maxmsg, attributes.mq_maxmsg);
                }

                if existing_attr.mq_msgsize != attributes.mq_msgsize {
                    fail!(from config, with MessageQueueOpenError::MessageSizeDoesNotFit,
                    "{} since the max message size of {} is not equal to the required max message size of {}.",
                        msg, existing_attr.mq_msgsize, attributes.mq_msgsize);
                }

                config.capacity = existing_attr.mq_maxmsg as usize;

                return Ok(mqdes);
            }

            handle_errno!(MessageQueueOpenError, from config,
              Errno::ENOENT => (DoesNotExist, "{} since the queue does not exist.", msg),
              Errno::EACCES => (PermissionDenied, "{} due to insufficient permissions.", msg),
              Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
              Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process limit of file handles was reached.", msg),
              Errno::ENFILE => (SystemMessageQueueLimitReached, "{} since the system limit of message queue was reached.", msg),
              v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            )
        }

        pub(super) fn does_exist<T>(name: &FileName) -> Result<bool, MessageQueueOpenError> {
            match MessageQueueBuilder::new(name).open_sender::<T>() {
                Ok(_) => Ok(true),
                Err(MessageQueueOpenError::DoesNotExist) => Ok(false),
                Err(v) => Err(v),
            }
        }

        pub(super) fn mq_getattr(mqdes: posix::mqd_t) -> posix::mq_attr {
            let mut attributes = posix::mq_attr::new();

            if unsafe { posix::mq_getattr(mqdes, &mut attributes) } != -1 {
                return attributes;
            }

            fatal_panic!(from "MessageQueue::mq_getattr",
            "Unable to acquire message queue attributes due to an corrupted message queue descriptor ({:?}).", mqdes);
        }

        pub(super) fn mq_set_nonblock(&mut self, is_non_blocking: bool) {
            if self.is_non_blocking == is_non_blocking {
                return;
            }

            let mut attributes = posix::mq_attr::new();
            attributes.mq_flags = if is_non_blocking {
                posix::O_NONBLOCK as posix::long
            } else {
                0
            };

            if unsafe {
                posix::mq_setattr(
                    self.mqdes,
                    &attributes,
                    std::ptr::null_mut::<posix::mq_attr>(),
                )
            } == -1
            {
                fatal_panic!(from "MessageQueue::mq_getattr",
                "Unable to set blocking mode due to an corrupted message queue descriptor ({:?}).", self.mqdes);
            }

            self.is_non_blocking = is_non_blocking;
        }

        pub(super) fn mq_unlink(name: &FileName) -> Result<(), MessageQueueRemoveError> {
            if unsafe { posix::mq_unlink(Self::add_slash(name).as_string().as_c_str()) } != -1 {
                return Ok(());
            }

            let msg = format!("Unable to remove message queue \"{}\"", name);
            handle_errno!(MessageQueueRemoveError, from "MessageQueue::remove",
                Errno::ENOENT => (DoesNotExist, "{} since the queue does not exist.", msg),
                Errno::EACCES => (PermissionDenied, "{} due to insufficient permissions.", msg),
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            )
        }

        fn create_attributes(config: &MessageQueueBuilder) -> posix::mq_attr {
            let mut attr = posix::mq_attr::new();
            attr.mq_maxmsg = config.capacity as _;
            attr.mq_msgsize = config.max_message_size as _;

            attr
        }

        pub(super) fn create(
            mut config: MessageQueueBuilder,
            mode: CreationMode,
        ) -> Result<Self, MessageQueueCreationError> {
            let msg = "Unable to create message queue";
            let (mqdes, has_ownership) = match mode {
                CreationMode::CreateExclusive => (Self::mq_create(&config)?, true),
                CreationMode::PurgeAndCreate => {
                    match Self::mq_unlink(&config.name) {
                        Ok(_) | Err(MessageQueueRemoveError::DoesNotExist) => (),
                        Err(v) => {
                            fail!(from config, with MessageQueueCreationError::MessageQueueRemoveError(v),
                              "{} since the old message queue could not be removed.", msg);
                        }
                    }
                    (Self::mq_create(&config)?, true)
                }
                CreationMode::OpenOrCreate => match Self::mq_open(&mut config) {
                    Ok(v) => (v, false),
                    Err(MessageQueueOpenError::DoesNotExist) => (Self::mq_create(&config)?, true),
                    Err(v) => return Err(MessageQueueCreationError::MessageQueueOpenError(v)),
                },
            };

            Ok(Self {
                mqdes,
                has_ownership,
                name: config.name,
                clock_type: config.clock_type,
                capacity: config.capacity,
                is_non_blocking: false,
            })
        }

        pub(super) fn open(mut config: MessageQueueBuilder) -> Result<Self, MessageQueueOpenError> {
            Ok(Self {
                mqdes: Self::mq_open(&mut config)?,
                has_ownership: false,
                name: config.name,
                clock_type: config.clock_type,
                capacity: config.capacity,
                is_non_blocking: false,
            })
        }
    }

    pub trait MessageQueueInterface {
        fn get(&self) -> &MessageQueue;
        fn get_mut(&mut self) -> &mut MessageQueue;
    }
}

/// Defines all operations common for [`MessageQueueSender`], [`MessageQueueReceiver`] and
/// [`MessageQueueDuplex`].
pub trait MessageQueueInterface: internal::MessageQueueInterface {
    /// Returns the name of the message queue
    fn name(&self) -> &FileName {
        &self.get().name
    }

    /// Returns the capacity of the message queue
    fn capacity(&self) -> usize {
        self.get().capacity
    }

    /// Returns the number of elements stored inside the queue
    fn len(&self) -> usize {
        internal::MessageQueue::mq_getattr(self.get().mqdes).mq_curmsgs as usize
    }

    fn is_empty(&self) -> bool {
        internal::MessageQueue::mq_getattr(self.get().mqdes).mq_curmsgs == 0
    }

    /// Returns the [`ClockType`] which is being used to wait in
    /// [`MessageQueueSenderInterface::timed_send()`],
    /// [`MessageQueueSenderInterface::timed_send_with_prio()`] or
    /// [`MessageQueueReceiverInterface::timed_receive()`].
    fn clock_type(&self) -> ClockType {
        self.get().clock_type
    }
}

/// Defines all send operations.
pub trait MessageQueueSenderInterface<T>: internal::MessageQueueInterface + Debug {
    #[doc(hidden)]
    fn __internal_send_with_prio(
        &mut self,
        value: &T,
        prio: u32,
        is_non_blocking: bool,
    ) -> Result<bool, MessageQueueSendError> {
        self.get_mut().mq_set_nonblock(is_non_blocking);

        if unsafe {
            posix::mq_send(
                self.get().mqdes,
                (value as *const T) as *const posix::char,
                std::mem::size_of::<T>(),
                prio,
            )
        } == 0
        {
            return Ok(true);
        }

        if Errno::get() == Errno::EAGAIN {
            return Ok(false);
        }

        let msg = "Unable to send message";
        handle_errno!(MessageQueueSendError, from self,
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EINVAL => (PriorityIsInvalid, "{} since the provided priority of {} is invalid.", msg, prio),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        )
    }

    /// Tries to send a message to the queue. If the queue is full it will not block and return
    /// false.
    fn try_send(&mut self, value: &T) -> Result<bool, MessageQueueSendError> {
        self.try_send_with_prio(value, 0)
    }

    /// Tries to send a message with a priority to the queue. If the queue is full it will not
    /// block and return false.
    fn try_send_with_prio(&mut self, value: &T, prio: u32) -> Result<bool, MessageQueueSendError> {
        self.__internal_send_with_prio(value, prio, true)
    }

    /// Sends a message with a time out. If the queue is full it will wait until the timeout is
    /// reached. If the message was sent it returns true, otherwise false.
    fn timed_send(
        &mut self,
        value: &T,
        timeout: Duration,
    ) -> Result<bool, MessageQueueTimedSendError> {
        self.timed_send_with_prio(value, timeout, 0)
    }

    /// Sends a message with a priority and a time out. If the queue is full it will wait until
    /// the timeout is reached. If the message was sent it returns true, otherwise false.
    fn timed_send_with_prio(
        &mut self,
        value: &T,
        timeout: Duration,
        prio: u32,
    ) -> Result<bool, MessageQueueTimedSendError> {
        let msg = "Failed to send message with timeout";
        match self.get().clock_type {
            ClockType::Realtime => {
                self.get_mut().mq_set_nonblock(false);
                if unsafe {
                    posix::mq_timedsend(
                        self.get().mqdes,
                        (value as *const T) as *const posix::char,
                        std::mem::size_of::<T>(),
                        prio,
                        &timeout.as_timespec(),
                    )
                } == 0
                {
                    return Ok(true);
                }

                if Errno::get() == Errno::EAGAIN || Errno::get() == Errno::ETIMEDOUT {
                    return Ok(false);
                }

                handle_errno!(MessageQueueTimedSendError, from self,
                    Errno::EINTR => (MessageQueueSendError(MessageQueueSendError::Interrupt), "{} since an interrupt signal was received.", msg),
                    Errno::EINVAL => (MessageQueueSendError(MessageQueueSendError::PriorityIsInvalid), "{} since the provided priority of {} is invalid.", msg, prio),
                    v => (MessageQueueSendError(MessageQueueSendError::UnknownError(v as i32)), "{} since an unknown error occurred ({}).", msg, v)
                )
            }
            ClockType::Monotonic => {
                let time = fail!(from self, when Time::now_with_clock(self.get().clock_type),
                    "{} due to a failure while acquiring current system time.", msg);
                let mut adaptive_wait = fail!(from self,
                    when AdaptiveWaitBuilder::new().clock_type(ClockType::Monotonic).create(),
                    "{} since the adaptive wait could not be created.", msg);

                loop {
                    match self.try_send_with_prio(value, prio) {
                        Ok(true) => return Ok(true),
                        Ok(false) => match fail!(from self, when time.elapsed(),
                            "{} due to a failure while acquiring elapsed system time.", msg)
                            < timeout
                        {
                            true => {
                                fail!(from self, when  adaptive_wait.wait(), "{} since AdaptiveWait failed.", msg);
                            }
                            false => return Ok(false),
                        },
                        Err(v) => {
                            fail!(from self, with MessageQueueTimedSendError::MessageQueueSendError(v),
                                "{} since the timed lock failed for duration {:?}.", msg, timeout);
                        }
                    }
                }
            }
        };
    }

    /// Sends a message. If the queue is full it blocks until the queue is empty again.
    fn blocking_send(&mut self, value: &T) -> Result<(), MessageQueueSendError> {
        self.blocking_send_with_prio(value, 0)
    }

    /// Sends a message with a priority. If the queue is full it blocks until the queue is
    /// empty again.
    fn blocking_send_with_prio(
        &mut self,
        value: &T,
        prio: u32,
    ) -> Result<(), MessageQueueSendError> {
        self.__internal_send_with_prio(value, prio, false)?;
        Ok(())
    }
}

/// Defines all receive operations.
pub trait MessageQueueReceiverInterface<T>: internal::MessageQueueInterface + Debug {
    #[doc(hidden)]
    fn __internal_receive(
        &mut self,
        is_non_blocking: bool,
    ) -> Result<Option<Message<T>>, MessageQueueReceiveError> {
        self.get_mut().mq_set_nonblock(is_non_blocking);

        let mut data = MaybeUninit::<T>::uninit();
        let mut priority = 0;

        let received_bytes = unsafe {
            posix::mq_receive(
                self.get().mqdes,
                data.as_mut_ptr() as *mut posix::char,
                std::mem::size_of::<T>(),
                &mut priority,
            )
        };

        let msg = "Unable to receive message";
        if received_bytes != -1 && received_bytes as usize != std::mem::size_of::<T>() {
            fail!(from self, with MessageQueueReceiveError::CorruptedMessageReceived,
                "{} since it was corrupted. Expected to receive {} bytes but received {} bytes.",
                msg, std::mem::size_of::<T>(), received_bytes);
        }

        if received_bytes != -1 {
            return Ok(Some(Message::<T> {
                value: unsafe { data.assume_init() },
                priority,
            }));
        }

        if Errno::get() == Errno::EAGAIN {
            return Ok(None);
        }

        handle_errno!(MessageQueueReceiveError, from self,
            Errno::EMSGSIZE => (MessageSizeInconsistencies, "{} since the required message size of the queue is larger than expected.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EBADMSG => (CorruptedMessageReceived, "{} since the implementation has detected a data corruption.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        )
    }

    /// Tries to receive a message. If the queue is empty it will not block and return [`None`].
    fn try_receive(&mut self) -> Result<Option<Message<T>>, MessageQueueReceiveError> {
        self.__internal_receive(true)
    }

    /// Receives a message with a timeout. If the queue is empty it will wait for at least the
    /// given timeout. If no message was received it returns [`None`] otherwise [`Message`].
    fn timed_receive(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<Message<T>>, MessageQueueTimedReceiveError> {
        let mut data = MaybeUninit::<T>::uninit();
        let mut priority = 0;

        let msg = "Unable to receive message with timeout";
        match self.get().clock_type {
            ClockType::Realtime => {
                self.get_mut().mq_set_nonblock(false);
                let received_bytes = unsafe {
                    posix::mq_timedreceive(
                        self.get().mqdes,
                        data.as_mut_ptr() as *mut posix::char,
                        std::mem::size_of::<T>(),
                        &mut priority,
                        &timeout.as_timespec(),
                    )
                };

                if received_bytes != -1 && received_bytes as usize != std::mem::size_of::<T>() {
                    fail!(from self, with MessageQueueTimedReceiveError::MessageQueueReceiveError(MessageQueueReceiveError::CorruptedMessageReceived),
                "{} since it was corrupted. Expected to receive {} bytes but received {} bytes.",
                msg, std::mem::size_of::<T>(), received_bytes);
                }

                if received_bytes != -1 {
                    return Ok(Some(Message::<T> {
                        value: unsafe { data.assume_init() },
                        priority,
                    }));
                }

                if Errno::get() == Errno::EAGAIN || Errno::get() == Errno::ETIMEDOUT {
                    return Ok(None);
                }

                handle_errno!(MessageQueueTimedReceiveError, from self,
                    Errno::EMSGSIZE => (MessageQueueReceiveError(MessageQueueReceiveError::MessageSizeInconsistencies), "{} since the required message size of the queue is larger than expected.", msg),
                    Errno::EINTR => (MessageQueueReceiveError(MessageQueueReceiveError::Interrupt), "{} since an interrupt signal was received.", msg),
                    Errno::EBADMSG => (MessageQueueReceiveError(MessageQueueReceiveError::CorruptedMessageReceived), "{} since the implementation has detected a data corruption.", msg),
                    v => (MessageQueueReceiveError(MessageQueueReceiveError::UnknownError(v as i32)), "{} since an unknown error occurred ({}).", msg, v)
                )
            }
            ClockType::Monotonic => {
                let time = fail!(from self, when Time::now_with_clock(self.get().clock_type),
                    "{} due to a failure while acquiring current system time.", msg);
                let mut adaptive_wait = fail!(from self,
                    when AdaptiveWaitBuilder::new().clock_type(ClockType::Monotonic).create(),
                    "{} since the adaptive wait could not be created.", msg);

                loop {
                    match self.try_receive() {
                        Ok(Some(v)) => return Ok(Some(v)),
                        Ok(None) => match fail!(from self, when time.elapsed(),
                            "{} due to a failure while acquiring elapsed system time.", msg)
                            < timeout
                        {
                            true => {
                                fail!(from self, when  adaptive_wait.wait(), "{} since AdaptiveWait failed.", msg);
                            }
                            false => return Ok(None),
                        },
                        Err(v) => {
                            fail!(from self, with MessageQueueTimedReceiveError::MessageQueueReceiveError(v),
                                "{} since the timed lock failed for duration {:?}.", msg, timeout);
                        }
                    }
                }
            }
        };
    }

    /// Blocks until a message was received.
    fn blocking_receive(&mut self) -> Result<Message<T>, MessageQueueReceiveError> {
        let result = self.__internal_receive(false)?;
        if result.is_none() {
            fatal_panic!(from self, "This should never happen! Received no message and no error in blocking mode");
        }

        Ok(result.unwrap())
    }
}

/// Can send messages to a message queue. Can be created or opened by the
/// [`MessageQueueBuilder`].
///
/// # Example
///
/// ```ignore
/// use iceoryx2_bb_posix::message_queue::*;
///
/// let mq_name = FileName::new(b"myMqName_3").unwrap();
/// let mut sender = MessageQueueBuilder::new(&mq_name)
///                     .create_sender::<u64>(CreationMode::PurgeAndCreate)
///                     .expect("Failed to create message queue");
///
/// sender.try_send(&1234);
/// ```
#[derive(Debug)]
pub struct MessageQueueSender<T> {
    queue: MessageQueueDuplex<T>,
}

impl<T> MessageQueueSender<T> {
    fn create(
        config: MessageQueueBuilder,
        mode: CreationMode,
    ) -> Result<Self, MessageQueueCreationError> {
        Ok(Self {
            queue: MessageQueueDuplex::create(config, mode)?,
        })
    }

    fn open(config: MessageQueueBuilder) -> Result<Self, MessageQueueOpenError> {
        Ok(Self {
            queue: MessageQueueDuplex::open(config)?,
        })
    }
}

impl<T> internal::MessageQueueInterface for MessageQueueSender<T> {
    fn get(&self) -> &internal::MessageQueue {
        &self.queue.queue
    }
    fn get_mut(&mut self) -> &mut internal::MessageQueue {
        &mut self.queue.queue
    }
}
impl<T> MessageQueueInterface for MessageQueueSender<T> {}
impl<T: Copy + Debug> MessageQueueSenderInterface<T> for MessageQueueSender<T> {}

/// Can receive messages from a message queue. Can be created or opened by the
/// [`MessageQueueBuilder`].
///
/// # Example
///
/// ```ignore
/// use iceoryx2_bb_posix::message_queue::*;
///
/// let mq_name = FileName::new(b"myMqName_4").unwrap();
/// let mut receiver = MessageQueueBuilder::new(&mq_name)
///                     .create_receiver::<u64>(CreationMode::PurgeAndCreate)
///                     .expect("Failed to create message queue");
///
/// let received_data = receiver.try_receive().expect("failed to receive");
/// assert!(received_data.is_none());
/// ```

#[derive(Debug)]
pub struct MessageQueueReceiver<T> {
    queue: MessageQueueDuplex<T>,
}

impl<T> MessageQueueReceiver<T> {
    fn create(
        config: MessageQueueBuilder,
        mode: CreationMode,
    ) -> Result<Self, MessageQueueCreationError> {
        Ok(Self {
            queue: MessageQueueDuplex::create(config, mode)?,
        })
    }

    fn open(config: MessageQueueBuilder) -> Result<Self, MessageQueueOpenError> {
        Ok(Self {
            queue: MessageQueueDuplex::open(config)?,
        })
    }
}

impl<T> internal::MessageQueueInterface for MessageQueueReceiver<T> {
    fn get(&self) -> &internal::MessageQueue {
        &self.queue.queue
    }
    fn get_mut(&mut self) -> &mut internal::MessageQueue {
        &mut self.queue.queue
    }
}
impl<T> MessageQueueInterface for MessageQueueReceiver<T> {}
impl<T: Copy + Debug> MessageQueueReceiverInterface<T> for MessageQueueReceiver<T> {}

/// Can send and receive messages to or from a message queue. Can be created or opened by the
/// [`MessageQueueBuilder`].
///
/// # Example
///
/// ```ignore
/// use iceoryx2_bb_posix::message_queue::*;
///
/// let mq_name = FileName::new(b"myMqName_5").unwrap();
/// let mut duplex = MessageQueueBuilder::new(&mq_name)
///                     .create_duplex::<u64>(CreationMode::PurgeAndCreate)
///                     .expect("Failed to create message queue");
///
/// duplex.try_send_with_prio(&7881238, 5);
///
/// let received_data = duplex.try_receive().expect("failed to receive");
/// assert!(received_data.is_some());
/// assert_eq!(received_data.as_ref().unwrap().value, 7881238);
/// assert_eq!(received_data.as_ref().unwrap().priority, 5);
/// ```
#[derive(Debug)]
pub struct MessageQueueDuplex<T> {
    queue: internal::MessageQueue,
    _phantom_data: PhantomData<T>,
}

impl<T> MessageQueueDuplex<T> {
    fn create(
        mut config: MessageQueueBuilder,
        mode: CreationMode,
    ) -> Result<Self, MessageQueueCreationError> {
        config.max_message_size = std::mem::size_of::<T>();

        Ok(Self {
            queue: internal::MessageQueue::create(config, mode)?,
            _phantom_data: PhantomData,
        })
    }

    fn open(mut config: MessageQueueBuilder) -> Result<Self, MessageQueueOpenError> {
        config.max_message_size = std::mem::size_of::<T>();

        Ok(Self {
            queue: internal::MessageQueue::open(config)?,
            _phantom_data: PhantomData,
        })
    }
}

impl<T> internal::MessageQueueInterface for MessageQueueDuplex<T> {
    fn get(&self) -> &internal::MessageQueue {
        &self.queue
    }
    fn get_mut(&mut self) -> &mut internal::MessageQueue {
        &mut self.queue
    }
}

impl<T> MessageQueueInterface for MessageQueueDuplex<T> {}
impl<T: Copy + Debug> MessageQueueSenderInterface<T> for MessageQueueDuplex<T> {}
impl<T: Copy + Debug> MessageQueueReceiverInterface<T> for MessageQueueDuplex<T> {}
