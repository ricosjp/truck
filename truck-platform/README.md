# truck-platform

[![Crates.io](https://img.shields.io/crates/v/truck-platform.svg)](https://crates.io/crates/truck-platform) [![Docs.rs](https://docs.rs/truck-platform/badge.svg)](https://docs.rs/truck-platform)

Graphic utility library based on wgpu.

## Sample Codes

### wgsl-sandbox

A sample of creating a render object by implementing `Rendered` on a new structure.

One can use their WGSL shader in the following way:

- Enter the shader path as an argument when executing the program.
- Drag and drop the shader into the window.

The rule of shaders:

- One can draw an image by implementing the function:

```wgsl
vec4<f32> main_image(coord: vec2<f32>, env: Environment);
```

- The parameter `coord` is the fragment coordinate. The origin is the lower left.
- The parameter `env` has the environment information. The declaration of struct is the following:

```wgsl
struct Environment {
    resolution: vec2<f32>;  // the resolution of the image
    mouse: vec4<f32>;       // the mouse information behaving the same as `iMouse` in Shadertoy.
    time: f32;              // the number of seconds since the application started.
};
```

Also, see the sample `newton-cuberoot.wgsl`, default shader, in `examples`.
