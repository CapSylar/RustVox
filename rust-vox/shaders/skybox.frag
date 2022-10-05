#version 460 core

out vec4 color;
in vec3 frag_color;

void main()
{
    color = vec4(frag_color,1.0);
}