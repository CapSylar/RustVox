use std::{ffi::{c_void, CStr}, f32::consts::PI, fs};
use glam::{Mat4, };
use sdl2::VideoSubsystem;

pub mod vertex_buffer;
pub mod index_buffer;
pub mod vertex_array;

use crate::engine:: {chunk::Chunk};
use self::{vertex_buffer::VertexBuffer, index_buffer::IndexBuffer, vertex_array::{VertexArray, VertexBufferLayout}};

use super::camera::Camera;

pub struct Renderer
{
    program: u32,
    vao: VertexArray,
    texture1: u32,
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

        unsafe
        {
            let mut texture1 = 0;
            let mut chunk = Chunk::new();

            // testing a single voxel
            // generating the mesh
            let mesh = chunk.generate_mesh();        

            let vertex_buffer = VertexBuffer::new(mesh.size_bytes(), &mesh.vertices );
            let mut vao = VertexArray::new();

            // create a vertex buffer layout
            let mut layout = VertexBufferLayout::new();
            //TODO: refactor, mesh should take care of this
            layout.push_f32(3); // vertex(x,y,z)
            layout.push_f32(3); // normal(x,y,z)
            layout.push_f32(2); // uv coordinates(u,v)

            vao.add_buffer(&vertex_buffer, &mut layout);

            let index_buffer = IndexBuffer::new(&mesh.indices);
        
            // load texture atlas
            let img = image::open("rust-vox/textures/atlas.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut texture1);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as _ );
            gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::NEAREST as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGB as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // ------------------------------------------ END 

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

            // load the program

            let vertex_src = fs::read_to_string("rust-vox/shaders/default_vertex.glsl").expect("could not read file");
            let fragment_src = fs::read_to_string("rust-vox/shaders/default_fragment.glsl").expect("could not read file");

            let program = Renderer::create_program(&vertex_src, &fragment_src); // set program 

            gl::Enable(gl::DEPTH_TEST);
            // gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            Renderer { program , vao , texture1 , chunk }
        }


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
            let tex_location1 = gl::GetUniformLocation(self.program, b"test_texture1\0".as_ptr() as _ );
            gl::Uniform1i(tex_location1 , 0); // texture unit 0

            let transform_location = gl::GetUniformLocation( self.program , b"transform\0".as_ptr() as _ );

            let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 100.0);
            // camera matrix 
            let trans =  projection * camera.get_look_at();
            gl::UniformMatrix4fv( transform_location , 1 , gl::FALSE , trans.as_ref().as_ptr().cast() );

            self.vao.bind();
            gl::DrawElements(gl::TRIANGLES, self.chunk.mesh.num_triangles() as _  , gl::UNSIGNED_INT, 0 as _ );
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }   
    }


    /// Compile + Link the vertex and fragment shaders => returns OpenGL ID
    pub fn create_program( vertex_src: &str , fragment_src : &str) -> u32
    {
        //TODO: refactor error checking code
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
            let program = gl::CreateProgram();
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

            program
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