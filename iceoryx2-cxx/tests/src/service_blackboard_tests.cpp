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

#include "iox2/bb/optional.hpp"
#include "iox2/bb/static_string.hpp"
#include "iox2/entry_handle_mut.hpp"
#include "iox2/entry_value_uninit.hpp"
#include "iox2/node.hpp"
#include "iox2/port_factory_blackboard.hpp"
#include "iox2/reader_error.hpp"
#include "iox2/service.hpp"
#include "iox2/service_builder_blackboard_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/type_variant.hpp"
#include "iox2/writer_error.hpp"
#include "test.hpp"
#include <cstdint>

namespace {
using namespace iox2;

template <typename T>
class ServiceBlackboardTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServiceBlackboardTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceBlackboardTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                .value());
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());
}

TYPED_TEST(ServiceBlackboardTest, service_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create()
                   .value();

    ASSERT_THAT(sut.name().to_string().unchecked_access().c_str(),
                StrEq(service_name.to_string().unchecked_access().c_str()));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceBlackboardTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("nala is hungry").value();
    const auto node_name_2 = NodeName::create("maybe octo-wolf can help?").value();
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().value();
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().value();

    auto sut_1 = node_1.service_builder(service_name)
                     .template blackboard_creator<uint64_t>()
                     .template add_with_default<uint64_t>(0)
                     .create()
                     .value();
    auto sut_2 = node_2.service_builder(service_name).template blackboard_opener<uint64_t>().open().value();

    auto counter = 0;
    auto verify_node = [&](const AliveNodeView<SERVICE_TYPE>& node_view) -> auto {
        counter++;
        if (node_view.id() == node_1.id()) {
            ASSERT_THAT(node_view.details()->name().to_string().unchecked_access().c_str(),
                        StrEq(node_1.name().to_string().unchecked_access().c_str()));
        } else {
            ASSERT_THAT(node_view.details()->name().to_string().unchecked_access().c_str(),
                        StrEq(node_2.name().to_string().unchecked_access().c_str()));
        }
    };

    auto result = sut_1.nodes([&](auto node_state) -> CallbackProgression {
        node_state.alive(verify_node);

        node_state.dead([](const auto&) -> auto { ASSERT_TRUE(false); });
        node_state.inaccessible([](const auto&) -> auto { ASSERT_TRUE(false); });
        node_state.undefined([](const auto&) -> auto { ASSERT_TRUE(false); });

        return CallbackProgression::Continue;
    });

    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(counter, Eq(2));
}
//NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServiceBlackboardTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create()
                   .value();

    auto sut_2 = node.service_builder(service_name)
                     .template blackboard_creator<uint64_t>()
                     .template add_with_default<uint64_t>(0)
                     .create();

    ASSERT_FALSE(sut_2.has_value());
    ASSERT_THAT(sut_2.error(), Eq(BlackboardCreateError::AlreadyExists));
}

TYPED_TEST(ServiceBlackboardTest, creating_fails_when_no_key_value_pairs_are_provided) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).template blackboard_creator<uint64_t>().create();

    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::NoEntriesProvided));
}

TYPED_TEST(ServiceBlackboardTest, create_fails_when_same_key_is_provided_twice) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add<uint8_t>(0, 0)
                   .create();

    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::ServiceInCorruptedState));
}

TYPED_TEST(ServiceBlackboardTest, create_with_mixed_add_methods_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add_with_default<uint8_t>(1)
                   .create();

    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceBlackboardTest, create_fails_when_same_key_is_provided_twice_with_mixed_add_methods) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add_with_default<uint8_t>(0)
                   .create();

    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::ServiceInCorruptedState));
}

TYPED_TEST(ServiceBlackboardTest, recreating_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create();
        ASSERT_TRUE(sut.has_value());
    }

    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceBlackboardTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(BlackboardOpenError::DoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<uint64_t>()
                          .template add_with_default<uint64_t>(0)
                          .create()
                          .value();
    auto sut = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceBlackboardTest, opening_existing_service_with_wrong_key_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<uint64_t>()
                          .template add_with_default<uint64_t>(0)
                          .create()
                          .value();
    auto sut = node.service_builder(service_name).template blackboard_opener<double>().open();
    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(BlackboardOpenError::IncompatibleKeys));
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_service_does_not_satisfy_max_nodes_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    auto service_fail =
        node.service_builder(service_name).template blackboard_opener<uint64_t>().max_nodes(NUMBER_OF_NODES + 1).open();

    ASSERT_FALSE(service_fail.has_value());
    ASSERT_THAT(service_fail.error(), Eq(BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes));

    auto service_success =
        node.service_builder(service_name).template blackboard_opener<uint64_t>().max_nodes(NUMBER_OF_NODES - 1).open();

    ASSERT_TRUE(service_success.has_value());
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_service_does_not_satisfy_max_readers_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_READERS = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_readers(NUMBER_OF_READERS)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    auto service_fail = node.service_builder(service_name)
                            .template blackboard_opener<uint64_t>()
                            .max_readers(NUMBER_OF_READERS + 1)
                            .open();

    ASSERT_FALSE(service_fail.has_value());
    ASSERT_THAT(service_fail.error(), Eq(BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders));

    auto service_success = node.service_builder(service_name)
                               .template blackboard_opener<uint64_t>()
                               .max_readers(NUMBER_OF_READERS - 1)
                               .open();

    ASSERT_TRUE(service_success.has_value());
}

TYPED_TEST(ServiceBlackboardTest, open_works_when_service_owner_goes_out_of_scope) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_creator =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint64_t>(0)
                                                                        .create()
                                                                        .value());

    auto sut_opener_1 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut_opener_1.has_value());

    sut_creator.reset();

    auto sut_opener_2 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut_opener_2.has_value());
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_all_previous_owners_are_gone) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_creator =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint64_t>(0)
                                                                        .create()
                                                                        .value());

    auto sut_opener_1 = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value());

    sut_creator.reset();
    sut_opener_1.reset();

    auto sut_opener_2 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_FALSE(sut_opener_2.has_value());
    ASSERT_THAT(sut_opener_2.error(), Eq(BlackboardOpenError::DoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, properties_are_set_to_config_default) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    auto config = Config();

    ASSERT_THAT(service.static_config().max_readers(), Eq(config.defaults().blackboard().max_readers()));
    ASSERT_THAT(service.static_config().max_nodes(), Eq(config.defaults().blackboard().max_nodes()));
}

TYPED_TEST(ServiceBlackboardTest, open_uses_predefined_settings_when_nothing_is_specified) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<uint64_t>()
                          .template add_with_default<uint64_t>(0)
                          .max_nodes(2)
                          .max_readers(4)
                          .create()
                          .value();
    ASSERT_THAT(sut_create.static_config().max_readers(), Eq(4));
    ASSERT_THAT(sut_create.static_config().max_nodes(), Eq(2));

    auto sut_open = node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value();
    ASSERT_THAT(sut_open.static_config().max_readers(), Eq(4));
    ASSERT_THAT(sut_open.static_config().max_nodes(), Eq(2));
}

TYPED_TEST(ServiceBlackboardTest, setting_service_properties_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 10;
    constexpr uint64_t NUMBER_OF_READERS = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .max_readers(NUMBER_OF_READERS)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    auto static_config = service.static_config();

    ASSERT_THAT(static_config.max_nodes(), Eq(NUMBER_OF_NODES));
    ASSERT_THAT(static_config.max_readers(), Eq(NUMBER_OF_READERS));
    ASSERT_THAT(static_config.type_details().variant(), Eq(TypeVariant::FixedSize));
    ASSERT_THAT(static_config.type_details().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.type_details().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.type_details().type_name(), StrEq("u64"));
}

TYPED_TEST(ServiceBlackboardTest, type_information_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    using KeyType = uint64_t;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<KeyType>()
                       .template add_with_default<uint8_t>(0)
                       .create()
                       .value();

    auto details = service.static_config().type_details();
    ASSERT_THAT(details.variant(), Eq(TypeVariant::FixedSize));
    ASSERT_THAT(details.type_name(), StrEq("u64"));
    ASSERT_THAT(details.size(), Eq(sizeof(KeyType)));
    ASSERT_THAT(details.alignment(), Eq(alignof(KeyType)));
}

TYPED_TEST(ServiceBlackboardTest, number_of_readers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(0));

    {
        auto sut_reader = service.reader_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(1));
    }

    ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(0));
}

TYPED_TEST(ServiceBlackboardTest, number_of_writers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    ASSERT_THAT(service.dynamic_config().number_of_writers(), Eq(0));

    {
        auto sut_writer = service.writer_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_writers(), Eq(1));
    }

    ASSERT_THAT(service.dynamic_config().number_of_writers(), Eq(0));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_can_be_acquired_for_existing_key_value_pair) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint64_t>(0);
    ASSERT_TRUE(entry_handle.has_value());
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_cannot_be_acquired_for_non_existing_key) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint64_t>(1);
    ASSERT_FALSE(entry_handle.has_value());
    ASSERT_THAT(entry_handle.error(), Eq(EntryHandleError::EntryDoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_cannot_be_acquired_for_wrong_value_type) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint16_t>(0);
    ASSERT_FALSE(entry_handle.has_value());
    ASSERT_THAT(entry_handle.error(), Eq(EntryHandleError::EntryDoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, add_with_default_stores_default_value) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    struct TestDefault {
        // NOLINTNEXTLINE(misc-non-private-member-variables-in-classes), come on, its a test
        uint16_t t { 27 };
    };

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<TestDefault>(0)
                       .template add_with_default<uint16_t>(1)
                       .create()
                       .value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle_0 = reader.template entry<TestDefault>(0).value();
    ASSERT_THAT((*entry_handle_0.get()).t, Eq(27));
    auto entry_handle_1 = reader.template entry<uint16_t>(1).value();
    ASSERT_THAT(*entry_handle_1.get(), Eq(0));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_can_be_acquired_for_existing_key_value_pair) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle = writer.template entry<uint64_t>(0);
    ASSERT_TRUE(entry_handle.has_value());
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_cannot_be_acquired_for_non_existing_key) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint64_t>(1);
    ASSERT_FALSE(entry_handle_mut.has_value());
    ASSERT_THAT(entry_handle_mut.error(), Eq(EntryHandleMutError::EntryDoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_cannot_be_acquired_for_wrong_value_type) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint16_t>(0);
    ASSERT_FALSE(entry_handle_mut.has_value());
    ASSERT_THAT(entry_handle_mut.error(), Eq(EntryHandleMutError::EntryDoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_cannot_be_acquired_twice) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut =
        bb::Optional<EntryHandleMut<SERVICE_TYPE, uint64_t, uint64_t>>(writer.template entry<uint64_t>(0).value());

    auto sut_1 = writer.template entry<uint64_t>(0);
    ASSERT_FALSE(sut_1.has_value());
    ASSERT_THAT(sut_1.error(), Eq(EntryHandleMutError::HandleAlreadyExists));

    entry_handle_mut.reset();

    auto sut_2 = writer.template entry<uint64_t>(0);
    ASSERT_TRUE(sut_2.has_value());
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_prevents_another_writer) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service.writer_builder().create().value());
    auto entry_handle_mut = writer->template entry<uint64_t>(0).value();

    writer.reset();

    auto sut = service.writer_builder().create();
    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(WriterCreateError::ExceedsMaxSupportedWriters));
}

TYPED_TEST(ServiceBlackboardTest, entry_value_can_still_be_used_after_every_previous_service_state_owner_was_dropped) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint32_t>(0)
                                                                        .create()
                                                                        .value());
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service->writer_builder().create().value());
    auto entry_handle_mut = writer->template entry<uint32_t>(0).value();
    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));

    writer.reset();
    service.reset();

    auto new_entry_handle_mut = update_with_copy(std::move(entry_value_uninit), static_cast<uint32_t>(1));
}

TYPED_TEST(ServiceBlackboardTest, simple_communication_works_reader_created_first) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint16_t VALUE_1 = 1234;
    constexpr uint16_t VALUE_2 = 4567;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint16_t>(0)
                       .create()
                       .value();

    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint16_t>(0).value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint16_t>(0).value();

    entry_handle_mut.update_with_copy(VALUE_1);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_1));

    entry_handle_mut.update_with_copy(VALUE_2);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_2));
}

TYPED_TEST(ServiceBlackboardTest, simple_communication_works_writer_created_first) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr int32_t VALUE_1 = 50;
    constexpr int32_t VALUE_2 = -12;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<int32_t>(3, -3)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<int32_t>(3).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<int32_t>(3).value();

    entry_handle_mut.update_with_copy(VALUE_1);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_1));

    entry_handle_mut.update_with_copy(VALUE_2);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_2));
}

TYPED_TEST(ServiceBlackboardTest, communication_with_max_readers) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t MAX_READERS = 6;
    constexpr uint64_t NUMBER_OF_ITERATIONS = 128;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint64_t>(0).value();

    std::vector<Reader<SERVICE_TYPE, uint64_t>> readers;
    readers.reserve(MAX_READERS);

    for (uint64_t i = 0; i < MAX_READERS; ++i) {
        readers.push_back(service.reader_builder().create().value());
    }

    for (uint64_t counter = 0; counter < NUMBER_OF_ITERATIONS; ++counter) {
        entry_handle_mut.update_with_copy(counter);

        for (auto& reader : readers) {
            auto entry_handle = reader.template entry<uint64_t>(0).value();
            ASSERT_THAT(*entry_handle.get(), Eq(counter));
        }
    }
}

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers) only for testing purposes
TYPED_TEST(ServiceBlackboardTest, communication_with_max_reader_and_writer_handles) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t MAX_HANDLES = 6;
    constexpr uint64_t VALUE = 7;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<uint64_t>(0, 0)
                       .template add<uint64_t>(1, 1)
                       .template add<uint64_t>(2, 2)
                       .template add<uint64_t>(3, 3)
                       .template add<uint64_t>(4, 4)
                       .template add<uint64_t>(5, 5)
                       .template add<uint64_t>(6, 6)
                       .max_readers(MAX_HANDLES)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    std::vector<EntryHandleMut<SERVICE_TYPE, uint64_t, uint64_t>> entry_handles_mut;
    entry_handles_mut.reserve(MAX_HANDLES);

    auto reader = service.reader_builder().create().value();
    std::vector<EntryHandle<SERVICE_TYPE, uint64_t, uint64_t>> entry_handles;
    entry_handles.reserve(MAX_HANDLES);

    for (uint64_t i = 0; i < MAX_HANDLES; ++i) {
        entry_handles_mut.push_back(writer.template entry<uint64_t>(i).value());
        entry_handles.push_back(reader.template entry<uint64_t>(i).value());
    }

    for (uint64_t i = 0; i < MAX_HANDLES; ++i) {
        entry_handles_mut[i].update_with_copy(VALUE);
        for (uint64_t j = 0; j < i + 1; ++j) {
            ASSERT_THAT(*entry_handles[j].get(), Eq(VALUE));
        }
        for (uint64_t j = i + 1; j < MAX_HANDLES; ++j) {
            ASSERT_THAT(*entry_handles[j].get(), Eq(j));
        }
    }
}
// NOLINTEND(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers)

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers, readability-identifier-length) only for testing purposes
TYPED_TEST(ServiceBlackboardTest, write_and_read_different_value_types_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    struct Groovy {
        Groovy() = default;
        Groovy(bool a, uint32_t b, int64_t c)
            : m_a { a }
            , m_b { b }
            , m_c { c } {
        }

        auto operator==(const Groovy& rhs) const -> bool {
            return m_a == rhs.m_a && m_b == rhs.m_b && m_c == rhs.m_c;
        }

      private:
        bool m_a { false };
        uint32_t m_b { 0 };
        int64_t m_c { 0 };
    };

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<uint64_t>(0, 0)
                       .template add<int8_t>(1, -5)
                       .template add<bool>(100, false)
                       .template add<Groovy>(13, Groovy(true, 7127, 609))
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    writer.template entry<Groovy>(13).value().update_with_copy(Groovy(false, 888, 906));
    writer.template entry<bool>(100).value().update_with_copy(true);
    writer.template entry<int8_t>(1).value().update_with_copy(11);
    writer.template entry<uint64_t>(0).value().update_with_copy(2008);

    auto reader = service.reader_builder().create().value();
    ASSERT_THAT(*reader.template entry<uint64_t>(0).value().get(), Eq(2008));
    ASSERT_THAT(*reader.template entry<int8_t>(1).value().get(), Eq(11));
    ASSERT_THAT(*reader.template entry<bool>(100).value().get(), Eq(true));
    ASSERT_THAT(*reader.template entry<Groovy>(13).value().get(), Eq(Groovy(false, 888, 906)));
}
// NOLINTEND(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers, readability-identifier-length)

TYPED_TEST(ServiceBlackboardTest, creating_max_supported_amount_of_ports_work) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t MAX_READERS = 8;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint8_t>(0)
                       .max_readers(MAX_READERS)
                       .create()
                       .value();

    std::vector<Reader<SERVICE_TYPE, uint64_t>> readers;
    readers.reserve(MAX_READERS);

    // acquire all possible ports
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service.writer_builder().create().value());

    for (uint64_t i = 0; i < MAX_READERS; ++i) {
        readers.push_back(service.reader_builder().create().value());
    }

    // create additional ports and fail
    auto failing_writer = service.writer_builder().create();
    ASSERT_FALSE(failing_writer.has_value());
    ASSERT_THAT(failing_writer.error(), Eq(WriterCreateError::ExceedsMaxSupportedWriters));

    auto failing_reader = service.reader_builder().create();
    ASSERT_FALSE(failing_reader.has_value());
    ASSERT_THAT(failing_reader.error(), Eq(ReaderCreateError::ExceedsMaxSupportedReaders));

    // remove one reader and the writer
    writer.reset();
    readers.pop_back();

    // create additional ports shall work again
    auto new_writer = service.writer_builder().create();
    ASSERT_TRUE(new_writer.has_value());

    auto new_reader = service.reader_builder().create();
    ASSERT_TRUE(new_reader.has_value());
}

TYPED_TEST(ServiceBlackboardTest, set_max_nodes_to_zero_adjusts_it_to_one) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .max_nodes(0)
                   .create()
                   .value();

    ASSERT_THAT(sut.static_config().max_nodes(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, set_max_readers_to_zero_adjusts_it_to_one) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .max_readers(0)
                   .create()
                   .value();

    ASSERT_THAT(sut.static_config().max_readers(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, dropping_service_keeps_established_communication) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                               .template blackboard_creator<uint64_t>()
                                                                               .template add_with_default<uint32_t>(0)
                                                                               .create()
                                                                               .value());

    auto writer = sut->writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint32_t>(0).value();
    auto reader = sut->reader_builder().create().value();
    auto entry_handle = reader.template entry<uint32_t>(0).value();

    sut.reset();

    constexpr uint32_t VALUE = 981293;
    entry_handle_mut.update_with_copy(VALUE);
    ASSERT_THAT(*entry_handle.get(), VALUE);
}

TYPED_TEST(ServiceBlackboardTest, ports_of_dropped_service_block_new_service_creation) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint8_t>(0)
                                                                        .create()
                                                                        .value());

    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service->writer_builder().create().value());
    auto reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(service->reader_builder().create().value());

    service.reset();

    auto sut1 = node.service_builder(service_name)
                    .template blackboard_creator<uint64_t>()
                    .template add_with_default<uint8_t>(0)
                    .create();
    ASSERT_FALSE(sut1.has_value());
    ASSERT_THAT(sut1.error(), Eq(BlackboardCreateError::AlreadyExists));

    reader.reset();

    auto sut2 = node.service_builder(service_name)
                    .template blackboard_creator<uint64_t>()
                    .template add_with_default<uint8_t>(0)
                    .create();
    ASSERT_FALSE(sut2.has_value());
    ASSERT_THAT(sut2.error(), Eq(BlackboardCreateError::AlreadyExists));

    writer.reset();

    auto sut3 = node.service_builder(service_name)
                    .template blackboard_creator<uint64_t>()
                    .template add_with_default<uint8_t>(0)
                    .create();
    ASSERT_TRUE(sut3.has_value());
}

TYPED_TEST(ServiceBlackboardTest, service_can_be_opened_when_there_is_a_writer) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t VALUE = 1809723987;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto creator =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint64_t>(0)
                                                                        .create()
                                                                        .value());
    auto reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(creator->reader_builder().create().value());
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(creator->writer_builder().create().value());
    auto entry_handle_mut =
        bb::Optional<EntryHandleMut<SERVICE_TYPE, uint64_t, uint64_t>>(writer->template entry<uint64_t>(0).value());

    creator.reset();

    auto opener1 = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value());
    opener1.reset();

    auto failing_creator = node.service_builder(service_name)
                               .template blackboard_creator<uint64_t>()
                               .template add_with_default<uint64_t>(0)
                               .create();
    ASSERT_FALSE(failing_creator.has_value());
    ASSERT_THAT(failing_creator.error(), Eq(BlackboardCreateError::AlreadyExists));
    reader.reset();

    auto opener2 = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value());
    auto opener_reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(opener2->reader_builder().create().value());
    auto entry_handle =
        bb::Optional<EntryHandle<SERVICE_TYPE, uint64_t, uint64_t>>(opener_reader->template entry<uint64_t>(0).value());
    entry_handle_mut->update_with_copy(VALUE);
    ASSERT_THAT(*entry_handle->get(), Eq(VALUE));

    entry_handle.reset();
    opener_reader.reset();
    opener2.reset();
    entry_handle_mut.reset();
    writer.reset();

    auto failing_opener = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_FALSE(failing_opener.has_value());
    ASSERT_THAT(failing_opener.error(), Eq(BlackboardOpenError::DoesNotExist));
    auto new_creator = node.service_builder(service_name)
                           .template blackboard_creator<uint64_t>()
                           .template add_with_default<uint64_t>(0)
                           .create();
    ASSERT_TRUE(new_creator.has_value());
}

TYPED_TEST(ServiceBlackboardTest, service_can_be_opened_when_there_is_a_reader) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t VALUE = 325183783;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto creator =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint64_t>(0)
                                                                        .create()
                                                                        .value());
    auto reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(creator->reader_builder().create().value());
    auto entry_handle =
        bb::Optional<EntryHandle<SERVICE_TYPE, uint64_t, uint64_t>>(reader->template entry<uint64_t>(0).value());
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(creator->writer_builder().create().value());

    creator.reset();

    auto opener1 = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value());
    opener1.reset();

    auto failing_creator = node.service_builder(service_name)
                               .template blackboard_creator<uint64_t>()
                               .template add_with_default<uint64_t>(0)
                               .create();
    ASSERT_FALSE(failing_creator.has_value());
    ASSERT_THAT(failing_creator.error(), Eq(BlackboardCreateError::AlreadyExists));
    writer.reset();

    auto opener2 = bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value());
    auto opener_writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(opener2->writer_builder().create().value());
    auto entry_handle_mut = bb::Optional<EntryHandleMut<SERVICE_TYPE, uint64_t, uint64_t>>(
        opener_writer->template entry<uint64_t>(0).value());
    entry_handle_mut->update_with_copy(VALUE);
    ASSERT_THAT(*entry_handle->get(), Eq(VALUE));

    entry_handle_mut.reset();
    opener_writer.reset();
    opener2.reset();
    entry_handle.reset();
    reader.reset();

    auto failing_opener = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_FALSE(failing_opener.has_value());
    ASSERT_THAT(failing_opener.error(), Eq(BlackboardOpenError::DoesNotExist));
    auto new_creator = node.service_builder(service_name)
                           .template blackboard_creator<uint64_t>()
                           .template add_with_default<uint64_t>(0)
                           .create();
    ASSERT_TRUE(new_creator.has_value());
}

TYPED_TEST(ServiceBlackboardTest, reader_can_still_read_value_when_writer_was_disconnected) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t VALUE = 5;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint8_t>(0)
                       .create()
                       .value();

    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service.writer_builder().create().value());
    auto entry_handle_mut =
        bb::Optional<EntryHandleMut<SERVICE_TYPE, uint64_t, uint8_t>>(writer->template entry<uint8_t>(0).value());
    entry_handle_mut->update_with_copy(VALUE);
    entry_handle_mut.reset();
    writer.reset();

    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint8_t>(0).value();
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE));
}

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers)
TYPED_TEST(ServiceBlackboardTest, reconnected_reader_sees_current_blackboard_status) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<uint8_t>(0, 0)
                       .template add<int32_t>(6, -9)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut_key_0 = writer.template entry<uint8_t>(0).value();
    entry_handle_mut_key_0.update_with_copy(5);

    auto reader_1 = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(service.reader_builder().create().value());
    ASSERT_THAT(*reader_1->template entry<uint8_t>(0).value().get(), Eq(5));
    ASSERT_THAT(*reader_1->template entry<int32_t>(6).value().get(), Eq(-9));

    reader_1.reset();

    auto entry_handle_mut_key_6 = writer.template entry<int32_t>(6).value();
    entry_handle_mut_key_6.update_with_copy(-567);

    auto reader_2 = service.reader_builder().create().value();
    ASSERT_THAT(*reader_2.template entry<uint8_t>(0).value().get(), Eq(5));
    ASSERT_THAT(*reader_2.template entry<int32_t>(6).value().get(), Eq(-567));
}
// NOLINTEND(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers)

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_can_still_write_after_writer_was_dropped) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint8_t>(0)
                       .create()
                       .value();
    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service.writer_builder().create().value());
    auto entry_handle_mut = writer->template entry<uint8_t>(0).value();

    writer.reset();
    entry_handle_mut.update_with_copy(1);

    auto reader = service.reader_builder().create().value();
    ASSERT_THAT(*reader.template entry<uint8_t>(0).value().get(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_can_still_read_after_reader_was_dropped) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint8_t>(0)
                       .create()
                       .value();
    auto reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(service.reader_builder().create().value());
    auto entry_handle = reader->template entry<uint8_t>(0).value();

    reader.reset();
    ASSERT_THAT(*entry_handle.get(), Eq(0));

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint8_t>(0).value();
    entry_handle_mut.update_with_copy(1);
    ASSERT_THAT(*entry_handle.get(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, loan_and_write_entry_value_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t VALUE = 333;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint64_t>(0).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint64_t>(0).value();

    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));
    auto new_entry_handle_mut = update_with_copy(std::move(entry_value_uninit), VALUE);

    ASSERT_THAT(*entry_handle.get(), Eq(VALUE));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_can_be_reused_after_entry_value_was_updated) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint32_t VALUE1 = 333;
    constexpr uint32_t VALUE2 = 999;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint32_t>(0)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint32_t>(0).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint32_t>(0).value();

    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));
    auto new_entry_handle_mut = update_with_copy(std::move(entry_value_uninit), VALUE1);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE1));

    new_entry_handle_mut.update_with_copy(VALUE2);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE2));
}

TYPED_TEST(ServiceBlackboardTest, entry_value_can_still_be_used_after_writer_was_dropped) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint32_t VALUE = 333;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint32_t>(0)
                       .create()
                       .value();

    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service.writer_builder().create().value());
    auto entry_handle_mut = writer->template entry<uint32_t>(0).value();
    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));

    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint32_t>(0).value();

    writer.reset();

    auto new_entry_handle_mut = update_with_copy(std::move(entry_value_uninit), VALUE);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_mut_can_be_reused_after_entry_value_uninit_was_discarded) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint32_t>(0)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint32_t>(0).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint32_t>(0).value();

    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));

    auto sut = discard(std::move(entry_value_uninit));
    sut.update_with_copy(1);
    ASSERT_THAT(*entry_handle.get(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_can_still_be_used_after_every_previous_service_state_owner_was_dropped) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint32_t>(0)
                                                                        .create()
                                                                        .value());

    auto writer = bb::Optional<Writer<SERVICE_TYPE, uint64_t>>(service->writer_builder().create().value());
    auto entry_handle_mut =
        bb::Optional<EntryHandleMut<SERVICE_TYPE, uint64_t, uint32_t>>(writer->template entry<uint32_t>(0).value());

    writer.reset();
    service.reset();

    entry_handle_mut->update_with_copy(3);
    entry_handle_mut.reset();

    auto new_service =
        bb::Optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                        .template blackboard_creator<uint64_t>()
                                                                        .template add_with_default<uint32_t>(0)
                                                                        .create()
                                                                        .value());

    auto reader = bb::Optional<Reader<SERVICE_TYPE, uint64_t>>(new_service->reader_builder().create().value());
    auto entry_handle =
        bb::Optional<EntryHandle<SERVICE_TYPE, uint64_t, uint32_t>>(reader->template entry<uint32_t>(0).value());

    reader.reset();
    new_service.reset();

    ASSERT_THAT(*entry_handle->get(), Eq(0));
}

TYPED_TEST(ServiceBlackboardTest, listing_all_readers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_READERS = 18;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .max_readers(NUMBER_OF_READERS)
                       .create()
                       .value();

    std::vector<Reader<SERVICE_TYPE, uint64_t>> readers;
    readers.reserve(NUMBER_OF_READERS);
    for (uint64_t i = 0; i < NUMBER_OF_READERS; ++i) {
        readers.push_back(service.reader_builder().create().value());
    }

    std::vector<UniqueReaderId> reader_ids;
    reader_ids.reserve(NUMBER_OF_READERS);
    service.dynamic_config().list_readers([&](auto reader_details_view) -> auto {
        reader_ids.push_back(reader_details_view.reader_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(reader_ids.size(), Eq(NUMBER_OF_READERS));
    for (auto& reader : readers) {
        auto iter = std::find(reader_ids.begin(), reader_ids.end(), reader.id());
        ASSERT_THAT(iter, Ne(reader_ids.end()));
    }
}

TYPED_TEST(ServiceBlackboardTest, listing_all_readers_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_READERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .max_readers(NUMBER_OF_READERS)
                   .create()
                   .value();

    std::vector<iox2::Reader<SERVICE_TYPE, uint64_t>> readers;
    readers.reserve(NUMBER_OF_READERS);
    for (uint64_t i = 0; i < NUMBER_OF_READERS; ++i) {
        readers.push_back(sut.reader_builder().create().value());
    }

    auto counter = 0;
    sut.dynamic_config().list_readers([&](auto) -> auto {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = *Attribute::Key::from_utf8("want to make your machine run faster:");
    auto value = *Attribute::Value::from_utf8("sudo rm -rf /");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value).value();
    auto service_create = node.service_builder(service_name)
                              .template blackboard_creator<uint64_t>()
                              .template add_with_default<uint64_t>(0)
                              .create_with_attributes(attribute_specifier)
                              .value();

    auto service_open = node.service_builder(service_name).template blackboard_opener<uint64_t>().open().value();


    auto attributes_create = service_create.attributes();
    auto attributes_open = service_open.attributes();

    ASSERT_THAT(attributes_create.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_create[0].key(), Eq(key));
    ASSERT_THAT(attributes_create[0].value(), Eq(value));

    ASSERT_THAT(attributes_open.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_open[0].key(), Eq(key));
    ASSERT_THAT(attributes_open[0].value(), Eq(value));
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_attributes_are_incompatible) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = *Attribute::Key::from_utf8("whats hypnotoad doing these days?");
    auto value = *Attribute::Value::from_utf8("eating hypnoflies?");
    auto missing_key = *Attribute::Key::from_utf8("no he is singing a song!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value).value();
    auto service_create = node.service_builder(service_name)
                              .template blackboard_creator<uint64_t>()
                              .template add_with_default<uint64_t>(0)
                              .create_with_attributes(attribute_specifier)
                              .value();

    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();
    attribute_verifier.require_key(missing_key).value();
    auto service_open = node.service_builder(service_name)
                            .template blackboard_opener<uint64_t>()
                            .open_with_attributes(attribute_verifier);

    ASSERT_THAT(service_open.has_value(), Eq(false));
    ASSERT_THAT(service_open.error(), Eq(BlackboardOpenError::IncompatibleAttributes));
}

TYPED_TEST(ServiceBlackboardTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service_1_create = node.service_builder(service_name_1)
                                .template blackboard_creator<uint64_t>()
                                .template add_with_default<uint64_t>(0)
                                .create()
                                .value();
    auto service_1_open = node.service_builder(service_name_1).template blackboard_opener<uint64_t>().open().value();
    auto service_2 = node.service_builder(service_name_2)
                         .template blackboard_creator<uint64_t>()
                         .template add_with_default<uint64_t>(0)
                         .create()
                         .value();

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}

TYPED_TEST(ServiceBlackboardTest, reader_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create()
                   .value();

    auto reader = sut.reader_builder().create().value();

    auto counter = 0;
    sut.dynamic_config().list_readers([&](auto reader_details_view) -> auto {
        counter++;
        EXPECT_TRUE(reader_details_view.reader_id() == reader.id());
        EXPECT_TRUE(reader_details_view.node_id() == node.id());
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, same_entry_id_for_same_key) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .template add_with_default<uint64_t>(1)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint64_t>(0).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle_0 = reader.template entry<uint64_t>(0).value();
    auto entry_handle_1 = reader.template entry<uint64_t>(1).value();

    ASSERT_EQ(entry_handle_mut.entry_id(), entry_handle_0.entry_id());
    ASSERT_NE(entry_handle_0.entry_id(), entry_handle_1.entry_id());
}

TYPED_TEST(ServiceBlackboardTest, entry_handle_is_up_to_date_works_correctly) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<uint16_t>(0, 0)
                       .create()
                       .value();

    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint16_t>(0).value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint16_t>(0).value();

    auto value = entry_handle.get();
    ASSERT_EQ(*value, 0);
    ASSERT_TRUE(entry_handle.is_up_to_date(value));

    entry_handle_mut.update_with_copy(1);
    ASSERT_FALSE(entry_handle.is_up_to_date(value));
    value = entry_handle.get();
    ASSERT_EQ(*value, 1);
    ASSERT_TRUE(entry_handle.is_up_to_date(value));

    entry_handle_mut.update_with_copy(4);
    value = entry_handle.get();
    ASSERT_EQ(*value, 4);
    ASSERT_TRUE(entry_handle.is_up_to_date(value));
}

TYPED_TEST(ServiceBlackboardTest, list_keys_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    std::vector<uint64_t> keys { 0, 1, 2, 3, 4 };
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add<uint64_t>(keys[0], 0)
                       .template add<uint64_t>(keys[1], 0)
                       .template add<uint64_t>(keys[2], 0)
                       .template add<uint64_t>(keys[3], 0)
                       .template add<uint64_t>(keys[4], 0)
                       .create()
                       .value();

    std::vector<uint64_t> listed_keys;
    service.list_keys([&listed_keys](uint64_t key) -> auto {
        listed_keys.push_back(key);
        return CallbackProgression::Continue;
    });
    ASSERT_EQ(listed_keys.size(), keys.size());
    for (auto& key : keys) {
        ASSERT_TRUE(std::find(listed_keys.begin(), listed_keys.end(), key) != listed_keys.end());
    }

    listed_keys.clear();

    service.list_keys([&listed_keys](uint64_t key) -> auto {
        listed_keys.push_back(key);
        return CallbackProgression::Stop;
    });
    ASSERT_EQ(listed_keys.size(), 1);
    ASSERT_TRUE(std::find(keys.begin(), keys.end(), listed_keys[0]) != keys.end());
}

constexpr uint64_t const STRING_CAPACITY = 25;
struct Foo {
    Foo() = default;
    // NOLINTNEXTLINE(readability-identifier-length), come on, its a test
    Foo(uint32_t a, int16_t b, uint8_t c, const bb::StaticString<STRING_CAPACITY>& d)
        : m_a { a }
        , m_b { b }
        , m_c { c }
        , m_d { d } {
    }

    auto operator==(const Foo& rhs) const -> bool {
        return m_a == rhs.m_a && m_b == rhs.m_b && m_c == rhs.m_c && m_d == rhs.m_d;
    }

  private:
    uint32_t m_a { 0 };
    int16_t m_b { 0 };
    uint8_t m_c { 0 };
    bb::StaticString<STRING_CAPACITY> m_d;
};

TYPED_TEST(ServiceBlackboardTest, simple_communication_with_key_struct_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr int32_t VALUE_1 = 50;
    constexpr int32_t VALUE_2 = -12;

    auto key_1 = Foo(2, -3, 0, bb::StaticString<STRING_CAPACITY>::from_utf8("hatschu").value());
    auto key_2 = Foo(2, -3, 0, bb::StaticString<STRING_CAPACITY>::from_utf8("hatschuu").value());

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<Foo>()
                       .template add<int32_t>(key_1, -3)
                       .template add<int32_t>(key_2, 3)
                       .create()
                       .value();

    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut_1 = writer.template entry<int32_t>(key_1).value();
    auto entry_handle_mut_2 = writer.template entry<int32_t>(key_2).value();
    auto reader = service.reader_builder().create().value();
    auto entry_handle_1 = reader.template entry<int32_t>(key_1).value();
    auto entry_handle_2 = reader.template entry<int32_t>(key_2).value();

    ASSERT_THAT(*entry_handle_1.get(), Eq(-3));
    ASSERT_THAT(*entry_handle_2.get(), Eq(3));

    entry_handle_mut_1.update_with_copy((VALUE_1));
    ASSERT_THAT(*entry_handle_1.get(), Eq(VALUE_1));
    ASSERT_THAT(*entry_handle_2.get(), Eq(3));

    entry_handle_mut_2.update_with_copy(VALUE_2);
    ASSERT_THAT(*entry_handle_1.get(), Eq(VALUE_1));
    ASSERT_THAT(*entry_handle_2.get(), Eq(VALUE_2));
}

TYPED_TEST(ServiceBlackboardTest, adding_key_struct_twice_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Foo(2, -3, 0, bb::StaticString<STRING_CAPACITY>::from_utf8("huiuiui").value());

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<Foo>()
                       .template add<int32_t>(key, -3)
                       .template add<uint32_t>(key, 3)
                       .create();
    ASSERT_FALSE(service.has_value());
    ASSERT_THAT(service.error(), Eq(BlackboardCreateError::ServiceInCorruptedState));
}

TYPED_TEST(ServiceBlackboardTest, list_keys_with_key_struct_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    std::vector<Foo> keys { Foo(2, -3, 0, bb::StaticString<STRING_CAPACITY>::from_utf8("hatschu").value()),
                            Foo(2, -3, 0, bb::StaticString<STRING_CAPACITY>::from_utf8("hatschuu").value()) };
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<Foo>()
                       .template add<int32_t>(keys[0], -3)
                       .template add<uint32_t>(keys[1], 3)
                       .create()
                       .value();

    std::vector<Foo> listed_keys;
    service.list_keys([&listed_keys](Foo key) -> auto {
        listed_keys.push_back(key);
        return CallbackProgression::Continue;
    });
    ASSERT_EQ(listed_keys.size(), keys.size());
    for (auto& key : keys) {
        ASSERT_TRUE(std::find(listed_keys.begin(), listed_keys.end(), key) != listed_keys.end());
    }

    listed_keys.clear();

    service.list_keys([&listed_keys](Foo key) -> auto {
        listed_keys.push_back(key);
        return CallbackProgression::Stop;
    });
    ASSERT_EQ(listed_keys.size(), 1);
    ASSERT_TRUE(std::find(keys.begin(), keys.end(), listed_keys[0]) != keys.end());
}

TYPED_TEST(ServiceBlackboardTest, new_value_can_be_written_using_value_mut) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint16_t VALUE_1 = 1234;
    constexpr uint16_t VALUE_2 = 4321;
    constexpr uint16_t VALUE_3 = 4567;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint16_t>(0)
                       .create()
                       .value();

    auto reader = service.reader_builder().create().value();
    auto entry_handle = reader.template entry<uint16_t>(0).value();
    auto writer = service.writer_builder().create().value();
    auto entry_handle_mut = writer.template entry<uint16_t>(0).value();
    auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut));

    entry_value_uninit.value_mut() = VALUE_1;
    entry_handle_mut = assume_init_and_update(std::move(entry_value_uninit));
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_1));

    entry_value_uninit = loan_uninit(std::move(entry_handle_mut));
    entry_value_uninit.value_mut() = VALUE_2;
    // before calling assume_init_and_update(), the old value is read
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_1));
    entry_handle_mut = discard(std::move(entry_value_uninit));

    entry_handle_mut.update_with_copy(VALUE_3);
    ASSERT_THAT(*entry_handle.get(), Eq(VALUE_3));
}
} // namespace
