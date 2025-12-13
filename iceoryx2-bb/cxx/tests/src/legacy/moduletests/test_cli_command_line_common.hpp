// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_MODULETESTS_TEST_CLI_COMMAND_LINE_COMMON_HPP
#define IOX2_BB_MODULETESTS_TEST_CLI_COMMAND_LINE_COMMON_HPP

#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

struct CmdArgs {
    int argc = 0;
    char** argv = nullptr;

    explicit CmdArgs(const std::vector<std::string>& arguments)
        : argc { static_cast<int>(arguments.size()) }
        , argv { new char*[static_cast<size_t>(argc)] } {
        contents = std::make_unique<std::vector<std::string>>(arguments);
        for (size_t i = 0; i < static_cast<size_t>(argc); ++i) {
            // NOLINTJUSTIFICATION required for test
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic)
            argv[i] = const_cast<char*>((*contents)[i].data());
        }
    }

    ~CmdArgs() {
        delete[] argv;
    }

    std::unique_ptr<std::vector<std::string>> contents;
};

class OutBuffer {
  public:
    OutBuffer() {
        std::cout.rdbuf(m_capture.rdbuf());
    }
    ~OutBuffer() {
        std::cout.rdbuf(m_originalOutBuffer);
    }

    void clear() {
        m_capture.str("");
    }

    std::string output() {
        return m_capture.str();
    }

  private:
    std::streambuf* m_originalOutBuffer { std::cout.rdbuf() };
    std::stringstream m_capture;
};

#endif // IOX2_BB_MODULETESTS_TEST_CLI_COMMAND_LINE_COMMON_HPP
