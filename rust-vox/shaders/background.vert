// for now this shader is used to render the sky
#version 460 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in uint normal_index;

out vec2 tex_coord;

uniform mat4 view;
uniform mat4 perspective;

void main()
{
    vec4 pos = perspective * view * vec4(pos,1.0);
    gl_Position = pos.xyww; // set z to w such that the depth is maximum
    tex_coord = in_tex_coord;
}