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

#include "iox2/version.hpp"
#include "test.hpp"

#include <sstream>

namespace {
using namespace iox2;

TEST(VersionTest, version_obtains_version_number) {
    ASSERT_EQ(package_version().major, 0);
    ASSERT_EQ(package_version().minor, 8);
    ASSERT_EQ(package_version().patch, 999);
}

TEST(VersionTest, version_numbers_compare_equal_if_all_components_are_equal) {
    PackageVersion sut1;
    sut1.major = 1;
    sut1.minor = 2;
    sut1.patch = 3;
    PackageVersion sut2;
    sut2.major = 1;
    sut2.minor = 2;
    sut2.patch = 3;
    EXPECT_EQ(sut1, sut1);
    EXPECT_EQ(sut1, sut2);
    EXPECT_EQ(sut2, sut1);
    sut1.major = 25;
    sut1.minor = 22;
    sut1.patch = 0;
    sut2.major = 25;
    sut2.minor = 22;
    sut2.patch = 0;
    EXPECT_EQ(sut1, sut1);
    EXPECT_EQ(sut1, sut2);
    EXPECT_EQ(sut2, sut1);
}

TEST(VersionTest, version_numbers_do_not_compare_equal_if_major_version_differs) {
    PackageVersion sut1;
    sut1.major = 1;
    sut1.minor = 2;
    sut1.patch = 3;
    PackageVersion sut2;
    sut2.major = 0;
    sut2.minor = 2;
    sut2.patch = 3;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
    sut1.major = 99;
    sut2.major = 6;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
}

TEST(VersionTest, version_numbers_do_not_compare_equal_if_minor_version_differs) {
    PackageVersion sut1;
    sut1.major = 1;
    sut1.minor = 2;
    sut1.patch = 3;
    PackageVersion sut2;
    sut2.major = 1;
    sut2.minor = 0;
    sut2.patch = 3;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
    sut1.minor = 99;
    sut2.minor = 6;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
}

TEST(VersionTest, version_numbers_do_not_compare_equal_if_patch_version_differs) {
    PackageVersion sut1;
    sut1.major = 1;
    sut1.minor = 2;
    sut1.patch = 3;
    PackageVersion sut2;
    sut2.major = 1;
    sut2.minor = 2;
    sut2.patch = 0;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
    sut1.patch = 99;
    sut2.patch = 6;
    EXPECT_FALSE(sut1 == sut2);
    EXPECT_FALSE(sut2 == sut1);
}

TEST(VersionTest, version_numbers_less_compares_lexicographically) {
    PackageVersion sut1;
    sut1.major = 1;
    sut1.minor = 2;
    sut1.patch = 3;
    PackageVersion sut2;
    sut2.major = 2;
    sut2.minor = 2;
    sut2.patch = 3;
    EXPECT_LT(sut1, sut2);
    EXPECT_FALSE(sut2 < sut1);
    sut2.major = sut1.major;
    sut2.minor = 3;
    EXPECT_LT(sut1, sut2);
    EXPECT_FALSE(sut2 < sut1);
    sut2.minor = sut1.minor;
    sut2.patch = 4;
    EXPECT_LT(sut1, sut2);
    EXPECT_FALSE(sut2 < sut1);
    sut2.patch = sut1.patch;
    EXPECT_FALSE(sut1 < sut2);
    EXPECT_FALSE(sut2 < sut1);
}

TEST(VersionTest, version_numbers_ostream_insertion_produces_version_string) {
    std::stringstream sstr;
    PackageVersion sut;
    sut.major = 0;
    sut.minor = 0;
    sut.patch = 0;
    sstr << sut;
    ASSERT_FALSE(sstr.fail());
    EXPECT_STREQ(sstr.str().c_str(), "0.0.0");
    sstr = std::stringstream {};
    sut.major = 22;
    sut.minor = 4;
    sut.patch = 102;
    sstr << sut;
    ASSERT_FALSE(sstr.fail());
    EXPECT_STREQ(sstr.str().c_str(), "22.4.102");
}

} // namespace
