# kessler-c

C and C++ bindings for [Kessler](../README.md). The header is [`include/kessler.h`](include/kessler.h).

## Use from CMake

[Corrosion](https://github.com/corrosion-rs/corrosion) builds the Rust library as part of your CMake build (you need
Rust 1.87 or later installed). [`example/`](example) is a complete project:

```cmake
include(FetchContent)
FetchContent_Declare(Corrosion GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git GIT_TAG v0.6.1)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH path/to/kessler/Cargo.toml CRATES kessler-c CRATE_TYPES staticlib)
target_include_directories(kessler_c INTERFACE path/to/kessler/kessler-c/include)
target_link_libraries(your_target PRIVATE kessler_c)
```

## Usage

```cpp
#include <kessler.h>

kessler::Fragments debris; // frees itself when it goes out of scope
// All inputs and outputs are SI (metres, kilograms, metres per second).
KesslerStatus status = kessler_explosion(839.0f, KESSLER_ROCKET_BODY, 0.1f, /* seed */ 42, debris.out());
if (status == KESSLER_STATUS_OK) {
    debris->len;        // number of fragments
    debris->mass[0];    // one entry per fragment
    debris->delta_v[0]; // 3 * len values: x0, y0, z0, x1, ...
}

kessler_collision(500.0f, KESSLER_SPACECRAFT, 5.0f, KESSLER_SPACECRAFT, 9500.0f, 0.1f, 42, debris.out());
```

From C, pass a `KesslerFragments` you declared and call `kessler_fragments_free` on it when done. Invalid input returns
a `KESSLER_STATUS_*` error and leaves the fragments empty. A given seed gives the same fragments as the Rust, Python and
JavaScript packages.

After changing `src/lib.rs`, regenerate the header:

```shell
cbindgen --config kessler-c/cbindgen.toml --output kessler-c/include/kessler.h kessler-c
```
