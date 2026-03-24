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

use iceoryx2_bb_testing::instantiate_conformance_tests_with_module;
use iceoryx2_cal_conformance_tests::shared_memory_trait::DefaultAllocator;

instantiate_conformance_tests_with_module!(
    posix,
    iceoryx2_cal_conformance_tests::shared_memory_trait,
    iceoryx2_cal::shared_memory::posix::Memory<super::DefaultAllocator>
);

instantiate_conformance_tests_with_module!(
    process_local,
    iceoryx2_cal_conformance_tests::shared_memory_trait,
    iceoryx2_cal::shared_memory::process_local::Memory<super::DefaultAllocator>
);

// disabled on windows since windows 2022 (not windows 10 or 11)
// has some weird file remove issue which cause unit test failures that are
// non-reproducible on windows 10 or 11
#[cfg(not(target_os = "windows"))]
instantiate_conformance_tests_with_module!(
    file,
    iceoryx2_cal_conformance_tests::shared_memory_trait,
    iceoryx2_cal::shared_memory::file::Memory<super::DefaultAllocator>
);
