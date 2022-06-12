#version 330 core
            
out vec4 color;
in vec2 tex_coord;

uniform sampler2D test_texture1;
void main()
{
    color = texture(test_texture1, tex_coord);
}