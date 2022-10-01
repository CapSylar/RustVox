#version 460 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in uint normal_index;

uniform mat4 view;
uniform mat4 ortho;
out vec4 frag_color;

void main()
{
    vec4 pos = ortho * view * vec4(pos,1.0);
    gl_Position = pos.xyww;
    frag_color = in_tex_coord.x > 0 ?  vec4(105/255.0,206/255.0,250/255.0,1.0) : vec4(252/255.0,61/255.0,3/255.0,1.0);
}