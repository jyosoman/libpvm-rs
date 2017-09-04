##
# build tests
#
set(TESTS_DIR ${CMAKE_CURRENT_SOURCE_DIR})

if(WITH_TESTS)

  set(TEST_LINK opus ${libopus_LINK} ${GTEST_MAIN_LIBRARY} ${GTEST_LIBRARY} pthread)

  set (api_test_SOURCES
    ${TESTS_DIR}/api_test.cpp
  )
  lib_test(api_test "${api_test_SOURCES}" "${TEST_LINK}")
  add_dependencies(api_test libneo4j_client)

endif(WITH_TESTS)
