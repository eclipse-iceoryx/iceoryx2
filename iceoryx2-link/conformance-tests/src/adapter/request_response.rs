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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod adapter_request_response {
    use alloc::vec::Vec;
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_adapter::{
        Adapter, ClientRequestId, RequestResponseEndpoints, ResponseKind, SampleBytes,
        SampleBytesRef, TakeOutcome,
    };

    use crate::fixture::{AdapterFixture, RequestResponseMessageEndpoints, ResponseMessage};
    use crate::testing::{ClientRequestIds, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const REQUEST: u64 = 7;
    const RESPONSE: u64 = 9;
    const NO_HEADER: &[u8] = &[];

    /// Sends a request without a header.
    fn send_request<E: RequestResponseEndpoints>(
        endpoints: &mut E,
        id: ClientRequestId,
        payload: &[u8],
    ) {
        endpoints
            .send_request(
                id,
                SampleBytesRef {
                    header: NO_HEADER,
                    payload,
                },
            )
            .expect("sending the request succeeds");
    }

    /// The pending request, if any, its id and its payload.
    fn take_request<E: RequestResponseEndpoints>(
        endpoints: &mut E,
    ) -> Option<(E::ServerRequestId, Vec<u8>)> {
        let mut bytes = SampleBytes::default();
        match endpoints
            .take_request(&mut bytes)
            .expect("taking the request succeeds")
        {
            TakeOutcome::Taken(id) => Some((id, bytes.payload)),
            TakeOutcome::Declined | TakeOutcome::Skipped | TakeOutcome::Empty => None,
        }
    }

    /// Sends a response without a header.
    fn send_response<E: RequestResponseEndpoints>(
        endpoints: &mut E,
        id: &E::ServerRequestId,
        payload: &[u8],
    ) {
        endpoints
            .send_response(
                id,
                SampleBytesRef {
                    header: NO_HEADER,
                    payload,
                },
            )
            .expect("sending the response succeeds");
    }

    /// The pending message of the response stream of `held`, if any.
    fn take_response<E: RequestResponseEndpoints>(
        endpoints: &mut E,
        held: ClientRequestId,
    ) -> Option<(ClientRequestId, ResponseMessage<Vec<u8>>)> {
        let mut bytes = SampleBytes::default();
        let destination = {
            let bytes = &mut bytes;
            move |id: ClientRequestId| (id == held).then_some(bytes)
        };
        match endpoints
            .take_response(destination)
            .expect("taking the response succeeds")
        {
            TakeOutcome::Taken((id, ResponseKind::Response)) => {
                Some((id, ResponseMessage::Response(bytes.payload)))
            }
            TakeOutcome::Taken((id, ResponseKind::End)) => Some((id, ResponseMessage::End)),
            TakeOutcome::Declined | TakeOutcome::Skipped | TakeOutcome::Empty => None,
        }
    }

    /// The id of the request the remote endpoints receive within the
    /// timeout.
    fn received_request_id<S, T, R: RequestResponseMessageEndpoints<S, T>>(
        remote: &R,
    ) -> R::RequestId {
        retry(
            || {
                remote
                    .receive_request()
                    .map(|(id, _)| id)
                    .ok_or("no request arrived")
            },
            TIMEOUT,
        )
        .expect("the request reaches the remote endpoints")
    }

    #[conformance_test]
    pub fn requests_reach_the_adapter_from_the_remote_endpoints<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");

        // === REQUEST ===
        assert_that!(remote.sync(TIMEOUT), eq true);

        let request = remote.request(REQUEST);
        remote.send_request(request.clone());

        // Deserialize the request from the wire form of the service.
        retry(
            || match take_request(&mut own).map(|(_, bytes)| remote.decode_request(&bytes)) {
                Some(received) if received == request => Ok(()),
                Some(_) => Err("an unexpected request arrived"),
                None => Err("no request arrived"),
            },
            TIMEOUT,
        )
        .expect("the request reaches the adapter");
    }

    #[conformance_test]
    pub fn responses_reach_the_remote_endpoints_from_the_adapter<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // A request from the remote endpoints taken by the gateway's
        // endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        assert_that!(remote.sync(TIMEOUT), eq true);
        let request_id = remote.send_request(remote.request(REQUEST));
        let taken = retry(
            || {
                take_request(&mut own)
                    .map(|(id, _)| id)
                    .ok_or("no request arrived")
            },
            TIMEOUT,
        )
        .expect("the request reaches the adapter");

        // === RESPOND ===
        // Serialize the response to the wire form of the service.
        let response = remote.response(RESPONSE);
        send_response(&mut own, &taken, &remote.encode_response(&response));
        own.end_response_stream(taken)
            .expect("ending the response stream succeeds");

        retry(
            || match remote.receive_response() {
                Some((id, ResponseMessage::Response(received)))
                    if id == request_id && received == response =>
                {
                    Ok(())
                }
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no response arrived"),
            },
            TIMEOUT,
        )
        .expect("the response reaches the remote endpoints");
        retry(
            || match remote.receive_response() {
                Some((id, ResponseMessage::End)) if id == request_id => Ok(()),
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no end arrived"),
            },
            TIMEOUT,
        )
        .expect("the end reaches the remote endpoints");
    }

    #[conformance_test]
    pub fn requests_reach_the_remote_endpoints_from_the_adapter<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let ids = ClientRequestIds::new();

        // === REQUEST ===
        assert_that!(remote.sync(TIMEOUT), eq true);

        // Serialize the request to the wire form of the service.
        let request = remote.request(REQUEST);
        send_request(&mut own, ids.next(), &remote.encode_request(&request));

        retry(
            || match remote.receive_request() {
                Some((_, received)) if received == request => Ok(()),
                Some(_) => Err("an unexpected request arrived"),
                None => Err("no request arrived"),
            },
            TIMEOUT,
        )
        .expect("the request reaches the remote endpoints");
    }

    #[conformance_test]
    pub fn responses_reach_the_adapter_from_the_remote_endpoints<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // A request from the gateway's endpoints received by the remote
        // endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let ids = ClientRequestIds::new();
        assert_that!(remote.sync(TIMEOUT), eq true);
        let id = ids.next();
        send_request(
            &mut own,
            id,
            &remote.encode_request(&remote.request(REQUEST)),
        );
        let request_id = received_request_id(&remote);

        // === RESPOND ===
        let response = remote.response(RESPONSE);
        remote.send_response(request_id, response.clone());
        remote.end_response_stream(request_id);

        // Deserialize the response from the wire form of the service.
        retry(
            || match take_response(&mut own, id) {
                Some((received_id, ResponseMessage::Response(bytes)))
                    if received_id == id && remote.decode_response(&bytes) == response =>
                {
                    Ok(())
                }
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no response arrived"),
            },
            TIMEOUT,
        )
        .expect("the response reaches the adapter");
        retry(
            || match take_response(&mut own, id) {
                Some((received_id, ResponseMessage::End)) if received_id == id => Ok(()),
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no end arrived"),
            },
            TIMEOUT,
        )
        .expect("the end reaches the adapter");
    }

    #[conformance_test]
    pub fn the_adapter_does_not_take_its_own_requests<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // The gateway's endpoints and matching remote endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let ids = ClientRequestIds::new();

        // === REQUEST ===
        // What the adapter's client sends does not reach its server.
        assert_that!(remote.sync(TIMEOUT), eq true);
        send_request(
            &mut own,
            ids.next(),
            &remote.encode_request(&remote.request(REQUEST)),
        );
        received_request_id(&remote);

        assert_that!(take_request(&mut own), is_none);
    }

    #[conformance_test]
    pub fn responses_to_requests_no_longer_held_are_skipped<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // A request from the gateway's endpoints answered by the remote
        // endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_request_response_endpoints();
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let ids = ClientRequestIds::new();

        assert_that!(remote.sync(TIMEOUT), eq true);
        send_request(
            &mut own,
            ids.next(),
            &remote.encode_request(&remote.request(REQUEST)),
        );
        let request_id = received_request_id(&remote);
        remote.send_response(request_id, remote.response(RESPONSE));

        // === TAKE ===
        // No destination for the request, the response is dropped.
        retry(
            || match own.take_response(|_| None::<&mut SampleBytes>) {
                Ok(TakeOutcome::Skipped) => Ok(()),
                Ok(TakeOutcome::Empty) => Err("no response arrived"),
                Ok(_) => Err("the response was not skipped"),
                Err(_) => Err("taking the response failed"),
            },
            TIMEOUT,
        )
        .expect("the response is skipped");
    }
}
