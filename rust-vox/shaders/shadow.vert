#version 460 core

layout (location = 0) in vec3 pos;

uniform vec3 animation_offset;

void main()
{
    vec4 pos_out = vec4(pos.x + animation_offset.x, pos.y + animation_offset.y , pos.z + animation_offset.z , 1.0) ;
    // pancake the geometry that is behind the near plane to z = 0
    if ( pos_out.z < -pos_out.w ) // if the vertex is behind the near plane
        pos_out.z = -pos_out.w + 0.00001; // clamp it to the near plane
    
    gl_Position = pos_out;
}