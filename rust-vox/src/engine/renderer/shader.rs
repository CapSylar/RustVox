use std::{fs, io::Error, ffi::{CStr, CString}, collections::HashMap};

use glam::{Vec4, Mat4};

pub struct Shader
{
    // state
    renderer_id: u32,

    // debugging
    vertex_filepath : String,
    fragment_filepath : String,

    // uniform locations
    locations: HashMap<CString,i32>
}

impl Shader
{
    /// Compile + Link the vertex and fragment shaders
    pub fn new(vertex_filepath: String, fragment_filepath: String) -> Result<Self,Error>
    {
        // load the vertex and fragment shader source code
        let vertex_src = fs::read_to_string(&vertex_filepath)?;
        let fragment_src = fs::read_to_string(&fragment_filepath)?;

        let mut program: u32 = 0;

        unsafe
        {
            // create program
            // first create the vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

            // copy shader source into the specified shader object
            gl::ShaderSource(vertex_shader, 1, &(vertex_src.as_bytes().as_ptr().cast()), &(vertex_src.len().try_into().unwrap()));
            gl::CompileShader(vertex_shader);

            // get compilation status on the *hopefully* compiled vertex shader
            let mut success = 0;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

            if success == 0
            {
                let mut log_length = 0_i32;
                gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_length );

                let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
                // get shader log text
                gl::GetShaderInfoLog(vertex_shader, log_length , &mut log_length , log_text.as_mut_ptr().cast());

                panic!("Vertex Shader Compile Error: {}", String::from_utf8_lossy(&log_text) );
            }

            // same for the fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            // copy shader source into the specified shader object
            gl::ShaderSource(fragment_shader, 1, &(fragment_src.as_bytes().as_ptr().cast()), &(fragment_src.len().try_into().unwrap()));
            gl::CompileShader(fragment_shader);

            //TODO: refactor duplicate code 
            // get compilation status on the *hopefully* compiled fragment shader
            let mut success = 0;
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

            if success == 0
            {
                let mut log_length = 0_i32;
                gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_length );

                let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
                // get shader log text
                gl::GetShaderInfoLog(fragment_shader, log_length , &mut log_length , log_text.as_mut_ptr().cast());

                panic!("Fragment Shader Compile Error: {}", String::from_utf8_lossy(&log_text) );
            }

            // attach shader and link program
            program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            //TODO: refactor duplicate code
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

            if success == 0
            {
                let mut log_length = 0_i32;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length );

                let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
                // get shader log text
                gl::GetProgramInfoLog(program, log_length , &mut log_length , log_text.as_mut_ptr().cast());

                panic!("Program Link Error: {}", String::from_utf8_lossy(&log_text) );
            }
        }

        Ok(Self{renderer_id:program , vertex_filepath , fragment_filepath, locations:HashMap::new() })
    }

    //TODO: API interface ambiguous
    pub fn set_uniform4f(&mut self, name: &CStr , value: Vec4 ) -> Result<bool,String>
    {
        let location = self.get_uniform_location(name);

        if location == -1
        {
            Err(String::from("an error occured"))
        }
        else
        {
            unsafe
            {
                gl::Uniform4f( location , value.x, value.y, value.z, value.w);
            }
            Ok(true)
        }
    }

    pub fn set_uniform1i(&mut self , name: &CStr , value: i32) -> Result<bool,String>
    {
        let location = self.get_uniform_location(name);

        if location == -1
        {
            Err(String::from("an error occured"))
        }
        else
        {
            unsafe
            {
                gl::Uniform1i( location , value );
            }
            Ok(true)
        }
    }

    pub fn set_uniform_matrix4fv(&mut self , name: &CStr, value : &Mat4 )  -> Result<bool,String>
    {
        let location = self.get_uniform_location(name);

        if location == -1
        {
            Err(String::from("an error occured"))
        }
        else
        {
            unsafe
            {
                gl::UniformMatrix4fv( location , 1 , gl::FALSE , value.as_ref().as_ptr().cast() );
            }
            Ok(true)
        }
    }

    pub fn bind(&self)
    {
        unsafe
        {
            gl::UseProgram(self.renderer_id);
        }
    }

    pub fn unbind()
    {
        unsafe
        {
            gl::UseProgram(0);
        }
    }

    pub fn delete(&self)
    {
        unsafe
        {
            gl::DeleteProgram(self.renderer_id);
        }
    }

    fn get_uniform_location(&mut self , name: &CStr) -> i32
    {
        // check if the location is cached
        match self.locations.get(name)
        {
            Some(&location) => location,
            None => {
                let location;
                unsafe
                {
                    location = gl::GetUniformLocation(self.renderer_id , name.as_ptr().cast() )
                }
                self.locations.insert(CString::from(name), location);
                location
            }
        }
    }

}