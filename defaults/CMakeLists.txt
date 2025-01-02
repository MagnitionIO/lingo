cmake_minimum_required(VERSION 3.5)
project(src_sink VERSION 0.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20 CACHE STRING "The C++ standard is cached for visibility in external tools." FORCE)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_BUILD_TYPE "Release" CACHE STRING "Choose the type of build." FORCE)

set(LF_MAIN_TARGET hello)

set (SHARED_PATH "/usr/local/share/magnition")
set (SHARED_LIB_PATH "/usr/local/lib")

add_executable(${LF_MAIN_TARGET}
    main.cc
)

include("${SHARED_PATH}/MagnitionReactorBase/magnition_reactor_base.cmake")

if(MSVC)
  target_compile_options(${LF_MAIN_TARGET} PRIVATE /W4)
else()
  target_compile_options(${LF_MAIN_TARGET} PRIVATE -Wall -Wextra -pedantic)
endif()