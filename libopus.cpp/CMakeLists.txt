# ========================================
# Global Resourceful build file
# Author: Lucian Carata <lc525@cam.ac.uk>
# ========================================
#
# The instructions below describe how to run cmake builds when you want to
# have full flexibility in controlling the build process. For normal usecases,
# the the stub Makefile provided at the root of the project will do the right
# thing when called (run `make help` for details), including fetching and
# building CMake itself if you don't have it in your PATH already.
#
# CMake build instructions:
#
# Run cmake from a separate build directory (out of source build is required).
# The provided .build script within the build directory wraps some of the common
# options:
#
# [..build]$ ./.build
#
# If you want to run cmake manually, that is of course also possible
# (but check for what variables need setting before the command - the .build
# script provides the required hints);
#
# Sample build command line:
# [..]$ cd build
# [..build]$ cmake -DCMAKE_BUILD_TYPE=Release ..
# [..build]$ make
#
#
# To look at available optional compilation features, run (from the separate
# build dir):
#
# [..build]$ cmake -LH ..
#
# Highlights from option list, add -D[OPTION_NAME]=ON to cmake cmd to enable:
#   - WITH_DOCS          - build project documentation
#        default:   ON
#        requires:  python3
#                   sphinx (sphinx-doc.org)
#
#   - WITH_API_DOCS      - build project API documentation
#        default:   OFF when WITH_DOCS is ON
#        requires:  doxygen (build time dependency)
#
#   - WITH_TESTS         - build the unit tests for each project. run them
#                          with "make check" after running make.
#        default:   ON
#        requires:  gtest (bundled with this project)
#        provides:  make target named "check"
#
# sample command line:
# [..build]$ cmake -DWITH_DOCS=ON ..
#
cmake_minimum_required(VERSION 2.8)

# change the project name here by setting PNAME. default is "generic".
#
set(PNAME libopus)

# set default project build options:
set(DEFAULT_WITH_TESTS ON)
set(DEFAULT_WITH_DOCS ON)
# Documentation: look for sphynx installations as part of the following python
# interpreters, giving priority to the versions appearing earlier in the list;
set(SPHINX_PYTHON_VERSIONS 3 2)

# General cmake project definition follows. Customize as necessary
project(${PNAME})

# variable definitions for generating configuration headers
set(PROJECT_MAJOR_VERSION 0)
set(PROJECT_MINOR_VERSION 1)
set(PROJECT_PATCH_VERSION 0)
execute_process(
  COMMAND git --git-dir ${CMAKE_SOURCE_DIR}/../.git rev-parse --short HEAD
    OUTPUT_VARIABLE GIT_REV
)
string(REPLACE "\n" "" GIT_REV ${GIT_REV})
if(NOT DEFINED PROJECT_TAG_VERSION)
  set(PROJECT_TAG_VERSION ${GIT_REV})
endif()

if(PROJECT_TAG_VERSION STREQUAL "rel")
  set(PVER ${PROJECT_MAJOR_VERSION}.${PROJECT_MINOR_VERSION}.${PROJECT_PATCH_VERSION})
else()
  set(PVER ${PROJECT_MAJOR_VERSION}.${PROJECT_MINOR_VERSION}.${PROJECT_PATCH_VERSION}-${PROJECT_TAG_VERSION})
endif()

message("
                _|  _|  _|
                _|      _|_|_|      _|_|    _|_|_|    _|    _|    _|_|_|
                _|  _|  _|    _|  _|    _|  _|    _|  _|    _|  _|_|
                _|  _|  _|    _|  _|    _|  _|    _|  _|    _|      _|_|
                _|  _|  _|_|_|      _|_|    _|_|_|      _|_|_|  _|_|_|
                                            _|
                                            _|

                  # Maintainers:         Lucian Carata (lc525),
                                         Thomas Bytheway (tb403)

                  # Build Type:          ${CMAKE_BUILD_TYPE}
                  # Version:             ${PVER}

---------------------------------------------------------------------------
Starting libopus build...
")
if(NOT DEFINED CMAKE_HEADER_SLEEP)
  set(CMAKE_HEADER_SLEEP 0)
endif()
execute_process(COMMAND ${CMAKE_COMMAND} -E sleep ${CMAKE_HEADER_SLEEP} )


###
#
#  CMAKE MODULES
#
###
set(CMAKE_MODULE_PATH ${${PNAME}_SOURCE_DIR}/cmake/modules ${CMAKE_MODULE_PATH})
set(CMAKE_MODULE_PATH ${${PNAME}_SOURCE_DIR}/cmake ${CMAKE_MODULE_PATH})
include(CMakeDependentOption)
include(FeatureSummary)
# custom modules
include(outofsource)
include(projectmacros)



###
#
# OPTIONS
#
###
option(WITH_DOCS
  "Build ${PNAME} documentation" ${DEFAULT_WITH_DOCS})
cmake_dependent_option(WITH_API_DOCS
  "Generate doxygen API documentation" OFF "WITH_DOCS" OFF)
option(WITH_TESTS
  "Build ${PNAME} tests" ${DEFAULT_WITH_TESTS})

#Google Testing Framework
# the options are mutually exclusive and are listed in order of preference,
# but if Gtest is not found using the given option the other ones will be
# attempted in turn.
option(USE_PROJECT_GTEST # recommended, automatic fallthrough
  "Use the local Gtest, if found within project subdirectories" ON)
option(USE_REPO_GTEST
  "Download and build gtest from online repository during make" OFF)
#option(USE_SYSTEM_GTEST
#  "Attempt using the system-wide Gtest installation" OFF)

if(NOT CMAKE_CONFIGURATION_TYPES AND NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Debug)
endif(NOT CMAKE_CONFIGURATION_TYPES AND NOT CMAKE_BUILD_TYPE)
message("!! Build type: ${CMAKE_BUILD_TYPE}")



###
#
# DEPENDENCIES
#
###
include(ExternalProject)

# Boost
#set(BOOST_USE_STATIC_LIBS OFF)
#set(BOOST_USE_MULTITHREADED ON)
#set(BOOST_USE_STATIC_RUNTIME OFF)
#find_package(Boost 1.46.0 REQUIRED COMPONENTS log system)

#if(Boost_FOUND)
#  include_directories( ${BOOST_INCLUDE_DIRS} )
#endif()

# Google Testing Framework
set(FALLTHROUGH "0")
if(USE_PROJECT_GTEST)
  find_package(gtest)
  if(NOT GTEST_FOUND)
    set(FALLTHROUGH "1")
  endif()
endif()
if(USE_REPO_GTEST OR FALLTHROUGH STREQUAL "1")
  set(FALLTHROUGH "0")
  set(GTEST_PREFIX ${CMAKE_SOURCE_DIR}/external/gtest)
  MESSAGE(" \\-- Gtest will be downloaded and built in ${GTEST_PREFIX}")
  ExternalProject_Add(
      gtest
      PREFIX ${CMAKE_SOURCE_DIR}/build/gtest
      GIT_REPOSITORY https://github.com/lc525/gtest.git
      GIT_TAG release-1.7.0
      CMAKE_ARGS
          -DCMAKE_CXX_FLAGS=${CMAKE_CXX_FLAGS}
          -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
      INSTALL_COMMAND ""
      UPDATE_COMMAND ""
      TMP_DIR ${CMAKE_SOURCE_DIR}/build/gtest
      STAMP_DIR ${CMAKE_SOURCE_DIR}/build/gtest
      DOWNLOAD_DIR ${GTEST_PREFIX}
      SOURCE_DIR ${GTEST_PREFIX}
      BINARY_DIR ${GTEST_PREFIX}/build-aux
      #INSTALL_DIR ${CMAKE_BINARY_DIR}/install
  )
  set(GTEST_LIBRARY "${GTEST_PREFIX}/build-aux/libgtest.a")
  set(GTEST_MAIN_LIBRARY "${GTEST_PREFIX}/build-aux/libgtest_main.a")
  set(GTEST_INCLUDE_DIR "${GTEST_PREFIX}/include" CACHE STRING "gtest include dir")

  set(GTEST_LIBRARIES ${GTEST_LIBRARY})
  set(GTEST_MAIN_LIBRARIES ${GTEST_MAIN_LIBRARY})
  set(GTEST_INCLUDE_DIRS ${GTEST_INCLUDE_DIR} CACHE STRING "gtest include dirs")
endif()
#  if(USE_SYSTEM_GTEST)
#endif()

# libneo4j-client driver
find_package(libneo4jclient)
if(NOT LIBNEO4JCLIENT_FOUND)
  set(LIBNEO4JCLIENT_PREFIX ${CMAKE_SOURCE_DIR}/external/libneo4j-client)
  MESSAGE(" \\-- libneo4j-client will be downloaded and built in ${LIBNEO4JCLIENT_PREFIX}")
  ExternalProject_Add(
      libneo4jclient_external
      PREFIX ${CMAKE_SOURCE_DIR}/build/libneo4jclient
      GIT_REPOSITORY https://github.com/cleishm/libneo4j-client.git
      GIT_TAG v2.1.3
      INSTALL_COMMAND ""
      UPDATE_COMMAND ""
      PATCH_COMMAND ./autogen.sh && git apply ${LIBNEO4JCLIENT_PREFIX}/../patches/libneo4jclient.patch
      CONFIGURE_COMMAND ${LIBNEO4JCLIENT_PREFIX}/configure --disable-tools
      BUILD_COMMAND make clean check
      TMP_DIR ${CMAKE_SOURCE_DIR}/build/libneo4jclient
      STAMP_DIR ${CMAKE_SOURCE_DIR}/build/libneo4jclient
      DOWNLOAD_DIR ${LIBNEO4JCLIENT_PREFIX}
      SOURCE_DIR ${LIBNEO4JCLIENT_PREFIX}
      BINARY_DIR ${LIBNEO4JCLIENT_PREFIX}
  )
set(LIBNEO4J_LIBRARY "${LIBNEO4JCLIENT_PREFIX}/src/lib/.libs/libneo4j-client.so")
set(LIBNEO4J_INCLUDE_DIR "${LIBNEO4JCLIENT_PREFIX}/src/lib")

set(LIBNEO4J_LIBRARIES ${LIBNEO4J_LIBRARY})
set(LIBNEO4J_INCLUDE_DIRS ${LIBNEO4J_INCLUDE_DIR})
endif()
include_directories(${LIBNEO4J_INCLUDE_DIR})

# Rapidjson
include_directories(${CMAKE_SOURCE_DIR}/external/rapidjson/include)

if(WITH_TESTS)
  add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND} -V)
  enable_testing()
endif(WITH_TESTS)

# Capnproto
#find_package(capnproto)
#set(CAPNPC_IMPORT_PATH ${CAPNPROTO_PREFIX}/c++/src)

# Nanomsg
# find package or download and build from sources otherwise
#find_package(nanomsg)

###
#
# BUILD
#
###

# Uninstall target, place it here because others might want to set
# dependencies on it
configure_file("${CMAKE_CURRENT_SOURCE_DIR}/cmake/uninstall.cmake.in"
               "${CMAKE_CURRENT_BINARY_DIR}/uninstall.cmake"
               IMMEDIATE @ONLY)
add_custom_target(uninstall_files
  COMMAND ${CMAKE_COMMAND} -P ${CMAKE_CURRENT_BINARY_DIR}/uninstall.cmake)
add_custom_target(uninstall)
add_dependencies(uninstall uninstall_files)
ensure_out_of_source_build("${PNAME} requires an out of source build. ")

configure_project(${PROJECT_MAJOR_VERSION} ${PROJECT_MINOR_VERSION} ${PROJECT_PATCH_VERSION}
                 ${CMAKE_CURRENT_SOURCE_DIR}/src/include
                 ${CMAKE_CURRENT_BINARY_DIR})
set(GEN_INCLUDE_DIR ${PROJECT_BINARY_DIR} CACHE STRING "include for gen configs")

# common includes and project settings
include_directories("${GEN_INCLUDE_DIR}") # for generated configs
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/src/include")
#include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/ezOptionParser")

set (${PNAME}_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)
set (${PNAME}_INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src/include)

if (WITH_DOCS)
  add_subdirectory(docs)
endif (WITH_DOCS)

add_library(libneo4j_client SHARED IMPORTED)
set_target_properties(libneo4j_client PROPERTIES IMPORTED_LOCATION ${LIBNEO4J_LIBRARY})
add_dependencies(libneo4j_client libneo4jclient_external)

###
## Project Sources and Tests
##
set (libopus_SOURCES
    ${libopus_SOURCE_DIR}/lib/libopus.cpp
    ${libopus_SOURCE_DIR}/lib/opus_session.cpp
    ${libopus_SOURCE_DIR}/lib/db_tr.cpp
    ${libopus_SOURCE_DIR}/lib/pvm_cache.cpp
    ${libopus_SOURCE_DIR}/lib/node.cpp
    ${libopus_SOURCE_DIR}/lib/pvm.cpp
)

set (libopus_HEADERS
    ${libopus_INCLUDE_DIR}/opus/opus.h
)

set (libopus_PRIVATE_HEADERS
    ${libopus_INCLUDE_DIR}/opus/internal/opus_session.h
    ${libopus_INCLUDE_DIR}/opus/internal/db_tr.h
    ${libopus_INCLUDE_DIR}/opus/internal/pvm_cache.h
    ${libopus_INCLUDE_DIR}/opus/internal/node.h
    ${libopus_INCLUDE_DIR}/opus/internal/pvm.h
)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(libopus_LINK libneo4j_client)

add_library(opus SHARED ${libopus_SOURCES} ${libopus_PRIVATE_HEADERS} ${libopus_HEADERS})
set_target_properties(opus PROPERTIES
  VERSION "${PROJECT_MAJOR_VERSION}.${PROJECT_MINOR_VERSION}.${PROJECT_PATCH_VERSION}"
  SOVERSION "${PROJECT_MAJOR_VERSION}"
  FRAMEWORK ON
  PUBLIC_HEADER "${libopus_HEADERS}"
  PRIVATE_HEADER "${libopus_PRIVATE_HEADERS}"
)

add_library(opus_static STATIC ${libopus_SOURCES} ${libopus_PRIVATE_HEADERS} ${libopus_HEADERS})
add_dependencies(opus libneo4j_client)
add_dependencies(opus_static libneo4j_client)

install(FILES ${PROJECT_BINARY_DIR}/config.h DESTINATION include/opus)
install(TARGETS opus opus_static
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    PUBLIC_HEADER DESTINATION include/opus
    PRIVATE_HEADER DESTINATION include/opus/internal
)

###
## Executableis:

# add tests
add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/src/tests)
