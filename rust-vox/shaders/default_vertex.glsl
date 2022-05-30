#version 330 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;

uniform mat4 transform;

out vec3 color;
out vec2 tex_coord;

void main()
{
    gl_Position = transform * vec4(pos.x, pos.y, pos.z, 1.0);
    tex_coord = in_tex_coord;
}