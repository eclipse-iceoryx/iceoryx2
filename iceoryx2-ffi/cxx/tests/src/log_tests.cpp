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

#include "iox/string.hpp"
#include "iox/vector.hpp"
#include "iox2/log.hpp"
#include "iox2/log_level.hpp"

#include "test.hpp"

namespace {
using namespace iox2;
constexpr uint64_t TEST_LOGGER_CAPACITY = 10;
constexpr uint64_t STRING_CAPACITY = 64;
class Entry {
  private:
    LogLevel m_log_level;
    iox::string<STRING_CAPACITY> m_origin;
    iox::string<STRING_CAPACITY> m_message;

  public:
    Entry(LogLevel log_level, const char* origin, const char* message)
        : m_log_level { log_level }
        , m_origin { iox::TruncateToCapacity, origin }
        , m_message { iox::TruncateToCapacity, message } {
    }

    auto is_equal(LogLevel log_level, const char* origin, const char* message) -> bool {
        return m_log_level == log_level && m_origin == iox::string<STRING_CAPACITY>(iox::TruncateToCapacity, origin)
               && m_message == iox::string<STRING_CAPACITY>(iox::TruncateToCapacity, message);
    }
};


class TestLogger : public Log {
  public:
    static auto get_instance() -> TestLogger& {
        static TestLogger INSTANCE;
        return INSTANCE;
    }

    void log(LogLevel log_level, const char* origin, const char* message) override {
        if (m_log_buffer.size() < TEST_LOGGER_CAPACITY) {
            m_log_buffer.emplace_back(log_level, origin, message);
        }
    }

    auto get_log_buffer() -> iox::vector<Entry, TEST_LOGGER_CAPACITY> {
        auto buffer = m_log_buffer;
        m_log_buffer.clear();
        return buffer;
    }

  private:
    iox::vector<Entry, TEST_LOGGER_CAPACITY> m_log_buffer;
};

TEST(Log, custom_logger_works) {
    set_log_level(LogLevel::Trace);
    ASSERT_TRUE(set_logger(TestLogger::get_instance()));

    log(LogLevel::Trace, "hello", "world");
    log(LogLevel::Debug, "goodbye", "hypnotoad");
    log(LogLevel::Info, "Who is looking for freedom?", "The Hoff!");
    log(LogLevel::Warn, "Blümchen", "Bassface");
    log(LogLevel::Error, "Blümchen should record a single with", "The almighty Hypnotoad");
    log(LogLevel::Fatal, "It is the end", "my beloved toad.");

    auto log_buffer = TestLogger::get_instance().get_log_buffer();

    ASSERT_THAT(log_buffer.size(), Eq(6));

    ASSERT_TRUE(log_buffer[0].is_equal(LogLevel::Trace, "hello", "world"));
    ASSERT_TRUE(log_buffer[1].is_equal(LogLevel::Debug, "goodbye", "hypnotoad"));
    ASSERT_TRUE(log_buffer[2].is_equal(LogLevel::Info, "Who is looking for freedom?", "The Hoff!"));
    ASSERT_TRUE(log_buffer[3].is_equal(LogLevel::Warn, "Blümchen", "Bassface"));
    ASSERT_TRUE(
        log_buffer[4].is_equal(LogLevel::Error, "Blümchen should record a single with", "The almighty Hypnotoad"));
    ASSERT_TRUE(log_buffer[5].is_equal(LogLevel::Fatal, "It is the end", "my beloved toad."));
}

TEST(Log, can_set_and_get_log_level) {
    set_log_level(LogLevel::Trace);
    EXPECT_EQ(get_log_level(), LogLevel::Trace);

    set_log_level(LogLevel::Debug);
    EXPECT_EQ(get_log_level(), LogLevel::Debug);

    set_log_level(LogLevel::Info);
    EXPECT_EQ(get_log_level(), LogLevel::Info);

    set_log_level(LogLevel::Warn);
    EXPECT_EQ(get_log_level(), LogLevel::Warn);

    set_log_level(LogLevel::Error);
    EXPECT_EQ(get_log_level(), LogLevel::Error);

    set_log_level(LogLevel::Fatal);
    EXPECT_EQ(get_log_level(), LogLevel::Fatal);
}

TEST(Log, can_set_and_get_log_level_from_env) {
    set_log_level_from_env_or_default();
    EXPECT_EQ(get_log_level(), LogLevel::Info);

    set_log_level_from_env_or(LogLevel::Trace);
    EXPECT_EQ(get_log_level(), LogLevel::Trace);

    set_log_level_from_env_or(LogLevel::Debug);
    EXPECT_EQ(get_log_level(), LogLevel::Debug);

    set_log_level_from_env_or(LogLevel::Info);
    EXPECT_EQ(get_log_level(), LogLevel::Info);

    set_log_level_from_env_or(LogLevel::Warn);
    EXPECT_EQ(get_log_level(), LogLevel::Warn);

    set_log_level_from_env_or(LogLevel::Error);
    EXPECT_EQ(get_log_level(), LogLevel::Error);

    set_log_level_from_env_or(LogLevel::Fatal);
    EXPECT_EQ(get_log_level(), LogLevel::Fatal);
}

} // namespace
