# Copyright (c) 2024 Contributors to the Eclipse Foundation
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

cmake_minimum_required(VERSION 3.22)

project(iceoryx2-bb-cxx-tests VERSION ${IOX2_VERSION} LANGUAGES CXX)

find_package(iceoryx2-bb-cxx REQUIRED)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2TestOptionsAndParams)

include(Iceoryx2PlatformSettings)

include(Iceoryx2GTest)

add_executable(iceoryx2-bb-cxx-tests ${TEST_FILES})
target_sources(iceoryx2-bb-cxx-tests
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/testing/observable.hpp
    ${PROJECT_SOURCE_DIR}/src/testing/test_utils.hpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/testing/observable.cpp
    ${PROJECT_SOURCE_DIR}/src/testing/test_utils.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/main.cpp
    ${PROJECT_SOURCE_DIR}/src/duration_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/expected_err_free_function_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/expected_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/into_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/layout_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/optional_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/path_and_file_verifier_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/raw_byte_storage_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/semantic_string_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/slice_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/static_function_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/static_string_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/static_vector_tests.cpp
    ${PROJECT_SOURCE_DIR}/src/std_chrono_support_tests.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/error_reporting/testing_error_handler.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/error_reporting/testing_support.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/mocks/logger_mock.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/barrier.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/fatal_failure.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/fatal_failure.inl
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/lifetime_and_assignment_tracker.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/include/iox2/legacy/testing/testing_logger.hpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/error_reporting/testing_error_handler.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/error_reporting/testing_support.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/testing/testing_logger.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/legacy/mocktests/test_fatal_failure.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/module_a/errors.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/module_a/error_reporting.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/module_b/errors.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/module_b/error_reporting.hpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_logstream.hpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_auxiliary_types.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_custom_error_reporting.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_default_error_handler.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_error_kind.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_error_reporting_macros.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_error_reporting_violation.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/error_reporting/test_testing_error_handler.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_container_uninitialized_array.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_design_polymorphic_handler.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_design_static_lifetime_guard.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_platform_atomic.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_posix_posix_call.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_primitives_type_traits.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_console_logger.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_logging.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_logstream.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_logstream_arithmetic.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_reporting_logstream_hex_oct_bin.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_utility_convert.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_utility_deprecation_marker.cpp
    ${PROJECT_SOURCE_DIR}/src/legacy/moduletests/test_vocabulary_expected.cpp
)
target_include_directories(iceoryx2-bb-cxx-tests
    PRIVATE ${PROJECT_SOURCE_DIR}/src
    PRIVATE ${PROJECT_SOURCE_DIR}/src/legacy/testing/include
)
target_link_libraries(
    iceoryx2-bb-cxx-tests
    PRIVATE
    iceoryx2-bb-cxx::iceoryx2-bb-cxx
    GTest::gtest GTest::gmock
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)
target_compile_options(
    iceoryx2-bb-cxx-tests
    PRIVATE
    ${ICEORYX2_TEST_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)
target_compile_features(iceoryx2-bb-cxx-tests PRIVATE ${ICEORYX2_CXX_STD})

set_target_properties(iceoryx2-bb-cxx-tests PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO "${CMAKE_BINARY_DIR}/tests"
)

add_test(NAME "iceoryx2 C++ BB Container Tests" COMMAND iceoryx2-bb-cxx-tests)
