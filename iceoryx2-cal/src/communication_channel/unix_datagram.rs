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

//! [CommunicationChannel] based on [`UnixDatagramSender`] & [`UnixDatagramSender`]. Can send and
//! receive data without restrictions.

use core::{fmt::Debug, marker::PhantomData, mem::MaybeUninit};

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::{
    directory::*, file::*, system_configuration::SystemInfo, unix_datagram_socket::*,
};
use iceoryx2_bb_system_types::path::Path;

pub use crate::communication_channel::*;
use crate::static_storage::file::{
    NamedConceptConfiguration, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptRemoveError,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: Channel::<()>::default_suffix(),
            prefix: Channel::<()>::default_prefix(),
            path_hint: Channel::<()>::default_path_hint(),
        }
    }
}

impl NamedConceptConfiguration for Configuration {
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
        self.path_hint = value.clone();
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

#[derive(Debug)]
pub struct Channel<T: Copy> {
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> NamedConceptMgmt for Channel<T> {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        let msg =
            format!("Unable to check if communication_channel::unix_datagram \"{name}\" exists");

        let full_path = cfg.path_for(name);

        match File::does_exist(&full_path) {
            Ok(true) => Ok(true),
            Ok(false) => Ok(false),
            Err(v) => {
                fail!(from "communication_channel::unix_datagram::Channel::does_exist_cfg()",
                        with NamedConceptDoesExistError::UnderlyingResourcesCorrupted,
                    "{} due to an internal failure ({:?}), is the communication channel in a corrupted state?", msg, v);
            }
        }
    }

    fn list_cfg(
        config: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        let msg = "Unable to list all communication_channel::unix_datagram";
        let origin = "communication_channel::unix_datagram::Channel::list_cfg()";

        let directory = fail!(from origin, when Directory::new(&config.path_hint),
            map DirectoryOpenError::InsufficientPermissions => NamedConceptListError::InsufficientPermissions,
            unmatched NamedConceptListError::InternalError,
            "{} due to a failure while reading the directory (\"{}\").", msg, config.path_hint);

        let entries = fail!(from origin,
                            when directory.contents(),
                            map DirectoryReadError::InsufficientPermissions => NamedConceptListError::InsufficientPermissions,
                            unmatched NamedConceptListError::InternalError,
                            "{} due to a failure while reading the directory (\"{}\") contents.", msg, config.path_hint);

        let mut result = vec![];
        for entry in &entries {
            if let Some(entry_name) = config.extract_name_from_file(entry.name()) {
                result.push(entry_name);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        config: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        let msg = format!("Unable to release static storage \"{name}\"");
        let origin = "communication_channel::unix_datagram::Channel::remove_cfg()";
        let file_path = config.path_for(name);

        match File::remove(&file_path) {
            Ok(v) => Ok(v),
            Err(FileRemoveError::InsufficientPermissions)
            | Err(FileRemoveError::PartOfReadOnlyFileSystem) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                        "{} due to insufficient permissions.", msg);
            }
            Err(v) => {
                fail!(from origin, with NamedConceptRemoveError::InternalError,
                        "{} due to unknown failure ({:?}).", msg, v);
            }
        }
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        crate::named_concept::remove_path_hint(value)
    }
}

impl<T: Copy + Debug> CommunicationChannel<T> for Channel<T> {
    type Sender = Sender<T>;
    type Receiver = Receiver<T>;
    type Creator = Creator<T>;
    type Connector = Connector<T>;

    fn has_configurable_buffer_size() -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Creator<T: Copy + Debug> {
    channel_name: FileName,
    enable_safe_overflow: bool,
    buffer_size: usize,
    config: Configuration,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> NamedConceptBuilder<Channel<T>> for Creator<T> {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: channel_name.clone(),
            enable_safe_overflow: false,
            buffer_size: DEFAULT_RECEIVER_BUFFER_SIZE,
            config: Configuration::default(),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl<T: Copy + Debug> CommunicationChannelCreator<T, Channel<T>> for Creator<T> {
    fn enable_safe_overflow(mut self) -> Self {
        self.enable_safe_overflow = true;
        self
    }

    fn buffer_size(mut self, value: usize) -> Self {
        self.buffer_size = value;
        self
    }

    fn create_receiver(self) -> Result<Receiver<T>, CommunicationChannelCreateError> {
        let msg = "Unable to create receiver";
        if self.enable_safe_overflow {
            fail!(from self, with CommunicationChannelCreateError::SafeOverflowNotSupported,
                "{} since the channel does not support the safe overflow feature.", msg);
        }

        let full_name = self.config.path_for(&self.channel_name);
        let receiver = UnixDatagramReceiverBuilder::new(&full_name)
            .creation_mode(CreationMode::CreateExclusive)
            .create();

        let receiver = match receiver {
            Ok(mut r) => {
                fail!(from self, when r.set_receive_buffer_min_size(self.buffer_size * SystemInfo::PageSize.value()),
                        with CommunicationChannelCreateError::InternalFailure,
                        "{} due to a failure while setting the channels buffer size of {}", msg, self.buffer_size);
                r
            }
            Err(UnixDatagramReceiverCreationError::SocketFileAlreadyExists)
            | Err(UnixDatagramReceiverCreationError::AddressAlreadyInUse) => {
                fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                    "{} since a channel with that name already exists.", msg);
            }
            _ => {
                fail!(from self, with CommunicationChannelCreateError::InternalFailure,
                    "{} since the underlying socket could not be created.", msg);
            }
        };

        Ok(Receiver {
            name: self.channel_name,
            receiver,
            _phantom_data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Connector<T: Copy + Debug> {
    channel_name: FileName,
    config: Configuration,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> Connector<T> {
    fn verify_and_open_sender(
        self,
        sender: Result<UnixDatagramSender, UnixDatagramSenderCreationError>,
    ) -> Result<Sender<T>, CommunicationChannelOpenError> {
        let msg = "Unable to create sender";
        let sender = match sender {
            Ok(s) => s,
            Err(UnixDatagramSenderCreationError::AlreadyConnected) => {
                fail!(from self, with CommunicationChannelOpenError::AnotherInstanceIsAlreadyConnected,
                    "{} since another instance is already connected.", msg);
            }
            Err(UnixDatagramSenderCreationError::DoesNotExist) => {
                return Err(CommunicationChannelOpenError::DoesNotExist)
            }
            _ => {
                fail!(from self, with CommunicationChannelOpenError::InternalFailure,
                    "{} since a connection to the underlying socket could not be established.", msg);
            }
        };

        Ok(Sender {
            name: self.channel_name,
            sender,
            _phantom_data: PhantomData,
        })
    }
}

impl<T: Copy + Debug> NamedConceptBuilder<Channel<T>> for Connector<T> {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: channel_name.clone(),
            config: Configuration::default(),
            _phantom_data: PhantomData,
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl<T: Copy + Debug> CommunicationChannelConnector<T, Channel<T>> for Connector<T> {
    fn open_sender(self) -> Result<Sender<T>, CommunicationChannelOpenError> {
        let msg = "Unable to create sender";
        let origin = format!("{self:?}");

        let full_name = self.config.path_for(&self.channel_name);
        let sender = UnixDatagramSenderBuilder::new(&full_name).create();

        match self.verify_and_open_sender(sender) {
            Ok(v) => Ok(v),
            Err(CommunicationChannelOpenError::DoesNotExist) => {
                fail!(from origin, with CommunicationChannelOpenError::DoesNotExist,
                            "{} since there is no receiver to connect to.", msg);
            }
            Err(v) => {
                fail!(from origin, with v,
                    "{} due to an unknown failure ({:?}).", msg, v);
            }
        }
    }

    fn try_open_sender(self) -> Result<Sender<T>, CommunicationChannelOpenError> {
        // create the uds channel first to avoid races when open is called and the receiver
        // is created at the same time. The uds is called as second in create_receiver and
        // it cannot exist when not everything is already set up.
        let full_name = self.config.path_for(&self.channel_name);
        let sender = UnixDatagramSenderBuilder::new(&full_name).create();
        self.verify_and_open_sender(sender)
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    name: FileName,
    sender: UnixDatagramSender,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> CommunicationChannelParticipant for Sender<T> {
    fn does_enable_safe_overflow(&self) -> bool {
        false
    }
}

impl<T: Copy + Debug> NamedConcept for Sender<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Copy + Debug> CommunicationChannelSender<T> for Sender<T> {
    fn send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError> {
        let msg = "Unable to send data";
        match self.try_send(data) {
            Err(CommunicationChannelSendError::ReceiverCacheIsFull) => {
                fail!(from self, with CommunicationChannelSendError::ReceiverCacheIsFull,
                    "{} since the receiver cache is full.", msg);
            }
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }

    fn try_send(&self, data: &T) -> Result<Option<T>, CommunicationChannelSendError> {
        let msg = "Unable to try send data";
        let result = self.sender.try_send(unsafe {
            core::slice::from_raw_parts((data as *const T) as *const u8, core::mem::size_of::<T>())
        });

        match result {
            Ok(true) => (),
            Ok(false) => {
                return Err(CommunicationChannelSendError::ReceiverCacheIsFull);
            }
            Err(UnixDatagramSendError::MessageTooLarge) => {
                fail!(from self, with CommunicationChannelSendError::MessageTooLarge,
                    "{} since the size ({} bytes) of the type \"{}\" is too large.",
                    msg, core::mem::size_of::<T>(), core::any::type_name::<T>());
            }
            Err(UnixDatagramSendError::ConnectionReset)
            | Err(UnixDatagramSendError::NotConnected) => {
                fail!(from self, with CommunicationChannelSendError::ConnectionBroken,
                    "{} since the connection seems to be broken.", msg);
            }
            Err(_) => {
                fail!(from self, with CommunicationChannelSendError::InternalFailure,
                    "{} due to an internal failure.", msg);
            }
        };

        Ok(None)
    }
}

#[derive(Debug)]
pub struct Receiver<T: Debug> {
    name: FileName,
    receiver: UnixDatagramReceiver,
    _phantom_data: PhantomData<T>,
}

impl<T: Copy + Debug> CommunicationChannelParticipant for Receiver<T> {
    fn does_enable_safe_overflow(&self) -> bool {
        false
    }
}

impl<T: Copy + Debug> NamedConcept for Receiver<T> {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl<T: Copy + Debug> CommunicationChannelReceiver<T> for Receiver<T> {
    fn buffer_size(&self) -> usize {
        // divided by page size since ?page size? is required for every datagram and we would like
        // to know how many packages one can handle
        fatal_panic!(from self, when self.receiver.get_receive_buffer_size(), "Unable to acquire the cache size")
            / SystemInfo::PageSize.value()
    }

    fn receive(&self) -> Result<Option<T>, CommunicationChannelReceiveError> {
        let msg = "Unable to receive data";
        let mut data = MaybeUninit::<T>::uninit();
        match self.receiver.try_receive(unsafe {
            core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        }) {
            Ok(0) => Ok(None),
            Ok(received_bytes) => {
                if received_bytes != core::mem::size_of::<T>() as u64 {
                    fail!(from self, with CommunicationChannelReceiveError::MessageCorrupt,
                    "The received message is corrupted. Expected to receive {} bytes but got {} bytes.",
                    core::mem::size_of::<T>(), received_bytes );
                }
                Ok(Some(unsafe { data.assume_init() }))
            }
            Err(e) => match e {
                UnixDatagramReceiveError::ConnectionReset => {
                    fail!(from self, with CommunicationChannelReceiveError::ConnectionBroken,
                    "{} since the connection is broken.", msg);
                }
                _ => {
                    fail!(from self, with CommunicationChannelReceiveError::InternalFailure,
                    "{} due to some internal failure.", msg);
                }
            },
        }
    }
}
