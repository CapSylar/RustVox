use std::{fs, io::Error, ffi::{CStr, CString}, collections::HashMap, ptr::null_mut, hash::Hash};

use glam::{Vec4, Mat4, Vec3};

pub struct Shader
{
    // state
    renderer_id: u32,

    // debugging
    _vertex_filepath : String,
    _fragment_filepath : String,

    // uniform locations
    locations: HashMap<String,i32>,
    strings: HashMap<String,CString>
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

            Shader::compile_shader(vertex_shader).expect("Error compiling Vertex Shader: ");

            // same for the fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            // copy shader source into the specified shader object
            gl::ShaderSource(fragment_shader, 1, &(fragment_src.as_bytes().as_ptr().cast()), &(fragment_src.len().try_into().unwrap()));
            
            Shader::compile_shader(fragment_shader).expect("Error compiling Fragment Shader: ");

            // attach shader and link program
            program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);

            Shader::link_program(program).expect("Error linking Program: ");
        }

        Ok(Self{renderer_id:program , _vertex_filepath: vertex_filepath , _fragment_filepath: fragment_filepath,
            locations:HashMap::new(),strings:HashMap::new() })
    }

    /// Cleaner Wrapper Around OpenGL's CompileShader(gluint) function
    pub fn compile_shader(shader_id: u32) -> Result<(),String>
    {
        unsafe
        {
            gl::CompileShader(shader_id);
            // get compilation status
            let mut success = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

            if success == 0
            {
                let mut log_length = 0_i32;
                gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_length );

                let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
                log_text.extend([b' '].iter().cycle().take(log_length as usize)); // fill Vec with log_length empty spaces

                // get shader log text
                gl::GetShaderInfoLog(shader_id, log_length , std::ptr::null_mut() , log_text.as_mut_ptr().cast());

                Err(String::from_utf8_lossy(&log_text).to_string())
            }
            else
            {
                Ok(())
            }
        }
    }

    pub fn link_program (program_id: u32) -> Result<(),String>
    {
        unsafe
        {
            gl::LinkProgram(program_id);

            let mut success = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

            if success == 0
            {
                let mut log_length = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_length );

                let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
                log_text.extend([b' '].iter().cycle().take(log_length as usize)); // fill Vec with log_length empty spaces

                // get shader log text
                gl::GetProgramInfoLog(program_id, log_length , std::ptr::null_mut() , log_text.as_mut_ptr().cast());

                Err(String::from_utf8_lossy(&log_text).to_string())
            }
            else
            {
                Ok(())
            }
        }
    }

    //TODO: API interface ambiguous
    pub fn set_uniform4f(&mut self, name: &str , value: Vec4 ) -> Result<bool,String>
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

    pub fn set_uniform1i(&mut self , name: &str , value: i32) -> Result<bool,String>
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

    pub fn set_uniform_matrix4fv(&mut self , name: &str, value : &Mat4 )  -> Result<bool,String>
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

    pub fn set_uniform3fv(&mut self, name: &str, value: &Vec3) -> Result<bool,String>
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
                gl::Uniform3fv( location , 1 , value.as_ref().as_ptr().cast() );
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

    fn get_uniform_location(&mut self , name: &str) -> i32
    {
        // check if the location is cached
        match self.locations.get(name)
        {
            Some(&location) => location,
            None => {

                // get the cstring
                let cstring = match self.strings.get(name)
                {
                    Some(str) => str,
                    None => { // convert to cstring and store
                        let cstring = CString::new(name).unwrap();
                        self.strings.insert(name.to_string(), cstring);
                        self.strings.get(name).unwrap() // i hate this
                    }
                };

                let location;
                unsafe
                {
                    location = gl::GetUniformLocation(self.renderer_id , cstring.as_ptr().cast() )
                }
                self.locations.insert( name.to_string(), location);
                location
            }
        }
    }

}