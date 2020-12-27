# truck-platform

`truck-platform` is a graphic utility library based on wgpu.

This crate is independent from other truck crates except `truck-base`.
It provides an API that allows users to handle drawing elements in a unified manner.
By implementing the [`Rendered`] trait, developers can define
their own rendering elements and have them rendered in [`Scene`]
in the same way as other rendering elements provided by truck.

This documentation is intended to be read by two kinds of people: users and developers.
Users, those who just want to draw the shape of an existing mesh or boundary representation,
will only use:
- [`Scene`],
- [`SceneDescriptor`],
- [`DeviceHandler`],
- [`Camera`], and
- [`Light`].

If you are a developer, who wants to try out new
visual representations, you can implement Rendered in your own structure and standardize it in
a form that can be used by users in [`Scene`].

The sample code in this crate is for developers.
Users may wish to refer to the one in `truck-rendimpl`.

[`Rendered`]: ./trait.Rendered.html
[`Scene`]: ./struct.Scene.html
[`DeviceHandler`]: ./struct.DeviceHandler.html
[`SceneDescriptor`]: ./struct.SceneDescriptor.html
[`Camera`]: ./struct.Camera.html
[`Light`]: ./struct.Light.html

License: Apache License 2.0

# Sample Codes
## glsl-toy
A sample of creating a render object by implementing "Rendered" in a new structure.

One can use xyr fragment shader in the following way:
- Enter the shader path as an argument when executing the program.
- Drag and drop the shader into the window.

The shader syntax follows that of shadertoy. One can use `iResolution`, `iTime` and `iMouse`.
Since this is a simple sample, not supports `iChannel`s, i.e. buffering textures, sounds, and so on.
The default shader sample is "newton-cuberoot.frag" in the same directory.
