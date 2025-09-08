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

use iceoryx2_cal_conformance_tests::serialize_trait;
use iceoryx2_cal_conformance_tests::serialize_trait_tests;

mod toml {
    use super::*;
    serialize_trait_tests!(iceoryx2_cal::serialize::toml::Toml);
}

mod cdr {
    use super::*;
    serialize_trait_tests!(iceoryx2_cal::serialize::cdr::Cdr);
}

mod postcard {
    use super::*;
    serialize_trait_tests!(iceoryx2_cal::serialize::postcard::Postcard);
}
