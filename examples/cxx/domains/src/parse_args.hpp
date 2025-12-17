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

#ifndef IOX2_EXAMPLES_PARSE_ARDS_HPP
#define IOX2_EXAMPLES_PARSE_ARDS_HPP

#include "iox2/bb/static_string.hpp"

#include <iostream>
#include <string>

// NOLINTNEXTLINE(misc-use-internal-linkage) fine for the example
template <typename F>
auto check_for_help_from_args(int argc, char** argv, F print_help) -> void { // NOLINT
    for (int i = 0; i < argc; ++i) {
        const std::string arg(argv[i]); // NOLINT
        const std::string help_short("-h");
        const std::string help_long("--help");
        if (arg == help_short || arg == help_long) {
            print_help();
            exit(0);
        }
    }
}

template <uint64_t N>
struct CliOption {
    std::string short_option;
    std::string long_option;
    iox2::bb::StaticString<N> default_value;
    std::string error_string;
};

template <uint64_t N>
auto parse_from_args(int argc, char** argv, CliOption<N> opt) -> iox2::bb::StaticString<N> {
    for (int i = 0; i < argc; ++i) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic) required to parse args
        const std::string arg(argv[i]);
        if (arg == opt.short_option || arg == opt.long_option) {
            if (i + 1 < argc) {
                // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic) required to parse args
                return { iox2::bb::StaticString<N>::from_utf8_null_terminated_unchecked_truncated(argv[i + 1], N) };
            } else {
                std::cout << opt.error_string << std::endl;
                exit(1);
            }
        }
    }

    return opt.default_value;
}

#endif
