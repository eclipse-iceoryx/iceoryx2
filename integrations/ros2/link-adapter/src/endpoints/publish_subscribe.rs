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

use iceoryx2_link_adapter::{LoanError, LoanableSample, ReceiveOutcome, TakeError, WritableSample};
use iceoryx2_log::{fail, origin, warn};

use crate::rcl::subscription::TakeError as RclTakeError;
use crate::rcl::{RclPublisher, RclSubscription};
use crate::ros_header::RosHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishSubscribeEndpointsError {
    Publish,
    Take,
    /// The rmw rejected the loaned sample as the destination buffer.
    SampleNotFilled,
    /// The rmw took a message without requesting a buffer.
    LoanNotUsed,
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

    /// ROS 2 carries no header, a header form is dropped with a warning.
    fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Failure> {
        let origin = origin!("PublishSubscribeEndpoints::publish");

        if !header.is_empty() {
            warn!(
                from origin,
                "Received a header of {} bytes to publish. Publishing custom headers is unsupported by ROS 2 and should be handled in the translator. Dropping header.",
                header.len()
            );
        }

        fail!(
            from origin,
            when self.publisher.publish(payload),
            with PublishSubscribeEndpointsError::Publish,
            "Failed to publish a message"
        );

        Ok(())
    }

    /// The rmw writes the serialized message directly into the payload region
    /// provided by the `loanable`. The `loanable` automatically takes care
    /// of translation if required.
    ///
    /// The message info is copied into the header region as a [`RosHeader`].
    fn take<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::Sample>, TakeError<Self::Failure>> {
        let origin = origin!("PublishSubscribeEndpoints::take");

        // Take the bytes in wire form directly into the region provided by
        // the `loanable`.
        let mut sample: Option<L::Sample> = None;
        let mut refusal: Option<LoanError> = None;
        let taken = self
            .subscription
            .take_into(|size| match loanable.loan(size) {
                Ok(mut loaned) => {
                    // Store the sample and provide the pointer to the payload
                    // region for the RMW to write to.
                    let pointer = loaned.payload().as_mut_ptr();
                    sample = Some(loaned);
                    Some(pointer)
                }
                Err(error) => {
                    refusal = Some(error);
                    None
                }
            });

        // Retrieve the pointer to the written message info.
        let info = match taken {
            Ok(Some((_, info))) => info,
            Ok(None) => return Ok(ReceiveOutcome::Empty),
            Err(RclTakeError::LoanDeclined) => match refusal {
                Some(refusal) => {
                    fail!(
                        from origin,
                        with TakeError::from(refusal),
                        "The sample refused a loan for the message"
                    );
                }
                None => {
                    fail!(
                        from origin,
                        with TakeError::Endpoints(PublishSubscribeEndpointsError::SampleNotFilled),
                        "The rmw rejected the loaned sample as its buffer"
                    );
                }
            },
            Err(RclTakeError::Take) => {
                fail!(
                    from origin,
                    with TakeError::Endpoints(PublishSubscribeEndpointsError::Take),
                    "The rmw failed to take a message"
                );
            }
        };
        let Some(mut sample) = sample else {
            fail!(
                from origin,
                with TakeError::Endpoints(PublishSubscribeEndpointsError::LoanNotUsed),
                "The rmw took a message without requesting a buffer"
            );
        };

        // Own messages are skipped here, not via ignore_local_publications.
        // rmw_fastrtps seems to have a bug (?) that drops the message that
        // follows a skipped local one when taking serialized messages.
        // TODO: Earlier opt-out, this wastes a loan.
        if info.gid == *self.publisher.gid() {
            return Ok(ReceiveOutcome::Skipped);
        }

        // Fill message info header.
        let header = RosHeader::from(info);
        let region = fail!(
            from origin,
            when sample.header(core::mem::size_of::<RosHeader>()),
            to TakeError<Self::Failure>,
            "Rejected the header of a message"
        );
        region.copy_from_slice(bytes_of(&header));

        Ok(ReceiveOutcome::Sample(sample))
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
