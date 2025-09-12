// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#include "iox2/config.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

TEST(Config, global_prefix) {
    const auto test_value = iox::FileName::create("oh_my_dot").expect("");
    auto config = Config();

    config.global().set_prefix(test_value);
    ASSERT_THAT(config.global().prefix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_root_path) {
    const auto test_value = iox::Path::create("some_path").expect("");
    auto config = Config();

    config.global().set_root_path(test_value);
    ASSERT_THAT(config.global().root_path(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, defaults_event_max_listeners) {
    const auto test_value = 123;
    auto config = Config();

    config.defaults().event().set_max_listeners(test_value);
    ASSERT_THAT(config.defaults().event().max_listeners(), Eq(test_value));
}

TEST(Config, defaults_event_notifier_created_event) {
    const auto test_value = iox::optional<size_t>(12);
    auto config = Config();

    config.defaults().event().set_notifier_created_event(test_value);
    ASSERT_THAT(config.defaults().event().notifier_created_event(), Eq(test_value));

    config.defaults().event().set_notifier_created_event(iox::nullopt);
    ASSERT_THAT(config.defaults().event().notifier_created_event(), Eq(iox::nullopt));
}

TEST(Config, defaults_event_notifier_dropped_event) {
    const auto test_value = iox::optional<size_t>(13);
    auto config = Config();

    config.defaults().event().set_notifier_dropped_event(test_value);
    ASSERT_THAT(config.defaults().event().notifier_dropped_event(), Eq(test_value));

    config.defaults().event().set_notifier_dropped_event(iox::nullopt);
    ASSERT_THAT(config.defaults().event().notifier_dropped_event(), Eq(iox::nullopt));
}

TEST(Config, defaults_event_notifier_dead_event) {
    const auto test_value = iox::optional<size_t>(14);
    auto config = Config();

    config.defaults().event().set_notifier_dead_event(test_value);
    ASSERT_THAT(config.defaults().event().notifier_dead_event(), Eq(test_value));

    config.defaults().event().set_notifier_dead_event(iox::nullopt);
    ASSERT_THAT(config.defaults().event().notifier_dead_event(), Eq(iox::nullopt));
}

TEST(Config, defaults_event_max_notifiers) {
    const auto test_value = 45;
    auto config = Config();

    config.defaults().event().set_max_notifiers(test_value);
    ASSERT_THAT(config.defaults().event().max_notifiers(), Eq(test_value));
}

TEST(Config, defaults_event_max_nodes) {
    const auto test_value = 78;
    auto config = Config();

    config.defaults().event().set_max_nodes(test_value);
    ASSERT_THAT(config.defaults().event().max_nodes(), Eq(test_value));
}

TEST(Config, defaults_event_event_id_max_value) {
    const auto test_value = 799;
    auto config = Config();

    config.defaults().event().set_event_id_max_value(test_value);
    ASSERT_THAT(config.defaults().event().event_id_max_value(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_max_subscribers) {
    const auto test_value = 313;
    auto config = Config();

    config.defaults().publish_subscribe().set_max_subscribers(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().max_subscribers(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_max_publishers) {
    const auto test_value = 424;
    auto config = Config();

    config.defaults().publish_subscribe().set_max_publishers(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().max_publishers(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_max_nodes) {
    const auto test_value = 535;
    auto config = Config();

    config.defaults().publish_subscribe().set_max_nodes(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().max_nodes(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_subscriber_max_buffer_size) {
    const auto test_value = 646;
    auto config = Config();

    config.defaults().publish_subscribe().set_subscriber_max_buffer_size(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().subscriber_max_buffer_size(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_subscriber_max_borrowed_samples) {
    const auto test_value = 757;
    auto config = Config();

    config.defaults().publish_subscribe().set_subscriber_max_borrowed_samples(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().subscriber_max_borrowed_samples(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_publisher_max_loaned_samples) {
    const auto test_value = 868;
    auto config = Config();

    config.defaults().publish_subscribe().set_publisher_max_loaned_samples(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().publisher_max_loaned_samples(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_publisher_history_size) {
    const auto test_value = 979;
    auto config = Config();

    config.defaults().publish_subscribe().set_publisher_history_size(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().publisher_history_size(), Eq(test_value));
}

TEST(Config, defaults_publish_subscribe_enable_safe_overflow) {
    auto config = Config();

    config.defaults().publish_subscribe().set_enable_safe_overflow(true);
    ASSERT_THAT(config.defaults().publish_subscribe().enable_safe_overflow(), Eq(true));
    config.defaults().publish_subscribe().set_enable_safe_overflow(false);
    ASSERT_THAT(config.defaults().publish_subscribe().enable_safe_overflow(), Eq(false));
}

TEST(Config, defaults_publish_subscribe_unable_to_deliver_strategy) {
    auto config = Config();

    config.defaults().publish_subscribe().set_unable_to_deliver_strategy(UnableToDeliverStrategy::Block);
    ASSERT_THAT(config.defaults().publish_subscribe().unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::Block));
    config.defaults().publish_subscribe().set_unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample);
    ASSERT_THAT(config.defaults().publish_subscribe().unable_to_deliver_strategy(),
                Eq(UnableToDeliverStrategy::DiscardSample));
}

TEST(Config, defaults_publish_subscribe_subscriber_expired_connection_buffer) {
    const auto test_value = 13113;
    auto config = Config();

    config.defaults().publish_subscribe().set_subscriber_expired_connection_buffer(test_value);
    ASSERT_THAT(config.defaults().publish_subscribe().subscriber_expired_connection_buffer(), Eq(test_value));
}

TEST(Config, global_service_directory) {
    const auto test_value = iox::Path::create("look/there/flies/a/dead/pidgin").expect("");
    auto config = Config();

    config.global().service().set_directory(test_value);
    ASSERT_THAT(config.global().service().directory(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_service_data_segment_suffix) {
    const auto test_value = iox::FileName::create("no_touchy_fishy").expect("");
    auto config = Config();

    config.global().service().set_data_segment_suffix(test_value);
    ASSERT_THAT(config.global().service().data_segment_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_service_static_config_storage_suffix) {
    const auto test_value = iox::FileName::create("its_a_smelly_fishy").expect("");
    auto config = Config();

    config.global().service().set_static_config_storage_suffix(test_value);
    ASSERT_THAT(config.global().service().static_config_storage_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_service_dynamic_config_storage_suffix) {
    const auto test_value = iox::FileName::create("nala_runs_while_dreaming").expect("");
    auto config = Config();

    config.global().service().set_dynamic_config_storage_suffix(test_value);
    ASSERT_THAT(config.global().service().dynamic_config_storage_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_service_creation_timeout) {
    const auto test_value = iox::units::Duration::fromSeconds(1234);
    auto config = Config();

    config.global().service().set_creation_timeout(test_value);
    ASSERT_THAT(config.global().service().creation_timeout(), Eq(test_value));
}

TEST(Config, global_service_connection_suffix) {
    const auto test_value = iox::FileName::create("what_dinosaur_ancester_has_the_pidgin").expect("");
    auto config = Config();

    config.global().service().set_connection_suffix(test_value);
    ASSERT_THAT(config.global().service().connection_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_service_event_connection_suffix) {
    const auto test_value = iox::FileName::create("dont_eat_elephants").expect("");
    auto config = Config();

    config.global().service().set_event_connection_suffix(test_value);
    ASSERT_THAT(config.global().service().event_connection_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_node_directory) {
    const auto test_value = iox::Path::create("eat/the/carrototier").expect("");
    auto config = Config();

    config.global().node().set_directory(test_value);
    ASSERT_THAT(config.global().node().directory(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_node_monitor_suffix) {
    const auto test_value = iox::FileName::create("why_i_am_so_happy_so_happy_oh_so_blurpy").expect("");
    auto config = Config();

    config.global().node().set_monitor_suffix(test_value);
    ASSERT_THAT(config.global().node().monitor_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_node_static_config_suffix) {
    const auto test_value = iox::FileName::create("spin_me_like_a_vinyl_record").expect("");
    auto config = Config();

    config.global().node().set_static_config_suffix(test_value);
    ASSERT_THAT(config.global().node().static_config_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_node_service_tag_suffix) {
    const auto test_value = iox::FileName::create("who_is_fluffy").expect("");
    auto config = Config();

    config.global().node().set_service_tag_suffix(test_value);
    ASSERT_THAT(config.global().node().service_tag_suffix(), StrEq(test_value.as_string().c_str()));
}

TEST(Config, global_node_cleanup_dead_nodes_on_creation) {
    auto config = Config();

    config.global().node().set_cleanup_dead_nodes_on_creation(true);
    ASSERT_THAT(config.global().node().cleanup_dead_nodes_on_creation(), Eq(true));
    config.global().node().set_cleanup_dead_nodes_on_creation(false);
    ASSERT_THAT(config.global().node().cleanup_dead_nodes_on_creation(), Eq(false));
}

TEST(Config, global_node_cleanup_dead_nodes_on_destruction) {
    auto config = Config();

    config.global().node().set_cleanup_dead_nodes_on_destruction(true);
    ASSERT_THAT(config.global().node().cleanup_dead_nodes_on_destruction(), Eq(true));
    config.global().node().set_cleanup_dead_nodes_on_destruction(false);
    ASSERT_THAT(config.global().node().cleanup_dead_nodes_on_destruction(), Eq(false));
}

TEST(Config, defaults_request_response_enable_safe_overflow) {
    auto config = Config();

    config.defaults().request_response().set_enable_safe_overflow_for_requests(true);
    ASSERT_THAT(config.defaults().request_response().enable_safe_overflow_for_requests(), Eq(true));
    config.defaults().request_response().set_enable_safe_overflow_for_requests(false);
    ASSERT_THAT(config.defaults().request_response().enable_safe_overflow_for_requests(), Eq(false));

    config.defaults().request_response().set_enable_safe_overflow_for_responses(true);
    ASSERT_THAT(config.defaults().request_response().enable_safe_overflow_for_responses(), Eq(true));
    config.defaults().request_response().set_enable_safe_overflow_for_responses(false);
    ASSERT_THAT(config.defaults().request_response().enable_safe_overflow_for_responses(), Eq(false));
}

TEST(Config, defaults_request_response_max_active_requests_per_client) {
    const auto test_value = 100;
    auto config = Config();

    config.defaults().request_response().set_max_active_requests_per_client(test_value);
    ASSERT_THAT(config.defaults().request_response().max_active_requests_per_client(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_response_buffer_size) {
    const auto test_value = 99;
    auto config = Config();

    config.defaults().request_response().set_max_response_buffer_size(test_value);
    ASSERT_THAT(config.defaults().request_response().max_response_buffer_size(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_servers) {
    const auto test_value = 98;
    auto config = Config();

    config.defaults().request_response().set_max_servers(test_value);
    ASSERT_THAT(config.defaults().request_response().max_servers(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_clients) {
    const auto test_value = 97;
    auto config = Config();

    config.defaults().request_response().set_max_clients(test_value);
    ASSERT_THAT(config.defaults().request_response().max_clients(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_nodes) {
    const auto test_value = 96;
    auto config = Config();

    config.defaults().request_response().set_max_nodes(test_value);
    ASSERT_THAT(config.defaults().request_response().max_nodes(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_borrowed_responses_per_pending_response) {
    const auto test_value = 95;
    auto config = Config();

    config.defaults().request_response().set_max_borrowed_responses_per_pending_response(test_value);
    ASSERT_THAT(config.defaults().request_response().max_borrowed_responses_per_pending_response(), Eq(test_value));
}

TEST(Config, defaults_request_response_max_loaned_requests) {
    const auto test_value = 94;
    auto config = Config();

    config.defaults().request_response().set_max_loaned_requests(test_value);
    ASSERT_THAT(config.defaults().request_response().max_loaned_requests(), Eq(test_value));
}

TEST(Config, defaults_request_response_server_max_loaned_responses_per_request) {
    const auto test_value = 93;
    auto config = Config();

    config.defaults().request_response().set_server_max_loaned_responses_per_request(test_value);
    ASSERT_THAT(config.defaults().request_response().server_max_loaned_responses_per_request(), Eq(test_value));
}

TEST(Config, defaults_request_response_unable_to_deliver_strategy) {
    auto config = Config();

    config.defaults().request_response().set_client_unable_to_deliver_strategy(UnableToDeliverStrategy::Block);
    ASSERT_THAT(config.defaults().request_response().client_unable_to_deliver_strategy(),
                Eq(UnableToDeliverStrategy::Block));
    config.defaults().request_response().set_client_unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample);
    ASSERT_THAT(config.defaults().request_response().client_unable_to_deliver_strategy(),
                Eq(UnableToDeliverStrategy::DiscardSample));

    config.defaults().request_response().set_server_unable_to_deliver_strategy(UnableToDeliverStrategy::Block);
    ASSERT_THAT(config.defaults().request_response().server_unable_to_deliver_strategy(),
                Eq(UnableToDeliverStrategy::Block));
    config.defaults().request_response().set_server_unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample);
    ASSERT_THAT(config.defaults().request_response().server_unable_to_deliver_strategy(),
                Eq(UnableToDeliverStrategy::DiscardSample));
}

TEST(Config, defaults_request_response_client_expired_connection_buffer) {
    const auto test_value = 92;
    auto config = Config();

    config.defaults().request_response().set_client_expired_connection_buffer(test_value);
    ASSERT_THAT(config.defaults().request_response().client_expired_connection_buffer(), Eq(test_value));
}

TEST(Config, defaults_request_response_enable_fire_and_forget_requests) {
    auto config = Config();

    config.defaults().request_response().set_enable_fire_and_forget_requests(true);
    ASSERT_THAT(config.defaults().request_response().enable_fire_and_forget_requests(), Eq(true));
    config.defaults().request_response().set_enable_fire_and_forget_requests(false);
    ASSERT_THAT(config.defaults().request_response().enable_fire_and_forget_requests(), Eq(false));
}

} // namespace
