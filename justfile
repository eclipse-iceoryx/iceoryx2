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

import '.just/common.just'
import '.just/build.just'
import '.just/test.just'
import '.just/bundle.just'
import '.just/verify.just'
import '.just/lint.just'
import '.just/setup.just'

# Show available commands and usage examples
default:
    @echo "iceoryx2 development helpers"
    @echo ""

    @echo "Usage:"
    @echo "  just setup all                              # Setup all dependencies for the workspace"
    @echo "  just setup script-dependencies              # Setup dependencies required for just scripts"
    @echo ""
    @echo "  just build workspace                        # Build all crates in the workspace (default features)"
    @echo "  just build tests                            # Build all tests in the workspace (standard framework, default features)"
    @echo "  just build tests --no_std [+toolchain]      # Build all no_std tests in the workspace (custom framework, no_std)"
    @echo "  just build <package>                        # Build specific package (default features)"
    @echo "  just build <package> --no_std [+toolchain]  # Build package (no_std)"
    @echo ""
    @echo "  just test workspace                         # Run tests in the workspace (standard framework, default features)"
    @echo "  just test workspace --no_std [+toolchain]   # Run all workspace tests (custom framework, no_std)"
    @echo "  just test <package>                         # Run tests for specific package (standard framework, default features)"
    @echo "  just test <package> --no_std [+toolchain]   # Run tests for specific package (custom framework, no_std)"
    @echo ""
    @echo "  just bundle tests --no_std [+toolchain] [--target=<triplet>] [--strip] [--compress]"
    @echo "                                              # Build all no_std tests in the workspace and bundle for deployment"
    @echo "                                              # +toolchain: Rust toolchain (e.g., +nightly, +stable)"
    @echo "                                              # --target=<triplet>: Target triplet (e.g., x86_64-unknown-linux-gnu)"
    @echo "                                              # --strip: Strip debug symbols from binaries"
    @echo "                                              # --compress: Create a compressed tarball"
    @echo ""
    @echo "  just verify std-propagation workspace       # Verify std feature propagation for all crates"
    @echo "  just verify std-propagation <crate>         # Verify std feature propagation for a specific crate"
    @echo ""
    @echo "  just lint markdown                          # Check markdown linting"
    @echo "  just lint markdown --fix                    # Fix markdown linting issues"
    @echo ""
    @echo "Examples:"
    @echo "  just setup all"
    @echo "  just build workspace"
    @echo "  just build tests"
    @echo "  just build tests --no_std +nightly"
    @echo "  just build iceoryx2 --no_std"
    @echo "  just build iceoryx2-bb-elementary --no_std"
    @echo "  just test iceoryx2-bb-elementary --no_std"
    @echo "  just test iceoryx2-bb-elementary --no_std +nightly"
    @echo "  just bundle tests --no_std --strip --compress"
    @echo "  just bundle tests --no_std +nightly --target=aarch64-unknown-linux-gnu --strip"
    @echo "  just bundle tests --no_std --target=x86_64-pc-nto-qnx800 +qnx800 --compress"
    @echo "  just verify std-propagation workspace"
    @echo "  just verify std-propagation iceoryx2-bb-posix"
    @echo "  just lint markdown"
    @echo ""
    @echo "Run 'just list' to see all available recipes"

# Build workspace or a specific package
build target *flags="":
    @just _build-dispatch "{{target}}" {{flags}}

# Run tests for workspace or a specific package
test target *flags="":
    @just _test-dispatch "{{target}}" {{flags}}

# Bundle tests for deployment
bundle target *flags="":
    @just _bundle-dispatch "{{target}}" {{flags}}

# Run verification checks
verify target *flags:
    @just _verify-dispatch "{{target}}" {{flags}}

# Run linting checks
lint target *flags:
    @just _lint-dispatch "{{target}}" {{flags}}

# Setup tasks
setup target:
    @just _setup-dispatch "{{target}}"

# List all available recipes
list:
    @just --list
