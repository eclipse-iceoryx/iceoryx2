// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX_HOOFS_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_HPP
#define IOX_HOOFS_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_HPP

#include "iox/string.hpp"

#include <cstdint>

namespace iox
{
namespace detail
{
// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1: Not used as an integer but as actual character.
constexpr char ASCII_A{'a'};
constexpr char ASCII_Z{'z'};
constexpr char ASCII_CAPITAL_A{'A'};
constexpr char ASCII_CAPITAL_Z{'Z'};
constexpr char ASCII_0{'0'};
constexpr char ASCII_9{'9'};
constexpr char ASCII_DASH{'-'};
constexpr char ASCII_DOT{'.'};
constexpr char ASCII_COLON{':'};
constexpr char ASCII_UNDERSCORE{'_'};
// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

enum class RelativePathComponents : uint8_t
{
    Reject,
    Accept
};

/// @brief checks if the given string is a valid path entry. A path entry is the string between
///        two path separators.
/// @note A valid path entry for iceoryx must be platform independent and also supported
///       by various file systems. The file systems we intend to support are
///         * linux: ext3, ext4, btrfs
///         * windows: ntfs, exfat, fat
///         * freebsd: ufs, ffs
///         * apple: apfs
///         * qnx: etfs
///         * android: ext3, ext4, fat
///
///       Sometimes it is also possible that a certain file character is supported by the filesystem
///       itself but not by the platforms SDK. One example are files which end with a dot like "myFile."
///       which are supported by ntfs but not by the Windows SDK.
/// @param[in] name the path entry in question
/// @param[in] relativePathComponents are relative path components are allowed for this path entry
/// @return true if it is valid, otherwise false
template <uint64_t StringCapacity>
bool isValidPathEntry(const string<StringCapacity>& name, const RelativePathComponents relativePathComponents) noexcept;

/// @brief checks if the given string is a valid filename. It must fulfill the
///        requirements of a valid path entry (see, isValidPathEntry) and is not allowed
///        to contain relative path components
/// @param[in] name the string to verify
/// @return true if the string is a filename, otherwise false
template <uint64_t StringCapacity>
bool isValidFileName(const string<StringCapacity>& name) noexcept;

/// @brief verifies if the given string is a valid path to a file
/// @param[in] name the string to verify
/// @return true if the string is a path to a file, otherwise false
template <uint64_t StringCapacity>
bool isValidPathToFile(const string<StringCapacity>& name) noexcept;

/// @brief returns true if the provided name is a valid path, otherwise false
/// @param[in] name the string to verify
template <uint64_t StringCapacity>
bool isValidPathToDirectory(const string<StringCapacity>& name) noexcept;

/// @brief returns true if the provided name ends with a path separator, otherwise false
/// @param[in] name the string which may contain a path separator at the end
template <uint64_t StringCapacity>
bool doesEndWithPathSeparator(const string<StringCapacity>& name) noexcept;

} // namespace detail
} // namespace iox

#include "iox/detail/path_and_file_verifier.inl"

#endif // IOX_HOOFS_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_HPP
