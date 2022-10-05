#version 460 core

const vec3 normal_lut[6] = vec3[6](
    vec3(0.0,1.0,0.0), // POSY
    vec3(0.0,-1.0,0.0), // NEGY
    vec3(0.0,0.0,1.0), // POSZ
    vec3(0.0,0.0,-1.0), // NEGZ
    vec3(1.0,0.0,0.0), // POSX
    vec3(-1.0,0.0,0.0) // NEGX
);

// transforms ubo
layout (std140, binding = 0) uniform transforms
{
    mat4 perspective;
    mat4 view;
};

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_coord;
layout (location = 2) in uint normal_index;

// uniform mat4 view;
// uniform mat4 perspective;
uniform vec3 animation_offset;

out vec3 frag_pos_world; // fragment position in world space
out vec3 frag_pos_view; // fragment position in the camera's view space
out vec2 tex_coord;
out vec3 normal;
// out vec4 frag_light_space_pos; // fragment position in light space

void main()
{
    vec4 pos = vec4(pos.x + animation_offset.x, pos.y + animation_offset.y, pos.z + animation_offset.z , 1.0);
    frag_pos_world = vec3(pos);
    vec4 pos_view = view * pos;
    frag_pos_view = vec3(pos_view);
    
    gl_Position = perspective * pos_view ;

    tex_coord = in_tex_coord;
    vec3 normal_vec = normal_lut[normal_index];
    normal = normal_vec;
}