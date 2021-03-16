# TDT4230 Project: GPGPU raytracing

This is a fork of [GPUPE](https://github.com/Avokadoen/GPUPE) with the aim of implementing raytracing on 
opengl compute shaders using [Raytracing in one weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

# Dependencies

On windows you will need [mingw](http://mingw-w64.org/doku.php) to compile SDL2. You can read more about it in Rust-SDL2 [readme](https://github.com/Rust-SDL2/rust-sdl2)

# Sources
Thanks Nercury for a great blog about opengl in rust!
http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html

Using gl and sdl bindings for rust: https://github.com/Nercury/rust-and-opengl-lessons 

Opengl textures: https://learnopengl.com/Getting-started/Textures

Using opengl compute shaders: https://antongerdelan.net/opengl/compute.html

Raytracing concepts: https://raytracing.github.io/books/RayTracingInOneWeekend.html

Reading vbo in compute shader: https://stackoverflow.com/a/21344861/11768869

Blue noise texure: http://momentsingraphics.de/BlueNoise.html

Storing an octree in a texture: https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu?fbclid=IwAR1iQ3i-t28gnm_XwP-MViIY11C4V9jjKniQonVQAbXym3BXcZ2muIofjWQ 