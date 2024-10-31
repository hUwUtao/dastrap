project(daStrap)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(libs/daScript/3rdparty/fmt/include)

add_library(libDaStrap STATIC
    src/interop/extended/ext.cpp
    # src/interop/extended/ext.h
)
target_include_directories(libDaStrap PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/libs/daScript/include
)
target_link_libraries(libDaStrap PRIVATE libs/daScript)

install(TARGETS libDaStrap
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
