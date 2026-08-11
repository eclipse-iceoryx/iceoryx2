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

use crate::identifiers::UniqueNodeId;

/// The header of every data-flow payload.
pub trait PayloadHeader {
    /// Returns the [`UniqueNodeId`] of the source node that sent the payload.
    fn node_id(&self) -> UniqueNodeId;

    /// Returns how many elements are stored inside the payload.
    ///
    /// # Details when using
    /// [`CustomPayloadMarker`](crate::service::marker::CustomPayloadMarker)
    ///
    /// In this case the number of elements relates to the element defined in the
    /// [`MessageTypeDetails`](crate::service::static_config::message_type_details::MessageTypeDetails).
    /// When the element has a `payload.size == 40` and the `payload.len == 120` it
    /// means that it contains 3 elements (3 * 40 == 120).
    fn number_of_elements(&self) -> u64;
}
