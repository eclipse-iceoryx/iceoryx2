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

mod node_name {
    use iceoryx2::prelude::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn creating_works() {
        let value = "tschi tschi bum bum";
        let sut = NodeName::new(value).unwrap();

        assert_that!(sut, eq value);
        assert_that!(&sut, eq value);
    }

    #[test]
    fn display_works() {
        let value = "lakirski materialski";
        let sut = NodeName::new(value).unwrap();

        assert_that!(format!("{}", sut), eq value);
    }

    #[test]
    fn try_into_works() {
        let value = "all glory to david hypnotoad";
        let sut: NodeName = value.try_into().unwrap();

        assert_that!(sut, eq value);
        assert_that!(&sut, eq value);
    }
}
