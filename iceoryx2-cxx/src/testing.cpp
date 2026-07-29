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

#include "iox2/testing.hpp"
#include "iox2/bb/file_name.hpp"
#include "iox2/bb/file_path.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
namespace testing {
void create_test_directory() {
    iox2_testing_create_test_directory();
}

auto generate_file_name() -> bb::FileName {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) C library abstraction
    char buffer[bb::platform::IOX2_MAX_FILENAME_LENGTH] {};

    iox2_testing_generate_file_name(&buffer[0], bb::platform::IOX2_MAX_FILENAME_LENGTH);
    return bb::FileName::create(&buffer[0]).value();
}

auto generate_file_path() -> bb::FilePath {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) C library abstraction
    char buffer[bb::platform::IOX2_MAX_PATH_LENGTH] {};

    iox2_testing_generate_file_path(&buffer[0], bb::platform::IOX2_MAX_PATH_LENGTH);
    return bb::FilePath::create(&buffer[0]).value();
}

auto test_directory_path() -> bb::Path {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) C library abstraction
    char buffer[bb::platform::IOX2_MAX_PATH_LENGTH] {};

    iox2_testing_test_directory_path(&buffer[0], bb::platform::IOX2_MAX_PATH_LENGTH);
    return bb::Path::create(&buffer[0]).value();
}

auto generate_service_name() -> ServiceName {
    static std::atomic<uint64_t> COUNTER { 0 };
    const auto now = std::chrono::system_clock::now().time_since_epoch().count();
    const auto random_number = rand(); // NOLINT(cert-msc30-c,cert-msc50-cpp, misc-predictable-rand)
    return ServiceName::create((std::string("test_") + std::to_string(COUNTER.fetch_add(1)) + "_" + std::to_string(now)
                                + "_" + std::to_string(random_number))
                                   .c_str())
        .value();
}
} // namespace testing
} // namespace iox2
