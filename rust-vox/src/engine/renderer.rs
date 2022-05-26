use std::{ffi::{c_void, CStr}, mem::{size_of_val, size_of}, f32::consts::PI};
use glam::{Mat4, Vec3, Vec2};
use sdl2::VideoSubsystem;

use crate::engine::{mesh::{Mesh}, voxel::{Voxel, VoxelType}, chunk::Chunk};

use super::camera::Camera;

pub struct Renderer
{
    program: u32,
    vao: u32,
    ebo: u32,

    texture1: u32,
    texture2: u32,

    chunk: Chunk,
    // mode: u32, // filled or lines 
}

impl Renderer
{
    pub fn new(video_subsystem: &VideoSubsystem) -> Renderer
    {
        // Setup

        // load up every opengl function, is this good ?
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        unsafe
        {
            gl::Enable(gl::DEBUG_OUTPUT);
            // TODO: what is extern "system" fn ? 
            gl::DebugMessageCallback( Some(error_callback) , 0 as *const c_void);
        }

        let mut program = 0;
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut texture1 = 0;
        let mut texture2 = 0;
        let mut chunk = Chunk::new();

        unsafe
        {
            // testing a single voxel
            // generating the mesh
            let mesh = chunk.generate_mesh();        

            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, mesh.size_bytes() as isize , mesh.vertices.as_ptr().cast() , gl::STATIC_DRAW);
            // specify the data format and buffer storage information for attribute index 0
            // this is specified for the currently bound VBO
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * size_of::<f32>()).try_into().unwrap()  , 0 as *const c_void );
            // vertex attributes are disabled by default 
            gl::EnableVertexAttribArray(0);

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (mesh.indices.len() * 4) as isize, mesh.indices.as_ptr().cast() , gl::STATIC_DRAW );

            // vertex attribute for the texture at location = 1
            gl::VertexAttribPointer(1, 2 , gl::FLOAT , gl::FALSE , (8 * size_of::<f32>()).try_into().unwrap() , (6 * size_of::<f32>()) as *const c_void  );
            gl::EnableVertexAttribArray(1);

            // load texture
            let img = image::open("rust-vox/textures/quakeIII-arena-logo.jpeg").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut texture1);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _ );
            gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::LINEAR as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGB as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);


            let img = image::open("rust-vox/textures/crate.jpeg").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut texture2);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _ );
            gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::LINEAR as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap(), height.try_into().unwrap(),
                0 , gl::RGB as _, gl::UNSIGNED_BYTE, data.as_ptr().cast());

            gl::GenerateMipmap(gl::TEXTURE_2D);

            // ------------------------------------------ END 

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

            // create program
            // first create the vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

            // vertex shader source
            //TODO: move this to a file and load it 
            const VERT_SHADER: &str = r##"
            #version 330 core
            layout (location = 0) in vec3 pos;
            layout (location = 1) in vec2 in_tex_coord;

            uniform mat4 transform;

            out vec3 color;
            out vec2 tex_coord;
            
            void main()
            {
                gl_Position = transform * vec4(pos.x, pos.y, pos.z, 1.0);
                tex_coord = in_tex_coord;
            }
            "##;

            // copy shader source into the specified shader object
            gl::ShaderSource(vertex_shader, 1, &(VERT_SHADER.as_bytes().as_ptr().cast()), &(VERT_SHADER.len().try_into().unwrap()));
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

            // fragment shader source

            const FRAG_SHADER: &str = r##"
            #version 330 core
            
            out vec4 color;
            in vec2 tex_coord;

            uniform vec4 our_color;
            uniform sampler2D test_texture1;
            uniform sampler2D test_texture2;

            void main()
            {
                color = texture(test_texture1, tex_coord) * texture( test_texture2, tex_coord) * our_color ;
            }
            "##;

            // copy shader source into the specified shader object
            gl::ShaderSource(fragment_shader, 1, &(FRAG_SHADER.as_bytes().as_ptr().cast()), &(FRAG_SHADER.len().try_into().unwrap()));
            gl::CompileShader(fragment_shader);

            //TODO: refactor duplicate code 
            // get compilation status on the *hopefully* compiled fragment shader
            let mut success = 0;
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

            //FIXME: can't get message from shader info
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

            gl::Enable(gl::DEPTH_TEST);
            // gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);
        }

        Renderer { program , vao , texture1 , texture2, ebo , chunk }

    }

    pub fn draw_world(&self, camera: &Camera )
    {
        unsafe
        {
            gl::ClearColor(0.2,0.2,0.2,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            // set program as current 
            gl::UseProgram(self.program); 

            let location = gl::GetUniformLocation( self.program , b"our_color\0".as_ptr() as _ );
            gl::Uniform4f(location, 1.0 , 1.0 , 1.0 , 1.0);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.texture2);

            let tex_location1 = gl::GetUniformLocation(self.program, b"test_texture1\0".as_ptr() as _ );
            gl::Uniform1i(tex_location1 , 0); // texture unit 0

            let tex_location2 = gl::GetUniformLocation( self.program , b"test_texture2\0".as_ptr() as _ );
            gl::Uniform1i( tex_location2 , 1); // texture unit 1 

            let transform_location = gl::GetUniformLocation( self.program , b"transform\0".as_ptr() as _ );

            let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 100.0);
            // camera matrix 
            let trans =  projection * camera.get_look_at();
            gl::UniformMatrix4fv( transform_location , 1 , gl::FALSE , trans.as_ref().as_ptr().cast() );

            gl::BindVertexArray(self.vao); // bind vao
            gl::DrawElements(gl::TRIANGLES, self.chunk.mesh.num_triangles() as _  , gl::UNSIGNED_INT, 0 as _ );
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }   
    }

    //FIXME: problematic interface
    pub fn set_mode(&mut self, mode: u32)
    {
        unsafe
        {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode );
        }
    }


}



// error callback function for opengl
extern "system" fn error_callback ( _source : u32 , error_type : u32 , _id : u32 , _severity : u32 , _len : i32 , message: *const i8 , _user_param : *mut c_void )
{
    unsafe
    {
        if error_type == gl::DEBUG_TYPE_ERROR
        {
            let x = CStr::from_ptr(message).to_string_lossy().to_string();
            println!("ERROR CALLBACK: {}" , x);
        }
    }
}