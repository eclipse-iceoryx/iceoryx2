// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

pub struct Listener<Service>
where
    Service: service::Service,
{
    listener: iceoryx2::port::listener::Listener<Service>,
    io: BridgedFd<RawFdBridge<AsyncSelector>>,
}

impl<Service> Listener<Service>
where
    Service: service::Service,
    <Service::Event as iceoryx2_cal::event::Event>::Listener: FileDescriptorBased,
{
    pub(crate) fn from(listener: iceoryx2::port::listener::Listener<Service>) -> Result<Self, CommonErrors> {
        // Safety:
        // - This FD is owned by iceoryx2 listener and we don't close it on drop of RawFdBridge
        // - The FD is kept along with listener so lifetime is take care of
        // - Each Listener has its own FD so no sharing is done in iceoryx2 layer
        let fd = unsafe { listener.file_descriptor().native_handle() };

        Ok(Self {
            listener,
            io: BridgedFd::new_with_interest(RawFdBridge::from(fd)?, IoEventInterest::READABLE)?,
        })
    }

    /// Returns the [`UniqueListenerId`] of the [`Listener`]
    pub fn id(&self) -> UniqueListenerId {
        self.listener.id()
    }

    /// Returns the deadline of the corresponding [`Service`](crate::service::Service).
    pub fn deadline(&self) -> Option<Duration> {
        self.listener.deadline()
    }

    /// Async wait for a new [`EventId`]. On error it returns [`ListenerWaitError`] is returned which describes
    /// the error in detail.
    pub async fn wait_one(&self) -> Result<EventId, ListenerWaitError> {
        self.io
            .async_call(IoEventInterest::READABLE, |raw_fd| {
                raw_fd.io_call(|fd| {
                    info!("Checking for Iceoryx event on fd: {}", fd);
                    self.wait_one_internal()
                })
            })
            .await
            .map_err(|_| ListenerWaitError::InternalFailure)
            .and_then(|r| match r {
                Ok(event) => Ok(event),
                Err(e) => Err(e),
            })
    }

    fn wait_one_internal(&self) -> IoResult<Result<EventId, ListenerWaitError>> {
        loop {
            match self.listener.try_wait_one() {
                Ok(event) if event.is_some() => return Ok(Ok(event.unwrap())),
                Ok(_) => {
                    // This is None, so there was and error, probably EAGAIN or EWOULDBLOCK
                    if std::io::Error::last_os_error().kind() == std::io::ErrorKind::WouldBlock {
                        error!("Iceoryx listener would block, should do re-register!... {}", unsafe {
                            self.listener.file_descriptor().native_handle()
                        });
                        return Err(std::io::ErrorKind::WouldBlock.into());
                    } else {
                        panic!("Something went wrong!");
                    }
                }
                Err(ListenerWaitError::InterruptSignal) => {
                    continue;
                }
                Err(e) => {
                    error!("Error waiting for Iceoryx event: {}", e);
                    return Ok(Err(e));
                }
            }
        }
    }
}

impl<T> Drop for Listener<T>
where
    T: service::Service,
{
    fn drop(&mut self) {
        // Leave the underlying fd open, as we don't own it and let iceoryx2 handle it
        self.io.close_on_drop(false);
    }
}

