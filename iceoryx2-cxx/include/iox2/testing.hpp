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

#ifndef IOX2_TESTING_HPP
#define IOX2_TESTING_HPP

#include "iox2/bb/file_name.hpp"
#include "iox2/bb/file_path.hpp"
#include "iox2/bb/path.hpp"
#include "iox2/service_name.hpp"

namespace iox2 {
namespace testing {
/// Generates a random all-time unique service name.
auto generate_service_name() -> ServiceName;

/// Creates the test directory to store test artifacts.
void create_test_directory();

/// Generates an all-time unique file name.
auto generate_file_name() -> bb::FileName;

/// Generates an all-time unique file name located inside the test directory.
auto generate_file_path() -> bb::FilePath;

/// Returns the current test directory path.
auto test_directory_path() -> bb::Path;
} // namespace testing
} // namespace iox2

#endif
