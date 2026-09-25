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

use iceoryx2_link_adapter::{SampleBytesRef, SampleLengths, TakeDestination, TakeOutcome};
use iceoryx2_log::{fail, origin};

use crate::rcl::subscription::TakeError as RclTakeError;
use crate::rcl::{RclPublisher, RclSubscription};
use crate::ros_header::RosHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishSubscribeEndpointsError {
    Publish,
    Take,
    /// The rmw took the message without writing it into the memory it
    /// was given.
    NotWritten,
}

impl core::fmt::Display for PublishSubscribeEndpointsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishSubscribeEndpointsError::{self:?}")
    }
}

impl core::error::Error for PublishSubscribeEndpointsError {}

/// The gateway's publisher and subscription on a ROS 2 topic.
#[derive(Debug)]
pub struct PublishSubscribeEndpoints {
    publisher: RclPublisher,
    subscription: RclSubscription,
}

impl PublishSubscribeEndpoints {
    pub(crate) fn new(publisher: RclPublisher, subscription: RclSubscription) -> Self {
        Self {
            publisher,
            subscription,
        }
    }
}

impl iceoryx2_link_adapter::PublishSubscribeEndpoints for PublishSubscribeEndpoints {
    type Failure = PublishSubscribeEndpointsError;

    /// Publishes the payload bytes of the provided sample. The header is
    /// ignored as ROS 2 cannot carry it.
    fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
        let origin = origin!("PublishSubscribeEndpoints::publish");

        fail!(
            from origin,
            when self.publisher.publish(sample.payload),
            with PublishSubscribeEndpointsError::Publish,
            "Failed to publish a message"
        );

        Ok(())
    }

    /// Writes the received bytes into the provided destination.
    ///
    /// The payload location is populated with the received bytes in wire form.
    /// The header location is populated with the corresponding message info
    /// as a [`RosHeader`].
    fn take<'a>(
        &mut self,
        destination: impl TakeDestination<'a>,
    ) -> Result<TakeOutcome, Self::Failure> {
        let origin = origin!("PublishSubscribeEndpoints::take");

        // Provide the payload location to the rmw as the buffer to write the
        // serialized message.
        let mut header_location = None;
        let mut declined = false;
        let taken = self.subscription.take_into(|size| {
            let lengths = SampleLengths {
                header: core::mem::size_of::<RosHeader>(),
                payload: size,
            };
            match destination.for_lengths(lengths) {
                Some(locations) => {
                    header_location = Some(locations.header);
                    Some(locations.payload.as_mut_ptr())
                }
                None => {
                    declined = true;
                    None
                }
            }
        });

        let info = match taken {
            Ok(Some((_, info))) => info,
            Ok(None) => return Ok(TakeOutcome::Empty),
            Err(RclTakeError::LoanDeclined) if declined => return Ok(TakeOutcome::Declined),
            Err(RclTakeError::LoanDeclined) => {
                fail!(
                    from origin,
                    with PublishSubscribeEndpointsError::NotWritten,
                    "The rmw rejected the memory it was given for the message"
                );
            }
            Err(RclTakeError::Take) => {
                fail!(
                    from origin,
                    with PublishSubscribeEndpointsError::Take,
                    "The rmw failed to take a message"
                );
            }
        };
        let Some(header_location) = header_location else {
            fail!(
                from origin,
                with PublishSubscribeEndpointsError::NotWritten,
                "The rmw took the message without asking for memory to write it into"
            );
        };

        // Own messages are skipped here, not via ignore_local_publications.
        // rmw_fastrtps seems to have a bug (?) that drops the message that
        // follows a skipped local one when taking serialized messages.
        // TODO: Earlier opt-out, this wastes the provided locations.
        if info.gid == *self.publisher.gid() {
            return Ok(TakeOutcome::Skipped);
        }

        // Copy the header into the destination.
        let header = RosHeader::from(info);
        header_location.copy_from_slice(bytes_of(&header));

        Ok(TakeOutcome::Taken)
    }
}

/// The bytes of a header.
fn bytes_of(header: &RosHeader) -> &[u8] {
    // SAFETY: `RosHeader` is `repr(C)` with no padding, sixteen bytes of gid,
    // an `i64` and a `u64`, so its bytes are all initialized.
    unsafe {
        core::slice::from_raw_parts(
            (header as *const RosHeader).cast::<u8>(),
            core::mem::size_of::<RosHeader>(),
        )
    }
}
