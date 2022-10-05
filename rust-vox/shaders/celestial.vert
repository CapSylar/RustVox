// for now this shader is used to render the sky
#version 460 core

layout(std140, binding = 0) uniform transforms
{
    mat4 perspective;
    mat4 view;
    mat4 view_no_trans; // view but with no translation = 0 always
};

uniform mat4 model;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;

out vec2 tex_coord;

void main()
{
    vec4 pos = perspective * view_no_trans * model * vec4(pos,1.0);
    gl_Position = pos.xyww; // set z to w such that the depth is maximum
    tex_coord = in_tex_coord;
}