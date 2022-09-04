#version 460 core

layout (triangles, invocations = 1) in;
layout (triangle_strip, max_vertices = 3) out;

layout (std140, binding = 0) uniform light_space_transforms
{
    mat4 transforms[8];
};

void main()
{
    for(int i = 0; i < 3;++i)
    {
        vec4 pos = transforms[gl_InvocationID] * gl_in[i].gl_Position;
            
        // // pancake the geometry that is behind the near plane to z = 0
        if ( pos.z < -pos.w ) // if the vertex is behind the near plane
            pos.z = -pos.w; // clamp it to the near plane

        gl_Position = pos;
        gl_Layer = gl_InvocationID;
        EmitVertex();
    }
    EndPrimitive();
}


