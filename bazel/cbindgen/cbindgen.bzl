# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

"""
cbindgen generator rules
"""

load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")

def _cbindgen_impl(ctx):
    rust_toolchain = ctx.toolchains["@rules_rust//rust:toolchain_type"]

    inputs = depset(
        direct = ctx.files.srcs,
        transitive = [rust_toolchain.all_files],
    )

    # Extract directory from manifest file
    manifest_path = ctx.file.manifest_dir.path
    manifest_dir = "/".join(manifest_path.split("/")[:-1])

    # Get cargo bin directory for PATH
    cargo_bin_dir = "/".join(rust_toolchain.cargo.path.split("/")[:-1])

    args = [
        manifest_dir,
        "--quiet",
        "--config",
        ctx.file.config.path,
        "--output",
        ctx.outputs.header.path,
    ]

    ctx.actions.run(
        executable = ctx.executable.cbindgen,
        arguments = args,
        inputs = inputs,
        outputs = [ctx.outputs.header],
        env = {
            "PATH": cargo_bin_dir,
            "CARGO_HOME": ctx.genfiles_dir.path + "/.cargo"
        },
    )

    # Get include directory (parent of header file directory)
    include_dir = "/".join(ctx.outputs.header.path.split("/")[:-2])

    return [
        DefaultInfo(files = depset([ctx.outputs.header])),
        CcInfo(
            compilation_context = cc_common.create_compilation_context(
                headers = depset([ctx.outputs.header]),
                includes = depset([include_dir]),
            ),
        ),
    ]

rust_cbindgen_headers = rule(
    implementation = _cbindgen_impl,
    attrs = {
        "cbindgen": attr.label(
            executable = True,
            cfg = "exec",
            default = "@crate_index//:cbindgen__cbindgen",
            doc = "The cbindgen executable. Defaults to the version from crate_index.",
        ),
        "manifest_dir": attr.label(
            allow_single_file = True,
            doc = "Label pointing to the Cargo.toml manifest file.",
        ),
        "config": attr.label(
            allow_single_file = True,
            doc = "Label pointing to the cbindgen configuration file (cbindgen.toml).",
        ),
        "srcs": attr.label_list(
            allow_files = True,
            doc = "List of Rust source files to analyze for header generation.",
        ),
        "header": attr.output(
            doc = "Output path for the generated header file."
        ),
    },
    toolchains = ["@rules_rust//rust:toolchain_type"],
)
