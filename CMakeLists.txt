cmake_minimum_required(VERSION 3.28)
project(iceoryx2)

#TODO how to handle feature flags

add_subdirectory(iceoryx2-lang)

#TODO add flags
# if(EXAMPLES)
    add_subdirectory(examples/c)
# endif()
