# GL GPUPE

Currently: just a playground for opengl + sdl in rust

Planned: 
This is a proof of concept for a pixel engine I have been drafting. 
The plan is to use gfx-hal + a window framework (winit?) to build the actual engine,
I wanted to have a simpler environment to build a prototype in to get some experience first.
If the mistakes are held to a minimum I will attempt to build the high-level systems in a way so that I can simply swap out rendering and window context systems.

The draft can be found [here TODO](todo). 

# Gifs

Using compute shader to update a texture

[compute shader test](https://i.imgur.com/ZeeIWbb.gif)

[compute shader test2](https://i.imgur.com/9YsamC4.gif)

[compute fragments bug](https://i.imgur.com/jaUMwTL.gif)

![first working water demo](https://i.imgur.com/A4S0D4M.gif)

# Contribution

The actual implementation will accept contribution. However, this prototype will not as it serves as a learning experience for me

# Sources
Thanks Nercury for a great blog about opengl in rust!
http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html

Using gl and sdl bindings for rust: https://github.com/Nercury/rust-and-opengl-lessons 

Opengl textures: https://learnopengl.com/Getting-started/Textures

Using opengl compute shaders: https://antongerdelan.net/opengl/compute.html