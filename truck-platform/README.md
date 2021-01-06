# truck-platform
[![Crates.io](https://img.shields.io/crates/v/truck-platform.svg)](https://crates.io/crates/truck-platform)
[![Docs.rs](https://docs.rs/truck-platform/badge.svg)](https://docs.rs/truck-platform)

Graphic utility library based on wgpu.
## Dependencies
The dev-dependencies of this crate includes [CMake](https://cmake.org).

# Sample Codes
## glsl-toy
A sample of creating a render object by implementing "Rendered" in a new structure.

One can use xyr fragment shader in the following way:
- Enter the shader path as an argument when executing the program.
- Drag and drop the shader into the window.

The shader syntax follows that of shadertoy. One can use `iResolution`, `iTime` and `iMouse`.
Since this is a simple sample, not supports `iChannel`s, i.e. buffering textures, sounds, and so on.
The default shader sample is "newton-cuberoot.frag" in the same directory.
