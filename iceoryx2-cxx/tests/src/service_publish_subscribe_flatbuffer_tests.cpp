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

#include "iox2/iceoryx2_cxx_deployment.hpp"

#if IOX2_FEATURE_FLATBUFFERS

#include "iox2/allocation_strategy.hpp"
#include "iox2/bb/file_name.hpp"
#include "iox2/bb/optional.hpp"
#include "iox2/bb/static_string.hpp"
#include "iox2/node.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/testing.hpp"
#include "iox2/type_name.hpp"

#include "test.hpp"

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <flatbuffers/flatbuffers.h>
#include <fstream>
#include <gtest/gtest.h>

// NOLINTBEGIN
namespace Example {

struct Entry;
struct EntryBuilder;

struct UnboundedData;
struct UnboundedDataBuilder;

struct Entry FLATBUFFERS_FINAL_CLASS : private ::flatbuffers::Table {
    typedef EntryBuilder Builder;
    enum FlatBuffersVTableOffset FLATBUFFERS_VTABLE_UNDERLYING_TYPE {
        VT_DATA_1 = 4,
        VT_DATA_2 = 6
    };
    int32_t data_1() const {
        return GetField<int32_t>(VT_DATA_1, 0);
    }
    uint64_t data_2() const {
        return GetField<uint64_t>(VT_DATA_2, 0);
    }
    template <bool B = false>
    bool Verify(::flatbuffers::VerifierTemplate<B>& verifier) const {
        return VerifyTableStart(verifier) && VerifyField<int32_t>(verifier, VT_DATA_1, 4)
               && VerifyField<uint64_t>(verifier, VT_DATA_2, 8) && verifier.EndTable();
    }
};

struct EntryBuilder {
    typedef Entry Table;
    ::flatbuffers::FlatBufferBuilder& fbb_;
    ::flatbuffers::uoffset_t start_;
    void add_data_1(int32_t data_1) {
        fbb_.AddElement<int32_t>(Entry::VT_DATA_1, data_1, 0);
    }
    void add_data_2(uint64_t data_2) {
        fbb_.AddElement<uint64_t>(Entry::VT_DATA_2, data_2, 0);
    }
    explicit EntryBuilder(::flatbuffers::FlatBufferBuilder& _fbb)
        : fbb_(_fbb) {
        start_ = fbb_.StartTable();
    }
    ::flatbuffers::Offset<Entry> Finish() {
        const auto end = fbb_.EndTable(start_);
        auto o = ::flatbuffers::Offset<Entry>(end);
        return o;
    }
};

inline ::flatbuffers::Offset<Entry>
CreateEntry(::flatbuffers::FlatBufferBuilder& _fbb, int32_t data_1 = 0, uint64_t data_2 = 0) {
    EntryBuilder builder_(_fbb);
    builder_.add_data_2(data_2);
    builder_.add_data_1(data_1);
    return builder_.Finish();
}

struct UnboundedData FLATBUFFERS_FINAL_CLASS : private ::flatbuffers::Table {
    typedef UnboundedDataBuilder Builder;
    enum FlatBuffersVTableOffset FLATBUFFERS_VTABLE_UNDERLYING_TYPE {
        VT_TITLE = 4,
        VT_ENTRIES = 6
    };
    const ::flatbuffers::String* title() const {
        return GetPointer<const ::flatbuffers::String*>(VT_TITLE);
    }
    const ::flatbuffers::Vector<::flatbuffers::Offset<Example::Entry>>* entries() const {
        return GetPointer<const ::flatbuffers::Vector<::flatbuffers::Offset<Example::Entry>>*>(VT_ENTRIES);
    }
    template <bool B = false>
    bool Verify(::flatbuffers::VerifierTemplate<B>& verifier) const {
        return VerifyTableStart(verifier) && VerifyOffset(verifier, VT_TITLE) && verifier.VerifyString(title())
               && VerifyOffset(verifier, VT_ENTRIES) && verifier.VerifyVector(entries())
               && verifier.VerifyVectorOfTables(entries()) && verifier.EndTable();
    }
};

struct UnboundedDataBuilder {
    typedef UnboundedData Table;
    ::flatbuffers::FlatBufferBuilder& fbb_;
    ::flatbuffers::uoffset_t start_;
    void add_title(::flatbuffers::Offset<::flatbuffers::String> title) {
        fbb_.AddOffset(UnboundedData::VT_TITLE, title);
    }
    void add_entries(::flatbuffers::Offset<::flatbuffers::Vector<::flatbuffers::Offset<Example::Entry>>> entries) {
        fbb_.AddOffset(UnboundedData::VT_ENTRIES, entries);
    }
    explicit UnboundedDataBuilder(::flatbuffers::FlatBufferBuilder& _fbb)
        : fbb_(_fbb) {
        start_ = fbb_.StartTable();
    }
    ::flatbuffers::Offset<UnboundedData> Finish() {
        const auto end = fbb_.EndTable(start_);
        auto o = ::flatbuffers::Offset<UnboundedData>(end);
        return o;
    }
};

inline ::flatbuffers::Offset<UnboundedData>
CreateUnboundedData(::flatbuffers::FlatBufferBuilder& _fbb,
                    ::flatbuffers::Offset<::flatbuffers::String> title = 0,
                    ::flatbuffers::Offset<::flatbuffers::Vector<::flatbuffers::Offset<Example::Entry>>> entries = 0) {
    UnboundedDataBuilder builder_(_fbb);
    builder_.add_entries(entries);
    builder_.add_title(title);
    return builder_.Finish();
}

inline ::flatbuffers::Offset<UnboundedData>
CreateUnboundedDataDirect(::flatbuffers::FlatBufferBuilder& _fbb,
                          const char* title = nullptr,
                          const std::vector<::flatbuffers::Offset<Example::Entry>>* entries = nullptr) {
    auto title__ = title ? _fbb.CreateString(title) : 0;
    auto entries__ = entries ? _fbb.CreateVector<::flatbuffers::Offset<Example::Entry>>(*entries) : 0;
    return Example::CreateUnboundedData(_fbb, title__, entries__);
}

inline const Example::UnboundedData* GetUnboundedData(const void* buf) {
    return ::flatbuffers::GetRoot<Example::UnboundedData>(buf);
}

inline const Example::UnboundedData* GetSizePrefixedUnboundedData(const void* buf) {
    return ::flatbuffers::GetSizePrefixedRoot<Example::UnboundedData>(buf);
}

template <bool B = false>
inline bool VerifyUnboundedDataBuffer(::flatbuffers::VerifierTemplate<B>& verifier) {
    return verifier.template VerifyBuffer<Example::UnboundedData>(nullptr);
}

template <bool B = false>
inline bool VerifySizePrefixedUnboundedDataBuffer(::flatbuffers::VerifierTemplate<B>& verifier) {
    return verifier.template VerifySizePrefixedBuffer<Example::UnboundedData>(nullptr);
}

inline void FinishUnboundedDataBuffer(::flatbuffers::FlatBufferBuilder& fbb,
                                      ::flatbuffers::Offset<Example::UnboundedData> root) {
    fbb.Finish(root);
}

inline void FinishSizePrefixedUnboundedDataBuffer(::flatbuffers::FlatBufferBuilder& fbb,
                                                  ::flatbuffers::Offset<Example::UnboundedData> root) {
    fbb.FinishSizePrefixed(root);
}

} // namespace Example
// NOLINTEND

IOX2_DEFINE_TYPE_NAME(Example::UnboundedData, "UnboundedData");

namespace {
using namespace iox2;

constexpr const char* SCHEMA = R"(
    namespace Example;

    table Entry {
        data_1: int32;
        data_2: uint64;
    }

    table UnboundedData {
        title: string;
        entries: [Entry];
    }

    root_type UnboundedData;
)";

constexpr const char* ALT_SCHEMA = R"(
    namespace Example;

    table BoundedData {
        data_1: int32;
    }

    root_type BoundedData;
)";

constexpr uint64_t INITIAL_RESERVED_MEMORY = 1024;

template <typename T>
class ServicePublishSubscribeFlatbufferTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;

    // NOLINTNEXTLINE(bugprone-easily-swappable-parameters) fine for tests
    auto create_schema_file(const char* content, const char* file_name = "") -> bb::FilePath {
        auto schema_file_path = iox2::testing::test_directory_path();
        auto file_name_str =
            bb::StaticString<bb::platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8_null_terminated_unchecked_truncated(
                file_name, strlen(file_name));
        schema_file_path
            .append(strlen(file_name) == 0 ? iox2::testing::generate_file_name().as_string() : file_name_str)
            .value();

        auto schema_file = bb::FilePath::create(schema_file_path.as_string()).value();

        std::ofstream file(schema_file.as_string().unchecked_access().c_str());
        EXPECT_THAT(file.is_open(), Eq(true));
        if (file.is_open()) {
            file << content;
        }

        m_schema_files.push_back(schema_file);
        return schema_file;
    }

  protected:
    void SetUp() override {
        iox2::testing::create_test_directory();
    }

    void TearDown() override {
        for (auto file : m_schema_files) {
            static_cast<void>(std::remove(file.as_string().unchecked_access().c_str()));
        }
    }

  private:
    std::vector<bb::FilePath> m_schema_files;
};

TYPED_TEST_SUITE(ServicePublishSubscribeFlatbufferTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, create_fails_when_no_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name).template publish_subscribe<Flatbuffer<uint64_t>>().create();

    ASSERT_THAT(sut.error(), Eq(PublishSubscribeCreateError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, create_succeeds_with_schema_file) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<uint64_t>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();

    ASSERT_THAT(sut.has_value(), Eq(true));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, open_fails_when_no_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<Flatbuffer<uint64_t>>()
                          .flatbuffer_schema_path(schema_file)
                          .create();

    auto sut_open = node.service_builder(service_name).template publish_subscribe<Flatbuffer<uint64_t>>().open();

    ASSERT_THAT(sut_open.error(), Eq(PublishSubscribeOpenError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, open_fails_when_schema_is_not_the_same) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto alt_schema_file = this->create_schema_file(ALT_SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<Flatbuffer<uint64_t>>()
                          .flatbuffer_schema_path(schema_file)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<Flatbuffer<uint64_t>>()
                        .flatbuffer_schema_path(alt_schema_file)
                        .open();

    ASSERT_THAT(sut_open.error(), Eq(PublishSubscribeOpenError::IncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, open_succeeds_when_schema_content_is_identical) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<Flatbuffer<uint64_t>>()
                          .flatbuffer_schema_path(schema_file)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<Flatbuffer<uint64_t>>()
                        .flatbuffer_schema_path(schema_file)
                        .open();

    ASSERT_THAT(sut_open.has_value(), Eq(true));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, schema_path_lookup_works_when_creating_a_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto sut =
        node.service_builder(service_name).template publish_subscribe<Flatbuffer<Example::UnboundedData>>().create();

    ASSERT_THAT(sut.has_value(), Eq(true));
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, schema_path_lookup_works_when_opening_a_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto schema_file = this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto sut_create =
        node.service_builder(service_name).template publish_subscribe<Flatbuffer<Example::UnboundedData>>().create();
    ASSERT_THAT(sut_create.has_value(), Eq(true));
    static_cast<void>(schema_file);

    auto sut_open =
        node.service_builder(service_name).template publish_subscribe<Flatbuffer<Example::UnboundedData>>().open();

    ASSERT_THAT(sut_open.has_value(), Eq(true));
}

// Helper function to produce example Flatbuffer data
// NOLINTNEXTLINE(readability-function-size) fine for tests
auto produce_example_data(flatbuffers::FlatBufferBuilder& builder,
                          const char* title,
                          int32_t data_1,
                          uint64_t data_2, // NOLINT (bugprone-easily-swappable-parameters) fine for tests
                          size_t number_of_entries) -> flatbuffers::Offset<Example::UnboundedData> {
    auto title_str = builder.CreateString(title);
    std::vector<flatbuffers::Offset<Example::Entry>> entries;
    entries.reserve(number_of_entries);
    for (size_t i = 0; i < number_of_entries; ++i) {
        entries.emplace_back(Example::CreateEntry(builder, data_1, data_2));
    }
    auto entries_vec = builder.CreateVector(entries);
    return Example::CreateUnboundedData(builder, title_str, entries_vec);
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest, publish_subscribe_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value().publisher_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto subscriber = sut.value().subscriber_builder().create().value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data = produce_example_data(builder, "Weg vom Tisch!", 123, 456, 2); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);
    send(std::move(initialized_sample)).value();

    auto recv_sample_result = subscriber.receive();
    ASSERT_THAT(recv_sample_result.has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value().has_value(), Eq(true));
    const auto* recv_data = recv_sample_result.value()->payload_root();

    ASSERT_STREQ(recv_data->title()->c_str(), "Weg vom Tisch!");
    ASSERT_EQ(recv_data->entries()->size(), 2);

    for (auto i = 0U; i < 2; ++i) {
        ASSERT_EQ(recv_data->entries()->Get(i)->data_1(), 123);
        ASSERT_EQ(recv_data->entries()->Get(i)->data_2(), 456);
    }
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest,
           publisher_allocates_more_memory_when_initial_reserve_is_out_with_allocation_strategy_power_of_two) {
    constexpr int64_t ARRAY_SIZE = 50;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value()
                         .publisher_builder()
                         .initial_reserved_memory(1)
                         .allocation_strategy(AllocationStrategy::PowerOfTwo)
                         .create()
                         .value();
    auto subscriber = sut.value().subscriber_builder().create().value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data = produce_example_data(builder, "zombieschlaechter", 78, 9, ARRAY_SIZE); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);
    send(std::move(initialized_sample)).value();

    auto recv_sample_result = subscriber.receive();
    ASSERT_THAT(recv_sample_result.has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value().has_value(), Eq(true));
    const auto* recv_data = recv_sample_result.value()->payload_root();

    ASSERT_STREQ(recv_data->title()->c_str(), "zombieschlaechter");
    ASSERT_EQ(recv_data->entries()->size(), ARRAY_SIZE);
    for (auto i = 0U; i < ARRAY_SIZE; ++i) {
        ASSERT_EQ(recv_data->entries()->Get(i)->data_1(), 78);
        ASSERT_EQ(recv_data->entries()->Get(i)->data_2(), 9);
    }
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest,
           publisher_allocates_more_memory_when_initial_reserve_is_out_with_allocation_strategy_best_fit) {
    constexpr int64_t ARRAY_SIZE = 50;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value()
                         .publisher_builder()
                         .initial_reserved_memory(1)
                         .allocation_strategy(AllocationStrategy::BestFit)
                         .create()
                         .value();
    auto subscriber = sut.value().subscriber_builder().create().value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data =
        produce_example_data(builder, "I am hungry, no I do not want to lick that frog!", 18, 19, ARRAY_SIZE); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);
    send(std::move(initialized_sample)).value();

    auto recv_sample_result = subscriber.receive();
    ASSERT_THAT(recv_sample_result.has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value().has_value(), Eq(true));
    const auto* recv_data = recv_sample_result.value()->payload_root();

    ASSERT_STREQ(recv_data->title()->c_str(), "I am hungry, no I do not want to lick that frog!");
    ASSERT_EQ(recv_data->entries()->size(), ARRAY_SIZE);
    for (auto i = 0U; i < ARRAY_SIZE; ++i) {
        ASSERT_EQ(recv_data->entries()->Get(i)->data_1(), 18);
        ASSERT_EQ(recv_data->entries()->Get(i)->data_2(), 19);
    }
}

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, publisher_does_not_allocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value()
                         .publisher_builder()
                         .initial_reserved_memory(1)
                         .allocation_strategy(AllocationStrategy::Static)
                         .create()
                         .value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();

    // This should fail because Static allocation strategy does not allow reallocations
    // and the initial_reserved_memory is set to 1, which is too small for a string.
    // flatbuffers handles out-of-memory from the allocator with an assertion.
    EXPECT_DEATH(builder.CreateString("oh no more memory"), "");
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest, data_can_be_reconstructed_from_payload_bytes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value().publisher_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto subscriber = sut.value().subscriber_builder().create().value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data = produce_example_data(builder, "are chameleons good at multi-tasking?", 44, 55, 1); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);
    send(std::move(initialized_sample)).value();

    auto recv_sample_result = subscriber.receive();
    ASSERT_THAT(recv_sample_result.has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value().has_value(), Eq(true));
    auto& recv_sample = recv_sample_result.value();
    auto payload_bytes = recv_sample->payload_bytes();
    const auto* recv_data = flatbuffers::GetRoot<Example::UnboundedData>(payload_bytes.data());

    ASSERT_STREQ(recv_data->title()->c_str(), "are chameleons good at multi-tasking?");
    ASSERT_EQ(recv_data->entries()->size(), 1);
    for (auto i = 0U; i < 1; ++i) {
        ASSERT_EQ(recv_data->entries()->Get(i)->data_1(), 44);
        ASSERT_EQ(recv_data->entries()->Get(i)->data_2(), 55);
    }
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest, publisher_can_read_its_own_serialized_data) {
    constexpr int64_t ARRAY_SIZE = 50;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value()
                         .publisher_builder()
                         .initial_reserved_memory(INITIAL_RESERVED_MEMORY)
                         .allocation_strategy(AllocationStrategy::PowerOfTwo)
                         .create()
                         .value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data = produce_example_data(builder, "dib dib dudel dib", 123, 221, ARRAY_SIZE); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);

    const auto* data = initialized_sample.payload_root();

    ASSERT_STREQ(data->title()->c_str(), "dib dib dudel dib");
    ASSERT_EQ(data->entries()->size(), ARRAY_SIZE);
    for (auto i = 0U; i < ARRAY_SIZE; ++i) {
        ASSERT_EQ(data->entries()->Get(i)->data_1(), 123);
        ASSERT_EQ(data->entries()->Get(i)->data_2(), 221);
    }
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) complexity created due to the expansion of the assert macros
TYPED_TEST(ServicePublishSubscribeFlatbufferTest, publish_subscribe_with_user_header_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();

    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<Flatbuffer<Example::UnboundedData>>()
                   .template user_header<uint64_t>()
                   .flatbuffer_schema_path(schema_file)
                   .create();
    ASSERT_THAT(sut.has_value(), Eq(true));

    auto publisher = sut.value().publisher_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto subscriber = sut.value().subscriber_builder().create().value();

    auto sample = publisher.loan_flatbuffer();
    ASSERT_THAT(sample.has_value(), Eq(true));
    auto& builder = sample->flatbuffer_builder();
    auto unbounded_data = produce_example_data(builder, "Weg vom Tisch!", 123, 456, 2); // NOLINT
    auto initialized_sample = assume_init(std::move(*sample), unbounded_data);
    initialized_sample.user_header_mut() = 819231; // NOLINT
    send(std::move(initialized_sample)).value();

    auto recv_sample_result = subscriber.receive();
    ASSERT_THAT(recv_sample_result.has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value().has_value(), Eq(true));
    ASSERT_THAT(recv_sample_result.value()->user_header(), Eq(819231));
    const auto* recv_data = recv_sample_result.value()->payload_root();

    ASSERT_STREQ(recv_data->title()->c_str(), "Weg vom Tisch!");
    ASSERT_EQ(recv_data->entries()->size(), 2);

    for (auto i = 0U; i < 2; ++i) {
        ASSERT_EQ(recv_data->entries()->Get(i)->data_1(), 123);
        ASSERT_EQ(recv_data->entries()->Get(i)->data_2(), 456);
    }
}
} // namespace

#endif // IOX2_FEATURE_FLATBUFFERS
