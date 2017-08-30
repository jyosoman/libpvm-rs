## CMake documentation
#
# Defines the following custom targets:
#   doxygen - generates doxygen documentation from source files in xml format
#   docs    - generates the sphinx html output of the documentation
#
##

# configured documentation tools and intermediate build results
set(BINARY_BUILD_DIR "${CMAKE_CURRENT_BINARY_DIR}/_tmp")

if(WITH_API_DOCS)
  find_package(Doxygen COMPONENTS Dot)

  ## Doxygen Configuration

  if(NOT DEFINED DOXYGEN_STRIP_PATH)
      set(DOXYGEN_STRIP_PATH ${PROJECT_SOURCE_DIR})
  endif()
  if(NOT DEFINED DOXYGEN_INPUT)
      set(DOXYGEN_INPUT ${PROJECT_SOURCE_DIR})
  endif()
  if(NOT DEFINED DOXYGEN_EXCLUDE)
    set(DOXYGEN_EXCLUDE "${PROJECT_SOURCE_DIR}/build ${PROJECT_SOURCE_DIR}/home ${PROJECT_SOURCE_DIR}/docs ${PROJECT_SOURCE_DIR}/external")
  endif()

  configure_file(
      "${CMAKE_CURRENT_SOURCE_DIR}/doxygen.conf.in"
      "${BINARY_BUILD_DIR}/doxygen.conf"
      @ONLY)

  add_custom_target(doxygen
      COMMAND doxygen ${BINARY_BUILD_DIR}/doxygen.conf
      COMMENT "Build doxygen xml files used by sphinx/breathe."
  )
endif(WITH_API_DOCS)



if(WITH_DOCS)
  find_package(Sphinx)
  ## Sphinx Configuration

  if(NOT DEFINED SPHINX_THEME)
      set(SPHINX_THEME sphinx_rtd_theme)
  endif()
  if(NOT DEFINED SPHINX_THEME_DIR)
      set(SPHINX_THEME_DIR 'themes')
  endif()

  # Sphinx cache with pickled ReST documents
  set(SPHINX_CACHE_DIR "${CMAKE_CURRENT_BINARY_DIR}/_doctrees")
  # HTML output directory
  set(SPHINX_HTML_DIR "${CMAKE_CURRENT_BINARY_DIR}/html")
  # Use the doxygen API documentation from path:
  set(SPHINX_DOXYGEN_PATH "${CMAKE_CURRENT_BINARY_DIR}/doxyxml")

  configure_file(
      "${CMAKE_CURRENT_SOURCE_DIR}/conf.py.in"
      "${BINARY_BUILD_DIR}/conf.py"
      @ONLY)
  file(COPY ${CMAKE_CURRENT_SOURCE_DIR}/_static
            ${CMAKE_CURRENT_SOURCE_DIR}/_templates
            ${CMAKE_CURRENT_SOURCE_DIR}/ext
            ${CMAKE_CURRENT_SOURCE_DIR}/themes
       DESTINATION ${BINARY_BUILD_DIR})

  if(WITH_API_DOCS)
    add_custom_target(docs
        ${SPHINX_EXECUTABLE}
        -q -b html
        -c "${BINARY_BUILD_DIR}"
        -d "${SPHINX_CACHE_DIR}"
        "${CMAKE_CURRENT_SOURCE_DIR}"
        "${SPHINX_HTML_DIR}"
        DEPENDS doxygen
        COMMENT "Building HTML documentation with Sphinx ${CMAKE_CURRENT_SOURCE_DIR}")
  else(WITH_API_DOCS)
    add_custom_target(docs
        ${SPHINX_EXECUTABLE}
        -q -b html
        -c "${BINARY_BUILD_DIR}"
        -d "${SPHINX_CACHE_DIR}"
        "${CMAKE_CURRENT_SOURCE_DIR}"
        "${SPHINX_HTML_DIR}"
        COMMENT "Building HTML documentation with Sphinx ${CMAKE_CURRENT_SOURCE_DIR}")
  endif(WITH_API_DOCS)

   if("${CMAKE_GENERATOR}" MATCHES Make)
        install(CODE "execute_process(COMMAND ${CMAKE_BUILD_TOOL} docs WORKING_DIRECTORY \"${CMAKE_CURRENT_BINARY_DIR}\")")
        install(DIRECTORY ${SPHINX_HTML_DIR}/ DESTINATION docs/${PROJECT_NAME})
   endif()
 endif(WITH_DOCS)
