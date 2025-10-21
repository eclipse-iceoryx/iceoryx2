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

use iceoryx2_bb_testing::instantiate_conformance_tests;
use iceoryx2_cal::resizable_shared_memory::dynamic::DynamicMemory;
use iceoryx2_cal_conformance_tests::resizable_shared_memory_trait::DefaultAllocator;

mod posix {
    use super::*;
    use iceoryx2_cal::shared_memory::posix::Memory;

    type SharedMemory = Memory<super::DefaultAllocator>;
    type ResizeableSharedMemory = DynamicMemory<super::DefaultAllocator, SharedMemory>;

    instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::resizable_shared_memory_trait,
        super::SharedMemory,
        super::ResizeableSharedMemory
    );
}

mod process_local {
    use super::*;
    use iceoryx2_cal::shared_memory::process_local::Memory;

    type SharedMemory = Memory<super::DefaultAllocator>;
    type ResizeableSharedMemory = DynamicMemory<super::DefaultAllocator, SharedMemory>;

    instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::resizable_shared_memory_trait,
        super::SharedMemory,
        super::ResizeableSharedMemory
    );
}
