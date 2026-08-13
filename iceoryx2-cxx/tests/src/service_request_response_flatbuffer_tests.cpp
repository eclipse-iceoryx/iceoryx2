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
#include "iox2/legacy/error_reporting/error_kind.hpp"

#if IOX2_FEATURE_FLATBUFFERS

#include "iox2/allocation_strategy.hpp"
#include "iox2/bb/file_name.hpp"
#include "iox2/bb/optional.hpp"
#include "iox2/bb/static_string.hpp"
#include "iox2/bb/testing/fatal_failure.hpp"
#include "iox2/node.hpp"
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
class ServiceRequestResponseFlatbufferTest : public ::testing::Test {
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

TYPED_TEST_SUITE(ServiceRequestResponseFlatbufferTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceRequestResponseFlatbufferTest, create_fails_when_no_request_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name).template request_response<Flatbuffer<uint64_t>, uint64_t>().create();

    ASSERT_THAT(sut.error(), Eq(RequestResponseCreateError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, create_fails_when_no_response_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name).template request_response<uint64_t, Flatbuffer<uint64_t>>().create();

    ASSERT_THAT(sut.error(), Eq(RequestResponseCreateError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, create_succeeds_with_request_schema_file) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<uint64_t>, uint64_t>()
                   .request_flatbuffer_schema_path(schema_file)
                   .create();

    ASSERT_THAT(sut.has_value(), Eq(true));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, create_succeeds_with_response_schema_file) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file = this->create_schema_file(SCHEMA);
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<uint64_t, Flatbuffer<uint64_t>>()
                   .response_flatbuffer_schema_path(schema_file)
                   .create();

    ASSERT_THAT(sut.has_value(), Eq(true));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, open_fails_when_no_request_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file_1 = this->create_schema_file(SCHEMA);
    auto schema_file_2 = this->create_schema_file(ALT_SCHEMA);

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                          .request_flatbuffer_schema_path(schema_file_1)
                          .response_flatbuffer_schema_path(schema_file_2)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                        .response_flatbuffer_schema_path(schema_file_2)
                        .open();

    ASSERT_THAT(sut_open.error(), Eq(RequestResponseOpenError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, open_fails_when_no_response_schema_file_is_available) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file_1 = this->create_schema_file(SCHEMA);
    auto schema_file_2 = this->create_schema_file(ALT_SCHEMA);

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                          .request_flatbuffer_schema_path(schema_file_1)
                          .response_flatbuffer_schema_path(schema_file_2)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                        .request_flatbuffer_schema_path(schema_file_1)
                        .open();

    ASSERT_THAT(sut_open.error(), Eq(RequestResponseOpenError::UnableToAcquireTypeDefinition));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, open_fails_when_request_schema_is_not_the_same) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file_1 = this->create_schema_file(SCHEMA);
    auto schema_file_2 = this->create_schema_file(ALT_SCHEMA);

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                          .request_flatbuffer_schema_path(schema_file_1)
                          .response_flatbuffer_schema_path(schema_file_2)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                        .request_flatbuffer_schema_path(schema_file_2)
                        .response_flatbuffer_schema_path(schema_file_2)
                        .open();

    ASSERT_THAT(sut_open.error(), Eq(RequestResponseOpenError::IncompatibleRequestOrResponseType));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, open_fails_when_response_schema_is_not_the_same) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file_1 = this->create_schema_file(SCHEMA);
    auto schema_file_2 = this->create_schema_file(ALT_SCHEMA);

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                          .request_flatbuffer_schema_path(schema_file_1)
                          .response_flatbuffer_schema_path(schema_file_2)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                        .request_flatbuffer_schema_path(schema_file_1)
                        .response_flatbuffer_schema_path(schema_file_1)
                        .open();

    ASSERT_THAT(sut_open.error(), Eq(RequestResponseOpenError::IncompatibleRequestOrResponseType));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, open_succeeds_when_schema_content_is_identical) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    auto schema_file_1 = this->create_schema_file(SCHEMA);
    auto schema_file_2 = this->create_schema_file(ALT_SCHEMA);

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                          .request_flatbuffer_schema_path(schema_file_1)
                          .response_flatbuffer_schema_path(schema_file_2)
                          .create();

    auto sut_open = node.service_builder(service_name)
                        .template request_response<Flatbuffer<int32_t>, Flatbuffer<uint64_t>>()
                        .request_flatbuffer_schema_path(schema_file_1)
                        .response_flatbuffer_schema_path(schema_file_2)
                        .open();

    ASSERT_THAT(sut_open.has_value(), Eq(true));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, schema_path_lookup_works_when_creating_a_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create();

    ASSERT_THAT(sut.has_value(), Eq(true));
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, schema_path_lookup_works_when_opening_a_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut_create =
        node.service_builder(service_name)
            .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
            .create();

    auto sut_open =
        node.service_builder(service_name)
            .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
            .open();

    ASSERT_THAT(sut_open.has_value(), Eq(true));
}

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

TYPED_TEST(ServiceRequestResponseFlatbufferTest, request_response_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto server = sut.server_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();
    auto unbounded_data = produce_example_data(request_builder, "nala sleeps tonight", 91, 19, 19); // NOLINT
    auto initialized_request = assume_init(std::move(request), unbounded_data);
    auto pending_response = send(std::move(initialized_request)).value();

    // receive request
    auto active_request = server.receive().value();
    ASSERT_THAT(active_request.has_value(), Eq(true));
    const auto* request_data = active_request->payload_root();

    ASSERT_STREQ(request_data->title()->c_str(), "nala sleeps tonight");
    ASSERT_EQ(request_data->entries()->size(), 19);

    for (auto i = 0U; i < 19U; ++i) { // NOLINT
        ASSERT_EQ(request_data->entries()->Get(i)->data_1(), 91);
        ASSERT_EQ(request_data->entries()->Get(i)->data_2(), 19);
    }

    // send response
    auto response = active_request->loan_flatbuffer().value();
    auto& response_builder = response.flatbuffer_builder();
    unbounded_data =
        produce_example_data(response_builder, "Nala: I will leave a little greeting here.", 14, 41, 14); // NOLINT
    auto initialized_response = assume_init(std::move(response), unbounded_data);
    send(std::move(initialized_response)).value();

    // receive response
    auto recv_response = pending_response.receive().value();
    ASSERT_THAT(recv_response.has_value(), Eq(true));
    const auto* response_data = recv_response->payload_root();

    ASSERT_STREQ(response_data->title()->c_str(), "Nala: I will leave a little greeting here.");

    ASSERT_EQ(response_data->entries()->size(), 14);

    for (auto i = 0U; i < 14U; ++i) { // NOLINT
        ASSERT_EQ(response_data->entries()->Get(i)->data_1(), 14);
        ASSERT_EQ(response_data->entries()->Get(i)->data_2(), 41);
    }
}

// NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseFlatbufferTest, server_and_client_allocate_more_memory_when_reserve_is_out) {
    for (auto allocation_strategy : { AllocationStrategy::PowerOfTwo, AllocationStrategy::BestFit }) {
        constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
        this->create_schema_file(SCHEMA, "unbounded_data.fbs");

        auto config = Config();
        config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
        auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
        auto service_name = iox2::testing::generate_service_name();
        auto sut =
            node.service_builder(service_name)
                .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                .create()
                .value();

        auto client =
            sut.client_builder().initial_reserved_memory(1).allocation_strategy(allocation_strategy).create().value();
        auto server =
            sut.server_builder().initial_reserved_memory(1).allocation_strategy(allocation_strategy).create().value();

        auto request = client.loan_flatbuffer().value();
        auto& request_builder = request.flatbuffer_builder();
        auto unbounded_data =
            produce_example_data(request_builder, "put your nose up in the air", 119, 991, 49); // NOLINT
        auto initialized_request = assume_init(std::move(request), unbounded_data);
        auto pending_response = send(std::move(initialized_request)).value();

        // receive request
        auto active_request = server.receive().value();
        ASSERT_THAT(active_request.has_value(), Eq(true));
        const auto* request_data = active_request->payload_root();

        ASSERT_STREQ(request_data->title()->c_str(), "put your nose up in the air");
        ASSERT_EQ(request_data->entries()->size(), 49);

        for (auto i = 0U; i < 49U; ++i) { // NOLINT
            ASSERT_EQ(request_data->entries()->Get(i)->data_1(), 119);
            ASSERT_EQ(request_data->entries()->Get(i)->data_2(), 991);
        }

        // send response
        auto response = active_request->loan_flatbuffer().value();
        auto& response_builder = response.flatbuffer_builder();
        unbounded_data =
            produce_example_data(response_builder, "And sniff as you just don't care!", 114, 441, 42); // NOLINT
        auto initialized_response = assume_init(std::move(response), unbounded_data);
        send(std::move(initialized_response)).value();

        // receive response
        auto recv_response = pending_response.receive().value();
        ASSERT_THAT(recv_response.has_value(), Eq(true));
        const auto* response_data = recv_response->payload_root();

        ASSERT_STREQ(response_data->title()->c_str(), "And sniff as you just don't care!");

        ASSERT_EQ(response_data->entries()->size(), 42);

        for (auto i = 0U; i < 42U; ++i) { // NOLINT
            ASSERT_EQ(response_data->entries()->Get(i)->data_1(), 114);
            ASSERT_EQ(response_data->entries()->Get(i)->data_2(), 441);
        }
    }
}
// NOLINTEND(readability-function-cognitive-complexity)

// NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseFlatbufferTest,
           server_and_client_with_user_header_allocate_more_memory_when_reserve_is_out) {
    for (auto allocation_strategy : { AllocationStrategy::PowerOfTwo, AllocationStrategy::BestFit }) {
        constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
        this->create_schema_file(SCHEMA, "unbounded_data.fbs");

        auto config = Config();
        config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
        auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
        auto service_name = iox2::testing::generate_service_name();
        auto sut =
            node.service_builder(service_name)
                .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                .template request_user_header<uint64_t>()
                .template response_user_header<uint64_t>()
                .create()
                .value();

        auto client =
            sut.client_builder().initial_reserved_memory(1).allocation_strategy(allocation_strategy).create().value();
        auto server =
            sut.server_builder().initial_reserved_memory(1).allocation_strategy(allocation_strategy).create().value();

        auto request = client.loan_flatbuffer().value();
        auto& request_builder = request.flatbuffer_builder();
        auto unbounded_data =
            produce_example_data(request_builder, "put your nose up in the air", 119, 991, 49); // NOLINT
        auto initialized_request = assume_init(std::move(request), unbounded_data);
        initialized_request.user_header_mut() = 99191; // NOLINT
        auto pending_response = send(std::move(initialized_request)).value();

        // receive request
        auto active_request = server.receive().value();
        ASSERT_THAT(active_request.has_value(), Eq(true));
        const auto* request_data = active_request->payload_root();

        ASSERT_EQ(active_request->user_header(), 99191);
        ASSERT_STREQ(request_data->title()->c_str(), "put your nose up in the air");
        ASSERT_EQ(request_data->entries()->size(), 49);

        for (auto i = 0U; i < 49U; ++i) { // NOLINT
            ASSERT_EQ(request_data->entries()->Get(i)->data_1(), 119);
            ASSERT_EQ(request_data->entries()->Get(i)->data_2(), 991);
        }

        // send response
        auto response = active_request->loan_flatbuffer().value();
        auto& response_builder = response.flatbuffer_builder();
        unbounded_data =
            produce_example_data(response_builder, "And sniff as you just don't care!", 114, 441, 42); // NOLINT
        auto initialized_response = assume_init(std::move(response), unbounded_data);
        initialized_response.user_header_mut() = 11919; // NOLINT
        send(std::move(initialized_response)).value();

        // receive response
        auto recv_response = pending_response.receive().value();
        ASSERT_THAT(recv_response.has_value(), Eq(true));
        const auto* response_data = recv_response->payload_root();

        ASSERT_EQ(recv_response->user_header(), 11919);
        ASSERT_STREQ(response_data->title()->c_str(), "And sniff as you just don't care!");

        ASSERT_EQ(response_data->entries()->size(), 42);

        for (auto i = 0U; i < 42U; ++i) { // NOLINT
            ASSERT_EQ(response_data->entries()->Get(i)->data_1(), 114);
            ASSERT_EQ(response_data->entries()->Get(i)->data_2(), 441);
        }
    }
}
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServiceRequestResponseFlatbufferTest, server_and_client_data_can_be_reconstructed_from_payload_bytes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto server = sut.server_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();
    auto unbounded_data = produce_example_data(request_builder, "put your nose up in the air", 119, 991, 3); // NOLINT
    auto initialized_request = assume_init(std::move(request), unbounded_data);
    auto pending_response = send(std::move(initialized_request)).value();

    // receive request
    auto active_request = server.receive().value();
    ASSERT_THAT(active_request.has_value(), Eq(true));
    const auto* request_data = flatbuffers::GetRoot<Example::UnboundedData>(active_request->payload_bytes().data());

    ASSERT_STREQ(request_data->title()->c_str(), "put your nose up in the air");
    ASSERT_EQ(request_data->entries()->size(), 3);

    for (auto i = 0U; i < 3U; ++i) { // NOLINT
        ASSERT_EQ(request_data->entries()->Get(i)->data_1(), 119);
        ASSERT_EQ(request_data->entries()->Get(i)->data_2(), 991);
    }

    // send response
    auto response = active_request->loan_flatbuffer().value();
    auto& response_builder = response.flatbuffer_builder();
    unbounded_data = produce_example_data(response_builder, "And sniff as you just don't care!", 114, 441, 2); // NOLINT
    auto initialized_response = assume_init(std::move(response), unbounded_data);
    send(std::move(initialized_response)).value();

    // receive response
    auto recv_response = pending_response.receive().value();
    ASSERT_THAT(recv_response.has_value(), Eq(true));
    const auto* response_data = flatbuffers::GetRoot<Example::UnboundedData>(recv_response->payload_bytes().data());

    ASSERT_STREQ(response_data->title()->c_str(), "And sniff as you just don't care!");

    ASSERT_EQ(response_data->entries()->size(), 2);

    for (auto i = 0U; i < 2U; ++i) { // NOLINT
        ASSERT_EQ(response_data->entries()->Get(i)->data_1(), 114);
        ASSERT_EQ(response_data->entries()->Get(i)->data_2(), 441);
    }
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, client_can_read_its_own_payload) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto server = sut.server_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();
    auto unbounded_data = produce_example_data(request_builder, "run nala run", 1119, 1991, 2); // NOLINT
    auto initialized_request = assume_init(std::move(request), unbounded_data);
    auto pending_response = send(std::move(initialized_request)).value();

    const auto* request_data = pending_response.payload_root();

    ASSERT_STREQ(request_data->title()->c_str(), "run nala run");
    ASSERT_EQ(request_data->entries()->size(), 2);

    for (auto i = 0U; i < 2U; ++i) { // NOLINT
        ASSERT_EQ(request_data->entries()->Get(i)->data_1(), 1119);
        ASSERT_EQ(request_data->entries()->Get(i)->data_2(), 1991);
    }
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, server_can_read_its_own_payload) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto server = sut.server_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();
    auto unbounded_data = produce_example_data(request_builder, "", 0, 0, 1); // NOLINT
    auto initialized_request = assume_init(std::move(request), unbounded_data);
    auto pending_response = send(std::move(initialized_request)).value();

    // receive request
    auto active_request = server.receive().value();
    ASSERT_THAT(active_request.has_value(), Eq(true));

    // send response
    auto response = active_request->loan_flatbuffer().value();
    auto& response_builder = response.flatbuffer_builder();
    unbounded_data = produce_example_data(response_builder, "Snooze the sniffles", 2114, 2441, 2); // NOLINT
    auto initialized_response = assume_init(std::move(response), unbounded_data);

    const auto* response_data = initialized_response.payload_root();

    ASSERT_STREQ(response_data->title()->c_str(), "Snooze the sniffles");

    ASSERT_EQ(response_data->entries()->size(), 2);

    for (auto i = 0U; i < 2U; ++i) { // NOLINT
        ASSERT_EQ(response_data->entries()->Get(i)->data_1(), 2114);
        ASSERT_EQ(response_data->entries()->Get(i)->data_2(), 2441);
    }
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, client_does_not_allocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder()
                      .initial_reserved_memory(1)
                      .allocation_strategy(AllocationStrategy::Static)
                      .create()
                      .value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();

    // This should fail because Static allocation strategy does not allow reallocations
    // and the initial_reserved_memory is set to 1, which is too small for a string.
    // flatbuffers handles out-of-memory from the allocator with an assertion.
    IOX2_TESTING_EXPECT_FATAL_FAILURE([&]() -> auto { request_builder.CreateString("oh no more memory"); },
                                      iox2::legacy::er::ASSERT_VIOLATION);
}

TYPED_TEST(ServiceRequestResponseFlatbufferTest, server_does_not_allocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA, "unbounded_data.fbs");

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(iox2::testing::test_directory_path());
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();
    auto service_name = iox2::testing::generate_service_name();
    auto sut = node.service_builder(service_name)
                   .template request_response<Flatbuffer<Example::UnboundedData>, Flatbuffer<Example::UnboundedData>>()
                   .create()
                   .value();

    auto client = sut.client_builder().initial_reserved_memory(INITIAL_RESERVED_MEMORY).create().value();
    auto server = sut.server_builder()
                      .initial_reserved_memory(1)
                      .allocation_strategy(AllocationStrategy::Static)
                      .create()
                      .value();

    auto request = client.loan_flatbuffer().value();
    auto& request_builder = request.flatbuffer_builder();
    auto unbounded_data = produce_example_data(request_builder, "", 0, 0, 0); // NOLINT
    auto initialized_request = assume_init(std::move(request), unbounded_data);
    auto pending_response = send(std::move(initialized_request)).value();

    // receive request
    auto active_request = server.receive().value();
    ASSERT_THAT(active_request.has_value(), Eq(true));
    auto response = active_request->loan_flatbuffer().value();
    auto& response_builder = response.flatbuffer_builder();

    // This should fail because Static allocation strategy does not allow reallocations
    // and the initial_reserved_memory is set to 1, which is too small for a string.
    // flatbuffers handles out-of-memory from the allocator with an assertion.
    IOX2_TESTING_EXPECT_FATAL_FAILURE([&]() -> auto { response_builder.CreateString("oh no more memory"); },
                                      iox2::legacy::er::ASSERT_VIOLATION);
}
} // namespace

#endif // IOX2_FEATURE_FLATBUFFERS
