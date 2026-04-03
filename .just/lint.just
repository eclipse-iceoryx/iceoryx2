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

# =============================================================================
# Lint Recipes
# =============================================================================

MARKDOWN_LINT_SCRIPT := justfile_directory() / "internal/scripts/ci_check_markdown_linting.sh"

[private]
_lint-dispatch what *flags:
    #!/usr/bin/env bash
    case "{{what}}" in
        "")
            echo "Usage: just lint <what> [flags]"
            echo ""
            echo "  markdown               lint markdown files"
            echo ""
            echo "Flags:"
            echo "  --fix                  fix linting issues"
            exit 1
            ;;
        markdown) just _lint-markdown {{flags}} ;;
        *) echo "Unknown lint target: {{what}}"; exit 1 ;;
    esac

[private]
_lint-markdown *flags:
    bash {{MARKDOWN_LINT_SCRIPT}} {{flags}}
