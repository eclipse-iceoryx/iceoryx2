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

# Sanitizer blacklist creation functions
function(iox2_create_asan_compile_time_blacklist BLACKLIST_FILE_PATH)
    # Suppressing Errors in Recompiled Code (Blacklist)
    # (https://clang.llvm.org/docs/AddressSanitizer.html#suppressing-errors-in-recompiled-code-blacklist)
    # More details about the syntax can be found here (https://clang.llvm.org/docs/SanitizerSpecialCaseList.html)
    if(NOT EXISTS ${BLACKLIST_FILE_PATH})
        file(WRITE  ${BLACKLIST_FILE_PATH} "# This file is auto-generated from iceoryx2-cmake-modules/modules/Iceoryx2Sanitizer.cmake\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# src:*file_name.cpp*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# fun:*Test_Name*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# End of file\n")
    endif()
endfunction()

function(iox2_create_asan_runtime_blacklist BLACKLIST_FILE_PATH)
    # Suppress errors in external libraries (https://clang.llvm.org/docs/AddressSanitizer.html#suppressing-reports-in-external-libraries)
    # List of errors generated in .inl files. These cannot be suppressed with -fsanitize-blacklist!
    # We enable sanitizer flags for core components, not in tests (mainly to avoid catching errors in test cases, at least for now)
    # NOTE : AddressSanitizer won't generate any report for the suppressed errors.
    #        Only way to see detailed errors is to disable the entries here  & run
    if(NOT EXISTS ${BLACKLIST_FILE_PATH})
        file(WRITE  ${BLACKLIST_FILE_PATH} "# This file is auto-generated from iceoryx2-cmake-modules/modules/Iceoryx2Sanitizer.cmake\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "#interceptor_via_fun:-[ClassName objCMethodToSuppress:]\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "#interceptor_via_lib:NameOfTheLibraryToSuppress\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# End of file\n")
    endif()
endfunction()

function(iox2_create_tsan_runtime_blacklist BLACKLIST_FILE_PATH)
    # (https://github.com/google/sanitizers/wiki/ThreadSanitizerSuppressions)
    # The suppression types are:
    # race              suppresses data races and use-after-free reports
    # race_top          same as race, but matched only against the top stack frame
    # thread            suppresses reports related to threads (leaks)
    # mutex             suppresses reports related to mutexes (destruction of a locked mutex)
    # signal            suppresses reports related to signal handlers (handler calls malloc())
    # deadlock          suppresses lock inversion reports
    # called_from_lib   suppresses all interceptors in a particular library
    if(NOT EXISTS ${BLACKLIST_FILE_PATH})
        file(WRITE  ${BLACKLIST_FILE_PATH} "# This file is auto-generated from iceoryx2-cmake-modules/modules/Iceoryx2Sanitizer.cmake\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "mutex:*MutexWithDeadlockDetectionsFailsWhenSameThreadTriesToUnlockItTwice*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "mutex:*MutexWithDeadlockDetectionsFailsWhenAnotherThreadTriesToUnlock*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "mutex:*MutexWithStallWhenLockedBehaviorDoesntUnlockMutexWhenThreadTerminates*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "race:*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "deadlock:*TimingTest_AttachingInCallbackWorks*\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# End of file\n")
    endif()
endfunction()

function(iox2_create_lsan_runtime_blacklist BLACKLIST_FILE_PATH)
    # Suppress known memory leaks (https://github.com/google/sanitizers/wiki/AddressSanitizerLeakSanitizer)
    # Below function/files contains memory leaks!
    # LeakSanitizer wont report the problem for the entries here , however you can find the suppression report in the log
    #
    # e.g.
    # Suppressions used:
    # count      bytes template
    #     8        642 libacl.so.1
    #     1         24 iox::UnixDomainSocket::timedReceive
    #     1         24 iox::MessageQueue::receive
    if(NOT EXISTS ${BLACKLIST_FILE_PATH})
        file(WRITE  ${BLACKLIST_FILE_PATH} "# This file is auto-generated from iceoryx2-cmake-modules/modules/Iceoryx2Sanitizer.cmake\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "#leak:libacl.so.1\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "#leak:iox::UnixDomainSocket::timedReceive\n")
        file(APPEND ${BLACKLIST_FILE_PATH} "# End of file\n")
    endif()
endfunction()

# Parse SANITIZERS string and set appropriate flags
set(ICEORYX2_SANITIZER_FLAGS "" CACHE INTERNAL "")
set(ICEORYX2_SANITIZER_BLACKLIST "" CACHE INTERNAL "")

# Strip any whitespace from SANITIZERS
if(DEFINED SANITIZERS)
    string(STRIP "${SANITIZERS}" SANITIZERS_STRIPPED)
else()
    set(SANITIZERS_STRIPPED "")
endif()

# Check for invalid combinations
if(SANITIZERS_STRIPPED MATCHES "address" AND SANITIZERS_STRIPPED MATCHES "thread")
    message(FATAL_ERROR "You cannot run address sanitizer and thread sanitizer together. Choose one or the other!")
endif()

# Create blacklist directories and files if sanitizers are enabled
if(DEFINED SANITIZERS AND NOT SANITIZERS_STRIPPED STREQUAL "" AND NOT SANITIZERS_STRIPPED STREQUAL "OFF" AND NOT SANITIZERS_STRIPPED STREQUAL "FALSE")
    # Create sanitizer blacklist directory
    set(ICEORYX2_SANITIZER_BLACKLIST_DIR ${CMAKE_BINARY_DIR}/sanitizer_blacklist)
    file(MAKE_DIRECTORY ${ICEORYX2_SANITIZER_BLACKLIST_DIR})
    
    # Create runtime blacklists (always created when any sanitizer is enabled)
    iox2_create_asan_runtime_blacklist(${ICEORYX2_SANITIZER_BLACKLIST_DIR}/asan_runtime.txt)
    iox2_create_lsan_runtime_blacklist(${ICEORYX2_SANITIZER_BLACKLIST_DIR}/lsan_runtime.txt)
    iox2_create_tsan_runtime_blacklist(${ICEORYX2_SANITIZER_BLACKLIST_DIR}/tsan_runtime.txt)
    
    # Create compile-time blacklist for Clang
    if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang" OR CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang")
        set(ICEORYX2_SANITIZER_BLACKLIST_FILE ${ICEORYX2_SANITIZER_BLACKLIST_DIR}/sanitizer_compile_time.txt)
        iox2_create_asan_compile_time_blacklist(${ICEORYX2_SANITIZER_BLACKLIST_FILE})
        set(ICEORYX2_SANITIZER_BLACKLIST -fsanitize-blacklist=${ICEORYX2_SANITIZER_BLACKLIST_FILE} CACHE INTERNAL "")
    endif()
endif()

# Common sanitizer flags
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR CMAKE_CXX_COMPILER_ID STREQUAL "Clang" OR CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang")
    set(ICEORYX2_SANITIZER_COMMON_FLAGS -fno-omit-frame-pointer -fno-optimize-sibling-calls)
else()
    set(ICEORYX2_SANITIZER_COMMON_FLAGS "")
endif()

# Set sanitizer flags based on SANITIZERS value
if(NOT DEFINED SANITIZERS OR SANITIZERS_STRIPPED STREQUAL "" OR SANITIZERS_STRIPPED STREQUAL "OFF" OR SANITIZERS_STRIPPED STREQUAL "FALSE")
    # No sanitizers enabled - empty string is the default
elseif(SANITIZERS_STRIPPED STREQUAL "address")
    set(ICEORYX2_SANITIZER_FLAGS ${ICEORYX2_SANITIZER_COMMON_FLAGS} -fsanitize=address -fsanitize-address-use-after-scope ${ICEORYX2_SANITIZER_BLACKLIST} CACHE INTERNAL "")
elseif(SANITIZERS_STRIPPED STREQUAL "ub")
    set(ICEORYX2_SANITIZER_FLAGS ${ICEORYX2_SANITIZER_COMMON_FLAGS} -fsanitize=undefined -fno-sanitize-recover=undefined CACHE INTERNAL "")
elseif(SANITIZERS_STRIPPED STREQUAL "address;ub")
    set(ICEORYX2_SANITIZER_FLAGS ${ICEORYX2_SANITIZER_COMMON_FLAGS} -fsanitize=address -fsanitize-address-use-after-scope -fsanitize=undefined -fno-sanitize-recover=undefined ${ICEORYX2_SANITIZER_BLACKLIST} CACHE INTERNAL "")
elseif(SANITIZERS_STRIPPED STREQUAL "thread")
    set(ICEORYX2_SANITIZER_FLAGS ${ICEORYX2_SANITIZER_COMMON_FLAGS} -fsanitize=thread CACHE INTERNAL "")
else()
    message(FATAL_ERROR "Invalid SANITIZERS value: '${SANITIZERS_STRIPPED}' (original: '${SANITIZERS}'). Valid options are: 'address', 'ub', 'address;ub', 'thread', or empty string (disabled)")
endif()

# Export the blacklist directory path for use in CI
if(DEFINED ICEORYX2_SANITIZER_BLACKLIST_DIR)
    set(ICEORYX2_SANITIZER_BLACKLIST_DIR ${ICEORYX2_SANITIZER_BLACKLIST_DIR} CACHE PATH "Path to sanitizer blacklist directory" FORCE)
endif()
