#version 460 core

out vec4 color;

in vec3 frag_pos_world;
in vec3 frag_pos_view;
in vec2 tex_coord;
in vec3 normal;

uniform sampler2D texture_atlas;
uniform sampler2DArray shadow_map;

uniform int cascade_count;
uniform float cascades[8]; // max 8

layout (std140, binding = 2) uniform light_space_transforms
{
    mat4 transforms[8];
};

float near = 0.1; 
float far  = 500.0; 

const vec4 clear_color = vec4(0.25,0.5,0.88,1.0);
const float fog_density = 7.0;
vec2 texel_size = 1.0 / textureSize(shadow_map,0).xy; // don'get the z component

//TODO: point light is hardcoded, fix this !!!
float shadow_bias = max(0.05 * (1.0 - dot(normal, normalize(vec3(0.5,0.2,-2.0)))), 0.01);
float doto = 1.0 - dot(normal , normalize(vec3(0.5,0.2,-2.0)));
  
//TODO: could be refactored
float linearize_depth(float depth) 
{
    float z = depth * 2.0 - 1.0; // back to NDC 
    return (2.0 * near * far) / (far + near - z * (far - near));	
}

float fog_intensity(float depth)
{
    float res = 1.0 - exp(-pow(fog_density * depth , 2.0));
    return clamp(res , 0.0 , 1.0 );
}

float pcf( vec3 pos , vec2 depth_deriv , int text_layer )
{
    float shadow = 0.0;

    // sample 9 texel from the shadow then average to get the value
    for ( int x = -3 ; x <= 3 ; ++x )
        for ( int y = -3; y <= 3 ; ++y )
        {
            vec2 tex_offset = (x,y) * texel_size;
            float shadow_map_depth = texture(shadow_map, vec3( ( pos.xy + tex_offset ) , text_layer) ).r;
            // the fragment is in shadow if the distance light - fragment > depth in the shadow buffer
            // meaning an object has occluded the fragment
            float fragment_depth = pos.z -shadow_bias -(depth_deriv.x * tex_offset.x + depth_deriv.y * tex_offset.y); // depth - bias
            shadow += (fragment_depth > shadow_map_depth ? 1.0 : 0.0);
        }

    return shadow / 49.0; // take the average and return
}

// calcualte the values required by receiver plane depth bias
vec2 get_derivative ( vec3 proj )
{
    // we need del(v)/del(x), del(u)/del(x), del(depth)/del(x)
    // same thing for y 
    // pack v,u and depth into a vector and call ddx and ddy on it
    
    vec3 uvd_x = dFdx(proj);
    vec3 uvd_y = dFdy(proj);

    // now calculate del(depth)/del(u) and del(depth)/del(v) 
    // TODO: Document calculations

    // we will manually calculate the inverse of the 2x2 matrix
    //                   1 / ( del(u)/del(x) * del(v)/del(y) - del(v)/del(x) * del(u)/del(y) )
    float inverse_det = 1/(uvd_x.x * uvd_y.y - uvd_x.y * uvd_y.x);
    vec2 d_uv; // result vector
    // calculate del(d)/del(u) = del(v)/del(y) * del(d)/del(x) - del(v)/del(x) * del(d)/del(y)
    d_uv.x = (uvd_y.y * uvd_x.z - uvd_x.y * uvd_y.z) ;
    // calculate del(d)/del(v) = -del(u)/del(y) * del(d)/del(x) + del(u)/del(x) * del(d)/del(y)
    d_uv.y = (-uvd_y.x * uvd_x.z + uvd_x.x * uvd_y.z) ;

    d_uv *= inverse_det;

    return d_uv;
}

float is_in_shadow(vec4 point, int shadow_map_layer, out bool color_x, out bool color_y)
{
    vec3 proj = point.xyz / point.w;
    // z is between -1 and 1, we need => 0 and 1
    proj = proj * 0.5 + 0.5;

    if (proj.z > 1.0) // outside the shadow maps' range, no shadow
        return 0.0;

    color_x = (int (proj.x * 2048.0)) % 2 == 1;
    color_y = (int (proj.y * 2048.0)) % 2 == 1;

    vec2 depth_deriv = get_derivative( proj );

    // float depth = texture(shadow_map, vec3( ( proj.xy ) , shadow_map_layer) ).r;
    // return (proj.z - shadow_bias) > depth ? 1.0 : 0.0;
    return pcf( proj, depth_deriv , shadow_map_layer);
}

void main()
{
    // choose the light space transform according to which cascade we are targeting
    float depth = abs(frag_pos_view.z);
    
    // determine which layer we must use
    int layer = cascade_count-1; // if nothing matches, use the biggest cascade
    for ( int i = 1 ; i < cascade_count ; ++i )
    {
        if ( depth < cascades[i] )
        {
            layer = i-1;
            break;
        }
    }

    bool color_x , color_y ;

    vec4 frag_light_space_pos = transforms[layer] * vec4(frag_pos_world,1.0);
    float shadow = is_in_shadow(frag_light_space_pos , layer, color_x, color_y);
    float diffuse = max(dot(normal,normalize(vec3(0.5,0.5,0.5))),0.0);
    // for now, being in shadow just means the texture's albedo colors get a bit darker

    float ambient = 0.5;
    // vec4 overlay_color = (color_x != color_y) ? vec4(1.5,0.5,0.5,1.0) : vec4(0.5,0.5,1.5,1.0); // display a sort of checkered pattern, blue and red
    vec4 albedo = (ambient + (1.0 - shadow) * diffuse ) * texture(texture_atlas, tex_coord);
    float fog_intensity =  fog_intensity(linearize_depth(gl_FragCoord.z) / far);
    color = albedo ;//* vec4(vec3(xxx*50), 1.0); //fog_intensity * clear_color + (1-fog_intensity) * albedo;
}