#version 460 core

out vec4 color;
in vec2 tex_coord;

uniform float sub;
uniform sampler2D text;

void main()
{
    color = texture(text,tex_coord-vec2(sub,0));
}