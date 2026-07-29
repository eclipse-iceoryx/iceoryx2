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

#include "iox2/bb/optional.hpp"
#include "iox2/custom_header_marker.hpp"
#include "iox2/custom_payload_marker.hpp"
#include "iox2/legacy/uninitialized_array.hpp"
#include "iox2/message_type_details.hpp"
#include "iox2/node.hpp"
#include "iox2/service.hpp"
#include "iox2/testing.hpp"
#include "iox2/type_variant.hpp"

#include "test.hpp"
#include <array>
#include <cstdint>
#include <cstdio>
#include <flatbuffers/flatbuffers.h>
#include <fstream>
#include <gtest/gtest.h>

namespace {
using namespace iox2;

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

const char* SCHEMA_FILE = R"(
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

template <typename T>
class ServicePublishSubscribeFlatbufferTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;


    ServicePublishSubscribeFlatbufferTest()
        : m_schema_file { iox2::testing::generate_file_path() } {
        iox2::testing::create_test_directory();
    }

    ~ServicePublishSubscribeFlatbufferTest() override {
        static_cast<void>(std::remove(m_schema_file.as_string().unchecked_access().c_str()));
    }

    void create_schema_file(const char* content) {
        std::ofstream file(m_schema_file.as_string().unchecked_access().c_str());
        EXPECT_THAT(file.is_open(), Eq(true));
        if (file.is_open()) {
            file << content;
        }
    }

  private:
    bb::FilePath m_schema_file;
};

TYPED_TEST_SUITE(ServicePublishSubscribeFlatbufferTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServicePublishSubscribeFlatbufferTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    this->create_schema_file(SCHEMA_FILE);
}
} // namespace
