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

import '.just/paths.just'
import '.just/common.just'
import '.just/ros2.just'
import '.just/build.just'
import '.just/test.just'
import '.just/test-e2e.just'
import '.just/doc.just'
import '.just/bundle.just'
import '.just/verify.just'
import '.just/lint.just'
import '.just/setup.just'
import '.just/coverage.just'
import '.just/publish.just'
import '.just/prepare-release.just'

# Show available commands
default:
    @echo "iceoryx2 development helpers"
    @echo ""
    @echo "Commands:"
    @echo "  setup           - Setup dependencies"
    @echo "  build           - Build workspace or a specific package"
    @echo "  test            - Run tests for workspace or a specific package"
    @echo "  test-e2e        - Run end-to-end tests for a workspace"
    @echo "  doc             - Build API documentation"
    @echo "  bundle          - Bundle tests for deployment"
    @echo "  verify          - Run verification checks"
    @echo "  lint            - Run linting checks"
    @echo "  coverage        - Run test coverage tasks"
    @echo "  prepare-release - Run release preparation tasks"
    @echo "  publish         - Publish crates to crates.io"
    @echo ""
    @echo "Run 'just <command>' for usage details on each command."

[no-exit-message]
build what="" *flags:
    @just _build-dispatch "{{what}}" {{flags}}

[no-exit-message]
test what="" *flags:
    @just _test-dispatch "{{what}}" {{flags}}

[no-exit-message]
test-e2e what="" *flags:
    #!/usr/bin/env bash
    # Keeps a glob passed to a flag from expanding against the working directory.
    set -f
    just _test-e2e-dispatch "{{what}}" {{flags}}

[no-exit-message]
doc workspace="" target="" *flags:
    @just _doc-dispatch "{{workspace}}" "{{target}}" {{flags}}

[no-exit-message]
bundle what="" *flags:
    @just _bundle-dispatch "{{what}}" {{flags}}

[no-exit-message]
verify workspace="" target="" *flags:
    @just _verify-dispatch "{{workspace}}" "{{target}}" {{flags}}

[no-exit-message]
lint workspace="" target="" *flags:
    @just _lint-dispatch "{{workspace}}" "{{target}}" {{flags}}

[no-exit-message]
setup what="":
    @just _setup-dispatch "{{what}}"

[no-exit-message]
coverage action="" *flags:
    @just _coverage-dispatch "{{action}}" {{flags}}

[no-exit-message]
publish workspace="" *flags:
    @just _publish-dispatch "{{workspace}}" {{flags}}

[no-exit-message]
prepare-release workspace="" action="" *flags:
    @just _prepare-release-dispatch "{{workspace}}" "{{action}}" {{flags}}
