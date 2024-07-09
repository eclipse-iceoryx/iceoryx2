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

#[cfg(test)]
mod attribute {
    use iceoryx2::service::attribute::AttributeVerifier;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn attribute_returns_correct_key_value() {
        let sut = AttributeVerifier::new().require("key_1", "value_1");

        for entry in sut.attributes().iter() {
            assert_that!(entry.key(), eq "key_1");
            assert_that!(entry.value(), eq "value_1");
        }

        assert_that!(sut.attributes().iter(), len 1);
    }
}
