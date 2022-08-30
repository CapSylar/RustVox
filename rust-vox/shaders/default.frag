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

layout (std140, binding = 0) uniform light_space_transforms
{
    mat4 transforms[8];
};

float near = 0.1; 
float far  = 500.0; 

const vec4 clear_color = vec4(0.25,0.5,0.88,1.0);
const float fog_density = 7.0;
vec2 texel_size = 1.0 / textureSize(shadow_map,0).xy; // don'get the z component

//TODO: point light is hardcoded, fix this !!!
float shadow_bias = max(0.05 * (1.0 - dot(normal, normalize(vec3(0.5,0.2,0)))), 0.005);
  
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

float pcf( vec3 pos , int text_layer )
{
    float shadow = 0.0;

    // sample 9 texel from the shadow then average to get the value
    for ( int x = -1 ; x <= 1 ; ++x )
        for ( int y = -1; y <= 1 ; ++y )
        {
            float depth = texture(shadow_map, vec3( ( pos.xy + (x,y) * texel_size) , text_layer) ).r;
            // the fragment is in shadow if the distance light - fragment > depth in the shadow buffer
            // meaning an object has occluded the fragment
            shadow += (pos.z - shadow_bias) > depth ? 1.0 : 0.0;
        }

    return shadow / 9.0; // take the average and return
}

float is_in_shadow(vec4 point , int shadow_map_layer)
{
    vec3 proj = point.xyz / point.w;
    // z is between -1 and 1, we need => 0 and 1
    proj = proj * 0.5 + 0.5;

    if (proj.z > 1.0) // outside the shadow maps' range, no shadow
        return 0.0;

    float depth = texture(shadow_map, vec3( ( proj.xy ) , shadow_map_layer) ).r;
    return (proj.z - shadow_bias) > depth ? 1.0 : 0.0;
    // return pcf( proj, shadow_map_layer);
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
    vec4 cascade_color = vec4(0.2,0.2,0.2,1.0) * (layer+1); // the closer the darker

    vec4 frag_light_space_pos = transforms[layer] * vec4(frag_pos_world,1.0);

    float shadow = is_in_shadow(frag_light_space_pos , layer);
    float diffuse = max(dot(normal,normalize(vec3(0.5,0.5,0.5))),0.0);
    // for now, being in shadow just means the texture's albedo colors get a bit darker

    float ambient = 0.5;
    vec4 albedo = (ambient + (1.0 - shadow) * diffuse ) * texture(texture_atlas, tex_coord);
    // vec4 albedo = cascade_color;
    float fog_intensity =  fog_intensity(linearize_depth(gl_FragCoord.z) / far);
    color = albedo; //fog_intensity * clear_color + (1-fog_intensity) * albedo;
}