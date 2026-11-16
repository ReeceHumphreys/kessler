cmake_minimum_required(VERSION 3.22)
project(kessler_example CXX)

include(FetchContent)
FetchContent_Declare(Corrosion GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git GIT_TAG v0.6.1)
FetchContent_MakeAvailable(Corrosion)

# In your own project, point this at a checkout of the kessler repo (or fetch it with FetchContent).
corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../Cargo.toml CRATES kessler-c CRATE_TYPES staticlib)
target_include_directories(kessler_c INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

add_executable(example main.cpp)
target_compile_features(example PRIVATE cxx_std_17)
target_link_libraries(example PRIVATE kessler_c)
