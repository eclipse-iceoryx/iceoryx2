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

#include "iox2/legacy/testing/error_reporting/testing_error_handler.hpp"
#include "iox2/legacy/testing/testing_logger.hpp"

#include <gtest/gtest.h>

auto main(int argc, char* argv[]) -> int {
    ::testing::InitGoogleTest(&argc, argv);

    iox2::legacy::testing::TestingLogger::init();
    iox2::legacy::testing::TestingErrorHandler::init();

    return RUN_ALL_TESTS();
}
