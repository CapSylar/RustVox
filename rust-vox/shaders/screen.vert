#version 460 core

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords_i;

out vec2 TexCoords;

void main()
{
    gl_Position = vec4(pos.x,pos.y,0.0,1.0);
    TexCoords = tex_coords_i; // just forward
}