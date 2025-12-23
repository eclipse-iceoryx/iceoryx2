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

ICEORYX2_RELEASE_VERSION_SET=false
ICEORYX2_RELEASE_VERSION='X.Y.Z'

STEP_COUNTER=0

print_step() {
    echo -e ""
    echo -e "${C_BLUE}# ${STEP_COUNTER}: ${1}${C_OFF}"
    echo -e ""
    STEP_COUNTER=$((STEP_COUNTER+1))
}

print_default_user_exit_hint() {
    echo -e "Canceled script execution!"
}

print_article_hint() {
    echo -e "${C_BOLD}Article Types${C_OFF}"
    echo -e "1. Write release announcement blog article"
    echo -e "2. Write LinkedIn post"
    echo -e "3. Write reddit/hacker/programming-dev news post"
    echo -e "4. Update the 'ROADMAP.md' document"
    echo -e ""
    echo -e "${C_BOLD}Article Template${C_OFF}"
    echo -e "The link in new release announcement shall always be the link to the"
    echo -e "release blog-article."
    echo -e ""
    echo -e "${C_BOLD}Blog Article - Add The Following Links${C_OFF}"
    echo -e "[Add it at the bottom]"
    echo -e ""
    echo -e " * Discuss on Reddit"
    echo -e " * Discuss on Hacker News"
    echo -e " * Project on GitHub"
    echo -e " * Project on crates.io"
    echo -e ""
    echo -e "${C_BOLD}Social Media Post - Add The Following Links${C_OFF}"
    echo -e "[Add it at the top]"
    echo -e " * Release Announcement: https://ekxide.io/blog/****************"
    echo -e ""
    echo -e "[Add it at the bottom]"
    echo -e " * repo: https://github.com/eclipse-iceoryx/iceoryx2"
    echo -e " * roadmap: https://github.com/eclipse-iceoryx/iceoryx2/blob/main/ROADMAP.md"
    echo -e " * crates.io: https://crates.io/crates/iceoryx2"
    echo -e " * docs.rs: https://docs.rs/iceoryx2/latest/iceoryx2"
    echo -e ""
    echo -e "${C_BOLD}Announcement (Major release only)${C_OFF}"
    echo -e "1. Write blog-article with some technical details, highlights etc."
    echo -e "2. Announce blog-article on"
    echo -e "   * https://www.reddit.com/r/rust/"
    echo -e "   * https://www.reddit.com/r/programming/"
    echo -e "   * https://www.reddit.com/r/python/"
    echo -e "   * https://www.linkedin.com/"
    echo -e "   * https://news.ycombinator.com/"
    echo -e "   * https://programming.dev/"
    echo -e "   * https://techhub.social/"
    echo -e "   * https://X.com/"
    echo -e "3. If there are interesting things to explore, play around with, post it on"
    echo -e "   * https://news.ycombinator.com/show"
}

print_manual_steps_hint() {
    echo -e "* Test if QNX builds and runs with the current codebase"
    echo -e "* Test if Yocto builds and runs with the current codebase"
    echo -e "* check if the new features are marked as done, e.g. README, ROADMAP, etc."
    echo -e "* grep for 'planned'"
    echo -e "* verify to be on the right branch, e.g. 'main' or 'release-x.y'"
}

print_sanity_checks() {
    echo -e "The sanity-checks from the 'crates_io_publish_script.sh' are run"
}

print_create_branch() {
    echo -e "The branch name follows the format 'iox2-77-release-X.Y.Z'"
}

print_finalize_release_notes() {
    echo -e "* move ${C_BOLD}iceoryx2-unreleased.md${C_OFF} to ${C_BOLD}iceoryx2-v${ICEORYX2_RELEASE_VERSION}.md${C_OFF}"
    echo -e "* replace the '?.?.?' placeholder with the appropriate versions"
    echo -e "* remove template example entries and clean up"
    echo -e "* copy 'iceoryx2-release-template.md' to 'iceoryx2-unreleased.md'"
    echo -e "* add the ${C_BOLD}iceoryx2-${ICEORYX2_RELEASE_VERSION}.md${C_OFF} to ${C_BOLD}CHANGELOG.md${C_OFF}"
}

print_set_new_version_number() {
    echo -e "* change the version number to ${ICEORYX2_RELEASE_VERSION} in all relevant files"
    echo -e "* the 'update_versions.sh' script can be utilized to automate the process"
    echo -e "* the script changes the version in:"
    echo -e "  * all Cargo.toml"
    echo -e "  * all CMakeLists.txt"
    echo -e "  * all *.cmake"
    echo -e "  * all package.xml"
    echo -e "  * all pyproject.toml"
    echo -e "  * all BUILD.bazel"
    echo -e "  * all markdown files where appropriate"
    echo -e "  * internal/VERSIONS"
}

print_do_crates_io_publishing_dry_run() {
    echo -e "Do a 'cargo publish --dry-run'"
}

print_merge_all_changes_to_main_and_create_release_branch() {
    echo -e "Congratulations! You made it!"
    echo -e "Please commit all the changes and create a pull request to 'main'!"
    echo -e "Once the pull request is merged, a release branch should be created!"
    echo -e "For the tag, the '\$GIT_ROOT$/internal/scripts/release/release_tagging.sh' script can be used!"
    echo -e "${C_YELLOW}But before creating the tag, port the reference system${C_OFF}"
    echo -e "${C_YELLOW}to the new iceoryx2 version to catch last minute bugs${C_OFF}"
}

print_howto() {
    STEP_COUNTER=0
    print_step "Start Always With Writing The Articles"
    print_article_hint

    print_step "Check Manual Steps"
    print_manual_steps_hint

    print_step "Sanity checks for crates.io release"
    print_sanity_checks

    print_step "Use generic release issue ([#77]) and create a new branch"
    print_create_branch

    print_step "Finalize Release Notes"
    print_finalize_release_notes

    print_step "Set New Version Number"
    print_set_new_version_number

    print_step "Do dry-run publish to crates.io"
    print_do_crates_io_publishing_dry_run

    print_step "Merge all changes to 'main' and Create Release Branch"
    print_merge_all_changes_to_main_and_create_release_branch
}

while (( "$#" )); do
    case "$1" in
        "howto")
            print_howto
            exit 0
            ;;
        "--new-version")
            ICEORYX2_RELEASE_VERSION=$2
            ICEORYX2_RELEASE_VERSION_SET=true
            shift 2
            ;;
        "help")
            echo -e "Script to automate parts of the iceoryx2 release process"
            echo -e ""
            echo -e "Usage: ${C_GREEN}$(basename $0)${C_OFF} --new-version 0.8.15"
            echo -e "Options:"
            echo -e "    howto                          Prints the how to release iceoryx2 guide"
            echo -e "    --new-version <VERSION>        The release <VERSION> in the format X.Y.Z"
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

if [[ ${ICEORYX2_RELEASE_VERSION_SET} == false ]];then
    echo -e "${C_RED}ERROR:${C_OFF} No new-version set! Please provide a release version with '--new-version 0.8.15'" >&2
    exit 1
fi
if ! [[ ${ICEORYX2_RELEASE_VERSION} =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${C_RED}ERROR:${C_OFF} Invalid version format for new-version!"
    echo -e "Expected X.Y.Z (e.g., 1.2.3)! but got '${ICEORYX2_RELEASE_VERSION}'" >&2
    exit 1
fi

ICEORYX2_PREVIOUS_VERSION=$(grep 'PREVIOUS_RELEASE:' internal/VERSIONS | sed 's/PREVIOUS_RELEASE: //')

cd $(git rev-parse --show-toplevel)

echo -e "${C_BLUE}Hello walking water bag. I will assist you in the iceoryx2 release process!${C_OFF}"

STEP_COUNTER=0

print_step "Did you wrote the articles? (Release announcement, social media post, etc.)"
show_default_selector print_article_hint

print_step "Check Manual Steps"
print_manual_steps_hint
show_default_selector

print_step "Sanity checks"
echo -e "Shall I run the sanity checks for the crates.io release?"
show_default_selector
if [[ ${SELECTION} == "${YES}" ]]; then
    internal/scripts/release/crates_io_publish_script.sh sanity-checks

    show_completion
fi

print_step "Release Version and Branch"
echo -e "Shall I create ${C_YELLOW}iox2-77-release-v${ICEORYX2_RELEASE_VERSION}${C_OFF} branch?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    git checkout -b iox2-77-release-v${ICEORYX2_RELEASE_VERSION}

    show_completion
fi

print_step "Finalizing The Release Notes"
echo -e "Shall the release notes be finalized?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    sed -i 's/^# iceoryx2 v?.?.?/# iceoryx2 v'"${ICEORYX2_RELEASE_VERSION}"'/g' \
        doc/release-notes/iceoryx2-unreleased.md
    sed -i 's/^## \[v?.?.?\]/## \[v'"${ICEORYX2_RELEASE_VERSION}"'\]/g' \
        doc/release-notes/iceoryx2-unreleased.md
    sed -i 's/iceoryx2\/tree\/v?.?.?)/iceoryx2\/tree\/v'"${ICEORYX2_RELEASE_VERSION}"')/g' \
        doc/release-notes/iceoryx2-unreleased.md
    sed -i 's/iceoryx2\/compare\/v?.?.?...v?.?.?)/iceoryx2\/compare\/v'"${ICEORYX2_PREVIOUS_VERSION}"'...v'"${ICEORYX2_RELEASE_VERSION}"')/g' \
        doc/release-notes/iceoryx2-unreleased.md

    git mv ./doc/release-notes/iceoryx2-unreleased.md ./doc/release-notes/iceoryx2-v${ICEORYX2_RELEASE_VERSION}.md

    cp ./doc/release-notes/iceoryx2-release-template.md ./doc/release-notes/iceoryx2-unreleased.md

    # NOTE: the second line must be most left, else whitespaces will be added to the changelog
    sed -i '/\* \[unreleased\](doc\/release-notes\/iceoryx2-unreleased.md)/a\
'"* \[v${ICEORYX2_RELEASE_VERSION}\](doc\/release-notes\/iceoryx2-v${ICEORYX2_RELEASE_VERSION}.md)" \
        CHANGELOG.md

    git add doc/release-notes/iceoryx2-unreleased.md
    git add doc/release-notes/iceoryx2-v${ICEORYX2_RELEASE_VERSION}.md
    git add CHANGELOG.md

    echo -e "Did you check the release notes for dummy entries and cleaned it up?"
    show_default_selector
    echo -e "Shall the changes be commited?"
    show_default_selector
    if [[ ${SELECTION} == ${YES} ]]; then
        git add doc/release-notes/iceoryx2-v${ICEORYX2_RELEASE_VERSION}.md
        git commit -m"[#77] Finalize release notes for v${ICEORYX2_RELEASE_VERSION}"
    fi

    show_completion
fi

print_step "Set New Version Number"
echo -e "Shall the ${C_YELLOW}${ICEORYX2_RELEASE_VERSION}${C_OFF} release version be set in all files?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    internal/scripts/update_versions.sh --iceoryx2 ${ICEORYX2_RELEASE_VERSION}

    sed -i 's/PREVIOUS_RELEASE: '"${ICEORYX2_PREVIOUS_VERSION}"'/PREVIOUS_RELEASE: '"${ICEORYX2_RELEASE_VERSION}"'/g' \
        internal/VERSIONS
    if grep -q "PREVIOUS_RELEASE: ${ICEORYX2_PREVIOUS_VERSION}" internal/VERSIONS; then
        echo -e "${C_RED}ERROR:${C_OFF} Could not update 'PREVIOUS_RELEASE' version in 'internal/VERSIONS'"

        exit 1
    fi

    git add .

    echo -e "Did you build with cargo, bazel and also the python bindings to update the corresponding lock files?"
    echo -e ""
    echo -e "cargo:  cargo build --all-targets"
    echo -e "bazel:  USE_BAZEL_VERSION=7.4.1 bazelisk build //..."
    echo -e "python: maturin build --manifest-path=iceoryx2-ffi/python/Cargo.toml"
    show_default_selector

    echo -e "Shall the changes be commited?"
    show_default_selector
    if [[ ${SELECTION} == ${YES} ]]; then
        git commit -m"[#77] Update version number to v${ICEORYX2_RELEASE_VERSION}"
    fi

    show_completion
fi

print_step "Do crates.io publishing dry-run"
echo -e "Shall a publishing dry-run be performed?"
show_default_selector
if [[ ${SELECTION} == ${YES} ]]; then
    internal/scripts/release/crates_io_publish_script.sh dry-run

    show_completion
fi

print_step "Continue With Release Tagging And Publishing"
print_merge_all_changes_to_main_and_create_release_branch

echo -e "${C_GREEN}FINISHED${C_OFF}"
