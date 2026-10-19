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

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use iceoryx2_link_adapter::{
    ClientRequestId, RequestDestination, ResponseDestination, ResponseKind, SampleBytesRef,
    SampleDestination, SampleLengths, TakeOutcome,
};
use iceoryx2_log::{debug, fail, origin, warn};

use crate::rcl::{RclClient, RclService, RequestId, SequenceNumber};
use crate::ros_header::{RosHeader, bytes_of};
use crate::ros_message::{RosMessage, SerializeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestResponseEndpointsError {
    /// The bytes are not a message of the service's type.
    Deserialize,
    /// The message could not be serialized into CDR.
    Serialize,
    /// rcl refused to send the request or response.
    Send,
    /// rcl failed to take a pending request or response.
    Take,
    /// The rmw serialized the message without writing it into the memory
    /// it was given.
    NotWritten,
}

impl core::fmt::Display for RequestResponseEndpointsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RequestResponseEndpointsError::{self:?}")
    }
}

impl core::error::Error for RequestResponseEndpointsError {}

/// The gateway's client and server on a ROS 2 service.
///
/// A ROS 2 server sends exactly one response per request. A taken
/// response is followed by the end of its stream, and only the first
/// response to a taken request is sent.
#[derive(Debug)]
pub struct RequestResponseEndpoints {
    client: RclClient,
    server: RclService,
    /// The request struct of the service's type, reset after each use.
    request: RosMessage,
    /// The response struct of the service's type, reset after each use.
    response: RosMessage,
    /// The id of each sent request by the sequence number rcl gave it.
    sent: BTreeMap<SequenceNumber, ClientRequestId>,
    /// The ends to report for responses taken.
    pending_ends: VecDeque<ClientRequestId>,
    /// The taken requests that got their one response.
    answered: BTreeSet<RequestId>,
}

impl RequestResponseEndpoints {
    pub(crate) fn new(
        client: RclClient,
        server: RclService,
        request: RosMessage,
        response: RosMessage,
    ) -> Self {
        Self {
            client,
            server,
            request,
            response,
            sent: BTreeMap::new(),
            pending_ends: VecDeque::new(),
            answered: BTreeSet::new(),
        }
    }

    /// Whether the request `request_id` identifies was sent by the
    /// endpoints' own client.
    fn is_own(&self, request_id: &RequestId) -> bool {
        request_id.participant_prefix() == *self.server.participant_prefix()
    }
}

impl iceoryx2_link_adapter::RequestResponseEndpoints for RequestResponseEndpoints {
    type Failure = RequestResponseEndpointsError;
    type ServerRequestId = RequestId;

    /// Sends the payload bytes of the request.
    ///
    /// The header is ignored as ROS 2 cannot carry it.
    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure> {
        let origin = origin!("RequestResponseEndpoints::send_request");

        // TODO: Avoid superfluous deserialization when ROS message is already in shared memory.
        self.request.reset();
        fail!(
            from origin,
            when self.request.deserialize_from(request.payload),
            with RequestResponseEndpointsError::Deserialize,
            "Failed to deserialize the request {}", id
        );

        let sequence_number = fail!(
            from origin,
            when self.client.send_request(&self.request),
            with RequestResponseEndpointsError::Send,
            "Failed to send the request {}", id
        );
        self.sent.insert(sequence_number, id);

        Ok(())
    }

    /// Writes the received response into the destination of its request.
    ///
    /// The payload location is populated with the received bytes in wire form.
    /// The header location is populated with the corresponding service info
    /// as a [`RosHeader`]. The end of the stream follows the response on the
    /// next take.
    fn take_response<'a>(
        &mut self,
        destination: impl ResponseDestination<'a>,
    ) -> Result<TakeOutcome<(ClientRequestId, ResponseKind)>, Self::Failure> {
        let origin = origin!("RequestResponseEndpoints::take_response");

        if let Some(id) = self.pending_ends.pop_front() {
            return Ok(TakeOutcome::Taken((id, ResponseKind::End)));
        }

        self.response.reset();
        let info = match self.client.take_response(&mut self.response) {
            Ok(Some(info)) => info,
            Ok(None) => return Ok(TakeOutcome::Empty),
            Err(_) => {
                fail!(
                    from origin,
                    with RequestResponseEndpointsError::Take,
                    "The rmw failed to take a response"
                );
            }
        };

        let sequence_number = info.request_id.sequence_number;
        let Some(id) = self.sent.remove(&sequence_number) else {
            warn!(
                "A response to the unknown request {} was taken and dropped",
                sequence_number
            );
            return Ok(TakeOutcome::Skipped);
        };
        let Some(destination) = destination.of(id) else {
            return Ok(TakeOutcome::Skipped);
        };

        // A single response is taken and then the stream is ended.
        self.pending_ends.push_back(id);

        // TODO: Avoid superfluous serialization when ROS message can be written directly
        //       to shared memory.
        match serialize_into(&self.response, RosHeader::from(info), destination)? {
            TakeOutcome::Taken(()) => Ok(TakeOutcome::Taken((id, ResponseKind::Response))),
            TakeOutcome::Declined => Ok(TakeOutcome::Declined),
            TakeOutcome::Skipped | TakeOutcome::Empty => {
                unreachable!("a serialization is taken or declined")
            }
        }
    }

    /// Writes the received request into the destination.
    ///
    /// The payload location is populated with the received bytes in wire form.
    /// The header location is populated with the corresponding service info
    /// as a [`RosHeader`].
    fn take_request<'a>(
        &mut self,
        destination: impl RequestDestination<'a>,
    ) -> Result<TakeOutcome<RequestId>, Self::Failure> {
        let origin = origin!("RequestResponseEndpoints::take_request");

        self.request.reset();
        let info = match self.server.take_request(&mut self.request) {
            Ok(Some(info)) => info,
            Ok(None) => return Ok(TakeOutcome::Empty),
            Err(_) => {
                fail!(
                    from origin,
                    with RequestResponseEndpointsError::Take,
                    "The rmw failed to take a request"
                );
            }
        };

        if self.is_own(&info.request_id) {
            return Ok(TakeOutcome::Skipped);
        }

        match serialize_into(&self.request, RosHeader::from(info), destination)? {
            TakeOutcome::Taken(()) => Ok(TakeOutcome::Taken(info.request_id)),
            TakeOutcome::Declined => Ok(TakeOutcome::Declined),
            TakeOutcome::Skipped | TakeOutcome::Empty => {
                unreachable!("a serialization is taken or declined")
            }
        }
    }

    /// Sends the payload bytes of the response to the client of the request.
    ///
    /// The header is ignored as ROS 2 cannot carry it. A second response to
    /// the same request is dropped.
    fn send_response(
        &mut self,
        id: &RequestId,
        response: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure> {
        let origin = origin!("RequestResponseEndpoints::send_response");

        if !self.answered.insert(*id) {
            warn!(
                "A further response to the request {:?} is dropped, a ROS 2 service answers once",
                id
            );
            return Ok(());
        }

        self.response.reset();
        fail!(
            from origin,
            when self.response.deserialize_from(response.payload),
            with RequestResponseEndpointsError::Deserialize,
            "Failed to deserialize the response to the request {:?}", id
        );
        fail!(
            from origin,
            when self.server.send_response(id, &self.response),
            with RequestResponseEndpointsError::Send,
            "Failed to send the response to the request {:?}", id
        );

        Ok(())
    }

    /// Forgets the request, since ROS 2 has no end of stream to send.
    fn end_response_stream(&mut self, id: RequestId) -> Result<(), Self::Failure> {
        if !self.answered.remove(&id) {
            debug!(
                "The response stream of the request {:?} ended without a response, its ROS 2 client is left to its timeout",
                id
            );
        }
        Ok(())
    }
}

/// Serializes `message` to CDR into the payload location `destination`
/// provides and writes `header` into the header location.
///
/// Declined when the destination does not honor the lengths.
fn serialize_into<'a>(
    message: &RosMessage,
    header: RosHeader,
    destination: impl SampleDestination<'a>,
) -> Result<TakeOutcome<()>, RequestResponseEndpointsError> {
    let origin = origin!("serialize_into");

    // Provide the payload location to the rmw as the buffer to write the
    // serialized message.
    let mut header_location = None;
    let mut declined = false;
    let serialized = message.serialize_into(|size| {
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

    match serialized {
        Ok(_) => {}
        Err(SerializeError::LoanDeclined) if declined => return Ok(TakeOutcome::Declined),
        Err(SerializeError::LoanDeclined) => {
            fail!(
                from origin,
                with RequestResponseEndpointsError::NotWritten,
                "The rmw rejected the memory it was given for the message"
            );
        }
        Err(SerializeError::Serialize) => {
            fail!(
                from origin,
                with RequestResponseEndpointsError::Serialize,
                "The rmw failed to serialize a message"
            );
        }
    }
    let Some(header_location) = header_location else {
        fail!(
            from origin,
            with RequestResponseEndpointsError::NotWritten,
            "The rmw serialized the message without asking for memory to write it into"
        );
    };

    // Copy the header into the destination.
    header_location.copy_from_slice(bytes_of(&header));

    Ok(TakeOutcome::Taken(()))
}
