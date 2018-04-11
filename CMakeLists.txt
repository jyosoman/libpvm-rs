cmake_minimum_required(VERSION 2.8.9)

include(ExternalProject)

ExternalProject_Add(
    libpvm
    SOURCE_DIR .
    BUILD_COMMAND cargo build --release
    INSTALL_COMMAND ""
    LOG_BUILD ON)

include_directories(src/include)

add_executable(copususer src/copususer.c)
add_executable(pvm2csv src/pvm2csv.c)

add_dependencies(copususer libpvm)
add_dependencies(pvm2csv libpvm)

target_link_libraries(copususer
    debug "${CMAKE_SOURCE_DIR}/target/debug/libopus.so"
    optimized "${CMAKE_SOURCE_DIR}/target/release/libopus.so")
target_link_libraries(pvm2csv
    debug "${CMAKE_SOURCE_DIR}/target/debug/libopus.so"
    optimized "${CMAKE_SOURCE_DIR}/target/release/libopus.so")