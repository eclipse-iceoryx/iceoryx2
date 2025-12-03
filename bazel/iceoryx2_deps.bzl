# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Module extensions providing custom repositories for iceoryx2.

load("@bazel_skylib//lib:modules.bzl", "modules")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive", "http_file")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

BINDGEN_VERSION = "v0.69.5"
CBINDGEN_VERSION = "0.26.0"

def _iceoryx2_extra_deps():
    maybe(
        repo_rule = http_archive,
        name = "bindgen",
        sha256 = "b7e2321ee8c617f14ccc5b9f39b3a804db173ee217e924ad93ed16af6bc62b1d",
        strip_prefix = "bindgen-cli-x86_64-unknown-linux-gnu",
        urls = ["https://github.com/rust-lang/rust-bindgen/releases/download/{version}/bindgen-cli-x86_64-unknown-linux-gnu.tar.xz".format(version = BINDGEN_VERSION)],
        build_file_content = """
filegroup(
    name = "bindgen-cli",
    srcs = ["bindgen"],
    visibility = ["//visibility:public"],
)
""",
    )
    maybe(
        repo_rule = http_file,
        name = "cbindgen",
        sha256 = "521836d00863cb129283054e5090eb17563614e6328b7a1610e30949a05feaea",
        urls = ["https://github.com/mozilla/cbindgen/releases/download/{version}/cbindgen".format(version = CBINDGEN_VERSION)],
        executable = True,
    )

iceoryx2_extra_deps = modules.as_extension(_iceoryx2_extra_deps)
