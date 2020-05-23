#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aUv;

uniform vec3 cameraPos = vec3(0, 0, 1);

out vec2 uv;

void main()
{
    uv = aUv;
    // TODO: refactor camera to use orthogonal matrix to avoid this 
    gl_Position = vec4(vec3((aPos.xy + vec2(cameraPos.xy)), aPos.z) * cameraPos.z, 1.0);
}