#version 330 core
            
out vec4 color;
in vec2 tex_coord;

uniform sampler2D test_texture1;

float near = 0.1; 
float far  = 500.0; 

const vec4 clear_color = vec4(0.25,0.5,0.88,1.0);
const float fog_density = 7.0;
  
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

void main()
{
    float fog_intensity =  fog_intensity(linearize_depth(gl_FragCoord.z) / far); // divide by far for demonstration
    color = fog_intensity * clear_color + (1-fog_intensity) * texture(test_texture1, tex_coord); //= vec4(vec3(depth), 1.0);
}