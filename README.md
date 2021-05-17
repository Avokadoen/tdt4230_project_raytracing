# TDT4230 Project: GPGPU raytracing

This is a fork of [GPUPE](https://github.com/Avokadoen/GPUPE) with the aim of implementing raytracing on 
opengl compute shaders using [Raytracing in one weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

[![Demo](https://img.youtube.com/vi/5jLadHg7BZ4/0.jpg)](https://www.youtube.com/watch?v=5jLadHg7BZ4)

# How to build

Download [rustup](https://www.rust-lang.org/learn/get-started). Code is tested with rustc version 1.51

When that is done you should be able to run ```cargo run``` from the project folders

# Keydindings

* wasd - move camera
* mouse movement - turn camera
* ctr - move down
* space - move up
* left mouse - spawn voxel
* right mouse - remove voxel
* 1 -> 9 - change voxel spawn type

# Sources

Raytracing concepts: https://raytracing.github.io/books/RayTracingInOneWeekend.html
Cube intersection test: http://jcgt.org/published/0007/03/04/

Storing an octree in a texture: https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu?fbclid=IwAR1iQ3i-t28gnm_XwP-MViIY11C4V9jjKniQonVQAbXym3BXcZ2muIofjWQ 

Using glutin (most of main is taken from this): https://github.com/mgimle/gloom-rs

Thanks Nercury for a great blog about opengl in rust!
http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html

Using gl and sdl bindings for rust: https://github.com/Nercury/rust-and-opengl-lessons 

Opengl textures: https://learnopengl.com/Getting-started/Textures

Using opengl compute shaders: https://antongerdelan.net/opengl/compute.html

Reading vbo in compute shader: https://stackoverflow.com/a/21344861/11768869

Generating blue noise: https://blog.demofox.org/2017/10/20/generating-blue-noise-sample-points-with-mitchells-best-candidate-algorithm/

Distributed hemosphere unit vector: http://jcgt.org/published/0007/04/01/

Frisvad orthogonal vectors: https://backend.orbit.dtu.dk/ws/portalfiles/portal/126824972/onb_frisvad_jgt2012_v2.pdf