#version 460 core

in vec2 pos;
in vec2 uv_in;

out vec2 uv;

void main()
{
    gl_Position = vec4(pos,0,1);
    uv = uv_in;
}