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
import '.just/coverage.just'

# Show available commands
default:
    @echo "iceoryx2 development helpers"
    @echo ""
    @echo "Commands:"
    @echo "  setup         - Setup dependencies"
    @echo "  build         - Build workspace or a specific package"
    @echo "  test          - Run tests for workspace or a specific package"
    @echo "  bundle        - Bundle tests for deployment"
    @echo "  verify        - Run verification checks"
    @echo "  lint          - Run linting checks"
    @echo "  coverage      - Run test coverage tasks"
    @echo ""
    @echo "Run 'just <command>' for usage details on each command."

build what="" *flags:
    @just _build-dispatch "{{what}}" {{flags}}

test what="" *flags:
    @just _test-dispatch "{{what}}" {{flags}}

bundle what="" *flags:
    @just _bundle-dispatch "{{what}}" {{flags}}

verify what="" *flags:
    @just _verify-dispatch "{{what}}" {{flags}}

lint what="" *flags:
    @just _lint-dispatch "{{what}}" {{flags}}

setup what="":
    @just _setup-dispatch "{{what}}"

coverage action="" *flags:
    @just _coverage-dispatch "{{action}}" {{flags}}
