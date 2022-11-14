#version 460 core

in vec2 uv;
out vec4 color;

uniform sampler2D ui_texture;

void main()
{
    color = texture(ui_texture,uv);
}