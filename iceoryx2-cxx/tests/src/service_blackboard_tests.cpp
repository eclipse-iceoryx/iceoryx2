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

#include "iox2/log.hpp"
#include "iox2/node.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

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

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .expect("");

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceBlackboardTest, service_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create()
                   .expect("");

    ASSERT_THAT(sut.name().to_string().c_str(), StrEq(service_name.to_string().c_str()));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceBlackboardTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("nala is hungry").expect("");
    const auto node_name_2 = NodeName::create("maybe octo-wolf can help?").expect("");
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().expect("");
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().expect("");

    auto sut_1 = node_1.service_builder(service_name)
                     .template blackboard_creator<uint64_t>()
                     .template add_with_default<uint64_t>(0)
                     .create()
                     .expect("");
    auto sut_2 = node_2.service_builder(service_name).template blackboard_opener<uint64_t>().open().expect("");

    auto counter = 0;
    auto verify_node = [&](const AliveNodeView<SERVICE_TYPE>& node_view) {
        counter++;
        if (node_view.id() == node_1.id()) {
            ASSERT_THAT(node_view.details()->name().to_string().c_str(), StrEq(node_1.name().to_string().c_str()));
        } else {
            ASSERT_THAT(node_view.details()->name().to_string().c_str(), StrEq(node_2.name().to_string().c_str()));
        }
    };

    auto result = sut_1.nodes([&](auto node_state) -> CallbackProgression {
        node_state.alive(verify_node);

        node_state.dead([](const auto&) { ASSERT_TRUE(false); });
        node_state.inaccessible([](const auto&) { ASSERT_TRUE(false); });
        node_state.undefined([](const auto&) { ASSERT_TRUE(false); });

        return CallbackProgression::Continue;
    });

    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(counter, Eq(2));
}
//NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServiceBlackboardTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create()
                   .expect("");
    auto sut_2 = node.service_builder(service_name)
                     .template blackboard_creator<uint64_t>()
                     .template add_with_default<uint64_t>(0)
                     .create();

    ASSERT_TRUE(sut_2.has_error());
    ASSERT_THAT(sut_2.error(), Eq(BlackboardCreateError::AlreadyExists));
}

TYPED_TEST(ServiceBlackboardTest, creating_fails_when_no_key_value_pairs_are_provided) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template blackboard_creator<uint64_t>().create();

    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::NoEntriesProvided));
}

TYPED_TEST(ServiceBlackboardTest, create_fails_when_same_key_is_provided_twice) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add<uint8_t>(0, 0)
                   .create();

    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::ServiceInCorruptedState));
}

TYPED_TEST(ServiceBlackboardTest, create_with_mixed_add_methods_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add_with_default<uint8_t>(1)
                   .create();

    ASSERT_FALSE(sut.has_error());
}

TYPED_TEST(ServiceBlackboardTest, create_fails_when_same_key_is_provided_twice_with_mixed_add_methods) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add<uint8_t>(0, 0)
                   .template add_with_default<uint8_t>(0)
                   .create();

    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(BlackboardCreateError::ServiceInCorruptedState));
}

TYPED_TEST(ServiceBlackboardTest, recreating_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Blackboard)
                     .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create();
        ASSERT_FALSE(sut.has_error());
    }

    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .create();
    ASSERT_FALSE(sut.has_error());
}

TYPED_TEST(ServiceBlackboardTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(BlackboardOpenError::DoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<uint64_t>()
                          .template add_with_default<uint64_t>(0)
                          .create()
                          .expect("");
    auto sut = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

// TODO: enable when key type is generic
// TYPED_TEST(ServiceBlackboardTest, opening_existing_service_with_wrong_key_type_fails) {
// constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

// const auto service_name = iox2_testing::generate_service_name();

// auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
// auto sut_create = node.service_builder(service_name)
//.template blackboard_creator<uint64_t>()
//.template add_with_default<uint64_t>(0)
//.create()
//.expect("");
// auto sut = node.service_builder(service_name).template blackboard_opener<double>().open();
// ASSERT_TRUE(sut.has_error());
// ASSERT_THAT(sut.error(), Eq(BlackboardOpenError::IncompatibleKeys));
//}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_service_does_not_satisfy_max_nodes_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .expect("");

    auto service_fail =
        node.service_builder(service_name).template blackboard_opener<uint64_t>().max_nodes(NUMBER_OF_NODES + 1).open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes));

    auto service_success =
        node.service_builder(service_name).template blackboard_opener<uint64_t>().max_nodes(NUMBER_OF_NODES - 1).open();

    ASSERT_FALSE(service_success.has_error());
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_service_does_not_satisfy_max_readers_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_READERS = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_readers(NUMBER_OF_READERS)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .expect("");

    auto service_fail = node.service_builder(service_name)
                            .template blackboard_opener<uint64_t>()
                            .max_readers(NUMBER_OF_READERS + 1)
                            .open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders));

    auto service_success = node.service_builder(service_name)
                               .template blackboard_opener<uint64_t>()
                               .max_readers(NUMBER_OF_READERS - 1)
                               .open();

    ASSERT_FALSE(service_success.has_error());
}

TYPED_TEST(ServiceBlackboardTest, open_works_when_service_owner_goes_out_of_scope) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_creator =
        iox::optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                         .template blackboard_creator<uint64_t>()
                                                                         .template add_with_default<uint64_t>(0)
                                                                         .create()
                                                                         .expect(""));

    auto sut_opener_1 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut_opener_1.has_value());

    sut_creator.reset();

    auto sut_opener_2 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut_opener_2.has_value());
}

TYPED_TEST(ServiceBlackboardTest, open_fails_when_all_previous_owners_are_gone) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_creator =
        iox::optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(node.service_builder(service_name)
                                                                         .template blackboard_creator<uint64_t>()
                                                                         .template add_with_default<uint64_t>(0)
                                                                         .create()
                                                                         .expect(""));

    auto sut_opener_1 = iox::optional<PortFactoryBlackboard<SERVICE_TYPE, uint64_t>>(
        node.service_builder(service_name).template blackboard_opener<uint64_t>().open().expect(""));

    sut_creator.reset();
    sut_opener_1.reset();

    auto sut_opener_2 = node.service_builder(service_name).template blackboard_opener<uint64_t>().open();
    ASSERT_TRUE(sut_opener_2.has_error());
    ASSERT_THAT(sut_opener_2.error(), Eq(BlackboardOpenError::DoesNotExist));
}

TYPED_TEST(ServiceBlackboardTest, properties_are_set_to_config_default) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .expect("");

    auto config = Config();

    ASSERT_THAT(service.static_config().max_readers(), Eq(config.defaults().blackboard().max_readers()));
    ASSERT_THAT(service.static_config().max_nodes(), Eq(config.defaults().blackboard().max_nodes()));
}

TYPED_TEST(ServiceBlackboardTest, open_uses_predefined_settings_when_nothing_is_specified) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template blackboard_creator<uint64_t>()
                          .template add_with_default<uint64_t>(0)
                          .max_nodes(2)
                          .max_readers(4)
                          .create()
                          .expect("");
    ASSERT_THAT(sut_create.static_config().max_readers(), Eq(4));
    ASSERT_THAT(sut_create.static_config().max_nodes(), Eq(2));

    auto sut_open = node.service_builder(service_name).template blackboard_opener<uint64_t>().open().expect("");
    ASSERT_THAT(sut_open.static_config().max_readers(), Eq(4));
    ASSERT_THAT(sut_open.static_config().max_nodes(), Eq(2));
}

TYPED_TEST(ServiceBlackboardTest, setting_service_properties_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 10;
    constexpr uint64_t NUMBER_OF_READERS = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template blackboard_creator<uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .max_readers(NUMBER_OF_READERS)
                       .template add_with_default<uint64_t>(0)
                       .create()
                       .expect("");

    auto static_config = service.static_config();

    ASSERT_THAT(static_config.max_nodes(), Eq(NUMBER_OF_NODES));
    ASSERT_THAT(static_config.max_readers(), Eq(NUMBER_OF_READERS));
    ASSERT_THAT(static_config.type_details().variant(), Eq(TypeVariant::FixedSize));
    ASSERT_THAT(static_config.type_details().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.type_details().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.type_details().type_name(), StrEq("u64"));
}

// TODO: adapt the test below when key type is generic
// #[test]
// fn type_information_are_correct<Sut: Service>() {
// type KeyType = u64;
// let config = generate_isolated_config();
// let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

// let service_name = generate_name();

// let sut = node
//.service_builder(&service_name)
//.blackboard_creator::<KeyType>()
//.add::<u8>(0, 0)
//.create()
//.unwrap();

// let d = sut.static_config().type_details();
// assert_that!(d.variant, eq TypeVariant::FixedSize);
// assert_that!(d.type_name, eq core::any::type_name::<KeyType>());
// assert_that!(d.size, eq core::mem::size_of::<KeyType>());
// assert_that!(d.alignment, eq core::mem::align_of::<KeyType>());
//}

// TYPED_TEST(ServiceBlackboardTest, number_of_readers_works) {
// constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

// const auto service_name = iox2_testing::generate_service_name();

// auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
// auto service = node.service_builder(service_name)
//.template blackboard_creator<uint64_t>()
//.template add_with_default<uint64_t>(0)
//.create()
//.expect("");

// ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(0));

//{
// auto sut_reader = service.reader_builder().create().expect("");
// ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(1));
//}

// ASSERT_THAT(service.dynamic_config().number_of_readers(), Eq(0));
//}

// TODO
TYPED_TEST(ServiceBlackboardTest, add_with_default_stores_default_value) {
}

TYPED_TEST(ServiceBlackboardTest, simple_communication_works_reader_created_first) {
}

TYPED_TEST(ServiceBlackboardTest, simple_communication_works_writer_created_first) {
}

TYPED_TEST(ServiceBlackboardTest, communication_with_max_readers) {
}

TYPED_TEST(ServiceBlackboardTest, communication_with_max_reader_and_writer_handles) {
}

TYPED_TEST(ServiceBlackboardTest, write_and_read_different_value_types_works) {
}

TYPED_TEST(ServiceBlackboardTest, creating_max_supported_amount_of_ports_work) {
}

TYPED_TEST(ServiceBlackboardTest, set_max_nodes_to_zero_adjusts_it_to_one) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .max_nodes(0)
                   .create()
                   .expect("");

    ASSERT_THAT(sut.static_config().max_nodes(), Eq(1));
}

TYPED_TEST(ServiceBlackboardTest, set_max_readers_to_zero_adjusts_it_to_one) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template blackboard_creator<uint64_t>()
                   .template add_with_default<uint64_t>(0)
                   .max_readers(0)
                   .create()
                   .expect("");

    ASSERT_THAT(sut.static_config().max_readers(), Eq(1));
}

// TODO
TYPED_TEST(ServiceBlackboardTest, dropping_service_keeps_established_communication) {
}

TYPED_TEST(ServiceBlackboardTest, ports_of_dropped_service_block_new_service_creation) {
}

TYPED_TEST(ServiceBlackboardTest, service_can_be_opened_when_there_is_a_writer) {
}

TYPED_TEST(ServiceBlackboardTest, service_can_be_opened_when_there_is_a_reader) {
}

TYPED_TEST(ServiceBlackboardTest, reader_can_stil_read_payload_when_writer_was_disconnected) {
}

TYPED_TEST(ServiceBlackboardTest, reconnected_reader_sees_current_blackboard_status) {
}

TYPED_TEST(ServiceBlackboardTest, writer_handle_can_still_write_after_writer_was_dropped) {
}

TYPED_TEST(ServiceBlackboardTest, reader_handle_can_still_read_after_reader_was_dropped) {
}

TYPED_TEST(ServiceBlackboardTest, loan_and_write_entry_value_works) {
}

TYPED_TEST(ServiceBlackboardTest, writer_handle_can_be_reused_after_entry_value_was_updated) {
}

TYPED_TEST(ServiceBlackboardTest, entry_value_can_still_be_used_after_writer_was_dropped) {
}

TYPED_TEST(ServiceBlackboardTest, writer_handle_can_be_reused_after_entry_value_uninit_was_discarded) {
}

TYPED_TEST(ServiceBlackboardTest, writer_handle_can_be_reused_after_entry_value_was_discarded) {
}

TYPED_TEST(ServiceBlackboardTest, handle_can_still_be_used_after_every_prvious_service_state_owner_was_dropped) {
}

TYPED_TEST(ServiceBlackboardTest, listing_all_readers_works) {
}

TYPED_TEST(ServiceBlackboardTest, listing_all_readers_stops_on_request) {
}

TYPED_TEST(ServiceBlackboardTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Attribute::Key("want to make your machine run faster:");
    auto value = Attribute::Value("sudo rm -rf /");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template blackboard_creator<uint64_t>()
                              .template add_with_default<uint64_t>(0)
                              .create_with_attributes(AttributeSpecifier().define(key, value))
                              .expect("");

    auto service_open = node.service_builder(service_name).template blackboard_opener<uint64_t>().open().expect("");


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

    auto key = Attribute::Key("whats hypnotoad doing these days?");
    auto value = Attribute::Value("eating hypnoflies?");
    auto missing_key = Attribute::Key("no he is singing a song!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template blackboard_creator<uint64_t>()
                              .template add_with_default<uint64_t>(0)
                              .create_with_attributes(AttributeSpecifier().define(key, value))
                              .expect("");

    auto service_open = node.service_builder(service_name)
                            .template blackboard_opener<uint64_t>()
                            .open_with_attributes(AttributeVerifier().require(key, value).require_key(missing_key));

    ASSERT_THAT(service_open.has_error(), Eq(true));
    ASSERT_THAT(service_open.error(), Eq(BlackboardOpenError::IncompatibleAttributes));
}

TYPED_TEST(ServiceBlackboardTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto service_1_create = node.service_builder(service_name_1)
                                .template blackboard_creator<uint64_t>()
                                .template add_with_default<uint64_t>(0)
                                .create()
                                .expect("");
    auto service_1_open = node.service_builder(service_name_1).template blackboard_opener<uint64_t>().open().expect("");
    auto service_2 = node.service_builder(service_name_2)
                         .template blackboard_creator<uint64_t>()
                         .template add_with_default<uint64_t>(0)
                         .create()
                         .expect("");

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}

// TYPED_TEST(ServiceBlackboardTest, subscriber_details_are_correct) {
// constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

// const auto service_name = iox2_testing::generate_service_name();
// auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
// auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

// iox2::Subscriber<SERVICE_TYPE, uint64_t, void> subscriber = sut.subscriber_builder().create().expect("");

// auto counter = 0;
// sut.dynamic_config().list_subscribers([&](auto subscriber_details_view) {
// counter++;
// EXPECT_TRUE(subscriber_details_view.subscriber_id() == subscriber.id());
// EXPECT_TRUE(subscriber_details_view.node_id() == node.id());
// EXPECT_TRUE(subscriber_details_view.buffer_size() == subscriber.buffer_size());
// return CallbackProgression::Stop;
//});

// ASSERT_THAT(counter, Eq(1));
//}

} // namespace
