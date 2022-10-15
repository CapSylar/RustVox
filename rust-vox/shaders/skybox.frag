#version 460 core

in gs_out_fs_in
{
    flat vec3 color[4]; // sent on with each triangle, dictated by the provoking vertex
    vec2 uv;
} fs_in;

out vec4 color;

void main()
{
    // interpolate the quad's colors
    
    vec3 c1 = mix(fs_in.color[0],fs_in.color[3], fs_in.uv.x);
    vec3 c2 = mix(fs_in.color[1],fs_in.color[2], fs_in.uv.x);

    color = vec4(mix(c1,c2,fs_in.uv.y),1.0);
}