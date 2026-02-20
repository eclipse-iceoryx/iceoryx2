# =============================================================================
# Bundle Recipes
# =============================================================================

[private]
_bundle-dispatch target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{target}}" != "tests" ]; then
        echo "Error: bundle command currently only supports 'tests' as target" >&2
        exit 1
    fi

    eval $(just _parse-bundle-flags {{flags}})

    if [ "$HAS_NOSTD" != "true" ]; then
        echo "Error: bundle command currently only supports 'tests --no_std'" >&2
        exit 1
    fi

    just _bundle-nostd-tests "$TOOLCHAIN" "$TARGET_TRIPLET" "$DO_STRIP" "$DO_COMPRESS"

[private]
_bundle-nostd-tests toolchain target_triplet do_strip do_compress:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -n "{{target_triplet}}" ]; then
        TARGET_DIR="target/{{target_triplet}}/debug"
        echo "Using cross-compilation target: {{target_triplet}}"
    else
        DEFAULT_TARGET=$(just _detect-host-target "{{toolchain}}")
        TARGET_DIR="target/$DEFAULT_TARGET/debug"
        echo "Using native target: $DEFAULT_TARGET"
    fi

    if [ -n "{{toolchain}}" ]; then
        echo "Using toolchain: {{toolchain}}"
    fi

    just _build-tests "{{toolchain}}" "true" "{{target_triplet}}"

    BUNDLE_DIR="target/{{target_triplet}}/tests-nostd"
    just _create-bundle-dir "$TARGET_DIR" "$BUNDLE_DIR"
    just _create-run-all-script "$BUNDLE_DIR"

    if [ "{{do_strip}}" = "true" ]; then
        just _strip-binaries "$BUNDLE_DIR"
    fi

    echo ""
    echo "Bundle created at: $BUNDLE_DIR"
    ls -lh "$BUNDLE_DIR"

    if [ "{{do_compress}}" = "true" ]; then
        just _compress-bundle "$BUNDLE_DIR" "{{target_triplet}}"
    fi

[private]
_create-bundle-dir target_dir bundle_dir:
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{bundle_dir}}"
    mkdir -p "{{bundle_dir}}"

    NOSTD_CRATES=$(just _discover-nostd-test-crates)

    echo "Bundling no_std test binaries from {{target_dir}}..."
    FOUND_COUNT=0

    for binary in $NOSTD_CRATES; do
        if [ -f "{{target_dir}}/$binary" ]; then
            cp "{{target_dir}}/$binary" "{{bundle_dir}}/"
            echo "  ✓ $binary"
            FOUND_COUNT=$((FOUND_COUNT + 1))
        else
            echo "  ✗ $binary (not found)"
        fi
    done

    if [ $FOUND_COUNT -eq 0 ]; then
        echo ""
        echo "Error: No test binaries found in {{target_dir}}" >&2
        echo "Make sure you've built the tests before bundling." >&2
        exit 1
    fi

[private]
_create-run-all-script bundle_dir:
    #!/usr/bin/env bash
    set -euo pipefail

    echo ""
    echo "Creating run-all.sh script..."

    {
        echo '#!/bin/sh'
        echo '# Script to run all no_std test binaries'
        echo '# Usage: ./run-all.sh'
        echo ''
        echo 'set -e'
        echo ''
        echo 'SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"'
        echo 'TOTAL=0'
        echo 'PASSED=0'
        echo 'FAILED=0'
        echo ''
        echo 'echo ""'
        echo 'echo "=========================================="'
        echo 'echo "Running iceoryx2 no_std tests"'
        echo 'echo "=========================================="'
        echo 'echo ""'
        echo ''
        echo 'for binary in "$SCRIPT_DIR"/*-tests-nostd; do'
        echo '    if [ -f "$binary" ] && [ -x "$binary" ]; then'
        echo '        binary_name=$(basename "$binary")'
        echo '        TOTAL=$((TOTAL + 1))'
        echo '        '
        echo '        echo "Running $binary_name..."'
        echo '        echo "------------------------------------------"'
        echo '        '
        echo '        if "$binary"; then'
        echo '            echo "✓ $binary_name PASSED"'
        echo '            PASSED=$((PASSED + 1))'
        echo '        else'
        echo '            echo "✗ $binary_name FAILED (exit code: $?)"'
        echo '            FAILED=$((FAILED + 1))'
        echo '        fi'
        echo '        '
        echo '        echo ""'
        echo '    fi'
        echo 'done'
        echo ''
        echo 'echo "=========================================="'
        echo 'echo "Test Summary"'
        echo 'echo "=========================================="'
        echo 'echo "Total:  $TOTAL"'
        echo 'echo "Passed: $PASSED"'
        echo 'echo "Failed: $FAILED"'
        echo 'echo ""'
        echo ''
        echo 'if [ $FAILED -gt 0 ]; then'
        echo '    exit 1'
        echo 'else'
        echo '    exit 0'
        echo 'fi'
    } > "{{bundle_dir}}/run-all.sh"

    chmod +x "{{bundle_dir}}/run-all.sh"

[private]
_strip-binaries bundle_dir:
    #!/usr/bin/env bash
    set -euo pipefail

    if command -v strip &> /dev/null; then
        echo ""
        echo "Stripping binaries..."
        strip "{{bundle_dir}}"/*-tests-nostd 2>/dev/null || true
    else
        echo ""
        echo "Warning: strip command not found, skipping stripping" >&2
    fi

[private]
_compress-bundle bundle_dir target_triplet:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -n "{{target_triplet}}" ]; then
        TARBALL="target/{{target_triplet}}/tests-nostd-{{target_triplet}}.tar.gz"
    else
        TARBALL="target/tests-nostd.tar.gz"
    fi

    echo ""
    echo "Creating compressed tarball..."
    tar -czf "$TARBALL" -C "{{bundle_dir}}" .
    echo "Tarball created at: $TARBALL"
    ls -lh "$TARBALL"
