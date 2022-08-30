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
        gl_Position = transforms[gl_InvocationID] * gl_in[i].gl_Position;
        gl_Layer = gl_InvocationID;
        EmitVertex();
    }
    EndPrimitive();
}


