cmake_minimum_required(VERSION 3.5)
project(src_sink VERSION 0.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20 CACHE STRING "The C++ standard is cached for visibility in external tools." FORCE)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_BUILD_TYPE "Release" CACHE STRING "Choose the type of build." FORCE)

set(MAG_MAIN_TARGET hello)

include(GNUInstallDirs)

set (MagnitionLIB  MagnitionLib)
set (ReactorSDK MagnitionReactorSDK)
set (MAGNITION_REACTORS MagnitionReactors)

set (MagnitionLIB_PATH ${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_INCLUDEDIR}/${MagnitionLIB})
set (ReactorSDK_PATH ${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_INCLUDEDIR}/${ReactorSDK})
set (MAGNITION_REACTORS_PATH ${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_INCLUDEDIR}/${MAGNITION_REACTORS})

add_executable(${MAG_MAIN_TARGET}
    main.cc
)

include(${ReactorSDK_PATH}/magnition_reactor_base.cmake)

target_compile_options(${MAG_MAIN_TARGET} PRIVATE -Wall -Wextra -pedantic)