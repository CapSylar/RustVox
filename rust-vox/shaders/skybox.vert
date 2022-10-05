#version 460 core

layout(std140, binding = 0) uniform transforms
{
    mat4 perspective;
    mat4 view;
    mat4 view_no_trans;
};

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 vert_color;

out vec3 frag_color;

void main()
{
    vec4 pos = perspective * view_no_trans * vec4(pos,1.0);
    gl_Position = pos.xyww;
    frag_color = vert_color;
}