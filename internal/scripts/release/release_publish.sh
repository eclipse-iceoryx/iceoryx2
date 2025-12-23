#!/usr/bin/env bash
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

set -e

C_OFF='\033[0m'
C_BOLD='\033[1m'
C_RED='\033[1;31m'
C_GREEN='\033[1;32m'
C_YELLOW='\033[1;33m'
C_BLUE='\033[1;34m'

YES=1
SKIP=2

STEP_COUNTER=0

# NOTE: 'PREVIOUS_RELEASE' in 'internal/VERSIONS' was already updated and contains the version to be released
NEW_VERSION=$(grep 'PREVIOUS_RELEASE:' internal/VERSIONS | sed 's/PREVIOUS_RELEASE: //')
IFS='.' read -r NEW_MAJOR NEW_MINOR NEW_PATCH <<< ${NEW_VERSION}

print_step() {
    echo -e ""
    echo -e "${C_BLUE}# ${STEP_COUNTER}: ${1}${C_OFF}"
    echo -e ""
    STEP_COUNTER=$((STEP_COUNTER+1))
}

print_default_user_exit_hint() {
    echo -e "Canceled script execution!"
}

print_preparations_hint() {
    echo -e "* Run internal/scripts/release/release_preparation.sh"
    echo -e "* Run internal/scripts/release/release_tagging.sh"
}

print_sanity_checks() {
    echo -e "* Check for new crates to be published"
    echo -e "* Check for cyclic dependencies"
    echo -e "* Run 'internal/scripts/release/crates_io_publish_script.sh sanity-checks'"
}

print_publish_crates_io() {
    echo -e "* All crates, including dev-dependencies, must be published"
    echo -e "* The crates must be published in the correct order"
    echo -e "  * When calling 'cargo publish -p crate-name', it's dependencies must already be published"
    echo -e "* If the crates.io publish script fails before finishing, a new release must be done"
    echo -e "* Run 'internal/scripts/release/crates_io_publish_script.sh publish'"
}

print_howto() {
    STEP_COUNTER=0

    print_step "Release Preparation And Tagging"
    print_preparations_hint

    print_step "Sanity Checks"
    print_sanity_checks

    print_step "Publish To crates.io"
    print_publish_crates_io
}

while (( "$#" )); do
    case "$1" in
        "howto")
            print_howto
            exit 0
            ;;
        "help")
            echo -e "Script to automate parts of the iceoryx2 release process"
            echo -e ""
            echo -e "Usage: ${C_GREEN}$(basename $0)${C_OFF}"
            echo -e ""
            exit 0
            ;;
        *)
            echo -e "${C_RED}ERROR:${C_OFF} Invalid argument '$1'. Try 'help' for options."
            exit 1
            ;;
    esac
done

SELECTION=-1
function show_default_selector() {
    EXIT_HINT=${1:-print_default_user_exit_hint}
    while true; do
        read -p "Yes, Cancel or Skip (Y/C/S) [default=Y]: " yns
        yns=${yns:-Y}
        case $yns in
            [Yy]*)
                SELECTION=${YES}
                break;
                ;;
            [Cc]*)
                $EXIT_HINT
                exit 1
                ;;
            [Ss]*)
                SELECTION=${SKIP}
                break;
                ;;
            *) echo -e "${C_YELLOW}Please use either 'Y', 'C' or 'S'.${C_OFF}";;
        esac
    done
}

show_completion() {
    # NOTE: read does not support to use variables for color codes
    read -p $'\033[32mDONE!\033[0m Continue to next step with \'enter\'' # blocks until enter is pressed
}

cd $(git rev-parse --show-toplevel)

echo -e "${C_BLUE}Hello walking water bag. I will assist you in the iceoryx2 release process!${C_OFF}"

STEP_COUNTER=0

print_step "Did you do the release preparation and tagging"
print_preparations_hint
show_default_selector

print_step "Sanity checks"
echo -e "Shall I run the sanity checks?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    internal/scripts/release/crates_io_publish_script.sh sanity-checks

    show_completion
fi

print_step "Publish to crates.io"
internal/scripts/release/crates_io_publish_script.sh list-crates-to-publish
echo -e "Shall I publish the listed crates to crates.io?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    internal/scripts/release/crates_io_publish_script.sh publish

    echo -e "Please check whether the release looks fine on 'docs.rs'."
    echo -e "(click through the documentation to check if everything was generated correctly)"

    show_completion
fi

echo -e "${C_GREEN}FINISHED${C_OFF}"
