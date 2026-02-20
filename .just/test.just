# =============================================================================
# Test Recipes
# =============================================================================

[private]
_test-dispatch target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-build-flags {{flags}})

    case "{{target}}" in
        workspace)
            just _test-workspace "$TOOLCHAIN" "$HAS_NOSTD" "$TARGET_TRIPLET"
            ;;
        *)
            just _test-package "{{target}}" "$TOOLCHAIN" "$HAS_NOSTD" "$TARGET_TRIPLET"
            ;;
    esac

[private]
_test-workspace toolchain has_nostd target_triplet:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        NOSTD_CRATES=$(just _discover-nostd-test-crates)

        CRATE_COUNT=$(echo "$NOSTD_CRATES" | wc -l)
        echo ""
        echo "=========================================="
        echo "Running $CRATE_COUNT no_std test crates"
        echo "=========================================="
        echo ""

        PASSED=0
        FAILED=0
        PASSED_CRATES=()
        FAILED_CRATES=()

        for crate in $NOSTD_CRATES; do
            BASE_CRATE="${crate%-tests-nostd}"

            echo "[$((PASSED + FAILED + 1))/$CRATE_COUNT] Running tests for $BASE_CRATE..."
            echo "------------------------------------------"

            if just _test-package "$BASE_CRATE" "{{toolchain}}" "true" "{{target_triplet}}"; then
                echo "✓ $BASE_CRATE PASSED"
                PASSED=$((PASSED + 1))
                PASSED_CRATES+=("$BASE_CRATE")
            else
                echo "✗ $BASE_CRATE FAILED"
                FAILED=$((FAILED + 1))
                FAILED_CRATES+=("$BASE_CRATE")
            fi
            echo ""
        done

        echo "=========================================="
        echo "Test Summary"
        echo "=========================================="
        echo "Total:  $CRATE_COUNT"
        echo "Passed: $PASSED"
        echo "Failed: $FAILED"
        echo ""

        if [ $PASSED -gt 0 ]; then
            echo "Passed crates:"
            for passed_crate in "${PASSED_CRATES[@]}"; do
                echo "  ✓ $passed_crate"
            done
            echo ""
        fi

        if [ $FAILED -gt 0 ]; then
            echo "Failed crates:"
            for failed_crate in "${FAILED_CRATES[@]}"; do
                echo "  ✗ $failed_crate"
            done
            echo ""
            exit 1
        fi
    else
        cargo {{toolchain}} test --workspace
    fi

[private]
_test-package package toolchain has_nostd target_triplet:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        else
            HOST_TARGET=$(just _detect-host-target "{{toolchain}}")
            TARGET_ARG="--target $HOST_TARGET"
        fi

        if [[ "{{package}}" == *-tests-nostd ]]; then
            TEST_PACKAGE="{{package}}"
        else
            TEST_PACKAGE="{{package}}-tests-nostd"
        fi

        RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C panic=abort" \
            cargo {{toolchain}} run \
                --package "$TEST_PACKAGE" \
                --no-default-features \
                -Zbuild-std=core,alloc \
                $TARGET_ARG
    else
        cargo {{toolchain}} test --package {{package}}
    fi
