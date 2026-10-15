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

mod event;
mod publish_subscribe;
mod request_response;

pub(crate) use event::EventPorts;
pub(crate) use publish_subscribe::PublishSubscribePorts;
pub(crate) use request_response::RequestResponsePorts;

use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_link_backend::service_description::TypeDescription;

/// The number of payload elements `number_of_bytes` holds.
fn number_of_elements(payload: &TypeDescription, number_of_bytes: usize) -> Option<usize> {
    match payload.variant {
        TypeVariant::FixedSize => (number_of_bytes == payload.size).then_some(1),
        TypeVariant::Dynamic => {
            let whole = payload.size != 0 && number_of_bytes.is_multiple_of(payload.size);
            whole.then(|| number_of_bytes / payload.size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::TypeIdentifier;

    const ALIGNMENT: usize = 1;

    fn payload(variant: TypeVariant, size: usize) -> TypeDescription {
        TypeDescription {
            variant,
            identifier: TypeIdentifier::Name("test_type".into()),
            size,
            alignment: ALIGNMENT,
        }
    }

    #[test]
    fn fixed_size_payload_holds_exactly_one_element() {
        const ELEMENT_SIZE: usize = 8;

        let payload = payload(TypeVariant::FixedSize, ELEMENT_SIZE);
        assert_that!(number_of_elements(&payload, ELEMENT_SIZE), eq Some(1));
        assert_that!(number_of_elements(&payload, 0), eq None);
        assert_that!(number_of_elements(&payload, 2 * ELEMENT_SIZE), eq None);
    }

    #[test]
    fn dynamic_payload_holds_whole_elements() {
        const ELEMENT_SIZE: usize = 4;

        let payload = payload(TypeVariant::Dynamic, ELEMENT_SIZE);
        assert_that!(number_of_elements(&payload, 0), eq Some(0));
        assert_that!(number_of_elements(&payload, 3 * ELEMENT_SIZE), eq Some(3));
        assert_that!(number_of_elements(&payload, ELEMENT_SIZE + ELEMENT_SIZE / 2), eq None);
    }
}
