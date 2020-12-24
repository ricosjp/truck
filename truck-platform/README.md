# truck-platform
Graphic utility library based on wgpu

## Sample Codes
### glsl-toy
A sample of creating a render object by implementing "Rendered" in a new structure.
 
One can use xyr fragment shader in the following way:
 - Enter the shader path as an argument when executing the program.
 - Drag and drop the shader into the window.

The shader syntax follows that of shadertoy.
Only supports uniforms `iResolution` and `iTime`, and not supports `iMouse` or many `iChannel`s.
The default shader sample is "newton-cuberoot.frag" in the same directory.
