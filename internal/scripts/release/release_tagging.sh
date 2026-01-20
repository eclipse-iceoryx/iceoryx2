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
    echo -e "* Test on QNX and Yocto"
    echo -e "* Port reference system to new iceoryx2 version"
    echo -e "* Check if new features are marked as done"
}

print_sanity_checks() {
    echo -e "The sanity-checks from the 'crates_io_publish_script.sh' are run"
}

print_release_branch() {
    echo -e "* Create a release branch for the new release"
    echo -e "* format: 'release_X.Y' -> release_${NEW_MAJOR}.${NEW_MINOR}"
}

print_release_tag() {
    echo -e "* Create a tag from the release_${NEW_MAJOR}.${NEW_MINOR} branch"
    echo -e "* format: 'vX.Y.Z' -> v${NEW_VERSION}"
}

print_publish_release() {
    echo -e "* Push release branch"
    echo -e "  ${C_YELLOW}git push -u origin release_${NEW_MAJOR}.${NEW_MINOR}${C_OFF}"
    echo -e "* Push tag"
    echo -e "  ${C_YELLOW}git push origin tag v${NEW_VERSION}${C_OFF}"

    echo -e "* Create release on github"
    echo -e "  * Go to https://github.com/eclipse-iceoryx/iceoryx2/releases/tag/v${NEW_VERSION}${C_OFF}"
    echo -e "  * Click on 'Create release from tag' button"
    echo -e "  * Select correct 'Previous tag'"
    echo -e "  * Add the content from 'doc/release-notes/iceoryx2-v.${NEW_VERSION}md', beginning at '[Full Changelog]', to 'Release notes'"

    echo -e "* Set iceoryx2 dev version on 'main'"
    echo -e "  * In case of a .0 release, update the version on main to x.y.999 with y being the next feature release minus 1"
    echo -e "    ${C_YELLOW}internal/scripts/update_versions.sh --iceoryx2 ${NEW_MAJOR}.${NEW_MINOR}.999${C_OFF}"

    echo -e "* Backport changelog to 'main'"
    echo -e "  * In case of a release from a branch, backport the changelog to main and remove the entries from the patch release from 'iceoryx2-unreleased.md'"
    echo -e "  * Set the PREVIOUS_VERSION in 'internal/VERSIONS' on main to the latest patch release"

    echo -e "* For the publishing, the '\$GIT_ROOT$/internal/scripts/release/release_publish.sh' script can be used!"
}

print_howto() {
    STEP_COUNTER=0

    print_step "Release Preparation"
    print_preparations_hint

    print_step "Sanity checks for crates.io release"
    print_sanity_checks

    print_step "Release Branch"
    print_release_branch

    print_step "Release Tag"
    print_release_tag

    print_step "Publish Release"
    print_publish_release
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
show_default_selector() {
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

print_step "Did you do the release preparation"
print_preparations_hint
show_default_selector

print_step "Sanity checks"
echo -e "Shall I run the sanity checks for the crates.io release?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    internal/scripts/release/crates_io_publish_script.sh sanity-checks

    show_completion
fi

print_step "Release branch"
echo -e "Shall I create ${C_YELLOW}release_${NEW_MAJOR}.${NEW_MINOR}${C_OFF} branch?"
echo -e "${C_YELLOW}Please verify to be on the right commit on the right branch!${C_OFF}"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    git checkout -b release_${NEW_MAJOR}.${NEW_MINOR}

    show_completion
fi

print_step "Create tag"
echo -e "Shall I create git tag ${C_YELLOW}v${NEW_VERSION}${C_OFF}?"
echo -e "${C_YELLOW}Please verify to be on the right commit on the right branch!${C_OFF}"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    git checkout release_${NEW_MAJOR}.${NEW_MINOR}
    git tag v${NEW_VERSION}

    show_completion
fi

print_step "Continue with publishing the release"
print_publish_release

echo -e "${C_GREEN}FINISHED${C_OFF}"
