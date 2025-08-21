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

#ifndef IOX2_FILE_DESCRIPTOR_HPP
#define IOX2_FILE_DESCRIPTOR_HPP

#include "iox/optional.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class FileDescriptorView;

/// Abstract class that can be implemented by a class that is based on a [`FileDescriptor`]
class FileDescriptorBased {
  public:
    FileDescriptorBased() = default;
    FileDescriptorBased(const FileDescriptorBased&) = default;
    FileDescriptorBased(FileDescriptorBased&&) = default;
    auto operator=(const FileDescriptorBased&) -> FileDescriptorBased& = default;
    auto operator=(FileDescriptorBased&&) -> FileDescriptorBased& = default;
    virtual ~FileDescriptorBased() = default;

    /// Returns a [`FileDescriptorView`] to the underlying [`FileDescriptor`].
    virtual auto file_descriptor() const -> FileDescriptorView = 0;
};

/// A view to a [`FileDescriptor`].
class FileDescriptorView : public FileDescriptorBased {
  public:
    /// Returns a [`FileDescriptorView`] to the underlying [`FileDescriptor`].
    auto file_descriptor() const -> FileDescriptorView override;

    /// Returns the underlying [`FileDescriptor`] value.
    ///
    /// # Safety
    ///
    ///  * the user shall not store the value in a variable otherwise lifetime issues may be
    ///    encountered
    ///  * do not manually close the file descriptor with a sys call
    ///
    auto unsafe_native_handle() const -> int32_t;

  private:
    template <ServiceType>
    friend class WaitSet;
    friend class FileDescriptor;
    template <ServiceType>
    friend class Listener;

    explicit FileDescriptorView(iox2_file_descriptor_ptr handle);

    iox2_file_descriptor_ptr m_handle = nullptr;
};

/// Contains a [`FileDescriptor`] that will be closed when the object owns the descriptor and
/// goes out of scope.
class FileDescriptor {
  public:
    /// Creates a new [`FileDescriptor`] object that owns it. If the provided value is an
    /// invalid [`FileDescriptor`] it returns [`iox::nullopt`].
    static auto create_owning(int32_t file_descriptor) -> iox::optional<FileDescriptor>;

    /// Creates a new [`FileDescriptor`] object that does not own it. If the provided value is an
    /// invalid [`FileDescriptor`] it returns [`iox::nullopt`].
    static auto create_non_owning(int32_t file_descriptor) -> iox::optional<FileDescriptor>;

    FileDescriptor(const FileDescriptor&) = delete;
    auto operator=(const FileDescriptor&) -> FileDescriptor& = delete;

    FileDescriptor(FileDescriptor&& rhs) noexcept;
    auto operator=(FileDescriptor&& rhs) noexcept -> FileDescriptor&;
    ~FileDescriptor();

    /// Returns the underlying [`FileDescriptor`] value.
    ///
    /// # Safety
    ///
    ///  * the user shall not store the value in a variable otherwise lifetime issues may be
    ///    encountered
    ///  * do not manually close the file descriptor with a sys call
    ///
    auto unsafe_native_handle() const -> int32_t;

    /// Creates a [`FileDescriptorView`] out of the [`FileDescriptor`]. The view is only valid as
    /// long as the [`FileDescriptor`] is living - otherwise it will be a dangling view.
    auto as_view() const -> FileDescriptorView;

  private:
    explicit FileDescriptor(iox2_file_descriptor_h handle);
    void drop();

    iox2_file_descriptor_h m_handle = nullptr;
};

} // namespace iox2

#endif
