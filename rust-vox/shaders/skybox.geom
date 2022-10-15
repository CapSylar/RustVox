#version 460 core

layout (lines_adjacency) in;
layout (triangle_strip, max_vertices = 6) out;

in vs_out_gs_in
{
    vec3 color;
} gs_in[4];

out gs_out_fs_in
{
    flat vec3 color[4]; // sent on with each triangle, dictated by the provoking vertex
    vec2 uv;
} gs_out;

void main()
{
    gl_Position = gl_in[0].gl_Position;
    gs_out.uv = vec2(0.0,0.0);
    EmitVertex();
    
    gl_Position = gl_in[1].gl_Position;
    gs_out.uv = vec2(0.0,1.0);
    EmitVertex();

    // this is the provoking vertex
    gl_Position = gl_in[2].gl_Position;
    gs_out.uv = vec2(1.0,1.0);
    gs_out.color[0] = gs_in[0].color;
    gs_out.color[1] = gs_in[1].color;
    gs_out.color[2] = gs_in[2].color;
    gs_out.color[3] = gs_in[3].color;
    EmitVertex();
    EndPrimitive();

    gl_Position = gl_in[0].gl_Position;
    gs_out.uv = vec2(0.0,0.0);
    EmitVertex();
    
    gl_Position = gl_in[2].gl_Position;
    gs_out.uv = vec2(1.0,1.0);
    EmitVertex();

    gl_Position = gl_in[3].gl_Position;
    gs_out.uv = vec2(1.0,0.0);
    gs_out.color[0] = gs_in[0].color;
    gs_out.color[1] = gs_in[1].color;
    gs_out.color[2] = gs_in[2].color;
    gs_out.color[3] = gs_in[3].color;
    EmitVertex();
    EndPrimitive();
}