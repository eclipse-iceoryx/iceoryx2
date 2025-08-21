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

#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/waitset.hpp"
#include "test.hpp"

#include <chrono>
#include <vector>

namespace {
using namespace iox2;
using namespace iox::units;

constexpr Duration TIMEOUT = Duration::fromMilliseconds(100);

auto generate_name() -> ServiceName {
    static std::atomic<uint64_t> COUNTER = 0;
    return ServiceName::create((std::string("waitset_tests_") + std::to_string(COUNTER.fetch_add(1))).c_str())
        .expect("");
}

template <typename T>
struct WaitSetTest : public ::testing::Test {
    static constexpr ServiceType TYPE = T::TYPE;

    WaitSetTest()
        : node { NodeBuilder().create<TYPE>().expect("") }
        , event { node.service_builder(generate_name()).event().create().expect("") } {
    }

    auto create_sut() -> WaitSet<TYPE> {
        return WaitSetBuilder().create<TYPE>().expect("");
    }

    auto create_listener() -> Listener<TYPE> {
        return event.listener_builder().create().expect("");
    }

    auto create_notifier() -> Notifier<TYPE> {
        return event.notifier_builder().create().expect("");
    }

    // NOLINTBEGIN(misc-non-private-member-variables-in-classes), come on, its a test
    Node<TYPE> node;
    PortFactoryEvent<TYPE> event;
    // NOLINTEND(misc-non-private-member-variables-in-classes)
};

TYPED_TEST_SUITE(WaitSetTest, iox2_testing::ServiceTypes, );

TYPED_TEST(WaitSetTest, newly_created_waitset_is_empty) {
    auto sut = this->create_sut();

    ASSERT_THAT(sut.len(), Eq(0));
    ASSERT_THAT(sut.is_empty(), Eq(true));
}

//NOLINTBEGIN(readability-function-cognitive-complexity)
TYPED_TEST(WaitSetTest, attaching_different_elements_works) {
    constexpr uint64_t NUMBER_OF_DEADLINES = 3;
    constexpr uint64_t NUMBER_OF_NOTIFICATIONS = 5;
    constexpr uint64_t NUMBER_OF_INTERVALS = 7;
    auto sut = this->create_sut();

    std::vector<Listener<TestFixture::TYPE>> listeners;
    std::vector<WaitSetGuard<TestFixture::TYPE>> guards;

    for (uint64_t idx = 0; idx < NUMBER_OF_INTERVALS; ++idx) {
        guards.emplace_back(sut.attach_interval(Duration::fromSeconds(idx + 1)).expect(""));
        ASSERT_THAT(sut.len(), Eq(idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_NOTIFICATIONS; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_notification(listener).expect(""));
        listeners.emplace_back(std::move(listener));
        ASSERT_THAT(sut.len(), Eq(NUMBER_OF_INTERVALS + idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_DEADLINES; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_deadline(listener, Duration::fromSeconds(idx + 1)).expect(""));
        listeners.emplace_back(std::move(listener));
        ASSERT_THAT(sut.len(), Eq(NUMBER_OF_INTERVALS + NUMBER_OF_NOTIFICATIONS + idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    guards.clear();
    listeners.clear();
    ASSERT_THAT(sut.len(), Eq(0));
    ASSERT_THAT(sut.is_empty(), Eq(true));
}
//NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(WaitSetTest, attaching_same_deadline_twice_fails) {
    auto sut = this->create_sut();
    auto listener = this->create_listener();

    auto result_1 = sut.attach_deadline(listener, Duration::fromSeconds(1));
    auto result_2 = sut.attach_deadline(listener, Duration::fromSeconds(1));

    ASSERT_THAT(result_1.has_error(), Eq(false));
    ASSERT_THAT(result_2.has_error(), Eq(true));
    ASSERT_THAT(result_2.get_error(), Eq(WaitSetAttachmentError::AlreadyAttached));
}

TYPED_TEST(WaitSetTest, attaching_same_notification_twice_fails) {
    auto sut = this->create_sut();
    auto listener = this->create_listener();

    auto result_1 = sut.attach_notification(listener);
    auto result_2 = sut.attach_notification(listener);

    ASSERT_THAT(result_1.has_error(), Eq(false));
    ASSERT_THAT(result_2.has_error(), Eq(true));
    ASSERT_THAT(result_2.get_error(), Eq(WaitSetAttachmentError::AlreadyAttached));
}

TYPED_TEST(WaitSetTest, empty_waitset_returns_error_on_run) {
    auto sut = this->create_sut();
    auto result = sut.wait_and_process([](auto) { return CallbackProgression::Continue; });

    ASSERT_THAT(result.has_error(), Eq(true));
    ASSERT_THAT(result.get_error(), Eq(WaitSetRunError::NoAttachments));
}

TYPED_TEST(WaitSetTest, empty_waitset_returns_error_on_run_once) {
    auto sut = this->create_sut();
    auto result = sut.wait_and_process_once([](auto) { return CallbackProgression::Continue; });

    ASSERT_THAT(result.has_error(), Eq(true));
    ASSERT_THAT(result.get_error(), Eq(WaitSetRunError::NoAttachments));
}

TYPED_TEST(WaitSetTest, interval_attachment_blocks_for_at_least_timeout) {
    auto sut = this->create_sut();

    auto begin = std::chrono::steady_clock::now();
    auto guard = sut.attach_interval(TIMEOUT).expect("");

    auto callback_called = false;
    auto result = sut.wait_and_process([&](auto attachment_id) -> CallbackProgression {
        callback_called = true;
        EXPECT_THAT(attachment_id.has_event_from(guard), Eq(true));
        EXPECT_THAT(attachment_id.has_missed_deadline(guard), Eq(false));
        return CallbackProgression::Stop;
    });

    auto end = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count();

    ASSERT_THAT(callback_called, Eq(true));
    ASSERT_THAT(elapsed, Ge(TIMEOUT.toMilliseconds()));
}

TYPED_TEST(WaitSetTest, deadline_attachment_blocks_for_at_least_timeout) {
    auto sut = this->create_sut();
    auto listener = this->create_listener();

    auto begin = std::chrono::steady_clock::now();
    auto guard = sut.attach_deadline(listener, TIMEOUT).expect("");

    auto callback_called = false;
    auto result = sut.wait_and_process([&](auto attachment_id) -> CallbackProgression {
        callback_called = true;
        EXPECT_THAT(attachment_id.has_event_from(guard), Eq(false));
        EXPECT_THAT(attachment_id.has_missed_deadline(guard), Eq(true));
        return CallbackProgression::Stop;
    });

    auto end = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count();

    ASSERT_THAT(callback_called, Eq(true));
    ASSERT_THAT(elapsed, Ge(TIMEOUT.toMilliseconds()));
}

TYPED_TEST(WaitSetTest, does_not_block_longer_than_provided_timeout) {
    auto sut = this->create_sut();

    auto begin = std::chrono::steady_clock::now();
    auto guard = sut.attach_interval(Duration::max()).expect("");

    auto callback_called = false;
    auto result = sut.wait_and_process_once_with_timeout(
        [&](auto) -> CallbackProgression {
            callback_called = true;
            return CallbackProgression::Stop;
        },
        TIMEOUT);

    auto end = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count();

    ASSERT_THAT(callback_called, Eq(false));
    ASSERT_THAT(elapsed, Ge(TIMEOUT.toMilliseconds()));
}

TYPED_TEST(WaitSetTest, blocks_until_interval_when_user_timeout_is_larger) {
    auto sut = this->create_sut();

    auto begin = std::chrono::steady_clock::now();
    auto guard = sut.attach_interval(TIMEOUT).expect("");

    auto callback_called = false;
    auto result = sut.wait_and_process_once([&](auto) -> CallbackProgression {
        callback_called = true;
        return CallbackProgression::Stop;
    });

    auto end = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count();

    ASSERT_THAT(callback_called, Eq(true));
    ASSERT_THAT(elapsed, Ge(TIMEOUT.toMilliseconds()));
}


TYPED_TEST(WaitSetTest, deadline_attachment_wakes_up_when_notified) {
    auto sut = this->create_sut();
    auto listener = this->create_listener();

    auto guard = sut.attach_deadline(listener, Duration::fromHours(1)).expect("");

    auto callback_called = false;
    std::thread notifier_thread([&]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT.toMilliseconds()));
        auto notifier = this->create_notifier();
        notifier.notify().expect("");
    });
    auto result = sut.wait_and_process([&](auto attachment_id) -> CallbackProgression {
        callback_called = true;
        EXPECT_THAT(attachment_id.has_event_from(guard), Eq(true));
        EXPECT_THAT(attachment_id.has_missed_deadline(guard), Eq(false));
        return CallbackProgression::Stop;
    });

    notifier_thread.join();
    ASSERT_THAT(callback_called, Eq(true));
}

TYPED_TEST(WaitSetTest, notification_attachment_wakes_up_when_notified) {
    auto sut = this->create_sut();
    auto listener = this->create_listener();

    auto guard = sut.attach_notification(listener).expect("");

    auto callback_called = false;
    std::thread notifier_thread([&]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT.toMilliseconds()));
        auto notifier = this->create_notifier();
        notifier.notify().expect("");
    });
    auto result = sut.wait_and_process([&](auto attachment_id) -> CallbackProgression {
        callback_called = true;
        EXPECT_THAT(attachment_id.has_event_from(guard), Eq(true));
        EXPECT_THAT(attachment_id.has_missed_deadline(guard), Eq(false));
        return CallbackProgression::Stop;
    });

    notifier_thread.join();
    ASSERT_THAT(callback_called, Eq(true));
}

TYPED_TEST(WaitSetTest, triggering_everything_works) {
    constexpr uint64_t NUMBER_OF_DEADLINES = 3;
    constexpr uint64_t NUMBER_OF_NOTIFICATIONS = 5;
    constexpr uint64_t NUMBER_OF_INTERVALS = 7;
    auto sut = this->create_sut();

    std::vector<Listener<TestFixture::TYPE>> listeners;
    std::vector<WaitSetGuard<TestFixture::TYPE>> guards;
    guards.reserve(NUMBER_OF_INTERVALS + NUMBER_OF_NOTIFICATIONS + NUMBER_OF_DEADLINES);
    listeners.reserve(NUMBER_OF_NOTIFICATIONS + NUMBER_OF_DEADLINES);

    for (uint64_t idx = 0; idx < NUMBER_OF_INTERVALS; ++idx) {
        guards.emplace_back(sut.attach_interval(Duration::fromNanoseconds(1)).expect(""));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_NOTIFICATIONS; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_notification(listener).expect(""));
        listeners.emplace_back(std::move(listener));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_DEADLINES; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_deadline(listener, Duration::fromHours(1)).expect(""));
        listeners.emplace_back(std::move(listener));
    }

    auto notifier = this->create_notifier();
    notifier.notify().expect("");

    std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT.toMilliseconds()));
    std::vector<bool> was_triggered(guards.size(), false);

    auto result = sut.wait_and_process_once([&](auto attachment_id) -> CallbackProgression {
        for (uint64_t idx = 0; idx < guards.size(); ++idx) {
            if (attachment_id.has_event_from(guards[idx])) {
                was_triggered[idx] = true;
                break;
            }
        }

        return CallbackProgression::Continue;
    });

    ASSERT_THAT(result.has_error(), Eq(false));

    for (auto triggered : was_triggered) {
        ASSERT_THAT(triggered, Eq(true));
    }
}

TYPED_TEST(WaitSetTest, signal_handling_mode_can_be_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto sut_1 = WaitSetBuilder().signal_handling_mode(SignalHandlingMode::Disabled).create<SERVICE_TYPE>().expect("");
    auto sut_2 = WaitSetBuilder()
                     .signal_handling_mode(SignalHandlingMode::HandleTerminationRequests)
                     .create<SERVICE_TYPE>()
                     .expect("");

    ASSERT_THAT(sut_1.signal_handling_mode(), Eq(SignalHandlingMode::Disabled));
    ASSERT_THAT(sut_2.signal_handling_mode(), Eq(SignalHandlingMode::HandleTerminationRequests));
}
} // namespace
