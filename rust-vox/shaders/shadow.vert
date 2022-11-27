#version 460 core

layout (location = 0) in vec3 pos;

void main()
{
    vec4 pos_out = vec4(pos.x, pos.y, pos.z, 1.0) ;    
    gl_Position = pos_out;
}