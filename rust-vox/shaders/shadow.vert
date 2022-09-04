#version 460 core

layout (location = 0) in vec3 pos;

uniform vec3 animation_offset;

void main()
{
    vec4 pos_out = vec4(pos.x + animation_offset.x, pos.y + animation_offset.y , pos.z + animation_offset.z , 1.0) ;    
    gl_Position = pos_out;
}