cmake_minimum_required(VERSION 3.16)

project(cp_library LANGUAGES CXX)

add_library(cp_library INTERFACE)
add_library(cp::library ALIAS cp_library)

target_include_directories(
    cp_library
    INTERFACE
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)
target_compile_features(cp_library INTERFACE cxx_std_17)

option(CP_LIBRARY_BUILD_TESTS "Build cp_library tests" ON)
if(CP_LIBRARY_BUILD_TESTS)
    enable_testing()
    add_executable(cp_io_test tests/io_test.cpp)
    target_link_libraries(cp_io_test PRIVATE cp::library)
    add_test(NAME cp_io_test COMMAND cp_io_test)
endif()

option(CP_LIBRARY_BUILD_EXAMPLES "Build cp_library examples" OFF)
if(CP_LIBRARY_BUILD_EXAMPLES)
    add_executable(output_example examples/output.cpp)
    target_link_libraries(output_example PRIVATE cp::library)
endif()

