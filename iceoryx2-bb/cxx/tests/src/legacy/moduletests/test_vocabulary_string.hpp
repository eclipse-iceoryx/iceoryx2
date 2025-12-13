// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_MODULETESTS_TEST_VOCABULARY_STRING_HPP
#define IOX2_BB_MODULETESTS_TEST_VOCABULARY_STRING_HPP

#include "iox2/legacy/string.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;
using namespace iox2::legacy;

template <typename T>
class stringTyped_test : public Test {
  protected:
    T testSubject;

    using stringType = T;
};

using StringImplementations = Types<string<1>, string<15>, string<100>, string<1000>>;

#endif // IOX2_BB_MODULETESTS_TEST_VOCABULARY_STRING_HPP
