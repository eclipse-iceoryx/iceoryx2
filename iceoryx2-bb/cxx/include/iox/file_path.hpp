// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_POSIX_VOCABULARY_FILE_PATH_HPP
#define IOX_HOOFS_POSIX_VOCABULARY_FILE_PATH_HPP

#include "iox/semantic_string.hpp"

namespace iox
{
namespace detail
{
bool file_path_does_contain_invalid_characters(const string<platform::IOX_MAX_PATH_LENGTH>& value) noexcept;
bool file_path_does_contain_invalid_content(const string<platform::IOX_MAX_PATH_LENGTH>& value) noexcept;
} // namespace detail

/// @brief Represents a path to a file. It is not allowed to end with a path separator
///        since this would then be a path to a directory. A valid file path is for
///        instance "path/to/file" but not "path/to/file/".
class FilePath : public SemanticString<FilePath,
                                       platform::IOX_MAX_PATH_LENGTH,
                                       detail::file_path_does_contain_invalid_content,
                                       detail::file_path_does_contain_invalid_characters>
{
    using Parent = SemanticString<FilePath,
                                  platform::IOX_MAX_PATH_LENGTH,
                                  detail::file_path_does_contain_invalid_content,
                                  detail::file_path_does_contain_invalid_characters>;
    using Parent::Parent;
};
} // namespace iox

#endif
