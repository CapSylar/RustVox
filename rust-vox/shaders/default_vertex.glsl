#version 330 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;

uniform mat4 transform;
uniform vec3 animation_offset;

out vec2 tex_coord;

void main()
{
    gl_Position = transform * vec4(pos.x + animation_offset.x, pos.y + animation_offset.y , pos.z + animation_offset.z , 1.0) ;
    tex_coord = in_tex_coord;
}