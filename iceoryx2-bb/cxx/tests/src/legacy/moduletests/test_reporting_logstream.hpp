// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#include "iox2/legacy/log/logstream.hpp"
#include "iox2/legacy/logging.hpp"
#include "iox2/legacy/testing/mocks/logger_mock.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;

using iox2::legacy::testing::Logger_Mock;

class LogStreamSut : public iox2::legacy::log::LogStream {
  public:
    explicit LogStreamSut(iox2::legacy::log::Logger& logger)
        : iox2::legacy::log::LogStream(logger, "file", 42, "function", iox2::legacy::log::LogLevel::Trace) {
    }
};

class IoxLogStreamBase_test : public Test {
  public:
    Logger_Mock loggerMock;
};
