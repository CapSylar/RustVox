#version 460 core

layout(std140, binding = 0) uniform transforms
{
    mat4 perspective;
    mat4 view;
    mat4 view_no_trans; // view but with no translation = 0 always
};

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 vert_color;

out vs_out_gs_in
{
    vec3 color;
} vs_out;

void main()
{
    vec4 pos = perspective * view_no_trans * vec4(pos,1.0);
    gl_Position = pos.xyww;
    vs_out.color = vert_color;
}