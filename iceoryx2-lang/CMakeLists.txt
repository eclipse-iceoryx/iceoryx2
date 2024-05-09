cmake_minimum_required(VERSION 3.28)
project(iceoryx2-lang)

add_subdirectory(c)

if(BUILD_TESTING)
    add_subdirectory(tests/tbd)
endif()
