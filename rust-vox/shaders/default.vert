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
layout (location = 1) in uint tex_coord_u;
layout (location = 2) in uint tex_coord_v;
layout (location = 3) in uint texture_index_in;
layout (location = 4) in uint normal_index;

uniform vec3 animation_offset;

out vec3 frag_pos_world; // fragment position in world space
out vec3 frag_pos_view; // fragment position in the camera's view space
out uint texture_index;
out vec2 texture_uv; // needs to be vec2 so that interpolation is enabled
out vec3 normal;

void main()
{
    vec4 pos = vec4(pos.x + animation_offset.x, pos.y + animation_offset.y, pos.z + animation_offset.z , 1.0);
    frag_pos_world = vec3(pos);
    vec4 pos_view = view * pos;
    frag_pos_view = vec3(pos_view);
    
    gl_Position = perspective * pos_view ;

    texture_index = texture_index_in;

    // TODO: why does the second bit manip not work ?
    texture_uv = vec2(tex_coord_u,tex_coord_v);

    vec3 normal_vec = normal_lut[normal_index];
    normal = normal_vec;
}