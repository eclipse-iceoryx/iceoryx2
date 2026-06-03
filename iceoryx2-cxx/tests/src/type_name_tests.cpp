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

#include "iox2/internal/service_builder_internal.hpp"
#include "iox2/node.hpp"
#include "iox2/service.hpp"
#include "iox2/type_name.hpp"

#include "test.hpp"
#include <cstdint>
#include <gtest/gtest.h>

// NOLINTBEGIN(misc-use-internal-linkage,misc-non-private-member-variables-in-classes) : test types
namespace type_name_tests {

constexpr const char* CUSTOM_TYPE_NAME = "type_name_tests::PayloadWithCustomName";
constexpr const char* DIFFERENT_CUSTOM_TYPE_NAME = "type_name_tests::PayloadWithDifferentCustomName";
constexpr const char* MEMBER_TYPE_NAME = "type_name_tests::PayloadWithMemberName";
constexpr const char* GENERATED_TYPE_NAME = "type_name_tests/generated/PayloadFromGenerator";

struct PayloadWithCustomName {
    int32_t x;
    double y;
    auto operator==(const PayloadWithCustomName& rhs) const -> bool {
        return x == rhs.x && y == rhs.y;
    }
};

struct DifferentPayloadWithSameCustomName {
    int32_t x;
    double y;
    auto operator==(const DifferentPayloadWithSameCustomName& rhs) const -> bool {
        return x == rhs.x && y == rhs.y;
    }
};

struct PayloadWithDifferentCustomName {
    int32_t x;
    double y;
    auto operator==(const PayloadWithDifferentCustomName& rhs) const -> bool {
        return x == rhs.x && y == rhs.y;
    }
};

// Neither a custom name nor an IOX2_TYPE_NAME member: falls back to the default
// (ABI-mangled) name.
struct PayloadWithoutCustomName {
    int32_t x;
    double y;
    auto operator==(const PayloadWithoutCustomName& rhs) const -> bool {
        return x == rhs.x && y == rhs.y;
    }
};

struct PayloadWithMemberName {
    static constexpr const char* IOX2_TYPE_NAME = MEMBER_TYPE_NAME;
    int32_t x;
    double y;
};

// Stands in for a code generator's accessor function.
struct PayloadFromGenerator {
    int32_t x;
    double y;
};

template <typename T>
auto generated_name() -> const char*;

template <>
auto generated_name<PayloadFromGenerator>() -> const char* {
    return GENERATED_TYPE_NAME;
}

} // namespace type_name_tests
// NOLINTEND(misc-use-internal-linkage,misc-non-private-member-variables-in-classes)

IOX2_DEFINE_TYPE_NAME(type_name_tests::PayloadWithCustomName, type_name_tests::CUSTOM_TYPE_NAME);
IOX2_DEFINE_TYPE_NAME(type_name_tests::DifferentPayloadWithSameCustomName, type_name_tests::CUSTOM_TYPE_NAME);
IOX2_DEFINE_TYPE_NAME(type_name_tests::PayloadWithDifferentCustomName, type_name_tests::DIFFERENT_CUSTOM_TYPE_NAME);
IOX2_DEFINE_TYPE_NAME(type_name_tests::PayloadFromGenerator,
                      type_name_tests::generated_name<type_name_tests::PayloadFromGenerator>());

namespace {

using namespace iox2;
using namespace type_name_tests;

TEST(GetTypeNameTest, get_type_name_retrieves_type_name_from_hard_coded_specialization) {
    ASSERT_STREQ(iox2::internal::get_type_name<PayloadWithCustomName>().unchecked_access().c_str(), CUSTOM_TYPE_NAME);
}

TEST(GetTypeNameTest, get_type_name_retrieves_type_name_from_generator_function) {
    ASSERT_STREQ(iox2::internal::get_type_name<PayloadFromGenerator>().unchecked_access().c_str(), GENERATED_TYPE_NAME);
}

TEST(GetTypeNameTest, get_type_name_retrieves_type_name_from_struct_member) {
    ASSERT_STREQ(iox2::internal::get_type_name<PayloadWithMemberName>().unchecked_access().c_str(), MEMBER_TYPE_NAME);
}

TEST(GetTypeNameTest, get_type_name_for_slice_uses_custom_name_from_inner_type) {
    ASSERT_STREQ(iox2::internal::get_type_name<bb::Slice<PayloadWithCustomName>>().unchecked_access().c_str(),
                 CUSTOM_TYPE_NAME);
}

// BEGIN publish-subscribe service

template <typename T>
class ServiceTypeNameTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServiceTypeNameTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceTypeNameTest, opening_existing_publish_subscribe_service_with_same_custom_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template publish_subscribe<PayloadWithCustomName>().create().value();
    auto sut_open = node.service_builder(service_name).template publish_subscribe<PayloadWithCustomName>().open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest,
           opening_existing_publish_subscribe_service_with_different_payload_but_same_custom_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template publish_subscribe<PayloadWithCustomName>().create().value();
    auto sut_open =
        node.service_builder(service_name).template publish_subscribe<DifferentPayloadWithSameCustomName>().open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_publish_subscribe_service_with_payload_using_default_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template publish_subscribe<PayloadWithCustomName>().create().value();
    auto sut_open = node.service_builder(service_name).template publish_subscribe<PayloadWithoutCustomName>().open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_publish_subscribe_service_with_different_custom_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template publish_subscribe<PayloadWithCustomName>().create().value();
    auto sut_open =
        node.service_builder(service_name).template publish_subscribe<PayloadWithDifferentCustomName>().open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

// END publish-subscribe service

// BEGIN request-response service

TYPED_TEST(ServiceTypeNameTest, opening_existing_request_response_service_with_same_custom_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<PayloadWithCustomName, PayloadWithCustomName>()
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<PayloadWithCustomName, PayloadWithCustomName>()
                        .open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest,
           opening_existing_request_response_service_with_different_payload_but_same_custom_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<PayloadWithCustomName, PayloadWithCustomName>()
                          .create()
                          .value();
    auto sut_open =
        node.service_builder(service_name)
            .template request_response<DifferentPayloadWithSameCustomName, DifferentPayloadWithSameCustomName>()
            .open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_request_response_service_with_payload_using_default_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<PayloadWithCustomName, PayloadWithCustomName>()
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<PayloadWithoutCustomName, PayloadWithCustomName>()
                        .open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), RequestResponseOpenError::IncompatibleRequestOrResponseType);
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_request_response_service_with_different_custom_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<PayloadWithCustomName, PayloadWithCustomName>()
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<PayloadWithDifferentCustomName, PayloadWithCustomName>()
                        .open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), RequestResponseOpenError::IncompatibleRequestOrResponseType);
}

// END request-response service

// BEGIN blackboard service

TYPED_TEST(ServiceTypeNameTest, opening_existing_blackboard_service_with_same_custom_key_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<PayloadWithCustomName>()
                          .template add_with_default<uint64_t>(PayloadWithCustomName {})
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name).template blackboard_opener<PayloadWithCustomName>().open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest,
           opening_existing_blackboard_service_with_different_key_but_same_custom_key_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<PayloadWithCustomName>()
                          .template add_with_default<uint64_t>(PayloadWithCustomName {})
                          .create()
                          .value();
    auto sut_open =
        node.service_builder(service_name).template blackboard_opener<DifferentPayloadWithSameCustomName>().open();

    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_blackboard_service_with_key_using_default_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<PayloadWithCustomName>()
                          .template add_with_default<uint64_t>(PayloadWithCustomName {})
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name).template blackboard_opener<PayloadWithoutCustomName>().open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), BlackboardOpenError::IncompatibleKeys);
}

TYPED_TEST(ServiceTypeNameTest, opening_existing_blackboard_service_with_different_custom_key_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<PayloadWithCustomName>()
                          .template add_with_default<uint64_t>(PayloadWithCustomName {})
                          .create()
                          .value();
    auto sut_open =
        node.service_builder(service_name).template blackboard_opener<PayloadWithDifferentCustomName>().open();

    ASSERT_FALSE(sut_open.has_value());
    EXPECT_EQ(sut_open.error(), BlackboardOpenError::IncompatibleKeys);
}

// END blackboard service

} // namespace
